#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P13-T03 Hidden Pi Assistant real inference — store-side negatives.
//!
//! `run_turn` may register a candidate only from a daemon-observed inference:
//! exact Pi reached the Provider through the daemon proxy (at least one
//! `provider_round_trips`) and returned a closed object chain where every field
//! carries typed provenance and every cited `sources[]` uri was fetched or
//! owner-supplied. Echo, fabricated sources, unprovenanced fields, ambient
//! tools, and assistant writes to authority/Secret/archive/Memory are refused.

use cognitive_store::{
    ASSISTANT_INFERENCE_PROTOCOL, ASSISTANT_RESEARCH_FETCH_FAMILY, ASSISTANT_SETTINGS_ROUTE,
    ArchiveAppendSpec, AssistantInferenceRecord, AssistantPlane, AssistantTurnSpec,
    CONTEXT_INJECT_ORDER, CONVERSATION_ARCHIVE_PROJECTION_ID, ConfirmCaller, EmployeeStore,
    PersonalDataLayout, ProjectAggregateError, ProjectAggregateStore, prepare_personal_databases,
    provider_unbound_guidance, validate_inferred_object_chain,
};
use serde_json::{Value, json};
use tempfile::TempDir;

fn stores() -> (
    TempDir,
    ProjectAggregateStore,
    EmployeeStore,
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
    let assistant = AssistantPlane::open_path(&path).expect("assistant");
    (temporary, projects, employees, assistant)
}

fn owner_stated() -> String {
    json!({"kind": "owner-stated"}).to_string()
}

fn chain(object_kind: &str) -> Value {
    json!([
        {
            "object_kind": object_kind,
            "summary": "inferred by exact Pi through the daemon proxy",
            "fields": {
                "title": {"value": "Weekly business report", "provenance": {"kind": "owner-stated"}},
                "cadence": {"value": "weekly", "provenance": {"kind": "assistant-assumption"}}
            }
        }
    ])
}

fn inference<'a>(
    objects: &'a Value,
    allowed: &'a [String],
    round_trips: u32,
) -> AssistantInferenceRecord<'a> {
    AssistantInferenceRecord {
        protocol: ASSISTANT_INFERENCE_PROTOCOL,
        model_id: "deepseek-chat",
        provider_round_trips: round_trips,
        objects,
        reply: "Here is a candidate charter; nothing is written until you confirm.",
        allowed_source_uris: allowed,
    }
}

fn spec<'a>(
    kind: &'a str,
    draft_id: &'a str,
    object_kind: &'a str,
    payload: &'a Value,
    provenance: &'a str,
    tools: &'a [&'a str],
    inference: &'a AssistantInferenceRecord<'a>,
) -> AssistantTurnSpec<'a> {
    AssistantTurnSpec {
        kind,
        draft_id,
        object_kind,
        payload,
        provenance_json: provenance,
        project_id: None,
        tools,
        inference,
        now_ms: 50,
    }
}

fn invalid_detail(error: ProjectAggregateError) -> &'static str {
    match error {
        ProjectAggregateError::Invalid { detail } => detail,
        other => panic!("expected Invalid, got {other:?}"),
    }
}

#[test]
fn turn_without_provider_round_trip_is_refused_as_echo() {
    let (_tmp, projects, _, assistant) = stores();
    let (draft_id, _) = projects.create_draft(b"payload", 10).expect("draft");
    let payload = json!({"text": "make me a weekly report"});
    let provenance = owner_stated();
    let objects = chain("charter");
    let no_round_trip = inference(&objects, &[], 0);
    let error = assistant
        .run_turn(&spec(
            "propose",
            &draft_id,
            "charter",
            &payload,
            &provenance,
            &[],
            &no_round_trip,
        ))
        .expect_err("zero Provider round trips is not inference");
    assert!(invalid_detail(error).contains("inference required"));

    let wrong_protocol = AssistantInferenceRecord {
        protocol: "cognitiveos.private-candidate/1",
        ..inference(&objects, &[], 1)
    };
    let error = assistant
        .run_turn(&spec(
            "propose",
            &draft_id,
            "charter",
            &payload,
            &provenance,
            &[],
            &wrong_protocol,
        ))
        .expect_err("protocol mismatch");
    assert!(invalid_detail(error).contains("protocol"));

    let empty_reply = AssistantInferenceRecord {
        reply: "   ",
        ..inference(&objects, &[], 1)
    };
    let error = assistant
        .run_turn(&spec(
            "propose",
            &draft_id,
            "charter",
            &payload,
            &provenance,
            &[],
            &empty_reply,
        ))
        .expect_err("empty reply");
    assert!(invalid_detail(error).contains("reply"));
    assert_eq!(projects.get_draft_seq(&draft_id).expect("seq"), 0);
}

#[test]
fn candidate_field_without_provenance_is_refused() {
    let unprovenanced = json!([
        {"object_kind": "charter", "fields": {"title": {"value": "x"}}}
    ]);
    let error = validate_inferred_object_chain(&unprovenanced, &[]).expect_err("no provenance");
    assert!(invalid_detail(error).contains("provenance"));

    let bare_value = json!([
        {"object_kind": "charter", "fields": {"title": "x"}}
    ]);
    let error = validate_inferred_object_chain(&bare_value, &[]).expect_err("bare value");
    assert!(invalid_detail(error).contains("provenance"));

    let unlabeled = json!([
        {"object_kind": "charter", "fields": {"title": {"value": "x", "provenance": {"note": "because"}}}}
    ]);
    let error = validate_inferred_object_chain(&unlabeled, &[]).expect_err("unlabeled");
    assert!(invalid_detail(error).contains("provenance"));

    let forged_confidence = json!([
        {"object_kind": "charter", "fields": {"title": {"value": "x", "provenance": {"kind": "owner-stated", "confidence": 0.99}}}}
    ]);
    let error =
        validate_inferred_object_chain(&forged_confidence, &[]).expect_err("forged confidence");
    assert!(invalid_detail(error).contains("confidence"));
}

#[test]
fn fabricated_sources_are_refused() {
    let fetched = vec!["https://example.invalid/report-format".to_owned()];
    let cites_unfetched = json!([
        {"object_kind": "research-run", "fields": {
            "finding": {"value": "x", "provenance": {"kind": "sources", "sources": [{"uri": "https://example.invalid/never-fetched"}]}}
        }}
    ]);
    let error = validate_inferred_object_chain(&cites_unfetched, &fetched).expect_err("fabricated");
    assert!(invalid_detail(error).contains("fabricated"));

    let cites_nothing = json!([
        {"object_kind": "research-run", "fields": {
            "finding": {"value": "x", "provenance": {"kind": "sources", "sources": []}}
        }}
    ]);
    let error =
        validate_inferred_object_chain(&cites_nothing, &fetched).expect_err("empty sources");
    assert!(invalid_detail(error).contains("sources"));

    let cites_fetched = json!([
        {"object_kind": "research-run", "fields": {
            "finding": {"value": "x", "provenance": {"kind": "sources", "sources": [{"uri": "https://example.invalid/report-format"}]}}
        }}
    ]);
    let kinds =
        validate_inferred_object_chain(&cites_fetched, &fetched).expect("fetched uri is citable");
    assert_eq!(kinds, vec!["research-run".to_owned()]);
    let error = validate_inferred_object_chain(&cites_fetched, &[]).expect_err("nothing fetched");
    assert!(invalid_detail(error).contains("fabricated"));
}

#[test]
fn chain_outside_closed_kinds_order_or_schema_is_refused() {
    let unknown_kind = json!([
        {"object_kind": "invoice", "fields": {"a": {"value": 1, "provenance": {"kind": "owner-stated"}}}}
    ]);
    assert!(
        invalid_detail(
            validate_inferred_object_chain(&unknown_kind, &[]).expect_err("unknown kind")
        )
        .contains("closed")
    );

    let out_of_order = json!([
        {"object_kind": "charter", "fields": {"a": {"value": 1, "provenance": {"kind": "owner-stated"}}}},
        {"object_kind": "business-brief", "fields": {"a": {"value": 1, "provenance": {"kind": "owner-stated"}}}}
    ]);
    assert!(
        invalid_detail(
            validate_inferred_object_chain(&out_of_order, &[]).expect_err("out of order")
        )
        .contains("order")
    );

    let duplicated = json!([
        {"object_kind": "charter", "fields": {"a": {"value": 1, "provenance": {"kind": "owner-stated"}}}},
        {"object_kind": "charter", "fields": {"b": {"value": 1, "provenance": {"kind": "owner-stated"}}}}
    ]);
    assert!(
        invalid_detail(validate_inferred_object_chain(&duplicated, &[]).expect_err("duplicate"))
            .contains("repeats")
    );

    for forbidden in [
        json!([{"object_kind": "recipe", "fields": {"grant": {"value": "workspace-write", "provenance": {"kind": "owner-stated"}}}}]),
        json!([{"object_kind": "recipe", "fields": {"k": {"value": {"secret": "sk-not-a-real-key"}, "provenance": {"kind": "owner-stated"}}}}]),
        json!([{"object_kind": "recipe", "fields": {"k": {"value": {"trigger_arm": true}, "provenance": {"kind": "owner-stated"}}}}]),
        json!([{"object_kind": "recipe", "fields": {"k": {"value": {"api_key": "x"}, "provenance": {"kind": "owner-stated"}}}}]),
    ] {
        let error = validate_inferred_object_chain(&forbidden, &[]).expect_err("closed schema");
        assert!(invalid_detail(error).contains("closed"), "{forbidden}");
    }

    let extra_key = json!([
        {"object_kind": "recipe", "tool_call": "bash", "fields": {"a": {"value": 1, "provenance": {"kind": "owner-stated"}}}}
    ]);
    assert!(
        invalid_detail(validate_inferred_object_chain(&extra_key, &[]).expect_err("extra key"))
            .contains("closed schema")
    );

    assert!(validate_inferred_object_chain(&json!([]), &[]).is_err());
    assert!(validate_inferred_object_chain(&json!({"object_kind": "charter"}), &[]).is_err());

    let full_chain = json!([
        {"object_kind": "business-brief", "fields": {"a": {"value": 1, "provenance": {"kind": "owner-stated"}}}},
        {"object_kind": "research-run", "fields": {"a": {"value": 1, "provenance": {"kind": "assistant-assumption"}}}},
        {"object_kind": "charter", "fields": {"a": {"value": 1, "provenance": {"kind": "owner-stated"}}}},
        {"object_kind": "axis", "fields": {"a": {"value": 1, "provenance": {"kind": "owner-stated"}}}},
        {"object_kind": "roster", "fields": {"a": {"value": 1, "provenance": {"kind": "owner-stated"}}}},
        {"object_kind": "recipe", "fields": {"a": {"value": 1, "provenance": {"kind": "owner-stated"}}}}
    ]);
    assert_eq!(
        validate_inferred_object_chain(&full_chain, &[]).expect("full chain"),
        [
            "business-brief",
            "research-run",
            "charter",
            "axis",
            "roster",
            "recipe"
        ]
        .map(str::to_owned)
    );
}

#[test]
fn ambient_tool_is_refused_before_any_inference() {
    let (_tmp, projects, _, assistant) = stores();
    let (draft_id, _) = projects.create_draft(b"payload", 10).expect("draft");
    let payload = json!({"text": "x"});
    let provenance = owner_stated();
    let objects = chain("charter");
    let good = inference(&objects, &[], 1);
    for tool in [
        "bash",
        "shell",
        "powershell",
        "edit",
        "write",
        "apply_patch",
    ] {
        let tools = [tool];
        let error = assistant
            .run_turn(&spec(
                "propose",
                &draft_id,
                "charter",
                &payload,
                &provenance,
                &tools,
                &good,
            ))
            .expect_err(tool);
        assert!(
            matches!(error, ProjectAggregateError::Forbidden { .. }),
            "{tool}"
        );
    }
    let error = AssistantPlane::admit_turn_request("propose", "charter", &provenance, &["bash"])
        .expect_err("admit before spawn");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));
    AssistantPlane::admit_turn_request(
        "research",
        "research-run",
        &provenance,
        &[ASSISTANT_RESEARCH_FETCH_FAMILY],
    )
    .expect("read-only fetch family on research");
    assert!(
        AssistantPlane::admit_turn_request(
            "propose",
            "charter",
            &provenance,
            &[ASSISTANT_RESEARCH_FETCH_FAMILY]
        )
        .is_err()
    );
    assert_eq!(projects.get_draft_seq(&draft_id).expect("seq"), 0);
}

#[test]
fn inferred_turn_registers_the_chain_not_the_echo() {
    let (_tmp, projects, _, assistant) = stores();
    let (draft_id, _) = projects.create_draft(b"payload", 10).expect("draft");
    let payload = json!({"text": "weekly report for my clients"});
    let provenance = owner_stated();
    let fetched = vec!["https://example.invalid/report-format".to_owned()];
    let objects = json!([
        {"object_kind": "business-brief", "fields": {
            "goal": {"value": "weekly client report", "provenance": {"kind": "owner-stated"}}
        }},
        {"object_kind": "research-run", "fields": {
            "format_reference": {"value": "one page, three sections", "provenance": {"kind": "sources", "sources": [{"uri": "https://example.invalid/report-format"}]}}
        }},
        {"object_kind": "charter", "fields": {
            "title": {"value": "Weekly client report", "provenance": {"kind": "owner-stated"}},
            "cadence": {"value": "weekly", "provenance": {"kind": "assistant-assumption"}}
        }}
    ]);
    let record = inference(&objects, &fetched, 2);
    let outcome = assistant
        .run_turn(&spec(
            "propose",
            &draft_id,
            "charter",
            &payload,
            &provenance,
            &[],
            &record,
        ))
        .expect("inferred propose");
    assert_eq!(outcome.provider_round_trips, 2);
    assert_eq!(outcome.model_id, "deepseek-chat");
    assert_eq!(outcome.inference_protocol, ASSISTANT_INFERENCE_PROTOCOL);
    assert_eq!(
        outcome.chain_object_kinds,
        ["business-brief", "research-run", "charter"].map(str::to_owned)
    );
    assert!(outcome.preview_id.is_some());
    assert_eq!(outcome.candidate_digest.len(), 64);

    let ops = &outcome.candidate_ops;
    let ops_bytes = serde_json::to_vec(ops).expect("ops bytes");
    assert_eq!(
        ProjectAggregateStore::digest_hex(&ops_bytes),
        outcome.candidate_digest,
        "registered digest binds exactly these ops"
    );
    assert_eq!(
        ops["chain"], objects,
        "candidate carries the inferred chain"
    );
    assert_eq!(
        ops["owner_payload"], payload,
        "owner input is labelled, not the candidate"
    );
    assert_eq!(ops["model_id"], "deepseek-chat");
    assert_eq!(ops["provider_round_trips"], 2);
    assert_eq!(ops["inference_protocol"], ASSISTANT_INFERENCE_PROTOCOL);
    assert_eq!(ops["inject_order_ref"], "CONTEXT_INJECT_ORDER");
    assert_eq!(ops["inject_order_layers"], CONTEXT_INJECT_ORDER.len());
    assert_eq!(
        ops["reply_digest"],
        ProjectAggregateStore::digest_hex(outcome.reply.as_bytes())
    );
    assert_eq!(ops["allowed_source_uris"], json!(fetched));
    assert!(ops.get("payload").is_none(), "no echo field survives");
    assert_eq!(
        projects.get_draft_seq(&draft_id).expect("seq"),
        0,
        "candidate is not applied"
    );

    let (explain_draft, _) = projects.create_draft(b"explain", 11).expect("draft");
    let brief = chain("business-brief");
    let brief_record = inference(&brief, &[], 1);
    let explain = assistant
        .run_turn(&spec(
            "explain",
            &explain_draft,
            "business-brief",
            &payload,
            &provenance,
            &[],
            &brief_record,
        ))
        .expect("explain");
    assert!(explain.preview_id.is_none(), "explain announces no preview");

    let error = assistant
        .run_turn(&spec(
            "navigate",
            &explain_draft,
            "axis",
            &payload,
            &provenance,
            &[],
            &brief_record,
        ))
        .expect_err("chain lacks requested object_kind");
    assert!(invalid_detail(error).contains("requested object_kind"));
}

#[test]
fn hyphenated_prose_registers_while_key_prefixed_tokens_stay_refused() {
    let (_tmp, projects, _, assistant) = stores();
    let (draft_id, _) = projects.create_draft(b"payload", 10).expect("draft");
    let payload = json!({"text": "summarise progress, risks and next steps"});
    let provenance = owner_stated();
    let prose_chain = json!([
        {"object_kind": "charter", "fields": {
            "risk_review": {"value": "risk-based weekly review at the desk-side stand-up; task-contract stays owner-confirmed", "provenance": {"kind": "assistant-assumption"}}
        }}
    ]);
    let record = inference(&prose_chain, &[], 1);
    assistant
        .run_turn(&spec(
            "propose",
            &draft_id,
            "charter",
            &payload,
            &provenance,
            &[],
            &record,
        ))
        .expect("hyphenated words such as risk-/desk-/task- are prose, not Provider material");
    assert_eq!(assistant.candidate_count(&draft_id).expect("count"), 1);

    let key_chain = json!([
        {"object_kind": "charter", "fields": {
            "note": {"value": "use sk-abcdefghijklmnopqrstuvwxyz for access", "provenance": {"kind": "assistant-assumption"}}
        }}
    ]);
    let key_record = inference(&key_chain, &[], 1);
    let error = assistant
        .run_turn(&spec(
            "propose",
            &draft_id,
            "charter",
            &payload,
            &provenance,
            &[],
            &key_record,
        ))
        .expect_err("a key-shaped token is still refused at registration");
    assert!(invalid_detail(error).contains("secret-shaped"));
    assert_eq!(assistant.candidate_count(&draft_id).expect("count"), 1);
}

#[test]
fn provider_unbound_guidance_is_a_settings_pointer_not_a_chat_box() {
    let guidance = provider_unbound_guidance();
    assert_eq!(guidance["status"], "provider_unbound");
    assert_eq!(guidance["settings_route"], ASSISTANT_SETTINGS_ROUTE);
    assert_eq!(guidance["chat_input"], false);
    assert_eq!(guidance["silent_bind"], false);
    assert_eq!(guidance["candidate_registered"], false);
    assert_eq!(guidance["installed_agent"], false);
    assert!(guidance.get("candidate_digest").is_none());
    assert!(guidance.get("preview_id").is_none());
    let rendered = guidance.to_string();
    assert!(!rendered.contains("Approve"));
    assert!(!rendered.to_ascii_lowercase().contains("api_key"));
    assert!(guidance["guidance"].as_str().unwrap().contains("Settings"));
}

#[test]
fn assistant_direct_writes_to_authority_secret_archive_memory_stay_refused() {
    let (_tmp, projects, employees, assistant) = stores();
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
    assert!(matches!(
        assistant
            .write_archive(&ArchiveAppendSpec {
                projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
                project_id: &project_id,
                employee_id: "employee-missing",
                kind: "note",
                body: "inferred reply must not land archive",
                now_ms: 20,
            })
            .expect_err("archive"),
        ProjectAggregateError::Forbidden { .. }
    ));
    assert!(matches!(
        assistant
            .write_secret("provider", "sk-not-a-real-key")
            .expect_err("secret"),
        ProjectAggregateError::Forbidden { .. }
    ));
    assert!(matches!(
        assistant.write_memory().expect_err("memory"),
        ProjectAggregateError::Forbidden { .. }
    ));
    assert!(matches!(
        assistant
            .grant_capability(
                &employees,
                &project_id,
                "employee-missing",
                "workspace-write",
                "project",
                21,
            )
            .expect_err("grant"),
        ProjectAggregateError::Forbidden { .. }
    ));
    assert!(matches!(
        assistant
            .confirm_preview(&preview_id, &preview_digest, 22)
            .expect_err("confirm"),
        ProjectAggregateError::Forbidden { .. }
    ));
    assert!(matches!(
        assistant
            .apply_candidate(&project_id, 0, "deadbeef", 23)
            .expect_err("apply onto authority"),
        ProjectAggregateError::Forbidden { .. } | ProjectAggregateError::Invalid { .. }
    ));
}
