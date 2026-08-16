//! P2-T25 default-on catalog overlay, Agent exposure, and bounded selection.
//!
//! Management callers mutate registered Tool lifecycle. Task-channel callers
//! may read the projection and record a least-set selection receipt. Overlay
//! state never enters the immutable descriptor digest.

use std::fs::{self, OpenOptions};
use std::io::Write;

use cognitive_kernel::tool_registry::{
    BUILTIN_TOOL_CATALOG, NativeToolDescriptor, ToolAvailability, ToolExecutionReadiness,
    tool_execution_readiness,
};
use cognitive_store::PersonalDataLayout;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::tool_executor::ASSEMBLED_EXECUTOR_FAMILIES;

const LIFECYCLE_FILE_NAME: &str = "personal-tool-lifecycle.json";
const LIFECYCLE_SCHEMA: &str = "cognitiveos.personal.tool-lifecycle/0.1";
const MAX_OVERLAYS: usize = 32;
const FORBIDDEN_SELECTION_KEYS: [&str; 6] = [
    "prompt",
    "body",
    "query_text",
    "receipt",
    "parameters",
    "skill_binding_id",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ToolLifecycleState {
    Enabled,
    Disabled,
    Quarantined,
    Revoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToolLifecycleChannel {
    Management,
    Task,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct OverlayRecord {
    operation_id: String,
    lifecycle: ToolLifecycleState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SelectionRecord {
    task_ref: String,
    candidate_set_digest: String,
    selected_operation_id: String,
    selected_descriptor_digest: String,
    selection_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct LifecycleFile {
    schema: String,
    overlays: Vec<OverlayRecord>,
    #[serde(default)]
    selections: Vec<SelectionRecord>,
}

pub(crate) struct ToolLifecycleResponse {
    pub status: u16,
    pub body: String,
}

pub(crate) fn handle(
    method_path: &str,
    body: &[u8],
    layout: &PersonalDataLayout,
    channel: ToolLifecycleChannel,
) -> ToolLifecycleResponse {
    let method_path = method_path.trim();
    let path = method_path
        .split_once('?')
        .map_or(method_path, |(path, _)| path);
    if channel == ToolLifecycleChannel::Task
        && (path.starts_with("POST /task/resource/v1/tool/enable")
            || path.starts_with("POST /task/resource/v1/tool/disable")
            || path.starts_with("POST /task/resource/v1/tool/quarantine")
            || path.starts_with("POST /task/resource/v1/tool/revoke")
            || path.starts_with("POST /task/resource/v1/tool/discover"))
    {
        return task_channel_mutation_forbidden();
    }
    if channel == ToolLifecycleChannel::Management
        && path.starts_with("POST /management/resource/v1/tool/selection")
    {
        return error(
            403,
            "RESOURCE_TOOL_SELECTION_CHANNEL_FORBIDDEN",
            "management callers cannot record Agent Tool selection receipts",
        );
    }
    if is_catalog_get(path)
        || path.starts_with("GET /management/resource/v1/tool/discover")
        || path.starts_with("GET /task/resource/v1/tool/discover")
    {
        return project_catalog(layout);
    }
    if path.starts_with("GET /task/resource/v1/tool/exposure")
        || path.starts_with("GET /management/resource/v1/tool/exposure")
    {
        return get_exposure(method_path, layout);
    }
    if path.starts_with("POST /management/resource/v1/tool/enable") {
        return mutate(body, layout, ToolLifecycleState::Enabled);
    }
    if path.starts_with("POST /management/resource/v1/tool/disable") {
        return mutate(body, layout, ToolLifecycleState::Disabled);
    }
    if path.starts_with("POST /management/resource/v1/tool/quarantine") {
        return mutate(body, layout, ToolLifecycleState::Quarantined);
    }
    if path.starts_with("POST /management/resource/v1/tool/revoke") {
        return mutate(body, layout, ToolLifecycleState::Revoked);
    }
    if path.starts_with("POST /task/resource/v1/tool/selection") {
        return record_selection(body, layout);
    }
    error(
        404,
        "RESOURCE_TOOL_LIFECYCLE_ROUTE_NOT_FOUND",
        "no tool lifecycle route matched",
    )
}

fn is_catalog_get(path: &str) -> bool {
    let catalog = path.starts_with("GET /management/resource/v1/tool")
        || path.starts_with("GET /task/resource/v1/tool");
    catalog
        && !path.starts_with("GET /management/resource/v1/tool/exposure")
        && !path.starts_with("GET /task/resource/v1/tool/exposure")
        && !path.starts_with("GET /management/resource/v1/tool/selection")
        && !path.starts_with("GET /task/resource/v1/tool/selection")
}

pub(crate) fn task_channel_mutation_forbidden() -> ToolLifecycleResponse {
    error(
        403,
        "RESOURCE_TOOL_LIFECYCLE_CHANNEL_FORBIDDEN",
        "ordinary task callers cannot mutate Tool lifecycle",
    )
}

fn project_catalog(layout: &PersonalDataLayout) -> ToolLifecycleResponse {
    let file = match load_file(layout) {
        Ok(file) => file,
        Err(response) => return response,
    };
    ok(json!({
        "kind": "tool.lifecycle.projection",
        "schema_version": 1,
        "authority_source": "daemon-native-tool-registry",
        "resources": catalog_resources(&file),
        "authority_side_effects": false,
    }))
}

fn get_exposure(method_path: &str, layout: &PersonalDataLayout) -> ToolLifecycleResponse {
    if forbidden_query_keys(method_path) {
        return error(
            400,
            "RESOURCE_TOOL_EXPOSURE_QUERY_FORBIDDEN",
            "tool exposure query cannot restate prompt, body, or receipt fields",
        );
    }
    let task_ref = match required_task_ref_query(method_path) {
        Ok(task_ref) => task_ref,
        Err(response) => return response,
    };
    let file = match load_file(layout) {
        Ok(file) => file,
        Err(response) => return response,
    };
    let exposure = agent_exposure(&file, &task_ref);
    let last_selection = file
        .selections
        .iter()
        .find(|record| record.task_ref == task_ref);
    ok(json!({
        "kind": "tool.agent.exposure",
        "schema_version": 1,
        "task_ref": task_ref,
        "exposure_digest": exposure.digest,
        "exposed": exposure.tools,
        "last_selection": last_selection,
        "authority_side_effects": false,
    }))
}

fn mutate(
    body: &[u8],
    layout: &PersonalDataLayout,
    next: ToolLifecycleState,
) -> ToolLifecycleResponse {
    let document = match parse_object(body, "RESOURCE_TOOL_LIFECYCLE_PAYLOAD_INVALID") {
        Ok(document) => document,
        Err(response) => return response,
    };
    if extra_keys(&document, &["operation_id"]) {
        return error(
            400,
            "RESOURCE_TOOL_LIFECYCLE_PAYLOAD_FORBIDDEN",
            "tool lifecycle mutation accepts only operation_id",
        );
    }
    let Some(operation_id) = document
        .get("operation_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return error(
            400,
            "RESOURCE_TOOL_OPERATION_ID_REQUIRED",
            "operation_id is required",
        );
    };
    let Some(descriptor) = catalog_descriptor(operation_id) else {
        return error(
            404,
            "RESOURCE_TOOL_UNKNOWN",
            "operation_id is not registered",
        );
    };
    let mut file = match load_file(layout) {
        Ok(file) => file,
        Err(response) => return response,
    };
    let current = overlay_state(&file, operation_id);
    if current == ToolLifecycleState::Revoked && next != ToolLifecycleState::Revoked {
        return error(
            409,
            "RESOURCE_TOOL_REVOKED",
            "a revoked Tool cannot be re-enabled, disabled, or quarantined",
        );
    }
    if current == ToolLifecycleState::Quarantined && next == ToolLifecycleState::Enabled {
        return error(
            409,
            "RESOURCE_TOOL_QUARANTINED",
            "a quarantined Tool cannot be enabled",
        );
    }
    upsert_overlay(&mut file, operation_id, next);
    if let Err(response) = persist_file(layout, &file) {
        return response;
    }
    let resources = catalog_resources(&file);
    let projected = resources
        .iter()
        .find(|resource| resource["operation_id"] == descriptor.operation_id)
        .cloned()
        .unwrap_or(json!({}));
    ToolLifecycleResponse {
        status: 200,
        body: json!({
            "kind": "tool.lifecycle.mutation",
            "schema_version": 1,
            "operation_id": operation_id,
            "lifecycle": next,
            "resource": projected,
            "authority_side_effects": true,
        })
        .to_string(),
    }
}

fn record_selection(body: &[u8], layout: &PersonalDataLayout) -> ToolLifecycleResponse {
    let document = match parse_object(body, "RESOURCE_TOOL_SELECTION_PAYLOAD_INVALID") {
        Ok(document) => document,
        Err(response) => return response,
    };
    if extra_keys(
        &document,
        &["task_ref", "operation_id", "candidate_set_digest"],
    ) || FORBIDDEN_SELECTION_KEYS
        .iter()
        .any(|key| document.get(*key).is_some())
    {
        return error(
            400,
            "RESOURCE_TOOL_SELECTION_QUERY_FORBIDDEN",
            "tool selection cannot restate prompt, body, receipt, or extra candidate fields",
        );
    }
    let Some(task_ref) = document
        .get("task_ref")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return error(
            400,
            "RESOURCE_TOOL_TASK_REF_REQUIRED",
            "task_ref is required",
        );
    };
    if cognitive_domain::UriRef::parse(task_ref).is_err() {
        return error(
            400,
            "RESOURCE_TOOL_TASK_REF_INVALID",
            "task_ref must be a canonical URI",
        );
    }
    let Some(operation_id) = document
        .get("operation_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return error(
            400,
            "RESOURCE_TOOL_OPERATION_ID_REQUIRED",
            "operation_id is required",
        );
    };
    let Some(candidate_set_digest) = document
        .get("candidate_set_digest")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return error(
            400,
            "RESOURCE_TOOL_CANDIDATE_SET_DIGEST_REQUIRED",
            "candidate_set_digest is required",
        );
    };
    let file = match load_file(layout) {
        Ok(file) => file,
        Err(response) => return response,
    };
    let exposure = agent_exposure(&file, task_ref);
    if candidate_set_digest != exposure.digest {
        return error(
            409,
            "RESOURCE_TOOL_SELECTION_EXPOSURE_MISMATCH",
            "candidate_set_digest must equal the current least Agent exposure",
        );
    }
    let Some(selected) = exposure
        .tools
        .iter()
        .find(|tool| tool["operation_id"] == operation_id)
    else {
        return error(
            403,
            "RESOURCE_TOOL_SELECTION_NOT_EXPOSED",
            "selected Tool is outside the current Agent exposure",
        );
    };
    let selected_descriptor_digest = selected["descriptor_digest"]
        .as_str()
        .unwrap_or("")
        .to_owned();
    let record = SelectionRecord {
        task_ref: task_ref.to_owned(),
        candidate_set_digest: exposure.digest.clone(),
        selected_operation_id: operation_id.to_owned(),
        selected_descriptor_digest: selected_descriptor_digest.clone(),
        selection_class: "selected".to_owned(),
    };
    let mut file = file;
    upsert_selection(&mut file, record.clone());
    if let Err(response) = persist_file(layout, &file) {
        return response;
    }
    ok(json!({
        "kind": "tool.selection.receipt",
        "schema_version": 1,
        "task_ref": record.task_ref,
        "candidate_set_digest": record.candidate_set_digest,
        "selected_operation_id": record.selected_operation_id,
        "selected_descriptor_digest": record.selected_descriptor_digest,
        "selection_class": record.selection_class,
        "authority_side_effects": true,
    }))
}

struct AgentExposure {
    digest: String,
    tools: Vec<Value>,
}

fn catalog_resources(file: &LifecycleFile) -> Vec<Value> {
    BUILTIN_TOOL_CATALOG
        .iter()
        .map(|descriptor| project_descriptor(file, descriptor))
        .collect()
}

fn project_descriptor(file: &LifecycleFile, descriptor: &NativeToolDescriptor) -> Value {
    let lifecycle = overlay_state(file, &descriptor.operation_id);
    let mut effective = descriptor.clone();
    effective.availability = match lifecycle {
        ToolLifecycleState::Enabled => ToolAvailability::Enabled,
        ToolLifecycleState::Disabled => ToolAvailability::Disabled,
        ToolLifecycleState::Quarantined | ToolLifecycleState::Revoked => {
            ToolAvailability::Quarantined
        }
    };
    let execution_readiness = tool_execution_readiness(&effective, &ASSEMBLED_EXECUTOR_FAMILIES);
    let agent_exposed = lifecycle == ToolLifecycleState::Enabled
        && execution_readiness == ToolExecutionReadiness::ExecutionReady;
    json!({
        "operation_id": descriptor.operation_id,
        "action": descriptor.action,
        "family": descriptor.family,
        "risk": descriptor.risk,
        "descriptor_version": descriptor.descriptor_version,
        "descriptor_digest": descriptor.descriptor_digest,
        "registered": true,
        "lifecycle": lifecycle,
        "execution_readiness": execution_readiness,
        "agent_exposed": agent_exposed,
    })
}

fn agent_exposure(file: &LifecycleFile, task_ref: &str) -> AgentExposure {
    let tools: Vec<Value> = catalog_resources(file)
        .into_iter()
        .filter(|resource| resource["agent_exposed"] == true)
        .collect();
    let mut hasher = Sha256::new();
    hasher.update(task_ref.as_bytes());
    hasher.update(b"\0least-exposure\0");
    for tool in &tools {
        hasher.update(tool["operation_id"].as_str().unwrap_or_default().as_bytes());
        hasher.update(b"\0");
        hasher.update(
            tool["descriptor_digest"]
                .as_str()
                .unwrap_or_default()
                .as_bytes(),
        );
        hasher.update(b"\0");
    }
    AgentExposure {
        digest: format!("{:x}", hasher.finalize()),
        tools,
    }
}

fn overlay_state(file: &LifecycleFile, operation_id: &str) -> ToolLifecycleState {
    file.overlays
        .iter()
        .find(|record| record.operation_id == operation_id)
        .map(|record| record.lifecycle)
        .unwrap_or(ToolLifecycleState::Enabled)
}

fn upsert_overlay(file: &mut LifecycleFile, operation_id: &str, lifecycle: ToolLifecycleState) {
    if let Some(existing) = file
        .overlays
        .iter_mut()
        .find(|record| record.operation_id == operation_id)
    {
        existing.lifecycle = lifecycle;
        return;
    }
    file.overlays.push(OverlayRecord {
        operation_id: operation_id.to_owned(),
        lifecycle,
    });
}

fn upsert_selection(file: &mut LifecycleFile, record: SelectionRecord) {
    if let Some(existing) = file
        .selections
        .iter_mut()
        .find(|item| item.task_ref == record.task_ref)
    {
        *existing = record;
        return;
    }
    file.selections.push(record);
}

fn catalog_descriptor(operation_id: &str) -> Option<&'static NativeToolDescriptor> {
    BUILTIN_TOOL_CATALOG
        .iter()
        .find(|descriptor| descriptor.operation_id == operation_id)
}

fn load_file(layout: &PersonalDataLayout) -> Result<LifecycleFile, ToolLifecycleResponse> {
    let path = layout.data_dir().join(LIFECYCLE_FILE_NAME);
    if !path.exists() {
        return Ok(LifecycleFile {
            schema: LIFECYCLE_SCHEMA.to_owned(),
            overlays: Vec::new(),
            selections: Vec::new(),
        });
    }
    let bytes = fs::read(&path).map_err(|_| {
        error(
            503,
            "RESOURCE_TOOL_LIFECYCLE_STORE_UNAVAILABLE",
            "tool lifecycle overlay cannot be read",
        )
    })?;
    let file: LifecycleFile = serde_json::from_slice(&bytes).map_err(|_| {
        error(
            503,
            "RESOURCE_TOOL_LIFECYCLE_STORE_UNAVAILABLE",
            "tool lifecycle overlay is malformed",
        )
    })?;
    if file.schema != LIFECYCLE_SCHEMA || file.overlays.len() > MAX_OVERLAYS {
        return Err(error(
            503,
            "RESOURCE_TOOL_LIFECYCLE_STORE_UNAVAILABLE",
            "tool lifecycle overlay schema is unsupported",
        ));
    }
    Ok(file)
}

fn persist_file(
    layout: &PersonalDataLayout,
    file: &LifecycleFile,
) -> Result<(), ToolLifecycleResponse> {
    let path = layout.data_dir().join(LIFECYCLE_FILE_NAME);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|_| {
            error(
                503,
                "RESOURCE_TOOL_LIFECYCLE_STORE_UNAVAILABLE",
                "tool lifecycle overlay directory cannot be created",
            )
        })?;
    }
    let bytes = serde_json::to_vec(file).map_err(|_| {
        error(
            503,
            "RESOURCE_TOOL_LIFECYCLE_STORE_UNAVAILABLE",
            "tool lifecycle overlay cannot be serialized",
        )
    })?;
    let temporary_path = path.with_extension("json.tmp");
    let _ = fs::remove_file(&temporary_path);
    let mut handle = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary_path)
        .map_err(|_| {
            error(
                503,
                "RESOURCE_TOOL_LIFECYCLE_STORE_UNAVAILABLE",
                "tool lifecycle overlay cannot be created",
            )
        })?;
    handle
        .write_all(&bytes)
        .and_then(|()| handle.sync_all())
        .map_err(|_| {
            error(
                503,
                "RESOURCE_TOOL_LIFECYCLE_STORE_UNAVAILABLE",
                "tool lifecycle overlay cannot be persisted",
            )
        })?;
    fs::rename(&temporary_path, path).map_err(|_| {
        error(
            503,
            "RESOURCE_TOOL_LIFECYCLE_STORE_UNAVAILABLE",
            "tool lifecycle overlay cannot be committed",
        )
    })
}

fn parse_object(body: &[u8], code: &str) -> Result<Value, ToolLifecycleResponse> {
    match serde_json::from_slice::<Value>(body) {
        Ok(Value::Object(map)) => Ok(Value::Object(map)),
        _ => Err(error(400, code, "JSON object payload is required")),
    }
}

fn extra_keys(document: &Value, allowed: &[&str]) -> bool {
    document
        .as_object()
        .map(|map| map.keys().any(|key| !allowed.contains(&key.as_str())))
        .unwrap_or(true)
}

fn forbidden_query_keys(method_path: &str) -> bool {
    let Some((_, query)) = method_path.split_once('?') else {
        return false;
    };
    query.split('&').any(|pair| {
        let key = pair.split_once('=').map_or(pair, |(key, _)| key);
        FORBIDDEN_SELECTION_KEYS.contains(&key)
    })
}

fn required_task_ref_query(method_path: &str) -> Result<String, ToolLifecycleResponse> {
    let query = method_path.split_once('?').map_or("", |(_, query)| query);
    let mut task_ref = None;
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if key == "task_ref" {
            task_ref = Some(value);
        } else if key != "task_ref" && !FORBIDDEN_SELECTION_KEYS.contains(&key) && key != "" {
            return Err(error(
                400,
                "RESOURCE_TOOL_EXPOSURE_QUERY_FORBIDDEN",
                "tool exposure query accepts only task_ref",
            ));
        }
    }
    let Some(encoded) = task_ref.filter(|value| !value.is_empty()) else {
        return Err(error(
            400,
            "RESOURCE_TOOL_TASK_REF_REQUIRED",
            "task_ref is required",
        ));
    };
    let decoded = percent_decode(encoded);
    if cognitive_domain::UriRef::parse(&decoded).is_err() {
        return Err(error(
            400,
            "RESOURCE_TOOL_TASK_REF_INVALID",
            "task_ref must be a canonical URI",
        ));
    }
    Ok(decoded)
}

fn percent_decode(value: &str) -> String {
    let mut bytes = Vec::new();
    let chars: Vec<char> = value.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '%' && index + 2 < chars.len() {
            let hex: String = chars[index + 1..index + 3].iter().collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                bytes.push(byte);
                index += 3;
                continue;
            }
        }
        if chars[index] == '+' {
            bytes.push(b' ');
        } else {
            let mut buffer = [0; 4];
            let encoded = chars[index].encode_utf8(&mut buffer);
            bytes.extend_from_slice(encoded.as_bytes());
        }
        index += 1;
    }
    String::from_utf8(bytes).unwrap_or_else(|_| value.to_owned())
}

fn ok(value: Value) -> ToolLifecycleResponse {
    ToolLifecycleResponse {
        status: 200,
        body: value.to_string(),
    }
}

fn error(status: u16, code: &str, message: &str) -> ToolLifecycleResponse {
    ToolLifecycleResponse {
        status,
        body: json!({"status":"error", "code": code, "message": message}).to_string(),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
mod tests {
    use super::*;
    use cognitive_store::PersonalDataLayout;

    fn layout() -> PersonalDataLayout {
        let root = std::env::temp_dir().join(format!(
            "cos-p2t25-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        PersonalDataLayout::from_xdg_roots(&root, &root, &root, &root, &root)
    }

    #[test]
    fn missing_overlay_exposes_enabled_execution_ready_tools() {
        let layout = layout();
        let response = project_catalog(&layout);
        assert_eq!(response.status, 200);
        let body: Value = serde_json::from_str(&response.body).expect("json");
        assert!(
            body["resources"].as_array().expect("resources").iter().all(
                |resource| resource["lifecycle"] == "enabled" && resource["registered"] == true
            )
        );
        assert!(
            body["resources"]
                .as_array()
                .expect("resources")
                .iter()
                .any(|resource| resource["agent_exposed"] == true)
        );
    }

    #[test]
    fn disable_drops_agent_exposure_and_rejects_stale_selection_digest() {
        let layout = layout();
        let disable = mutate(
            br#"{"operation_id":"native.workspace.read"}"#,
            &layout,
            ToolLifecycleState::Disabled,
        );
        assert_eq!(disable.status, 200, "{}", disable.body);
        let projection: Value =
            serde_json::from_str(&project_catalog(&layout).body).expect("projection");
        let read = projection["resources"]
            .as_array()
            .expect("resources")
            .iter()
            .find(|resource| resource["operation_id"] == "native.workspace.read")
            .expect("read");
        assert_eq!(read["lifecycle"], "disabled");
        assert_eq!(read["agent_exposed"], false);
        assert_eq!(read["execution_readiness"], "not_dispatchable");

        let before = agent_exposure(&load_file(&layout).expect("file"), "task://personal/one");
        let disable_write = mutate(
            br#"{"operation_id":"native.workspace.write"}"#,
            &layout,
            ToolLifecycleState::Disabled,
        );
        assert_eq!(disable_write.status, 200, "{}", disable_write.body);
        let stale = record_selection(
            &json!({
                "task_ref": "task://personal/one",
                "operation_id": "native.workspace.search",
                "candidate_set_digest": before.digest
            })
            .to_string()
            .into_bytes(),
            &layout,
        );
        assert_eq!(stale.status, 409, "{}", stale.body);
        assert!(
            stale
                .body
                .contains("RESOURCE_TOOL_SELECTION_EXPOSURE_MISMATCH")
        );
    }

    #[test]
    fn quarantine_blocks_enable_and_prompt_restatement_is_rejected() {
        let layout = layout();
        let quarantine = mutate(
            br#"{"operation_id":"native.http.fetch"}"#,
            &layout,
            ToolLifecycleState::Quarantined,
        );
        assert_eq!(quarantine.status, 200, "{}", quarantine.body);
        let enable = mutate(
            br#"{"operation_id":"native.http.fetch"}"#,
            &layout,
            ToolLifecycleState::Enabled,
        );
        assert_eq!(enable.status, 409, "{}", enable.body);
        assert!(enable.body.contains("RESOURCE_TOOL_QUARANTINED"));

        let exposure = get_exposure(
            "GET /task/resource/v1/tool/exposure?task_ref=task://personal/one",
            &layout,
        );
        let exposure_json: Value = serde_json::from_str(&exposure.body).expect("json");
        let digest = exposure_json["exposure_digest"].as_str().expect("digest");
        let restated = record_selection(
            &json!({
                "task_ref": "task://personal/one",
                "operation_id": "native.workspace.search",
                "candidate_set_digest": digest,
                "prompt": "use write"
            })
            .to_string()
            .into_bytes(),
            &layout,
        );
        assert_eq!(restated.status, 400, "{}", restated.body);
        assert!(
            restated
                .body
                .contains("RESOURCE_TOOL_SELECTION_QUERY_FORBIDDEN")
        );
    }
}
