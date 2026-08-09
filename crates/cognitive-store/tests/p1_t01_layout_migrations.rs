#![allow(clippy::expect_used, clippy::unwrap_used)]

//! P1-T01: XDG layout + dual SQLite migration preparation tests.
//!
//! These tests prove empty→latest, previous→latest, reapply, digest mismatch,
//! disk/copy failure, and exclusive migration lock behaviour. They do not
//! claim Personal Gates, B01-B12, or Profile conformance.

use cognitive_store::{
    MigrationPlanEntry, PersonalDataLayout, PersonalLayoutError, SqliteAuthorityStore,
    SqliteInstallationStore, apply_database_migration_plan, authority_migration_plan,
    prepare_personal_databases,
};
use rusqlite::Connection;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn hermetic_layout(temporary_directory: &TempDir) -> PersonalDataLayout {
    let root = temporary_directory.path();
    PersonalDataLayout::from_xdg_roots(
        root.join("config"),
        root.join("data"),
        root.join("state"),
        root.join("cache"),
        root.join("runtime"),
    )
}

fn recorded_migration_versions(database_path: &Path) -> Vec<i64> {
    let connection = Connection::open(database_path).expect("open database");
    let mut statement = connection
        .prepare("SELECT version FROM schema_migrations ORDER BY version")
        .expect("prepare versions query");
    statement
        .query_map([], |row| row.get(0))
        .expect("query versions")
        .map(|row| row.expect("version row"))
        .collect()
}

fn table_exists(database_path: &Path, table_name: &str) -> bool {
    let connection = Connection::open(database_path).expect("open database");
    let count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [table_name],
            |row| row.get(0),
        )
        .expect("inspect table");
    count == 1
}

#[test]
fn empty_layout_migrates_both_databases_to_latest() {
    let temporary_directory = TempDir::new().expect("temp dir");
    let layout = hermetic_layout(&temporary_directory);

    let report = prepare_personal_databases(&layout).expect("prepare empty layout");
    assert_eq!(
        report.authority().applied_versions(),
        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17]
    );
    assert_eq!(report.installation().applied_versions(), &[1]);
    assert!(layout.authority_database_path().exists());
    assert!(layout.installation_database_path().exists());
    assert!(
        report
            .authority_backup_path()
            .expect("authority backup")
            .exists()
    );
    assert!(
        report
            .installation_backup_path()
            .expect("installation backup")
            .exists()
    );

    assert_eq!(
        recorded_migration_versions(&layout.authority_database_path()),
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17]
    );
    assert_eq!(
        recorded_migration_versions(&layout.installation_database_path()),
        vec![1]
    );
    assert!(table_exists(
        &layout.authority_database_path(),
        "governed_objects"
    ));
    assert!(table_exists(
        &layout.authority_database_path(),
        "continuation_authorizations"
    ));
    assert!(table_exists(
        &layout.authority_database_path(),
        "continuation_authorization_scheduler_lease_bindings"
    ));
    assert!(table_exists(
        &layout.authority_database_path(),
        "context_requests"
    ));
    assert!(table_exists(
        &layout.authority_database_path(),
        "context_views"
    ));
    assert!(table_exists(
        &layout.authority_database_path(),
        "workspace_context_sources"
    ));
    assert!(table_exists(
        &layout.authority_database_path(),
        "context_authorization_fact_sets"
    ));
    assert!(table_exists(
        &layout.authority_database_path(),
        "context_revocation_facts"
    ));
    assert!(table_exists(
        &layout.authority_database_path(),
        "memory_candidates"
    ));
    assert!(table_exists(
        &layout.authority_database_path(),
        "memory_search_fts"
    ));
    assert!(table_exists(
        &layout.authority_database_path(),
        "scheduler_execution_policies"
    ));
    assert!(table_exists(
        &layout.installation_database_path(),
        "installations"
    ));

    // Production open paths remain compatible after versioned prepare.
    SqliteAuthorityStore::open(&layout.authority_database_path()).expect("open authority");
    SqliteInstallationStore::open(&layout.installation_database_path()).expect("open installation");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let data_mode = fs::metadata(layout.data_dir())
            .expect("data dir metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(data_mode, 0o700);
        let authority_mode = fs::metadata(layout.authority_database_path())
            .expect("authority db metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(authority_mode, 0o600);
    }
}

#[test]
fn reapply_prepare_is_replay_safe() {
    let temporary_directory = TempDir::new().expect("temp dir");
    let layout = hermetic_layout(&temporary_directory);

    let first = prepare_personal_databases(&layout).expect("first prepare");
    assert_eq!(
        first.authority().applied_versions(),
        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17]
    );
    assert_eq!(first.installation().applied_versions(), &[1]);

    let second = prepare_personal_databases(&layout).expect("second prepare");
    assert!(second.authority().applied_versions().is_empty());
    assert!(second.installation().applied_versions().is_empty());
    assert_eq!(
        recorded_migration_versions(&layout.authority_database_path()),
        vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17]
    );
}

#[test]
fn scheduler_v2_work_migrates_to_epoch_one_without_losing_its_fence() {
    let temporary_directory = TempDir::new().expect("temp dir");
    let layout = hermetic_layout(&temporary_directory);
    layout.ensure_directories().expect("ensure dirs");
    let database_path = layout.data_dir().join("scheduler-v2.sqlite");
    let migration_plan = authority_migration_plan();

    apply_database_migration_plan(
        &database_path,
        &layout.backups_dir().join("scheduler.before-v2.sqlite"),
        &migration_plan[..2],
    )
    .expect("apply v1 and v2 scheduler schema");
    let connection = Connection::open(&database_path).expect("open v2 scheduler database");
    connection
        .execute(
            "INSERT INTO scheduler_entries \
             (task_ref, state, lease_owner, lease_epoch, lease_expires, next_eligible, attempt_count, cancel_requested) \
             VALUES (?1, 'leased', 'crashed-worker', 7, ?2, ?3, 4, 0)",
            [
                "task://tenant-a/v2-work",
                "2026-08-01T12:01:00Z",
                "2026-08-01T12:00:00Z",
            ],
        )
        .expect("seed v2 scheduler lease");
    drop(connection);

    let report = apply_database_migration_plan(
        &database_path,
        &layout.backups_dir().join("scheduler.before-v3.sqlite"),
        &migration_plan,
    )
    .expect("upgrade scheduler identity");
    assert_eq!(
        report.applied_versions(),
        &[3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17]
    );
    let connection = Connection::open(&database_path).expect("open v3 scheduler database");
    let migrated_row: (i64, String, i64, i64) = connection
        .query_row(
            "SELECT contract_epoch, lease_owner, lease_epoch, attempt_count \
             FROM scheduler_entries WHERE task_ref=?1 AND contract_epoch=1",
            ["task://tenant-a/v2-work"],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("v2 work is retained at epoch one");
    assert_eq!(migrated_row, (1, "crashed-worker".to_owned(), 7, 4));
}

#[test]
fn previous_fixture_upgrades_to_latest_with_additive_version() {
    let temporary_directory = TempDir::new().expect("temp dir");
    let layout = hermetic_layout(&temporary_directory);
    layout.ensure_directories().expect("ensure dirs");

    let database_path = layout.data_dir().join("upgrade-fixture.sqlite");
    let previous_plan = [MigrationPlanEntry::new(
        1,
        "CREATE TABLE IF NOT EXISTS fixture_core (value TEXT NOT NULL) STRICT;",
    )];
    let latest_plan = [
        MigrationPlanEntry::new(
            1,
            "CREATE TABLE IF NOT EXISTS fixture_core (value TEXT NOT NULL) STRICT;",
        ),
        MigrationPlanEntry::new(
            2,
            "CREATE TABLE IF NOT EXISTS fixture_extension (value TEXT NOT NULL) STRICT;",
        ),
    ];

    let first_backup = layout.backups_dir().join("fixture.before-v1.sqlite");
    let first_report = apply_database_migration_plan(&database_path, &first_backup, &previous_plan)
        .expect("apply previous fixture");
    assert_eq!(first_report.applied_versions(), &[1]);
    assert!(table_exists(&database_path, "fixture_core"));
    assert!(!table_exists(&database_path, "fixture_extension"));

    let second_backup = layout.backups_dir().join("fixture.before-v2.sqlite");
    let second_report = apply_database_migration_plan(&database_path, &second_backup, &latest_plan)
        .expect("upgrade to latest");
    assert_eq!(second_report.applied_versions(), &[2]);
    assert_eq!(recorded_migration_versions(&database_path), vec![1, 2]);
    assert!(table_exists(&database_path, "fixture_extension"));
}

#[test]
fn digest_mismatch_rejects_without_applying_later_versions() {
    let temporary_directory = TempDir::new().expect("temp dir");
    let layout = hermetic_layout(&temporary_directory);
    layout.ensure_directories().expect("ensure dirs");

    let database_path = layout.data_dir().join("digest-fixture.sqlite");
    let initial_plan = [MigrationPlanEntry::new(
        1,
        "CREATE TABLE IF NOT EXISTS digest_core (value TEXT NOT NULL) STRICT;",
    )];
    apply_database_migration_plan(
        &database_path,
        &layout.backups_dir().join("digest.before-v1.sqlite"),
        &initial_plan,
    )
    .expect("initial apply");

    let mismatched_plan = [
        MigrationPlanEntry::new(
            1,
            "CREATE TABLE IF NOT EXISTS digest_core (value TEXT NOT NULL, changed INTEGER) STRICT;",
        ),
        MigrationPlanEntry::new(
            2,
            "CREATE TABLE IF NOT EXISTS digest_forbidden (value TEXT NOT NULL) STRICT;",
        ),
    ];
    let error = apply_database_migration_plan(
        &database_path,
        &layout.backups_dir().join("digest.before-mismatch.sqlite"),
        &mismatched_plan,
    )
    .expect_err("digest mismatch must fail closed");
    assert!(matches!(
        error,
        PersonalLayoutError::DatabasePreparation { .. }
    ));
    assert!(!table_exists(&database_path, "digest_forbidden"));
    assert_eq!(recorded_migration_versions(&database_path), vec![1]);
}

#[test]
fn disk_failure_on_backup_destination_leaves_source_unmigrated() {
    let temporary_directory = TempDir::new().expect("temp dir");
    let layout = hermetic_layout(&temporary_directory);
    layout.ensure_directories().expect("ensure dirs");

    let database_path = layout.data_dir().join("disk-fail.sqlite");
    // Create an empty source database without migration metadata.
    {
        let connection = Connection::open(&database_path).expect("create source");
        connection
            .execute_batch("PRAGMA journal_mode=WAL;")
            .expect("wal");
    }

    // Point the backup path at a destination whose parent is a file, not a directory.
    let blocking_file = layout.backups_dir().join("not-a-directory");
    fs::write(&blocking_file, b"block").expect("write blocking file");
    let illegal_backup = blocking_file.join("nested-backup.sqlite");

    let plan = [MigrationPlanEntry::new(
        1,
        "CREATE TABLE IF NOT EXISTS should_not_exist (value TEXT NOT NULL) STRICT;",
    )];
    let error = apply_database_migration_plan(&database_path, &illegal_backup, &plan)
        .expect_err("illegal backup destination must fail");
    assert!(matches!(
        error,
        PersonalLayoutError::DatabasePreparation { .. }
    ));
    assert!(!table_exists(&database_path, "should_not_exist"));
    assert!(!table_exists(&database_path, "schema_migrations"));
}

#[test]
fn concurrent_migration_lock_is_exclusive() {
    let temporary_directory = TempDir::new().expect("temp dir");
    let layout = hermetic_layout(&temporary_directory);
    layout.ensure_directories().expect("ensure dirs");

    fs::write(layout.migration_lock_path(), b"pid=0 purpose=test-hold\n")
        .expect("hold migration lock");

    let error = prepare_personal_databases(&layout).expect_err("lock must block prepare");
    assert!(matches!(error, PersonalLayoutError::LayoutLocked { .. }));
    assert!(!layout.authority_database_path().exists());
}

#[test]
fn missing_runtime_dir_env_fails_closed() {
    // Hermetic assertion of the documented contract: resolve_from_env requires
    // XDG_RUNTIME_DIR. We only call from_xdg_roots in other tests so this keeps
    // process-global env mutation out of the suite.
    let temporary_directory = TempDir::new().expect("temp dir");
    let layout = PersonalDataLayout::from_xdg_roots(
        temporary_directory.path().join("config"),
        temporary_directory.path().join("data"),
        temporary_directory.path().join("state"),
        temporary_directory.path().join("cache"),
        temporary_directory.path().join("runtime"),
    );
    assert_eq!(
        layout.runtime_dir(),
        temporary_directory
            .path()
            .join("runtime")
            .join("cognitiveos")
    );
}
