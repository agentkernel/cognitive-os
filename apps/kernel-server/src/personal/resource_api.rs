//! Private, versioned resource projection for Personal daemon clients.
//!
//! This is deliberately not a public contract or a durable generic Resource
//! aggregate. It exposes the six fixed product families as daemon observations
//! and makes missing authority backends explicit rather than fabricating rows.

use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use cognitive_contracts::generated::context_view::{
    LoadedContextItemRepresentation, LoadedContextItemRole, LoadedContextItemTrustLevel,
};
use cognitive_contracts::generated::governed_object_header::GovernedObjectHeader;
use cognitive_domain::ObjectId;
use cognitive_kernel::BUILTIN_TOOL_CATALOG;
use cognitive_kernel::authz::ObjectGovernance;
use cognitive_kernel::memory_admission::MemoryAdmissionPolicy;
use cognitive_kernel::memory_skill_consumption::MemorySkillConsumptionStore;
use cognitive_kernel::ports::{
    ContextStore, IntentChainStore, MemoryAdmissionDecisionRow, MemoryCandidateRow,
    MemoryObjectRow, MemorySearchQuery, MemoryStore, MemoryTombstoneRow, ProtocolStore,
    SchedulerExecutionPolicyStore, SkillBindingRevocationRow, SkillBindingRow, SkillPackageRow,
    SkillRevisionRow, SkillRevisionSupersedeRequest, SkillStore, StorePortError,
    WorkspaceContextSourceRow,
};
use cognitive_store::{SqliteAuthorityStore, UuidV7Generator, admit_memory_candidate};
use serde_json::{Value, json};

const PROJECTION_VERSION: &str = "personal-resource-projection/1";
const MAX_WATCH_EVENTS: usize = 128;
const RESOURCE_FAMILIES: [&str; 6] = ["memory", "skill", "tool", "context", "task", "runtime"];

/// A response ready for the loopback HTTP transport.
pub(crate) struct ResourceApiResponse {
    pub status: u16,
    pub body: String,
    pub content_type: &'static str,
}

/// Process-lifetime delivery state for private resource observations.
pub(crate) struct ResourceApi {
    next_watch_sequence: u64,
    watch_events: VecDeque<(u64, String, Value)>,
}

impl ResourceApi {
    pub(crate) fn new() -> Self {
        let mut api = Self {
            next_watch_sequence: 1,
            watch_events: VecDeque::new(),
        };
        for family in RESOURCE_FAMILIES {
            api.publish(family, "projection.initialized", family_projection(family));
        }
        api
    }

    pub(crate) fn handle(&self, method_path: &str) -> ResourceApiResponse {
        self.handle_projection(method_path, None)
    }

    pub(crate) fn handle_task(&self, method_path: &str) -> ResourceApiResponse {
        let task_reference = method_path
            .split_once('?')
            .and_then(|(_, query)| query_parameter(query, "task_ref"));
        let Some(task_reference) = task_reference.filter(|value| !value.is_empty()) else {
            return error(
                400,
                "RESOURCE_TASK_REFERENCE_REQUIRED",
                "task-bound resource projection requires task_ref",
            );
        };
        self.handle_projection(method_path, Some(task_reference))
    }

    pub(crate) fn handle_task_consumption(
        &self,
        body: &[u8],
        store: &SqliteAuthorityStore,
    ) -> ResourceApiResponse {
        let Ok(document) = serde_json::from_slice::<Value>(body) else {
            return error(
                400,
                "RESOURCE_CONSUMPTION_PAYLOAD_INVALID",
                "consumption payload is invalid",
            );
        };
        let Some(task_reference) =
            string_field(&document, "task_ref").filter(|value| !value.is_empty())
        else {
            return error(
                400,
                "RESOURCE_TASK_REFERENCE_REQUIRED",
                "task_ref is required",
            );
        };
        let Some(query_text) =
            string_field(&document, "query_text").filter(|value| !value.is_empty())
        else {
            return error(
                400,
                "RESOURCE_CONSUMPTION_QUERY_REQUIRED",
                "query_text is required",
            );
        };
        let Some(binding_id) = object_id_field(&document, "skill_binding_id") else {
            return error(
                400,
                "RESOURCE_SKILL_BINDING_ID_INVALID",
                "skill_binding_id is required",
            );
        };
        let contract_epoch = match store.current_contract_epoch(&task_reference) {
            Ok(epoch) if epoch > 0 => epoch,
            Ok(_) => {
                return error(
                    404,
                    "RESOURCE_TASK_NOT_FOUND",
                    "task has no current contract",
                );
            }
            Err(_) => {
                return error(
                    503,
                    "RESOURCE_CONSUMPTION_UNAVAILABLE",
                    "Task authority store is unavailable",
                );
            }
        };
        let contract = match store.load_task_contract(&task_reference, contract_epoch) {
            Ok(Some(contract)) => contract,
            Ok(None) => {
                return error(
                    404,
                    "RESOURCE_TASK_NOT_FOUND",
                    "task contract is unavailable",
                );
            }
            Err(_) => {
                return error(
                    503,
                    "RESOURCE_CONSUMPTION_UNAVAILABLE",
                    "Task authority store is unavailable",
                );
            }
        };
        let policy = match store.load_scheduler_execution_policy(&task_reference, contract_epoch) {
            Ok(Some(policy)) => policy,
            Ok(None) => {
                return error(
                    409,
                    "RESOURCE_TASK_POLICY_MISSING",
                    "task has no Context execution policy",
                );
            }
            Err(_) => {
                return error(
                    503,
                    "RESOURCE_CONSUMPTION_UNAVAILABLE",
                    "Task authority store is unavailable",
                );
            }
        };
        let Ok(policy_document) = serde_json::from_str::<Value>(&policy.canonical_json) else {
            return error(
                409,
                "RESOURCE_TASK_POLICY_INVALID",
                "task Context execution policy is malformed",
            );
        };
        let policy_task_reference = string_value_at(&policy_document, &["task_ref"]);
        let policy_contract_epoch = policy_document
            .get("contract_epoch")
            .and_then(Value::as_i64);
        let policy_context_request_id =
            string_value_at(&policy_document, &["context", "request_id"]);
        let Some(governance_scope) =
            string_value_at(&policy_document, &["context", "resource_scope_prefix"])
                .filter(|value| !value.is_empty())
        else {
            return error(
                409,
                "RESOURCE_TASK_POLICY_INVALID",
                "task Context scope is missing",
            );
        };
        if policy.task_ref != task_reference
            || policy.contract_epoch != contract_epoch
            || policy_task_reference != Some(task_reference.as_str())
            || policy_contract_epoch != Some(contract_epoch)
            || policy_context_request_id != Some(policy.context_request_id.as_str())
        {
            return error(
                409,
                "RESOURCE_TASK_POLICY_INVALID",
                "task Context execution policy does not match its authority binding",
            );
        }
        let context_request = match store.load_context_request(&policy.context_request_id) {
            Ok(Some(request)) => request,
            Ok(None) => {
                return error(
                    409,
                    "RESOURCE_TASK_CONTEXT_MISSING",
                    "task ContextRequest is unavailable",
                );
            }
            Err(_) => {
                return error(
                    503,
                    "RESOURCE_CONSUMPTION_UNAVAILABLE",
                    "Context authority store is unavailable",
                );
            }
        };
        if context_request.task_ref != task_reference {
            return error(
                409,
                "RESOURCE_TASK_CONTEXT_MISMATCH",
                "task ContextRequest does not match the current task contract",
            );
        }
        let now_unix_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs() as i64)
            .unwrap_or_default();
        let memory_candidates = match store.search_memory_candidates(&MemorySearchQuery {
            governance_scope: governance_scope.to_owned(),
            // Purpose is daemon-owned for this task consumption slice rather
            // than a client-selected retrieval filter.
            purpose: "task fact".to_owned(),
            observed_at_unix_seconds: now_unix_seconds,
            query_text,
            maximum_results: 8,
        }) {
            Ok(candidates) => candidates,
            Err(StorePortError::Conflict { .. }) => {
                return error(
                    409,
                    "RESOURCE_CONSUMPTION_CONFLICT",
                    "Memory eligibility conflicts with authority facts",
                );
            }
            Err(StorePortError::Unavailable { .. }) => {
                return error(
                    503,
                    "RESOURCE_CONSUMPTION_UNAVAILABLE",
                    "Memory authority store is unavailable",
                );
            }
        };
        let Some(binding) = (match store.load_active_skill_binding(&binding_id) {
            Ok(binding) => binding,
            Err(StorePortError::Conflict { .. }) => {
                return error(
                    409,
                    "RESOURCE_CONSUMPTION_CONFLICT",
                    "Skill eligibility conflicts with authority facts",
                );
            }
            Err(StorePortError::Unavailable { .. }) => {
                return error(
                    503,
                    "RESOURCE_CONSUMPTION_UNAVAILABLE",
                    "Skill authority store is unavailable",
                );
            }
        }) else {
            return error(
                404,
                "RESOURCE_SKILL_NOT_ELIGIBLE",
                "Skill binding is not active or has been revoked",
            );
        };
        let task_binding_matches = (binding.target_kind == "task"
            && binding.target_ref == task_reference)
            || (binding.target_kind == "workspace" && binding.target_ref == governance_scope);
        if binding.workspace_scope != governance_scope || !task_binding_matches {
            return error(
                403,
                "RESOURCE_SKILL_SCOPE_MISMATCH",
                "Skill binding is outside the task workspace scope",
            );
        }
        let memory_rows: Vec<Value> = memory_candidates
            .iter()
            .map(|candidate| {
                json!({
                    "memory_id": candidate.memory_id.to_string(),
                    "source_id": candidate.source_id.to_string(),
                    "source_digest": candidate.source_digest,
                })
            })
            .collect();
        let explanation = match store.explain_skill_binding(&binding_id) {
            Ok(Some(explanation)) => explanation,
            Ok(None) => {
                return error(
                    404,
                    "RESOURCE_SKILL_NOT_ELIGIBLE",
                    "Skill binding explanation is unavailable",
                );
            }
            Err(_) => {
                return error(
                    503,
                    "RESOURCE_CONSUMPTION_UNAVAILABLE",
                    "Skill authority store is unavailable",
                );
            }
        };
        json_response(
            200,
            json!({
                "kind": "task.resource.consumption",
                "authority_source": "daemon-memory-skill-stores",
                "task_ref": task_reference,
                "contract_epoch": contract_epoch,
                "contract_digest": contract.contract_digest,
                "context_request_id": policy.context_request_id.to_string(),
                "context_request_digest": context_request.request_digest,
                "memory": memory_rows,
                "skill": {
                    "binding_id": binding.binding_id.to_string(),
                    "revision_id": binding.revision_id.to_string(),
                    "package_id": explanation.package_id.to_string(),
                    "content_digest": explanation.content_digest,
                },
                "consumption_trace": {
                    "task_ref": task_reference,
                    "contract_epoch": contract_epoch,
                    "context_request_id": policy.context_request_id.to_string(),
                    "context_request_digest": context_request.request_digest,
                    "memory_count": memory_candidates.len(),
                    "skill_binding_id": binding.binding_id.to_string(),
                },
                "authority_side_effects": false,
            }),
        )
    }

    pub(crate) fn handle_authority(
        &self,
        method_path: &str,
        store: &SqliteAuthorityStore,
    ) -> ResourceApiResponse {
        let (_, query) = method_path
            .split_once('?')
            .map_or((method_path, ""), |(path, query)| (path, query));
        let Some(identifier) = query_parameter(query, "id") else {
            return error(
                400,
                "RESOURCE_OBJECT_ID_REQUIRED",
                "an authority object id is required",
            );
        };
        let Ok(object_id) = ObjectId::parse(identifier) else {
            return error(
                400,
                "RESOURCE_OBJECT_ID_INVALID",
                "authority object id is invalid",
            );
        };
        if method_path.starts_with("GET /management/resource/v1/memory/object") {
            return match store.load_memory_object(&object_id) {
                Ok(Some(object)) => json_response(
                    200,
                    json!({
                        "kind": "memory.explain",
                        "authority_source": "daemon-memory-store",
                        "memory": {
                            "memory_id": object.memory_id.to_string(),
                            "candidate_id": object.candidate_id.to_string(),
                            "decision_id": object.decision_id.to_string(),
                            "canonical_json": object.canonical_json,
                        },
                        "authority_side_effects": false,
                    }),
                ),
                Ok(None) => error(404, "RESOURCE_MEMORY_NOT_FOUND", "Memory object not found"),
                Err(_) => error(
                    503,
                    "RESOURCE_MEMORY_UNAVAILABLE",
                    "Memory authority store is unavailable",
                ),
            };
        }
        if method_path.starts_with("GET /management/resource/v1/skill/binding/explain")
            && query_parameter(query, "kind") == Some("revision")
        {
            return match store.load_skill_revision_payload(&object_id) {
                Ok(Some((content_digest, canonical_json))) => json_response(
                    200,
                    json!({
                        "kind": "skill.revision.inspect",
                        "authority_source": "daemon-skill-store",
                        "revision": {
                            "revision_id": object_id.to_string(),
                            "content_digest": content_digest,
                            "canonical_json": canonical_json,
                        },
                        "authority_side_effects": false,
                    }),
                ),
                Ok(None) => error(
                    404,
                    "RESOURCE_SKILL_REVISION_NOT_FOUND",
                    "Skill revision not found",
                ),
                Err(_) => error(
                    503,
                    "RESOURCE_SKILL_UNAVAILABLE",
                    "Skill authority store is unavailable",
                ),
            };
        }
        if method_path.starts_with("GET /management/resource/v1/skill/binding/explain") {
            return match store.explain_skill_binding(&object_id) {
                Ok(Some(explanation)) => json_response(
                    200,
                    json!({
                        "kind": "skill.binding.explain",
                        "authority_source": "daemon-skill-store",
                        "binding": {
                            "binding_id": explanation.binding.binding_id.to_string(),
                            "revision_id": explanation.binding.revision_id.to_string(),
                            "workspace_scope": explanation.binding.workspace_scope,
                            "target_kind": explanation.binding.target_kind,
                            "target_ref": explanation.binding.target_ref,
                            "status": explanation.binding.status,
                            "canonical_json": explanation.binding.canonical_json,
                            "package_id": explanation.package_id.to_string(),
                            "manifest_digest": explanation.manifest_digest,
                            "content_digest": explanation.content_digest,
                            "revocation_reason": explanation.revocation_reason,
                        },
                        "authority_side_effects": false,
                    }),
                ),
                Ok(None) => error(
                    404,
                    "RESOURCE_SKILL_BINDING_NOT_FOUND",
                    "Skill binding not found",
                ),
                Err(_) => error(
                    503,
                    "RESOURCE_SKILL_UNAVAILABLE",
                    "Skill authority store is unavailable",
                ),
            };
        }
        error(
            404,
            "RESOURCE_AUTHORITY_ROUTE_NOT_FOUND",
            "no authority-backed resource route matched",
        )
    }

    pub(crate) fn handle_authority_or_mutation(
        &self,
        method_path: &str,
        body: &[u8],
        store: &SqliteAuthorityStore,
    ) -> ResourceApiResponse {
        if method_path.starts_with("POST /management/resource/v1/memory/forget") {
            return self.forget_memory(body, store);
        }
        if method_path.starts_with("POST /management/resource/v1/memory/remember") {
            return self.remember_memory(body, store);
        }
        if method_path.starts_with("POST /management/resource/v1/skill/import") {
            return self.import_skill(body, store);
        }
        // `skill/bind` is a prefix of `skill/binding/revoke`, so the longer
        // route must be matched first or revoke is silently handled as a bind.
        if method_path.starts_with("POST /management/resource/v1/skill/binding/revoke") {
            return self.revoke_skill_binding(body, store);
        }
        if method_path.starts_with("POST /management/resource/v1/skill/bind") {
            return self.bind_skill(body, store);
        }
        self.handle_authority(method_path, store)
    }

    fn admit_context_source(
        &self,
        document: &Value,
        store: &SqliteAuthorityStore,
    ) -> Result<WorkspaceContextSourceRow, ResourceApiResponse> {
        let Some(header) = governed_header(document) else {
            return Err(error(
                400,
                "RESOURCE_CONTEXT_SOURCE_HEADER_INVALID",
                "Context source requires a valid governed header",
            ));
        };
        let Ok(source_id) = ObjectId::parse(&header.id.0) else {
            return Err(error(
                400,
                "RESOURCE_CONTEXT_SOURCE_ID_INVALID",
                "Context source header id is invalid",
            ));
        };
        let (
            Some(tenant_id),
            Some(owner_ref),
            Some(resource_scope),
            Some(provenance_ref),
            Some(role),
            Some(trust_level),
            Some(representation),
            Some(content_bytes),
        ) = (
            string_field(document, "tenant_id"),
            string_field(document, "owner_ref"),
            string_field(document, "resource_scope"),
            string_field(document, "provenance_ref"),
            context_role(document),
            context_trust_level(document),
            context_representation(document),
            integer_field(document, "content_bytes"),
        )
        else {
            return Err(error(
                400,
                "RESOURCE_CONTEXT_SOURCE_METADATA_INVALID",
                "Context source metadata is incomplete or invalid",
            ));
        };
        let source = WorkspaceContextSourceRow {
            source_id: source_id.clone(),
            source_digest: header.content_digest.0,
            governance: ObjectGovernance {
                object_ref: source_id.to_string(),
                tenant_id: Some(tenant_id),
                owner_ref,
                resource_scope,
                conversation_ref: string_field(document, "conversation_ref"),
            },
            role,
            trust_level,
            representation,
            provenance_ref,
            content_bytes,
            content_tokens: integer_field(document, "content_tokens"),
            canonical_json: document.to_string(),
        };
        match store.append_workspace_context_source(&source) {
            Ok(()) => Ok(source),
            Err(StorePortError::Conflict { .. }) => {
                match store.load_workspace_context_source_body(&source.source_id) {
                    Ok(Some(existing)) if existing == source => Ok(source),
                    Ok(_) => Err(error(
                        409,
                        "RESOURCE_CONTEXT_SOURCE_CONFLICT",
                        "Context source conflicts with existing authority facts",
                    )),
                    Err(_) => Err(error(
                        503,
                        "RESOURCE_CONTEXT_SOURCE_UNAVAILABLE",
                        "Context authority store is unavailable",
                    )),
                }
            }
            Err(StorePortError::Unavailable { .. }) => Err(error(
                503,
                "RESOURCE_CONTEXT_SOURCE_UNAVAILABLE",
                "Context authority store is unavailable",
            )),
        }
    }

    fn remember_memory(&self, body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
        let Ok(envelope) = serde_json::from_slice::<Value>(body) else {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "Memory remember payload is invalid",
            );
        };
        let (Some(source_document), Some(document)) =
            (envelope.get("source"), envelope.get("candidate").cloned())
        else {
            return error(
                400,
                "RESOURCE_MEMORY_PRECONDITION_INVALID",
                "Memory remember requires sealed source and candidate members",
            );
        };
        let Some(header) = governed_header(&document).filter(|header| {
            header.r#type == "MemoryCandidate" && header.schema_version == "cognitiveos.memory/0.1"
        }) else {
            return error(
                400,
                "RESOURCE_MEMORY_HEADER_INVALID",
                "Memory remember requires a sealed MemoryCandidate",
            );
        };
        let Ok(candidate_id) = ObjectId::parse(&header.id.0) else {
            return error(
                400,
                "RESOURCE_MEMORY_ID_INVALID",
                "MemoryCandidate header id is invalid",
            );
        };
        let (
            Some(source_id),
            Some(source_digest),
            Some(source_provenance_ref),
            Some(governance_scope),
            Some(target_scope),
            Some(purpose),
            Some(retention_expires_at_unix_seconds),
            Some(observed_at_unix_seconds),
        ) = (
            object_id_field(&document, "source_id"),
            string_field(&document, "source_digest"),
            string_field(&document, "source_provenance_ref"),
            string_field(&document, "governance_scope"),
            string_field(&document, "target_scope"),
            string_field(&document, "purpose"),
            integer_field(&document, "retention_expires_at_unix_seconds"),
            integer_field(&document, "observed_at_unix_seconds"),
        )
        else {
            return error(
                400,
                "RESOURCE_MEMORY_PRECONDITION_INVALID",
                "MemoryCandidate source, scope, purpose, and retention bindings are required",
            );
        };
        let decision_id = match new_resource_object_id() {
            Ok(identifier) => identifier,
            Err(response) => return response,
        };
        let memory_id = match new_resource_object_id() {
            Ok(identifier) => identifier,
            Err(response) => return response,
        };
        let candidate = MemoryCandidateRow {
            candidate_id: candidate_id.clone(),
            candidate_digest: header.content_digest.0.clone(),
            source_id,
            source_digest,
            source_provenance_ref,
            governance_scope,
            target_scope,
            purpose,
            retention_expires_at_unix_seconds,
            observed_at_unix_seconds,
            canonical_json: document.to_string(),
        };
        let decision = MemoryAdmissionDecisionRow {
            decision_id: decision_id.clone(),
            candidate_id: candidate.candidate_id.clone(),
            candidate_digest: candidate.candidate_digest.clone(),
            decision: "admit".to_owned(),
            policy_version: 1,
            reason_codes_json: "[\"MEMORY_ADMISSION_ACCEPTED\"]".to_owned(),
            canonical_json: json!({
                "decision_id": decision_id.to_string(),
                "candidate_id": candidate.candidate_id.to_string(),
                "candidate_digest": candidate.candidate_digest,
                "decision": "admit",
                "policy_version": 1,
                "reason_codes": ["MEMORY_ADMISSION_ACCEPTED"],
            })
            .to_string(),
        };
        let object = MemoryObjectRow {
            memory_id: memory_id.clone(),
            candidate_id: candidate.candidate_id.clone(),
            decision_id: decision.decision_id.clone(),
            canonical_json: json!({
                "memory_id": memory_id.to_string(),
                "candidate_id": candidate.candidate_id.to_string(),
                "decision_id": decision.decision_id.to_string(),
            })
            .to_string(),
        };
        if let Err(response) = self.admit_context_source(source_document, store) {
            return response;
        }
        let policy = MemoryAdmissionPolicy {
            policy_version: 1,
            now_unix_seconds: now_unix_seconds(),
            maximum_retention_seconds: 31_536_000,
        };
        match admit_memory_candidate(store, &candidate, &decision, Some(&object), &policy) {
            Ok(outcome) => json_response(
                201,
                json!({
                    "status": "remembered",
                    "outcome": format!("{outcome:?}").to_lowercase(),
                    "candidate_id": candidate.candidate_id.to_string(),
                    "decision_id": decision.decision_id.to_string(),
                    "memory_id": object.memory_id.to_string(),
                    "source_id": candidate.source_id.to_string(),
                }),
            ),
            Err(StorePortError::Conflict { .. }) => error(
                409,
                "RESOURCE_MEMORY_CONFLICT",
                "Memory admission conflicts with existing authority facts",
            ),
            Err(StorePortError::Unavailable { .. }) => error(
                503,
                "RESOURCE_MEMORY_UNAVAILABLE",
                "Memory authority store is unavailable",
            ),
        }
    }

    fn import_skill(&self, body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
        let Ok(document) = serde_json::from_slice::<Value>(body) else {
            return error(
                400,
                "RESOURCE_SKILL_PAYLOAD_INVALID",
                "Skill import payload is invalid",
            );
        };
        if document.get("previous_revision_id").is_some() {
            return self.supersede_skill_revision(&document, store);
        }
        let (Some(package_id), Some(revision_id)) = (
            object_id_field(&document, "package_id"),
            object_id_field(&document, "revision_id"),
        ) else {
            return error(
                400,
                "RESOURCE_SKILL_ID_INVALID",
                "package_id and revision_id are required",
            );
        };
        let package = SkillPackageRow {
            package_id: package_id.clone(),
            workspace_scope: string_field(&document, "workspace_scope").unwrap_or_default(),
            local_source_path: string_field(&document, "local_source_path").unwrap_or_default(),
            provenance_ref: string_field(&document, "provenance_ref").unwrap_or_default(),
            manifest_digest: string_field(&document, "manifest_digest").unwrap_or_default(),
            canonical_json: document.to_string(),
        };
        let revision = SkillRevisionRow {
            revision_id,
            package_id,
            content_digest: string_field(&document, "content_digest").unwrap_or_default(),
            compatibility: string_field(&document, "compatibility")
                .unwrap_or_else(|| "compatible".to_owned()),
            canonical_json: document.to_string(),
        };
        match store.append_skill_import(&package, &revision) {
            Ok(()) => json_response(
                201,
                json!({"status":"imported", "package_id": package.package_id.to_string(), "revision_id": revision.revision_id.to_string()}),
            ),
            Err(StorePortError::Conflict { .. }) => error(
                409,
                "RESOURCE_SKILL_CONFLICT",
                "Skill import conflicts with existing authority facts",
            ),
            Err(StorePortError::Unavailable { .. }) => error(
                503,
                "RESOURCE_SKILL_UNAVAILABLE",
                "Skill authority store is unavailable",
            ),
        }
    }

    fn supersede_skill_revision(
        &self,
        document: &Value,
        store: &SqliteAuthorityStore,
    ) -> ResourceApiResponse {
        let (Some(previous_revision_id), Some(revision_id), Some(package_id)) = (
            object_id_field(document, "previous_revision_id"),
            object_id_field(document, "revision_id"),
            object_id_field(document, "package_id"),
        ) else {
            return error(
                400,
                "RESOURCE_SKILL_ID_INVALID",
                "previous_revision_id, revision_id, and package_id are required",
            );
        };
        let Some(content_digest) = string_field(document, "content_digest") else {
            return error(
                400,
                "RESOURCE_SKILL_DIGEST_REQUIRED",
                "replacement content_digest is required",
            );
        };
        let replacement = SkillRevisionRow {
            revision_id: revision_id.clone(),
            package_id,
            content_digest,
            compatibility: string_field(document, "compatibility")
                .unwrap_or_else(|| "compatible".to_owned()),
            canonical_json: document.to_string(),
        };
        let supersede = SkillRevisionSupersedeRequest {
            previous_revision_id,
            replacement,
            canonical_json: document.to_string(),
        };
        match store.append_skill_revision_supersede(&supersede) {
            Ok(()) => json_response(
                201,
                json!({
                    "status": "superseded",
                    "revision_id": revision_id.to_string(),
                    "supersedes_revision_id": supersede.previous_revision_id.to_string(),
                }),
            ),
            Err(StorePortError::Conflict { .. }) => error(
                409,
                "RESOURCE_SKILL_CONFLICT",
                "Skill revision supersede conflicts with authority facts",
            ),
            Err(StorePortError::Unavailable { .. }) => error(
                503,
                "RESOURCE_SKILL_UNAVAILABLE",
                "Skill authority store is unavailable",
            ),
        }
    }

    fn bind_skill(&self, body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
        let Ok(document) = serde_json::from_slice::<Value>(body) else {
            return error(
                400,
                "RESOURCE_SKILL_PAYLOAD_INVALID",
                "Skill binding payload is invalid",
            );
        };
        let (Some(binding_id), Some(revision_id)) = (
            object_id_field(&document, "binding_id"),
            object_id_field(&document, "revision_id"),
        ) else {
            return error(
                400,
                "RESOURCE_SKILL_ID_INVALID",
                "binding_id and revision_id are required",
            );
        };
        let binding = SkillBindingRow {
            binding_id,
            revision_id,
            workspace_scope: string_field(&document, "workspace_scope").unwrap_or_default(),
            target_kind: string_field(&document, "target_kind").unwrap_or_default(),
            target_ref: string_field(&document, "target_ref").unwrap_or_default(),
            status: "active".to_owned(),
            canonical_json: document.to_string(),
        };
        match store.append_skill_binding(&binding) {
            Ok(()) => json_response(
                201,
                json!({"status":"bound", "binding_id": binding.binding_id.to_string()}),
            ),
            Err(StorePortError::Conflict { .. }) => error(
                409,
                "RESOURCE_SKILL_CONFLICT",
                "Skill binding conflicts with existing authority facts",
            ),
            Err(StorePortError::Unavailable { .. }) => error(
                503,
                "RESOURCE_SKILL_UNAVAILABLE",
                "Skill authority store is unavailable",
            ),
        }
    }

    pub(crate) fn forget_memory(
        &self,
        body: &[u8],
        store: &SqliteAuthorityStore,
    ) -> ResourceApiResponse {
        let Ok(document) = serde_json::from_slice::<Value>(body) else {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "Memory forget payload is invalid",
            );
        };
        let Some(memory_id) = object_id_field(&document, "memory_id") else {
            return error(
                400,
                "RESOURCE_MEMORY_ID_INVALID",
                "memory_id is required and invalid",
            );
        };
        let lifecycle_id = match object_id_field(&document, "lifecycle_id") {
            Some(identifier) => identifier,
            None => match new_resource_object_id() {
                Ok(identifier) => identifier,
                Err(response) => return response,
            },
        };
        let Some(reason) = document
            .get("reason")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
        else {
            return error(
                400,
                "RESOURCE_MEMORY_REASON_REQUIRED",
                "forget reason is required",
            );
        };
        let occurred_at = document
            .get("occurred_at_unix_seconds")
            .and_then(Value::as_i64)
            .unwrap_or_else(now_unix_seconds);
        let canonical_json = document
            .get("canonical_json")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| document.to_string());
        let tombstone = MemoryTombstoneRow {
            lifecycle_id,
            memory_id,
            action: "forget".to_owned(),
            occurred_at_unix_seconds: occurred_at,
            reason: reason.to_owned(),
            canonical_json,
        };
        match store.append_memory_tombstone(&tombstone) {
            Ok(()) => json_response(
                201,
                json!({"status":"forgotten", "memory_id": tombstone.memory_id.to_string()}),
            ),
            Err(StorePortError::Conflict { .. }) => error(
                409,
                "RESOURCE_MEMORY_CONFLICT",
                "Memory forget conflicts with existing authority facts",
            ),
            Err(StorePortError::Unavailable { .. }) => error(
                503,
                "RESOURCE_MEMORY_UNAVAILABLE",
                "Memory authority store is unavailable",
            ),
        }
    }

    pub(crate) fn revoke_skill_binding(
        &self,
        body: &[u8],
        store: &SqliteAuthorityStore,
    ) -> ResourceApiResponse {
        let Ok(document) = serde_json::from_slice::<Value>(body) else {
            return error(
                400,
                "RESOURCE_SKILL_PAYLOAD_INVALID",
                "Skill revoke payload is invalid",
            );
        };
        let Some(binding_id) = object_id_field(&document, "binding_id") else {
            return error(
                400,
                "RESOURCE_SKILL_BINDING_ID_INVALID",
                "binding_id is required and invalid",
            );
        };
        let Some(revocation_id) = object_id_field(&document, "revocation_id") else {
            return error(
                400,
                "RESOURCE_SKILL_REVOCATION_ID_INVALID",
                "revocation_id is required and invalid",
            );
        };
        let Some(reason) = document
            .get("reason")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
        else {
            return error(
                400,
                "RESOURCE_SKILL_REASON_REQUIRED",
                "revocation reason is required",
            );
        };
        let canonical_json = document
            .get("canonical_json")
            .and_then(Value::as_str)
            .unwrap_or("{}");
        let revocation = SkillBindingRevocationRow {
            revocation_id,
            binding_id,
            reason: reason.to_owned(),
            canonical_json: canonical_json.to_owned(),
        };
        match store.append_skill_binding_revocation(&revocation) {
            Ok(()) => json_response(
                201,
                json!({"status":"revoked", "binding_id": revocation.binding_id.to_string()}),
            ),
            Err(StorePortError::Conflict { .. }) => error(
                409,
                "RESOURCE_SKILL_CONFLICT",
                "Skill revocation conflicts with existing authority facts",
            ),
            Err(StorePortError::Unavailable { .. }) => error(
                503,
                "RESOURCE_SKILL_UNAVAILABLE",
                "Skill authority store is unavailable",
            ),
        }
    }

    fn handle_projection(
        &self,
        method_path: &str,
        task_reference: Option<&str>,
    ) -> ResourceApiResponse {
        let (path, query) = method_path
            .split_once('?')
            .map_or((method_path, ""), |(path, query)| (path, query));
        let family = match query_parameter(query, "family") {
            Some(family) if RESOURCE_FAMILIES.contains(&family) => family,
            Some(_) => {
                return error(
                    400,
                    "RESOURCE_PROJECTION_FAMILY_INVALID",
                    "resource family must be one of memory, skill, tool, context, task, or runtime",
                );
            }
            None => "task",
        };
        if let Some(version) = query_parameter(query, "version")
            && version != "1"
        {
            return error(
                400,
                "RESOURCE_PROJECTION_VERSION_UNSUPPORTED",
                "private resource projection version 1 is required",
            );
        }
        if path == "GET /resource/v1/projection" {
            return json_response(
                200,
                snapshot(
                    family,
                    self.next_watch_sequence.saturating_sub(1),
                    task_reference,
                ),
            );
        }
        if path == "GET /resource/v1/watch" {
            return self.watch(family, query, task_reference);
        }
        error(
            404,
            "RESOURCE_PROJECTION_ROUTE_NOT_FOUND",
            "no private resource projection route matched",
        )
    }

    fn watch(
        &self,
        family: &str,
        query: &str,
        task_reference: Option<&str>,
    ) -> ResourceApiResponse {
        let requested_sequence = query_parameter(query, "resume_from")
            .map(str::parse::<u64>)
            .transpose()
            .ok()
            .flatten();
        let oldest_sequence = self
            .watch_events
            .iter()
            .find(|(_, event_family, _)| event_family == family)
            .map(|(sequence, _, _)| *sequence)
            .unwrap_or(self.next_watch_sequence);
        if requested_sequence.is_some_and(|sequence| sequence.saturating_add(1) < oldest_sequence) {
            return error(
                409,
                "RESOURCE_WATCH_RESUME_STALE",
                "requested resource projection cursor is no longer retained for this family",
            );
        }
        let mut frames = vec![format!(
            "event: snapshot\ndata: {}\n\n",
            snapshot(
                family,
                self.next_watch_sequence.saturating_sub(1),
                task_reference,
            )
        )];
        for (sequence, event_family, event) in
            self.watch_events
                .iter()
                .filter(|(sequence, event_family, _)| {
                    event_family == family
                        && requested_sequence.is_none_or(|resume| *sequence > resume)
                })
        {
            frames.push(format!(
                "id: {sequence}\nevent: delta\ndata: {}\n\n",
                json!({"kind":"delta", "family": event_family, "sequence": sequence, "event": event})
            ));
        }
        ResourceApiResponse {
            status: 200,
            body: frames.concat(),
            content_type: "text/event-stream",
        }
    }

    fn publish(&mut self, family: &str, kind: &str, body: Value) {
        self.watch_events.push_back((
            self.next_watch_sequence,
            family.to_owned(),
            json!({"kind": kind, "body": body}),
        ));
        self.next_watch_sequence = self.next_watch_sequence.saturating_add(1);
        if self.watch_events.len() > MAX_WATCH_EVENTS {
            self.watch_events.pop_front();
        }
    }
}

fn snapshot(family: &str, latest_sequence: u64, task_reference: Option<&str>) -> Value {
    json!({
        "kind": "snapshot",
        "projection_version": PROJECTION_VERSION,
        "family": family,
        "latest_sequence": latest_sequence,
        "task_ref": task_reference,
        "projection": family_projection(family),
    })
}

fn governed_header(document: &Value) -> Option<GovernedObjectHeader> {
    document
        .get("header")
        .cloned()
        .and_then(|header| serde_json::from_value(header).ok())
}

fn context_role(document: &Value) -> Option<LoadedContextItemRole> {
    match document.get("role").and_then(Value::as_str)? {
        "control" => Some(LoadedContextItemRole::Control),
        "authoritative_state" => Some(LoadedContextItemRole::AuthoritativeState),
        "evidence" => Some(LoadedContextItemRole::Evidence),
        "working" => Some(LoadedContextItemRole::Working),
        "untrusted_input" => Some(LoadedContextItemRole::UntrustedInput),
        _ => None,
    }
}

fn context_trust_level(document: &Value) -> Option<LoadedContextItemTrustLevel> {
    match document.get("trust_level").and_then(Value::as_str)? {
        "control" => Some(LoadedContextItemTrustLevel::Control),
        "authoritative" => Some(LoadedContextItemTrustLevel::Authoritative),
        "verified" => Some(LoadedContextItemTrustLevel::Verified),
        "untrusted" => Some(LoadedContextItemTrustLevel::Untrusted),
        _ => None,
    }
}

fn context_representation(document: &Value) -> Option<LoadedContextItemRepresentation> {
    match document.get("representation").and_then(Value::as_str)? {
        "structured" => Some(LoadedContextItemRepresentation::Structured),
        "text" => Some(LoadedContextItemRepresentation::Text),
        "binary_ref" => Some(LoadedContextItemRepresentation::BinaryRef),
        _ => None,
    }
}

fn new_resource_object_id() -> Result<ObjectId, ResourceApiResponse> {
    cognitive_kernel::ports::IdGenerator::next_uuid_v7(&UuidV7Generator)
        .ok()
        .and_then(|value| ObjectId::parse(&value).ok())
        .ok_or_else(|| {
            error(
                503,
                "RESOURCE_ID_GENERATION_FAILED",
                "daemon could not mint a lifecycle identity",
            )
        })
}

fn now_unix_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or_default()
}

fn family_projection(family: &str) -> Value {
    if family == "memory" {
        return json!({
            "family": family,
            "availability": "not-backed",
            "authority_source": "authority-service-not-yet-implemented",
            "lifecycle": {
                "remember": "/management/resource/v1/memory/remember",
                "remember_input": "sealed source + sealed candidate envelope",
                "review": "/management/resource/v1/memory/object?id={memory_id}",
                "forget": "/management/resource/v1/memory/forget",
            },
            "resources": [],
            "authority_side_effects": false,
        });
    }
    if family == "skill" {
        return json!({
            "family": family,
            "availability": "not-backed",
            "authority_source": "authority-service-not-yet-implemented",
            "lifecycle": {
                "import": "/management/resource/v1/skill/import",
                "inspect": "/management/resource/v1/skill/binding/explain?kind=revision&id={revision_id}",
                "bind": "/management/resource/v1/skill/bind",
                "supersede": "/management/resource/v1/skill/import",
                "revoke": "/management/resource/v1/skill/binding/revoke",
            },
            "resources": [],
            "authority_side_effects": false,
        });
    }
    if family == "tool" {
        let resources = BUILTIN_TOOL_CATALOG
            .iter()
            .map(|descriptor| {
                json!({
                    "operation_id": descriptor.operation_id,
                    "action": descriptor.action,
                    "descriptor_version": descriptor.descriptor_version,
                    "descriptor_digest": descriptor.descriptor_digest,
                    "risk": descriptor.risk,
                    "executor": descriptor.executor,
                    "required_capability": descriptor.required_capability,
                    "family": descriptor.family,
                    "availability": descriptor.availability,
                    // Registry availability is a descriptor fact; readiness is
                    // this daemon's assembled-executor fact. They are reported
                    // separately so an unimplemented family cannot read as
                    // executable.
                    "execution_readiness": cognitive_kernel::tool_registry::tool_execution_readiness(
                        descriptor,
                        &super::tool_executor::ASSEMBLED_EXECUTOR_FAMILIES,
                    ),
                    "input_limit_bytes": descriptor.input_limit_bytes,
                    "output_limit_bytes": descriptor.output_limit_bytes,
                })
            })
            .collect::<Vec<_>>();
        return json!({
            "family": family,
            "availability": "available",
            "authority_source": "daemon-native-tool-registry",
            "resources": resources,
            "authority_side_effects": false,
        });
    }
    let (availability, authority_source) = match family {
        "task" => ("available", "daemon-task-application-service"),
        "runtime" => ("available", "personal-daemon-runtime"),
        _ => ("not-backed", "authority-service-not-yet-implemented"),
    };
    json!({
        "family": family,
        "availability": availability,
        "authority_source": authority_source,
        "resources": [],
        "authority_side_effects": false,
    })
}

fn query_parameter<'query>(query: &'query str, name: &str) -> Option<&'query str> {
    query
        .split('&')
        .find_map(|pair| pair.strip_prefix(&format!("{name}=")))
        .and_then(|value| value.split_whitespace().next())
}

fn object_id_field(document: &Value, field_name: &str) -> Option<ObjectId> {
    document
        .get(field_name)
        .and_then(Value::as_str)
        .and_then(|value| ObjectId::parse(value).ok())
}

fn string_field(document: &Value, field_name: &str) -> Option<String> {
    document
        .get(field_name)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn string_value_at<'value>(document: &'value Value, path: &[&str]) -> Option<&'value str> {
    path.iter()
        .try_fold(document, |current, field_name| current.get(*field_name))
        .and_then(Value::as_str)
}

fn integer_field(document: &Value, field_name: &str) -> Option<i64> {
    document.get(field_name).and_then(Value::as_i64)
}

fn json_response(status: u16, body: Value) -> ResourceApiResponse {
    ResourceApiResponse {
        status,
        body: body.to_string(),
        content_type: "application/json",
    }
}

fn error(status: u16, code: &str, message: &str) -> ResourceApiResponse {
    json_response(
        status,
        json!({"status":"error", "code": code, "message": message}),
    )
}
