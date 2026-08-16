//! P2-T28/D01 frozen UJ1–UJ6 capability-truth register.
//!
//! Required rows name a public caller, mechanical oracle, cleanup, and evidence
//! schema. Web UI and Multi-Agent are scope-excluded and must not block this
//! register. This module does not execute journeys or claim Gate/EVAL results.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityTruthScope {
    Required,
    ScopeExcluded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityTruthRow {
    pub id: &'static str,
    pub family: &'static str,
    pub title: &'static str,
    pub scope: CapabilityTruthScope,
    pub public_caller: &'static str,
    pub mechanical_oracle: &'static str,
    pub cleanup: &'static str,
    pub evidence_schema: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityTruthError {
    MissingCaller,
    MissingOracle,
    MissingCleanup,
    MissingEvidenceSchema,
    DuplicateId,
    MissingFamily,
    ExcludedRowMarkedRequired,
}

const REQUIRED_FAMILIES: [&str; 6] = ["UJ1", "UJ2", "UJ3", "UJ4", "UJ5", "UJ6"];

/// Frozen UJ1–UJ6 register for BR-08. Callers are public CLI/HTTP surfaces
/// delivered by P2-T14..P2-T27; this freeze does not re-implement them.
pub const FROZEN_UJ_CAPABILITY_TRUTH: &[CapabilityTruthRow] = &[
    CapabilityTruthRow {
        id: "uj1.install-init-first-response",
        family: "UJ1",
        title: "install, init, first response",
        scope: CapabilityTruthScope::Required,
        public_caller: "cognitive init / cognitive doctor / Pi first conversation",
        mechanical_oracle: "doctor JSON first_conversation_ready plus fail-closed secret probe",
        cleanup: "runtime root and daemon.lock removed after the sample",
        evidence_schema: "cognitiveos.personal.readiness/0.1",
    },
    CapabilityTruthRow {
        id: "uj2.cold-warm-nested-timing",
        family: "UJ2",
        title: "cold/warm conversation with nested timing",
        scope: CapabilityTruthScope::Required,
        public_caller: "Pi daemon-provider nested stage observation",
        mechanical_oracle: "seven monotonic stages, content-free terminal records",
        cleanup: "sidecar session stopped; no orphan Pi process",
        evidence_schema: "cognitiveos.pi.nested-stage-timing/0.1",
    },
    CapabilityTruthRow {
        id: "uj3.status-doctor-resource-task",
        family: "UJ3",
        title: "status, doctor, resource, and Task operations",
        scope: CapabilityTruthScope::Required,
        public_caller: "cognitive status / doctor; GET /personal/status; six-family resource GET; Task GET",
        mechanical_oracle: "channel isolation 401/403/200; snapshot-first cursor; no secret material",
        cleanup: "loopback daemon stopped; hermetic runtime root removed",
        evidence_schema: "cognitiveos.personal.status/0.1",
    },
    CapabilityTruthRow {
        id: "uj3.restart-bounded-replay",
        family: "UJ3",
        title: "restart bounded replay",
        scope: CapabilityTruthScope::Required,
        public_caller: "GET /task/observation?family=o13 and six-family resource snapshot after restart",
        mechanical_oracle: "stale cursor 409; digest-break 409; restart-stable empty-window digest",
        cleanup: "observation overlay and runtime root removed",
        evidence_schema: "cognitiveos.personal.observation-plane/0.1",
    },
    CapabilityTruthRow {
        id: "uj4.task-admission-terminal-evidence",
        family: "UJ4",
        title: "Task admission, execution, and durable terminal query",
        scope: CapabilityTruthScope::Required,
        public_caller: "POST /task/intent admit; production Workspace*/RegisteredCheckRun; GET /task/evidence; admin-cli evidence",
        mechanical_oracle: "redacted terminal evidence; no raw SQLite; no false-completion event",
        cleanup: "task-scoped runtime root removed",
        evidence_schema: "cognitiveos.personal.task-evidence/0.1",
    },
    CapabilityTruthRow {
        id: "uj5.fault-restart-cleanup",
        family: "UJ5",
        title: "daemon/Pi kill, deadline, restart, cleanup (merged B3)",
        scope: CapabilityTruthScope::Required,
        public_caller: "GET /task/effects; authorized fault profiles; original-key restart reconcile",
        mechanical_oracle: "mutation count 0 or 1; Indeterminate/open Effects never complete a Task",
        cleanup: "fault profile overlay cleared; no leftover process/socket",
        evidence_schema: "cognitiveos.personal.effect-history/0.1",
    },
    CapabilityTruthRow {
        id: "uj6.memory-lifecycle",
        family: "UJ6",
        title: "Memory remember/review/forget",
        scope: CapabilityTruthScope::Required,
        public_caller: "POST /task/resource/v1/memory/{remember,review,forget}; GET /task/resource/v1/consumption",
        mechanical_oracle: "redacted pins; forged-prompt/digest fail closed; session-2 GET resume",
        cleanup: "memory objects remain in the hermetic store only",
        evidence_schema: "cognitiveos.personal.memory-consumption/0.1",
    },
    CapabilityTruthRow {
        id: "uj6.skill-lifecycle",
        family: "UJ6",
        title: "Skill import/inspect/bind",
        scope: CapabilityTruthScope::Required,
        public_caller: "POST /management/resource/v1/skill/{import,inspect,bind,binding/revoke}",
        mechanical_oracle: "revoked/forgotten pins absent from consumption; scripts never execute",
        cleanup: "imported package remains in the hermetic store only",
        evidence_schema: "cognitiveos.personal.skill-binding/0.1",
    },
    CapabilityTruthRow {
        id: "uj6.workspace-read-search-write-patch-check",
        family: "UJ6",
        title: "Workspace read/search/write/patch/check",
        scope: CapabilityTruthScope::Required,
        public_caller: "production WorkspaceRead/Search/Write/Patch plus check_id-only RegisteredCheckRun",
        mechanical_oracle: "containment; expected-preimage CAS; hidden-test gutting fail closed",
        cleanup: "workspace fixture restored; RegisteredCheckRun process family reaped",
        evidence_schema: "cognitiveos.personal.governed-workspace/0.1",
    },
    CapabilityTruthRow {
        id: "uj6.pi-lifecycle",
        family: "UJ6",
        title: "managed Pi install through recover",
        scope: CapabilityTruthScope::Required,
        public_caller: "admin-cli install/activate-root/register/activate/pause/resume/upgrade/rollback/stop/recover/uninstall",
        mechanical_oracle: "process-bound STATE_CONFLICT; monotonic rollback activation_version; orphan recover",
        cleanup: "installation store uninstalled; no leftover sidecar session",
        evidence_schema: "cognitiveos.personal.pi-lifecycle/0.1",
    },
    CapabilityTruthRow {
        id: "uj6.backup-restore",
        family: "UJ6",
        title: "backup and restore",
        scope: CapabilityTruthScope::Required,
        public_caller: "cognitive backup/restore; POST /management/resource/v1/backup and /restore",
        mechanical_oracle: "secret/SQLite excluded; tamper/missing-part/schema fail closed; byte-equal restore",
        cleanup: "no restore-staging-* or restore-snapshot-*; /tmp/cos-p2t27-* removed",
        evidence_schema: "cognitiveos.personal.backup/0.1",
    },
    CapabilityTruthRow {
        id: "uj6.verified-completion",
        family: "UJ6",
        title: "verified Task completion",
        scope: CapabilityTruthScope::Required,
        public_caller: "complete_task_from_persisted_verification; GET /task/evidence",
        mechanical_oracle: "missing report/CAS/open Effect fail closed; evidence-bound COMPLETED",
        cleanup: "hermetic authority store removed",
        evidence_schema: "cognitiveos.personal.task-completion/0.1",
    },
    CapabilityTruthRow {
        id: "uj6.web-ui",
        family: "UJ6",
        title: "Web UI / Console",
        scope: CapabilityTruthScope::ScopeExcluded,
        public_caller: "scope-excluded",
        mechanical_oracle: "scope-excluded",
        cleanup: "scope-excluded",
        evidence_schema: "scope-excluded",
    },
    CapabilityTruthRow {
        id: "uj6.multi-agent",
        family: "UJ6",
        title: "Multi-Agent",
        scope: CapabilityTruthScope::ScopeExcluded,
        public_caller: "scope-excluded",
        mechanical_oracle: "scope-excluded",
        cleanup: "scope-excluded",
        evidence_schema: "scope-excluded",
    },
];

pub fn validate_capability_truth_matrix(
    rows: &[CapabilityTruthRow],
) -> Result<(), CapabilityTruthError> {
    let mut seen = std::collections::BTreeSet::new();
    let mut families = std::collections::BTreeSet::new();
    for row in rows {
        if !seen.insert(row.id) {
            return Err(CapabilityTruthError::DuplicateId);
        }
        families.insert(row.family);
        match row.scope {
            CapabilityTruthScope::Required => {
                if row.public_caller.is_empty() || row.public_caller == "scope-excluded" {
                    return Err(CapabilityTruthError::MissingCaller);
                }
                if row.mechanical_oracle.is_empty() || row.mechanical_oracle == "scope-excluded" {
                    return Err(CapabilityTruthError::MissingOracle);
                }
                if row.cleanup.is_empty() || row.cleanup == "scope-excluded" {
                    return Err(CapabilityTruthError::MissingCleanup);
                }
                if row.evidence_schema.is_empty() || row.evidence_schema == "scope-excluded" {
                    return Err(CapabilityTruthError::MissingEvidenceSchema);
                }
            }
            CapabilityTruthScope::ScopeExcluded => {
                if row.public_caller != "scope-excluded"
                    || row.mechanical_oracle != "scope-excluded"
                {
                    return Err(CapabilityTruthError::ExcludedRowMarkedRequired);
                }
            }
        }
    }
    for family in REQUIRED_FAMILIES {
        if !families.contains(family) {
            return Err(CapabilityTruthError::MissingFamily);
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn required_template() -> CapabilityTruthRow {
        CapabilityTruthRow {
            id: "uj1.probe",
            family: "UJ1",
            title: "probe",
            scope: CapabilityTruthScope::Required,
            public_caller: "cognitive status",
            mechanical_oracle: "HTTP 200",
            cleanup: "runtime root removed",
            evidence_schema: "cognitiveos.personal.status/0.1",
        }
    }

    #[test]
    fn frozen_register_validates() {
        assert_eq!(
            validate_capability_truth_matrix(FROZEN_UJ_CAPABILITY_TRUTH),
            Ok(())
        );
        assert!(
            FROZEN_UJ_CAPABILITY_TRUTH
                .iter()
                .filter(|row| row.scope == CapabilityTruthScope::Required)
                .count()
                >= 12
        );
        assert!(
            FROZEN_UJ_CAPABILITY_TRUTH
                .iter()
                .any(|row| row.id == "uj6.web-ui"
                    && row.scope == CapabilityTruthScope::ScopeExcluded)
        );
        assert!(
            FROZEN_UJ_CAPABILITY_TRUTH
                .iter()
                .any(|row| row.id == "uj6.multi-agent"
                    && row.scope == CapabilityTruthScope::ScopeExcluded)
        );
    }

    #[test]
    fn missing_caller_is_rejected() {
        let mut row = required_template();
        row.public_caller = "";
        assert_eq!(
            validate_capability_truth_matrix(&[row]),
            Err(CapabilityTruthError::MissingCaller)
        );
        row.public_caller = "scope-excluded";
        assert_eq!(
            validate_capability_truth_matrix(&[row]),
            Err(CapabilityTruthError::MissingCaller)
        );
    }

    #[test]
    fn missing_oracle_is_rejected() {
        let mut row = required_template();
        row.mechanical_oracle = "";
        assert_eq!(
            validate_capability_truth_matrix(&[row]),
            Err(CapabilityTruthError::MissingOracle)
        );
    }

    #[test]
    fn excluded_web_ui_cannot_block_required_arm() {
        let excluded = CapabilityTruthRow {
            id: "uj6.web-ui",
            family: "UJ6",
            title: "Web UI",
            scope: CapabilityTruthScope::Required,
            public_caller: "scope-excluded",
            mechanical_oracle: "scope-excluded",
            cleanup: "scope-excluded",
            evidence_schema: "scope-excluded",
        };
        assert_eq!(
            validate_capability_truth_matrix(&[excluded]),
            Err(CapabilityTruthError::MissingCaller)
        );
    }

    #[test]
    fn excluded_row_cannot_smuggle_a_required_caller() {
        let row = CapabilityTruthRow {
            id: "uj6.web-ui",
            family: "UJ6",
            title: "Web UI",
            scope: CapabilityTruthScope::ScopeExcluded,
            public_caller: "GET /console",
            mechanical_oracle: "scope-excluded",
            cleanup: "scope-excluded",
            evidence_schema: "scope-excluded",
        };
        assert_eq!(
            validate_capability_truth_matrix(&[row]),
            Err(CapabilityTruthError::ExcludedRowMarkedRequired)
        );
    }

    #[test]
    fn duplicate_id_is_rejected() {
        let row = required_template();
        assert_eq!(
            validate_capability_truth_matrix(&[row, row]),
            Err(CapabilityTruthError::DuplicateId)
        );
    }

    #[test]
    fn missing_family_is_rejected() {
        assert_eq!(
            validate_capability_truth_matrix(&[required_template()]),
            Err(CapabilityTruthError::MissingFamily)
        );
    }
}
