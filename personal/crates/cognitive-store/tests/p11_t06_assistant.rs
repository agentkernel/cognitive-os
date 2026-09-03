#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P11-T06 Hidden Pi Assistant: provenance, closed schema, default-deny,
//! no archive/SecretStore/authority writes, candidate-only preview handoff.

use cognitive_store::{
    ASSISTANT_ENGINE_ID, ASSISTANT_INFERENCE_PROTOCOL, ASSISTANT_PI_PIN,
    ASSISTANT_PRIVATE_CANDIDATE_PROTOCOL, ASSISTANT_RESEARCH_FETCH_FAMILY, ArchiveAppendSpec,
    AssistantInferenceRecord, AssistantPlane, AssistantTurnSpec,
    CONVERSATION_ARCHIVE_PROJECTION_ID, ConfirmCaller, ConversationStore, EmployeeStore,
    PersonalDataLayout, ProjectAggregateError, ProjectAggregateStore, prepare_personal_databases,
};
use serde_json::json;
use tempfile::TempDir;

/// P13-T03: turns need a daemon-observed inference; this is the minimal chain
/// for the requested object kind (owner-stated provenance, no cited sources).
fn inferred_chain(object_kind: &str) -> serde_json::Value {
    json!([
        {
            "object_kind": object_kind,
            "fields": {
                "title": {"value": "brief", "provenance": {"kind": "owner-stated"}}
            }
        }
    ])
}

fn stores() -> (
    TempDir,
    ProjectAggregateStore,
    EmployeeStore,
    ConversationStore,
    AssistantPlane,
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
    let assistant = AssistantPlane::open_path(&path).expect("assistant");
    (temporary, projects, employees, conversations, assistant)
}

fn provenance(kind: &str) -> String {
    match kind {
        "sources" => json!({"kind":"sources","sources":[{"uri":"https://example.invalid/brief"}]})
            .to_string(),
        other => json!({"kind": other}).to_string(),
    }
}

fn turn<'a>(
    kind: &'a str,
    draft_id: &'a str,
    object_kind: &'a str,
    payload: &'a serde_json::Value,
    provenance_json: &'a str,
    tools: &'a [&'a str],
    inference: &'a AssistantInferenceRecord<'a>,
) -> AssistantTurnSpec<'a> {
    AssistantTurnSpec {
        kind,
        draft_id,
        object_kind,
        payload,
        provenance_json,
        project_id: None,
        tools,
        inference,
        now_ms: 40,
    }
}

fn inference<'a>(objects: &'a serde_json::Value) -> AssistantInferenceRecord<'a> {
    AssistantInferenceRecord {
        protocol: ASSISTANT_INFERENCE_PROTOCOL,
        model_id: "deepseek-chat",
        provider_round_trips: 1,
        objects,
        reply: "candidate proposed; owner review required",
        allowed_source_uris: &[],
    }
}

#[test]
fn unlabeled_assistant_candidate_register_is_rejected() {
    let (_tmp, projects, _, _, _) = stores();
    let (draft_id, _) = projects.create_draft(b"payload", 10).expect("draft");
    let ops = json!({"object_kind":"charter","title":"x"}).to_string();
    let ops_bytes = ops.as_bytes();
    let unlabeled = [
        None,
        Some(""),
        Some("notes"),
        Some("{}"),
        Some("[]"),
        Some("owner-stated"),
        Some("{\"kind\":\"sources\"}"),
        Some("{\"kind\":\"sources\",\"sources\":[]}"),
        Some("{\"kind\":\"guess\"}"),
        Some("{\"note\":\"because\"}"),
        Some("{\"kind\":\"owner-stated\",\"confidence\":0.9}"),
    ];
    for sources in unlabeled {
        let error = projects
            .register_candidate(&draft_id, 0, ops_bytes, "assistant", sources)
            .expect_err("unlabeled");
        assert!(
            matches!(error, ProjectAggregateError::Invalid { .. }),
            "expected Invalid for {sources:?}, got {error:?}"
        );
        let detail = match error {
            ProjectAggregateError::Invalid { detail } => detail,
            _ => unreachable!(),
        };
        assert!(
            detail.contains("provenance")
                || detail.contains("unlabeled")
                || detail.contains("typed"),
            "typed provenance, not merely non-null: {detail}"
        );
    }
}

#[test]
fn draft_apply_targeting_authority_objects_is_rejected() {
    let (_tmp, projects, _, _, assistant) = stores();
    let (draft_id, _) = projects.create_draft(b"charter-v1", 10).expect("draft");
    projects
        .put_draft_charter(&draft_id, b"charter-body", 11)
        .expect("charter");
    let (preview_id, preview_digest) = projects
        .request_preview("activation", &draft_id, b"preview", 12)
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
    let error = projects
        .apply_candidate(
            ConfirmCaller::OwnerManagement,
            &project_id,
            0,
            "deadbeef",
            14,
        )
        .expect_err("authority target");
    assert!(
        matches!(error, ProjectAggregateError::Invalid { detail } if detail.contains("authority"))
    );
    let assistant_error = assistant
        .apply_candidate(&project_id, 0, "deadbeef", 15)
        .expect_err("assistant apply");
    assert!(matches!(
        assistant_error,
        ProjectAggregateError::Forbidden { .. } | ProjectAggregateError::Invalid { .. }
    ));
}

#[test]
fn assistant_cannot_write_archive_secret_or_authority() {
    let (_tmp, projects, employees, _, assistant) = stores();
    let (draft_id, _) = projects.create_draft(b"charter-v1", 10).expect("draft");
    projects
        .put_draft_charter(&draft_id, b"charter-body", 11)
        .expect("charter");
    let (preview_id, preview_digest) = projects
        .request_preview("activation", &draft_id, b"preview", 12)
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
    let archive_err = assistant
        .write_archive(&ArchiveAppendSpec {
            projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
            project_id: &project_id,
            employee_id: "employee-missing",
            kind: "note",
            body: "assistant must not land archive",
            now_ms: 20,
        })
        .expect_err("archive");
    assert!(matches!(
        archive_err,
        ProjectAggregateError::Forbidden { detail } if detail.contains("archive")
    ));
    let secret_err = assistant
        .write_secret("provider", "sk-not-a-real-key")
        .expect_err("secret");
    assert!(matches!(
        secret_err,
        ProjectAggregateError::Forbidden { detail } if detail.contains("SecretStore")
    ));
    let memory_err = assistant.write_memory().expect_err("memory");
    assert!(matches!(
        memory_err,
        ProjectAggregateError::Forbidden { detail } if detail.contains("Memory")
    ));
    let grant_err = assistant
        .grant_capability(
            &employees,
            &project_id,
            "employee-missing",
            "workspace-write",
            "project",
            21,
        )
        .expect_err("grant");
    assert!(matches!(
        grant_err,
        ProjectAggregateError::Forbidden { detail } if detail.contains("authority")
    ));
    let confirm_err = assistant
        .confirm_preview(&preview_id, &preview_digest, 22)
        .expect_err("confirm");
    assert!(matches!(
        confirm_err,
        ProjectAggregateError::Forbidden { .. }
    ));
}

#[test]
fn closed_schema_rejects_grant_secret_and_trigger_arm() {
    let (_tmp, projects, _, _, _) = stores();
    let (draft_id, _) = projects.create_draft(b"payload", 10).expect("draft");
    let provenance = provenance("owner-stated");
    for forbidden in [
        json!({"object_kind":"recipe","grant":"workspace-write"}),
        json!({"object_kind":"recipe","secret":"sk-not-a-real-key"}),
        json!({"object_kind":"recipe","trigger-arm":true}),
        json!({"object_kind":"recipe","nested":{"trigger_arm":"1"}}),
        json!({"object_kind":"recipe","grant_id":"g1"}),
        json!({"object_kind":"recipe","secret_ref":"ref"}),
    ] {
        let bytes = serde_json::to_vec(&forbidden).expect("json");
        let error = projects
            .register_candidate(&draft_id, 0, &bytes, "assistant", Some(&provenance))
            .expect_err("closed schema");
        assert!(
            matches!(error, ProjectAggregateError::Invalid { detail } if detail.contains("grant") || detail.contains("secret") || detail.contains("trigger") || detail.contains("closed")),
            "{forbidden} -> {error:?}"
        );
    }
}

#[test]
fn default_deny_tools_and_ambient_shell_are_rejected() {
    for tool in ["bash", "shell", "edit", "write", "powershell", "cmd"] {
        let error = AssistantPlane::admit_tool("explain", tool).expect_err(tool);
        assert!(matches!(
            error,
            ProjectAggregateError::Forbidden { detail } if detail.contains("ambient") || detail.contains("default-deny")
        ));
    }
    let unknown =
        AssistantPlane::admit_tool("propose", "WorkspaceWrite").expect_err("write family");
    assert!(matches!(unknown, ProjectAggregateError::Forbidden { .. }));
    AssistantPlane::admit_tool("research", ASSISTANT_RESEARCH_FETCH_FAMILY).expect("pinned fetch");
    let fetch_on_propose =
        AssistantPlane::admit_tool("propose", ASSISTANT_RESEARCH_FETCH_FAMILY).expect_err("scope");
    assert!(matches!(
        fetch_on_propose,
        ProjectAggregateError::Forbidden { .. }
    ));
}

#[test]
fn vertical_turns_register_candidate_and_preview_handoff() {
    let (_tmp, projects, _, _, assistant) = stores();
    let (draft_id, _) = projects.create_draft(b"payload", 10).expect("draft");
    let payload = json!({"title":"brief"});
    let sources = provenance("sources");
    let owner_stated = provenance("owner-stated");
    let assumption = provenance("assistant-assumption");
    let brief_chain = inferred_chain("business-brief");
    let brief_inference = inference(&brief_chain);
    let explain = assistant
        .run_turn(&turn(
            "explain",
            &draft_id,
            "business-brief",
            &payload,
            &sources,
            &[],
            &brief_inference,
        ))
        .expect("explain");
    assert!(explain.preview_id.is_none());
    assert_eq!(explain.engine_id, ASSISTANT_ENGINE_ID);
    assert_eq!(explain.pi_pin, ASSISTANT_PI_PIN);
    assert_eq!(explain.protocol, ASSISTANT_PRIVATE_CANDIDATE_PROTOCOL);
    assert_eq!(explain.candidate_digest.len(), 64);
    let axis_chain = inferred_chain("axis");
    let axis_inference = inference(&axis_chain);
    assistant
        .run_turn(&turn(
            "navigate",
            &draft_id,
            "axis",
            &payload,
            &owner_stated,
            &[],
            &axis_inference,
        ))
        .expect("navigate");
    let (research_draft, _) = projects
        .create_draft(b"research", 11)
        .expect("research draft");
    let research_chain = inferred_chain("research-run");
    let research_inference = inference(&research_chain);
    let research = assistant
        .run_turn(&turn(
            "research",
            &research_draft,
            "research-run",
            &payload,
            &assumption,
            &[ASSISTANT_RESEARCH_FETCH_FAMILY],
            &research_inference,
        ))
        .expect("research");
    assert!(research.preview_id.is_some());
    let (propose_draft, _) = projects
        .create_draft(b"propose", 12)
        .expect("propose draft");
    let charter_chain = inferred_chain("charter");
    let charter_inference = inference(&charter_chain);
    let propose = assistant
        .run_turn(&turn(
            "propose",
            &propose_draft,
            "charter",
            &payload,
            &owner_stated,
            &[],
            &charter_inference,
        ))
        .expect("propose");
    assert!(propose.preview_id.is_some());
    assert_eq!(propose.candidate_digest.len(), 64);
    let confirm_err = assistant
        .confirm_preview(propose.preview_id.as_deref().unwrap(), "deadbeef", 50)
        .expect_err("chat has no Approve");
    assert!(matches!(
        confirm_err,
        ProjectAggregateError::Forbidden { .. }
    ));
}
