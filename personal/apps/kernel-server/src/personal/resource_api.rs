//! Private, versioned resource projection for Personal daemon clients.
//!
//! This is deliberately not a public contract or a durable generic Resource
//! aggregate. It exposes the six fixed product families as daemon observations
//! and makes missing authority backends explicit rather than fabricating rows.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use cognitive_contracts::generated::context_view::{
    LoadedContextItemRepresentation, LoadedContextItemRole, LoadedContextItemTrustLevel,
};
use cognitive_contracts::generated::governed_object_header::GovernedObjectHeader;
use cognitive_domain::{ObjectId, WallTimestamp};
use cognitive_kernel::BUILTIN_TOOL_CATALOG;
use cognitive_kernel::authz::ObjectGovernance;
use cognitive_kernel::intent_chain::{
    compose_governed_header, seal_governed_object_content_digest,
};
use cognitive_kernel::memory_admission::MemoryAdmissionPolicy;
use cognitive_kernel::memory_skill_consumption::{
    MemoryConsumptionPin, MemorySkillConsumptionStore, SkillConsumptionPin,
};
use cognitive_kernel::ports::{
    Clock, ContextStore, IntentChainStore, MemoryAdmissionDecisionRow, MemoryCandidateRow,
    MemoryObjectRow, MemorySearchQuery, MemoryStore, MemoryTombstoneRow, MemoryUpdateRequest,
    ProtocolStore, SchedulerExecutionPolicyStore, SkillBindingRevocationRow, SkillBindingRow,
    SkillPackageRow, SkillRevisionRow, SkillRevisionSupersedeRequest, SkillStore, StorePortError,
    WorkspaceContextSourceRow,
};
use cognitive_store::memory_store::KnowledgeMemoryStore;
use cognitive_store::vault::{VaultReadSpec, VaultStore};
use cognitive_store::{
    ConfirmCaller, EmployeeStore, EpisodicRecallSpec, ProjectAggregateError, SqliteAuthorityStore,
    SystemClock, UuidV7Generator, admit_memory_candidate, canonical_episodic_scope,
    forget_episodic_memory, load_memory_governance_scope, rebuild_episodic_memory_index,
    recall_episodic_memory, require_employee_in_project, screen_memory_admission,
};
use serde_json::{Value, json};

const PROJECTION_VERSION: &str = "personal-resource-projection/1";
const MAX_WATCH_EVENTS: usize = 128;
const RESOURCE_FAMILIES: [&str; 6] = ["memory", "skill", "tool", "context", "task", "runtime"];
const LOCAL_OWNER_PRINCIPAL: &str = "principal://local/owner";

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
    governance_data_dir: Option<PathBuf>,
}

impl ResourceApi {
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Self::with_governance_data_dir(None)
    }

    pub(crate) fn with_governance_data_dir(governance_data_dir: Option<PathBuf>) -> Self {
        let mut api = Self {
            next_watch_sequence: 1,
            watch_events: VecDeque::new(),
            governance_data_dir,
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

    pub(crate) fn handle_task_consumption_query(
        &self,
        method_path: &str,
        store: &SqliteAuthorityStore,
    ) -> ResourceApiResponse {
        let query = method_path
            .split_once('?')
            .map(|(_, rest)| rest)
            .unwrap_or("");
        if query_parameter(query, "query_text").is_some()
            || query_parameter(query, "skill_binding_id").is_some()
        {
            return error(
                400,
                "RESOURCE_CONSUMPTION_RESTATEMENT_FORBIDDEN",
                "session resume must read durable pins; query_text and skill_binding_id are restatement",
            );
        }
        let Some(encoded_task_ref) =
            query_parameter(query, "task_ref").filter(|value| !value.is_empty())
        else {
            return error(
                400,
                "RESOURCE_TASK_REFERENCE_REQUIRED",
                "task_ref is required",
            );
        };
        let task_reference = match percent_decode_query(encoded_task_ref) {
            Ok(value) if !value.is_empty() => value,
            Ok(_) | Err(()) => {
                return error(
                    400,
                    "RESOURCE_TASK_REFERENCE_REQUIRED",
                    "task_ref is required",
                );
            }
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
        let context_request_id = match context_request_id_from_contract(&contract) {
            Ok(identifier) => identifier,
            Err(response) => return response,
        };
        let context_request = match store.load_context_request(&context_request_id) {
            Ok(Some(request)) if request.task_ref == task_reference => request,
            Ok(Some(_)) => {
                return error(
                    409,
                    "RESOURCE_TASK_CONTEXT_MISMATCH",
                    "task ContextRequest does not match the current task contract",
                );
            }
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
        let record = match store.load_latest_memory_skill_consumption(
            &task_reference,
            contract_epoch,
            &context_request_id,
        ) {
            Ok(Some(record)) => record,
            Ok(None) => {
                return error(
                    404,
                    "RESOURCE_CONSUMPTION_NOT_FOUND",
                    "no durable Memory/Skill consumption record exists for this Task",
                );
            }
            Err(_) => {
                return error(
                    503,
                    "RESOURCE_CONSUMPTION_UNAVAILABLE",
                    "Memory/Skill consumption store is unavailable",
                );
            }
        };
        if record.context_request_digest != context_request.request_digest {
            return error(
                409,
                "RESOURCE_CONSUMPTION_NOT_ELIGIBLE",
                "durable Memory/Skill consumption request digest differs from the current request",
            );
        }
        if let Err(response) = revalidate_redacted_consumption(store, &record) {
            return response;
        }
        json_response(
            200,
            json!({
                "kind": "task.resource.consumption",
                "authority_source": "daemon-memory-skill-consumption",
                "task_ref": record.task_ref,
                "contract_epoch": record.contract_epoch,
                "context_request_id": record.context_request_id.to_string(),
                "context_request_digest": context_request.request_digest,
                "session_ref": record.session_ref,
                "reuse_of": record.reuse_of.as_ref().map(ObjectId::to_string),
                "decision_class": "authorized_exact_pin",
                "memory": record.memory.iter().map(redacted_memory_pin).collect::<Vec<_>>(),
                "skill": record.skill.iter().map(redacted_skill_pin).collect::<Vec<_>>(),
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
                    "RESOURCE_SKILL_NOT_ELIGIBLE",
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
        // Task-channel Memory mutation aliases exist as a documented surface and
        // must fail closed (N6). Literals stay here so handbook extraction sees them.
        if method_path.starts_with("POST /task/resource/v1/memory/remember")
            || method_path.starts_with("POST /task/resource/v1/memory/forget")
            || method_path.starts_with("POST /task/resource/v1/memory/recall")
            || method_path.starts_with("POST /task/resource/v1/memory/correct")
            || method_path.starts_with("POST /task/resource/v1/memory/index.rebuild")
            || method_path.starts_with("POST /task/resource/v1/memory/review")
            || method_path.starts_with("POST /task/resource/v1/memory/auto-admit.chat")
            || method_path.starts_with("POST /task/resource/v1/memory/promote.request")
            || method_path.starts_with("POST /task/resource/v1/memory/promote.confirm")
            || method_path.starts_with("GET /task/resource/v1/vault.labeled")
            || method_path.starts_with("GET /task/resource/v1/vault.documents")
            || method_path.starts_with("GET /task/resource/v1/memory/promotes")
        {
            return error(
                403,
                "RESOURCE_MEMORY_CHANNEL_FORBIDDEN",
                "Memory mutations are management-channel only",
            );
        }
        if method_path.starts_with("GET /management/resource/v1/vault.labeled") {
            return self.vault_labeled(method_path, store);
        }
        if method_path.starts_with("GET /management/resource/v1/vault.documents") {
            return self.vault_documents(method_path, store);
        }
        if method_path.starts_with("GET /management/resource/v1/memory/promotes") {
            return self.memory_promotes(method_path, store);
        }
        if method_path.starts_with("POST /management/resource/v1/memory/auto-admit.chat") {
            return self.auto_admit_chat(body, store);
        }
        if method_path.starts_with("POST /management/resource/v1/memory/promote.request") {
            return self.promote_request(body, store);
        }
        if method_path.starts_with("POST /management/resource/v1/memory/promote.confirm") {
            return self.promote_confirm(body, store);
        }
        if method_path.starts_with("POST /management/resource/v1/memory/forget") {
            return self.forget_memory(body, store);
        }
        if method_path.starts_with("POST /management/resource/v1/memory/remember") {
            return self.remember_memory(body, store);
        }
        if method_path.starts_with("POST /management/resource/v1/memory/recall") {
            return self.recall_memory(body, store);
        }
        if method_path.starts_with("POST /management/resource/v1/memory/correct") {
            return self.correct_memory(body, store);
        }
        if method_path.starts_with("POST /management/resource/v1/memory/index.rebuild") {
            return self.rebuild_memory_index(store);
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
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "Context source requires a valid governed header",
            ));
        };
        let Ok(source_id) = ObjectId::parse(&header.id.0) else {
            return Err(error(
                400,
                "RESOURCE_MEMORY_ID_INVALID",
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
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
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
                        "RESOURCE_MEMORY_CONFLICT",
                        "Context source conflicts with existing authority facts",
                    )),
                    Err(_) => Err(error(
                        503,
                        "RESOURCE_MEMORY_UNAVAILABLE",
                        "Context authority store is unavailable",
                    )),
                }
            }
            Err(StorePortError::Unavailable { .. }) => Err(error(
                503,
                "RESOURCE_MEMORY_UNAVAILABLE",
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
        let has_source = envelope.get("source").is_some();
        let has_candidate = envelope.get("candidate").is_some();
        let has_unsealed_text = public_remember_text(&envelope).is_some();
        if (has_source || has_candidate) && has_unsealed_text {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "Memory remember cannot mix a sealed envelope with unsealed public fields",
            );
        }
        if has_source && has_candidate {
            return self.remember_sealed_envelope(&envelope, store);
        }
        if has_source || has_candidate {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "Memory remember requires sealed source and candidate members",
            );
        }
        self.remember_unsealed_public(&envelope, store)
    }

    fn remember_unsealed_public(
        &self,
        document: &Value,
        store: &SqliteAuthorityStore,
    ) -> ResourceApiResponse {
        if document.get("header").is_some() {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "unsealed Memory remember must not include a caller-minted header",
            );
        }
        let Some(text) = public_remember_text(document) else {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "unsealed Memory remember requires text",
            );
        };
        if let Err(response) = screen_remember_payload(&text, document) {
            return response;
        }
        let project_id = string_field(document, "project_id");
        let employee_id = string_field(document, "employee_id");
        let (governance_scope, target_scope) = match (project_id, employee_id) {
            (Some(project_id), Some(employee_id)) => {
                if let Err(response) = require_scoped_employee(store, &project_id, &employee_id) {
                    return response;
                }
                let canonical = canonical_episodic_scope(&project_id, &employee_id);
                if let Some(provided) = string_field(document, "governance_scope")
                    && provided != canonical
                {
                    return error(
                        403,
                        "RESOURCE_MEMORY_SCOPE_FORBIDDEN",
                        "Memory governance_scope does not match the caller project/employee",
                    );
                }
                (canonical.clone(), canonical)
            }
            (None, None) => {
                let Some(governance_scope) = string_field(document, "governance_scope") else {
                    return error(
                        400,
                        "RESOURCE_MEMORY_PAYLOAD_INVALID",
                        "unsealed Memory remember requires governance_scope",
                    );
                };
                let target_scope = string_field(document, "target_scope")
                    .unwrap_or_else(|| governance_scope.clone());
                (governance_scope, target_scope)
            }
            _ => {
                return error(
                    400,
                    "RESOURCE_MEMORY_PAYLOAD_INVALID",
                    "project_id and employee_id must be supplied together",
                );
            }
        };
        let purpose =
            string_field(document, "purpose").unwrap_or_else(|| "task_execution".to_owned());
        let Some(retention_expires_at_unix_seconds) =
            integer_field(document, "retention_expires_at_unix_seconds")
        else {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "unsealed Memory remember requires retention_expires_at_unix_seconds",
            );
        };
        let provenance_ref = string_field(document, "provenance_ref")
            .unwrap_or_else(|| "management://personal/memory/remember".to_owned());
        let Some(data_dir) = self.governance_data_dir.as_ref() else {
            return error(
                503,
                "RESOURCE_MEMORY_GOVERNANCE_UNAVAILABLE",
                "daemon-owned governance root is unavailable",
            );
        };
        let seed = match super::task_api::personal_governance_seed(
            data_dir,
            LOCAL_OWNER_PRINCIPAL,
            vec!["memory_admission".to_owned()],
        ) {
            Ok(seed) => seed,
            Err(response) => {
                return error(
                    if response.status == 403 { 403 } else { 503 },
                    "RESOURCE_MEMORY_GOVERNANCE_UNAVAILABLE",
                    "daemon-owned governance root is unavailable",
                );
            }
        };
        let created_at = match SystemClock.now() {
            Ok(timestamp) => timestamp,
            Err(_) => {
                return error(
                    503,
                    "RESOURCE_MEMORY_UNAVAILABLE",
                    "daemon could not read the wall clock",
                );
            }
        };
        let source_id = match new_resource_object_id() {
            Ok(identifier) => identifier,
            Err(response) => return response,
        };
        let candidate_id = match new_resource_object_id() {
            Ok(identifier) => identifier,
            Err(response) => return response,
        };
        let content_bytes = i64::try_from(text.len()).unwrap_or(i64::MAX);
        let content_tokens = i64::try_from(text.split_whitespace().count()).unwrap_or(i64::MAX);
        let source_payload = json!({
            "tenant_id": "personal",
            "owner_ref": LOCAL_OWNER_PRINCIPAL,
            "resource_scope": governance_scope,
            "conversation_ref": null,
            "role": "working",
            "trust_level": "verified",
            "representation": "text",
            "provenance_ref": provenance_ref,
            "content_bytes": content_bytes,
            "content_tokens": content_tokens,
            "body": { "text": text },
        });
        let (source, source_digest) = match seal_public_governed_object(
            &source_id,
            "WorkspaceContextSource",
            "cognitiveos.workspace-context-source/0.1",
            source_payload,
            &seed,
            &created_at,
        ) {
            Ok(sealed) => sealed,
            Err(response) => return response,
        };
        let observed_at_unix_seconds = now_unix_seconds();
        let candidate_payload = json!({
            "source_id": source_id.to_string(),
            "source_digest": source_digest,
            "source_provenance_ref": provenance_ref,
            "governance_scope": governance_scope,
            "target_scope": target_scope,
            "purpose": purpose,
            "retention_expires_at_unix_seconds": retention_expires_at_unix_seconds,
            "observed_at_unix_seconds": observed_at_unix_seconds,
        });
        let (candidate, _) = match seal_public_governed_object(
            &candidate_id,
            "MemoryCandidate",
            "cognitiveos.memory/0.1",
            candidate_payload,
            &seed,
            &created_at,
        ) {
            Ok(sealed) => sealed,
            Err(response) => return response,
        };
        self.remember_sealed_envelope(&json!({"source": source, "candidate": candidate}), store)
    }

    fn remember_sealed_envelope(
        &self,
        envelope: &Value,
        store: &SqliteAuthorityStore,
    ) -> ResourceApiResponse {
        let (Some(source_document), Some(document)) =
            (envelope.get("source"), envelope.get("candidate").cloned())
        else {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "Memory remember requires sealed source and candidate members",
            );
        };
        let Some(header) = governed_header(&document).filter(|header| {
            header.r#type == "MemoryCandidate" && header.schema_version == "cognitiveos.memory/0.1"
        }) else {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
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
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "MemoryCandidate source, scope, purpose, and retention bindings are required",
            );
        };
        let source_text = source_document
            .pointer("/body/text")
            .and_then(Value::as_str)
            .unwrap_or("");
        if let Err(response) = screen_remember_payload(source_text, &document) {
            return response;
        }
        if let (Some(project_id), Some(employee_id)) = (
            string_field(&document, "project_id").or_else(|| string_field(envelope, "project_id")),
            string_field(&document, "employee_id")
                .or_else(|| string_field(envelope, "employee_id")),
        ) {
            if let Err(response) = require_scoped_employee(store, &project_id, &employee_id) {
                return response;
            }
            let canonical = canonical_episodic_scope(&project_id, &employee_id);
            if governance_scope != canonical {
                return error(
                    403,
                    "RESOURCE_MEMORY_SCOPE_FORBIDDEN",
                    "Memory governance_scope does not match the caller project/employee",
                );
            }
        } else if string_field(&document, "project_id").is_some()
            || string_field(&document, "employee_id").is_some()
            || string_field(envelope, "project_id").is_some()
            || string_field(envelope, "employee_id").is_some()
        {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "project_id and employee_id must be supplied together",
            );
        }
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
                "RESOURCE_SKILL_PAYLOAD_INVALID",
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

    pub(crate) fn bind_skill(
        &self,
        body: &[u8],
        store: &SqliteAuthorityStore,
    ) -> ResourceApiResponse {
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

    fn vault_labeled(
        &self,
        method_path: &str,
        store: &SqliteAuthorityStore,
    ) -> ResourceApiResponse {
        let Some(spec) = vault_read_from_path(method_path) else {
            return error(
                400,
                "RESOURCE_VAULT_PROJECT_REQUIRED",
                "project_id required",
            );
        };
        let vault = VaultStore::from_authority_store(store);
        match vault.read_labeled_index(&spec) {
            Ok(entries) => json_response(
                200,
                json!({
                    "status": "ok",
                    "is_authority": false,
                    "entries": entries.iter().map(|entry| json!({
                        "entry_id": entry.entry_id,
                        "document_id": entry.document_id,
                        "relative_path": entry.relative_path,
                        "excerpt": entry.excerpt,
                        "layer": entry.layer,
                        "provenance_source_uri": entry.provenance_source_uri,
                        "rights_class": entry.rights_class,
                        "freshness": entry.freshness,
                        "exclusion": entry.exclusion,
                        "exclusion_reason": entry.exclusion_reason,
                        "untrusted_observation": entry.untrusted_observation,
                        "is_authority": entry.is_authority,
                    })).collect::<Vec<_>>(),
                }),
            ),
            Err(error) => privacy_error(error),
        }
    }

    fn vault_documents(
        &self,
        method_path: &str,
        store: &SqliteAuthorityStore,
    ) -> ResourceApiResponse {
        let Some(spec) = vault_read_from_path(method_path) else {
            return error(
                400,
                "RESOURCE_VAULT_PROJECT_REQUIRED",
                "project_id required",
            );
        };
        let vault = VaultStore::from_authority_store(store);
        match vault.list_document_statuses(&spec) {
            Ok(rows) => json_response(
                200,
                json!({
                    "status": "ok",
                    "is_authority": false,
                    "documents": rows.iter().map(|row| json!({
                        "document_id": row.document_id,
                        "relative_path": row.relative_path,
                        "provenance_source_uri": row.provenance_source_uri,
                        "index_status": row.index_status,
                    })).collect::<Vec<_>>(),
                }),
            ),
            Err(error) => privacy_error(error),
        }
    }

    fn memory_promotes(
        &self,
        method_path: &str,
        store: &SqliteAuthorityStore,
    ) -> ResourceApiResponse {
        let query = method_path
            .split_once('?')
            .map(|(_, query)| query)
            .unwrap_or("");
        let Some(project_id) =
            query_parameter(query, "project_id").filter(|value| !value.is_empty())
        else {
            return error(
                400,
                "RESOURCE_MEMORY_PROJECT_REQUIRED",
                "project_id required",
            );
        };
        let knowledge = KnowledgeMemoryStore::from_authority_store(store);
        match knowledge.list_promotes(project_id) {
            Ok(rows) => json_response(
                200,
                json!({
                    "status": "ok",
                    "promotes": rows.iter().map(promote_json).collect::<Vec<_>>(),
                }),
            ),
            Err(error) => privacy_error(error),
        }
    }

    fn auto_admit_chat(&self, body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
        let Ok(document) = serde_json::from_slice::<Value>(body) else {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "Memory auto-admit payload is invalid",
            );
        };
        let (Some(projection_id), Some(project_id), Some(record_id)) = (
            string_field(&document, "projection_id"),
            string_field(&document, "project_id"),
            string_field(&document, "record_id"),
        ) else {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "auto-admit requires projection_id, project_id, and record_id",
            );
        };
        let knowledge = KnowledgeMemoryStore::from_authority_store(store);
        match knowledge.auto_admit_chat(
            ConfirmCaller::OwnerManagement,
            &projection_id,
            &project_id,
            &record_id,
            now_unix_seconds().saturating_mul(1000),
        ) {
            Ok(admitted) => json_response(
                201,
                json!({
                    "status": "admitted",
                    "memory_id": admitted.memory_id,
                    "inspectable": true,
                }),
            ),
            Err(error) => privacy_error(error),
        }
    }

    fn promote_request(&self, body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
        let Ok(document) = serde_json::from_slice::<Value>(body) else {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "Memory promote payload is invalid",
            );
        };
        let (Some(memory_id), Some(from_project_id), Some(to_project_id), Some(to_employee_id)) = (
            string_field(&document, "memory_id"),
            string_field(&document, "from_project_id"),
            string_field(&document, "to_project_id"),
            string_field(&document, "to_employee_id"),
        ) else {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "promote.request requires memory_id, from_project_id, to_project_id, to_employee_id",
            );
        };
        let knowledge = KnowledgeMemoryStore::from_authority_store(store);
        match knowledge.request_promote(
            ConfirmCaller::OwnerManagement,
            &memory_id,
            &from_project_id,
            &to_project_id,
            &to_employee_id,
            now_unix_seconds().saturating_mul(1000),
        ) {
            Ok(pending) => json_response(201, promote_json(&pending)),
            Err(error) => privacy_error(error),
        }
    }

    fn promote_confirm(&self, body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
        let Ok(document) = serde_json::from_slice::<Value>(body) else {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "Memory promote confirm payload is invalid",
            );
        };
        let (Some(promote_id), Some(preview_digest)) = (
            string_field(&document, "promote_id"),
            string_field(&document, "preview_digest"),
        ) else {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "promote.confirm requires promote_id and preview_digest",
            );
        };
        let knowledge = KnowledgeMemoryStore::from_authority_store(store);
        match knowledge.confirm_promote(
            ConfirmCaller::OwnerManagement,
            &promote_id,
            &preview_digest,
            now_unix_seconds().saturating_mul(1000),
        ) {
            Ok(confirmed) => json_response(200, promote_json(&confirmed)),
            Err(error) => privacy_error(error),
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
        let project_id = string_field(&document, "project_id");
        let employee_id = string_field(&document, "employee_id");
        let forget_result = match (project_id, employee_id) {
            (Some(project_id), Some(employee_id)) => {
                let employees = EmployeeStore::from_authority_store(store);
                forget_episodic_memory(store, &employees, &project_id, &employee_id, &tombstone)
                    .map_err(privacy_error)
            }
            (None, None) => store
                .append_memory_tombstone(&tombstone)
                .map_err(store_memory_error),
            _ => {
                return error(
                    400,
                    "RESOURCE_MEMORY_PAYLOAD_INVALID",
                    "project_id and employee_id must be supplied together",
                );
            }
        };
        match forget_result {
            Ok(()) => json_response(
                201,
                json!({"status":"forgotten", "memory_id": tombstone.memory_id.to_string()}),
            ),
            Err(response) => response,
        }
    }

    fn recall_memory(&self, body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
        let Ok(document) = serde_json::from_slice::<Value>(body) else {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "Memory recall payload is invalid",
            );
        };
        let (
            Some(caller_project_id),
            Some(target_project_id),
            Some(caller_employee_id),
            Some(target_employee_id),
            Some(query_text),
        ) = (
            string_field(&document, "caller_project_id"),
            string_field(&document, "target_project_id"),
            string_field(&document, "caller_employee_id"),
            string_field(&document, "target_employee_id"),
            string_field(&document, "query_text"),
        )
        else {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "Memory recall requires caller and target project/employee plus query_text",
            );
        };
        let purpose =
            string_field(&document, "purpose").unwrap_or_else(|| "task_execution".to_owned());
        let employees = EmployeeStore::from_authority_store(store);
        let spec = EpisodicRecallSpec {
            caller_project_id: &caller_project_id,
            target_project_id: &target_project_id,
            caller_employee_id: &caller_employee_id,
            target_employee_id: &target_employee_id,
            query_text: &query_text,
            purpose: &purpose,
            observed_at_unix_seconds: now_unix_seconds(),
            maximum_results: 32,
        };
        match recall_episodic_memory(store, &employees, &spec) {
            Ok(rows) => json_response(
                200,
                json!({
                    "status": "ok",
                    "candidates": rows.iter().map(|row| json!({
                        "memory_id": row.memory_id.to_string(),
                        "source_id": row.source_id.to_string(),
                        "source_digest": row.source_digest,
                    })).collect::<Vec<_>>(),
                }),
            ),
            Err(error) => privacy_error(error),
        }
    }

    fn rebuild_memory_index(&self, store: &SqliteAuthorityStore) -> ResourceApiResponse {
        match rebuild_episodic_memory_index(store) {
            Ok(()) => json_response(200, json!({"status": "rebuilt", "index": "memory_fts"})),
            Err(error) => privacy_error(error),
        }
    }

    fn correct_memory(&self, body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
        let Ok(document) = serde_json::from_slice::<Value>(body) else {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "Memory correct payload is invalid",
            );
        };
        let (Some(memory_id), Some(project_id), Some(employee_id), Some(text)) = (
            object_id_field(&document, "memory_id"),
            string_field(&document, "project_id"),
            string_field(&document, "employee_id"),
            public_remember_text(&document),
        ) else {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "Memory correct requires memory_id, project_id, employee_id, and text",
            );
        };
        if let Err(response) = screen_remember_payload(&text, &document) {
            return response;
        }
        if let Err(response) = require_scoped_employee(store, &project_id, &employee_id) {
            return response;
        }
        let canonical = canonical_episodic_scope(&project_id, &employee_id);
        match load_memory_governance_scope(store, memory_id.as_str()) {
            Ok(actual) if actual == canonical => {}
            Ok(_) => {
                return error(
                    403,
                    "RESOURCE_MEMORY_SCOPE_FORBIDDEN",
                    "Memory governance_scope does not match the caller project/employee",
                );
            }
            Err(error) => return privacy_error(error),
        }
        let Some(data_dir) = self.governance_data_dir.as_ref() else {
            return error(
                503,
                "RESOURCE_MEMORY_GOVERNANCE_UNAVAILABLE",
                "daemon-owned governance root is unavailable",
            );
        };
        let seed = match super::task_api::personal_governance_seed(
            data_dir,
            LOCAL_OWNER_PRINCIPAL,
            vec!["memory_admission".to_owned()],
        ) {
            Ok(seed) => seed,
            Err(response) => {
                return error(
                    if response.status == 403 { 403 } else { 503 },
                    "RESOURCE_MEMORY_GOVERNANCE_UNAVAILABLE",
                    "daemon-owned governance root is unavailable",
                );
            }
        };
        let created_at = match SystemClock.now() {
            Ok(timestamp) => timestamp,
            Err(_) => {
                return error(
                    503,
                    "RESOURCE_MEMORY_UNAVAILABLE",
                    "daemon could not read the wall clock",
                );
            }
        };
        let source_id = match new_resource_object_id() {
            Ok(identifier) => identifier,
            Err(response) => return response,
        };
        let candidate_id = match new_resource_object_id() {
            Ok(identifier) => identifier,
            Err(response) => return response,
        };
        let decision_id = match new_resource_object_id() {
            Ok(identifier) => identifier,
            Err(response) => return response,
        };
        let replacement_memory_id = match new_resource_object_id() {
            Ok(identifier) => identifier,
            Err(response) => return response,
        };
        let lifecycle_id = match new_resource_object_id() {
            Ok(identifier) => identifier,
            Err(response) => return response,
        };
        let content_bytes = i64::try_from(text.len()).unwrap_or(i64::MAX);
        let content_tokens = i64::try_from(text.split_whitespace().count()).unwrap_or(i64::MAX);
        let provenance_ref = "management://personal/memory/correct".to_owned();
        let source_payload = json!({
            "tenant_id": "personal",
            "owner_ref": LOCAL_OWNER_PRINCIPAL,
            "resource_scope": canonical,
            "conversation_ref": null,
            "role": "working",
            "trust_level": "verified",
            "representation": "text",
            "provenance_ref": provenance_ref,
            "content_bytes": content_bytes,
            "content_tokens": content_tokens,
            "body": { "text": text },
        });
        let (source, source_digest) = match seal_public_governed_object(
            &source_id,
            "WorkspaceContextSource",
            "cognitiveos.workspace-context-source/0.1",
            source_payload,
            &seed,
            &created_at,
        ) {
            Ok(sealed) => sealed,
            Err(response) => return response,
        };
        if let Err(response) = self.admit_context_source(&source, store) {
            return response;
        }
        let observed_at_unix_seconds = now_unix_seconds();
        let retention_expires_at_unix_seconds =
            integer_field(&document, "retention_expires_at_unix_seconds")
                .unwrap_or(observed_at_unix_seconds.saturating_add(31_536_000));
        let candidate_payload = json!({
            "source_id": source_id.to_string(),
            "source_digest": source_digest,
            "source_provenance_ref": provenance_ref,
            "governance_scope": canonical,
            "target_scope": canonical,
            "purpose": "task_execution",
            "retention_expires_at_unix_seconds": retention_expires_at_unix_seconds,
            "observed_at_unix_seconds": observed_at_unix_seconds,
        });
        let (candidate_document, candidate_digest) = match seal_public_governed_object(
            &candidate_id,
            "MemoryCandidate",
            "cognitiveos.memory/0.1",
            candidate_payload,
            &seed,
            &created_at,
        ) {
            Ok(sealed) => sealed,
            Err(response) => return response,
        };
        let expected_version = integer_field(&document, "expected_version").unwrap_or(1);
        let update = MemoryUpdateRequest {
            previous_memory_id: memory_id.clone(),
            expected_version,
            candidate: MemoryCandidateRow {
                candidate_id: candidate_id.clone(),
                candidate_digest: candidate_digest.clone(),
                source_id: source_id.clone(),
                source_digest,
                source_provenance_ref: provenance_ref,
                governance_scope: canonical.clone(),
                target_scope: canonical,
                purpose: "task_execution".to_owned(),
                retention_expires_at_unix_seconds,
                observed_at_unix_seconds,
                canonical_json: candidate_document.to_string(),
            },
            decision: MemoryAdmissionDecisionRow {
                decision_id: decision_id.clone(),
                candidate_id: candidate_id.clone(),
                candidate_digest,
                decision: "admit".to_owned(),
                policy_version: 1,
                reason_codes_json: "[\"MEMORY_UPDATE_ACCEPTED\"]".to_owned(),
                canonical_json: json!({
                    "decision_id": decision_id.to_string(),
                    "decision": "admit",
                    "reason_codes": ["MEMORY_UPDATE_ACCEPTED"],
                })
                .to_string(),
            },
            replacement: MemoryObjectRow {
                memory_id: replacement_memory_id.clone(),
                candidate_id,
                decision_id,
                canonical_json: json!({
                    "memory_id": replacement_memory_id.to_string(),
                    "supersedes": memory_id.to_string(),
                })
                .to_string(),
            },
            supersede_tombstone: MemoryTombstoneRow {
                lifecycle_id,
                memory_id,
                action: "supersede".to_owned(),
                occurred_at_unix_seconds: observed_at_unix_seconds,
                reason: "owner corrected Memory".to_owned(),
                canonical_json: "{\"action\":\"supersede\"}".to_owned(),
            },
        };
        match store.append_memory_update(&update) {
            Ok(()) => json_response(
                201,
                json!({
                    "status": "corrected",
                    "memory_id": replacement_memory_id.to_string(),
                    "supersedes": update.previous_memory_id.to_string(),
                }),
            ),
            Err(error) => store_memory_error(error),
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

fn public_remember_text(document: &Value) -> Option<String> {
    if let Some(text) = string_field(document, "text") {
        return Some(text);
    }
    if let Some(text) = string_value_at(document, &["body", "text"]) {
        return Some(text.to_owned());
    }
    string_field(document, "body")
}

fn seal_public_governed_object(
    identifier: &ObjectId,
    object_type: &str,
    schema_version: &str,
    mut payload: Value,
    seed: &cognitive_kernel::intent_chain::GovernanceSeed,
    created_at: &WallTimestamp,
) -> Result<(Value, String), ResourceApiResponse> {
    let header = compose_governed_header(
        identifier,
        object_type,
        schema_version,
        seed,
        Vec::new(),
        Vec::new(),
        "personal-public-memory-remember",
        created_at,
    )
    .map_err(|_| {
        error(
            503,
            "RESOURCE_MEMORY_GOVERNANCE_UNAVAILABLE",
            "daemon could not compose a governed header",
        )
    })?;
    payload["header"] = serde_json::to_value(header).map_err(|_| {
        error(
            503,
            "RESOURCE_MEMORY_UNAVAILABLE",
            "daemon could not serialize a governed header",
        )
    })?;
    seal_governed_object_content_digest(payload).map_err(|_| {
        error(
            503,
            "RESOURCE_MEMORY_UNAVAILABLE",
            "daemon could not seal Memory content digest",
        )
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
                "RESOURCE_MEMORY_UNAVAILABLE",
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
                "remember_input": "unsealed public fields (daemon-composed sealed headers) or sealed source + sealed candidate envelope",
                "review": "/management/resource/v1/memory/object?id={memory_id}",
                "recall": "/management/resource/v1/memory/recall",
                "correct": "/management/resource/v1/memory/correct",
                "forget": "/management/resource/v1/memory/forget",
                "index_rebuild": "/management/resource/v1/memory/index.rebuild",
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

fn context_request_id_from_contract(
    contract: &cognitive_kernel::ports::TaskContractRow,
) -> Result<ObjectId, ResourceApiResponse> {
    let document: Value = serde_json::from_str(&contract.canonical_json).map_err(|_| {
        error(
            409,
            "RESOURCE_TASK_CONTEXT_MISSING",
            "task contract ContextRequest binding is malformed",
        )
    })?;
    let Some(identifier) = document
        .pointer("/context_request_ref/id")
        .and_then(Value::as_str)
    else {
        return Err(error(
            409,
            "RESOURCE_TASK_CONTEXT_MISSING",
            "task contract has no ContextRequest binding",
        ));
    };
    ObjectId::parse(identifier).map_err(|_| {
        error(
            409,
            "RESOURCE_TASK_CONTEXT_MISSING",
            "task contract ContextRequest id is invalid",
        )
    })
}

fn revalidate_redacted_consumption(
    store: &SqliteAuthorityStore,
    record: &cognitive_kernel::memory_skill_consumption::MemorySkillConsumptionRecord,
) -> Result<(), ResourceApiResponse> {
    let document: Value = serde_json::from_str(&record.canonical_json).map_err(|_| {
        error(
            503,
            "RESOURCE_CONSUMPTION_UNAVAILABLE",
            "durable Memory/Skill consumption payload is malformed",
        )
    })?;
    let Some(resource_scope) = document.get("resource_scope").and_then(Value::as_str) else {
        return Err(error(
            503,
            "RESOURCE_CONSUMPTION_UNAVAILABLE",
            "durable Memory/Skill consumption scope is missing",
        ));
    };
    let Some(purpose) = document.get("purpose").and_then(Value::as_str) else {
        return Err(error(
            503,
            "RESOURCE_CONSUMPTION_UNAVAILABLE",
            "durable Memory/Skill consumption purpose is missing",
        ));
    };
    let live_memory = match store.list_eligible_memory_pins(
        resource_scope,
        &record.task_ref,
        purpose,
        now_unix_seconds(),
    ) {
        Ok(rows) => rows,
        Err(_) => {
            return Err(error(
                503,
                "RESOURCE_CONSUMPTION_UNAVAILABLE",
                "Memory eligibility could not be revalidated",
            ));
        }
    };
    for pin in &record.memory {
        if !live_memory.iter().any(|live| live.pin == *pin) {
            return Err(error(
                409,
                "RESOURCE_CONSUMPTION_NOT_ELIGIBLE",
                "forgotten, expired, or digest-drifted Memory cannot be reused",
            ));
        }
    }
    let live_skill = match store.list_eligible_skill_pins(resource_scope, &record.task_ref) {
        Ok(rows) => rows,
        Err(_) => {
            return Err(error(
                503,
                "RESOURCE_CONSUMPTION_UNAVAILABLE",
                "Skill eligibility could not be revalidated",
            ));
        }
    };
    for pin in &record.skill {
        if !live_skill.iter().any(|live| {
            live.binding_id == pin.binding_id
                && live.revision_id == pin.revision_id
                && live.package_id == pin.package_id
                && live.content_digest == pin.content_digest
        }) {
            return Err(error(
                409,
                "RESOURCE_CONSUMPTION_NOT_ELIGIBLE",
                "revoked or digest-drifted Skill cannot be reused",
            ));
        }
    }
    Ok(())
}

fn redacted_memory_pin(pin: &MemoryConsumptionPin) -> Value {
    json!({
        "memory_id": pin.memory_id.to_string(),
        "source_id": pin.source_id.to_string(),
        "source_digest": pin.source_digest,
    })
}

fn redacted_skill_pin(pin: &SkillConsumptionPin) -> Value {
    json!({
        "binding_id": pin.binding_id.to_string(),
        "revision_id": pin.revision_id.to_string(),
        "package_id": pin.package_id.to_string(),
        "content_digest": pin.content_digest,
    })
}

fn percent_decode_query(value: &str) -> Result<String, ()> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err(());
            }
            let high = hex_nibble(bytes[index + 1])?;
            let low = hex_nibble(bytes[index + 2])?;
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).map_err(|_| ())
}

fn hex_nibble(byte: u8) -> Result<u8, ()> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(()),
    }
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

fn screen_remember_payload(text: &str, document: &Value) -> Result<(), ResourceApiResponse> {
    screen_memory_admission(text, &document.to_string()).map_err(privacy_error)
}

fn require_scoped_employee(
    store: &SqliteAuthorityStore,
    project_id: &str,
    employee_id: &str,
) -> Result<(), ResourceApiResponse> {
    let employees = EmployeeStore::from_authority_store(store);
    require_employee_in_project(&employees, project_id, employee_id).map_err(privacy_error)
}

fn vault_read_from_path(method_path: &str) -> Option<VaultReadSpec<'_>> {
    let query = method_path
        .split_once('?')
        .map(|(_, query)| query)
        .unwrap_or("");
    let project_id = query_parameter(query, "project_id").filter(|value| !value.is_empty())?;
    let caller = query_parameter(query, "caller_project_id")
        .filter(|value| !value.is_empty())
        .unwrap_or(project_id);
    Some(VaultReadSpec {
        caller_project_id: caller,
        target_project_id: project_id,
    })
}

fn promote_json(row: &cognitive_store::memory_store::MemoryPromoteRow) -> Value {
    json!({
        "status": row.status,
        "promote_id": row.promote_id,
        "memory_id": row.memory_id,
        "from_project_id": row.from_project_id,
        "to_project_id": row.to_project_id,
        "preview_digest": row.preview_digest,
        "promoted_memory_id": row.promoted_memory_id,
    })
}

fn privacy_error(cause: ProjectAggregateError) -> ResourceApiResponse {
    match cause {
        ProjectAggregateError::Forbidden { detail } => {
            error(403, "RESOURCE_MEMORY_SCOPE_FORBIDDEN", detail)
        }
        ProjectAggregateError::NotFound { detail } => {
            error(404, "RESOURCE_MEMORY_NOT_FOUND", detail)
        }
        ProjectAggregateError::Conflict { detail } => {
            error(409, "RESOURCE_MEMORY_CONFLICT", detail)
        }
        ProjectAggregateError::Invalid { detail } => {
            error(422, "RESOURCE_MEMORY_PRIVACY_REJECTED", detail)
        }
        ProjectAggregateError::Unavailable { .. } => error(
            503,
            "RESOURCE_MEMORY_UNAVAILABLE",
            "Memory authority store is unavailable",
        ),
        ProjectAggregateError::Stale { detail }
        | ProjectAggregateError::Unconfirmed { detail }
        | ProjectAggregateError::Rejected { detail } => {
            error(422, "RESOURCE_MEMORY_PRIVACY_REJECTED", detail)
        }
    }
}

fn store_memory_error(cause: StorePortError) -> ResourceApiResponse {
    match cause {
        StorePortError::Conflict { .. } => error(
            409,
            "RESOURCE_MEMORY_CONFLICT",
            "Memory admission conflicts with existing authority facts",
        ),
        StorePortError::Unavailable { .. } => error(
            503,
            "RESOURCE_MEMORY_UNAVAILABLE",
            "Memory authority store is unavailable",
        ),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod p11_t11_tests {
    use super::*;
    use cognitive_store::{
        ConfirmCaller, EmployeeStore, PersonalDataLayout, ProjectAggregateStore, RosterProposal,
        StageSpec, prepare_personal_databases,
    };
    use tempfile::TempDir;

    pub(crate) fn authority() -> (
        TempDir,
        SqliteAuthorityStore,
        ResourceApi,
        std::path::PathBuf,
    ) {
        let temporary = TempDir::new().expect("temp");
        let root = temporary.path();
        let layout = PersonalDataLayout::from_xdg_roots(
            root.join("config"),
            root.join("data"),
            root.join("state"),
            root.join("cache"),
            root.join("runtime"),
        );
        prepare_personal_databases(&layout).expect("prepare");
        let data_dir = layout.data_dir().to_path_buf();
        let store = SqliteAuthorityStore::open(&layout.authority_database_path()).expect("open");
        let api = ResourceApi::with_governance_data_dir(Some(data_dir.clone()));
        (temporary, store, api, data_dir)
    }

    fn stage(id: &str, title: &str, slot: &str) -> StageSpec {
        StageSpec {
            stage_id: id.to_owned(),
            title: title.to_owned(),
            objective: format!("{title} objective"),
            output_contract_digest: ProjectAggregateStore::digest_hex(
                format!("out-{id}").as_bytes(),
            ),
            acceptance_spec_ref: Some(format!("cas:spec-{id}")),
            cadence_json: Some(r#"{"kind":"manual"}"#.to_owned()),
            responsible_slot: slot.to_owned(),
            blocking_gap: None,
        }
    }

    pub(crate) fn activate_project(store: &SqliteAuthorityStore) -> String {
        let plane = ProjectAggregateStore::from_authority_store(store);
        let (draft_id, _) = plane.create_draft(b"charter-v1", 10).unwrap();
        plane
            .put_draft_charter(&draft_id, b"charter-body-v1", 11)
            .unwrap();
        let (preview_id, preview_digest) = plane
            .request_preview("activation", &draft_id, b"activation-preview", 12)
            .unwrap();
        plane
            .confirm_preview(
                ConfirmCaller::OwnerManagement,
                &preview_id,
                &preview_digest,
                13,
            )
            .unwrap()
            .new_ref
    }

    pub(crate) fn roster(store: &SqliteAuthorityStore, project_id: &str) -> Vec<String> {
        let projects = ProjectAggregateStore::from_authority_store(store);
        let employees = EmployeeStore::from_authority_store(store);
        let plan_id = projects
            .apply_plan_revision(
                project_id,
                project_id,
                &[
                    stage("s1", "Manage", "manager"),
                    stage("s2", "Research", "researcher"),
                ],
                20,
            )
            .unwrap();
        employees
            .register_roster(
                ConfirmCaller::OwnerManagement,
                project_id,
                &plan_id,
                &[
                    RosterProposal {
                        slot: "manager".to_owned(),
                        specialization: "project-manager".to_owned(),
                        prompt: "coordinate".to_owned(),
                        tools_declared: vec!["workspace-write".to_owned()],
                    },
                    RosterProposal {
                        slot: "researcher".to_owned(),
                        specialization: "member".to_owned(),
                        prompt: "file notes".to_owned(),
                        tools_declared: vec!["workspace-write".to_owned()],
                    },
                ],
                21,
            )
            .unwrap()
    }

    pub(crate) fn remember_body(project_id: &str, employee_id: &str, text: &str) -> String {
        // HTTP remember runs admit_memory_candidate with maximum_retention_seconds
        // 31_536_000. A far-future 4_000_000_000 expiry is a correct 409 policy
        // mismatch (Reject vs requested Admit), not a happy-path 201. P4 409
        // conflict checks stay on product paths; this is fixture setup only.
        json!({
            "text": text,
            "project_id": project_id,
            "employee_id": employee_id,
            "retention_expires_at_unix_seconds": now_unix_seconds() + 3_600,
        })
        .to_string()
    }

    #[test]
    fn p11_t11_scoped_recall_privacy_forget_and_task_channel() {
        let (_tmp, store, api, _data_dir) = authority();
        let project_a = activate_project(&store);
        let project_b = activate_project(&store);
        let ids_a = roster(&store, &project_a);
        let ids_b = roster(&store, &project_b);

        for path in [
            "POST /task/resource/v1/memory/remember",
            "POST /task/resource/v1/memory/forget",
            "POST /task/resource/v1/memory/recall",
            "POST /task/resource/v1/memory/correct",
            "POST /task/resource/v1/memory/index.rebuild",
            "POST /task/resource/v1/memory/review",
        ] {
            let task = api.handle_authority_or_mutation(
                path,
                remember_body(&project_a, &ids_a[1], "p11t11-task lantern hangs east").as_bytes(),
                &store,
            );
            assert_eq!(task.status, 403, "{path}: {}", task.body);
            assert!(
                task.body.contains("RESOURCE_MEMORY_CHANNEL_FORBIDDEN"),
                "{path}: {}",
                task.body
            );
        }

        let secret = api.handle_authority_or_mutation(
            "POST /management/resource/v1/memory/remember",
            remember_body(&project_a, &ids_a[1], "api_key=sk-p11t11-http-fixture").as_bytes(),
            &store,
        );
        assert_eq!(secret.status, 422, "{}", secret.body);

        let letta = api.handle_authority_or_mutation(
            "POST /management/resource/v1/memory/remember",
            json!({
                "text": "ordinary lantern",
                "project_id": project_a,
                "employee_id": ids_a[1],
                "engine": "letta",
                "admitted_by": "agent",
                "retention_expires_at_unix_seconds": 4_000_000_000i64,
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(letta.status, 422, "{}", letta.body);

        let remembered = api.handle_authority_or_mutation(
            "POST /management/resource/v1/memory/remember",
            remember_body(&project_a, &ids_a[1], "p11t11-scoped lantern hangs east").as_bytes(),
            &store,
        );
        assert_eq!(remembered.status, 201, "{}", remembered.body);
        let memory_id = serde_json::from_str::<Value>(&remembered.body)
            .unwrap()
            .get("memory_id")
            .and_then(Value::as_str)
            .unwrap()
            .to_owned();
        let viewed = api.handle_authority(
            &format!("GET /management/resource/v1/memory/object?id={memory_id}"),
            &store,
        );
        assert_eq!(viewed.status, 200, "{}", viewed.body);

        let same = api.handle_authority_or_mutation(
            "POST /management/resource/v1/memory/recall",
            json!({
                "caller_project_id": project_a,
                "target_project_id": project_a,
                "caller_employee_id": ids_a[1],
                "target_employee_id": ids_a[1],
                "query_text": "lantern",
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(same.status, 200, "{}", same.body);
        assert!(same.body.contains(&memory_id));

        let cross = api.handle_authority_or_mutation(
            "POST /management/resource/v1/memory/recall",
            json!({
                "caller_project_id": project_b,
                "target_project_id": project_a,
                "caller_employee_id": ids_b[1],
                "target_employee_id": ids_a[1],
                "query_text": "lantern",
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(cross.status, 403, "{}", cross.body);

        let forgotten = api.handle_authority_or_mutation(
            "POST /management/resource/v1/memory/forget",
            json!({
                "memory_id": memory_id,
                "project_id": project_a,
                "employee_id": ids_a[1],
                "reason": "owner forgot scoped Memory",
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(forgotten.status, 201, "{}", forgotten.body);
        let rebuilt = api.handle_authority_or_mutation(
            "POST /management/resource/v1/memory/index.rebuild",
            b"{}",
            &store,
        );
        assert_eq!(rebuilt.status, 200, "{}", rebuilt.body);
        let after = api.handle_authority_or_mutation(
            "POST /management/resource/v1/memory/recall",
            json!({
                "caller_project_id": project_a,
                "target_project_id": project_a,
                "caller_employee_id": ids_a[1],
                "target_employee_id": ids_a[1],
                "query_text": "lantern",
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(after.status, 200, "{}", after.body);
        assert!(!after.body.contains(&memory_id));
        assert!(
            after.body.contains("\"candidates\":[]") || after.body.contains("\"candidates\": []")
        );
    }

    #[test]
    fn p11_t11_management_correct_is_fail_closed_for_cross_scope_and_secret() {
        let (_tmp, store, api, _data_dir) = authority();
        let project_a = activate_project(&store);
        let project_b = activate_project(&store);
        let ids_a = roster(&store, &project_a);
        let ids_b = roster(&store, &project_b);

        let remembered = api.handle_authority_or_mutation(
            "POST /management/resource/v1/memory/remember",
            remember_body(&project_a, &ids_a[1], "p11t11-correct lantern hangs east").as_bytes(),
            &store,
        );
        assert_eq!(remembered.status, 201, "{}", remembered.body);
        let memory_id = serde_json::from_str::<Value>(&remembered.body)
            .unwrap()
            .get("memory_id")
            .and_then(Value::as_str)
            .unwrap()
            .to_owned();

        let secret = api.handle_authority_or_mutation(
            "POST /management/resource/v1/memory/correct",
            json!({
                "memory_id": memory_id,
                "project_id": project_a,
                "employee_id": ids_a[1],
                "text": "api_key=sk-p11t11-correct-fixture",
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(secret.status, 422, "{}", secret.body);
        assert!(secret.body.contains("RESOURCE_MEMORY_PRIVACY_REJECTED"));

        let cross = api.handle_authority_or_mutation(
            "POST /management/resource/v1/memory/correct",
            json!({
                "memory_id": memory_id,
                "project_id": project_b,
                "employee_id": ids_b[1],
                "text": "compass now points west",
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(cross.status, 403, "{}", cross.body);
        assert!(cross.body.contains("RESOURCE_MEMORY_SCOPE_FORBIDDEN"));
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod p13_t07_tests {
    use super::*;
    use cognitive_store::{
        ArchiveAppendSpec, CONVERSATION_ARCHIVE_PROJECTION_ID, ConversationStore, VaultImportSpec,
        VaultStore,
    };

    fn import_note(store: &SqliteAuthorityStore, project_id: &str, path: &str, body: &str) {
        VaultStore::from_authority_store(store)
            .import(
                ConfirmCaller::OwnerManagement,
                &VaultImportSpec {
                    project_id,
                    relative_path: path,
                    rights_class: "owner-owned",
                    provenance_json: r#"{"source_uri":"owner-paste:t07"}"#,
                    source_kind: "owner-paste",
                    body,
                    cas_ref: None,
                    conflict_policy: None,
                    now_ms: 9_000,
                },
            )
            .expect("import");
    }

    #[test]
    fn p13_t07_labeled_documents_and_task_aliases() {
        let (_tmp, store, api, _data_dir) = super::p11_t11_tests::authority();
        let project_id = super::p11_t11_tests::activate_project(&store);
        import_note(&store, &project_id, "notes/pending.md", "Stored pending.");
        let documents = api.handle_authority_or_mutation(
            &format!("GET /management/resource/v1/vault.documents?project_id={project_id}&caller_project_id={project_id}"),
            b"",
            &store,
        );
        assert_eq!(documents.status, 200, "{}", documents.body);
        assert!(documents.body.contains("not-indexed"), "{}", documents.body);
        VaultStore::from_authority_store(&store)
            .rebuild_index(ConfirmCaller::OwnerManagement, &project_id, 9_100)
            .expect("rebuild");
        let labeled = api.handle_authority_or_mutation(
            &format!("GET /management/resource/v1/vault.labeled?project_id={project_id}&caller_project_id={project_id}"),
            b"",
            &store,
        );
        assert_eq!(labeled.status, 200, "{}", labeled.body);
        assert!(labeled.body.contains("owner-owned"), "{}", labeled.body);
        assert!(
            labeled.body.contains("\"is_authority\":false"),
            "{}",
            labeled.body
        );
        let overreach = api.handle_authority_or_mutation(
            &format!("GET /management/resource/v1/vault.labeled?project_id={project_id}&caller_project_id=task://personal/other"),
            b"",
            &store,
        );
        assert_eq!(overreach.status, 403, "{}", overreach.body);
        for path in [
            "GET /task/resource/v1/vault.labeled",
            "GET /task/resource/v1/vault.documents",
            "GET /task/resource/v1/memory/promotes",
            "POST /task/resource/v1/memory/auto-admit.chat",
            "POST /task/resource/v1/memory/promote.request",
            "POST /task/resource/v1/memory/promote.confirm",
        ] {
            let task = api.handle_authority_or_mutation(path, b"{}", &store);
            assert_eq!(task.status, 403, "{path}: {}", task.body);
        }
    }

    #[test]
    fn p13_t07_promote_preview_then_confirm_on_management_http() {
        let (_tmp, store, api, _data_dir) = super::p11_t11_tests::authority();
        let from_project = super::p11_t11_tests::activate_project(&store);
        let to_project = super::p11_t11_tests::activate_project(&store);
        let from_ids = super::p11_t11_tests::roster(&store, &from_project);
        let to_ids = super::p11_t11_tests::roster(&store, &to_project);
        let remembered = api.handle_authority_or_mutation(
            "POST /management/resource/v1/memory/remember",
            super::p11_t11_tests::remember_body(
                &from_project,
                &from_ids[1],
                "p13t07 promote lantern hangs east",
            )
            .as_bytes(),
            &store,
        );
        assert_eq!(remembered.status, 201, "{}", remembered.body);
        let memory_id = serde_json::from_str::<Value>(&remembered.body)
            .unwrap()
            .get("memory_id")
            .and_then(Value::as_str)
            .unwrap()
            .to_owned();
        let pending = api.handle_authority_or_mutation(
            "POST /management/resource/v1/memory/promote.request",
            json!({
                "memory_id": memory_id,
                "from_project_id": from_project,
                "to_project_id": to_project,
                "to_employee_id": to_ids[1],
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(pending.status, 201, "{}", pending.body);
        assert!(
            pending.body.contains("\"status\":\"pending\""),
            "{}",
            pending.body
        );
        let pending_json = serde_json::from_str::<Value>(&pending.body).unwrap();
        let confirmed = api.handle_authority_or_mutation(
            "POST /management/resource/v1/memory/promote.confirm",
            json!({
                "promote_id": pending_json.get("promote_id").and_then(Value::as_str),
                "preview_digest": pending_json.get("preview_digest").and_then(Value::as_str),
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(confirmed.status, 200, "{}", confirmed.body);
        assert!(
            confirmed.body.contains("\"status\":\"confirmed\""),
            "{}",
            confirmed.body
        );
        let auto = api.handle_authority_or_mutation(
            "POST /management/resource/v1/memory/auto-admit.chat",
            json!({
                "projection_id": CONVERSATION_ARCHIVE_PROJECTION_ID,
                "project_id": from_project,
                "record_id": "missing-archive",
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(auto.status, 404, "{}", auto.body);
        let record_id = ConversationStore::from_authority_store(&store)
            .append(
                ConfirmCaller::OwnerManagement,
                &ArchiveAppendSpec {
                    projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
                    project_id: &from_project,
                    employee_id: &from_ids[1],
                    kind: "note",
                    body: "Archive note admitted over HTTP.",
                    now_ms: 10_000,
                },
            )
            .expect("archive");
        let admitted = api.handle_authority_or_mutation(
            "POST /management/resource/v1/memory/auto-admit.chat",
            json!({
                "projection_id": CONVERSATION_ARCHIVE_PROJECTION_ID,
                "project_id": from_project,
                "record_id": record_id,
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(admitted.status, 201, "{}", admitted.body);
        assert!(
            admitted.body.contains("\"inspectable\":true"),
            "{}",
            admitted.body
        );
    }
}
