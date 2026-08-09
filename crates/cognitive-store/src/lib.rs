//! `cognitive-store`: persistence adapter of the CognitiveOS reference
//! implementation.
//!
//! M2 scope (per `docs/plan/DEVELOPMENT-PLAN.md`): the SQLite (WAL)
//! authority store implementing the `cognitive-kernel` port traits —
//! governed object rows with CAS versioning, the append-only event log
//! (storage-level triggers), transition records, the outbox, and hard
//! budget ledger rows — plus the system wall-clock and UUIDv7 adapters.
//! State and event writes commit atomically in one transaction; a failed
//! commit fails closed (`STATE_STORE_UNAVAILABLE` at the kernel gate),
//! never buffering authoritative writes in memory (REQ-REC-003).
//!
//! Technology decision: `docs/adr/0002-sqlite-wal.md` (reference
//! implementation choice, not a CognitiveOS specification requirement).
//! SQLite types stay inside this crate; kernel and domain only ever see
//! the port DTOs.

pub mod artifact_store;
pub mod clock;
pub mod context_store;
pub mod faults;
pub mod ids;
pub mod installation;
pub mod layout;
pub mod memory_store;
pub mod migration;
pub mod personal_db;
pub mod scheduler;
pub mod sqlite;
pub mod worker_authorization;

pub use artifact_store::{ArtifactStore, ArtifactStoreError};
pub use clock::SystemClock;
pub use faults::{CrashHarness, CrashPoint, RecordedDispatch, ScriptedExecutor, ScriptedOutcome};
pub use ids::UuidV7Generator;
pub use installation::{
    InstallationCommit, InstallationEvidence, InstallationStoreError, SqliteInstallationStore,
};
pub use layout::{
    AUTHORITY_DATABASE_FILE_NAME, INSTALLATION_DATABASE_FILE_NAME, PERSONAL_PRODUCT_DIR_NAME,
    PersonalDataLayout, PersonalLayoutError,
};
pub use migration::{
    MigrationExecutionMode, MigrationExecutionReport, MigrationPlanEntry, SqliteMigrationError,
    execute_sqlite_migration_plan,
};
pub use personal_db::{
    PersonalDatabasePrepareReport, apply_database_migration_plan, authority_migration_plan,
    installation_migration_plan, prepare_personal_databases,
};
pub use sqlite::SqliteAuthorityStore;

/// Authority store backend implemented by this crate (ADR-0002).
pub const STORE_BACKEND: &str = "sqlite-wal";

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    #[test]
    fn depends_on_domain_and_kernel_layers() {
        assert_eq!(cognitive_domain::EXECUTION_LIFECYCLE_DOMAINS.len(), 5);
        assert!(!cognitive_kernel::KERNEL_PORTS.is_empty());
    }
}
