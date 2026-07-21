//! `cognitive-runtime`: execution layer of the CognitiveOS reference
//! implementation.
//!
//! Scope (M4-M6, per `docs/plan/DEVELOPMENT-PLAN.md`): the Operation
//! executor (OperationDescriptor is never an AuthorizationCapability),
//! sandbox and adapter ports for C0/C1 agent integration, and the bounded
//! Harness Loop with progress/stagnation judgment. Dispatch goes through the
//! kernel's Effect protocol; this crate never commits authority state
//! directly.

pub mod adapters;
pub mod channel_binding;
pub mod event_envelope;
pub mod harness_loop;
pub mod installer;
pub mod intent_flow;
pub mod oob;
pub mod perf;
pub mod readiness;
pub mod recovery_flow;
pub mod sandbox;
pub mod shell;

pub use adapters::{
    CheckpointAdapter, CompatibilityProfile, CompletionAdapter, FeatureStatus, IdentityAdapter,
    MemoryAdapter, ToolAdapter, compatibility_matrix, on_adapter_failure,
};
pub use channel_binding::{
    AuthorityChannel, ChannelBindingDecision, ChannelBindingRequest, admit_channel_binding,
    is_privileged_management_action, request_from_vector_input,
};
pub use event_envelope::{EventEnvelopeError, assemble_event};
pub use harness_loop::{BoundedHarness, HarnessDecision, StagnationPolicy, decide_stagnation};
pub use installer::{
    AcceptingSignaturePort, InstallCrashPoint, InstallPhase, InstallationLedger, InstallerError,
    PackageInstallRequest, RejectingSignaturePort, SignatureProvenancePort, install_package,
    reject_package, verify_package,
};
pub use intent_flow::{admit_and_mint_contract, correct_and_supersede};
pub use oob::{OobCandidate, OobReconciler, ProjectionObject};
pub use perf::{GovernanceOverheadSample, StageLatencyMs};
pub use readiness::{R0ThinPath, ReadinessEvaluator, ReadinessFacts, ReadinessGrade};
pub use recovery_flow::{
    ObligationDecision, RecoveryContinuationPlan, plan_recovery_continuations,
    pre_crash_binding_is_stale,
};
pub use sandbox::{
    ChannelClaim, PlatformChannelRow, SandboxChannel, SandboxGate, SandboxPlatform, SandboxPolicy,
    refuse_cross_platform_merge,
};
pub use shell::{ShellError, ShellPhase, ShellService};

/// Runtime role marker (M6: harness/shell + install/sandbox/adapter/readiness/PERF).
pub const RUNTIME_ROLE: &str =
    "operation-executor+harness-loop+shell+install+sandbox+adapter+readiness+perf";

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use cognitive_domain::capability::{EffectiveRights, LeaseWindow};
    use cognitive_domain::{ObjectId, WallTimestamp};
    use cognitive_kernel::AuthorizationGrant;
    use cognitive_kernel::effects::GovernanceCurrency;
    use cognitive_kernel::recovery::{ContextRebinding, ReauthorizationObligation, RecoveryReport};
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn depends_on_kernel_layer() {
        assert!(cognitive_kernel::KERNEL_PORTS.contains(&"outbox"));
    }

    #[test]
    fn shell_detach_does_not_cancel() {
        let mut shell = crate::ShellService::new();
        shell
            .proposal("p1", serde_json::json!({"action":"demo"}))
            .unwrap();
        shell.preview("p1", "sha256:aa").unwrap();
        shell.submit("p1", "sha256:aa", "task://t1").unwrap();
        shell.attach("task://t1").unwrap();
        let out = shell.detach("task://t1").unwrap();
        assert_eq!(out["cancelled"], false);
        assert_eq!(shell.phase("task://t1"), Some(crate::ShellPhase::Detached));
    }

    #[test]
    fn shell_cancel_pending_and_too_late() {
        let mut shell = crate::ShellService::new();
        shell
            .proposal("p2", serde_json::json!({"action":"demo"}))
            .unwrap();
        shell.preview("p2", "sha256:bb").unwrap();
        shell.submit("p2", "sha256:bb", "task://t2").unwrap();
        let pending = shell
            .cancel(
                "task://t2",
                ObjectId::parse("00000000-0000-7000-8000-0000000000c1").unwrap(),
                false,
            )
            .unwrap();
        assert_eq!(pending["status"], "CANCEL_PENDING");
        let late = shell
            .cancel(
                "task://t2",
                ObjectId::parse("00000000-0000-7000-8000-0000000000c2").unwrap(),
                true,
            )
            .unwrap_err();
        assert_eq!(late.code, "CANCEL_TOO_LATE");
    }

    #[test]
    fn stagnation_policy_stops_without_spinning() {
        use cognitive_kernel::harness::{ProgressStatus, StagnationFacts};
        let facts = StagnationFacts {
            consecutive_without_progress: 3,
            last_advanced_iteration: Some(1),
            recorded_iterations: 4,
        };
        let decision = crate::decide_stagnation(
            &facts,
            ProgressStatus::NoProgress,
            4,
            3,
            crate::StagnationPolicy::Stop,
        );
        assert_eq!(
            decision,
            crate::HarnessDecision::StoppedForStagnation {
                consecutive_without_progress: 3
            }
        );
        let advanced = crate::decide_stagnation(
            &facts,
            ProgressStatus::Advanced,
            5,
            3,
            crate::StagnationPolicy::Escalate,
        );
        assert_eq!(advanced, crate::HarnessDecision::Advanced { iteration: 5 });
    }

    #[test]
    fn recovery_step6_rejects_stale_grant_and_step7_marks_precrash_binding_stale() {
        let obligation = ReauthorizationObligation {
            effect_object_id: ObjectId::parse("00000000-0000-7000-8000-0000000000e1").unwrap(),
            idempotency_key: "idem://e1".to_owned(),
            grant_epoch: 41,
            capability_set_version: 7,
        };
        let report = RecoveryReport {
            new_epoch: 42,
            fenced_epoch: 41,
            replayed_events: 0,
            projection_digest: "sha256:dead".to_owned(),
            reconciled: vec![],
            reauthorization_obligations: vec![obligation],
            context_rebinding: ContextRebinding {
                fenced_epoch: 41,
                new_epoch: 42,
            },
            step_order: vec![],
            resumable_loops: vec![],
        };
        let now = WallTimestamp::parse("2026-07-21T12:00:00Z").unwrap();
        let currency = GovernanceCurrency {
            revocation_epoch: 42,
            capability_set_version: 8,
        };
        let mut actions = BTreeSet::new();
        actions.insert("payments.refund".to_owned());
        let stale = AuthorizationGrant {
            effective: EffectiveRights {
                actions,
                resource: Some("scope://tenant-a/payments".to_owned()),
                purpose: Some("refund_processing".to_owned()),
                parameter_bounds: BTreeMap::new(),
                lease: Some(LeaseWindow {
                    not_before: WallTimestamp::parse("2026-07-21T11:00:00Z").unwrap(),
                    expires: WallTimestamp::parse("2026-07-21T14:00:00Z").unwrap(),
                }),
                oldest_issued_epoch: 41,
            },
            decided_at_epoch: 41,
            capability_set_version: 7,
        };
        let plan = crate::plan_recovery_continuations(&report, &[stale], &currency, &now, Some(41));
        assert!(!plan.obligations[0].satisfied);
        assert!(!plan.declared_binding_current);
        assert!(crate::pre_crash_binding_is_stale(
            &report.context_rebinding,
            41
        ));
    }

    /// REQ-SHELL-CHANNEL-001 / vector SHELL-CHANNEL-ISOLATION-003:
    /// task credential + system.configure → deny SHELL_CHANNEL_BINDING_MISMATCH,
    /// management_context_leaked=false.
    #[test]
    fn shell_channel_isolation_denies_task_credential_on_system_configure() {
        let request = crate::request_from_vector_input(true, "system.configure", false);
        let decision = crate::admit_channel_binding(&request);
        assert_eq!(decision.decision, "deny");
        assert_eq!(decision.error_code, Some("SHELL_CHANNEL_BINDING_MISMATCH"));
        assert_eq!(decision.error_category, Some("auth"));
        assert!(!decision.management_context_leaked);
    }

    #[test]
    fn shell_channel_isolation_allows_management_with_privileged_session() {
        let request = crate::ChannelBindingRequest {
            credential_channel: crate::AuthorityChannel::Management,
            requested_action: "system.configure".to_owned(),
            privileged_session: true,
        };
        let decision = crate::admit_channel_binding(&request);
        assert_eq!(decision.decision, "allow");
        assert!(decision.error_code.is_none());
        assert!(!decision.management_context_leaked);
    }

    #[test]
    fn shell_channel_isolation_rejects_management_cred_on_task_action() {
        let request = crate::ChannelBindingRequest {
            credential_channel: crate::AuthorityChannel::Management,
            requested_action: "task.preview".to_owned(),
            privileged_session: false,
        };
        let decision = crate::admit_channel_binding(&request);
        assert_eq!(decision.decision, "deny");
        assert_eq!(decision.error_code, Some("SHELL_CHANNEL_BINDING_MISMATCH"));
        assert!(!decision.management_context_leaked);
    }
}
