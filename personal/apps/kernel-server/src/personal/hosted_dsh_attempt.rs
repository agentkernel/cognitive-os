//! Personal-private hosted DSH Attempt routes (P13-T02).
//!
//! `POST /management/project/v1/dsh.hosted.attempt.run` is the real caller of
//! the hidden hosted DSH loop: it records an artifact health fact, persists the
//! Attempt Intent, binds the v31 child identity, and only then lets the
//! `cognitive-runtime` broker spawn the exact-artifact child with a bounded
//! Context on stdin. Frames come back as observations; the daemon writes the
//! terminal observation itself. Process death is never completion, unknown
//! output is never success, and the child reaches the Provider only through
//! the daemon proxy. Management-channel only; task-channel aliases are 403.
//! Windows sandbox / ACL / supply-chain E2E remains `not-run` until `P13-T13`.

use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use cognitive_runtime::{
    HOSTED_DEFAULT_TIMEOUT, HOSTED_MAX_TIMEOUT, HostedBrokerError, HostedContextPayload,
    HostedDshArtifact, HostedTerminalKind, run_hosted_child,
};
use cognitive_store::{
    ConfirmCaller, EmployeeStore, HOSTED_ATTEMPT_PROJECTION_ID, HOSTED_DSH_ARTIFACT_DIGEST,
    HOSTED_DSH_ENGINE_ID, HOSTED_DSH_PROTOCOL, HOSTED_DSH_PROVIDER_PROXY, HostedArtifactFact,
    HostedAttemptFrameRow, HostedAttemptIntentSpec, HostedAttemptRow, HostedAttemptTerminalSpec,
    HostedDshAttemptStore, HostedDshPlane, HostedDshStartSpec, PersonalDataLayout,
    ProjectAggregateError, SqliteAuthorityStore,
};
use serde_json::{Value, json};

use super::resource_api::ResourceApiResponse;

const ROUTE_LITERALS: &[&str] = &[
    "POST /management/project/v1/dsh.hosted.attempt.run",
    "GET /management/project/v1/dsh.hosted.attempt.list",
    "GET /management/project/v1/dsh.hosted.attempt.detail",
    "POST /management/project/v1/dsh.hosted.artifact.check",
    "GET /management/project/v1/dsh.hosted.artifact.facts",
    "POST /task/project/v1/dsh.hosted.attempt.run",
    "GET /task/project/v1/dsh.hosted.attempt.list",
    "GET /task/project/v1/dsh.hosted.attempt.detail",
    "POST /task/project/v1/dsh.hosted.artifact.check",
    "GET /task/project/v1/dsh.hosted.artifact.facts",
];

const ENDPOINT_FILE_NAME: &str = "daemon-endpoint.json";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Channel {
    Management,
    Task,
}

/// Daemon-owned host facts the broker needs. Paths only — never secret bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HostedAttemptHost {
    pub config_dir: PathBuf,
    pub bootstrap_file: PathBuf,
    pub endpoint_file: PathBuf,
    /// The daemon's single P3-T03 CAS root (`<data_dir>/artifacts`), where a
    /// terminal Attempt's `DeliverableDraft` candidates are ingested (P13-T04).
    pub artifact_root: PathBuf,
}

impl HostedAttemptHost {
    pub(crate) fn from_layout(layout: &PersonalDataLayout) -> Self {
        Self {
            config_dir: layout.config_dir().to_path_buf(),
            bootstrap_file: layout.local_bootstrap_secret_path(),
            endpoint_file: layout.state_dir().join(ENDPOINT_FILE_NAME),
            artifact_root: super::attempt_artifacts::artifact_root(layout),
        }
    }

    /// Loopback daemon origin from the published endpoint file, if any.
    fn daemon_origin(&self) -> Option<String> {
        let document = std::fs::read_to_string(&self.endpoint_file).ok()?;
        let value: Value = serde_json::from_str(&document).ok()?;
        let endpoint = value.get("endpoint")?.as_str()?;
        let socket: std::net::SocketAddr = endpoint.parse().ok()?;
        if !socket.ip().is_loopback() {
            return None;
        }
        Some(format!("http://{endpoint}"))
    }
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
        "HOSTED_ATTEMPT_CHANNEL_FORBIDDEN",
        "hosted DSH Attempt operations are management-channel only",
    )
}

pub(crate) fn handle(
    method_path: &str,
    body: &[u8],
    store: &SqliteAuthorityStore,
    host: &HostedAttemptHost,
) -> ResourceApiResponse {
    let Some((channel, literal)) = parse_route(method_path) else {
        return error(
            404,
            "HOSTED_ATTEMPT_ROUTE_NOT_FOUND",
            "no hosted DSH Attempt route matched",
        );
    };
    if channel == Channel::Task {
        return channel_forbidden();
    }
    match literal {
        "POST /management/project/v1/dsh.hosted.attempt.run" => attempt_run(body, store, host),
        "GET /management/project/v1/dsh.hosted.attempt.list" => attempt_list(method_path, store),
        "GET /management/project/v1/dsh.hosted.attempt.detail" => {
            attempt_detail(method_path, store)
        }
        "POST /management/project/v1/dsh.hosted.artifact.check" => artifact_check(store, host),
        "GET /management/project/v1/dsh.hosted.artifact.facts" => {
            artifact_facts(method_path, store)
        }
        _ => error(
            404,
            "HOSTED_ATTEMPT_ROUTE_NOT_FOUND",
            "no hosted DSH Attempt route matched",
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

fn record_artifact_fact(
    store: &SqliteAuthorityStore,
    config_dir: &Path,
) -> Result<HostedArtifactFact, ProjectAggregateError> {
    let observation = HostedDshArtifact::observe(config_dir);
    HostedDshAttemptStore::from_authority_store(store).record_artifact_observation(
        ConfirmCaller::OwnerManagement,
        &observation,
        now_ms(),
    )
}

fn artifact_check(store: &SqliteAuthorityStore, host: &HostedAttemptHost) -> ResourceApiResponse {
    match record_artifact_fact(store, &host.config_dir) {
        Ok(fact) => ok(json!({
            "projection": HOSTED_ATTEMPT_PROJECTION_ID,
            "fact": fact_json(&fact),
            "admits_spawn": fact.admits_spawn(),
            "windows_supply_chain_e2e": "not-run",
        })),
        Err(err) => store_error(err),
    }
}

fn artifact_facts(method_path: &str, store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let limit = query_parameter(method_path, "limit")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(16);
    let attempts = HostedDshAttemptStore::from_authority_store(store);
    match (
        attempts.latest_artifact_fact(),
        attempts.list_artifact_facts(limit),
    ) {
        (Ok(latest), Ok(history)) => ok(json!({
            "projection": HOSTED_ATTEMPT_PROJECTION_ID,
            "expected_revision": HOSTED_DSH_ARTIFACT_DIGEST,
            "latest": latest.as_ref().map(fact_json),
            "admits_spawn": latest.as_ref().is_some_and(HostedArtifactFact::admits_spawn),
            "history": history.iter().map(fact_json).collect::<Vec<_>>(),
            "windows_supply_chain_e2e": "not-run",
        })),
        (Err(err), _) | (_, Err(err)) => store_error(err),
    }
}

fn attempt_list(method_path: &str, store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let Some(project_id) = query_parameter(method_path, "project_id").filter(|v| !v.is_empty())
    else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    let limit = query_parameter(method_path, "limit")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(32);
    match HostedDshAttemptStore::from_authority_store(store).list_attempts(&project_id, limit) {
        Ok(rows) => redacted_ok(json!({
            "projection": HOSTED_ATTEMPT_PROJECTION_ID,
            "project_id": project_id,
            "attempts": rows.iter().map(attempt_json).collect::<Vec<_>>(),
            "receipt_is_not_completion": true,
        })),
        Err(err) => store_error(err),
    }
}

fn attempt_detail(method_path: &str, store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let Some(attempt_id) = query_parameter(method_path, "attempt_id").filter(|v| !v.is_empty())
    else {
        return error(400, "ATTEMPT_ID_REQUIRED", "attempt_id required");
    };
    let limit = query_parameter(method_path, "limit")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(128);
    let attempts = HostedDshAttemptStore::from_authority_store(store);
    match attempts.get_attempt(&attempt_id) {
        Ok(None) => error(404, "HOSTED_ATTEMPT_NOT_FOUND", "attempt not found"),
        Ok(Some(row)) => match attempts.list_frames(&attempt_id, limit) {
            Ok(frames) => redacted_ok(json!({
                "projection": HOSTED_ATTEMPT_PROJECTION_ID,
                "attempt": attempt_json(&row),
                "frames": frames.iter().map(frame_json).collect::<Vec<_>>(),
                "receipt_is_not_completion": true,
            })),
            Err(err) => store_error(err),
        },
        Err(err) => store_error(err),
    }
}

fn attempt_run(
    body: &[u8],
    store: &SqliteAuthorityStore,
    host: &HostedAttemptHost,
) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "HOSTED_ATTEMPT_JSON_REQUIRED", "JSON body required");
    };
    let Some(employee_id) = document.get("employee_id").and_then(Value::as_str) else {
        return error(400, "EMPLOYEE_ID_REQUIRED", "employee_id required");
    };
    let Some(task_ref) = document.get("task_ref").and_then(Value::as_str) else {
        return error(400, "TASK_REF_REQUIRED", "task_ref required");
    };
    let Some(bounded_context) = document.get("bounded_context").and_then(Value::as_str) else {
        return error(400, "BOUNDED_CONTEXT_REQUIRED", "bounded_context required");
    };
    let wait = document
        .get("wait")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let timeout = document
        .get("timeout_ms")
        .and_then(Value::as_u64)
        .map(Duration::from_millis)
        .unwrap_or(HOSTED_DEFAULT_TIMEOUT);
    if timeout.is_zero() || timeout > HOSTED_MAX_TIMEOUT {
        return error(
            422,
            "HOSTED_ATTEMPT_TIMEOUT_INVALID",
            "timeout_ms must be within (0, 30m]",
        );
    }
    let employees = EmployeeStore::from_authority_store(store);
    let latest_revision = match employees.latest_revision_id(employee_id) {
        Ok(Some(revision)) => revision,
        Ok(None) => {
            return error(
                404,
                "HOSTED_ATTEMPT_NOT_FOUND",
                "employee revision not found",
            );
        }
        Err(err) => return store_error(err),
    };
    let employee_revision_id = document
        .get("employee_revision_id")
        .and_then(Value::as_str)
        .unwrap_or(latest_revision.as_str());

    // 1. Artifact health fact first: an unhealthy artifact never reaches spawn.
    let fact = match record_artifact_fact(store, &host.config_dir) {
        Ok(fact) => fact,
        Err(err) => return store_error(err),
    };
    if !fact.admits_spawn() {
        return ResourceApiResponse {
            status: 422,
            body: json!({
                "status": "error",
                "code": "HOSTED_ARTIFACT_UNHEALTHY",
                "message": "hosted DSH artifact is not pinned; spawn refused",
                "fact": fact_json(&fact),
            })
            .to_string(),
            content_type: "application/json",
        };
    }

    // 2. Persist the Attempt Intent before anything is spawned.
    let attempts = HostedDshAttemptStore::from_authority_store(store);
    let attempt = match attempts.persist_intent(
        ConfirmCaller::OwnerManagement,
        &HostedAttemptIntentSpec {
            employee_id,
            employee_revision_id,
            task_ref,
            bounded_context,
            artifact_digest: HOSTED_DSH_ARTIFACT_DIGEST,
            now_ms: now_ms(),
        },
    ) {
        Ok(attempt) => attempt,
        Err(err) => return store_error(err),
    };

    // 3. Resolve the exact artifact and the launch plan (still nothing spawned).
    let artifact = match HostedDshArtifact::resolve(&host.config_dir) {
        Ok(artifact) => artifact,
        Err(err) => return spawn_refused(&attempts, &attempt, &err.to_string()),
    };
    let plan = artifact.launch_plan(timeout);
    let Some(daemon_origin) = host.daemon_origin() else {
        return spawn_refused(
            &attempts,
            &attempt,
            "daemon endpoint is not published; the child cannot reach the Provider proxy",
        );
    };
    let payload = HostedContextPayload {
        attempt_id: attempt.attempt_id.clone(),
        task_ref: task_ref.to_owned(),
        employee_id: employee_id.to_owned(),
        project_id: attempt.project_id.clone(),
        bounded_context: bounded_context.to_owned(),
        daemon_origin: Some(daemon_origin),
        bootstrap_file: Some(host.bootstrap_file.clone()),
    };

    // 4. Bind the v31 child identity onto the Employee (pid arrives at spawn).
    let plane = HostedDshPlane::from_authority_store(store);
    let argv: Vec<&str> = plan.args.iter().map(String::as_str).collect();
    let env_pairs: Vec<(&str, &str)> = plan
        .env
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    let child = match plane.start(
        ConfirmCaller::OwnerManagement,
        &HostedDshStartSpec {
            employee_id,
            employee_revision_id,
            task_ref,
            bounded_context,
            artifact_digest: HOSTED_DSH_ARTIFACT_DIGEST,
            protocol: HOSTED_DSH_PROTOCOL,
            engine_id: HOSTED_DSH_ENGINE_ID,
            observed_pid: None,
            argv: &argv,
            env_pairs: &env_pairs,
            child_output: None,
            now_ms: now_ms(),
        },
    ) {
        Ok(child) => child,
        Err(err) => return spawn_refused(&attempts, &attempt, &err.to_string()),
    };
    if let Err(err) = attempts.bind_child_identity(&attempt.attempt_id, &child.child_id) {
        return store_error(err);
    }

    // 5. Real spawn on a daemon thread; the terminal observation is written by
    //    the daemon regardless of what the child claimed.
    let worker_store = store.clone();
    let attempt_id = attempt.attempt_id.clone();
    let child_id = child.child_id.clone();
    let worker_employee_id = employee_id.to_owned();
    let worker_artifact_root = host.artifact_root.clone();
    let handle = thread::Builder::new()
        .name(format!("hosted-dsh-{attempt_id}"))
        .spawn(move || {
            run_attempt_child(
                &worker_store,
                &worker_artifact_root,
                &attempt_id,
                &child_id,
                &worker_employee_id,
                &plan,
                &payload,
            )
        });
    let handle = match handle {
        Ok(handle) => handle,
        Err(_) => {
            return spawn_refused(
                &attempts,
                &attempt,
                "daemon could not start the broker thread",
            );
        }
    };
    if wait {
        return match handle.join() {
            Ok(Ok(row)) => attempt_response(&row, &fact, &child.runtime_binding_ref),
            Ok(Err(err)) => store_error(err),
            Err(_) => error(
                503,
                "HOSTED_ATTEMPT_UNAVAILABLE",
                "broker thread panicked; the Attempt row keeps its last durable state",
            ),
        };
    }
    drop(handle);
    match attempts.get_attempt(&attempt.attempt_id) {
        Ok(Some(row)) => attempt_response(&row, &fact, &child.runtime_binding_ref),
        Ok(None) => error(404, "HOSTED_ATTEMPT_NOT_FOUND", "attempt not found"),
        Err(err) => store_error(err),
    }
}

/// Broker thread body: spawn, observe frames, write the terminal observation,
/// then hand the terminal run to the P13-T04 artifact ingest (CAS put +
/// independent verifier) — daemon-side, never from the child.
fn run_attempt_child(
    store: &SqliteAuthorityStore,
    artifact_root: &Path,
    attempt_id: &str,
    child_id: &str,
    employee_id: &str,
    plan: &cognitive_runtime::HostedChildLaunchPlan,
    payload: &HostedContextPayload,
) -> Result<HostedAttemptRow, ProjectAggregateError> {
    let attempts = HostedDshAttemptStore::from_authority_store(store);
    let plane = HostedDshPlane::from_authority_store(store);
    let outcome = run_hosted_child(plan, payload, |pid| {
        let _ = attempts.mark_dispatched(attempt_id, Some(child_id), pid, now_ms());
        let _ = plane.observe_spawn(child_id, pid, now_ms());
    });
    let row = match outcome {
        Ok(run) => {
            let frames = run.ledger_frames();
            let _ = attempts.record_frames(attempt_id, &frames, now_ms());
            let response_status = run.response_status.clone();
            let terminal = attempts.record_terminal(
                attempt_id,
                &HostedAttemptTerminalSpec {
                    terminal_kind: run.terminal.as_str(),
                    exit_code: run.terminal.exit_code(),
                    response_status: response_status.as_deref(),
                    candidate_count: run.candidate_count(),
                    observation_count: run.observation_count(),
                    rejected_frame_count: run.rejected_frames.len(),
                    unknown_line_count: run.unknown_lines,
                    stdout_bytes: run.stdout_bytes,
                    stdout_truncated: run.stdout_truncated,
                    stderr_tail_redacted: &run.stderr_tail_redacted,
                    elapsed_ms: run.elapsed_ms,
                    now_ms: now_ms(),
                },
            )?;
            super::attempt_artifacts::ingest_terminal_run(store, artifact_root, attempt_id, &run);
            terminal
        }
        Err(refused) => attempts.record_terminal(
            attempt_id,
            &HostedAttemptTerminalSpec {
                terminal_kind: HostedTerminalKind::SpawnFailed.as_str(),
                exit_code: None,
                response_status: None,
                candidate_count: 0,
                observation_count: 0,
                rejected_frame_count: 0,
                unknown_line_count: 0,
                stdout_bytes: 0,
                stdout_truncated: false,
                stderr_tail_redacted: &broker_refusal_text(&refused),
                elapsed_ms: 0,
                now_ms: now_ms(),
            },
        )?,
    };
    let _ = plane.observe_exit(employee_id);
    Ok(row)
}

fn broker_refusal_text(refused: &HostedBrokerError) -> String {
    format!("spawn refused before any process existed: {refused}")
}

/// Pre-spawn refusal: the persisted Intent gets a `spawn-failed` terminal so
/// it never lingers as a crash-shaped row, and the caller sees 422.
fn spawn_refused(
    attempts: &HostedDshAttemptStore,
    attempt: &HostedAttemptRow,
    detail: &str,
) -> ResourceApiResponse {
    let terminal = attempts.record_terminal(
        &attempt.attempt_id,
        &HostedAttemptTerminalSpec {
            terminal_kind: "spawn-failed",
            exit_code: None,
            response_status: None,
            candidate_count: 0,
            observation_count: 0,
            rejected_frame_count: 0,
            unknown_line_count: 0,
            stdout_bytes: 0,
            stdout_truncated: false,
            stderr_tail_redacted: detail,
            elapsed_ms: 0,
            now_ms: now_ms(),
        },
    );
    match terminal {
        Ok(row) => ResourceApiResponse {
            status: 422,
            body: json!({
                "status": "error",
                "code": "HOSTED_ATTEMPT_SPAWN_REFUSED",
                "message": "hosted DSH spawn refused before any process existed",
                "attempt": attempt_json(&row),
            })
            .to_string(),
            content_type: "application/json",
        },
        Err(err) => store_error(err),
    }
}

fn attempt_response(
    row: &HostedAttemptRow,
    fact: &HostedArtifactFact,
    runtime_binding_ref: &str,
) -> ResourceApiResponse {
    redacted_ok(json!({
        "projection": HOSTED_ATTEMPT_PROJECTION_ID,
        "attempt": attempt_json(row),
        "artifact": fact_json(fact),
        "runtime_binding_ref": runtime_binding_ref,
        "engine": HOSTED_DSH_ENGINE_ID,
        "provider_proxy": HOSTED_DSH_PROVIDER_PROXY,
        "secret_bearer": "daemon-proxy-only",
        "installed_agent": false,
        "pi_member_engine": false,
        "receipt_is_not_completion": true,
        "windows_sandbox_e2e": "not-run",
    }))
}

fn attempt_json(row: &HostedAttemptRow) -> Value {
    json!({
        "attempt_id": row.attempt_id,
        "project_id": row.project_id,
        "employee_id": row.employee_id,
        "employee_revision_id": row.employee_revision_id,
        "task_ref": row.task_ref,
        "child_id": row.child_id,
        "artifact_digest": row.artifact_digest,
        "artifact_fact_id": row.artifact_fact_id,
        "context_digest": row.context_digest,
        "context_bytes": row.context_bytes,
        "intent_persisted": row.intent_persisted,
        "state": row.state,
        "pid": row.pid,
        "terminal_kind": row.terminal_kind,
        "exit_code": row.exit_code,
        "response_status": row.response_status,
        "completion_claimed": row.completion_claimed,
        "verification_status": row.verification_status,
        "candidate_count": row.candidate_count,
        "observation_count": row.observation_count,
        "rejected_frame_count": row.rejected_frame_count,
        "unknown_line_count": row.unknown_line_count,
        "stdout_bytes": row.stdout_bytes,
        "stdout_truncated": row.stdout_truncated,
        "stderr_tail_redacted": row.stderr_tail_redacted,
        "elapsed_ms": row.elapsed_ms,
        "created_at": row.created_at,
        "dispatched_at": row.dispatched_at,
        "terminal_at": row.terminal_at,
    })
}

fn frame_json(frame: &HostedAttemptFrameRow) -> Value {
    json!({
        "frame_id": frame.frame_id,
        "seq": frame.seq,
        "kind": frame.kind,
        "operation": frame.operation,
        "payload_digest": frame.payload_digest,
        "reject_reason": frame.reject_reason,
        "text_redacted": frame.text_redacted,
        "authority_written": frame.authority_written,
    })
}

fn fact_json(fact: &HostedArtifactFact) -> Value {
    json!({
        "fact_id": fact.fact_id,
        "kind": fact.kind,
        "expected_revision": fact.expected_revision,
        "configured_revision": fact.configured_revision,
        "pin_file_revision": fact.pin_file_revision,
        "health": fact.health,
        "child_script_digest": fact.child_script_digest,
        "previous_fact_id": fact.previous_fact_id,
        "detail": fact.detail_redacted,
        "created_at": fact.created_at,
    })
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

/// Serialize, then refuse the response if any secret shape survived redaction.
fn redacted_ok(body: Value) -> ResourceApiResponse {
    let serialized = body.to_string();
    let lowered = serialized.to_ascii_lowercase();
    if lowered.contains("sk-live")
        || lowered.contains("\"sess-")
        || lowered.contains("boot-")
        || lowered.contains("ssv1:")
        || lowered.contains("secretref:")
    {
        return error(
            500,
            "HOSTED_ATTEMPT_REDACTION",
            "attempt projection redaction failed",
        );
    }
    ResourceApiResponse {
        status: 200,
        body: serialized,
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
        ProjectAggregateError::Forbidden { detail } => {
            error(403, "HOSTED_ATTEMPT_FORBIDDEN", detail)
        }
        ProjectAggregateError::NotFound { detail } => {
            error(404, "HOSTED_ATTEMPT_NOT_FOUND", detail)
        }
        ProjectAggregateError::Conflict { detail } => error(409, "HOSTED_ATTEMPT_CONFLICT", detail),
        ProjectAggregateError::Stale { detail } => error(409, "HOSTED_ATTEMPT_STALE", detail),
        ProjectAggregateError::Unconfirmed { detail }
        | ProjectAggregateError::Rejected { detail } => {
            error(422, "HOSTED_ATTEMPT_REJECTED", detail)
        }
        ProjectAggregateError::Invalid { detail } => error(422, "HOSTED_ATTEMPT_INVALID", detail),
        ProjectAggregateError::Unavailable { .. } => {
            error(503, "HOSTED_ATTEMPT_UNAVAILABLE", "store unavailable")
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use cognitive_runtime::{HOSTED_DSH_CONFIG_FILE_NAME, HOSTED_DSH_REVISION_FILE_NAME};
    use cognitive_store::{
        ProjectAggregateStore, RosterProposal, StageSpec, prepare_personal_databases,
    };
    use std::fs;
    use tempfile::TempDir;

    struct Harness {
        _tmp: TempDir,
        store: SqliteAuthorityStore,
        host: HostedAttemptHost,
        dsh_root: PathBuf,
        adapter_root: PathBuf,
        project_id: String,
        employee_id: String,
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

    /// Fake exact-artifact child: reads the request frame and emits frames.
    const FAKE_CHILD: &str = r#"
let data = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => { data += chunk; });
process.stdin.on("end", () => {
  const request = JSON.parse(data.trim().split("\n")[0]);
  const emit = (frame) => process.stdout.write(JSON.stringify(frame) + "\n");
  emit({ frame: "observation", text: "child.started digest:" + request.context_digest + " origin:" + request.daemon_origin });
  emit({ frame: "heartbeat" });
  emit({ frame: "provider_request", url: "https://api.deepseek.com/v1" });
  emit({ frame: "task_complete" });
  emit({ frame: "candidate", operation: "DeliverableDraft", payload: { text: "Summary: " + request.context } });
  process.stdout.write("success\n");
  emit({ frame: "observation", text: "Authorization: Bearer sess-not-real-token" });
  const mode = (request.context.match(/mode=(\w+)/) || [])[1] || "done";
  if (mode === "hang") { setTimeout(() => {}, 60000); return; }
  if (mode === "crash") { process.exit(9); }
  emit({ frame: "response", status: "done" });
  process.exit(0);
});
"#;

    fn harness() -> Harness {
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
        let dsh_root = root.join("dsh");
        let adapter_root = root.join("adapter");
        fs::create_dir_all(&dsh_root).expect("dsh root");
        fs::create_dir_all(adapter_root.join("scripts")).expect("adapter");
        fs::write(
            dsh_root.join(HOSTED_DSH_REVISION_FILE_NAME),
            format!("{HOSTED_DSH_ARTIFACT_DIGEST}\n"),
        )
        .expect("pin");
        fs::write(
            adapter_root.join("scripts/hosted-attempt-child.mjs"),
            FAKE_CHILD,
        )
        .expect("child");
        fs::create_dir_all(layout.config_dir()).expect("config dir");
        fs::write(
            layout.config_dir().join(HOSTED_DSH_CONFIG_FILE_NAME),
            json!({
                "schema_version": 1,
                "surface": "personal-dsh-config",
                "dsh_root": dsh_root.display().to_string(),
                "adapter_root": adapter_root.display().to_string(),
                "revision": HOSTED_DSH_ARTIFACT_DIGEST,
                "adapter_id": "deepseek.dsh.akp",
                "candidate_only": true,
            })
            .to_string(),
        )
        .expect("config");
        fs::create_dir_all(layout.state_dir()).expect("state dir");
        fs::write(
            layout.state_dir().join(ENDPOINT_FILE_NAME),
            json!({"schema_version": 1, "endpoint": "127.0.0.1:48181", "surface": "personal-daemon-endpoint"}).to_string(),
        )
        .expect("endpoint");
        let bootstrap = layout.local_bootstrap_secret_path();
        fs::create_dir_all(bootstrap.parent().expect("parent")).expect("runtime dir");
        fs::write(&bootstrap, "boot-test-not-real\n").expect("bootstrap");
        let host = HostedAttemptHost::from_layout(&layout);

        let projects = ProjectAggregateStore::from_authority_store(&store);
        let employees = EmployeeStore::from_authority_store(&store);
        let (draft_id, _) = projects.create_draft(b"charter-v1", 10).expect("draft");
        projects
            .put_draft_charter(&draft_id, b"charter-body-v1", 11)
            .expect("charter");
        let (preview_id, preview_digest) = projects
            .request_preview("activation", &draft_id, b"activation-preview", 12)
            .expect("preview");
        let project_id = projects
            .confirm_preview(
                ConfirmCaller::OwnerManagement,
                &preview_id,
                &preview_digest,
                13,
            )
            .expect("G1")
            .new_ref;
        let plan_id = projects
            .apply_plan_revision(
                &project_id,
                &project_id,
                &[
                    stage("s1", "Manage", "manager"),
                    stage("s2", "Research", "researcher"),
                ],
                20,
            )
            .expect("plan");
        let ids = employees
            .register_roster(
                ConfirmCaller::OwnerManagement,
                &project_id,
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
                        prompt: "research".to_owned(),
                        tools_declared: vec!["workspace-write".to_owned()],
                    },
                ],
                21,
            )
            .expect("roster");
        employees
            .request_seating(ConfirmCaller::OwnerManagement, &ids[0], 30)
            .expect("seating");
        employees
            .confirm_seating(
                ConfirmCaller::OwnerManagement,
                &ids[0],
                Some("flash"),
                true,
                31,
            )
            .expect("seat");
        Harness {
            _tmp: temporary,
            store,
            host,
            dsh_root,
            adapter_root,
            project_id,
            employee_id: ids[0].clone(),
        }
    }

    fn run_body(harness: &Harness, context: &str) -> Vec<u8> {
        json!({
            "employee_id": harness.employee_id,
            "task_ref": "task://personal/p13-t02-http",
            "bounded_context": context,
            "timeout_ms": 20000,
            "wait": true,
        })
        .to_string()
        .into_bytes()
    }

    #[test]
    fn hosted_attempt_task_channel_is_forbidden_and_artifact_facts_are_readable() {
        let harness = harness();
        for route in [
            "POST /task/project/v1/dsh.hosted.attempt.run",
            "GET /task/project/v1/dsh.hosted.attempt.list?project_id=x",
            "POST /task/project/v1/dsh.hosted.artifact.check",
        ] {
            let forbidden = handle(route, b"{}", &harness.store, &harness.host);
            assert_eq!(forbidden.status, 403, "{route}");
            assert!(forbidden.body.contains("HOSTED_ATTEMPT_CHANNEL_FORBIDDEN"));
        }
        let empty = handle(
            "GET /management/project/v1/dsh.hosted.artifact.facts",
            b"",
            &harness.store,
            &harness.host,
        );
        assert_eq!(empty.status, 200, "{}", empty.body);
        assert!(empty.body.contains("\"latest\":null"));
        assert!(empty.body.contains("\"admits_spawn\":false"));
        let checked = handle(
            "POST /management/project/v1/dsh.hosted.artifact.check",
            b"",
            &harness.store,
            &harness.host,
        );
        assert_eq!(checked.status, 200, "{}", checked.body);
        let checked_json: Value = serde_json::from_str(&checked.body).unwrap();
        assert_eq!(checked_json["fact"]["health"], "pinned");
        assert_eq!(checked_json["fact"]["kind"], "health-check");
        assert_eq!(checked_json["admits_spawn"], true);
        assert_eq!(
            checked_json["fact"]["child_script_digest"]
                .as_str()
                .map(str::len),
            Some(64)
        );

        // Drift the pin file → update fact with mismatch; restore → rollback.
        fs::write(
            harness.dsh_root.join(HOSTED_DSH_REVISION_FILE_NAME),
            "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef\n",
        )
        .expect("drift");
        let drifted = handle(
            "POST /management/project/v1/dsh.hosted.artifact.check",
            b"",
            &harness.store,
            &harness.host,
        );
        let drifted_json: Value = serde_json::from_str(&drifted.body).unwrap();
        assert_eq!(drifted_json["fact"]["health"], "mismatch");
        assert_eq!(drifted_json["admits_spawn"], false);
        let refused = handle(
            "POST /management/project/v1/dsh.hosted.attempt.run",
            &run_body(&harness, "mode=done summarize"),
            &harness.store,
            &harness.host,
        );
        assert_eq!(refused.status, 422, "{}", refused.body);
        assert!(refused.body.contains("HOSTED_ARTIFACT_UNHEALTHY"));
        fs::write(
            harness.dsh_root.join(HOSTED_DSH_REVISION_FILE_NAME),
            format!("{HOSTED_DSH_ARTIFACT_DIGEST}\n"),
        )
        .expect("restore");
        let facts = handle(
            "GET /management/project/v1/dsh.hosted.artifact.facts?limit=10",
            b"",
            &harness.store,
            &harness.host,
        );
        let facts_json: Value = serde_json::from_str(&facts.body).unwrap();
        let history = facts_json["history"].as_array().unwrap();
        assert!(history.len() >= 2);
        assert_eq!(history[0]["health"], "mismatch");
        let list = handle(
            &format!(
                "GET /management/project/v1/dsh.hosted.attempt.list?project_id={}",
                harness.project_id
            ),
            b"",
            &harness.store,
            &harness.host,
        );
        assert_eq!(list.status, 200, "{}", list.body);
        assert!(list.body.contains("\"attempts\":[]"));
    }

    #[test]
    fn hosted_attempt_run_spawns_real_child_and_writes_daemon_terminal_observation() {
        let harness = harness();
        if HostedDshPlane::isolated_spawn_is_fenced() {
            let fenced = handle(
                "POST /management/project/v1/dsh.hosted.attempt.run",
                &run_body(&harness, "mode=done summarize README"),
                &harness.store,
                &harness.host,
            );
            assert_eq!(fenced.status, 422, "{}", fenced.body);
            assert!(fenced.body.contains("DEV-WIN-GNU-01"));
            return;
        }
        let missing_body = handle(
            "POST /management/project/v1/dsh.hosted.attempt.run",
            br#"{"employee_id":"x"}"#,
            &harness.store,
            &harness.host,
        );
        assert_eq!(missing_body.status, 400);
        let secret = handle(
            "POST /management/project/v1/dsh.hosted.attempt.run",
            &run_body(&harness, "call with sk-live-not-real"),
            &harness.store,
            &harness.host,
        );
        assert_eq!(secret.status, 422, "{}", secret.body);
        assert!(!secret.body.contains("sk-live-not-real"));

        let ran = handle(
            "POST /management/project/v1/dsh.hosted.attempt.run",
            &run_body(&harness, "mode=done summarize README"),
            &harness.store,
            &harness.host,
        );
        assert_eq!(ran.status, 200, "{}", ran.body);
        let ran_json: Value = serde_json::from_str(&ran.body).unwrap();
        let attempt = &ran_json["attempt"];
        assert_eq!(attempt["state"], "terminal");
        assert_eq!(attempt["terminal_kind"], "exited");
        assert_eq!(attempt["exit_code"], 0);
        assert_eq!(attempt["response_status"], "done");
        assert_eq!(attempt["completion_claimed"], false);
        assert_eq!(attempt["verification_status"], "not-run");
        assert_eq!(attempt["intent_persisted"], true);
        assert_eq!(attempt["candidate_count"], 1);
        assert_eq!(attempt["rejected_frame_count"], 2);
        assert_eq!(attempt["unknown_line_count"], 1);
        assert!(attempt["dispatched_at"].is_number());
        assert!(attempt["terminal_at"].is_number());
        assert!(
            attempt["child_id"]
                .as_str()
                .unwrap()
                .starts_with("dshchild-")
        );
        assert_eq!(ran_json["receipt_is_not_completion"], true);
        assert_eq!(ran_json["artifact"]["health"], "pinned");
        assert!(!ran.body.contains("sess-not-real-token"));
        assert!(!ran.body.contains("boot-test-not-real"));
        let attempt_id = attempt["attempt_id"].as_str().unwrap().to_owned();

        let detail = handle(
            &format!(
                "GET /management/project/v1/dsh.hosted.attempt.detail?attempt_id={attempt_id}"
            ),
            b"",
            &harness.store,
            &harness.host,
        );
        assert_eq!(detail.status, 200, "{}", detail.body);
        let detail_json: Value = serde_json::from_str(&detail.body).unwrap();
        let frames = detail_json["frames"].as_array().unwrap();
        assert_eq!(frames.len(), 7, "{}", detail.body);
        assert_eq!(attempt["observation_count"], 3);
        assert!(
            frames
                .iter()
                .all(|frame| frame["authority_written"] == false)
        );
        assert!(
            frames
                .iter()
                .any(|frame| frame["reject_reason"] == "child-direct-provider")
        );
        assert!(
            frames
                .iter()
                .any(|frame| frame["reject_reason"] == "child-cannot-emit-authority-frame")
        );
        let started = frames[0]["text_redacted"].as_str().unwrap();
        assert!(started.contains(&format!(
            "digest:{}",
            HostedDshAttemptStore::context_digest("mode=done summarize README")
        )));
        assert!(started.contains("origin:http://127.0.0.1:48181"));
        assert!(!detail.body.contains("sess-not-real-token"));
        assert!(detail.body.contains("Bearer [redacted]"));

        let employees = EmployeeStore::from_authority_store(&harness.store);
        let employee = employees
            .get_employee(&harness.employee_id)
            .expect("get")
            .expect("row");
        assert_eq!(employee.state, "seated");
        assert!(
            employee
                .runtime_binding_ref
                .as_deref()
                .unwrap()
                .starts_with("hosted-dsh:")
        );
        let child = HostedDshPlane::from_authority_store(&harness.store)
            .latest_child(&harness.employee_id)
            .expect("child")
            .expect("row");
        assert_eq!(child.state, "exited");
        assert!(child.pid.is_none());

        let crashed = handle(
            "POST /management/project/v1/dsh.hosted.attempt.run",
            &run_body(&harness, "mode=crash summarize README"),
            &harness.store,
            &harness.host,
        );
        assert_eq!(crashed.status, 200, "{}", crashed.body);
        let crashed_json: Value = serde_json::from_str(&crashed.body).unwrap();
        assert_eq!(crashed_json["attempt"]["terminal_kind"], "exited");
        assert_eq!(crashed_json["attempt"]["exit_code"], 9);
        assert_eq!(crashed_json["attempt"]["response_status"], "unknown");
        assert_eq!(crashed_json["attempt"]["completion_claimed"], false);

        let hung = handle(
            "POST /management/project/v1/dsh.hosted.attempt.run",
            &json!({
                "employee_id": harness.employee_id,
                "task_ref": "task://personal/p13-t02-http",
                "bounded_context": "mode=hang summarize README",
                "timeout_ms": 800,
                "wait": true,
            })
            .to_string()
            .into_bytes(),
            &harness.store,
            &harness.host,
        );
        assert_eq!(hung.status, 200, "{}", hung.body);
        let hung_json: Value = serde_json::from_str(&hung.body).unwrap();
        assert_eq!(hung_json["attempt"]["terminal_kind"], "timed-out");
        assert_eq!(hung_json["attempt"]["completion_claimed"], false);

        let list = handle(
            &format!(
                "GET /management/project/v1/dsh.hosted.attempt.list?project_id={}&limit=10",
                harness.project_id
            ),
            b"",
            &harness.store,
            &harness.host,
        );
        let list_json: Value = serde_json::from_str(&list.body).unwrap();
        let attempts = list_json["attempts"].as_array().unwrap();
        assert_eq!(attempts.len(), 3);
        assert!(
            attempts
                .iter()
                .all(|row| row["completion_claimed"] == false)
        );
        assert!(attempts.iter().all(|row| row["terminal_kind"] != "success"));

        // Missing child script after config → spawn refused, durable terminal.
        fs::remove_file(
            harness
                .adapter_root
                .join("scripts/hosted-attempt-child.mjs"),
        )
        .expect("remove");
        let refused = handle(
            "POST /management/project/v1/dsh.hosted.attempt.run",
            &run_body(&harness, "mode=done summarize README"),
            &harness.store,
            &harness.host,
        );
        assert_eq!(refused.status, 422, "{}", refused.body);
        assert!(refused.body.contains("HOSTED_ARTIFACT_UNHEALTHY"));
    }
}
