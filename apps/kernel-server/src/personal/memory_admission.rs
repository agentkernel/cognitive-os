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

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use cognitive_contracts::generated::context_view::{
        LoadedContextItemRepresentation, LoadedContextItemRole, LoadedContextItemTrustLevel,
    };
    use cognitive_domain::ObjectId;
    use cognitive_kernel::authz::ObjectGovernance;
    use cognitive_kernel::ports::{
        ContextCandidateMetadata, ContextCandidateQuery, ContextRequestRow, ContextViewRow,
        MemorySearchCandidateRow, MemorySearchQuery, WorkspaceContextSourceRow,
    };

    struct MemoryAdmissionTestStore {
        source: WorkspaceContextSourceRow,
        persist_called: std::cell::Cell<bool>,
    }

    fn unsupported_test_operation(operation: &str) -> StorePortError {
        StorePortError::Unavailable {
            detail: format!("Memory admission test store does not support {operation}"),
        }
    }

    impl ContextStore for MemoryAdmissionTestStore {
        fn append_context_request(&self, _: &ContextRequestRow) -> Result<(), StorePortError> {
            Err(unsupported_test_operation("append ContextRequest"))
        }

        fn load_context_request(
            &self,
            _: &ObjectId,
        ) -> Result<Option<ContextRequestRow>, StorePortError> {
            Err(unsupported_test_operation("load ContextRequest"))
        }

        fn append_context_view(&self, _: &ContextViewRow) -> Result<(), StorePortError> {
            Err(unsupported_test_operation("append ContextView"))
        }

        fn load_context_view(&self, _: &ObjectId) -> Result<Option<ContextViewRow>, StorePortError> {
            Err(unsupported_test_operation("load ContextView"))
        }

        fn append_workspace_context_source(
            &self,
            _: &WorkspaceContextSourceRow,
        ) -> Result<(), StorePortError> {
            Err(unsupported_test_operation("append Context source"))
        }

        fn query_context_candidate_metadata(
            &self,
            _: &ContextCandidateQuery,
        ) -> Result<Vec<ContextCandidateMetadata>, StorePortError> {
            Err(unsupported_test_operation("query Context metadata"))
        }

        fn load_workspace_context_source_body(
            &self,
            source_id: &ObjectId,
        ) -> Result<Option<WorkspaceContextSourceRow>, StorePortError> {
            Ok((source_id == &self.source.source_id).then(|| self.source.clone()))
        }
    }

    impl MemoryStore for MemoryAdmissionTestStore {
        fn append_memory_admission(
            &self,
            _: &MemoryCandidateRow,
            _: &MemoryAdmissionDecisionRow,
            _: Option<&MemoryObjectRow>,
        ) -> Result<(), StorePortError> {
            self.persist_called.set(true);
            Ok(())
        }

        fn load_memory_object(
            &self,
            _: &ObjectId,
        ) -> Result<Option<MemoryObjectRow>, StorePortError> {
            Err(unsupported_test_operation("load MemoryObject"))
        }

        fn search_memory_candidates(
            &self,
            _: &MemorySearchQuery,
        ) -> Result<Vec<MemorySearchCandidateRow>, StorePortError> {
            Err(unsupported_test_operation("search Memory candidates"))
        }

        fn rebuild_memory_search_index(&self) -> Result<(), StorePortError> {
            Err(unsupported_test_operation("rebuild Memory search index"))
        }
    }

    fn object_id(sequence: u64) -> ObjectId {
        ObjectId::parse(&format!("00000000-0000-7000-9000-{sequence:012x}")).unwrap()
    }

    #[test]
    fn daemon_rejects_producer_admit_when_current_source_digest_changed() {
        let source_id = object_id(1);
        let store = MemoryAdmissionTestStore {
            source: WorkspaceContextSourceRow {
                source_id: source_id.clone(),
                source_digest: "sha256:current".to_owned(),
                governance: ObjectGovernance {
                    object_ref: source_id.as_str().to_owned(),
                    tenant_id: Some("tenant-a".to_owned()),
                    owner_ref: "principal://tenant-a/daemon".to_owned(),
                    resource_scope: "workspace://tenant-a/project".to_owned(),
                    conversation_ref: None,
                },
                role: LoadedContextItemRole::Working,
                trust_level: LoadedContextItemTrustLevel::Verified,
                representation: LoadedContextItemRepresentation::Text,
                provenance_ref: "source://context/1".to_owned(),
                content_bytes: 1,
                content_tokens: Some(1),
                canonical_json: "{}".to_owned(),
            },
            persist_called: std::cell::Cell::new(false),
        };
        let candidate = MemoryCandidateRow {
            candidate_id: object_id(2),
            candidate_digest: "sha256:candidate".to_owned(),
            source_id,
            source_digest: "sha256:stale".to_owned(),
            source_provenance_ref: "source://context/1".to_owned(),
            governance_scope: "workspace://tenant-a/project".to_owned(),
            target_scope: "workspace://tenant-a/project".to_owned(),
            purpose: "task fact".to_owned(),
            retention_expires_at_unix_seconds: 200,
            observed_at_unix_seconds: 100,
            canonical_json: "{}".to_owned(),
        };
        let requested_decision = MemoryAdmissionDecisionRow {
            decision_id: object_id(3),
            candidate_id: candidate.candidate_id.clone(),
            candidate_digest: candidate.candidate_digest.clone(),
            decision: "admit".to_owned(),
            policy_version: 1,
            reason_codes_json: "[\"MEMORY_ADMISSION_ACCEPTED\"]".to_owned(),
            canonical_json: "{}".to_owned(),
        };
        let policy = MemoryAdmissionPolicy {
            policy_version: 1,
            now_unix_seconds: 150,
            maximum_retention_seconds: 100,
        };

        assert!(matches!(
            admit_memory_candidate(&store, &candidate, &requested_decision, None, &policy),
            Err(StorePortError::Conflict { .. })
        ));
        assert!(!store.persist_called.get());
    }
}
mod memory_admission;
