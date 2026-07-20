//! Runner execution acceptance tests (Lane-CFR; M1 static-contract batch +
//! M2 behavioral batch).
//!
//! Pins, against the real committed corpus:
//! 1. the five-state distribution of the reference run (honest counts:
//!    every pass is an executed result with evidence, every behavioral
//!    vector of later milestones stays not-run with a recorded reason);
//! 2. the F-003 closure gate: both governed-object legacy negatives are
//!    actually executed and pass (schema rejects the dual-track shapes);
//! 3. the M2 behavioral executions: STATE-CAS-002 /
//!    EFFECT-STATE-CLOSURE-008 / GW-REMOTE-COMPLETE-001 run against the
//!    real `cognitive-kernel` gate over the `cognitive-store` SQLite WAL
//!    adapter, and STATE-STORE-DEGRADE-001 carries the real read-only
//!    degradation subset as recorded assertions (still not-run);
//! 4. the runner self-check: the deliberately wrong implementation
//!    (schema-valid outputs, wrong behavior; behaviorally a gate-bypassing
//!    direct store writer) is failed on every corrupted vector —
//!    "schema-valid alone is never pass"
//!    (docs/standards/conformance-evidence.md section 3, DEVELOPMENT-PLAN
//!    M1 acceptance 2 and M2 review).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cognitive_conformance::{ImplementationKind, build_report, enumerate_vectors, execute_all};
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

/// Reference-run distribution over the committed 81-vector corpus. These
/// numbers are intentionally pinned: they may only change together with a
/// reviewed vector or capability change (IMP-17 measured-count discipline).
/// 2026-07-20 Lane-CFR M4 batch: seven effect/recovery vectors leave
/// not-run (behavioral execution through the fault-injection framework),
/// so pass 39 -> 46, not-run 42 -> 35.
/// 2026-07-20 Lane-CTR F-011 batch: +3 behavioral management-approval
/// negatives (MGMT-APPROVAL-R1-009/SELF-010/FATIGUE-011), honestly not-run
/// until the CFR M5 behavioral batch.
const TOTAL: usize = 84;
const PASS: usize = 46;
const NOT_RUN: usize = 38;

/// The M2 kernel-behavioral executions and their report modes.
const BEHAVIORAL: [(&str, &str); 3] = [
    ("STATE-CAS-002", "CasBehavior"),
    ("EFFECT-STATE-CLOSURE-008", "EffectClosureBehavior"),
    ("GW-REMOTE-COMPLETE-001", "TaskAcceptanceBehavior"),
];

/// The M3 governance/context behavioral executions and their report modes.
const BEHAVIORAL_M3: [(&str, &str); 9] = [
    ("GOBJ-TENANT-LATERAL-001", "LateralReadBehavior"),
    ("CAP-ATTEN-004", "AttenuationBehavior"),
    ("CTX-REVOKE-CACHE-001", "RevocationCacheBehavior"),
    ("CTX-RANK-AUTH-001", "RankBeforeAuthBehavior"),
    ("CTX-REQ-007", "RequiredBudgetBehavior"),
    ("CTX-RENDER-001", "RenderStabilityBehavior"),
    ("DISC-STAGNATION-004", "StagnationBehavior"),
    ("DISC-ADMISSION-002", "CandidateAdmissionBehavior"),
    ("CTX-TRUST-004", "TrustPlaneBehavior"),
];

/// The M4 effect/recovery behavioral executions and their report modes.
const BEHAVIORAL_M4: [(&str, &str); 7] = [
    ("EFF-CRASH-001", "CrashPoint1Behavior"),
    ("EFF-CRASH-002", "CrashPoint2Behavior"),
    ("EFF-CRASH-003", "CrashPoint3Behavior"),
    ("RECOVERY-CRASH-006", "CrashRecoveryBehavior"),
    ("EFF-UNK-003", "UnknownOutcomeBehavior"),
    ("EFF-IDEM-CONFLICT-001", "IdempotencyConflictBehavior"),
    ("AGENT-RECOVERY-003", "RecoveryReconciliationBehavior"),
];

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

/// M3 governance/context behavioral executions run against the real
/// authz/context/context_cache/capability surface — the execution record
/// must say so, and every one must pass.
#[test]
fn m3_behavioral_vectors_execute_against_the_governance_surface() {
    let root = repo_root();
    let vectors = enumerate_vectors(&root).expect("corpus enumerates");
    let outcomes =
        execute_all(&root, &vectors, ImplementationKind::Reference).expect("reference execution");
    for (id, mode) in BEHAVIORAL_M3 {
        let outcome = outcomes
            .iter()
            .find(|o| o.id == id)
            .unwrap_or_else(|| panic!("{id} missing from corpus"));
        assert_eq!(
            outcome.result,
            "pass",
            "{id} must pass behaviorally: {:?}",
            outcome.execution.as_ref().map(|e| &e.mismatches)
        );
        let record = outcome.execution.as_ref().expect("execution record");
        assert_eq!(format!("{:?}", record.mode), mode);
        assert!(
            record
                .implementation
                .contains("authz/context/context_cache"),
            "{id} implementation label must name the M3 surface, got {}",
            record.implementation
        );
    }
    // Delta consumption is M5: the vector stays honestly not-run with the
    // recorded reason.
    let delta = outcomes
        .iter()
        .find(|o| o.id == "DISC-DELTA-SCOPE-003")
        .expect("delta vector present");
    assert_eq!(delta.result, "not-run");
    assert!(
        delta
            .not_run_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("M5")),
        "delta not-run reason must record the M5 deferral"
    );
}

/// M4 effect/recovery behavioral executions run through the public
/// fault-injection framework — the execution record must say so, every one
/// must pass, and the degradation vector must carry the executed fencing
/// subset.
#[test]
fn m4_behavioral_vectors_execute_through_the_fault_framework() {
    let root = repo_root();
    let vectors = enumerate_vectors(&root).expect("corpus enumerates");
    let outcomes =
        execute_all(&root, &vectors, ImplementationKind::Reference).expect("reference execution");
    for (id, mode) in BEHAVIORAL_M4 {
        let outcome = outcomes
            .iter()
            .find(|o| o.id == id)
            .unwrap_or_else(|| panic!("{id} missing from corpus"));
        assert_eq!(
            outcome.result,
            "pass",
            "{id} must pass behaviorally: {:?}",
            outcome.execution.as_ref().map(|e| &e.mismatches)
        );
        let record = outcome.execution.as_ref().expect("execution record");
        assert_eq!(format!("{:?}", record.mode), mode);
        assert!(
            record.implementation.contains("faults"),
            "{id} implementation label must name the fault-injection surface, got {}",
            record.implementation
        );
    }
    // The degradation vector stays not-run but now carries the executed M4
    // fencing subset alongside the M1 static and M2 read-only subsets.
    let degradation = outcomes
        .iter()
        .find(|o| o.id == "STATE-STORE-DEGRADE-001")
        .expect("vector present");
    assert_eq!(degradation.result, "not-run");
    let assertions = degradation
        .partial_contract_assertions
        .as_ref()
        .expect("partial contract assertions recorded");
    let fencing = assertions
        .pointer("/m4_behavioral_fencing_subset")
        .expect("fencing subset recorded");
    assert!(
        fencing.get("probe_error").is_none(),
        "fencing probe failed: {fencing}"
    );
    for key in ["stale_epoch_write_rejected", "current_epoch_write_commits"] {
        assert_eq!(
            fencing.get(key).and_then(serde_json::Value::as_bool),
            Some(true),
            "fencing subset assertion {key} does not hold"
        );
    }
}

/// M2 behavioral executions run against the real kernel/store authority
/// path — the execution record must say so, and every one must pass.
#[test]
fn m2_behavioral_vectors_execute_against_the_real_kernel_path() {
    let root = repo_root();
    let vectors = enumerate_vectors(&root).expect("corpus enumerates");
    let outcomes =
        execute_all(&root, &vectors, ImplementationKind::Reference).expect("reference execution");
    for (id, mode) in BEHAVIORAL {
        let outcome = outcomes
            .iter()
            .find(|o| o.id == id)
            .unwrap_or_else(|| panic!("{id} missing from corpus"));
        assert_eq!(
            outcome.result,
            "pass",
            "{id} must pass behaviorally: {:?}",
            outcome.execution.as_ref().map(|e| &e.mismatches)
        );
        let record = outcome.execution.as_ref().expect("execution record");
        assert_eq!(format!("{:?}", record.mode), mode);
        assert!(
            record.implementation.contains("cognitive-kernel")
                && record.implementation.contains("SqliteAuthorityStore"),
            "{id} implementation label must name the real authority path, got {}",
            record.implementation
        );
        assert!(
            record
                .grounding
                .iter()
                .any(|g| g.contains("cognitive-kernel")),
            "{id} grounding must include the kernel gate"
        );
    }
}

/// STATE-STORE-DEGRADE-001 stays honestly not-run (disk-full and
/// dispatch/stop/revoke expectations are M4/M5), but the M2 read-only
/// degradation subset must have been executed for real and recorded.
#[test]
fn store_degradation_carries_the_executed_m2_subset() {
    let root = repo_root();
    let vectors = enumerate_vectors(&root).expect("corpus enumerates");
    let outcomes =
        execute_all(&root, &vectors, ImplementationKind::Reference).expect("reference execution");
    let degradation = outcomes
        .iter()
        .find(|o| o.id == "STATE-STORE-DEGRADE-001")
        .expect("vector present");
    assert_eq!(degradation.result, "not-run");
    let assertions = degradation
        .partial_contract_assertions
        .as_ref()
        .expect("partial contract assertions recorded");
    // M1 static side still present.
    assert_eq!(
        assertions
            .pointer("/static_contract/error_registered/fail_closed_description")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    // M2 behavioral read-only subset executed for real (probe must not
    // error and every executed assertion must hold).
    let subset = assertions
        .pointer("/m2_behavioral_read_only_subset")
        .expect("behavioral subset recorded");
    assert!(
        subset.get("probe_error").is_none(),
        "behavioral degradation probe failed: {subset}"
    );
    for key in [
        "governed_write_rejected_fail_closed",
        "read_only_inspection_available",
        "nothing_buffered_as_committed",
        "replay_digest_stable_across_degradation",
        "same_write_commits_after_recovery",
    ] {
        assert_eq!(
            subset.get(key).and_then(serde_json::Value::as_bool),
            Some(true),
            "degradation subset assertion {key} does not hold"
        );
    }
    assert_eq!(
        subset
            .pointer("/degraded_write_error/code")
            .and_then(serde_json::Value::as_str),
        Some("STATE_STORE_UNAVAILABLE")
    );
    assert_eq!(
        subset
            .get("committed_history_lost")
            .and_then(serde_json::Value::as_bool),
        Some(false)
    );
}

#[test]
fn plan_named_static_contract_vectors_are_executed() {
    let root = repo_root();
    let vectors = enumerate_vectors(&root).expect("corpus enumerates");
    let outcomes =
        execute_all(&root, &vectors, ImplementationKind::Reference).expect("reference execution");
    // DEVELOPMENT-PLAN M1 acceptance 4 named these vectors; closure-008 has
    // since been upgraded to behavioral execution (M2 batch).
    let by_id = |id: &str| {
        outcomes
            .iter()
            .find(|o| o.id == id)
            .expect("vector present")
    };
    assert_eq!(by_id("EFFECT-STATE-CLOSURE-008").result, "pass");
    assert_eq!(by_id("CTX-TRUST-004").result, "pass");
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
    // Every observably corrupted gate must be represented and flipped
    // (M2: gate-bypassing store writer; M3: governance anti-patterns;
    // M4: effect/recovery anti-patterns).
    assert_eq!(report.must_flip.len(), 27, "corrupted vector set drifted");
    assert_eq!(report.flipped_to_fail.len(), 27);
    assert!(report.corrupted_but_still_passing.is_empty());
    for id in [
        "GOBJ-LEGACY-METADATA-001",
        "GOBJ-LEGACY-STRONGREF-001",
        "STATE-CAS-002",
        "EFFECT-STATE-CLOSURE-008",
        "GW-REMOTE-COMPLETE-001",
        "PERF-REPORT-CONTRACT-001",
        "CTX-TRUST-004",
        "AKP-ENVELOPE-NO-SCHEMA-PIN-001",
        "AKP-ENVELOPE-AMBIGUOUS-PAYLOAD-002",
        "AKP-RESULT-ERROR-WITHOUT-MACHINE-CODE-003",
        "AKP-STREAM-FRAME-UNSEQUENCED-004",
        "SHELL-CONTROL-UNREASONED-CANCEL-001",
        "GOBJ-TENANT-LATERAL-001",
        "CAP-ATTEN-004",
        "CTX-REVOKE-CACHE-001",
        "CTX-RANK-AUTH-001",
        "CTX-REQ-007",
        "CTX-RENDER-001",
        "DISC-STAGNATION-004",
        "DISC-ADMISSION-002",
        "EFF-CRASH-001",
        "EFF-CRASH-002",
        "EFF-CRASH-003",
        "RECOVERY-CRASH-006",
        "EFF-UNK-003",
        "EFF-IDEM-CONFLICT-001",
        "AGENT-RECOVERY-003",
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
        "vector passes (static or behavioral) must not move any profile off planned"
    );
    assert_eq!(
        manifest
            .pointer("/cognitiveos_conformance/test_runs")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(0),
        "no conformance claim: test_runs stays empty until a profile-level behavioral evidence pipeline exists"
    );
}
