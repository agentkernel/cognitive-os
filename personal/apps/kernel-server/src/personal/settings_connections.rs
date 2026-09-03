//! P13-T08 Settings Model Connections, diagnostics, and notification groups.
//!
//! Settings is the real caller of Provider Control Plane write: template or
//! custom URL/compat + key + model go through SecretStore takeover. The
//! response is connected/failed with secret presence only. Task-channel
//! aliases are 403. Windows SecretStore host E2E stays `not-run` until
//! `P13-T13`. Engine health is honest-empty when P13-T02 facts are absent.

use cognitive_store::{
    ASSISTANT_PI_PIN, HOSTED_DSH_ARTIFACT_DIGEST, HostedDshAttemptStore, SqliteAuthorityStore,
    WindowsHostStore,
};
use serde_json::{Value, json};

use super::provider_control_plane;
use super::resource_api::ResourceApiResponse;

const ROUTE_LITERALS: &[&str] = &[
    "POST /management/settings/v1/connection.connect",
    "GET /management/settings/v1/diagnostics",
    "GET /management/settings/v1/notifications",
    "POST /task/settings/v1/connection.connect",
    "GET /task/settings/v1/diagnostics",
    "GET /task/settings/v1/notifications",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Channel {
    Management,
    Task,
}

pub(crate) fn matches(method_path: &str) -> bool {
    parse_route(method_path).is_some()
}

pub(crate) fn is_task_channel(method_path: &str) -> bool {
    parse_route(method_path).is_some_and(|(channel, _)| channel == Channel::Task)
}

pub(crate) fn channel_forbidden() -> ResourceApiResponse {
    error(
        403,
        "SETTINGS_CONNECTIONS_CHANNEL_FORBIDDEN",
        "Settings connection operations are management-channel only",
    )
}

pub(crate) fn handle(
    method_path: &str,
    body: &[u8],
    store: &SqliteAuthorityStore,
) -> ResourceApiResponse {
    let Some((channel, literal)) = parse_route(method_path) else {
        return error(
            404,
            "SETTINGS_CONNECTIONS_ROUTE_NOT_FOUND",
            "no Settings connection route matched",
        );
    };
    if channel == Channel::Task {
        return channel_forbidden();
    }
    match literal {
        "POST /management/settings/v1/connection.connect" => connect(body, store),
        "GET /management/settings/v1/diagnostics" => diagnostics(store),
        "GET /management/settings/v1/notifications" => notifications(method_path, store),
        _ => error(
            404,
            "SETTINGS_CONNECTIONS_ROUTE_NOT_FOUND",
            "no Settings connection route matched",
        ),
    }
}

fn parse_route(method_path: &str) -> Option<(Channel, &'static str)> {
    for literal in ROUTE_LITERALS {
        if method_path.starts_with(literal) {
            let channel = if literal.contains("/task/") {
                Channel::Task
            } else {
                Channel::Management
            };
            return Some((channel, *literal));
        }
    }
    None
}

fn connect(body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let mut document = match serde_json::from_slice::<Value>(body) {
        Ok(Value::Object(map)) => Value::Object(map),
        _ => {
            return error(
                400,
                "SETTINGS_CONNECTION_JSON_REQUIRED",
                "JSON object required",
            );
        }
    };
    let api_key = take_string(&mut document, "api_key");
    if api_key.as_ref().is_none_or(|key| key.trim().is_empty()) {
        return error(
            400,
            "SETTINGS_CONNECTION_KEY_REQUIRED",
            "Settings Model Connections require an API key; keyless persist is refused",
        );
    }
    let Some(api_key) = api_key else {
        return error(
            400,
            "SETTINGS_CONNECTION_KEY_REQUIRED",
            "Settings Model Connections require an API key; keyless persist is refused",
        );
    };
    let template = document
        .get("template")
        .and_then(Value::as_str)
        .unwrap_or("custom")
        .trim()
        .to_ascii_lowercase();
    let mapped = match map_template(&template, &document) {
        Ok(mapped) => mapped,
        Err(response) => return response,
    };
    let model = document
        .get("model")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    let create_body = json!({
        "display_name": mapped.display_name,
        "provider_kind": mapped.provider_kind,
        "endpoint": mapped.endpoint,
        "allow_private_network": mapped.allow_private_network,
        "allow_insecure_http": mapped.allow_insecure_http,
        "api_key": api_key,
    });
    let created = provider_control_plane::handle(
        "POST /management/providers/accounts",
        &create_body.to_string().into_bytes(),
        store,
    );
    if created.status >= 300 {
        return rewrite_failed(created);
    }
    let parsed = parse_json_object(&created.body);
    let account = parsed
        .get("account")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let account_id = account
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();
    if let Some(model_id) = model.as_deref()
        && !account_id.is_empty()
    {
        let _ = provider_control_plane::handle(
            "POST /management/providers/models/add",
            &json!({
                "account_id": account_id,
                "model_id": model_id,
            })
            .to_string()
            .into_bytes(),
            store,
        );
    }
    let connection_status = map_connection_status(
        account.get("status").and_then(Value::as_str).unwrap_or(""),
        account.get("last_discovery_error").and_then(Value::as_str),
    );
    let secret = if account.get("secret_ref").and_then(Value::as_str).is_some() {
        "present"
    } else {
        "absent"
    };
    redacted_ok(json!({
        "status": "ok",
        "connection": {
            "id": account_id,
            "display_name": account.get("display_name").cloned().unwrap_or(json!("unknown")),
            "provider_kind": mapped.provider_kind,
            "connection_status": connection_status,
            "secret": secret,
            "model_id": model,
            "last_discovery_error": account.get("last_discovery_error").cloned().unwrap_or(Value::Null),
        },
        "windows_secretstore_e2e": "not-run",
    }))
}

struct MappedTemplate {
    display_name: String,
    provider_kind: &'static str,
    endpoint: Option<String>,
    allow_private_network: bool,
    allow_insecure_http: bool,
}

fn map_template(template: &str, document: &Value) -> Result<MappedTemplate, ResourceApiResponse> {
    let display_name = document
        .get("display_name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("settings-{template}"));
    let allow_private_network = document
        .get("allow_private_network")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let allow_insecure_http = document
        .get("allow_insecure_http")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    match template {
        "openai" => Ok(MappedTemplate {
            display_name,
            provider_kind: "openai_official",
            endpoint: None,
            allow_private_network: false,
            allow_insecure_http: false,
        }),
        "anthropic" => Ok(MappedTemplate {
            display_name,
            provider_kind: "anthropic_official",
            endpoint: None,
            allow_private_network: false,
            allow_insecure_http: false,
        }),
        "deepseek" => Ok(MappedTemplate {
            display_name,
            provider_kind: "openai_compatible",
            endpoint: Some("https://api.deepseek.com".to_owned()),
            allow_private_network: false,
            allow_insecure_http: false,
        }),
        "custom" => {
            let endpoint = document
                .get("endpoint")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned);
            if endpoint.is_none() {
                return Err(error(
                    400,
                    "SETTINGS_CONNECTION_ENDPOINT_REQUIRED",
                    "custom / compatible mode requires a URL",
                ));
            }
            Ok(MappedTemplate {
                display_name,
                provider_kind: "openai_compatible",
                endpoint,
                allow_private_network,
                allow_insecure_http,
            })
        }
        _ => Err(error(
            400,
            "SETTINGS_CONNECTION_TEMPLATE_UNKNOWN",
            "template must be openai, anthropic, deepseek, or custom",
        )),
    }
}

fn map_connection_status(status: &str, discovery_error: Option<&str>) -> &'static str {
    if discovery_error.is_some() || status == "degraded" || status == "revoked" {
        return "failed";
    }
    if status == "active" {
        return "connected";
    }
    "failed"
}

fn rewrite_failed(created: ResourceApiResponse) -> ResourceApiResponse {
    let parsed = parse_json_object(&created.body);
    let code = parsed
        .get("code")
        .and_then(Value::as_str)
        .unwrap_or("SETTINGS_CONNECTION_FAILED");
    let message = parsed
        .get("message")
        .and_then(Value::as_str)
        .or_else(|| parsed.get("detail").and_then(Value::as_str))
        .unwrap_or("connection failed");
    let mut body = json!({
        "status": "error",
        "code": code,
        "message": message,
        "connection_status": "failed",
        "windows_secretstore_e2e": "not-run",
    });
    if let Some(object) = body.as_object_mut() {
        object.remove("api_key");
        object.remove("secret_ref");
    }
    ResourceApiResponse {
        status: created.status,
        body: body.to_string(),
        content_type: "application/json",
    }
}

fn diagnostics(store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let attempts = HostedDshAttemptStore::from_authority_store(store);
    let dsh = match attempts.latest_artifact_fact() {
        Ok(Some(fact)) => json!({
            "facts": "present",
            "kind": fact.kind,
            "expected_revision": fact.expected_revision,
            "configured_revision": fact.configured_revision,
            "health": fact.health,
            "update": fact.kind,
            "rollback": fact.previous_fact_id,
            "detail": fact.detail_redacted,
        }),
        Ok(None) => json!({
            "facts": "empty",
            "expected_revision": HOSTED_DSH_ARTIFACT_DIGEST,
            "configured_revision": null,
            "health": null,
            "update": null,
            "rollback": null,
        }),
        Err(_) => json!({
            "facts": "empty",
            "expected_revision": HOSTED_DSH_ARTIFACT_DIGEST,
            "health": null,
            "update": null,
            "rollback": null,
        }),
    };
    redacted_ok(json!({
        "status": "ok",
        "dsh": dsh,
        "pi": {
            "facts": "empty",
            "exact_version": ASSISTANT_PI_PIN,
            "health": null,
            "update": null,
            "rollback": null,
            "note": "engine health is honest-empty until a live Pi status fact exists; P13-T02 is not a mutex",
        },
        "windows_secretstore_e2e": "not-run",
    }))
}

fn notifications(method_path: &str, store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let home_id = query_parameter(method_path, "home_id");
    let mut missed = Vec::new();
    let mut offline = Vec::new();
    let mut resume = Vec::new();
    if let Some(home_id) = home_id
        && let Ok(status) =
            WindowsHostStore::from_authority_store(store).observe_status(&home_id, None)
    {
        if status.missed_segments > 0 {
            missed.push(json!({
                "kind": "missed",
                "detail": format!("{} host segments", status.missed_segments),
                "source": "host",
            }));
        }
        if status.daemon_state == "offline"
            || status.daemon_state == "stopped"
            || status.close_disposition.as_deref() == Some("offline")
        {
            offline.push(json!({
                "kind": "offline",
                "detail": status.daemon_state,
                "source": "host",
            }));
        }
        if status.resume_eligible {
            resume.push(json!({
                "kind": "resume",
                "detail": "resume-eligible-only",
                "source": "host",
            }));
        }
    }
    redacted_ok(json!({
        "status": "ok",
        "missed": missed,
        "offline": offline,
        "resume": resume,
        "windows_host_e2e": "not-run",
    }))
}

fn take_string(document: &mut Value, key: &str) -> Option<String> {
    document
        .as_object_mut()?
        .remove(key)?
        .as_str()
        .map(str::to_owned)
}

fn parse_json_object(body: &str) -> Value {
    serde_json::from_str::<Value>(body).unwrap_or(json!({}))
}

fn query_parameter(method_path: &str, name: &str) -> Option<String> {
    let (_, query) = method_path.split_once('?')?;
    for pair in query.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        if key == name {
            return Some(value.split_whitespace().next().unwrap_or(value).to_owned());
        }
    }
    None
}

fn ok(body: Value) -> ResourceApiResponse {
    ResourceApiResponse {
        status: 200,
        body: body.to_string(),
        content_type: "application/json",
    }
}

fn redacted_ok(body: Value) -> ResourceApiResponse {
    let serialized = body.to_string();
    let lowered = serialized.to_ascii_lowercase();
    if lowered.contains("sk-")
        || lowered.contains("api_key")
        || lowered.contains("secretref:")
        || lowered.contains("ssv1:")
    {
        return error(
            500,
            "SETTINGS_CONNECTIONS_REDACTION",
            "Settings response contained secret-shaped material",
        );
    }
    ok(body)
}

fn error(status: u16, code: &str, message: &str) -> ResourceApiResponse {
    ResourceApiResponse {
        status,
        body: json!({
            "status": "error",
            "code": code,
            "message": message,
            "connection_status": if status >= 400 { "failed" } else { "unknown" },
        })
        .to_string(),
        content_type: "application/json",
    }
}

// Test fixture setup uses expect for filesystem/database construction.
#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use cognitive_store::{PersonalDataLayout, prepare_personal_databases};
    use tempfile::TempDir;

    fn store() -> (TempDir, SqliteAuthorityStore) {
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
        let store = SqliteAuthorityStore::open(&layout.authority_database_path()).expect("open");
        (temporary, store)
    }

    #[test]
    fn task_alias_is_forbidden() {
        let (_tmp, store) = store();
        let response = handle(
            "POST /task/settings/v1/connection.connect",
            br#"{"template":"openai","api_key":"sk-live-hidden"}"#,
            &store,
        );
        assert_eq!(response.status, 403);
        assert!(!response.body.to_ascii_lowercase().contains("sk-live"));
    }

    #[test]
    fn connect_without_key_is_refused() {
        let (_tmp, store) = store();
        let response = handle(
            "POST /management/settings/v1/connection.connect",
            br#"{"template":"openai","model":"gpt-4o"}"#,
            &store,
        );
        assert_eq!(response.status, 400, "{}", response.body);
        assert!(response.body.contains("SETTINGS_CONNECTION_KEY_REQUIRED"));
        assert!(response.body.contains("\"connection_status\":\"failed\""));
    }

    #[test]
    fn connect_blank_key_is_refused() {
        let (_tmp, store) = store();
        let response = handle(
            "POST /management/settings/v1/connection.connect",
            br#"{"template":"openai","api_key":"   "}"#,
            &store,
        );
        assert_eq!(response.status, 400, "{}", response.body);
        assert!(!response.body.contains("api_key"));
    }

    #[test]
    fn custom_without_url_is_refused() {
        let (_tmp, store) = store();
        let response = handle(
            "POST /management/settings/v1/connection.connect",
            br#"{"template":"custom","api_key":"sk-live-hidden"}"#,
            &store,
        );
        assert_eq!(response.status, 400, "{}", response.body);
        assert!(
            response
                .body
                .contains("SETTINGS_CONNECTION_ENDPOINT_REQUIRED")
        );
        assert!(!response.body.to_ascii_lowercase().contains("sk-live"));
    }

    #[test]
    fn diagnostics_are_honest_empty_without_engine_facts() {
        let (_tmp, store) = store();
        let response = handle("GET /management/settings/v1/diagnostics", b"", &store);
        assert_eq!(response.status, 200, "{}", response.body);
        assert!(response.body.contains("\"facts\":\"empty\""));
        assert!(response.body.contains(ASSISTANT_PI_PIN));
        assert!(!response.body.contains("sk-"));
    }

    #[test]
    fn notifications_are_empty_groups_without_home() {
        let (_tmp, store) = store();
        let response = handle("GET /management/settings/v1/notifications", b"", &store);
        assert_eq!(response.status, 200, "{}", response.body);
        assert!(response.body.contains("\"missed\":[]"));
        assert!(response.body.contains("\"offline\":[]"));
        assert!(response.body.contains("\"resume\":[]"));
    }

    #[test]
    fn connect_never_echoes_the_key_when_secretstore_is_unavailable() {
        let (_tmp, store) = store();
        let response = handle(
            "POST /management/settings/v1/connection.connect",
            br#"{"template":"openai","api_key":"sk-live-must-never-return","model":"gpt-4o"}"#,
            &store,
        );
        assert!(
            !response.body.to_ascii_lowercase().contains("sk-live"),
            "{}",
            response.body
        );
        assert!(!response.body.contains("api_key"), "{}", response.body);
        if response.status >= 400 {
            assert!(
                response.body.contains("\"connection_status\":\"failed\""),
                "{}",
                response.body
            );
        } else {
            assert!(
                response
                    .body
                    .contains("\"connection_status\":\"connected\"")
                    || response.body.contains("\"connection_status\":\"failed\""),
                "{}",
                response.body
            );
        }
    }
}
