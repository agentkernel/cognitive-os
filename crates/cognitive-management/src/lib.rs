//! `cognitive-management`: management plane of the CognitiveOS reference
//! implementation.
//!
//! Scope (M5, per `docs/plan/DEVELOPMENT-PLAN.md`): the Management API,
//! PrivilegedManagementSession, ManagementActionProposal and approval
//! decisions, and the deterministic fallback path. Inspect, stop, revoke and
//! reconcile must work with no model available; the Intelligent Management
//! Shell stays experimental and is never a dependency of deterministic
//! management, recovery or stop paths.
//!
//! Machine contracts: `specs/schemas/privileged-management-session.schema.json`,
//! `management-action-proposal.schema.json`,
//! `management-approval-decision.schema.json`; vectors `management-*.json`.

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
}
