//! Deterministic Memory proposal admission for the Personal daemon.
//!
//! This module intentionally accepts only daemon-reloaded source facts. A
//! producer can describe a candidate, but it cannot choose its admission
//! outcome or promote its governance scope.

/// Immutable proposal facts supplied by an upstream producer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryProposal {
    pub candidate_id: String,
    pub source_id: String,
    pub source_digest: String,
    pub source_provenance_ref: String,
    pub governance_scope: String,
    pub target_scope: String,
    pub purpose: String,
    pub retention_expires_at_unix_seconds: i64,
    pub observed_at_unix_seconds: i64,
}

/// Current durable source facts reloaded by the daemon before admission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentMemorySourceFacts {
    pub source_id: String,
    pub source_digest: String,
    pub provenance_ref: String,
    pub governance_scope: String,
    pub observed_at_unix_seconds: i64,
}

/// Deterministic policy inputs that cannot be selected by a proposal producer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryAdmissionPolicy {
    pub policy_version: i64,
    pub now_unix_seconds: i64,
    pub maximum_retention_seconds: i64,
}

/// The only durable-policy outcomes a caller may persist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryAdmissionOutcome {
    Admit,
    Reject,
}

/// Reason-coded deterministic outcome. An `Admit` result is necessary but not
/// sufficient for persistence: the daemon-owned store transaction remains the
/// authority that appends the candidate, decision, and Memory object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryAdmissionDecision {
    pub outcome: MemoryAdmissionOutcome,
    pub policy_version: i64,
    pub reason_codes: Vec<&'static str>,
}

/// Derive a Memory admission decision from current daemon-owned source facts.
pub fn decide_memory_admission(
    proposal: &MemoryProposal,
    current_source: &CurrentMemorySourceFacts,
    policy: &MemoryAdmissionPolicy,
) -> MemoryAdmissionDecision {
    let mut reason_codes = Vec::new();

    if policy.policy_version < 1 || policy.maximum_retention_seconds < 1 {
        reason_codes.push("MEMORY_ADMISSION_POLICY_INVALID");
    }
    if proposal.candidate_id.is_empty() || proposal.purpose.trim().is_empty() {
        reason_codes.push("MEMORY_ADMISSION_DENIED");
    }
    if proposal.source_id != current_source.source_id
        || proposal.source_digest != current_source.source_digest
        || proposal.source_provenance_ref != current_source.provenance_ref
        || proposal.observed_at_unix_seconds != current_source.observed_at_unix_seconds
    {
        reason_codes.push("MEMORY_DERIVATION_INVALIDATED");
    }
    if proposal.governance_scope != current_source.governance_scope {
        reason_codes.push("MEMORY_ADMISSION_DENIED");
    }
    if proposal.target_scope != proposal.governance_scope {
        reason_codes.push("MEMORY_SCOPE_PROMOTION_REQUIRED");
    }

    let retention_seconds = proposal
        .retention_expires_at_unix_seconds
        .saturating_sub(policy.now_unix_seconds);
    if retention_seconds < 1 || retention_seconds > policy.maximum_retention_seconds {
        reason_codes.push("MEMORY_ADMISSION_DENIED");
    }

    if reason_codes.is_empty() {
        MemoryAdmissionDecision {
            outcome: MemoryAdmissionOutcome::Admit,
            policy_version: policy.policy_version,
            reason_codes: vec!["MEMORY_ADMISSION_ACCEPTED"],
        }
    } else {
        reason_codes.sort_unstable();
        reason_codes.dedup();
        MemoryAdmissionDecision {
            outcome: MemoryAdmissionOutcome::Reject,
            policy_version: policy.policy_version,
            reason_codes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn current_source() -> CurrentMemorySourceFacts {
        CurrentMemorySourceFacts {
            source_id: "context-source-1".to_owned(),
            source_digest: "sha256:a".to_owned(),
            provenance_ref: "workspace://source/1".to_owned(),
            governance_scope: "workspace://personal".to_owned(),
            observed_at_unix_seconds: 100,
        }
    }

    fn proposal() -> MemoryProposal {
        MemoryProposal {
            candidate_id: "memory-candidate-1".to_owned(),
            source_id: "context-source-1".to_owned(),
            source_digest: "sha256:a".to_owned(),
            source_provenance_ref: "workspace://source/1".to_owned(),
            governance_scope: "workspace://personal".to_owned(),
            target_scope: "workspace://personal".to_owned(),
            purpose: "retain task-relevant fact".to_owned(),
            retention_expires_at_unix_seconds: 200,
            observed_at_unix_seconds: 100,
        }
    }

    fn policy() -> MemoryAdmissionPolicy {
        MemoryAdmissionPolicy {
            policy_version: 1,
            now_unix_seconds: 150,
            maximum_retention_seconds: 100,
        }
    }

    #[test]
    fn admits_a_current_same_scope_proposal_with_bounded_retention() {
        assert_eq!(
            decide_memory_admission(&proposal(), &current_source(), &policy()).outcome,
            MemoryAdmissionOutcome::Admit
        );
    }

    #[test]
    fn rejects_stale_source_binding_and_scope_promotion() {
        let mut stale_proposal = proposal();
        stale_proposal.source_digest = "sha256:b".to_owned();
        stale_proposal.target_scope = "owner://personal".to_owned();

        let decision = decide_memory_admission(&stale_proposal, &current_source(), &policy());
        assert_eq!(decision.outcome, MemoryAdmissionOutcome::Reject);
        assert_eq!(
            decision.reason_codes,
            vec![
                "MEMORY_DERIVATION_INVALIDATED",
                "MEMORY_SCOPE_PROMOTION_REQUIRED"
            ]
        );
    }
}
