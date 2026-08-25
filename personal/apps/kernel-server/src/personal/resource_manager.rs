//! P8-T12 common Resource Manager envelope over six family stores.
//!
//! This is a management-only projection and dispatcher. It does not create a
//! public contract, a generic Resource table, or generic create/install/execute
//! /complete verbs. Watch remains on the existing `/resource/v1/watch` surface.

use cognitive_domain::ObjectId;
use cognitive_kernel::ports::{IntentChainStore, SkillStore, StorePortError};
use cognitive_kernel::tool_registry::BUILTIN_TOOL_CATALOG;
use cognitive_store::{PersonalDataLayout, SqliteAuthorityStore, UuidV7Generator};
use serde_json::{Value, json};

use super::resource_api::{ResourceApi, ResourceApiResponse};
use super::tool_lifecycle::{self, ToolLifecycleState};

const PROJECTION_VERSION: &str = "personal-resource-manager/1";
const LIST_LIMIT: usize = 64;
const LOCAL_OWNER: &str = "principal://local/owner";

const FORBIDDEN_GENERIC_OPS: [&str; 4] = ["create", "install", "execute", "complete"];

/// Route literals scanned by the handbook HTTP generator. Keep these exact.
const ROUTE_LITERALS: &[&str] = &[
    "GET /management/resource/v1/list",
    "GET /management/resource/v1/inspect",
    "POST /management/resource/v1/bind",
    "POST /management/resource/v1/unbind",
    "POST /management/resource/v1/enable",
    "POST /management/resource/v1/disable",
    "POST /management/resource/v1/revoke",
    "POST /management/resource/v1/create",
    "POST /management/resource/v1/install",
    "POST /management/resource/v1/execute",
    "POST /management/resource/v1/complete",
    "GET /task/resource/v1/list",
    "GET /task/resource/v1/inspect",
    "POST /task/resource/v1/bind",
    "POST /task/resource/v1/unbind",
    "POST /task/resource/v1/enable",
    "POST /task/resource/v1/disable",
    "POST /task/resource/v1/revoke",
    "POST /task/resource/v1/create",
    "POST /task/resource/v1/install",
    "POST /task/resource/v1/execute",
    "POST /task/resource/v1/complete",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Channel {
    Management,
    Task,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Operation {
    List,
    Inspect,
    Bind,
    Unbind,
    Enable,
    Disable,
    Revoke,
    ForbiddenGeneric,
}

/// True when this request is the common Resource Manager surface (not family
/// lifecycle paths such as `/skill/bind` or `/tool/enable`).
pub(crate) fn matches(method_path: &str) -> bool {
    parse_route(method_path).is_some()
}

pub(crate) fn is_task_channel(method_path: &str) -> bool {
    parse_route(method_path).is_some_and(|(channel, _, _)| channel == Channel::Task)
}

pub(crate) fn channel_forbidden() -> ResourceApiResponse {
    error(
        403,
        "RESOURCE_MANAGER_CHANNEL_FORBIDDEN",
        "Resource Manager operations are management-channel only",
    )
}

pub(crate) fn handle(
    method_path: &str,
    body: &[u8],
    layout: &PersonalDataLayout,
    store: &SqliteAuthorityStore,
    resource_api: &ResourceApi,
) -> ResourceApiResponse {
    let Some((channel, operation, _)) = parse_route(method_path) else {
        return error(
            404,
            "RESOURCE_MANAGER_ROUTE_NOT_FOUND",
            "no Resource Manager route matched",
        );
    };
    if channel == Channel::Task {
        return channel_forbidden();
    }
    match operation {
        Operation::ForbiddenGeneric => error(
            400,
            "RESOURCE_MANAGER_OPERATION_FORBIDDEN",
            "generic create, install, execute, and complete are not Resource Manager operations",
        ),
        Operation::List => list_family(method_path, layout, store),
        Operation::Inspect => inspect_one(method_path, layout, store),
        Operation::Bind
        | Operation::Unbind
        | Operation::Enable
        | Operation::Disable
        | Operation::Revoke => mutate(operation, body, layout, store, resource_api),
    }
}

fn parse_route(method_path: &str) -> Option<(Channel, Operation, &'static str)> {
    for literal in ROUTE_LITERALS {
        if method_path.starts_with(literal) {
            let channel = if literal.contains("/task/") {
                Channel::Task
            } else {
                Channel::Management
            };
            let op_name = literal.rsplit('/').next().unwrap_or_default();
            let operation = match op_name {
                "list" => Operation::List,
                "inspect" => Operation::Inspect,
                "bind" => Operation::Bind,
                "unbind" => Operation::Unbind,
                "enable" => Operation::Enable,
                "disable" => Operation::Disable,
                "revoke" => Operation::Revoke,
                name if FORBIDDEN_GENERIC_OPS.contains(&name) => Operation::ForbiddenGeneric,
                _ => continue,
            };
            return Some((channel, operation, *literal));
        }
    }
    None
}

fn list_family(
    method_path: &str,
    layout: &PersonalDataLayout,
    store: &SqliteAuthorityStore,
) -> ResourceApiResponse {
    let family = match required_family_query(method_path) {
        Ok(family) => family,
        Err(response) => return response,
    };
    match family.as_str() {
        "tool" => list_tools(layout),
        "memory" => list_memory(store),
        "skill" => list_skills(store),
        "task" => list_tasks(store),
        "context" => empty_projection("context", "projection-only"),
        "runtime" => empty_projection("runtime", "projection-only"),
        _ => unknown_family(),
    }
}

fn inspect_one(
    method_path: &str,
    layout: &PersonalDataLayout,
    store: &SqliteAuthorityStore,
) -> ResourceApiResponse {
    let family = match required_family_query(method_path) {
        Ok(family) => family,
        Err(response) => return response,
    };
    let Some(id) = query_parameter(method_path, "id").filter(|value| !value.is_empty()) else {
        return error(400, "RESOURCE_MANAGER_ID_REQUIRED", "inspect requires id");
    };
    match family.as_str() {
        "tool" => inspect_tool(layout, &id),
        "memory" => inspect_memory(store, &id),
        "skill" => inspect_skill(store, &id),
        "task" => inspect_task(store, &id),
        "context" | "runtime" => error(
            404,
            "RESOURCE_MANAGER_NOT_FOUND",
            "context and runtime have no authority-backed Resource Manager rows",
        ),
        _ => unknown_family(),
    }
}

fn mutate(
    operation: Operation,
    body: &[u8],
    layout: &PersonalDataLayout,
    store: &SqliteAuthorityStore,
    resource_api: &ResourceApi,
) -> ResourceApiResponse {
    let document = match serde_json::from_slice::<Value>(body) {
        Ok(Value::Object(map)) => Value::Object(map),
        Ok(_) | Err(_) => {
            return error(
                400,
                "RESOURCE_MANAGER_PAYLOAD_INVALID",
                "Resource Manager mutation payload must be a JSON object",
            );
        }
    };
    let Some(family) = string_field(&document, "family") else {
        return error(
            400,
            "RESOURCE_MANAGER_FAMILY_REQUIRED",
            "family is required",
        );
    };
    if !matches!(
        family.as_str(),
        "memory" | "skill" | "tool" | "context" | "task" | "runtime"
    ) {
        return unknown_family();
    }
    let Some(id) = string_field(&document, "id").filter(|value| !value.is_empty()) else {
        return error(400, "RESOURCE_MANAGER_ID_REQUIRED", "id is required");
    };
    let Some(expected_version) = integer_field(&document, "expected_version") else {
        return error(
            400,
            "RESOURCE_MANAGER_VERSION_INVALID",
            "expected_version is required and must be an integer",
        );
    };
    let Some(idempotency_key) = string_field(&document, "idempotency_key") else {
        return error(
            400,
            "RESOURCE_MANAGER_IDEMPOTENCY_REQUIRED",
            "idempotency_key is required",
        );
    };
    if idempotency_key.len() > 128
        || idempotency_key.to_ascii_lowercase().contains("sk-")
        || idempotency_key.to_ascii_lowercase().contains("api_key")
    {
        return error(
            400,
            "RESOURCE_MANAGER_IDEMPOTENCY_FORBIDDEN",
            "idempotency_key is empty, oversized, or secret-shaped",
        );
    }
    match (family.as_str(), operation) {
        ("skill", Operation::Bind) => mutate_skill_bind(
            document,
            &id,
            expected_version,
            &idempotency_key,
            store,
            resource_api,
        ),
        ("skill", Operation::Unbind | Operation::Revoke) => mutate_skill_revoke(
            document,
            &id,
            expected_version,
            &idempotency_key,
            store,
            resource_api,
        ),
        ("tool", Operation::Enable) => mutate_tool(
            layout,
            &id,
            expected_version,
            &idempotency_key,
            ToolLifecycleState::Enabled,
            "enable",
        ),
        ("tool", Operation::Disable) => mutate_tool(
            layout,
            &id,
            expected_version,
            &idempotency_key,
            ToolLifecycleState::Disabled,
            "disable",
        ),
        ("tool", Operation::Revoke) => mutate_tool(
            layout,
            &id,
            expected_version,
            &idempotency_key,
            ToolLifecycleState::Revoked,
            "revoke",
        ),
        _ => error(
            400,
            "RESOURCE_MANAGER_OPERATION_UNSUPPORTED",
            "this family does not support that common Resource Manager operation",
        ),
    }
}

fn mutate_skill_bind(
    document: Value,
    id: &str,
    expected_version: i64,
    idempotency_key: &str,
    store: &SqliteAuthorityStore,
    resource_api: &ResourceApi,
) -> ResourceApiResponse {
    let current = skill_object_version(store, id);
    if let Some(version) = current {
        if expected_version == version && version == 1 {
            return mutation_ok(
                "bind",
                "skill",
                id,
                version,
                idempotency_key,
                json!({"status":"bound"}),
            );
        }
        return version_stale();
    }
    if expected_version != 0 {
        return version_stale();
    }
    let Some(revision_id) = string_field(&document, "revision_id") else {
        return error(
            400,
            "RESOURCE_MANAGER_PAYLOAD_INVALID",
            "skill bind requires revision_id",
        );
    };
    let payload = json!({
        "binding_id": id,
        "revision_id": revision_id,
        "workspace_scope": string_field(&document, "workspace_scope").unwrap_or_default(),
        "target_kind": string_field(&document, "target_kind").unwrap_or_default(),
        "target_ref": string_field(&document, "target_ref").unwrap_or_default(),
    });
    let domain = resource_api.bind_skill(payload.to_string().as_bytes(), store);
    wrap_domain("bind", "skill", id, 1, idempotency_key, domain)
}

fn mutate_skill_revoke(
    document: Value,
    id: &str,
    expected_version: i64,
    idempotency_key: &str,
    store: &SqliteAuthorityStore,
    resource_api: &ResourceApi,
) -> ResourceApiResponse {
    let Some(current) = skill_object_version(store, id) else {
        return error(
            404,
            "RESOURCE_MANAGER_NOT_FOUND",
            "Skill binding was not found",
        );
    };
    if current == 2 {
        if expected_version == 2 {
            return mutation_ok(
                "revoke",
                "skill",
                id,
                2,
                idempotency_key,
                json!({"status":"revoked"}),
            );
        }
        return version_stale();
    }
    if expected_version != 1 {
        return version_stale();
    }
    let Some(reason) = string_field(&document, "reason").filter(|value| !value.is_empty()) else {
        return error(
            400,
            "RESOURCE_MANAGER_PAYLOAD_INVALID",
            "skill unbind/revoke requires reason",
        );
    };
    let revocation_id = match string_field(&document, "revocation_id") {
        Some(value) => value,
        None => match mint_object_id() {
            Ok(value) => value,
            Err(response) => return response,
        },
    };
    let payload = json!({
        "binding_id": id,
        "revocation_id": revocation_id,
        "reason": reason,
        "canonical_json": json!({"binding_id": id, "reason": reason}).to_string(),
    });
    let domain = resource_api.revoke_skill_binding(payload.to_string().as_bytes(), store);
    wrap_domain("revoke", "skill", id, 2, idempotency_key, domain)
}

fn mutate_tool(
    layout: &PersonalDataLayout,
    id: &str,
    expected_version: i64,
    idempotency_key: &str,
    next: ToolLifecycleState,
    operation: &str,
) -> ResourceApiResponse {
    let current = match tool_lifecycle::current_lifecycle_state(layout, id) {
        Ok(state) => state,
        Err(response) => {
            return ResourceApiResponse {
                status: response.status,
                body: response.body,
                content_type: "application/json",
            };
        }
    };
    let current_version = tool_lifecycle::lifecycle_object_version(current);
    if current == next {
        if expected_version == current_version {
            return mutation_ok(
                operation,
                "tool",
                id,
                current_version,
                idempotency_key,
                json!({"status": operation, "lifecycle": tool_lifecycle_name(next)}),
            );
        }
        return version_stale();
    }
    if expected_version != current_version {
        return version_stale();
    }
    let reconstructed = format!("POST /management/resource/v1/tool/{operation} ");
    let domain = tool_lifecycle::handle(
        &reconstructed,
        json!({"operation_id": id}).to_string().as_bytes(),
        layout,
        tool_lifecycle::ToolLifecycleChannel::Management,
    );
    wrap_tool_domain(operation, id, next, idempotency_key, domain)
}

fn wrap_domain(
    operation: &str,
    family: &str,
    id: &str,
    object_version: i64,
    idempotency_key: &str,
    domain: ResourceApiResponse,
) -> ResourceApiResponse {
    if domain.status >= 400 {
        return domain;
    }
    let domain_body = serde_json::from_str::<Value>(&domain.body).unwrap_or(Value::Null);
    mutation_ok(
        operation,
        family,
        id,
        object_version,
        idempotency_key,
        domain_body,
    )
}

fn wrap_tool_domain(
    operation: &str,
    id: &str,
    next: ToolLifecycleState,
    idempotency_key: &str,
    domain: tool_lifecycle::ToolLifecycleResponse,
) -> ResourceApiResponse {
    if domain.status >= 400 {
        return ResourceApiResponse {
            status: domain.status,
            body: domain.body,
            content_type: "application/json",
        };
    }
    let domain_body = serde_json::from_str::<Value>(&domain.body).unwrap_or(Value::Null);
    mutation_ok(
        operation,
        "tool",
        id,
        tool_lifecycle::lifecycle_object_version(next),
        idempotency_key,
        domain_body,
    )
}

fn list_tools(layout: &PersonalDataLayout) -> ResourceApiResponse {
    let mut resources = Vec::new();
    for descriptor in BUILTIN_TOOL_CATALOG.iter() {
        match tool_envelope(layout, &descriptor.operation_id) {
            Ok(envelope) => resources.push(envelope),
            Err(response) => return response,
        }
    }
    list_ok("tool", "daemon-native-tool-registry", resources, false)
}

fn inspect_tool(layout: &PersonalDataLayout, id: &str) -> ResourceApiResponse {
    if BUILTIN_TOOL_CATALOG
        .iter()
        .all(|descriptor| descriptor.operation_id != id)
    {
        return error(
            404,
            "RESOURCE_MANAGER_NOT_FOUND",
            "operation_id is not registered",
        );
    }
    match tool_envelope(layout, id) {
        Ok(envelope) => inspect_ok("tool", "daemon-native-tool-registry", envelope),
        Err(response) => response,
    }
}

fn tool_envelope(
    layout: &PersonalDataLayout,
    operation_id: &str,
) -> Result<Value, ResourceApiResponse> {
    let state =
        tool_lifecycle::current_lifecycle_state(layout, operation_id).map_err(|response| {
            ResourceApiResponse {
                status: response.status,
                body: response.body,
                content_type: "application/json",
            }
        })?;
    let descriptor = BUILTIN_TOOL_CATALOG
        .iter()
        .find(|item| item.operation_id == operation_id);
    let health = match state {
        ToolLifecycleState::Enabled => "enabled",
        ToolLifecycleState::Disabled => "disabled",
        ToolLifecycleState::Quarantined => "quarantined",
        ToolLifecycleState::Revoked => "revoked",
    };
    let allowed_actions = match state {
        ToolLifecycleState::Enabled => json!(["inspect", "disable", "revoke"]),
        ToolLifecycleState::Disabled => json!(["inspect", "enable", "revoke"]),
        ToolLifecycleState::Quarantined => json!(["inspect", "revoke"]),
        ToolLifecycleState::Revoked => json!(["inspect"]),
    };
    Ok(json!({
        "id": operation_id,
        "family": "tool",
        "object_version": tool_lifecycle::lifecycle_object_version(state),
        "projection_version": PROJECTION_VERSION,
        "health": health,
        "owner": LOCAL_OWNER,
        "scope": "native",
        "revision_digest": descriptor.map(|item| item.descriptor_digest.clone()),
        "revision_digest_unavailable_reason": Value::Null,
        "blocked_reason": if matches!(state, ToolLifecycleState::Enabled) { Value::Null } else { json!(health) },
        "allowed_actions": allowed_actions,
        "typed_bindings": [],
    }))
}

fn list_memory(store: &SqliteAuthorityStore) -> ResourceApiResponse {
    match store.list_non_tombstoned_memory_objects(LIST_LIMIT) {
        Ok((rows, truncated)) => {
            let resources = rows
                .into_iter()
                .map(|row| memory_envelope(&row.memory_id.to_string()))
                .collect();
            list_ok(
                "memory",
                "sqlite-authority-memory-objects",
                resources,
                truncated,
            )
        }
        Err(error) => store_unavailable(error),
    }
}

fn inspect_memory(store: &SqliteAuthorityStore, id: &str) -> ResourceApiResponse {
    let Ok(memory_id) = ObjectId::parse(id) else {
        return error(
            400,
            "RESOURCE_MANAGER_ID_REQUIRED",
            "memory id is not a valid object id",
        );
    };
    match store.load_non_tombstoned_memory_object(&memory_id) {
        Ok(Some(_)) => inspect_ok(
            "memory",
            "sqlite-authority-memory-objects",
            memory_envelope(id),
        ),
        Ok(None) => error(
            404,
            "RESOURCE_MANAGER_NOT_FOUND",
            "Memory object was not found or is tombstoned",
        ),
        Err(error) => store_unavailable(error),
    }
}

fn memory_envelope(id: &str) -> Value {
    json!({
        "id": id,
        "family": "memory",
        "object_version": 1,
        "projection_version": PROJECTION_VERSION,
        "health": "admitted",
        "owner": LOCAL_OWNER,
        "scope": "owner-local",
        "revision_digest": Value::Null,
        "revision_digest_unavailable_reason": "memory objects are admitted rows, not immutable package revisions",
        "blocked_reason": Value::Null,
        "allowed_actions": ["inspect"],
        "typed_bindings": [],
    })
}

fn list_skills(store: &SqliteAuthorityStore) -> ResourceApiResponse {
    match store.list_skill_bindings(LIST_LIMIT) {
        Ok((rows, truncated)) => {
            let resources = rows
                .into_iter()
                .map(|(binding, revoked)| skill_envelope(&binding.binding_id.to_string(), revoked))
                .collect();
            list_ok(
                "skill",
                "sqlite-authority-skill-bindings",
                resources,
                truncated,
            )
        }
        Err(error) => store_unavailable(error),
    }
}

fn inspect_skill(store: &SqliteAuthorityStore, id: &str) -> ResourceApiResponse {
    let Ok(binding_id) = ObjectId::parse(id) else {
        return error(
            400,
            "RESOURCE_MANAGER_ID_REQUIRED",
            "skill id is not a valid object id",
        );
    };
    match store.explain_skill_binding(&binding_id) {
        Ok(Some(explanation)) => inspect_ok(
            "skill",
            "sqlite-authority-skill-bindings",
            skill_envelope(id, explanation.revocation_reason.is_some()),
        ),
        Ok(None) => error(
            404,
            "RESOURCE_MANAGER_NOT_FOUND",
            "Skill binding was not found",
        ),
        Err(error) => store_unavailable(error),
    }
}

fn skill_envelope(id: &str, revoked: bool) -> Value {
    json!({
        "id": id,
        "family": "skill",
        "object_version": if revoked { 2 } else { 1 },
        "projection_version": PROJECTION_VERSION,
        "health": if revoked { "revoked" } else { "bound" },
        "owner": LOCAL_OWNER,
        "scope": "owner-local",
        "revision_digest": Value::Null,
        "revision_digest_unavailable_reason": "revision digest is on the Skill package, not the binding envelope",
        "blocked_reason": if revoked { json!("revoked") } else { Value::Null },
        "allowed_actions": if revoked { json!(["inspect"]) } else { json!(["inspect", "unbind", "revoke"]) },
        "typed_bindings": [{"kind": "skill_binding", "id": id}],
    })
}

fn skill_object_version(store: &SqliteAuthorityStore, id: &str) -> Option<i64> {
    let binding_id = ObjectId::parse(id).ok()?;
    let explanation = store.explain_skill_binding(&binding_id).ok().flatten()?;
    Some(if explanation.revocation_reason.is_some() {
        2
    } else {
        1
    })
}

fn list_tasks(store: &SqliteAuthorityStore) -> ResourceApiResponse {
    match store.list_current_task_contracts() {
        Ok(rows) => {
            let truncated = rows.len() > LIST_LIMIT;
            let resources = rows
                .into_iter()
                .take(LIST_LIMIT)
                .map(|row| task_envelope(&row.task_ref, row.contract_epoch, &row.contract_digest))
                .collect();
            list_ok(
                "task",
                "sqlite-authority-task-contracts",
                resources,
                truncated,
            )
        }
        Err(error) => store_unavailable(error),
    }
}

fn inspect_task(store: &SqliteAuthorityStore, id: &str) -> ResourceApiResponse {
    match store.list_current_task_contracts() {
        Ok(rows) => {
            if let Some(row) = rows.into_iter().find(|row| row.task_ref == id) {
                inspect_ok(
                    "task",
                    "sqlite-authority-task-contracts",
                    task_envelope(&row.task_ref, row.contract_epoch, &row.contract_digest),
                )
            } else {
                error(
                    404,
                    "RESOURCE_MANAGER_NOT_FOUND",
                    "Task contract was not found",
                )
            }
        }
        Err(error) => store_unavailable(error),
    }
}

fn task_envelope(task_ref: &str, epoch: i64, digest: &str) -> Value {
    json!({
        "id": task_ref,
        "family": "task",
        "object_version": epoch,
        "projection_version": PROJECTION_VERSION,
        "health": "contracted",
        "owner": LOCAL_OWNER,
        "scope": task_ref,
        "revision_digest": digest,
        "revision_digest_unavailable_reason": Value::Null,
        "blocked_reason": Value::Null,
        "allowed_actions": ["inspect"],
        "typed_bindings": [],
    })
}

fn empty_projection(family: &str, authority_source: &str) -> ResourceApiResponse {
    list_ok(family, authority_source, Vec::new(), false)
}

fn list_ok(
    family: &str,
    authority_source: &str,
    resources: Vec<Value>,
    truncated: bool,
) -> ResourceApiResponse {
    json_response(
        200,
        json!({
            "kind": "resource.manager.list",
            "schema_version": 1,
            "family": family,
            "projection_version": PROJECTION_VERSION,
            "authority_source": authority_source,
            "truncated": truncated,
            "resources": resources,
            "authority_side_effects": false,
        }),
    )
}

fn inspect_ok(family: &str, authority_source: &str, resource: Value) -> ResourceApiResponse {
    json_response(
        200,
        json!({
            "kind": "resource.manager.inspect",
            "schema_version": 1,
            "family": family,
            "projection_version": PROJECTION_VERSION,
            "authority_source": authority_source,
            "resource": resource,
            "authority_side_effects": false,
        }),
    )
}

fn mutation_ok(
    operation: &str,
    family: &str,
    id: &str,
    object_version: i64,
    idempotency_key: &str,
    domain: Value,
) -> ResourceApiResponse {
    json_response(
        200,
        json!({
            "kind": "resource.manager.mutation",
            "schema_version": 1,
            "operation": operation,
            "family": family,
            "id": id,
            "object_version": object_version,
            "projection_version": PROJECTION_VERSION,
            "idempotency_key": idempotency_key,
            "domain": domain,
            "authority_side_effects": true,
        }),
    )
}

fn required_family_query(method_path: &str) -> Result<String, ResourceApiResponse> {
    let Some(family) = query_parameter(method_path, "family").filter(|value| !value.is_empty())
    else {
        return Err(error(
            400,
            "RESOURCE_MANAGER_FAMILY_REQUIRED",
            "family query parameter is required",
        ));
    };
    if !matches!(
        family.as_str(),
        "memory" | "skill" | "tool" | "context" | "task" | "runtime"
    ) {
        return Err(unknown_family());
    }
    Ok(family)
}

fn unknown_family() -> ResourceApiResponse {
    error(
        400,
        "RESOURCE_MANAGER_FAMILY_UNKNOWN",
        "family must be memory|skill|tool|context|task|runtime",
    )
}

fn version_stale() -> ResourceApiResponse {
    error(
        409,
        "RESOURCE_MANAGER_VERSION_STALE",
        "expected_version does not match the current object version",
    )
}

fn store_unavailable(error: StorePortError) -> ResourceApiResponse {
    let _ = error;
    self::error(
        503,
        "RESOURCE_MANAGER_UNAVAILABLE",
        "authority store is unavailable",
    )
}

fn mint_object_id() -> Result<String, ResourceApiResponse> {
    cognitive_kernel::ports::IdGenerator::next_uuid_v7(&UuidV7Generator)
        .ok()
        .and_then(|value| ObjectId::parse(&value).ok())
        .map(|id| id.to_string())
        .ok_or_else(|| {
            error(
                503,
                "RESOURCE_MANAGER_UNAVAILABLE",
                "daemon could not mint a revocation identity",
            )
        })
}

fn query_parameter(method_path: &str, name: &str) -> Option<String> {
    let (_, query) = method_path.split_once('?')?;
    let query = query.trim();
    for pair in query.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        if key == name {
            return Some(percent_decode(
                value.trim_end_matches(|ch: char| ch.is_whitespace()),
            ));
        }
    }
    None
}

fn percent_decode(value: &str) -> String {
    let mut output = String::new();
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = &value[index + 1..index + 3];
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                output.push(char::from(byte));
                index += 3;
                continue;
            }
        }
        output.push(char::from(bytes[index]));
        index += 1;
    }
    output
}

fn string_field(document: &Value, name: &str) -> Option<String> {
    document
        .get(name)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn integer_field(document: &Value, name: &str) -> Option<i64> {
    match document.get(name) {
        Some(Value::Number(number)) => number.as_i64(),
        Some(Value::String(text)) => text.parse().ok(),
        _ => None,
    }
}

fn tool_lifecycle_name(state: ToolLifecycleState) -> &'static str {
    match state {
        ToolLifecycleState::Enabled => "enabled",
        ToolLifecycleState::Disabled => "disabled",
        ToolLifecycleState::Quarantined => "quarantined",
        ToolLifecycleState::Revoked => "revoked",
    }
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
