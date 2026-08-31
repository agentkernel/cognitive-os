//! Personal-private `/management/host/v1/*` projection (P11-T02).
//!
//! Tray/UI observes and requests typed lifecycle. Task-channel writes are 403.
//! Native Windows tray/ACL/sleep E2E is not claimed by this HTTP surface.

use cognitive_store::{
    CloseRequestSpec, ConfirmCaller, DaemonBindSpec, HomeAdmitSpec, ProjectAggregateError,
    SqliteAuthorityStore, WAKE_RECOVERY_STEPS, WINDOWS_HOST_PROJECTION_ID, WindowsHostDaemon,
    WindowsHostHome, WindowsHostRecovery, WindowsHostStatus, WindowsHostStore,
};
use serde_json::{Value, json};

use super::resource_api::ResourceApiResponse;

const ROUTE_LITERALS: &[&str] = &[
    "POST /management/host/v1/home.admit",
    "POST /management/host/v1/daemon.bind",
    "POST /management/host/v1/close.request",
    "POST /management/host/v1/offline.record",
    "POST /management/host/v1/dsh.bind",
    "POST /management/host/v1/recovery.run",
    "POST /management/host/v1/recovery.advance",
    "POST /management/host/v1/restore-point.record",
    "GET /management/host/v1/status",
    "POST /task/host/v1/home.admit",
    "POST /task/host/v1/daemon.bind",
    "POST /task/host/v1/close.request",
    "POST /task/host/v1/offline.record",
    "POST /task/host/v1/dsh.bind",
    "POST /task/host/v1/recovery.run",
    "POST /task/host/v1/recovery.advance",
    "POST /task/host/v1/restore-point.record",
    "GET /task/host/v1/status",
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
        "WINDOWS_HOST_CHANNEL_FORBIDDEN",
        "Windows host operations are management-channel only",
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
            "WINDOWS_HOST_ROUTE_NOT_FOUND",
            "no Windows host route matched",
        );
    };
    if channel == Channel::Task {
        return channel_forbidden();
    }
    match literal {
        "POST /management/host/v1/home.admit" => home_admit(body, store),
        "POST /management/host/v1/daemon.bind" => daemon_bind(body, store),
        "POST /management/host/v1/close.request" => close_request(body, store),
        "POST /management/host/v1/offline.record" => offline_record(body, store),
        "POST /management/host/v1/dsh.bind" => dsh_bind(body, store),
        "POST /management/host/v1/recovery.run" => recovery_run(body, store),
        "POST /management/host/v1/recovery.advance" => recovery_advance(body, store),
        "POST /management/host/v1/restore-point.record" => restore_point(body, store),
        "GET /management/host/v1/status" => status(method_path, store),
        _ => error(
            404,
            "WINDOWS_HOST_ROUTE_NOT_FOUND",
            "no Windows host route matched",
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

fn home_admit(body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "HOST_JSON_REQUIRED", "JSON body required");
    };
    let Some(install_root) = document.get("install_root").and_then(Value::as_str) else {
        return error(400, "INSTALL_ROOT_REQUIRED", "install_root required");
    };
    let Some(app_dir) = document.get("app_dir").and_then(Value::as_str) else {
        return error(400, "APP_DIR_REQUIRED", "app_dir required");
    };
    let Some(data_dir) = document.get("data_dir").and_then(Value::as_str) else {
        return error(400, "DATA_DIR_REQUIRED", "data_dir required");
    };
    let acl_policy = document
        .get("acl_policy")
        .and_then(Value::as_str)
        .unwrap_or("owner-only-dacl");
    let argv_owned = string_list(&document, "argv");
    let argv: Vec<&str> = argv_owned.iter().map(String::as_str).collect();
    let env_owned = env_pairs(&document);
    let env_refs: Vec<(&str, &str)> = env_owned
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    let host = WindowsHostStore::from_authority_store(store);
    match host.admit_home(
        ConfirmCaller::OwnerManagement,
        &HomeAdmitSpec {
            install_root,
            app_dir,
            data_dir,
            acl_policy,
            argv: &argv,
            env_pairs: &env_refs,
            now_ms: now_ms(),
        },
    ) {
        Ok(home) => ok(home_json(&home)),
        Err(error) => store_error(error),
    }
}

fn daemon_bind(body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "HOST_JSON_REQUIRED", "JSON body required");
    };
    let Some(home_id) = document.get("home_id").and_then(Value::as_str) else {
        return error(400, "HOME_ID_REQUIRED", "home_id required");
    };
    let can_honor_background = document
        .get("can_honor_background")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let host = WindowsHostStore::from_authority_store(store);
    match host.bind_daemon(
        ConfirmCaller::OwnerManagement,
        &DaemonBindSpec {
            home_id,
            can_honor_background,
            now_ms: now_ms(),
        },
    ) {
        Ok(daemon) => ok(daemon_json(&daemon)),
        Err(error) => store_error(error),
    }
}

fn close_request(body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "HOST_JSON_REQUIRED", "JSON body required");
    };
    let Some(home_id) = document.get("home_id").and_then(Value::as_str) else {
        return error(400, "HOME_ID_REQUIRED", "home_id required");
    };
    let Some(choice) = document.get("choice").and_then(Value::as_str) else {
        return error(400, "CLOSE_CHOICE_REQUIRED", "choice required");
    };
    let host = WindowsHostStore::from_authority_store(store);
    match host.request_close(
        ConfirmCaller::OwnerManagement,
        &CloseRequestSpec {
            home_id,
            choice,
            now_ms: now_ms(),
        },
    ) {
        Ok(daemon) => ok(daemon_json(&daemon)),
        Err(error) => store_error(error),
    }
}

fn offline_record(body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "HOST_JSON_REQUIRED", "JSON body required");
    };
    let Some(home_id) = document.get("home_id").and_then(Value::as_str) else {
        return error(400, "HOME_ID_REQUIRED", "home_id required");
    };
    let Some(cause) = document.get("cause").and_then(Value::as_str) else {
        return error(400, "OFFLINE_CAUSE_REQUIRED", "cause required");
    };
    let host = WindowsHostStore::from_authority_store(store);
    match host.record_offline(ConfirmCaller::OwnerManagement, home_id, cause, now_ms()) {
        Ok(segment_id) => ok(json!({
            "status": "ok",
            "projection_id": WINDOWS_HOST_PROJECTION_ID,
            "segment_id": segment_id,
            "missed_visible": true,
        })),
        Err(error) => store_error(error),
    }
}

fn dsh_bind(body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "HOST_JSON_REQUIRED", "JSON body required");
    };
    let Some(home_id) = document.get("home_id").and_then(Value::as_str) else {
        return error(400, "HOME_ID_REQUIRED", "home_id required");
    };
    let host = WindowsHostStore::from_authority_store(store);
    match host.bind_dsh_child(ConfirmCaller::OwnerManagement, home_id, now_ms()) {
        Ok(child_id) => ok(json!({
            "status": "ok",
            "projection_id": WINDOWS_HOST_PROJECTION_ID,
            "child_id": child_id,
        })),
        Err(error) => store_error(error),
    }
}

fn recovery_run(body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "HOST_JSON_REQUIRED", "JSON body required");
    };
    let Some(home_id) = document.get("home_id").and_then(Value::as_str) else {
        return error(400, "HOME_ID_REQUIRED", "home_id required");
    };
    let host_awake = document
        .get("host_awake")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let host = WindowsHostStore::from_authority_store(store);
    match host.run_ordered_recovery(
        ConfirmCaller::OwnerManagement,
        home_id,
        host_awake,
        now_ms(),
    ) {
        Ok(recovery) => ok(recovery_json(&recovery)),
        Err(error) => store_error(error),
    }
}

fn recovery_advance(body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "HOST_JSON_REQUIRED", "JSON body required");
    };
    let Some(home_id) = document.get("home_id").and_then(Value::as_str) else {
        return error(400, "HOME_ID_REQUIRED", "home_id required");
    };
    let Some(expected_step) = document.get("expected_step").and_then(Value::as_i64) else {
        return error(400, "RECOVERY_STEP_REQUIRED", "expected_step required");
    };
    let host = WindowsHostStore::from_authority_store(store);
    match host.advance_recovery(
        ConfirmCaller::OwnerManagement,
        home_id,
        expected_step,
        now_ms(),
    ) {
        Ok(recovery) => ok(recovery_json(&recovery)),
        Err(error) => store_error(error),
    }
}

fn restore_point(body: &[u8], store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "HOST_JSON_REQUIRED", "JSON body required");
    };
    let Some(home_id) = document.get("home_id").and_then(Value::as_str) else {
        return error(400, "HOME_ID_REQUIRED", "home_id required");
    };
    let claimed_as_backup = document
        .get("claimed_as_backup")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let kind = document
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("local-restore-point");
    let host = WindowsHostStore::from_authority_store(store);
    match host.record_restore_point(
        ConfirmCaller::OwnerManagement,
        home_id,
        claimed_as_backup,
        kind,
        now_ms(),
    ) {
        Ok(restore_point_id) => ok(json!({
            "status": "ok",
            "projection_id": WINDOWS_HOST_PROJECTION_ID,
            "restore_point_id": restore_point_id,
            "kind": "local-restore-point",
            "disaster_backup": false,
        })),
        Err(error) => store_error(error),
    }
}

fn status(method_path: &str, store: &SqliteAuthorityStore) -> ResourceApiResponse {
    let Some(home_id) = query_parameter(method_path, "home_id").filter(|value| !value.is_empty())
    else {
        return error(400, "HOME_ID_REQUIRED", "home_id required");
    };
    let host = WindowsHostStore::from_authority_store(store);
    match host.observe_status(&home_id, None) {
        Ok(status) => ok(status_json(&status)),
        Err(error) => store_error(error),
    }
}

fn home_json(home: &WindowsHostHome) -> Value {
    json!({
        "status": "ok",
        "projection_id": WINDOWS_HOST_PROJECTION_ID,
        "home_id": home.home_id,
        "install_root": home.install_root,
        "app_dir": home.app_dir,
        "data_dir": home.data_dir,
        "data_preserved": home.data_preserved,
        "app_replaced": home.app_replaced,
        "tray_proves_work": false,
    })
}

fn daemon_json(daemon: &WindowsHostDaemon) -> Value {
    json!({
        "status": "ok",
        "projection_id": WINDOWS_HOST_PROJECTION_ID,
        "daemon_id": daemon.daemon_id,
        "home_id": daemon.home_id,
        "epoch": daemon.epoch,
        "state": daemon.state,
        "can_honor_background": daemon.can_honor_background,
        "tray_role": daemon.tray_role,
        "tray_proves_work": daemon.tray_proves_work,
        "close_disposition": daemon.close_disposition,
    })
}

fn recovery_json(recovery: &WindowsHostRecovery) -> Value {
    json!({
        "status": "ok",
        "projection_id": WINDOWS_HOST_PROJECTION_ID,
        "recovery_id": recovery.recovery_id,
        "home_id": recovery.home_id,
        "epoch": recovery.epoch,
        "current_step": recovery.current_step,
        "current_step_name": recovery.current_step_name,
        "catch_up_asked": recovery.catch_up_asked,
        "resume_eligible": recovery.resume_eligible,
        "ordered_steps": WAKE_RECOVERY_STEPS,
    })
}

fn status_json(status: &WindowsHostStatus) -> Value {
    json!({
        "status": "ok",
        "projection_id": WINDOWS_HOST_PROJECTION_ID,
        "home_id": status.home_id,
        "install_root": status.install_root,
        "app_dir": status.app_dir,
        "data_dir": status.data_dir,
        "data_preserved": status.data_preserved,
        "daemon_id": status.daemon_id,
        "epoch": status.epoch,
        "daemon_state": status.daemon_state,
        "can_honor_background": status.can_honor_background,
        "tray_role": status.tray_role,
        "tray_proves_work": status.tray_proves_work,
        "close_disposition": status.close_disposition,
        "missed_segments": status.missed_segments,
        "recovery_step": status.recovery_step,
        "resume_eligible": status.resume_eligible,
        "restore_kind": status.restore_kind,
    })
}

fn string_list(document: &Value, field: &str) -> Vec<String> {
    document
        .get(field)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn env_pairs(document: &Value) -> Vec<(String, String)> {
    document
        .get("env")
        .and_then(Value::as_object)
        .map(|object| {
            object
                .iter()
                .filter_map(|(key, value)| {
                    value.as_str().map(|text| (key.clone(), text.to_owned()))
                })
                .collect()
        })
        .unwrap_or_default()
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
            return Some(value.split_whitespace().next().unwrap_or(value).to_owned());
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
        ProjectAggregateError::Forbidden { detail } => error(403, "HOST_FORBIDDEN", detail),
        ProjectAggregateError::NotFound { detail } => error(404, "HOST_NOT_FOUND", detail),
        ProjectAggregateError::Conflict { detail } => error(409, "HOST_CONFLICT", detail),
        ProjectAggregateError::Stale { detail } => error(409, "HOST_STALE", detail),
        ProjectAggregateError::Unconfirmed { detail }
        | ProjectAggregateError::Rejected { detail } => error(422, "HOST_REJECTED", detail),
        ProjectAggregateError::Invalid { detail } => error(422, "HOST_INVALID", detail),
        ProjectAggregateError::Unavailable { .. } => {
            error(503, "HOST_UNAVAILABLE", "store unavailable")
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use cognitive_store::{PersonalDataLayout, prepare_personal_databases};
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

    fn admit(store: &SqliteAuthorityStore) -> String {
        let response = handle(
            "POST /management/host/v1/home.admit",
            json!({
                "install_root": r"C:\Users\owner\Personal Home",
                "app_dir": r"C:\Users\owner\Personal Home\app",
                "data_dir": r"C:\Users\owner\Personal Home\data",
                "acl_policy": "owner-only-dacl"
            })
            .to_string()
            .as_bytes(),
            store,
        );
        assert_eq!(response.status, 200, "{}", response.body);
        serde_json::from_str::<Value>(&response.body).unwrap()["home_id"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    #[test]
    fn p11_t02_host_negatives_and_task_channel_is_forbidden() {
        let (_tmp, store) = authority();
        let forbidden = handle(
            "POST /task/host/v1/home.admit",
            json!({"install_root": r"C:\Users\owner\Personal Home"})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(forbidden.status, 403);
        assert!(forbidden.body.contains("WINDOWS_HOST_CHANNEL_FORBIDDEN"));

        let wrong = handle(
            "POST /management/host/v1/home.admit",
            json!({
                "install_root": r"C:\Program Files\CognitiveOS",
                "app_dir": r"C:\Program Files\CognitiveOS\app",
                "data_dir": r"C:\Program Files\CognitiveOS\data"
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(wrong.status, 422, "{}", wrong.body);
        assert!(wrong.body.contains("wrong install root"));

        let secret = handle(
            "POST /management/host/v1/home.admit",
            json!({
                "install_root": r"C:\Users\owner\Personal Home",
                "app_dir": r"C:\Users\owner\Personal Home\app",
                "data_dir": r"C:\Users\owner\Personal Home\data",
                "env": {"OPENAI_API_KEY": "sk-http"}
            })
            .to_string()
            .as_bytes(),
            &store,
        );
        assert_eq!(secret.status, 422, "{}", secret.body);
        assert!(secret.body.contains("secret must not enter env or argv"));
        assert!(!secret.body.contains("sk-http"));

        let home_id = admit(&store);
        let first = handle(
            "POST /management/host/v1/daemon.bind",
            json!({"home_id": home_id, "can_honor_background": false})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(first.status, 200, "{}", first.body);
        assert!(first.body.contains("\"tray_proves_work\":false"));
        let duplicate = handle(
            "POST /management/host/v1/daemon.bind",
            json!({"home_id": home_id, "can_honor_background": false})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(duplicate.status, 409, "{}", duplicate.body);
        assert!(duplicate.body.contains("duplicate daemon"));

        let fake = handle(
            "POST /management/host/v1/close.request",
            json!({"home_id": home_id, "choice": "background"})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(fake.status, 422, "{}", fake.body);
        assert!(fake.body.contains("fake background"));

        let orphan = handle(
            "POST /management/host/v1/dsh.bind",
            json!({"home_id": "missing-home"}).to_string().as_bytes(),
            &store,
        );
        assert_eq!(orphan.status, 422, "{}", orphan.body);
        assert!(orphan.body.contains("orphan DSH"));

        let backup = handle(
            "POST /management/host/v1/restore-point.record",
            json!({"home_id": home_id, "claimed_as_backup": true})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(backup.status, 422, "{}", backup.body);
        assert!(backup.body.contains("restore-as-backup"));

        let skip = handle(
            "POST /management/host/v1/recovery.advance",
            json!({"home_id": home_id, "expected_step": 7})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert!(skip.status == 404 || skip.status == 422, "{}", skip.body);

        let offline = handle(
            "POST /management/host/v1/offline.record",
            json!({"home_id": home_id, "cause": "sleep"})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(offline.status, 200, "{}", offline.body);
        let asleep = handle(
            "POST /management/host/v1/recovery.run",
            json!({"home_id": home_id, "host_awake": false})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(asleep.status, 422, "{}", asleep.body);
        let recovered = handle(
            "POST /management/host/v1/recovery.run",
            json!({"home_id": home_id, "host_awake": true})
                .to_string()
                .as_bytes(),
            &store,
        );
        assert_eq!(recovered.status, 200, "{}", recovered.body);
        assert!(recovered.body.contains("resume-eligible-only"));
        assert!(recovered.body.contains("\"resume_eligible\":true"));

        let status = handle(
            &format!("GET /management/host/v1/status?home_id={home_id}"),
            b"",
            &store,
        );
        assert_eq!(status.status, 200, "{}", status.body);
        assert!(status.body.contains("\"tray_proves_work\":false"));
        assert!(!status.body.contains("sk-"));
        assert!(!status.body.contains("ssv1:"));
    }
}
