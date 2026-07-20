//! `cognitive-kernel`: deterministic governance core of the CognitiveOS
//! reference implementation.
//!
//! M2 scope (per `docs/plan/DEVELOPMENT-PLAN.md`): the centralized
//! transition gate over the five registered lifecycle tables (CAS, guard
//! and evidence verification, registered-error rejection), deterministic
//! hard-budget metering, projection replay over the committed event
//! history, and the port traits adapters implement (`cognitive-store`).
//! M3/M4 add capabilities, the Context gate, Intent/Effect and recovery.
//!
//! Hard rule (architecture invariant): authorization, CAS, state
//! transitions, budgets, idempotency, fencing and final commits are
//! executed by deterministic code in this crate. LLMs, retrieval and
//! rankers only ever produce candidates or proposals upstream; nothing in
//! this crate calls a probabilistic component.
//!
//! REQ coverage: REQ-STATE-001/002/003, REQ-EVT-002/004 (port contract),
//! REQ-REC-003 (fail-closed mapping). Registered code mapping: [`error`].

pub mod authz;
pub mod budget;
pub mod context;
pub mod context_cache;
pub mod effects;
pub mod engine;
pub mod error;
pub mod executor;
pub mod intent_chain;
pub mod ports;
pub mod recovery;
pub mod replay;

pub use authz::{
    AccessDenial, AccessRequest, ActorChainFacts, AuthorizationGrant, AuthzSnapshot, DeniedAccess,
    DenyRule, MembershipFacts, ObjectGovernance, PrincipalFacts, authorize,
    capability_and_revocation_current, protected_read, revalidate_grant,
};
pub use budget::{BudgetCharge, BudgetError, BudgetExhausted, BudgetState};
pub use context::{
    ArrivalOrderRanker, CandidateObject, ContextBudget, ProposalRanker, RankerCandidate,
    RenderSpec, RequiredItem, ResolutionFailure, ResolutionRequest, ResolutionSession,
    ResolvedContextView, admit_control_mutation, effective_control_plane, resolve,
};
pub use context_cache::{
    CacheDecision, CachedView, ContextViewCache, DerivedCacheKind, GovernanceBinding,
    InvalidationReport,
};
pub use effects::{
    CommitSink, EffectClass, EffectError, EffectProtocol, GovernanceCurrency, IntentCommand,
    MintedIntent, OperationDescriptor, ProtocolDenial, RecoveryClosure, VerificationRecord,
    VerificationStatus, WriterLease, acquire_lease, admit_operation, parameters_digest,
    verification_still_current,
};
pub use engine::{
    AdmitCommand, AdmittedObject, BudgetChargeCommand, Causation, CommittedTransition, Reason,
    TablePin, TransitionCommand, TransitionEngine,
};
pub use error::{RegisteredError, RejectionKind, TransitionRejection};
pub use executor::{
    DispatchOutcome, EffectExecutor, ExecutorCall, ExecutorCapabilities, ExecutorQueryResult,
};
pub use intent_chain::{
    AcceptanceCommand, AdmittedInterpretation, AmbiguityFact, GovernanceSeed,
    InterpretationCandidate, PendingWork, PendingWorkDisposition, SupersedeCommand,
    SupersedeReport, TaskContractCommand, UserIntentCommand, admit_interpretation,
    derive_candidate_status, mint_task_contract, record_interpretation_candidate,
    record_user_intent, supersede_task_contract, verify_task_binding_current,
};
pub use ports::{
    AuthorityStore, CheckpointRow, Clock, HarnessStore, IdGenerator, IntentChainStore, IntentRow,
    InterpretationRow, PortFailure, ProgressFactRow, ProtocolStore, StorePortError, TaskBinding,
    TaskContractRow, UserIntentRecordRow,
};
pub use recovery::{
    EffectDisposition, RECOVERY_ORDER, RecoveryError, RecoveryReport, RecoverySequencer,
    RecoveryStep, run_recovery, validate_checkpoint,
};
pub use replay::{ReplayError, ReplayedProjection, replay_projection};

/// Port capability surface defined by this crate and implemented by
/// adapters. `event-log` and `outbox` are capabilities of the
/// [`ports::AuthorityStore`] trait (they share its atomic commit unit and
/// must never be split into independently committing stores); `clock` and
/// `id-generator` are standalone port traits.
pub const KERNEL_PORTS: [&str; 5] = [
    "authority-store",
    "event-log",
    "outbox",
    "clock",
    "id-generator",
];

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn kernel_sits_above_domain_only() {
        assert!(cognitive_domain::EXECUTION_LIFECYCLE_DOMAINS.contains(&"effect"));
        assert_eq!(KERNEL_PORTS.len(), 5);
        assert!(KERNEL_PORTS.contains(&"authority-store"));
    }

    #[test]
    fn table_pins_are_available_for_all_five_domains() {
        for domain in cognitive_domain::LifecycleDomain::ALL {
            let pin = TablePin::current(domain).unwrap();
            assert!(pin.digest.starts_with("sha256:"));
            assert!(!pin.version.is_empty());
        }
    }
}
