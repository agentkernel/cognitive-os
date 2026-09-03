//! Bounded loopback Personal HTTP front door (P1-T04 / ADR-0019).

#[cfg(unix)]
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use cognitive_kernel::ports::{
    ContextAuthorizationFactStore, ContextAuthorizationFactsRow, ContextRevocationFactRow,
    ProtocolStore,
};
use cognitive_runtime::DSH_PACKAGE_REVISION;
use cognitive_runtime::loopback_transport::{self, LoopbackTransportStage};
use cognitive_secret::{
    ProviderConfigRepository, SelectedModel, SelectedModelRepository,
    select_production_secret_store,
};
use cognitive_store::{
    HostedDshAttemptStore, PersonalDataLayout, ProviderControlPlaneStore, SqliteAuthorityStore,
    prepare_personal_databases, scheduler::SchedulerRepository,
};
use serde_json::json;

use super::assistant_inference;
use super::attempt_artifacts;
use super::auth::{ChannelClass, LocalAuthError, LocalSessionAuthority, SessionIssueRequest};
use super::bounds::{
    PersonalResourceBounds, RequestBoundError, validate_body_length, validate_header_block,
};
use super::fault_profile;
use super::hosted_dsh_attempt;
use super::lifecycle::{DaemonLifecycleError, DaemonSingleInstanceLock};
use super::pinned_https;
use super::project_aggregate;
use super::provider_control_plane;
use super::provider_proxy::{
    ProviderProxyError, ProviderProxyService, RustlsProviderTransport, SseToolCallNormalizer,
};
use super::readiness::{
    ReadinessEvaluationContext, doctor_projection_json, evaluate_personal_readiness,
    status_projection_json,
};
use super::resource_api::ResourceApi;
use super::resource_manager;
use super::route_observation;
use super::routine_runs;
use super::scheduler_authority::{
    reconcile_scheduler_recovery_with_store, run_private_scheduler_tick_with_store,
};
use super::settings_connections;
use super::task_api::TaskApi;
use super::tool_executor::{ProductionNativeToolExecutorRouter, ensure_builtin_native_descriptors};
use super::tool_lifecycle;
use super::user_backup;
use super::verification_executor::open_daemon_artifact_store;
use super::windows_host;
use super::x_connector;

const ENDPOINT_FILE_NAME: &str = "daemon-endpoint.json";
const SCHEDULER_TICK_INTERVAL: Duration = Duration::from_millis(250);
const WEB_UI_CSP: &str =
    "default-src 'self'; object-src 'none'; base-uri 'self'; form-action 'self'";
const WEB_UI_MAX_ASSET_BYTES: u64 = 1024 * 1024;
const WEB_UI_UNAVAILABLE_HTML: &str = concat!(
    "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">",
    "<title>CognitiveOS Personal UI</title></head><body>",
    "<p>Web UI bundle is not_available (LOCAL_UI_BUNDLE_UNAVAILABLE). ",
    "This origin serves GET /ui only after a pinned cognitiveos-clients/pc/web ",
    "build is installed. This is not a Gate, release, Profile, or readiness claim.</p>",
    "</body></html>",
);
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

#[derive(Debug, PartialEq, Eq)]
enum SchedulerTickRun<T> {
    Executed(T),
    AlreadyRunning,
}

struct SchedulerTickActiveGuard<'a> {
    active: &'a AtomicBool,
}

impl Drop for SchedulerTickActiveGuard<'_> {
    fn drop(&mut self) {
        self.active.store(false, Ordering::Release);
    }
}

fn run_scheduler_tick_non_reentrant<T>(
    active: &AtomicBool,
    tick: impl FnOnce() -> T,
) -> SchedulerTickRun<T> {
    if active
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return SchedulerTickRun::AlreadyRunning;
    }
    let _active_guard = SchedulerTickActiveGuard { active };
    SchedulerTickRun::Executed(tick())
}

struct PeriodicSchedulerWorker {
    cancellation_requested: Arc<AtomicBool>,
    worker_thread: Option<JoinHandle<()>>,
}

impl PeriodicSchedulerWorker {
    fn spawn<Tick, Error>(interval: Duration, mut tick: Tick) -> Result<Self, std::io::Error>
    where
        Tick: FnMut() -> Result<(), Error> + Send + 'static,
        Error: std::fmt::Display,
    {
        if interval.is_zero() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "scheduler tick interval must be non-zero",
            ));
        }
        let cancellation_requested = Arc::new(AtomicBool::new(false));
        let worker_cancellation = Arc::clone(&cancellation_requested);
        let tick_active = AtomicBool::new(false);
        let worker_thread = std::thread::Builder::new()
            .name("personal-scheduler-tick".to_owned())
            .spawn(move || {
                while !worker_cancellation.load(Ordering::Acquire) {
                    match run_scheduler_tick_non_reentrant(&tick_active, &mut tick) {
                        SchedulerTickRun::Executed(Ok(())) => {}
                        SchedulerTickRun::Executed(Err(error)) => {
                            eprintln!(
                                "kernel-server personal scheduler tick: pass failed and will retry: {error}"
                            );
                        }
                        SchedulerTickRun::AlreadyRunning => {
                            eprintln!(
                                "kernel-server personal scheduler tick: skipped reentrant pass"
                            );
                        }
                    }
                    if worker_cancellation.load(Ordering::Acquire) {
                        break;
                    }
                    std::thread::park_timeout(interval);
                }
            })?;
        Ok(Self {
            cancellation_requested,
            worker_thread: Some(worker_thread),
        })
    }

    fn shutdown(&mut self) -> Result<(), String> {
        self.cancellation_requested.store(true, Ordering::Release);
        let Some(worker_thread) = self.worker_thread.take() else {
            return Ok(());
        };
        worker_thread.thread().unpark();
        worker_thread
            .join()
            .map_err(|_| "personal scheduler worker panicked during shutdown".to_owned())
    }
}

impl Drop for PeriodicSchedulerWorker {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

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
    // Open the authority store once for the daemon process and reuse the
    // single-writer handle for startup recovery, periodic scheduler ticks, and
    // personal request-path handlers (P9-T03/D01-D02).
    let authority_store = SqliteAuthorityStore::open(&config.layout.authority_database_path())
        .map_err(|error| PersonalDaemonError::Io {
            detail: format!("open Personal authority store for daemon startup: {error}"),
        })?;
    let mut scheduler_repository =
        SchedulerRepository::open(&config.layout.authority_database_path()).map_err(|error| {
            PersonalDaemonError::Io {
                detail: format!("open Personal scheduler repository for daemon startup: {error}"),
            }
        })?;
    reconcile_scheduler_recovery_with_store(&authority_store, &mut scheduler_repository).map_err(
        |error| PersonalDaemonError::Io {
            detail: format!("reconcile durable scheduler recovery before startup: {error}"),
        },
    )?;
    // Retain the single-writer authority store for the daemon accept loop
    // and scheduler worker (P9-T03/D02). Request handlers must reuse this
    // handle instead of opening a second connection per call.
    let authority_store = Arc::new(authority_store);
    ensure_builtin_native_descriptors(authority_store.as_ref()).map_err(|error| {
        PersonalDaemonError::Io {
            detail: format!("publish immutable native Tool descriptors: {error}"),
        }
    })?;
    // Hosted DSH Attempts left `persisted`/`dispatched` by a previous daemon
    // process reconcile to `unknown-outcome` (A3); they never become success.
    let reconciled_attempts = HostedDshAttemptStore::from_authority_store(authority_store.as_ref())
        .reconcile_unknown_outcomes(cognitive_store::now_ms())
        .map_err(|error| PersonalDaemonError::Io {
            detail: format!("reconcile hosted DSH attempts before startup: {error}"),
        })?;
    if !reconciled_attempts.is_empty() {
        eprintln!(
            "kernel-server personal: {} hosted DSH attempt(s) reconciled to unknown-outcome",
            reconciled_attempts.len()
        );
    }
    let artifact_store = Arc::new(open_daemon_artifact_store(&config.layout).map_err(|error| {
        PersonalDaemonError::Io {
            detail: format!("assemble daemon ArtifactStore: {error}"),
        }
    })?);
    let mut executor_router = ProductionNativeToolExecutorRouter::open_with_artifact_store(
        authority_store
            .current_fencing_epoch()
            .map_err(|error| PersonalDaemonError::Io {
                detail: format!("load native Tool executor fencing epoch: {error}"),
            })?,
        config.layout.data_dir().join("workspace"),
        artifact_store.as_ref().clone(),
    )
    .map_err(|error| PersonalDaemonError::Io {
        detail: format!("assemble native Tool executor router: {error}"),
    })?;
    executor_router.bind_fault_profiles(config.layout.data_dir().to_path_buf());
    executor_router.bind_origin_registry(config.layout.data_dir().to_path_buf());
    let executor_router = Arc::new(executor_router);
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
    let task_api = Arc::new(Mutex::new(TaskApi::with_shared_store(
        config.layout.clone(),
        Arc::clone(&authority_store),
    )));
    crate::personal::observation::bind_observation_store(config.layout.data_dir().to_path_buf());
    let resource_api = Arc::new(Mutex::new(ResourceApi::with_governance_data_dir(Some(
        config.layout.data_dir().to_path_buf(),
    ))));
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
    let scheduler_authority_store = Arc::clone(&authority_store);
    let scheduler_executor_router = Arc::clone(&executor_router);
    let scheduler_artifact_store = Arc::clone(&artifact_store);
    let scheduler_config_dir = config.layout.config_dir().to_path_buf();
    // P13-T05: the same daemon tick is the only dispatcher of Routine
    // occurrences (`task://personal/routine/*`); there is no second scheduler.
    let routine_host = hosted_dsh_attempt::HostedAttemptHost::from_layout(&config.layout);
    let mut scheduler_worker = PeriodicSchedulerWorker::spawn(SCHEDULER_TICK_INTERVAL, move || {
        run_private_scheduler_tick_with_store(
            scheduler_authority_store.as_ref(),
            &mut scheduler_repository,
            &scheduler_config_dir,
            scheduler_executor_router.as_ref(),
            scheduler_artifact_store.as_ref(),
        )
        .map_err(|error| error.to_string())?;
        let report = routine_runs::run_routine_tick(
            scheduler_authority_store.as_ref(),
            &routine_host,
            &mut scheduler_repository,
            cognitive_store::now_ms(),
        )?;
        if !report.is_quiet() {
            eprintln!("kernel-server personal routine tick: {report:?}");
        }
        Ok::<(), String>(())
    })
    .map_err(|error| PersonalDaemonError::Io {
        detail: format!("start periodic Personal scheduler worker: {error}"),
    })?;

    if config.once {
        let (stream, _) = listener.accept().map_err(|error| PersonalDaemonError::Io {
            detail: error.to_string(),
        })?;
        handle_connection_with_task_api(
            stream,
            &config.bounds,
            &config.layout,
            &authority,
            &authority_store,
            &task_api,
            &resource_api,
            &active_connections,
            &in_flight,
        );
        if let Ok(mut guard) = authority.lock() {
            guard.revoke_all();
        }
        shutting_down.store(true, Ordering::SeqCst);
        scheduler_worker
            .shutdown()
            .map_err(|detail| PersonalDaemonError::Io { detail })?;
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
                let authority_store = Arc::clone(&authority_store);
                let active_connections = Arc::clone(&active_connections);
                let in_flight = Arc::clone(&in_flight);
                let _connection_thread = std::thread::spawn(move || {
                    handle_connection_with_task_api(
                        stream,
                        &bounds,
                        &layout,
                        &authority,
                        &authority_store,
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
    shutting_down.store(true, Ordering::SeqCst);
    scheduler_worker
        .shutdown()
        .map_err(|detail| PersonalDaemonError::Io { detail })?;
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
    authority_store: &Arc<SqliteAuthorityStore>,
    task_api: &Arc<Mutex<TaskApi>>,
    resource_api: &Arc<Mutex<ResourceApi>>,
    active_connections: &Arc<AtomicUsize>,
    in_flight: &Arc<AtomicUsize>,
) {
    loopback_transport::begin_connection();
    let connection_admission_started = Instant::now();
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
    loopback_transport::record_stage(
        LoopbackTransportStage::ConnectionAdmission,
        connection_admission_started.elapsed().as_nanos(),
    );

    let result = process_http_request(
        &mut stream,
        bounds,
        layout,
        authority,
        authority_store,
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
    let _ = loopback_transport::finish_connection();
}

/// Single-connection test helper. Production keeps one shared TaskApi for
/// process-lifetime watch continuity and binds the daemon-owned authority
/// store so HTTP admit and the scheduler tick share one writer.
#[cfg(test)]
fn handle_connection(
    stream: TcpStream,
    bounds: &PersonalResourceBounds,
    layout: &PersonalDataLayout,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    authority_store: &Arc<SqliteAuthorityStore>,
    active_connections: &Arc<AtomicUsize>,
    in_flight: &Arc<AtomicUsize>,
) {
    let task_api = Arc::new(Mutex::new(TaskApi::with_shared_store(
        layout.clone(),
        Arc::clone(authority_store),
    )));
    let resource_api = Arc::new(Mutex::new(ResourceApi::with_governance_data_dir(Some(
        layout.data_dir().to_path_buf(),
    ))));
    handle_connection_with_task_api(
        stream,
        bounds,
        layout,
        authority,
        authority_store,
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
    authority_store: &Arc<SqliteAuthorityStore>,
    task_api: &Arc<Mutex<TaskApi>>,
    resource_api: &Arc<Mutex<ResourceApi>>,
) -> Result<(), String> {
    let request_read_started = Instant::now();
    let (request_line, headers, body) = read_bounded_http_request(stream, bounds)?;
    loopback_transport::record_stage(
        LoopbackTransportStage::RequestRead,
        request_read_started.elapsed().as_nanos(),
    );
    loopback_transport::add_request_bytes(bounded_request_size(&request_line, &headers, &body));

    let header_admission_started = Instant::now();
    let cookie_rejected = headers_contain_cookie(&headers);
    let host_error = validate_host_header(&headers);
    loopback_transport::record_stage(
        LoopbackTransportStage::HeaderAdmission,
        header_admission_started.elapsed().as_nanos(),
    );
    if cookie_rejected {
        return timed_route_dispatch(|| {
            write_error_response(
                stream,
                403,
                LocalAuthError::CookieAuthForbidden.code(),
                "cookie auth forbidden",
            )
        });
    }
    if let Some(host_error) = host_error {
        return timed_route_dispatch(|| {
            write_error_response(stream, 400, "LOCAL_HOST_HEADER_REJECTED", host_error)
        });
    }
    if let Some(origin_error) = validate_browser_origin_headers(&headers) {
        return timed_route_dispatch(|| {
            write_error_response(stream, 403, "LOCAL_ORIGIN_HEADER_REJECTED", origin_error)
        });
    }

    timed_route_dispatch(|| {
        dispatch_http_route(
            stream,
            layout,
            authority,
            authority_store,
            task_api,
            resource_api,
            &request_line,
            &headers,
            &body,
        )
    })
}

/// Measure one route window and attribute accumulated socket writes to the
/// response-write stage instead of route work.
fn timed_route_dispatch<Route>(route: Route) -> Result<(), String>
where
    Route: FnOnce() -> Result<(), String>,
{
    let dispatch_started = Instant::now();
    let outcome = route();
    loopback_transport::record_route_dispatch(dispatch_started.elapsed().as_nanos());
    outcome
}

/// Bounded byte count of one request. Only the size is retained; the request
/// line, headers, and body never enter a transport observation.
fn bounded_request_size(request_line: &str, headers: &str, body: &[u8]) -> u64 {
    let total = request_line.len() + headers.len() + body.len();
    u64::try_from(total).unwrap_or(u64::MAX)
}

#[allow(clippy::too_many_arguments)] // The front door owns all shared daemon state.
fn dispatch_http_route(
    stream: &mut TcpStream,
    layout: &PersonalDataLayout,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    authority_store: &Arc<SqliteAuthorityStore>,
    task_api: &Arc<Mutex<TaskApi>>,
    resource_api: &Arc<Mutex<ResourceApi>>,
    request_line: &str,
    headers: &str,
    body: &[u8],
) -> Result<(), String> {
    let method_path = parse_request_line(request_line)?;
    if method_path.starts_with("POST /local/session ") {
        return handle_session_issue(stream, body, authority);
    }
    if method_path.starts_with("POST /provider/v1/dsh/chat/completions ") {
        return handle_provider_proxy_route(
            stream,
            headers,
            body,
            layout,
            authority,
            authority_store,
            provider_control_plane::DSH_AGENT,
        );
    }
    if method_path.starts_with("POST /provider/v1/chat/completions ") {
        return handle_provider_proxy_route(
            stream,
            headers,
            body,
            layout,
            authority,
            authority_store,
            provider_control_plane::PI_AGENT,
        );
    }
    if method_path.starts_with("GET /provider/v1/dsh/selected-model ") {
        return handle_selected_model_route(
            stream,
            headers,
            layout,
            authority,
            authority_store,
            provider_control_plane::DSH_AGENT,
        );
    }
    if method_path.starts_with("GET /provider/v1/selected-model ") {
        return handle_selected_model_route(
            stream,
            headers,
            layout,
            authority,
            authority_store,
            provider_control_plane::PI_AGENT,
        );
    }
    if settings_connections::matches(&method_path) {
        return handle_settings_connections_route(
            stream,
            &method_path,
            headers,
            body,
            authority,
            authority_store,
        );
    }
    if provider_control_plane::matches(&method_path) {
        return handle_provider_control_plane_route(
            stream,
            &method_path,
            headers,
            body,
            layout,
            authority,
            authority_store,
        );
    }
    if hosted_dsh_attempt::matches(&method_path) {
        return handle_hosted_dsh_attempt_route(
            stream,
            &method_path,
            headers,
            body,
            layout,
            authority,
            authority_store,
        );
    }
    if attempt_artifacts::matches(&method_path) {
        return handle_attempt_artifacts_route(
            stream,
            &method_path,
            headers,
            body,
            layout,
            authority,
            authority_store,
        );
    }
    if routine_runs::matches(&method_path) {
        return handle_routine_runs_route(
            stream,
            &method_path,
            headers,
            body,
            authority,
            authority_store,
        );
    }
    if project_aggregate::matches(&method_path) {
        return handle_project_aggregate_route(
            stream,
            &method_path,
            headers,
            body,
            layout,
            authority,
            authority_store,
        );
    }
    if windows_host::matches(&method_path) {
        return handle_windows_host_route(
            stream,
            &method_path,
            headers,
            body,
            authority,
            authority_store,
        );
    }
    if x_connector::matches(&method_path) {
        return handle_x_connector_route(
            stream,
            &method_path,
            headers,
            body,
            authority,
            authority_store,
        );
    }
    if method_path.starts_with("GET /personal/dsh/runtime ")
        || method_path.starts_with("POST /personal/dsh/runtime ")
    {
        return handle_dsh_runtime_route(
            stream,
            &method_path,
            headers,
            body,
            layout,
            authority,
            authority_store,
            task_api,
        );
    }
    if method_path.starts_with("POST /management/context-authorization/facts ") {
        return handle_context_authorization_fact_admission(
            stream,
            headers,
            body,
            authority,
            authority_store,
        );
    }
    if method_path.starts_with("POST /management/context-authorization/revocations ") {
        return handle_context_revocation_fact_admission(
            stream,
            headers,
            body,
            authority,
            authority_store,
        );
    }
    if method_path.starts_with("GET /management/resource/v1/fault-profile")
        || method_path.starts_with("POST /management/resource/v1/fault-profile")
    {
        return handle_fault_profile_route(
            stream,
            &method_path,
            headers,
            body,
            layout,
            authority,
            authority_store,
        );
    }
    if method_path.starts_with("GET /management/resource/v1/http-origin")
        || method_path.starts_with("POST /management/resource/v1/http-origin")
    {
        return handle_management_pinned_https_route(
            stream,
            &method_path,
            headers,
            body,
            layout,
            authority,
        );
    }
    if method_path.starts_with("GET /management/resource/v1/observation")
        || method_path.starts_with("POST /management/resource/v1/observation")
    {
        return handle_management_observation_forbidden(stream, headers, authority);
    }
    if method_path.starts_with("GET /management/resource/v1/tool")
        || method_path.starts_with("POST /management/resource/v1/tool/")
    {
        return handle_management_tool_lifecycle_route(
            stream,
            &method_path,
            headers,
            body,
            layout,
            authority,
        );
    }
    if method_path.starts_with("POST /management/resource/v1/backup")
        || method_path.starts_with("POST /management/resource/v1/backup/preflight")
        || method_path.starts_with("POST /management/resource/v1/restore")
    {
        return handle_management_user_backup_route(
            stream,
            &method_path,
            headers,
            body,
            layout,
            authority,
        );
    }
    if resource_manager::matches(&method_path) {
        return handle_resource_manager_route(
            stream,
            &method_path,
            headers,
            body,
            layout,
            authority,
            authority_store,
            resource_api,
        );
    }
    if method_path.starts_with("GET /management/resource/")
        || method_path.starts_with("POST /management/resource/")
    {
        return handle_authority_resource_route(
            stream,
            &method_path,
            headers,
            authority,
            authority_store,
            resource_api,
            body,
        );
    }
    if method_path.starts_with("POST /management/") {
        return handle_channel_route(
            stream,
            headers,
            ChannelClass::Management,
            authority,
            "management",
        );
    }
    if method_path.starts_with("GET /task/resource/v1/fault-profile")
        || method_path.starts_with("POST /task/resource/v1/fault-profile")
        || method_path.starts_with("POST /task/fault-profile")
        || method_path.starts_with("GET /task/fault-profile")
    {
        return handle_task_fault_profile_forbidden(stream, headers, authority);
    }
    if method_path.starts_with("GET /task/resource/v1/http-origin")
        || method_path.starts_with("POST /task/resource/v1/http-origin")
        || method_path.starts_with("POST /task/http-origin")
        || method_path.starts_with("GET /task/http-origin")
    {
        return handle_task_pinned_https_forbidden(stream, headers, authority);
    }
    if method_path.starts_with("GET /task/observation")
        || method_path.starts_with("POST /task/observation")
        || method_path.starts_with("GET /task/resource/v1/observation")
        || method_path.starts_with("POST /task/resource/v1/observation")
    {
        return handle_task_observation_route(stream, &method_path, headers, layout, authority);
    }
    if method_path.starts_with("POST /task/resource/v1/backup")
        || method_path.starts_with("POST /task/resource/v1/restore")
        || method_path.starts_with("POST /task/backup")
        || method_path.starts_with("POST /task/restore")
    {
        return handle_task_user_backup_forbidden(
            stream,
            &method_path,
            headers,
            body,
            layout,
            authority,
        );
    }
    if method_path.starts_with("GET /task/resource/v1/tool")
        || method_path.starts_with("POST /task/resource/v1/tool/")
    {
        return handle_task_tool_lifecycle_route(
            stream,
            &method_path,
            headers,
            body,
            layout,
            authority,
        );
    }
    if method_path.starts_with("GET /task/resource/v1/consumption") {
        return handle_task_consumption_query_route(
            stream,
            &method_path,
            headers,
            authority,
            authority_store,
            resource_api,
        );
    }
    if method_path.starts_with("POST /task/resource/v1/consumption") {
        return handle_task_consumption_route(
            stream,
            headers,
            body,
            authority,
            authority_store,
            resource_api,
        );
    }
    if method_path.starts_with("POST /task/resource/v1/memory/")
        || method_path.starts_with("GET /task/resource/v1/memory/")
    {
        return handle_task_memory_forbidden(stream, headers, authority);
    }
    if resource_manager::matches(&method_path) {
        return handle_resource_manager_route(
            stream,
            &method_path,
            headers,
            body,
            layout,
            authority,
            authority_store,
            resource_api,
        );
    }
    if method_path.starts_with("GET /task/resource/") {
        return handle_task_resource_route(stream, &method_path, headers, authority, resource_api);
    }
    if method_path.starts_with("POST /task/") || method_path.starts_with("GET /task/") {
        return handle_task_route(stream, &method_path, headers, body, authority, task_api);
    }
    if method_path.starts_with("GET /resource/") {
        return handle_resource_route(stream, &method_path, headers, authority, resource_api);
    }
    if method_path.starts_with("GET /personal/status ")
        || method_path.starts_with("GET /personal/readiness ")
    {
        return handle_readiness_route(stream, headers, layout, authority, "status");
    }
    if method_path.starts_with("GET /personal/doctor ") {
        return handle_readiness_route(stream, headers, layout, authority, "doctor");
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
    if method_path.starts_with("GET /ui ") || method_path.starts_with("GET /ui/") {
        return handle_web_ui_route(stream, layout, &method_path);
    }

    write_error_response(
        stream,
        404,
        "PERSONAL_ROUTE_NOT_FOUND",
        "no personal route matched",
    )?;
    Ok(())
}

fn authorize_daemon_administrator_request(
    headers: &str,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
) -> Result<(), (u16, LocalAuthError)> {
    let Some(token) = extract_bearer_token(headers) else {
        return Err((401, LocalAuthError::Unauthorized));
    };
    let mut authority_guard = authority.lock().map_err(|_| {
        (
            500,
            LocalAuthError::Io {
                detail: "session authority lock poisoned",
            },
        )
    })?;
    authority_guard
        .authorize_daemon_administrator(&token, Instant::now())
        .map(|_| ())
        .map_err(|error| {
            let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
                403
            } else {
                401
            };
            (status, error)
        })
}

fn handle_context_authorization_fact_admission(
    stream: &mut TcpStream,
    headers: &str,
    body: &[u8],
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    authority_store: &Arc<SqliteAuthorityStore>,
) -> Result<(), String> {
    if let Err((status, error)) = authorize_daemon_administrator_request(headers, authority) {
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    let facts: ContextAuthorizationFactsRow = serde_json::from_slice(body).map_err(|_| {
        "Context authorization facts must be a valid daemon-admin payload".to_owned()
    })?;
    authority_store
        .append_context_authorization_facts(&facts)
        .map_err(|error| format!("admit Context authorization facts: {error}"))?;
    write_json_response(stream, 201, &json!({"status": "admitted"}).to_string())
}

fn handle_context_revocation_fact_admission(
    stream: &mut TcpStream,
    headers: &str,
    body: &[u8],
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    authority_store: &Arc<SqliteAuthorityStore>,
) -> Result<(), String> {
    if let Err((status, error)) = authorize_daemon_administrator_request(headers, authority) {
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    let fact: ContextRevocationFactRow = serde_json::from_slice(body)
        .map_err(|_| "Context revocation fact must be a valid daemon-admin payload".to_owned())?;
    authority_store
        .append_context_revocation_fact(&fact)
        .map_err(|error| format!("admit Context revocation fact: {error}"))?;
    write_json_response(stream, 201, &json!({"status": "admitted"}).to_string())
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
    let authorization = match required_channel {
        ChannelClass::Management => guard
            .authorize_daemon_administrator(&token, Instant::now())
            .map(|_| ()),
        ChannelClass::Task => guard.authorize(&token, ChannelClass::Task, Instant::now()),
    };
    match authorization {
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

fn handle_task_resource_route(
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
    if let Err(error) = authority_guard.authorize(&token, ChannelClass::Task, Instant::now()) {
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
        .handle_task(&method_path.replacen("/task/resource/", "/resource/", 1));
    write_response(
        stream,
        response.status,
        response.content_type,
        response.body.as_bytes(),
    )
}

fn handle_task_consumption_query_route(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    authority_store: &Arc<SqliteAuthorityStore>,
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
    if let Err(error) = authority_guard.authorize(&token, ChannelClass::Task, Instant::now()) {
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
        .handle_task_consumption_query(method_path, authority_store.as_ref());
    write_response(
        stream,
        response.status,
        response.content_type,
        response.body.as_bytes(),
    )
}

fn handle_task_consumption_route(
    stream: &mut TcpStream,
    headers: &str,
    body: &[u8],
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    authority_store: &Arc<SqliteAuthorityStore>,
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
    if let Err(error) = authority_guard.authorize(&token, ChannelClass::Task, Instant::now()) {
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
        .handle_task_consumption(body, authority_store.as_ref());
    write_response(
        stream,
        response.status,
        response.content_type,
        response.body.as_bytes(),
    )
}

fn handle_fault_profile_route(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    body: &[u8],
    layout: &PersonalDataLayout,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    authority_store: &Arc<SqliteAuthorityStore>,
) -> Result<(), String> {
    if let Err((status, error)) = authorize_daemon_administrator_request(headers, authority) {
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    let response = fault_profile::handle(method_path, body, layout, authority_store.as_ref());
    write_response(
        stream,
        response.status,
        "application/json",
        response.body.as_bytes(),
    )
}

fn handle_management_pinned_https_route(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    body: &[u8],
    layout: &PersonalDataLayout,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
) -> Result<(), String> {
    if let Err((status, error)) = authorize_daemon_administrator_request(headers, authority) {
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    let response = pinned_https::handle(method_path, body, layout);
    write_response(
        stream,
        response.status,
        "application/json",
        response.body.as_bytes(),
    )
}

fn handle_task_pinned_https_forbidden(
    stream: &mut TcpStream,
    headers: &str,
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
    if let Err(error) = authority_guard.authorize(&token, ChannelClass::Task, Instant::now()) {
        let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
            403
        } else {
            401
        };
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    drop(authority_guard);
    let response = pinned_https::task_channel_forbidden();
    write_response(
        stream,
        response.status,
        "application/json",
        response.body.as_bytes(),
    )
}

fn handle_management_observation_forbidden(
    stream: &mut TcpStream,
    headers: &str,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
) -> Result<(), String> {
    if let Err((status, error)) = authorize_daemon_administrator_request(headers, authority) {
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    let response = super::observation::management_channel_forbidden();
    write_response(
        stream,
        response.status,
        "application/json",
        response.body.as_bytes(),
    )
}

fn handle_task_observation_route(
    stream: &mut TcpStream,
    method_path: &str,
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
    if let Err(error) = authority_guard.authorize(&token, ChannelClass::Task, Instant::now()) {
        let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
            403
        } else {
            401
        };
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    drop(authority_guard);
    let response = super::observation::handle(method_path, layout);
    write_response(
        stream,
        response.status,
        "application/json",
        response.body.as_bytes(),
    )
}

fn handle_management_tool_lifecycle_route(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    body: &[u8],
    layout: &PersonalDataLayout,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
) -> Result<(), String> {
    if let Err((status, error)) = authorize_daemon_administrator_request(headers, authority) {
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    let response = tool_lifecycle::handle(
        method_path,
        body,
        layout,
        tool_lifecycle::ToolLifecycleChannel::Management,
    );
    write_response(
        stream,
        response.status,
        "application/json",
        response.body.as_bytes(),
    )
}

fn handle_management_user_backup_route(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    body: &[u8],
    layout: &PersonalDataLayout,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
) -> Result<(), String> {
    if let Err((status, error)) = authorize_daemon_administrator_request(headers, authority) {
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    let response = user_backup::handle(
        method_path,
        body,
        layout,
        user_backup::UserBackupChannel::Management,
    );
    write_response(
        stream,
        response.status,
        "application/json",
        response.body.as_bytes(),
    )
}

fn handle_task_memory_forbidden(
    stream: &mut TcpStream,
    headers: &str,
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
    if let Err(error) = authority_guard.authorize(&token, ChannelClass::Task, Instant::now()) {
        let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
            403
        } else {
            401
        };
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    drop(authority_guard);
    write_error_response(
        stream,
        403,
        "RESOURCE_MEMORY_CHANNEL_FORBIDDEN",
        "Memory mutations are management-channel only",
    )
}

fn handle_task_user_backup_forbidden(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    body: &[u8],
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
    if let Err(error) = authority_guard.authorize(&token, ChannelClass::Task, Instant::now()) {
        let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
            403
        } else {
            401
        };
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    drop(authority_guard);
    let response = user_backup::handle(
        method_path,
        body,
        layout,
        user_backup::UserBackupChannel::Task,
    );
    write_response(
        stream,
        response.status,
        "application/json",
        response.body.as_bytes(),
    )
}

fn handle_task_tool_lifecycle_route(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    body: &[u8],
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
    if let Err(error) = authority_guard.authorize(&token, ChannelClass::Task, Instant::now()) {
        let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
            403
        } else {
            401
        };
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    drop(authority_guard);
    let response = tool_lifecycle::handle(
        method_path,
        body,
        layout,
        tool_lifecycle::ToolLifecycleChannel::Task,
    );
    write_response(
        stream,
        response.status,
        "application/json",
        response.body.as_bytes(),
    )
}

fn handle_task_fault_profile_forbidden(
    stream: &mut TcpStream,
    headers: &str,
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
    if let Err(error) = authority_guard.authorize(&token, ChannelClass::Task, Instant::now()) {
        let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
            403
        } else {
            401
        };
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    drop(authority_guard);
    let response = fault_profile::task_channel_forbidden();
    write_response(
        stream,
        response.status,
        "application/json",
        response.body.as_bytes(),
    )
}

#[allow(clippy::too_many_arguments)] // Shared daemon state is explicit at the connection boundary.
fn handle_settings_connections_route(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    body: &[u8],
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    authority_store: &Arc<SqliteAuthorityStore>,
) -> Result<(), String> {
    if settings_connections::is_task_channel(method_path) {
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
        if let Err(error) = authority_guard.authorize(&token, ChannelClass::Task, Instant::now()) {
            let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
                403
            } else {
                401
            };
            return write_error_response(stream, status, error.code(), &error.to_string());
        }
        drop(authority_guard);
        let response = settings_connections::channel_forbidden();
        return write_response(
            stream,
            response.status,
            response.content_type,
            response.body.as_bytes(),
        );
    }
    if let Err((status, error)) = authorize_daemon_administrator_request(headers, authority) {
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    let response = settings_connections::handle(method_path, body, authority_store.as_ref());
    write_response(
        stream,
        response.status,
        response.content_type,
        response.body.as_bytes(),
    )
}

fn handle_provider_control_plane_route(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    body: &[u8],
    layout: &PersonalDataLayout,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    authority_store: &Arc<SqliteAuthorityStore>,
) -> Result<(), String> {
    if provider_control_plane::is_task_channel(method_path) {
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
        if let Err(error) = authority_guard.authorize(&token, ChannelClass::Task, Instant::now()) {
            let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
                403
            } else {
                401
            };
            return write_error_response(stream, status, error.code(), &error.to_string());
        }
        drop(authority_guard);
        let response = provider_control_plane::channel_forbidden();
        return write_response(
            stream,
            response.status,
            response.content_type,
            response.body.as_bytes(),
        );
    }
    if let Err((status, error)) = authorize_daemon_administrator_request(headers, authority) {
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    let response = provider_control_plane::handle(method_path, body, authority_store.as_ref());
    if response.status < 300 && dsh_web_overlay_should_sync(method_path) {
        let _ = sync_dsh_web_control_plane_overlay(layout, authority_store.as_ref(), false);
    }
    write_response(
        stream,
        response.status,
        response.content_type,
        response.body.as_bytes(),
    )
}

fn handle_project_aggregate_route(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    body: &[u8],
    layout: &PersonalDataLayout,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    authority_store: &Arc<SqliteAuthorityStore>,
) -> Result<(), String> {
    if project_aggregate::is_task_channel(method_path) {
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
        if let Err(error) = authority_guard.authorize(&token, ChannelClass::Task, Instant::now()) {
            let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
                403
            } else {
                401
            };
            return write_error_response(stream, status, error.code(), &error.to_string());
        }
        drop(authority_guard);
        let response = project_aggregate::channel_forbidden();
        return write_response(
            stream,
            response.status,
            response.content_type,
            response.body.as_bytes(),
        );
    }
    if let Err((status, error)) = authorize_daemon_administrator_request(headers, authority) {
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    // P13-T03: the hidden assistant infers through the exact-Pi runtime of
    // this daemon; the Provider is reached only via the private proxy.
    let assistant = assistant_inference::DaemonAssistantRuntime::new(
        layout.config_dir(),
        layout.data_dir(),
        authority_store.as_ref().clone(),
    );
    let response = project_aggregate::handle_with_assistant(
        method_path,
        body,
        authority_store.as_ref(),
        &assistant,
    );
    write_response(
        stream,
        response.status,
        response.content_type,
        response.body.as_bytes(),
    )
}

fn handle_windows_host_route(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    body: &[u8],
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    authority_store: &Arc<SqliteAuthorityStore>,
) -> Result<(), String> {
    if windows_host::is_task_channel(method_path) {
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
        if let Err(error) = authority_guard.authorize(&token, ChannelClass::Task, Instant::now()) {
            let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
                403
            } else {
                401
            };
            return write_error_response(stream, status, error.code(), &error.to_string());
        }
        drop(authority_guard);
        let response = windows_host::channel_forbidden();
        return write_response(
            stream,
            response.status,
            response.content_type,
            response.body.as_bytes(),
        );
    }
    if let Err((status, error)) = authorize_daemon_administrator_request(headers, authority) {
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    let response = windows_host::handle(method_path, body, authority_store.as_ref());
    write_response(
        stream,
        response.status,
        response.content_type,
        response.body.as_bytes(),
    )
}

fn handle_hosted_dsh_attempt_route(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    body: &[u8],
    layout: &PersonalDataLayout,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    authority_store: &Arc<SqliteAuthorityStore>,
) -> Result<(), String> {
    if hosted_dsh_attempt::is_task_channel(method_path) {
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
        if let Err(error) = authority_guard.authorize(&token, ChannelClass::Task, Instant::now()) {
            let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
                403
            } else {
                401
            };
            return write_error_response(stream, status, error.code(), &error.to_string());
        }
        drop(authority_guard);
        let response = hosted_dsh_attempt::channel_forbidden();
        return write_response(
            stream,
            response.status,
            response.content_type,
            response.body.as_bytes(),
        );
    }
    if let Err((status, error)) = authorize_daemon_administrator_request(headers, authority) {
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    let host = hosted_dsh_attempt::HostedAttemptHost::from_layout(layout);
    let response = hosted_dsh_attempt::handle(method_path, body, authority_store.as_ref(), &host);
    write_response(
        stream,
        response.status,
        response.content_type,
        response.body.as_bytes(),
    )
}

fn handle_attempt_artifacts_route(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    body: &[u8],
    layout: &PersonalDataLayout,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    authority_store: &Arc<SqliteAuthorityStore>,
) -> Result<(), String> {
    if attempt_artifacts::is_task_channel(method_path) {
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
        if let Err(error) = authority_guard.authorize(&token, ChannelClass::Task, Instant::now()) {
            let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
                403
            } else {
                401
            };
            return write_error_response(stream, status, error.code(), &error.to_string());
        }
        drop(authority_guard);
        let response = attempt_artifacts::channel_forbidden();
        return write_response(
            stream,
            response.status,
            response.content_type,
            response.body.as_bytes(),
        );
    }
    if let Err((status, error)) = authorize_daemon_administrator_request(headers, authority) {
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    let host = attempt_artifacts::ArtifactHost::from_layout(layout);
    let response = attempt_artifacts::handle(method_path, body, authority_store.as_ref(), &host);
    write_response(
        stream,
        response.status,
        response.content_type,
        response.body.as_bytes(),
    )
}

/// P13-T05 Routine arming / occurrence ledger / Today overview routes.
/// Management only; task-channel aliases are 403 after bearer validation.
fn handle_routine_runs_route(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    body: &[u8],
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    authority_store: &Arc<SqliteAuthorityStore>,
) -> Result<(), String> {
    if routine_runs::is_task_channel(method_path) {
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
        if let Err(error) = authority_guard.authorize(&token, ChannelClass::Task, Instant::now()) {
            let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
                403
            } else {
                401
            };
            return write_error_response(stream, status, error.code(), &error.to_string());
        }
        drop(authority_guard);
        let response = routine_runs::channel_forbidden();
        return write_response(
            stream,
            response.status,
            response.content_type,
            response.body.as_bytes(),
        );
    }
    if let Err((status, error)) = authorize_daemon_administrator_request(headers, authority) {
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    let response = routine_runs::handle(method_path, body, authority_store.as_ref());
    write_response(
        stream,
        response.status,
        response.content_type,
        response.body.as_bytes(),
    )
}

fn handle_x_connector_route(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    body: &[u8],
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    authority_store: &Arc<SqliteAuthorityStore>,
) -> Result<(), String> {
    if x_connector::is_task_channel(method_path) {
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
        if let Err(error) = authority_guard.authorize(&token, ChannelClass::Task, Instant::now()) {
            let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
                403
            } else {
                401
            };
            return write_error_response(stream, status, error.code(), &error.to_string());
        }
        drop(authority_guard);
        let response = x_connector::channel_forbidden();
        return write_response(
            stream,
            response.status,
            response.content_type,
            response.body.as_bytes(),
        );
    }
    if let Err((status, error)) = authorize_daemon_administrator_request(headers, authority) {
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    let response = x_connector::handle(method_path, body, authority_store.as_ref());
    write_response(
        stream,
        response.status,
        response.content_type,
        response.body.as_bytes(),
    )
}

#[allow(clippy::too_many_arguments)] // Shared daemon state is explicit at the connection boundary.
fn handle_resource_manager_route(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    body: &[u8],
    layout: &PersonalDataLayout,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    authority_store: &Arc<SqliteAuthorityStore>,
    resource_api: &Arc<Mutex<ResourceApi>>,
) -> Result<(), String> {
    if resource_manager::is_task_channel(method_path) {
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
        if let Err(error) = authority_guard.authorize(&token, ChannelClass::Task, Instant::now()) {
            let status = if matches!(error, LocalAuthError::ChannelBindingMismatch) {
                403
            } else {
                401
            };
            return write_error_response(stream, status, error.code(), &error.to_string());
        }
        drop(authority_guard);
        let response = resource_manager::channel_forbidden();
        return write_response(
            stream,
            response.status,
            response.content_type,
            response.body.as_bytes(),
        );
    }
    if let Err((status, error)) = authorize_daemon_administrator_request(headers, authority) {
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    let resource_api = resource_api
        .lock()
        .map_err(|_| "resource projection lock poisoned".to_owned())?;
    let response = resource_manager::handle(
        method_path,
        body,
        layout,
        authority_store.as_ref(),
        &resource_api,
    );
    write_response(
        stream,
        response.status,
        response.content_type,
        response.body.as_bytes(),
    )
}

fn handle_authority_resource_route(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    authority_store: &Arc<SqliteAuthorityStore>,
    resource_api: &Arc<Mutex<ResourceApi>>,
    body: &[u8],
) -> Result<(), String> {
    if let Err((status, error)) = authorize_daemon_administrator_request(headers, authority) {
        return write_error_response(stream, status, error.code(), &error.to_string());
    }
    let response = resource_api
        .lock()
        .map_err(|_| "resource projection lock poisoned".to_owned())?
        .handle_authority_or_mutation(method_path, body, authority_store.as_ref());
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
    authority_store: &Arc<SqliteAuthorityStore>,
    agent: &str,
) -> Result<(), String> {
    let _ = stream.set_nodelay(true);
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

    let stream_requested = request_asks_for_stream(request_body);
    match provider_control_plane::plan_bound_proxy(
        authority_store.as_ref(),
        agent,
        request_body,
        stream_requested,
    ) {
        Ok(Some(plan)) => {
            return handle_bound_provider_proxy(
                stream,
                headers,
                authority_store.as_ref(),
                agent,
                plan,
                stream_requested,
            );
        }
        Err(error) => {
            let mapped = map_bound_plan_error(error);
            return write_error_response(
                stream,
                mapped.status_code(),
                mapped.code(),
                "provider proxy request was not completed",
            );
        }
        Ok(None) => {}
    }

    let secret_backend = select_production_secret_store();
    let transport = RustlsProviderTransport::default();
    let service = ProviderProxyService::new(
        secret_backend.as_secret_store(),
        ProviderConfigRepository::under_config_dir(layout.config_dir()),
        &transport,
    );
    let correlation_id = route_observation::extract_correlation_id(headers);
    let observation_authorized = route_observation::route_observation_authorized();
    if stream_requested {
        return handle_provider_proxy_streaming(
            stream,
            request_body,
            &service,
            observation_authorized,
            correlation_id.as_deref().map_err(|refusal| *refusal),
        );
    }
    match service.forward_chat_completion_with_timing(request_body) {
        Ok(timed_response) => write_provider_response(
            stream,
            timed_response.response.status,
            &timed_response.response.body,
            timed_response.provider_network_elapsed_nanos,
            &route_observation::observation_response_headers(
                observation_authorized,
                correlation_id.as_deref().map_err(|refusal| *refusal),
                route_observation::NestedProviderStages {
                    preflight_elapsed_nanos: timed_response.preflight_elapsed_nanos,
                    provider_network_elapsed_nanos: timed_response.provider_network_elapsed_nanos,
                },
            ),
        ),
        Err(error) => write_error_response(
            stream,
            error.status_code(),
            error.code(),
            "provider proxy request was not completed",
        ),
    }
}

fn map_bound_plan_error(error: provider_control_plane::BoundPlanError) -> ProviderProxyError {
    match error {
        provider_control_plane::BoundPlanError::BindingMismatch => {
            ProviderProxyError::BindingMismatch
        }
        provider_control_plane::BoundPlanError::AccountUnavailable => {
            ProviderProxyError::AccountUnavailable
        }
        provider_control_plane::BoundPlanError::SecretUnavailable => {
            ProviderProxyError::SecretUnavailable
        }
        provider_control_plane::BoundPlanError::InvalidRequest => {
            ProviderProxyError::InvalidRequest
        }
        provider_control_plane::BoundPlanError::StreamingUnsupported => {
            ProviderProxyError::StreamingUnsupported
        }
        provider_control_plane::BoundPlanError::Trust => ProviderProxyError::TransportUnavailable,
        provider_control_plane::BoundPlanError::UpstreamFailed => {
            ProviderProxyError::UpstreamRequestFailed
        }
    }
}

fn handle_bound_provider_proxy(
    stream: &mut TcpStream,
    headers: &str,
    authority_store: &SqliteAuthorityStore,
    agent: &str,
    plan: provider_control_plane::BoundProxyPlan,
    stream_requested: bool,
) -> Result<(), String> {
    let correlation_id = route_observation::extract_correlation_id(headers);
    let observation_authorized = route_observation::route_observation_authorized();
    if stream_requested {
        if plan.uses_http || plan.anthropic {
            return write_error_response(
                stream,
                ProviderProxyError::StreamingUnsupported.status_code(),
                ProviderProxyError::StreamingUnsupported.code(),
                "provider proxy request was not completed",
            );
        }
        let transport = RustlsProviderTransport::default();
        return handle_bound_provider_streaming(
            stream,
            &transport,
            authority_store,
            agent,
            plan,
            observation_authorized,
            correlation_id.as_deref().map_err(|refusal| *refusal),
        );
    }
    let started = Instant::now();
    match provider_control_plane::execute_bound_unary_plan(authority_store, agent, plan) {
        Ok(response) => {
            let elapsed_nanos = started.elapsed().as_nanos().max(1);
            write_provider_response(
                stream,
                response.status,
                &response.body,
                elapsed_nanos,
                &route_observation::observation_response_headers(
                    observation_authorized,
                    correlation_id.as_deref().map_err(|refusal| *refusal),
                    route_observation::NestedProviderStages {
                        preflight_elapsed_nanos: 1,
                        provider_network_elapsed_nanos: elapsed_nanos,
                    },
                ),
            )
        }
        Err(error) => {
            let mapped = map_bound_plan_error(error);
            write_error_response(
                stream,
                mapped.status_code(),
                mapped.code(),
                "provider proxy request was not completed",
            )
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_bound_provider_streaming<T: cognitive_secret::ProviderTransport + ?Sized>(
    stream: &mut TcpStream,
    transport: &T,
    authority_store: &SqliteAuthorityStore,
    agent: &str,
    plan: provider_control_plane::BoundProxyPlan,
    observation_authorized: bool,
    correlation_id: Result<&str, route_observation::CorrelationRefusal>,
) -> Result<(), String> {
    let stream_cell = std::cell::RefCell::new(stream);
    let sse_started = std::cell::Cell::new(false);
    let error_body = std::cell::RefCell::new(Vec::new());
    let normalizer = std::cell::RefCell::new(SseToolCallNormalizer::new());
    let observation_headers = route_observation::observation_streaming_response_headers(
        observation_authorized,
        correlation_id,
    );
    let outcome = {
        let on_status = |status: u16| -> Result<(), ProviderProxyError> {
            normalizer.borrow_mut().set_passthrough(status != 200);
            if status == 200 {
                write_provider_sse_headers(&mut *stream_cell.borrow_mut(), 1, &observation_headers)
                    .map_err(|_| ProviderProxyError::UpstreamRequestFailed)?;
                sse_started.set(true);
            }
            Ok(())
        };
        let on_chunk = |chunk: &[u8]| -> Result<(), ProviderProxyError> {
            let forwarded = normalizer.borrow_mut().push(chunk);
            if forwarded.is_empty() {
                return Ok(());
            }
            if sse_started.get() {
                let write_started = Instant::now();
                {
                    let mut stream = stream_cell.borrow_mut();
                    stream
                        .write_all(&forwarded)
                        .and_then(|()| stream.flush())
                        .map_err(|_| ProviderProxyError::UpstreamRequestFailed)?;
                }
                loopback_transport::add_response_write(
                    write_started.elapsed().as_nanos(),
                    u64::try_from(forwarded.len()).unwrap_or(u64::MAX),
                );
            } else {
                error_body.borrow_mut().extend_from_slice(&forwarded);
            }
            Ok(())
        };
        transport.exchange_stream(
            &plan.request,
            &mut |status| {
                on_status(status).map_err(|_| cognitive_secret::ProviderTransportError::Network {
                    detail: "streaming callback failed",
                })
            },
            &mut |chunk| {
                on_chunk(chunk).map_err(|_| cognitive_secret::ProviderTransportError::Network {
                    detail: "streaming callback failed",
                })
            },
        )
    };
    let tail = normalizer.borrow_mut().flush();
    if !tail.is_empty() {
        if sse_started.get() {
            let mut stream = stream_cell.borrow_mut();
            stream
                .write_all(&tail)
                .and_then(|()| stream.flush())
                .map_err(|_| "provider streaming tail write failed".to_owned())?;
        } else {
            error_body.borrow_mut().extend_from_slice(&tail);
        }
    }
    provider_control_plane::record_proxy_usage(
        authority_store,
        &plan.account,
        &plan.model_id,
        agent,
        b"{}",
        0,
        if outcome
            .as_ref()
            .map(|timed| timed.status == 200)
            .unwrap_or(false)
        {
            "ok"
        } else {
            "failed"
        },
    );
    let stream = stream_cell.into_inner();
    match outcome {
        Ok(timed) if sse_started.get() => {
            let _ = timed;
            Ok(())
        }
        Ok(timed) => write_provider_response(
            stream,
            timed.status,
            &error_body.into_inner(),
            timed.provider_network_elapsed_nanos,
            &route_observation::observation_response_headers(
                observation_authorized,
                correlation_id,
                route_observation::NestedProviderStages {
                    preflight_elapsed_nanos: 1,
                    provider_network_elapsed_nanos: timed.provider_network_elapsed_nanos,
                },
            ),
        ),
        Err(_) if sse_started.get() => Ok(()),
        Err(_) => write_error_response(
            stream,
            ProviderProxyError::UpstreamRequestFailed.status_code(),
            ProviderProxyError::UpstreamRequestFailed.code(),
            "provider proxy request was not completed",
        ),
    }
}

fn request_asks_for_stream(request_body: &[u8]) -> bool {
    serde_json::from_slice::<serde_json::Value>(request_body)
        .ok()
        .and_then(|value| value.get("stream").and_then(serde_json::Value::as_bool))
        == Some(true)
}

fn handle_provider_proxy_streaming<T: cognitive_secret::ProviderTransport + ?Sized>(
    stream: &mut TcpStream,
    request_body: &[u8],
    service: &ProviderProxyService<'_, T>,
    observation_authorized: bool,
    correlation_id: Result<&str, route_observation::CorrelationRefusal>,
) -> Result<(), String> {
    let stream_cell = std::cell::RefCell::new(stream);
    let sse_started = std::cell::Cell::new(false);
    let error_body = std::cell::RefCell::new(Vec::new());
    let preflight_elapsed_nanos = std::cell::Cell::new(0_u128);
    let observation_headers = route_observation::observation_streaming_response_headers(
        observation_authorized,
        correlation_id,
    );
    let outcome = {
        let mut on_preflight = |nanos: u128| {
            preflight_elapsed_nanos.set(nanos);
            Ok(())
        };
        let mut on_status = |status: u16| {
            if status == 200 {
                write_provider_sse_headers(
                    &mut *stream_cell.borrow_mut(),
                    preflight_elapsed_nanos.get(),
                    &observation_headers,
                )
                .map_err(|_| ProviderProxyError::UpstreamRequestFailed)?;
                sse_started.set(true);
            }
            Ok(())
        };
        let mut on_chunk = |chunk: &[u8]| {
            if sse_started.get() {
                let write_started = Instant::now();
                {
                    let mut stream = stream_cell.borrow_mut();
                    stream
                        .write_all(chunk)
                        .and_then(|()| stream.flush())
                        .map_err(|_| ProviderProxyError::UpstreamRequestFailed)?;
                }
                loopback_transport::add_response_write(
                    write_started.elapsed().as_nanos(),
                    u64::try_from(chunk.len()).unwrap_or(u64::MAX),
                );
            } else {
                error_body.borrow_mut().extend_from_slice(chunk);
            }
            Ok(())
        };
        service.forward_streaming_chat_completion(
            request_body,
            &mut on_preflight,
            &mut on_status,
            &mut on_chunk,
        )
    };
    let stream = stream_cell.into_inner();
    match outcome {
        Ok(timed) if sse_started.get() => {
            let _ = timed;
            Ok(())
        }
        Ok(timed) => write_provider_response(
            stream,
            timed.status,
            &error_body.into_inner(),
            timed.provider_network_elapsed_nanos,
            &route_observation::observation_response_headers(
                observation_authorized,
                correlation_id,
                route_observation::NestedProviderStages {
                    preflight_elapsed_nanos: timed.preflight_elapsed_nanos,
                    provider_network_elapsed_nanos: timed.provider_network_elapsed_nanos,
                },
            ),
        ),
        Err(error) if sse_started.get() => {
            let _ = error;
            Ok(())
        }
        Err(error) => write_error_response(
            stream,
            error.status_code(),
            error.code(),
            "provider proxy request was not completed",
        ),
    }
}

fn write_provider_sse_headers(
    stream: &mut impl Write,
    preflight_elapsed_nanos: u128,
    observation_headers: &str,
) -> Result<(), String> {
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nX-CognitiveOS-Daemon-Preflight-Nanos: {preflight_elapsed_nanos}\r\n{observation_headers}Connection: close\r\n\r\n"
    );
    let write_started = Instant::now();
    let outcome = stream
        .write_all(header.as_bytes())
        .and_then(|()| stream.flush());
    loopback_transport::add_response_write(
        write_started.elapsed().as_nanos(),
        u64::try_from(header.len()).unwrap_or(u64::MAX),
    );
    outcome.map_err(|error| error.to_string())
}

#[allow(clippy::too_many_arguments)] // Shared daemon state is explicit at the connection boundary.
fn handle_dsh_runtime_route(
    stream: &mut TcpStream,
    method_path: &str,
    headers: &str,
    body: &[u8],
    layout: &PersonalDataLayout,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    authority_store: &Arc<SqliteAuthorityStore>,
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

    let mut task_api = task_api
        .lock()
        .map_err(|_| "task API lock poisoned".to_owned())?;
    if method_path.starts_with("GET /personal/dsh/runtime ") {
        return write_json_response(stream, 200, &task_api.dsh_runtime_snapshot().to_string());
    }

    let request_json: serde_json::Value = match serde_json::from_slice(body) {
        Ok(value) => value,
        Err(_) => {
            return write_error_response(
                stream,
                400,
                "DSH_RUNTIME_INVALID_REQUEST",
                "dsh runtime request must be JSON",
            );
        }
    };
    let schema_version = request_json
        .get("schema_version")
        .and_then(serde_json::Value::as_u64);
    let surface = request_json
        .get("surface")
        .and_then(serde_json::Value::as_str);
    if schema_version != Some(1) || surface != Some("personal-dsh-runtime") {
        return write_error_response(
            stream,
            400,
            "DSH_RUNTIME_INVALID_REQUEST",
            "dsh runtime request schema is not personal-dsh-runtime/1",
        );
    }
    let op = request_json
        .get("op")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let snapshot = match op {
        "bind" => {
            let Some(process_id) = request_json
                .get("process_id")
                .and_then(serde_json::Value::as_u64)
                .and_then(|value| u32::try_from(value).ok())
            else {
                return write_error_response(
                    stream,
                    400,
                    "DSH_RUNTIME_INVALID_REQUEST",
                    "dsh runtime bind requires a nonzero process_id",
                );
            };
            match task_api.dsh_bind_process(process_id) {
                Ok(snapshot) => snapshot,
                Err(detail) => {
                    return write_error_response(
                        stream,
                        400,
                        "DSH_RUNTIME_INVALID_REQUEST",
                        &detail,
                    );
                }
            }
        }
        "heartbeat" => match task_api.dsh_heartbeat() {
            Ok(snapshot) => snapshot,
            Err(detail) => {
                return write_error_response(stream, 409, "DSH_RUNTIME_INACTIVE", &detail);
            }
        },
        "clear" => task_api.dsh_clear_process(),
        "apply" => {
            return apply_dsh_binding_to_runtime(
                stream,
                &request_json,
                layout,
                authority_store,
                &task_api,
            );
        }
        _ => {
            return write_error_response(
                stream,
                400,
                "DSH_RUNTIME_INVALID_REQUEST",
                "dsh runtime op must be bind, heartbeat, clear, or apply",
            );
        }
    };
    write_json_response(stream, 200, &snapshot.to_string())
}

const DSH_WEB_HOME_DIR: &str = "dsh-web-home";
const DSH_WEB_OVERLAY_FILE: &str = "control-plane-overlay.json";
const DSH_WEB_OVERLAY_APPLIED_FILE: &str = "control-plane-overlay.applied.json";
const DSH_WEB_OVERLAY_SURFACE: &str = "personal-dsh-web-overlay";

fn dsh_web_home(layout: &PersonalDataLayout) -> PathBuf {
    layout.runtime_dir().join(DSH_WEB_HOME_DIR)
}

fn yaml_safe_catalog_id(value: &str) -> Option<&str> {
    let id = value.trim();
    if id.is_empty()
        || id
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte == b'#' || byte == b':')
    {
        None
    } else {
        Some(id)
    }
}

fn dsh_web_overlay_should_sync(method_path: &str) -> bool {
    method_path.starts_with("POST /management/agent-bindings")
        || method_path.starts_with("POST /management/providers/models/refresh")
        || method_path.starts_with("POST /management/providers/models/add")
        || method_path.starts_with("POST /management/providers/accounts/delete")
        || method_path.starts_with("POST /management/providers/accounts/update")
}

fn unix_now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|elapsed| i64::try_from(elapsed.as_millis()).ok())
        .unwrap_or(0)
}

fn build_dsh_web_control_plane_overlay(
    authority_store: &SqliteAuthorityStore,
) -> serde_json::Value {
    let plane = ProviderControlPlaneStore::from_authority_store(authority_store);
    match plane
        .get_active_binding(provider_control_plane::DSH_AGENT)
        .ok()
        .flatten()
    {
        Some(binding) => {
            let catalog = plane
                .list_models(&binding.account_id)
                .unwrap_or_default()
                .into_iter()
                .filter_map(|model| {
                    yaml_safe_catalog_id(&model.model_id).map(|id| json!({ "id": id, "name": id }))
                })
                .collect::<Vec<_>>();
            json!({
                "schema_version": 1,
                "surface": DSH_WEB_OVERLAY_SURFACE,
                "bound": true,
                "account_id": binding.account_id,
                "model": yaml_safe_catalog_id(&binding.model_id).unwrap_or(""),
                "catalog": catalog,
                "binding_revision": binding.revision,
            })
        }
        None => json!({
            "schema_version": 1,
            "surface": DSH_WEB_OVERLAY_SURFACE,
            "bound": false,
            "account_id": serde_json::Value::Null,
            "model": "",
            "catalog": [],
            "binding_revision": 0,
        }),
    }
}

fn overlay_projection_matches(existing: &serde_json::Value, next: &serde_json::Value) -> bool {
    existing.get("bound") == next.get("bound")
        && existing.get("account_id") == next.get("account_id")
        && existing.get("model") == next.get("model")
        && existing.get("catalog") == next.get("catalog")
        && existing.get("binding_revision") == next.get("binding_revision")
}

fn sync_dsh_web_control_plane_overlay(
    layout: &PersonalDataLayout,
    authority_store: &SqliteAuthorityStore,
    force: bool,
) -> Result<serde_json::Value, String> {
    let mut document = build_dsh_web_control_plane_overlay(authority_store);
    let home = dsh_web_home(layout);
    fs::create_dir_all(&home).map_err(|error| error.to_string())?;
    let path = home.join(DSH_WEB_OVERLAY_FILE);
    if !force
        && let Ok(text) = fs::read_to_string(&path)
        && let Ok(existing) = serde_json::from_str::<serde_json::Value>(&text)
        && overlay_projection_matches(&existing, &document)
    {
        return Ok(existing);
    }
    document["written_at_ms"] = json!(unix_now_ms());
    let encoded = serde_json::to_vec_pretty(&document).map_err(|error| error.to_string())?;
    fs::write(&path, encoded).map_err(|error| error.to_string())?;
    Ok(document)
}

fn wait_dsh_web_overlay_applied(layout: &PersonalDataLayout, written_at_ms: i64) -> bool {
    let path = dsh_web_home(layout).join(DSH_WEB_OVERLAY_APPLIED_FILE);
    let deadline = Instant::now() + Duration::from_secs(4);
    while Instant::now() < deadline {
        if let Ok(text) = fs::read_to_string(&path)
            && let Ok(value) = serde_json::from_str::<serde_json::Value>(&text)
            && value
                .get("written_at_ms")
                .and_then(serde_json::Value::as_i64)
                == Some(written_at_ms)
        {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

fn apply_dsh_binding_to_runtime(
    stream: &mut TcpStream,
    request_json: &serde_json::Value,
    layout: &PersonalDataLayout,
    authority_store: &Arc<SqliteAuthorityStore>,
    task_api: &TaskApi,
) -> Result<(), String> {
    let snapshot = task_api.dsh_runtime_snapshot();
    let state = snapshot
        .get("state")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    if state != "ACTIVE" {
        return write_error_response(
            stream,
            409,
            "DSH_RUNTIME_INACTIVE",
            "apply requires an ACTIVE Cos-installed dsh web process",
        );
    }
    let config_path = layout.config_dir().join("dsh.json");
    let config = match fs::read_to_string(&config_path)
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
    {
        Some(document) => document,
        None => {
            return write_error_response(
                stream,
                409,
                "DSH_RUNTIME_INVALID_REQUEST",
                "dsh.json is missing or unreadable",
            );
        }
    };
    let revision = config
        .get("revision")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let dsh_root = config
        .get("dsh_root")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    if revision != DSH_PACKAGE_REVISION || dsh_root.is_empty() || !Path::new(dsh_root).is_dir() {
        return write_error_response(
            stream,
            409,
            "DSH_RUNTIME_INVALID_REQUEST",
            "dsh.json pin or dsh_root is not the Cos-installed tree",
        );
    }
    let plane = ProviderControlPlaneStore::from_authority_store(authority_store);
    let Some(binding) = plane
        .get_active_binding(provider_control_plane::DSH_AGENT)
        .ok()
        .flatten()
    else {
        return write_error_response(
            stream,
            404,
            "PROVIDER_CONTROL_NOT_FOUND",
            "no active dsh binding to apply",
        );
    };
    if let Some(expected) = request_json
        .get("expected_revision")
        .and_then(serde_json::Value::as_i64)
        && expected != binding.revision
    {
        return write_error_response(
            stream,
            409,
            "PROVIDER_BINDING_REVISION_STALE",
            "expected_revision does not match the current dsh binding revision",
        );
    }
    if plane
        .get_model(&binding.account_id, &binding.model_id)
        .ok()
        .flatten()
        .is_none()
    {
        return write_error_response(
            stream,
            404,
            "PROVIDER_MODEL_NOT_FOUND",
            "dsh binding model is not in that account catalog",
        );
    }
    let account = plane.get_account(&binding.account_id).ok().flatten();
    let model_endpoint_compatible = account.as_ref().is_some_and(|account| {
        provider_control_plane::model_servable_on_account(account, &binding.model_id)
    });
    if let Ok(selected) = SelectedModel::new(binding.model_id.clone(), "binding", true) {
        let _ = SelectedModelRepository::under_config_dir(layout.config_dir()).store(&selected);
    }
    let overlay = sync_dsh_web_control_plane_overlay(layout, authority_store.as_ref(), true);
    let restart_performed = overlay
        .as_ref()
        .ok()
        .and_then(|document| {
            document
                .get("written_at_ms")
                .and_then(serde_json::Value::as_i64)
        })
        .is_some_and(|written_at_ms| wait_dsh_web_overlay_applied(layout, written_at_ms));
    let mut applied = task_api.dsh_runtime_snapshot();
    applied["op"] = json!("apply");
    applied["status"] = json!("ok");
    applied["applied"] = json!(true);
    applied["agent"] = json!(provider_control_plane::DSH_AGENT);
    applied["applied_model"] = json!(binding.model_id);
    applied["binding_revision"] = json!(binding.revision);
    applied["selected_snapshot_digest"] = json!("binding");
    applied["catalog_ok"] = json!(true);
    applied["dsh_root_ok"] = json!(true);
    applied["revision_pin_ok"] = json!(true);
    applied["native_catalog_overlays_cos_binding"] = json!(true);
    applied["overlay_written"] = json!(overlay.is_ok());
    if let Ok(document) = &overlay {
        applied["overlay_bound"] = document["bound"].clone();
        applied["overlay_catalog"] = document["catalog"].clone();
        applied["overlay_written_at_ms"] = document["written_at_ms"].clone();
    }
    applied["path_b_uses_bound_account"] = json!(true);
    applied["model_endpoint_compatible"] = json!(model_endpoint_compatible);
    applied["path_b_will_fail_closed"] = json!(!model_endpoint_compatible);
    applied["restart_performed"] = json!(restart_performed);
    applied["restart_required_for_native_models_chrome"] = json!(!restart_performed);
    write_json_response(stream, 200, &applied.to_string())
}

fn handle_selected_model_route(
    stream: &mut TcpStream,
    headers: &str,
    layout: &PersonalDataLayout,
    authority: &Arc<Mutex<LocalSessionAuthority>>,
    authority_store: &Arc<SqliteAuthorityStore>,
    agent: &str,
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

    if let Some(model_id) =
        provider_control_plane::selected_binding_model(authority_store.as_ref(), agent)
    {
        return write_json_response(
            stream,
            200,
            &json!({
                "schema_version": 1,
                "surface": "personal-provider-selected-model",
                "selected_model": model_id,
                "selected_snapshot_digest": "binding",
                "chat_capable": true,
                "authority_side_effects": false,
                "binding_agent": agent,
            })
            .to_string(),
        );
    }

    // Unbound agents still read the dedicated non-secret carrier. This never
    // creates a SecretStore or resolves Provider material.
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
                provider_secret_resolution_override: None,
                provider_secret_store_override: None,
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

fn header_values<'a>(headers: &'a str, name: &str) -> Vec<&'a str> {
    headers
        .lines()
        .filter_map(|line| {
            let (raw_name, value) = line.split_once(':')?;
            if raw_name.eq_ignore_ascii_case(name) {
                let trimmed = value.trim();
                (!trimmed.is_empty()).then_some(trimmed)
            } else {
                None
            }
        })
        .collect()
}

fn parse_authority(authority: &str) -> Option<(String, u16)> {
    let authority = authority.trim();
    if authority.is_empty() {
        return None;
    }
    if let Some(rest) = authority.strip_prefix('[') {
        let (host, after) = rest.split_once(']')?;
        let port = if after.is_empty() {
            80
        } else {
            after.strip_prefix(':')?.parse().ok()?
        };
        return Some((host.to_ascii_lowercase(), port));
    }
    match authority.rsplit_once(':') {
        Some((host, port)) if !host.is_empty() && !host.contains(':') => {
            Some((host.to_ascii_lowercase(), port.parse().ok()?))
        }
        _ => Some((authority.to_ascii_lowercase(), 80)),
    }
}

fn loopback_http_host(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost" | "localhost." | "::1")
}

fn parse_http_origin_url(value: &str) -> Option<(String, u16)> {
    let rest = value.trim().strip_prefix("http://")?;
    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    parse_authority(authority)
}

/// ADR-0053: a present Origin or Referer must be this daemon's loopback HTTP
/// origin. Missing headers remain allowed for CLI/curl. Cookies stay forbidden.
fn validate_browser_origin_headers(headers: &str) -> Option<&'static str> {
    let origins = header_values(headers, "origin");
    let referers = header_values(headers, "referer");
    if origins.is_empty() && referers.is_empty() {
        return None;
    }
    if origins.len() > 1 || referers.len() > 1 {
        return Some("origin header must be a single loopback product origin");
    }

    let hosts = header_values(headers, "host");
    if hosts.len() != 1 {
        return Some("origin header must be a single loopback product origin");
    }
    let Some((_, host_port)) = parse_authority(hosts[0]) else {
        return Some("origin header must be a single loopback product origin");
    };

    let mut candidates = Vec::new();
    if let Some(origin) = origins.first().copied() {
        candidates.push(origin);
    }
    if let Some(referer) = referers.first().copied() {
        candidates.push(referer);
    }
    if candidates.is_empty() {
        return None;
    }

    for candidate in candidates {
        if candidate.eq_ignore_ascii_case("null") {
            return Some("origin header must be a single loopback product origin");
        }
        let Some((name, port)) = parse_http_origin_url(candidate) else {
            return Some("origin header must be a single loopback product origin");
        };
        if !loopback_http_host(&name) || port != host_port {
            return Some("origin header must be a single loopback product origin");
        }
    }
    None
}

fn web_ui_relative_asset(request_path: &str) -> Result<String, &'static str> {
    let without_prefix = if request_path == "/ui" || request_path == "/ui/" {
        "index.html"
    } else {
        request_path
            .strip_prefix("/ui/")
            .ok_or("web UI path rejected")?
    };
    if without_prefix.is_empty() {
        return Ok("index.html".to_owned());
    }
    if without_prefix.contains('\\') || without_prefix.contains('\0') {
        return Err("web UI path rejected");
    }
    for segment in without_prefix.split('/') {
        if segment.is_empty()
            || segment == "."
            || segment == ".."
            || !segment.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_')
            })
        {
            return Err("web UI path rejected");
        }
    }
    Ok(without_prefix.to_owned())
}

fn web_ui_content_type(relative: &str) -> &'static str {
    match Path::new(relative)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("")
    {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "woff2" => "font/woff2",
        "txt" | "map" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn handle_web_ui_route(
    stream: &mut TcpStream,
    layout: &PersonalDataLayout,
    method_path: &str,
) -> Result<(), String> {
    let request_path = method_path
        .strip_prefix("GET ")
        .map(str::trim)
        .unwrap_or("");
    let relative = match web_ui_relative_asset(request_path) {
        Ok(relative) => relative,
        Err(message) => {
            return write_error_response(stream, 400, "LOCAL_UI_PATH_REJECTED", message);
        }
    };
    let bundle_root = layout.data_dir().join("ui");
    let asset = bundle_root.join(&relative);
    if relative == "index.html" && !asset.is_file() {
        return write_web_ui_response(
            stream,
            503,
            "text/html; charset=utf-8",
            WEB_UI_UNAVAILABLE_HTML.as_bytes(),
        );
    }
    let metadata = match fs::metadata(&asset) {
        Ok(metadata) if metadata.is_file() => metadata,
        _ => {
            return write_error_response(
                stream,
                404,
                "LOCAL_UI_ASSET_NOT_FOUND",
                "web UI asset not found",
            );
        }
    };
    if metadata.len() > WEB_UI_MAX_ASSET_BYTES {
        return write_error_response(
            stream,
            400,
            "LOCAL_UI_ASSET_TOO_LARGE",
            "web UI asset exceeds one mebibyte",
        );
    }
    let body = fs::read(&asset).map_err(|error| error.to_string())?;
    write_web_ui_response(stream, 200, web_ui_content_type(&relative), &body)
}

fn write_web_ui_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    let reason = match status {
        200 => "OK",
        503 => "Service Unavailable",
        _ => "Error",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nContent-Security-Policy: {WEB_UI_CSP}\r\nX-Content-Type-Options: nosniff\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    write_timed(stream, &header, body)
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
    write_timed(stream, &header, body)
}

/// Write one response and attribute its socket time to the transport
/// response-write stage rather than to route work.
fn write_timed(stream: &mut TcpStream, header: &str, body: &[u8]) -> Result<(), String> {
    let write_started = Instant::now();
    let outcome = stream
        .write_all(header.as_bytes())
        .and_then(|()| stream.write_all(body));
    loopback_transport::add_response_write(
        write_started.elapsed().as_nanos(),
        u64::try_from(header.len() + body.len()).unwrap_or(u64::MAX),
    );
    outcome.map_err(|error| error.to_string())
}

fn write_provider_response(
    stream: &mut TcpStream,
    status: u16,
    body: &[u8],
    provider_network_elapsed_nanos: u128,
    observation_headers: &str,
) -> Result<(), String> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        408 => "Request Timeout",
        409 => "Conflict",
        429 => "Too Many Requests",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Error",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nX-CognitiveOS-Provider-Network-Nanos: {provider_network_elapsed_nanos}\r\n{observation_headers}Content-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    write_timed(stream, &header, body)
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
        409 => "Conflict",
        429 => "Too Many Requests",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Error",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    write_timed(stream, &header, body)
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
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex, mpsc};
    use std::time::Duration;

    use cognitive_store::{PersonalDataLayout, SqliteAuthorityStore};

    use super::{
        LocalSessionAuthority, LoopbackTransportStage, PeriodicSchedulerWorker,
        PersonalResourceBounds, SchedulerTickRun, ensure_loopback_bind, handle_connection,
        loopback_transport, run_scheduler_tick_non_reentrant, validate_browser_origin_headers,
        web_ui_relative_asset,
    };
    use cognitive_runtime::loopback_transport::validate_loopback_transport_observation;

    fn test_fixture(
        test_name: &str,
    ) -> (
        PersonalDataLayout,
        Arc<Mutex<LocalSessionAuthority>>,
        Arc<SqliteAuthorityStore>,
    ) {
        use cognitive_store::prepare_personal_databases;

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
        prepare_personal_databases(&layout).expect("prepare test authority databases");
        let authority_store = Arc::new(
            SqliteAuthorityStore::open(&layout.authority_database_path())
                .expect("open shared test authority store"),
        );
        let authority = LocalSessionAuthority::initialize(
            layout.local_bootstrap_secret_path(),
            PersonalResourceBounds::personal_v1_baseline(),
        )
        .expect("test authority");
        (layout, Arc::new(Mutex::new(authority)), authority_store)
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
    fn browser_origin_allowlist_rejects_foreign_and_null() {
        assert!(validate_browser_origin_headers("Host: 127.0.0.1:48181\r\n").is_none());
        assert!(
            validate_browser_origin_headers(
                "Host: 127.0.0.1:48181\r\nOrigin: http://127.0.0.1:48181\r\n",
            )
            .is_none()
        );
        assert!(
            validate_browser_origin_headers(
                "Host: 127.0.0.1:48181\r\nOrigin: http://localhost:48181\r\n",
            )
            .is_none()
        );
        assert!(
            validate_browser_origin_headers(
                "Host: 127.0.0.1:48181\r\nReferer: http://127.0.0.1:48181/ui/\r\n",
            )
            .is_none()
        );
        assert!(
            validate_browser_origin_headers(
                "Host: 127.0.0.1:48181\r\nOrigin: http://evil.example\r\n",
            )
            .is_some()
        );
        assert!(
            validate_browser_origin_headers("Host: 127.0.0.1:48181\r\nOrigin: null\r\n").is_some()
        );
        assert!(
            validate_browser_origin_headers(
                "Host: 127.0.0.1:48181\r\nOrigin: https://127.0.0.1:48181\r\n",
            )
            .is_some()
        );
        assert!(
            validate_browser_origin_headers(
                "Host: 127.0.0.1:48181\r\nOrigin: http://127.0.0.1:80\r\n",
            )
            .is_some()
        );
        assert!(
            validate_browser_origin_headers(
                "Host: 127.0.0.1:48181\r\nReferer: http://evil.example/ui/\r\n",
            )
            .is_some()
        );
    }

    #[test]
    fn web_ui_paths_reject_traversal_and_percent_encoding() {
        assert_eq!(web_ui_relative_asset("/ui").unwrap(), "index.html");
        assert_eq!(web_ui_relative_asset("/ui/").unwrap(), "index.html");
        assert_eq!(
            web_ui_relative_asset("/ui/assets/app.js").unwrap(),
            "assets/app.js"
        );
        assert!(web_ui_relative_asset("/ui/../authority.sqlite").is_err());
        assert!(web_ui_relative_asset("/ui/%2e%2e/secret").is_err());
        assert!(web_ui_relative_asset("/ui/foo/../../etc/passwd").is_err());
        assert!(web_ui_relative_asset("/ui/foo\\bar").is_err());
    }

    fn front_door_exchange(
        test_name: &str,
        prepare: impl FnOnce(&PersonalDataLayout),
        request: impl FnOnce(u16) -> String,
    ) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test listener");
        let port = listener.local_addr().expect("listener address").port();
        let bounds = PersonalResourceBounds::personal_v1_baseline();
        let (layout, authority, authority_store) = test_fixture(test_name);
        prepare(&layout);
        let active_connections = Arc::new(AtomicUsize::new(0));
        let in_flight = Arc::new(AtomicUsize::new(0));
        let server = std::thread::spawn({
            let authority = Arc::clone(&authority);
            let authority_store = Arc::clone(&authority_store);
            let active_connections = Arc::clone(&active_connections);
            let in_flight = Arc::clone(&in_flight);
            move || {
                handle_connection(
                    accept_connection(&listener),
                    &bounds,
                    &layout,
                    &authority,
                    &authority_store,
                    &active_connections,
                    &in_flight,
                );
            }
        });
        let mut client = TcpStream::connect(("127.0.0.1", port)).expect("client connection");
        client
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("client timeout");
        client.write_all(request(port).as_bytes()).expect("request");
        let mut response = String::new();
        client.read_to_string(&mut response).expect("response");
        server.join().expect("server thread");
        response
    }

    #[test]
    fn foreign_origin_is_rejected_on_the_front_door() {
        let response = front_door_exchange(
            "origin-foreign",
            |_| {},
            |port| {
                format!(
                    "GET /personal/health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: http://evil.example\r\nConnection: close\r\n\r\n"
                )
            },
        );
        assert!(response.contains("HTTP/1.1 403"), "{response}");
        assert!(
            response.contains("LOCAL_ORIGIN_HEADER_REJECTED"),
            "{response}"
        );
        assert!(!response.contains("personal-health"), "{response}");
    }

    #[test]
    fn matching_loopback_origin_is_accepted_on_health() {
        let response = front_door_exchange(
            "origin-loopback",
            |_| {},
            |port| {
                format!(
                    "GET /personal/health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nOrigin: http://127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
                )
            },
        );
        assert!(response.contains("HTTP/1.1 200"), "{response}");
        assert!(response.contains("personal-health"), "{response}");
    }

    #[test]
    fn missing_ui_bundle_is_not_available_not_a_fake_spa() {
        let response = front_door_exchange(
            "ui-missing",
            |_| {},
            |_| "GET /ui HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n".to_owned(),
        );
        assert!(response.contains("HTTP/1.1 503"), "{response}");
        assert!(
            response.contains("LOCAL_UI_BUNDLE_UNAVAILABLE"),
            "{response}"
        );
        assert!(response.contains("not_available"), "{response}");
        assert!(
            response.contains("Content-Security-Policy: default-src 'self'"),
            "{response}"
        );
        assert!(!response.contains("<script"), "{response}");
    }

    #[test]
    fn web_ui_serves_index_with_csp_and_rejects_traversal() {
        let served = front_door_exchange(
            "ui-index",
            |layout| {
                let ui = layout.data_dir().join("ui");
                std::fs::create_dir_all(&ui).expect("ui dir");
                std::fs::write(ui.join("index.html"), "<!doctype html><p>ok</p>").expect("index");
            },
            |_| "GET /ui/ HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n".to_owned(),
        );
        assert!(served.contains("HTTP/1.1 200"), "{served}");
        assert!(served.contains("<p>ok</p>"), "{served}");
        assert!(
            served.contains("Content-Security-Policy: default-src 'self'"),
            "{served}"
        );

        let traversal = front_door_exchange(
            "ui-traversal",
            |_| {},
            |_| {
                "GET /ui/../authority.sqlite HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
                .to_owned()
            },
        );
        assert!(traversal.contains("HTTP/1.1 400"), "{traversal}");
        assert!(traversal.contains("LOCAL_UI_PATH_REJECTED"), "{traversal}");
        assert!(!traversal.contains("SQLite"), "{traversal}");
    }

    #[test]
    fn scheduler_tick_gate_rejects_self_reentry() {
        let active = AtomicBool::new(false);
        let nested_tick_called = AtomicBool::new(false);

        let outer = run_scheduler_tick_non_reentrant(&active, || {
            let nested = run_scheduler_tick_non_reentrant(&active, || {
                nested_tick_called.store(true, Ordering::SeqCst);
                Ok::<(), String>(())
            });
            assert!(matches!(nested, SchedulerTickRun::AlreadyRunning));
            Ok::<(), String>(())
        });

        assert!(matches!(outer, SchedulerTickRun::Executed(Ok(()))));
        assert!(!nested_tick_called.load(Ordering::SeqCst));
        assert!(!active.load(Ordering::SeqCst));
    }

    #[test]
    fn periodic_scheduler_worker_survives_tick_error_and_cancels_cleanly() {
        let calls = Arc::new(AtomicUsize::new(0));
        let active = Arc::new(AtomicUsize::new(0));
        let maximum_active = Arc::new(AtomicUsize::new(0));
        let (call_sender, call_receiver) = mpsc::channel();
        let mut worker = PeriodicSchedulerWorker::spawn(Duration::from_millis(10), {
            let calls = Arc::clone(&calls);
            let active = Arc::clone(&active);
            let maximum_active = Arc::clone(&maximum_active);
            move || {
                let current_active = active.fetch_add(1, Ordering::SeqCst) + 1;
                maximum_active.fetch_max(current_active, Ordering::SeqCst);
                let call = calls.fetch_add(1, Ordering::SeqCst) + 1;
                call_sender.send(call).unwrap();
                active.fetch_sub(1, Ordering::SeqCst);
                if call == 1 {
                    Err("injected row-independent tick failure".to_owned())
                } else {
                    Ok(())
                }
            }
        })
        .unwrap();

        assert_eq!(
            call_receiver.recv_timeout(Duration::from_secs(1)).unwrap(),
            1
        );
        assert_eq!(
            call_receiver.recv_timeout(Duration::from_secs(1)).unwrap(),
            2,
            "a failed pass must not terminate the periodic worker"
        );
        worker.shutdown().unwrap();
        assert_eq!(maximum_active.load(Ordering::SeqCst), 1);
        let stopped_at = calls.load(Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(40));
        assert_eq!(calls.load(Ordering::SeqCst), stopped_at);
    }

    /// P9-T04/D02: one real loopback request must produce disjoint transport
    /// stages that stay separate from `effect_persistence` and never retain the
    /// credential the client presented.
    #[test]
    fn real_loopback_request_records_redacted_disjoint_transport_stages() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test listener");
        let port = listener.local_addr().expect("listener address").port();
        let bounds = PersonalResourceBounds::personal_v1_baseline();
        let (layout, authority, authority_store) = test_fixture("transport-stages");
        let active_connections = Arc::new(AtomicUsize::new(0));
        let in_flight = Arc::new(AtomicUsize::new(0));
        let server = std::thread::spawn({
            let authority = Arc::clone(&authority);
            let authority_store = Arc::clone(&authority_store);
            let active_connections = Arc::clone(&active_connections);
            let in_flight = Arc::clone(&in_flight);
            move || {
                handle_connection(
                    accept_connection(&listener),
                    &bounds,
                    &layout,
                    &authority,
                    &authority_store,
                    &active_connections,
                    &in_flight,
                );
                loopback_transport::last_observation()
            }
        });

        let mut client = TcpStream::connect(("127.0.0.1", port)).expect("client connection");
        client
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("client timeout");
        client
            .write_all(
                b"GET /personal/health HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer transport-probe-value\r\nConnection: close\r\n\r\n",
            )
            .expect("request");
        let mut response = String::new();
        client.read_to_string(&mut response).expect("response");
        let observation = server
            .join()
            .expect("server thread")
            .expect("transport observation");

        assert!(response.contains("personal-health"), "{response}");
        validate_loopback_transport_observation(&observation).expect("publishable observation");
        assert_eq!(
            observation
                .stages
                .iter()
                .map(|sample| sample.stage)
                .collect::<Vec<_>>(),
            vec![
                LoopbackTransportStage::ConnectionAdmission,
                LoopbackTransportStage::RequestRead,
                LoopbackTransportStage::HeaderAdmission,
                LoopbackTransportStage::RouteDispatch,
                LoopbackTransportStage::ResponseWrite,
            ]
        );
        assert!(observation.stages.iter().all(|sample| !sample.omitted));
        assert!(observation.request_bytes > 0);
        assert!(observation.response_bytes > 0);
        assert!(
            observation
                .excluded_attributions
                .contains(&"effect_persistence")
        );
        let serialized = serde_json::to_string(&observation).expect("serialize observation");
        assert!(
            !serialized.contains("transport-probe-value"),
            "{serialized}"
        );
        assert!(!serialized.contains("Bearer"), "{serialized}");
    }

    #[test]
    fn slow_header_read_times_out_with_stable_protocol_code() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test listener");
        let port = listener.local_addr().expect("listener address").port();
        let mut bounds = PersonalResourceBounds::personal_v1_baseline();
        bounds.read_header_timeout_secs = 1;
        let (layout, authority, authority_store) = test_fixture("timeout");
        let active_connections = Arc::new(AtomicUsize::new(0));
        let in_flight = Arc::new(AtomicUsize::new(0));
        let server = std::thread::spawn({
            let authority = Arc::clone(&authority);
            let authority_store = Arc::clone(&authority_store);
            let active_connections = Arc::clone(&active_connections);
            let in_flight = Arc::clone(&in_flight);
            move || {
                handle_connection(
                    accept_connection(&listener),
                    &bounds,
                    &layout,
                    &authority,
                    &authority_store,
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
        let (layout, authority, authority_store) = test_fixture("body-timeout");
        let active_connections = Arc::new(AtomicUsize::new(0));
        let in_flight = Arc::new(AtomicUsize::new(0));
        let server = std::thread::spawn({
            let authority = Arc::clone(&authority);
            let authority_store = Arc::clone(&authority_store);
            let active_connections = Arc::clone(&active_connections);
            let in_flight = Arc::clone(&in_flight);
            move || {
                handle_connection(
                    accept_connection(&listener),
                    &bounds,
                    &layout,
                    &authority,
                    &authority_store,
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
        let (layout, authority, authority_store) = test_fixture("concurrency");
        let active_connections = Arc::new(AtomicUsize::new(0));
        let in_flight = Arc::new(AtomicUsize::new(0));
        let mut first = TcpStream::connect(("127.0.0.1", port)).expect("first connection");
        let first_listener = listener.try_clone().expect("first listener clone");
        let first_server = std::thread::spawn({
            let authority = Arc::clone(&authority);
            let layout = layout.clone();
            let active_connections = Arc::clone(&active_connections);
            let in_flight = Arc::clone(&in_flight);
            let authority_store = Arc::clone(&authority_store);
            let bounds = bounds;
            move || {
                handle_connection(
                    accept_connection(&first_listener),
                    &bounds,
                    &layout,
                    &authority,
                    &authority_store,
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
            let authority_store = Arc::clone(&authority_store);
            let bounds = bounds;
            move || {
                handle_connection(
                    accept_connection(&second_listener),
                    &bounds,
                    &layout,
                    &authority,
                    &authority_store,
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
            let authority_store = Arc::clone(&authority_store);
            let bounds = bounds;
            move || {
                handle_connection(
                    accept_connection(&third_listener),
                    &bounds,
                    &layout,
                    &authority,
                    &authority_store,
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
        let (layout, authority, authority_store) = test_fixture("in-flight");
        let active_connections = Arc::new(AtomicUsize::new(0));
        let in_flight = Arc::new(AtomicUsize::new(0));

        let mut first = TcpStream::connect(("127.0.0.1", port)).expect("first connection");
        let first_listener = listener.try_clone().expect("first listener clone");
        let first_server = std::thread::spawn({
            let authority = Arc::clone(&authority);
            let layout = layout.clone();
            let active_connections = Arc::clone(&active_connections);
            let in_flight = Arc::clone(&in_flight);
            let authority_store = Arc::clone(&authority_store);
            let bounds = bounds;
            move || {
                handle_connection(
                    accept_connection(&first_listener),
                    &bounds,
                    &layout,
                    &authority,
                    &authority_store,
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
            let authority_store = Arc::clone(&authority_store);
            let bounds = bounds;
            move || {
                handle_connection(
                    accept_connection(&second_listener),
                    &bounds,
                    &layout,
                    &authority,
                    &authority_store,
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
            let authority_store = Arc::clone(&authority_store);
            let bounds = bounds;
            move || {
                handle_connection(
                    accept_connection(&third_listener),
                    &bounds,
                    &layout,
                    &authority,
                    &authority_store,
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

    #[test]
    fn request_path_handlers_reuse_daemon_owned_authority_store() {
        use std::time::Instant;

        use cognitive_kernel::ProtocolStore;

        use super::{
            ChannelClass, ResourceApi, SessionIssueRequest, handle_task_consumption_route,
        };

        let (_layout, authority, authority_store) = test_fixture("shared-store-request");
        let task_token = {
            let mut guard = authority.lock().expect("authority lock");
            let bootstrap_secret = guard.bootstrap_secret_for_tests().to_owned();
            guard
                .issue_session(
                    SessionIssueRequest {
                        channel: ChannelClass::Task,
                        principal_id: "principal://tenant-a/owner".to_owned(),
                        bootstrap_secret,
                    },
                    Instant::now(),
                )
                .expect("issue task session")
                .token
        };
        let resource_api = Arc::new(Mutex::new(ResourceApi::new()));
        let body = br#"{"task_ref":"task://local/missing","query_text":"fact","skill_binding_id":"00000000-0000-7000-9000-000000000001"}"#;
        let headers = format!("Authorization: Bearer {task_token}\r\n");

        // Two sequential request-path calls must reuse the same open store and
        // still reach the store-backed fail-closed outcome (no second open).
        for _ in 0..2 {
            let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
            let port = listener.local_addr().expect("addr").port();
            let server = {
                let authority = Arc::clone(&authority);
                let authority_store = Arc::clone(&authority_store);
                let resource_api = Arc::clone(&resource_api);
                let headers = headers.clone();
                std::thread::spawn(move || {
                    let mut stream = accept_connection(&listener);
                    handle_task_consumption_route(
                        &mut stream,
                        &headers,
                        body,
                        &authority,
                        &authority_store,
                        &resource_api,
                    )
                    .expect("shared-store consumption route");
                })
            };
            let mut client = TcpStream::connect(("127.0.0.1", port)).expect("client");
            client
                .set_read_timeout(Some(Duration::from_secs(5)))
                .expect("client timeout");
            let mut response = String::new();
            client
                .read_to_string(&mut response)
                .expect("shared-store response");
            server.join().expect("server thread");
            assert!(
                response.contains("RESOURCE_TASK_NOT_FOUND"),
                "expected store-backed miss on shared handle, got {response}"
            );
        }

        assert_eq!(
            authority_store
                .current_contract_epoch("task://local/missing")
                .expect("shared store still readable"),
            0
        );
    }
}
