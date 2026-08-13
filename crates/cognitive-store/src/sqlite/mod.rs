//! SQLite (WAL) authority store adapter — the reference implementation of
//! the `cognitive-kernel` [`AuthorityStore`] port (ADR-0002).
//!
//! Binding rules implemented here (ADR-0002, all five):
//!
//! 1. One SQLite transaction per authoritative commit: object CAS update +
//!    event append + transition record + optional budget debit + outbox
//!    rows commit together or not at all.
//! 2. `PRAGMA journal_mode=WAL`, `synchronous=FULL` on authority databases
//!    (asserted at open; tests that shortcut durability must say so).
//! 3. CAS is enforced with `WHERE version = ?expected`; zero affected rows
//!    map to [`StorePortError::Conflict`] without side effects.
//! 4. Any failed commit surfaces [`StorePortError::Unavailable`]
//!    (`STATE_STORE_UNAVAILABLE` at the kernel gate) and fails closed;
//!    governed writes are never buffered in memory (REQ-REC-003).
//! 5. Single writer connection per authority database (the connection sits
//!    behind a mutex; readers can open read-only snapshots).
//!
//! Append-only enforcement (REQ-EVT-004) lives in the STORAGE layer:
//! `BEFORE UPDATE` / `BEFORE DELETE` triggers on `events` and
//! `transition_records` abort any rewrite attempt, from any connection.
//!
//! Production helpers are split into cohesive submodules (P9-T02/D03) without
//! behavior change; this façade preserves crate import paths.

mod context;
mod continuation;
mod harness_skill;
mod intent_chain;
mod memory;
mod memory_skill_consumption;
mod protocol;
mod schema;
mod store;
mod util;
mod worker;

pub(crate) use context::*;
pub(crate) use schema::AUTHORITY_SCHEMA_V1;
pub use store::SqliteAuthorityStore;
pub(crate) use util::*;

#[cfg(test)]
mod tests;
