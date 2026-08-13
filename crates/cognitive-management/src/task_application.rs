//! P2-T01 TaskApplicationService: the L5 product entry point over the
//! L3/L4 intent-chain kernel.
//!
//! This module exposes the six product operations —
//! proposal / clarify / preview / admit / control / query — and composes
//! ONLY the deterministic kernel primitives from
//! `cognitive_kernel::intent_chain`. It never duplicates kernel logic:
//! raw-intent fixing, candidate persistence, admission and contract
//! minting all run in the kernel against the real store transaction.
//!
//! Hard rules preserved from the kernel contract:
//! - raw intent is durably fixed BEFORE any semantic interpretation;
//! - an admission binds the exact digest the authority reviewed;
//! - supersession mints a new epoch and fences old-epoch bindings;
//! - the admission preview is the default human authorization point
//!   (ADR-0026).
//!
//! No SQLite table is added here; all persistence is the existing store
//! surface behind the kernel ports.

use cognitive_contracts::canonical;
use cognitive_domain::{ObjectId, UriRef};
use cognitive_kernel::effects::{EffectError, WriterLease};
use cognitive_kernel::intent_chain::{
    AcceptanceCommand, GovernanceSeed, InterpretationCandidate, SupersedeCommand, SupersedeReport,
    TaskContractCommand, UserIntentCommand, admit_interpretation, mint_schedulable_task_contract,
    record_interpretation_candidate, record_user_intent, supersede_task_contract,
};
use cognitive_kernel::ports::{
    AuthorityStore, Clock, ContextStore, IdGenerator, IntentChainStore, InterpretationRow,
    ProtocolStore, StorePortError, TaskContractRow, UserIntentRecordRow,
};
use serde::Serialize;

/// Service surface of the task lifecycle (P2-T01).
pub trait TaskApplicationService {
    /// proposal: durably fix the raw user intent BEFORE interpretation.
    fn propose(
        &mut self,
        lease: &WriterLease,
        intent: &UserIntentCommand,
    ) -> Result<UserIntentRecordRow, TaskApplicationError>;

    /// clarify: persist an interpretation candidate against a fixed
    /// record; material ambiguities mark it `clarification_required`.
    fn clarify(
        &mut self,
        lease: &WriterLease,
        user_intent_record_id: &ObjectId,
        candidate: &InterpretationCandidate,
        governance: &GovernanceSeed,
        correlation_id: &UriRef,
    ) -> Result<InterpretationRow, TaskApplicationError>;

    /// preview: generate the digest-bound admission preview for one
    /// TaskContract composition. Nothing is persisted; the returned
    /// `preview_digest` MUST be passed back to `admit`.
    fn preview(
        &mut self,
        contract: &TaskContractCommand,
    ) -> Result<ContractPreview, TaskApplicationError>;

    /// admit: bind the authority acceptance to the exact digest it
    /// reviewed and mint the TaskContract (epoch CAS inside the store).
    fn admit(
        &mut self,
        lease: &WriterLease,
        preview_digest: &str,
        acceptance: &AcceptanceCommand,
        contract: &TaskContractCommand,
        expected_current_epoch: i64,
    ) -> Result<TaskContractRow, TaskApplicationError>;

    /// control: user-correction supersession to a new epoch (fences old
    /// bindings), or a cancel request through the same correction path.
    fn control(
        &mut self,
        lease: &WriterLease,
        supersede: &SupersedeCommand,
    ) -> Result<SupersedeReport, TaskApplicationError>;

    /// query: read-only projection of a fixed intent record.
    fn query_intent(
        &self,
        record_id: &ObjectId,
    ) -> Result<Option<UserIntentRecordRow>, TaskApplicationError>;
}

/// Deterministic digest-bound preview of a TaskContract composition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractPreview {
    /// Canonical digest over the composed TaskContract content.
    pub preview_digest: String,
    /// Task URI the contract governs (non-secret projection).
    pub task_ref: String,
    /// Contract objective (non-secret projection).
    pub objective: String,
    /// Number of declared conditions (non-secret projection).
    pub condition_count: usize,
    /// Tool-call budget frozen at admission (hard rail, ADR-0026).
    pub tool_calls_frozen: Option<i64>,
}

/// Task-application service error surface.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TaskApplicationError {
    /// Rejected by the kernel intent chain.
    #[error(transparent)]
    Kernel(#[from] EffectError),
    /// Preview digest did not match the digest-bound admission.
    #[error("preview digest mismatch: admission binds a different contract preview")]
    PreviewDigestMismatch,
    /// The TaskContract composition is not previewable (missing content).
    #[error("contract preview requires a task ref and at least one condition")]
    NonPreviewableContract,
    /// Canonical encoding of the preview failed (fail closed).
    #[error("contract preview canonicalization failed")]
    PreviewCanonicalization,
    /// The authority store failed (fail closed).
    #[error(transparent)]
    Store(#[from] StorePortError),
}

/// Concrete composition root over the kernel store/clock/id ports.
pub struct KernelTaskApplicationService<S, C, G> {
    store: S,
    clock: C,
    ids: G,
}

impl<S, C, G> KernelTaskApplicationService<S, C, G> {
    /// Build the service from the authoritative store and deterministic
    /// infrastructure ports.
    pub fn new(store: S, clock: C, ids: G) -> Self {
        Self { store, clock, ids }
    }

    /// Borrow the authoritative store for read-only projection checks.
    pub fn store(&self) -> &S {
        &self.store
    }
}

impl<S, C, G> TaskApplicationService for KernelTaskApplicationService<S, C, G>
where
    S: AuthorityStore + ProtocolStore + IntentChainStore + ContextStore,
    C: Clock,
    G: IdGenerator,
{
    fn propose(
        &mut self,
        lease: &WriterLease,
        intent: &UserIntentCommand,
    ) -> Result<UserIntentRecordRow, TaskApplicationError> {
        record_user_intent(&self.store, &self.clock, &self.ids, lease, intent)
            .map_err(TaskApplicationError::Kernel)
    }

    fn clarify(
        &mut self,
        lease: &WriterLease,
        user_intent_record_id: &ObjectId,
        candidate: &InterpretationCandidate,
        governance: &GovernanceSeed,
        correlation_id: &UriRef,
    ) -> Result<InterpretationRow, TaskApplicationError> {
        record_interpretation_candidate(
            &self.store,
            &self.clock,
            &self.ids,
            lease,
            user_intent_record_id,
            candidate,
            governance,
            correlation_id,
        )
        .map_err(TaskApplicationError::Kernel)
    }

    fn preview(
        &mut self,
        contract: &TaskContractCommand,
    ) -> Result<ContractPreview, TaskApplicationError> {
        if contract.task_ref.as_str().is_empty() || contract.conditions.is_empty() {
            return Err(TaskApplicationError::NonPreviewableContract);
        }
        let digest = contract_preview_digest(contract)?;
        Ok(ContractPreview {
            preview_digest: digest,
            task_ref: contract.task_ref.as_str().to_owned(),
            objective: contract.objective.clone(),
            condition_count: contract.conditions.len(),
            tool_calls_frozen: contract.budget.tool_calls,
        })
    }

    fn admit(
        &mut self,
        lease: &WriterLease,
        preview_digest: &str,
        acceptance: &AcceptanceCommand,
        contract: &TaskContractCommand,
        expected_current_epoch: i64,
    ) -> Result<TaskContractRow, TaskApplicationError> {
        let recomputed = contract_preview_digest(contract)?;
        if preview_digest != recomputed {
            return Err(TaskApplicationError::PreviewDigestMismatch);
        }
        let admitted =
            admit_interpretation(&self.store, acceptance).map_err(TaskApplicationError::Kernel)?;
        mint_schedulable_task_contract(
            &self.store,
            &self.clock,
            &self.ids,
            lease,
            &admitted,
            contract,
            expected_current_epoch,
        )
        .map_err(TaskApplicationError::Kernel)
    }

    fn control(
        &mut self,
        lease: &WriterLease,
        supersede: &SupersedeCommand,
    ) -> Result<SupersedeReport, TaskApplicationError> {
        supersede_task_contract(&self.store, &self.clock, &self.ids, lease, supersede)
            .map_err(TaskApplicationError::Kernel)
    }

    fn query_intent(
        &self,
        record_id: &ObjectId,
    ) -> Result<Option<UserIntentRecordRow>, TaskApplicationError> {
        self.store
            .load_user_intent(record_id)
            .map_err(TaskApplicationError::Store)
    }
}

/// Canonical digest over the TaskContract composition content.
fn contract_preview_digest(contract: &TaskContractCommand) -> Result<String, TaskApplicationError> {
    #[derive(Serialize)]
    struct PreviewBody<'a> {
        task_ref: &'a str,
        objective: &'a str,
        in_scope: &'a [String],
        out_of_scope: &'a [String],
        conditions: &'a [ConditionPreview<'a>],
        budget: BudgetPreview,
        max_iterations: i64,
        max_retries: i64,
        deadline: &'a str,
        loop_object_id: &'a str,
        budget_id: &'a str,
        allowed_tools: &'a [String],
    }

    #[derive(Serialize)]
    struct ConditionPreview<'a> {
        id: &'a str,
        kind: String,
        description: &'a str,
    }

    #[derive(Serialize)]
    struct BudgetPreview {
        tool_calls: Option<i64>,
        input_tokens: Option<i64>,
        output_tokens: Option<i64>,
        wall_time_ms: Option<i64>,
        money_microunits: Option<i64>,
    }

    let conditions: Vec<ConditionPreview> = contract
        .conditions
        .iter()
        .map(|c| ConditionPreview {
            id: c.id.as_str(),
            kind: format!("{:?}", c.kind),
            description: c.description.as_str(),
        })
        .collect();
    let body = PreviewBody {
        task_ref: contract.task_ref.as_str(),
        objective: contract.objective.as_str(),
        in_scope: &contract.in_scope,
        out_of_scope: &contract.out_of_scope,
        conditions: &conditions,
        budget: BudgetPreview {
            tool_calls: contract.budget.tool_calls,
            input_tokens: contract.budget.input_tokens,
            output_tokens: contract.budget.output_tokens,
            wall_time_ms: contract.budget.wall_time_ms,
            money_microunits: contract.budget.money_microunits,
        },
        max_iterations: contract.max_iterations,
        max_retries: contract.max_retries,
        deadline: contract.deadline.as_str(),
        loop_object_id: contract.loop_object_id.as_str(),
        budget_id: contract.budget_id.as_str(),
        allowed_tools: &contract.allowed_tools,
    };
    let value =
        serde_json::to_value(body).map_err(|_| TaskApplicationError::PreviewCanonicalization)?;
    let bytes = canonical::canonical_bytes_of_value(&value)
        .map_err(|_| TaskApplicationError::PreviewCanonicalization)?;
    canonical::digest(&bytes, "cognitiveos.personal.task-contract-preview")
        .map_err(|_| TaskApplicationError::PreviewCanonicalization)
}
