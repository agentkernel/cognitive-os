#![allow(clippy::unwrap_used)]

use cognitive_contracts::generated::audit_commit_receipt::OrdinaryCoreAuditCommitReceipt;
use cognitive_contracts::generated::privileged_read_decision::{
    OrdinaryCorePrivilegedReadDecision, OrdinaryCorePrivilegedReadDecisionOutcome,
    OrdinaryCorePrivilegedReadDecisionRecordKind, OrdinaryCorePrivilegedReadDecisionSafeReason,
};
use cognitive_management::{
    AuditPortFailure, ManagementAuditPort, ResultReleaseGate, privileged_read_decision_digest,
};
use serde_json::json;

struct FormalBindingPort;

fn success_decision() -> OrdinaryCorePrivilegedReadDecision {
    OrdinaryCorePrivilegedReadDecision {
        observed_at: "2026-07-23T12:00:00Z".to_owned(),
        outcome: OrdinaryCorePrivilegedReadDecisionOutcome::Success,
        record_id: "01983bb4-7480-7000-8000-000000000001".to_owned(),
        record_kind: OrdinaryCorePrivilegedReadDecisionRecordKind::PrivilegedReadDecision,
        request_digest: "sha256:1111111111111111111111111111111111111111111111111111111111111111"
            .to_owned(),
        result_digest: Some(
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
        ),
        safe_reason: None,
    }
}

impl ManagementAuditPort for FormalBindingPort {
    fn commit_privileged_read_decision(
        &self,
        record: &OrdinaryCorePrivilegedReadDecision,
        record_digest: &str,
    ) -> Result<OrdinaryCoreAuditCommitReceipt, AuditPortFailure> {
        Ok(OrdinaryCoreAuditCommitReceipt {
            committed_at: record.observed_at.clone(),
            record_digest: record_digest.to_owned(),
            record_id: record.record_id.clone(),
            request_digest: record.request_digest.clone(),
            sequence: 1,
            writer_epoch: 1,
        })
    }
}

#[test]
fn production_port_boundary_accepts_and_returns_formal_bindings() {
    let record = success_decision();

    let receipt = FormalBindingPort
        .commit_privileged_read_decision(
            &record,
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        )
        .unwrap();

    assert_eq!(receipt.record_id, record.record_id);
    assert_eq!(receipt.request_digest, record.request_digest);
}

#[test]
fn unregistered_safe_reason_cannot_enter_the_formal_port_carrier() {
    let value = json!({
        "observed_at": "2026-07-23T12:00:00Z",
        "outcome": "denied",
        "record_id": "01983bb4-7480-7000-8000-000000000002",
        "record_kind": "privileged_read_decision",
        "request_digest": "sha256:2222222222222222222222222222222222222222222222222222222222222222",
        "safe_reason": "UNREGISTERED_AUDIT_REASON"
    });

    assert!(serde_json::from_value::<OrdinaryCorePrivilegedReadDecision>(value).is_err());
}

#[test]
fn runtime_validation_enforces_registered_outcome_conditionals() {
    let mut success = success_decision();
    success.safe_reason = Some(OrdinaryCorePrivilegedReadDecisionSafeReason::ContextAuthDenied);
    assert!(privileged_read_decision_digest(&success).is_err());

    let mut success_without_result = success_decision();
    success_without_result.result_digest = None;
    assert!(privileged_read_decision_digest(&success_without_result).is_err());

    for outcome in [
        OrdinaryCorePrivilegedReadDecisionOutcome::Denied,
        OrdinaryCorePrivilegedReadDecisionOutcome::Error,
    ] {
        let mut failure = success_decision();
        failure.outcome = outcome;
        failure.result_digest = None;
        failure.safe_reason =
            Some(OrdinaryCorePrivilegedReadDecisionSafeReason::StateStoreUnavailable);
        assert!(privileged_read_decision_digest(&failure).is_ok());

        failure.result_digest = Some(
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
        );
        assert!(privileged_read_decision_digest(&failure).is_err());

        failure.result_digest = None;
        failure.safe_reason = None;
        assert!(privileged_read_decision_digest(&failure).is_err());
    }
}

#[test]
fn release_gate_rejects_subject_nonpositive_and_time_mismatches() {
    let record = success_decision();
    let record_digest = privileged_read_decision_digest(&record).unwrap();
    let receipt = OrdinaryCoreAuditCommitReceipt {
        committed_at: record.observed_at.clone(),
        record_digest: record_digest.clone(),
        record_id: record.record_id.clone(),
        request_digest: record.request_digest.clone(),
        sequence: 1,
        writer_epoch: 1,
    };
    ResultReleaseGate::validate(&record, &record_digest, &receipt).unwrap();

    let mut invalid = receipt.clone();
    invalid.record_id = "01983bb4-7480-7000-8000-000000000099".to_owned();
    assert!(ResultReleaseGate::validate(&record, &record_digest, &invalid).is_err());

    let mut invalid = receipt.clone();
    invalid.record_digest =
        "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_owned();
    assert!(ResultReleaseGate::validate(&record, &record_digest, &invalid).is_err());

    let mut invalid = receipt.clone();
    invalid.request_digest =
        "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_owned();
    assert!(ResultReleaseGate::validate(&record, &record_digest, &invalid).is_err());

    let mut invalid = receipt.clone();
    invalid.sequence = 0;
    assert!(ResultReleaseGate::validate(&record, &record_digest, &invalid).is_err());

    let mut invalid = receipt.clone();
    invalid.writer_epoch = 0;
    assert!(ResultReleaseGate::validate(&record, &record_digest, &invalid).is_err());

    let mut invalid = receipt;
    invalid.committed_at = "2026-07-23T11:59:59.999Z".to_owned();
    assert!(ResultReleaseGate::validate(&record, &record_digest, &invalid).is_err());
}

#[test]
fn production_audit_module_no_longer_defines_duplicate_wire_dtos() {
    let source = include_str!("../src/audit.rs");
    for duplicate in [
        "pub struct PrivilegedReadDecision",
        "pub enum PrivilegedReadOutcome",
        "pub struct AuditCommitReceipt",
    ] {
        assert!(
            !source.contains(duplicate),
            "duplicate DTO remains: {duplicate}"
        );
    }
}
