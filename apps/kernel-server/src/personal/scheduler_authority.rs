//! Daemon-only durable scheduler authority reads (P2-T03).
//!
//! This module deliberately performs no lease acquisition or worker dispatch.
//! It reloads the immutable TaskContract and the identities it binds, deriving
//! ceiling inputs solely from authority-store facts.

#![allow(dead_code)] // Activated only after the fenced quiescence protocol exists.

use cognitive_contracts::generated::task_contract::TaskContract;
use cognitive_domain::{BudgetId, LifecycleDomain, ObjectId};
use cognitive_kernel::ports::{AuthorityStore, HarnessStore, IntentChainStore, ProtocolStore};
use cognitive_runtime::SchedulerCeilingFacts;
use serde::Deserialize;
use thiserror::Error;

const TASK_CONTRACT_EXECUTION_SCHEMA_VERSION: &str = "cognitiveos.task-contract/0.2";

#[derive(Deserialize)]
struct TaskContractVersionEnvelope {
    header: TaskContractVersionHeader,
}

#[derive(Deserialize)]
struct TaskContractVersionHeader {
    schema_version: String,
}

/// Exact identities fixed by an immutable task contract epoch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SchedulerAuthorityBinding {
    pub task_ref: String,
    pub action_fingerprint: String,
}

/// Fail-closed authority-read failures before scheduler lease acquisition.
#[derive(Debug, Error)]
pub(crate) enum SchedulerAuthorityError {
    #[error("scheduler task reference must not be empty")]
    EmptyTaskReference,
    #[error("scheduler action fingerprint must not be empty")]
    EmptyActionFingerprint,
    #[error("scheduler authority store failed: {0}")]
    Store(String),
    #[error("scheduler task has no current contract: {0}")]
    MissingContract(String),
    #[error("scheduler contract is not execution-bound: {0}")]
    LegacyContract(String),
    #[error("scheduler contract is malformed: {0}")]
    MalformedContract(String),
    #[error("scheduler bound loop is unavailable or not dispatchable: {0}")]
    LoopUnavailable(String),
    #[error("scheduler bound budget is unavailable or inconsistent: {0}")]
    BudgetUnavailable(String),
}

/// Reload durable facts that must precede a scheduler dispatch decision.
pub(crate) fn load_scheduler_ceiling_facts<S>(
    store: &S,
    binding: &SchedulerAuthorityBinding,
) -> Result<SchedulerCeilingFacts, SchedulerAuthorityError>
where
    S: AuthorityStore + HarnessStore + IntentChainStore + ProtocolStore,
{
    if binding.task_ref.is_empty() {
        return Err(SchedulerAuthorityError::EmptyTaskReference);
    }
    if binding.action_fingerprint.is_empty() {
        return Err(SchedulerAuthorityError::EmptyActionFingerprint);
    }

    let contract_epoch = store
        .current_contract_epoch(&binding.task_ref)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    if contract_epoch == 0 {
        return Err(SchedulerAuthorityError::MissingContract(
            binding.task_ref.clone(),
        ));
    }
    let contract_row = store
        .load_task_contract(&binding.task_ref, contract_epoch)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| SchedulerAuthorityError::MissingContract(binding.task_ref.clone()))?;
    let version_envelope: TaskContractVersionEnvelope =
        serde_json::from_str(&contract_row.canonical_json)
            .map_err(|error| SchedulerAuthorityError::MalformedContract(error.to_string()))?;
    if version_envelope.header.schema_version != TASK_CONTRACT_EXECUTION_SCHEMA_VERSION {
        return Err(SchedulerAuthorityError::LegacyContract(
            version_envelope.header.schema_version,
        ));
    }
    let contract: TaskContract = serde_json::from_str(&contract_row.canonical_json)
        .map_err(|error| SchedulerAuthorityError::MalformedContract(error.to_string()))?;

    let deadline = contract.deadline.ok_or_else(|| {
        SchedulerAuthorityError::MalformedContract("v0.2 contract has no deadline".to_owned())
    })?;
    let loop_object_id = ObjectId::parse(
        &contract
            .loop_object_id
            .ok_or_else(|| {
                SchedulerAuthorityError::MalformedContract(
                    "v0.2 contract has no loop object identity".to_owned(),
                )
            })?
            .0,
    )
    .map_err(|error| SchedulerAuthorityError::MalformedContract(error.to_string()))?;
    let budget_id = BudgetId::parse(
        &contract
            .budget_id
            .ok_or_else(|| {
                SchedulerAuthorityError::MalformedContract(
                    "v0.2 contract has no budget identity".to_owned(),
                )
            })?
            .0,
    )
    .map_err(|error| SchedulerAuthorityError::MalformedContract(error.to_string()))?;
    let loop_object = store
        .load_object(LifecycleDomain::Loop, &loop_object_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::LoopUnavailable(loop_object_id.as_str().to_owned())
        })?;
    if !matches!(loop_object.state.as_str(), "START" | "CONTINUE") {
        return Err(SchedulerAuthorityError::LoopUnavailable(format!(
            "{} is {}",
            loop_object_id.as_str(),
            loop_object.state.as_str()
        )));
    }

    let progress_facts = store
        .list_progress_facts(&loop_object_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    let retry_count = progress_facts
        .iter()
        .filter(|fact| {
            fact.action_fingerprint == binding.action_fingerprint && fact.status != "advanced"
        })
        .count() as i64;
    let completed_steps = progress_facts.len() as i64;
    let stored_budget = store
        .load_budget(&budget_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| SchedulerAuthorityError::BudgetUnavailable(budget_id.as_str().to_owned()))?;
    let configured_cost = contract.budget.money_microunits;
    let (spent_cost_microunits, cost_ceiling_microunits) = match configured_cost {
        Some(cost_ceiling) => {
            let remaining_cost = stored_budget
                .state
                .remaining()
                .get("money_microunits")
                .copied()
                .ok_or_else(|| {
                    SchedulerAuthorityError::BudgetUnavailable(budget_id.as_str().to_owned())
                })?;
            if remaining_cost > cost_ceiling {
                return Err(SchedulerAuthorityError::BudgetUnavailable(
                    budget_id.as_str().to_owned(),
                ));
            }
            (cost_ceiling - remaining_cost, cost_ceiling)
        }
        None => (0, i64::MAX),
    };

    Ok(SchedulerCeilingFacts {
        deadline: Some(deadline),
        retry_count,
        retry_ceiling: contract.max_retries,
        completed_steps,
        step_ceiling: contract.max_iterations,
        spent_cost_microunits,
        cost_ceiling_microunits,
    })
}
