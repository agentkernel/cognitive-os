//! Durable governance currency of the management plane: the revocation
//! epoch and capability-set version that M3 authorization decisions are
//! revalidated against (`cognitive_kernel::authz::revalidate_grant`,
//! REQ-CAP-005, F-007).
//!
//! The M3/M4 kernel takes the CURRENT currency as a caller-supplied fact
//! (`GovernanceCurrency`: "supplied by the deterministic caller — the
//! runtime in M5"); this ledger is that caller-side authoritative record.
//! Batch-1 persistence is a canonical-JSON file owned by the management
//! plane (fail closed on any read/write error). Folding it into a
//! store-backed table is a Lane-KRN port follow-up recorded in the batch
//! handoff.

use crate::error::ManagementError;
use cognitive_domain::WallTimestamp;
use cognitive_kernel::effects::GovernanceCurrency;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// Durable ledger of the management-plane governance currency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceLedger {
    path: PathBuf,
    revocation_epoch: i64,
    capability_set_version: i64,
    updated_at: WallTimestamp,
}

fn ledger_error(detail: impl Into<String>) -> ManagementError {
    ManagementError::Ledger(detail.into())
}

fn canonical_text(value: &Value) -> Result<String, ManagementError> {
    let bytes = cognitive_contracts::canonical::canonical_bytes_of_value(value)
        .map_err(|err| ledger_error(format!("not canonicalizable: {err}")))?;
    String::from_utf8(bytes).map_err(|err| ledger_error(format!("not utf-8: {err}")))
}

impl GovernanceLedger {
    /// Create a NEW ledger file with the initial currency. Refuses to
    /// overwrite an existing ledger (provisioning is explicit).
    pub fn create(
        path: &Path,
        revocation_epoch: i64,
        capability_set_version: i64,
    ) -> Result<Self, ManagementError> {
        if path.exists() {
            return Err(ledger_error(format!(
                "ledger already exists at {}",
                path.display()
            )));
        }
        if revocation_epoch < 0 || capability_set_version < 0 {
            return Err(ledger_error("currency values must be non-negative"));
        }
        let ledger = Self {
            path: path.to_path_buf(),
            revocation_epoch,
            capability_set_version,
            updated_at: WallTimestamp::parse("1970-01-01T00:00:00Z")
                .map_err(|err| ledger_error(format!("epoch timestamp: {err}")))?,
        };
        ledger.persist()?;
        Ok(ledger)
    }

    /// Load an existing ledger file. Any parse or shape failure fails
    /// closed — governance currency is never guessed.
    pub fn load(path: &Path) -> Result<Self, ManagementError> {
        let text = std::fs::read_to_string(path)
            .map_err(|err| ledger_error(format!("read {}: {err}", path.display())))?;
        let value: Value = serde_json::from_str(&text)
            .map_err(|err| ledger_error(format!("parse {}: {err}", path.display())))?;
        let revocation_epoch = value
            .get("revocation_epoch")
            .and_then(Value::as_i64)
            .ok_or_else(|| ledger_error("missing integer `revocation_epoch`"))?;
        let capability_set_version = value
            .get("capability_set_version")
            .and_then(Value::as_i64)
            .ok_or_else(|| ledger_error("missing integer `capability_set_version`"))?;
        if revocation_epoch < 0 || capability_set_version < 0 {
            return Err(ledger_error("currency values must be non-negative"));
        }
        let updated_at = value
            .get("updated_at")
            .and_then(Value::as_str)
            .ok_or_else(|| ledger_error("missing string `updated_at`"))?;
        let updated_at = WallTimestamp::parse(updated_at)
            .map_err(|err| ledger_error(format!("`updated_at` not canonical: {err}")))?;
        Ok(Self {
            path: path.to_path_buf(),
            revocation_epoch,
            capability_set_version,
            updated_at,
        })
    }

    /// The currency the kernel revalidates grants against.
    pub fn currency(&self) -> GovernanceCurrency {
        GovernanceCurrency {
            revocation_epoch: self.revocation_epoch,
            capability_set_version: self.capability_set_version,
        }
    }

    /// Advance the revocation epoch by exactly one and persist durably.
    /// Every grant decided under the previous epoch becomes stale material
    /// from this instant (M3 `revalidate_grant` fails for it). Returns
    /// `(previous_epoch, new_epoch)`.
    pub fn advance_revocation_epoch(
        &mut self,
        now: &WallTimestamp,
    ) -> Result<(i64, i64), ManagementError> {
        let previous = self.revocation_epoch;
        let next = previous
            .checked_add(1)
            .ok_or_else(|| ledger_error("revocation epoch overflow"))?;
        self.revocation_epoch = next;
        self.updated_at = now.clone();
        self.persist()?;
        Ok((previous, next))
    }

    fn persist(&self) -> Result<(), ManagementError> {
        let value = json!({
            "capability_set_version": self.capability_set_version,
            "revocation_epoch": self.revocation_epoch,
            "updated_at": self.updated_at.as_str(),
        });
        let text = canonical_text(&value)?;
        std::fs::write(&self.path, text)
            .map_err(|err| ledger_error(format!("write {}: {err}", self.path.display())))
    }
}
