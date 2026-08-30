//! Personal-private `/management/project/v1/*` projection (P11-T03).
//!
//! Not P7-T05 frozen inventory. Empty/unavailable when there is no authority.
//! Task-channel writes are 403 (N12).

use cognitive_store::{
    ASSISTANT_ENGINE_ID, ASSISTANT_PI_PIN, ASSISTANT_PRIVATE_CANDIDATE_PROTOCOL, ArchiveAppendSpec,
    ArchiveReadSpec, AssistantPlane, AssistantTurnSpec, CONVERSATION_ARCHIVE_PROJECTION_ID,
    ConfirmCaller, ConversationStore, EmployeeStore, HandoffSpec, PendingPreviewRow,
    ProjectAggregateError, ProjectAggregateStore, ProjectRow, RosterProposal, SpeechArchiveSpec,
    SqliteAuthorityStore, reject_closed_candidate_schema,
};
use serde_json::{Value, json};

use super::resource_api::ResourceApiResponse;

const ROUTE_LITERALS: &[&str] = &[
    "GET /management/project/v1/list",
    "GET /management/project/v1/detail",
    "GET /management/project/v1/axis",
    "GET /management/project/v1/roster",
    "GET /management/project/v1/employee.catalog",
    "GET /management/project/v1/pending-previews",
    "GET /management/project/v1/preview-detail",
    "POST /management/project/v1/draft.apply",
    "POST /management/project/v1/preview.request",
    "POST /management/project/v1/preview.reject",
    "POST /management/project/v1/preview.narrow",
    "POST /management/project/v1/confirm",
    "GET /management/project/v1/standing-policies",
    "POST /management/project/v1/standing-policy.create",
    "POST /management/project/v1/standing-policy.revoke",
    "POST /management/project/v1/roster.register",
    "POST /management/project/v1/employee.seat.request",
    "POST /management/project/v1/employee.seat.confirm",
    "POST /management/project/v1/employee.runtime.bind",
    "POST /management/project/v1/speech.candidate",
    "POST /management/project/v1/conversation.append",
    "GET /management/project/v1/conversation.archive",
    "GET /management/project/v1/conversation.record",
    "POST /management/project/v1/handoff.record",
    "POST /management/project/v1/assistant.turn",
    "GET /task/project/v1/list",
    "POST /task/project/v1/draft.apply",
    "POST /task/project/v1/preview.request",
    "POST /task/project/v1/preview.reject",
    "POST /task/project/v1/preview.narrow",
    "POST /task/project/v1/confirm",
    "GET /task/project/v1/standing-policies",
    "POST /task/project/v1/standing-policy.create",
    "POST /task/project/v1/standing-policy.revoke",
    "GET /task/project/v1/roster",
    "GET /task/project/v1/employee.catalog",
    "POST /task/project/v1/roster.register",
    "POST /task/project/v1/employee.seat.request",
    "POST /task/project/v1/employee.seat.confirm",
    "POST /task/project/v1/employee.runtime.bind",
    "POST /task/project/v1/speech.candidate",
    "POST /task/project/v1/conversation.append",
    "GET /task/project/v1/conversation.archive",
    "GET /task/project/v1/conversation.record",
    "POST /task/project/v1/handoff.record",
    "POST /task/project/v1/assistant.turn",
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
    let employees = EmployeeStore::from_authority_store(store);
    let conversations = ConversationStore::from_authority_store(store);
    match literal {
        "GET /management/project/v1/list" => list_projects(method_path, &plane),
        "GET /management/project/v1/detail" => detail(method_path, &plane),
        "GET /management/project/v1/axis" => axis(method_path, &plane, &employees),
        "GET /management/project/v1/roster" => roster(method_path, &plane, &employees),
        "GET /management/project/v1/employee.catalog" => employee_catalog(method_path, &employees),
        "GET /management/project/v1/pending-previews" => pending_previews(method_path, &plane),
        "GET /management/project/v1/preview-detail" => preview_detail(method_path, &plane),
        "POST /management/project/v1/draft.apply" => draft_apply(body, &plane),
        "POST /management/project/v1/preview.request" => preview_request(body, &plane),
        "POST /management/project/v1/preview.reject" => preview_reject(body, &plane),
        "POST /management/project/v1/preview.narrow" => preview_narrow(body, &plane),
        "POST /management/project/v1/confirm" => confirm(body, &plane),
        "GET /management/project/v1/standing-policies" => standing_policies(&plane),
        "POST /management/project/v1/standing-policy.create" => {
            standing_policy_create(body, &plane)
        }
        "POST /management/project/v1/standing-policy.revoke" => {
            standing_policy_revoke(body, &plane)
        }
        "POST /management/project/v1/roster.register" => roster_register(body, &employees),
        "POST /management/project/v1/employee.seat.request" => seat_request(body, &employees),
        "POST /management/project/v1/employee.seat.confirm" => seat_confirm(body, &employees),
        "POST /management/project/v1/employee.runtime.bind" => runtime_bind(body, &employees),
        "POST /management/project/v1/speech.candidate" => {
            speech_candidate(body, &employees, &conversations)
        }
        "POST /management/project/v1/conversation.append" => {
            conversation_append(body, &conversations)
        }
        "GET /management/project/v1/conversation.archive" => {
            conversation_archive(method_path, &conversations)
        }
        "GET /management/project/v1/conversation.record" => {
            conversation_record(method_path, &conversations)
        }
        "POST /management/project/v1/handoff.record" => handoff_record(body, &employees),
        "POST /management/project/v1/assistant.turn" => assistant_turn(body, store),
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

fn axis(
    method_path: &str,
    plane: &ProjectAggregateStore,
    employees: &EmployeeStore,
) -> ResourceApiResponse {
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
                            "seated": employees
                                .stage_is_seated(&project_id, &plan_id, &stage.stage_id)
                                .unwrap_or(false),
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

fn roster(
    method_path: &str,
    plane: &ProjectAggregateStore,
    employees: &EmployeeStore,
) -> ResourceApiResponse {
    let Some(project_id) = query_parameter(method_path, "project_id").filter(|v| !v.is_empty())
    else {
        return error(400, "PROJECT_ID_REQUIRED", "roster requires project_id");
    };
    match plane.get_project(&project_id) {
        Ok(None) => error(404, "PROJECT_NOT_FOUND", "project not found"),
        Ok(Some(_)) => match employees.list_roster(&project_id) {
            Ok(rows) => {
                let progress = employees.seating_progress(&project_id).ok();
                ok(json!({
                    "status": "ok",
                    "projection": "personal-private",
                    "roster": rows.iter().map(|row| json!({
                        "employee_id": row.employee_id,
                        "state": row.state,
                        "responsible_stage_ids": serde_json::from_str::<Value>(
                            &row.responsible_stage_ids_json,
                        )
                        .unwrap_or(Value::Array(vec![])),
                        "model_bound": row.provider_model_binding.is_some(),
                        "is_current_manager": row.is_current_manager,
                        "runtime_binding_ref": row.runtime_binding_ref,
                    })).collect::<Vec<_>>(),
                    "authority_note": if rows.is_empty() { "empty-roster" } else { "employee" },
                    "seated": progress.map(|p| p.seated),
                    "roster_count": progress.map(|p| p.roster),
                }))
            }
            Err(error) => store_error(error),
        },
        Err(error) => store_error(error),
    }
}

fn employee_catalog(method_path: &str, employees: &EmployeeStore) -> ResourceApiResponse {
    let Some(project_id) = query_parameter(method_path, "project_id").filter(|v| !v.is_empty())
    else {
        return error(400, "PROJECT_ID_REQUIRED", "catalog requires project_id");
    };
    let Some(employee_id) = query_parameter(method_path, "employee_id").filter(|v| !v.is_empty())
    else {
        return error(400, "EMPLOYEE_ID_REQUIRED", "catalog requires employee_id");
    };
    match employees.tool_catalog(&project_id, &employee_id) {
        Ok(catalog) => ok(json!({
            "status": "ok",
            "projection": "personal-private",
            "catalog": catalog,
        })),
        Err(error) => store_error(error),
    }
}

fn roster_register(body: &[u8], employees: &EmployeeStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(project_id) = document.get("project_id").and_then(Value::as_str) else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    let Some(plan_revision_id) = document.get("plan_revision_id").and_then(Value::as_str) else {
        return error(
            400,
            "PLAN_REVISION_ID_REQUIRED",
            "plan_revision_id required",
        );
    };
    let Some(items) = document.get("proposals").and_then(Value::as_array) else {
        return error(400, "PROPOSALS_REQUIRED", "proposals required");
    };
    let mut proposals = Vec::new();
    for item in items {
        let Some(slot) = item.get("slot").and_then(Value::as_str) else {
            return error(400, "SLOT_REQUIRED", "proposal.slot required");
        };
        let Some(specialization) = item.get("specialization").and_then(Value::as_str) else {
            return error(
                400,
                "SPECIALIZATION_REQUIRED",
                "proposal.specialization required",
            );
        };
        let prompt = item
            .get("prompt")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let tools_declared = item
            .get("tools_declared")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        proposals.push(RosterProposal {
            slot: slot.to_owned(),
            specialization: specialization.to_owned(),
            prompt,
            tools_declared,
        });
    }
    match employees.register_roster(
        ConfirmCaller::OwnerManagement,
        project_id,
        plan_revision_id,
        &proposals,
        now_ms(),
    ) {
        Ok(ids) => ok(json!({ "status": "ok", "employee_ids": ids })),
        Err(error) => store_error(error),
    }
}

fn seat_request(body: &[u8], employees: &EmployeeStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(employee_id) = document.get("employee_id").and_then(Value::as_str) else {
        return error(400, "EMPLOYEE_ID_REQUIRED", "employee_id required");
    };
    match employees.request_seating(ConfirmCaller::OwnerManagement, employee_id, now_ms()) {
        Ok(()) => ok(json!({ "status": "ok", "state": "seating" })),
        Err(error) => store_error(error),
    }
}

fn seat_confirm(body: &[u8], employees: &EmployeeStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(employee_id) = document.get("employee_id").and_then(Value::as_str) else {
        return error(400, "EMPLOYEE_ID_REQUIRED", "employee_id required");
    };
    let model = document.get("model_binding").and_then(Value::as_str);
    let accept = document
        .get("accept")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    match employees.confirm_seating(
        ConfirmCaller::OwnerManagement,
        employee_id,
        model,
        accept,
        now_ms(),
    ) {
        Ok(state) => ok(json!({ "status": "ok", "state": state })),
        Err(error) => store_error(error),
    }
}

fn runtime_bind(body: &[u8], employees: &EmployeeStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(employee_id) = document.get("employee_id").and_then(Value::as_str) else {
        return error(400, "EMPLOYEE_ID_REQUIRED", "employee_id required");
    };
    let Some(runtime_binding_ref) = document.get("runtime_binding_ref").and_then(Value::as_str)
    else {
        return error(
            400,
            "RUNTIME_BINDING_REQUIRED",
            "runtime_binding_ref required",
        );
    };
    match employees.bind_runtime(
        ConfirmCaller::OwnerManagement,
        employee_id,
        runtime_binding_ref,
        now_ms(),
    ) {
        Ok(()) => ok(json!({ "status": "ok" })),
        Err(error) => store_error(error),
    }
}

fn speech_candidate(
    body: &[u8],
    employees: &EmployeeStore,
    conversations: &ConversationStore,
) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(project_id) = document.get("project_id").and_then(Value::as_str) else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    let Some(employee_id) = document.get("employee_id").and_then(Value::as_str) else {
        return error(400, "EMPLOYEE_ID_REQUIRED", "employee_id required");
    };
    let kind = document
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("chatter");
    let mentioned = document
        .get("mentioned")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let projection_id = document
        .get("projection_id")
        .and_then(Value::as_str)
        .unwrap_or(CONVERSATION_ARCHIVE_PROJECTION_ID);
    let speech_body = document.get("body").and_then(Value::as_str).unwrap_or("");
    match conversations.land_speech(
        employees,
        &SpeechArchiveSpec {
            projection_id,
            project_id,
            employee_id,
            kind,
            mentioned,
            body: speech_body,
            now_ms: now_ms(),
        },
    ) {
        Ok(outcome) => ok(json!({
            "status": "ok",
            "delivered": outcome.delivered,
            "reason": outcome.reason,
            "audit_id": outcome.audit_id,
            "archive_record_id": outcome.record_id,
            "projection_id": CONVERSATION_ARCHIVE_PROJECTION_ID,
        })),
        Err(error) => store_error(error),
    }
}

fn conversation_archive(
    method_path: &str,
    conversations: &ConversationStore,
) -> ResourceApiResponse {
    let Some(project_id) = query_parameter(method_path, "project_id").filter(|v| !v.is_empty())
    else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
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
    let include_bodies = query_parameter(method_path, "include_bodies")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let projection_id = query_parameter(method_path, "projection_id")
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| CONVERSATION_ARCHIVE_PROJECTION_ID.to_owned());
    let employee_id = query_parameter(method_path, "employee_id").filter(|v| !v.is_empty());
    let resume_from = query_parameter(method_path, "resume_from").filter(|v| !v.is_empty());
    match conversations.read_index(&ArchiveReadSpec {
        projection_id: &projection_id,
        caller_project_id: &project_id,
        target_project_id: &project_id,
        employee_id: employee_id.as_deref(),
        limit,
        resume_from: resume_from.as_deref(),
        include_bodies,
    }) {
        Ok(page) => ok(json!({
            "status": "ok",
            "projection_id": CONVERSATION_ARCHIVE_PROJECTION_ID,
            "project_id": project_id,
            "observation_only": true,
            "truncated": page.truncated,
            "next_cursor": page.next_cursor,
            "records": page.records.iter().map(|row| json!({
                "record_id": row.record_id,
                "employee_id": row.employee_id,
                "kind": row.kind,
                "body_digest": row.body_digest,
                "created_at": row.created_at,
            })).collect::<Vec<_>>(),
        })),
        Err(error) => store_error(error),
    }
}

fn conversation_record(
    method_path: &str,
    conversations: &ConversationStore,
) -> ResourceApiResponse {
    let Some(project_id) = query_parameter(method_path, "project_id").filter(|v| !v.is_empty())
    else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    let Some(record_id) = query_parameter(method_path, "record_id").filter(|v| !v.is_empty())
    else {
        return error(400, "RECORD_ID_REQUIRED", "record_id required");
    };
    let projection_id = query_parameter(method_path, "projection_id")
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| CONVERSATION_ARCHIVE_PROJECTION_ID.to_owned());
    match conversations.read_record(&projection_id, &project_id, &record_id) {
        Ok(row) => ok(json!({
            "status": "ok",
            "observation_only": true,
            "record_id": row.record_id,
            "employee_id": row.employee_id,
            "kind": row.kind,
            "body_digest": row.body_digest,
            "body_redacted": row.body_redacted,
            "created_at": row.created_at,
        })),
        Err(error) => store_error(error),
    }
}

fn conversation_append(body: &[u8], conversations: &ConversationStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(project_id) = document.get("project_id").and_then(Value::as_str) else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    let Some(employee_id) = document.get("employee_id").and_then(Value::as_str) else {
        return error(400, "EMPLOYEE_ID_REQUIRED", "employee_id required");
    };
    let Some(kind) = document.get("kind").and_then(Value::as_str) else {
        return error(400, "KIND_REQUIRED", "kind required");
    };
    let speech_body = document.get("body").and_then(Value::as_str).unwrap_or("");
    let projection_id = document
        .get("projection_id")
        .and_then(Value::as_str)
        .unwrap_or(CONVERSATION_ARCHIVE_PROJECTION_ID);
    match conversations.append(
        ConfirmCaller::OwnerManagement,
        &ArchiveAppendSpec {
            projection_id,
            project_id,
            employee_id,
            kind,
            body: speech_body,
            now_ms: now_ms(),
        },
    ) {
        Ok(record_id) => ok(json!({
            "status": "ok",
            "archive_record_id": record_id,
            "observation_only": true,
            "projection_id": CONVERSATION_ARCHIVE_PROJECTION_ID,
        })),
        Err(error) => store_error(error),
    }
}

fn handoff_record(body: &[u8], employees: &EmployeeStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(project_id) = document.get("project_id").and_then(Value::as_str) else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    let Some(source) = document.get("source_employee_id").and_then(Value::as_str) else {
        return error(400, "SOURCE_REQUIRED", "source_employee_id required");
    };
    let Some(target) = document.get("target_employee_id").and_then(Value::as_str) else {
        return error(400, "TARGET_REQUIRED", "target_employee_id required");
    };
    let Some(digest) = document.get("bounded_work_digest").and_then(Value::as_str) else {
        return error(400, "DIGEST_REQUIRED", "bounded_work_digest required");
    };
    let blocked_or_ready = document
        .get("blocked_or_ready")
        .and_then(Value::as_str)
        .unwrap_or("ready");
    match employees.record_handoff(
        ConfirmCaller::OwnerManagement,
        &HandoffSpec {
            project_id,
            source_employee_id: source,
            target_employee_id: target,
            bounded_work_digest: digest,
            blocked_or_ready,
            now_ms: now_ms(),
        },
    ) {
        Ok(handoff_id) => ok(json!({ "status": "ok", "handoff_id": handoff_id })),
        Err(error) => store_error(error),
    }
}

fn assistant_turn(body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
    if reject_closed_candidate_schema(body).is_err() {
        return error(
            422,
            "ASSISTANT_SCHEMA_CLOSED",
            "closed candidate schema: grant/secret/trigger-arm fields rejected",
        );
    }
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(kind) = document.get("kind").and_then(Value::as_str) else {
        return error(400, "ASSISTANT_KIND_REQUIRED", "kind required");
    };
    let Some(draft_id) = document.get("draft_id").and_then(Value::as_str) else {
        return error(400, "DRAFT_ID_REQUIRED", "draft_id required");
    };
    let Some(object_kind) = document.get("object_kind").and_then(Value::as_str) else {
        return error(400, "OBJECT_KIND_REQUIRED", "object_kind required");
    };
    let payload = document
        .get("payload")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let provenance_json = match document.get("provenance") {
        Some(Value::Object(_)) => document
            .get("provenance")
            .cloned()
            .and_then(|value| serde_json::to_string(&value).ok()),
        Some(Value::String(raw)) => Some(raw.clone()),
        Some(_) | None => None,
    };
    let Some(provenance_json) = provenance_json else {
        return error(
            422,
            "ASSISTANT_PROVENANCE_REQUIRED",
            "typed provenance required (sources | owner-stated | assistant-assumption)",
        );
    };
    let project_id = document
        .get("project_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty());
    let tool_owned: Vec<String> = document
        .get("tools")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let tools: Vec<&str> = tool_owned.iter().map(String::as_str).collect();
    let plane = AssistantPlane::from_authority_store(store);
    match plane.run_turn(&AssistantTurnSpec {
        kind,
        draft_id,
        object_kind,
        payload: &payload,
        provenance_json: &provenance_json,
        project_id,
        tools: &tools,
        now_ms: now_ms(),
    }) {
        Ok(outcome) => ok(json!({
            "status": "ok",
            "engine": ASSISTANT_ENGINE_ID,
            "pi_pin": ASSISTANT_PI_PIN,
            "protocol": ASSISTANT_PRIVATE_CANDIDATE_PROTOCOL,
            "installed_agent": false,
            "candidate_id": outcome.candidate_id,
            "candidate_digest": outcome.candidate_digest,
            "preview_id": outcome.preview_id,
            "object_kind": outcome.object_kind,
            "context_refs": outcome.context_refs,
            "observation_only": true,
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
            "receipt_ref": detail.receipt_ref,
            "superseded_by": detail.superseded_by,
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
        Ok((preview_id, preview_digest)) => ok(json!({
            "status": "ok",
            "preview_id": preview_id,
            "preview_digest": preview_digest,
            "created_at": now_ms(),
        })),
        Err(error) => store_error(error),
    }
}

fn standing_policies(plane: &ProjectAggregateStore) -> ResourceApiResponse {
    match plane.list_standing_policies(now_ms()) {
        Ok(policies) => ok(json!({
            "status": "ok",
            "projection": "personal-private",
            "policies": policies.iter().map(|row| json!({
                "policy_id": row.policy_id,
                "subject_class": row.subject_class,
                "subject_ref": row.subject_ref,
                "expires_at": row.expires_at,
                "created_at": row.created_at,
                "active": row.active,
            })).collect::<Vec<_>>(),
        })),
        Err(error) => store_error(error),
    }
}

fn standing_policy_create(body: &[u8], plane: &ProjectAggregateStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(subject_class) = document.get("subject_class").and_then(Value::as_str) else {
        return error(400, "SUBJECT_CLASS_REQUIRED", "subject_class required");
    };
    let Some(subject_ref) = document.get("subject_ref").and_then(Value::as_str) else {
        return error(400, "SUBJECT_REF_REQUIRED", "subject_ref required");
    };
    let expires_at = document.get("expires_at").and_then(Value::as_i64);
    match plane.create_standing_policy(
        ConfirmCaller::OwnerManagement,
        subject_class,
        subject_ref,
        expires_at,
        now_ms(),
    ) {
        Ok(policy_id) => ok(json!({
            "status": "ok",
            "policy_id": policy_id,
        })),
        Err(error) => store_error(error),
    }
}

fn standing_policy_revoke(body: &[u8], plane: &ProjectAggregateStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(policy_id) = document.get("policy_id").and_then(Value::as_str) else {
        return error(400, "POLICY_ID_REQUIRED", "policy_id required");
    };
    match plane.revoke_standing_policy(ConfirmCaller::OwnerManagement, policy_id, now_ms()) {
        Ok(()) => ok(json!({
            "status": "ok",
            "result": "revoked",
            "policy_id": policy_id,
        })),
        Err(error) => store_error(error),
    }
}

fn preview_reject(body: &[u8], plane: &ProjectAggregateStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(preview_id) = document.get("preview_id").and_then(Value::as_str) else {
        return error(400, "PREVIEW_ID_REQUIRED", "preview_id required");
    };
    let Some(preview_digest) = document.get("preview_digest").and_then(Value::as_str) else {
        return error(400, "PREVIEW_DIGEST_REQUIRED", "preview_digest required");
    };
    match plane.reject_preview(
        ConfirmCaller::OwnerManagement,
        preview_id,
        preview_digest,
        now_ms(),
    ) {
        Ok(receipt_ref) => ok(json!({
            "status": "ok",
            "result": "rejected",
            "receipt_ref": receipt_ref,
        })),
        Err(error) => store_error(error),
    }
}

fn preview_narrow(body: &[u8], plane: &ProjectAggregateStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(preview_id) = document.get("preview_id").and_then(Value::as_str) else {
        return error(400, "PREVIEW_ID_REQUIRED", "preview_id required");
    };
    let Some(preview_digest) = document.get("preview_digest").and_then(Value::as_str) else {
        return error(400, "PREVIEW_DIGEST_REQUIRED", "preview_digest required");
    };
    let Some(preview_bytes) = document.get("preview_bytes").and_then(Value::as_str) else {
        return error(400, "PREVIEW_BYTES_REQUIRED", "preview_bytes required");
    };
    match plane.narrow_preview(
        ConfirmCaller::OwnerManagement,
        preview_id,
        preview_digest,
        preview_bytes.as_bytes(),
        now_ms(),
    ) {
        Ok(result) => ok(json!({
            "status": "ok",
            "preview_id": result.preview_id,
            "preview_digest": result.preview_digest,
            "superseded_preview_id": result.superseded_preview_id,
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
        for path in [
            "POST /task/project/v1/confirm",
            "POST /task/project/v1/preview.reject",
            "POST /task/project/v1/preview.narrow",
            "POST /task/project/v1/standing-policy.create",
            "POST /task/project/v1/standing-policy.revoke",
            "GET /task/project/v1/standing-policies",
        ] {
            let response = handle(
                path,
                br#"{"preview_id":"x","preview_digest":"y","preview_bytes":"z"}"#,
                &store,
            );
            assert_eq!(response.status, 403, "{path}");
            assert!(
                response
                    .body
                    .contains("PROJECT_AGGREGATE_CHANNEL_FORBIDDEN")
            );
            assert!(!response.body.contains("Approve"));
        }
    }

    #[test]
    fn http_reject_leaves_receipt_and_blocks_old_digest() {
        let (_tmp, store) = authority();
        let plane = ProjectAggregateStore::from_authority_store(&store);
        let (draft_id, _) = plane.create_draft(b"payload", 1).unwrap();
        plane.put_draft_charter(&draft_id, b"charter", 2).unwrap();
        let (preview_id, digest) = plane
            .request_preview("activation", &draft_id, b"bytes", 3)
            .unwrap();
        let body = json!({"preview_id": preview_id, "preview_digest": digest}).to_string();
        let rejected = handle(
            "POST /management/project/v1/preview.reject",
            body.as_bytes(),
            &store,
        );
        assert_eq!(rejected.status, 200, "{}", rejected.body);
        assert!(rejected.body.contains("rejected"));
        assert!(rejected.body.contains("receipt_ref"));
        assert!(!rejected.body.contains("Approve"));
        let confirm = handle(
            "POST /management/project/v1/confirm",
            body.as_bytes(),
            &store,
        );
        assert_eq!(confirm.status, 422, "{}", confirm.body);
        let detail = handle(
            &format!("GET /management/project/v1/preview-detail?preview_id={preview_id}"),
            b"",
            &store,
        );
        assert!(detail.body.contains("\"rejected\""));
        assert!(detail.body.contains("receipt_ref"));
    }

    #[test]
    fn http_narrow_supersedes_old_and_confirm_works_for_new() {
        let (_tmp, store) = authority();
        let plane = ProjectAggregateStore::from_authority_store(&store);
        let (draft_id, _) = plane.create_draft(b"payload", 1).unwrap();
        plane.put_draft_charter(&draft_id, b"charter", 2).unwrap();
        let (old_id, old_digest) = plane
            .request_preview("activation", &draft_id, b"bytes", 3)
            .unwrap();
        let narrow_body = json!({
            "preview_id": old_id,
            "preview_digest": old_digest,
            "preview_bytes": "narrowed-bytes"
        })
        .to_string();
        let narrowed = handle(
            "POST /management/project/v1/preview.narrow",
            narrow_body.as_bytes(),
            &store,
        );
        assert_eq!(narrowed.status, 200, "{}", narrowed.body);
        assert!(narrowed.body.contains("superseded_preview_id"));
        assert!(!narrowed.body.contains("Approve"));
        let old_confirm = handle(
            "POST /management/project/v1/confirm",
            json!({"preview_id": old_id, "preview_digest": old_digest})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(old_confirm.status, 422, "{}", old_confirm.body);
        let new_id = serde_json::from_str::<Value>(&narrowed.body)
            .unwrap()
            .get("preview_id")
            .and_then(Value::as_str)
            .unwrap()
            .to_owned();
        let new_digest = serde_json::from_str::<Value>(&narrowed.body)
            .unwrap()
            .get("preview_digest")
            .and_then(Value::as_str)
            .unwrap()
            .to_owned();
        let confirmed = handle(
            "POST /management/project/v1/confirm",
            json!({"preview_id": new_id, "preview_digest": new_digest})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(confirmed.status, 200, "{}", confirmed.body);
        assert!(confirmed.body.contains("activated"));
    }

    #[test]
    fn http_wrong_digest_fail_closed() {
        let (_tmp, store) = authority();
        let plane = ProjectAggregateStore::from_authority_store(&store);
        let (draft_id, _) = plane.create_draft(b"payload", 1).unwrap();
        plane.put_draft_charter(&draft_id, b"charter", 2).unwrap();
        let (preview_id, _) = plane
            .request_preview("activation", &draft_id, b"bytes", 3)
            .unwrap();
        let wrong = json!({
            "preview_id": preview_id,
            "preview_digest": "0".repeat(64),
            "preview_bytes": "nope"
        })
        .to_string();
        for path in [
            "POST /management/project/v1/confirm",
            "POST /management/project/v1/preview.reject",
            "POST /management/project/v1/preview.narrow",
        ] {
            let response = handle(path, wrong.as_bytes(), &store);
            assert_eq!(response.status, 409, "{path} {}", response.body);
        }
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
    fn http_standing_policy_missing_expires_at_is_rejected() {
        let (_tmp, store) = authority();
        let missing = handle(
            "POST /management/project/v1/standing-policy.create",
            br#"{"subject_class":"outbound","subject_ref":"grant-expansion"}"#,
            &store,
        );
        assert_eq!(missing.status, 422, "{}", missing.body);
        assert!(missing.body.contains("expires_at required"));
        let too_long = cognitive_store::now_ms() + cognitive_store::STANDING_POLICY_MAX_TTL_MS + 1;
        let over = handle(
            "POST /management/project/v1/standing-policy.create",
            json!({
                "subject_class": "outbound",
                "subject_ref": "grant-expansion",
                "expires_at": too_long
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(over.status, 422, "{}", over.body);
        assert!(over.body.contains("7-day"));
    }

    #[test]
    fn http_standing_policy_list_and_revoke() {
        let (_tmp, store) = authority();
        let expires = cognitive_store::now_ms() + 3 * 24 * 60 * 60 * 1000;
        let created = handle(
            "POST /management/project/v1/standing-policy.create",
            json!({
                "subject_class": "outbound",
                "subject_ref": "grant-expansion",
                "expires_at": expires
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(created.status, 200, "{}", created.body);
        let policy_id = serde_json::from_str::<Value>(&created.body)
            .unwrap()
            .get("policy_id")
            .and_then(Value::as_str)
            .unwrap()
            .to_owned();
        let listed = handle("GET /management/project/v1/standing-policies", b"", &store);
        assert_eq!(listed.status, 200, "{}", listed.body);
        assert!(listed.body.contains(&policy_id));
        assert!(listed.body.contains("\"active\":true"));
        assert!(!listed.body.contains("Approve"));
        let revoked = handle(
            "POST /management/project/v1/standing-policy.revoke",
            json!({"policy_id": policy_id}).to_string().as_bytes(),
            &store,
        );
        assert_eq!(revoked.status, 200, "{}", revoked.body);
        let empty = handle("GET /management/project/v1/standing-policies", b"", &store);
        assert!(!empty.body.contains(&policy_id));
    }

    #[test]
    fn http_grant_expansion_confirm_returns_digest_on_canvas_path() {
        use cognitive_store::{EmployeeStore, RosterProposal, StageSpec};
        let (_tmp, store) = authority();
        let plane = ProjectAggregateStore::from_authority_store(&store);
        let employees = EmployeeStore::from_authority_store(&store);
        let (draft_id, _) = plane.create_draft(b"payload", 1).unwrap();
        plane.put_draft_charter(&draft_id, b"charter", 2).unwrap();
        let (preview_id, digest) = plane
            .request_preview("activation", &draft_id, b"bytes", 3)
            .unwrap();
        let project_id = plane
            .confirm_preview(ConfirmCaller::OwnerManagement, &preview_id, &digest, 4)
            .unwrap()
            .new_ref;
        let plan_id = plane
            .apply_plan_revision(
                &project_id,
                &project_id,
                &[StageSpec {
                    stage_id: "s1".to_owned(),
                    title: "Manage".to_owned(),
                    objective: "manage".to_owned(),
                    output_contract_digest: ProjectAggregateStore::digest_hex(b"out"),
                    acceptance_spec_ref: Some("cas:spec".to_owned()),
                    cadence_json: None,
                    responsible_slot: "manager".to_owned(),
                    blocking_gap: None,
                }],
                20,
            )
            .unwrap();
        let ids = employees
            .register_roster(
                ConfirmCaller::OwnerManagement,
                &project_id,
                &plan_id,
                &[RosterProposal {
                    slot: "manager".to_owned(),
                    specialization: "project-manager".to_owned(),
                    prompt: "coordinate".to_owned(),
                    tools_declared: vec!["workspace-write".to_owned()],
                }],
                21,
            )
            .unwrap();
        employees
            .request_seating(ConfirmCaller::OwnerManagement, &ids[0], 30)
            .unwrap();
        employees
            .confirm_seating(
                ConfirmCaller::OwnerManagement,
                &ids[0],
                Some("flash"),
                true,
                31,
            )
            .unwrap();
        employees
            .record_install_fact("mcp:search", "1.0.0", 32)
            .unwrap();
        let subject = json!({
            "project_id": project_id,
            "employee_id": ids[0],
            "capability_ref": "mcp:search",
            "scope": "project-a"
        })
        .to_string();
        let minted = handle(
            "POST /management/project/v1/preview.request",
            json!({"subject_kind": "grant-expansion", "subject_ref": subject})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(minted.status, 200, "{}", minted.body);
        let minted_json = serde_json::from_str::<Value>(&minted.body).unwrap();
        let grant_preview_id = minted_json
            .get("preview_id")
            .and_then(Value::as_str)
            .unwrap();
        let grant_digest = minted_json
            .get("preview_digest")
            .and_then(Value::as_str)
            .expect("canvas HTTP returns digest");
        let chat = handle(
            "POST /task/project/v1/confirm",
            json!({"preview_id": grant_preview_id, "preview_digest": grant_digest})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(chat.status, 403);
        let confirmed = handle(
            "POST /management/project/v1/confirm",
            json!({"preview_id": grant_preview_id, "preview_digest": grant_digest})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(confirmed.status, 200, "{}", confirmed.body);
        assert!(confirmed.body.contains("granted"));
        let catalog = handle(
            &format!(
                "GET /management/project/v1/employee.catalog?project_id={project_id}&employee_id={}",
                ids[0]
            ),
            b"",
            &store,
        );
        assert!(catalog.body.contains("mcp:search"), "{}", catalog.body);
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
    fn roster_is_empty_before_employees() {
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
        assert!(response.body.contains("empty-roster"));
        assert!(!response.body.contains("employee-authority-not-implemented"));
    }

    #[test]
    fn roster_register_and_seat_via_http() {
        use cognitive_store::StageSpec;
        let (_tmp, store) = authority();
        let plane = ProjectAggregateStore::from_authority_store(&store);
        let (draft_id, _) = plane.create_draft(b"payload", 1).unwrap();
        plane.put_draft_charter(&draft_id, b"charter", 2).unwrap();
        let (preview_id, digest) = plane
            .request_preview("activation", &draft_id, b"bytes", 3)
            .unwrap();
        let project_id = plane
            .confirm_preview(ConfirmCaller::OwnerManagement, &preview_id, &digest, 4)
            .unwrap()
            .new_ref;
        let plan_id = plane
            .apply_plan_revision(
                &project_id,
                &project_id,
                &[StageSpec {
                    stage_id: "s1".to_owned(),
                    title: "Manage".to_owned(),
                    objective: "manage".to_owned(),
                    output_contract_digest: ProjectAggregateStore::digest_hex(b"out"),
                    acceptance_spec_ref: Some("cas:spec".to_owned()),
                    cadence_json: None,
                    responsible_slot: "manager".to_owned(),
                    blocking_gap: None,
                }],
                20,
            )
            .unwrap();
        let body = json!({
            "project_id": project_id,
            "plan_revision_id": plan_id,
            "proposals": [{
                "slot": "manager",
                "specialization": "project-manager",
                "prompt": "coordinate",
                "tools_declared": ["workspace-write"]
            }]
        })
        .to_string();
        let registered = handle(
            "POST /management/project/v1/roster.register",
            body.as_bytes(),
            &store,
        );
        assert_eq!(registered.status, 200, "{}", registered.body);
        let employee_id = serde_json::from_str::<Value>(&registered.body)
            .unwrap()
            .get("employee_ids")
            .and_then(Value::as_array)
            .and_then(|ids| ids[0].as_str())
            .unwrap()
            .to_owned();
        let seat = handle(
            "POST /management/project/v1/employee.seat.request",
            json!({"employee_id": employee_id}).to_string().as_bytes(),
            &store,
        );
        assert_eq!(seat.status, 200, "{}", seat.body);
        let confirm = handle(
            "POST /management/project/v1/employee.seat.confirm",
            json!({"employee_id": employee_id, "model_binding": "flash", "accept": true})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(confirm.status, 200, "{}", confirm.body);
        let roster = handle(
            &format!("GET /management/project/v1/roster?project_id={project_id}"),
            b"",
            &store,
        );
        assert!(roster.body.contains(&employee_id));
        assert!(roster.body.contains("\"is_current_manager\":true"));
        let task = handle(
            "POST /task/project/v1/roster.register",
            body.as_bytes(),
            &store,
        );
        assert_eq!(task.status, 403);
    }

    #[test]
    fn delivered_speech_lands_in_archive_via_http() {
        use cognitive_store::StageSpec;
        let (_tmp, store) = authority();
        let plane = ProjectAggregateStore::from_authority_store(&store);
        let (draft_id, _) = plane.create_draft(b"payload", 1).unwrap();
        plane.put_draft_charter(&draft_id, b"charter", 2).unwrap();
        let (preview_id, digest) = plane
            .request_preview("activation", &draft_id, b"bytes", 3)
            .unwrap();
        let project_id = plane
            .confirm_preview(ConfirmCaller::OwnerManagement, &preview_id, &digest, 4)
            .unwrap()
            .new_ref;
        let plan_id = plane
            .apply_plan_revision(
                &project_id,
                &project_id,
                &[
                    StageSpec {
                        stage_id: "s1".to_owned(),
                        title: "Manage".to_owned(),
                        objective: "manage".to_owned(),
                        output_contract_digest: ProjectAggregateStore::digest_hex(b"out"),
                        acceptance_spec_ref: Some("cas:spec".to_owned()),
                        cadence_json: None,
                        responsible_slot: "manager".to_owned(),
                        blocking_gap: None,
                    },
                    StageSpec {
                        stage_id: "s2".to_owned(),
                        title: "Research".to_owned(),
                        objective: "research".to_owned(),
                        output_contract_digest: ProjectAggregateStore::digest_hex(b"out2"),
                        acceptance_spec_ref: Some("cas:spec2".to_owned()),
                        cadence_json: None,
                        responsible_slot: "researcher".to_owned(),
                        blocking_gap: None,
                    },
                ],
                20,
            )
            .unwrap();
        let registered = handle(
            "POST /management/project/v1/roster.register",
            json!({
                "project_id": project_id,
                "plan_revision_id": plan_id,
                "proposals": [
                    {
                        "slot": "manager",
                        "specialization": "project-manager",
                        "prompt": "coordinate",
                        "tools_declared": ["workspace-write"]
                    },
                    {
                        "slot": "researcher",
                        "specialization": "member",
                        "prompt": "notes",
                        "tools_declared": ["workspace-write"]
                    }
                ]
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(registered.status, 200, "{}", registered.body);
        let ids = serde_json::from_str::<Value>(&registered.body)
            .unwrap()
            .get("employee_ids")
            .and_then(Value::as_array)
            .unwrap()
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect::<Vec<_>>();
        let seat = handle(
            "POST /management/project/v1/employee.seat.request",
            json!({"employee_id": ids[0]}).to_string().as_bytes(),
            &store,
        );
        assert_eq!(seat.status, 200, "{}", seat.body);
        let confirm = handle(
            "POST /management/project/v1/employee.seat.confirm",
            json!({"employee_id": ids[0], "model_binding": "flash", "accept": true})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(confirm.status, 200, "{}", confirm.body);
        let chatter = handle(
            "POST /management/project/v1/speech.candidate",
            json!({
                "project_id": project_id,
                "employee_id": ids[1],
                "kind": "chatter",
                "body": "side talk"
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(chatter.status, 200, "{}", chatter.body);
        assert!(chatter.body.contains("\"delivered\":false"));
        assert!(chatter.body.contains("\"archive_record_id\":null"));
        let deliverable = handle(
            "POST /management/project/v1/speech.candidate",
            json!({
                "project_id": project_id,
                "employee_id": ids[1],
                "kind": "deliverable",
                "body": "openable note"
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(deliverable.status, 200, "{}", deliverable.body);
        assert!(deliverable.body.contains("\"delivered\":true"));
        assert!(!deliverable.body.contains("\"archive_record_id\":null"));
        let unbounded = handle(
            &format!("GET /management/project/v1/conversation.archive?project_id={project_id}"),
            b"",
            &store,
        );
        assert_eq!(unbounded.status, 422, "{}", unbounded.body);
        assert!(unbounded.body.contains("unbounded conversation resume"));
        let inject = handle(
            &format!(
                "GET /management/project/v1/conversation.archive?project_id={project_id}&limit=32&include_bodies=1"
            ),
            b"",
            &store,
        );
        assert_eq!(inject.status, 422, "{}", inject.body);
        let listed = handle(
            &format!(
                "GET /management/project/v1/conversation.archive?project_id={project_id}&limit=32"
            ),
            b"",
            &store,
        );
        assert_eq!(listed.status, 200, "{}", listed.body);
        assert!(listed.body.contains(&ids[1]));
        assert!(listed.body.contains("deliverable"));
        assert!(listed.body.contains("\"observation_only\":true"));
        assert!(!listed.body.contains("openable note"));
        assert!(!listed.body.contains("side talk"));
        let appended = handle(
            "POST /management/project/v1/conversation.append",
            json!({
                "project_id": project_id,
                "employee_id": ids[1],
                "kind": "note",
                "body": "composer note"
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(appended.status, 200, "{}", appended.body);
        assert!(appended.body.contains("\"observation_only\":true"));
        let listed = handle(
            &format!(
                "GET /management/project/v1/conversation.archive?project_id={project_id}&limit=32"
            ),
            b"",
            &store,
        );
        assert!(listed.body.contains("note"));
        assert!(!listed.body.contains("composer note"));
        let record_id = serde_json::from_str::<Value>(&appended.body)
            .unwrap()
            .get("archive_record_id")
            .and_then(Value::as_str)
            .unwrap()
            .to_owned();
        let one = handle(
            &format!(
                "GET /management/project/v1/conversation.record?project_id={project_id}&record_id={record_id}"
            ),
            b"",
            &store,
        );
        assert_eq!(one.status, 200, "{}", one.body);
        assert!(one.body.contains("composer note"));
        let task_append = handle(
            "POST /task/project/v1/conversation.append",
            json!({
                "project_id": project_id,
                "employee_id": ids[1],
                "kind": "note",
                "body": "task must fail"
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(task_append.status, 403);
        let legacy = handle(
            "POST /management/project/v1/speech.candidate",
            json!({
                "project_id": project_id,
                "employee_id": ids[1],
                "kind": "deliverable",
                "projection_id": "cognitiveos.personal.conversation-projection/0.1",
                "body": "must not coerce"
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(legacy.status, 422, "{}", legacy.body);
        let v01 = handle(
            &format!(
                "GET /management/project/v1/conversation.archive?project_id={project_id}&limit=32&projection_id=v01"
            ),
            b"",
            &store,
        );
        assert_eq!(v01.status, 422, "{}", v01.body);
        let task = handle(
            &format!("GET /task/project/v1/conversation.archive?project_id={project_id}"),
            b"",
            &store,
        );
        assert_eq!(task.status, 403);
    }

    #[test]
    fn assistant_turn_registers_candidate_and_omits_approve() {
        let (_tmp, store) = authority();
        let plane = ProjectAggregateStore::from_authority_store(&store);
        let (draft_id, _) = plane.create_draft(b"payload", 1).unwrap();
        let unlabeled = handle(
            "POST /management/project/v1/assistant.turn",
            json!({
                "kind": "propose",
                "draft_id": draft_id,
                "object_kind": "charter",
                "payload": {"title": "x"},
                "provenance": "notes"
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(unlabeled.status, 422, "{}", unlabeled.body);
        let closed = handle(
            "POST /management/project/v1/assistant.turn",
            json!({
                "kind": "propose",
                "draft_id": draft_id,
                "object_kind": "recipe",
                "payload": {"title": "x"},
                "provenance": {"kind": "owner-stated"},
                "grant": "workspace-write"
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(closed.status, 422, "{}", closed.body);
        let ambient = handle(
            "POST /management/project/v1/assistant.turn",
            json!({
                "kind": "explain",
                "draft_id": draft_id,
                "object_kind": "business-brief",
                "payload": {"title": "x"},
                "provenance": {"kind": "assistant-assumption"},
                "tools": ["bash"]
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(ambient.status, 403, "{}", ambient.body);
        let task = handle(
            "POST /task/project/v1/assistant.turn",
            json!({
                "kind": "propose",
                "draft_id": draft_id,
                "object_kind": "charter",
                "provenance": {"kind": "owner-stated"}
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(task.status, 403);
        let proposed = handle(
            "POST /management/project/v1/assistant.turn",
            json!({
                "kind": "propose",
                "draft_id": draft_id,
                "object_kind": "charter",
                "payload": {"title": "research charter"},
                "provenance": {"kind": "owner-stated"}
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(proposed.status, 200, "{}", proposed.body);
        assert!(proposed.body.contains("candidate_digest"));
        assert!(proposed.body.contains("preview_id"));
        assert!(proposed.body.contains(ASSISTANT_ENGINE_ID));
        assert!(proposed.body.contains("\"installed_agent\":false"));
        assert!(!proposed.body.contains("Approve"));
        assert!(!proposed.body.contains("preview_digest"));
        let (g1_draft, _) = plane.create_draft(b"g1", 2).unwrap();
        plane.put_draft_charter(&g1_draft, b"charter", 3).unwrap();
        let (preview_id, digest) = plane
            .request_preview("activation", &g1_draft, b"bytes", 4)
            .unwrap();
        let project_id = plane
            .confirm_preview(ConfirmCaller::OwnerManagement, &preview_id, &digest, 5)
            .unwrap()
            .new_ref;
        let authority_apply = handle(
            "POST /management/project/v1/draft.apply",
            json!({
                "draft_id": project_id,
                "base_seq": 0,
                "candidate_digest": "abcd"
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(authority_apply.status, 422, "{}", authority_apply.body);
        assert!(authority_apply.body.contains("authority"));
    }
}
