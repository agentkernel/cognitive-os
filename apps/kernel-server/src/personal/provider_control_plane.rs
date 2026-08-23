//! P8-T13 LLM Provider Control Plane management surface.
//!
//! Private daemon projection: named accounts, opaque Secret Store refs, model
//! catalog, fixed agent bindings, usage/cost/audit, observe-only budgets.
//! Not a public contract. Keys never enter SQLite or responses.

use cognitive_kernel::ports::IdGenerator;
use cognitive_provider_transport::{InsecureHttpProviderTransport, RustlsProviderTransport};
use cognitive_secret::{
    EndpointTrustError, EndpointTrustGrant, ProviderHttpMethod, ProviderHttpRequest,
    ProviderHttpResponse, ProviderKind, ProviderTransport, SecretMaterial, TrustedEndpoint,
    anthropic_api_key_header_value, bearer_authorization_header_value, evaluate_resolved_targets,
    provider_account_secret_attributes, provider_secret_label, reject_caller_headers,
    select_production_secret_store,
};
use cognitive_store::{
    AgentProviderBindingRecord, ProviderAccountRecord, ProviderControlPlaneStore,
    ProviderModelRecord, SqliteAuthorityStore, UuidV7Generator, apply_builtin_prices, now_ms,
    usage_from_anthropic_json, usage_from_openai_json,
};
use serde_json::{Value, json};
use std::net::ToSocketAddrs;
use std::time::Instant;

use super::resource_api::ResourceApiResponse;

const DISCOVERY_TIMEOUT_MS: u32 = 8_000;
pub(crate) const PI_AGENT: &str = "agent://personal/pi";
pub(crate) const DSH_AGENT: &str = "agent://personal/dsh";
const PROXY_TIMEOUT_MS: u32 = 60_000;

/// Route literals scanned by the handbook HTTP generator. Keep these exact.
const ROUTE_LITERALS: &[&str] = &[
    "GET /management/providers/accounts/inspect",
    "POST /management/providers/accounts/update",
    "POST /management/providers/accounts/delete",
    "POST /management/providers/accounts/key",
    "GET /management/providers/accounts",
    "POST /management/providers/accounts",
    "POST /management/providers/models/refresh",
    "POST /management/providers/models/add",
    "POST /management/providers/models/set-price",
    "GET /management/providers/models",
    "POST /management/agent-bindings/remove",
    "GET /management/agent-bindings",
    "POST /management/agent-bindings",
    "GET /management/usage",
    "POST /management/budgets/remove",
    "GET /management/budgets",
    "POST /management/budgets",
    "POST /management/alerts/acknowledge",
    "GET /management/alerts",
    "GET /management/audit",
    "GET /task/providers/accounts",
    "POST /task/providers/accounts",
    "GET /task/agent-bindings",
    "POST /task/agent-bindings",
    "GET /task/usage",
    "GET /task/budgets",
    "POST /task/budgets",
    "GET /task/alerts",
    "POST /task/alerts",
    "GET /task/audit",
    "POST /provider/v1/dsh/chat/completions",
    "GET /provider/v1/dsh/selected-model",
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
        "PROVIDER_CONTROL_CHANNEL_FORBIDDEN",
        "Provider Control Plane operations are management-channel only",
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
            "PROVIDER_CONTROL_ROUTE_NOT_FOUND",
            "no Provider Control Plane route matched",
        );
    };
    if channel == Channel::Task {
        return channel_forbidden();
    }
    let plane = ProviderControlPlaneStore::from_authority_store(store);
    match literal {
        "GET /management/providers/accounts" => list_accounts(&plane),
        "GET /management/providers/accounts/inspect" => inspect_account(method_path, &plane),
        "POST /management/providers/accounts" => create_account(body, &plane),
        "POST /management/providers/accounts/update" => update_account(body, &plane),
        "POST /management/providers/accounts/delete" => delete_account(body, &plane),
        "POST /management/providers/accounts/key" => set_or_remove_key(body, &plane),
        "POST /management/providers/models/refresh" => refresh_models(body, &plane),
        "GET /management/providers/models" => list_models(method_path, &plane),
        "POST /management/providers/models/add" => add_manual_model(body, &plane),
        "POST /management/providers/models/set-price" => set_price(body, &plane),
        "GET /management/agent-bindings" => list_bindings(&plane),
        "POST /management/agent-bindings" => set_binding(body, &plane),
        "POST /management/agent-bindings/remove" => remove_binding(body, &plane),
        "GET /management/usage" => query_usage(&plane),
        "GET /management/budgets" => list_budgets(&plane),
        "POST /management/budgets" => set_budget(body, &plane),
        "POST /management/budgets/remove" => remove_budget(body, &plane),
        "GET /management/alerts" => list_alerts(&plane),
        "POST /management/alerts/acknowledge" => acknowledge_alert(body, &plane),
        "GET /management/audit" => query_audit(&plane),
        _ => error(
            404,
            "PROVIDER_CONTROL_ROUTE_NOT_FOUND",
            "no Provider Control Plane route matched",
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

fn list_accounts(plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    match plane.list_accounts() {
        Ok(accounts) => ok(json!({
            "status": "ok",
            "accounts": accounts.iter().map(account_json).collect::<Vec<_>>(),
        })),
        Err(error) => store_error(error),
    }
}

fn inspect_account(method_path: &str, plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    let Some(id) = query_parameter(method_path, "id").filter(|value| !value.is_empty()) else {
        return error(400, "PROVIDER_ACCOUNT_ID_REQUIRED", "inspect requires id");
    };
    match plane.get_account(&id) {
        Ok(Some(account)) => ok(json!({"status":"ok","account": account_json(&account)})),
        Ok(None) => error(404, "PROVIDER_ACCOUNT_NOT_FOUND", "account not found"),
        Err(error) => store_error(error),
    }
}

fn create_account(body: &[u8], plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    let mut document = match json_object(body) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let api_key = take_string(&mut document, "api_key");
    let display_name = match required_name(document.get("display_name").and_then(Value::as_str)) {
        Ok(name) => name,
        Err(response) => return response,
    };
    let kind = match parse_kind(document.get("provider_kind").and_then(Value::as_str)) {
        Ok(kind) => kind,
        Err(response) => return response,
    };
    let grant = EndpointTrustGrant {
        allow_private_network: document
            .get("allow_private_network")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        allow_insecure_http: document
            .get("allow_insecure_http")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    };
    let endpoint = document.get("endpoint").and_then(Value::as_str);
    let trusted = match TrustedEndpoint::evaluate(kind, endpoint, grant) {
        Ok(endpoint) => endpoint,
        Err(error) => return trust_error(error),
    };
    if let Err(response) = reject_injected_headers(&document, kind) {
        return response;
    }
    let now = now_ms();
    let account_id = match UuidV7Generator.next_uuid_v7() {
        Ok(id) => format!("acct-{id}"),
        Err(_) => return error(503, "PROVIDER_CONTROL_UNAVAILABLE", "id generation failed"),
    };
    let record = ProviderAccountRecord {
        account_id: account_id.clone(),
        display_name,
        provider_kind: kind.as_str().to_owned(),
        endpoint: trusted.normalized().to_owned(),
        secret_ref: None,
        allow_private_network: grant.allow_private_network,
        allow_insecure_http: grant.allow_insecure_http,
        network_scope: trusted.scope().as_str().to_owned(),
        status: "revoked".to_owned(),
        catalog_revision: 0,
        last_discovery_error: None,
        created_at_ms: now,
        updated_at_ms: now,
    };
    if let Err(error) = plane.insert_account(&record) {
        return store_error(error);
    }
    let _ = plane.append_audit(
        &format!("aud-{account_id}-create"),
        now,
        "account.create",
        Some(&account_id),
        None,
        "ok",
        "account persisted without secret material",
    );
    if let Some(api_key) = api_key {
        return bind_key_and_discover(plane, &account_id, &api_key, false);
    }
    match plane.get_account(&account_id) {
        Ok(Some(account)) => ok(json!({"status":"ok","account": account_json(&account)})),
        Ok(None) => error(
            500,
            "PROVIDER_CONTROL_UNAVAILABLE",
            "account disappeared after create",
        ),
        Err(error) => store_error(error),
    }
}

fn update_account(body: &[u8], plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    let document = match json_object(body) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let Some(account_id) = document.get("id").and_then(Value::as_str) else {
        return error(400, "PROVIDER_ACCOUNT_ID_REQUIRED", "update requires id");
    };
    let existing = match plane.get_account(account_id) {
        Ok(Some(account)) => account,
        Ok(None) => return error(404, "PROVIDER_ACCOUNT_NOT_FOUND", "account not found"),
        Err(error) => return store_error(error),
    };
    let kind = match ProviderKind::parse(&existing.provider_kind) {
        Ok(kind) => kind,
        Err(error) => return trust_error(error),
    };
    let grant = EndpointTrustGrant {
        allow_private_network: document
            .get("allow_private_network")
            .and_then(Value::as_bool)
            .unwrap_or(existing.allow_private_network),
        allow_insecure_http: document
            .get("allow_insecure_http")
            .and_then(Value::as_bool)
            .unwrap_or(existing.allow_insecure_http),
    };
    let endpoint = document
        .get("endpoint")
        .and_then(Value::as_str)
        .unwrap_or(&existing.endpoint);
    let next = match TrustedEndpoint::evaluate(kind, Some(endpoint), grant) {
        Ok(endpoint) => endpoint,
        Err(error) => return trust_error(error),
    };
    let current = match TrustedEndpoint::from_persisted(
        kind,
        &existing.endpoint,
        &existing.network_scope,
        EndpointTrustGrant {
            allow_private_network: existing.allow_private_network,
            allow_insecure_http: existing.allow_insecure_http,
        },
    ) {
        Ok(endpoint) => endpoint,
        Err(error) => return trust_error(error),
    };
    if current.requires_reconfirm(&next)
        && document.get("reconfirm").and_then(Value::as_bool) != Some(true)
    {
        return error(
            409,
            EndpointTrustError::ReconfirmRequired.code(),
            "endpoint authority, DNS scope, or HTTPS to HTTP change requires reconfirm",
        );
    }
    if let Err(error) = plane.update_account_endpoint_trust(
        account_id,
        next.normalized(),
        grant.allow_private_network,
        grant.allow_insecure_http,
        next.scope().as_str(),
        now_ms(),
    ) {
        return store_error(error);
    }
    let _ = plane.append_audit(
        &format!("aud-{account_id}-update"),
        now_ms(),
        "account.update",
        Some(account_id),
        None,
        "ok",
        "endpoint trust updated",
    );
    match plane.get_account(account_id) {
        Ok(Some(account)) => ok(json!({"status":"ok","account": account_json(&account)})),
        Ok(None) => error(404, "PROVIDER_ACCOUNT_NOT_FOUND", "account not found"),
        Err(error) => store_error(error),
    }
}

fn delete_account(body: &[u8], plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    let document = match json_object(body) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let Some(account_id) = document.get("id").and_then(Value::as_str) else {
        return error(400, "PROVIDER_ACCOUNT_ID_REQUIRED", "delete requires id");
    };
    match plane.delete_account(account_id) {
        Ok(()) => {
            let _ = plane.append_audit(
                &format!("aud-{account_id}-delete"),
                now_ms(),
                "account.delete",
                Some(account_id),
                None,
                "ok",
                "account deleted",
            );
            ok(json!({"status":"ok"}))
        }
        Err(error) => store_error(error),
    }
}

fn set_or_remove_key(body: &[u8], plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    let mut document = match json_object(body) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let Some(account_id) = document
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_owned)
    else {
        return error(
            400,
            "PROVIDER_ACCOUNT_ID_REQUIRED",
            "key operation requires id",
        );
    };
    let op = document
        .get("op")
        .and_then(Value::as_str)
        .unwrap_or("set")
        .to_owned();
    match op.as_str() {
        "remove" => remove_key(plane, &account_id),
        "set" | "rotate" => {
            let Some(api_key) = take_string(&mut document, "api_key") else {
                return error(
                    400,
                    "PROVIDER_KEY_REQUIRED",
                    "key set/rotate requires api_key",
                );
            };
            bind_key_and_discover(plane, &account_id, &api_key, op == "rotate")
        }
        _ => error(
            400,
            "PROVIDER_KEY_OP_INVALID",
            "op must be set, rotate, or remove",
        ),
    }
}

fn remove_key(plane: &ProviderControlPlaneStore, account_id: &str) -> ResourceApiResponse {
    let account = match plane.get_account(account_id) {
        Ok(Some(account)) => account,
        Ok(None) => return error(404, "PROVIDER_ACCOUNT_NOT_FOUND", "account not found"),
        Err(error) => return store_error(error),
    };
    if let Some(secret_ref) = account.secret_ref {
        let backend = select_production_secret_store();
        if let Ok(reference) = cognitive_secret::SecretRef::from_opaque(secret_ref) {
            let _ = backend.as_secret_store().delete(&reference);
        }
    }
    if let Err(error) =
        plane.update_account_secret_and_status(account_id, None, "revoked", now_ms())
    {
        return store_error(error);
    }
    let _ = plane.append_audit(
        &format!("aud-{account_id}-key-remove"),
        now_ms(),
        "key.remove",
        Some(account_id),
        None,
        "ok",
        "secret ref cleared; account revoked",
    );
    ok(json!({"status":"ok","account_id": account_id, "account_status": "revoked"}))
}

fn bind_key_and_discover(
    plane: &ProviderControlPlaneStore,
    account_id: &str,
    api_key: &str,
    rotate: bool,
) -> ResourceApiResponse {
    let account = match plane.get_account(account_id) {
        Ok(Some(account)) => account,
        Ok(None) => return error(404, "PROVIDER_ACCOUNT_NOT_FOUND", "account not found"),
        Err(error) => return store_error(error),
    };
    let material = match SecretMaterial::from_bytes(api_key.as_bytes().to_vec()) {
        Ok(material) => material,
        Err(_) => return error(400, "PROVIDER_KEY_INVALID", "api key material is invalid"),
    };
    let backend = select_production_secret_store();
    let store = backend.as_secret_store();
    if !matches!(
        store.probe(),
        Ok(cognitive_secret::SecretStoreAvailability::Available)
    ) {
        return error(
            503,
            "PROVIDER_SECRET_STORE_UNAVAILABLE",
            "approved Secret Store is not available",
        );
    }
    let label = match provider_secret_label() {
        Ok(label) => label,
        Err(_) => return error(500, "PROVIDER_CONTROL_UNAVAILABLE", "secret label invalid"),
    };
    let attributes = match provider_account_secret_attributes(account_id, &account.provider_kind) {
        Ok(attributes) => attributes,
        Err(_) => {
            return error(
                400,
                "PROVIDER_ACCOUNT_ID_INVALID",
                "account id is not a valid secret attribute",
            );
        }
    };
    let secret_ref = match store.put(&label, &attributes, material) {
        Ok(secret_ref) => secret_ref,
        Err(_) => {
            return error(
                503,
                "PROVIDER_SECRET_STORE_UNAVAILABLE",
                "Secret Store rejected the key put",
            );
        }
    };
    if rotate
        && let Some(previous) = account.secret_ref.as_deref()
        && let Ok(previous_ref) = cognitive_secret::SecretRef::from_opaque(previous)
    {
        let _ = store.delete(&previous_ref);
    }
    if let Err(error) = plane.update_account_secret_and_status(
        account_id,
        Some(secret_ref.as_str()),
        "active",
        now_ms(),
    ) {
        return store_error(error);
    }
    let _ = plane.append_audit(
        &format!("aud-{account_id}-key"),
        now_ms(),
        if rotate { "key.rotate" } else { "key.set" },
        Some(account_id),
        None,
        "ok",
        "opaque secret_ref stored",
    );
    discover_models(plane, account_id)
}

fn refresh_models(body: &[u8], plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    let document = match json_object(body) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let Some(account_id) = document.get("id").and_then(Value::as_str) else {
        return error(400, "PROVIDER_ACCOUNT_ID_REQUIRED", "refresh requires id");
    };
    discover_models(plane, account_id)
}

fn discover_models(plane: &ProviderControlPlaneStore, account_id: &str) -> ResourceApiResponse {
    let account = match plane.get_account(account_id) {
        Ok(Some(account)) => account,
        Ok(None) => return error(404, "PROVIDER_ACCOUNT_NOT_FOUND", "account not found"),
        Err(error) => return store_error(error),
    };
    let previous_models = plane.list_models(account_id).unwrap_or_default();
    let kind = match ProviderKind::parse(&account.provider_kind) {
        Ok(kind) => kind,
        Err(error) => return trust_error(error),
    };
    let grant = EndpointTrustGrant {
        allow_private_network: account.allow_private_network,
        allow_insecure_http: account.allow_insecure_http,
    };
    let trusted = match TrustedEndpoint::evaluate(kind, Some(&account.endpoint), grant) {
        Ok(endpoint) => endpoint,
        Err(error) => return trust_error(error),
    };
    if let Err(error) = pin_resolved_targets(&trusted) {
        record_discovery_failure(plane, account_id, &previous_models, error.code());
        return trust_error(error);
    }
    let Some(secret_ref) = account.secret_ref.as_deref() else {
        record_discovery_failure(plane, account_id, &previous_models, "missing secret_ref");
        return error(409, "PROVIDER_KEY_MISSING", "account has no secret_ref");
    };
    let backend = select_production_secret_store();
    let reference = match cognitive_secret::SecretRef::from_opaque(secret_ref) {
        Ok(reference) => reference,
        Err(_) => return error(409, "PROVIDER_KEY_MISSING", "secret_ref is not usable"),
    };
    let material = match backend.as_secret_store().get(&reference) {
        Ok(material) => material,
        Err(_) => {
            let _ = plane.update_account_secret_and_status(
                account_id,
                Some(secret_ref),
                "revoked",
                now_ms(),
            );
            record_discovery_failure(
                plane,
                account_id,
                &previous_models,
                "secret_ref did not resolve",
            );
            return error(
                503,
                "PROVIDER_SECRET_UNAVAILABLE",
                "secret_ref did not resolve",
            );
        }
    };
    let headers = match discovery_headers(kind, material.expose_bytes()) {
        Ok(headers) => headers,
        Err(error) => return trust_error(error),
    };
    let models_path = if kind == ProviderKind::AnthropicOfficial {
        "/v1/models"
    } else {
        "/models"
    };
    let url = match trusted.join_api_path(models_path) {
        Ok(url) => url,
        Err(error) => return trust_error(error),
    };
    let request = ProviderHttpRequest {
        method: ProviderHttpMethod::Get,
        url,
        headers,
        body: None,
        timeout_ms: DISCOVERY_TIMEOUT_MS,
        cancel_requested: false,
    };
    let exchange = if trusted.uses_http() {
        InsecureHttpProviderTransport.exchange(&request)
    } else {
        RustlsProviderTransport::default().exchange(&request)
    };
    match exchange {
        Ok(response) if (200..300).contains(&response.status) => {
            match parse_discovered_models(&account, &response.body) {
                Ok(models) => {
                    let revision = account.catalog_revision + 1;
                    if let Err(error) =
                        plane.replace_discovered_models(account_id, revision, &models)
                    {
                        return store_error(error);
                    }
                    let _ = plane.mark_discovery_outcome(
                        account_id,
                        "active",
                        revision,
                        None,
                        now_ms(),
                    );
                    let _ = plane.append_audit(
                        &format!("aud-{account_id}-discover"),
                        now_ms(),
                        "models.refresh",
                        Some(account_id),
                        None,
                        "ok",
                        "catalog refreshed",
                    );
                    list_models_inner(plane, account_id)
                }
                Err(detail) => {
                    record_discovery_failure(plane, account_id, &previous_models, detail);
                    error(502, "PROVIDER_DISCOVERY_MALFORMED", detail)
                }
            }
        }
        Ok(response) => {
            let detail = classified_upstream(response.status);
            record_discovery_failure(plane, account_id, &previous_models, detail);
            error(502, "PROVIDER_DISCOVERY_FAILED", detail)
        }
        Err(_) => {
            record_discovery_failure(plane, account_id, &previous_models, "transport failure");
            error(
                502,
                "PROVIDER_DISCOVERY_FAILED",
                "discovery transport failed",
            )
        }
    }
}

fn record_discovery_failure(
    plane: &ProviderControlPlaneStore,
    account_id: &str,
    previous: &[ProviderModelRecord],
    detail: &str,
) {
    let catalog_revision = previous
        .iter()
        .map(|model| model.catalog_revision)
        .max()
        .unwrap_or(0);
    let _ = plane.mark_discovery_outcome(
        account_id,
        "degraded",
        catalog_revision,
        Some(detail),
        now_ms(),
    );
    let _ = plane.append_audit(
        &format!("aud-{account_id}-discover-fail"),
        now_ms(),
        "models.refresh",
        Some(account_id),
        None,
        "failed",
        "discovery failed; catalog and bindings preserved",
    );
}

fn list_models(method_path: &str, plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    let Some(account_id) = query_parameter(method_path, "account_id") else {
        return error(
            400,
            "PROVIDER_ACCOUNT_ID_REQUIRED",
            "models list requires account_id",
        );
    };
    list_models_inner(plane, &account_id)
}

fn list_models_inner(plane: &ProviderControlPlaneStore, account_id: &str) -> ResourceApiResponse {
    match plane.list_models(account_id) {
        Ok(models) => ok(json!({
            "status": "ok",
            "models": models.iter().map(model_json).collect::<Vec<_>>(),
        })),
        Err(error) => store_error(error),
    }
}

fn add_manual_model(body: &[u8], plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    let document = match json_object(body) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let Some(account_id) = document.get("account_id").and_then(Value::as_str) else {
        return error(
            400,
            "PROVIDER_ACCOUNT_ID_REQUIRED",
            "add model requires account_id",
        );
    };
    let Some(model_id) = document.get("model_id").and_then(Value::as_str) else {
        return error(
            400,
            "PROVIDER_MODEL_ID_REQUIRED",
            "add model requires model_id",
        );
    };
    let account = match plane.get_account(account_id) {
        Ok(Some(account)) => account,
        Ok(None) => return error(404, "PROVIDER_ACCOUNT_NOT_FOUND", "account not found"),
        Err(error) => return store_error(error),
    };
    let mut model = ProviderModelRecord {
        account_id: account_id.to_owned(),
        model_id: model_id.to_owned(),
        source: "manually_configured".to_owned(),
        pricing_version: document
            .get("pricing_version")
            .and_then(Value::as_str)
            .map(str::to_owned),
        price_input_per_million: document
            .get("price_input_per_million")
            .and_then(Value::as_str)
            .map(str::to_owned),
        price_output_per_million: document
            .get("price_output_per_million")
            .and_then(Value::as_str)
            .map(str::to_owned),
        price_cache_read_per_million: document
            .get("price_cache_read_per_million")
            .and_then(Value::as_str)
            .map(str::to_owned),
        price_cache_write_per_million: document
            .get("price_cache_write_per_million")
            .and_then(Value::as_str)
            .map(str::to_owned),
        catalog_revision: account.catalog_revision,
    };
    apply_builtin_prices(&account.provider_kind, &mut model);
    if let Err(error) = plane.upsert_manual_model(&model) {
        return store_error(error);
    }
    ok(json!({"status":"ok","model": model_json(&model)}))
}

fn set_price(body: &[u8], plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    let document = match json_object(body) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let Some(account_id) = document.get("account_id").and_then(Value::as_str) else {
        return error(
            400,
            "PROVIDER_ACCOUNT_ID_REQUIRED",
            "set-price requires account_id",
        );
    };
    let Some(model_id) = document.get("model_id").and_then(Value::as_str) else {
        return error(
            400,
            "PROVIDER_MODEL_ID_REQUIRED",
            "set-price requires model_id",
        );
    };
    match plane.set_model_prices(
        account_id,
        model_id,
        document
            .get("pricing_version")
            .and_then(Value::as_str)
            .unwrap_or("manual"),
        document
            .get("price_input_per_million")
            .and_then(Value::as_str),
        document
            .get("price_output_per_million")
            .and_then(Value::as_str),
        document
            .get("price_cache_read_per_million")
            .and_then(Value::as_str),
        document
            .get("price_cache_write_per_million")
            .and_then(Value::as_str),
    ) {
        Ok(()) => ok(json!({"status":"ok"})),
        Err(error) => store_error(error),
    }
}

fn list_bindings(plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    match plane.list_bindings() {
        Ok(bindings) => ok(json!({
            "status": "ok",
            "bindings": bindings.iter().map(binding_json).collect::<Vec<_>>(),
        })),
        Err(error) => store_error(error),
    }
}

fn set_binding(body: &[u8], plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    let document = match json_object(body) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let agent = match normalize_agent(document.get("agent").and_then(Value::as_str)) {
        Ok(agent) => agent,
        Err(response) => return response,
    };
    let Some(account_id) = document.get("account_id").and_then(Value::as_str) else {
        return error(
            400,
            "PROVIDER_ACCOUNT_ID_REQUIRED",
            "binding requires account_id",
        );
    };
    let Some(model_id) = document.get("model_id").and_then(Value::as_str) else {
        return error(
            400,
            "PROVIDER_MODEL_ID_REQUIRED",
            "binding requires model_id",
        );
    };
    if let Some(expected) = document.get("expected_revision").and_then(Value::as_i64) {
        let current = plane
            .get_active_binding(&agent)
            .ok()
            .flatten()
            .map(|binding| binding.revision)
            .unwrap_or(0);
        if expected != current {
            return error(
                409,
                "PROVIDER_BINDING_REVISION_STALE",
                "expected_revision does not match the current binding revision",
            );
        }
    }
    if plane.get_account(account_id).ok().flatten().is_none() {
        return error(404, "PROVIDER_ACCOUNT_NOT_FOUND", "account not found");
    }
    if plane
        .get_model(account_id, model_id)
        .ok()
        .flatten()
        .is_none()
    {
        return error(
            404,
            "PROVIDER_MODEL_NOT_FOUND",
            "model not in catalog; add it manually",
        );
    }
    match plane.set_binding(
        &AgentProviderBindingRecord {
            agent_instance_id: agent.clone(),
            account_id: account_id.to_owned(),
            model_id: model_id.to_owned(),
            revision: 1,
            status: "active".to_owned(),
        },
        now_ms(),
    ) {
        Ok(binding) => {
            let _ = plane.append_audit(
                &format!("aud-bind-{agent}"),
                now_ms(),
                "binding.set",
                Some(account_id),
                Some(&agent),
                "ok",
                "fixed account+provider+model binding stored",
            );
            ok(json!({"status":"ok","binding": binding_json(&binding)}))
        }
        Err(error) => store_error(error),
    }
}

fn remove_binding(body: &[u8], plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    let document = match json_object(body) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let agent = match normalize_agent(document.get("agent").and_then(Value::as_str)) {
        Ok(agent) => agent,
        Err(response) => return response,
    };
    match plane.remove_binding(&agent, now_ms()) {
        Ok(()) => ok(json!({"status":"ok"})),
        Err(error) => store_error(error),
    }
}

fn query_usage(plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    let _ = plane.apply_retention(now_ms());
    match plane.list_usage_events(0) {
        Ok(events) => ok(json!({
            "status": "ok",
            "events": events.iter().map(|(id, account, cost, status)| json!({
                "event_id": id,
                "account_id": account,
                "cost_micros": cost,
                "cost_status": status
            })).collect::<Vec<_>>(),
        })),
        Err(error) => store_error(error),
    }
}

fn list_budgets(plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    match plane.list_budgets() {
        Ok(budgets) => ok(json!({
            "status": "ok",
            "budgets": budgets.iter().map(|(id, kind, scope, tokens, amount)| json!({
                "budget_id": id,
                "scope_kind": kind,
                "scope_id": scope,
                "token_limit": tokens,
                "amount_micros_limit": amount
            })).collect::<Vec<_>>(),
        })),
        Err(error) => store_error(error),
    }
}

fn set_budget(body: &[u8], plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    let document = match json_object(body) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let budget_id = document
        .get("budget_id")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| format!("bud-{}", now_ms()));
    let Some(scope_kind) = document.get("scope_kind").and_then(Value::as_str) else {
        return error(
            400,
            "PROVIDER_BUDGET_SCOPE_REQUIRED",
            "budget requires scope_kind",
        );
    };
    let Some(scope_id) = document.get("scope_id").and_then(Value::as_str) else {
        return error(
            400,
            "PROVIDER_BUDGET_SCOPE_REQUIRED",
            "budget requires scope_id",
        );
    };
    match plane.upsert_budget(
        &budget_id,
        scope_kind,
        scope_id,
        document.get("token_limit").and_then(Value::as_i64),
        document.get("amount_micros_limit").and_then(Value::as_i64),
        now_ms(),
    ) {
        Ok(()) => {
            let _ = plane.maybe_issue_budget_alerts(now_ms());
            ok(json!({"status":"ok","budget_id": budget_id}))
        }
        Err(error) => store_error(error),
    }
}

fn remove_budget(body: &[u8], plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    let document = match json_object(body) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let Some(budget_id) = document.get("budget_id").and_then(Value::as_str) else {
        return error(
            400,
            "PROVIDER_BUDGET_ID_REQUIRED",
            "remove requires budget_id",
        );
    };
    match plane.remove_budget(budget_id) {
        Ok(()) => ok(json!({"status":"ok"})),
        Err(error) => store_error(error),
    }
}

fn list_alerts(plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    let _ = plane.maybe_issue_budget_alerts(now_ms());
    match plane.list_alerts() {
        Ok(alerts) => ok(json!({
            "status": "ok",
            "alerts": alerts.iter().map(|(id, budget, kind, issued, ack)| json!({
                "alert_id": id,
                "budget_id": budget,
                "threshold_kind": kind,
                "issued_at_ms": issued,
                "acknowledged_at_ms": ack
            })).collect::<Vec<_>>(),
        })),
        Err(error) => store_error(error),
    }
}

fn acknowledge_alert(body: &[u8], plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    let document = match json_object(body) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let Some(alert_id) = document.get("alert_id").and_then(Value::as_str) else {
        return error(
            400,
            "PROVIDER_ALERT_ID_REQUIRED",
            "acknowledge requires alert_id",
        );
    };
    match plane.acknowledge_alert(alert_id, now_ms()) {
        Ok(()) => ok(json!({"status":"ok"})),
        Err(error) => store_error(error),
    }
}

fn query_audit(plane: &ProviderControlPlaneStore) -> ResourceApiResponse {
    match plane.list_audit(0) {
        Ok(events) => ok(json!({
            "status": "ok",
            "events": events.iter().map(|(id, action, outcome, detail)| json!({
                "audit_id": id,
                "action": action,
                "outcome": outcome,
                "detail": detail
            })).collect::<Vec<_>>(),
        })),
        Err(error) => store_error(error),
    }
}

fn parse_discovered_models(
    account: &ProviderAccountRecord,
    body: &[u8],
) -> Result<Vec<ProviderModelRecord>, &'static str> {
    if body.len() > 1_048_576 {
        return Err("discovery body exceeds local limit");
    }
    let parsed: Value = serde_json::from_slice(body).map_err(|_| "malformed discovery json")?;
    let data = parsed
        .get("data")
        .and_then(Value::as_array)
        .ok_or("malformed discovery json")?;
    let mut models = Vec::new();
    for item in data {
        let Some(model_id) = item.get("id").and_then(Value::as_str) else {
            continue;
        };
        let mut model = ProviderModelRecord {
            account_id: account.account_id.clone(),
            model_id: model_id.to_owned(),
            source: "provider_discovered".to_owned(),
            pricing_version: None,
            price_input_per_million: None,
            price_output_per_million: None,
            price_cache_read_per_million: None,
            price_cache_write_per_million: None,
            catalog_revision: account.catalog_revision + 1,
        };
        apply_builtin_prices(&account.provider_kind, &mut model);
        models.push(model);
    }
    Ok(models)
}

fn discovery_headers(
    kind: ProviderKind,
    material: &[u8],
) -> Result<Vec<(String, String)>, EndpointTrustError> {
    let headers = match kind {
        ProviderKind::AnthropicOfficial => vec![
            (
                "x-api-key".to_owned(),
                anthropic_api_key_header_value(material)
                    .map_err(|_| EndpointTrustError::Invalid)?,
            ),
            (
                "anthropic-version".to_owned(),
                cognitive_secret::ANTHROPIC_API_VERSION.to_owned(),
            ),
        ],
        ProviderKind::OpenaiOfficial | ProviderKind::OpenaiCompatible => vec![(
            "Authorization".to_owned(),
            bearer_authorization_header_value(material).map_err(|_| EndpointTrustError::Invalid)?,
        )],
    };
    reject_caller_headers(&headers, kind)?;
    Ok(headers)
}

pub(crate) fn pin_resolved_targets(endpoint: &TrustedEndpoint) -> Result<(), EndpointTrustError> {
    let host_port = format!("{}:{}", endpoint.host(), endpoint.port());
    let resolved = host_port
        .to_socket_addrs()
        .map_err(|_| EndpointTrustError::Invalid)?
        .map(|addr| addr.ip())
        .collect::<Vec<_>>();
    evaluate_resolved_targets(endpoint, &resolved)
}

fn classified_upstream(status: u16) -> &'static str {
    match status {
        401 => "upstream 401",
        403 => "upstream 403",
        404 => "upstream 404",
        429 => "upstream 429",
        500..=599 => "upstream 5xx",
        _ => "upstream error",
    }
}

fn reject_injected_headers(
    document: &Value,
    kind: ProviderKind,
) -> Result<(), ResourceApiResponse> {
    if document.get("headers").is_some() || document.get("authorization").is_some() {
        return Err(trust_error(EndpointTrustError::ArbitraryHeaderForbidden));
    }
    let _ = kind;
    Ok(())
}

fn normalize_agent(raw: Option<&str>) -> Result<String, ResourceApiResponse> {
    match raw {
        Some("pi") | Some(PI_AGENT) => Ok(PI_AGENT.to_owned()),
        Some("dsh") | Some(DSH_AGENT) => Ok(DSH_AGENT.to_owned()),
        Some(_) => Err(error(
            400,
            "PROVIDER_AGENT_UNSUPPORTED",
            "agent must be pi or dsh (independent adapters)",
        )),
        None => Err(error(
            400,
            "PROVIDER_AGENT_REQUIRED",
            "binding requires agent",
        )),
    }
}

fn parse_kind(raw: Option<&str>) -> Result<ProviderKind, ResourceApiResponse> {
    match raw {
        Some(token) => ProviderKind::parse(token).map_err(trust_error),
        None => Err(error(
            400,
            "PROVIDER_KIND_REQUIRED",
            "provider_kind is required",
        )),
    }
}

fn required_name(raw: Option<&str>) -> Result<String, ResourceApiResponse> {
    let Some(name) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return Err(error(
            400,
            "PROVIDER_ACCOUNT_NAME_REQUIRED",
            "display_name is required",
        ));
    };
    if name.len() > 64
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(error(
            400,
            "PROVIDER_ACCOUNT_NAME_INVALID",
            "display_name is invalid",
        ));
    }
    Ok(name.to_owned())
}

fn take_string(document: &mut Value, key: &str) -> Option<String> {
    document
        .as_object_mut()?
        .remove(key)?
        .as_str()
        .map(str::to_owned)
}

fn json_object(body: &[u8]) -> Result<Value, ResourceApiResponse> {
    match serde_json::from_slice::<Value>(body) {
        Ok(Value::Object(map)) => Ok(Value::Object(map)),
        Ok(_) | Err(_) => Err(error(
            400,
            "PROVIDER_CONTROL_BODY_INVALID",
            "JSON object required",
        )),
    }
}

fn query_parameter(method_path: &str, name: &str) -> Option<String> {
    let (_, query) = method_path.split_once('?')?;
    for pair in query.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        if key == name {
            return Some(value.trim().to_owned());
        }
    }
    None
}

fn account_json(account: &ProviderAccountRecord) -> Value {
    json!({
        "id": account.account_id,
        "display_name": account.display_name,
        "provider_kind": account.provider_kind,
        "endpoint": account.endpoint,
        "secret_ref": account.secret_ref,
        "status": account.status,
        "catalog_revision": account.catalog_revision,
        "last_discovery_error": account.last_discovery_error,
        "allow_private_network": account.allow_private_network,
        "allow_insecure_http": account.allow_insecure_http,
        "network_scope": account.network_scope
    })
}

fn model_json(model: &ProviderModelRecord) -> Value {
    json!({
        "account_id": model.account_id,
        "model_id": model.model_id,
        "source": model.source,
        "pricing_version": model.pricing_version,
        "price_input_per_million": model.price_input_per_million,
        "price_output_per_million": model.price_output_per_million,
        "price_cache_read_per_million": model.price_cache_read_per_million,
        "price_cache_write_per_million": model.price_cache_write_per_million
    })
}

fn binding_json(binding: &AgentProviderBindingRecord) -> Value {
    json!({
        "agent": binding.agent_instance_id,
        "account_id": binding.account_id,
        "model_id": binding.model_id,
        "revision": binding.revision,
        "status": binding.status
    })
}

fn trust_error(err: EndpointTrustError) -> ResourceApiResponse {
    error(400, err.code(), err.code())
}

fn store_error(err: cognitive_store::ProviderControlPlaneError) -> ResourceApiResponse {
    match err {
        cognitive_store::ProviderControlPlaneError::Conflict { detail } => {
            error(409, "PROVIDER_CONTROL_CONFLICT", detail)
        }
        cognitive_store::ProviderControlPlaneError::NotFound { detail } => {
            error(404, "PROVIDER_CONTROL_NOT_FOUND", detail)
        }
        cognitive_store::ProviderControlPlaneError::Invalid { detail } => {
            error(400, "PROVIDER_CONTROL_INVALID", detail)
        }
        cognitive_store::ProviderControlPlaneError::Unavailable { .. } => {
            error(503, "PROVIDER_CONTROL_UNAVAILABLE", "store unavailable")
        }
    }
}

fn ok(body: Value) -> ResourceApiResponse {
    json_response(200, body)
}

fn error(status: u16, code: &str, message: &str) -> ResourceApiResponse {
    json_response(
        status,
        json!({"status":"error","code": code, "message": message}),
    )
}

fn json_response(status: u16, body: Value) -> ResourceApiResponse {
    ResourceApiResponse {
        status,
        body: body.to_string(),
        content_type: "application/json",
    }
}

pub(crate) fn record_proxy_usage(
    store: &SqliteAuthorityStore,
    account: &ProviderAccountRecord,
    model_id: &str,
    agent: &str,
    body: &[u8],
    duration_ms: u128,
    outcome: &str,
) {
    let plane = ProviderControlPlaneStore::from_authority_store(store);
    let model = plane
        .get_model(&account.account_id, model_id)
        .ok()
        .flatten();
    let sample = match account.provider_kind.as_str() {
        "anthropic_official" => {
            usage_from_anthropic_json(&serde_json::from_slice(body).unwrap_or(Value::Null))
        }
        _ => usage_from_openai_json(&serde_json::from_slice(body).unwrap_or(Value::Null)),
    };
    let cost = cognitive_store::compute_cost(&sample, model.as_ref());
    let metering_source = if sample.input_tokens.is_some() && sample.output_tokens.is_some() {
        "provider_reported"
    } else {
        "unavailable"
    };
    let event_id = UuidV7Generator
        .next_uuid_v7()
        .unwrap_or_else(|_| format!("evt-{}", now_ms()));
    let _ = plane.record_usage(&cognitive_store::NewUsageEvent {
        event_id: event_id.clone(),
        idempotency_key: event_id,
        recorded_at_ms: now_ms(),
        account_id: account.account_id.clone(),
        provider_kind: account.provider_kind.clone(),
        model_id: model_id.to_owned(),
        agent_instance_id: agent.to_owned(),
        sample,
        duration_ms: i64::try_from(duration_ms).ok(),
        outcome: outcome.to_owned(),
        metering_source: metering_source.to_owned(),
        estimation_method: None,
        cost,
    });
    let _ = plane.maybe_issue_budget_alerts(now_ms());
}

/// Prepared bound-agent Provider call. Absence means the P1-T07 provider.json path.
pub(crate) struct BoundProxyPlan {
    pub account: ProviderAccountRecord,
    pub model_id: String,
    pub request: ProviderHttpRequest,
    pub uses_http: bool,
    pub anthropic: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoundPlanError {
    BindingMismatch,
    AccountUnavailable,
    SecretUnavailable,
    InvalidRequest,
    StreamingUnsupported,
    Trust,
    UpstreamFailed,
}

pub(crate) fn execute_bound_unary_plan(
    store: &SqliteAuthorityStore,
    agent: &str,
    plan: BoundProxyPlan,
) -> Result<ProviderHttpResponse, BoundPlanError> {
    let started = Instant::now();
    let exchange = if plan.uses_http {
        InsecureHttpProviderTransport.exchange(&plan.request)
    } else {
        RustlsProviderTransport::default().exchange(&plan.request)
    };
    let elapsed_nanos = started.elapsed().as_nanos().max(1);
    let elapsed_ms = u128::from(u64::try_from(elapsed_nanos / 1_000_000).unwrap_or(u64::MAX));
    match exchange {
        Ok(response) => {
            let outcome = if (200..300).contains(&response.status) {
                "ok"
            } else {
                "failed"
            };
            record_proxy_usage(
                store,
                &plan.account,
                &plan.model_id,
                agent,
                &response.body,
                elapsed_ms,
                outcome,
            );
            if plan.anthropic && response.status == 200 {
                let body = anthropic_messages_to_openai_chat(&response.body)
                    .unwrap_or_else(|_| response.body.clone());
                Ok(ProviderHttpResponse {
                    status: response.status,
                    body,
                })
            } else {
                Ok(response)
            }
        }
        Err(_) => {
            record_proxy_usage(
                store,
                &plan.account,
                &plan.model_id,
                agent,
                b"{}",
                elapsed_ms,
                "failed",
            );
            Err(BoundPlanError::UpstreamFailed)
        }
    }
}

pub(crate) fn plan_bound_proxy(
    store: &SqliteAuthorityStore,
    agent: &str,
    request_body: &[u8],
    stream: bool,
) -> Result<Option<BoundProxyPlan>, BoundPlanError> {
    let plane = ProviderControlPlaneStore::from_authority_store(store);
    let Some(binding) = plane
        .get_active_binding(agent)
        .map_err(|_| BoundPlanError::AccountUnavailable)?
    else {
        return Ok(None);
    };
    let request_json: Value =
        serde_json::from_slice(request_body).map_err(|_| BoundPlanError::InvalidRequest)?;
    let requested_model = request_json
        .get("model")
        .and_then(Value::as_str)
        .filter(|model| !model.is_empty())
        .ok_or(BoundPlanError::InvalidRequest)?;
    // Path B / native dsh catalog ids (deepseek-chat, …) must not block the
    // Cos-assigned dsh binding. Pi keeps exact match so a dsh model cannot
    // leak onto agent://personal/pi.
    let outbound_model = if agent == DSH_AGENT {
        binding.model_id.clone()
    } else if requested_model != binding.model_id {
        return Err(BoundPlanError::BindingMismatch);
    } else {
        binding.model_id.clone()
    };
    let account = plane
        .get_account(&binding.account_id)
        .map_err(|_| BoundPlanError::AccountUnavailable)?
        .ok_or(BoundPlanError::AccountUnavailable)?;
    if account.status == "revoked" || account.secret_ref.is_none() {
        return Err(BoundPlanError::AccountUnavailable);
    }
    let kind = ProviderKind::parse(&account.provider_kind).map_err(|_| BoundPlanError::Trust)?;
    if kind == ProviderKind::AnthropicOfficial && stream {
        return Err(BoundPlanError::StreamingUnsupported);
    }
    let grant = EndpointTrustGrant {
        allow_private_network: account.allow_private_network,
        allow_insecure_http: account.allow_insecure_http,
    };
    let trusted =
        TrustedEndpoint::from_persisted(kind, &account.endpoint, &account.network_scope, grant)
            .map_err(|_| BoundPlanError::Trust)?;
    pin_resolved_targets(&trusted).map_err(|_| BoundPlanError::Trust)?;
    let secret_ref = account
        .secret_ref
        .as_deref()
        .ok_or(BoundPlanError::SecretUnavailable)?;
    let reference = cognitive_secret::SecretRef::from_opaque(secret_ref)
        .map_err(|_| BoundPlanError::SecretUnavailable)?;
    let backend = select_production_secret_store();
    let material = backend
        .as_secret_store()
        .get(&reference)
        .map_err(|_| BoundPlanError::SecretUnavailable)?;
    let outbound_body = if kind == ProviderKind::AnthropicOfficial {
        openai_chat_to_anthropic_messages(request_body, &outbound_model)?
    } else {
        rewrite_openai_model(request_body, &outbound_model)?
    };
    let mut headers =
        discovery_headers(kind, material.expose_bytes()).map_err(|_| BoundPlanError::Trust)?;
    headers.push(("Content-Type".to_owned(), "application/json".to_owned()));
    if stream {
        headers.push(("Accept".to_owned(), "text/event-stream".to_owned()));
    }
    let path = if kind == ProviderKind::AnthropicOfficial {
        "/v1/messages"
    } else {
        "/chat/completions"
    };
    let url = trusted
        .join_api_path(path)
        .map_err(|_| BoundPlanError::Trust)?;
    Ok(Some(BoundProxyPlan {
        account,
        model_id: binding.model_id,
        request: ProviderHttpRequest {
            method: ProviderHttpMethod::Post,
            url,
            headers,
            body: Some(outbound_body),
            timeout_ms: PROXY_TIMEOUT_MS,
            cancel_requested: false,
        },
        uses_http: trusted.uses_http(),
        anthropic: kind == ProviderKind::AnthropicOfficial,
    }))
}

pub(crate) fn selected_binding_model(store: &SqliteAuthorityStore, agent: &str) -> Option<String> {
    let plane = ProviderControlPlaneStore::from_authority_store(store);
    plane
        .get_active_binding(agent)
        .ok()
        .flatten()
        .map(|binding| binding.model_id)
}

fn rewrite_openai_model(request_body: &[u8], bound_model: &str) -> Result<Vec<u8>, BoundPlanError> {
    let mut parsed: Value =
        serde_json::from_slice(request_body).map_err(|_| BoundPlanError::InvalidRequest)?;
    let Some(object) = parsed.as_object_mut() else {
        return Err(BoundPlanError::InvalidRequest);
    };
    object.insert("model".to_owned(), Value::String(bound_model.to_owned()));
    serde_json::to_vec(&parsed).map_err(|_| BoundPlanError::InvalidRequest)
}

fn openai_chat_to_anthropic_messages(
    request_body: &[u8],
    bound_model: &str,
) -> Result<Vec<u8>, BoundPlanError> {
    let parsed: Value =
        serde_json::from_slice(request_body).map_err(|_| BoundPlanError::InvalidRequest)?;
    let Some(messages) = parsed.get("messages").and_then(Value::as_array) else {
        return Err(BoundPlanError::InvalidRequest);
    };
    let mut system = Vec::new();
    let mut anthropic_messages = Vec::new();
    for message in messages {
        let role = message.get("role").and_then(Value::as_str).unwrap_or("");
        let content = message.get("content").and_then(Value::as_str).unwrap_or("");
        match role {
            "system" => system.push(content.to_owned()),
            "user" | "assistant" => anthropic_messages.push(json!({
                "role": role,
                "content": content,
            })),
            _ => return Err(BoundPlanError::InvalidRequest),
        }
    }
    if anthropic_messages.is_empty() {
        return Err(BoundPlanError::InvalidRequest);
    }
    let max_tokens = parsed
        .get("max_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(1024);
    let mut body = json!({
        "model": bound_model,
        "max_tokens": max_tokens,
        "messages": anthropic_messages,
    });
    if !system.is_empty() {
        body["system"] = Value::String(system.join("\n"));
    }
    serde_json::to_vec(&body).map_err(|_| BoundPlanError::InvalidRequest)
}

pub(crate) fn anthropic_messages_to_openai_chat(body: &[u8]) -> Result<Vec<u8>, BoundPlanError> {
    let parsed: Value = serde_json::from_slice(body).map_err(|_| BoundPlanError::InvalidRequest)?;
    let text = parsed
        .get("content")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|block| {
            if block.get("type").and_then(Value::as_str) == Some("text") {
                block.get("text").and_then(Value::as_str)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("");
    let usage = parsed.get("usage").cloned().unwrap_or(Value::Null);
    let input = usage.get("input_tokens").and_then(Value::as_i64);
    let output = usage.get("output_tokens").and_then(Value::as_i64);
    let cache_read = usage.get("cache_read_input_tokens").and_then(Value::as_i64);
    let openai = json!({
        "choices": [{"message": {"role": "assistant", "content": text}}],
        "usage": {
            "prompt_tokens": input,
            "completion_tokens": output,
            "prompt_tokens_details": {"cached_tokens": cache_read}
        }
    });
    serde_json::to_vec(&openai).map_err(|_| BoundPlanError::InvalidRequest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_openai_model_replaces_catalog_id_with_binding() {
        let body = serde_json::to_vec(&json!({
            "model": "deepseek-chat",
            "messages": [{"role": "user", "content": "hi"}]
        }))
        .expect("body");
        let rewritten = rewrite_openai_model(&body, "grok-4.6").expect("rewrite");
        let parsed: Value = serde_json::from_slice(&rewritten).expect("json");
        assert_eq!(parsed["model"], "grok-4.6");
        assert_eq!(parsed["messages"][0]["content"], "hi");
    }
}
