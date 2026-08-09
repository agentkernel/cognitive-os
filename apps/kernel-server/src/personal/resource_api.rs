//! Private, versioned resource projection for Personal daemon clients.
//!
//! This is deliberately not a public contract or a durable generic Resource
//! aggregate. It exposes the six fixed product families as daemon observations
//! and makes missing authority backends explicit rather than fabricating rows.

use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

use super::memory_admission::admit_memory_candidate;
use cognitive_domain::ObjectId;
use cognitive_kernel::BUILTIN_TOOL_CATALOG;
use cognitive_kernel::memory_admission::MemoryAdmissionPolicy;
use cognitive_kernel::ports::{
    MemoryAdmissionDecisionRow, MemoryCandidateRow, MemoryObjectRow, MemoryStore,
    MemoryTombstoneRow, SkillBindingRevocationRow, SkillBindingRow, SkillPackageRow,
    SkillRevisionRow, SkillStore, StorePortError,
};
use cognitive_store::SqliteAuthorityStore;
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
        if method_path.starts_with("POST /management/resource/v1/skill/bind") {
            return self.bind_skill(body, store);
        }
        if method_path.starts_with("POST /management/resource/v1/skill/binding/revoke") {
            return self.revoke_skill_binding(body, store);
        }
        self.handle_authority(method_path, store)
    }

    fn remember_memory(&self, body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
        let Ok(document) = serde_json::from_slice::<Value>(body) else {
            return error(
                400,
                "RESOURCE_MEMORY_PAYLOAD_INVALID",
                "Memory remember payload is invalid",
            );
        };
        let required_identifier = |name: &str| object_id_field(&document, name);
        let (Some(candidate_id), Some(source_id), Some(decision_id), Some(memory_id)) = (
            required_identifier("candidate_id"),
            required_identifier("source_id"),
            required_identifier("decision_id"),
            required_identifier("memory_id"),
        ) else {
            return error(
                400,
                "RESOURCE_MEMORY_ID_INVALID",
                "candidate_id, source_id, decision_id, and memory_id are required",
            );
        };
        let Some(candidate_digest) = string_field(&document, "candidate_digest") else {
            return error(
                400,
                "RESOURCE_MEMORY_DIGEST_REQUIRED",
                "candidate_digest is required",
            );
        };
        let Some(governance_scope) = string_field(&document, "governance_scope") else {
            return error(
                400,
                "RESOURCE_MEMORY_SCOPE_REQUIRED",
                "governance_scope is required",
            );
        };
        let Some(purpose) = string_field(&document, "purpose") else {
            return error(
                400,
                "RESOURCE_MEMORY_PURPOSE_REQUIRED",
                "purpose is required",
            );
        };
        let candidate = MemoryCandidateRow {
            candidate_id: candidate_id.clone(),
            candidate_digest: candidate_digest.clone(),
            source_id,
            source_digest: string_field(&document, "source_digest").unwrap_or_default(),
            source_provenance_ref: string_field(&document, "source_provenance_ref")
                .unwrap_or_default(),
            governance_scope,
            target_scope: string_field(&document, "target_scope").unwrap_or_default(),
            purpose,
            retention_expires_at_unix_seconds: integer_field(
                &document,
                "retention_expires_at_unix_seconds",
            )
            .unwrap_or_default(),
            observed_at_unix_seconds: integer_field(&document, "observed_at_unix_seconds")
                .unwrap_or_default(),
            canonical_json: document.to_string(),
        };
        let decision = MemoryAdmissionDecisionRow {
            decision_id,
            candidate_id: candidate.candidate_id.clone(),
            candidate_digest,
            decision: string_field(&document, "decision").unwrap_or_else(|| "admit".to_owned()),
            policy_version: integer_field(&document, "policy_version").unwrap_or(1),
            reason_codes_json: string_field(&document, "reason_codes_json")
                .unwrap_or_else(|| "[]".to_owned()),
            canonical_json: document.to_string(),
        };
        let object = MemoryObjectRow {
            memory_id,
            candidate_id: candidate.candidate_id.clone(),
            decision_id: decision.decision_id.clone(),
            canonical_json: document.to_string(),
        };
        let now_unix_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs() as i64)
            .unwrap_or_default();
        let policy = MemoryAdmissionPolicy {
            policy_version: decision.policy_version,
            now_unix_seconds,
            maximum_retention_seconds: 31_536_000,
        };
        match admit_memory_candidate(store, &candidate, &decision, Some(&object), &policy) {
            Ok(outcome) => json_response(
                201,
                json!({"status":"remembered", "outcome": format!("{outcome:?}").to_lowercase(), "memory_id": object.memory_id.to_string()}),
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
        let Some(lifecycle_id) = object_id_field(&document, "lifecycle_id") else {
            return error(
                400,
                "RESOURCE_MEMORY_LIFECYCLE_ID_INVALID",
                "lifecycle_id is required and invalid",
            );
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
            .unwrap_or(0);
        let canonical_json = document
            .get("canonical_json")
            .and_then(Value::as_str)
            .unwrap_or("{}");
        let tombstone = MemoryTombstoneRow {
            lifecycle_id,
            memory_id,
            action: "forget".to_owned(),
            occurred_at_unix_seconds: occurred_at,
            reason: reason.to_owned(),
            canonical_json: canonical_json.to_owned(),
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

fn family_projection(family: &str) -> Value {
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
