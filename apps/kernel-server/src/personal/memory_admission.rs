//! Daemon-private Memory proposal admission service.
//!
//! Callers may provide a candidate and an evidence-shaped decision record, but
//! this service reloads the current Context source and re-derives the policy
//! outcome before it allows the authority store transaction to persist.

use cognitive_kernel::memory_admission::{
    CurrentMemorySourceFacts, MemoryAdmissionOutcome, MemoryAdmissionPolicy, MemoryProposal,
    decide_memory_admission,
};
use cognitive_kernel::ports::{
    ContextStore, MemoryAdmissionDecisionRow, MemoryCandidateRow, MemoryObjectRow, MemoryStore,
    StorePortError,
};

pub(crate) fn admit_memory_candidate<S>(
    store: &S,
    candidate: &MemoryCandidateRow,
    requested_decision: &MemoryAdmissionDecisionRow,
    requested_memory_object: Option<&MemoryObjectRow>,
    policy: &MemoryAdmissionPolicy,
) -> Result<MemoryAdmissionOutcome, StorePortError>
where
    S: ContextStore + MemoryStore,
{
    let source = store
        .load_workspace_context_source_body(&candidate.source_id)?
        .ok_or_else(|| StorePortError::Conflict {
            detail: format!("Memory candidate {} names an unknown Context source", candidate.candidate_id),
        })?;
    let proposal = MemoryProposal {
        candidate_id: candidate.candidate_id.as_str().to_owned(),
        source_id: candidate.source_id.as_str().to_owned(),
        source_digest: candidate.source_digest.clone(),
        source_provenance_ref: candidate.source_provenance_ref.clone(),
        governance_scope: candidate.governance_scope.clone(),
        target_scope: candidate.target_scope.clone(),
        purpose: candidate.purpose.clone(),
        retention_expires_at_unix_seconds: candidate.retention_expires_at_unix_seconds,
        observed_at_unix_seconds: candidate.observed_at_unix_seconds,
    };
    let current_source = CurrentMemorySourceFacts {
        source_id: source.source_id.as_str().to_owned(),
        source_digest: source.source_digest,
        provenance_ref: source.provenance_ref,
        governance_scope: source.governance.resource_scope,
        // Context source identity/digest/provenance/scope are authoritative
        // current facts here. Timestamp freshness policy remains a later P4
        // policy extension, so do not invent a second time interpretation.
        observed_at_unix_seconds: candidate.observed_at_unix_seconds,
    };
    let derived_decision = decide_memory_admission(&proposal, &current_source, policy);
    let expected_decision = match derived_decision.outcome {
        MemoryAdmissionOutcome::Admit => "admit",
        MemoryAdmissionOutcome::Reject => "reject",
    };
    if requested_decision.decision != expected_decision
        || requested_decision.policy_version != derived_decision.policy_version
        || requested_decision.reason_codes_json
            != serde_json::to_string(&derived_decision.reason_codes).map_err(|error| {
                StorePortError::Unavailable {
                    detail: format!("serialize derived Memory reason codes: {error}"),
                }
            })?
    {
        return Err(StorePortError::Conflict {
            detail: "Memory admission record does not match the daemon-derived policy outcome"
                .to_owned(),
        });
    }
    store.append_memory_admission(candidate, requested_decision, requested_memory_object)?;
    Ok(derived_decision.outcome)
}
mod memory_admission;
