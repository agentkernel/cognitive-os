//! P2-T25/D02 task/campaign-scoped pinned HTTPS origin registry.
//!
//! Default empty: production HttpFetchReadOnly stays fail-closed until a
//! management caller with an authorized campaign pins exact HTTPS origins.
//! Task-channel callers cannot mutate or inspect the registry. Pins never
//! carry credentials, headers, or request bodies.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use cognitive_kernel::valid_https_authority;
use cognitive_store::PersonalDataLayout;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const PINNED_HTTPS_FILE_NAME: &str = "personal-pinned-https.json";
const PINNED_HTTPS_SCHEMA: &str = "cognitiveos.personal.pinned-https/0.1";
const MAX_TASKS: usize = 32;
const MAX_ORIGINS: usize = 8;
const MAX_ORIGIN_CHARS: usize = 128;
const MAX_CAMPAIGN_ID_CHARS: usize = 32;
const FORBIDDEN_KEYS: [&str; 6] = [
    "prompt",
    "body",
    "query_text",
    "receipt",
    "parameters",
    "headers",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PinnedHttpsRecord {
    task_ref: String,
    campaign_id: String,
    origins: Vec<String>,
    methods: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PinnedHttpsFile {
    schema: String,
    pins: Vec<PinnedHttpsRecord>,
}

#[derive(Debug)]
pub(crate) struct PinnedHttpsResponse {
    pub status: u16,
    pub body: String,
}

pub(crate) fn handle(
    method_path: &str,
    body: &[u8],
    layout: &PersonalDataLayout,
) -> PinnedHttpsResponse {
    let method_path = method_path.trim();
    if method_path.starts_with("GET /management/resource/v1/http-origin") {
        return get_pin(method_path, layout);
    }
    if method_path.starts_with("POST /management/resource/v1/http-origin") {
        return put_pin(body, layout);
    }
    error(
        404,
        "RESOURCE_PINNED_HTTPS_ROUTE_NOT_FOUND",
        "no pinned HTTPS origin route matched",
    )
}

pub(crate) fn task_channel_forbidden() -> PinnedHttpsResponse {
    error(
        403,
        "RESOURCE_PINNED_HTTPS_CHANNEL_FORBIDDEN",
        "ordinary task callers cannot pin or inspect HTTPS origins",
    )
}

pub(crate) fn allowed_origins(data_dir: &Path, task_ref: &str) -> Vec<String> {
    let path = data_dir.join(PINNED_HTTPS_FILE_NAME);
    if !path.exists() {
        return Vec::new();
    }
    let Ok(file) = read_file(&path) else {
        return Vec::new();
    };
    file.pins
        .into_iter()
        .find(|pin| pin.task_ref == task_ref && campaign_is_authorized(&pin.campaign_id))
        .map(|pin| pin.origins)
        .unwrap_or_default()
}

fn get_pin(method_path: &str, layout: &PersonalDataLayout) -> PinnedHttpsResponse {
    if forbidden_query_keys(method_path) {
        return error(
            400,
            "RESOURCE_PINNED_HTTPS_QUERY_FORBIDDEN",
            "pinned HTTPS query accepts only task_ref",
        );
    }
    let task_ref = match required_task_ref_query(method_path) {
        Ok(task_ref) => task_ref,
        Err(response) => return response,
    };
    let pin = match load_pin(layout, &task_ref) {
        Ok(Some(pin)) => pin,
        Ok(None) => PinnedHttpsRecord {
            task_ref: task_ref.clone(),
            campaign_id: String::new(),
            origins: Vec::new(),
            methods: vec!["GET".to_owned(), "HEAD".to_owned()],
        },
        Err(response) => return response,
    };
    ok(json!({
        "schema_version": 1,
        "task_ref": pin.task_ref,
        "campaign_id": pin.campaign_id,
        "origins": pin.origins,
        "methods": pin.methods,
        "authority_side_effects": false,
    }))
}

fn put_pin(body: &[u8], layout: &PersonalDataLayout) -> PinnedHttpsResponse {
    let document = match parse_object(body) {
        Ok(document) => document,
        Err(response) => return response,
    };
    if extra_keys(
        &document,
        &["task_ref", "campaign_id", "origins", "methods"],
    ) {
        return error(
            400,
            "RESOURCE_PINNED_HTTPS_QUERY_FORBIDDEN",
            "pinned HTTPS pin refuses prompt, body, receipt, or header restatement",
        );
    }
    let Some(task_ref) = document.get("task_ref").and_then(Value::as_str) else {
        return error(
            400,
            "RESOURCE_PINNED_HTTPS_TASK_REF_REQUIRED",
            "task_ref is required",
        );
    };
    if cognitive_domain::UriRef::parse(task_ref).is_err() {
        return error(
            400,
            "RESOURCE_PINNED_HTTPS_TASK_REF_INVALID",
            "task_ref must be a canonical URI",
        );
    }
    let Some(campaign_id) = document.get("campaign_id").and_then(Value::as_str) else {
        return error(
            403,
            "RESOURCE_PINNED_HTTPS_UNAUTHORIZED",
            "campaign_id is required",
        );
    };
    if !campaign_is_authorized(campaign_id) {
        return error(
            403,
            "RESOURCE_PINNED_HTTPS_UNAUTHORIZED",
            "campaign is not authorized to pin HTTPS origins",
        );
    }
    let origins = match document.get("origins") {
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(Value::as_str)
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>(),
        None => Vec::new(),
        Some(_) => {
            return error(
                400,
                "RESOURCE_PINNED_HTTPS_ORIGIN_INVALID",
                "origins must be an array of HTTPS origin strings",
            );
        }
    };
    if origins.len() > MAX_ORIGINS {
        return error(
            409,
            "RESOURCE_PINNED_HTTPS_CAPACITY",
            "bounded origin capacity would be exceeded",
        );
    }
    for origin in &origins {
        if !valid_pinned_origin(origin) {
            return error(
                400,
                "RESOURCE_PINNED_HTTPS_ORIGIN_INVALID",
                "origins must be exact HTTPS origins without userinfo, path, query, or fragment",
            );
        }
    }
    let methods = match document.get("methods") {
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_ascii_uppercase)
            .collect::<Vec<_>>(),
        None => vec!["GET".to_owned(), "HEAD".to_owned()],
        Some(_) => {
            return error(
                400,
                "RESOURCE_PINNED_HTTPS_METHOD_INVALID",
                "methods must be GET and/or HEAD",
            );
        }
    };
    if methods.is_empty()
        || methods
            .iter()
            .any(|method| method != "GET" && method != "HEAD")
    {
        return error(
            400,
            "RESOURCE_PINNED_HTTPS_METHOD_INVALID",
            "methods must be GET and/or HEAD",
        );
    }
    let record = PinnedHttpsRecord {
        task_ref: task_ref.to_owned(),
        campaign_id: campaign_id.to_owned(),
        origins,
        methods,
    };
    if let Err(response) = persist_pin(layout, record.clone()) {
        return response;
    }
    ok(json!({
        "schema_version": 1,
        "task_ref": record.task_ref,
        "campaign_id": record.campaign_id,
        "origins": record.origins,
        "methods": record.methods,
        "authority_side_effects": false,
    }))
}

fn load_pin(
    layout: &PersonalDataLayout,
    task_ref: &str,
) -> Result<Option<PinnedHttpsRecord>, PinnedHttpsResponse> {
    let path = layout.data_dir().join(PINNED_HTTPS_FILE_NAME);
    if !path.exists() {
        return Ok(None);
    }
    let file = read_file(&path)?;
    Ok(file.pins.into_iter().find(|pin| pin.task_ref == task_ref))
}

fn persist_pin(
    layout: &PersonalDataLayout,
    record: PinnedHttpsRecord,
) -> Result<(), PinnedHttpsResponse> {
    let path = layout.data_dir().join(PINNED_HTTPS_FILE_NAME);
    let mut file = if path.exists() {
        read_file(&path)?
    } else {
        PinnedHttpsFile {
            schema: PINNED_HTTPS_SCHEMA.to_owned(),
            pins: Vec::new(),
        }
    };
    if file.schema != PINNED_HTTPS_SCHEMA {
        return Err(error(
            503,
            "RESOURCE_PINNED_HTTPS_STORE_UNAVAILABLE",
            "pinned HTTPS schema is not current",
        ));
    }
    file.pins.retain(|pin| pin.task_ref != record.task_ref);
    if file.pins.len() >= MAX_TASKS {
        return Err(error(
            409,
            "RESOURCE_PINNED_HTTPS_CAPACITY",
            "bounded pin capacity would be exceeded",
        ));
    }
    file.pins.push(record);
    write_file_atomically(&path, &file)
}

fn read_file(path: &Path) -> Result<PinnedHttpsFile, PinnedHttpsResponse> {
    let bytes = fs::read(path).map_err(|_| {
        error(
            503,
            "RESOURCE_PINNED_HTTPS_STORE_UNAVAILABLE",
            "pinned HTTPS registry cannot be read",
        )
    })?;
    serde_json::from_slice(&bytes).map_err(|_| {
        error(
            503,
            "RESOURCE_PINNED_HTTPS_STORE_UNAVAILABLE",
            "pinned HTTPS registry is not valid JSON",
        )
    })
}

fn write_file_atomically(path: &Path, file: &PinnedHttpsFile) -> Result<(), PinnedHttpsResponse> {
    let bytes = serde_json::to_vec_pretty(file).map_err(|_| {
        error(
            503,
            "RESOURCE_PINNED_HTTPS_STORE_UNAVAILABLE",
            "pinned HTTPS registry cannot be serialized",
        )
    })?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|_| {
            error(
                503,
                "RESOURCE_PINNED_HTTPS_STORE_UNAVAILABLE",
                "pinned HTTPS registry directory cannot be created",
            )
        })?;
    }
    let temporary_path = path.with_extension("json.tmp");
    let mut out = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&temporary_path)
        .map_err(|_| {
            error(
                503,
                "RESOURCE_PINNED_HTTPS_STORE_UNAVAILABLE",
                "pinned HTTPS registry cannot be persisted",
            )
        })?;
    out.write_all(&bytes).map_err(|_| {
        error(
            503,
            "RESOURCE_PINNED_HTTPS_STORE_UNAVAILABLE",
            "pinned HTTPS registry cannot be persisted",
        )
    })?;
    out.flush().map_err(|_| {
        error(
            503,
            "RESOURCE_PINNED_HTTPS_STORE_UNAVAILABLE",
            "pinned HTTPS registry cannot be persisted",
        )
    })?;
    fs::rename(&temporary_path, path).map_err(|_| {
        error(
            503,
            "RESOURCE_PINNED_HTTPS_STORE_UNAVAILABLE",
            "pinned HTTPS registry cannot be committed",
        )
    })
}

fn campaign_is_authorized(campaign_id: &str) -> bool {
    if campaign_id.is_empty() || campaign_id.len() > MAX_CAMPAIGN_ID_CHARS {
        return false;
    }
    let allowed = campaign_id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');
    allowed && (campaign_id.starts_with("PERSONAL-PERF-EVAL-") || campaign_id == "P2-T25")
}

fn valid_pinned_origin(origin: &str) -> bool {
    if origin.is_empty() || origin.len() > MAX_ORIGIN_CHARS {
        return false;
    }
    let Some(("https", authority)) = origin.split_once("://") else {
        return false;
    };
    if authority.contains('/') || authority.contains('?') || authority.contains('#') {
        return false;
    }
    valid_https_authority(authority)
}

fn parse_object(body: &[u8]) -> Result<Value, PinnedHttpsResponse> {
    match serde_json::from_slice::<Value>(body) {
        Ok(Value::Object(map)) => Ok(Value::Object(map)),
        _ => Err(error(
            400,
            "RESOURCE_PINNED_HTTPS_QUERY_FORBIDDEN",
            "JSON object payload is required",
        )),
    }
}

fn extra_keys(document: &Value, allowed: &[&str]) -> bool {
    document
        .as_object()
        .map(|map| map.keys().any(|key| !allowed.contains(&key.as_str())))
        .unwrap_or(true)
}

fn query_string(method_path: &str) -> Option<&str> {
    method_path
        .split_whitespace()
        .nth(1)
        .and_then(|path| path.split_once('?').map(|(_, query)| query))
}

fn forbidden_query_keys(method_path: &str) -> bool {
    let Some(query) = query_string(method_path) else {
        return false;
    };
    query.split('&').any(|pair| {
        let key = pair.split_once('=').map_or(pair, |(key, _)| key);
        FORBIDDEN_KEYS.contains(&key)
    })
}

fn required_task_ref_query(method_path: &str) -> Result<String, PinnedHttpsResponse> {
    let query = query_string(method_path).unwrap_or("");
    let mut task_ref = None;
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        if key == "task_ref" {
            task_ref = Some(value);
        } else if !FORBIDDEN_KEYS.contains(&key) && key != "task_ref" && !key.is_empty() {
            return Err(error(
                400,
                "RESOURCE_PINNED_HTTPS_QUERY_FORBIDDEN",
                "pinned HTTPS query accepts only task_ref",
            ));
        }
    }
    let Some(encoded) = task_ref.filter(|value| !value.is_empty()) else {
        return Err(error(
            400,
            "RESOURCE_PINNED_HTTPS_TASK_REF_REQUIRED",
            "task_ref is required",
        ));
    };
    let decoded = percent_decode(encoded);
    if cognitive_domain::UriRef::parse(&decoded).is_err() {
        return Err(error(
            400,
            "RESOURCE_PINNED_HTTPS_TASK_REF_INVALID",
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
        bytes.extend(chars[index].to_string().as_bytes());
        index += 1;
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn ok(value: Value) -> PinnedHttpsResponse {
    PinnedHttpsResponse {
        status: 200,
        body: value.to_string(),
    }
}

fn error(status: u16, code: &str, message: &str) -> PinnedHttpsResponse {
    PinnedHttpsResponse {
        status,
        body: json!({
            "error": { "code": code, "message": message }
        })
        .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cognitive_store::PersonalDataLayout;

    fn layout() -> PersonalDataLayout {
        let root = std::env::temp_dir().join(format!(
            "cos-p2t25-https-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("temp layout");
        PersonalDataLayout::from_xdg_roots(&root, &root, &root, &root, &root)
    }

    #[test]
    fn missing_pin_exposes_empty_allowlist() {
        let layout = layout();
        assert!(allowed_origins(layout.data_dir(), "task://personal/p2-t25").is_empty());
    }

    #[test]
    fn authorized_campaign_can_pin_loopback_origin_with_port() {
        let layout = layout();
        let response = handle(
            "POST /management/resource/v1/http-origin",
            br#"{"task_ref":"task://personal/p2-t25","campaign_id":"P2-T25","origins":["https://localhost:8443"],"methods":["GET","HEAD"]}"#,
            &layout,
        );
        assert_eq!(response.status, 200, "{}", response.body);
        assert_eq!(
            allowed_origins(layout.data_dir(), "task://personal/p2-t25"),
            vec!["https://localhost:8443".to_owned()]
        );
    }

    #[test]
    fn unauthorized_campaign_and_credential_origin_fail_closed() {
        let layout = layout();
        let unauthorized = handle(
            "POST /management/resource/v1/http-origin",
            br#"{"task_ref":"task://personal/p2-t25","campaign_id":"owner-local","origins":["https://example.com"]}"#,
            &layout,
        );
        assert_eq!(unauthorized.status, 403, "{}", unauthorized.body);
        assert!(
            unauthorized
                .body
                .contains("RESOURCE_PINNED_HTTPS_UNAUTHORIZED")
        );

        let credential = handle(
            "POST /management/resource/v1/http-origin",
            br#"{"task_ref":"task://personal/p2-t25","campaign_id":"P2-T25","origins":["https://user:pass@example.com"]}"#,
            &layout,
        );
        assert_eq!(credential.status, 400, "{}", credential.body);
        assert!(
            credential
                .body
                .contains("RESOURCE_PINNED_HTTPS_ORIGIN_INVALID")
        );
    }

    #[test]
    fn prompt_restatement_is_rejected() {
        let layout = layout();
        let response = handle(
            "POST /management/resource/v1/http-origin",
            br#"{"task_ref":"task://personal/p2-t25","campaign_id":"P2-T25","origins":["https://example.com"],"prompt":"widen"}"#,
            &layout,
        );
        assert_eq!(response.status, 400, "{}", response.body);
        assert!(
            response
                .body
                .contains("RESOURCE_PINNED_HTTPS_QUERY_FORBIDDEN")
        );
    }

    #[test]
    fn repin_replaces_origins_for_the_same_task() {
        let layout = layout();
        let first = handle(
            "POST /management/resource/v1/http-origin",
            br#"{"task_ref":"task://personal/p2-t25","campaign_id":"P2-T25","origins":["https://example.com"]}"#,
            &layout,
        );
        assert_eq!(first.status, 200, "{}", first.body);
        let second = handle(
            "POST /management/resource/v1/http-origin",
            br#"{"task_ref":"task://personal/p2-t25","campaign_id":"P2-T25","origins":["https://localhost:8443"]}"#,
            &layout,
        );
        assert_eq!(second.status, 200, "{}", second.body);
        assert_eq!(
            allowed_origins(layout.data_dir(), "task://personal/p2-t25"),
            vec!["https://localhost:8443".to_owned()]
        );
        let get = handle(
            "GET /management/resource/v1/http-origin?task_ref=task%3A%2F%2Fpersonal%2Fp2-t25",
            b"",
            &layout,
        );
        assert_eq!(get.status, 200, "{}", get.body);
        assert!(get.body.contains("https://localhost:8443"));
    }
}
