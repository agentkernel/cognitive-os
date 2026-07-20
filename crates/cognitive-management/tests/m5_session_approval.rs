//! M5 batch 2a behavior: REQ-MGMT-SESSION-LIFECYCLE-001 and F-011/IMP-05.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cognitive_domain::WallTimestamp;
use cognitive_management::{
    ApprovalGate, ApprovalPresentation, ManagementActionProposal, ManagementSessionArchive,
    RiskClass,
};
use serde_json::{Value, json};

fn ts(value: &str) -> WallTimestamp {
    WallTimestamp::parse(value).unwrap()
}

fn session_document() -> Value {
    json!({"schema_version":"cognitiveos.privileged-management-session/0.1","session_id":"pms_batch2a-session-01","object_version":1,"management_domain":"cognitiveos.management","session_authority":"authority://tenant-a/management","human_principal":"principal://tenant-a/alice","actor_chain_digest":format!("sha256:{}","ab".repeat(32)),"authentication_context_ref":"authn://tenant-a/webauthn","activity_context_ref":"activity://tenant-a/m5","scope":{"domains":["cognitiveos.management.execution"],"actions":["execution.stop"],"resources":["agent-execution://"]},"risk_ceiling":"R1","policy_version":1,"revocation_epoch":7,"issued_at":"2026-07-21T00:00:00Z","last_activity_at":"2026-07-21T00:00:00Z","idle_timeout_seconds":300,"absolute_expires_at":"2026-07-21T01:00:00Z","state":"active","session_digest":format!("sha256:{}","cd".repeat(32)),"authority_signature":"authority-signature-fixture"})
}

#[test]
fn session_issue_renew_expire_and_revoke_are_canonical_and_immediate() {
    let mut archive = ManagementSessionArchive::new();
    let issued = archive.issue(&session_document()).unwrap();
    assert_eq!(
        archive.canonical(&issued.session_id).unwrap(),
        issued.canonical_json().unwrap()
    );
    let renewed = archive
        .renew(
            &issued.session_id,
            &ts("2026-07-21T00:04:00Z"),
            &ts("2026-07-21T01:30:00Z"),
        )
        .unwrap();
    assert_eq!(renewed.object_version, 2);
    assert_eq!(
        archive
            .authorize_current(&issued.session_id, &ts("2026-07-21T00:10:00Z"))
            .unwrap_err()
            .code_str(),
        "MANAGEMENT_SESSION_EXPIRED"
    );
    assert_eq!(
        archive
            .revoke(&issued.session_id, &ts("2026-07-21T00:04:30Z"))
            .unwrap()
            .object_version,
        3
    );
    assert_eq!(
        archive
            .authorize_current(&issued.session_id, &ts("2026-07-21T00:04:31Z"))
            .unwrap_err()
            .code_str(),
        "MANAGEMENT_SESSION_REVOKED"
    );
}

fn proposal() -> ManagementActionProposal {
    ManagementActionProposal::new(
        "map_batch2a-proposal-01",
        "pms://batch2a/session",
        "execution.stop",
        vec!["agent-execution://tenant-a/42".to_owned()],
        json!({"reason":"operator"}),
        RiskClass::R1,
        "principal://tenant-a/worker-7",
        format!("sha256:{}", "11".repeat(32)),
        &ts("2026-07-21T00:00:00Z"),
        &ts("2026-07-21T00:05:00Z"),
    )
    .unwrap()
}

#[test]
fn r1_gate_covers_all_f011_negative_semantics_and_zero_dispatch() {
    let mut gate = ApprovalGate::new(2);
    let proposal = proposal();
    let request = gate
        .issue_request(
            &proposal,
            "principal://tenant-a/alice",
            "channel://os/management",
            &ts("2026-07-21T00:00:01Z"),
            &ts("2026-07-21T00:01:00Z"),
        )
        .unwrap();
    for p in [
        ApprovalPresentation::Missing,
        ApprovalPresentation::NaturalLanguage("approved".to_owned()),
    ] {
        assert_eq!(
            gate.authorize(&proposal, &request, p, &ts("2026-07-21T00:00:10Z"))
                .unwrap_err()
                .code_str(),
            "MANAGEMENT_INDEPENDENT_APPROVAL_REQUIRED"
        );
    }
    let self_decision = gate.test_decision(
        &request,
        "principal://tenant-a/worker-7",
        proposal.proposer_actor_chain_digest.clone(),
    );
    assert_eq!(
        gate.authorize(
            &proposal,
            &request,
            ApprovalPresentation::Structured(self_decision),
            &ts("2026-07-21T00:00:10Z")
        )
        .unwrap_err()
        .code_str(),
        "MANAGEMENT_SELF_AUTHORIZATION_DENIED"
    );
    let forged = gate.test_decision(
        &request,
        "principal://tenant-a/alice",
        proposal.proposer_actor_chain_digest.clone(),
    );
    assert_eq!(
        gate.authorize(
            &proposal,
            &request,
            ApprovalPresentation::Structured(forged),
            &ts("2026-07-21T00:00:10Z")
        )
        .unwrap_err()
        .code_str(),
        "MANAGEMENT_SELF_AUTHORIZATION_DENIED"
    );
    assert_eq!(gate.dispatches(), 0);

    let good = gate.test_decision(
        &request,
        "principal://tenant-a/alice",
        format!("sha256:{}", "22".repeat(32)),
    );
    gate.authorize(
        &proposal,
        &request,
        ApprovalPresentation::Structured(good.clone()),
        &ts("2026-07-21T00:00:30Z"),
    )
    .unwrap();
    assert_eq!(
        gate.authorize(
            &proposal,
            &request,
            ApprovalPresentation::Structured(good),
            &ts("2026-07-21T00:00:31Z")
        )
        .unwrap_err()
        .code_str(),
        "MANAGEMENT_INDEPENDENT_APPROVAL_REQUIRED"
    );
    let expired = gate
        .issue_request(
            &proposal,
            "principal://tenant-a/bob",
            "channel://os/management",
            &ts("2026-07-21T00:00:02Z"),
            &ts("2026-07-21T00:00:03Z"),
        )
        .unwrap();
    let expired_decision = gate.test_decision(
        &expired,
        "principal://tenant-a/bob",
        format!("sha256:{}", "33".repeat(32)),
    );
    assert_eq!(
        gate.authorize(
            &proposal,
            &expired,
            ApprovalPresentation::Structured(expired_decision),
            &ts("2026-07-21T00:00:04Z")
        )
        .unwrap_err()
        .code_str(),
        "MANAGEMENT_INDEPENDENT_APPROVAL_REQUIRED"
    );
    let mismatch_request = gate
        .issue_request(
            &proposal,
            "principal://tenant-a/carol",
            "channel://os/management",
            &ts("2026-07-21T00:00:05Z"),
            &ts("2026-07-21T00:02:00Z"),
        )
        .unwrap();
    let mut mismatch = gate.test_decision(
        &mismatch_request,
        "principal://tenant-a/carol",
        format!("sha256:{}", "44".repeat(32)),
    );
    mismatch.challenge_digest.0 = format!("sha256:{}", "ff".repeat(32));
    assert_eq!(
        gate.authorize(
            &proposal,
            &mismatch_request,
            ApprovalPresentation::Structured(mismatch),
            &ts("2026-07-21T00:00:10Z")
        )
        .unwrap_err()
        .code_str(),
        "MANAGEMENT_INDEPENDENT_APPROVAL_REQUIRED"
    );
    let first = gate
        .issue_request(
            &proposal,
            "principal://tenant-a/dana",
            "channel://os/management",
            &ts("2026-07-21T00:00:11Z"),
            &ts("2026-07-21T00:02:00Z"),
        )
        .unwrap();
    let second = gate
        .issue_request(
            &proposal,
            "principal://tenant-a/dana",
            "channel://os/management",
            &ts("2026-07-21T00:00:12Z"),
            &ts("2026-07-21T00:02:00Z"),
        )
        .unwrap();
    assert_eq!(first.request_id, second.request_id);
    assert_eq!(gate.dispatches(), 1);
}
