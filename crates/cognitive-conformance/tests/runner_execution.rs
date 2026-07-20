//! M1 runner execution acceptance tests (Lane-CFR).
//!
//! Pins, against the real committed corpus:
//! 1. the five-state distribution of the reference run (honest counts:
//!    every pass is a static-contract execution with evidence, every
//!    behavioral vector stays not-run with a recorded reason);
//! 2. the F-003 closure gate: both governed-object legacy negatives are
//!    actually executed and pass (schema rejects the dual-track shapes);
//! 3. the runner self-check: the deliberately wrong implementation
//!    (schema-valid outputs, wrong behavior) is failed on every corrupted
//!    vector — "schema-valid alone is never pass"
//!    (docs/standards/conformance-evidence.md section 3, DEVELOPMENT-PLAN
//!    M1 acceptance 2).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cognitive_conformance::{ImplementationKind, build_report, enumerate_vectors, execute_all};
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

/// Reference-run distribution over the committed 76-vector corpus. These
/// numbers are intentionally pinned: they may only change together with a
/// reviewed vector or capability change (IMP-17 measured-count discipline).
const TOTAL: usize = 76;
const PASS: usize = 25;
const NOT_RUN: usize = 51;

#[test]
fn reference_run_distribution_is_honest_and_pinned() {
    let root = repo_root();
    let vectors = enumerate_vectors(&root).expect("corpus enumerates");
    assert_eq!(vectors.len(), TOTAL, "vector corpus size changed");
    let outcomes =
        execute_all(&root, &vectors, ImplementationKind::Reference).expect("reference execution");
    let report = build_report(outcomes);

    assert_eq!(report.summary.total_vectors, TOTAL);
    assert_eq!(report.summary.pass, PASS, "pass count drifted");
    assert_eq!(
        report.summary.fail,
        0,
        "reference run must not fail: {:?}",
        {
            report
                .vectors
                .iter()
                .filter(|v| v.result == "fail")
                .map(|v| {
                    (
                        v.id.clone(),
                        v.execution
                            .as_ref()
                            .map(|e| format!("{:?}", e.mismatches))
                            .unwrap_or_default(),
                    )
                })
                .collect::<Vec<_>>()
        }
    );
    assert_eq!(report.summary.not_applicable, 0);
    assert_eq!(report.summary.documented_degradation, 0);
    assert_eq!(report.summary.not_run, NOT_RUN);
    assert_eq!(
        report.summary.pass + report.summary.fail + report.summary.not_run,
        TOTAL
    );

    for vector in &report.vectors {
        match vector.result {
            "pass" | "fail" => {
                let record = vector
                    .execution
                    .as_ref()
                    .unwrap_or_else(|| panic!("{} executed without record", vector.id));
                assert!(
                    !record.grounding.is_empty(),
                    "{} pass lacks machine-asset grounding",
                    vector.id
                );
                assert!(
                    record.compared_fields >= 1,
                    "{} pass compared no fields",
                    vector.id
                );
            }
            "not-run" => {
                assert!(
                    vector.not_run_reason.is_some(),
                    "{} not-run without recorded reason",
                    vector.id
                );
                assert!(
                    vector.execution.is_none(),
                    "{} not-run must not carry an execution record",
                    vector.id
                );
            }
            other => panic!("{} unexpected state {other}", vector.id),
        }
    }
}

#[test]
fn f003_legacy_negatives_are_executed_and_pass() {
    let root = repo_root();
    let vectors = enumerate_vectors(&root).expect("corpus enumerates");
    let outcomes =
        execute_all(&root, &vectors, ImplementationKind::Reference).expect("reference execution");
    for id in ["GOBJ-LEGACY-METADATA-001", "GOBJ-LEGACY-STRONGREF-001"] {
        let outcome = outcomes
            .iter()
            .find(|o| o.id == id)
            .unwrap_or_else(|| panic!("{id} missing from corpus"));
        assert_eq!(
            outcome.result, "pass",
            "{id} must pass under the reference gates"
        );
        let record = outcome.execution.as_ref().expect("execution record");
        assert_eq!(format!("{:?}", record.mode), "SchemaGate");
        assert!(record.mismatches.is_empty());
    }
}

#[test]
fn plan_named_static_contract_vectors_are_executed() {
    let root = repo_root();
    let vectors = enumerate_vectors(&root).expect("corpus enumerates");
    let outcomes =
        execute_all(&root, &vectors, ImplementationKind::Reference).expect("reference execution");
    // DEVELOPMENT-PLAN M1 acceptance 4 names these three vectors.
    let by_id = |id: &str| {
        outcomes
            .iter()
            .find(|o| o.id == id)
            .expect("vector present")
    };
    assert_eq!(by_id("EFFECT-STATE-CLOSURE-008").result, "pass");
    assert_eq!(by_id("CTX-TRUST-004").result, "pass");
    // state-store-degradation stays honestly not-run but must carry the
    // recorded static contract-side assertions.
    let degradation = by_id("STATE-STORE-DEGRADE-001");
    assert_eq!(degradation.result, "not-run");
    let assertions = degradation
        .static_contract_assertions
        .as_ref()
        .expect("static contract assertions recorded");
    assert_eq!(
        assertions
            .pointer("/error_registered/fail_closed_description")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        assertions
            .pointer("/dispatch_requires_durable_intent_guard_in_transition_table")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
}

#[test]
fn wrong_implementation_is_failed_by_the_runner() {
    let root = repo_root();
    let vectors = enumerate_vectors(&root).expect("corpus enumerates");
    let report = cognitive_conformance::self_check(&root, &vectors).expect("self-check runs");
    assert!(
        cognitive_conformance::self_check_passed(&report),
        "self-check failed: {:?}",
        report.corrupted_but_still_passing
    );
    // Every observably corrupted gate must be represented and flipped.
    assert_eq!(report.must_flip.len(), 6, "corrupted vector set drifted");
    assert_eq!(report.flipped_to_fail.len(), 6);
    assert!(report.corrupted_but_still_passing.is_empty());
    for id in [
        "GOBJ-LEGACY-METADATA-001",
        "GOBJ-LEGACY-STRONGREF-001",
        "STATE-CAS-002",
        "EFFECT-STATE-CLOSURE-008",
        "PERF-REPORT-CONTRACT-001",
        "CTX-TRUST-004",
    ] {
        assert!(
            report.flipped_to_fail.iter().any(|f| f == id),
            "{id} was not failed under the wrong implementation"
        );
    }
}

#[test]
fn sample_manifest_stays_all_planned() {
    let root = repo_root();
    let encoding = cognitive_conformance::golden_fixture_digest(&root).expect("golden digest");
    let manifest =
        cognitive_conformance::sample_profile_manifest(&root, &encoding).expect("manifest");
    let profiles = manifest
        .pointer("/cognitiveos_conformance/profiles")
        .and_then(serde_json::Value::as_object)
        .expect("profiles object");
    assert_eq!(profiles.len(), 13);
    assert!(
        profiles.values().all(|v| v == "planned"),
        "static-contract passes must not move any profile off planned"
    );
    assert_eq!(
        manifest
            .pointer("/cognitiveos_conformance/test_runs")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(0),
        "no conformance claim: test_runs stays empty until behavioral evidence exists"
    );
}
