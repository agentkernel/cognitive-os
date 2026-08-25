#![allow(clippy::unwrap_used, clippy::panic)]

use cognitive_contracts::canonical::canonical_bytes_of_value;
use cognitive_contracts::generated::audit_commit_receipt::OrdinaryCoreAuditCommitReceipt;
use cognitive_contracts::generated::privileged_read_decision::{
    OrdinaryCorePrivilegedReadDecision, OrdinaryCorePrivilegedReadDecisionOutcome,
    OrdinaryCorePrivilegedReadDecisionRecordKind,
};
use cognitive_domain::WallTimestamp;
use cognitive_kernel::ports::{Clock, PortFailure};
use cognitive_management::{
    FileManagementAuditLog, ManagementAuditPort, ResultReleaseGate, privileged_read_decision_digest,
};
use serde_json::Value;
use std::path::{Path, PathBuf};

type JournalMutationCase = (&'static str, fn(&mut Value), &'static str);

#[derive(Debug, Clone, Copy)]
struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> Result<WallTimestamp, PortFailure> {
        WallTimestamp::parse("2026-07-23T12:00:00Z").map_err(|err| PortFailure {
            detail: err.to_string(),
        })
    }
}

fn decision(record_id: &str, request_digest: &str) -> OrdinaryCorePrivilegedReadDecision {
    OrdinaryCorePrivilegedReadDecision {
        record_kind: OrdinaryCorePrivilegedReadDecisionRecordKind::PrivilegedReadDecision,
        record_id: record_id.to_owned(),
        request_digest: request_digest.to_owned(),
        outcome: OrdinaryCorePrivilegedReadDecisionOutcome::Success,
        safe_reason: None,
        result_digest: Some(
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
        ),
        observed_at: "2026-07-23T12:00:00Z".to_owned(),
    }
}

fn persisted_audit_journal() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("management-audit.jsonl");
    let record = decision(
        "01983bb4-7480-7000-8000-000000000010",
        "sha256:1010101010101010101010101010101010101010101010101010101010101010",
    );
    {
        let audit = FileManagementAuditLog::open(&path, FixedClock).unwrap();
        let digest = privileged_read_decision_digest(&record).unwrap();
        audit
            .commit_privileged_read_decision(&record, &digest)
            .unwrap();
    }
    (dir, path)
}

fn mutate_decision_frame(path: &Path, mutate: impl FnOnce(&mut Value)) {
    let text = std::fs::read_to_string(path).unwrap();
    let mut frames: Vec<Value> = text
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    mutate(&mut frames[1]);
    let mut rewritten = Vec::new();
    for frame in frames {
        rewritten.extend(canonical_bytes_of_value(&frame).unwrap());
        rewritten.push(b'\n');
    }
    std::fs::write(path, rewritten).unwrap();
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
        let digest = privileged_read_decision_digest(&first_record).unwrap();
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
        let digest = privileged_read_decision_digest(&second_record).unwrap();
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
    let decision_frame: Value = serde_json::from_str(persisted.lines().nth(1).unwrap()).unwrap();
    let persisted_record: OrdinaryCorePrivilegedReadDecision =
        serde_json::from_value(decision_frame["record"].clone()).unwrap();
    assert_eq!(persisted_record, first_record);
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
fn durable_audit_readback_rejects_nonformal_record_shapes_and_damaged_digest() {
    let cases: [JournalMutationCase; 4] = [
        (
            "unknown field",
            |frame| {
                frame["record"]["object_id"] = Value::String("must-not-be-admitted".to_owned());
            },
            "decode audit journal record",
        ),
        (
            "unknown outcome",
            |frame| {
                frame["record"]["outcome"] = Value::String("invented".to_owned());
            },
            "decode audit journal record",
        ),
        (
            "cross-field violation",
            |frame| {
                frame["record"]["safe_reason"] = Value::String("CONTEXT_AUTH_DENIED".to_owned());
            },
            "successful audit decision",
        ),
        (
            "damaged digest",
            |frame| {
                frame["record_digest"] = Value::String(
                    "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                        .to_owned(),
                );
            },
            "audit record digest mismatch",
        ),
    ];

    for (case, mutate, expected) in cases {
        let (_dir, path) = persisted_audit_journal();
        mutate_decision_frame(&path, mutate);
        let error = match FileManagementAuditLog::open(&path, FixedClock) {
            Ok(_) => panic!("{case} unexpectedly passed audit readback"),
            Err(error) => error,
        };
        assert!(error.detail.contains(expected), "{case}: {error}");
    }
}

#[test]
fn release_gate_orders_fractional_timestamps_by_time_not_raw_text() {
    let record = decision(
        "01983bb4-7480-7000-8000-000000000003",
        "sha256:3333333333333333333333333333333333333333333333333333333333333333",
    );
    let record_digest = privileged_read_decision_digest(&record).unwrap();
    let receipt = OrdinaryCoreAuditCommitReceipt {
        committed_at: "2026-07-23T12:00:00.001Z".to_owned(),
        record_id: record.record_id.clone(),
        record_digest: record_digest.clone(),
        request_digest: record.request_digest.clone(),
        sequence: 1,
        writer_epoch: 1,
    };

    ResultReleaseGate::validate(&record, &record_digest, &receipt).unwrap();
}
