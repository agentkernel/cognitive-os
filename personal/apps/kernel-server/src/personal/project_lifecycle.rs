//! P13-T09 Project lifecycle HTTP (forwarded from closed T06 `project_chat`).
//!
//! Management-only copy / archive / delete.preview / delete.confirm /
//! restore-point / export / lifecycle. Task-channel aliases are 403. Chat has
//! no Approve. Settings chrome is T08 and is not used here.

use cognitive_store::project_aggregate::{
    LifecycleArchiveSpec, LifecycleCopySpec, LifecycleDeleteConfirmSpec,
    LifecycleDeletePreviewSpec, LifecycleExportSpec, LifecycleRestoreSpec, ProjectLifecycleStore,
};
use cognitive_store::{ConfirmCaller, SqliteAuthorityStore};
use serde_json::{Value, json};

use crate::personal::project_aggregate::{error, now_ms, ok, parse_json, store_error};
use crate::personal::resource_api::ResourceApiResponse;

const ROUTE_LITERALS: &[&str] = &[
    "POST /management/project/v1/copy",
    "POST /management/project/v1/archive",
    "POST /management/project/v1/delete.preview",
    "POST /management/project/v1/delete.confirm",
    "POST /management/project/v1/restore-point",
    "POST /management/project/v1/export",
    "GET /management/project/v1/lifecycle",
    "POST /task/project/v1/copy",
    "POST /task/project/v1/archive",
    "POST /task/project/v1/delete.preview",
    "POST /task/project/v1/delete.confirm",
    "POST /task/project/v1/restore-point",
    "POST /task/project/v1/export",
    "GET /task/project/v1/lifecycle",
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

pub(crate) fn handle(
    method_path: &str,
    body: &[u8],
    store: &SqliteAuthorityStore,
) -> ResourceApiResponse {
    let Some((channel, literal)) = parse_route(method_path) else {
        return error(
            404,
            "PROJECT_LIFECYCLE_ROUTE_NOT_FOUND",
            "no Project lifecycle route matched",
        );
    };
    if channel == Channel::Task {
        return error(
            403,
            "PROJECT_LIFECYCLE_CHANNEL_FORBIDDEN",
            "Project lifecycle is management-channel only",
        );
    }
    let lifecycle = ProjectLifecycleStore::from_authority_store(store);
    match literal {
        "POST /management/project/v1/copy" => copy(body, &lifecycle),
        "POST /management/project/v1/archive" => archive(body, &lifecycle),
        "POST /management/project/v1/delete.preview" => delete_preview(body, &lifecycle),
        "POST /management/project/v1/delete.confirm" => delete_confirm(body, &lifecycle),
        "POST /management/project/v1/restore-point" => restore_point(body, &lifecycle),
        "POST /management/project/v1/export" => export(body, &lifecycle),
        "GET /management/project/v1/lifecycle" => lifecycle_get(method_path, &lifecycle),
        _ => error(
            404,
            "PROJECT_LIFECYCLE_ROUTE_NOT_FOUND",
            "no Project lifecycle route matched",
        ),
    }
}

fn parse_route(method_path: &str) -> Option<(Channel, &'static str)> {
    let without_query = method_path.split('?').next().unwrap_or(method_path).trim();
    for literal in ROUTE_LITERALS {
        if without_query == *literal {
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

fn copy(body: &[u8], lifecycle: &ProjectLifecycleStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    if secret_shaped(&document) {
        return error(422, "PROJECT_INVALID", "export excludes secrets");
    }
    if bool_flag(&document, "inherit_grants")
        || bool_flag(&document, "inherit_seats")
        || bool_flag(&document, "inherit_runtime")
    {
        return error(422, "PROJECT_REJECTED", "copy excludes grants and seating");
    }
    let Some(source) = document.get("project_id").and_then(Value::as_str) else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    match lifecycle.copy_project(LifecycleCopySpec {
        caller: ConfirmCaller::OwnerManagement,
        source_project_id: source,
        inherit_grants: bool_flag(&document, "inherit_grants"),
        inherit_seats: bool_flag(&document, "inherit_seats"),
        inherit_runtime: bool_flag(&document, "inherit_runtime"),
        now_ms: now_ms(),
    }) {
        Ok(copy_id) => ok(json!({
            "status": "ok",
            "copy_project_id": copy_id,
            "state": "inactive",
            "inherited_grants": false,
            "inherited_seats": false,
            "inherited_runtime": false,
        })),
        Err(err) => store_error(err),
    }
}

fn archive(body: &[u8], lifecycle: &ProjectLifecycleStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    if bool_flag(&document, "skip_stop_triggers") {
        return error(422, "PROJECT_REJECTED", "archive must stop live triggers");
    }
    let Some(project_id) = document.get("project_id").and_then(Value::as_str) else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    match lifecycle.archive_project(LifecycleArchiveSpec {
        caller: ConfirmCaller::OwnerManagement,
        project_id,
        skip_stop_triggers: bool_flag(&document, "skip_stop_triggers"),
        now_ms: now_ms(),
    }) {
        Ok(view) => ok(json!({
            "status": "ok",
            "project_id": view.project_id,
            "state": view.state,
            "triggers_stopped": true,
            "paused_armings": view.paused_armings,
            "is_disaster_backup": false,
        })),
        Err(err) => store_error(err),
    }
}

fn delete_preview(body: &[u8], lifecycle: &ProjectLifecycleStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(project_id) = document.get("project_id").and_then(Value::as_str) else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    match lifecycle.preview_delete(LifecycleDeletePreviewSpec {
        caller: ConfirmCaller::OwnerManagement,
        project_id,
        now_ms: now_ms(),
    }) {
        Ok(preview) => ok(json!({
            "status": "ok",
            "preview_id": format!("delete-preview:{project_id}"),
            "confirmation_digest": preview.impact_digest,
            "routines": preview.routines,
            "members": preview.members,
            "outputs": preview.outputs,
            "grants": preview.grants,
            "triggers_stopped": preview.armed_triggers == 0,
            "physical_delete": false,
        })),
        Err(err) => store_error(err),
    }
}

fn delete_confirm(body: &[u8], lifecycle: &ProjectLifecycleStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(project_id) = document.get("project_id").and_then(Value::as_str) else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    if bool_flag(&document, "physical_delete") {
        return error(422, "PROJECT_REJECTED", "physical delete is forbidden");
    }
    if document.get("second_confirm").and_then(Value::as_bool) != Some(true) {
        return error(
            422,
            "PROJECT_REJECTED",
            "delete requires second confirmation",
        );
    }
    let digest = document
        .get("confirmation_digest")
        .and_then(Value::as_str)
        .or_else(|| document.get("impact_digest").and_then(Value::as_str));
    let Some(digest) = digest else {
        return error(
            400,
            "CONFIRMATION_DIGEST_REQUIRED",
            "confirmation_digest required",
        );
    };
    match lifecycle.confirm_delete(LifecycleDeleteConfirmSpec {
        caller: ConfirmCaller::OwnerManagement,
        project_id,
        impact_digest: digest,
        second_confirm: bool_flag(&document, "second_confirm"),
        physical_delete: bool_flag(&document, "physical_delete"),
        now_ms: now_ms(),
    }) {
        Ok(preview) => ok(json!({
            "status": "ok",
            "project_id": project_id,
            "logically_deleted": preview.tombstoned,
            "physical_delete": false,
        })),
        Err(err) => store_error(err),
    }
}

fn restore_point(body: &[u8], lifecycle: &ProjectLifecycleStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    let Some(project_id) = document.get("project_id").and_then(Value::as_str) else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    match lifecycle.record_restore_point(LifecycleRestoreSpec {
        caller: ConfirmCaller::OwnerManagement,
        project_id,
        home_id: document.get("home_id").and_then(Value::as_str),
        claimed_as_backup: bool_flag(&document, "claimed_as_backup")
            || bool_flag(&document, "is_disaster_backup"),
        now_ms: now_ms(),
    }) {
        Ok(point) => ok(json!({
            "status": "ok",
            "event_id": point.restore_point_id,
            "version_name": point.kind,
            "kind": point.kind,
            "same_disk": point.same_disk,
            "is_disaster_backup": false,
            "claimed_as_backup": false,
        })),
        Err(err) => store_error(err),
    }
}

fn export(body: &[u8], lifecycle: &ProjectLifecycleStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "PROJECT_JSON_REQUIRED", "JSON body required");
    };
    if secret_shaped(&document) {
        return error(422, "PROJECT_INVALID", "export excludes secrets");
    }
    let Some(project_id) = document.get("project_id").and_then(Value::as_str) else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    match lifecycle.export_project(LifecycleExportSpec {
        caller: ConfirmCaller::OwnerManagement,
        project_id,
        include_secrets: bool_flag(&document, "include_secrets"),
        now_ms: now_ms(),
    }) {
        Ok(exported) => ok(json!({
            "status": "ok",
            "export_id": exported.project_id,
            "relative_path": exported.path,
            "is_authority": exported.is_authority,
            "is_backup": exported.is_backup,
            "include_secrets": false,
        })),
        Err(err) => store_error(err),
    }
}

fn lifecycle_get(method_path: &str, lifecycle: &ProjectLifecycleStore) -> ResourceApiResponse {
    let Some(project_id) = query_parameter(method_path, "project_id") else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    match lifecycle.lifecycle_view(&project_id) {
        Ok(view) => ok(json!({
            "status": "ok",
            "project_id": view.project_id,
            "state": view.state,
            "data_dir": view.data_dir,
            "logically_deleted": view.tombstoned,
            "is_disaster_backup": false,
            "events": [],
            "restore_points": view.restore_points.iter().map(|point| json!({
                "event_id": point.restore_point_id,
                "version_name": point.kind,
                "created_at": 0,
                "is_disaster_backup": false,
            })).collect::<Vec<_>>(),
            "pending_delete_preview": view.pending_impact_digest.as_ref().map(|digest| json!({
                "preview_id": format!("delete-preview:{project_id}"),
                "confirmation_digest": digest,
                "triggers_stopped": true,
                "status": "pending",
            })),
        })),
        Err(err) => store_error(err),
    }
}

fn bool_flag(document: &Value, key: &str) -> bool {
    document.get(key).and_then(Value::as_bool).unwrap_or(false)
}

fn secret_shaped(document: &Value) -> bool {
    let lowered = document.to_string().to_ascii_lowercase();
    lowered.contains("api_key") || lowered.contains("sk-")
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
    use crate::personal::project_chat::{handle as chat_handle, matches as chat_matches};
    use cognitive_store::{
        ConfirmCaller, HomeAdmitSpec, PersonalDataLayout, ProjectAggregateStore, WindowsHostStore,
        prepare_personal_databases,
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

    fn admit_home(store: &SqliteAuthorityStore, root: &std::path::Path) -> String {
        let home = root.join("Personal Home");
        let app = home.join("app");
        let data = home.join("data");
        std::fs::create_dir_all(&app).expect("app");
        std::fs::create_dir_all(&data).expect("data");
        let home_s = home.to_string_lossy().into_owned();
        let app_s = app.to_string_lossy().into_owned();
        let data_s = data.to_string_lossy().into_owned();
        WindowsHostStore::from_authority_store(store)
            .admit_home(
                ConfirmCaller::OwnerManagement,
                &HomeAdmitSpec {
                    install_root: &home_s,
                    app_dir: &app_s,
                    data_dir: &data_s,
                    acl_policy: "owner-only-dacl",
                    argv: &[],
                    env_pairs: &[],
                    now_ms: 40,
                },
            )
            .expect("admit")
            .home_id
    }

    fn body_json(response: &ResourceApiResponse) -> Value {
        serde_json::from_str(&response.body).expect("json")
    }

    #[test]
    fn task_channel_aliases_are_forbidden() {
        let (_tmp, store) = authority();
        for path in [
            "POST /task/project/v1/copy",
            "POST /task/project/v1/archive",
            "POST /task/project/v1/delete.preview",
            "POST /task/project/v1/export",
            "GET /task/project/v1/lifecycle?project_id=x",
        ] {
            let response = chat_handle(path, br#"{"project_id":"x"}"#, &store);
            assert_eq!(response.status, 403, "{path}");
            assert!(
                response
                    .body
                    .contains("PROJECT_LIFECYCLE_CHANNEL_FORBIDDEN")
            );
        }
        assert!(chat_matches("POST /management/project/v1/copy"));
        assert!(chat_matches("POST /management/project/v1/copy "));
    }

    #[test]
    fn inherit_export_secret_and_backup_claims_are_refused() {
        let (_tmp, store) = authority();
        let project_id = activate(&store);
        let inherit = chat_handle(
            "POST /management/project/v1/copy",
            json!({"project_id": project_id, "inherit_grants": true})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(inherit.status, 422, "{}", inherit.body);
        let secrets = chat_handle(
            "POST /management/project/v1/export",
            json!({"project_id": project_id, "include_secrets": true})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(secrets.status, 422, "{}", secrets.body);
        let backup = chat_handle(
            "POST /management/project/v1/restore-point",
            json!({"project_id": project_id, "claimed_as_backup": true})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(backup.status, 422, "{}", backup.body);
        let live = chat_handle(
            "POST /management/project/v1/delete.preview",
            json!({"project_id": project_id}).to_string().as_bytes(),
            &store,
        );
        assert_eq!(live.status, 422, "{}", live.body);
        assert!(
            body_json(&live)["message"]
                .as_str()
                .unwrap()
                .contains("live triggers")
        );
    }

    #[test]
    fn management_copy_archive_delete_restore_export_round_trip() {
        let (tmp, store) = authority();
        let _home = admit_home(&store, tmp.path());
        let project_id = activate(&store);
        let copied = chat_handle(
            "POST /management/project/v1/copy ",
            json!({"project_id": project_id}).to_string().as_bytes(),
            &store,
        );
        assert_eq!(copied.status, 200, "{}", copied.body);
        let copy_id = body_json(&copied)["copy_project_id"]
            .as_str()
            .unwrap()
            .to_owned();
        assert_eq!(body_json(&copied)["state"], "inactive");
        let archived = chat_handle(
            "POST /management/project/v1/archive",
            json!({"project_id": project_id}).to_string().as_bytes(),
            &store,
        );
        assert_eq!(archived.status, 200, "{}", archived.body);
        let preview = chat_handle(
            "POST /management/project/v1/delete.preview",
            json!({"project_id": project_id}).to_string().as_bytes(),
            &store,
        );
        assert_eq!(preview.status, 200, "{}", preview.body);
        let digest = body_json(&preview)["confirmation_digest"]
            .as_str()
            .unwrap()
            .to_owned();
        let confirmed = chat_handle(
            "POST /management/project/v1/delete.confirm",
            json!({
                "project_id": project_id,
                "confirmation_digest": digest,
                "second_confirm": true
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(confirmed.status, 200, "{}", confirmed.body);
        assert_eq!(body_json(&confirmed)["physical_delete"], false);
        let restore = chat_handle(
            "POST /management/project/v1/restore-point",
            json!({"project_id": copy_id}).to_string().as_bytes(),
            &store,
        );
        assert_eq!(restore.status, 200, "{}", restore.body);
        assert_eq!(body_json(&restore)["is_disaster_backup"], false);
        let exported = chat_handle(
            "POST /management/project/v1/export",
            json!({"project_id": copy_id}).to_string().as_bytes(),
            &store,
        );
        assert_eq!(exported.status, 200, "{}", exported.body);
        assert_eq!(body_json(&exported)["is_authority"], false);
        let view = chat_handle(
            &format!("GET /management/project/v1/lifecycle?project_id={copy_id} "),
            b"",
            &store,
        );
        assert_eq!(view.status, 200, "{}", view.body);
        assert_eq!(body_json(&view)["is_disaster_backup"], false);
    }
}
