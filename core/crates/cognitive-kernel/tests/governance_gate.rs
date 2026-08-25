//! Behavioral tests of the six-step authorization gate (M3 acceptance
//! criteria 1, 2 and 7; `docs/standards/authn-authz-capability.md`
//! section 2; every test includes the negative it protects).
//!
//! Security discipline (`.cursor/rules/14-security-testing.mdc`): default
//! deny, explicit deny beats allow, denial before side effects, denial
//! responses leak no existence.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cognitive_domain::capability::{CapabilityConstraints, LeaseWindow, ParameterBound};
use cognitive_domain::{UriRef, WallTimestamp};
use cognitive_kernel::authz::{
    AccessRequest, ActorChainFacts, AuthzSnapshot, DenyRule, MembershipFacts, ObjectGovernance,
    PrincipalFacts, authorize, protected_read, revalidate_grant,
};
use std::collections::BTreeSet;

fn ts(text: &str) -> WallTimestamp {
    WallTimestamp::parse(text).unwrap()
}

fn uri(text: &str) -> UriRef {
    UriRef::parse(text).unwrap()
}

/// Capability link: `actions` on `resource` for `purpose`, epoch 41,
/// valid 12:00-13:00.
fn link(resource: &str, actions: &[&str], purpose: &str) -> CapabilityConstraints {
    CapabilityConstraints {
        subject: "principal://tenant-a/user-b".to_owned(),
        audience: "service://tenant-a/context".to_owned(),
        resource: resource.to_owned(),
        purpose: purpose.to_owned(),
        actions: actions.iter().map(|a| (*a).to_owned()).collect(),
        parameter_bounds: Default::default(),
        lease: LeaseWindow {
            not_before: ts("2026-07-20T12:00:00Z"),
            expires: ts("2026-07-20T13:00:00Z"),
        },
        depth_remaining: 1,
        issued_epoch: 41,
    }
}

fn snapshot(links: Vec<CapabilityConstraints>) -> AuthzSnapshot {
    AuthzSnapshot {
        tenant_id: "tenant-a".to_owned(),
        principal: PrincipalFacts {
            principal_ref: uri("principal://tenant-a/user-b"),
            authenticated: true,
            active: true,
            tenant_id: Some("tenant-a".to_owned()),
        },
        actor_chain: ActorChainFacts {
            chain_digest: format!("sha256:{}", "aa11".repeat(16)),
            resolved: true,
        },
        membership: Some(MembershipFacts {
            valid: true,
            roles: ["member".to_owned()].into(),
        }),
        capability_links: links,
        capability_set_version: 7,
        explicit_denies: vec![],
        revocation_epoch: 41,
        decided_at: ts("2026-07-20T12:30:00Z"),
    }
}

fn object(scope: &str, owner: &str) -> ObjectGovernance {
    ObjectGovernance {
        object_ref: format!("memory://tenant-a/{owner}/subject-profile?version=5"),
        tenant_id: Some("tenant-a".to_owned()),
        owner_ref: format!("principal://tenant-a/{owner}"),
        resource_scope: scope.to_owned(),
        conversation_ref: None,
    }
}

fn read_body() -> AccessRequest {
    AccessRequest {
        action: "read_body".to_owned(),
        purpose: "browse".to_owned(),
    }
}

/// M3 acceptance criterion 1 (vector `tenant-lateral-read-denial.json`):
/// same-tenant membership grants no lateral body read, the denial carries
/// the registered code, and it is byte-identical with not-found.
#[test]
fn criterion_1_same_tenant_lateral_read_denied_isomorphic_with_not_found() {
    // user-b holds a capability for their OWN personal scope and valid
    // membership — but the target lives in user-c's personal scope with no
    // share grant.
    let snapshot = snapshot(vec![link(
        "scope://tenant-a/user-b/personal",
        &["read_body"],
        "browse",
    )]);
    let target = object("scope://tenant-a/user-c/personal", "user-c");
    let body = serde_json::json!({"private": "user-c data"});

    let denied = protected_read(&snapshot, Some((&target, &body)), &read_body())
        .expect_err("membership alone must not grant lateral read");
    assert_eq!(denied.denial.code, "CONTEXT_AUTH_DENIED");
    assert_eq!(denied.denial.category, "auth");
    assert!(!denied.denial.retryable);

    // Existence isomorphism: not-found produces the byte-identical public
    // denial shape (no existence leak beyond policy).
    let not_found = protected_read::<serde_json::Value>(&snapshot, None, &read_body())
        .expect_err("absent object is denied identically");
    assert_eq!(denied.denial, not_found.denial);
    assert_eq!(
        serde_json::to_string(&denied.denial).unwrap(),
        serde_json::to_string(&not_found.denial).unwrap(),
        "serialized denial shapes are byte-identical"
    );
    // The fixed detail text never names the object or owner.
    assert!(!format!("{:?}", denied.denial).contains("user-c"));

    // Audit is produced server-side (REQ-SEC-001) without body content.
    assert_eq!(denied.audit.code, "CONTEXT_AUTH_DENIED");
    assert!(!format!("{:?}", denied.audit).contains("private"));

    // Positive control: the owner's own scope capability reads fine.
    let own_target = object("scope://tenant-a/user-b/personal", "user-b");
    let own_body = serde_json::json!({"mine": true});
    assert!(protected_read(&snapshot, Some((&own_target, &own_body)), &read_body()).is_ok());
}

/// M3 acceptance criterion 2: a governance/management capability does not
/// read task bodies — managing is not reading (management != content).
#[test]
fn criterion_2_admin_governance_capability_does_not_read_body() {
    // The admin's capability covers the whole tenant scope but only the
    // govern action.
    let admin = snapshot(vec![link(
        "scope://tenant-a",
        &["govern"],
        "administration",
    )]);
    let target = object("scope://tenant-a/user-c/personal", "user-c");
    let body = serde_json::json!({"secret": "task body"});

    // Governance action is allowed on the same scope.
    let govern = AccessRequest {
        action: "govern".to_owned(),
        purpose: "administration".to_owned(),
    };
    assert!(authorize(&admin, &target, &govern).is_ok());

    // Body read with the same capability is denied with the registered
    // code and no body disclosure.
    let read = AccessRequest {
        action: "read_body".to_owned(),
        purpose: "administration".to_owned(),
    };
    let denied = protected_read(&admin, Some((&target, &body)), &read)
        .expect_err("management right must not read bodies");
    assert_eq!(denied.denial.code, "CONTEXT_AUTH_DENIED");
    assert!(!format!("{:?}", denied.denial).contains("secret"));
}

/// Default deny and explicit deny (decision order steps 3-4): no grant
/// means deny; an explicit deny beats a valid allow.
#[test]
fn default_deny_and_explicit_deny_beat_allow() {
    // No capability at all: default deny.
    let no_grants = snapshot(vec![]);
    let target = object("scope://tenant-a/user-b/personal", "user-b");
    let denied = authorize(&no_grants, &target, &read_body()).expect_err("default deny");
    assert_eq!(denied.denial.code, "CONTEXT_AUTH_DENIED");

    // Valid allow + explicit deny on the scope: deny wins.
    let mut with_deny = snapshot(vec![link(
        "scope://tenant-a/user-b/personal",
        &["read_body"],
        "browse",
    )]);
    with_deny.explicit_denies.push(DenyRule {
        resource_prefix: "scope://tenant-a/user-b".to_owned(),
        actions: BTreeSet::new(), // all actions
    });
    let denied = authorize(&with_deny, &target, &read_body()).expect_err("deny beats allow");
    assert_eq!(denied.denial.code, "CONTEXT_AUTH_DENIED");
}

/// Decision order step 5: outside the lease window the registered code is
/// AUTH_CAPABILITY_EXPIRED and an expired lease is never extended.
#[test]
fn lease_expiry_fails_with_the_registered_code() {
    let mut expired = snapshot(vec![link(
        "scope://tenant-a/user-b/personal",
        &["read_body"],
        "browse",
    )]);
    expired.decided_at = ts("2026-07-20T13:00:00Z"); // == expires (half-open)
    let target = object("scope://tenant-a/user-b/personal", "user-b");
    let denied = authorize(&expired, &target, &read_body()).expect_err("expired");
    assert_eq!(denied.denial.code, "AUTH_CAPABILITY_EXPIRED");
    assert!(denied.denial.retryable, "expiry is retryable after refresh");

    // Not-yet-valid is equally outside the window.
    let mut early = snapshot(vec![link(
        "scope://tenant-a/user-b/personal",
        &["read_body"],
        "browse",
    )]);
    early.decided_at = ts("2026-07-20T11:59:59Z");
    let denied = authorize(&early, &target, &read_body()).expect_err("not yet valid");
    assert_eq!(denied.denial.code, "AUTH_CAPABILITY_EXPIRED");
}

/// Step 1 negatives: unauthenticated, suspended, or unresolved-chain
/// requests are denied before any other consideration.
#[test]
fn authentication_and_chain_resolution_fail_closed() {
    let target = object("scope://tenant-a/user-b/personal", "user-b");
    let valid_link = || link("scope://tenant-a/user-b/personal", &["read_body"], "browse");

    let mut unauthenticated = snapshot(vec![valid_link()]);
    unauthenticated.principal.authenticated = false;
    assert_eq!(
        authorize(&unauthenticated, &target, &read_body())
            .unwrap_err()
            .denial
            .code,
        "CONTEXT_AUTH_DENIED"
    );

    let mut suspended = snapshot(vec![valid_link()]);
    suspended.principal.active = false;
    assert!(authorize(&suspended, &target, &read_body()).is_err());

    let mut unresolved = snapshot(vec![valid_link()]);
    unresolved.actor_chain.resolved = false;
    assert!(authorize(&unresolved, &target, &read_body()).is_err());

    let mut no_membership = snapshot(vec![valid_link()]);
    no_membership.membership = None;
    assert!(authorize(&no_membership, &target, &read_body()).is_err());
}

/// Step 2 negative (REQ-SEC-001): a cross-tenant reference fails closed
/// with the same public shape as any other denial.
#[test]
fn cross_tenant_reference_fails_closed_without_existence_leak() {
    let snapshot = snapshot(vec![link("scope://tenant-a", &["read_body"], "browse")]);
    let mut foreign = object("scope://tenant-b/user-z/personal", "user-z");
    foreign.tenant_id = Some("tenant-b".to_owned());
    foreign.object_ref = "memory://tenant-b/user-z/profile?version=1".to_owned();

    let denied = authorize(&snapshot, &foreign, &read_body()).expect_err("cross-tenant");
    assert_eq!(denied.denial.code, "CONTEXT_AUTH_DENIED");
    assert!(!format!("{:?}", denied.denial).contains("tenant-b"));
}

/// M3 acceptance criterion 7 (F-007 arithmetic at the gate): the effective
/// right of a chain is the intersection — an action only one link allows
/// is denied, and adding links can only narrow, never widen.
#[test]
fn criterion_7_capability_intersection_only_narrows_at_the_gate() {
    let broad = link(
        "scope://tenant-a/user-b/personal",
        &["read_body", "annotate"],
        "browse",
    );
    let narrow = link("scope://tenant-a/user-b/personal", &["read_body"], "browse");
    let target = object("scope://tenant-a/user-b/personal", "user-b");

    // Single broad link authorizes annotate.
    let annotate = AccessRequest {
        action: "annotate".to_owned(),
        purpose: "browse".to_owned(),
    };
    assert!(authorize(&snapshot(vec![broad.clone()]), &target, &annotate).is_ok());

    // Adding the narrow link REMOVES annotate (intersection, never union).
    let chained = snapshot(vec![broad.clone(), narrow.clone()]);
    let denied = authorize(&chained, &target, &annotate).expect_err("intersection narrows");
    assert_eq!(denied.denial.code, "CONTEXT_AUTH_DENIED");
    // read_body survives in both links and stays authorized.
    assert!(authorize(&chained, &target, &read_body()).is_ok());

    // A parameter bound tightens through the chain as well: the effective
    // grant carries the tightest bound.
    let mut bounded = broad.clone();
    bounded
        .parameter_bounds
        .insert("max_items".to_owned(), ParameterBound::NumericMax(10));
    let mut tighter = narrow.clone();
    tighter
        .parameter_bounds
        .insert("max_items".to_owned(), ParameterBound::NumericMax(3));
    let grant = authorize(&snapshot(vec![bounded, tighter]), &target, &read_body()).unwrap();
    assert_eq!(
        grant.effective.parameter_bounds.get("max_items"),
        Some(&ParameterBound::NumericMax(3))
    );
}

/// Revocation currency (REQ-CAP-005): a chain whose stalest link predates
/// the current revocation epoch is not current material, and a grant
/// decided under an older epoch fails dispatch-time revalidation (F-007
/// race point 1: after resolution, before dispatch).
#[test]
fn revoked_epoch_denies_decision_and_dispatch_revalidation() {
    let target = object("scope://tenant-a/user-b/personal", "user-b");
    let valid = snapshot(vec![link(
        "scope://tenant-a/user-b/personal",
        &["read_body"],
        "browse",
    )]);
    let grant = authorize(&valid, &target, &read_body()).unwrap();

    // Revocation advances the epoch: a NEW decision from the stale chain
    // is denied outright.
    let mut after_revocation = valid.clone();
    after_revocation.revocation_epoch = 42; // links still carry epoch 41
    let denied = authorize(&after_revocation, &target, &read_body())
        .expect_err("stale-epoch chain is not current material");
    assert_eq!(denied.denial.code, "CONTEXT_AUTH_DENIED");

    // And the PREVIOUSLY issued grant fails revalidation at dispatch time.
    let now = ts("2026-07-20T12:31:00Z");
    assert!(
        revalidate_grant(&grant, 41, 7, &now).is_ok(),
        "same epoch revalidates"
    );
    let stale = revalidate_grant(&grant, 42, 7, &now).expect_err("epoch advanced");
    assert_eq!(stale.code, "CONTEXT_AUTH_DENIED");
    // Capability-set version changes invalidate identically (REQ-CAP-005).
    let swapped = revalidate_grant(&grant, 41, 8, &now).expect_err("cap set changed");
    assert_eq!(swapped.code, "CONTEXT_AUTH_DENIED");
    // Lease runs out between decision and dispatch: expired, not extended.
    let too_late = revalidate_grant(&grant, 41, 7, &ts("2026-07-20T13:00:00Z"))
        .expect_err("lease elapsed before dispatch");
    assert_eq!(too_late.code, "AUTH_CAPABILITY_EXPIRED");
}

/// Purpose binding (decision order step 6): the same capability under a
/// different purpose is denied.
#[test]
fn purpose_mismatch_is_denied() {
    let snapshot = snapshot(vec![link(
        "scope://tenant-a/user-b/personal",
        &["read_body"],
        "browse",
    )]);
    let target = object("scope://tenant-a/user-b/personal", "user-b");
    let wrong_purpose = AccessRequest {
        action: "read_body".to_owned(),
        purpose: "exfiltrate".to_owned(),
    };
    let denied = authorize(&snapshot, &target, &wrong_purpose).expect_err("purpose bound");
    assert_eq!(denied.denial.code, "CONTEXT_AUTH_DENIED");
}
