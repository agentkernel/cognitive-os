//! True TCP process-level M5 HTTP JSON + SSE evidence (ADR-0003).
#![allow(clippy::unwrap_used, clippy::expect_used)]
use cognitive_contracts::generated::akp_request_envelope::SCHEMA_DIGEST;
use serde_json::json;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    process::Command,
    time::Duration,
};
fn port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}
fn request(port: u16, wire: &str) -> String {
    let mut stream = loop {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(s) => break s,
            Err(_) => std::thread::sleep(Duration::from_millis(20)),
        }
    };
    stream.write_all(wire.as_bytes()).unwrap();
    stream.shutdown(std::net::Shutdown::Write).unwrap();
    let mut out = String::new();
    stream.read_to_string(&mut out).unwrap();
    out
}
fn spawn(port: u16) -> std::process::Child {
    Command::new(env!("CARGO_BIN_EXE_kernel-server"))
        .args(["--once", "--bind", &format!("127.0.0.1:{port}")])
        .spawn()
        .unwrap()
}
#[test]
fn management_post_returns_authoritative_akp_result_and_error_envelopes() {
    let p = port();
    let mut child = spawn(p);
    let body=json!({"message_id":"m1","operation":"management.inspect","protocol_version":"cognitiveos.akp/0.2","schema_digest":SCHEMA_DIGEST,"sender":"principal://a","audience":"service://kernel/management","correlation_id":"c1","deadline":"2026-07-21T01:00:00Z","payload":{"target":"agent-execution://1"}}).to_string();
    let wire = format!(
        "POST /management/inspect HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let response = request(p, &wire);
    assert!(response.contains("HTTP/1.1 200"));
    assert!(response.contains("\"status\":\"ok\""));
    child.wait().unwrap();
    let p = port();
    let mut child = spawn(p);
    let bad = body.replace(SCHEMA_DIGEST, &format!("sha256:{}", "0".repeat(64)));
    let wire = format!(
        "POST /management/inspect HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        bad.len(),
        bad
    );
    let response = request(p, &wire);
    assert!(response.contains("\"status\":\"error\""));
    assert!(response.contains("PROTOCOL_SCHEMA_DIGEST_MISMATCH"));
    child.wait().unwrap();
}
#[test]
fn watch_endpoint_streams_snapshot_then_ordered_delta_frames() {
    let p = port();
    let mut child = spawn(p);
    let response = request(
        p,
        "GET /task/watch HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    );
    assert!(response.contains("Content-Type: text/event-stream"));
    assert!(response.contains("\"kind\":\"snapshot\""));
    assert!(response.contains("\"kind\":\"delta\""));
    assert!(
        response.find("\"kind\":\"snapshot\"").unwrap()
            < response.find("\"kind\":\"delta\"").unwrap()
    );
    child.wait().unwrap();
}
