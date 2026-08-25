//! P9-T07 daemon 侧 nested observation：相关头只影响响应头，不改产品 body，
//! 也不写入权威库。本文件只走未配置 Provider 的拒绝路径，因此观测头不会出现在
//! 成功响应上——那正是「失败仍保持 header-only、授权开关不能改写错误 body」的证明。
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, Command};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;

static PERSONAL_PROCESS_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

const VALID_CORRELATION_ID: &str = "campaign-0123456789abcdef0123456789abcdef";
const SECRET_SHAPED_CORRELATION: &str = "sk-0123456789abcdef0123456789abcdef";

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn create_runtime_root() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "cos-p9t05-route-observation-{}-{}",
        std::process::id(),
        free_port()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn spawn_personal_daemon(port: u16, runtime_root: &Path, observation_enabled: bool) -> Child {
    let mut command = Command::new(env!("CARGO_BIN_EXE_kernel-server"));
    command.args([
        "--personal",
        "--bind",
        &format!("127.0.0.1:{port}"),
        "--runtime-root",
        runtime_root.to_str().unwrap(),
    ]);
    // 父进程环境不得把插桩授权泄漏进子 daemon。
    command.env_remove("COGNITIVEOS_PI_ROUTE_OBSERVATION");
    command.env_remove("COGNITIVEOS_PI_ROUTE_OBSERVATION_CAMPAIGN");
    command.env_remove("COGNITIVEOS_PI_ROUTE_OBSERVATION_SINK");
    if observation_enabled {
        command.env("COGNITIVEOS_PI_ROUTE_OBSERVATION", "enabled");
    }
    command.spawn().unwrap()
}

fn wait_for_connection(port: u16) -> TcpStream {
    for _ in 0..100 {
        if let Ok(stream) = TcpStream::connect(("127.0.0.1", port)) {
            return stream;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("personal daemon did not accept connections on {port}");
}

fn exchange_http_request(port: u16, request: &str) -> String {
    let mut stream = wait_for_connection(port);
    stream
        .set_read_timeout(Some(Duration::from_secs(20)))
        .unwrap();
    stream.write_all(request.as_bytes()).unwrap();
    stream.shutdown(std::net::Shutdown::Write).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

fn http_parts(response: &str) -> (&str, &str) {
    let (headers, body) = response
        .split_once("\r\n\r\n")
        .expect("HTTP response must separate headers from body");
    (headers, body)
}

fn read_bootstrap_secret(runtime_root: &Path) -> String {
    let path = runtime_root
        .join("cognitiveos")
        .join("local-bootstrap.secret");
    for _ in 0..500 {
        if let Ok(contents) = std::fs::read_to_string(&path) {
            let secret = contents.trim();
            if !secret.is_empty() {
                return secret.to_owned();
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("bootstrap secret not found at {}", path.display());
}

fn issue_management_token(port: u16, bootstrap_secret: &str) -> String {
    let body = format!(
        "{{\"channel\":\"management\",\"principal_id\":\"principal://local/owner\",\"bootstrap_secret\":\"{bootstrap_secret}\"}}"
    );
    let request = format!(
        "POST /local/session HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let response = exchange_http_request(port, &request);
    assert!(response.contains("HTTP/1.1 200"), "{response}");
    let token_key = "\"token\":\"";
    let token_start = response.find(token_key).expect("token field") + token_key.len();
    let token_end = response[token_start..].find('"').expect("token end") + token_start;
    response[token_start..token_end].to_owned()
}

fn proxy_request(token: &str, extra_headers: &str) -> String {
    let request_body = "{\"model\":\"test-model\",\"stream\":false,\"messages\":[]}";
    format!(
        "POST /provider/v1/chat/completions HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{extra_headers}Connection: close\r\n\r\n{request_body}",
        request_body.len()
    )
}

fn assert_no_observation_headers(headers: &str) {
    let lower = headers.to_ascii_lowercase();
    assert!(
        !lower.contains("x-cognitiveos-correlation-id"),
        "error path must not echo a correlation id: {headers}"
    );
    assert!(
        !lower.contains("x-cognitiveos-daemon-preflight-nanos"),
        "error path must not report a preflight stage: {headers}"
    );
}

fn run_header_only_front_door(observation_enabled: bool) {
    let _guard = PERSONAL_PROCESS_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let port = free_port();
    let runtime_root = create_runtime_root();
    let mut daemon = spawn_personal_daemon(port, &runtime_root, observation_enabled);
    let bootstrap_secret = read_bootstrap_secret(&runtime_root);
    let management_token = issue_management_token(port, &bootstrap_secret);

    let baseline = exchange_http_request(port, &proxy_request(&management_token, ""));
    let valid = exchange_http_request(
        port,
        &proxy_request(
            &management_token,
            &format!("X-CognitiveOS-Correlation-Id: {VALID_CORRELATION_ID}\r\n"),
        ),
    );
    let malformed = exchange_http_request(
        port,
        &proxy_request(
            &management_token,
            &format!("X-CognitiveOS-Correlation-Id: {SECRET_SHAPED_CORRELATION}\r\n"),
        ),
    );
    let duplicated = exchange_http_request(
        port,
        &proxy_request(
            &management_token,
            &format!(
                "X-CognitiveOS-Correlation-Id: {VALID_CORRELATION_ID}\r\nx-cognitiveos-correlation-id: {VALID_CORRELATION_ID}\r\n"
            ),
        ),
    );

    let (baseline_headers, baseline_body) = http_parts(&baseline);
    let (valid_headers, valid_body) = http_parts(&valid);
    let (malformed_headers, malformed_body) = http_parts(&malformed);
    let (duplicated_headers, duplicated_body) = http_parts(&duplicated);

    assert!(
        baseline.contains("PERSONAL_PROVIDER_NOT_CONFIGURED"),
        "{baseline}"
    );
    assert_eq!(valid_body, baseline_body);
    assert_eq!(malformed_body, baseline_body);
    assert_eq!(duplicated_body, baseline_body);

    assert_no_observation_headers(baseline_headers);
    assert_no_observation_headers(valid_headers);
    assert_no_observation_headers(malformed_headers);
    assert_no_observation_headers(duplicated_headers);

    assert!(
        !malformed.contains(SECRET_SHAPED_CORRELATION),
        "refused correlation value leaked into the response: {malformed}"
    );
    assert!(
        !valid.contains(&management_token),
        "session credential leaked in proxy response: {valid}"
    );
    assert!(
        !valid.contains(&bootstrap_secret),
        "bootstrap secret leaked in proxy response: {valid}"
    );

    // 观测模块不得成为第二 writer：runtime 根下不得出现观测或 campaign 文件。
    let listing = std::fs::read_dir(runtime_root.join("cognitiveos"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    assert!(
        listing.iter().all(|name| {
            let lossy = name.to_string_lossy();
            !lossy.contains("observation") && !lossy.contains("campaign")
        }),
        "observation must not create an authority-adjacent file: {listing:?}"
    );

    daemon.kill().unwrap();
    daemon.wait().unwrap();
    let _ = std::fs::remove_dir_all(&runtime_root);
}

#[test]
fn unauthorized_daemon_keeps_the_proxy_error_body_identical_across_correlation_headers() {
    run_header_only_front_door(false);
}

#[test]
fn authorized_daemon_still_leaves_the_proxy_error_body_and_store_untouched() {
    run_header_only_front_door(true);
}
