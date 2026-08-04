//! Bounded loopback Personal HTTP front door (P1-T04 / ADR-0019).

#[cfg(unix)]
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cognitive_secret::{
    ProviderConfigRepository, SelectedModelRepository, select_production_secret_store,
};
use cognitive_store::{PersonalDataLayout, prepare_personal_databases};
use serde_json::json;

use super::auth::{ChannelClass, LocalAuthError, LocalSessionAuthority, SessionIssueRequest};
use super::bounds::{
    PersonalResourceBounds, RequestBoundError, validate_body_length, validate_header_block,
};
use super::lifecycle::{DaemonLifecycleError, DaemonSingleInstanceLock};
use super::provider_proxy::{ProviderProxyService, RustlsProviderTransport};
use super::readiness::{
    ReadinessEvaluationContext, doctor_projection_json, evaluate_personal_readiness,
    status_projection_json,
};
use super::resource_api::ResourceApi;
use super::scheduler_authority::{
    reconcile_scheduler_recovery_at_startup, run_private_scheduler_tick,
};
use super::task_api::TaskApi;

const ENDPOINT_FILE_NAME: &str = "daemon-endpoint.json";
static ENDPOINT_TEMPORARY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Configuration for the Personal loopback daemon surface.
#[derive(Debug, Clone)]
pub struct PersonalDaemonConfig {
    pub bind_address: String,
    pub layout: PersonalDataLayout,
    pub bounds: PersonalResourceBounds,
    /// When true, accept a single connection then exit (tests).
    pub once: bool,
}

/// Failures starting or serving the Personal daemon.
#[derive(Debug)]
pub enum PersonalDaemonError {
    BindRefused { detail: String },
    Lifecycle(DaemonLifecycleError),
    Auth(LocalAuthError),
    Io { detail: String },
}

/// Owns the endpoint document for the lifetime of the bound listener.
///
/// The document is published only after a successful loopback bind and is
/// removed during orderly shutdown. A forced process termination can leave a
/// stale document, so consumers must still verify the daemon lock/process.
struct EndpointPublication {
    path: PathBuf,
}

impl Drop for EndpointPublication {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

impl std::fmt::Display for PersonalDaemonError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BindRefused { detail } => write!(formatter, "bind refused: {detail}"),
            Self::Lifecycle(error) => write!(formatter, "{error}"),
            Self::Auth(error) => write!(formatter, "{error}"),
            Self::Io { detail } => write!(formatter, "personal daemon I/O: {detail}"),
        }
    }
}

impl std::error::Error for PersonalDaemonError {}

/// Serve Personal loopback HTTP with auth, bounds, and single-instance lock.
pub fn serve_personal_loopback(config: PersonalDaemonConfig) -> Result<(), PersonalDaemonError> {
    ensure_loopback_bind(&config.bind_address)?;
    config
        .layout
        .ensure_directories()
        .map_err(|error| PersonalDaemonError::Io {
            detail: error.to_string(),
        })?;
    prepare_personal_databases(&config.layout).map_err(|error| PersonalDaemonError::Io {
        detail: format!("prepare Personal authority databases: {error}"),
    })?;
    let lock = DaemonSingleInstanceLock::acquire(config.layout.daemon_lock_path())
        .map_err(PersonalDaemonError::Lifecycle)?;
    eprintln!(
        "kernel-server personal: acquired single-instance lock at {}",
        lock.path().display()
    );
    reconcile_scheduler_recovery_at_startup(&config.layout.authority_database_path()).map_err(
        |error| PersonalDaemonError::Io {
            detail: format!("reconcile durable scheduler recovery before startup: {error}"),
        },
    )?;
    run_private_scheduler_tick(&config.layout.authority_database_path()).map_err(|error| {
        PersonalDaemonError::Io {
            detail: format!("run private scheduler tick before startup: {error}"),
        }
    })?;
    let bootstrap_path = config.layout.local_bootstrap_secret_path();
    let authority = if bootstrap_path.exists() {
        LocalSessionAuthority::load_existing(&bootstrap_path, config.bounds)
    } else {
        LocalSessionAuthority::initialize(&bootstrap_path, config.bounds)
    }
    .map_err(PersonalDaemonError::Auth)?;
    eprintln!(
        "kernel-server personal: bootstrap secret path {}",
        authority.bootstrap_secret_path().display()
    );
    let _lock = lock;
    let authority = Arc::new(Mutex::new(authority));
    let task_api = Arc::new(Mutex::new(TaskApi::new(config.layout.clone())));
    let resource_api = Arc::new(Mutex::new(ResourceApi::new()));
    let active_connections = Arc::new(AtomicUsize::new(0));
    let in_flight = Arc::new(AtomicUsize::new(0));
    let shutting_down = Arc::new(AtomicBool::new(false));

    let listener = TcpListener::bind(&config.bind_address).map_err(|error| {
        PersonalDaemonError::BindRefused {
            detail: error.to_string(),
        }
    })?;
    let local_address = listener
        .local_addr()
        .map_err(|error| PersonalDaemonError::Io {
            detail: error.to_string(),
        })?;
    let _endpoint_publication = publish_endpoint(&config.layout, local_address.to_string())?;
    eprintln!("kernel-server personal: listening on {local_address} (loopback auth enabled)");

    if config.once {
        let (stream, _) = listener.accept().map_err(|error| PersonalDaemonError::Io {
            detail: error.to_string(),
        })?;
        handle_connection_with_task_api(
            stream,
            &config.bounds,
            &config.layout,
            &authority,
            &task_api,
            &resource_api,
            &active_connections,
            &in_flight,
        );
        if let Ok(mut guard) = authority.lock() {
            guard.revoke_all();
        }
        shutting_down.store(true, Ordering::SeqCst);
        return Ok(());
    }

    for incoming in listener.incoming() {
        if shutting_down.load(Ordering::SeqCst) {
            break;
        }
        match incoming {
            Ok(stream) => {
                let bounds = config.bounds;
                let layout = config.layout.clone();
                let authority = Arc::clone(&authority);
                let task_api = Arc::clone(&task_api);
                let resource_api = Arc::clone(&resource_api);
                let active_connections = Arc::clone(&active_connections);
                let in_flight = Arc::clone(&in_flight);
                let _connection_thread = std::thread::spawn(move || {
                    handle_connection_with_task_api(
                        stream,
                        &bounds,
                        &layout,
                        &authority,
                        &task_api,
                        &resource_api,
                        &active_connections,
                        &in_flight,
                    );
                });
            }
            Err(error) => {
                eprintln!("kernel-server personal accept: {error}");
            }
        }
    }
    Ok(())
}

fn publish_endpoint(
    layout: &PersonalDataLayout,
    endpoint: String,
) -> Result<EndpointPublication, PersonalDaemonError> {
    let endpoint_path = layout.state_dir().join(ENDPOINT_FILE_NAME);
    let temporary_path = endpoint_path.with_file_name(format!(
        ".{ENDPOINT_FILE_NAME}.{}-{}",
        std::process::id(),
        ENDPOINT_TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed),
    ));
    let document = json!({
        "schema_version": 1,
        "endpoint": endpoint,
        "surface": "personal-daemon-endpoint"
    });
    let serialized_document =
        serde_json::to_vec_pretty(&document).map_err(|error| PersonalDaemonError::Io {
            detail: error.to_string(),
        })?;
    let write_result = (|| -> Result<(), std::io::Error> {
        let mut temporary_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)?;
        temporary_file.write_all(&serialized_document)?;
        temporary_file.sync_all()?;
        drop(temporary_file);
        fs::rename(&temporary_path, &endpoint_path)?;
        #[cfg(unix)]
        File::open(layout.state_dir())?.sync_all()?;
        Ok(())
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_file(&temporary_path);
        return Err(PersonalDaemonError::Io {
            detail: format!("unable to publish daemon endpoint: {error}"),
        });
    }
    Ok(EndpointPublication {
        path: endpoint_path,
    })
}

fn ensure_loopback_bind(bind_address: &str) -> Result<(), PersonalDaemonError> {
    let allowed = bind_address.starts_with("127.")
        || bind_address.starts_with("[::1]")
        || bind_address.starts_with("localhost:");
    if !allowed {
        return Err(PersonalDaemonError::BindRefused {
            detail: "personal daemon refuses non-loopback binds".to_owned(),
        });
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)] // Shared daemon state is explicit at the connection boundary.
fn handle_connection_with_task_api(
    mut stream: TcpStream,
    bounds: &PersonalResourceBounds,
    layout: &PersonalDataLayout,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    task_api: &Arc<Mutex<TaskApi>>,
    resource_api: &Arc<Mutex<ResourceApi>>,
    active_connections: &Arc<AtomicUsize>,
    in_flight: &Arc<AtomicUsize>,
) {
    if stream
        .set_read_timeout(Some(Duration::from_secs(bounds.read_header_timeout_secs)))
        .is_err()
    {
        let _ = write_error_response(
            &mut stream,
            500,
            "PERSONAL_SOCKET_TIMEOUT_CONFIGURATION_FAILED",
            "unable to configure request read timeout",
        );
        return;
    }
    let current_connections = active_connections.fetch_add(1, Ordering::SeqCst) + 1;
    if current_connections > bounds.max_concurrent_connections {
        active_connections.fetch_sub(1, Ordering::SeqCst);
        let _ = write_error_response(
            &mut stream,
            429,
            RequestBoundError::ConnectionLimitExceeded.code(),
            "connection limit exceeded",
        );
        return;
    }
    let current_in_flight = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
    if current_in_flight > bounds.max_in_flight_requests {
        in_flight.fetch_sub(1, Ordering::SeqCst);
        active_connections.fetch_sub(1, Ordering::SeqCst);
        let _ = write_error_response(
            &mut stream,
            429,
            RequestBoundError::InFlightLimitExceeded.code(),
            "in-flight request limit exceeded",
        );
        return;
    }

    let result = process_http_request(
        &mut stream,
        bounds,
        layout,
        authority,
        task_api,
        resource_api,
    );
    if let Err(error) = result {
        let (status, code) = if error == "PERSONAL_REQUEST_READ_TIMEOUT" {
            (408, error.as_str())
        } else {
            (400, "PERSONAL_HTTP_PARSE_ERROR")
        };
        let _ = write_error_response(&mut stream, status, code, &error);
    }

    in_flight.fetch_sub(1, Ordering::SeqCst);
    active_connections.fetch_sub(1, Ordering::SeqCst);
}

/// Single-connection test helper. Production keeps one shared TaskApi for
/// process-lifetime watch continuity; pre-existing front-door tests do not
/// exercise that state and use an isolated instance.
#[cfg(test)]
fn handle_connection(
    stream: TcpStream,
    bounds: &PersonalResourceBounds,
    layout: &PersonalDataLayout,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    active_connections: &Arc<AtomicUsize>,
    in_flight: &Arc<AtomicUsize>,
) {
    let task_api = Arc::new(Mutex::new(TaskApi::new(layout.clone())));
    let resource_api = Arc::new(Mutex::new(ResourceApi::new()));
    handle_connection_with_task_api(
        stream,
        bounds,
        layout,
        authority,
        &task_api,
        &resource_api,
        active_connections,
        in_flight,
    );
}

fn process_http_request(
    stream: &mut TcpStream,
    bounds: &PersonalResourceBounds,
    layout: &PersonalDataLayout,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    task_api: &Arc<Mutex<TaskApi>>,
    resource_api: &Arc<Mutex<ResourceApi>>,
) -> Result<(), String> {
    let (request_line, headers, body) = read_bounded_http_request(stream, bounds)?;
    if headers_contain_cookie(&headers) {
        write_error_response(
            stream,
            403,
            LocalAuthError::CookieAuthForbidden.code(),
            "cookie auth forbidden",
        )?;
        return Ok(());
    }
    if let Some(host_error) = validate_host_header(&headers) {
        write_error_response(stream, 400, "LOCAL_HOST_HEADER_REJECTED", host_error)?;
        return Ok(());
    }

    let method_path = parse_request_line(&request_line)?;
    if method_path.starts_with("POST /local/session ") {
        return handle_session_issue(stream, &body, authority);
    }
    if method_path.starts_with("POST /provider/v1/chat/completions ") {
        return handle_provider_proxy_route(stream, &headers, &body, layout, authority);
    }
    if method_path.starts_with("GET /provider/v1/selected-model ") {
        return handle_selected_model_route(stream, &headers, layout, authority);
    }
    if method_path.starts_with("POST /management/") {
        return handle_channel_route(
            stream,
            &headers,
            ChannelClass::Management,
            authority,
            "management",
        );
    }
    if method_path.starts_with("POST /task/") || method_path.starts_with("GET /task/") {
        return handle_task_route(stream, &method_path, &headers, &body, authority, task_api);
    }
    if method_path.starts_with("GET /resource/") {
        return handle_resource_route(stream, &method_path, &headers, authority, resource_api);
    }
    if method_path.starts_with("GET /personal/status ")
        || method_path.starts_with("GET /personal/readiness ")
    {
        return handle_readiness_route(stream, &headers, layout, authority, "status");
    }
    if method_path.starts_with("GET /personal/doctor ") {
        return handle_readiness_route(stream, &headers, layout, authority, "doctor");
    }
    if method_path.starts_with("GET /personal/health ") {
        let body = json!({
            "schema_version": 1,
            "surface": "personal-health",
            "status": "ok",
            "authority_side_effects": false,
            "readiness_claim": "not-claimed",
            "profile_claim": "not-claimed"
        })
        .to_string();
        return write_json_response(stream, 200, &body);
    }

    write_error_response(
        stream,
        404,
        "PERSONAL_ROUTE_NOT_FOUND",
        "no personal route matched",
    )?;
    Ok(())
}

fn handle_session_issue(
    stream: &mut TcpStream,
    body: &[u8],
    authority: &Arc<Mutex<LocalSessionAuthority>>,
) -> Result<(), String> {
    let document = std::str::from_utf8(body).map_err(|_| "session body is not utf-8".to_owned())?;
    let channel_raw = extract_json_string(document, "channel")
        .ok_or_else(|| "channel field required".to_owned())?;
    let principal_id = extract_json_string(document, "principal_id")
        .ok_or_else(|| "principal_id field required".to_owned())?;
    let bootstrap_secret = extract_json_string(document, "bootstrap_secret")
        .ok_or_else(|| "bootstrap_secret field required".to_owned())?;
    let channel = ChannelClass::parse(&channel_raw)
        .ok_or_else(|| "channel must be task or management".to_owned())?;

    let mut guard = authority
        .lock()
        .map_err(|_| "session authority lock poisoned".to_owned())?;
    match guard.issue_session(
        SessionIssueRequest {
            channel,
            principal_id,
            bootstrap_secret,
        },
        Instant::now(),
    ) {
        Ok(view) => {
            let response = json!({
                "status": "ok",
                "token": view.token,
                "channel": view.channel.as_str(),
                "session_id": view.session_id,
                "absolute_expiry_secs": view.absolute_expiry_secs_from_now,
                "idle_expiry_secs": view.idle_expiry_secs_from_now
            })
            .to_string();
            write_json_response(stream, 200, &response)
        }
        Err(error) => write_error_response(stream, 401, error.code(), &error.to_string()),
    }
}

fn handle_channel_route(
    stream: &mut TcpStream,
    headers: &str,
    required_channel: ChannelClass,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    channel_label: &str,
) -> Result<(), String> {
    let Some(token) = extract_bearer_token(headers) else {
        return write_error_response(
            stream,
            401,
            LocalAuthError::Unauthorized.code(),
            "authorization bearer required",
        );
    };
    let mut guard = authority
        .lock()
        .map_err(|_| "session authority lock poisoned".to_owned())?;
    match guard.authorize(&token, required_channel, Instant::now()) {
        Ok(()) => {
            let response = json!({
                "status": "ok",
                "channel": channel_label,
                "authority_side_effects": false,
                "note": "authenticated personal front door; business routes deferred"
            })
            .to_string();
            write_json_response(stream, 200, &response)
        }
        Err(error) => {
            let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
                403
            } else {
                401
            };
            write_error_response(stream, status, error.code(), &error.to_string())
        }
    }
}

fn handle_task_route(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    body: &[u8],
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    task_api: &Arc<Mutex<TaskApi>>,
) -> Result<(), String> {
    let Some(token) = extract_bearer_token(headers) else {
        return write_error_response(
            stream,
            401,
            LocalAuthError::Unauthorized.code(),
            "authorization bearer required",
        );
    };
    let principal = match authority
        .lock()
        .map_err(|_| "session authority lock poisoned".to_owned())?
        .authorize_principal(&token, ChannelClass::Task, Instant::now())
    {
        Ok(principal) => principal,
        Err(error) => {
            let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
                403
            } else {
                401
            };
            return write_error_response(stream, status, error.code(), &error.to_string());
        }
    };
    let response = task_api
        .lock()
        .map_err(|_| "task API lock poisoned".to_owned())?
        .handle(method_path, body, &principal);
    write_response(
        stream,
        response.status,
        response.content_type,
        response.body.as_bytes(),
    )
}

fn handle_resource_route(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    resource_api: &Arc<Mutex<ResourceApi>>,
) -> Result<(), String> {
    let Some(token) = extract_bearer_token(headers) else {
        return write_error_response(
            stream,
            401,
            LocalAuthError::Unauthorized.code(),
            "authorization bearer required",
        );
    };
    let mut authority_guard = authority
        .lock()
        .map_err(|_| "session authority lock poisoned".to_owned())?;
    if let Err(error) = authority_guard.authorize(&token, ChannelClass::Management, Instant::now())
    {
        let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
            403
        } else {
            401
        };
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    drop(authority_guard);
    let response = resource_api
        .lock()
        .map_err(|_| "resource projection lock poisoned".to_owned())?
        .handle(method_path);
    write_response(
        stream,
        response.status,
        response.content_type,
        response.body.as_bytes(),
    )
}

fn handle_provider_proxy_route(
    stream: &mut TcpStream,
    headers: &str,
    request_body: &[u8],
    layout: &PersonalDataLayout,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
) -> Result<(), String> {
    let Some(token) = extract_bearer_token(headers) else {
        return write_error_response(
            stream,
            401,
            LocalAuthError::Unauthorized.code(),
            "authorization bearer required",
        );
    };
    let mut authority_guard = authority
        .lock()
        .map_err(|_| "session authority lock poisoned".to_owned())?;
    if let Err(error) = authority_guard.authorize(&token, ChannelClass::Management, Instant::now())
    {
        let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
            403
        } else {
            401
        };
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    drop(authority_guard);

    let secret_backend = select_production_secret_store();
    let transport = RustlsProviderTransport::default();
    let service = ProviderProxyService::new(
        secret_backend.as_secret_store(),
        ProviderConfigRepository::under_config_dir(layout.config_dir()),
        &transport,
    );
    match service.forward_chat_completion(request_body) {
        Ok(response) => write_provider_response(stream, response.status, &response.body),
        Err(error) => write_error_response(
            stream,
            error.status_code(),
            error.code(),
            "provider proxy request was not completed",
        ),
    }
}

fn handle_selected_model_route(
    stream: &mut TcpStream,
    headers: &str,
    layout: &PersonalDataLayout,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
) -> Result<(), String> {
    let Some(token) = extract_bearer_token(headers) else {
        return write_error_response(
            stream,
            401,
            LocalAuthError::Unauthorized.code(),
            "authorization bearer required",
        );
    };
    let mut authority_guard = authority
        .lock()
        .map_err(|_| "session authority lock poisoned".to_owned())?;
    if let Err(error) = authority_guard.authorize(&token, ChannelClass::Management, Instant::now())
    {
        let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
            403
        } else {
            401
        };
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    drop(authority_guard);

    // This route reads only the dedicated non-secret carrier. It never creates
    // a SecretStore or resolves Provider material.
    match SelectedModelRepository::under_config_dir(layout.config_dir()).load() {
        Ok(Some(selected_model)) => write_json_response(
            stream,
            200,
            &json!({
                "schema_version": 1,
                "surface": "personal-provider-selected-model",
                "selected_model": selected_model.model_id(),
                "selected_snapshot_digest": selected_model.selected_snapshot_digest(),
                "chat_capable": selected_model.chat_capable(),
                "authority_side_effects": false,
            })
            .to_string(),
        ),
        Ok(None) => write_error_response(
            stream,
            503,
            "PERSONAL_PROVIDER_SELECTED_MODEL_UNAVAILABLE",
            "selected model state is unavailable",
        ),
        Err(_) => write_error_response(
            stream,
            503,
            "PERSONAL_PROVIDER_SELECTED_MODEL_UNAVAILABLE",
            "selected model state is unavailable",
        ),
    }
}

fn handle_readiness_route(
    stream: &mut TcpStream,
    headers: &str,
    layout: &PersonalDataLayout,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    surface: &str,
) -> Result<(), String> {
    let Some(token) = extract_bearer_token(headers) else {
        return write_error_response(
            stream,
            401,
            LocalAuthError::Unauthorized.code(),
            "authorization bearer required",
        );
    };
    let mut guard = authority
        .lock()
        .map_err(|_| "session authority lock poisoned".to_owned())?;
    match guard.authorize(&token, ChannelClass::Management, Instant::now()) {
        Ok(()) => {
            let session_count = guard.session_count();
            drop(guard);
            let report = evaluate_personal_readiness(&ReadinessEvaluationContext {
                layout: layout.clone(),
                daemon_listening: true,
                session_count,
                secret_probe_override: None,
                provider_config_path_override: None,
                pi_observation_override: None,
            });
            let body = if surface == "doctor" {
                doctor_projection_json(&report).to_string()
            } else {
                status_projection_json(&report).to_string()
            };
            write_json_response(stream, 200, &body)
        }
        Err(error) => {
            let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
                403
            } else {
                401
            };
            write_error_response(stream, status, error.code(), &error.to_string())
        }
    }
}

fn read_bounded_http_request(
    stream: &mut TcpStream,
    bounds: &PersonalResourceBounds,
) -> Result<(String, String, Vec<u8>), String> {
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 4096];
    let hard_read_ceiling = bounds
        .hard_body_ceiling_bytes
        .saturating_add(bounds.max_header_block_bytes)
        .saturating_add(1024);
    loop {
        let read = stream.read(&mut chunk).map_err(map_request_read_error)?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..read]);
        if bytes.len() > hard_read_ceiling {
            return Err("request exceeded hard read ceiling".to_owned());
        }
        if let Some(split) = find_bytes(&bytes, b"\r\n\r\n") {
            let head = &bytes[..split];
            let head_text = String::from_utf8_lossy(head);
            let mut lines = head_text.lines();
            let request_line = lines
                .next()
                .ok_or_else(|| "missing request line".to_owned())?
                .to_owned();
            let header_block_start = request_line.len() + 2;
            let header_block = if head.len() >= header_block_start {
                &head[header_block_start..]
            } else {
                &[]
            };
            validate_header_block(header_block, bounds).map_err(|error| error.code().to_owned())?;
            let headers = String::from_utf8_lossy(header_block).into_owned();
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let lower = line.to_ascii_lowercase();
                    lower
                        .strip_prefix("content-length:")
                        .map(str::trim)
                        .and_then(|value| value.parse::<usize>().ok())
                })
                .unwrap_or(0);
            validate_body_length(content_length, bounds)
                .map_err(|error| error.code().to_owned())?;
            stream
                .set_read_timeout(Some(Duration::from_secs(
                    bounds.request_body_read_timeout_secs,
                )))
                .map_err(|error| error.to_string())?;
            let body_start = split + 4;
            while bytes.len() < body_start + content_length {
                let read = stream.read(&mut chunk).map_err(map_request_read_error)?;
                if read == 0 {
                    break;
                }
                bytes.extend_from_slice(&chunk[..read]);
                if bytes.len() > hard_read_ceiling {
                    return Err("request exceeded hard read ceiling while reading body".to_owned());
                }
            }
            if bytes.len() < body_start + content_length {
                return Err("incomplete request body".to_owned());
            }
            let body = bytes[body_start..body_start + content_length].to_vec();
            return Ok((request_line, headers, body));
        }
    }
    Err("malformed HTTP request".to_owned())
}

fn map_request_read_error(error: std::io::Error) -> String {
    match error.kind() {
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => {
            "PERSONAL_REQUEST_READ_TIMEOUT".to_owned()
        }
        _ => error.to_string(),
    }
}

fn parse_request_line(request_line: &str) -> Result<String, String> {
    let mut parts = request_line.split_whitespace();
    let method = parts.next().ok_or_else(|| "missing method".to_owned())?;
    let path = parts.next().ok_or_else(|| "missing path".to_owned())?;
    Ok(format!("{method} {path} "))
}

fn headers_contain_cookie(headers: &str) -> bool {
    headers.lines().any(|line| {
        let lower = line.to_ascii_lowercase();
        lower.starts_with("cookie:")
    })
}

fn validate_host_header(headers: &str) -> Option<&'static str> {
    let host = headers.lines().find_map(|line| {
        let lower = line.to_ascii_lowercase();
        lower
            .strip_prefix("host:")
            .map(|value| value.trim().to_owned())
    })?;
    let host_without_port = host
        .split(':')
        .next()
        .unwrap_or(host.as_str())
        .trim_matches(|character| character == '[' || character == ']');
    let allowed = matches!(
        host_without_port,
        "127.0.0.1" | "localhost" | "::1" | "localhost."
    );
    if allowed {
        None
    } else {
        Some("host header must be loopback")
    }
}

fn extract_bearer_token(headers: &str) -> Option<String> {
    for line in headers.lines() {
        let Some((_, value)) = line.split_once(':') else {
            continue;
        };
        if !line.to_ascii_lowercase().starts_with("authorization:") {
            continue;
        }
        let value = value.trim();
        if let Some(token) = value
            .strip_prefix("Bearer ")
            .or_else(|| value.strip_prefix("bearer "))
        {
            return Some(token.trim().to_owned());
        }
    }
    None
}

fn extract_json_string(document: &str, field_name: &str) -> Option<String> {
    let pattern = format!("\"{field_name}\"");
    let field_offset = document.find(&pattern)?;
    let after_field = &document[field_offset + pattern.len()..];
    let colon = after_field.find(':')?;
    let after_colon = after_field[colon + 1..].trim_start();
    if !after_colon.starts_with('"') {
        return None;
    }
    let mut value = String::new();
    let mut chars = after_colon[1..].chars();
    while let Some(character) = chars.next() {
        match character {
            '"' => return Some(value),
            '\\' => {
                if let Some(escaped) = chars.next() {
                    value.push(escaped);
                }
            }
            other => value.push(other),
        }
    }
    None
}

fn write_json_response(stream: &mut TcpStream, status: u16, body: &str) -> Result<(), String> {
    write_json_bytes_response(stream, status, body.as_bytes())
}

fn write_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        409 => "Conflict",
        503 => "Service Unavailable",
        _ => "Error",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(header.as_bytes())
        .and_then(|()| stream.write_all(body))
        .map_err(|error| error.to_string())
}

fn write_provider_response(stream: &mut TcpStream, status: u16, body: &[u8]) -> Result<(), String> {
    write_json_bytes_response(stream, status, body)
}

fn write_json_bytes_response(
    stream: &mut TcpStream,
    status: u16,
    body: &[u8],
) -> Result<(), String> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        408 => "Request Timeout",
        429 => "Too Many Requests",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Error",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(header.as_bytes())
        .and_then(|()| stream.write_all(body))
        .map_err(|error| error.to_string())
}

fn write_error_response(
    stream: &mut TcpStream,
    status: u16,
    code: &str,
    message: &str,
) -> Result<(), String> {
    let body = json!({
        "status": "error",
        "error": {
            "code": code,
            "message": message,
            "category": "protocol",
            "retryable": false,
            "stage": "personal-front-door"
        }
    })
    .to_string();
    write_json_response(stream, status, &body)
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
mod tests {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use cognitive_store::PersonalDataLayout;

    use super::{
        LocalSessionAuthority, PersonalResourceBounds, ensure_loopback_bind, handle_connection,
    };

    fn test_fixture(test_name: &str) -> (PersonalDataLayout, Arc<Mutex<LocalSessionAuthority>>) {
        let temporary_root = std::env::temp_dir().join(format!(
            "cos-personal-server-test-{test_name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0),
        ));
        let layout = PersonalDataLayout::from_xdg_roots(
            &temporary_root,
            &temporary_root,
            &temporary_root,
            &temporary_root,
            &temporary_root,
        );
        layout.ensure_directories().expect("test directories");
        let authority = LocalSessionAuthority::initialize(
            layout.local_bootstrap_secret_path(),
            PersonalResourceBounds::personal_v1_baseline(),
        )
        .expect("test authority");
        (layout, Arc::new(Mutex::new(authority)))
    }

    fn accept_connection(listener: &TcpListener) -> TcpStream {
        listener.accept().expect("accepted test connection").0
    }

    fn wait_for_count(counter: &AtomicUsize, expected: usize) {
        for _ in 0..200 {
            if counter.load(Ordering::SeqCst) == expected {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("counter did not reach {expected}");
    }

    #[test]
    fn non_loopback_bind_is_rejected() {
        assert!(ensure_loopback_bind("0.0.0.0:8080").is_err());
        assert!(ensure_loopback_bind("127.0.0.1:0").is_ok());
    }

    #[test]
    fn slow_header_read_times_out_with_stable_protocol_code() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test listener");
        let port = listener.local_addr().expect("listener address").port();
        let mut bounds = PersonalResourceBounds::personal_v1_baseline();
        bounds.read_header_timeout_secs = 1;
        let (layout, authority) = test_fixture("timeout");
        let active_connections = Arc::new(AtomicUsize::new(0));
        let in_flight = Arc::new(AtomicUsize::new(0));
        let server = std::thread::spawn({
            let authority = Arc::clone(&authority);
            let active_connections = Arc::clone(&active_connections);
            let in_flight = Arc::clone(&in_flight);
            move || {
                handle_connection(
                    accept_connection(&listener),
                    &bounds,
                    &layout,
                    &authority,
                    &active_connections,
                    &in_flight,
                );
            }
        });

        let mut client = TcpStream::connect(("127.0.0.1", port)).expect("client connection");
        client
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("client timeout");
        client
            .write_all(b"GET /personal/health HTTP/1.1\r\nHost: 127.0.0.1")
            .expect("partial header");
        let mut response = String::new();
        client
            .read_to_string(&mut response)
            .expect("timeout response");
        server.join().expect("server thread");

        assert!(
            response.contains("PERSONAL_REQUEST_READ_TIMEOUT"),
            "{response}"
        );
        assert_eq!(
            active_connections.load(std::sync::atomic::Ordering::SeqCst),
            0
        );
        assert_eq!(in_flight.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[test]
    fn slow_body_read_times_out_with_stable_protocol_code() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test listener");
        let port = listener.local_addr().expect("listener address").port();
        let mut bounds = PersonalResourceBounds::personal_v1_baseline();
        bounds.read_header_timeout_secs = 1;
        bounds.request_body_read_timeout_secs = 1;
        let (layout, authority) = test_fixture("body-timeout");
        let active_connections = Arc::new(AtomicUsize::new(0));
        let in_flight = Arc::new(AtomicUsize::new(0));
        let server = std::thread::spawn({
            let authority = Arc::clone(&authority);
            let active_connections = Arc::clone(&active_connections);
            let in_flight = Arc::clone(&in_flight);
            move || {
                handle_connection(
                    accept_connection(&listener),
                    &bounds,
                    &layout,
                    &authority,
                    &active_connections,
                    &in_flight,
                );
            }
        });

        let mut client = TcpStream::connect(("127.0.0.1", port)).expect("client connection");
        client
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("client timeout");
        client
            .write_all(
                b"POST /local/session HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 4\r\n\r\n",
            )
            .expect("headers without body");
        let mut response = String::new();
        client
            .read_to_string(&mut response)
            .expect("timeout response");
        server.join().expect("server thread");

        assert!(
            response.contains("PERSONAL_REQUEST_READ_TIMEOUT"),
            "{response}"
        );
        assert_eq!(active_connections.load(Ordering::SeqCst), 0);
        assert_eq!(in_flight.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn concurrent_connection_limit_rejects_excess_connection() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test listener");
        let port = listener.local_addr().expect("listener address").port();
        let mut bounds = PersonalResourceBounds::personal_v1_baseline();
        bounds.max_concurrent_connections = 2;
        // Keep in-flight high so the third connection is rejected by the
        // connection ceiling, not the in-flight ceiling.
        bounds.max_in_flight_requests = 8;
        bounds.read_header_timeout_secs = 1;
        let (layout, authority) = test_fixture("concurrency");
        let active_connections = Arc::new(AtomicUsize::new(0));
        let in_flight = Arc::new(AtomicUsize::new(0));
        let mut first = TcpStream::connect(("127.0.0.1", port)).expect("first connection");
        let first_listener = listener.try_clone().expect("first listener clone");
        let first_server = std::thread::spawn({
            let authority = Arc::clone(&authority);
            let layout = layout.clone();
            let active_connections = Arc::clone(&active_connections);
            let in_flight = Arc::clone(&in_flight);
            let bounds = bounds;
            move || {
                handle_connection(
                    accept_connection(&first_listener),
                    &bounds,
                    &layout,
                    &authority,
                    &active_connections,
                    &in_flight,
                );
            }
        });
        first
            .write_all(b"GET /personal/health HTTP/1.1\r\n")
            .expect("first partial header");
        wait_for_count(&active_connections, 1);

        let mut second = TcpStream::connect(("127.0.0.1", port)).expect("second connection");
        let second_listener = listener.try_clone().expect("second listener clone");
        let second_server = std::thread::spawn({
            let authority = Arc::clone(&authority);
            let layout = layout.clone();
            let active_connections = Arc::clone(&active_connections);
            let in_flight = Arc::clone(&in_flight);
            let bounds = bounds;
            move || {
                handle_connection(
                    accept_connection(&second_listener),
                    &bounds,
                    &layout,
                    &authority,
                    &active_connections,
                    &in_flight,
                );
            }
        });
        second
            .write_all(b"GET /personal/health HTTP/1.1\r\n")
            .expect("second partial header");
        wait_for_count(&active_connections, 2);

        let mut third = TcpStream::connect(("127.0.0.1", port)).expect("third connection");
        let third_listener = listener.try_clone().expect("third listener clone");
        let third_server = std::thread::spawn({
            let authority = Arc::clone(&authority);
            let layout = layout.clone();
            let active_connections = Arc::clone(&active_connections);
            let in_flight = Arc::clone(&in_flight);
            let bounds = bounds;
            move || {
                handle_connection(
                    accept_connection(&third_listener),
                    &bounds,
                    &layout,
                    &authority,
                    &active_connections,
                    &in_flight,
                );
            }
        });
        third
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("third timeout");

        let mut response = String::new();
        third.read_to_string(&mut response).expect("limit response");
        assert!(response.contains("CONNECTION_LIMIT_EXCEEDED"), "{response}");

        drop(first);
        drop(second);
        first_server.join().expect("first server thread");
        second_server.join().expect("second server thread");
        third_server.join().expect("third server thread");
        assert_eq!(active_connections.load(Ordering::SeqCst), 0);
        assert_eq!(in_flight.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn in_flight_request_limit_rejects_excess_request() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test listener");
        let port = listener.local_addr().expect("listener address").port();
        let mut bounds = PersonalResourceBounds::personal_v1_baseline();
        bounds.max_concurrent_connections = 3;
        bounds.max_in_flight_requests = 2;
        bounds.read_header_timeout_secs = 1;
        let (layout, authority) = test_fixture("in-flight");
        let active_connections = Arc::new(AtomicUsize::new(0));
        let in_flight = Arc::new(AtomicUsize::new(0));

        let mut first = TcpStream::connect(("127.0.0.1", port)).expect("first connection");
        let first_listener = listener.try_clone().expect("first listener clone");
        let first_server = std::thread::spawn({
            let authority = Arc::clone(&authority);
            let layout = layout.clone();
            let active_connections = Arc::clone(&active_connections);
            let in_flight = Arc::clone(&in_flight);
            let bounds = bounds;
            move || {
                handle_connection(
                    accept_connection(&first_listener),
                    &bounds,
                    &layout,
                    &authority,
                    &active_connections,
                    &in_flight,
                );
            }
        });
        first
            .write_all(b"GET /personal/health HTTP/1.1\r\n")
            .expect("first partial header");
        wait_for_count(&in_flight, 1);

        let mut second = TcpStream::connect(("127.0.0.1", port)).expect("second connection");
        let second_listener = listener.try_clone().expect("second listener clone");
        let second_server = std::thread::spawn({
            let authority = Arc::clone(&authority);
            let layout = layout.clone();
            let active_connections = Arc::clone(&active_connections);
            let in_flight = Arc::clone(&in_flight);
            let bounds = bounds;
            move || {
                handle_connection(
                    accept_connection(&second_listener),
                    &bounds,
                    &layout,
                    &authority,
                    &active_connections,
                    &in_flight,
                );
            }
        });
        second
            .write_all(b"GET /personal/health HTTP/1.1\r\n")
            .expect("second partial header");
        wait_for_count(&in_flight, 2);

        let mut third = TcpStream::connect(("127.0.0.1", port)).expect("third connection");
        let third_listener = listener.try_clone().expect("third listener clone");
        let third_server = std::thread::spawn({
            let authority = Arc::clone(&authority);
            let layout = layout.clone();
            let active_connections = Arc::clone(&active_connections);
            let in_flight = Arc::clone(&in_flight);
            let bounds = bounds;
            move || {
                handle_connection(
                    accept_connection(&third_listener),
                    &bounds,
                    &layout,
                    &authority,
                    &active_connections,
                    &in_flight,
                );
            }
        });
        third
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("third timeout");

        let mut response = String::new();
        third.read_to_string(&mut response).expect("limit response");
        assert!(response.contains("IN_FLIGHT_LIMIT_EXCEEDED"), "{response}");

        drop(first);
        drop(second);
        first_server.join().expect("first server thread");
        second_server.join().expect("second server thread");
        third_server.join().expect("third server thread");
        assert_eq!(active_connections.load(Ordering::SeqCst), 0);
        assert_eq!(in_flight.load(Ordering::SeqCst), 0);
    }
}
