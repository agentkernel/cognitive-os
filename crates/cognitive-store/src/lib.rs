//! `cognitive-store`: persistence adapter of the CognitiveOS reference
//! implementation.
//!
//! Scope (M2, per `docs/plan/DEVELOPMENT-PLAN.md`): SQLite (WAL) repositories
//! for governed objects, the append-only event log, the outbox, and
//! snapshots — implementing the port traits defined by `cognitive-kernel`.
//! State and event writes commit atomically in one transaction; a failed
//! commit path fails closed (`STATE_STORE_UNAVAILABLE`), never buffering
//! authoritative writes in memory.
//!
//! Technology decision: `docs/adr/0002-sqlite-wal.md` (reference
//! implementation choice, not a CognitiveOS specification requirement).

/// Placeholder marker asserting the crate wires into the workspace.
pub const STORE_BACKEND: &str = "sqlite-wal (planned, M2)";

#[cfg(test)]
mod tests {
    #[test]
    fn depends_on_domain_and_kernel_layers() {
        assert_eq!(cognitive_domain::EXECUTION_LIFECYCLE_DOMAINS.len(), 5);
        assert!(!cognitive_kernel::KERNEL_PORTS.is_empty());
    }
}
