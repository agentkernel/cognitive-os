#![allow(dead_code, unused_imports)]

use crate::context_store::{
    CONTEXT_AUTHORIZATION_FACT_SCHEMA_V14, CONTEXT_STORE_SCHEMA_V12,
    SCHEDULER_EXECUTION_POLICY_SCHEMA_V15, WORKSPACE_CONTEXT_SOURCE_SCHEMA_V13,
};
use crate::memory_store::{MEMORY_ADMISSION_SCHEMA_V16, MEMORY_SEARCH_SCHEMA_V17};
use crate::scheduler::SCHEDULER_SCHEMA_CURRENT;
use crate::worker_authorization::{
    CONTINUATION_AUTHORITY_CONSUMPTION_SCHEMA_V11, CONTINUATION_AUTHORITY_SCHEMA_V10,
    DAEMON_AUTHORIZATION_SNAPSHOT_SCHEMA_V6, DAEMON_OPERATION_DESCRIPTOR_SCHEMA_V5,
    WORKER_AUTHORIZATION_LEASE_BINDING_SCHEMA_V9, WORKER_AUTHORIZATION_SCHEMA_V4,
    WORKER_ITERATION_AUTHORIZATION_CONSUMPTION_SCHEMA_V8, WORKER_ITERATION_AUTHORIZATION_SCHEMA_V7,
};
use cognitive_contracts::generated::context_request::ContextRequest;
use cognitive_contracts::generated::context_view::ContextView;
use cognitive_contracts::generated::governed_object_header::GovernedObjectHeader;
use cognitive_contracts::generated::object_reference::StrongReferenceKind;
use cognitive_contracts::projection::verify_content_digest;
use cognitive_domain::{
    BudgetId, EventId, LifecycleDomain, ObjectId, StateName, Version, WallTimestamp,
};
use cognitive_kernel::authz::ObjectGovernance;
use cognitive_kernel::effects::GOVERNED_OBJECT_CONTENT_DIGEST_DOMAIN;
use cognitive_kernel::ports::{
    AuthorityStore, BoundContinuationAuthorizationConsumption, BoundWorkerAuthorizationConsumption,
    CandidateAdmissionCommit, CandidateAdmissionReceipt, CheckpointRow, CommitReceipt,
    CommittedEvent, ConsumedWorkerIterationAuthorization, ContextAuthorizationFactStore,
    ContextAuthorizationFactsRow, ContextCandidateMetadata, ContextCandidateQuery,
    ContextRequestRow, ContextRevocationFactRow, ContextStore, ContextViewRow,
    ContinuationAuthorityStore, ContinuationAuthorizationRow, DaemonAuthorizationSnapshotRow,
    DaemonOperationDescriptorRow, FixedPostStateRow, GovernanceObjectStore, HarnessStore,
    IntentChainStore, IntentRow, InterpretationRow, MemoryAdmissionDecisionRow, MemoryCandidateRow,
    MemoryObjectRow, MemorySearchCandidateRow, MemorySearchQuery, MemoryStore, MemoryTombstoneRow,
    MemoryUpdateRequest, ObjectAdmission, OperationCandidateProposalRow, OutboxEntry,
    ProgressFactRow, ProtocolStore, SchedulerExecutionPolicyRow, SchedulerExecutionPolicyStore,
    SchedulerLeaseBinding, SkillBindingExplanationRow, SkillBindingRevocationRow, SkillBindingRow,
    SkillPackageRow, SkillRevisionRow, SkillRevisionSupersedeRequest, SkillStore, StorePortError,
    StoredBudget, StoredObject, TaskBinding, TaskContractRow, TransitionCommit,
    UserIntentRecordRow, VerificationReportRow, VerificationRequestRow, WorkerAuthorizationStore,
    WorkerIterationAuthorizationConsumptionRow, WorkerIterationAuthorizationRow,
    WorkspaceContextSourceRow,
};
use cognitive_kernel::{BudgetState, EffectClass, ExecutorCapabilities, OperationDescriptor};
use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use super::*;

impl HarnessStore for SqliteAuthorityStore {
    fn append_progress_fact(&self, fact: &ProgressFactRow) -> Result<(), StorePortError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin progress fact"))?;
        // Same sink discipline as checkpoints (F-014 store-transaction
        // class): a stale writer cannot poison the stagnation counters.
        verify_fencing_in_tx(&tx, Some(fact.fencing_epoch))?;
        let inserted = tx.execute(
            "INSERT INTO loop_progress_facts
               (loop_object_id, iteration, status, action_fingerprint, evidence_refs_json,
                recorded_at, fencing_epoch)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                fact.loop_object_id.as_str(),
                fact.iteration,
                fact.status.as_str(),
                fact.action_fingerprint.as_str(),
                fact.evidence_refs_json.as_str(),
                fact.recorded_at.as_str(),
                fact.fencing_epoch,
            ),
        );
        match inserted {
            Ok(_) => {}
            Err(err) if is_constraint_violation(&err) => {
                return Err(StorePortError::Conflict {
                    detail: format!(
                        "progress fact for loop {} iteration {} already recorded",
                        fact.loop_object_id, fact.iteration
                    ),
                });
            }
            Err(err) => return Err(unavailable("insert progress fact")(err)),
        }
        tx.commit().map_err(unavailable("commit progress fact"))?;
        Ok(())
    }

    fn list_progress_facts(
        &self,
        loop_object_id: &ObjectId,
    ) -> Result<Vec<ProgressFactRow>, StorePortError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare_cached(
                "SELECT loop_object_id, iteration, status, action_fingerprint,
                        evidence_refs_json, recorded_at, fencing_epoch
                 FROM loop_progress_facts WHERE loop_object_id = ?1 ORDER BY iteration ASC",
            )
            .map_err(unavailable("prepare list_progress_facts"))?;
        let mut rows = statement
            .query((loop_object_id.as_str(),))
            .map_err(unavailable("query list_progress_facts"))?;
        let mut facts = Vec::new();
        while let Some(row) = rows.next().map_err(unavailable("read progress fact"))? {
            let loop_id: String = row.get(0).map_err(unavailable("column loop_object_id"))?;
            let recorded_at: String = row.get(5).map_err(unavailable("column recorded_at"))?;
            facts.push(ProgressFactRow {
                loop_object_id: ObjectId::parse(&loop_id)
                    .map_err(|err| corrupt("loop_object_id", err))?,
                iteration: row.get(1).map_err(unavailable("column iteration"))?,
                status: row.get(2).map_err(unavailable("column status"))?,
                action_fingerprint: row
                    .get(3)
                    .map_err(unavailable("column action_fingerprint"))?,
                evidence_refs_json: row
                    .get(4)
                    .map_err(unavailable("column evidence_refs_json"))?,
                recorded_at: WallTimestamp::parse(&recorded_at)
                    .map_err(|err| corrupt("recorded_at", err))?,
                fencing_epoch: row.get(6).map_err(unavailable("column fencing_epoch"))?,
            });
        }
        Ok(facts)
    }
}

impl SkillStore for SqliteAuthorityStore {
    fn append_skill_import(
        &self,
        package: &SkillPackageRow,
        revision: &SkillRevisionRow,
    ) -> Result<(), StorePortError> {
        let unsafe_local_path = package.local_source_path.starts_with('/')
            || package.local_source_path.contains("\\\\")
            || package
                .local_source_path
                .split('/')
                .any(|segment| segment == "..");
        let manifest_digest_matches_payload = canonical_json_digest_matches(
            &package.canonical_json,
            "manifest_digest",
            &package.manifest_digest,
        );
        let content_digest_matches_payload = canonical_json_digest_matches(
            &revision.canonical_json,
            "content_digest",
            &revision.content_digest,
        );
        let invalid_import = package.workspace_scope.trim().is_empty()
            || package.local_source_path.trim().is_empty()
            || package.provenance_ref.trim().is_empty()
            || package.manifest_digest.trim().is_empty()
            || revision.package_id != package.package_id
            || revision.content_digest.trim().is_empty()
            || !matches!(
                revision.compatibility.as_str(),
                "compatible" | "incompatible"
            )
            || !manifest_digest_matches_payload
            || !content_digest_matches_payload;
        if unsafe_local_path || invalid_import {
            return Err(StorePortError::Conflict {
                detail: "Skill import has unsafe local provenance or invalid immutable bindings"
                    .to_owned(),
            });
        }

        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin Skill import transaction"))?;
        let insert_result = (|| -> Result<(), rusqlite::Error> {
            transaction.execute(
                "INSERT INTO skill_packages (package_id, workspace_scope, local_source_path, provenance_ref, manifest_digest, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (package.package_id.as_str(), package.workspace_scope.as_str(), package.local_source_path.as_str(), package.provenance_ref.as_str(), package.manifest_digest.as_str(), package.canonical_json.as_str()),
            )?;
            transaction.execute(
                "INSERT INTO skill_revisions (revision_id, package_id, content_digest, compatibility, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5)",
                (revision.revision_id.as_str(), revision.package_id.as_str(), revision.content_digest.as_str(), revision.compatibility.as_str(), revision.canonical_json.as_str()),
            )?;
            Ok(())
        })();
        match insert_result {
            Ok(()) => transaction
                .commit()
                .map_err(unavailable("commit Skill import transaction")),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: "Skill import conflicts with an immutable package or revision".to_owned(),
            }),
            Err(error) => Err(unavailable("insert Skill import")(error)),
        }
    }

    fn append_skill_revision_supersede(
        &self,
        supersede: &SkillRevisionSupersedeRequest,
    ) -> Result<(), StorePortError> {
        let replacement = &supersede.replacement;
        let invalid_supersede = replacement.revision_id == supersede.previous_revision_id
            || replacement.content_digest.trim().is_empty()
            || !matches!(
                replacement.compatibility.as_str(),
                "compatible" | "incompatible"
            )
            || !canonical_json_digest_matches(
                &replacement.canonical_json,
                "content_digest",
                &replacement.content_digest,
            )
            || serde_json::from_str::<Value>(&supersede.canonical_json).is_err();
        if invalid_supersede {
            return Err(StorePortError::Conflict {
                detail: "Skill revision supersede has invalid immutable replacement bindings"
                    .to_owned(),
            });
        }

        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin Skill revision supersede transaction"))?;
        let prior_package_id = transaction
            .query_row(
                "SELECT package_id FROM skill_revisions WHERE revision_id=?1",
                (supersede.previous_revision_id.as_str(),),
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(unavailable("load prior Skill revision"))?;
        let Some(prior_package_id) = prior_package_id else {
            return Err(StorePortError::Conflict {
                detail: "Skill revision supersede names an unknown prior revision".to_owned(),
            });
        };
        if prior_package_id != replacement.package_id.as_str() {
            return Err(StorePortError::Conflict {
                detail: "Skill revision supersede must remain in the same package".to_owned(),
            });
        }
        let insert_result = (|| -> Result<(), rusqlite::Error> {
            transaction.execute(
                "INSERT INTO skill_revisions (revision_id, package_id, content_digest, compatibility, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5)",
                (replacement.revision_id.as_str(), replacement.package_id.as_str(), replacement.content_digest.as_str(), replacement.compatibility.as_str(), replacement.canonical_json.as_str()),
            )?;
            transaction.execute(
                "INSERT INTO skill_revision_lineage (revision_id, supersedes_revision_id, canonical_json) VALUES (?1, ?2, ?3)",
                (replacement.revision_id.as_str(), supersede.previous_revision_id.as_str(), supersede.canonical_json.as_str()),
            )?;
            Ok(())
        })();
        match insert_result {
            Ok(()) => transaction
                .commit()
                .map_err(unavailable("commit Skill revision supersede transaction")),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: "Skill revision supersede conflicts with immutable revision lineage"
                    .to_owned(),
            }),
            Err(error) => Err(unavailable("insert Skill revision supersede")(error)),
        }
    }

    fn append_skill_binding(&self, binding: &SkillBindingRow) -> Result<(), StorePortError> {
        let invalid_binding = binding.workspace_scope.trim().is_empty()
            || binding.target_ref.trim().is_empty()
            || !matches!(binding.target_kind.as_str(), "agent" | "task" | "workspace")
            || !matches!(binding.status.as_str(), "active" | "revoked")
            || serde_json::from_str::<Value>(&binding.canonical_json).is_err();
        if invalid_binding {
            return Err(StorePortError::Conflict {
                detail: "Skill binding has invalid target, lifecycle, or canonical payload"
                    .to_owned(),
            });
        }

        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin Skill binding transaction"))?;
        let revision_scope = transaction
            .query_row(
                "SELECT skill_packages.workspace_scope, skill_revisions.compatibility FROM skill_revisions JOIN skill_packages ON skill_packages.package_id = skill_revisions.package_id WHERE skill_revisions.revision_id=?1",
                (binding.revision_id.as_str(),),
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(unavailable("load Skill revision for binding"))?;
        let Some((workspace_scope, compatibility)) = revision_scope else {
            return Err(StorePortError::Conflict {
                detail: "Skill binding names an unknown revision".to_owned(),
            });
        };
        if workspace_scope != binding.workspace_scope || compatibility != "compatible" {
            return Err(StorePortError::Conflict {
                detail: "Skill binding crosses workspace scope or names an incompatible revision"
                    .to_owned(),
            });
        }
        match transaction.execute(
            "INSERT INTO skill_bindings (binding_id, revision_id, workspace_scope, target_kind, target_ref, status, canonical_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (binding.binding_id.as_str(), binding.revision_id.as_str(), binding.workspace_scope.as_str(), binding.target_kind.as_str(), binding.target_ref.as_str(), binding.status.as_str(), binding.canonical_json.as_str()),
        ) {
            Ok(_) => transaction
                .commit()
                .map_err(unavailable("commit Skill binding transaction")),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: "Skill binding conflicts with an immutable binding".to_owned(),
            }),
            Err(error) => Err(unavailable("insert Skill binding")(error)),
        }
    }

    fn load_skill_binding(
        &self,
        binding_id: &ObjectId,
    ) -> Result<Option<SkillBindingRow>, StorePortError> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT revision_id, workspace_scope, target_kind, target_ref, status, canonical_json FROM skill_bindings WHERE binding_id=?1",
                (binding_id.as_str(),),
                |row| Ok(SkillBindingRow {
                    binding_id: binding_id.clone(),
                    revision_id: ObjectId::parse(&row.get::<_, String>(0)?).map_err(|error| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error)))?,
                    workspace_scope: row.get(1)?,
                    target_kind: row.get(2)?,
                    target_ref: row.get(3)?,
                    status: row.get(4)?,
                    canonical_json: row.get(5)?,
                }),
            )
            .optional()
            .map_err(unavailable("load Skill binding"))
    }

    fn append_skill_binding_revocation(
        &self,
        revocation: &SkillBindingRevocationRow,
    ) -> Result<(), StorePortError> {
        if revocation.reason.trim().is_empty()
            || serde_json::from_str::<Value>(&revocation.canonical_json).is_err()
        {
            return Err(StorePortError::Conflict {
                detail: "Skill binding revocation has an invalid reason or canonical payload"
                    .to_owned(),
            });
        }

        let mut connection = self.lock()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(unavailable("begin Skill binding revocation transaction"))?;
        let binding_exists = transaction
            .query_row(
                "SELECT 1 FROM skill_bindings WHERE binding_id=?1",
                (revocation.binding_id.as_str(),),
                |_| Ok(()),
            )
            .optional()
            .map_err(unavailable("load Skill binding for revocation"))?
            .is_some();
        if !binding_exists {
            return Err(StorePortError::Conflict {
                detail: "Skill binding revocation names an unknown binding".to_owned(),
            });
        }
        match transaction.execute(
            "INSERT INTO skill_binding_revocations (revocation_id, binding_id, reason, canonical_json) VALUES (?1, ?2, ?3, ?4)",
            (revocation.revocation_id.as_str(), revocation.binding_id.as_str(), revocation.reason.as_str(), revocation.canonical_json.as_str()),
        ) {
            Ok(_) => transaction
                .commit()
                .map_err(unavailable("commit Skill binding revocation transaction")),
            Err(error) if is_constraint_violation(&error) => Err(StorePortError::Conflict {
                detail: "Skill binding already has an immutable revocation".to_owned(),
            }),
            Err(error) => Err(unavailable("insert Skill binding revocation")(error)),
        }
    }

    fn load_active_skill_binding(
        &self,
        binding_id: &ObjectId,
    ) -> Result<Option<SkillBindingRow>, StorePortError> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT revision_id, workspace_scope, target_kind, target_ref, status, canonical_json FROM skill_bindings WHERE binding_id=?1 AND status='active' AND NOT EXISTS (SELECT 1 FROM skill_binding_revocations WHERE skill_binding_revocations.binding_id=skill_bindings.binding_id)",
                (binding_id.as_str(),),
                |row| Ok(SkillBindingRow {
                    binding_id: binding_id.clone(),
                    revision_id: ObjectId::parse(&row.get::<_, String>(0)?).map_err(|error| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error)))?,
                    workspace_scope: row.get(1)?,
                    target_kind: row.get(2)?,
                    target_ref: row.get(3)?,
                    status: row.get(4)?,
                    canonical_json: row.get(5)?,
                }),
            )
            .optional()
            .map_err(unavailable("load active Skill binding"))
    }

    fn explain_skill_binding(
        &self,
        binding_id: &ObjectId,
    ) -> Result<Option<SkillBindingExplanationRow>, StorePortError> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT skill_bindings.revision_id, skill_bindings.workspace_scope, skill_bindings.target_kind, skill_bindings.target_ref, skill_bindings.status, skill_bindings.canonical_json, skill_packages.package_id, skill_packages.manifest_digest, skill_revisions.content_digest, skill_binding_revocations.reason FROM skill_bindings JOIN skill_revisions ON skill_revisions.revision_id=skill_bindings.revision_id JOIN skill_packages ON skill_packages.package_id=skill_revisions.package_id LEFT JOIN skill_binding_revocations ON skill_binding_revocations.binding_id=skill_bindings.binding_id WHERE skill_bindings.binding_id=?1",
                (binding_id.as_str(),),
                |row| Ok(SkillBindingExplanationRow {
                    binding: SkillBindingRow {
                        binding_id: binding_id.clone(),
                        revision_id: ObjectId::parse(&row.get::<_, String>(0)?).map_err(|error| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error)))?,
                        workspace_scope: row.get(1)?,
                        target_kind: row.get(2)?,
                        target_ref: row.get(3)?,
                        status: row.get(4)?,
                        canonical_json: row.get(5)?,
                    },
                    package_id: ObjectId::parse(&row.get::<_, String>(6)?).map_err(|error| rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, Box::new(error)))?,
                    manifest_digest: row.get(7)?,
                    content_digest: row.get(8)?,
                    revocation_reason: row.get(9)?,
                }),
            )
            .optional()
            .map_err(unavailable("explain Skill binding"))
    }
}

impl SqliteAuthorityStore {
    /// Bounded list of Skill bindings plus whether each has a revocation.
    pub fn list_skill_bindings(
        &self,
        limit: usize,
    ) -> Result<(Vec<(SkillBindingRow, bool)>, bool), StorePortError> {
        let fetch = i64::try_from(limit.saturating_add(1)).unwrap_or(i64::MAX);
        let connection = self.lock()?;
        let mut statement = connection
            .prepare(
                "SELECT binding_id, revision_id, workspace_scope, target_kind, target_ref, status, canonical_json,
                        EXISTS(
                            SELECT 1 FROM skill_binding_revocations
                            WHERE skill_binding_revocations.binding_id = skill_bindings.binding_id
                        )
                 FROM skill_bindings
                 ORDER BY binding_id
                 LIMIT ?1",
            )
            .map_err(unavailable("prepare list Skill bindings"))?;
        let rows = statement
            .query_map([fetch], |row| {
                Ok((
                    SkillBindingRow {
                        binding_id: ObjectId::parse(&row.get::<_, String>(0)?).map_err(
                            |error| {
                                rusqlite::Error::FromSqlConversionFailure(
                                    0,
                                    rusqlite::types::Type::Text,
                                    Box::new(error),
                                )
                            },
                        )?,
                        revision_id: ObjectId::parse(&row.get::<_, String>(1)?).map_err(
                            |error| {
                                rusqlite::Error::FromSqlConversionFailure(
                                    1,
                                    rusqlite::types::Type::Text,
                                    Box::new(error),
                                )
                            },
                        )?,
                        workspace_scope: row.get(2)?,
                        target_kind: row.get(3)?,
                        target_ref: row.get(4)?,
                        status: row.get(5)?,
                        canonical_json: row.get(6)?,
                    },
                    row.get::<_, i64>(7)? != 0,
                ))
            })
            .map_err(unavailable("query list Skill bindings"))?;
        let mut bindings = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("read Skill bindings"))?;
        let truncated = bindings.len() > limit;
        if truncated {
            bindings.truncate(limit);
        }
        Ok((bindings, truncated))
    }
}
