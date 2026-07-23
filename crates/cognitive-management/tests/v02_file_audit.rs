#![allow(clippy::unwrap_used, clippy::panic)]

use cognitive_domain::WallTimestamp;
use cognitive_kernel::ports::{Clock, PortFailure};
use cognitive_management::{
    AuditCommitReceipt, FileManagementAuditLog, ManagementAuditPort, PrivilegedReadDecision,
    PrivilegedReadOutcome, ResultReleaseGate,
};

#[derive(Debug, Clone, Copy)]
struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> Result<WallTimestamp, PortFailure> {
        WallTimestamp::parse("2026-07-23T12:00:00Z").map_err(|err| PortFailure {
            detail: err.to_string(),
        })
    }
}

fn decision(record_id: &str, request_digest: &str) -> PrivilegedReadDecision {
    PrivilegedReadDecision {
        record_kind: "privileged_read_decision".to_owned(),
        record_id: record_id.to_owned(),
        request_digest: request_digest.to_owned(),
        outcome: PrivilegedReadOutcome::Success,
        safe_reason: None,
        result_digest: Some(
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
        ),
        observed_at: WallTimestamp::parse("2026-07-23T12:00:00Z").unwrap(),
    }
}

#[test]
fn durable_audit_log_recovers_sequence_and_advances_writer_epoch() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("management-audit.jsonl");
    let first_record = decision(
        "01983bb4-7480-7000-8000-000000000001",
        "sha256:1111111111111111111111111111111111111111111111111111111111111111",
    );

    let first_receipt = {
        let audit = FileManagementAuditLog::open(&path, FixedClock).unwrap();
        let digest = first_record.canonical_digest().unwrap();
        audit
            .commit_privileged_read_decision(&first_record, &digest)
            .unwrap()
    };
    assert_eq!(first_receipt.sequence, 1);
    assert_eq!(first_receipt.writer_epoch, 1);

    let second_record = decision(
        "01983bb4-7480-7000-8000-000000000002",
        "sha256:2222222222222222222222222222222222222222222222222222222222222222",
    );
    let second_receipt = {
        let audit = FileManagementAuditLog::open(&path, FixedClock).unwrap();
        let digest = second_record.canonical_digest().unwrap();
        audit
            .commit_privileged_read_decision(&second_record, &digest)
            .unwrap()
    };
    assert_eq!(second_receipt.sequence, 2);
    assert_eq!(second_receipt.writer_epoch, 2);

    let persisted = std::fs::read_to_string(path).unwrap();
    assert_eq!(persisted.lines().count(), 4, "epoch + decision per open");
    assert!(
        persisted
            .lines()
            .all(|line| serde_json::from_str::<serde_json::Value>(line).is_ok())
    );
}

#[test]
fn durable_audit_log_refuses_a_second_writer() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("management-audit.jsonl");
    let _first = FileManagementAuditLog::open(&path, FixedClock).unwrap();

    let error = match FileManagementAuditLog::open(&path, FixedClock) {
        Ok(_) => panic!("a second writer unexpectedly acquired the audit log"),
        Err(error) => error,
    };
    assert!(error.detail.contains("writer lock"), "{error}");
}

#[test]
fn durable_audit_log_fails_closed_on_an_incomplete_journal() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("management-audit.jsonl");
    std::fs::write(&path, b"{\"kind\":\"writer_epoch\"").unwrap();

    let error = match FileManagementAuditLog::open(&path, FixedClock) {
        Ok(_) => panic!("an incomplete audit journal unexpectedly opened"),
        Err(error) => error,
    };
    assert!(
        error.detail.contains("incomplete trailing frame"),
        "{error}"
    );
}

#[test]
fn release_gate_orders_fractional_timestamps_by_time_not_raw_text() {
    let record = decision(
        "01983bb4-7480-7000-8000-000000000003",
        "sha256:3333333333333333333333333333333333333333333333333333333333333333",
    );
    let record_digest = record.canonical_digest().unwrap();
    let receipt = AuditCommitReceipt {
        record_id: record.record_id.clone(),
        record_digest: record_digest.clone(),
        request_digest: record.request_digest.clone(),
        sequence: 1,
        writer_epoch: 1,
        committed_at: WallTimestamp::parse("2026-07-23T12:00:00.001Z").unwrap(),
    };

    ResultReleaseGate::validate(&record, &record_digest, &receipt).unwrap();
}
