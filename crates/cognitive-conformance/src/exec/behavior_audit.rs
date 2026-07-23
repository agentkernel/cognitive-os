//! Ordinary Core AUDIT behavioral execution against the existing public
//! `status.inspect` consumer and durable file-backed audit adapter.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use super::{AssetContext, ExecError, GateOutput, ImplementationKind};
use crate::LoadedVector;
use cognitive_contracts::canonical::{canonical_bytes_of_value, digest};
use cognitive_domain::{LifecycleDomain, ObjectId, UriRef, WallTimestamp};
use cognitive_kernel::ports::{Clock, IdGenerator, PortFailure};
use cognitive_kernel::{AdmitCommand, TransitionEngine};
use cognitive_management::audit::{
    PRIVILEGED_READ_RECORD_DOMAIN, PRIVILEGED_READ_REQUEST_DOMAIN, PRIVILEGED_READ_RESULT_DOMAIN,
};
use cognitive_management::{
    AuditPortFailure, AuditedInspectError, FileManagementAuditLog, InspectRequest,
    ManagementAuditPort, ManagementPlane, OrdinaryCoreAuditCommitReceipt,
    OrdinaryCorePrivilegedReadDecision, OrdinaryCorePrivilegedReadDecisionOutcome,
    OrdinaryCorePrivilegedReadDecisionRecordKind, PrivilegedManagementSession, ResultReleaseGate,
    privileged_read_decision_digest,
};
use cognitive_store::SqliteAuthorityStore;
use serde_json::{Value, json};
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicU64, Ordering};

const REFERENCE_IMPLEMENTATION: &str = "cognitive-management ManagementPlane::inspect_with_audit + \
     FileManagementAuditLog + ResultReleaseGate (existing Ordinary Core audited public consumer)";
const WRONG_IMPLEMENTATION: &str = "deliberately wrong Ordinary Core audit path (releases before \
     durable commit and ignores receipt subject mismatch)";

fn env_err(what: impl Into<String>) -> ExecError {
    ExecError::Environment(what.into())
}

#[derive(Clone, Copy)]
struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> Result<WallTimestamp, PortFailure> {
        WallTimestamp::parse("2026-07-23T12:00:00Z").map_err(|err| PortFailure {
            detail: err.to_string(),
        })
    }
}

struct SeqIds(AtomicU64);

impl SeqIds {
    fn new() -> Self {
        Self(AtomicU64::new(1))
    }
}

impl IdGenerator for SeqIds {
    fn next_uuid_v7(&self) -> Result<String, PortFailure> {
        let next = self.0.fetch_add(1, Ordering::SeqCst);
        Ok(format!("00000000-0000-7000-8000-{next:012x}"))
    }
}

type CapturedCommit = (
    OrdinaryCorePrivilegedReadDecision,
    String,
    OrdinaryCoreAuditCommitReceipt,
);

struct CapturingFileAudit {
    inner: FileManagementAuditLog<FixedClock>,
    captured: RefCell<Vec<CapturedCommit>>,
    corrupt_next_receipt: Cell<bool>,
}

impl CapturingFileAudit {
    fn new(inner: FileManagementAuditLog<FixedClock>) -> Self {
        Self {
            inner,
            captured: RefCell::new(Vec::new()),
            corrupt_next_receipt: Cell::new(false),
        }
    }

    fn corrupt_next_receipt(&self) {
        self.corrupt_next_receipt.set(true);
    }
}

impl ManagementAuditPort for CapturingFileAudit {
    fn commit_privileged_read_decision(
        &self,
        record: &OrdinaryCorePrivilegedReadDecision,
        record_digest: &str,
    ) -> Result<OrdinaryCoreAuditCommitReceipt, AuditPortFailure> {
        let receipt = self
            .inner
            .commit_privileged_read_decision(record, record_digest)?;
        self.captured.borrow_mut().push((
            record.clone(),
            record_digest.to_owned(),
            receipt.clone(),
        ));
        if self.corrupt_next_receipt.replace(false) {
            let mut mismatched = receipt;
            mismatched.record_digest =
                "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                    .to_owned();
            Ok(mismatched)
        } else {
            Ok(receipt)
        }
    }
}

fn object_id() -> Result<ObjectId, ExecError> {
    ObjectId::parse("00000000-0000-7000-9000-000000003301")
        .map_err(|err| env_err(format!("object id: {err}")))
}

fn uri(value: &str) -> Result<UriRef, ExecError> {
    UriRef::parse(value).map_err(|err| env_err(format!("uri {value}: {err}")))
}

fn active_session() -> Result<PrivilegedManagementSession, ExecError> {
    PrivilegedManagementSession::from_json_value(&json!({
        "schema_version": "cognitiveos.privileged-management-session/0.1",
        "session_id": "pms_cfr-audit-inspect-01",
        "object_version": 1,
        "management_domain": "cognitiveos.management",
        "session_authority": "authority://tenant-a/management-authority",
        "human_principal": "principal://tenant-a/operator-1",
        "actor_chain_digest": format!("sha256:{}", "ab12".repeat(16)),
        "authentication_context_ref": "authn://tenant-a/webauthn-9",
        "activity_context_ref": "activity://tenant-a/cfr-audit-inspect",
        "scope": {
            "domains": ["cognitiveos.management.status"],
            "actions": ["status.inspect"],
            "resources": ["agent-execution://"]
        },
        "risk_ceiling": "R0",
        "policy_version": 1,
        "revocation_epoch": 41,
        "issued_at": "2026-07-23T12:00:00Z",
        "last_activity_at": "2026-07-23T12:00:00Z",
        "idle_timeout_seconds": 3600,
        "absolute_expires_at": "2030-01-01T00:00:00Z",
        "state": "active",
        "session_digest": format!("sha256:{}", "cd34".repeat(16)),
        "authority_signature": "sig-cfr-audit-inspect-0001"
    }))
    .map_err(|err| env_err(format!("management session: {err}")))
}

fn digest_value(value: &Value, domain: &str) -> Result<String, ExecError> {
    let bytes = canonical_bytes_of_value(value)
        .map_err(|err| env_err(format!("canonicalize {domain}: {err}")))?;
    digest(&bytes, domain).map_err(|err| env_err(format!("digest {domain}: {err}")))
}

pub(super) fn ordinary_core_audit_inspect_001_behavior(
    ctx: &AssetContext,
    _vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    if matches!(kind, ImplementationKind::DeliberatelyWrong) {
        return Ok(GateOutput {
            actual: json!({
                "public_consumer_executed": false,
                "status_inspect_result_released": true,
                "audit_commit_completed_before_success_return": false,
                "formal_decision_shape_valid": false,
                "formal_receipt_shape_valid": false,
                "request_digest_matches_projection": false,
                "result_digest_matches_report": false,
                "record_digest_matches_readback": false,
                "record_id_bound": false,
                "request_digest_bound": false,
                "sequence_positive": false,
                "writer_epoch_positive": false,
                "commit_not_before_observation": false,
                "durable_readback_present": false,
                "mismatched_receipt_withholds_result": false
            }),
            grounding: vec!["specs/core/ordinary-core-audit.md".into()],
            informative: vec![],
            implementation: Some(WRONG_IMPLEMENTATION),
            evidence: json!({"anti_pattern": "success released without a matching durable receipt"}),
        });
    }

    let directory = tempfile::tempdir().map_err(|err| env_err(format!("tempdir: {err}")))?;
    let store = SqliteAuthorityStore::open(&directory.path().join("authority.db"))
        .map_err(|err| env_err(format!("authority store: {err}")))?;
    let clock = FixedClock;
    let ids = SeqIds::new();
    let object_id = object_id()?;
    TransitionEngine::new(&store, &clock, &ids)
        .admit_object(&AdmitCommand {
            object_id: object_id.clone(),
            domain: LifecycleDomain::AgentExecution,
            subject_ref: uri(&format!("agent-execution://tenant-a/{object_id}"))?,
            body: json!({"ordinary_core_audit": true}),
            actor_ref: uri("actor://tenant-a/agent-1")?,
            authority_ref: uri("authority://tenant-a/state-authority")?,
            correlation_id: uri("corr://tenant-a/cfr-audit-inspect")?,
            outbox_destinations: vec![],
            fencing_epoch: None,
        })
        .map_err(|err| env_err(format!("admit audited object: {err}")))?;

    let journal_path = directory.path().join("management-audit.jsonl");
    let audit = CapturingFileAudit::new(
        FileManagementAuditLog::open(&journal_path, clock)
            .map_err(|err| env_err(format!("open audit journal: {err}")))?,
    );
    let plane = ManagementPlane::deterministic(&store, &clock, &ids);
    let session = active_session()?;
    let request = InspectRequest {
        domain: LifecycleDomain::AgentExecution,
        object_id: object_id.clone(),
    };

    let report = plane
        .inspect_with_audit(&session, &request, &audit)
        .map_err(|err| env_err(format!("audited status.inspect: {err}")))?;
    let commit_completed_before_success_return = !audit.captured.borrow().is_empty();
    let first_commit = audit
        .captured
        .borrow()
        .first()
        .cloned()
        .ok_or_else(|| env_err("audit port returned without a captured durable commit"))?;
    let (record, supplied_record_digest, receipt) = first_commit;
    drop(audit);

    let journal = std::fs::read_to_string(&journal_path)
        .map_err(|err| env_err(format!("read audit journal: {err}")))?;
    let mut durable_frame = None;
    let mut every_frame_canonical = true;
    for line in journal.lines() {
        let value: Value = serde_json::from_str(line)
            .map_err(|err| env_err(format!("decode audit journal frame: {err}")))?;
        every_frame_canonical &= canonical_bytes_of_value(&value)
            .map_err(|err| env_err(format!("canonicalize audit journal frame: {err}")))?
            == line.as_bytes();
        if value.get("kind") == Some(&Value::String("privileged_read_decision".to_owned()))
            && value.pointer("/record/record_id").and_then(Value::as_str)
                == Some(record.record_id.as_str())
        {
            durable_frame = Some(value);
        }
    }
    let durable_frame =
        durable_frame.ok_or_else(|| env_err("committed record missing on readback"))?;

    let decision_value = serde_json::to_value(&record)
        .map_err(|err| env_err(format!("serialize decision: {err}")))?;
    let receipt_value = serde_json::to_value(&receipt)
        .map_err(|err| env_err(format!("serialize receipt: {err}")))?;
    let request_digest = digest_value(
        &json!({
            "domain": request.domain.as_str(),
            "object_id": request.object_id.as_str(),
        }),
        PRIVILEGED_READ_REQUEST_DOMAIN,
    )?;
    let result_digest = digest_value(
        &serde_json::to_value(&report)
            .map_err(|err| env_err(format!("serialize inspect report: {err}")))?,
        PRIVILEGED_READ_RESULT_DOMAIN,
    )?;
    let calculated_record_digest = privileged_read_decision_digest(&record)
        .map_err(|err| env_err(format!("decision digest: {err}")))?;
    let independent_record_digest = digest_value(&decision_value, PRIVILEGED_READ_RECORD_DOMAIN)?;

    let decision_shape_valid = ctx
        .validator("privileged-read-decision.schema.json")?
        .is_valid(&decision_value)
        && serde_json::from_value::<OrdinaryCorePrivilegedReadDecision>(decision_value.clone())
            .is_ok()
        && record.record_kind
            == OrdinaryCorePrivilegedReadDecisionRecordKind::PrivilegedReadDecision
        && record.outcome == OrdinaryCorePrivilegedReadDecisionOutcome::Success
        && record.safe_reason.is_none();
    let receipt_shape_valid = ctx
        .validator("audit-commit-receipt.schema.json")?
        .is_valid(&receipt_value)
        && serde_json::from_value::<OrdinaryCoreAuditCommitReceipt>(receipt_value.clone()).is_ok();

    let mismatch_audit = CapturingFileAudit::new(
        FileManagementAuditLog::open(&journal_path, clock)
            .map_err(|err| env_err(format!("reopen audit journal: {err}")))?,
    );
    mismatch_audit.corrupt_next_receipt();
    let mismatch = plane.inspect_with_audit(&session, &request, &mismatch_audit);
    let mismatch_withheld = matches!(mismatch, Err(AuditedInspectError::Audit(_)));

    let actual = json!({
        "public_consumer_executed": true,
        "status_inspect_result_released": report.domain == "agent-execution"
            && report.object_id == object_id.as_str()
            && report.state == "CREATED",
        "audit_commit_completed_before_success_return": commit_completed_before_success_return
            && durable_frame.is_object(),
        "formal_decision_shape_valid": decision_shape_valid,
        "formal_receipt_shape_valid": receipt_shape_valid,
        "request_digest_matches_projection": record.request_digest == request_digest,
        "result_digest_matches_report": record.result_digest.as_deref() == Some(result_digest.as_str()),
        "record_digest_matches_readback": supplied_record_digest == calculated_record_digest
            && calculated_record_digest == independent_record_digest
            && durable_frame.get("record_digest").and_then(Value::as_str)
                == Some(calculated_record_digest.as_str())
            && durable_frame.get("record") == Some(&decision_value),
        "record_id_bound": receipt.record_id == record.record_id,
        "request_digest_bound": receipt.request_digest == record.request_digest,
        "sequence_positive": receipt.sequence > 0,
        "writer_epoch_positive": receipt.writer_epoch > 0,
        "commit_not_before_observation": ResultReleaseGate::validate(
            &record,
            &calculated_record_digest,
            &receipt,
        ).is_ok(),
        "durable_readback_present": every_frame_canonical && durable_frame.is_object(),
        "mismatched_receipt_withholds_result": mismatch_withheld,
    });

    Ok(GateOutput {
        actual,
        grounding: vec![
            "specs/core/ordinary-core-audit.md".into(),
            "specs/schemas/privileged-read-decision.schema.json".into(),
            "specs/schemas/audit-commit-receipt.schema.json".into(),
            "crates/cognitive-management/src/plane.rs#ManagementPlane::inspect_with_audit".into(),
            "crates/cognitive-management/src/audit.rs#FileManagementAuditLog".into(),
        ],
        informative: vec![],
        implementation: Some(REFERENCE_IMPLEMENTATION),
        evidence: json!({
            "decision": decision_value,
            "receipt": receipt_value,
            "durable_frame": durable_frame,
            "journal_frames": journal.lines().count(),
            "mismatched_receipt_error": mismatch.err().map(|err| err.to_string()),
        }),
    })
}
