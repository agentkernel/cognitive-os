//! Ordinary Core management AUDIT tracer types.
//!
//! These are internal candidate contracts from ADR-0014. They deliberately do
//! not claim machine registration. The deterministic release gate is real code:
//! no `status.inspect` result crosses it without a matching audit commit receipt.

use cognitive_contracts::canonical::{canonical_bytes_of_value, digest};
use cognitive_domain::WallTimestamp;
use serde::Serialize;
use serde_json::Value;

/// Candidate digest domain for an inspect selector.
pub const PRIVILEGED_READ_REQUEST_DOMAIN: &str = "management-privileged-read-request/0.2";
/// Candidate digest domain for an inspect result.
pub const PRIVILEGED_READ_RESULT_DOMAIN: &str = "management-privileged-read-result/0.2";
/// Candidate digest domain for the audit decision record.
pub const PRIVILEGED_READ_RECORD_DOMAIN: &str = "management-privileged-read-record/0.2";

/// Terminal outcome recorded before a privileged-read result may be released.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivilegedReadOutcome {
    /// An authoritative result was produced.
    Success,
    /// The protected read was denied or the object was not visible.
    Denied,
    /// Infrastructure prevented the read from completing.
    Error,
}

/// Closed internal candidate record for one privileged-read decision.
///
/// It intentionally contains no raw selector/object identity. The selector is
/// represented only by a domain-separated digest, preserving protected
/// not-found/denial isomorphism in the audit carrier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrivilegedReadDecision {
    /// Candidate record kind.
    pub record_kind: String,
    /// UUIDv7 identity allocated by the authority process.
    pub record_id: String,
    /// Digest of the exact requested domain/object selector.
    pub request_digest: String,
    /// Terminal outcome.
    pub outcome: PrivilegedReadOutcome,
    /// Safe registered reason code for denial/error outcomes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safe_reason: Option<String>,
    /// Digest of the authoritative report on success.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_digest: Option<String>,
    /// Trusted observation time.
    pub observed_at: WallTimestamp,
}

impl PrivilegedReadDecision {
    /// Canonical candidate digest of this record.
    pub fn canonical_digest(&self) -> Result<String, AuditPortFailure> {
        let value = serde_json::to_value(self)
            .map_err(|err| AuditPortFailure::new(format!("serialize audit record: {err}")))?;
        digest_value(&value, PRIVILEGED_READ_RECORD_DOMAIN)
    }
}

/// Durable receipt returned by the AUDIT port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditCommitReceipt {
    /// Exact committed record identity.
    pub record_id: String,
    /// Exact committed record digest.
    pub record_digest: String,
    /// Exact request digest bound by the record.
    pub request_digest: String,
    /// Positive contiguous sequence in the audit stream.
    pub sequence: i64,
    /// Positive audit-writer epoch.
    pub writer_epoch: i64,
    /// Trusted commit time.
    pub committed_at: WallTimestamp,
}

/// Fail-closed AUDIT-port or receipt-validation failure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("management audit unavailable: {detail}")]
pub struct AuditPortFailure {
    /// Deterministic internal detail. No unregistered wire code is surfaced.
    pub detail: String,
}

impl AuditPortFailure {
    /// Construct an internal candidate failure.
    pub fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }
}

/// Deterministic AUDIT port consumed by the Ordinary Core result-release path.
pub trait ManagementAuditPort {
    /// Durably commit the decision and return the exact receipt.
    fn commit_privileged_read_decision(
        &self,
        record: &PrivilegedReadDecision,
        record_digest: &str,
    ) -> Result<AuditCommitReceipt, AuditPortFailure>;
}

/// Failure surface of the audited inspect tracer.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum AuditedInspectError {
    /// The underlying deterministic management read failed after its decision
    /// was audited.
    #[error(transparent)]
    Management(#[from] crate::ManagementError),
    /// Audit commit failed or the receipt did not match, so no result released.
    #[error(transparent)]
    Audit(#[from] AuditPortFailure),
}

/// Deterministic result-release receipt validator.
pub struct ResultReleaseGate;

impl ResultReleaseGate {
    /// Validate that the receipt authorizes release of exactly this decision.
    pub fn validate(
        record: &PrivilegedReadDecision,
        record_digest: &str,
        receipt: &AuditCommitReceipt,
    ) -> Result<(), AuditPortFailure> {
        if receipt.record_id != record.record_id
            || receipt.record_digest != record_digest
            || receipt.request_digest != record.request_digest
        {
            return Err(AuditPortFailure::new("audit receipt subject mismatch"));
        }
        if receipt.sequence < 1 || receipt.writer_epoch < 1 {
            return Err(AuditPortFailure::new(
                "audit receipt sequence/writer epoch is not positive",
            ));
        }
        if receipt.committed_at < record.observed_at {
            return Err(AuditPortFailure::new(
                "audit receipt predates the observed decision",
            ));
        }
        Ok(())
    }
}

pub(crate) fn digest_value(value: &Value, domain: &str) -> Result<String, AuditPortFailure> {
    let canonical = canonical_bytes_of_value(value)
        .map_err(|err| AuditPortFailure::new(format!("canonicalize audit value: {err}")))?;
    digest(&canonical, domain)
        .map_err(|err| AuditPortFailure::new(format!("digest audit value: {err}")))
}
