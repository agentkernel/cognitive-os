#![allow(clippy::expect_used)]

use cognitive_store::{
    MigrationExecutionMode, MigrationPlanEntry, SqliteMigrationError, execute_sqlite_migration_plan,
};
use rusqlite::Connection;
use tempfile::TempDir;

const CREATE_EXAMPLE_TABLE: &str = "CREATE TABLE migration_example (value TEXT NOT NULL) STRICT;";

fn create_database_fixture() -> (TempDir, std::path::PathBuf) {
    let temporary_directory = TempDir::new().expect("temporary directory");
    let database_path = temporary_directory.path().join("authority.sqlite");
    let connection = Connection::open(&database_path).expect("open fixture database");
    connection
        .execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE legacy (value TEXT NOT NULL) STRICT;")
        .expect("create legacy schema");
    connection
        .execute("INSERT INTO legacy (value) VALUES ('preserve-me')", [])
        .expect("insert legacy row");
    drop(connection);
    (temporary_directory, database_path)
}

#[test]
fn dry_run_uses_a_copy_and_apply_is_replay_safe() {
    let (temporary_directory, database_path) = create_database_fixture();
    let migration_plan = [MigrationPlanEntry::new(1, CREATE_EXAMPLE_TABLE)];

    let dry_run_report = execute_sqlite_migration_plan(
        &database_path,
        &migration_plan,
        MigrationExecutionMode::DryRun {
            scratch_database_path: temporary_directory.path().join("dry-run.sqlite"),
        },
    )
    .expect("dry run succeeds");
    assert_eq!(dry_run_report.applied_versions(), &[1]);

    let source_connection = Connection::open(&database_path).expect("reopen source database");
    let source_migration_metadata_exists: i64 = source_connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'schema_migrations'",
            [],
            |row| row.get(0),
        )
        .expect("inspect source schema");
    assert_eq!(source_migration_metadata_exists, 0);
    drop(source_connection);

    let scratch_connection = Connection::open(temporary_directory.path().join("dry-run.sqlite"))
        .expect("open dry-run copy");
    let scratch_migration_metadata_exists: i64 = scratch_connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'schema_migrations'",
            [],
            |row| row.get(0),
        )
        .expect("inspect dry-run schema");
    assert_eq!(scratch_migration_metadata_exists, 1);
    drop(scratch_connection);

    let backup_path = temporary_directory
        .path()
        .join("authority.before-migration.sqlite");
    let apply_report = execute_sqlite_migration_plan(
        &database_path,
        &migration_plan,
        MigrationExecutionMode::Apply {
            backup_database_path: backup_path.clone(),
        },
    )
    .expect("apply succeeds");
    assert_eq!(apply_report.applied_versions(), &[1]);
    assert_eq!(
        apply_report.backup_database_path(),
        Some(backup_path.as_path())
    );

    let backup_connection = Connection::open(&backup_path).expect("open pre-migration backup");
    let backup_migration_metadata_exists: i64 = backup_connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'schema_migrations'",
            [],
            |row| row.get(0),
        )
        .expect("inspect backup schema");
    assert_eq!(backup_migration_metadata_exists, 0);
    let backup_legacy_value: String = backup_connection
        .query_row("SELECT value FROM legacy", [], |row| row.get(0))
        .expect("read preserved backup data");
    assert_eq!(backup_legacy_value, "preserve-me");

    let replay_report = execute_sqlite_migration_plan(
        &database_path,
        &migration_plan,
        MigrationExecutionMode::Apply {
            backup_database_path: temporary_directory
                .path()
                .join("authority.before-replay.sqlite"),
        },
    )
    .expect("reapply succeeds");
    assert!(replay_report.applied_versions().is_empty());
}

#[test]
fn digest_mismatch_rejects_before_running_new_migration_sql() {
    let (temporary_directory, database_path) = create_database_fixture();
    let initial_plan = [MigrationPlanEntry::new(1, CREATE_EXAMPLE_TABLE)];
    execute_sqlite_migration_plan(
        &database_path,
        &initial_plan,
        MigrationExecutionMode::Apply {
            backup_database_path: temporary_directory.path().join("before-initial.sqlite"),
        },
    )
    .expect("initial migration succeeds");

    let mismatched_plan = [
        MigrationPlanEntry::new(
            1,
            "CREATE TABLE migration_example (value TEXT NOT NULL, changed INTEGER) STRICT;",
        ),
        MigrationPlanEntry::new(2, "CREATE TABLE forbidden (value TEXT);"),
    ];
    let error = execute_sqlite_migration_plan(
        &database_path,
        &mismatched_plan,
        MigrationExecutionMode::Apply {
            backup_database_path: temporary_directory.path().join("before-mismatch.sqlite"),
        },
    )
    .expect_err("checksum mismatch must fail closed");
    assert!(matches!(
        error,
        SqliteMigrationError::DigestMismatch { version: 1, .. }
    ));

    let connection = Connection::open(&database_path).expect("open database after rejection");
    let forbidden_table_exists: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'forbidden'",
            [],
            |row| row.get(0),
        )
        .expect("inspect rejected database");
    assert_eq!(forbidden_table_exists, 0);
}

#[test]
fn failed_migration_rolls_back_metadata_and_schema_changes() {
    let (temporary_directory, database_path) = create_database_fixture();
    let invalid_migration_plan = [MigrationPlanEntry::new(
        1,
        "CREATE TABLE partially_created (value TEXT); THIS IS NOT SQL;",
    )];

    let error = execute_sqlite_migration_plan(
        &database_path,
        &invalid_migration_plan,
        MigrationExecutionMode::Apply {
            backup_database_path: temporary_directory.path().join("before-failure.sqlite"),
        },
    )
    .expect_err("invalid migration must fail");
    assert!(matches!(error, SqliteMigrationError::Sqlite { .. }));

    let connection = Connection::open(&database_path).expect("open database after failure");
    let partial_table_exists: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'partially_created'",
            [],
            |row| row.get(0),
        )
        .expect("inspect failed migration");
    assert_eq!(partial_table_exists, 0);
    let migration_metadata_exists: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'schema_migrations'",
            [],
            |row| row.get(0),
        )
        .expect("inspect rolled back migration metadata");
    assert_eq!(migration_metadata_exists, 0);
}
