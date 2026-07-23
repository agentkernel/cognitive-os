//! `cognitive-management`: management plane of the CognitiveOS reference
//! implementation.
//!
//! M5 batch 1 (per `docs/plan/DEVELOPMENT-PLAN.md` and
//! `docs/prompts/milestone-m5.md`): the deterministic fallback path —
//! PrivilegedManagementSession gate + the four no-model verbs inspect /
//! stop / revoke / reconcile (REQ-MGMT-FALLBACK-001,
//! REQ-MGMT-SESSION-002/003, REQ-MGMT-GATE-001,
//! REQ-MGMT-SESSION-LIFECYCLE-001). The Management API surface,
//! ManagementActionProposal handling and the R1 structured-confirmation
//! flow (generated bindings `management_approval_request` /
//! `management_approval_decision`) land in the next batch.
//!
//! Hard rule: the Intelligent Management Shell stays experimental and is
//! never a dependency of deterministic management, recovery or stop paths.
//! The only model seam is [`ModelProvider`], which no deterministic verb
//! reads (proven by a zero-call probe in the behavior tests).
//!
//! Machine contracts: `specs/schemas/privileged-management-session.schema.json`,
//! `management-action-proposal.schema.json`,
//! `management-approval-request.schema.json`,
//! `management-approval-decision.schema.json`; vectors `management-*.json`
//! (registered not-run; behavioral execution is Lane-CFR's M5 batch).

pub mod approval;
pub mod audit;
pub mod error;
pub mod governance;
pub mod model;
pub mod plane;
pub mod session;
pub use approval::{ApprovalGate, ApprovalPresentation, ManagementActionProposal};
pub use audit::{
    AuditCommitReceipt, AuditPortFailure, AuditedInspectError, FileManagementAuditLog,
    ManagementAuditPort, PrivilegedReadDecision, PrivilegedReadOutcome, ResultReleaseGate,
};

pub use error::{ManagementDenial, ManagementError, RegisteredParts, category_str};
pub use governance::GovernanceLedger;
pub use model::ModelProvider;

/// Executor-port types that appear in this crate's public API
/// ([`ManagementPlane::reconcile`] takes `&dyn EffectExecutor`),
/// re-exported so thin clients (admin-cli) need no direct kernel
/// dependency.
pub mod executor_port {
    pub use cognitive_kernel::executor::{
        DispatchOutcome, EffectExecutor, ExecutorCall, ExecutorCapabilities, ExecutorQueryResult,
    };
    pub use cognitive_kernel::ports::PortFailure;
}
pub use plane::{
    FallbackVerb, InspectReport, InspectRequest, ManagementPlane, ReconcileReport,
    ReconciledEffect, RevokeReport, StopReport, StopRequest,
};
pub use session::{
    ManagementAction, ManagementSessionArchive, PrivilegedManagementSession, RiskClass,
    SessionScope, SessionState,
};

/// Deterministic management verbs that must work without any model.
pub const DETERMINISTIC_FALLBACK_VERBS: [&str; 4] = ["inspect", "stop", "revoke", "reconcile"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_verbs_are_deterministic_and_modelless() {
        assert_eq!(DETERMINISTIC_FALLBACK_VERBS.len(), 4);
        assert!(cognitive_kernel::KERNEL_PORTS.contains(&"event-log"));
    }

    #[test]
    fn verb_actions_match_the_fallback_vector_operations() {
        // `management-deterministic-fallback.json` requested_operations.
        assert_eq!(FallbackVerb::Inspect.action_name(), "status.inspect");
        assert_eq!(FallbackVerb::Stop.action_name(), "execution.stop");
        assert_eq!(FallbackVerb::Revoke.action_name(), "capability.revoke");
        assert_eq!(FallbackVerb::Reconcile.action_name(), "effect.reconcile");
    }
}
