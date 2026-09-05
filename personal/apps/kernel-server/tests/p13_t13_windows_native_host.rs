//! P13-T13 D01: unsigned Windows native host path on a real `kernel-server --personal`.
//!
//! The unrendered bootstrap template must fail closed (exit 64) before any
//! network or filesystem side effect. A cargo-built daemon on this host must
//! admit a Windows path ending in `Personal Home`, bind, and serve `/ui/`.
//! GNU/WSL/Linux roots and the task channel stay fail-closed.
//!
//! This is implementation evidence on whatever Windows host runs it
//! (`DEV-WINDOWS-NATIVE-OPC-01` locally; `CI-WINDOWS-MSVC-01` in required CI).
//! It is not Gate, release, Profile, B01-W, or a signed installer.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(not(windows), allow(unused))]

mod common;

use serde_json::Value;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

static P13_T13_PROCESS_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("kernel-server must be nested under the repository root")
        .to_path_buf()
}

fn runtime_root() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "cos-p13t13-rt-{}-{}",
        std::process::id(),
        free_port()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn spawn_personal(port: u16, runtime_root: &Path) -> Child {
    Command::new(env!("CARGO_BIN_EXE_kernel-server"))
        .args([
            "--personal",
            "--bind",
            &format!("127.0.0.1:{port}"),
            "--runtime-root",
            runtime_root.to_str().unwrap(),
        ])
        .spawn()
        .unwrap()
}

fn request(port: u16, wire: &str) -> String {
    let mut stream = common::connect_when_ready(port);
    stream
        .set_read_timeout(Some(Duration::from_secs(20)))
        .unwrap();
    stream.write_all(wire.as_bytes()).unwrap();
    stream.shutdown(std::net::Shutdown::Write).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

fn issue_token(port: u16, secret: &str, channel: &str) -> String {
    let body = format!(
        "{{\"channel\":\"{channel}\",\"principal_id\":\"principal://local/owner\",\"bootstrap_secret\":\"{secret}\"}}"
    );
    let response = request(
        port,
        &format!(
            "POST /local/session HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    );
    assert!(
        response.contains("HTTP/1.1 200"),
        "session must succeed without leaking the bootstrap secret"
    );
    assert!(
        !response.contains(secret),
        "session response must not echo the bootstrap secret"
    );
    let marker = "\"token\":\"";
    let start = response.find(marker).expect("token") + marker.len();
    let end = start + response[start..].find('"').unwrap();
    response[start..end].to_owned()
}

fn send_json(port: u16, method_path: &str, token: &str, body: &str) -> String {
    request(
        port,
        &format!(
            "POST {method_path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    )
}

fn get(port: u16, path: &str, token: Option<&str>) -> String {
    let auth = token.map_or(String::new(), |token| {
        format!("Authorization: Bearer {token}\r\n")
    });
    request(
        port,
        &format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\n{auth}Connection: close\r\n\r\n"),
    )
}

fn response_json(response: &str) -> Value {
    let (_, body) = response
        .split_once("\r\n\r\n")
        .expect("HTTP response has a header/body separator");
    serde_json::from_str(body).expect("HTTP response body is JSON")
}

struct Hermetic {
    daemon: Child,
    root: PathBuf,
    home: PathBuf,
}

impl Drop for Hermetic {
    fn drop(&mut self) {
        let _ = self.daemon.kill();
        let _ = self.daemon.wait();
        let _ = std::fs::remove_dir_all(&self.root);
        let _ = std::fs::remove_dir_all(&self.home);
    }
}

#[cfg(not(windows))]
#[test]
fn p13_t13_live_unsigned_host_path_is_windows_only() {
    assert!(
        !cfg!(windows),
        "this compile-only gate records that live unsigned-host E2E is cfg(windows)"
    );
}

#[cfg(windows)]
mod windows_native {
    use super::*;
    use std::fs;
    use std::process::Output;

    fn system_powershell() -> PathBuf {
        let system_root = std::env::var_os("SystemRoot").expect("SystemRoot must exist");
        Path::new(&system_root)
            .join("System32")
            .join("WindowsPowerShell")
            .join("v1.0")
            .join("powershell.exe")
    }

    fn run_unrendered_bootstrap() -> Output {
        let observed_temp = std::env::temp_dir().join(format!(
            "cos-p13t13-bootstrap-temp-{}-{}",
            std::process::id(),
            super::free_port()
        ));
        fs::create_dir_all(&observed_temp).unwrap();
        let script = super::repository_root().join("personal/deploy/windows/install.ps1");
        let output = Command::new(system_powershell())
            .arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-File")
            .arg(&script)
            .env("TEMP", &observed_temp)
            .env("TMP", &observed_temp)
            .output()
            .unwrap();
        let leftover: Vec<_> = fs::read_dir(&observed_temp)
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        let _ = fs::remove_dir_all(&observed_temp);
        assert!(
            leftover.is_empty(),
            "unrendered bootstrap must not create temp side effects"
        );
        output
    }

    #[test]
    fn unrendered_unsigned_bootstrap_rejects_before_network_or_temp_side_effect() {
        let output = run_unrendered_bootstrap();
        assert_eq!(
            output.status.code(),
            Some(64),
            "unrendered unsigned path must fail closed"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("release policy is not rendered"),
            "unsigned path must name the unrendered policy, not a usable installer"
        );
    }

    #[test]
    fn unsigned_daemon_admits_windows_personal_home_and_serves_ui() {
        let _guard = P13_T13_PROCESS_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let port = super::free_port();
        let root = super::runtime_root();
        let home = std::env::temp_dir().join(format!(
            "cos-p13t13-home-{}-{}\\Personal Home",
            std::process::id(),
            port
        ));
        fs::create_dir_all(home.join("app")).unwrap();
        fs::create_dir_all(home.join("data")).unwrap();
        let daemon = super::spawn_personal(port, &root);
        let mut hermetic = Hermetic {
            daemon,
            root,
            home: home.clone(),
        };
        let secret = common::wait_for_bootstrap_secret_from(&mut hermetic.daemon, &hermetic.root);
        let management = super::issue_token(port, &secret, "management");
        let task = super::issue_token(port, &secret, "task");

        let gnu_rejected = super::send_json(
            port,
            "/management/host/v1/home.admit",
            &management,
            r#"{"install_root":"/home/owner/Personal Home","app_dir":"/home/owner/Personal Home/app","data_dir":"/home/owner/Personal Home/data","acl_policy":"owner-only-dacl"}"#,
        );
        assert!(
            gnu_rejected.contains("HTTP/1.1 422"),
            "GNU/Linux roots must fail closed"
        );
        assert!(gnu_rejected.contains("GNU/WSL/Linux") || gnu_rejected.contains("HOST_REJECTED"));

        let task_forbidden = super::send_json(
            port,
            "/task/host/v1/home.admit",
            &task,
            &format!(
                r#"{{"install_root":{},"app_dir":{},"data_dir":{},"acl_policy":"owner-only-dacl"}}"#,
                serde_json::to_string(home.to_str().unwrap()).unwrap(),
                serde_json::to_string(home.join("app").to_str().unwrap()).unwrap(),
                serde_json::to_string(home.join("data").to_str().unwrap()).unwrap()
            ),
        );
        assert!(task_forbidden.contains("HTTP/1.1 403"));
        assert!(task_forbidden.contains("WINDOWS_HOST_CHANNEL_FORBIDDEN"));

        let install_root = home.to_str().unwrap();
        let app_dir = home.join("app");
        let data_dir = home.join("data");
        let admit_body = serde_json::json!({
            "install_root": install_root,
            "app_dir": app_dir.to_str().unwrap(),
            "data_dir": data_dir.to_str().unwrap(),
            "acl_policy": "owner-only-dacl"
        })
        .to_string();
        let admitted = super::send_json(
            port,
            "/management/host/v1/home.admit",
            &management,
            &admit_body,
        );
        assert!(
            admitted.contains("HTTP/1.1 200"),
            "unsigned daemon must admit a Windows Personal Home"
        );
        let admitted_json = super::response_json(&admitted);
        let home_id = admitted_json["home_id"].as_str().expect("home_id");
        assert_eq!(admitted_json["tray_proves_work"], false);

        let bound = super::send_json(
            port,
            "/management/host/v1/daemon.bind",
            &management,
            &serde_json::json!({
                "home_id": home_id,
                "can_honor_background": true
            })
            .to_string(),
        );
        assert!(bound.contains("HTTP/1.1 200"), "daemon.bind must succeed");
        let bound_json = super::response_json(&bound);
        assert_eq!(bound_json["tray_role"], "observe-and-request");
        assert_eq!(bound_json["tray_proves_work"], false);

        let status = super::get(
            port,
            &format!("/management/host/v1/status?home_id={home_id}"),
            Some(&management),
        );
        assert!(status.contains("HTTP/1.1 200"));
        let status_json = super::response_json(&status);
        assert_eq!(status_json["daemon_state"], "bound");
        assert_eq!(status_json["tray_proves_work"], false);

        let offline = super::send_json(
            port,
            "/management/host/v1/offline.record",
            &management,
            &serde_json::json!({
                "home_id": home_id,
                "cause": "sleep"
            })
            .to_string(),
        );
        assert!(
            offline.contains("HTTP/1.1 200"),
            "typed host.offline.record(sleep) is not an OS sleep"
        );
        assert!(
            super::response_json(&offline)["missed_visible"]
                .as_bool()
                .unwrap()
        );

        let missing_ui = super::get(port, "/ui/", None);
        assert!(
            missing_ui.contains("HTTP/1.1 503"),
            "unsigned cargo daemon without a /ui/ bundle must fail closed, not pretend to serve chrome"
        );
        assert!(
            missing_ui.contains("LOCAL_UI_BUNDLE_UNAVAILABLE")
                || missing_ui.contains("not_available")
                || missing_ui.contains("unavailable"),
            "missing bundle must be named, not a fake 200"
        );

        // `--runtime-root` layout appends `cognitiveos` (PersonalDataLayout).
        let ui_dir = hermetic.root.join("data").join("cognitiveos").join("ui");
        fs::create_dir_all(&ui_dir).unwrap();
        fs::write(
            ui_dir.join("index.html"),
            "<!doctype html><title>P13-T13 unsigned /ui/</title><p>personal-home</p>",
        )
        .unwrap();
        let ui = super::get(port, "/ui/", None);
        let ui_status = ui.lines().next().unwrap_or("");
        assert!(
            ui.contains("HTTP/1.1 200"),
            "unsigned daemon must serve product origin /ui/ once the bundle exists; got {ui_status}"
        );
        assert!(ui.contains("personal-home"));
        assert!(!ui.to_ascii_lowercase().contains("sk-"));
        assert!(!ui.contains(&secret));
    }
}
