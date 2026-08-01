//! Failure-first coverage for the P2-T03 loop-scoped dispatch barrier.
//!
//! The tests use the real SQLite authority store. They intentionally begin
//! from the store boundary: an executor must never be reached after a durable
//! loop barrier has fenced the dispatch generation.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use cognitive_domain::ObjectId;
use cognitive_kernel::ports::{
    DispatchAdmission, DispatchBinding, LoopDispatchBarrier, QuiescenceStore,
};
use cognitive_store::SqliteAuthorityStore;

fn object_id(number: u64) -> ObjectId {
    ObjectId::parse(&format!("00000000-0000-7000-9000-{number:012x}")).unwrap()
}

#[test]
fn closed_barrier_rejects_a_stale_generation_after_database_reopen() {
    let temporary_directory = tempfile::tempdir().unwrap();
    let database_path = temporary_directory.path().join("authority.db");
    let loop_object_id = object_id(3001);

    let store = SqliteAuthorityStore::open(&database_path).unwrap();
    let open_barrier = store
        .open_loop_dispatch_barrier(&LoopDispatchBarrier::open(loop_object_id.clone(), 1, 1))
        .unwrap();
    let closed_barrier = store
        .close_loop_dispatch_barrier(&loop_object_id, open_barrier.generation, 1)
        .unwrap();
    drop(store);

    let reopened_store = SqliteAuthorityStore::open(&database_path).unwrap();
    let rejected = reopened_store.admit_dispatch(&DispatchAdmission {
        binding: DispatchBinding {
            task_ref: "task://tenant-a/3001".to_owned(),
            contract_epoch: 4,
            loop_object_id,
            dispatch_generation: open_barrier.generation,
        },
        expected_fencing_epoch: 1,
    });

    assert!(rejected.is_err(), "closed barrier must survive recovery");
    assert!(!closed_barrier.dispatch_enabled);
}
