#![allow(clippy::unwrap_used)]

use super::campaign_observation::{
    CampaignAuthorization, CampaignExternalStateFixture, CampaignFaultPoint,
    CampaignMutationObservationService, CampaignMutationRequest, CampaignObservationError,
    CampaignOutcomeClass, FixtureBounds,
};
use std::path::PathBuf;
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

#[test]
fn mutation_success_before_receipt_persistence_reconciles_once_after_restart() {
    let root = unique_test_root("success-before-receipt");
    let fixture_root = root.join("fixture");
    let authority_root = root.join("authority");
    let fixture = CampaignExternalStateFixture::open(
        &fixture_root,
        FixtureBounds {
            maximum_records: 8,
            maximum_absolute_delta: 10,
        },
    )
    .unwrap();
    let authorization =
        CampaignAuthorization::authorized("PERSONAL-PERF-EVAL-003", "A7-011").unwrap();
    let service = CampaignMutationObservationService::open(
        &authority_root,
        fixture.endpoint(),
        authorization.clone(),
        7,
    )
    .unwrap();
    let prepared = service
        .persist_effect_before_dispatch(CampaignMutationRequest {
            task_ref: "task://personal/eval003/a7-011".to_owned(),
            expected_fixture_version: 0,
            delta: 1,
            scheduler_lease_epoch: 11,
        })
        .unwrap();
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
    std::fs::remove_dir_all(root).unwrap_or(());
}
