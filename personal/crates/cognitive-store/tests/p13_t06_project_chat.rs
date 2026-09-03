#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P13-T06 Project group chat + manager / member routing.
//!
//! Failure-first negatives from the card: chat Approve, Member-to-Member
//! authority transfer, cross-Project read / route, secret in chat. Positive
//! path: `@manager` → daemon PlanRevision candidate → preview (never a direct
//! write) → canvas Confirm applies; `@member` → task-revision candidate bounded
//! to that Member's responsible stage; speech rules enforced by record kinds;
//! Conversation is never completion.

use cognitive_store::{
    ArchiveReadSpec, CHAT_THREAD_LIMIT, CONVERSATION_ARCHIVE_PROJECTION_ID,
    CONVERSATION_RESUME_LIMIT, ChatTurnSpec, ConfirmCaller, ConversationStore, EmployeeStore,
    PersonalDataLayout, ProjectAggregateError, ProjectAggregateStore, ProjectChatStore,
    RosterProposal, SpeechArchiveSpec, StageSpec, prepare_personal_databases,
};
use rusqlite::Connection;
use serde_json::{Value, json};
use tempfile::TempDir;

struct Stores {
    _tmp: TempDir,
    path: std::path::PathBuf,
    projects: ProjectAggregateStore,
    employees: EmployeeStore,
    conversations: ConversationStore,
    chat: ProjectChatStore,
}

fn stores() -> Stores {
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
    Stores {
        projects: ProjectAggregateStore::open_path(&path).expect("projects"),
        employees: EmployeeStore::open_path(&path).expect("employees"),
        conversations: ConversationStore::open_path(&path).expect("conversations"),
        chat: ProjectChatStore::open_path(&path).expect("chat"),
        path,
        _tmp: temporary,
    }
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
            tools_declared: vec![],
        },
        RosterProposal {
            slot: "researcher".to_owned(),
            specialization: "member".to_owned(),
            prompt: "research".to_owned(),
            tools_declared: vec![],
        },
    ]
}

/// Activated Project with a two-slot plan, seated manager (ids[0]) and seated
/// researcher (ids[1]).
fn seated_project(s: &Stores) -> (String, String, Vec<String>) {
    let project_id = activate(&s.projects);
    let plan_id = plan_two_slots(&s.projects, &project_id);
    let ids = s
        .employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_id,
            &proposals(),
            21,
        )
        .expect("roster");
    for (offset, id) in ids.iter().enumerate() {
        let base = 30 + offset as i64 * 2;
        s.employees
            .request_seating(ConfirmCaller::OwnerManagement, id, base)
            .expect("request");
        s.employees
            .confirm_seating(
                ConfirmCaller::OwnerManagement,
                id,
                Some("flash"),
                true,
                base + 1,
            )
            .expect("seat");
    }
    (project_id, plan_id, ids)
}

fn turn<'a>(
    project_id: &'a str,
    mention: &'a str,
    target: Option<&'a str>,
    body: &'a str,
    proposal: Option<&'a Value>,
    now_ms: i64,
) -> ChatTurnSpec<'a> {
    ChatTurnSpec {
        projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
        caller_project_id: project_id,
        project_id,
        mention,
        target_employee_id: target,
        body,
        proposal,
        now_ms,
    }
}

fn plan_proposal() -> Value {
    json!({
        "kind": "plan-revision",
        "stages": [
            { "stage_id": "s1", "title": "Manage", "objective": "coordinate the weekly report", "responsible_slot": "manager" },
            { "stage_id": "s2", "title": "Research", "objective": "collect the week's sources", "responsible_slot": "researcher" },
            { "stage_id": "s3", "title": "Review", "objective": "review the draft before it goes out", "responsible_slot": "researcher" }
        ]
    })
}

fn archive_index(s: &Stores, project_id: &str) -> Vec<String> {
    s.conversations
        .read_index(&ArchiveReadSpec {
            projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
            caller_project_id: project_id,
            target_project_id: project_id,
            employee_id: None,
            limit: CONVERSATION_RESUME_LIMIT,
            resume_from: None,
            include_bodies: false,
        })
        .expect("index")
        .records
        .into_iter()
        .map(|row| row.kind)
        .collect()
}

#[test]
fn p13_t06_chat_has_no_approve() {
    let s = stores();
    let (project_id, _plan, _ids) = seated_project(&s);
    let outcome = s
        .chat
        .post_turn(&turn(
            &project_id,
            "manager",
            None,
            "@manager please add a review ring",
            Some(&plan_proposal()),
            100,
        ))
        .expect("post");
    let preview_id = outcome.preview_id.clone().expect("preview announced");
    let detail = s
        .projects
        .preview_detail(&preview_id)
        .expect("detail")
        .expect("row");
    assert_eq!(detail.status, "pending");

    // The chat surface has no Approve verb: the store refuses it outright and
    // the preview stays pending.
    let refused = s
        .chat
        .approve_from_chat(&project_id, &preview_id, &detail.preview_digest, 101)
        .expect_err("chat approve");
    assert!(matches!(refused, ProjectAggregateError::Forbidden { .. }));
    assert!(format!("{refused}").contains("chat has no Approve"));
    // Neither the assistant nor the task channel can confirm the announced
    // preview; only owner management on the canvas may.
    for caller in [ConfirmCaller::Assistant, ConfirmCaller::TaskChannel] {
        let error = s
            .projects
            .confirm_preview(caller, &preview_id, &detail.preview_digest, 102)
            .expect_err("non-owner confirm");
        assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));
    }
    let still = s
        .projects
        .preview_detail(&preview_id)
        .expect("detail")
        .expect("row");
    assert_eq!(still.status, "pending");
    // An "approve" message is conversation, not authority: nothing changes.
    let chatter = s
        .chat
        .post_turn(&turn(
            &project_id,
            "none",
            None,
            "approve it, ship it",
            None,
            103,
        ))
        .expect("post");
    assert!(chatter.candidate_digest.is_none());
    assert!(chatter.preview_id.is_none());
    let unchanged = s
        .projects
        .preview_detail(&preview_id)
        .expect("detail")
        .expect("row");
    assert_eq!(unchanged.status, "pending");

    // The schema itself cannot record a chat Approve.
    let conn = Connection::open(&s.path).expect("open");
    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(p13_project_chat_turn)")
        .expect("pragma")
        .query_map([], |row| row.get::<_, String>(1))
        .expect("columns")
        .collect::<Result<Vec<_>, _>>()
        .expect("rows");
    assert!(columns.iter().any(|column| column == "approve_attempted"));
    let forced = conn.execute(
        "UPDATE p13_project_chat_turn SET approve_attempted = 1 WHERE project_id = ?1",
        [&project_id],
    );
    assert!(forced.is_err(), "CHECK approve_attempted = 0 must hold");
}

#[test]
fn p13_t06_same_timestamp_owner_turn_precedes_manager_reply() {
    let s = stores();
    let (project_id, _plan, _ids) = seated_project(&s);
    s.chat
        .post_turn(&turn(
            &project_id,
            "manager",
            None,
            "@manager where are we this week?",
            None,
            100,
        ))
        .expect("post");
    let thread = s
        .chat
        .read_thread(&project_id, &project_id, CHAT_THREAD_LIMIT)
        .expect("thread");
    assert_eq!(thread.rows.len(), 2, "{:?}", thread.rows);
    assert_eq!(thread.rows[0].author, "owner");
    assert_eq!(thread.rows[0].kind, "owner-message");
    assert_eq!(thread.rows[1].author, "manager");
    assert_eq!(thread.rows[1].kind, "announce");
    assert_eq!(thread.rows[0].created_at, thread.rows[1].created_at);
}

#[test]
fn p13_t06_chat_cannot_transfer_authority_between_members() {
    let s = stores();
    let (project_id, plan_id, ids) = seated_project(&s);
    let manager = &ids[0];
    let researcher = &ids[1];
    let grants_before = s.employees.grant_count(researcher).expect("grants");
    let manager_count_before = s
        .employees
        .current_manager_count(&project_id)
        .expect("count");

    for forbidden in [
        json!({ "kind": "task-revision", "stage_id": "s2", "objective": "hand this to the manager", "employee_id": manager }),
        json!({ "kind": "task-revision", "stage_id": "s2", "objective": "reassign", "responsible_slot": "manager" }),
        json!({ "kind": "task-revision", "stage_id": "s2", "objective": "escalate", "assignee": manager }),
        json!({ "kind": "task-revision", "stage_id": "s2", "objective": "grant me", "grant": { "capability_ref": "workspace-write" } }),
        json!({ "kind": "task-revision", "stage_id": "s2", "objective": "make me manager", "is_current_manager": true }),
    ] {
        let error = s
            .chat
            .post_turn(&turn(
                &project_id,
                "member",
                Some(researcher),
                "@researcher take over",
                Some(&forbidden),
                110,
            ))
            .expect_err("authority transfer via chat");
        assert!(
            matches!(
                error,
                ProjectAggregateError::Forbidden { .. } | ProjectAggregateError::Invalid { .. }
            ),
            "{forbidden}: {error}"
        );
    }
    // A plan-revision proposal cannot ride on @member either: that would let a
    // Member message reshape another Member's work.
    let error = s
        .chat
        .post_turn(&turn(
            &project_id,
            "member",
            Some(researcher),
            "@researcher rewrite the plan",
            Some(&plan_proposal()),
            111,
        ))
        .expect_err("plan through member");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));

    // The employee store's own guard is unchanged.
    let error = s
        .employees
        .apply_chat_authority_transfer("give researcher the manager seat")
        .expect_err("transfer");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));

    assert_eq!(
        s.employees.grant_count(researcher).expect("grants"),
        grants_before
    );
    assert_eq!(
        s.employees
            .current_manager_count(&project_id)
            .expect("count"),
        manager_count_before
    );
    let manager_row = s
        .employees
        .get_employee(manager)
        .expect("get")
        .expect("row");
    assert!(manager_row.is_current_manager);
    let researcher_row = s
        .employees
        .get_employee(researcher)
        .expect("get")
        .expect("row");
    assert!(!researcher_row.is_current_manager);
    let thread = s
        .chat
        .read_thread(&project_id, &project_id, CHAT_THREAD_LIMIT)
        .expect("thread");
    assert!(
        thread.rows.iter().all(|row| row.candidate_digest.is_none()),
        "no candidate may have been registered"
    );
    let stage = s
        .projects
        .get_stage(&plan_id, "s2")
        .expect("stage")
        .expect("row");
    assert_eq!(stage.responsible_slot, "researcher");
}

#[test]
fn p13_t06_cross_project_read_and_route_refused() {
    let s = stores();
    let (project_a, _plan_a, ids_a) = seated_project(&s);
    let (project_b, _plan_b, _ids_b) = seated_project(&s);
    s.chat
        .post_turn(&turn(&project_a, "none", None, "only in a", None, 120))
        .expect("post a");

    let cross_read = s
        .chat
        .read_thread(&project_b, &project_a, CHAT_THREAD_LIMIT)
        .expect_err("cross read");
    assert!(matches!(
        cross_read,
        ProjectAggregateError::Forbidden { .. }
    ));

    let mut cross_post = turn(&project_a, "none", None, "from b into a", None, 121);
    cross_post.caller_project_id = &project_b;
    let error = s.chat.post_turn(&cross_post).expect_err("cross post");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));

    // @member naming another Project's employee is a cross-Project route.
    let error = s
        .chat
        .post_turn(&turn(
            &project_b,
            "member",
            Some(&ids_a[1]),
            "@researcher help here",
            None,
            122,
        ))
        .expect_err("cross member");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));

    let b_thread = s
        .chat
        .read_thread(&project_b, &project_b, CHAT_THREAD_LIMIT)
        .expect("b thread");
    assert!(
        b_thread.rows.iter().all(|row| row.body != "only in a"),
        "project B must not see A's turn"
    );
    assert!(
        b_thread.rows.iter().all(|row| row.body != "from b into a"),
        "the refused cross post left nothing"
    );
    let a_thread = s
        .chat
        .read_thread(&project_a, &project_a, CHAT_THREAD_LIMIT)
        .expect("a thread");
    assert!(a_thread.rows.iter().any(|row| row.body == "only in a"));
    assert!(
        a_thread
            .rows
            .iter()
            .all(|row| row.body != "@researcher help here")
    );
}

#[test]
fn p13_t06_secret_in_chat_refused_and_never_persisted() {
    let s = stores();
    let (project_id, _plan, ids) = seated_project(&s);
    for body in [
        "api_key=sk-p13t06-fixture-not-a-real-key",
        "here is my token: Bearer abc.def.ghi",
        "x-api-key: whatever",
        "ssv1:opaque-material",
    ] {
        let error = s
            .chat
            .post_turn(&turn(&project_id, "manager", None, body, None, 130))
            .expect_err("secret body");
        assert!(
            matches!(error, ProjectAggregateError::Invalid { .. }),
            "{body}"
        );
        assert!(format!("{error}").contains("secret-shaped"));
    }
    // A secret hidden inside a structured proposal is refused the same way.
    let secret_proposal = json!({
        "kind": "task-revision",
        "stage_id": "s2",
        "objective": "use sk-p13t06-inside-proposal to fetch"
    });
    let error = s
        .chat
        .post_turn(&turn(
            &project_id,
            "member",
            Some(&ids[1]),
            "@researcher go",
            Some(&secret_proposal),
            131,
        ))
        .expect_err("secret proposal");
    assert!(matches!(error, ProjectAggregateError::Invalid { .. }));
    assert!(!s.projects.leak_scan_contains("sk-p13t06").expect("scan"));
    assert!(!s.projects.leak_scan_contains("ssv1:opaque").expect("scan"));
    let thread = s
        .chat
        .read_thread(&project_id, &project_id, CHAT_THREAD_LIMIT)
        .expect("thread");
    assert!(thread.rows.is_empty(), "nothing was posted or archived");
    assert!(archive_index(&s, &project_id).is_empty());
    // The refusal points the owner at Settings: keys enter through SecretStore
    // takeover, never through chat.
    assert_eq!(
        cognitive_store::chat_secret_refusal_guidance()["settings_route"],
        json!(cognitive_store::ASSISTANT_SETTINGS_ROUTE)
    );
}

#[test]
fn p13_t06_manager_mention_registers_plan_revision_candidate_then_canvas_confirm_applies() {
    let s = stores();
    // Bootstrap path: an activated Project has no plan yet (G1 mints none), so
    // the first PlanRevision must arrive through this product path.
    let project_id = activate(&s.projects);
    let before = s
        .projects
        .get_project(&project_id)
        .expect("get")
        .expect("row");
    assert!(before.current_plan_revision_id.is_none());

    let outcome = s
        .chat
        .post_turn(&turn(
            &project_id,
            "manager",
            None,
            "@manager set up the weekly report plan",
            Some(&plan_proposal()),
            140,
        ))
        .expect("post");
    assert_eq!(outcome.routing, "manager-plan-revision");
    assert_eq!(outcome.candidate_kind.as_deref(), Some("plan-revision"));
    let digest = outcome.candidate_digest.clone().expect("digest");
    assert_eq!(digest.len(), 64);
    let preview_id = outcome.preview_id.clone().expect("preview");
    // No manager is seated yet, so nobody speaks; the daemon says so.
    assert!(outcome.reply.is_none());
    assert_eq!(outcome.reply_reason, "no-current-manager");

    // Not a direct write: the Project still has no plan.
    let mid = s
        .projects
        .get_project(&project_id)
        .expect("get")
        .expect("row");
    assert!(mid.current_plan_revision_id.is_none());
    let pending = s
        .projects
        .list_pending_previews(&outcome.turn_id)
        .expect("pending");
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].subject_kind, "plan-revision");

    // Owner confirms on the canvas (digest-bound) → the daemon applies the
    // PlanRevision and the receipt returns to the conversation.
    let detail = s
        .projects
        .preview_detail(&preview_id)
        .expect("detail")
        .expect("row");
    let wrong = s
        .projects
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &"0".repeat(64),
            141,
        )
        .expect_err("wrong digest");
    assert!(matches!(wrong, ProjectAggregateError::Stale { .. }));
    let result = s
        .projects
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &detail.preview_digest,
            142,
        )
        .expect("confirm");
    assert_eq!(result.kind, "plan_revision_applied");
    let after = s
        .projects
        .get_project(&project_id)
        .expect("get")
        .expect("row");
    assert_eq!(
        after.current_plan_revision_id.as_deref(),
        Some(result.new_ref.as_str())
    );
    let stages = s.projects.list_stages(&result.new_ref).expect("stages");
    assert_eq!(
        stages
            .iter()
            .map(|row| row.stage_id.as_str())
            .collect::<Vec<_>>(),
        ["s1", "s2", "s3"]
    );
    assert_eq!(stages[2].responsible_slot, "researcher");
    assert!(stages.iter().all(|row| row.confirm_status == "unconfirmed"));
    let consumed = s
        .projects
        .preview_detail(&preview_id)
        .expect("detail")
        .expect("row");
    assert_eq!(consumed.status, "consumed");
    let thread = s
        .chat
        .read_thread(&project_id, &project_id, CHAT_THREAD_LIMIT)
        .expect("thread");
    let owner_turn = thread
        .rows
        .iter()
        .find(|row| row.turn_id.as_deref() == Some(outcome.turn_id.as_str()))
        .expect("turn row");
    assert_eq!(
        owner_turn.receipt_ref.as_deref(),
        Some(result.receipt_ref.as_str())
    );
    assert_eq!(
        owner_turn.applied_ref.as_deref(),
        Some(result.new_ref.as_str())
    );
    // Confirming twice is refused: the preview is consumed.
    let again = s
        .projects
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &detail.preview_digest,
            143,
        )
        .expect_err("consumed");
    assert!(matches!(again, ProjectAggregateError::Invalid { .. }));

    // The roster can now register against the applied plan (the P13-T02 gap).
    let ids = s
        .employees
        .register_roster(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &result.new_ref,
            &proposals(),
            150,
        )
        .expect("roster");
    assert_eq!(ids.len(), 2);

    // A second plan-revision candidate goes stale when the plan moves under it.
    let second = s
        .chat
        .post_turn(&turn(
            &project_id,
            "manager",
            None,
            "@manager drop the review ring",
            Some(&json!({
                "kind": "plan-revision",
                "stages": [
                    { "stage_id": "s1", "title": "Manage", "objective": "coordinate", "responsible_slot": "manager" },
                    { "stage_id": "s2", "title": "Research", "objective": "collect", "responsible_slot": "researcher" }
                ]
            })),
            151,
        ))
        .expect("second");
    let second_preview = second.preview_id.expect("preview");
    s.projects
        .apply_plan_revision(
            &project_id,
            &project_id,
            &[stage("s1", "Manage", "manager")],
            152,
        )
        .expect("plan moved");
    let second_detail = s
        .projects
        .preview_detail(&second_preview)
        .expect("detail")
        .expect("row");
    let stale = s
        .projects
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &second_preview,
            &second_detail.preview_digest,
            153,
        )
        .expect_err("stale");
    assert!(matches!(stale, ProjectAggregateError::Stale { .. }));
}

#[test]
fn p13_t06_member_mention_routes_only_that_members_task() {
    let s = stores();
    let (project_id, plan_id, ids) = seated_project(&s);
    let manager = &ids[0];
    let researcher = &ids[1];
    // Confirm the manager's ring so we can prove it survives a member revision.
    let ring = s
        .projects
        .get_stage(&plan_id, "s1")
        .expect("stage")
        .expect("row");
    s.projects
        .confirm_stage(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_id,
            "s1",
            &ring.stage_digest,
        )
        .expect("confirm s1");

    // @member may not redirect a stage that is not that Member's Task.
    let error = s
        .chat
        .post_turn(&turn(
            &project_id,
            "member",
            Some(researcher),
            "@researcher take the manage ring too",
            Some(&json!({ "kind": "task-revision", "stage_id": "s1", "objective": "manage" })),
            160,
        ))
        .expect_err("other member's task");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));
    assert!(format!("{error}").contains("that Member"));
    // @member needs a Member; the manager is addressed with @manager.
    let error = s
        .chat
        .post_turn(&turn(
            &project_id,
            "member",
            Some(manager),
            "@manager as member",
            None,
            161,
        ))
        .expect_err("manager via @member");
    assert!(matches!(error, ProjectAggregateError::Invalid { .. }));
    // @member without a target is not routable.
    let error = s
        .chat
        .post_turn(&turn(&project_id, "member", None, "@someone", None, 162))
        .expect_err("no target");
    assert!(matches!(error, ProjectAggregateError::Invalid { .. }));

    // Plain @member text is a bounded redirect of that Member's one Task.
    let outcome = s
        .chat
        .post_turn(&turn(
            &project_id,
            "member",
            Some(researcher),
            "@researcher focus on primary sources this week",
            None,
            163,
        ))
        .expect("post");
    assert_eq!(outcome.routing, "member-task-revision");
    assert_eq!(outcome.candidate_kind.as_deref(), Some("task-revision"));
    assert_eq!(
        outcome.target_employee_id.as_deref(),
        Some(researcher.as_str())
    );
    assert_eq!(outcome.target_stage_id.as_deref(), Some("s2"));
    // The mentioned Member did not speak: the daemon does not fabricate
    // Member prose; the manager was not addressed either.
    assert!(outcome.reply.is_none());
    assert_eq!(outcome.reply_reason, "member-mentioned");
    let preview_id = outcome.preview_id.clone().expect("preview");
    // Not a direct write.
    let mid = s
        .projects
        .get_project(&project_id)
        .expect("get")
        .expect("row");
    assert_eq!(
        mid.current_plan_revision_id.as_deref(),
        Some(plan_id.as_str())
    );

    let detail = s
        .projects
        .preview_detail(&preview_id)
        .expect("detail")
        .expect("row");
    assert_eq!(detail.subject_kind, "task-revision");
    let result = s
        .projects
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &detail.preview_digest,
            164,
        )
        .expect("confirm");
    assert_eq!(result.kind, "task_revision_applied");
    let new_plan = result.new_ref.clone();
    assert_ne!(new_plan, plan_id);
    let stages = s.projects.list_stages(&new_plan).expect("stages");
    assert_eq!(stages.len(), 2);
    let s1 = stages.iter().find(|row| row.stage_id == "s1").expect("s1");
    let s2 = stages.iter().find(|row| row.stage_id == "s2").expect("s2");
    // Only the researcher's ring changed; the manager's confirmed ring is kept.
    assert_eq!(s1.confirm_status, "confirmed");
    assert_eq!(s1.objective, "Manage objective");
    assert_eq!(s2.confirm_status, "unconfirmed");
    assert!(s2.objective.contains("focus on primary sources this week"));
    assert!(s2.objective.contains("Research objective"));
    assert_eq!(s2.responsible_slot, "researcher");
    // Seating carries forward for unchanged slots: the revision is not a
    // shadow plan that silently unseats the roster.
    assert!(
        s.employees
            .stage_is_seated(&project_id, &new_plan, "s2")
            .expect("seated")
    );
    assert!(
        s.employees
            .stage_is_seated(&project_id, &new_plan, "s1")
            .expect("seated")
    );
    let researcher_row = s
        .employees
        .get_employee(researcher)
        .expect("get")
        .expect("row");
    assert!(!researcher_row.is_current_manager);
    assert_eq!(
        s.employees.grant_count(researcher).expect("grants"),
        0,
        "a task revision grants nothing"
    );
}

#[test]
fn p13_t06_speech_rules_are_record_kinds() {
    let s = stores();
    let (project_id, _plan, ids) = seated_project(&s);
    let manager = &ids[0];
    let researcher = &ids[1];

    // An un-addressed owner message goes to the manager by default; the
    // manager's briefing is a daemon `announce` record landed through the
    // speech router with reason `manager-default`.
    let outcome = s
        .chat
        .post_turn(&turn(
            &project_id,
            "none",
            None,
            "where are we this week?",
            None,
            170,
        ))
        .expect("post");
    assert_eq!(outcome.routing, "manager-briefing");
    let reply = outcome.reply.expect("manager speaks by default");
    assert_eq!(reply.employee_id, *manager);
    assert_eq!(reply.role, "manager");
    assert_eq!(reply.kind, "announce");
    assert_eq!(outcome.reply_reason, "manager-default");
    assert!(reply.body.contains("Observed now"));
    assert!(reply.body.to_ascii_lowercase().contains("cannot approve"));

    // Explicit @manager briefing is the same manager-default record kind.
    let outcome = s
        .chat
        .post_turn(&turn(
            &project_id,
            "manager",
            None,
            "@manager status?",
            None,
            171,
        ))
        .expect("post");
    assert_eq!(outcome.routing, "manager-briefing");
    assert_eq!(outcome.reply.expect("reply").kind, "announce");

    // Member proactive speech: chatter is audit-only; whitelist kinds land;
    // a mentioned Member's note lands.
    let chatter = s
        .conversations
        .land_speech(
            &s.employees,
            &SpeechArchiveSpec {
                projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
                project_id: &project_id,
                employee_id: researcher,
                kind: "chatter",
                mentioned: false,
                body: "just thinking aloud",
                now_ms: 172,
            },
        )
        .expect("chatter");
    assert!(!chatter.delivered);
    assert_eq!(chatter.reason, "speech-filtered");
    let deliverable = s
        .conversations
        .land_speech(
            &s.employees,
            &SpeechArchiveSpec {
                projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
                project_id: &project_id,
                employee_id: researcher,
                kind: "deliverable",
                mentioned: false,
                body: "sources.md is ready",
                now_ms: 173,
            },
        )
        .expect("deliverable");
    assert!(deliverable.delivered);
    let blocked = s
        .conversations
        .land_speech(
            &s.employees,
            &SpeechArchiveSpec {
                projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
                project_id: &project_id,
                employee_id: researcher,
                kind: "blocked",
                mentioned: false,
                body: "need the archive export",
                now_ms: 174,
            },
        )
        .expect("blocked");
    assert!(blocked.delivered);
    let note_unmentioned = s
        .conversations
        .land_speech(
            &s.employees,
            &SpeechArchiveSpec {
                projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
                project_id: &project_id,
                employee_id: researcher,
                kind: "note",
                mentioned: false,
                body: "by the way",
                now_ms: 175,
            },
        )
        .expect("note");
    assert!(!note_unmentioned.delivered);
    let note_mentioned = s
        .conversations
        .land_speech(
            &s.employees,
            &SpeechArchiveSpec {
                projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
                project_id: &project_id,
                employee_id: researcher,
                kind: "note",
                mentioned: true,
                body: "answering the owner",
                now_ms: 176,
            },
        )
        .expect("note");
    assert!(note_mentioned.delivered);
    assert_eq!(note_mentioned.reason, "mentioned");

    // The thread merges owner turns and delivered speech in time order and
    // never shows filtered chatter.
    let thread = s
        .chat
        .read_thread(&project_id, &project_id, CHAT_THREAD_LIMIT)
        .expect("thread");
    let bodies: Vec<&str> = thread.rows.iter().map(|row| row.body.as_str()).collect();
    assert!(bodies.contains(&"where are we this week?"));
    assert!(bodies.contains(&"sources.md is ready"));
    assert!(bodies.contains(&"need the archive export"));
    assert!(bodies.contains(&"answering the owner"));
    assert!(!bodies.contains(&"just thinking aloud"));
    assert!(!bodies.contains(&"by the way"));
    let created: Vec<i64> = thread.rows.iter().map(|row| row.created_at).collect();
    let mut sorted = created.clone();
    sorted.sort_unstable();
    assert_eq!(created, sorted);
    let announce_rows = thread
        .rows
        .iter()
        .filter(|row| row.author == "manager" && row.kind == "announce")
        .count();
    assert_eq!(announce_rows, 2);
    assert!(
        thread
            .rows
            .iter()
            .any(|row| row.author == "member" && row.kind == "deliverable")
    );
    // Participants: Owner, the manager (handle `manager`), the researcher
    // (handle = slot).
    assert!(thread.participants.iter().any(|p| p.role == "owner"));
    let manager_p = thread
        .participants
        .iter()
        .find(|p| p.role == "manager")
        .expect("manager participant");
    assert_eq!(manager_p.handle, "manager");
    assert_eq!(manager_p.employee_id.as_deref(), Some(manager.as_str()));
    let member_p = thread
        .participants
        .iter()
        .find(|p| p.role == "member")
        .expect("member participant");
    assert_eq!(member_p.handle, "researcher");
    assert_eq!(member_p.stage_ids, vec!["s2".to_owned()]);
    assert_eq!(archive_index(&s, &project_id).len(), 5);
}

#[test]
fn p13_t06_conversation_is_not_completion_and_reads_are_bounded() {
    let s = stores();
    let (project_id, plan_id, ids) = seated_project(&s);
    let before = s
        .projects
        .get_project(&project_id)
        .expect("get")
        .expect("row");
    s.chat
        .post_turn(&turn(&project_id, "none", None, "done!", None, 180))
        .expect("post");
    s.conversations
        .land_speech(
            &s.employees,
            &SpeechArchiveSpec {
                projection_id: CONVERSATION_ARCHIVE_PROJECTION_ID,
                project_id: &project_id,
                employee_id: &ids[1],
                kind: "deliverable",
                mentioned: false,
                body: "all finished, accept it",
                now_ms: 181,
            },
        )
        .expect("deliverable");
    let after = s
        .projects
        .get_project(&project_id)
        .expect("get")
        .expect("row");
    assert_eq!(after.state, before.state);
    assert_eq!(after.accepted_at, before.accepted_at);
    assert_eq!(
        after.current_plan_revision_id,
        before.current_plan_revision_id
    );
    let s2 = s
        .projects
        .get_stage(&plan_id, "s2")
        .expect("stage")
        .expect("row");
    assert_eq!(s2.confirm_status, "unconfirmed");
    assert!(!s2.ready);

    for bad in [0_u32, CHAT_THREAD_LIMIT + 1] {
        let error = s
            .chat
            .read_thread(&project_id, &project_id, bad)
            .expect_err("bounded");
        assert!(matches!(error, ProjectAggregateError::Invalid { .. }));
    }
    let oversize = "x".repeat(cognitive_store::CHAT_BODY_LIMIT + 1);
    let error = s
        .chat
        .post_turn(&turn(&project_id, "none", None, &oversize, None, 182))
        .expect_err("oversize");
    assert!(matches!(error, ProjectAggregateError::Invalid { .. }));
    let legacy = ChatTurnSpec {
        projection_id: cognitive_store::LEGACY_CONVERSATION_PROJECTION_ID,
        caller_project_id: &project_id,
        project_id: &project_id,
        mention: "none",
        target_employee_id: None,
        body: "old client",
        proposal: None,
        now_ms: 183,
    };
    let error = s.chat.post_turn(&legacy).expect_err("legacy projection");
    assert!(matches!(error, ProjectAggregateError::Invalid { .. }));
}
