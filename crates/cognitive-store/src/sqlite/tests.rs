//! Focused sqlite open/WAL trigger test (P9-T02/D03).

#![allow(clippy::unwrap_used, clippy::panic)]

use super::SqliteAuthorityStore;
use rusqlite::Connection;

#[test]
fn open_asserts_wal_and_installs_append_only_triggers() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("authority.db");
    drop(SqliteAuthorityStore::open(&path).unwrap());
    let conn = Connection::open(&path).unwrap();
    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap();
    assert_eq!(journal_mode.to_ascii_lowercase(), "wal");
    let triggers: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='trigger'
                 AND name IN ('events_append_only_update','events_append_only_delete',
                              'records_append_only_update','records_append_only_delete')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(triggers, 4);
}
