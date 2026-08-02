//! Daemon-only durable scheduler authority reads (P2-T03).
//!
//! This module deliberately performs no lease acquisition or worker dispatch.
//! It reloads the immutable TaskContract and the identities it binds, deriving
//! ceiling inputs solely from authority-store facts.

#![allow(dead_code)] // Activated only after the fenced quiescence protocol exists.

use cognitive_contracts::generated::task_contract::TaskContract;
use cognitive_domain::{BudgetId, LifecycleDomain, ObjectId, Version};
use cognitive_kernel::effects::WriterLease;
use cognitive_kernel::engine::CommittedTransition;
use cognitive_kernel::harness::LoopDriver;
use cognitive_kernel::ports::{
    AuthorityStore, Clock, HarnessStore, IdGenerator, IntentChainStore, ProtocolStore,
};
use cognitive_runtime::{
    SchedulerCeilingDispatch, SchedulerCeilingDispatchError, SchedulerCeilingFacts,
    SchedulerDispatch, SchedulerService, SchedulerServiceError,
};
use cognitive_store::scheduler::SchedulerRepository;
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

/// Durable authority inputs required to decide one scheduler admission.
///
/// These facts are reloaded from the current immutable TaskContract and the
/// authority store for every attempt. They are never taken from the worker or
/// a prior scheduler projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SchedulerAuthoritySnapshot {
    pub ceiling_facts: SchedulerCeilingFacts,
    pub loop_object_id: ObjectId,
    pub budget_id: BudgetId,
}

/// One daemon-owned scheduler admission result.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SchedulerDispatchAdmission {
    Stopped(CommittedTransition),
    Leased(SchedulerDispatch),
}

/// The only scheduler-facing result a daemon worker may accept from Effect
/// processing. The worker callback must derive either state from the durable
/// Effect protocol; an external receipt is not a closed Effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SchedulerEffectClosure {
    Closed,
    PendingReconciliation,
}

/// Daemon-owned outcome after scheduler admission reaches the Effect boundary.
///
/// This remains distinct from Task acceptance. A closed Effect only permits a
/// later fenced scheduler release; independent verification still decides
/// whether a Task may complete.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SchedulerWorkerAttempt {
    Stopped(CommittedTransition),
    EffectClosed(SchedulerDispatch),
    AwaitingReconciliation(SchedulerDispatch),
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
    #[error(transparent)]
    Scheduler(#[from] SchedulerServiceError),
    #[error(transparent)]
    CeilingStop(#[from] SchedulerCeilingDispatchError),
}

/// Parse only a current execution-bound contract before reading its bindings.
///
/// The explicit version check preserves old contract rows for audit while
/// preventing their deserialization from becoming a scheduler admission path.
fn parse_execution_bound_contract(
    canonical_json: &str,
) -> Result<TaskContract, SchedulerAuthorityError> {
    let version_envelope: TaskContractVersionEnvelope = serde_json::from_str(canonical_json)
        .map_err(|error| SchedulerAuthorityError::MalformedContract(error.to_string()))?;
    if version_envelope.header.schema_version != TASK_CONTRACT_EXECUTION_SCHEMA_VERSION {
        return Err(SchedulerAuthorityError::LegacyContract(
            version_envelope.header.schema_version,
        ));
    }

    serde_json::from_str(canonical_json)
        .map_err(|error| SchedulerAuthorityError::MalformedContract(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{
        SchedulerAuthorityError, SchedulerDispatchAdmission, SchedulerEffectClosure,
        SchedulerWorkerAttempt, complete_scheduler_admission, complete_scheduler_worker_attempt,
        parse_execution_bound_contract,
    };
    use cognitive_domain::{EventId, RecordId, Version, WallTimestamp};
    use cognitive_kernel::engine::CommittedTransition;
    use cognitive_runtime::{SchedulerCeilingDispatch, SchedulerDispatch};
    use std::cell::Cell;

    fn committed_ceiling_stop() -> CommittedTransition {
        CommittedTransition {
            record_id: RecordId::parse("00000000-0000-7000-8000-000000000001").unwrap(),
            event_id: EventId::parse("00000000-0000-7000-8000-000000000002").unwrap(),
            event_sequence: 1,
            after_version: Version::new(2).unwrap(),
            committed_at: WallTimestamp::parse("2026-08-02T00:00:00Z").unwrap(),
        }
    }

    #[test]
    fn legacy_contract_is_rejected_before_execution_binding_deserialization() {
        let legacy_contract = r#"{
            "header": {
                "schema_version": "cognitiveos.task-contract/0.1"
            }
        }"#;

        assert!(matches!(parse_execution_bound_contract(legacy_contract),
            Err(SchedulerAuthorityError::LegacyContract(version))
            if version == "cognitiveos.task-contract/0.1"));
    }

    #[test]
    fn execution_schema_without_required_bindings_is_rejected_as_malformed() {
        let incomplete_execution_contract = r#"{
            "header": {
                "schema_version": "cognitiveos.task-contract/0.2"
            }
        }"#;

        assert!(matches!(
            parse_execution_bound_contract(incomplete_execution_contract),
            Err(SchedulerAuthorityError::MalformedContract(_))
        ));
    }

    #[test]
    fn reached_ceiling_returns_durable_stop_without_attempting_a_scheduler_lease() {
        let lease_acquisition_attempted = Cell::new(false);

        let admission = complete_scheduler_admission(
            SchedulerCeilingDispatch::Stopped(committed_ceiling_stop()),
            || {
                lease_acquisition_attempted.set(true);
                unreachable!("a reached ceiling must not acquire a scheduler lease")
            },
        )
        .unwrap();

        assert!(matches!(admission, SchedulerDispatchAdmission::Stopped(_)));
        assert!(
            !lease_acquisition_attempted.get(),
            "a terminal ceiling STOP must precede every lease attempt"
        );
    }

    #[test]
    fn clear_ceiling_acquires_exactly_one_scheduler_lease() {
        let lease_acquisition_count = Cell::new(0);

        let admission = complete_scheduler_admission(SchedulerCeilingDispatch::Proceed, || {
            lease_acquisition_count.set(lease_acquisition_count.get() + 1);
            Ok(SchedulerDispatch {
                task_ref: "task://personal/admission-order".to_owned(),
                lease_owner: "scheduler-worker".to_owned(),
                lease_epoch: 3,
                lease_expires: "2026-08-02T00:01:00Z".to_owned(),
                attempt_count: 1,
            })
        })
        .unwrap();

        assert!(matches!(admission, SchedulerDispatchAdmission::Leased(_)));
        assert_eq!(lease_acquisition_count.get(), 1);
    }

    #[test]
    fn ceiling_stop_skips_the_effect_closure_callback() {
        let effect_closure_attempted = Cell::new(false);

        let attempt = complete_scheduler_worker_attempt(
            SchedulerDispatchAdmission::Stopped(committed_ceiling_stop()),
            |_| {
                effect_closure_attempted.set(true);
                unreachable!("a stopped scheduler attempt must not process an Effect")
            },
        )
        .unwrap();

        assert!(matches!(attempt, SchedulerWorkerAttempt::Stopped(_)));
        assert!(
            !effect_closure_attempted.get(),
            "a durable ceiling STOP must precede every Effect-closure callback"
        );
    }

    #[test]
    fn unresolved_effect_keeps_the_fenced_dispatch_for_reconciliation() {
        let dispatch = SchedulerDispatch {
            task_ref: "task://personal/effect-reconciliation".to_owned(),
            lease_owner: "scheduler-worker".to_owned(),
            lease_epoch: 7,
            lease_expires: "2026-08-02T00:01:00Z".to_owned(),
            attempt_count: 1,
        };

        let attempt = complete_scheduler_worker_attempt(
            SchedulerDispatchAdmission::Leased(dispatch.clone()),
            |received_dispatch| {
                assert_eq!(received_dispatch, dispatch);
                Ok(SchedulerEffectClosure::PendingReconciliation)
            },
        )
        .unwrap();

        assert_eq!(
            attempt,
            SchedulerWorkerAttempt::AwaitingReconciliation(dispatch),
            "an unresolved Effect must not be converted into a scheduler success"
        );
    }
}

/// Reload durable facts that must precede a scheduler dispatch decision.
pub(crate) fn load_scheduler_ceiling_facts<S>(
    store: &S,
    binding: &SchedulerAuthorityBinding,
) -> Result<SchedulerCeilingFacts, SchedulerAuthorityError>
where
    S: AuthorityStore + HarnessStore + IntentChainStore + ProtocolStore,
{
    Ok(load_scheduler_authority_snapshot(store, binding)?.ceiling_facts)
}

/// Reload the full durable input set required before scheduler admission.
pub(crate) fn load_scheduler_authority_snapshot<S>(
    store: &S,
    binding: &SchedulerAuthorityBinding,
) -> Result<SchedulerAuthoritySnapshot, SchedulerAuthorityError>
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
    let contract = parse_execution_bound_contract(&contract_row.canonical_json)?;

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

    Ok(SchedulerAuthoritySnapshot {
        ceiling_facts: SchedulerCeilingFacts {
            deadline: Some(deadline),
            retry_count,
            retry_ceiling: contract.max_retries,
            completed_steps,
            step_ceiling: contract.max_iterations,
            spent_cost_microunits,
            cost_ceiling_microunits,
        },
        loop_object_id,
        budget_id,
    })
}

/// Finish daemon admission after the runtime has evaluated the fresh ceiling
/// snapshot. A committed STOP is terminal for this attempt: the lease closure
/// remains uncalled, ensuring no scheduler worker is admitted after a ceiling.
fn complete_scheduler_admission(
    ceiling_dispatch: SchedulerCeilingDispatch,
    acquire_lease: impl FnOnce() -> Result<SchedulerDispatch, SchedulerServiceError>,
) -> Result<SchedulerDispatchAdmission, SchedulerAuthorityError> {
    match ceiling_dispatch {
        SchedulerCeilingDispatch::Stopped(transition) => {
            Ok(SchedulerDispatchAdmission::Stopped(transition))
        }
        SchedulerCeilingDispatch::Proceed => {
            Ok(SchedulerDispatchAdmission::Leased(acquire_lease()?))
        }
    }
}

/// Invoke the daemon-owned Effect-closure boundary only after a fenced lease.
///
/// A persisted ceiling STOP never reaches the callback. A callback error or a
/// pending reconciliation leaves the durable lease untouched so a stale or
/// uncertain Effect cannot be reported as a scheduler or Task success.
fn complete_scheduler_worker_attempt(
    admission: SchedulerDispatchAdmission,
    complete_effect: impl FnOnce(
        SchedulerDispatch,
    ) -> Result<SchedulerEffectClosure, SchedulerAuthorityError>,
) -> Result<SchedulerWorkerAttempt, SchedulerAuthorityError> {
    match admission {
        SchedulerDispatchAdmission::Stopped(transition) => {
            Ok(SchedulerWorkerAttempt::Stopped(transition))
        }
        SchedulerDispatchAdmission::Leased(dispatch) => match complete_effect(dispatch.clone())? {
            SchedulerEffectClosure::Closed => Ok(SchedulerWorkerAttempt::EffectClosed(dispatch)),
            SchedulerEffectClosure::PendingReconciliation => {
                Ok(SchedulerWorkerAttempt::AwaitingReconciliation(dispatch))
            }
        },
    }
}

/// Commit a reached ceiling STOP before a worker can obtain a scheduler lease.
///
/// This is the daemon composition boundary: it reloads the current authority
/// snapshot, delegates the fenced STOP commit to the kernel, and only calls
/// the scheduler repository when no hard ceiling was reached. It deliberately
/// stops before external worker or Effect dispatch.
#[allow(clippy::too_many_arguments)]
pub(crate) fn admit_scheduler_dispatch<S, C, G>(
    authority_store: &S,
    scheduler_repository: &mut SchedulerRepository,
    scheduler_service: &mut SchedulerService,
    driver: &LoopDriver<'_, S, C, G>,
    binding: &SchedulerAuthorityBinding,
    lease_epoch: i64,
    observed_wall_time: &str,
    expected_loop_version: Version,
    writer_lease: &WriterLease,
) -> Result<SchedulerDispatchAdmission, SchedulerAuthorityError>
where
    S: AuthorityStore + HarnessStore + IntentChainStore + ProtocolStore,
    C: Clock,
    G: IdGenerator,
{
    let snapshot = load_scheduler_authority_snapshot(authority_store, binding)?;
    let ceiling_dispatch = scheduler_service.stop_before_dispatch_when_ceiling_reached(
        &snapshot.ceiling_facts,
        observed_wall_time,
        driver,
        &snapshot.loop_object_id,
        expected_loop_version,
        &binding.task_ref,
        &snapshot.budget_id,
        writer_lease,
    )?;
    complete_scheduler_admission(ceiling_dispatch, || {
        scheduler_service.claim_eligible(
            scheduler_repository,
            &binding.task_ref,
            lease_epoch,
            observed_wall_time,
        )
    })
}
