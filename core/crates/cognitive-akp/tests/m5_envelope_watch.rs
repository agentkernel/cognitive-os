//! M5 AKP/watch behavior: REQ-AKP-ENV-001/002, REQ-AKP-CAN-001,
//! REQ-AKP-VER-001, REQ-AKP-CONT-001, REQ-AKP-STR-001/002 and REQ-SHELL-WATCH-001.
#![allow(clippy::unwrap_used)]
use cognitive_akp::{AkpError, WatchLog, parse_request, result_ok};
use cognitive_contracts::generated::akp_request_envelope::SCHEMA_DIGEST;
use serde_json::json;

fn request() -> serde_json::Value {
    json!({"message_id":"msg-1","operation":"management.inspect","protocol_version":"cognitiveos.akp/0.2","schema_digest":SCHEMA_DIGEST,"sender":"principal://tenant-a/alice","audience":"service://kernel/management","correlation_id":"corr-1","deadline":"2026-07-21T01:00:00Z","payload":{"target":"agent-execution://1"}})
}

#[test]
fn envelope_gates_version_extensions_schema_and_canonical_digest() {
    let parsed = parse_request(&serde_json::to_vec(&request()).unwrap(), SCHEMA_DIGEST).unwrap();
    let result = result_ok(&parsed, json!({"state":"RUNNABLE"})).unwrap();
    assert_eq!(result["in_reply_to"], "msg-1");
    assert!(
        result["result_digest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );
    let mut bad = request();
    bad["protocol_version"] = json!("cognitiveos.akp/1.0");
    assert_eq!(
        parse_request(&serde_json::to_vec(&bad).unwrap(), SCHEMA_DIGEST)
            .unwrap_err()
            .code(),
        "VERSION_UNSUPPORTED"
    );
    let mut bad = request();
    bad["schema_digest"] = json!(format!("sha256:{}", "0".repeat(64)));
    assert_eq!(
        parse_request(&serde_json::to_vec(&bad).unwrap(), SCHEMA_DIGEST)
            .unwrap_err()
            .code(),
        "PROTOCOL_SCHEMA_DIGEST_MISMATCH"
    );
    let mut bad = request();
    bad["extensions"] = json!([{"id":"unknown","critical":true}]);
    assert_eq!(
        parse_request(&serde_json::to_vec(&bad).unwrap(), SCHEMA_DIGEST)
            .unwrap_err()
            .code(),
        "CRITICAL_EXTENSION_UNKNOWN"
    );
}

#[test]
fn watch_snapshot_resume_dedup_and_stale_cursor_force_resnapshot() {
    let mut log = WatchLog::new("stream-1", 2);
    log.append(json!({"event_id":"e1"})).unwrap();
    log.append(json!({"event_id":"e2"})).unwrap();
    let opened = log.open(json!({"objects":[]})).unwrap();
    assert_eq!(opened[0]["kind"], "snapshot");
    assert_eq!(opened[1]["sequence"], 1);
    assert_eq!(opened[2]["sequence"], 2);
    log.append(json!({"event_id":"e3"})).unwrap();
    let resumed = log.resume(2).unwrap();
    assert_eq!(resumed.len(), 1);
    assert_eq!(resumed[0]["sequence"], 3);
    log.compact_through(2);
    let stale = log.resume(1).unwrap_err();
    assert!(matches!(stale, AkpError::Registered { .. }));
    assert_eq!(stale.code(), "WATCH_CURSOR_STALE");
    assert_eq!(
        log.open(json!({"objects":[{"id":"fresh"}]})).unwrap()[0]["kind"],
        "snapshot"
    );
}
