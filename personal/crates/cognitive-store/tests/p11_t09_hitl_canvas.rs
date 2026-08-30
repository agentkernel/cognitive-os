#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P11-T09/D01 HITL ApprovalPreview: reject, narrow, mechanical stale, channel fail-closed.
//! Chat/task cannot complete approval. Host UI E2E is not this crate.

use cognitive_store::{
    ConfirmCaller, PersonalDataLayout, ProjectAggregateError, ProjectAggregateStore,
    prepare_personal_databases,
};
use tempfile::TempDir;

fn store() -> (TempDir, ProjectAggregateStore) {
    let temporary = TempDir::new().expect("temp");
    let root = temporary.path();
    let layout = PersonalDataLayout::from_xdg_roots(
        root.join("config"),
        root.join("data"),
        root.join("state"),
        root.join("cache"),
        root.join("runtime"),
    );
    prepare_personal_databases(&layout).expect("prepare");
    let store = ProjectAggregateStore::open_path(&layout.authority_database_path()).expect("open");
    (temporary, store)
}

fn pending_activation(store: &ProjectAggregateStore) -> (String, String, String) {
    let (draft_id, _) = store.create_draft(b"charter-v1", 10).expect("draft");
    store
        .put_draft_charter(&draft_id, b"charter-body-v1", 11)
        .expect("charter");
    let (preview_id, preview_digest) = store
        .request_preview("activation", &draft_id, b"activation-preview", 12)
        .expect("preview");
    (draft_id, preview_id, preview_digest)
}

#[test]
fn chat_and_task_channel_cannot_complete_approval() {
    let (_tmp, store) = store();
    let (_draft, preview_id, preview_digest) = pending_activation(&store);
    for caller in [ConfirmCaller::TaskChannel, ConfirmCaller::Assistant] {
        let confirm = store
            .confirm_preview(caller, &preview_id, &preview_digest, 13)
            .expect_err("chat confirm");
        assert!(matches!(confirm, ProjectAggregateError::Forbidden { .. }));
        let reject = store
            .reject_preview(caller, &preview_id, &preview_digest, 14)
            .expect_err("chat reject");
        assert!(matches!(reject, ProjectAggregateError::Forbidden { .. }));
        let narrow = store
            .narrow_preview(caller, &preview_id, &preview_digest, b"narrower", 15)
            .expect_err("chat narrow");
        assert!(matches!(narrow, ProjectAggregateError::Forbidden { .. }));
    }
    let still = store
        .preview_detail(&preview_id)
        .expect("detail")
        .expect("row");
    assert_eq!(still.status, "pending");
}

#[test]
fn stale_is_mechanical_base_digest_mismatch_not_time() {
    let (_tmp, store) = store();
    let (_draft, preview_id, preview_digest) = pending_activation(&store);
    let later_ms = 12 + 8 * 24 * 60 * 60 * 1000;
    store
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            later_ms,
        )
        .expect("elapsed wall clock is not stale");
}

#[test]
fn wrong_digest_fail_closed_on_confirm_reject_narrow() {
    let (_tmp, store) = store();
    let (_draft, preview_id, preview_digest) = pending_activation(&store);
    let wrong = "0".repeat(64);
    let confirm = store
        .confirm_preview(ConfirmCaller::OwnerManagement, &preview_id, &wrong, 20)
        .expect_err("wrong confirm digest");
    assert!(matches!(confirm, ProjectAggregateError::Stale { .. }));
    let reject = store
        .reject_preview(ConfirmCaller::OwnerManagement, &preview_id, &wrong, 21)
        .expect_err("wrong reject digest");
    assert!(matches!(reject, ProjectAggregateError::Stale { .. }));
    let narrow = store
        .narrow_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &wrong,
            b"narrower",
            22,
        )
        .expect_err("wrong narrow digest");
    assert!(matches!(narrow, ProjectAggregateError::Stale { .. }));
    let still = store
        .preview_detail(&preview_id)
        .expect("detail")
        .expect("row");
    assert_eq!(still.status, "pending");
    assert_eq!(still.preview_digest, preview_digest);
}

#[test]
fn reject_leaves_receipt_and_rejected_digest_is_not_confirmable() {
    let (_tmp, store) = store();
    let (_draft, preview_id, preview_digest) = pending_activation(&store);
    let receipt = store
        .reject_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            30,
        )
        .expect("reject");
    assert!(receipt.contains(&preview_id));
    let detail = store
        .preview_detail(&preview_id)
        .expect("detail")
        .expect("row");
    assert_eq!(detail.status, "rejected");
    assert_eq!(detail.receipt_ref.as_deref(), Some(receipt.as_str()));
    let confirm = store
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            31,
        )
        .expect_err("rejected digest");
    assert!(matches!(confirm, ProjectAggregateError::Invalid { .. }));
    assert!(store.list_projects(16).expect("list").is_empty());
}

#[test]
fn narrow_mints_new_preview_and_freezes_old_digest() {
    let (_tmp, store) = store();
    let (draft_id, old_id, old_digest) = pending_activation(&store);
    let narrowed = store
        .narrow_preview(
            ConfirmCaller::OwnerManagement,
            &old_id,
            &old_digest,
            b"narrowed-activation-preview",
            40,
        )
        .expect("narrow");
    assert_ne!(narrowed.preview_id, old_id);
    assert_ne!(narrowed.preview_digest, old_digest);
    assert_eq!(narrowed.superseded_preview_id, old_id);
    let old = store.preview_detail(&old_id).expect("old").expect("row");
    assert_eq!(old.status, "superseded");
    assert_eq!(
        old.superseded_by.as_deref(),
        Some(narrowed.preview_id.as_str())
    );
    let old_confirm = store
        .confirm_preview(ConfirmCaller::OwnerManagement, &old_id, &old_digest, 41)
        .expect_err("old digest");
    assert!(matches!(old_confirm, ProjectAggregateError::Invalid { .. }));
    let pending = store.list_pending_previews(&draft_id).expect("pending");
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].preview_id, narrowed.preview_id);
    let result = store
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &narrowed.preview_id,
            &narrowed.preview_digest,
            42,
        )
        .expect("new pending still confirmable");
    assert_eq!(result.kind, "activated");
}

#[test]
fn confirm_still_works_for_a_fresh_pending_after_reject() {
    let (_tmp, store) = store();
    let (draft_id, first_id, first_digest) = pending_activation(&store);
    store
        .reject_preview(ConfirmCaller::OwnerManagement, &first_id, &first_digest, 50)
        .expect("reject first");
    let (second_id, second_digest) = store
        .request_preview("activation", &draft_id, b"second-preview", 51)
        .expect("second preview");
    let result = store
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &second_id,
            &second_digest,
            52,
        )
        .expect("second confirm");
    assert_eq!(result.kind, "activated");
}
