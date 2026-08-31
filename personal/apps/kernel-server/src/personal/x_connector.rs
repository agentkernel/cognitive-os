//! Personal-private `/management/connector/x/v1/*` projection (P11-T14).
//!
//! Management-channel only. Task-channel writes are 403. Live X API E2E is
//! not claimed by this HTTP surface.

use cognitive_store::{
    ConfirmCaller, ProjectAggregateError, SqliteAuthorityStore, X_CONNECTOR_PROJECTION_ID,
    XConnectorBindSpec, XConnectorConfirmSpec, XConnectorDispatchSpec, XConnectorPreviewSpec,
    XConnectorStore,
};
use serde_json::{Value, json};

use super::resource_api::ResourceApiResponse;

const ROUTE_LITERALS: &[&str] = &[
    "POST /management/connector/x/v1/account.bind",
    "POST /management/connector/x/v1/preview.request",
    "POST /management/connector/x/v1/preview.confirm",
    "POST /management/connector/x/v1/publish.dispatch",
    "GET /management/connector/x/v1/status",
    "POST /task/connector/x/v1/account.bind",
    "POST /task/connector/x/v1/preview.request",
    "POST /task/connector/x/v1/preview.confirm",
    "POST /task/connector/x/v1/publish.dispatch",
    "GET /task/connector/x/v1/status",
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
        "X_CONNECTOR_CHANNEL_FORBIDDEN",
        "X connector operations are management-channel only",
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
            "X_CONNECTOR_ROUTE_NOT_FOUND",
            "no X connector route matched",
        );
    };
    if channel == Channel::Task {
        return channel_forbidden();
    }
    match literal {
        "POST /management/connector/x/v1/account.bind" => account_bind(body, store),
        "POST /management/connector/x/v1/preview.request" => preview_request(body, store),
        "POST /management/connector/x/v1/preview.confirm" => preview_confirm(body, store),
        "POST /management/connector/x/v1/publish.dispatch" => publish_dispatch(body, store),
        "GET /management/connector/x/v1/status" => status(method_path, store),
        _ => error(
            404,
            "X_CONNECTOR_ROUTE_NOT_FOUND",
            "no X connector route matched",
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

fn account_bind(body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "X_JSON_REQUIRED", "JSON body required");
    };
    let Some(project_id) = document.get("project_id").and_then(Value::as_str) else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    let Some(handle) = document.get("handle").and_then(Value::as_str) else {
        return error(400, "HANDLE_REQUIRED", "handle required");
    };
    let Some(secret_ref) = document.get("secret_ref").and_then(Value::as_str) else {
        return error(400, "SECRET_REF_REQUIRED", "secret_ref required");
    };
    let consent = document
        .get("consent")
        .and_then(Value::as_str)
        .unwrap_or("owner-per-source");
    let argv_owned = string_list(&document, "argv");
    let env_owned = env_pairs(&document);
    let argv: Vec<&str> = argv_owned.iter().map(String::as_str).collect();
    let env: Vec<(&str, &str)> = env_owned
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    let connector = XConnectorStore::from_authority_store(store);
    match connector.bind_account(
        ConfirmCaller::OwnerManagement,
        &XConnectorBindSpec {
            project_id,
            handle,
            secret_ref,
            consent,
            argv: &argv,
            env_pairs: &env,
            hero_claim: document
                .get("hero_claim")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            default_demo: document
                .get("default_demo")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            p0_success_path: document
                .get("p0_success_path")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            platform_qualified_claim: document
                .get("platform_qualified")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            evasion: document
                .get("evasion")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            now_ms: now_ms(),
        },
    ) {
        Ok(account) => ok(json!({
            "projection": X_CONNECTOR_PROJECTION_ID,
            "account_id": account.account_id,
            "project_id": account.project_id,
            "handle": account.handle,
            "is_p0_hero": false,
            "platform_qualified": false
        })),
        Err(err) => store_error(err),
    }
}

fn preview_request(body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "X_JSON_REQUIRED", "JSON body required");
    };
    let Some(account_id) = document.get("account_id").and_then(Value::as_str) else {
        return error(400, "ACCOUNT_ID_REQUIRED", "account_id required");
    };
    let Some(project_id) = document.get("project_id").and_then(Value::as_str) else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    let Some(content) = document.get("content").and_then(Value::as_str) else {
        return error(400, "CONTENT_REQUIRED", "content required");
    };
    let content_kind = document
        .get("content_kind")
        .and_then(Value::as_str)
        .unwrap_or("original");
    let rights_attestation = document
        .get("rights_attestation")
        .and_then(Value::as_str)
        .unwrap_or("original-owner-rights");
    let connector = XConnectorStore::from_authority_store(store);
    match connector.request_preview(
        ConfirmCaller::OwnerManagement,
        &XConnectorPreviewSpec {
            account_id,
            project_id,
            content,
            content_kind,
            rights_attestation,
            evasion: document
                .get("evasion")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            chat_approve: document
                .get("chat_approve")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            now_ms: now_ms(),
        },
    ) {
        Ok(preview) => ok(json!({
            "projection": X_CONNECTOR_PROJECTION_ID,
            "preview_id": preview.preview_id,
            "account_id": preview.account_id,
            "project_id": preview.project_id,
            "content_digest": preview.content_digest,
            "confirmed": preview.confirmed
        })),
        Err(err) => store_error(err),
    }
}

fn preview_confirm(body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "X_JSON_REQUIRED", "JSON body required");
    };
    let Some(preview_id) = document.get("preview_id").and_then(Value::as_str) else {
        return error(400, "PREVIEW_ID_REQUIRED", "preview_id required");
    };
    let Some(expected_digest) = document.get("expected_digest").and_then(Value::as_str) else {
        return error(400, "DIGEST_REQUIRED", "expected_digest required");
    };
    let connector = XConnectorStore::from_authority_store(store);
    match connector.confirm_preview(
        ConfirmCaller::OwnerManagement,
        &XConnectorConfirmSpec {
            preview_id,
            expected_digest,
            chat_approve: document
                .get("chat_approve")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            now_ms: now_ms(),
        },
    ) {
        Ok(preview) => ok(json!({
            "projection": X_CONNECTOR_PROJECTION_ID,
            "preview_id": preview.preview_id,
            "confirmed": preview.confirmed,
            "receipt_is_not_completion": true
        })),
        Err(err) => store_error(err),
    }
}

fn publish_dispatch(body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "X_JSON_REQUIRED", "JSON body required");
    };
    let Some(preview_id) = document.get("preview_id").and_then(Value::as_str) else {
        return error(400, "PREVIEW_ID_REQUIRED", "preview_id required");
    };
    let connector = XConnectorStore::from_authority_store(store);
    match connector.dispatch_publish(
        ConfirmCaller::OwnerManagement,
        &XConnectorDispatchSpec {
            preview_id,
            claim_complete: document
                .get("claim_complete")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            impressions: document.get("impressions").and_then(Value::as_str),
            now_ms: now_ms(),
        },
    ) {
        Ok(published) => ok(json!({
            "projection": X_CONNECTOR_PROJECTION_ID,
            "publish_id": published.publish_id,
            "preview_id": published.preview_id,
            "intent_persisted": published.intent_persisted,
            "dispatched": published.dispatched,
            "readback_status": published.readback_status,
            "impressions": published.impressions,
            "receipt_is_not_completion": true
        })),
        Err(err) => store_error(err),
    }
}

fn status(method_path: &str, store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let Some(account_id) = query_parameter(method_path, "account_id") else {
        return error(400, "ACCOUNT_ID_REQUIRED", "account_id required");
    };
    let connector = XConnectorStore::from_authority_store(store);
    match connector.status(ConfirmCaller::OwnerManagement, &account_id) {
        Ok(status) => {
            let body = json!({
                "projection": X_CONNECTOR_PROJECTION_ID,
                "account_id": status.account_id,
                "project_id": status.project_id,
                "handle": status.handle,
                "is_p0_hero": false,
                "platform_qualified": false,
                "preview_id": status.preview_id,
                "confirmed": status.confirmed,
                "dispatched": status.dispatched,
                "readback_status": status.readback_status,
                "impressions": status.impressions,
                "receipt_is_not_completion": true
            });
            let serialized = body.to_string();
            if serialized.contains("secretref:")
                || serialized.contains("sk-")
                || serialized.contains("\"impressions\":0")
            {
                return error(500, "X_STATUS_REDACTION", "status redaction failed");
            }
            ok(body)
        }
        Err(err) => store_error(err),
    }
}

fn string_list(document: &Value, name: &str) -> Vec<String> {
    document
        .get(name)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn env_pairs(document: &Value) -> Vec<(String, String)> {
    document
        .get("env")
        .and_then(Value::as_object)
        .map(|object| {
            object
                .iter()
                .filter_map(|(key, value)| {
                    value.as_str().map(|text| (key.clone(), text.to_owned()))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_json(body: &[u8]) -> Option<Value> {
    serde_json::from_slice(body).ok()
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

fn now_ms() -> i64 {
    cognitive_store::now_ms()
}

fn ok(body: Value) -> ResourceApiResponse {
    ResourceApiResponse {
        status: 200,
        body: body.to_string(),
        content_type: "application/json",
    }
}

fn error(status: u16, code: &str, message: &str) -> ResourceApiResponse {
    ResourceApiResponse {
        status,
        body: json!({"status":"error","code": code, "message": message}).to_string(),
        content_type: "application/json",
    }
}

fn store_error(err: ProjectAggregateError) -> ResourceApiResponse {
    match err {
        ProjectAggregateError::Forbidden { detail } => error(403, "X_FORBIDDEN", detail),
        ProjectAggregateError::NotFound { detail } => error(404, "X_NOT_FOUND", detail),
        ProjectAggregateError::Conflict { detail } => error(409, "X_CONFLICT", detail),
        ProjectAggregateError::Stale { detail } => error(409, "X_STALE", detail),
        ProjectAggregateError::Unconfirmed { detail }
        | ProjectAggregateError::Rejected { detail } => error(422, "X_REJECTED", detail),
        ProjectAggregateError::Invalid { detail } => error(422, "X_INVALID", detail),
        ProjectAggregateError::Unavailable { .. } => {
            error(503, "X_UNAVAILABLE", "store unavailable")
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use cognitive_store::{
        ConfirmCaller, PersonalDataLayout, ProjectAggregateStore, prepare_personal_databases,
    };
    use tempfile::TempDir;

    fn authority() -> (TempDir, SqliteAuthorityStore) {
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

    fn activate(store: &SqliteAuthorityStore) -> String {
        let projects = ProjectAggregateStore::from_authority_store(store);
        let (draft_id, _) = projects.create_draft(b"charter-v1", 10).expect("draft");
        projects
            .put_draft_charter(&draft_id, b"charter-body-v1", 11)
            .expect("charter");
        let (preview_id, preview_digest) = projects
            .request_preview("activation", &draft_id, b"activation-preview", 12)
            .expect("preview");
        projects
            .confirm_preview(
                ConfirmCaller::OwnerManagement,
                &preview_id,
                &preview_digest,
                13,
            )
            .expect("G1")
            .new_ref
    }

    #[test]
    fn p11_t14_connector_negatives_and_task_channel_is_forbidden() {
        let (_tmp, store) = authority();
        let forbidden = handle(
            "POST /task/connector/x/v1/account.bind",
            json!({"project_id": "x", "handle": "@o", "secret_ref": "secretref:h"})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(forbidden.status, 403);
        assert!(forbidden.body.contains("X_CONNECTOR_CHANNEL_FORBIDDEN"));

        let project_id = activate(&store);
        let secret = handle(
            "POST /management/connector/x/v1/account.bind",
            json!({
                "project_id": project_id,
                "handle": "@owner",
                "secret_ref": "sk-http"
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(secret.status, 422, "{}", secret.body);
        assert!(secret.body.contains("raw secret"));
        assert!(!secret.body.contains("sk-http"));

        let evasion = handle(
            "POST /management/connector/x/v1/account.bind",
            json!({
                "project_id": project_id,
                "handle": "@owner",
                "secret_ref": "secretref:opaque-x-handle",
                "argv": ["--fingerprint"]
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(evasion.status, 422, "{}", evasion.body);
        assert!(evasion.body.contains("evasion"));

        let bound = handle(
            "POST /management/connector/x/v1/account.bind",
            json!({
                "project_id": project_id,
                "handle": "@owner",
                "secret_ref": "secretref:opaque-x-handle"
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(bound.status, 200, "{}", bound.body);
        let account_id = serde_json::from_str::<Value>(&bound.body).unwrap()["account_id"]
            .as_str()
            .unwrap()
            .to_owned();

        let preview = handle(
            "POST /management/connector/x/v1/preview.request",
            json!({
                "account_id": account_id,
                "project_id": project_id,
                "content": "original note from the owner"
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(preview.status, 200, "{}", preview.body);
        let preview_json = serde_json::from_str::<Value>(&preview.body).unwrap();
        let preview_id = preview_json["preview_id"].as_str().unwrap();
        let digest = preview_json["content_digest"].as_str().unwrap();

        let too_soon = handle(
            "POST /management/connector/x/v1/publish.dispatch",
            json!({"preview_id": preview_id}).to_string().as_bytes(),
            &store,
        );
        assert_eq!(too_soon.status, 422, "{}", too_soon.body);
        assert!(too_soon.body.contains("HITL confirm"));

        let confirmed = handle(
            "POST /management/connector/x/v1/preview.confirm",
            json!({
                "preview_id": preview_id,
                "expected_digest": digest
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(confirmed.status, 200, "{}", confirmed.body);
        assert!(
            confirmed
                .body
                .contains("\"receipt_is_not_completion\":true")
        );

        let complete = handle(
            "POST /management/connector/x/v1/publish.dispatch",
            json!({"preview_id": preview_id, "claim_complete": true})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(complete.status, 422, "{}", complete.body);
        assert!(complete.body.contains("receipt is not completion"));

        let published = handle(
            "POST /management/connector/x/v1/publish.dispatch",
            json!({"preview_id": preview_id}).to_string().as_bytes(),
            &store,
        );
        assert_eq!(published.status, 200, "{}", published.body);
        assert!(published.body.contains("\"impressions\":\"unknown\""));
        assert!(!published.body.contains("\"impressions\":0"));
        assert!(
            published
                .body
                .contains("\"receipt_is_not_completion\":true")
        );

        let status = handle(
            &format!("GET /management/connector/x/v1/status?account_id={account_id}"),
            b"",
            &store,
        );
        assert_eq!(status.status, 200, "{}", status.body);
        assert!(status.body.contains("\"impressions\":\"unknown\""));
        assert!(!status.body.contains("secretref:"));
        assert!(!status.body.contains("sk-"));
        assert!(status.body.contains("\"platform_qualified\":false"));
    }
}
