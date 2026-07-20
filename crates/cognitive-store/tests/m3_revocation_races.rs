//! F-007 behavioral evidence: capability revocation races against the
//! Effect lifecycle, executed through the real deterministic gate and the
//! SQLite authority store (M3; `docs/standards/authn-authz-capability.md`
//! section 3: long-running dispatch paths re-validate capability at
//! dispatch AND at commit, not only at Context resolution time).
//!
//! Race point 1: revocation lands AFTER context resolution/authorization
//! but BEFORE dispatch — the `AUTHORIZED -> EXECUTING` edge must not fire.
//! Race point 2: revocation lands AFTER dispatch but BEFORE commit — the
//! `VERIFIED -> COMMITTED` edge must not fire.
//!
//! The guard `capability_and_revocation_current` is attested exclusively
//! through `cognitive_kernel::authz::capability_and_revocation_current`
//! (the sanctioned deterministic derivation); a false attestation keeps
//! the guard out of the established set and the central gate rejects with
//! the registered code, leaving authoritative state unchanged.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cognitive_contracts::generated::common_defs::Digest;
use cognitive_contracts::generated::object_reference::{
    StrongReference, StrongReferenceKind, UuidV7,
};
use cognitive_domain::capability::{CapabilityConstraints, LeaseWindow};
use cognitive_domain::{
    EventId, LifecycleDomain, ObjectId, ReasonCode, StateName, UriRef, Version, WallTimestamp,
    table,
};
use cognitive_kernel::authz::{
    AccessRequest, ActorChainFacts, AuthzSnapshot, MembershipFacts, ObjectGovernance,
    PrincipalFacts, authorize, capability_and_revocation_current,
};
use cognitive_kernel::ports::{
    AuthorityStore, Clock, EventDraft, IdGenerator, ObjectAdmission, PortFailure, StoredObject,
};
use cognitive_kernel::{Causation, Reason, TablePin, TransitionCommand, TransitionEngine};
use cognitive_store::SqliteAuthorityStore;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};

struct FixedClock(WallTimestamp);

impl Clock for FixedClock {
    fn now(&self) -> Result<WallTimestamp, PortFailure> {
        Ok(self.0.clone())
    }
}

struct SeqIds(AtomicU64);

impl IdGenerator for SeqIds {
    fn next_uuid_v7(&self) -> Result<String, PortFailure> {
        let n = self.0.fetch_add(1, Ordering::SeqCst);
        Ok(format!("00000000-0000-7000-8000-{n:012x}"))
    }
}

fn ts(text: &str) -> WallTimestamp {
    WallTimestamp::parse(text).unwrap()
}

fn uri(text: &str) -> UriRef {
    UriRef::parse(text).unwrap()
}

fn state(name: &str) -> StateName {
    StateName::parse(name).unwrap()
}

fn strong_ref(n: u64) -> StrongReference {
    StrongReference {
        content_digest: Digest(format!(
            "sha256:{}",
            format!("{n:x}").repeat(64)[..64].to_owned()
        )),
        id: UuidV7(format!("00000000-0000-7000-a000-{n:012x}")),
        kind: StrongReferenceKind::Strong,
        object_version: 1,
    }
}

/// Seed one effect object at an arbitrary state through the port.
fn seed_effect(store: &SqliteAuthorityStore, id: &ObjectId, at: &str) {
    let admitted_at = ts("2026-07-20T11:00:00Z");
    let event_id = EventId::parse(&format!(
        "00000000-0000-7000-8000-9{}",
        &id.as_str()[id.as_str().len() - 11..]
    ))
    .unwrap();
    let event_value = json!({
        "event_id": event_id.as_str(),
        "event_type": "cognitiveos.object.admitted",
        "domain": "effect",
        "object_id": id.as_str(),
        "subject_ref": format!("effect://tenant-a/{}", id.as_str()),
        "after_state": at,
        "after_version": 1,
        "event_time": admitted_at.as_str(),
    });
    let canonical_json = String::from_utf8(
        cognitive_contracts::canonical::canonical_bytes_of_value(&event_value).unwrap(),
    )
    .unwrap();
    store
        .admit_object(&ObjectAdmission {
            object: StoredObject {
                object_id: id.clone(),
                domain: LifecycleDomain::Effect,
                state: state(at),
                version: Version::INITIAL,
                body: json!({"seeded": true}),
            },
            admitted_at,
            event: EventDraft {
                event_id,
                object_id: id.clone(),
                domain: LifecycleDomain::Effect,
                object_version: Version::INITIAL,
                event_type: "cognitiveos.object.admitted".to_owned(),
                canonical_json,
            },
            outbox: vec![],
        })
        .unwrap();
}

fn capability_link() -> CapabilityConstraints {
    CapabilityConstraints {
        subject: "principal://tenant-a/agent-1".to_owned(),
        audience: "service://tenant-a/payments".to_owned(),
        resource: "scope://tenant-a/payments".to_owned(),
        purpose: "refund_processing".to_owned(),
        actions: ["effect.dispatch".to_owned()].into(),
        parameter_bounds: Default::default(),
        lease: LeaseWindow {
            not_before: ts("2026-07-20T11:00:00Z"),
            expires: ts("2026-07-20T14:00:00Z"),
        },
        depth_remaining: 1,
        issued_epoch: 41,
    }
}

fn snapshot(revocation_epoch: i64) -> AuthzSnapshot {
    AuthzSnapshot {
        tenant_id: "tenant-a".to_owned(),
        principal: PrincipalFacts {
            principal_ref: uri("principal://tenant-a/agent-1"),
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
        capability_links: vec![capability_link()],
        capability_set_version: 7,
        explicit_denies: vec![],
        revocation_epoch,
        decided_at: ts("2026-07-20T12:00:00Z"),
    }
}

fn effect_target() -> ObjectGovernance {
    ObjectGovernance {
        object_ref: "effect://tenant-a/refund-17".to_owned(),
        tenant_id: Some("tenant-a".to_owned()),
        owner_ref: "principal://tenant-a/agent-1".to_owned(),
        resource_scope: "scope://tenant-a/payments/refunds".to_owned(),
        conversation_ref: None,
    }
}

/// Build the transition command for one effect edge, attesting the
/// non-capability guards as fixture facts and deriving
/// `capability_and_revocation_current` ONLY through the sanctioned
/// revalidation (never as a hardcoded true).
fn effect_command(
    object_id: &ObjectId,
    from: &str,
    to: &str,
    reason: &str,
    capability_current: bool,
) -> TransitionCommand {
    let loaded = table(LifecycleDomain::Effect).unwrap();
    let edge = loaded
        .find_edge(&state(from), &state(to), reason)
        .expect("edge is registered");
    let established: BTreeSet<String> = edge
        .guards
        .iter()
        .filter(|guard| guard.as_str() != "capability_and_revocation_current" || capability_current)
        .cloned()
        .collect();
    let evidence: BTreeMap<String, StrongReference> = edge
        .required_evidence
        .iter()
        .enumerate()
        .map(|(index, item)| (item.clone(), strong_ref(index as u64 + 1)))
        .collect();
    TransitionCommand {
        request_id: uri(&format!("request://f007/{from}-{to}")),
        domain: LifecycleDomain::Effect,
        object_id: object_id.clone(),
        subject_ref: uri("effect://tenant-a/refund-17"),
        from: state(from),
        to: state(to),
        expected_version: Version::INITIAL,
        reason: Reason {
            code: ReasonCode::parse(reason).unwrap(),
            detail: None,
        },
        causation: Causation {
            causation_id: uri("intent://tenant-a/intent-9"),
            correlation_id: uri("corr://tenant-a/chain-f007"),
        },
        actor_ref: uri("actor://tenant-a/agent-1"),
        authority_ref: uri("authority://tenant-a/effect-authority"),
        requested_at: ts("2026-07-20T12:00:30Z"),
        table_pin: TablePin::current(LifecycleDomain::Effect).unwrap(),
        established_guards: established,
        evidence,
        budget: None,
        outbox_destinations: vec![],
    }
}

/// F-007 race point 1: revocation between authorization and dispatch.
/// The AUTHORIZED effect must not move to EXECUTING under a stale grant.
#[test]
fn revocation_after_resolution_blocks_dispatch() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&dir.path().join("authority.db")).unwrap();
    let clock = FixedClock(ts("2026-07-20T12:01:00Z"));
    let ids = SeqIds(AtomicU64::new(1));
    let engine = TransitionEngine::new(&store, &clock, &ids);

    let object_id = ObjectId::parse("00000000-0000-7000-9000-00000000f007").unwrap();
    seed_effect(&store, &object_id, "AUTHORIZED");

    // Authorization happened under epoch 41 and produced a grant.
    let grant = authorize(
        &snapshot(41),
        &effect_target(),
        &AccessRequest {
            action: "effect.dispatch".to_owned(),
            purpose: "refund_processing".to_owned(),
        },
    )
    .unwrap();

    // Revocation advances the epoch to 42 BEFORE dispatch: the sanctioned
    // guard derivation now yields false.
    let now = ts("2026-07-20T12:01:00Z");
    let current_after_revocation = capability_and_revocation_current(&grant, 42, 7, &now);
    assert!(!current_after_revocation, "stale grant is not current");

    let cmd = effect_command(
        &object_id,
        "AUTHORIZED",
        "EXECUTING",
        "DISPATCHED",
        current_after_revocation,
    );
    let rejection = engine
        .commit_transition(&cmd)
        .expect_err("dispatch blocked");
    assert_eq!(rejection.registered().code, "STATE_CONFLICT");
    assert!(
        rejection
            .detail
            .contains("capability_and_revocation_current")
    );

    // Authoritative state unchanged: still AUTHORIZED at version 1, and no
    // dispatch event was appended.
    let stored = store
        .load_object(LifecycleDomain::Effect, &object_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "AUTHORIZED");
    assert_eq!(stored.version, Version::INITIAL);

    // Control: without the revocation (epoch still 41) the same command
    // dispatches — proving the revocation is what blocked it.
    let still_current = capability_and_revocation_current(&grant, 41, 7, &now);
    assert!(still_current);
    let cmd = effect_command(
        &object_id,
        "AUTHORIZED",
        "EXECUTING",
        "DISPATCHED",
        still_current,
    );
    engine.commit_transition(&cmd).unwrap();
    let stored = store
        .load_object(LifecycleDomain::Effect, &object_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "EXECUTING");
}

/// F-007 race point 2: revocation between dispatch and commit. The
/// VERIFIED effect must not move to COMMITTED under a stale grant.
#[test]
fn revocation_after_dispatch_blocks_commit() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteAuthorityStore::open(&dir.path().join("authority.db")).unwrap();
    let clock = FixedClock(ts("2026-07-20T12:05:00Z"));
    let ids = SeqIds(AtomicU64::new(1));
    let engine = TransitionEngine::new(&store, &clock, &ids);

    let object_id = ObjectId::parse("00000000-0000-7000-9000-0000000f0072").unwrap();
    // The effect already executed and verified; the commit decision is
    // still pending when the revocation lands.
    seed_effect(&store, &object_id, "VERIFIED");
    let grant = authorize(
        &snapshot(41),
        &effect_target(),
        &AccessRequest {
            action: "effect.dispatch".to_owned(),
            purpose: "refund_processing".to_owned(),
        },
    )
    .unwrap();

    let now = ts("2026-07-20T12:05:00Z");
    let stale = capability_and_revocation_current(&grant, 42, 7, &now);
    assert!(!stale);
    let cmd = effect_command(
        &object_id,
        "VERIFIED",
        "COMMITTED",
        "COMMIT_AUTHORIZED",
        stale,
    );
    let rejection = engine.commit_transition(&cmd).expect_err("commit blocked");
    assert_eq!(rejection.registered().code, "STATE_CONFLICT");
    assert!(
        rejection
            .detail
            .contains("capability_and_revocation_current")
    );
    let stored = store
        .load_object(LifecycleDomain::Effect, &object_id)
        .unwrap()
        .unwrap();
    assert_eq!(
        (stored.state.as_str(), stored.version),
        ("VERIFIED", Version::INITIAL),
        "no commit happened under the revoked capability"
    );

    // Control: current grant commits.
    let current = capability_and_revocation_current(&grant, 41, 7, &now);
    let cmd = effect_command(
        &object_id,
        "VERIFIED",
        "COMMITTED",
        "COMMIT_AUTHORIZED",
        current,
    );
    engine.commit_transition(&cmd).unwrap();
    let stored = store
        .load_object(LifecycleDomain::Effect, &object_id)
        .unwrap()
        .unwrap();
    assert_eq!(stored.state.as_str(), "COMMITTED");
}
