//! Personal-private `/management/project/v1/*` projection (P11-T03).
//!
//! Not P7-T05 frozen inventory. Empty/unavailable when there is no authority.
//! Task-channel writes are 403 (N12).

use cognitive_store::{
    ConfirmCaller, PendingPreviewRow, ProjectAggregateError, ProjectAggregateStore, ProjectRow,
    SqliteAuthorityStore,
};
use serde_json::{Value, json};

use super::resource_api::ResourceApiResponse;

const ROUTE_LITERALS: &[&str] = &[
    "GET /management/project/v1/list",
    "GET /management/project/v1/detail",
    "GET /management/project/v1/axis",
    "GET /management/project/v1/roster",
    "GET /management/project/v1/pending-previews",
    "GET /management/project/v1/preview-detail",
    "POST /management/project/v1/draft.apply",
    "POST /management/project/v1/preview.request",
    "POST /management/project/v1/confirm",
    "GET /task/project/v1/list",
    "POST /task/project/v1/draft.apply",
    "POST /task/project/v1/preview.request",
    "POST /task/project/v1/confirm",
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
        "PROJECT_AGGREGATE_CHANNEL_FORBIDDEN",
        "Project aggregate operations are management-channel only",
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
            "PROJECT_AGGREGATE_ROUTE_NOT_FOUND",
            "no Project aggregate route matched",
        );
    };
    if channel == Channel::Task {
        return channel_forbidden();
    }
    let plane = ProjectAggregateStore::from_authority_store(store);
    match literal {
        "GET /management/project/v1/list" => list_projects(method_path, &plane),
        "GET /management/project/v1/detail" => detail(method_path, &plane),
        "GET /management/project/v1/axis" => axis(method_path, &plane),
        "GET /management/project/v1/roster" => roster(method_path, &plane),
        "GET /management/project/v1/pending-previews" => pending_previews(method_path, &plane),
        "GET /management/project/v1/preview-detail" => preview_detail(method_path, &plane),
        "POST /management/project/v1/draft.apply" => draft_apply(body, &plane),
        "POST /management/project/v1/preview.request" => preview_request(body, &plane),
        "POST /management/project/v1/confirm" => confirm(body, &plane),
        _ => error(
            404,
            "PROJECT_AGGREGATE_ROUTE_NOT_FOUND",
            "no Project aggregate route matched",
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

fn list_projects(method_path: &str, plane: &ProjectAggregateStore) -> ResourceApiResponse {
    let limit = query_parameter(method_path, "limit")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(32);
    match plane.list_projects(limit) {
        Ok(projects) => ok(json!({
            "status": "ok",
            "projection": "personal-private",
            "projects": projects.iter().map(project_list_json).collect::<Vec<_>>(),
        })),
        Err(error) => store_error(error),
    }
}

fn project_list_json(project: &ProjectRow) -> Value {
    json!({
        "project_id": project.project_id,
        "state": project.state,
        "title_summary": "unknown",
        "created_at": project.created_at,
        "activated_at": project.activated_at,
        "accepted_at": project.accepted_at,
        "cost": "unknown",
    })
}

fn detail(method_path: &str, plane: &ProjectAggregateStore) -> ResourceApiResponse {
    let Some(project_id) = query_parameter(method_path, "project_id").filter(|v| !v.is_empty())
    else {
        return error(400, "PROJECT_ID_REQUIRED", "detail requires project_id");
    };
    if project_id.starts_with("task://") {
        return error(
            404,
            "PROJECT_NOT_FOUND",
            "Task ref cannot be resolved as a Project",
        );
    }
    match plane.get_project(&project_id) {
        Ok(None) => error(404, "PROJECT_NOT_FOUND", "project not found"),
        Ok(Some(project)) => match plane.get_charter(&project.current_charter_revision_id) {
            Ok(charter) => {
                let pending = plane
                    .list_pending_previews(&project_id)
                    .map(|rows| rows.len())
                    .unwrap_or(0);
                let plan = project.current_plan_revision_id.as_ref().map(|plan_id| {
                    json!({
                        "plan_revision_id": plan_id,
                    })
                });
                ok(json!({
                    "status": "ok",
                    "projection": "personal-private",
                    "project": {
                        "project_id": project.project_id,
                        "state": project.state,
                        "created_at": project.created_at,
                        "activated_at": project.activated_at,
                        "accepted_at": project.accepted_at,
                    },
                    "charter": charter.map(|row| json!({
                        "charter_revision_id": row.charter_revision_id,
                        "seq": row.seq,
                        "status": row.status,
                        "content_digest": row.content_digest,
                        "confirmed_at": row.confirmed_at,
                    })),
                    "plan": plan,
                    "pending_preview_count": pending,
                    "cost": ProjectAggregateStore::unknown_cost_projection(),
                }))
            }
            Err(error) => store_error(error),
        },
        Err(error) => store_error(error),
    }
}

fn axis(method_path: &str, plane: &ProjectAggregateStore) -> ResourceApiResponse {
    let Some(project_id) = query_parameter(method_path, "project_id").filter(|v| !v.is_empty())
    else {
        return error(400, "PROJECT_ID_REQUIRED", "axis requires project_id");
    };
    match plane.get_project(&project_id) {
        Ok(None) => error(404, "PROJECT_NOT_FOUND", "project not found"),
        Ok(Some(project)) => {
            let Some(plan_id) = query_parameter(method_path, "plan_revision_id")
                .filter(|value| !value.is_empty())
                .or(project.current_plan_revision_id)
            else {
                return ok(json!({
                    "status": "ok",
                    "projection": "personal-private",
                    "plan_revision_id": Value::Null,
                    "stages": [],
                }));
            };
            match plane.list_stages(&plan_id) {
                Ok(stages) => ok(json!({
                    "status": "ok",
                    "projection": "personal-private",
                    "plan_revision_id": plan_id,
                    "stages": stages.iter().map(|stage| {
                        json!({
                            "stage_id": stage.stage_id,
                            "position": stage.position,
                            "title": stage.title,
                            "objective": stage.objective,
                            "confirm_status": stage.confirm_status,
                            "ready": stage.ready,
                            "stage_digest": stage.stage_digest,
                            "output_contract": {
                                "digest": stage.output_contract_digest,
                                "deliverable_type": "unknown",
                                "save_format": "unknown",
                                "open_with": "unknown",
                            },
                            "acceptance_spec_present": stage.acceptance_spec_ref.is_some(),
                            "responsible_slot": stage.responsible_slot,
                            "seated": false,
                            "cadence_json": stage.cadence_json,
                            "gaps": plane.list_gaps(&plan_id, &stage.stage_id).unwrap_or_else(|_| Vec::new()).iter().map(|gap| json!({
                                "gap_id": gap.gap_id,
                                "blocking": gap.blocking,
                                "description": gap.description,
                                "accepted_as_limitation": gap.accepted_as_limitation,
                            })).collect::<Vec<_>>(),
                        })
                    }).collect::<Vec<_>>(),
                })),
                Err(error) => store_error(error),
            }
        }
        Err(error) => store_error(error),
    }
}

fn roster(method_path: &str, plane: &ProjectAggregateStore) -> ResourceApiResponse {
    let Some(project_id) = query_parameter(method_path, "project_id").filter(|v| !v.is_empty())
    else {
        return error(400, "PROJECT_ID_REQUIRED", "roster requires project_id");
    };
    match plane.get_project(&project_id) {
        Ok(None) => error(404, "PROJECT_NOT_FOUND", "project not found"),
        Ok(Some(_)) => ok(json!({
            "status": "ok",
            "projection": "personal-private",
            "roster": [],
            "authority_note": "employee-authority-not-implemented",
        })),
        Err(error) => store_error(error),
    }
}

fn pending_previews(method_path: &str, plane: &ProjectAggregateStore) -> ResourceApiResponse {
    let Some(subject_ref) = query_parameter(method_path, "subject_ref").filter(|v| !v.is_empty())
    else {
        return error(
            400,
            "SUBJECT_REF_REQUIRED",
            "pending-previews requires subject_ref",
        );
    };
    match plane.list_pending_previews(&subject_ref) {
        Ok(rows) => ok(json!({
            "status": "ok",
            "projection": "personal-private",
            "previews": rows.iter().map(pending_json).collect::<Vec<_>>(),
        })),
        Err(error) => store_error(error),
    }
}

fn pending_json(row: &PendingPreviewRow) -> Value {
    json!({
        "preview_id": row.preview_id,
        "subject_kind": row.subject_kind,
        "subject_ref": row.subject_ref,
        "status": row.status,
        "created_at": row.created_at,
    })
}

fn preview_detail(method_path: &str, plane: &ProjectAggregateStore) -> ResourceApiResponse {
    let Some(preview_id) = query_parameter(method_path, "preview_id").filter(|v| !v.is_empty())
    else {
        return error(
            400,
            "PREVIEW_ID_REQUIRED",
            "preview-detail requires preview_id",
        );
    };
    match plane.preview_detail(&preview_id) {
        Ok(None) => error(404, "PREVIEW_NOT_FOUND", "preview not found"),
        Ok(Some(detail)) => ok(json!({
            "status": "ok",
            "projection": "personal-private",
            "preview_id": detail.preview_id,
            "subject_kind": detail.subject_kind,
            "base_state_digest": detail.base_state_digest,
            "preview_digest": detail.preview_digest,
            "preview_bytes_ref": detail.preview_bytes_ref,
            "status": detail.status,
        })),
        Err(error) => store_error(error),
    }
}

fn draft_apply(body: &[u8], plane: &ProjectAggregateStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(draft_id) = document.get("draft_id").and_then(Value::as_str) else {
        return error(400, "DRAFT_ID_REQUIRED", "draft_id required");
    };
    let Some(base_seq) = document.get("base_seq").and_then(Value::as_i64) else {
        return error(400, "BASE_SEQ_REQUIRED", "base_seq required");
    };
    let Some(candidate_digest) = document.get("candidate_digest").and_then(Value::as_str) else {
        return error(
            400,
            "CANDIDATE_DIGEST_REQUIRED",
            "candidate_digest required",
        );
    };
    match plane.apply_candidate(
        ConfirmCaller::OwnerManagement,
        draft_id,
        base_seq,
        candidate_digest,
        now_ms(),
    ) {
        Ok((new_base_seq, payload_digest)) => ok(json!({
            "status": "ok",
            "new_base_seq": new_base_seq,
            "payload_digest": payload_digest,
        })),
        Err(error) => store_error(error),
    }
}

fn preview_request(body: &[u8], plane: &ProjectAggregateStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(subject_kind) = document.get("subject_kind").and_then(Value::as_str) else {
        return error(400, "SUBJECT_KIND_REQUIRED", "subject_kind required");
    };
    let Some(subject_ref) = document.get("subject_ref").and_then(Value::as_str) else {
        return error(400, "SUBJECT_REF_REQUIRED", "subject_ref required");
    };
    let preview_bytes = format!("{subject_kind}\n{subject_ref}").into_bytes();
    match plane.request_preview(subject_kind, subject_ref, &preview_bytes, now_ms()) {
        Ok((preview_id, _)) => ok(json!({
            "status": "ok",
            "preview_id": preview_id,
            "created_at": now_ms(),
        })),
        Err(error) => store_error(error),
    }
}

fn confirm(body: &[u8], plane: &ProjectAggregateStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(preview_id) = document.get("preview_id").and_then(Value::as_str) else {
        return error(400, "PREVIEW_ID_REQUIRED", "preview_id required");
    };
    let Some(preview_digest) = document.get("preview_digest").and_then(Value::as_str) else {
        return error(400, "PREVIEW_DIGEST_REQUIRED", "preview_digest required");
    };
    match plane.confirm_preview(
        ConfirmCaller::OwnerManagement,
        preview_id,
        preview_digest,
        now_ms(),
    ) {
        Ok(result) => ok(json!({
            "status": "ok",
            "receipt_ref": result.receipt_ref,
            "result": result.kind,
            "new_ref": result.new_ref,
        })),
        Err(error) => store_error(error),
    }
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
            return Some(value.trim().to_owned());
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

fn error(status: u16, code: &str, message: &str) -> ResourceApiResponse {
    ResourceApiResponse {
        status,
        body: json!({"status":"error","code": code, "message": message}).to_string(),
        content_type: "application/json",
    }
}

fn store_error(err: ProjectAggregateError) -> ResourceApiResponse {
    match err {
        ProjectAggregateError::Forbidden { detail } => error(403, "PROJECT_FORBIDDEN", detail),
        ProjectAggregateError::NotFound { detail } => error(404, "PROJECT_NOT_FOUND", detail),
        ProjectAggregateError::Conflict { detail } => error(409, "PROJECT_CONFLICT", detail),
        ProjectAggregateError::Stale { detail } => error(409, "PROJECT_STALE", detail),
        ProjectAggregateError::Unconfirmed { detail }
        | ProjectAggregateError::Rejected { detail } => error(422, "PROJECT_REJECTED", detail),
        ProjectAggregateError::Invalid { detail } => error(422, "PROJECT_INVALID", detail),
        ProjectAggregateError::Unavailable { .. } => {
            error(503, "PROJECT_UNAVAILABLE", "store unavailable")
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use cognitive_store::{ConfirmCaller, PersonalDataLayout, prepare_personal_databases};
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

    #[test]
    fn task_channel_confirm_is_forbidden() {
        let (_tmp, store) = authority();
        let response = handle(
            "POST /task/project/v1/confirm",
            br#"{"preview_id":"x","preview_digest":"y"}"#,
            &store,
        );
        assert_eq!(response.status, 403);
        assert!(
            response
                .body
                .contains("PROJECT_AGGREGATE_CHANNEL_FORBIDDEN")
        );
    }

    #[test]
    fn empty_list_has_no_fake_buttons() {
        let (_tmp, store) = authority();
        let response = handle("GET /management/project/v1/list", b"", &store);
        assert_eq!(response.status, 200);
        assert!(response.body.contains("\"projects\":[]"));
        assert!(!response.body.contains("Approve"));
        assert!(!response.body.contains("fake"));
    }

    #[test]
    fn task_ref_is_not_a_project_detail() {
        let (_tmp, store) = authority();
        let response = handle(
            "GET /management/project/v1/detail?project_id=task://personal/p11",
            b"",
            &store,
        );
        assert_eq!(response.status, 404);
    }

    #[test]
    fn pending_previews_omit_digest() {
        let (_tmp, store) = authority();
        let plane = ProjectAggregateStore::from_authority_store(&store);
        let (draft_id, _) = plane.create_draft(b"payload", 1).unwrap();
        plane.put_draft_charter(&draft_id, b"charter", 2).unwrap();
        let (preview_id, digest) = plane
            .request_preview("activation", &draft_id, b"bytes", 3)
            .unwrap();
        let listed = handle(
            &format!("GET /management/project/v1/pending-previews?subject_ref={draft_id}"),
            b"",
            &store,
        );
        assert_eq!(listed.status, 200);
        assert!(!listed.body.contains(&digest));
        let detail = handle(
            &format!("GET /management/project/v1/preview-detail?preview_id={preview_id}"),
            b"",
            &store,
        );
        assert!(detail.body.contains(&digest));
    }

    #[test]
    fn g1_confirm_mints_creating_project() {
        let (_tmp, store) = authority();
        let plane = ProjectAggregateStore::from_authority_store(&store);
        let (draft_id, _) = plane.create_draft(b"payload", 1).unwrap();
        plane.put_draft_charter(&draft_id, b"charter", 2).unwrap();
        let (preview_id, digest) = plane
            .request_preview("activation", &draft_id, b"bytes", 3)
            .unwrap();
        let body = json!({"preview_id": preview_id, "preview_digest": digest}).to_string();
        let response = handle(
            "POST /management/project/v1/confirm",
            body.as_bytes(),
            &store,
        );
        assert_eq!(response.status, 200, "{}", response.body);
        assert!(response.body.contains("activated"));
        let list = handle("GET /management/project/v1/list", b"", &store);
        assert!(list.body.contains("creating"));
        assert!(list.body.contains("\"cost\":\"unknown\""));
        assert!(!list.body.contains("\"cost\":0"));
    }

    #[test]
    fn roster_is_empty_before_t04() {
        let (_tmp, store) = authority();
        let plane = ProjectAggregateStore::from_authority_store(&store);
        let (draft_id, _) = plane.create_draft(b"payload", 1).unwrap();
        plane.put_draft_charter(&draft_id, b"charter", 2).unwrap();
        let (preview_id, digest) = plane
            .request_preview("activation", &draft_id, b"bytes", 3)
            .unwrap();
        plane
            .confirm_preview(ConfirmCaller::OwnerManagement, &preview_id, &digest, 4)
            .unwrap();
        let project = plane.list_projects(8).unwrap();
        let response = handle(
            &format!(
                "GET /management/project/v1/roster?project_id={}",
                project[0].project_id
            ),
            b"",
            &store,
        );
        assert!(response.body.contains("\"roster\":[]"));
        assert!(response.body.contains("employee-authority-not-implemented"));
    }
}
