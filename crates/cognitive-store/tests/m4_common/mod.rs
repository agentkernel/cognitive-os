//! Shared harness for the M4 effect/recovery/tracer suites: deterministic
//! clock and ids, M3 governance snapshot/grant builders, effect/task/loop
//! admission helpers, protocol driver construction.

#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use cognitive_contracts::generated::common_defs::Digest;
use cognitive_contracts::generated::object_reference::{StrongReference, StrongReferenceKind};
use cognitive_domain::capability::{CapabilityConstraints, LeaseWindow};
use cognitive_domain::{
    LifecycleDomain, ObjectId, ReasonCode, StateName, UriRef, Version, WallTimestamp, table,
};
use cognitive_kernel::authz::{
    AccessRequest, ActorChainFacts, AuthorizationGrant, AuthzSnapshot, MembershipFacts,
    ObjectGovernance, PrincipalFacts, authorize,
};
use cognitive_kernel::effects::{
    EffectClass, EffectProtocol, GovernanceCurrency, IntentCommand, OperationDescriptor,
    WriterLease,
};
use cognitive_kernel::executor::ExecutorCapabilities;
use cognitive_kernel::ports::{Clock, IdGenerator, PortFailure};
use cognitive_kernel::{
    AdmitCommand, Causation, Reason, TablePin, TransitionCommand, TransitionEngine,
};
use cognitive_store::SqliteAuthorityStore;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct FixedClock(pub WallTimestamp);

impl FixedClock {
    pub fn new() -> Self {
        Self(WallTimestamp::parse("2026-07-20T12:00:00Z").unwrap())
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Result<WallTimestamp, PortFailure> {
        Ok(self.0.clone())
    }
}

pub struct SeqIds(pub AtomicU64);

impl SeqIds {
    pub fn new() -> Self {
        Self(AtomicU64::new(1))
    }

    pub fn from(start: u64) -> Self {
        Self(AtomicU64::new(start))
    }
}

impl IdGenerator for SeqIds {
    fn next_uuid_v7(&self) -> Result<String, PortFailure> {
        let n = self.0.fetch_add(1, Ordering::SeqCst);
        Ok(format!("00000000-0000-7000-8000-{n:012x}"))
    }
}

pub fn ts(text: &str) -> WallTimestamp {
    WallTimestamp::parse(text).unwrap()
}

pub fn uri(text: &str) -> UriRef {
    UriRef::parse(text).unwrap()
}

pub fn state(name: &str) -> StateName {
    StateName::parse(name).unwrap()
}

pub fn oid(n: u64) -> ObjectId {
    ObjectId::parse(&format!("00000000-0000-7000-9000-{n:012x}")).unwrap()
}

pub fn capability_link(actions: &[&str]) -> CapabilityConstraints {
    CapabilityConstraints {
        subject: "principal://tenant-a/agent-1".to_owned(),
        audience: "service://tenant-a/payments".to_owned(),
        resource: "scope://tenant-a/payments".to_owned(),
        purpose: "refund_processing".to_owned(),
        actions: actions.iter().map(|a| (*a).to_owned()).collect(),
        parameter_bounds: Default::default(),
        lease: LeaseWindow {
            not_before: ts("2026-07-20T11:00:00Z"),
            expires: ts("2026-07-20T14:00:00Z"),
        },
        depth_remaining: 1,
        issued_epoch: 41,
    }
}

pub fn snapshot(actions: &[&str]) -> AuthzSnapshot {
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
        capability_links: vec![capability_link(actions)],
        capability_set_version: 7,
        explicit_denies: vec![],
        revocation_epoch: 41,
        decided_at: ts("2026-07-20T12:00:00Z"),
    }
}

pub fn payments_target() -> ObjectGovernance {
    ObjectGovernance {
        object_ref: "effect://tenant-a/refund-17".to_owned(),
        tenant_id: Some("tenant-a".to_owned()),
        owner_ref: "principal://tenant-a/agent-1".to_owned(),
        resource_scope: "scope://tenant-a/payments/refunds".to_owned(),
        conversation_ref: None,
    }
}

/// A real M3 grant for `action` (epoch 41, cap set version 7).
pub fn grant_for(action: &str) -> AuthorizationGrant {
    authorize(
        &snapshot(&[action]),
        &payments_target(),
        &AccessRequest {
            action: action.to_owned(),
            purpose: "refund_processing".to_owned(),
        },
    )
    .unwrap()
}

pub fn currency() -> GovernanceCurrency {
    GovernanceCurrency {
        revocation_epoch: 41,
        capability_set_version: 7,
    }
}

pub fn descriptor(queryable: bool, idempotent: bool) -> OperationDescriptor {
    OperationDescriptor {
        operation_id: "op://tenant-a/payments/refund".to_owned(),
        action: "payments.refund".to_owned(),
        effect_class: EffectClass::GovernedExternal,
        executor: "executor://tenant-a/payments".to_owned(),
        capabilities: ExecutorCapabilities {
            queryable,
            idempotent,
        },
        descriptor_version: 1,
    }
}

pub fn protocol<'a>(
    store: &'a SqliteAuthorityStore,
    clock: &'a FixedClock,
    ids: &'a SeqIds,
) -> EffectProtocol<'a, SqliteAuthorityStore, FixedClock, SeqIds> {
    EffectProtocol::new(
        store,
        clock,
        ids,
        uri("actor://tenant-a/agent-1"),
        uri("authority://tenant-a/effect-authority"),
        uri("corr://tenant-a/m4-chain"),
    )
}

/// Admit a governed object at its table's initial state through the engine.
pub fn admit(
    store: &SqliteAuthorityStore,
    clock: &FixedClock,
    ids: &SeqIds,
    object_id: &ObjectId,
    domain: LifecycleDomain,
    lease_epoch: Option<i64>,
) {
    let engine = TransitionEngine::new(store, clock, ids);
    engine
        .admit_object(&AdmitCommand {
            object_id: object_id.clone(),
            domain,
            subject_ref: uri(&format!("{}://tenant-a/{}", domain.as_str(), object_id)),
            body: json!({"m4": true}),
            actor_ref: uri("actor://tenant-a/agent-1"),
            authority_ref: uri("authority://tenant-a/state-authority"),
            correlation_id: uri("corr://tenant-a/m4-chain"),
            outbox_destinations: vec![],
            fencing_epoch: lease_epoch,
        })
        .unwrap();
}

pub fn evidence_ref(n: u64) -> StrongReference {
    StrongReference {
        content_digest: Digest(format!(
            "sha256:{}",
            format!("{n:x}").repeat(64)[..64].to_owned()
        )),
        id: cognitive_contracts::generated::object_reference::UuidV7(format!(
            "00000000-0000-7000-a000-{n:012x}"
        )),
        kind: StrongReferenceKind::Strong,
        object_version: 1,
    }
}

/// Drive one legal transition with the edge's guards attested as fixture
/// facts (used for task/loop/verification machines in the tracer bullet;
/// effect transitions go through the protocol driver's sanctioned
/// derivations instead).
#[allow(clippy::too_many_arguments)]
pub fn drive(
    store: &SqliteAuthorityStore,
    clock: &FixedClock,
    ids: &SeqIds,
    domain: LifecycleDomain,
    object_id: &ObjectId,
    from: &str,
    to: &str,
    reason: &str,
    expected_version: Version,
    lease_epoch: Option<i64>,
) -> Version {
    let loaded = table(domain).unwrap();
    let edge = loaded.find_edge(&state(from), &state(to), reason).unwrap();
    let established: BTreeSet<String> = edge.guards.iter().cloned().collect();
    let evidence: BTreeMap<String, StrongReference> = edge
        .required_evidence
        .iter()
        .enumerate()
        .map(|(index, item)| (item.clone(), evidence_ref(index as u64 + 1)))
        .collect();
    let engine = TransitionEngine::new(store, clock, ids);
    let committed = engine
        .commit_transition(&TransitionCommand {
            request_id: uri(&format!("request://m4/{}/{from}-{to}", object_id.as_str())),
            domain,
            object_id: object_id.clone(),
            subject_ref: uri(&format!("{}://tenant-a/{}", domain.as_str(), object_id)),
            from: state(from),
            to: state(to),
            expected_version,
            reason: Reason {
                code: ReasonCode::parse(reason).unwrap(),
                detail: None,
            },
            causation: Causation {
                causation_id: uri("corr://tenant-a/m4-chain"),
                correlation_id: uri("corr://tenant-a/m4-chain"),
            },
            actor_ref: uri("actor://tenant-a/agent-1"),
            authority_ref: uri("authority://tenant-a/state-authority"),
            requested_at: ts("2026-07-20T11:59:00Z"),
            table_pin: TablePin::current(domain).unwrap(),
            established_guards: established,
            evidence,
            budget: None,
            outbox_destinations: vec![],
            fencing_epoch: lease_epoch,
        })
        .unwrap();
    committed.after_version
}

/// Standard intent command for the shared refund scenario.
pub fn intent_command(
    intent_n: u64,
    effect_id: &ObjectId,
    key: &str,
    amount: i64,
    desc: OperationDescriptor,
) -> IntentCommand {
    IntentCommand {
        intent_id: oid(intent_n),
        effect_object_id: effect_id.clone(),
        descriptor: desc,
        target: "https://payments.example/api/refunds".to_owned(),
        parameters: json!({"amount_minor": amount, "currency": "EUR"}),
        idempotency_key: key.to_owned(),
        expected_state_version: Version::INITIAL,
        grant_epoch: 41,
        capability_set_version: 7,
        actor_ref: uri("actor://tenant-a/agent-1"),
        authority_ref: uri("authority://tenant-a/effect-authority"),
        correlation_id: uri("corr://tenant-a/m4-chain"),
    }
}

pub fn lease(epoch: i64) -> WriterLease {
    WriterLease { epoch }
}
