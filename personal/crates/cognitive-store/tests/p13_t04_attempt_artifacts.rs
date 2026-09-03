#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P13-T04 Attempt artifacts → daemon CAS → independent verifier evidence →
//! StageTestPassed derived from evidence → run acceptance only on the last
//! ring. Failure-first: model text / `response done` / exit 0 / HTTP receipt
//! are never completion; intermediate-ring acceptance is refused;
//! file-as-authority is refused; evidence is append-only and verifier-owned;
//! planned ≠ published.

use cognitive_store::{
    ATTEMPT_ARTIFACT_FORMAT_MARKDOWN, ATTEMPT_ARTIFACT_SOURCE, ATTEMPT_ARTIFACT_VERIFIER_PRINCIPAL,
    ATTEMPT_ARTIFACT_VERIFIER_REF, ArtifactIngestSpec, ArtifactStore, AttemptArtifactStore,
    ConfirmCaller, EXTERNAL_SEND_SUBJECT_KIND, EmployeeStore, ExternalSendSpec,
    HOSTED_DSH_ARTIFACT_DIGEST, HostedArtifactObservation, HostedAttemptFrameSpec,
    HostedAttemptIntentSpec, HostedAttemptTerminalSpec, HostedDshAttemptStore, HostedDshPlane,
    PersonalDataLayout, ProjectAggregateError, ProjectAggregateStore, RUN_ACCEPTANCE_SUBJECT_KIND,
    RosterProposal, StageSpec, prepare_personal_databases,
};
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

struct Fixture {
    _tmp: TempDir,
    projects: ProjectAggregateStore,
    employees: EmployeeStore,
    attempts: HostedDshAttemptStore,
    artifacts: AttemptArtifactStore,
    cas: ArtifactStore,
    path: std::path::PathBuf,
    project_id: String,
    plan_id: String,
    manager_id: String,
    researcher_id: String,
}

fn stage(id: &str, title: &str, slot: &str) -> StageSpec {
    StageSpec {
        stage_id: id.to_owned(),
        title: title.to_owned(),
        objective: format!("{title} objective"),
        output_contract_digest: ProjectAggregateStore::digest_hex(format!("out-{id}").as_bytes()),
        acceptance_spec_ref: Some(format!("cas:spec-{id}")),
        cadence_json: Some(r#"{"kind":"manual"}"#.to_owned()),
        responsible_slot: slot.to_owned(),
        blocking_gap: None,
    }
}

fn fixture() -> Fixture {
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
    let path = layout.authority_database_path();
    let projects = ProjectAggregateStore::open_path(&path).expect("projects");
    let employees = EmployeeStore::open_path(&path).expect("employees");
    let attempts = HostedDshAttemptStore::open_path(&path).expect("attempts");
    let artifacts = AttemptArtifactStore::open_path(&path).expect("artifacts");
    let cas =
        ArtifactStore::open(layout.data_dir().join("artifacts"), 8 * 1024 * 1024).expect("cas");

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
    attempts
        .record_artifact_observation(
            ConfirmCaller::OwnerManagement,
            &HostedArtifactObservation {
                configured_revision: Some(HOSTED_DSH_ARTIFACT_DIGEST.to_owned()),
                pin_file_revision: Some(HOSTED_DSH_ARTIFACT_DIGEST.to_owned()),
                health: "pinned".to_owned(),
                child_script_digest: Some("a".repeat(64)),
                detail: "config, pin file and child script agree".to_owned(),
            },
            1,
        )
        .expect("pinned");
    Fixture {
        _tmp: temporary,
        projects,
        employees,
        attempts,
        artifacts,
        cas,
        path,
        project_id,
        plan_id,
        manager_id: ids[0].clone(),
        researcher_id: ids[1].clone(),
    }
}

fn canonical_payload(text: &str, attempt_id: &str) -> String {
    let payload = serde_json::json!({
        "attempt_id": attempt_id,
        "context_digest": "c".repeat(64),
        "dsh_exit": 0,
        "task_ref": "task://personal/p13-t04",
        "text": text,
    });
    serde_json_canonicalizer::to_string(&payload).expect("canonical")
}

fn digest_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// One terminal Attempt whose child claimed `done` with exit 0 and emitted one
/// `DeliverableDraft` candidate. Returns `(attempt_id, canonical payload)`.
fn terminal_attempt(fixture: &Fixture, employee_id: &str, text: &str) -> Option<(String, String)> {
    let revision = fixture
        .employees
        .latest_revision_id(employee_id)
        .expect("rev")
        .expect("id");
    let outcome = fixture.attempts.persist_intent(
        ConfirmCaller::OwnerManagement,
        &HostedAttemptIntentSpec {
            employee_id,
            employee_revision_id: &revision,
            task_ref: "task://personal/p13-t04",
            bounded_context: "write the weekly report",
            artifact_digest: HOSTED_DSH_ARTIFACT_DIGEST,
            now_ms: 40,
        },
    );
    if HostedDshPlane::isolated_spawn_is_fenced() {
        outcome.expect_err("gnu fence");
        return None;
    }
    let attempt_id = outcome.expect("persist").attempt_id;
    fixture
        .attempts
        .mark_dispatched(&attempt_id, Some("dshchild-p13-t04"), 4242, 50)
        .expect("dispatched");
    let payload = canonical_payload(text, &attempt_id);
    let frames = vec![
        HostedAttemptFrameSpec {
            seq: 1,
            kind: "observation".to_owned(),
            operation: None,
            payload_digest: None,
            reject_reason: None,
            text_redacted: "child.started".to_owned(),
        },
        HostedAttemptFrameSpec {
            seq: 2,
            kind: "candidate".to_owned(),
            operation: Some("DeliverableDraft".to_owned()),
            payload_digest: Some(digest_hex(payload.as_bytes())),
            reject_reason: None,
            text_redacted: text.chars().take(64).collect(),
        },
        HostedAttemptFrameSpec {
            seq: 3,
            kind: "response".to_owned(),
            operation: None,
            payload_digest: None,
            reject_reason: None,
            text_redacted: "done".to_owned(),
        },
    ];
    fixture
        .attempts
        .record_frames(&attempt_id, &frames, 55)
        .expect("frames");
    fixture
        .attempts
        .record_terminal(
            &attempt_id,
            &HostedAttemptTerminalSpec {
                terminal_kind: "exited",
                exit_code: Some(0),
                response_status: Some("done"),
                candidate_count: 1,
                observation_count: 1,
                rejected_frame_count: 0,
                unknown_line_count: 0,
                stdout_bytes: 512,
                stdout_truncated: false,
                stderr_tail_redacted: "",
                elapsed_ms: 12,
                now_ms: 60,
            },
        )
        .expect("terminal");
    Some((attempt_id, payload))
}

fn ingest(
    fixture: &Fixture,
    attempt_id: &str,
    payload: &str,
) -> cognitive_store::AttemptArtifactRow {
    fixture
        .artifacts
        .ingest_candidate(
            &fixture.cas,
            &ArtifactIngestSpec {
                attempt_id,
                source_frame_seq: 2,
                payload_canonical: payload,
                now_ms: 70,
            },
        )
        .expect("ingest")
}

#[test]
fn p13_t04_model_text_and_done_receipt_are_never_completion() {
    let fixture = fixture();
    let Some((attempt_id, payload)) = terminal_attempt(
        &fixture,
        &fixture.researcher_id,
        "TASK COMPLETE. Success! The weekly report is done and verified.",
    ) else {
        return;
    };
    let artifact = ingest(&fixture, &attempt_id, &payload);
    assert_eq!(artifact.attempt_id, attempt_id);
    assert_eq!(artifact.project_id, fixture.project_id);
    assert_eq!(artifact.format, ATTEMPT_ARTIFACT_FORMAT_MARKDOWN);
    assert_eq!(artifact.source, ATTEMPT_ARTIFACT_SOURCE);
    assert_eq!(artifact.source_frame_seq, 2);
    assert_eq!(
        artifact.source_payload_digest,
        digest_hex(payload.as_bytes())
    );
    assert!(artifact.cas_ref.starts_with("sha256:"));
    assert_eq!(
        artifact.byte_length as usize,
        "TASK COMPLETE. Success! The weekly report is done and verified.".len()
    );
    assert_eq!(artifact.freshness, "current");
    // Ingest is an observation: verification is not-run until the verifier ran.
    assert_eq!(artifact.verification_status, "not-run");
    assert!(artifact.accepted_at.is_none());
    let attempt = fixture
        .attempts
        .get_attempt(&attempt_id)
        .expect("get")
        .expect("row");
    assert_eq!(attempt.response_status, "done");
    assert_eq!(attempt.exit_code, Some(0));
    assert!(!attempt.completion_claimed);
    assert_eq!(attempt.verification_status, "not-run");

    // `response done` + exit 0 + model prose cannot derive StageTestPassed
    // without verifier evidence.
    let error = fixture
        .artifacts
        .derive_stage_test(
            ConfirmCaller::OwnerManagement,
            &fixture.projects,
            &fixture.employees,
            &fixture.cas,
            &artifact.artifact_id,
            "s2",
            80,
        )
        .expect_err("no evidence");
    assert!(
        matches!(error, ProjectAggregateError::Rejected { detail } if detail.contains("verification")),
        "{error}"
    );
    // ... and cannot mint a run-acceptance preview either.
    let error = fixture
        .artifacts
        .request_run_acceptance(
            ConfirmCaller::OwnerManagement,
            &fixture.projects,
            &fixture.project_id,
            "s2",
            81,
        )
        .expect_err("no StageTestPassed");
    assert!(
        matches!(error, ProjectAggregateError::Rejected { .. }),
        "{error}"
    );
    assert!(
        fixture
            .artifacts
            .list_run_acceptances(&fixture.project_id)
            .expect("list")
            .is_empty()
    );
}

#[test]
fn p13_t04_independent_verifier_writes_cas_backed_evidence_and_ignores_response_status() {
    let fixture = fixture();
    let Some((attempt_id, payload)) = terminal_attempt(
        &fixture,
        &fixture.researcher_id,
        "# Weekly report\n\n4 follow-ups.",
    ) else {
        return;
    };
    let artifact = ingest(&fixture, &attempt_id, &payload);
    let evidence = fixture
        .artifacts
        .verify_artifact(&fixture.cas, &artifact.artifact_id, 90)
        .expect("verify");
    assert_eq!(evidence.artifact_id, artifact.artifact_id);
    assert_eq!(evidence.verifier_ref, ATTEMPT_ARTIFACT_VERIFIER_REF);
    assert_eq!(evidence.principal, ATTEMPT_ARTIFACT_VERIFIER_PRINCIPAL);
    assert_eq!(evidence.disposition, "passed");
    assert!(evidence.report_cas_ref.starts_with("sha256:"));
    let report = fixture
        .cas
        .get(&evidence.report_cas_ref)
        .expect("cas")
        .expect("report bytes");
    let report: serde_json::Value = serde_json::from_slice(&report).expect("json");
    assert_eq!(report["verifier_ref"], ATTEMPT_ARTIFACT_VERIFIER_REF);
    assert_eq!(report["disposition"], "passed");
    let criteria = report["criteria"].as_array().expect("criteria");
    assert!(
        criteria
            .iter()
            .any(|c| c["id"] == "cas-bytes-match-digest" && c["result"] == "pass")
    );
    assert!(
        criteria
            .iter()
            .any(|c| c["id"] == "source-frame-bound" && c["result"] == "pass")
    );
    assert!(
        criteria
            .iter()
            .any(|c| c["id"] == "attempt-terminal-observed" && c["result"] == "pass")
    );
    // The child's own `response done` is recorded as ignored, never as a pass input.
    assert!(
        criteria
            .iter()
            .any(|c| c["id"] == "attempt-response-status" && c["result"] == "not-used")
    );

    let detail = fixture
        .artifacts
        .get_artifact(&artifact.artifact_id)
        .expect("get")
        .expect("row");
    assert_eq!(detail.verification_status, "passed");
    assert_eq!(
        detail.latest_evidence_id.as_deref(),
        Some(evidence.evidence_id.as_str())
    );
    // The Attempt row itself still never claims completion (v36 CHECK).
    let attempt = fixture
        .attempts
        .get_attempt(&attempt_id)
        .expect("get")
        .expect("row");
    assert!(!attempt.completion_claimed);
    assert_eq!(attempt.verification_status, "not-run");

    // Evidence is verifier-owned and append-only: no caller can write `passed`.
    let conn = Connection::open(&fixture.path).expect("open");
    let error = conn
        .execute(
            "UPDATE p13_artifact_evidence SET disposition = 'failed' WHERE evidence_id = ?1",
            [&evidence.evidence_id],
        )
        .expect_err("evidence is append-only");
    assert!(format!("{error}").contains("append-only"));
    let error = conn
        .execute(
            "INSERT INTO p13_artifact_evidence (
                evidence_id, artifact_id, verifier_ref, verifier_version, principal, disposition,
                criteria_json, report_cas_ref, checked_cas_ref, verified_at
             ) VALUES ('ev-forged', ?1, 'verifier://child/self-report', 'v1',
                       'principal://personal/independent-verifier', 'passed', '[]',
                       'sha256:0000000000000000000000000000000000000000000000000000000000000000',
                       ?2, 91)",
            [&artifact.artifact_id, &artifact.cas_ref],
        )
        .expect_err("foreign verifier identity is refused by CHECK");
    assert!(format!("{error}").contains("CHECK"));
}

#[test]
fn p13_t04_file_as_authority_is_refused_and_tampered_cas_fails_verification() {
    let fixture = fixture();
    let Some((attempt_id, payload)) =
        terminal_attempt(&fixture, &fixture.researcher_id, "# Weekly report\n\nbody")
    else {
        return;
    };
    // A filesystem path is never an artifact reference.
    for forged in [
        "file:///home/owner/report.md",
        "/home/owner/report.md",
        "C:\\Users\\owner\\report.md",
        "artifact://sha256/not-cas",
    ] {
        let error = fixture
            .artifacts
            .resolve_openable_ref(forged)
            .expect_err("file is not authority");
        assert!(
            matches!(error, ProjectAggregateError::Invalid { .. }),
            "{forged}"
        );
    }
    // A payload whose digest does not match the observed candidate frame is
    // not the child's deliverable and cannot be ingested.
    let error = fixture
        .artifacts
        .ingest_candidate(
            &fixture.cas,
            &ArtifactIngestSpec {
                attempt_id: &attempt_id,
                source_frame_seq: 2,
                payload_canonical: &canonical_payload("someone else's text", &attempt_id),
                now_ms: 70,
            },
        )
        .expect_err("payload digest must bind to the frame");
    assert!(
        matches!(error, ProjectAggregateError::Rejected { .. }),
        "{error}"
    );
    // A non-candidate frame cannot become an artifact.
    let error = fixture
        .artifacts
        .ingest_candidate(
            &fixture.cas,
            &ArtifactIngestSpec {
                attempt_id: &attempt_id,
                source_frame_seq: 1,
                payload_canonical: &payload,
                now_ms: 70,
            },
        )
        .expect_err("observation frame is not a deliverable");
    assert!(
        matches!(error, ProjectAggregateError::Rejected { .. }),
        "{error}"
    );

    let artifact = ingest(&fixture, &attempt_id, &payload);
    let passed = fixture
        .artifacts
        .verify_artifact(&fixture.cas, &artifact.artifact_id, 90)
        .expect("verify");
    assert_eq!(passed.disposition, "passed");

    // Tamper the CAS bytes on disk: the verifier re-reads and fails; the
    // earlier passed evidence is superseded and StageTestPassed is refused.
    let digest = artifact.cas_ref.strip_prefix("sha256:").unwrap().to_owned();
    let cas_path = fixture
        ._tmp
        .path()
        .join("data/cognitiveos/artifacts")
        .join(&digest);
    assert!(cas_path.exists(), "{}", cas_path.display());
    std::fs::write(&cas_path, b"# tampered on disk").expect("tamper");
    let failed = fixture
        .artifacts
        .verify_artifact(&fixture.cas, &artifact.artifact_id, 95)
        .expect("verify again");
    assert_eq!(failed.disposition, "failed");
    let detail = fixture
        .artifacts
        .get_artifact(&artifact.artifact_id)
        .expect("get")
        .expect("row");
    assert_eq!(detail.verification_status, "failed");
    let error = fixture
        .artifacts
        .derive_stage_test(
            ConfirmCaller::OwnerManagement,
            &fixture.projects,
            &fixture.employees,
            &fixture.cas,
            &artifact.artifact_id,
            "s2",
            96,
        )
        .expect_err("tampered bytes cannot pass");
    assert!(
        matches!(error, ProjectAggregateError::Rejected { .. }),
        "{error}"
    );
    let history = fixture
        .artifacts
        .list_evidence(&artifact.artifact_id)
        .expect("history");
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].disposition, "failed");
    assert_eq!(history[1].disposition, "passed");
}

#[test]
fn p13_t04_stage_test_is_derived_from_evidence_and_acceptance_only_on_last_ring() {
    let fixture = fixture();
    // Intermediate ring (s1, manager) produces a verified artifact.
    let Some((manager_attempt, manager_payload)) =
        terminal_attempt(&fixture, &fixture.manager_id, "# Plan\n\nring one output")
    else {
        return;
    };
    let manager_artifact = ingest(&fixture, &manager_attempt, &manager_payload);
    fixture
        .artifacts
        .verify_artifact(&fixture.cas, &manager_artifact.artifact_id, 90)
        .expect("verify");
    // The researcher is not responsible for s1: cross-slot stage test refused.
    let error = fixture
        .artifacts
        .derive_stage_test(
            ConfirmCaller::OwnerManagement,
            &fixture.projects,
            &fixture.employees,
            &fixture.cas,
            &manager_artifact.artifact_id,
            "s2",
            91,
        )
        .expect_err("manager artifact is not the s2 slot");
    assert!(
        matches!(error, ProjectAggregateError::Rejected { .. }),
        "{error}"
    );
    let s1_fact = fixture
        .artifacts
        .derive_stage_test(
            ConfirmCaller::OwnerManagement,
            &fixture.projects,
            &fixture.employees,
            &fixture.cas,
            &manager_artifact.artifact_id,
            "s1",
            92,
        )
        .expect("s1 stage test");
    assert!(s1_fact.starts_with("fact-"));
    // Intermediate-ring acceptance is refused even with a current StageTestPassed.
    let error = fixture
        .artifacts
        .request_run_acceptance(
            ConfirmCaller::OwnerManagement,
            &fixture.projects,
            &fixture.project_id,
            "s1",
            93,
        )
        .expect_err("not the last ring");
    assert!(
        matches!(error, ProjectAggregateError::Rejected { detail } if detail.contains("last ring")),
        "{error}"
    );

    // Last ring (s2, researcher).
    let Some((attempt_id, payload)) =
        terminal_attempt(&fixture, &fixture.researcher_id, "# Weekly report\n\nfinal")
    else {
        return;
    };
    let artifact = ingest(&fixture, &attempt_id, &payload);
    // Freshness is per Member: the researcher's later deliverable on the same
    // task ref does not supersede the manager's ring-one artifact.
    assert_eq!(
        fixture
            .artifacts
            .get_artifact(&manager_artifact.artifact_id)
            .expect("get")
            .expect("row")
            .freshness,
        "current"
    );
    assert_eq!(artifact.freshness, "current");
    // Task channel / assistant cannot derive a stage test.
    let error = fixture
        .artifacts
        .derive_stage_test(
            ConfirmCaller::TaskChannel,
            &fixture.projects,
            &fixture.employees,
            &fixture.cas,
            &artifact.artifact_id,
            "s2",
            94,
        )
        .expect_err("task channel");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));
    // Without evidence the last ring still cannot accept.
    let error = fixture
        .artifacts
        .request_run_acceptance(
            ConfirmCaller::OwnerManagement,
            &fixture.projects,
            &fixture.project_id,
            "s2",
            95,
        )
        .expect_err("no StageTestPassed on the last ring");
    assert!(matches!(error, ProjectAggregateError::Rejected { .. }));
    fixture
        .artifacts
        .verify_artifact(&fixture.cas, &artifact.artifact_id, 96)
        .expect("verify");
    let fact_id = fixture
        .artifacts
        .derive_stage_test(
            ConfirmCaller::OwnerManagement,
            &fixture.projects,
            &fixture.employees,
            &fixture.cas,
            &artifact.artifact_id,
            "s2",
            97,
        )
        .expect("s2 stage test");
    let (preview_id, preview_digest) = fixture
        .artifacts
        .request_run_acceptance(
            ConfirmCaller::OwnerManagement,
            &fixture.projects,
            &fixture.project_id,
            "s2",
            98,
        )
        .expect("preview");
    let pending = fixture
        .projects
        .list_pending_previews(&fixture.project_id)
        .expect("pending");
    assert!(pending.iter().any(|row| {
        row.preview_id == preview_id && row.subject_kind == RUN_ACCEPTANCE_SUBJECT_KIND
    }));
    // Task channel cannot confirm; owner confirm consumes the digest-bound preview.
    let error = fixture
        .projects
        .confirm_preview(ConfirmCaller::TaskChannel, &preview_id, &preview_digest, 99)
        .expect_err("task channel confirm");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));
    let result = fixture
        .projects
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            100,
        )
        .expect("accept run");
    assert_eq!(result.kind, "run_accepted");
    assert!(result.receipt_ref.starts_with("cas:"));
    let acceptances = fixture
        .artifacts
        .list_run_acceptances(&fixture.project_id)
        .expect("list");
    assert_eq!(acceptances.len(), 1);
    assert_eq!(acceptances[0].stage_id, "s2");
    assert_eq!(acceptances[0].stage_test_fact_id, fact_id);
    assert_eq!(acceptances[0].artifact_id, artifact.artifact_id);
    assert_eq!(acceptances[0].plan_revision_id, fixture.plan_id);
    let detail = fixture
        .artifacts
        .get_artifact(&artifact.artifact_id)
        .expect("get")
        .expect("row");
    assert_eq!(detail.accepted_at, Some(100));
    // The project is still not "published" anywhere; acceptance ≠ publication.
    assert!(
        fixture
            .artifacts
            .list_external_sends(&fixture.project_id)
            .expect("sends")
            .is_empty()
    );
    // A second acceptance for the same fact is refused (one decision per fact).
    let error = fixture
        .artifacts
        .request_run_acceptance(
            ConfirmCaller::OwnerManagement,
            &fixture.projects,
            &fixture.project_id,
            "s2",
            101,
        )
        .expect_err("already accepted");
    assert!(
        matches!(error, ProjectAggregateError::Conflict { .. }),
        "{error}"
    );
    // Acceptance rows are append-only.
    let conn = Connection::open(&fixture.path).expect("open");
    let error = conn
        .execute(
            "DELETE FROM p13_run_acceptance WHERE acceptance_id = ?1",
            [&acceptances[0].acceptance_id],
        )
        .expect_err("append-only");
    assert!(format!("{error}").contains("append-only"));
}

#[test]
fn p13_t04_newer_artifact_supersedes_and_stale_preview_cannot_confirm() {
    let fixture = fixture();
    let Some((first_attempt, first_payload)) =
        terminal_attempt(&fixture, &fixture.researcher_id, "# Weekly report v1")
    else {
        return;
    };
    let first = ingest(&fixture, &first_attempt, &first_payload);
    fixture
        .artifacts
        .verify_artifact(&fixture.cas, &first.artifact_id, 90)
        .expect("verify v1");
    fixture
        .artifacts
        .derive_stage_test(
            ConfirmCaller::OwnerManagement,
            &fixture.projects,
            &fixture.employees,
            &fixture.cas,
            &first.artifact_id,
            "s2",
            91,
        )
        .expect("stage test v1");
    let (preview_id, preview_digest) = fixture
        .artifacts
        .request_run_acceptance(
            ConfirmCaller::OwnerManagement,
            &fixture.projects,
            &fixture.project_id,
            "s2",
            92,
        )
        .expect("preview v1");

    // A newer Attempt on the same task supersedes v1.
    let Some((second_attempt, second_payload)) =
        terminal_attempt(&fixture, &fixture.researcher_id, "# Weekly report v2")
    else {
        return;
    };
    let second = ingest(&fixture, &second_attempt, &second_payload);
    let first_now = fixture
        .artifacts
        .get_artifact(&first.artifact_id)
        .expect("get")
        .expect("row");
    assert_eq!(first_now.freshness, "superseded");
    assert_eq!(second.freshness, "current");
    // A superseded artifact can no longer back a StageTestPassed.
    let error = fixture
        .artifacts
        .derive_stage_test(
            ConfirmCaller::OwnerManagement,
            &fixture.projects,
            &fixture.employees,
            &fixture.cas,
            &first.artifact_id,
            "s2",
            93,
        )
        .expect_err("superseded artifact");
    assert!(
        matches!(error, ProjectAggregateError::Rejected { .. }),
        "{error}"
    );
    // The v2 artifact, once verified, replaces the current stage fact, which
    // makes the pending v1 acceptance preview stale (base digest moved).
    fixture
        .artifacts
        .verify_artifact(&fixture.cas, &second.artifact_id, 94)
        .expect("verify v2");
    fixture
        .artifacts
        .derive_stage_test(
            ConfirmCaller::OwnerManagement,
            &fixture.projects,
            &fixture.employees,
            &fixture.cas,
            &second.artifact_id,
            "s2",
            95,
        )
        .expect("stage test v2");
    let error = fixture
        .projects
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            96,
        )
        .expect_err("stale preview");
    assert!(
        matches!(error, ProjectAggregateError::Stale { .. }),
        "{error}"
    );
    let stale = fixture
        .projects
        .preview_detail(&preview_id)
        .expect("detail")
        .expect("row");
    assert_eq!(stale.status, "stale");
    assert!(
        fixture
            .artifacts
            .list_run_acceptances(&fixture.project_id)
            .expect("list")
            .is_empty()
    );
    // Outputs list shows both artifacts newest-first with honest freshness.
    let outputs = fixture
        .artifacts
        .list_artifacts(&fixture.project_id, 10)
        .expect("list");
    assert_eq!(outputs.len(), 2);
    assert_eq!(outputs[0].artifact_id, second.artifact_id);
    assert_eq!(outputs[0].freshness, "current");
    assert_eq!(outputs[1].freshness, "superseded");
}

#[test]
fn p13_t04_secret_shape_never_enters_artifact_or_evidence() {
    let fixture = fixture();
    let Some((attempt_id, _)) = terminal_attempt(
        &fixture,
        &fixture.researcher_id,
        "# Weekly report\n\nauth: Bearer sess-not-real-token sk-live-not-real",
    ) else {
        return;
    };
    let payload = canonical_payload(
        "# Weekly report\n\nauth: Bearer sess-not-real-token sk-live-not-real",
        &attempt_id,
    );
    let error = fixture
        .artifacts
        .ingest_candidate(
            &fixture.cas,
            &ArtifactIngestSpec {
                attempt_id: &attempt_id,
                source_frame_seq: 2,
                payload_canonical: &payload,
                now_ms: 70,
            },
        )
        .expect_err("secret-shaped deliverable is refused");
    assert!(
        matches!(error, ProjectAggregateError::Invalid { .. }),
        "{error}"
    );
    assert!(
        fixture
            .artifacts
            .list_artifacts(&fixture.project_id, 10)
            .expect("list")
            .is_empty()
    );
    assert!(
        !fixture
            .artifacts
            .leak_scan_contains("sess-not-real-token")
            .expect("scan")
    );
}

#[test]
fn p13_t04_external_send_is_previewed_then_planned_never_published() {
    let fixture = fixture();
    let Some((attempt_id, payload)) = terminal_attempt(
        &fixture,
        &fixture.researcher_id,
        "# Weekly report\n\nsend me",
    ) else {
        return;
    };
    let artifact = ingest(&fixture, &attempt_id, &payload);
    // Unverified artifact cannot be packaged for external send.
    let error = fixture
        .artifacts
        .request_external_send(
            ConfirmCaller::OwnerManagement,
            &fixture.projects,
            &ExternalSendSpec {
                project_id: &fixture.project_id,
                artifact_id: &artifact.artifact_id,
                recipients: &["customer-a".to_owned(), "customer-b".to_owned()],
                now_ms: 80,
            },
        )
        .expect_err("unverified");
    assert!(
        matches!(error, ProjectAggregateError::Rejected { .. }),
        "{error}"
    );
    fixture
        .artifacts
        .verify_artifact(&fixture.cas, &artifact.artifact_id, 90)
        .expect("verify");
    let packet = fixture
        .artifacts
        .publication_packet(
            &fixture.projects,
            &fixture.project_id,
            &artifact.artifact_id,
            91,
        )
        .expect("packet");
    assert_eq!(packet["planned"], true);
    assert_eq!(packet["published"], false);
    assert_eq!(packet["chat_can_confirm"], false);
    assert_eq!(packet["connector"], "none-qualified");
    for section in [
        "preview",
        "override",
        "tiered_authority",
        "observable",
        "outcome_verify",
        "memory_of_actions",
        "yield",
    ] {
        assert!(
            packet["autonomy_packet"].get(section).is_some(),
            "{section}"
        );
    }
    // Chat / task channel cannot request an external send.
    let error = fixture
        .artifacts
        .request_external_send(
            ConfirmCaller::Assistant,
            &fixture.projects,
            &ExternalSendSpec {
                project_id: &fixture.project_id,
                artifact_id: &artifact.artifact_id,
                recipients: &["customer-a".to_owned()],
                now_ms: 92,
            },
        )
        .expect_err("assistant");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));
    let send = fixture
        .artifacts
        .request_external_send(
            ConfirmCaller::OwnerManagement,
            &fixture.projects,
            &ExternalSendSpec {
                project_id: &fixture.project_id,
                artifact_id: &artifact.artifact_id,
                recipients: &["customer-a".to_owned(), "customer-b".to_owned()],
                now_ms: 93,
            },
        )
        .expect("send preview");
    assert_eq!(send.state, "previewed");
    assert!(!send.published);
    assert_eq!(send.recipient_count, 2);
    let pending = fixture
        .projects
        .list_pending_previews(&fixture.project_id)
        .expect("pending");
    assert!(pending.iter().any(|row| {
        row.preview_id == send.preview_id && row.subject_kind == EXTERNAL_SEND_SUBJECT_KIND
    }));
    let result = fixture
        .projects
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &send.preview_id,
            &send.preview_digest,
            94,
        )
        .expect("confirm send");
    assert_eq!(result.kind, "external_send_planned");
    let sends = fixture
        .artifacts
        .list_external_sends(&fixture.project_id)
        .expect("sends");
    assert_eq!(sends.len(), 1);
    assert_eq!(sends[0].state, "planned");
    assert!(!sends[0].published);
    assert!(sends[0].intent_persisted);
    assert_eq!(sends[0].connector, "none-qualified");
    // Schema pins planned ≠ published: no row can ever say published.
    let conn = Connection::open(&fixture.path).expect("open");
    let error = conn
        .execute(
            "UPDATE p13_external_send SET published = 1, state = 'published' WHERE send_id = ?1",
            [&sends[0].send_id],
        )
        .expect_err("published is unrepresentable");
    let text = format!("{error}");
    assert!(
        text.contains("CHECK") || text.contains("append-only"),
        "{text}"
    );
}
