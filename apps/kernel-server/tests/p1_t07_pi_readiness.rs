//! P1-T07 Pi readiness component evidence.
//!
//! Drives the real Personal daemon over its real HTTP front door and asserts
//! that the `pi` component now reports observed facts instead of a hard-coded
//! placeholder — and that it never turns a configuration file into a readiness
//! claim it has not observed.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

static PERSONAL_PROCESS_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn runtime_root(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "cos-p1t07-{}-{}-{}",
        label,
        std::process::id(),
        free_port()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

/// `--runtime-root <dir>` nests the XDG config root at `<dir>/config`, and the
/// layout appends the `cognitiveos` product directory to it.
fn personal_config_dir(runtime_root: &std::path::Path) -> std::path::PathBuf {
    runtime_root.join("config").join("cognitiveos")
}

fn spawn_personal(port: u16, runtime_root: &std::path::Path) -> Child {
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

fn wait_connect(port: u16) -> TcpStream {
    for _ in 0..100 {
        if let Ok(stream) = TcpStream::connect(("127.0.0.1", port)) {
            return stream;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("personal daemon did not accept connections on {port}");
}

fn http_exchange(port: u16, wire: &str) -> String {
    let mut stream = wait_connect(port);
    let _ = stream.write_all(wire.as_bytes());
    let _ = stream.shutdown(std::net::Shutdown::Write);
    let mut out = String::new();
    let _ = stream.read_to_string(&mut out);
    out
}

fn bootstrap_secret(runtime_root: &std::path::Path) -> String {
    let path = runtime_root
        .join("cognitiveos")
        .join("local-bootstrap.secret");
    for _ in 0..100 {
        if let Ok(contents) = std::fs::read_to_string(&path) {
            let trimmed = contents.trim().to_owned();
            if !trimmed.is_empty() {
                return trimmed;
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("bootstrap secret not found at {}", path.display());
}

fn issue_management_token(port: u16, secret: &str) -> String {
    let body = format!(
        "{{\"channel\":\"management\",\"principal_id\":\"principal://local/owner\",\"bootstrap_secret\":\"{secret}\"}}"
    );
    let wire = format!(
        "POST /local/session HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let response = http_exchange(port, &wire);
    assert!(response.contains("HTTP/1.1 200"), "{response}");
    let token_key = "\"token\":\"";
    let start = response.find(token_key).expect("token field") + token_key.len();
    let end = response[start..].find('"').expect("token end") + start;
    response[start..end].to_owned()
}

fn doctor(port: u16, token: &str) -> String {
    let wire = format!(
        "GET /personal/doctor HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nConnection: close\r\n\r\n"
    );
    http_exchange(port, &wire)
}

fn json_escape_path(path: &std::path::Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}

#[test]
fn pi_component_reports_observed_facts_over_the_real_front_door() {
    let _guard = PERSONAL_PROCESS_TEST_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    // 1. No pi.json at all: the component reports absence, exactly as before
    //    P1-T07 introduced a real probe.
    let root = runtime_root("pi-absent");
    let port = free_port();
    let mut child = spawn_personal(port, &root);
    let secret = bootstrap_secret(&root);
    let token = issue_management_token(port, &secret);
    let absent = doctor(port, &token);
    let _ = child.kill();
    let _ = child.wait();

    assert!(absent.contains("HTTP/1.1 200"), "{absent}");
    assert!(absent.contains("\"component\":\"pi\""), "{absent}");
    assert!(absent.contains("\"pi_not_configured\""), "{absent}");
    assert!(absent.contains("\"not_configured\""), "{absent}");
    assert!(
        absent.contains("\"first_conversation_ready\":false"),
        "{absent}"
    );
    // The probe must never claim containment or a Gate from a readiness read.
    assert!(absent.contains("\"containment_claim\""), "{absent}");
    assert!(
        absent.contains("\"gate_claim\":\"not-claimed\""),
        "{absent}"
    );
    let _ = std::fs::remove_dir_all(&root);

    // 2. A pi.json pointing at paths that do not exist must be Blocked, not
    //    Ready. Writing a configuration file is not evidence that Pi is
    //    installed.
    let root = runtime_root("pi-configured");
    let config_dir = personal_config_dir(&root);
    std::fs::create_dir_all(&config_dir).unwrap();
    let missing_executable = config_dir.join("pi-not-installed");
    let missing_extension = config_dir.join("extension-not-built.js");
    let document = format!(
        "{{\"schema_version\":1,\"surface\":\"personal-pi-config\",\"executable_path\":\"{}\",\"extension_entry_path\":\"{}\"}}",
        json_escape_path(&missing_executable),
        json_escape_path(&missing_extension)
    );
    std::fs::write(config_dir.join("pi.json"), document).unwrap();

    let port = free_port();
    let mut child = spawn_personal(port, &root);
    let secret = bootstrap_secret(&root);
    let token = issue_management_token(port, &secret);
    let configured = doctor(port, &token);
    let _ = child.kill();
    let _ = child.wait();

    assert!(configured.contains("HTTP/1.1 200"), "{configured}");
    assert!(
        configured.contains("\"pi_executable_missing\""),
        "a configuration file must not be mistaken for an installed Pi: {configured}"
    );
    assert!(
        configured.contains("\"first_conversation_ready\":false"),
        "{configured}"
    );
    assert!(
        !configured.contains("\"gate_claim\":\"passed\""),
        "{configured}"
    );
    let _ = std::fs::remove_dir_all(&root);

    // 3. A corrupt pi.json is blocked with its own class rather than silently
    //    falling back to `not_configured`.
    let root = runtime_root("pi-corrupt");
    let config_dir = personal_config_dir(&root);
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(config_dir.join("pi.json"), "{ not json").unwrap();

    let port = free_port();
    let mut child = spawn_personal(port, &root);
    let secret = bootstrap_secret(&root);
    let token = issue_management_token(port, &secret);
    let corrupt = doctor(port, &token);
    let _ = child.kill();
    let _ = child.wait();

    assert!(corrupt.contains("HTTP/1.1 200"), "{corrupt}");
    assert!(corrupt.contains("\"pi_config_unusable\""), "{corrupt}");
    assert!(
        corrupt.contains("\"first_conversation_ready\":false"),
        "{corrupt}"
    );
    let _ = std::fs::remove_dir_all(&root);
}
