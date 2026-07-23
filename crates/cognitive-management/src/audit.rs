//! Ordinary Core management AUDIT tracer types.
//!
//! These are internal candidate contracts from ADR-0014. They deliberately do
//! not claim machine registration. The deterministic release gate is real code:
//! no `status.inspect` result crosses it without a matching audit commit receipt.

use cognitive_contracts::canonical::{canonical_bytes_of_value, digest};
use cognitive_domain::WallTimestamp;
use cognitive_kernel::ports::Clock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Mutex;

/// Candidate digest domain for an inspect selector.
pub const PRIVILEGED_READ_REQUEST_DOMAIN: &str = "management-privileged-read-request/0.2";
/// Candidate digest domain for an inspect result.
pub const PRIVILEGED_READ_RESULT_DOMAIN: &str = "management-privileged-read-result/0.2";
/// Candidate digest domain for the audit decision record.
pub const PRIVILEGED_READ_RECORD_DOMAIN: &str = "management-privileged-read-record/0.2";

/// Terminal outcome recorded before a privileged-read result may be released.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Lightweight durable Ordinary Core AUDIT adapter.
///
/// One process holds an exclusive OS file lock for the adapter lifetime. Each
/// open durably advances `writer_epoch`; each decision receives one global,
/// contiguous sequence and is synced before its receipt is returned. The
/// journal contains only the safe [`PrivilegedReadDecision`] carrier—never the
/// raw selector or protected object identity.
pub struct FileManagementAuditLog<C> {
    clock: C,
    state: Mutex<FileAuditState>,
}

struct FileAuditState {
    file: File,
    writer_epoch: i64,
    last_sequence: i64,
    record_ids: BTreeSet<String>,
}

impl<C> FileManagementAuditLog<C>
where
    C: Clock,
{
    /// Open or create a canonical JSON-lines journal and claim its single
    /// writer lock. Corrupt or incomplete journals fail closed.
    pub fn open(path: &Path, clock: C) -> Result<Self, AuditPortFailure> {
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(path)
            .map_err(|err| AuditPortFailure::new(format!("open audit journal: {err}")))?;
        file.try_lock()
            .map_err(|err| AuditPortFailure::new(format!("acquire audit writer lock: {err}")))?;

        let mut text = String::new();
        file.read_to_string(&mut text)
            .map_err(|err| AuditPortFailure::new(format!("read audit journal: {err}")))?;
        if !text.is_empty() && !text.ends_with('\n') {
            return Err(AuditPortFailure::new(
                "audit journal has an incomplete trailing frame",
            ));
        }
        let (last_epoch, last_sequence, record_ids) = scan_journal(&text)?;
        let writer_epoch = last_epoch
            .checked_add(1)
            .ok_or_else(|| AuditPortFailure::new("audit writer epoch exhausted"))?;
        let opened_at = clock
            .now()
            .map_err(|err| AuditPortFailure::new(format!("read audit clock: {err}")))?;
        file.seek(SeekFrom::End(0))
            .map_err(|err| AuditPortFailure::new(format!("seek audit journal: {err}")))?;
        append_value(
            &mut file,
            &serde_json::json!({
                "kind": "writer_epoch",
                "opened_at": opened_at,
                "writer_epoch": writer_epoch,
            }),
        )?;

        Ok(Self {
            clock,
            state: Mutex::new(FileAuditState {
                file,
                writer_epoch,
                last_sequence,
                record_ids,
            }),
        })
    }
}

impl<C> ManagementAuditPort for FileManagementAuditLog<C>
where
    C: Clock,
{
    fn commit_privileged_read_decision(
        &self,
        record: &PrivilegedReadDecision,
        record_digest: &str,
    ) -> Result<AuditCommitReceipt, AuditPortFailure> {
        let calculated = record.canonical_digest()?;
        if calculated != record_digest {
            return Err(AuditPortFailure::new(
                "audit record digest does not match the decision",
            ));
        }
        let committed_at = self
            .clock
            .now()
            .map_err(|err| AuditPortFailure::new(format!("read audit clock: {err}")))?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| AuditPortFailure::new("audit writer state lock poisoned"))?;
        if state.record_ids.contains(&record.record_id) {
            return Err(AuditPortFailure::new("duplicate audit record identity"));
        }
        let sequence = state
            .last_sequence
            .checked_add(1)
            .ok_or_else(|| AuditPortFailure::new("audit sequence exhausted"))?;
        let writer_epoch = state.writer_epoch;
        append_value(
            &mut state.file,
            &serde_json::json!({
                "committed_at": committed_at,
                "kind": "privileged_read_decision",
                "record": record,
                "record_digest": record_digest,
                "sequence": sequence,
                "writer_epoch": writer_epoch,
            }),
        )?;
        state.last_sequence = sequence;
        state.record_ids.insert(record.record_id.clone());

        Ok(AuditCommitReceipt {
            record_id: record.record_id.clone(),
            record_digest: record_digest.to_owned(),
            request_digest: record.request_digest.clone(),
            sequence,
            writer_epoch,
            committed_at,
        })
    }
}

fn scan_journal(text: &str) -> Result<(i64, i64, BTreeSet<String>), AuditPortFailure> {
    let mut last_epoch = 0_i64;
    let mut last_sequence = 0_i64;
    let mut record_ids = BTreeSet::new();
    for (index, line) in text.lines().enumerate() {
        let line_number = index + 1;
        let value: Value = serde_json::from_str(line).map_err(|err| {
            AuditPortFailure::new(format!("parse audit journal line {line_number}: {err}"))
        })?;
        let canonical = canonical_bytes_of_value(&value).map_err(|err| {
            AuditPortFailure::new(format!(
                "canonicalize audit journal line {line_number}: {err}"
            ))
        })?;
        if canonical != line.as_bytes() {
            return Err(AuditPortFailure::new(format!(
                "audit journal line {line_number} is not canonical"
            )));
        }
        match required_str(&value, "kind", line_number)? {
            "writer_epoch" => {
                let epoch = required_i64(&value, "writer_epoch", line_number)?;
                if epoch != last_epoch + 1 {
                    return Err(AuditPortFailure::new(format!(
                        "audit writer epoch discontinuity at line {line_number}"
                    )));
                }
                last_epoch = epoch;
            }
            "privileged_read_decision" => {
                let epoch = required_i64(&value, "writer_epoch", line_number)?;
                let sequence = required_i64(&value, "sequence", line_number)?;
                if epoch != last_epoch || epoch < 1 {
                    return Err(AuditPortFailure::new(format!(
                        "audit decision writer epoch mismatch at line {line_number}"
                    )));
                }
                if sequence != last_sequence + 1 {
                    return Err(AuditPortFailure::new(format!(
                        "audit sequence discontinuity at line {line_number}"
                    )));
                }
                let record: PrivilegedReadDecision =
                    serde_json::from_value(value.get("record").cloned().ok_or_else(|| {
                        AuditPortFailure::new(format!(
                            "audit journal line {line_number} missing record"
                        ))
                    })?)
                    .map_err(|err| {
                        AuditPortFailure::new(format!(
                            "decode audit journal record at line {line_number}: {err}"
                        ))
                    })?;
                let stored_digest = required_str(&value, "record_digest", line_number)?;
                if record.canonical_digest()? != stored_digest {
                    return Err(AuditPortFailure::new(format!(
                        "audit record digest mismatch at line {line_number}"
                    )));
                }
                if !record_ids.insert(record.record_id) {
                    return Err(AuditPortFailure::new(format!(
                        "duplicate audit record identity at line {line_number}"
                    )));
                }
                last_sequence = sequence;
            }
            other => {
                return Err(AuditPortFailure::new(format!(
                    "unknown audit journal kind `{other}` at line {line_number}"
                )));
            }
        }
    }
    Ok((last_epoch, last_sequence, record_ids))
}

fn required_str<'a>(
    value: &'a Value,
    field: &str,
    line_number: usize,
) -> Result<&'a str, AuditPortFailure> {
    value.get(field).and_then(Value::as_str).ok_or_else(|| {
        AuditPortFailure::new(format!(
            "audit journal line {line_number} missing string `{field}`"
        ))
    })
}

fn required_i64(value: &Value, field: &str, line_number: usize) -> Result<i64, AuditPortFailure> {
    value.get(field).and_then(Value::as_i64).ok_or_else(|| {
        AuditPortFailure::new(format!(
            "audit journal line {line_number} missing integer `{field}`"
        ))
    })
}

fn append_value(file: &mut File, value: &Value) -> Result<(), AuditPortFailure> {
    let bytes = canonical_bytes_of_value(value)
        .map_err(|err| AuditPortFailure::new(format!("canonicalize audit frame: {err}")))?;
    file.write_all(&bytes)
        .and_then(|()| file.write_all(b"\n"))
        .and_then(|()| file.sync_all())
        .map_err(|err| AuditPortFailure::new(format!("persist audit frame: {err}")))
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
        if timestamp_order_key(&receipt.committed_at) < timestamp_order_key(&record.observed_at) {
            return Err(AuditPortFailure::new(
                "audit receipt predates the observed decision",
            ));
        }
        Ok(())
    }
}

fn timestamp_order_key(timestamp: &WallTimestamp) -> String {
    let without_z = timestamp
        .as_str()
        .strip_suffix('Z')
        .unwrap_or(timestamp.as_str());
    let (seconds, fraction) = without_z
        .split_once('.')
        .map_or((without_z, ""), |(seconds, fraction)| (seconds, fraction));
    let mut key = String::with_capacity(seconds.len() + 10);
    key.push_str(seconds);
    key.push('.');
    key.push_str(fraction);
    for _ in fraction.len()..9 {
        key.push('0');
    }
    key
}

pub(crate) fn digest_value(value: &Value, domain: &str) -> Result<String, AuditPortFailure> {
    let canonical = canonical_bytes_of_value(value)
        .map_err(|err| AuditPortFailure::new(format!("canonicalize audit value: {err}")))?;
    digest(&canonical, domain)
        .map_err(|err| AuditPortFailure::new(format!("digest audit value: {err}")))
}
