//! Ordinary Core management AUDIT runtime boundary.
//!
//! The port consumes the registered generated decision and receipt bindings.
//! JSON Schema conditionals are enforced again here because the generated Rust
//! shapes intentionally do not encode cross-field constraints.

use cognitive_contracts::canonical::{canonical_bytes_of_value, digest};
pub use cognitive_contracts::generated::audit_commit_receipt::OrdinaryCoreAuditCommitReceipt;
pub use cognitive_contracts::generated::privileged_read_decision::{
    OrdinaryCorePrivilegedReadDecision, OrdinaryCorePrivilegedReadDecisionOutcome,
    OrdinaryCorePrivilegedReadDecisionRecordKind, OrdinaryCorePrivilegedReadDecisionSafeReason,
};
use cognitive_domain::WallTimestamp;
use cognitive_kernel::ports::Clock;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Mutex;

/// Registered digest domain for an inspect selector.
pub const PRIVILEGED_READ_REQUEST_DOMAIN: &str = "management-privileged-read-request/0.2";
/// Registered digest domain for an inspect result.
pub const PRIVILEGED_READ_RESULT_DOMAIN: &str = "management-privileged-read-result/0.2";
/// Registered digest domain for the audit decision record.
pub const PRIVILEGED_READ_RECORD_DOMAIN: &str = "management-privileged-read-record/0.2";

/// Validate the registered decision shape and compute its registered-domain
/// canonical digest. All and only admitted record fields participate.
pub fn privileged_read_decision_digest(
    record: &OrdinaryCorePrivilegedReadDecision,
) -> Result<String, AuditPortFailure> {
    validate_privileged_read_decision(record)?;
    let value = serde_json::to_value(record)
        .map_err(|err| AuditPortFailure::new(format!("serialize audit record: {err}")))?;
    digest_value(&value, PRIVILEGED_READ_RECORD_DOMAIN)
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
        record: &OrdinaryCorePrivilegedReadDecision,
        record_digest: &str,
    ) -> Result<OrdinaryCoreAuditCommitReceipt, AuditPortFailure>;
}

/// Lightweight durable Ordinary Core AUDIT adapter.
///
/// One process holds an exclusive OS file lock for the adapter lifetime. Each
/// open durably advances `writer_epoch`; each decision receives one global,
/// contiguous sequence and is synced before its receipt is returned. The
/// journal contains only the registered safe
/// [`OrdinaryCorePrivilegedReadDecision`] carrier—never the raw selector or
/// protected object identity.
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
        record: &OrdinaryCorePrivilegedReadDecision,
        record_digest: &str,
    ) -> Result<OrdinaryCoreAuditCommitReceipt, AuditPortFailure> {
        let calculated = privileged_read_decision_digest(record)?;
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

        Ok(OrdinaryCoreAuditCommitReceipt {
            committed_at: committed_at.as_str().to_owned(),
            record_id: record.record_id.clone(),
            record_digest: record_digest.to_owned(),
            request_digest: record.request_digest.clone(),
            sequence,
            writer_epoch,
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
                let record: OrdinaryCorePrivilegedReadDecision =
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
                if privileged_read_decision_digest(&record)? != stored_digest {
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
        record: &OrdinaryCorePrivilegedReadDecision,
        record_digest: &str,
        receipt: &OrdinaryCoreAuditCommitReceipt,
    ) -> Result<(), AuditPortFailure> {
        validate_privileged_read_decision(record)?;
        validate_digest(record_digest, "record digest")?;
        validate_audit_commit_receipt(receipt)?;
        if receipt.record_id != record.record_id
            || receipt.record_digest != record_digest
            || receipt.request_digest != record.request_digest
        {
            return Err(AuditPortFailure::new("audit receipt subject mismatch"));
        }
        let committed_at = parse_timestamp(&receipt.committed_at, "receipt committed_at")?;
        let observed_at = parse_timestamp(&record.observed_at, "decision observed_at")?;
        if timestamp_order_key(&committed_at) < timestamp_order_key(&observed_at) {
            return Err(AuditPortFailure::new(
                "audit receipt predates the observed decision",
            ));
        }
        Ok(())
    }
}

pub(crate) fn registered_safe_reason(
    code: &str,
) -> Result<OrdinaryCorePrivilegedReadDecisionSafeReason, AuditPortFailure> {
    serde_json::from_value(Value::String(code.to_owned())).map_err(|_| {
        AuditPortFailure::new(format!(
            "registered management code `{code}` is not admitted by the privileged-read decision schema"
        ))
    })
}

fn validate_privileged_read_decision(
    record: &OrdinaryCorePrivilegedReadDecision,
) -> Result<(), AuditPortFailure> {
    validate_uuid(&record.record_id, "decision record_id")?;
    validate_digest(&record.request_digest, "decision request_digest")?;
    parse_timestamp(&record.observed_at, "decision observed_at")?;

    match record.outcome {
        OrdinaryCorePrivilegedReadDecisionOutcome::Success => {
            if record.safe_reason.is_some() || record.result_digest.is_none() {
                return Err(AuditPortFailure::new(
                    "successful audit decision must have result_digest and no safe_reason",
                ));
            }
        }
        OrdinaryCorePrivilegedReadDecisionOutcome::Denied
        | OrdinaryCorePrivilegedReadDecisionOutcome::Error => {
            if record.safe_reason.is_none() || record.result_digest.is_some() {
                return Err(AuditPortFailure::new(
                    "denied/error audit decision must have safe_reason and no result_digest",
                ));
            }
        }
    }
    if let Some(result_digest) = &record.result_digest {
        validate_digest(result_digest, "decision result_digest")?;
    }
    Ok(())
}

fn validate_audit_commit_receipt(
    receipt: &OrdinaryCoreAuditCommitReceipt,
) -> Result<(), AuditPortFailure> {
    validate_uuid(&receipt.record_id, "receipt record_id")?;
    validate_digest(&receipt.record_digest, "receipt record_digest")?;
    validate_digest(&receipt.request_digest, "receipt request_digest")?;
    parse_timestamp(&receipt.committed_at, "receipt committed_at")?;
    if receipt.sequence < 1 || receipt.writer_epoch < 1 {
        return Err(AuditPortFailure::new(
            "audit receipt sequence/writer epoch is not positive",
        ));
    }
    Ok(())
}

fn validate_uuid(value: &str, field: &str) -> Result<(), AuditPortFailure> {
    let bytes = value.as_bytes();
    let valid = bytes.len() == 36
        && bytes.iter().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                *byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        });
    if valid {
        Ok(())
    } else {
        Err(AuditPortFailure::new(format!("{field} is not a UUID")))
    }
}

fn validate_digest(value: &str, field: &str) -> Result<(), AuditPortFailure> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(AuditPortFailure::new(format!(
            "{field} is not a sha256 digest"
        )));
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(AuditPortFailure::new(format!(
            "{field} is not a lowercase sha256 digest"
        )));
    }
    Ok(())
}

fn parse_timestamp(value: &str, field: &str) -> Result<WallTimestamp, AuditPortFailure> {
    WallTimestamp::parse(value)
        .map_err(|err| AuditPortFailure::new(format!("{field} is invalid: {err}")))
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
