//! P13-T11 reflection HTTP. Nested from `project_aggregate` so T08-owned
//! `server.rs` / `mod.rs` stay untouched. Task-channel aliases are 403.
//! Owner canvas Confirm (`POST …/confirm`) is the only Apply path.

use cognitive_store::{
    ConfirmCaller, MEMBER_RUNTIME_SUBJECT_KIND, ProjectAggregateError, ProjectAggregateStore,
    REFLECTION_PROJECTION_ID, ROLE_TEMPLATE_SUBJECT_KIND, ReflectionCandidateRow, ReflectionStore,
    RuntimeImprovementRow, RuntimeImprovementSpec, SqliteAuthorityStore,
};
use serde_json::{Value, json};

use super::super::resource_api::ResourceApiResponse;
use super::{error, now_ms, ok, parse_json, store_error};

const TASK: &str = "/task/project/v1/";

const LITERALS: &[&str] = &[
    "POST /management/project/v1/reflection.generate",
    "GET /management/project/v1/reflection.list",
    "POST /management/project/v1/reflection.improve.propose",
    "POST /management/project/v1/reflection.improve.confirm",
    "POST /management/project/v1/reflection.improve.rollback",
    "POST /management/project/v1/reflection.role-template.propose",
    "POST /management/project/v1/reflection.role-template.confirm",
    "POST /management/project/v1/reflection.admit-self-report",
    "POST /management/project/v1/reflection.as-completion",
    "POST /management/project/v1/reflection.inject-attempt",
    "POST /management/project/v1/reflection.reuse-member",
    "POST /task/project/v1/reflection.generate",
    "GET /task/project/v1/reflection.list",
    "POST /task/project/v1/reflection.improve.propose",
    "POST /task/project/v1/reflection.improve.confirm",
    "POST /task/project/v1/reflection.improve.rollback",
    "POST /task/project/v1/reflection.role-template.propose",
    "POST /task/project/v1/reflection.role-template.confirm",
    "POST /task/project/v1/reflection.admit-self-report",
    "POST /task/project/v1/reflection.as-completion",
    "POST /task/project/v1/reflection.inject-attempt",
    "POST /task/project/v1/reflection.reuse-member",
];

pub(crate) fn matches(method_path: &str) -> bool {
    literal(method_path).is_some()
}

pub(crate) fn is_task_channel(method_path: &str) -> bool {
    literal(method_path).is_some_and(|item| item.contains(TASK))
}

/// Canvas `POST /confirm` applies Member Runtime / Role Template previews.
/// Returns `None` when the preview is a different subject.
pub(crate) fn confirm_if_owned(
    body: &[u8],
    store: &SqliteAuthorityStore,
) -> Option<ResourceApiResponse> {
    let document = parse_json(body)?;
    let preview_id = document.get("preview_id").and_then(Value::as_str)?;
    let preview_digest = document.get("preview_digest").and_then(Value::as_str)?;
    let plane = ProjectAggregateStore::from_authority_store(store);
    let detail = match plane.preview_detail(preview_id) {
        Ok(Some(row)) => row,
        Ok(None) => return None,
        Err(_) => return None,
    };
    let reflections = ReflectionStore::from_authority_store(store);
    match detail.subject_kind.as_str() {
        kind if kind == MEMBER_RUNTIME_SUBJECT_KIND => Some(
            match reflections.confirm_runtime_improvement(
                ConfirmCaller::OwnerManagement,
                preview_id,
                preview_digest,
                now_ms(),
            ) {
                Ok(row) => ok(improvement_json(&row, "confirmed")),
                Err(error) => store_error(error),
            },
        ),
        kind if kind == ROLE_TEMPLATE_SUBJECT_KIND => Some(
            match reflections.confirm_role_template_preview(
                ConfirmCaller::OwnerManagement,
                preview_id,
                preview_digest,
                now_ms(),
            ) {
                Ok(proposal_id) => ok(json!({
                    "status": "ok",
                    "projection": REFLECTION_PROJECTION_ID,
                    "proposal_id": proposal_id,
                    "state": "confirmed",
                    "copied_employee": false,
                    "granted": false,
                })),
                Err(error) => store_error(error),
            },
        ),
        _ => None,
    }
}

pub(crate) fn handle(
    method_path: &str,
    body: &[u8],
    store: &SqliteAuthorityStore,
) -> ResourceApiResponse {
    let Some(literal) = literal(method_path) else {
        return error(
            404,
            "PROJECT_AGGREGATE_ROUTE_NOT_FOUND",
            "no reflection route matched",
        );
    };
    let reflections = ReflectionStore::from_authority_store(store);
    let name = literal
        .strip_prefix("POST /management/project/v1/")
        .or_else(|| literal.strip_prefix("GET /management/project/v1/"))
        .unwrap_or(literal);
    match name {
        "reflection.generate" => generate(body, &reflections),
        "reflection.list" => list(method_path, &reflections),
        "reflection.improve.propose" => propose_improve(body, &reflections),
        "reflection.improve.confirm" => confirm_improve(body, &reflections),
        "reflection.improve.rollback" => rollback_improve(body, &reflections),
        "reflection.role-template.propose" => propose_role(body, &reflections),
        "reflection.role-template.confirm" => confirm_role(body, &reflections),
        "reflection.admit-self-report" => admit_self_report(body, &reflections),
        "reflection.as-completion" => as_completion(body, &reflections),
        "reflection.inject-attempt" => inject_attempt(body, &reflections),
        "reflection.reuse-member" => reuse_member(body, &reflections),
        _ => error(
            404,
            "PROJECT_AGGREGATE_ROUTE_NOT_FOUND",
            "no reflection route matched",
        ),
    }
}

fn literal(method_path: &str) -> Option<&'static str> {
    LITERALS
        .iter()
        .copied()
        .find(|item| method_path.starts_with(item))
}

fn generate(body: &[u8], reflections: &ReflectionStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(project_id) = document.get("project_id").and_then(Value::as_str) else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    match reflections.generate_from_facts(project_id, now_ms()) {
        Ok(rows) => ok(json!({
            "status": "ok",
            "projection": REFLECTION_PROJECTION_ID,
            "generated": rows.iter().map(candidate_json).collect::<Vec<_>>(),
            "completion_claimed": false,
            "model_self_report": false,
        })),
        Err(error) => store_error(error),
    }
}

fn list(method_path: &str, reflections: &ReflectionStore) -> ResourceApiResponse {
    let Some(project_id) = query_parameter(method_path, "project_id").filter(|v| !v.is_empty())
    else {
        return error(400, "PROJECT_ID_REQUIRED", "list requires project_id");
    };
    let employee_id = query_parameter(method_path, "employee_id").filter(|v| !v.is_empty());
    match reflections.list_candidates(&project_id) {
        Ok(rows) => {
            let filtered: Vec<_> = rows
                .into_iter()
                .filter(|row| {
                    employee_id
                        .as_deref()
                        .is_none_or(|wanted| row.employee_id == wanted)
                })
                .collect();
            ok(json!({
                "status": "ok",
                "projection": REFLECTION_PROJECTION_ID,
                "candidates": filtered.iter().map(candidate_json).collect::<Vec<_>>(),
            }))
        }
        Err(error) => store_error(error),
    }
}

fn propose_improve(body: &[u8], reflections: &ReflectionStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(candidate_id) = document.get("candidate_id").and_then(Value::as_str) else {
        return error(400, "CANDIDATE_ID_REQUIRED", "candidate_id required");
    };
    let Some(proposed_prompt) = document.get("proposed_prompt").and_then(Value::as_str) else {
        return error(400, "PROPOSED_PROMPT_REQUIRED", "proposed_prompt required");
    };
    let tools = document
        .get("proposed_tools")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let blueprint = document
        .get("new_blueprint_revision_id")
        .and_then(Value::as_str);
    match reflections.propose_runtime_improvement(
        ConfirmCaller::OwnerManagement,
        &RuntimeImprovementSpec {
            candidate_id,
            proposed_prompt,
            proposed_tools: &tools,
            new_blueprint_revision_id: blueprint,
            now_ms: now_ms(),
        },
    ) {
        Ok(row) => ok(improvement_json(&row, "preview")),
        Err(error) => store_error(error),
    }
}

fn confirm_improve(body: &[u8], reflections: &ReflectionStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(preview_id) = document.get("preview_id").and_then(Value::as_str) else {
        return error(400, "PREVIEW_ID_REQUIRED", "preview_id required");
    };
    let Some(preview_digest) = document.get("preview_digest").and_then(Value::as_str) else {
        return error(400, "PREVIEW_DIGEST_REQUIRED", "preview_digest required");
    };
    match reflections.confirm_runtime_improvement(
        ConfirmCaller::OwnerManagement,
        preview_id,
        preview_digest,
        now_ms(),
    ) {
        Ok(row) => ok(improvement_json(&row, "confirmed")),
        Err(error) => store_error(error),
    }
}

fn rollback_improve(body: &[u8], reflections: &ReflectionStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(improvement_id) = document.get("improvement_id").and_then(Value::as_str) else {
        return error(400, "IMPROVEMENT_ID_REQUIRED", "improvement_id required");
    };
    match reflections.rollback_runtime_improvement(
        ConfirmCaller::OwnerManagement,
        improvement_id,
        now_ms(),
    ) {
        Ok(row) => ok(improvement_json(&row, "rolled-back")),
        Err(error) => store_error(error),
    }
}

fn propose_role(body: &[u8], reflections: &ReflectionStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(employee_id) = document.get("employee_id").and_then(Value::as_str) else {
        return error(400, "EMPLOYEE_ID_REQUIRED", "employee_id required");
    };
    match reflections.propose_role_template(ConfirmCaller::OwnerManagement, employee_id, now_ms()) {
        Ok((proposal_id, preview_id)) => ok(json!({
            "status": "ok",
            "projection": REFLECTION_PROJECTION_ID,
            "proposal_id": proposal_id,
            "preview_id": preview_id,
            "copied_employee": false,
            "granted": false,
        })),
        Err(error) => store_error(error),
    }
}

fn confirm_role(body: &[u8], reflections: &ReflectionStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(preview_id) = document.get("preview_id").and_then(Value::as_str) else {
        return error(400, "PREVIEW_ID_REQUIRED", "preview_id required");
    };
    let Some(preview_digest) = document.get("preview_digest").and_then(Value::as_str) else {
        return error(400, "PREVIEW_DIGEST_REQUIRED", "preview_digest required");
    };
    match reflections.confirm_role_template_preview(
        ConfirmCaller::OwnerManagement,
        preview_id,
        preview_digest,
        now_ms(),
    ) {
        Ok(proposal_id) => ok(json!({
            "status": "ok",
            "projection": REFLECTION_PROJECTION_ID,
            "proposal_id": proposal_id,
            "state": "confirmed",
            "copied_employee": false,
        })),
        Err(error) => store_error(error),
    }
}

fn admit_self_report(body: &[u8], reflections: &ReflectionStore) -> ResourceApiResponse {
    let document = parse_json(body).unwrap_or_else(|| json!({}));
    let project_id = document
        .get("project_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    let prose = document.get("body").and_then(Value::as_str).unwrap_or("");
    store_error(
        match reflections.admit_model_self_report(project_id, prose) {
            Err(error) => error,
            Ok(()) => ProjectAggregateError::Rejected {
                detail: "model self-report is not a Member Runtime improvement",
            },
        },
    )
}

fn as_completion(body: &[u8], reflections: &ReflectionStore) -> ResourceApiResponse {
    let document = parse_json(body).unwrap_or_else(|| json!({}));
    let candidate_id = document
        .get("candidate_id")
        .and_then(Value::as_str)
        .unwrap_or("reflect-none");
    store_error(
        match reflections.claim_reflection_is_completion(candidate_id) {
            Err(error) => error,
            Ok(()) => ProjectAggregateError::Rejected {
                detail: "reflection is never completion",
            },
        },
    )
}

fn inject_attempt(body: &[u8], reflections: &ReflectionStore) -> ResourceApiResponse {
    let document = parse_json(body).unwrap_or_else(|| json!({}));
    let attempt_id = document
        .get("attempt_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    let prompt = document
        .get("proposed_prompt")
        .and_then(Value::as_str)
        .unwrap_or("");
    store_error(
        match reflections.overwrite_running_attempt_context(attempt_id, prompt) {
            Err(error) => error,
            Ok(()) => ProjectAggregateError::Rejected {
                detail: "silent prompt injection into a running Attempt is refused",
            },
        },
    )
}

fn reuse_member(body: &[u8], reflections: &ReflectionStore) -> ResourceApiResponse {
    let document = parse_json(body).unwrap_or_else(|| json!({}));
    let employee_id = document
        .get("employee_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    let other = document
        .get("other_project_id")
        .and_then(Value::as_str)
        .unwrap_or("");
    store_error(
        match reflections.reuse_member_in_other_project(employee_id, other) {
            Err(error) => error,
            Ok(()) => ProjectAggregateError::Forbidden {
                detail: "silent cross-Project Member reuse is refused",
            },
        },
    )
}

fn candidate_json(row: &ReflectionCandidateRow) -> Value {
    json!({
        "candidate_id": row.candidate_id,
        "project_id": row.project_id,
        "employee_id": row.employee_id,
        "kind": row.kind,
        "source": row.source,
        "attempt_id": row.attempt_id,
        "evidence_id": row.evidence_id,
        "fact_digest": row.fact_digest,
        "completion_claimed": row.completion_claimed,
    })
}

fn improvement_json(row: &RuntimeImprovementRow, result: &str) -> Value {
    json!({
        "status": "ok",
        "projection": REFLECTION_PROJECTION_ID,
        "result": result,
        "improvement_id": row.improvement_id,
        "candidate_id": row.candidate_id,
        "employee_id": row.employee_id,
        "base_revision_id": row.base_revision_id,
        "applied_revision_id": row.applied_revision_id,
        "preview_id": row.preview_id,
        "preview_digest": row.preview_digest,
        "state": row.state,
        "granted": false,
        "implicit_blueprint": false,
        "chat_can_confirm": false,
    })
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
    use super::super::handle;
    use cognitive_store::{
        ConfirmCaller, EmployeeStore, HOSTED_DSH_ARTIFACT_DIGEST, HostedArtifactObservation,
        HostedAttemptFrameSpec, HostedAttemptIntentSpec, HostedAttemptTerminalSpec,
        HostedDshAttemptStore, HostedDshPlane, PersonalDataLayout, ProjectAggregateStore,
        RosterProposal, SqliteAuthorityStore, StageSpec, prepare_personal_databases,
    };
    use serde_json::{Value, json};
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

    fn seated_project(store: &SqliteAuthorityStore) -> (String, String) {
        let plane = ProjectAggregateStore::from_authority_store(store);
        let employees = EmployeeStore::from_authority_store(store);
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
                &[stage("s1", "Manage", "manager")],
                5,
            )
            .unwrap();
        let ids = employees
            .register_roster(
                ConfirmCaller::OwnerManagement,
                &project_id,
                &plan_id,
                &[RosterProposal {
                    slot: "manager".into(),
                    specialization: "project-manager".into(),
                    prompt: "coordinate".into(),
                    tools_declared: vec![],
                }],
                6,
            )
            .unwrap();
        employees
            .request_seating(ConfirmCaller::OwnerManagement, &ids[0], 7)
            .unwrap();
        employees
            .confirm_seating(
                ConfirmCaller::OwnerManagement,
                &ids[0],
                Some("flash"),
                true,
                8,
            )
            .unwrap();
        (project_id, ids[0].clone())
    }

    fn failed_terminal(store: &SqliteAuthorityStore, employee_id: &str) -> Option<String> {
        let employees = EmployeeStore::from_authority_store(store);
        let attempts = HostedDshAttemptStore::from_authority_store(store);
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
        let revision = employees.latest_revision_id(employee_id).ok().flatten()?;
        let outcome = attempts.persist_intent(
            ConfirmCaller::OwnerManagement,
            &HostedAttemptIntentSpec {
                employee_id,
                employee_revision_id: &revision,
                task_ref: "task://personal/p13-t11-http",
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
        attempts
            .mark_dispatched(&attempt_id, Some("dshchild-p13-t11-http"), 4242, 50)
            .expect("dispatched");
        attempts
            .record_frames(
                &attempt_id,
                &[HostedAttemptFrameSpec {
                    seq: 1,
                    kind: "response".to_owned(),
                    operation: None,
                    payload_digest: None,
                    reject_reason: None,
                    text_redacted: "failed".to_owned(),
                }],
                55,
            )
            .expect("frames");
        attempts
            .record_terminal(
                &attempt_id,
                &HostedAttemptTerminalSpec {
                    terminal_kind: "exited",
                    exit_code: Some(6),
                    response_status: Some("failed"),
                    candidate_count: 0,
                    observation_count: 0,
                    rejected_frame_count: 0,
                    unknown_line_count: 0,
                    stdout_bytes: 64,
                    stdout_truncated: false,
                    stderr_tail_redacted: "",
                    elapsed_ms: 12,
                    now_ms: 60,
                },
            )
            .expect("terminal");
        Some(attempt_id)
    }

    #[test]
    fn http_reflection_task_channel_and_negatives_are_refused() {
        let (_tmp, store) = authority();
        let (project_id, employee_id) = seated_project(&store);
        let forbidden = handle(
            "POST /task/project/v1/reflection.generate",
            json!({"project_id": project_id}).to_string().as_bytes(),
            &store,
        );
        assert_eq!(forbidden.status, 403, "{}", forbidden.body);
        let self_report = handle(
            "POST /management/project/v1/reflection.admit-self-report",
            json!({"project_id": project_id, "body": "I improved myself"})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(self_report.status, 422, "{}", self_report.body);
        assert!(self_report.body.contains("self-report"));
        let completion = handle(
            "POST /management/project/v1/reflection.as-completion",
            json!({"candidate_id": "reflect-none"})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(completion.status, 422, "{}", completion.body);
        assert!(completion.body.contains("never completion"));
        let reuse = handle(
            "POST /management/project/v1/reflection.reuse-member",
            json!({"employee_id": employee_id, "other_project_id": "project-other"})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(reuse.status, 403, "{}", reuse.body);
        let blueprint = handle(
            "POST /management/project/v1/reflection.improve.propose",
            json!({
                "candidate_id": "missing",
                "proposed_prompt": "tighten",
                "new_blueprint_revision_id": "role-blueprint-rev:forged"
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(blueprint.status, 422, "{}", blueprint.body);
        assert!(blueprint.body.contains("Blueprint"));
    }

    #[test]
    fn http_reflection_generate_confirm_via_canvas_and_rollback() {
        let (_tmp, store) = authority();
        let (project_id, employee_id) = seated_project(&store);
        let Some(_) = failed_terminal(&store, &employee_id) else {
            return;
        };
        let generated = handle(
            "POST /management/project/v1/reflection.generate",
            json!({"project_id": project_id}).to_string().as_bytes(),
            &store,
        );
        assert_eq!(generated.status, 200, "{}", generated.body);
        assert!(generated.body.contains("incident"), "{}", generated.body);
        assert!(generated.body.contains("daily"), "{}", generated.body);
        assert!(!generated.body.contains("\"completion_claimed\":true"));
        let listed = handle(
            &format!(
                "GET /management/project/v1/reflection.list?project_id={project_id}&employee_id={employee_id}"
            ),
            b"",
            &store,
        );
        assert_eq!(listed.status, 200, "{}", listed.body);
        let listed_json = serde_json::from_str::<Value>(&listed.body).unwrap();
        let candidate_id = listed_json["candidates"][0]["candidate_id"]
            .as_str()
            .expect("candidate");
        let proposed = handle(
            "POST /management/project/v1/reflection.improve.propose",
            json!({
                "candidate_id": candidate_id,
                "proposed_prompt": "tighten research",
                "proposed_tools": ["workspace-write"]
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(proposed.status, 200, "{}", proposed.body);
        assert!(proposed.body.contains("chat_can_confirm"));
        let proposed_json = serde_json::from_str::<Value>(&proposed.body).unwrap();
        let preview_id = proposed_json["preview_id"].as_str().unwrap();
        let preview_digest = proposed_json["preview_digest"].as_str().unwrap();
        let improvement_id = proposed_json["improvement_id"].as_str().unwrap();
        let confirmed = handle(
            "POST /management/project/v1/confirm",
            json!({"preview_id": preview_id, "preview_digest": preview_digest})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(confirmed.status, 200, "{}", confirmed.body);
        assert!(
            confirmed.body.contains("confirmed") || confirmed.body.contains("active"),
            "{}",
            confirmed.body
        );
        assert!(confirmed.body.contains("applied_revision_id"));
        let rolled = handle(
            "POST /management/project/v1/reflection.improve.rollback",
            json!({"improvement_id": improvement_id})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(rolled.status, 200, "{}", rolled.body);
        assert!(rolled.body.contains("rolled-back"));
    }
}
