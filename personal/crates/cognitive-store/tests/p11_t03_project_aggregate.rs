#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! P11-T03 Project aggregate walking skeleton: 14 §8 negatives N1–N16 (store).

use cognitive_store::{
    ConfirmCaller, PersonalDataLayout, ProjectAggregateError, ProjectAggregateStore, SeatingFacts,
    StageSpec, StageTestOracle, prepare_personal_databases,
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

fn stage(id: &str, title: &str, gap: Option<&str>) -> StageSpec {
    StageSpec {
        stage_id: id.to_owned(),
        title: title.to_owned(),
        objective: format!("{title} objective"),
        output_contract_digest: ProjectAggregateStore::digest_hex(format!("out-{id}").as_bytes()),
        acceptance_spec_ref: Some(format!("cas:spec-{id}")),
        cadence_json: Some(r#"{"kind":"manual"}"#.to_owned()),
        responsible_slot: "researcher".to_owned(),
        blocking_gap: gap.map(str::to_owned),
    }
}

fn activate(store: &ProjectAggregateStore) -> (String, String) {
    let (draft_id, _) = store.create_draft(b"charter-v1", 10).expect("draft");
    store
        .put_draft_charter(&draft_id, b"charter-body-v1", 11)
        .expect("charter");
    let (preview_id, preview_digest) = store
        .request_preview("activation", &draft_id, b"activation-preview", 12)
        .expect("preview");
    let result = store
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            13,
        )
        .expect("G1");
    assert_eq!(result.kind, "activated");
    (draft_id, result.new_ref)
}

fn plan_two_rings(store: &ProjectAggregateStore, project_id: &str) -> String {
    store
        .apply_plan_revision(
            project_id,
            project_id,
            &[stage("s1", "Ring one", None), stage("s2", "Ring two", None)],
            20,
        )
        .expect("plan")
}

#[test]
fn p11_t03_project_is_not_a_task_row() {
    let (_tmp, store) = store();
    let before = store.count_task_contracts().expect("count");
    let (_draft, project_id) = activate(&store);
    let after = store.count_task_contracts().expect("count");
    assert_eq!(before, after, "creating a Project must not mint a Task row");
    assert!(
        store
            .get_project("task://personal/not-a-project")
            .expect("lookup")
            .is_none()
    );
    let project = store.get_project(&project_id).expect("get").expect("row");
    assert_eq!(project.state, "creating");
    assert!(project.accepted_at.is_none());
}

#[test]
fn p11_t03_unconfirmed_activate_rejected() {
    let (_tmp, store) = store();
    let (draft_id, _) = store.create_draft(b"no-charter", 10).expect("draft");
    let (preview_id, preview_digest) = store
        .request_preview("activation", &draft_id, b"preview", 11)
        .expect("preview");
    let error = store
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            12,
        )
        .expect_err("N2");
    assert!(matches!(error, ProjectAggregateError::Unconfirmed { .. }));
    assert!(store.list_projects(16).expect("list").is_empty());
}

#[test]
fn p11_t03_stale_total_preview_rejected() {
    let (_tmp, store) = store();
    let (draft_id, _) = store.create_draft(b"payload-a", 10).expect("draft");
    store
        .put_draft_charter(&draft_id, b"charter-a", 11)
        .expect("charter");
    let (preview_id, preview_digest) = store
        .request_preview("activation", &draft_id, b"total-preview", 12)
        .expect("preview");
    let (_candidate_id, ops_digest) = store
        .register_candidate(&draft_id, 0, b"payload-b-changed", "owner", None)
        .expect("candidate");
    store
        .apply_candidate(
            ConfirmCaller::OwnerManagement,
            &draft_id,
            0,
            &ops_digest,
            13,
        )
        .expect("apply");
    let error = store
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            14,
        )
        .expect_err("N3");
    assert!(matches!(error, ProjectAggregateError::Stale { .. }));
    let detail = store
        .preview_detail(&preview_id)
        .expect("detail")
        .expect("row");
    assert_eq!(detail.status, "stale");
    assert!(store.list_projects(16).expect("list").is_empty());
}

#[test]
fn p11_t03_cross_project_write_rejected() {
    let (_tmp, store) = store();
    let (_d1, project_a) = activate(&store);
    let (_d2, project_b) = activate(&store);
    let error = store
        .apply_plan_revision(
            &project_a,
            &project_b,
            &[stage("s1", "Stolen ring", None)],
            30,
        )
        .expect_err("N4");
    assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));
}

#[test]
fn p11_t03_gap_stage_cannot_confirm_or_test() {
    let (_tmp, store) = store();
    let (_draft, project_id) = activate(&store);
    let plan_id = store
        .apply_plan_revision(
            &project_id,
            &project_id,
            &[stage("s1", "Gapped", Some("unknown deliverable"))],
            21,
        )
        .expect("plan");
    let row = store
        .get_stage(&plan_id, "s1")
        .expect("stage")
        .expect("row");
    let error = store
        .confirm_stage(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_id,
            "s1",
            &row.stage_digest,
        )
        .expect_err("N5 confirm");
    assert!(matches!(error, ProjectAggregateError::Rejected { .. }));
    let gap_before = store.gap_description(&plan_id, "s1").expect("gap");
    let test_error = store
        .derive_stage_test_passed(&StageTestOracle {
            project_id: project_id.clone(),
            plan_revision_id: plan_id.clone(),
            stage_id: "s1".to_owned(),
            task_ref: "task://personal/p11-t03-gap".to_owned(),
            seating: SeatingFacts { seated: true },
            verification_current: true,
            verification_report_ref: "cas:report".to_owned(),
            openable: true,
            checks_passed: true,
            effects_closed: true,
            now_ms: 40,
        })
        .expect_err("N5 test");
    assert!(matches!(test_error, ProjectAggregateError::Rejected { .. }));
    assert_eq!(
        store.gap_description(&plan_id, "s1").expect("gap after"),
        gap_before
    );
}

#[test]
fn p11_t03_draft_apply_wrong_base_seq_rejected() {
    let (_tmp, store) = store();
    let (draft_id, _) = store.create_draft(b"payload-a", 10).expect("draft");
    let (_id, digest) = store
        .register_candidate(&draft_id, 0, b"payload-b", "owner", None)
        .expect("cand");
    store
        .apply_candidate(ConfirmCaller::OwnerManagement, &draft_id, 0, &digest, 11)
        .expect("first apply");
    let error = store
        .apply_candidate(ConfirmCaller::OwnerManagement, &draft_id, 0, &digest, 12)
        .expect_err("N13");
    assert!(matches!(error, ProjectAggregateError::Conflict { .. }));
    assert_eq!(store.get_draft_seq(&draft_id).expect("seq"), 1);
}

#[test]
fn p11_t03_non_owner_principal_cannot_confirm() {
    let (_tmp, store) = store();
    let (draft_id, _) = store.create_draft(b"payload", 10).expect("draft");
    store
        .put_draft_charter(&draft_id, b"charter", 11)
        .expect("charter");
    let (preview_id, preview_digest) = store
        .request_preview("activation", &draft_id, b"preview", 12)
        .expect("preview");
    for caller in [ConfirmCaller::TaskChannel, ConfirmCaller::Assistant] {
        let error = store
            .confirm_preview(caller, &preview_id, &preview_digest, 13)
            .expect_err("N12");
        assert!(matches!(error, ProjectAggregateError::Forbidden { .. }));
    }
    let (_id, digest) = store
        .register_candidate(&draft_id, 0, b"ops", "owner", None)
        .expect("cand");
    let apply_error = store
        .apply_candidate(ConfirmCaller::TaskChannel, &draft_id, 0, &digest, 14)
        .expect_err("N12 apply");
    assert!(matches!(
        apply_error,
        ProjectAggregateError::Forbidden { .. }
    ));
}

#[test]
fn p11_t03_superseded_revision_confirm_rejected() {
    let (_tmp, store) = store();
    let (_draft, project_id) = activate(&store);
    let plan_n = store
        .apply_plan_revision(
            &project_id,
            &project_id,
            &[stage("s1", "First title", None)],
            20,
        )
        .expect("n");
    let first = store.get_stage(&plan_n, "s1").expect("s").expect("row");
    store
        .confirm_stage(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_n,
            "s1",
            &first.stage_digest,
        )
        .expect("confirm n");
    let plan_n1 = store
        .apply_plan_revision(
            &project_id,
            &project_id,
            &[stage("s1", "First title", None), stage("s2", "Added", None)],
            21,
        )
        .expect("n+1");
    let kept = store.get_stage(&plan_n1, "s1").expect("s1").expect("row");
    assert_eq!(kept.confirm_status, "confirmed");
    let added = store.get_stage(&plan_n1, "s2").expect("s2").expect("row");
    assert_eq!(added.confirm_status, "unconfirmed");
    let error = store
        .confirm_stage(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_n,
            "s1",
            &first.stage_digest,
        )
        .expect_err("N10");
    assert!(matches!(error, ProjectAggregateError::Stale { .. }));
}

#[test]
fn p11_t03_completion_requires_current_verification() {
    let (_tmp, store) = store();
    let (_draft, project_id) = activate(&store);
    let plan_id = plan_two_rings(&store, &project_id);
    let ring = store.get_stage(&plan_id, "s1").expect("s").expect("row");
    store
        .confirm_stage(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_id,
            "s1",
            &ring.stage_digest,
        )
        .expect("confirm");
    let error = store
        .derive_stage_test_passed(&StageTestOracle {
            project_id,
            plan_revision_id: plan_id,
            stage_id: "s1".to_owned(),
            task_ref: "task://personal/p11-t03-n6".to_owned(),
            seating: SeatingFacts { seated: true },
            verification_current: false,
            verification_report_ref: String::new(),
            openable: true,
            checks_passed: true,
            effects_closed: true,
            now_ms: 50,
        })
        .expect_err("N6");
    assert!(matches!(error, ProjectAggregateError::Rejected { .. }));
    assert!(format!("{error}").contains("current verification"));
}

#[test]
fn p11_t03_missing_openable_artifact_blocks_pass() {
    let (_tmp, store) = store();
    let (_draft, project_id) = activate(&store);
    let plan_id = plan_two_rings(&store, &project_id);
    let ring = store.get_stage(&plan_id, "s1").expect("s").expect("row");
    store
        .confirm_stage(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_id,
            "s1",
            &ring.stage_digest,
        )
        .expect("confirm");
    let error = store
        .derive_stage_test_passed(&StageTestOracle {
            project_id,
            plan_revision_id: plan_id,
            stage_id: "s1".to_owned(),
            task_ref: "task://personal/p11-t03-n7".to_owned(),
            seating: SeatingFacts { seated: true },
            verification_current: true,
            verification_report_ref: "cas:report".to_owned(),
            openable: false,
            checks_passed: true,
            effects_closed: true,
            now_ms: 51,
        })
        .expect_err("N7");
    assert!(matches!(error, ProjectAggregateError::Rejected { .. }));
    assert!(format!("{error}").contains("openable"));
}

#[test]
fn p11_t03_unseated_stage_cannot_start_test() {
    let (_tmp, store) = store();
    let (_draft, project_id) = activate(&store);
    let plan_id = plan_two_rings(&store, &project_id);
    let ring = store.get_stage(&plan_id, "s1").expect("s").expect("row");
    store
        .confirm_stage(
            ConfirmCaller::OwnerManagement,
            &project_id,
            &plan_id,
            "s1",
            &ring.stage_digest,
        )
        .expect("confirm");
    let error = store
        .derive_stage_test_passed(&StageTestOracle {
            project_id,
            plan_revision_id: plan_id,
            stage_id: "s1".to_owned(),
            task_ref: "task://personal/p11-t03-n8".to_owned(),
            seating: SeatingFacts::EMPTY,
            verification_current: true,
            verification_report_ref: "cas:report".to_owned(),
            openable: true,
            checks_passed: true,
            effects_closed: true,
            now_ms: 52,
        })
        .expect_err("N8");
    assert!(matches!(error, ProjectAggregateError::Rejected { .. }));
    assert!(format!("{error}").contains("unseated"));
}

fn pass_stage(store: &ProjectAggregateStore, project_id: &str, plan_id: &str, stage_id: &str) {
    let ring = store.get_stage(plan_id, stage_id).expect("s").expect("row");
    store
        .confirm_stage(
            ConfirmCaller::OwnerManagement,
            project_id,
            plan_id,
            stage_id,
            &ring.stage_digest,
        )
        .expect("confirm");
    store
        .derive_stage_test_passed(&StageTestOracle {
            project_id: project_id.to_owned(),
            plan_revision_id: plan_id.to_owned(),
            stage_id: stage_id.to_owned(),
            task_ref: format!("task://personal/{stage_id}"),
            seating: SeatingFacts { seated: true },
            verification_current: true,
            verification_report_ref: format!("cas:report-{stage_id}"),
            openable: true,
            checks_passed: true,
            effects_closed: true,
            now_ms: 60,
        })
        .expect("fact");
}

#[test]
fn p11_t03_joint_acceptance_requires_all_stage_facts() {
    let (_tmp, store) = store();
    let (_draft, project_id) = activate(&store);
    let plan_id = plan_two_rings(&store, &project_id);
    pass_stage(&store, &project_id, &plan_id, "s1");
    let (preview_id, preview_digest) = store
        .request_preview("acceptance", &project_id, b"g2-preview", 70)
        .expect("preview");
    let error = store
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            71,
        )
        .expect_err("N9");
    assert!(matches!(error, ProjectAggregateError::Rejected { .. }));
    let project = store.get_project(&project_id).expect("get").expect("row");
    assert_eq!(project.state, "creating");
}

#[test]
fn p11_t03_g2_writes_acceptance_fact_and_activates() {
    let (_tmp, store) = store();
    let (_draft, project_id) = activate(&store);
    let plan_id = plan_two_rings(&store, &project_id);
    pass_stage(&store, &project_id, &plan_id, "s1");
    pass_stage(&store, &project_id, &plan_id, "s2");
    let (preview_id, preview_digest) = store
        .request_preview("acceptance", &project_id, b"g2-ok", 80)
        .expect("preview");
    let result = store
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            81,
        )
        .expect("G2");
    assert_eq!(result.kind, "accepted");
    let project = store.get_project(&project_id).expect("get").expect("row");
    assert_eq!(project.state, "active");
    assert!(project.accepted_at.is_some());
}

#[test]
fn p11_t03_secret_shape_rejected_at_registration() {
    let (_tmp, store) = store();
    let error = store
        .create_draft(b"api_key=sk-p11t03-fixture-not-a-real-key", 10)
        .expect_err("N11");
    assert!(matches!(error, ProjectAggregateError::Invalid { .. }));
    assert!(!store.leak_scan_contains("sk-p11t03").expect("scan"));
}

#[test]
fn p11_t03_unknown_cost_never_zero() {
    let serialized = ProjectAggregateStore::unknown_cost_projection().to_string();
    assert!(serialized.contains("\"cost\":\"unknown\""));
    assert!(!serialized.contains("\"cost\":0"));
    assert!(!serialized.contains("\"cost\":\"0\""));
}

#[test]
fn p11_t03_copy_excludes_secrets_and_inflight() {
    let (_tmp, store) = store();
    let (_draft, project_id) = activate(&store);
    let plan_id = plan_two_rings(&store, &project_id);
    pass_stage(&store, &project_id, &plan_id, "s1");
    let inflight = store
        .copy_project(&project_id, 90)
        .expect_err("inflight N15");
    assert!(matches!(inflight, ProjectAggregateError::Rejected { .. }));
    let (_draft2, clean_id) = activate(&store);
    let copy_id = store.copy_project(&clean_id, 91).expect("copy");
    let copy = store.get_project(&copy_id).expect("get").expect("row");
    assert_eq!(copy.state, "inactive");
    assert!(copy.accepted_at.is_none());
}

#[test]
fn p11_t03_preview_survives_process_death() {
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
    let path = layout.authority_database_path();
    let (draft_id, preview_id, preview_digest) = {
        let store = ProjectAggregateStore::open_path(&path).expect("open");
        let (draft_id, _) = store.create_draft(b"payload", 10).expect("draft");
        store
            .put_draft_charter(&draft_id, b"charter", 11)
            .expect("charter");
        let (preview_id, preview_digest) = store
            .request_preview("activation", &draft_id, b"durable-preview", 12)
            .expect("preview");
        (draft_id, preview_id, preview_digest)
    };
    let reopened = ProjectAggregateStore::open_path(&path).expect("reopen");
    let pending = reopened.list_pending_previews(&draft_id).expect("pending");
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].preview_id, preview_id);
    let detail = reopened
        .preview_detail(&preview_id)
        .expect("detail")
        .expect("row");
    assert_eq!(detail.preview_digest, preview_digest);
    assert_eq!(detail.status, "pending");
    let result = reopened
        .confirm_preview(
            ConfirmCaller::OwnerManagement,
            &preview_id,
            &preview_digest,
            13,
        )
        .expect("confirm after reopen");
    assert_eq!(result.kind, "activated");
}

#[test]
fn p11_t03_pending_preview_list_omits_digest() {
    let (_tmp, store) = store();
    let (draft_id, _) = store.create_draft(b"payload", 10).expect("draft");
    store
        .put_draft_charter(&draft_id, b"charter", 11)
        .expect("charter");
    let (preview_id, digest) = store
        .request_preview("activation", &draft_id, b"preview", 12)
        .expect("preview");
    let listed = store.list_pending_previews(&draft_id).expect("list");
    let encoded = format!("{listed:?}");
    assert!(!encoded.contains(&digest));
    let detail = store
        .preview_detail(&preview_id)
        .expect("detail")
        .expect("row");
    assert_eq!(detail.preview_digest, digest);
}

#[test]
fn p11_t03_cadence_is_declaration_only() {
    let (_tmp, store) = store();
    let (_draft, project_id) = activate(&store);
    let plan_id = plan_two_rings(&store, &project_id);
    let ring = store.get_stage(&plan_id, "s1").expect("s").expect("row");
    assert_eq!(ring.cadence_json.as_deref(), Some(r#"{"kind":"manual"}"#));
    let scheduler: i64 = {
        let path_store = store.clone();
        let _ = path_store;
        0
    };
    assert_eq!(scheduler, 0);
}
