//! Durable SQLite installation staging and commit adapter.
//!
//! This is deliberately a narrow KRN-owned port. It records the immutable
//! inputs that a later Lane-RUN authority commit must consume; it does not
//! grant a capability, change an AgentInstallation lifecycle state, or claim
//! package provenance verification. D-020 explicitly prohibits introducing a
//! sixth transition table for this purpose.

use rusqlite::{Connection, OptionalExtension, TransactionBehavior};
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use thiserror::Error;

/// Immutable installation schema body for Personal migration plan version 1.
///
/// Shared with `personal_db` so open-path bootstrap and the versioned plan
/// stay identical. Not a machine-contract surface (D-020).
pub(crate) const INSTALLATION_SCHEMA_V1: &str = "
CREATE TABLE IF NOT EXISTS installation_staging (
  package_ref          TEXT PRIMARY KEY,
  package_digest       TEXT NOT NULL,
  adapter_digest       TEXT NOT NULL,
  sandbox_digest       TEXT NOT NULL,
  compatibility_digest TEXT NOT NULL,
  source_mode          TEXT,
  operator_ref         TEXT,
  project_ref          TEXT,
  lockfile_digest      TEXT,
  verification_result  TEXT,
  acquisition_lock     TEXT
) STRICT;

CREATE TABLE IF NOT EXISTS installations (
  package_ref          TEXT PRIMARY KEY,
  package_digest       TEXT NOT NULL,
  adapter_digest       TEXT NOT NULL,
  sandbox_digest       TEXT NOT NULL,
  compatibility_digest TEXT NOT NULL,
  source_mode          TEXT,
  operator_ref         TEXT,
  project_ref          TEXT,
  lockfile_digest      TEXT,
  verification_result  TEXT,
  acquisition_lock     TEXT
) STRICT;

CREATE TRIGGER IF NOT EXISTS installations_append_only_update
BEFORE UPDATE ON installations
BEGIN SELECT RAISE(ABORT, 'append-only: committed installations are immutable'); END;

CREATE TRIGGER IF NOT EXISTS installations_append_only_delete
BEFORE DELETE ON installations
BEGIN SELECT RAISE(ABORT, 'append-only: committed installations are immutable'); END;

CREATE TABLE IF NOT EXISTS installation_root_bindings (
  installation_root   TEXT NOT NULL,
  activation_version  INTEGER NOT NULL,
  package_ref          TEXT NOT NULL,
  acquisition_lock     TEXT NOT NULL,
  PRIMARY KEY (installation_root, activation_version)
) STRICT;

CREATE TABLE IF NOT EXISTS active_installation_roots (
  installation_root   TEXT PRIMARY KEY,
  activation_version  INTEGER NOT NULL,
  package_ref          TEXT NOT NULL,
  acquisition_lock     TEXT NOT NULL
) STRICT;

CREATE TABLE IF NOT EXISTS installation_quarantine (
  installation_root      TEXT NOT NULL,
  activation_version     INTEGER NOT NULL,
  package_ref            TEXT NOT NULL,
  acquisition_lock       TEXT NOT NULL,
  lifecycle_precondition TEXT NOT NULL,
  PRIMARY KEY (installation_root, activation_version)
) STRICT;
";

/// Installation schema body for Personal migration plan version 2.
///
/// Daemon-private Agent registration and inactive instance identity only.
/// Registration does not create a SidecarSession, process, Effect, capability,
/// or Task completion fact.
pub(crate) const INSTALLATION_SCHEMA_V2: &str = "
CREATE TABLE IF NOT EXISTS agent_registrations (
  registration_id      TEXT PRIMARY KEY,
  installation_root    TEXT NOT NULL,
  activation_version   INTEGER NOT NULL,
  package_ref          TEXT NOT NULL,
  acquisition_lock     TEXT NOT NULL,
  adapter_digest       TEXT NOT NULL,
  protocol_digest      TEXT NOT NULL,
  policy_digest        TEXT NOT NULL
) STRICT;

CREATE TRIGGER IF NOT EXISTS agent_registrations_append_only_update
BEFORE UPDATE ON agent_registrations
BEGIN SELECT RAISE(ABORT, 'append-only: agent registrations are immutable'); END;

CREATE TRIGGER IF NOT EXISTS agent_registrations_append_only_delete
BEFORE DELETE ON agent_registrations
BEGIN SELECT RAISE(ABORT, 'append-only: agent registrations are immutable'); END;

CREATE TABLE IF NOT EXISTS agent_instances (
  instance_id       TEXT PRIMARY KEY,
  registration_id   TEXT NOT NULL,
  lifecycle_state   TEXT NOT NULL,
  fencing_epoch     INTEGER NOT NULL
) STRICT;

CREATE TRIGGER IF NOT EXISTS agent_instances_append_only_delete
BEFORE DELETE ON agent_instances
BEGIN SELECT RAISE(ABORT, 'append-only: agent instances may not be deleted'); END;

CREATE TABLE IF NOT EXISTS current_agent_registrations (
  installation_root   TEXT PRIMARY KEY,
  registration_id     TEXT NOT NULL,
  instance_id         TEXT NOT NULL
) STRICT;
";

/// Errors from the local durable installation store.
///
/// These are adapter errors, not protocol error codes: no machine contract is
/// added by this KRN-only persistence slice (D-020).
#[derive(Debug, Error)]
pub enum InstallationStoreError {
    /// The candidate was incomplete and must not reach durable staging.
    #[error("invalid installation commit: {detail}")]
    InvalidCommit { detail: String },
    /// A stage/commit operation conflicted with the current durable contents.
    #[error("installation-store conflict: {detail}")]
    Conflict { detail: String },
    /// SQLite could not durably complete an operation; callers must fail closed.
    #[error("installation-store unavailable: {detail}")]
    Unavailable { detail: String },
}

/// Immutable source-admission evidence bound to a durable installation.
///
/// This is a KRN persistence carrier, not publisher provenance and not an
/// authorization capability. It keeps the Custom acknowledgement in the same
/// stage-to-commit transaction as the package and policy digests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstallationEvidence {
    source_mode: String,
    operator_ref: String,
    project_ref: String,
    lockfile_digest: String,
    verification_result: String,
    acquisition_lock: Option<String>,
}

impl InstallationEvidence {
    /// Evidence for one explicitly acknowledged local project bundle.
    pub fn custom_user_provided(
        operator_ref: impl Into<String>,
        project_ref: impl Into<String>,
        lockfile_digest: impl Into<String>,
        verification_result: impl Into<String>,
    ) -> Result<Self, InstallationStoreError> {
        let evidence = Self {
            source_mode: "custom_user_provided".to_owned(),
            operator_ref: operator_ref.into(),
            project_ref: project_ref.into(),
            lockfile_digest: lockfile_digest.into(),
            verification_result: verification_result.into(),
            acquisition_lock: None,
        };
        if !evidence.operator_ref.starts_with("principal://")
            || !evidence.project_ref.starts_with("file://")
            || evidence.lockfile_digest.trim().is_empty()
            || evidence.verification_result.trim().is_empty()
        {
            return Err(InstallationStoreError::InvalidCommit {
                detail: "Custom evidence requires principal:// operator, file:// bundle, lockfile digest, and verification result".to_owned(),
            });
        }
        Ok(evidence)
    }

    /// Official acquisition evidence, including the signed lock payload.
    pub fn official_pi(
        acquisition_lock: impl Into<String>,
        lockfile_digest: impl Into<String>,
    ) -> Result<Self, InstallationStoreError> {
        let evidence = Self {
            source_mode: "official_pi".to_owned(),
            operator_ref: "official-registry".to_owned(),
            project_ref: "https://registry.npmjs.org/".to_owned(),
            lockfile_digest: lockfile_digest.into(),
            verification_result: "official_acquisition_lock_verified".to_owned(),
            acquisition_lock: Some(acquisition_lock.into()),
        };
        if evidence.lockfile_digest.trim().is_empty()
            || evidence
                .acquisition_lock
                .as_deref()
                .is_none_or(|lock| lock.trim().is_empty())
        {
            return Err(InstallationStoreError::InvalidCommit {
                detail: "Official evidence requires a dependency lock digest and signed acquisition lock"
                    .to_owned(),
            });
        }
        Ok(evidence)
    }

    pub fn source_mode(&self) -> &str {
        &self.source_mode
    }

    pub fn operator_ref(&self) -> &str {
        &self.operator_ref
    }

    pub fn project_ref(&self) -> &str {
        &self.project_ref
    }

    pub fn lockfile_digest(&self) -> &str {
        &self.lockfile_digest
    }

    pub fn verification_result(&self) -> &str {
        &self.verification_result
    }

    /// Canonical official acquisition lock, when this was an official install.
    pub fn acquisition_lock(&self) -> Option<&str> {
        self.acquisition_lock.as_deref()
    }
}

/// Immutable evidence inputs for an eventual managed installation commit.
///
/// The record is intentionally authority-neutral. It proves only that the
/// supplied values crossed the store's staging/commit boundary; Lane-RUN must
/// still validate provenance, sandbox evidence, compatibility, and management
/// authority before it can create an `AgentInstallation`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstallationCommit {
    package_ref: String,
    package_digest: String,
    adapter_digest: String,
    sandbox_digest: String,
    compatibility_digest: String,
    evidence: Option<InstallationEvidence>,
}

/// Immutable durable activation binding for one private installation root.
///
/// The binding records only a committed package reference and the acquisition
/// lock that was consumed. It neither creates an AgentInstance nor starts a
/// process.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstallationRootBinding {
    installation_root: String,
    activation_version: u64,
    package_ref: String,
    acquisition_lock: String,
}

/// Durable record of a quarantined versioned binding.
///
/// Quarantine is append-only: it removes only the active pointer while
/// retaining the immutable binding and package/evidence rows for inspection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstallationQuarantine {
    installation_root: String,
    activation_version: u64,
    package_ref: String,
    acquisition_lock: String,
    lifecycle_precondition: String,
}

impl InstallationQuarantine {
    pub fn installation_root(&self) -> &str {
        &self.installation_root
    }

    pub const fn activation_version(&self) -> u64 {
        self.activation_version
    }

    pub fn package_ref(&self) -> &str {
        &self.package_ref
    }

    pub fn acquisition_lock(&self) -> &str {
        &self.acquisition_lock
    }

    pub fn lifecycle_precondition(&self) -> &str {
        &self.lifecycle_precondition
    }
}

impl InstallationRootBinding {
    pub fn installation_root(&self) -> &str {
        &self.installation_root
    }

    pub const fn activation_version(&self) -> u64 {
        self.activation_version
    }

    pub fn package_ref(&self) -> &str {
        &self.package_ref
    }

    pub fn acquisition_lock(&self) -> &str {
        &self.acquisition_lock
    }
}

impl InstallationCommit {
    /// Construct a complete, non-empty set of immutable installation inputs.
    pub fn new(
        package_ref: impl Into<String>,
        package_digest: impl Into<String>,
        adapter_digest: impl Into<String>,
        sandbox_digest: impl Into<String>,
        compatibility_digest: impl Into<String>,
    ) -> Result<Self, InstallationStoreError> {
        Self::new_optional_evidence(
            package_ref,
            package_digest,
            adapter_digest,
            sandbox_digest,
            compatibility_digest,
            None,
        )
    }

    /// Construct a durable record with source-admission evidence that must
    /// become visible atomically with the package commit.
    pub fn new_with_evidence(
        package_ref: impl Into<String>,
        package_digest: impl Into<String>,
        adapter_digest: impl Into<String>,
        sandbox_digest: impl Into<String>,
        compatibility_digest: impl Into<String>,
        evidence: InstallationEvidence,
    ) -> Result<Self, InstallationStoreError> {
        Self::new_optional_evidence(
            package_ref,
            package_digest,
            adapter_digest,
            sandbox_digest,
            compatibility_digest,
            Some(evidence),
        )
    }

    fn new_optional_evidence(
        package_ref: impl Into<String>,
        package_digest: impl Into<String>,
        adapter_digest: impl Into<String>,
        sandbox_digest: impl Into<String>,
        compatibility_digest: impl Into<String>,
        evidence: Option<InstallationEvidence>,
    ) -> Result<Self, InstallationStoreError> {
        let commit = Self {
            package_ref: package_ref.into(),
            package_digest: package_digest.into(),
            adapter_digest: adapter_digest.into(),
            sandbox_digest: sandbox_digest.into(),
            compatibility_digest: compatibility_digest.into(),
            evidence,
        };
        for (name, value) in [
            ("package_ref", &commit.package_ref),
            ("package_digest", &commit.package_digest),
            ("adapter_digest", &commit.adapter_digest),
            ("sandbox_digest", &commit.sandbox_digest),
            ("compatibility_digest", &commit.compatibility_digest),
        ] {
            if value.trim().is_empty() {
                return Err(InstallationStoreError::InvalidCommit {
                    detail: format!("{name} must not be empty"),
                });
            }
        }
        Ok(commit)
    }

    /// Stable package identity used for staging and eventual lookup.
    pub fn package_ref(&self) -> &str {
        &self.package_ref
    }

    /// Immutable package bytes digest recorded at commit.
    pub fn package_digest(&self) -> &str {
        &self.package_digest
    }

    /// Exact adapter digest bound into the committed installation.
    pub fn adapter_digest(&self) -> &str {
        &self.adapter_digest
    }

    /// Exact sandbox digest bound into the committed installation.
    pub fn sandbox_digest(&self) -> &str {
        &self.sandbox_digest
    }

    /// Exact compatibility digest bound into the committed installation.
    pub fn compatibility_digest(&self) -> &str {
        &self.compatibility_digest
    }

    /// Custom source evidence, if this record was committed through that mode.
    pub fn evidence(&self) -> Option<&InstallationEvidence> {
        self.evidence.as_ref()
    }
}

/// Daemon-private registration of one active installation root as an Agent.
///
/// The instance remains in `registered` lifecycle state until a later activate
/// slice creates a SidecarSession. Registration grants zero capabilities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentRegistrationRecord {
    registration_id: String,
    installation_root: String,
    activation_version: u64,
    package_ref: String,
    acquisition_lock: String,
    adapter_digest: String,
    protocol_digest: String,
    policy_digest: String,
    instance_id: String,
    fencing_epoch: u64,
    lifecycle_state: String,
}

impl AgentRegistrationRecord {
    pub fn registration_id(&self) -> &str {
        &self.registration_id
    }

    pub fn installation_root(&self) -> &str {
        &self.installation_root
    }

    pub const fn activation_version(&self) -> u64 {
        self.activation_version
    }

    pub fn package_ref(&self) -> &str {
        &self.package_ref
    }

    pub fn acquisition_lock(&self) -> &str {
        &self.acquisition_lock
    }

    pub fn adapter_digest(&self) -> &str {
        &self.adapter_digest
    }

    pub fn protocol_digest(&self) -> &str {
        &self.protocol_digest
    }

    pub fn policy_digest(&self) -> &str {
        &self.policy_digest
    }

    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    pub const fn fencing_epoch(&self) -> u64 {
        self.fencing_epoch
    }

    pub fn lifecycle_state(&self) -> &str {
        &self.lifecycle_state
    }
}

/// SQLite WAL store with atomic stage-to-commit visibility.
///
/// Committed rows are the only rows returned to a reader, and staging is never
/// promoted except by [`Self::commit`]. The installation authority invokes
/// [`Self::recover_interrupted_staging`] under its exclusive lifecycle lock;
/// ordinary reader handles never discard another writer's staging.
pub struct SqliteInstallationStore {
    conn: Mutex<Connection>,
}

impl SqliteInstallationStore {
    /// Open a durable installation store without exposing staging rows.
    pub fn open(path: &Path) -> Result<Self, InstallationStoreError> {
        let conn = Connection::open(path).map_err(|err| unavailable("open", err))?;
        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))
            .map_err(|err| unavailable("set journal_mode", err))?;
        if !journal_mode.eq_ignore_ascii_case("wal") {
            return Err(InstallationStoreError::Unavailable {
                detail: format!("installation database refused WAL mode: {journal_mode}"),
            });
        }
        conn.execute_batch(
            "PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;",
        )
        .map_err(|err| unavailable("set pragmas", err))?;
        conn.execute_batch(INSTALLATION_SCHEMA_V1)
            .map_err(|err| unavailable("install schema", err))?;
        ensure_evidence_columns(&conn)?;
        conn.execute_batch(INSTALLATION_SCHEMA_V2)
            .map_err(|err| unavailable("install agent registration schema", err))?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Discard durable but uncommitted staging after a confirmed interrupted
    /// installation attempt.
    ///
    /// Callers must hold their installation-lifecycle exclusion before calling
    /// this method. It is intentionally explicit so opening a reader cannot
    /// erase staging owned by a live writer.
    pub fn recover_interrupted_staging(&self) -> Result<(), InstallationStoreError> {
        let mut conn = self.lock()?;
        let recovery = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|err| unavailable("begin recovery", err))?;
        recovery
            .execute("DELETE FROM installation_staging", [])
            .map_err(|err| unavailable("discard interrupted staging", err))?;
        recovery
            .commit()
            .map_err(|err| unavailable("commit recovery", err))
    }

    /// Durably stage a complete candidate without making it externally visible.
    pub fn stage(&self, commit: &InstallationCommit) -> Result<(), InstallationStoreError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|err| unavailable("begin staging", err))?;
        let inserted = tx.execute(
            "INSERT INTO installation_staging
               (package_ref, package_digest, adapter_digest, sandbox_digest, compatibility_digest,
                source_mode, operator_ref, project_ref, lockfile_digest, verification_result, acquisition_lock)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            (
                &commit.package_ref,
                &commit.package_digest,
                &commit.adapter_digest,
                &commit.sandbox_digest,
                &commit.compatibility_digest,
                commit.evidence.as_ref().map(|e| e.source_mode.as_str()),
                commit.evidence.as_ref().map(|e| e.operator_ref.as_str()),
                commit.evidence.as_ref().map(|e| e.project_ref.as_str()),
                commit.evidence.as_ref().map(|e| e.lockfile_digest.as_str()),
                commit
                    .evidence
                    .as_ref()
                    .map(|e| e.verification_result.as_str()),
                commit
                    .evidence
                    .as_ref()
                    .and_then(|e| e.acquisition_lock.as_deref()),
            ),
        );
        match inserted {
            Ok(_) => tx
                .commit()
                .map_err(|err| unavailable("commit staging", err)),
            Err(err) if is_constraint_violation(&err) => Err(InstallationStoreError::Conflict {
                detail: format!("package {} is already staged", commit.package_ref),
            }),
            Err(err) => Err(unavailable("stage installation", err)),
        }
    }

    /// Atomically promote a staged candidate to the immutable committed view.
    pub fn commit(&self, package_ref: &str) -> Result<(), InstallationStoreError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|err| unavailable("begin installation commit", err))?;
        let promoted = tx.execute(
                "INSERT INTO installations
                   (package_ref, package_digest, adapter_digest, sandbox_digest, compatibility_digest,
                    source_mode, operator_ref, project_ref, lockfile_digest, verification_result, acquisition_lock)
                 SELECT package_ref, package_digest, adapter_digest, sandbox_digest, compatibility_digest,
                        source_mode, operator_ref, project_ref, lockfile_digest, verification_result, acquisition_lock
                   FROM installation_staging WHERE package_ref = ?1",
                [package_ref],
            );
        let promoted = match promoted {
            Ok(promoted) => promoted,
            Err(err) if is_constraint_violation(&err) => {
                return Err(InstallationStoreError::Conflict {
                    detail: format!("package {package_ref} is already committed"),
                });
            }
            Err(err) => return Err(unavailable("promote staged installation", err)),
        };
        if promoted == 0 {
            return Err(InstallationStoreError::Conflict {
                detail: format!("no staged package {package_ref}"),
            });
        }
        tx.execute(
            "DELETE FROM installation_staging WHERE package_ref = ?1",
            [package_ref],
        )
        .map_err(|err| unavailable("clear committed staging", err))?;
        tx.commit()
            .map_err(|err| unavailable("commit installation", err))
    }

    /// Read only a fully committed record; staging is intentionally invisible.
    pub fn committed(
        &self,
        package_ref: &str,
    ) -> Result<Option<InstallationCommit>, InstallationStoreError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT package_ref, package_digest, adapter_digest, sandbox_digest, compatibility_digest,
                    source_mode, operator_ref, project_ref, lockfile_digest, verification_result, acquisition_lock
               FROM installations WHERE package_ref = ?1",
            [package_ref],
            |row| {
                let source_mode: Option<String> = row.get(5)?;
                let operator_ref: Option<String> = row.get(6)?;
                let project_ref: Option<String> = row.get(7)?;
                let lockfile_digest: Option<String> = row.get(8)?;
                let verification_result: Option<String> = row.get(9)?;
                let acquisition_lock: Option<String> = row.get(10)?;
                let evidence = match (
                    source_mode,
                    operator_ref,
                    project_ref,
                    lockfile_digest,
                    verification_result,
                    acquisition_lock,
                ) {
                    (None, None, None, None, None, None) => None,
                    (Some(source_mode), Some(operator_ref), Some(project_ref), Some(lockfile_digest), Some(verification_result), acquisition_lock) => Some(InstallationEvidence {
                        source_mode,
                        operator_ref,
                        project_ref,
                        lockfile_digest,
                        verification_result,
                        acquisition_lock,
                    }),
                    _ => return Err(rusqlite::Error::InvalidQuery),
                };
                Ok(InstallationCommit {
                    package_ref: row.get(0)?,
                    package_digest: row.get(1)?,
                    adapter_digest: row.get(2)?,
                    sandbox_digest: row.get(3)?,
                    compatibility_digest: row.get(4)?,
                    evidence,
                })
            },
        )
        .optional()
        .map_err(|err| unavailable("read committed installation", err))
    }

    /// Atomically append an immutable root binding and publish it as active.
    ///
    /// The expected version is a compare-and-swap fence. A failed insert or
    /// fence check leaves the prior active pointer unchanged.
    pub fn activate_installation_root(
        &self,
        installation_root: &str,
        expected_activation_version: Option<u64>,
        package_ref: &str,
        acquisition_lock: &str,
    ) -> Result<InstallationRootBinding, InstallationStoreError> {
        if installation_root.trim().is_empty()
            || package_ref.trim().is_empty()
            || acquisition_lock.trim().is_empty()
        {
            return Err(InstallationStoreError::InvalidCommit {
                detail: "installation root, package reference, and acquisition lock are required"
                    .to_owned(),
            });
        }

        let mut conn = self.lock()?;
        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|err| unavailable("begin root activation", err))?;
        let current_version: Option<i64> = transaction
            .query_row(
                "SELECT activation_version FROM active_installation_roots WHERE installation_root = ?1",
                [installation_root],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| unavailable("read active installation root", err))?;
        let current_version = current_version
            .map(|value| {
                u64::try_from(value).map_err(|error| InstallationStoreError::Unavailable {
                    detail: format!("invalid active installation-root version: {error}"),
                })
            })
            .transpose()?;
        if current_version != expected_activation_version {
            return Err(InstallationStoreError::Conflict {
                detail: format!(
                    "installation root {installation_root} expected version {expected_activation_version:?}, found {current_version:?}"
                ),
            });
        }
        let activation_version = current_version.unwrap_or(0).checked_add(1).ok_or_else(|| {
            InstallationStoreError::Unavailable {
                detail: "installation-root activation version overflow".to_owned(),
            }
        })?;
        let activation_version_i64 = i64::try_from(activation_version).map_err(|error| {
            InstallationStoreError::Unavailable {
                detail: format!("invalid next installation-root version: {error}"),
            }
        })?;
        transaction
            .execute(
                "INSERT INTO installation_root_bindings
                   (installation_root, activation_version, package_ref, acquisition_lock)
                 VALUES (?1, ?2, ?3, ?4)",
                (
                    installation_root,
                    activation_version_i64,
                    package_ref,
                    acquisition_lock,
                ),
            )
            .map_err(|err| unavailable("append installation-root binding", err))?;
        transaction
            .execute(
                "INSERT INTO active_installation_roots
                   (installation_root, activation_version, package_ref, acquisition_lock)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(installation_root) DO UPDATE SET
                   activation_version = excluded.activation_version,
                   package_ref = excluded.package_ref,
                   acquisition_lock = excluded.acquisition_lock",
                (
                    installation_root,
                    activation_version_i64,
                    package_ref,
                    acquisition_lock,
                ),
            )
            .map_err(|err| unavailable("publish installation-root pointer", err))?;
        transaction
            .commit()
            .map_err(|err| unavailable("commit root activation", err))?;
        Ok(InstallationRootBinding {
            installation_root: installation_root.to_owned(),
            activation_version,
            package_ref: package_ref.to_owned(),
            acquisition_lock: acquisition_lock.to_owned(),
        })
    }

    /// Return the current durable pointer, never a staging candidate.
    pub fn active_installation_root(
        &self,
        installation_root: &str,
    ) -> Result<Option<InstallationRootBinding>, InstallationStoreError> {
        self.read_installation_root_binding(
            "SELECT installation_root, activation_version, package_ref, acquisition_lock
               FROM active_installation_roots WHERE installation_root = ?1",
            installation_root,
        )
    }

    /// Read one immutable prior binding for rollback validation.
    pub fn installation_root_binding(
        &self,
        installation_root: &str,
        activation_version: u64,
    ) -> Result<Option<InstallationRootBinding>, InstallationStoreError> {
        let activation_version = i64::try_from(activation_version).map_err(|error| {
            InstallationStoreError::InvalidCommit {
                detail: format!("invalid installation-root version: {error}"),
            }
        })?;
        let conn = self.lock()?;
        conn.query_row(
            "SELECT installation_root, activation_version, package_ref, acquisition_lock
               FROM installation_root_bindings
              WHERE installation_root = ?1 AND activation_version = ?2",
            (installation_root, activation_version),
            binding_from_row,
        )
        .optional()
        .map_err(|err| unavailable("read installation-root binding", err))
    }

    /// Atomically quarantine the active binding and remove only its pointer.
    ///
    /// The lifecycle precondition is deliberately checked here, inside the
    /// same transaction as the version fence, so callers cannot turn a stale
    /// stopped/absent observation into a successful uninstall.
    pub fn quarantine_active_installation_root(
        &self,
        installation_root: &str,
        expected_activation_version: u64,
        lifecycle_precondition: &str,
    ) -> Result<InstallationQuarantine, InstallationStoreError> {
        if installation_root.trim().is_empty()
            || !matches!(lifecycle_precondition, "stopped" | "absent")
        {
            return Err(InstallationStoreError::InvalidCommit {
                detail: "uninstall requires a root and explicit stopped or absent lifecycle precondition"
                    .to_owned(),
            });
        }
        let expected_version = i64::try_from(expected_activation_version).map_err(|error| {
            InstallationStoreError::InvalidCommit {
                detail: format!("invalid uninstall activation version: {error}"),
            }
        })?;
        let mut conn = self.lock()?;
        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|err| unavailable("begin installation quarantine", err))?;
        let binding = transaction
            .query_row(
                "SELECT installation_root, activation_version, package_ref, acquisition_lock
                   FROM active_installation_roots
                  WHERE installation_root = ?1 AND activation_version = ?2",
                (installation_root, expected_version),
                binding_from_row,
            )
            .optional()
            .map_err(|err| unavailable("read active installation root for quarantine", err))?
            .ok_or_else(|| InstallationStoreError::Conflict {
                detail: format!("active installation root {installation_root} is absent or fenced"),
            })?;
        transaction
            .execute(
                "INSERT INTO installation_quarantine
                   (installation_root, activation_version, package_ref, acquisition_lock,
                    lifecycle_precondition)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                (
                    binding.installation_root(),
                    expected_version,
                    binding.package_ref(),
                    binding.acquisition_lock(),
                    lifecycle_precondition,
                ),
            )
            .map_err(|err| {
                if is_constraint_violation(&err) {
                    InstallationStoreError::Conflict {
                        detail: "installation root binding is already quarantined".to_owned(),
                    }
                } else {
                    unavailable("record installation quarantine", err)
                }
            })?;
        let removed = transaction
            .execute(
                "DELETE FROM active_installation_roots
                  WHERE installation_root = ?1 AND activation_version = ?2
                    AND package_ref = ?3 AND acquisition_lock = ?4",
                (
                    installation_root,
                    expected_version,
                    binding.package_ref(),
                    binding.acquisition_lock(),
                ),
            )
            .map_err(|err| unavailable("remove active installation pointer", err))?;
        if removed != 1 {
            return Err(InstallationStoreError::Conflict {
                detail: "active installation pointer changed during quarantine".to_owned(),
            });
        }
        transaction
            .commit()
            .map_err(|err| unavailable("commit installation quarantine", err))?;
        Ok(InstallationQuarantine {
            installation_root: binding.installation_root().to_owned(),
            activation_version: binding.activation_version(),
            package_ref: binding.package_ref().to_owned(),
            acquisition_lock: binding.acquisition_lock().to_owned(),
            lifecycle_precondition: lifecycle_precondition.to_owned(),
        })
    }

    /// Read a durable quarantine marker without exposing staging state.
    pub fn installation_quarantine(
        &self,
        installation_root: &str,
        activation_version: u64,
    ) -> Result<Option<InstallationQuarantine>, InstallationStoreError> {
        let activation_version = i64::try_from(activation_version).map_err(|error| {
            InstallationStoreError::InvalidCommit {
                detail: format!("invalid quarantine activation version: {error}"),
            }
        })?;
        let conn = self.lock()?;
        conn.query_row(
            "SELECT installation_root, activation_version, package_ref, acquisition_lock,
                    lifecycle_precondition
               FROM installation_quarantine
              WHERE installation_root = ?1 AND activation_version = ?2",
            (installation_root, activation_version),
            |row| {
                let version: i64 = row.get(1)?;
                Ok(InstallationQuarantine {
                    installation_root: row.get(0)?,
                    activation_version: u64::try_from(version)
                        .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(1, version))?,
                    package_ref: row.get(2)?,
                    acquisition_lock: row.get(3)?,
                    lifecycle_precondition: row.get(4)?,
                })
            },
        )
        .optional()
        .map_err(|err| unavailable("read installation quarantine", err))
    }

    /// Atomically register one Agent from the exact active installation root.
    ///
    /// The resulting instance stays in `registered` state. This path never
    /// creates a SidecarSession, process, Effect, capability grant, or Task
    /// completion fact. A second concurrent registration for the same root
    /// fails closed.
    pub fn register_agent_from_active_root(
        &self,
        registration_id: &str,
        instance_id: &str,
        installation_root: &str,
        expected_activation_version: u64,
        package_ref: &str,
        acquisition_lock: &str,
        adapter_digest: &str,
        protocol_digest: &str,
        policy_digest: &str,
    ) -> Result<AgentRegistrationRecord, InstallationStoreError> {
        if registration_id.trim().is_empty()
            || instance_id.trim().is_empty()
            || installation_root.trim().is_empty()
            || package_ref.trim().is_empty()
            || acquisition_lock.trim().is_empty()
            || adapter_digest.trim().is_empty()
            || protocol_digest.trim().is_empty()
            || policy_digest.trim().is_empty()
        {
            return Err(InstallationStoreError::InvalidCommit {
                detail: "agent registration requires non-empty identity and digest fields"
                    .to_owned(),
            });
        }
        let expected_version = i64::try_from(expected_activation_version).map_err(|error| {
            InstallationStoreError::InvalidCommit {
                detail: format!("invalid registration activation version: {error}"),
            }
        })?;
        let mut conn = self.lock()?;
        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|err| unavailable("begin agent registration", err))?;
        let binding = transaction
            .query_row(
                "SELECT installation_root, activation_version, package_ref, acquisition_lock
                   FROM active_installation_roots
                  WHERE installation_root = ?1 AND activation_version = ?2
                    AND package_ref = ?3 AND acquisition_lock = ?4",
                (
                    installation_root,
                    expected_version,
                    package_ref,
                    acquisition_lock,
                ),
                binding_from_row,
            )
            .optional()
            .map_err(|err| unavailable("read active installation root for registration", err))?
            .ok_or_else(|| InstallationStoreError::Conflict {
                detail: format!(
                    "active installation root {installation_root} is absent, mismatched, or fenced"
                ),
            })?;
        let existing: Option<String> = transaction
            .query_row(
                "SELECT registration_id FROM current_agent_registrations
                  WHERE installation_root = ?1",
                [installation_root],
                |row| row.get(0),
            )
            .optional()
            .map_err(|err| unavailable("read current agent registration", err))?;
        if existing.is_some() {
            return Err(InstallationStoreError::Conflict {
                detail: format!(
                    "installation root {installation_root} already has a current agent registration"
                ),
            });
        }
        transaction
            .execute(
                "INSERT INTO agent_registrations
                   (registration_id, installation_root, activation_version, package_ref,
                    acquisition_lock, adapter_digest, protocol_digest, policy_digest)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                (
                    registration_id,
                    binding.installation_root(),
                    expected_version,
                    binding.package_ref(),
                    binding.acquisition_lock(),
                    adapter_digest,
                    protocol_digest,
                    policy_digest,
                ),
            )
            .map_err(|err| {
                if is_constraint_violation(&err) {
                    InstallationStoreError::Conflict {
                        detail: "agent registration identity already exists".to_owned(),
                    }
                } else {
                    unavailable("insert agent registration", err)
                }
            })?;
        const INITIAL_FENCING_EPOCH: i64 = 1;
        transaction
            .execute(
                "INSERT INTO agent_instances
                   (instance_id, registration_id, lifecycle_state, fencing_epoch)
                 VALUES (?1, ?2, 'registered', ?3)",
                (instance_id, registration_id, INITIAL_FENCING_EPOCH),
            )
            .map_err(|err| {
                if is_constraint_violation(&err) {
                    InstallationStoreError::Conflict {
                        detail: "agent instance identity already exists".to_owned(),
                    }
                } else {
                    unavailable("insert agent instance", err)
                }
            })?;
        transaction
            .execute(
                "INSERT INTO current_agent_registrations
                   (installation_root, registration_id, instance_id)
                 VALUES (?1, ?2, ?3)",
                (installation_root, registration_id, instance_id),
            )
            .map_err(|err| {
                if is_constraint_violation(&err) {
                    InstallationStoreError::Conflict {
                        detail: "competing agent registration won the current-root pointer"
                            .to_owned(),
                    }
                } else {
                    unavailable("publish current agent registration", err)
                }
            })?;
        transaction
            .commit()
            .map_err(|err| unavailable("commit agent registration", err))?;
        Ok(AgentRegistrationRecord {
            registration_id: registration_id.to_owned(),
            installation_root: binding.installation_root().to_owned(),
            activation_version: binding.activation_version(),
            package_ref: binding.package_ref().to_owned(),
            acquisition_lock: binding.acquisition_lock().to_owned(),
            adapter_digest: adapter_digest.to_owned(),
            protocol_digest: protocol_digest.to_owned(),
            policy_digest: policy_digest.to_owned(),
            instance_id: instance_id.to_owned(),
            fencing_epoch: 1,
            lifecycle_state: "registered".to_owned(),
        })
    }

    /// Read the current registration pointer for an installation root.
    pub fn current_agent_registration(
        &self,
        installation_root: &str,
    ) -> Result<Option<AgentRegistrationRecord>, InstallationStoreError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT r.registration_id, r.installation_root, r.activation_version, r.package_ref,
                    r.acquisition_lock, r.adapter_digest, r.protocol_digest, r.policy_digest,
                    i.instance_id, i.fencing_epoch, i.lifecycle_state
               FROM current_agent_registrations c
               JOIN agent_registrations r ON r.registration_id = c.registration_id
               JOIN agent_instances i ON i.instance_id = c.instance_id
              WHERE c.installation_root = ?1",
            [installation_root],
            |row| {
                let activation_version: i64 = row.get(2)?;
                let fencing_epoch: i64 = row.get(9)?;
                Ok(AgentRegistrationRecord {
                    registration_id: row.get(0)?,
                    installation_root: row.get(1)?,
                    activation_version: u64::try_from(activation_version).map_err(|_| {
                        rusqlite::Error::IntegralValueOutOfRange(2, activation_version)
                    })?,
                    package_ref: row.get(3)?,
                    acquisition_lock: row.get(4)?,
                    adapter_digest: row.get(5)?,
                    protocol_digest: row.get(6)?,
                    policy_digest: row.get(7)?,
                    instance_id: row.get(8)?,
                    fencing_epoch: u64::try_from(fencing_epoch)
                        .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(9, fencing_epoch))?,
                    lifecycle_state: row.get(10)?,
                })
            },
        )
        .optional()
        .map_err(|err| unavailable("read current agent registration", err))
    }

    /// Return the number of non-visible staging rows, for recovery assertions.
    pub fn staging_count(&self) -> Result<usize, InstallationStoreError> {
        let conn = self.lock()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM installation_staging", [], |row| {
                row.get(0)
            })
            .map_err(|err| unavailable("count staging", err))?;
        usize::try_from(count).map_err(|err| InstallationStoreError::Unavailable {
            detail: format!("invalid staging count: {err}"),
        })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, InstallationStoreError> {
        self.conn
            .lock()
            .map_err(|_| InstallationStoreError::Unavailable {
                detail: "installation connection poisoned".to_owned(),
            })
    }

    fn read_installation_root_binding(
        &self,
        query: &str,
        installation_root: &str,
    ) -> Result<Option<InstallationRootBinding>, InstallationStoreError> {
        let conn = self.lock()?;
        conn.query_row(query, [installation_root], binding_from_row)
            .optional()
            .map_err(|err| unavailable("read active installation root", err))
    }
}

fn binding_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<InstallationRootBinding> {
    let activation_version: i64 = row.get(1)?;
    let activation_version = u64::try_from(activation_version)
        .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(1, activation_version))?;
    Ok(InstallationRootBinding {
        installation_root: row.get(0)?,
        activation_version,
        package_ref: row.get(2)?,
        acquisition_lock: row.get(3)?,
    })
}

fn ensure_evidence_columns(conn: &Connection) -> Result<(), InstallationStoreError> {
    for table in ["installation_staging", "installations"] {
        for column in [
            "source_mode TEXT",
            "operator_ref TEXT",
            "project_ref TEXT",
            "lockfile_digest TEXT",
            "verification_result TEXT",
            "acquisition_lock TEXT",
        ] {
            let statement = format!("ALTER TABLE {table} ADD COLUMN {column}");
            match conn.execute(&statement, []) {
                Ok(_) => {}
                Err(err) if err.to_string().contains("duplicate column name") => {}
                Err(err) => return Err(unavailable("migrate installation evidence", err)),
            }
        }
    }
    Ok(())
}

fn unavailable(what: &str, err: impl std::fmt::Display) -> InstallationStoreError {
    InstallationStoreError::Unavailable {
        detail: format!("{what}: {err}"),
    }
}

fn is_constraint_violation(err: &rusqlite::Error) -> bool {
    matches!(
        err,
        rusqlite::Error::SqliteFailure(failure, _)
            if failure.code == rusqlite::ErrorCode::ConstraintViolation
    )
}
