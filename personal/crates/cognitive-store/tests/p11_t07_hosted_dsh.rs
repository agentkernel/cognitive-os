#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P11-T07 hidden hosted DSH: digest/protocol mismatch, secret isolation,
//! unknown output ≠ success, process death preserves Employee/conversation/Memory,
//! Pi is not the Member engine, task channel cannot bind.

use cognitive_store::{
    ArchiveAppendSpec, CONVERSATION_ARCHIVE_PROJECTION_ID, ConfirmCaller, ConversationStore,
    EmployeeStore, HOSTED_DSH_ARTIFACT_DIGEST, HOSTED_DSH_ENGINE_ID, HOSTED_DSH_PATH_B_AGENT,
    HOSTED_DSH_PROTOCOL, HOSTED_DSH_PROVIDER_PROXY, HOSTED_DSH_WIN_GNU_FENCE, HostedDshPlane,
    HostedDshStartSpec, PersonalDataLayout, ProjectAggregateError, ProjectAggregateStore,
    RosterProposal, StageSpec, prepare_personal_databases,
};
use rusqlite::Connection;
use tempfile::TempDir;

fn stores() -> (
    TempDir,
    ProjectAggregateStore,
    EmployeeStore,
    ConversationStore,
    HostedDshPlane,
    std::path::PathBuf,
) {
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
    let conversations = ConversationStore::open_path(&path).expect("conversations");
    let hosted = HostedDshPlane::open_path(&path).expect("hosted");
    (temporary, projects, employees, conversations, hosted, path)
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

fn activate(projects: &ProjectAggregateStore) -> String {
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

fn plan_two_slots(projects: &ProjectAggregateStore, project_id: &str) -> String {
    projects
        .apply_plan_revision(
            project_id,
            project_id,
            &[
                stage("s1", "Manage", "manager"),
                stage("s2", "Research", "researcher"),
            ],
            20,
        )
        .expect("plan")
}

fn proposals() -> [RosterProposal; 2] {
    [
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
    ]
}

struct Fixture {
    _tmp: TempDir,
    employees: EmployeeStore,
    conversations: ConversationStore,
    hosted: HostedDshPlane,
    path: std::path::PathBuf,
    project_id: String,
    employee_id: String,
    revision_id: String,
}

fn seated_fixture() -> Fixture {
    let (tmp, projects, employees, conversations, hosted, path) = stores();
    let project_id = activate(&projects);
    let plan_id = plan_two_slots(&projects, &project_id);
    let ids = employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_id,
            &proposals(),
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
    let revision_id = employees
        .latest_revision_id(&ids[0])
        .expect("rev")
        .expect("id");
    Fixture {
        _tmp: tmp,
        employees,
        conversations,
        hosted,
        path,
        project_id,
        employee_id: ids[0].clone(),
        revision_id,
    }
}

fn start_spec<'a>(
    fixture: &'a Fixture,
    artifact: &'a str,
    protocol: &'a str,
    engine: &'a str,
    argv: &'a [&'a str],
    env_pairs: &'a [(&'a str, &'a str)],
    child_output: Option<&'a str>,
) -> HostedDshStartSpec<'a> {
    HostedDshStartSpec {
        employee_id: &fixture.employee_id,
        employee_revision_id: &fixture.revision_id,
        task_ref: "task://personal/hosted-dsh-attempt",
        bounded_context: "sha256:bounded-context",
        artifact_digest: artifact,
        protocol,
        engine_id: engine,
        observed_pid: Some(4242),
        argv,
        env_pairs,
        child_output,
        now_ms: 40,
    }
}

fn seed_memory_object(path: &std::path::Path) -> String {
    let conn = Connection::open(path).expect("open memory seed");
    conn.execute(
        "INSERT INTO workspace_context_sources (
            source_id, source_digest, tenant_id, owner_ref, resource_scope,
            conversation_ref, role, trust_level, representation, provenance_ref,
            content_bytes, content_tokens, canonical_json
         ) VALUES (
            'src-hosted-dsh', 'digest', 'tenant', 'owner', 'scope',
            NULL, 'working', 'verified', 'text', 'prov',
            1, 1, '{}'
         )",
        [],
    )
    .expect("source");
    conn.execute(
        "INSERT INTO memory_candidates (
            candidate_id, source_id, source_digest, source_provenance_ref,
            governance_scope, target_scope, purpose, retention_expires_at_unix_seconds,
            observed_at_unix_seconds, canonical_json
         ) VALUES (
            'memcand-hosted-dsh', 'src-hosted-dsh', 'digest', 'prov',
            'project', 'project', 'note', 1, 1, '{}'
         )",
        [],
    )
    .expect("candidate");
    conn.execute(
        "INSERT INTO memory_admission_decisions (
            decision_id, candidate_id, candidate_digest, decision, policy_version,
            reason_codes_json, canonical_json
         ) VALUES (
            'memdec-hosted-dsh', 'memcand-hosted-dsh', 'digest', 'admit', 1,
            '[]', '{}'
         )",
        [],
    )
    .expect("decision");
    conn.execute(
        "INSERT INTO memory_objects (
            memory_id, candidate_id, decision_id, canonical_json
         ) VALUES (
            'mem-hosted-dsh', 'memcand-hosted-dsh', 'memdec-hosted-dsh', '{}'
         )",
        [],
    )
    .expect("object");
    "mem-hosted-dsh".to_owned()
}

fn start_or_skip_gnu(
    hosted: &HostedDshPlane,
    spec: &HostedDshStartSpec<'_>,
) -> Result<cognitive_store::HostedDshObservation, ProjectAggregateError> {
    hosted.start(ConfirmCaller::OwnerManagement, spec)
}

#[test]
fn p11_t07_digest_mismatch_is_rejected() {
    let fixture = seated_fixture();
    let spec = start_spec(
        &fixture,
        "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
        HOSTED_DSH_PROTOCOL,
        HOSTED_DSH_ENGINE_ID,
        &[],
        &[],
        None,
    );
    let error = start_or_skip_gnu(&fixture.hosted, &spec).expect_err("digest");
    if HostedDshPlane::isolated_spawn_is_fenced() {
        assert!(format!("{error}").contains(HOSTED_DSH_WIN_GNU_FENCE));
        return;
    }
    assert!(
        matches!(error, ProjectAggregateError::Rejected { detail } if detail.contains("digest"))
    );
}

#[test]
fn p11_t07_protocol_mismatch_is_rejected() {
    let fixture = seated_fixture();
    let spec = start_spec(
        &fixture,
        HOSTED_DSH_ARTIFACT_DIGEST,
        "stdio-jsonl",
        HOSTED_DSH_ENGINE_ID,
        &[],
        &[],
        None,
    );
    let error = start_or_skip_gnu(&fixture.hosted, &spec).expect_err("protocol");
    if HostedDshPlane::isolated_spawn_is_fenced() {
        assert!(format!("{error}").contains(HOSTED_DSH_WIN_GNU_FENCE));
        return;
    }
    assert!(
        matches!(error, ProjectAggregateError::Rejected { detail } if detail.contains("protocol"))
    );
}

#[test]
fn p11_t07_secret_never_enters_child_env_or_argv() {
    let fixture = seated_fixture();
    let env = [("OPENAI_API_KEY", "sk-test-not-a-real-key")];
    let spec = start_spec(
        &fixture,
        HOSTED_DSH_ARTIFACT_DIGEST,
        HOSTED_DSH_PROTOCOL,
        HOSTED_DSH_ENGINE_ID,
        &[],
        &env,
        None,
    );
    let error = start_or_skip_gnu(&fixture.hosted, &spec).expect_err("env secret");
    if HostedDshPlane::isolated_spawn_is_fenced() {
        assert!(format!("{error}").contains(HOSTED_DSH_WIN_GNU_FENCE));
        return;
    }
    assert!(format!("{error}").contains("secret"));
    let argv = ["--token", "sk-test-not-a-real-key"];
    let spec = start_spec(
        &fixture,
        HOSTED_DSH_ARTIFACT_DIGEST,
        HOSTED_DSH_PROTOCOL,
        HOSTED_DSH_ENGINE_ID,
        &argv,
        &[],
        None,
    );
    let error = start_or_skip_gnu(&fixture.hosted, &spec).expect_err("argv secret");
    assert!(format!("{error}").contains("secret"));
}

#[test]
fn p11_t07_unknown_child_output_is_not_success() {
    let fixture = seated_fixture();
    let spec = start_spec(
        &fixture,
        HOSTED_DSH_ARTIFACT_DIGEST,
        HOSTED_DSH_PROTOCOL,
        HOSTED_DSH_ENGINE_ID,
        &[],
        &[],
        Some("success"),
    );
    let error = start_or_skip_gnu(&fixture.hosted, &spec).expect_err("unknown");
    if HostedDshPlane::isolated_spawn_is_fenced() {
        assert!(format!("{error}").contains(HOSTED_DSH_WIN_GNU_FENCE));
        return;
    }
    assert!(
        matches!(error, ProjectAggregateError::Rejected { detail } if detail.contains("not success"))
    );
}

#[test]
fn p11_t07_pi_is_not_the_member_execution_engine() {
    let fixture = seated_fixture();
    let spec = start_spec(
        &fixture,
        HOSTED_DSH_ARTIFACT_DIGEST,
        HOSTED_DSH_PROTOCOL,
        "cognitiveos.personal.hidden-pi-assistant/0.1",
        &[],
        &[],
        None,
    );
    let error = start_or_skip_gnu(&fixture.hosted, &spec).expect_err("pi");
    if HostedDshPlane::isolated_spawn_is_fenced() {
        assert!(format!("{error}").contains(HOSTED_DSH_WIN_GNU_FENCE));
        return;
    }
    assert!(format!("{error}").contains("Pi is not the Member"));
    let bind_err = fixture
        .employees
        .bind_runtime(
            ConfirmCaller::OwnerManagement,
            &fixture.employee_id,
            "pi:member-engine",
            41,
        )
        .expect_err("bind pi");
    assert!(format!("{bind_err}").contains("Pi is not the Member"));
}

#[test]
fn p11_t07_task_channel_cannot_bind_hosted_dsh() {
    let fixture = seated_fixture();
    let spec = start_spec(
        &fixture,
        HOSTED_DSH_ARTIFACT_DIGEST,
        HOSTED_DSH_PROTOCOL,
        HOSTED_DSH_ENGINE_ID,
        &[],
        &[],
        None,
    );
    let error = fixture
        .hosted
        .start(ConfirmCaller::TaskChannel, &spec)
        .expect_err("task");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));
}

#[test]
fn p11_t07_process_death_does_not_delete_employee_conversation_or_memory() {
    let fixture = seated_fixture();
    let spec = start_spec(
        &fixture,
        HOSTED_DSH_ARTIFACT_DIGEST,
        HOSTED_DSH_PROTOCOL,
        HOSTED_DSH_ENGINE_ID,
        &["--isolated"],
        &[("PATH", "/usr/bin")],
        None,
    );
    if HostedDshPlane::isolated_spawn_is_fenced() {
        let error = start_or_skip_gnu(&fixture.hosted, &spec).expect_err("gnu fence");
        assert!(format!("{error}").contains(HOSTED_DSH_WIN_GNU_FENCE));
        return;
    }
    let started = start_or_skip_gnu(&fixture.hosted, &spec).expect("start");
    assert_eq!(started.terminal_kind, "started");
    assert_ne!(started.terminal_kind, "success");
    assert_eq!(started.provider_proxy, HOSTED_DSH_PROVIDER_PROXY);
    assert_eq!(started.path_b_agent, HOSTED_DSH_PATH_B_AGENT);
    assert!(!started.installed_agent);
    assert!(!started.pi_member_engine);
    assert_eq!(started.secret_bearer, "daemon-proxy-only");
    let encoded = serde_json::to_string(&serde_json::json!({
        "env": "PATH",
        "argv": started.spawn_kind,
        "proxy": started.provider_proxy,
    }))
    .expect("json");
    assert!(!encoded.to_ascii_lowercase().contains("sk-"));
    assert!(!encoded.to_ascii_lowercase().contains("api_key"));

    let record_id = fixture
        .conversations
        .append(
            ConfirmCaller::OwnerManagement,
            &ArchiveAppendSpec {
                projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
                project_id: &fixture.project_id,
                employee_id: &fixture.employee_id,
                kind: "note",
                body: "keep me",
                now_ms: 50,
            },
        )
        .expect("archive");
    let memory_id = seed_memory_object(&fixture.path);
    let before_employee = fixture
        .employees
        .get_employee(&fixture.employee_id)
        .expect("get")
        .expect("row");

    fixture
        .hosted
        .observe_exit(&fixture.employee_id)
        .expect("exit")
        .expect("child");
    let after_employee = fixture
        .employees
        .get_employee(&fixture.employee_id)
        .expect("get after")
        .expect("row");
    assert_eq!(before_employee.employee_id, after_employee.employee_id);
    assert_eq!(after_employee.state, "seated");
    assert_eq!(
        after_employee.runtime_binding_ref.as_deref(),
        Some(started.runtime_binding_ref.as_str())
    );
    let after_child = fixture
        .hosted
        .latest_child(&fixture.employee_id)
        .expect("latest")
        .expect("row");
    assert_eq!(after_child.state, "exited");
    assert_eq!(after_child.terminal_kind, "exited");
    assert!(after_child.pid.is_none());

    let conn = Connection::open(&fixture.path).expect("open");
    let archive_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM p11_conversation_archive WHERE record_id = ?1",
            [&record_id],
            |row| row.get(0),
        )
        .expect("archive count");
    assert_eq!(archive_count, 1);
    let memory_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM memory_objects WHERE memory_id = ?1",
            [&memory_id],
            |row| row.get(0),
        )
        .expect("memory count");
    assert_eq!(memory_count, 1);
}

#[test]
fn p11_t07_isolated_spawn_fence_matches_windows_gnu() {
    assert_eq!(
        HostedDshPlane::isolated_spawn_is_fenced(),
        cfg!(all(windows, target_env = "gnu"))
    );
}
