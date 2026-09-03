//! Personal-private Project group chat routes (P13-T06).
//!
//! `POST /management/project/v1/chat.post` is the real caller of the group
//! conversation: the Owner's message is routed by the daemon (`@manager` →
//! PlanRevision candidate + `plan-revision` preview; `@member` → task-revision
//! candidate bounded to that Member's Task; un-addressed → manager-default
//! briefing), the turn lands in `p13_project_chat_turn`, manager speech lands
//! through the P11-T05 speech router, and any preview is only announced —
//! the Owner confirms it on the Projects canvas through the existing
//! `confirm` route. Approve-shaped bodies are 403 before any write;
//! secret-shaped bodies are 422 with a Settings pointer (SecretStore
//! takeover); cross-Project reads and routes are 403. Management-channel
//! only; task-channel aliases are 403.

use cognitive_store::{
    CHAT_THREAD_LIMIT, CONVERSATION_ARCHIVE_PROJECTION_ID, ChatTurnOutcome, ChatTurnSpec,
    ProjectAggregateError, ProjectChatStore, SqliteAuthorityStore, chat_secret_refusal_guidance,
};
use serde_json::{Value, json};

use super::project_aggregate::{error, now_ms, ok, parse_json, store_error};
use super::resource_api::ResourceApiResponse;

const ROUTE_LITERALS: &[&str] = &[
    "POST /management/project/v1/chat.post",
    "GET /management/project/v1/chat.thread",
    "POST /task/project/v1/chat.post",
    "GET /task/project/v1/chat.thread",
];

/// Body keys that would turn a chat message into an approval. The chat has no
/// Approve control; these are refused before the store is touched.
const APPROVE_SHAPED_KEYS: &[&str] = &[
    "approve",
    "approved",
    "approval",
    "confirm",
    "confirmed",
    "preview_digest",
    "decision",
    "accept",
    "dont_ask_again",
    "standing_policy",
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
        "PROJECT_CHAT_CHANNEL_FORBIDDEN",
        "Project group chat is management-channel only",
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
            "PROJECT_CHAT_ROUTE_NOT_FOUND",
            "no Project chat route matched",
        );
    };
    if channel == Channel::Task {
        return channel_forbidden();
    }
    let chat = ProjectChatStore::from_authority_store(store);
    match literal {
        "POST /management/project/v1/chat.post" => chat_post(body, &chat),
        "GET /management/project/v1/chat.thread" => chat_thread(method_path, &chat),
        _ => error(
            404,
            "PROJECT_CHAT_ROUTE_NOT_FOUND",
            "no Project chat route matched",
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

fn chat_post(body: &[u8], chat: &ProjectChatStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    if let Some(key) = approve_shaped_key(&document) {
        return ResourceApiResponse {
            status: 403,
            body: json!({
                "status": "error",
                "code": "CHAT_APPROVE_FORBIDDEN",
                "message": format!("chat has no Approve (`{key}` refused); confirm previews on the Projects canvas"),
                "posted": false,
                "chat_approve": false,
                "observation_only": true,
            })
            .to_string(),
            content_type: "application/json",
        };
    }
    let Some(project_id) = document.get("project_id").and_then(Value::as_str) else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    let caller_project_id = document
        .get("caller_project_id")
        .and_then(Value::as_str)
        .unwrap_or(project_id);
    let Some(text) = document.get("body").and_then(Value::as_str) else {
        return error(400, "CHAT_BODY_REQUIRED", "body required");
    };
    let mention = document
        .get("mention")
        .and_then(Value::as_str)
        .unwrap_or("none");
    let target_employee_id = document
        .get("target_employee_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty());
    let projection_id = document
        .get("projection_id")
        .and_then(Value::as_str)
        .unwrap_or(CONVERSATION_ARCHIVE_PROJECTION_ID);
    let proposal = document.get("proposal").filter(|value| !value.is_null());
    match chat.post_turn(&ChatTurnSpec {
        projection_id,
        caller_project_id,
        project_id,
        mention,
        target_employee_id,
        body: text,
        proposal,
        now_ms: now_ms(),
    }) {
        Ok(outcome) => ok(outcome_json(&outcome)),
        Err(ProjectAggregateError::Invalid { detail }) if detail.contains("secret-shaped") => {
            let mut response = json!({
                "status": "error",
                "code": "CHAT_SECRET_SHAPED_REFUSED",
                "message": detail,
            });
            merge(&mut response, chat_secret_refusal_guidance());
            response["status"] = json!("error");
            ResourceApiResponse {
                status: 422,
                body: response.to_string(),
                content_type: "application/json",
            }
        }
        Err(refusal) => store_error(refusal),
    }
}

fn outcome_json(outcome: &ChatTurnOutcome) -> Value {
    json!({
        "status": "ok",
        "projection_id": CONVERSATION_ARCHIVE_PROJECTION_ID,
        "project_id": outcome.project_id,
        "turn_id": outcome.turn_id,
        "mention": outcome.mention,
        "routing": outcome.routing,
        "target_employee_id": outcome.target_employee_id,
        "target_stage_id": outcome.target_stage_id,
        "candidate_registered": outcome.candidate_digest.is_some(),
        "candidate_kind": outcome.candidate_kind,
        "candidate_digest": outcome.candidate_digest,
        "preview_id": outcome.preview_id,
        "preview_is_announcement": true,
        "chat_approve": false,
        "reply": outcome.reply.as_ref().map(|reply| json!({
            "record_id": reply.record_id,
            "employee_id": reply.employee_id,
            "role": reply.role,
            "kind": reply.kind,
            "body": reply.body,
            "reason": reply.reason,
        })),
        "reply_reason": outcome.reply_reason,
        "created_at": outcome.created_at,
        "observation_only": true,
    })
}

fn chat_thread(method_path: &str, chat: &ProjectChatStore) -> ResourceApiResponse {
    let Some(project_id) = query_parameter(method_path, "project_id").filter(|v| !v.is_empty())
    else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    let caller_project_id = query_parameter(method_path, "caller_project_id")
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| project_id.clone());
    let Some(limit) = query_parameter(method_path, "limit")
        .filter(|v| !v.is_empty())
        .and_then(|value| value.parse::<u32>().ok())
    else {
        return error(
            422,
            "PROJECT_INVALID",
            "unbounded conversation resume rejected",
        );
    };
    match chat.read_thread(&caller_project_id, &project_id, limit) {
        Ok(thread) => ok(json!({
            "status": "ok",
            "projection_id": CONVERSATION_ARCHIVE_PROJECTION_ID,
            "project_id": thread.project_id,
            "observation_only": true,
            "chat_approve": false,
            "limit_ceiling": CHAT_THREAD_LIMIT,
            "truncated": thread.truncated,
            "participants": thread.participants.iter().map(|p| json!({
                "role": p.role,
                "employee_id": p.employee_id,
                "handle": p.handle,
                "state": p.state,
                "stage_ids": p.stage_ids,
            })).collect::<Vec<_>>(),
            "rows": thread.rows.iter().map(|row| json!({
                "row_id": row.row_id,
                "author": row.author,
                "employee_id": row.employee_id,
                "kind": row.kind,
                "body": row.body,
                "created_at": row.created_at,
                "turn_id": row.turn_id,
                "mention": row.mention,
                "routing": row.routing,
                "target_employee_id": row.target_employee_id,
                "target_stage_id": row.target_stage_id,
                "candidate_kind": row.candidate_kind,
                "candidate_digest": row.candidate_digest,
                "preview_id": row.preview_id,
                "reply_reason": row.reply_reason,
                "receipt_ref": row.receipt_ref,
                "applied_ref": row.applied_ref,
            })).collect::<Vec<_>>(),
        })),
        Err(refusal) => store_error(refusal),
    }
}

fn approve_shaped_key(document: &Value) -> Option<&'static str> {
    let object = document.as_object()?;
    APPROVE_SHAPED_KEYS
        .iter()
        .copied()
        .find(|key| object.contains_key(*key))
}

fn merge(target: &mut Value, extra: Value) {
    if let (Some(target), Some(extra)) = (target.as_object_mut(), extra.as_object()) {
        for (key, value) in extra {
            target.insert(key.clone(), value.clone());
        }
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use cognitive_store::{
        ConfirmCaller, EmployeeStore, PersonalDataLayout, ProjectAggregateStore, RosterProposal,
        StageSpec, prepare_personal_databases,
    };
    use tempfile::TempDir;

    fn authority() -> (TempDir, SqliteAuthorityStore) {
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
        (temporary, store)
    }

    fn body_json(response: &ResourceApiResponse) -> Value {
        serde_json::from_str(&response.body).expect("json body")
    }

    fn activate(store: &SqliteAuthorityStore) -> String {
        let projects = ProjectAggregateStore::from_authority_store(store);
        let (draft_id, _) = projects.create_draft(b"charter", 10).expect("draft");
        projects
            .put_draft_charter(&draft_id, b"charter-body", 11)
            .expect("charter");
        let (preview_id, digest) = projects
            .request_preview("activation", &draft_id, b"activation", 12)
            .expect("preview");
        projects
            .confirm_preview(ConfirmCaller::OwnerManagement, &preview_id, &digest, 13)
            .expect("G1")
            .new_ref
    }

    fn stage(id: &str, slot: &str) -> StageSpec {
        StageSpec {
            stage_id: id.to_owned(),
            title: id.to_owned(),
            objective: format!("{id} objective"),
            output_contract_digest: ProjectAggregateStore::digest_hex(
                format!("out-{id}").as_bytes(),
            ),
            acceptance_spec_ref: None,
            cadence_json: None,
            responsible_slot: slot.to_owned(),
            blocking_gap: None,
        }
    }

    /// Activated Project with a plan, a seated manager and a seated researcher.
    fn seated(store: &SqliteAuthorityStore) -> (String, Vec<String>) {
        let project_id = activate(store);
        let projects = ProjectAggregateStore::from_authority_store(store);
        let plan_id = projects
            .apply_plan_revision(
                &project_id,
                &project_id,
                &[stage("s1", "manager"), stage("s2", "researcher")],
                20,
            )
            .expect("plan");
        let employees = EmployeeStore::from_authority_store(store);
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
                        tools_declared: vec![],
                    },
                    RosterProposal {
                        slot: "researcher".to_owned(),
                        specialization: "member".to_owned(),
                        prompt: "research".to_owned(),
                        tools_declared: vec![],
                    },
                ],
                21,
            )
            .expect("roster");
        for (offset, id) in ids.iter().enumerate() {
            let base = 30 + offset as i64 * 2;
            employees
                .request_seating(ConfirmCaller::OwnerManagement, id, base)
                .expect("request");
            employees
                .confirm_seating(
                    ConfirmCaller::OwnerManagement,
                    id,
                    Some("flash"),
                    true,
                    base + 1,
                )
                .expect("seat");
        }
        (project_id, ids)
    }

    fn post(store: &SqliteAuthorityStore, body: Value) -> ResourceApiResponse {
        handle(
            "POST /management/project/v1/chat.post",
            body.to_string().as_bytes(),
            store,
        )
    }

    fn thread(store: &SqliteAuthorityStore, project_id: &str) -> Value {
        let response = handle(
            &format!("GET /management/project/v1/chat.thread?project_id={project_id}&limit=32"),
            b"",
            store,
        );
        assert_eq!(response.status, 200, "{}", response.body);
        body_json(&response)
    }

    #[test]
    fn task_channel_aliases_are_forbidden() {
        let (_tmp, store) = authority();
        for path in [
            "POST /task/project/v1/chat.post",
            "GET /task/project/v1/chat.thread?project_id=x&limit=1",
        ] {
            let response = handle(path, br#"{"project_id":"x","body":"hi"}"#, &store);
            assert_eq!(response.status, 403, "{path}");
            assert!(response.body.contains("PROJECT_CHAT_CHANNEL_FORBIDDEN"));
            assert!(is_task_channel(path));
        }
        assert!(matches("POST /management/project/v1/chat.post"));
        assert!(!matches("POST /management/project/v1/assistant.turn"));
    }

    #[test]
    fn approve_shaped_chat_bodies_are_refused_before_any_write() {
        let (_tmp, store) = authority();
        let (project_id, _ids) = seated(&store);
        for extra in [
            json!({ "approve": true }),
            json!({ "confirm": { "preview_id": "preview-x", "preview_digest": "y" } }),
            json!({ "preview_digest": "0000" }),
            json!({ "decision": "approve" }),
            json!({ "dont_ask_again": true }),
        ] {
            let mut body = json!({ "project_id": project_id, "body": "@manager approve this" });
            merge(&mut body, extra.clone());
            let response = post(&store, body);
            assert_eq!(response.status, 403, "{extra}: {}", response.body);
            let json = body_json(&response);
            assert_eq!(json["code"], json!("CHAT_APPROVE_FORBIDDEN"));
            assert_eq!(json["posted"], json!(false));
            assert_eq!(json["chat_approve"], json!(false));
        }
        let thread = thread(&store, &project_id);
        assert_eq!(thread["rows"].as_array().expect("rows").len(), 0);
    }

    #[test]
    fn secret_shaped_chat_is_refused_with_settings_pointer_and_never_stored() {
        let (_tmp, store) = authority();
        let (project_id, _ids) = seated(&store);
        let response = post(
            &store,
            json!({
                "project_id": project_id,
                "mention": "manager",
                "body": "@manager use api_key=sk-p13t06-http-fixture-not-a-key"
            }),
        );
        assert_eq!(response.status, 422, "{}", response.body);
        let json = body_json(&response);
        assert_eq!(json["code"], json!("CHAT_SECRET_SHAPED_REFUSED"));
        assert_eq!(json["settings_route"], json!("#/settings"));
        assert_eq!(json["posted"], json!(false));
        assert_eq!(json["archived"], json!(false));
        let projects = ProjectAggregateStore::from_authority_store(&store);
        assert!(!projects.leak_scan_contains("sk-p13t06").expect("scan"));
        let thread = thread(&store, &project_id);
        assert_eq!(thread["rows"].as_array().expect("rows").len(), 0);
    }

    #[test]
    fn cross_project_thread_read_and_member_route_are_forbidden() {
        let (_tmp, store) = authority();
        let (project_a, ids_a) = seated(&store);
        let (project_b, _ids_b) = seated(&store);
        let response = handle(
            &format!(
                "GET /management/project/v1/chat.thread?project_id={project_a}&caller_project_id={project_b}&limit=32"
            ),
            b"",
            &store,
        );
        assert_eq!(response.status, 403);
        assert!(response.body.contains("cross-scope"));
        let response = post(
            &store,
            json!({
                "project_id": project_b,
                "mention": "member",
                "target_employee_id": ids_a[1],
                "body": "@researcher come over"
            }),
        );
        assert_eq!(response.status, 403, "{}", response.body);
        assert!(response.body.contains("cross-project"));
        let response = post(
            &store,
            json!({
                "project_id": project_a,
                "caller_project_id": project_b,
                "body": "from b"
            }),
        );
        assert_eq!(response.status, 403, "{}", response.body);
        let unbounded = handle(
            &format!("GET /management/project/v1/chat.thread?project_id={project_a}"),
            b"",
            &store,
        );
        assert_eq!(unbounded.status, 422);
        let thread_a = thread(&store, &project_a);
        assert_eq!(thread_a["rows"].as_array().expect("rows").len(), 0);
    }

    #[test]
    fn manager_mention_registers_plan_revision_candidate_and_only_canvas_confirm_applies() {
        let (_tmp, store) = authority();
        let (project_id, ids) = seated(&store);
        let projects = ProjectAggregateStore::from_authority_store(&store);
        let before = projects
            .get_project(&project_id)
            .expect("get")
            .expect("row");
        let response = post(
            &store,
            json!({
                "project_id": project_id,
                "mention": "manager",
                "body": "@manager add a review ring after research",
                "proposal": {
                    "kind": "plan-revision",
                    "stages": [
                        { "stage_id": "s1", "title": "Manage", "objective": "coordinate", "responsible_slot": "manager" },
                        { "stage_id": "s2", "title": "Research", "objective": "collect", "responsible_slot": "researcher" },
                        { "stage_id": "s3", "title": "Review", "objective": "review the draft", "responsible_slot": "researcher" }
                    ]
                }
            }),
        );
        assert_eq!(response.status, 200, "{}", response.body);
        let json = body_json(&response);
        assert_eq!(json["routing"], json!("manager-plan-revision"));
        assert_eq!(json["candidate_registered"], json!(true));
        assert_eq!(json["candidate_kind"], json!("plan-revision"));
        assert_eq!(json["chat_approve"], json!(false));
        assert_eq!(json["preview_is_announcement"], json!(true));
        assert!(
            json.get("preview_digest").is_none(),
            "chat never carries a digest"
        );
        let preview_id = json["preview_id"].as_str().expect("preview id").to_owned();
        assert_eq!(json["reply"]["role"], json!("manager"));
        assert_eq!(json["reply"]["kind"], json!("announce"));
        assert_eq!(json["reply"]["employee_id"], json!(ids[0]));
        assert_eq!(json["reply_reason"], json!("manager-default"));
        assert!(
            json["reply"]["body"]
                .as_str()
                .expect("body")
                .contains("Chat cannot approve")
        );

        // Not a direct write.
        let mid = projects
            .get_project(&project_id)
            .expect("get")
            .expect("row");
        assert_eq!(
            mid.current_plan_revision_id,
            before.current_plan_revision_id
        );

        // The thread carries the owner turn, the manager announce, and no digest.
        let thread_json = thread(&store, &project_id);
        let rows = thread_json["rows"].as_array().expect("rows");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["author"], json!("owner"));
        assert_eq!(rows[0]["preview_id"], json!(preview_id));
        assert!(rows[0].get("preview_digest").is_none());
        assert_eq!(rows[1]["author"], json!("manager"));
        assert_eq!(rows[1]["kind"], json!("announce"));
        let participants = thread_json["participants"]
            .as_array()
            .expect("participants");
        assert!(participants.iter().any(|p| p["handle"] == json!("manager")));
        assert!(
            participants
                .iter()
                .any(|p| p["handle"] == json!("researcher"))
        );

        // Digest lives only on preview-detail; the canvas confirm applies.
        let detail = super::super::project_aggregate::handle(
            &format!("GET /management/project/v1/preview-detail?preview_id={preview_id}"),
            b"",
            &store,
        );
        assert_eq!(detail.status, 200, "{}", detail.body);
        let detail_json = body_json(&detail);
        assert_eq!(detail_json["subject_kind"], json!("plan-revision"));
        let digest = detail_json["preview_digest"]
            .as_str()
            .expect("digest")
            .to_owned();
        let confirm = super::super::project_aggregate::handle(
            "POST /management/project/v1/confirm",
            json!({ "preview_id": preview_id, "preview_digest": digest })
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(confirm.status, 200, "{}", confirm.body);
        let confirm_json = body_json(&confirm);
        assert_eq!(confirm_json["result"], json!("plan_revision_applied"));
        let new_plan = confirm_json["new_ref"]
            .as_str()
            .expect("new plan")
            .to_owned();
        let after = projects
            .get_project(&project_id)
            .expect("get")
            .expect("row");
        assert_eq!(
            after.current_plan_revision_id.as_deref(),
            Some(new_plan.as_str())
        );
        assert_eq!(projects.list_stages(&new_plan).expect("stages").len(), 3);
        let thread_json = thread(&store, &project_id);
        let owner_row = &thread_json["rows"][0];
        assert_eq!(owner_row["applied_ref"], json!(new_plan));
        assert!(
            owner_row["receipt_ref"]
                .as_str()
                .expect("receipt")
                .starts_with("receipt:chat:plan-revision:")
        );
    }

    #[test]
    fn member_mention_routes_only_that_members_task() {
        let (_tmp, store) = authority();
        let (project_id, ids) = seated(&store);
        let other = post(
            &store,
            json!({
                "project_id": project_id,
                "mention": "member",
                "target_employee_id": ids[1],
                "body": "@researcher take s1",
                "proposal": { "kind": "task-revision", "stage_id": "s1", "objective": "manage" }
            }),
        );
        assert_eq!(other.status, 403, "{}", other.body);
        assert!(other.body.contains("that Member"));
        let transfer = post(
            &store,
            json!({
                "project_id": project_id,
                "mention": "member",
                "target_employee_id": ids[1],
                "body": "@researcher be manager",
                "proposal": { "kind": "task-revision", "stage_id": "s2", "objective": "x", "is_current_manager": true }
            }),
        );
        assert_eq!(transfer.status, 403, "{}", transfer.body);
        let ok_response = post(
            &store,
            json!({
                "project_id": project_id,
                "mention": "member",
                "target_employee_id": ids[1],
                "body": "@researcher prioritise primary sources"
            }),
        );
        assert_eq!(ok_response.status, 200, "{}", ok_response.body);
        let json = body_json(&ok_response);
        assert_eq!(json["routing"], json!("member-task-revision"));
        assert_eq!(json["target_stage_id"], json!("s2"));
        assert_eq!(json["candidate_kind"], json!("task-revision"));
        assert!(
            json["reply"].is_null(),
            "the daemon does not fabricate Member prose"
        );
        assert_eq!(json["reply_reason"], json!("member-mentioned"));
        assert!(json["preview_id"].as_str().is_some());
    }
}
