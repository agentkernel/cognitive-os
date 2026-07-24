//! M6 package verification and installation orchestration (Lane-RUN).
//!
//! Companion prose states: SUBMITTED → VERIFIED → ANALYZED → ADAPTED →
//! TESTED → ADMITTED → COMMITTED (any step may REJECTED|QUARANTINED).
//! There is **no** installation transition machine table (D-020); this
//! module follows the prose sequence and uses registered error codes only.
//!
//! Authority visibility: only `COMMITTED` installations appear in
//! [`InstallationLedger::committed_view`]. Staging rows are invisible to
//! that view; crash/interrupt before commit leaves zero committed state.

use cognitive_contracts::canonical;
use cognitive_contracts::generated::error_registry::RegisteredErrorCode;
use cognitive_store::{InstallationCommit, InstallationStoreError, SqliteInstallationStore};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

/// Deterministic installer / verification error carrying a registered code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallerError {
    pub code: &'static str,
    pub detail: String,
}

impl InstallerError {
    pub fn new(code: RegisteredErrorCode, detail: impl Into<String>) -> Self {
        Self {
            code: code.as_str(),
            detail: detail.into(),
        }
    }
}

/// Injected signature / provenance check (crypto stays outside this crate).
pub trait SignatureProvenancePort: Send + Sync {
    fn verify_artifact(
        &self,
        artifact_digest: &str,
        signature_ref: &str,
        provenance_ref: &str,
        artifact: &[u8],
    ) -> Result<(), InstallerError>;
}

/// Always-accepting port for positive-path unit tests.
#[derive(Debug, Default, Clone, Copy)]
pub struct AcceptingSignaturePort;

impl SignatureProvenancePort for AcceptingSignaturePort {
    fn verify_artifact(
        &self,
        _artifact_digest: &str,
        _signature_ref: &str,
        _provenance_ref: &str,
        _artifact: &[u8],
    ) -> Result<(), InstallerError> {
        Ok(())
    }
}

/// Rejects every signature check with the package verification failure code.
#[derive(Debug, Default, Clone, Copy)]
pub struct RejectingSignaturePort;

impl SignatureProvenancePort for RejectingSignaturePort {
    fn verify_artifact(
        &self,
        _artifact_digest: &str,
        _signature_ref: &str,
        _provenance_ref: &str,
        _artifact: &[u8],
    ) -> Result<(), InstallerError> {
        Err(InstallerError::new(
            RegisteredErrorCode::AgentPackageVerificationFailed,
            "signature or provenance verification failed",
        ))
    }
}

/// Inputs for a single install attempt.
#[derive(Debug, Clone)]
pub struct PackageInstallRequest {
    pub package_id: String,
    pub publisher: String,
    pub package_version: String,
    pub artifact: Vec<u8>,
    pub declared_artifact_digest: String,
    pub signature_ref: String,
    pub provenance_ref: String,
    pub adapter_digest: String,
    pub sandbox_digest: String,
    pub compatibility_digest: String,
    pub expected_adapter_digest: String,
    pub expected_sandbox_digest: String,
    pub expected_compatibility_digest: String,
}

/// Pipeline phase (companion prose; not a registered transition table).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallPhase {
    Submitted,
    Verified,
    Analyzed,
    Adapted,
    Tested,
    Admitted,
    Committed,
    Rejected,
    Quarantined,
}

impl InstallPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Submitted => "SUBMITTED",
            Self::Verified => "VERIFIED",
            Self::Analyzed => "ANALYZED",
            Self::Adapted => "ADAPTED",
            Self::Tested => "TESTED",
            Self::Admitted => "ADMITTED",
            Self::Committed => "COMMITTED",
            Self::Rejected => "REJECTED",
            Self::Quarantined => "QUARANTINED",
        }
    }
}

/// Fault injection points for installation atomicity tests (M6-A3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallCrashPoint {
    AfterVerified,
    AfterAdapted,
    AfterTested,
    BeforeCommit,
}

#[derive(Debug, Clone)]
struct StagingRecord {
    package_id: String,
    phase: InstallPhase,
    artifact_digest: String,
    adapter_digest: String,
    sandbox_digest: String,
}

#[derive(Debug, Clone)]
pub struct CommittedInstallation {
    pub package_id: String,
    pub artifact_digest: String,
    pub adapter_digest: String,
    pub sandbox_digest: String,
    pub phase: InstallPhase,
}

/// In-process installation ledger: staging is invisible; only commit publishes.
#[derive(Debug, Default)]
pub struct InstallationLedger {
    inner: Mutex<LedgerInner>,
}

#[derive(Debug, Default)]
struct LedgerInner {
    staging: BTreeMap<String, StagingRecord>,
    committed: BTreeMap<String, CommittedInstallation>,
    capability_grants: u64,
}

impl InstallationLedger {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> MutexGuard<'_, LedgerInner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Committed installations only — half-install staging never appears here.
    pub fn committed_view(&self) -> Vec<CommittedInstallation> {
        self.lock().committed.values().cloned().collect()
    }

    pub fn capability_grants(&self) -> u64 {
        self.lock().capability_grants
    }

    pub fn staging_count(&self) -> usize {
        self.lock().staging.len()
    }

    fn stage(&self, record: StagingRecord) {
        let mut g = self.lock();
        g.staging.insert(record.package_id.clone(), record);
    }

    fn advance_stage(&self, package_id: &str, phase: InstallPhase) -> Result<(), InstallerError> {
        let mut g = self.lock();
        let row = g.staging.get_mut(package_id).ok_or_else(|| {
            InstallerError::new(
                RegisteredErrorCode::StateConflict,
                format!("no staging row for {package_id}"),
            )
        })?;
        row.phase = phase;
        Ok(())
    }

    fn reject_stage(&self, package_id: &str) {
        let mut g = self.lock();
        g.staging.remove(package_id);
    }

    fn commit(&self, package_id: &str) -> Result<CommittedInstallation, InstallerError> {
        let mut g = self.lock();
        let staging = g.staging.remove(package_id).ok_or_else(|| {
            InstallerError::new(
                RegisteredErrorCode::StateConflict,
                format!("commit without staging for {package_id}"),
            )
        })?;
        if staging.phase != InstallPhase::Admitted {
            g.staging.insert(package_id.to_owned(), staging);
            return Err(InstallerError::new(
                RegisteredErrorCode::StateConflict,
                "commit requires ADMITTED staging phase",
            ));
        }
        let committed = CommittedInstallation {
            package_id: staging.package_id,
            artifact_digest: staging.artifact_digest,
            adapter_digest: staging.adapter_digest,
            sandbox_digest: staging.sandbox_digest,
            phase: InstallPhase::Committed,
        };
        g.committed
            .insert(committed.package_id.clone(), committed.clone());
        g.capability_grants = g.capability_grants.saturating_add(1);
        Ok(committed)
    }

    /// Simulate process crash: drop all staging; committed view unchanged.
    pub fn crash_drop_staging(&self) {
        let mut g = self.lock();
        g.staging.clear();
    }
}

/// Durable installation authority boundary backed by the KRN SQLite store.
///
/// It deliberately grants no capability. A future management action may only
/// consume its committed record after its own deterministic authorization and
/// lifecycle checks have succeeded.
pub struct DurableInstallationAuthority {
    store: SqliteInstallationStore,
    lifecycle_lock: Mutex<()>,
}

/// Exclusive installation lifecycle session owned by the deterministic manager.
///
/// This is intentionally the only capability that can stage/commit or recover
/// through [`DurableInstallationAuthority`]. It keeps an ordinary reader from
/// accidentally turning recovery into a deletion path.
pub struct DurableInstallationManager<'authority> {
    authority: &'authority DurableInstallationAuthority,
    _lifecycle_lock: MutexGuard<'authority, ()>,
}

impl DurableInstallationAuthority {
    /// Open the durable store; reader opening alone never clears live staging.
    pub fn open(path: &std::path::Path) -> Result<Self, InstallerError> {
        Ok(Self {
            store: SqliteInstallationStore::open(path).map_err(map_store_error)?,
            lifecycle_lock: Mutex::new(()),
        })
    }

    /// Acquire the exclusive lifecycle session required for durable mutation.
    pub fn acquire_installation_manager(
        &self,
    ) -> Result<DurableInstallationManager<'_>, InstallerError> {
        let lifecycle_lock = self.lifecycle_lock.lock().map_err(|_| {
            InstallerError::new(
                RegisteredErrorCode::StateStoreUnavailable,
                "installation lifecycle lock poisoned",
            )
        })?;
        Ok(DurableInstallationManager {
            authority: self,
            _lifecycle_lock: lifecycle_lock,
        })
    }

    fn recover_interrupted_staging(&self) -> Result<(), InstallerError> {
        self.store
            .recover_interrupted_staging()
            .map_err(map_store_error)
    }

    /// Whether a package has crossed the durable commit boundary.
    pub fn is_committed(&self, package_id: &str) -> Result<bool, InstallerError> {
        self.store
            .committed(package_id)
            .map(|record| record.is_some())
            .map_err(map_store_error)
    }

    /// No capability is granted by durable installation persistence alone.
    pub const fn capability_grants(&self) -> u64 {
        0
    }
}

impl DurableInstallationManager<'_> {
    /// Discard interrupted staging while this manager owns the lifecycle lock.
    pub fn recover_interrupted_installation(&self) -> Result<(), InstallerError> {
        self.authority.recover_interrupted_staging()
    }
}

fn map_store_error(error: InstallationStoreError) -> InstallerError {
    let code = match error {
        InstallationStoreError::InvalidCommit { .. } => {
            RegisteredErrorCode::AgentPackageVerificationFailed
        }
        InstallationStoreError::Conflict { .. } => RegisteredErrorCode::StateConflict,
        InstallationStoreError::Unavailable { .. } => RegisteredErrorCode::StateStoreUnavailable,
    };
    InstallerError::new(code, error.to_string())
}

fn artifact_digest(bytes: &[u8]) -> Result<String, InstallerError> {
    // Domain-separated digest over raw artifact bytes projected as a JSON string
    // of hex is overkill; use digest of the bytes via JSON string of base16.
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let value = Value::String(hex);
    let canonical = canonical::canonical_bytes_of_value(&value).map_err(|e| {
        InstallerError::new(
            RegisteredErrorCode::AgentPackageVerificationFailed,
            e.to_string(),
        )
    })?;
    canonical::digest(&canonical, "agent-package-artifact/0.1").map_err(|e| {
        InstallerError::new(
            RegisteredErrorCode::AgentPackageVerificationFailed,
            e.to_string(),
        )
    })
}

/// Verify package digests / signature / evidence bindings. Fail-closed.
pub fn verify_package(
    req: &PackageInstallRequest,
    signatures: &dyn SignatureProvenancePort,
) -> Result<String, InstallerError> {
    let live = artifact_digest(&req.artifact)?;
    if live != req.declared_artifact_digest {
        return Err(InstallerError::new(
            RegisteredErrorCode::AgentPackageVerificationFailed,
            format!(
                "artifact digest mismatch: live={live} declared={}",
                req.declared_artifact_digest
            ),
        ));
    }
    signatures.verify_artifact(
        &req.declared_artifact_digest,
        &req.signature_ref,
        &req.provenance_ref,
        &req.artifact,
    )?;
    if req.adapter_digest != req.expected_adapter_digest
        || req.sandbox_digest != req.expected_sandbox_digest
        || req.compatibility_digest != req.expected_compatibility_digest
    {
        return Err(InstallerError::new(
            RegisteredErrorCode::AgentPackageVerificationFailed,
            "adapter/sandbox/compatibility evidence digest mismatch",
        ));
    }
    Ok(live)
}

/// Run the install pipeline. Optional crash point aborts before publish.
pub fn install_package(
    ledger: &InstallationLedger,
    req: &PackageInstallRequest,
    signatures: &dyn SignatureProvenancePort,
    crash_at: Option<InstallCrashPoint>,
) -> Result<CommittedInstallation, InstallerError> {
    let live = match verify_package(req, signatures) {
        Ok(d) => d,
        Err(e) => {
            return Err(e);
        }
    };

    ledger.stage(StagingRecord {
        package_id: req.package_id.clone(),
        phase: InstallPhase::Submitted,
        artifact_digest: live.clone(),
        adapter_digest: req.adapter_digest.clone(),
        sandbox_digest: req.sandbox_digest.clone(),
    });
    ledger.advance_stage(&req.package_id, InstallPhase::Verified)?;
    if crash_at == Some(InstallCrashPoint::AfterVerified) {
        ledger.crash_drop_staging();
        return Err(InstallerError::new(
            RegisteredErrorCode::StateConflict,
            "install interrupted after VERIFIED",
        ));
    }

    ledger.advance_stage(&req.package_id, InstallPhase::Analyzed)?;
    ledger.advance_stage(&req.package_id, InstallPhase::Adapted)?;
    if crash_at == Some(InstallCrashPoint::AfterAdapted) {
        ledger.crash_drop_staging();
        return Err(InstallerError::new(
            RegisteredErrorCode::StateConflict,
            "install interrupted after ADAPTED",
        ));
    }

    ledger.advance_stage(&req.package_id, InstallPhase::Tested)?;
    if crash_at == Some(InstallCrashPoint::AfterTested) {
        ledger.crash_drop_staging();
        return Err(InstallerError::new(
            RegisteredErrorCode::StateConflict,
            "install interrupted after TESTED",
        ));
    }

    ledger.advance_stage(&req.package_id, InstallPhase::Admitted)?;
    if crash_at == Some(InstallCrashPoint::BeforeCommit) {
        ledger.crash_drop_staging();
        return Err(InstallerError::new(
            RegisteredErrorCode::StateConflict,
            "install interrupted before COMMITTED",
        ));
    }

    let committed = ledger.commit(&req.package_id)?;
    Ok(committed)
}

/// Verify and durably commit an installation record without granting authority.
pub fn install_package_durable(
    manager: &DurableInstallationManager<'_>,
    req: &PackageInstallRequest,
    signatures: &dyn SignatureProvenancePort,
) -> Result<CommittedInstallation, InstallerError> {
    let live = verify_package(req, signatures)?;
    let commit = InstallationCommit::new(
        &req.package_id,
        live.clone(),
        &req.adapter_digest,
        &req.sandbox_digest,
        &req.compatibility_digest,
    )
    .map_err(map_store_error)?;
    manager
        .authority
        .store
        .stage(&commit)
        .map_err(map_store_error)?;
    manager
        .authority
        .store
        .commit(commit.package_ref())
        .map_err(map_store_error)?;
    if !manager.authority.is_committed(&req.package_id)? {
        return Err(InstallerError::new(
            RegisteredErrorCode::StateStoreUnavailable,
            "installation commit missing after durable commit",
        ));
    }
    Ok(CommittedInstallation {
        package_id: req.package_id.clone(),
        artifact_digest: live,
        adapter_digest: req.adapter_digest.clone(),
        sandbox_digest: req.sandbox_digest.clone(),
        phase: InstallPhase::Committed,
    })
}

/// Explicit reject path: clear staging, never grant capability.
pub fn reject_package(ledger: &InstallationLedger, package_id: &str) {
    ledger.reject_stage(package_id);
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn sample_req(digest_override: Option<&str>) -> PackageInstallRequest {
        let artifact = b"agent-bytes-v1".to_vec();
        let live = artifact_digest(&artifact).unwrap();
        PackageInstallRequest {
            package_id: "pkg://demo".into(),
            publisher: "example".into(),
            package_version: "0.1.0".into(),
            artifact,
            declared_artifact_digest: digest_override.unwrap_or(&live).to_owned(),
            signature_ref: "sig://ok".into(),
            provenance_ref: "prov://ok".into(),
            adapter_digest: "sha256:adapter".into(),
            sandbox_digest: "sha256:sandbox".into(),
            compatibility_digest: "sha256:compat".into(),
            expected_adapter_digest: "sha256:adapter".into(),
            expected_sandbox_digest: "sha256:sandbox".into(),
            expected_compatibility_digest: "sha256:compat".into(),
        }
    }

    #[test]
    fn tampered_artifact_digest_is_rejected_with_zero_commit() {
        let ledger = InstallationLedger::new();
        let req = sample_req(Some(
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        ));
        let err = install_package(&ledger, &req, &AcceptingSignaturePort, None).unwrap_err();
        assert_eq!(err.code, "AGENT_PACKAGE_VERIFICATION_FAILED");
        assert!(ledger.committed_view().is_empty());
        assert_eq!(ledger.capability_grants(), 0);
        assert_eq!(ledger.staging_count(), 0);
    }

    #[test]
    fn invalid_signature_is_rejected() {
        let ledger = InstallationLedger::new();
        let req = sample_req(None);
        let err = install_package(&ledger, &req, &RejectingSignaturePort, None).unwrap_err();
        assert_eq!(err.code, "AGENT_PACKAGE_VERIFICATION_FAILED");
        assert!(ledger.committed_view().is_empty());
        assert_eq!(ledger.capability_grants(), 0);
    }

    #[test]
    fn evidence_digest_mismatch_is_rejected() {
        let ledger = InstallationLedger::new();
        let mut req = sample_req(None);
        req.adapter_digest = "sha256:wrong".into();
        let err = install_package(&ledger, &req, &AcceptingSignaturePort, None).unwrap_err();
        assert_eq!(err.code, "AGENT_PACKAGE_VERIFICATION_FAILED");
        assert!(ledger.committed_view().is_empty());
    }

    #[test]
    fn happy_path_commits_atomically() {
        let ledger = InstallationLedger::new();
        let req = sample_req(None);
        let committed = install_package(&ledger, &req, &AcceptingSignaturePort, None).unwrap();
        assert_eq!(committed.phase, InstallPhase::Committed);
        assert_eq!(ledger.committed_view().len(), 1);
        assert_eq!(ledger.capability_grants(), 1);
        assert_eq!(ledger.staging_count(), 0);
    }

    #[test]
    fn crash_before_commit_leaves_no_half_install() {
        let ledger = InstallationLedger::new();
        let req = sample_req(None);
        let err = install_package(
            &ledger,
            &req,
            &AcceptingSignaturePort,
            Some(InstallCrashPoint::BeforeCommit),
        )
        .unwrap_err();
        assert_eq!(err.code, "STATE_CONFLICT");
        assert!(ledger.committed_view().is_empty());
        assert_eq!(ledger.staging_count(), 0);
        assert_eq!(ledger.capability_grants(), 0);
    }

    #[test]
    fn crash_after_verified_leaves_no_half_install() {
        let ledger = InstallationLedger::new();
        let req = sample_req(None);
        let _ = install_package(
            &ledger,
            &req,
            &AcceptingSignaturePort,
            Some(InstallCrashPoint::AfterVerified),
        )
        .unwrap_err();
        assert!(ledger.committed_view().is_empty());
        assert_eq!(ledger.staging_count(), 0);
    }

    #[test]
    fn durable_authority_commits_only_after_verification_without_capability_grant() {
        let directory = tempfile::tempdir().unwrap();
        let authority =
            DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
        let req = sample_req(None);
        let manager = authority.acquire_installation_manager().unwrap();

        let committed = install_package_durable(&manager, &req, &AcceptingSignaturePort).unwrap();

        assert_eq!(committed.phase, InstallPhase::Committed);
        assert!(authority.is_committed(&req.package_id).unwrap());
        assert_eq!(authority.capability_grants(), 0);
    }

    #[test]
    fn durable_authority_rejects_unverified_package_without_visible_commit() {
        let directory = tempfile::tempdir().unwrap();
        let authority =
            DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
        let req = sample_req(None);
        let manager = authority.acquire_installation_manager().unwrap();

        let error = install_package_durable(&manager, &req, &RejectingSignaturePort).unwrap_err();

        assert_eq!(error.code, "AGENT_PACKAGE_VERIFICATION_FAILED");
        assert!(!authority.is_committed(&req.package_id).unwrap());
        assert_eq!(authority.capability_grants(), 0);
    }

    #[test]
    fn durable_recovery_requires_an_exclusive_installation_manager() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("install.db");
        let authority = DurableInstallationAuthority::open(&path).unwrap();
        let interrupted_writer = SqliteInstallationStore::open(&path).unwrap();
        let staged = InstallationCommit::new(
            "pkg://interrupted",
            "sha256:artifact",
            "sha256:adapter",
            "sha256:sandbox",
            "sha256:compatibility",
        )
        .unwrap();
        interrupted_writer.stage(&staged).unwrap();
        assert_eq!(interrupted_writer.staging_count().unwrap(), 1);
        assert!(!authority.is_committed("pkg://interrupted").unwrap());

        let manager = authority.acquire_installation_manager().unwrap();
        manager.recover_interrupted_installation().unwrap();

        assert_eq!(interrupted_writer.staging_count().unwrap(), 0);
        assert!(!authority.is_committed("pkg://interrupted").unwrap());
    }
}
