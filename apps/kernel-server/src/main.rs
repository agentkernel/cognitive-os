//! `kernel-server`: single-node composition root of the CognitiveOS
//! reference implementation (M5/M6 delivery; M0 skeleton HTTP surface).
//!
//! Readiness is graded, never binary (whitepaper section 16 and the
//! readiness case of M6): `MANAGEMENT_READY` (deterministic management and
//! recovery verbs available) before `USER_READY` (task channel accepts
//! intents) before `OPERATIONAL` (full governed execution). Machine carrier
//! for readiness remains absent (D-021); grades below mirror
//! `cognitive_runtime::ReadinessGrade`.

use cognitive_contracts::generated::akp_request_envelope::SCHEMA_DIGEST;
use cognitive_runtime::ReadinessGrade;
use serde_json::json;
use std::io::{Read, Write};
use std::net::TcpListener;

fn readiness_grades() -> [&'static str; 3] {
    [
        ReadinessGrade::ManagementReady.as_str(),
        ReadinessGrade::UserReady.as_str(),
        ReadinessGrade::Operational.as_str(),
    ]
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--once") {
        let bind = args
            .iter()
            .position(|arg| arg == "--bind")
            .and_then(|i| args.get(i + 1))
            .map_or("127.0.0.1:0", String::as_str);
        if let Err(error) = serve_once(bind) {
            eprintln!("kernel-server: {error}");
            std::process::exit(1);
        }
        return;
    }
    println!(
        "kernel-server M0 skeleton: no server is started. Readiness grades: {}.",
        readiness_grades().join(" -> ")
    );
    println!(
        "Layers wired: contracts={}, domains={}, ports={}, store={}, runtime={}, mgmt-fallback={:?}, akp={}",
        cognitive_contracts::ENCODING_PROFILE,
        cognitive_domain::EXECUTION_LIFECYCLE_DOMAINS.len(),
        cognitive_kernel::KERNEL_PORTS.len(),
        cognitive_store::STORE_BACKEND,
        cognitive_runtime::RUNTIME_ROLE,
        cognitive_management::DETERMINISTIC_FALLBACK_VERBS,
        cognitive_akp::TRANSPORT_PROFILE,
    );
}

fn serve_once(bind: &str) -> Result<(), String> {
    let listener = TcpListener::bind(bind).map_err(|e| e.to_string())?;
    let (mut stream, _) = listener.accept().map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 4096];
    loop {
        let read = stream.read(&mut chunk).map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..read]);
        if let Some(split) = find(&bytes, b"\r\n\r\n") {
            let headers = String::from_utf8_lossy(&bytes[..split]);
            let length = headers
                .lines()
                .find_map(|line| {
                    line.strip_prefix("Content-Length: ")
                        .and_then(|v| v.parse::<usize>().ok())
                })
                .unwrap_or(0);
            if bytes.len() >= split + 4 + length {
                break;
            }
        }
    }
    let split = find(&bytes, b"\r\n\r\n").ok_or("malformed HTTP request")?;
    let head = String::from_utf8_lossy(&bytes[..split]);
    let request_line = head.lines().next().ok_or("missing request line")?;
    let body = &bytes[split + 4..];
    let (content_type, response) = if request_line.starts_with("GET /task/watch ") {
        let mut log = cognitive_akp::WatchLog::new("watch-http-1", 16);
        log.append(json!({"event_id":"event-1","state":"RUNNABLE"}))
            .map_err(|e| e.to_string())?;
        let frames = log.open(json!({"objects":[]})).map_err(|e| e.to_string())?;
        let mut s = String::new();
        for frame in frames {
            let data = serde_json::to_string(&frame).map_err(|e| e.to_string())?;
            s.push_str("data: ");
            s.push_str(&data);
            s.push_str("\n\n");
        }
        ("text/event-stream", s)
    } else if request_line.starts_with("POST /management/") {
        match cognitive_akp::parse_request(body, SCHEMA_DIGEST) {
            Ok(request) => {
                let value = cognitive_akp::result_ok(
                    &request,
                    json!({"operation":request.operation,"management_ready":true}),
                )
                .map_err(|e| e.to_string())?;
                (
                    "application/json",
                    serde_json::to_string(&value).map_err(|e| e.to_string())?,
                )
            }
            Err(error) => {
                let value = json!({"in_reply_to":"unknown","correlation_id":"unknown","protocol_version":"cognitiveos.akp/0.2","status":"error","error":{"code":error.code(),"category":"protocol","retryable":false,"stage":"envelope"}});
                (
                    "application/json",
                    serde_json::to_string(&value).map_err(|e| e.to_string())?,
                )
            }
        }
    } else if request_line.starts_with("POST /shell/") {
        // Deterministic shell surface (non-authority). Full session state is
        // process-local in this --once reference profile; semantics are proven
        // in `cognitive-runtime` unit tests.
        let op = request_line
            .trim_start_matches("POST /shell/")
            .split_whitespace()
            .next()
            .unwrap_or("");
        let value = match op {
            "detach" => {
                json!({"status":"ok","phase":"detached","cancelled":false,"authority":false})
            }
            "cancel" => json!({"status":"CANCEL_PENDING","authority":false}),
            "attach" => json!({"status":"ok","phase":"attached","authority":false}),
            _ => {
                json!({"status":"error","error":{"code":"SCHEMA_MISMATCH","category":"protocol","retryable":false,"stage":"routing"}})
            }
        };
        (
            "application/json",
            serde_json::to_string(&value).map_err(|e| e.to_string())?,
        )
    } else {
        ("application/json",json!({"status":"error","error":{"code":"SCHEMA_MISMATCH","category":"protocol","retryable":false,"stage":"routing"}}).to_string())
    };
    let wire = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",
        response.len()
    );
    stream
        .write_all(wire.as_bytes())
        .map_err(|e| e.to_string())?;
    Ok(())
}
fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readiness_grades_are_ordered_management_first() {
        let grades = readiness_grades();
        assert_eq!(grades[0], "MANAGEMENT_READY");
        assert_eq!(grades.len(), 3);
        assert_eq!(grades[2], "OPERATIONAL");
    }
}
