//! Personal-private Attempt artifact / verifier / acceptance / publication
//! routes (P13-T04).
//!
//! The vertical chain: a hosted Attempt's daemon-observed terminal run
//! (`hosted_dsh_attempt::run_attempt_child`) hands its `DeliverableDraft`
//! candidates to [`ingest_terminal_run`], which puts the observed bytes into
//! the daemon's single P3-T03 CAS and immediately runs the independent
//! verifier (`verifier://personal/attempt-artifact`). The `outputs` page reads
//! `outputs` / `outputs.detail`, opens bytes through `outputs.open` (CAS
//! re-read, digest-checked) and may export a copy into Personal Home `data/`
//! through `outputs.export` — the copy is never authority. StageTestPassed is
//! derived through `attempt.artifact.stage-test`; last-ring acceptance and
//! external send are ApprovalPreviews confirmed by the existing P11-T09
//! `confirm` route. Management-channel only; task-channel aliases are 403.
//! Host file-open E2E stays `not-run` until `P13-T13`.

use std::path::{Path, PathBuf};

use cognitive_runtime::{HostedChildRun, HostedFrameKind};
use cognitive_store::{
    ATTEMPT_ARTIFACT_FORMAT_MARKDOWN, ATTEMPT_ARTIFACT_PROJECTION_ID,
    ATTEMPT_ARTIFACT_VERIFIER_PRINCIPAL, ATTEMPT_ARTIFACT_VERIFIER_REF, ArtifactEvidenceRow,
    ArtifactIngestSpec, ArtifactStore, AttemptArtifactRow, AttemptArtifactStore, ConfirmCaller,
    EXTERNAL_SEND_SUBJECT_KIND, EmployeeStore, ExternalSendRow, ExternalSendSpec,
    PersonalDataLayout, ProjectAggregateError, ProjectAggregateStore, RUN_ACCEPTANCE_SUBJECT_KIND,
    RunAcceptanceRow, SqliteAuthorityStore,
};
use serde_json::{Value, json};

use super::resource_api::ResourceApiResponse;

const ROUTE_LITERALS: &[&str] = &[
    "GET /management/project/v1/outputs",
    "GET /management/project/v1/outputs.detail",
    "GET /management/project/v1/outputs.open",
    "POST /management/project/v1/outputs.export",
    "POST /management/project/v1/attempt.artifact.verify",
    "POST /management/project/v1/attempt.artifact.stage-test",
    "POST /management/project/v1/run.acceptance.request",
    "GET /management/project/v1/run.acceptance",
    "GET /management/project/v1/publication.packet",
    "POST /management/project/v1/publication.external-send.request",
    "GET /management/project/v1/publication.sends",
    "GET /task/project/v1/outputs",
    "GET /task/project/v1/outputs.detail",
    "GET /task/project/v1/outputs.open",
    "POST /task/project/v1/outputs.export",
    "POST /task/project/v1/attempt.artifact.verify",
    "POST /task/project/v1/attempt.artifact.stage-test",
    "POST /task/project/v1/run.acceptance.request",
    "GET /task/project/v1/run.acceptance",
    "GET /task/project/v1/publication.packet",
    "POST /task/project/v1/publication.external-send.request",
    "GET /task/project/v1/publication.sends",
];

/// The daemon's single CAS root. Shared with the verification executor and the
/// native Tool executor; never a second store.
const DAEMON_ARTIFACT_MAXIMUM_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Channel {
    Management,
    Task,
}

/// Daemon-owned filesystem facts for this route family. Paths only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArtifactHost {
    pub artifact_root: PathBuf,
    pub data_dir: PathBuf,
}

impl ArtifactHost {
    pub(crate) fn from_layout(layout: &PersonalDataLayout) -> Self {
        Self {
            artifact_root: artifact_root(layout),
            data_dir: layout.data_dir().to_path_buf(),
        }
    }

    fn cas(&self) -> Result<ArtifactStore, ResourceApiResponse> {
        ArtifactStore::open(&self.artifact_root, DAEMON_ARTIFACT_MAXIMUM_BYTES).map_err(|_| {
            error(
                503,
                "ATTEMPT_ARTIFACT_UNAVAILABLE",
                "daemon artifact store unavailable",
            )
        })
    }

    /// Personal Home `data/` location for exported copies of one Project's outputs.
    fn exports_dir(&self, project_id: &str) -> PathBuf {
        self.data_dir
            .join("projects")
            .join(sanitize_segment(project_id))
            .join("outputs")
    }
}

/// `<data_dir>/artifacts` — the same root `verification_executor` composes.
pub(crate) fn artifact_root(layout: &PersonalDataLayout) -> PathBuf {
    layout.data_dir().join("artifacts")
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
        "ATTEMPT_ARTIFACT_CHANNEL_FORBIDDEN",
        "Attempt artifact / verification / acceptance operations are management-channel only",
    )
}

pub(crate) fn handle(
    method_path: &str,
    body: &[u8],
    store: &SqliteAuthorityStore,
    host: &ArtifactHost,
) -> ResourceApiResponse {
    let Some((channel, literal)) = parse_route(method_path) else {
        return error(
            404,
            "ATTEMPT_ARTIFACT_ROUTE_NOT_FOUND",
            "no Attempt artifact route matched",
        );
    };
    if channel == Channel::Task {
        return channel_forbidden();
    }
    let cas = match host.cas() {
        Ok(cas) => cas,
        Err(response) => return response,
    };
    let artifacts = AttemptArtifactStore::from_authority_store(store);
    let projects = ProjectAggregateStore::from_authority_store(store);
    let employees = EmployeeStore::from_authority_store(store);
    match literal {
        "GET /management/project/v1/outputs" => outputs_list(method_path, &artifacts),
        "GET /management/project/v1/outputs.detail" => {
            outputs_detail(method_path, &artifacts, host)
        }
        "GET /management/project/v1/outputs.open" => outputs_open(method_path, &artifacts, &cas),
        "POST /management/project/v1/outputs.export" => {
            outputs_export(body, &artifacts, &cas, host)
        }
        "POST /management/project/v1/attempt.artifact.verify" => {
            artifact_verify(body, &artifacts, &cas)
        }
        "POST /management/project/v1/attempt.artifact.stage-test" => {
            artifact_stage_test(body, &artifacts, &projects, &employees, &cas)
        }
        "POST /management/project/v1/run.acceptance.request" => {
            run_acceptance_request(body, &artifacts, &projects)
        }
        "GET /management/project/v1/run.acceptance" => run_acceptance_list(method_path, &artifacts),
        "GET /management/project/v1/publication.packet" => {
            publication_packet(method_path, &artifacts, &projects)
        }
        "POST /management/project/v1/publication.external-send.request" => {
            external_send_request(body, &artifacts, &projects)
        }
        "GET /management/project/v1/publication.sends" => {
            publication_sends(method_path, &artifacts)
        }
        _ => error(
            404,
            "ATTEMPT_ARTIFACT_ROUTE_NOT_FOUND",
            "no Attempt artifact route matched",
        ),
    }
}

/// Exact-literal match: the route literal must be followed by end of input,
/// a query string, or the HTTP-version separator, so `outputs` never swallows
/// `outputs.detail`.
fn parse_route(method_path: &str) -> Option<(Channel, &'static str)> {
    for literal in ROUTE_LITERALS {
        if let Some(rest) = method_path.strip_prefix(literal)
            && (rest.is_empty() || rest.starts_with('?') || rest.starts_with(' '))
        {
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

// ----------------------------------------------------------------------
// Daemon-side ingest after the terminal observation
// ----------------------------------------------------------------------

/// Put every `DeliverableDraft` candidate of a terminal run into the CAS and
/// run the independent verifier over each new artifact. Failures are logged
/// and never touch the terminal observation already written: an artifact that
/// cannot be ingested simply does not exist, and `outputs` stays honest.
/// Returns the ingested artifact ids.
pub(crate) fn ingest_terminal_run(
    store: &SqliteAuthorityStore,
    artifact_root: &Path,
    attempt_id: &str,
    run: &HostedChildRun,
) -> Vec<String> {
    let cas = match ArtifactStore::open(artifact_root, DAEMON_ARTIFACT_MAXIMUM_BYTES) {
        Ok(cas) => cas,
        Err(error) => {
            eprintln!("kernel-server personal: attempt {attempt_id}: CAS unavailable: {error}");
            return Vec::new();
        }
    };
    let artifacts = AttemptArtifactStore::from_authority_store(store);
    let mut ingested = Vec::new();
    for frame in run.frames.iter().filter(|frame| {
        frame.kind == HostedFrameKind::Candidate
            && frame.operation.as_deref() == Some("DeliverableDraft")
    }) {
        let Some(payload) = frame.payload_canonical.as_deref() else {
            eprintln!(
                "kernel-server personal: attempt {attempt_id}: candidate frame {} exceeded the retained payload ceiling; not ingested",
                frame.seq
            );
            continue;
        };
        let now = now_ms();
        match artifacts.ingest_candidate(
            &cas,
            &ArtifactIngestSpec {
                attempt_id,
                source_frame_seq: i64::try_from(frame.seq).unwrap_or(i64::MAX),
                payload_canonical: payload,
                now_ms: now,
            },
        ) {
            Ok(artifact) => {
                if let Err(error) = artifacts.verify_artifact(&cas, &artifact.artifact_id, now_ms())
                {
                    eprintln!(
                        "kernel-server personal: attempt {attempt_id}: verifier did not run for {}: {error}",
                        artifact.artifact_id
                    );
                }
                ingested.push(artifact.artifact_id);
            }
            Err(error) => {
                eprintln!(
                    "kernel-server personal: attempt {attempt_id}: candidate frame {} not ingested: {error}",
                    frame.seq
                );
            }
        }
    }
    ingested
}

// ----------------------------------------------------------------------
// outputs reads
// ----------------------------------------------------------------------

fn outputs_list(method_path: &str, artifacts: &AttemptArtifactStore) -> ResourceApiResponse {
    let Some(project_id) = query_parameter(method_path, "project_id").filter(|v| !v.is_empty())
    else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    let limit = query_parameter(method_path, "limit")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(32);
    match (
        artifacts.list_artifacts(&project_id, limit),
        artifacts.list_run_acceptances(&project_id),
    ) {
        (Ok(rows), Ok(acceptances)) => redacted_ok(json!({
            "projection": ATTEMPT_ARTIFACT_PROJECTION_ID,
            "project_id": project_id,
            "artifacts": rows.iter().map(artifact_json).collect::<Vec<_>>(),
            "run_acceptances": acceptances.iter().map(acceptance_json).collect::<Vec<_>>(),
            "verifier_ref": ATTEMPT_ARTIFACT_VERIFIER_REF,
            "files_are_authority": false,
            "chat_can_confirm": false,
            "host_file_open_e2e": "not-run",
        })),
        (Err(err), _) | (_, Err(err)) => store_error(err),
    }
}

fn outputs_detail(
    method_path: &str,
    artifacts: &AttemptArtifactStore,
    host: &ArtifactHost,
) -> ResourceApiResponse {
    let Some(artifact_id) = query_parameter(method_path, "artifact_id").filter(|v| !v.is_empty())
    else {
        return error(400, "ARTIFACT_ID_REQUIRED", "artifact_id required");
    };
    let artifact = match artifacts.get_artifact(&artifact_id) {
        Ok(Some(row)) => row,
        Ok(None) => return error(404, "ATTEMPT_ARTIFACT_NOT_FOUND", "artifact not found"),
        Err(err) => return store_error(err),
    };
    let evidence = match artifacts.list_evidence(&artifact_id) {
        Ok(rows) => rows,
        Err(err) => return store_error(err),
    };
    let acceptance = match artifacts.list_run_acceptances(&artifact.project_id) {
        Ok(rows) => rows.into_iter().find(|row| row.artifact_id == artifact_id),
        Err(err) => return store_error(err),
    };
    let export_path = export_path(host, &artifact);
    redacted_ok(json!({
        "projection": ATTEMPT_ARTIFACT_PROJECTION_ID,
        "artifact": artifact_json(&artifact),
        "evidence": evidence.iter().map(evidence_json).collect::<Vec<_>>(),
        "run_acceptance": acceptance.as_ref().map(acceptance_json),
        "open_route": format!("/management/project/v1/outputs.open?artifact_id={artifact_id}"),
        "export": {
            "exists": export_path.exists(),
            "path": export_path.display().to_string(),
            "is_authority": false,
        },
        "files_are_authority": false,
        "chat_can_confirm": false,
        "host_file_open_e2e": "not-run",
    }))
}

/// Serve the deliverable bytes from the CAS. The bytes are re-hashed on read
/// (`ArtifactStore::get`), so a tampered file is a 409, never a download.
fn outputs_open(
    method_path: &str,
    artifacts: &AttemptArtifactStore,
    cas: &ArtifactStore,
) -> ResourceApiResponse {
    let Some(artifact_id) = query_parameter(method_path, "artifact_id").filter(|v| !v.is_empty())
    else {
        return error(400, "ARTIFACT_ID_REQUIRED", "artifact_id required");
    };
    let artifact = match artifacts.get_artifact(&artifact_id) {
        Ok(Some(row)) => row,
        Ok(None) => return error(404, "ATTEMPT_ARTIFACT_NOT_FOUND", "artifact not found"),
        Err(err) => return store_error(err),
    };
    match read_deliverable(artifacts, cas, &artifact) {
        Ok(text) => ResourceApiResponse {
            status: 200,
            body: text,
            content_type: if artifact.format == ATTEMPT_ARTIFACT_FORMAT_MARKDOWN {
                "text/markdown; charset=utf-8"
            } else {
                "text/plain; charset=utf-8"
            },
        },
        Err(response) => response,
    }
}

fn read_deliverable(
    artifacts: &AttemptArtifactStore,
    cas: &ArtifactStore,
    artifact: &AttemptArtifactRow,
) -> Result<String, ResourceApiResponse> {
    let reference = artifacts
        .resolve_openable_ref(&artifact.cas_ref)
        .map_err(store_error)?;
    match cas.get(&reference) {
        Ok(Some(bytes)) => String::from_utf8(bytes).map_err(|_| {
            error(
                409,
                "ATTEMPT_ARTIFACT_FORMAT_INVALID",
                "deliverable bytes are not valid UTF-8",
            )
        }),
        Ok(None) => Err(error(
            404,
            "ATTEMPT_ARTIFACT_BYTES_MISSING",
            "deliverable bytes are missing from the CAS",
        )),
        Err(cognitive_store::ArtifactStoreError::DigestMismatch { .. }) => Err(error(
            409,
            "ATTEMPT_ARTIFACT_DIGEST_MISMATCH",
            "deliverable bytes on disk no longer hash to the artifact digest",
        )),
        Err(_) => Err(error(
            503,
            "ATTEMPT_ARTIFACT_UNAVAILABLE",
            "daemon artifact store unavailable",
        )),
    }
}

fn export_path(host: &ArtifactHost, artifact: &AttemptArtifactRow) -> PathBuf {
    let extension = if artifact.format == ATTEMPT_ARTIFACT_FORMAT_MARKDOWN {
        "md"
    } else {
        "txt"
    };
    host.exports_dir(&artifact.project_id).join(format!(
        "{}.{extension}",
        sanitize_segment(&artifact.artifact_id)
    ))
}

/// Write a copy of the CAS bytes into Personal Home `data/`. The copy is for
/// the owner to open with a host application; it is never read back as
/// authority and the response says so.
fn outputs_export(
    body: &[u8],
    artifacts: &AttemptArtifactStore,
    cas: &ArtifactStore,
    host: &ArtifactHost,
) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "ATTEMPT_ARTIFACT_JSON_REQUIRED", "JSON body required");
    };
    let Some(artifact_id) = document.get("artifact_id").and_then(Value::as_str) else {
        return error(400, "ARTIFACT_ID_REQUIRED", "artifact_id required");
    };
    let artifact = match artifacts.get_artifact(artifact_id) {
        Ok(Some(row)) => row,
        Ok(None) => return error(404, "ATTEMPT_ARTIFACT_NOT_FOUND", "artifact not found"),
        Err(err) => return store_error(err),
    };
    let text = match read_deliverable(artifacts, cas, &artifact) {
        Ok(text) => text,
        Err(response) => return response,
    };
    let target = export_path(host, &artifact);
    let Some(parent) = target.parent() else {
        return error(
            503,
            "ATTEMPT_ARTIFACT_EXPORT_FAILED",
            "export path has no parent",
        );
    };
    if let Err(err) = std::fs::create_dir_all(parent) {
        return error(
            503,
            "ATTEMPT_ARTIFACT_EXPORT_FAILED",
            &format!("create export directory: {}", err.kind()),
        );
    }
    let staging = parent.join(format!(
        ".{}.partial",
        sanitize_segment(&artifact.artifact_id)
    ));
    if let Err(err) = std::fs::write(&staging, text.as_bytes()) {
        return error(
            503,
            "ATTEMPT_ARTIFACT_EXPORT_FAILED",
            &format!("write export copy: {}", err.kind()),
        );
    }
    if let Err(err) = std::fs::rename(&staging, &target) {
        let _ = std::fs::remove_file(&staging);
        return error(
            503,
            "ATTEMPT_ARTIFACT_EXPORT_FAILED",
            &format!("publish export copy: {}", err.kind()),
        );
    }
    redacted_ok(json!({
        "projection": ATTEMPT_ARTIFACT_PROJECTION_ID,
        "artifact_id": artifact.artifact_id,
        "cas_ref": artifact.cas_ref,
        "export": {
            "path": target.display().to_string(),
            "byte_length": text.len(),
            "is_authority": false,
            "location": "personal-home-data",
        },
        "host_file_open_e2e": "not-run",
    }))
}

// ----------------------------------------------------------------------
// verifier / stage test / acceptance / publication
// ----------------------------------------------------------------------

fn artifact_verify(
    body: &[u8],
    artifacts: &AttemptArtifactStore,
    cas: &ArtifactStore,
) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "ATTEMPT_ARTIFACT_JSON_REQUIRED", "JSON body required");
    };
    let Some(artifact_id) = document.get("artifact_id").and_then(Value::as_str) else {
        return error(400, "ARTIFACT_ID_REQUIRED", "artifact_id required");
    };
    match artifacts.verify_artifact(cas, artifact_id, now_ms()) {
        Ok(evidence) => redacted_ok(json!({
            "projection": ATTEMPT_ARTIFACT_PROJECTION_ID,
            "evidence": evidence_json(&evidence),
            "verifier_ref": ATTEMPT_ARTIFACT_VERIFIER_REF,
            "principal": ATTEMPT_ARTIFACT_VERIFIER_PRINCIPAL,
            "evidence_is_not_acceptance": true,
        })),
        Err(err) => store_error(err),
    }
}

fn artifact_stage_test(
    body: &[u8],
    artifacts: &AttemptArtifactStore,
    projects: &ProjectAggregateStore,
    employees: &EmployeeStore,
    cas: &ArtifactStore,
) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "ATTEMPT_ARTIFACT_JSON_REQUIRED", "JSON body required");
    };
    let Some(artifact_id) = document.get("artifact_id").and_then(Value::as_str) else {
        return error(400, "ARTIFACT_ID_REQUIRED", "artifact_id required");
    };
    let Some(stage_id) = document.get("stage_id").and_then(Value::as_str) else {
        return error(400, "STAGE_ID_REQUIRED", "stage_id required");
    };
    match artifacts.derive_stage_test(
        ConfirmCaller::OwnerManagement,
        projects,
        employees,
        cas,
        artifact_id,
        stage_id,
        now_ms(),
    ) {
        Ok(fact_id) => redacted_ok(json!({
            "projection": ATTEMPT_ARTIFACT_PROJECTION_ID,
            "stage_test_fact_id": fact_id,
            "artifact_id": artifact_id,
            "stage_id": stage_id,
            "derived_from": ["seating", "independent-verifier-evidence", "cas-re-read", "attempt-terminal"],
            "stage_test_is_not_acceptance": true,
        })),
        Err(err) => store_error(err),
    }
}

fn run_acceptance_request(
    body: &[u8],
    artifacts: &AttemptArtifactStore,
    projects: &ProjectAggregateStore,
) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "ATTEMPT_ARTIFACT_JSON_REQUIRED", "JSON body required");
    };
    let Some(project_id) = document.get("project_id").and_then(Value::as_str) else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    let Some(stage_id) = document.get("stage_id").and_then(Value::as_str) else {
        return error(400, "STAGE_ID_REQUIRED", "stage_id required");
    };
    match artifacts.request_run_acceptance(
        ConfirmCaller::OwnerManagement,
        projects,
        project_id,
        stage_id,
        now_ms(),
    ) {
        Ok((preview_id, preview_digest)) => ok(json!({
            "status": "ok",
            "projection": ATTEMPT_ARTIFACT_PROJECTION_ID,
            "subject_kind": RUN_ACCEPTANCE_SUBJECT_KIND,
            "subject_ref": project_id,
            "preview_id": preview_id,
            "preview_digest": preview_digest,
            "confirm_route": "/management/project/v1/confirm",
            "chat_can_confirm": false,
        })),
        Err(err) => store_error(err),
    }
}

fn run_acceptance_list(method_path: &str, artifacts: &AttemptArtifactStore) -> ResourceApiResponse {
    let Some(project_id) = query_parameter(method_path, "project_id").filter(|v| !v.is_empty())
    else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    match artifacts.list_run_acceptances(&project_id) {
        Ok(rows) => ok(json!({
            "status": "ok",
            "projection": ATTEMPT_ARTIFACT_PROJECTION_ID,
            "project_id": project_id,
            "run_acceptances": rows.iter().map(acceptance_json).collect::<Vec<_>>(),
        })),
        Err(err) => store_error(err),
    }
}

fn publication_packet(
    method_path: &str,
    artifacts: &AttemptArtifactStore,
    projects: &ProjectAggregateStore,
) -> ResourceApiResponse {
    let Some(project_id) = query_parameter(method_path, "project_id").filter(|v| !v.is_empty())
    else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    let Some(artifact_id) = query_parameter(method_path, "artifact_id").filter(|v| !v.is_empty())
    else {
        return error(400, "ARTIFACT_ID_REQUIRED", "artifact_id required");
    };
    match artifacts.publication_packet(projects, &project_id, &artifact_id, now_ms()) {
        Ok(mut packet) => {
            if let Some(object) = packet.as_object_mut() {
                object.insert("status".to_owned(), json!("ok"));
            }
            redacted_ok(packet)
        }
        Err(err) => store_error(err),
    }
}

fn external_send_request(
    body: &[u8],
    artifacts: &AttemptArtifactStore,
    projects: &ProjectAggregateStore,
) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "ATTEMPT_ARTIFACT_JSON_REQUIRED", "JSON body required");
    };
    let Some(project_id) = document.get("project_id").and_then(Value::as_str) else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    let Some(artifact_id) = document.get("artifact_id").and_then(Value::as_str) else {
        return error(400, "ARTIFACT_ID_REQUIRED", "artifact_id required");
    };
    let recipients: Vec<String> = document
        .get("recipients")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    match artifacts.request_external_send(
        ConfirmCaller::OwnerManagement,
        projects,
        &ExternalSendSpec {
            project_id,
            artifact_id,
            recipients: &recipients,
            now_ms: now_ms(),
        },
    ) {
        Ok(send) => redacted_ok(json!({
            "status": "ok",
            "projection": ATTEMPT_ARTIFACT_PROJECTION_ID,
            "subject_kind": EXTERNAL_SEND_SUBJECT_KIND,
            "subject_ref": project_id,
            "send": send_json(&send),
            "preview_id": send.preview_id,
            "preview_digest": send.preview_digest,
            "confirm_route": "/management/project/v1/confirm",
            "planned": true,
            "published": false,
            "chat_can_confirm": false,
        })),
        Err(err) => store_error(err),
    }
}

fn publication_sends(method_path: &str, artifacts: &AttemptArtifactStore) -> ResourceApiResponse {
    let Some(project_id) = query_parameter(method_path, "project_id").filter(|v| !v.is_empty())
    else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    match artifacts.list_external_sends(&project_id) {
        Ok(rows) => redacted_ok(json!({
            "status": "ok",
            "projection": ATTEMPT_ARTIFACT_PROJECTION_ID,
            "project_id": project_id,
            "sends": rows.iter().map(send_json).collect::<Vec<_>>(),
            "published_any": false,
        })),
        Err(err) => store_error(err),
    }
}

// ----------------------------------------------------------------------
// JSON shapes
// ----------------------------------------------------------------------

fn artifact_json(row: &AttemptArtifactRow) -> Value {
    json!({
        "artifact_id": row.artifact_id,
        "attempt_id": row.attempt_id,
        "project_id": row.project_id,
        "task_ref": row.task_ref,
        "employee_id": row.employee_id,
        "cas_ref": row.cas_ref,
        "byte_length": row.byte_length,
        "format": row.format,
        "source": row.source,
        "source_frame_seq": row.source_frame_seq,
        "source_payload_digest": row.source_payload_digest,
        "context_digest": row.context_digest,
        "produced_at": row.produced_at,
        "created_at": row.created_at,
        "freshness": row.freshness,
        "verification_status": row.verification_status,
        "latest_evidence_id": row.latest_evidence_id,
        "stage_id": row.stage_id,
        "accepted_at": row.accepted_at,
        "openable": true,
        "is_authority": true,
    })
}

fn evidence_json(row: &ArtifactEvidenceRow) -> Value {
    let criteria: Value = serde_json::from_str(&row.criteria_json).unwrap_or(Value::Null);
    json!({
        "evidence_id": row.evidence_id,
        "artifact_id": row.artifact_id,
        "verifier_ref": row.verifier_ref,
        "verifier_version": row.verifier_version,
        "principal": row.principal,
        "disposition": row.disposition,
        "criteria": criteria,
        "report_cas_ref": row.report_cas_ref,
        "checked_cas_ref": row.checked_cas_ref,
        "verified_at": row.verified_at,
    })
}

fn acceptance_json(row: &RunAcceptanceRow) -> Value {
    json!({
        "acceptance_id": row.acceptance_id,
        "project_id": row.project_id,
        "plan_revision_id": row.plan_revision_id,
        "stage_id": row.stage_id,
        "stage_position": row.stage_position,
        "stage_count": row.stage_count,
        "last_ring": row.stage_position == row.stage_count - 1,
        "stage_test_fact_id": row.stage_test_fact_id,
        "artifact_id": row.artifact_id,
        "evidence_id": row.evidence_id,
        "acceptance_decision_ref": row.acceptance_decision_ref,
        "accepted_at": row.accepted_at,
    })
}

fn send_json(row: &ExternalSendRow) -> Value {
    json!({
        "send_id": row.send_id,
        "project_id": row.project_id,
        "artifact_id": row.artifact_id,
        "evidence_id": row.evidence_id,
        "acceptance_id": row.acceptance_id,
        "preview_id": row.preview_id,
        "packet_digest": row.packet_digest,
        "recipient_count": row.recipient_count,
        "state": row.state,
        "published": row.published,
        "connector": row.connector,
        "intent_persisted": row.intent_persisted,
        "receipt_ref": row.receipt_ref,
        "created_at": row.created_at,
        "planned_at": row.planned_at,
    })
}

// ----------------------------------------------------------------------
// helpers
// ----------------------------------------------------------------------

fn sanitize_segment(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
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

/// Serialize, then refuse the response if any secret shape survived.
fn redacted_ok(mut body: Value) -> ResourceApiResponse {
    if let Some(object) = body.as_object_mut()
        && !object.contains_key("status")
    {
        object.insert("status".to_owned(), json!("ok"));
    }
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
            "ATTEMPT_ARTIFACT_REDACTION",
            "artifact projection redaction failed",
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
            error(403, "ATTEMPT_ARTIFACT_FORBIDDEN", detail)
        }
        ProjectAggregateError::NotFound { detail } => {
            error(404, "ATTEMPT_ARTIFACT_NOT_FOUND", detail)
        }
        ProjectAggregateError::Conflict { detail } => {
            error(409, "ATTEMPT_ARTIFACT_CONFLICT", detail)
        }
        ProjectAggregateError::Stale { detail } => error(409, "ATTEMPT_ARTIFACT_STALE", detail),
        ProjectAggregateError::Unconfirmed { detail }
        | ProjectAggregateError::Rejected { detail } => {
            error(422, "ATTEMPT_ARTIFACT_REJECTED", detail)
        }
        ProjectAggregateError::Invalid { detail } => error(422, "ATTEMPT_ARTIFACT_INVALID", detail),
        ProjectAggregateError::Unavailable { .. } => {
            error(503, "ATTEMPT_ARTIFACT_UNAVAILABLE", "store unavailable")
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::personal::hosted_dsh_attempt::{self, HostedAttemptHost};
    use cognitive_runtime::{HOSTED_DSH_CONFIG_FILE_NAME, HOSTED_DSH_REVISION_FILE_NAME};
    use cognitive_store::{
        HOSTED_DSH_ARTIFACT_DIGEST, HostedDshAttemptStore, HostedDshPlane, RosterProposal,
        StageSpec, prepare_personal_databases,
    };
    use std::fs;
    use tempfile::TempDir;

    struct Harness {
        _tmp: TempDir,
        store: SqliteAuthorityStore,
        layout: PersonalDataLayout,
        hosted: HostedAttemptHost,
        host: ArtifactHost,
        project_id: String,
        manager_id: String,
        researcher_id: String,
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

    /// Fake exact-artifact child: claims `done`, emits one DeliverableDraft
    /// whose text asserts completion (which must never be believed).
    const FAKE_CHILD: &str = r##"
let data = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => { data += chunk; });
process.stdin.on("end", () => {
  const request = JSON.parse(data.trim().split("\n")[0]);
  const emit = (frame) => process.stdout.write(JSON.stringify(frame) + "\n");
  emit({ frame: "observation", text: "child.started" });
  emit({ frame: "task_complete" });
  emit({ frame: "candidate", operation: "DeliverableDraft", payload: { text: "# Report\n\nTASK COMPLETE: " + request.context, attempt_id: request.attempt_id } });
  emit({ frame: "response", status: "done", completion_claimed: false });
  process.exit(0);
});
"##;

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
            layout.state_dir().join("daemon-endpoint.json"),
            json!({"schema_version": 1, "endpoint": "127.0.0.1:48181", "surface": "personal-daemon-endpoint"}).to_string(),
        )
        .expect("endpoint");
        let bootstrap = layout.local_bootstrap_secret_path();
        fs::create_dir_all(bootstrap.parent().expect("parent")).expect("runtime dir");
        fs::write(&bootstrap, "boot-test-not-real\n").expect("bootstrap");

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
        for stage_id in ["s1", "s2"] {
            let row = projects
                .get_stage(&plan_id, stage_id)
                .expect("stage")
                .expect("row");
            projects
                .confirm_stage(
                    ConfirmCaller::OwnerManagement,
                    &project_id,
                    &plan_id,
                    stage_id,
                    &row.stage_digest,
                )
                .expect("confirm stage");
        }
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
        for id in &ids {
            employees
                .request_seating(ConfirmCaller::OwnerManagement, id, 30)
                .expect("seating");
            employees
                .confirm_seating(ConfirmCaller::OwnerManagement, id, Some("flash"), true, 31)
                .expect("seat");
        }
        Harness {
            hosted: HostedAttemptHost::from_layout(&layout),
            host: ArtifactHost::from_layout(&layout),
            _tmp: temporary,
            store,
            layout,
            project_id,
            manager_id: ids[0].clone(),
            researcher_id: ids[1].clone(),
        }
    }

    fn run_attempt(harness: &Harness, employee_id: &str, context: &str) -> ResourceApiResponse {
        hosted_dsh_attempt::handle(
            "POST /management/project/v1/dsh.hosted.attempt.run",
            &json!({
                "employee_id": employee_id,
                "task_ref": "task://personal/p13-t04-http",
                "bounded_context": context,
                "timeout_ms": 20000,
                "wait": true,
            })
            .to_string()
            .into_bytes(),
            &harness.store,
            &harness.hosted,
        )
    }

    fn get(harness: &Harness, route: &str) -> Value {
        let response = handle(route, b"", &harness.store, &harness.host);
        assert_eq!(response.status, 200, "{route}: {}", response.body);
        serde_json::from_str(&response.body).unwrap()
    }

    fn post(harness: &Harness, route: &str, body: Value) -> ResourceApiResponse {
        handle(
            route,
            body.to_string().as_bytes(),
            &harness.store,
            &harness.host,
        )
    }

    #[test]
    fn p13_t04_task_channel_is_forbidden_and_files_are_never_artifact_refs() {
        let harness = harness();
        for route in [
            "GET /task/project/v1/outputs?project_id=x",
            "GET /task/project/v1/outputs.open?artifact_id=x",
            "POST /task/project/v1/attempt.artifact.verify",
            "POST /task/project/v1/run.acceptance.request",
            "POST /task/project/v1/publication.external-send.request",
        ] {
            let forbidden = handle(route, b"{}", &harness.store, &harness.host);
            assert_eq!(forbidden.status, 403, "{route}");
            assert!(
                forbidden
                    .body
                    .contains("ATTEMPT_ARTIFACT_CHANNEL_FORBIDDEN")
            );
        }
        assert!(!matches("GET /management/project/v1/outputsX "));
        assert!(matches("GET /management/project/v1/outputs?project_id=p "));
        assert!(matches(
            "GET /management/project/v1/outputs.detail?artifact_id=a "
        ));
        let empty = get(
            &harness,
            &format!(
                "GET /management/project/v1/outputs?project_id={}",
                harness.project_id
            ),
        );
        assert_eq!(empty["artifacts"].as_array().unwrap().len(), 0);
        assert_eq!(empty["files_are_authority"], false);
        assert_eq!(empty["chat_can_confirm"], false);
        assert_eq!(empty["host_file_open_e2e"], "not-run");
        // A path is not an artifact id and never reaches the filesystem.
        for forged in [
            "file:///etc/passwd",
            "..%2F..%2Fauthority.sqlite",
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        ] {
            let opened = handle(
                &format!("GET /management/project/v1/outputs.open?artifact_id={forged}"),
                b"",
                &harness.store,
                &harness.host,
            );
            assert_eq!(opened.status, 404, "{forged}: {}", opened.body);
        }
        let refused = post(
            &harness,
            "POST /management/project/v1/run.acceptance.request",
            json!({"project_id": harness.project_id, "stage_id": "s2"}),
        );
        assert_eq!(refused.status, 422, "{}", refused.body);
        assert!(refused.body.contains("no current StageTestPassed"));
        let accepted = get(
            &harness,
            &format!(
                "GET /management/project/v1/run.acceptance?project_id={}",
                harness.project_id
            ),
        );
        assert_eq!(accepted["run_acceptances"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn p13_t04_real_attempt_enters_cas_is_verified_and_accepts_only_on_the_last_ring() {
        let harness = harness();
        if HostedDshPlane::isolated_spawn_is_fenced() {
            let fenced = run_attempt(&harness, &harness.researcher_id, "weekly report");
            assert_eq!(fenced.status, 422, "{}", fenced.body);
            assert!(fenced.body.contains("DEV-WIN-GNU-01"));
            return;
        }

        // Ring one: the manager's Attempt produces an artifact.
        let manager_run = run_attempt(&harness, &harness.manager_id, "plan the week");
        assert_eq!(manager_run.status, 200, "{}", manager_run.body);
        // Last ring: the researcher's Attempt produces the deliverable.
        let ran = run_attempt(&harness, &harness.researcher_id, "write the weekly report");
        assert_eq!(ran.status, 200, "{}", ran.body);
        let ran_json: Value = serde_json::from_str(&ran.body).unwrap();
        let attempt = &ran_json["attempt"];
        assert_eq!(attempt["response_status"], "done");
        assert_eq!(attempt["exit_code"], 0);
        assert_eq!(attempt["completion_claimed"], false);
        assert_eq!(attempt["verification_status"], "not-run");
        assert_eq!(attempt["candidate_count"], 1);
        assert_eq!(ran_json["receipt_is_not_completion"], true);
        let attempt_id = attempt["attempt_id"].as_str().unwrap().to_owned();

        // The daemon ingested the candidate into the CAS and ran the verifier.
        let list = get(
            &harness,
            &format!(
                "GET /management/project/v1/outputs?project_id={}",
                harness.project_id
            ),
        );
        let artifacts = list["artifacts"].as_array().unwrap();
        assert_eq!(artifacts.len(), 2, "{list}");
        let artifact = artifacts
            .iter()
            .find(|row| row["attempt_id"] == attempt_id)
            .expect("researcher artifact");
        assert_eq!(
            artifact["source"],
            "hosted-dsh-child:candidate:DeliverableDraft"
        );
        assert_eq!(artifact["format"], "text/markdown");
        assert_eq!(artifact["freshness"], "current");
        assert_eq!(artifact["verification_status"], "passed");
        assert!(artifact["accepted_at"].is_null());
        assert!(artifact["cas_ref"].as_str().unwrap().starts_with("sha256:"));
        let artifact_id = artifact["artifact_id"].as_str().unwrap().to_owned();
        let manager_artifact_id = artifacts
            .iter()
            .find(|row| row["attempt_id"] != attempt_id)
            .map(|row| row["artifact_id"].as_str().unwrap().to_owned())
            .expect("manager artifact");
        // The CAS file exists under the daemon data dir; nothing else was written.
        let digest = artifact["cas_ref"]
            .as_str()
            .unwrap()
            .strip_prefix("sha256:")
            .unwrap();
        assert!(
            harness
                .layout
                .data_dir()
                .join("artifacts")
                .join(digest)
                .exists()
        );

        // outputs.open serves the CAS bytes (re-hashed on read).
        let opened = handle(
            &format!("GET /management/project/v1/outputs.open?artifact_id={artifact_id}"),
            b"",
            &harness.store,
            &harness.host,
        );
        assert_eq!(opened.status, 200, "{}", opened.body);
        assert!(opened.content_type.starts_with("text/markdown"));
        assert!(
            opened
                .body
                .contains("TASK COMPLETE: write the weekly report")
        );
        assert!(!opened.body.contains("boot-test-not-real"));

        // outputs.detail carries evidence and the verifier identity.
        let detail = get(
            &harness,
            &format!("GET /management/project/v1/outputs.detail?artifact_id={artifact_id}"),
        );
        let evidence = detail["evidence"].as_array().unwrap();
        assert_eq!(evidence.len(), 1);
        assert_eq!(evidence[0]["disposition"], "passed");
        assert_eq!(evidence[0]["verifier_ref"], ATTEMPT_ARTIFACT_VERIFIER_REF);
        assert_eq!(
            evidence[0]["principal"],
            ATTEMPT_ARTIFACT_VERIFIER_PRINCIPAL
        );
        assert!(
            evidence[0]["criteria"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["id"] == "attempt-response-status" && c["result"] == "not-used")
        );
        assert_eq!(detail["export"]["exists"], false);
        assert_eq!(detail["export"]["is_authority"], false);

        // outputs.export writes a copy into Personal Home data/ (never authority).
        let exported = post(
            &harness,
            "POST /management/project/v1/outputs.export",
            json!({"artifact_id": artifact_id}),
        );
        assert_eq!(exported.status, 200, "{}", exported.body);
        let exported_json: Value = serde_json::from_str(&exported.body).unwrap();
        let path = PathBuf::from(exported_json["export"]["path"].as_str().unwrap());
        assert!(path.starts_with(harness.layout.data_dir().join("projects")));
        assert_eq!(exported_json["export"]["is_authority"], false);
        assert_eq!(exported_json["host_file_open_e2e"], "not-run");
        let copy = fs::read_to_string(&path).expect("export copy");
        assert!(copy.contains("TASK COMPLETE: write the weekly report"));
        // Tampering the copy changes nothing about authority: open still
        // serves the CAS bytes and the artifact stays verified.
        fs::write(&path, "edited by hand").expect("edit copy");
        let reopened = handle(
            &format!("GET /management/project/v1/outputs.open?artifact_id={artifact_id}"),
            b"",
            &harness.store,
            &harness.host,
        );
        assert!(reopened.body.contains("TASK COMPLETE"));
        assert!(!reopened.body.contains("edited by hand"));

        // The Attempt row still never claims completion.
        let attempts = HostedDshAttemptStore::from_authority_store(&harness.store);
        let row = attempts.get_attempt(&attempt_id).unwrap().unwrap();
        assert!(!row.completion_claimed);
        assert_eq!(row.verification_status, "not-run");

        // No acceptance without StageTestPassed; intermediate ring never accepts.
        let refused = post(
            &harness,
            "POST /management/project/v1/run.acceptance.request",
            json!({"project_id": harness.project_id, "stage_id": "s2"}),
        );
        assert_eq!(refused.status, 422, "{}", refused.body);
        let s1_fact = post(
            &harness,
            "POST /management/project/v1/attempt.artifact.stage-test",
            json!({"artifact_id": manager_artifact_id, "stage_id": "s1"}),
        );
        assert_eq!(s1_fact.status, 200, "{}", s1_fact.body);
        let intermediate = post(
            &harness,
            "POST /management/project/v1/run.acceptance.request",
            json!({"project_id": harness.project_id, "stage_id": "s1"}),
        );
        assert_eq!(intermediate.status, 422, "{}", intermediate.body);
        assert!(intermediate.body.contains("last ring"));
        // Wrong slot: the manager's artifact cannot test the researcher's ring.
        let wrong_slot = post(
            &harness,
            "POST /management/project/v1/attempt.artifact.stage-test",
            json!({"artifact_id": manager_artifact_id, "stage_id": "s2"}),
        );
        assert_eq!(wrong_slot.status, 422, "{}", wrong_slot.body);

        // Last ring: derive StageTestPassed from evidence, then preview → confirm.
        let s2_fact = post(
            &harness,
            "POST /management/project/v1/attempt.artifact.stage-test",
            json!({"artifact_id": artifact_id, "stage_id": "s2"}),
        );
        assert_eq!(s2_fact.status, 200, "{}", s2_fact.body);
        let s2_json: Value = serde_json::from_str(&s2_fact.body).unwrap();
        assert_eq!(s2_json["stage_test_is_not_acceptance"], true);
        let requested = post(
            &harness,
            "POST /management/project/v1/run.acceptance.request",
            json!({"project_id": harness.project_id, "stage_id": "s2"}),
        );
        assert_eq!(requested.status, 200, "{}", requested.body);
        let requested_json: Value = serde_json::from_str(&requested.body).unwrap();
        assert_eq!(requested_json["subject_kind"], RUN_ACCEPTANCE_SUBJECT_KIND);
        assert_eq!(requested_json["chat_can_confirm"], false);
        let preview_id = requested_json["preview_id"].as_str().unwrap();
        let preview_digest = requested_json["preview_digest"].as_str().unwrap();
        // The pending preview is visible on the Project canvas list.
        let pending = crate::personal::project_aggregate::handle(
            &format!(
                "GET /management/project/v1/pending-previews?subject_ref={}",
                harness.project_id
            ),
            b"",
            &harness.store,
        );
        assert!(pending.body.contains(preview_id), "{}", pending.body);
        assert!(pending.body.contains(RUN_ACCEPTANCE_SUBJECT_KIND));
        // Stale digest never confirms.
        let stale = crate::personal::project_aggregate::handle(
            "POST /management/project/v1/confirm",
            json!({"preview_id": preview_id, "preview_digest": "0".repeat(64)})
                .to_string()
                .as_bytes(),
            &harness.store,
        );
        assert_ne!(stale.status, 200, "{}", stale.body);
        let confirmed = crate::personal::project_aggregate::handle(
            "POST /management/project/v1/confirm",
            json!({"preview_id": preview_id, "preview_digest": preview_digest})
                .to_string()
                .as_bytes(),
            &harness.store,
        );
        assert_eq!(confirmed.status, 200, "{}", confirmed.body);
        assert!(
            confirmed.body.contains("run_accepted"),
            "{}",
            confirmed.body
        );
        let accepted = get(
            &harness,
            &format!(
                "GET /management/project/v1/run.acceptance?project_id={}",
                harness.project_id
            ),
        );
        let acceptances = accepted["run_acceptances"].as_array().unwrap();
        assert_eq!(acceptances.len(), 1);
        assert_eq!(acceptances[0]["stage_id"], "s2");
        assert_eq!(acceptances[0]["last_ring"], true);
        assert_eq!(acceptances[0]["artifact_id"], artifact_id);
        let detail = get(
            &harness,
            &format!("GET /management/project/v1/outputs.detail?artifact_id={artifact_id}"),
        );
        assert!(detail["artifact"]["accepted_at"].is_number());
        assert_eq!(detail["run_acceptance"]["stage_id"], "s2");

        // Publication packet: planned, not published; external send via preview.
        let packet = get(
            &harness,
            &format!(
                "GET /management/project/v1/publication.packet?project_id={}&artifact_id={artifact_id}",
                harness.project_id
            ),
        );
        assert_eq!(packet["planned"], true);
        assert_eq!(packet["published"], false);
        assert_eq!(packet["chat_can_confirm"], false);
        assert_eq!(
            packet["autonomy_packet"]["outcome_verify"]["accepted"],
            true
        );
        assert_eq!(
            packet["autonomy_packet"]["outcome_verify"]["verified"],
            true
        );
        let send = post(
            &harness,
            "POST /management/project/v1/publication.external-send.request",
            json!({
                "project_id": harness.project_id,
                "artifact_id": artifact_id,
                "recipients": ["customer-a", "customer-b"],
            }),
        );
        assert_eq!(send.status, 200, "{}", send.body);
        let send_json: Value = serde_json::from_str(&send.body).unwrap();
        assert_eq!(send_json["published"], false);
        assert_eq!(send_json["send"]["state"], "previewed");
        let send_preview = send_json["preview_id"].as_str().unwrap();
        let send_digest = send_json["preview_digest"].as_str().unwrap();
        let planned = crate::personal::project_aggregate::handle(
            "POST /management/project/v1/confirm",
            json!({"preview_id": send_preview, "preview_digest": send_digest})
                .to_string()
                .as_bytes(),
            &harness.store,
        );
        assert_eq!(planned.status, 200, "{}", planned.body);
        assert!(planned.body.contains("external_send_planned"));
        let sends = get(
            &harness,
            &format!(
                "GET /management/project/v1/publication.sends?project_id={}",
                harness.project_id
            ),
        );
        let rows = sends["sends"].as_array().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["state"], "planned");
        assert_eq!(rows[0]["published"], false);
        assert_eq!(rows[0]["connector"], "none-qualified");
        assert_eq!(sends["published_any"], false);
    }
}
