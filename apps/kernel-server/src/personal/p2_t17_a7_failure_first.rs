//! P2-T17 聚焦负例：崩溃中途变更、重启原键对账、重复 Effect 拒绝。
//! 本地/fixture 证据不得升格为 Gate、release 或 Profile。

#![allow(clippy::unwrap_used)]

use super::campaign_observation::{
    CampaignAuthorization, CampaignExternalStateFixture, CampaignFaultPoint,
    CampaignMutationObservationService, CampaignMutationRequest, CampaignObservationError,
    CampaignOutcomeClass, FixtureBounds, FixtureMutationFault, FixtureQueryFault,
    PreparedCampaignMutation,
};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_test_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "cognitiveos-p2-t17-{label}-{}-{nonce}",
        std::process::id()
    ))
}

fn open_fixture(root: &Path) -> CampaignExternalStateFixture {
    CampaignExternalStateFixture::open(
        root,
        FixtureBounds {
            maximum_records: 8,
            maximum_absolute_delta: 10,
        },
    )
    .unwrap()
}

fn open_service(
    authority_root: &Path,
    fixture: &CampaignExternalStateFixture,
    case_ref: &str,
    epoch: i64,
) -> CampaignMutationObservationService {
    CampaignMutationObservationService::open(
        authority_root,
        fixture.endpoint(),
        CampaignAuthorization::authorized("PERSONAL-PERF-EVAL-003", case_ref).unwrap(),
        epoch,
    )
    .unwrap()
}

fn persist_increment(
    service: &CampaignMutationObservationService,
    case_suffix: &str,
    lease_epoch: i64,
) -> PreparedCampaignMutation {
    service
        .persist_effect_before_dispatch(CampaignMutationRequest {
            task_ref: format!("task://personal/eval003/{case_suffix}"),
            expected_fixture_version: 0,
            delta: 1,
            scheduler_lease_epoch: lease_epoch,
        })
        .unwrap()
}

fn cleanup_tree(root: &Path) {
    std::fs::remove_dir_all(root).unwrap_or(());
}

#[test]
fn mutation_success_before_receipt_persistence_reconciles_once_after_restart() {
    let root = unique_test_root("success-before-receipt");
    let fixture_root = root.join("fixture");
    let authority_root = root.join("authority");
    let fixture = open_fixture(&fixture_root);
    let authorization =
        CampaignAuthorization::authorized("PERSONAL-PERF-EVAL-003", "A7-011").unwrap();
    let service = CampaignMutationObservationService::open(
        &authority_root,
        fixture.endpoint(),
        authorization.clone(),
        7,
    )
    .unwrap();
    let prepared = persist_increment(&service, "a7-011", 11);
    let original_key_digest = prepared.idempotency_key_digest.clone();

    let injected = service
        .dispatch(
            &prepared.run_ref,
            CampaignFaultPoint::MutationAfterReceiptBefore,
            11,
        )
        .unwrap_err();
    assert_eq!(
        injected,
        CampaignObservationError::InjectedCrash(CampaignFaultPoint::MutationAfterReceiptBefore)
    );
    assert_eq!(fixture.mutation_count().unwrap(), 1);
    assert_eq!(fixture.mutation_request_count().unwrap(), 1);
    drop(service);

    let restarted = CampaignMutationObservationService::open(
        &authority_root,
        fixture.endpoint(),
        authorization,
        7,
    )
    .unwrap();
    let observation = restarted
        .reconcile_after_restart(&prepared.run_ref, 11)
        .unwrap();

    assert_eq!(
        observation.outcome_class,
        CampaignOutcomeClass::ReconciledExecuted
    );
    assert_eq!(observation.idempotency_key_digest, original_key_digest);
    assert_eq!(observation.mutation_count, 1);
    assert_eq!(fixture.mutation_count().unwrap(), 1);
    assert_eq!(fixture.mutation_request_count().unwrap(), 1);
    assert_eq!(fixture.query_count().unwrap(), 1);
    assert_eq!(
        fixture.last_query_key_digest().unwrap(),
        Some(original_key_digest)
    );
    assert_eq!(restarted.dispatch_count(&prepared.run_ref).unwrap(), 1);
    assert_eq!(observation.effect_ref, prepared.effect_ref);
    assert!(observation.verification_report_ref.is_some());
    assert_eq!(observation.acceptance_ref, None);

    fixture.cleanup().unwrap();
    assert!(!fixture_root.exists());
    cleanup_tree(&root);
}

#[test]
fn lost_mutation_response_reconciles_by_original_key_without_redispatch() {
    let root = unique_test_root("lost-response");
    let fixture_root = root.join("fixture");
    let authority_root = root.join("authority");
    let fixture = open_fixture(&fixture_root);
    let authorization =
        CampaignAuthorization::authorized("PERSONAL-PERF-EVAL-003", "A7-020").unwrap();
    let service = CampaignMutationObservationService::open(
        &authority_root,
        fixture.endpoint(),
        authorization.clone(),
        16,
    )
    .unwrap();
    let prepared = persist_increment(&service, "a7-020", 20);
    let original_key_digest = prepared.idempotency_key_digest.clone();
    fixture
        .set_mutation_fault(FixtureMutationFault::DropResponseAfterCommit)
        .unwrap();

    assert_eq!(
        service
            .dispatch_without_fault(&prepared.run_ref, 20)
            .unwrap_err(),
        CampaignObservationError::Indeterminate
    );
    assert_eq!(fixture.mutation_count().unwrap(), 1);
    assert_eq!(fixture.mutation_request_count().unwrap(), 1);
    drop(service);

    let restarted = CampaignMutationObservationService::open(
        &authority_root,
        fixture.endpoint(),
        authorization,
        16,
    )
    .unwrap();
    let observation = restarted
        .reconcile_after_restart(&prepared.run_ref, 20)
        .unwrap();

    assert_eq!(
        observation.outcome_class,
        CampaignOutcomeClass::ReconciledExecuted
    );
    assert_eq!(observation.idempotency_key_digest, original_key_digest);
    assert_eq!(observation.mutation_count, 1);
    assert_eq!(fixture.mutation_count().unwrap(), 1);
    assert_eq!(fixture.mutation_request_count().unwrap(), 1);
    assert_eq!(fixture.query_count().unwrap(), 1);
    assert_eq!(
        fixture.last_query_key_digest().unwrap(),
        Some(original_key_digest)
    );
    assert_eq!(restarted.dispatch_count(&prepared.run_ref).unwrap(), 1);
    assert!(observation.verification_report_ref.is_some());
    assert_eq!(observation.acceptance_ref, None);

    fixture.cleanup().unwrap();
    assert!(!fixture_root.exists());
    cleanup_tree(&root);
}

#[test]
fn restart_replays_only_the_original_key_and_keeps_mutation_count_one() {
    let root = unique_test_root("original-key-replay");
    let fixture = open_fixture(&root.join("fixture"));
    let service = open_service(&root.join("authority"), &fixture, "A7-012", 8);
    let prepared = persist_increment(&service, "a7-012", 12);
    let injected = service
        .dispatch(
            &prepared.run_ref,
            CampaignFaultPoint::MutationAfterReceiptBefore,
            12,
        )
        .unwrap_err();
    assert_eq!(
        injected,
        CampaignObservationError::InjectedCrash(CampaignFaultPoint::MutationAfterReceiptBefore)
    );
    drop(service);

    let restarted = open_service(&root.join("authority"), &fixture, "A7-012", 8);
    let observation = restarted
        .reconcile_after_restart(&prepared.run_ref, 12)
        .unwrap();
    assert_eq!(observation.mutation_count, 1);
    assert_eq!(fixture.mutation_request_count().unwrap(), 1);
    assert_eq!(
        fixture.replay_first_recorded_key().unwrap(),
        200,
        "原键重放必须幂等命中已有记录"
    );
    assert_eq!(fixture.mutation_count().unwrap(), 1);
    assert_eq!(fixture.mutation_request_count().unwrap(), 2);
    assert_eq!(restarted.dispatch_count(&prepared.run_ref).unwrap(), 1);
    assert_eq!(observation.acceptance_ref, None);
    fixture.cleanup().unwrap();
    cleanup_tree(&root);
}

#[test]
fn duplicate_effect_with_original_key_is_rejected() {
    let root = unique_test_root("duplicate-effect");
    let fixture = open_fixture(&root.join("fixture"));
    let service = open_service(&root.join("authority"), &fixture, "A7-013", 9);
    let prepared = persist_increment(&service, "a7-013", 13);
    let error = service
        .reject_duplicate_original_key_intent(&prepared.run_ref)
        .unwrap_err();
    assert_eq!(error, CampaignObservationError::DuplicateEffect);
    assert_eq!(fixture.mutation_count().unwrap(), 0);
    fixture.cleanup().unwrap();
    cleanup_tree(&root);
}

#[test]
fn duplicate_dispatch_does_not_mutate_twice() {
    let root = unique_test_root("duplicate-dispatch");
    let fixture = open_fixture(&root.join("fixture"));
    let service = open_service(&root.join("authority"), &fixture, "A7-014", 10);
    let prepared = persist_increment(&service, "a7-014", 14);
    let first = service
        .dispatch_without_fault(&prepared.run_ref, 14)
        .unwrap();
    assert_eq!(first.mutation_count, 1);
    let second = service
        .dispatch_without_fault(&prepared.run_ref, 14)
        .unwrap_err();
    assert!(matches!(
        second,
        CampaignObservationError::Infrastructure(message)
            if message.contains("not dispatchable")
    ));
    assert_eq!(fixture.mutation_count().unwrap(), 1);
    assert_eq!(first.acceptance_ref, None);
    fixture.cleanup().unwrap();
    cleanup_tree(&root);
}

#[test]
fn post_dispatch_fault_points_reconcile_without_redispatch_or_task_acceptance() {
    for (case_ref, case_suffix, fault, expected_queries) in [
        (
            "A7-021",
            "a7-021",
            CampaignFaultPoint::ReceiptAfterEffectCloseBefore,
            1,
        ),
        (
            "A7-022",
            "a7-022",
            CampaignFaultPoint::VerificationBefore,
            2,
        ),
    ] {
        let root = unique_test_root(case_suffix);
        let fixture = open_fixture(&root.join("fixture"));
        let authority_root = root.join("authority");
        let authorization =
            CampaignAuthorization::authorized("PERSONAL-PERF-EVAL-003", case_ref).unwrap();
        let service = CampaignMutationObservationService::open(
            &authority_root,
            fixture.endpoint(),
            authorization.clone(),
            17,
        )
        .unwrap();
        let prepared = persist_increment(&service, case_suffix, 21);

        assert_eq!(
            service.dispatch(&prepared.run_ref, fault, 21).unwrap_err(),
            CampaignObservationError::InjectedCrash(fault)
        );
        let pending = service.observation(&prepared.run_ref).unwrap();
        assert_eq!(pending.verification_report_ref, None);
        assert_eq!(pending.acceptance_ref, None);
        assert_eq!(fixture.mutation_count().unwrap(), 1);
        assert_eq!(fixture.mutation_request_count().unwrap(), 1);
        drop(service);

        let restarted = CampaignMutationObservationService::open(
            &authority_root,
            fixture.endpoint(),
            authorization,
            17,
        )
        .unwrap();
        let observation = restarted
            .reconcile_after_restart(&prepared.run_ref, 21)
            .unwrap();
        assert_eq!(
            observation.outcome_class,
            CampaignOutcomeClass::ReconciledExecuted
        );
        assert_eq!(observation.mutation_count, 1);
        assert_eq!(fixture.mutation_count().unwrap(), 1);
        assert_eq!(fixture.mutation_request_count().unwrap(), 1);
        assert_eq!(fixture.query_count().unwrap(), expected_queries);
        assert_eq!(restarted.dispatch_count(&prepared.run_ref).unwrap(), 1);
        assert!(observation.verification_report_ref.is_some());
        assert_eq!(observation.acceptance_ref, None);
        fixture.cleanup().unwrap();
        cleanup_tree(&root);
    }
}

#[test]
fn crash_before_dispatch_leaves_fixture_untouched_and_restart_indeterminate() {
    let root = unique_test_root("dispatch-before");
    let fixture = open_fixture(&root.join("fixture"));
    let authorization =
        CampaignAuthorization::authorized("PERSONAL-PERF-EVAL-003", "A7-015").unwrap();
    let service = CampaignMutationObservationService::open(
        &root.join("authority"),
        fixture.endpoint(),
        authorization.clone(),
        11,
    )
    .unwrap();
    let prepared = persist_increment(&service, "a7-015", 15);
    let injected = service
        .dispatch(&prepared.run_ref, CampaignFaultPoint::DispatchBefore, 15)
        .unwrap_err();
    assert_eq!(
        injected,
        CampaignObservationError::InjectedCrash(CampaignFaultPoint::DispatchBefore)
    );
    assert_eq!(fixture.mutation_count().unwrap(), 0);
    assert_eq!(fixture.mutation_request_count().unwrap(), 0);
    drop(service);
    let restarted = CampaignMutationObservationService::open(
        &root.join("authority"),
        fixture.endpoint(),
        authorization,
        11,
    )
    .unwrap();
    let observation = restarted
        .reconcile_after_restart(&prepared.run_ref, 15)
        .unwrap();
    assert_eq!(
        observation.outcome_class,
        CampaignOutcomeClass::Indeterminate
    );
    assert_eq!(fixture.mutation_count().unwrap(), 0);
    assert_eq!(fixture.mutation_request_count().unwrap(), 0);
    assert_eq!(fixture.query_count().unwrap(), 0);
    assert_eq!(observation.acceptance_ref, None);
    fixture.cleanup().unwrap();
    cleanup_tree(&root);
}

#[test]
fn fixture_duplicate_conflict_bounds_reset_and_residue_fail_closed() {
    let root = unique_test_root("fixture-contract");
    let fixture_root = root.join("fixture");
    let fixture = open_fixture(&fixture_root);
    assert_eq!(
        fixture
            .apply_bounded_mutation("a7-key-one", 0, 1, "sha256:params-one")
            .unwrap(),
        201
    );
    assert_eq!(
        fixture
            .apply_bounded_mutation("a7-key-one", 0, 1, "sha256:params-one")
            .unwrap(),
        200
    );
    assert_eq!(fixture.mutation_count().unwrap(), 1);
    assert_eq!(fixture.conflict_first_recorded_key().unwrap(), 409);
    assert_eq!(fixture.mutation_count().unwrap(), 1);
    assert_eq!(
        fixture
            .apply_bounded_mutation("a7-key-two", 0, 99, "sha256:params-two")
            .unwrap(),
        409
    );
    fixture.reset(8).unwrap();
    assert_eq!(fixture.mutation_count().unwrap(), 0);
    std::fs::write(fixture_root.join("residue.bin"), b"orphan").unwrap();
    assert_eq!(
        fixture.cleanup().unwrap_err(),
        CampaignObservationError::CleanupResidue
    );
    std::fs::remove_file(fixture_root.join("residue.bin")).unwrap();
    fixture.cleanup().unwrap();
    cleanup_tree(&root);
}

#[test]
fn stale_lease_and_unauthorized_fault_fail_before_external_mutation() {
    let root = unique_test_root("authz-fence");
    let fixture = open_fixture(&root.join("fixture"));
    let service = open_service(&root.join("authority"), &fixture, "A7-016", 12);
    let prepared = persist_increment(&service, "a7-016", 16);
    assert_eq!(
        service
            .dispatch_without_fault(&prepared.run_ref, 99)
            .unwrap_err(),
        CampaignObservationError::StaleLease
    );
    drop(service);
    let disabled = CampaignMutationObservationService::open(
        &root.join("authority"),
        fixture.endpoint(),
        CampaignAuthorization::authorized_faults_disabled("PERSONAL-PERF-EVAL-003", "A7-016")
            .unwrap(),
        12,
    )
    .unwrap();
    assert_eq!(
        disabled
            .dispatch(
                &prepared.run_ref,
                CampaignFaultPoint::MutationAfterReceiptBefore,
                16
            )
            .unwrap_err(),
        CampaignObservationError::FaultUnauthorized
    );
    assert_eq!(fixture.mutation_count().unwrap(), 0);
    fixture.cleanup().unwrap();
    cleanup_tree(&root);
}

#[test]
fn ambiguous_and_timeout_queries_remain_indeterminate() {
    for (case_ref, case_suffix, query_fault) in [
        ("A7-023", "ambiguous-query", FixtureQueryFault::Ambiguous),
        ("A7-024", "timeout-query", FixtureQueryFault::Timeout),
    ] {
        let root = unique_test_root(case_suffix);
        let fixture = open_fixture(&root.join("fixture"));
        let authorization =
            CampaignAuthorization::authorized("PERSONAL-PERF-EVAL-003", case_ref).unwrap();
        let service = CampaignMutationObservationService::open(
            &root.join("authority"),
            fixture.endpoint(),
            authorization.clone(),
            18,
        )
        .unwrap();
        let prepared = persist_increment(&service, case_suffix, 22);
        service
            .dispatch(
                &prepared.run_ref,
                CampaignFaultPoint::MutationAfterReceiptBefore,
                22,
            )
            .unwrap_err();
        drop(service);
        fixture.set_query_fault(query_fault).unwrap();
        let restarted = CampaignMutationObservationService::open(
            &root.join("authority"),
            fixture.endpoint(),
            authorization,
            18,
        )
        .unwrap();
        let observation = restarted
            .reconcile_after_restart(&prepared.run_ref, 22)
            .unwrap();
        assert_eq!(
            observation.outcome_class,
            CampaignOutcomeClass::Indeterminate
        );
        assert_eq!(fixture.mutation_count().unwrap(), 1);
        assert_eq!(fixture.mutation_request_count().unwrap(), 1);
        assert_eq!(fixture.query_count().unwrap(), 1);
        assert_eq!(observation.verification_report_ref, None);
        assert_eq!(observation.acceptance_ref, None);
        fixture.cleanup().unwrap();
        cleanup_tree(&root);
    }
}

#[test]
fn duplicate_restart_worker_is_rejected() {
    let root = unique_test_root("duplicate-worker");
    let fixture = open_fixture(&root.join("fixture"));
    let service = open_service(&root.join("authority"), &fixture, "A7-018", 14);
    let prepared = persist_increment(&service, "a7-018", 18);
    assert_eq!(
        service
            .reject_duplicate_restart_worker(&prepared.run_ref)
            .unwrap_err(),
        CampaignObservationError::DuplicateRestartWorker
    );
    fixture.cleanup().unwrap();
    cleanup_tree(&root);
}

#[test]
fn tampered_post_state_digest_is_receipt_mismatch() {
    let root = unique_test_root("receipt-mismatch");
    let fixture = open_fixture(&root.join("fixture"));
    let authorization =
        CampaignAuthorization::authorized("PERSONAL-PERF-EVAL-003", "A7-019").unwrap();
    let service = CampaignMutationObservationService::open(
        &root.join("authority"),
        fixture.endpoint(),
        authorization.clone(),
        15,
    )
    .unwrap();
    let prepared = persist_increment(&service, "a7-019", 19);
    service
        .dispatch(
            &prepared.run_ref,
            CampaignFaultPoint::MutationAfterReceiptBefore,
            19,
        )
        .unwrap_err();
    drop(service);
    fixture
        .set_query_fault(FixtureQueryFault::TamperedPostStateDigest)
        .unwrap();
    let restarted = CampaignMutationObservationService::open(
        &root.join("authority"),
        fixture.endpoint(),
        authorization,
        15,
    )
    .unwrap();
    assert_eq!(
        restarted
            .reconcile_after_restart(&prepared.run_ref, 19)
            .unwrap_err(),
        CampaignObservationError::ReceiptMismatch
    );
    let observation = restarted.observation(&prepared.run_ref).unwrap();
    assert_eq!(observation.verification_report_ref, None);
    assert_eq!(observation.acceptance_ref, None);
    fixture.cleanup().unwrap();
    cleanup_tree(&root);
}

#[test]
fn tampered_receipt_reference_is_rejected_before_verification() {
    let root = unique_test_root("receipt-reference-mismatch");
    let fixture = open_fixture(&root.join("fixture"));
    let authorization =
        CampaignAuthorization::authorized("PERSONAL-PERF-EVAL-003", "A7-025").unwrap();
    let service = CampaignMutationObservationService::open(
        &root.join("authority"),
        fixture.endpoint(),
        authorization.clone(),
        19,
    )
    .unwrap();
    let prepared = persist_increment(&service, "a7-025", 23);
    service
        .dispatch(
            &prepared.run_ref,
            CampaignFaultPoint::MutationAfterReceiptBefore,
            23,
        )
        .unwrap_err();
    drop(service);
    fixture
        .set_query_fault(FixtureQueryFault::TamperedReceiptRef)
        .unwrap();
    let restarted = CampaignMutationObservationService::open(
        &root.join("authority"),
        fixture.endpoint(),
        authorization,
        19,
    )
    .unwrap();

    assert_eq!(
        restarted
            .reconcile_after_restart(&prepared.run_ref, 23)
            .unwrap_err(),
        CampaignObservationError::ReceiptMismatch
    );
    let observation = restarted.observation(&prepared.run_ref).unwrap();
    assert_eq!(observation.verification_report_ref, None);
    assert_eq!(observation.acceptance_ref, None);
    fixture.cleanup().unwrap();
    cleanup_tree(&root);
}

#[test]
fn duplicate_mutation_count_is_rejected_before_verification() {
    let root = unique_test_root("duplicate-mutation-count");
    let fixture = open_fixture(&root.join("fixture"));
    let authorization =
        CampaignAuthorization::authorized("PERSONAL-PERF-EVAL-003", "A7-026").unwrap();
    let service = CampaignMutationObservationService::open(
        &root.join("authority"),
        fixture.endpoint(),
        authorization.clone(),
        20,
    )
    .unwrap();
    let prepared = persist_increment(&service, "a7-026", 24);
    service
        .dispatch(
            &prepared.run_ref,
            CampaignFaultPoint::MutationAfterReceiptBefore,
            24,
        )
        .unwrap_err();
    drop(service);
    fixture
        .set_query_fault(FixtureQueryFault::DuplicateMutationCount)
        .unwrap();
    let restarted = CampaignMutationObservationService::open(
        &root.join("authority"),
        fixture.endpoint(),
        authorization,
        20,
    )
    .unwrap();

    assert_eq!(
        restarted
            .reconcile_after_restart(&prepared.run_ref, 24)
            .unwrap_err(),
        CampaignObservationError::DuplicateMutation
    );
    let observation = restarted.observation(&prepared.run_ref).unwrap();
    assert_eq!(observation.verification_report_ref, None);
    assert_eq!(observation.acceptance_ref, None);
    fixture.cleanup().unwrap();
    cleanup_tree(&root);
}

#[test]
fn stale_writer_epoch_cannot_reopen_authority_or_mutate_fixture() {
    let root = unique_test_root("stale-writer-epoch");
    let fixture = open_fixture(&root.join("fixture"));
    let authority_root = root.join("authority");
    let current = open_service(&authority_root, &fixture, "A7-027", 21);
    drop(current);

    assert!(matches!(
        CampaignMutationObservationService::open(
            &authority_root,
            fixture.endpoint(),
            CampaignAuthorization::authorized("PERSONAL-PERF-EVAL-003", "A7-027").unwrap(),
            20,
        ),
        Err(CampaignObservationError::StaleEpoch)
    ));
    assert_eq!(fixture.mutation_count().unwrap(), 0);
    assert_eq!(fixture.mutation_request_count().unwrap(), 0);
    fixture.cleanup().unwrap();
    cleanup_tree(&root);
}
