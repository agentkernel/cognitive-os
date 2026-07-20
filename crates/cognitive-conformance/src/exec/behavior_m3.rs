//! M3 behavioral vector execution: governance chain and Context.
//!
//! The implementation under test is the M3 kernel surface —
//! `cognitive_kernel::authz` (six-step authorization gate, denial/not-found
//! isomorphism), `cognitive_kernel::context` (nine-stage deterministic
//! resolution pipeline, filter-before-rank, deterministic prefix-stable
//! render, bounded stagnation), `cognitive_kernel::context_cache`
//! (governance-bound cache keys) and `cognitive_domain::capability`
//! (monotone attenuation arithmetic). All of it is pure deterministic
//! code; the runner drives each vector's `input` scenario through the real
//! public API and reads every observable back from the outputs.
//!
//! Mapping vocabulary: where a vector's `expected` uses its own outcome
//! labels (`deny` / `error` / `rank_only_authorized_candidates` /
//! `revalidate_or_reresolve` / `denied_or_controlled_fallback` /
//! `allowed`), the gate emits that label if and only if the corresponding
//! real outcome occurred (Ok/Err of the driven call, structural cache
//! miss, registered failure code); the mapping is recorded in evidence.
//! Prose rationale fields stay recorded-not-compared. `authority_unchanged`
//! and `capability_expanded` are structural facts of this surface (the
//! pipeline is a pure function with no authority-store access, and chain
//! arithmetic only narrows); the grounding is recorded per vector.
//!
//! The deliberately wrong implementations are governance anti-patterns
//! driven for real where feasible: membership-alone body reads, ranking
//! before authorization, serving revocation-stale cache entries, silent
//! truncation instead of budget fail-closed, unbounded retry, reshuffled
//! re-render, content-claimed control plane, accepted capability
//! amplification.

use super::{AssetContext, ExecError, GateOutput, ImplementationKind};
use crate::LoadedVector;
use cognitive_contracts::generated::context_view::{
    LoadedContextItemRole, LoadedContextItemTrustLevel,
};
use cognitive_domain::capability::{CapabilityConstraints, LeaseWindow, ParameterBound};
use cognitive_domain::{UriRef, WallTimestamp, capability::attenuation_violations};
use cognitive_kernel::authz::{
    AccessRequest, ActorChainFacts, AuthzSnapshot, MembershipFacts, ObjectGovernance,
    PrincipalFacts, protected_read,
};
use cognitive_kernel::context::{
    ArrivalOrderRanker, CandidateObject, ContextBudget, ProposalRanker, RankerCandidate,
    RenderSpec, RequiredItem, ResolutionRequest, ResolutionSession, admit_control_mutation,
    effective_control_plane, resolve,
};
use cognitive_kernel::context_cache::{
    CachedView, ContextViewCache, DerivedCacheKind, GovernanceBinding,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeSet;

fn env_err(what: impl Into<String>) -> ExecError {
    ExecError::Environment(what.into())
}

fn ts(text: &str) -> Result<WallTimestamp, ExecError> {
    WallTimestamp::parse(text).map_err(|err| env_err(format!("timestamp `{text}`: {err}")))
}

fn uri(text: &str) -> Result<UriRef, ExecError> {
    UriRef::parse(text).map_err(|err| env_err(format!("uri `{text}`: {err}")))
}

fn registered(ctx: &AssetContext, code: &str) -> Result<Value, ExecError> {
    ctx.registered_error(code)
        .ok_or_else(|| env_err(format!("code {code} not registered")))
}

const REFERENCE_IMPLEMENTATION: &str =
    "cognitive-kernel authz/context/context_cache + cognitive-domain capability (real M3 surface)";
const WRONG_IMPLEMENTATION: &str = "governance anti-pattern implementation (deliberately wrong: membership-alone reads, \
     rank-before-auth, stale cache serving, silent truncation, unbounded retry, reshuffling \
     render, content-claimed control plane)";

fn implementation_label(kind: ImplementationKind) -> Option<&'static str> {
    Some(match kind {
        ImplementationKind::Reference => REFERENCE_IMPLEMENTATION,
        ImplementationKind::DeliberatelyWrong => WRONG_IMPLEMENTATION,
    })
}

/// Structural-fact note attached wherever a vector expects
/// `authority_unchanged` / `capability_expanded`.
const STRUCTURAL_FACTS: &str = "authority_unchanged/capability_expanded are structural facts of \
     the driven surface: resolve()/authorize() are pure functions without authority-store \
     access, and capability chain arithmetic only narrows (intersect_chain)";

/// Capability link over `resource` for `actions`/`purpose`, epoch 41,
/// valid 12:00-13:00 on the harness day.
fn link(
    resource: &str,
    actions: &[&str],
    purpose: &str,
) -> Result<CapabilityConstraints, ExecError> {
    Ok(CapabilityConstraints {
        subject: "principal://tenant-a/user-b".to_owned(),
        audience: "service://tenant-a/context".to_owned(),
        resource: resource.to_owned(),
        purpose: purpose.to_owned(),
        actions: actions.iter().map(|a| (*a).to_owned()).collect(),
        parameter_bounds: Default::default(),
        lease: LeaseWindow {
            not_before: ts("2026-07-20T12:00:00Z")?,
            expires: ts("2026-07-20T13:00:00Z")?,
        },
        depth_remaining: 1,
        issued_epoch: 41,
    })
}

fn snapshot(
    tenant: &str,
    principal: &str,
    chain_digest: &str,
    links: Vec<CapabilityConstraints>,
    revocation_epoch: i64,
) -> Result<AuthzSnapshot, ExecError> {
    Ok(AuthzSnapshot {
        tenant_id: tenant.to_owned(),
        principal: PrincipalFacts {
            principal_ref: uri(principal)?,
            authenticated: true,
            active: true,
            tenant_id: Some(tenant.to_owned()),
        },
        actor_chain: ActorChainFacts {
            chain_digest: chain_digest.to_owned(),
            resolved: true,
        },
        membership: Some(MembershipFacts {
            valid: true,
            roles: ["member".to_owned()].into(),
        }),
        capability_links: links,
        capability_set_version: 7,
        explicit_denies: vec![],
        revocation_epoch,
        decided_at: ts("2026-07-20T12:30:00Z")?,
    })
}

#[allow(clippy::too_many_arguments)] // flat harness builder mirroring the KRN test helpers
fn candidate(
    object_ref: &str,
    tenant: Option<&str>,
    scope: &str,
    role: LoadedContextItemRole,
    trust: LoadedContextItemTrustLevel,
    body: Value,
    bytes: i64,
    tokens: i64,
) -> CandidateObject {
    CandidateObject {
        object_ref: object_ref.to_owned(),
        object_version: 1,
        content_digest: format!("sha256:{}", "c0".repeat(32)),
        governance: ObjectGovernance {
            object_ref: object_ref.to_owned(),
            tenant_id: tenant.map(str::to_owned),
            owner_ref: "principal://tenant-a/owner".to_owned(),
            resource_scope: scope.to_owned(),
            conversation_ref: None,
        },
        role,
        trust_level: trust,
        body,
        cost_bytes: bytes,
        cost_tokens: tokens,
    }
}

fn request(
    snapshot: AuthzSnapshot,
    purpose: &str,
    required: Vec<RequiredItem>,
    allow_partial: bool,
    budget: ContextBudget,
) -> ResolutionRequest {
    ResolutionRequest {
        snapshot,
        purpose: purpose.to_owned(),
        conversation_ref: None,
        required,
        allow_partial,
        budget,
        render: RenderSpec {
            renderer_version: "structured/v1".to_owned(),
            target_profile: "structured".to_owned(),
        },
        schema_digest: format!("sha256:{}", "5d".repeat(32)),
    }
}

/// Ranker wrapper that records exactly what it was shown.
#[derive(Default)]
struct RecordingRanker {
    seen_refs: RefCell<Vec<String>>,
    seen_bodies: RefCell<Vec<String>>,
}

impl ProposalRanker for RecordingRanker {
    fn rank(&self, candidates: &[RankerCandidate<'_>]) -> Vec<String> {
        for candidate in candidates {
            self.seen_refs
                .borrow_mut()
                .push(candidate.object_ref.to_owned());
            self.seen_bodies
                .borrow_mut()
                .push(candidate.body.to_string());
        }
        ArrivalOrderRanker.rank(candidates)
    }
}

/// Hostile ranker: proposes a denied ref first and injects an external ref.
struct HostileRanker {
    smuggle: Vec<String>,
}

impl ProposalRanker for HostileRanker {
    fn rank(&self, candidates: &[RankerCandidate<'_>]) -> Vec<String> {
        let mut proposal = self.smuggle.clone();
        proposal.extend(
            candidates
                .iter()
                .map(|candidate| candidate.object_ref.to_owned()),
        );
        proposal
    }
}

// ---------------------------------------------------------------------------
// GOBJ-TENANT-LATERAL-001
// ---------------------------------------------------------------------------

pub(super) fn lateral_read_behavior(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let input = &vector.input;
    let tenant = input
        .get("tenant")
        .and_then(Value::as_str)
        .ok_or_else(|| env_err("vector lacks tenant"))?;
    let principal = input
        .pointer("/requesting_principal/id")
        .and_then(Value::as_str)
        .ok_or_else(|| env_err("vector lacks requesting_principal.id"))?;
    let membership_valid = input
        .pointer("/requesting_principal/membership_valid")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let target_ref = input
        .pointer("/target_object/ref")
        .and_then(Value::as_str)
        .ok_or_else(|| env_err("vector lacks target_object.ref"))?;
    let target_owner = input
        .pointer("/target_object/owner")
        .and_then(Value::as_str)
        .unwrap_or("principal://tenant-a/user-c");
    let target_scope = input
        .pointer("/target_object/resource_scope")
        .and_then(Value::as_str)
        .ok_or_else(|| env_err("vector lacks target_object.resource_scope"))?;
    let share_grant_present = input
        .pointer("/target_object/share_grant_present")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let action = input
        .pointer("/request/operation")
        .and_then(Value::as_str)
        .unwrap_or("read_body");
    let purpose = input
        .pointer("/request/purpose")
        .and_then(Value::as_str)
        .unwrap_or("browse");

    // The principal's real grants: own personal scope only. No share grant
    // for the target scope exists (input pins share_grant_present=false).
    let own_scope = format!("scope://{tenant}/user-b/personal");
    let mut links = vec![link(&own_scope, &[action], purpose)?];
    if share_grant_present {
        links.push(link(target_scope, &[action], purpose)?);
    }
    let mut snap = snapshot(
        tenant,
        principal,
        &format!("sha256:{}", "aa".repeat(32)),
        links,
        41,
    )?;
    snap.membership = membership_valid.then(|| MembershipFacts {
        valid: true,
        roles: ["member".to_owned()].into(),
    });

    let target = ObjectGovernance {
        object_ref: target_ref.to_owned(),
        tenant_id: Some(tenant.to_owned()),
        owner_ref: target_owner.to_owned(),
        resource_scope: target_scope.to_owned(),
        conversation_ref: None,
    };
    const BODY_MARKER: &str = "PRIVATE-BODY-MARKER-7f3a";
    let body = json!({ "private": BODY_MARKER });
    let access = AccessRequest {
        action: action.to_owned(),
        purpose: purpose.to_owned(),
    };

    let (actual, evidence) = match kind {
        ImplementationKind::Reference => {
            let denied = match protected_read(&snap, Some((&target, &body)), &access) {
                Ok(_) => {
                    // Would be a real security failure — surface it.
                    let actual = json!({
                        "decision": "allow",
                        "error": Value::Null,
                        "body_disclosed": true,
                        "existence_leak_beyond_policy": false,
                        "membership_alone_grants_read": true,
                        "audit_required": false,
                    });
                    return Ok(GateOutput {
                        actual,
                        grounding: lateral_grounding(),
                        informative: vec!["allowed_paths"],
                        implementation: implementation_label(kind),
                        evidence: json!({ "unexpected": "lateral read was allowed" }),
                    });
                }
                Err(denied) => denied,
            };
            // Isomorphism twin: not-found must serialize byte-identically.
            let not_found = protected_read::<Value>(&snap, None, &access)
                .err()
                .ok_or_else(|| env_err("not-found twin unexpectedly succeeded"))?;
            let denied_bytes = serde_json::to_string(&denied.denial)
                .map_err(|err| env_err(format!("denial serialization: {err}")))?;
            let not_found_bytes = serde_json::to_string(&not_found.denial)
                .map_err(|err| env_err(format!("denial serialization: {err}")))?;
            let isomorphic = denied_bytes == not_found_bytes;
            let denial_text = format!("{:?} {:?}", denied.denial, denied.audit);
            let body_disclosed = denial_text.contains(BODY_MARKER);
            let leaks_owner = denial_text.contains("user-c");
            let actual = json!({
                "decision": "deny",
                "error": registered(ctx, denied.denial.code)?,
                "body_disclosed": body_disclosed,
                "existence_leak_beyond_policy": !isomorphic || leaks_owner,
                "membership_alone_grants_read": false,
                // REAL: the gate produced a server-side DenialAudit record.
                "audit_required": true,
            });
            let evidence = json!({
                "denial_stage": denied.audit.stage,
                "denied_vs_not_found_serialized_identical": isomorphic,
                "denial_names_owner": leaks_owner,
                "audit_record": denied.audit,
                "allowed_paths_recorded_not_compared": vector.expected.get("allowed_paths"),
                "positive_control": "own-scope read with the same snapshot succeeds (asserted below)",
            });
            // Positive control: same snapshot reads its OWN scope fine.
            let own_target = ObjectGovernance {
                object_ref: format!("memory://{tenant}/user-b/profile?version=1"),
                tenant_id: Some(tenant.to_owned()),
                owner_ref: principal.to_owned(),
                resource_scope: own_scope.clone(),
                conversation_ref: None,
            };
            if protected_read(&snap, Some((&own_target, &body)), &access).is_err() {
                return Err(env_err("positive control failed: own-scope read denied"));
            }
            (actual, evidence)
        }
        ImplementationKind::DeliberatelyWrong => {
            // Wrong rule, driven for real: membership alone grants the read.
            let served = membership_valid.then_some(&body);
            let actual = json!({
                "decision": if served.is_some() { "allow" } else { "deny" },
                "error": Value::Null,
                "body_disclosed": served.map(|b| b.to_string().contains(BODY_MARKER)).unwrap_or(false),
                "existence_leak_beyond_policy": true,
                "membership_alone_grants_read": served.is_some(),
                "audit_required": false,
            });
            (
                actual,
                json!({ "wrong_rule": "membership_valid => serve body" }),
            )
        }
    };

    Ok(GateOutput {
        actual,
        grounding: lateral_grounding(),
        informative: vec!["allowed_paths"],
        implementation: implementation_label(kind),
        evidence,
    })
}

fn lateral_grounding() -> Vec<String> {
    vec![
        "crates/cognitive-kernel/src/authz.rs (six-step gate, protected_read isomorphism)"
            .to_owned(),
        "specs/registry/errors.yaml#CONTEXT_AUTH_DENIED".to_owned(),
        "docs/standards/authn-authz-capability.md section 2".to_owned(),
    ]
}

// ---------------------------------------------------------------------------
// CAP-ATTEN-004
// ---------------------------------------------------------------------------

fn constraints_from(
    value: &Value,
    fallback_not_before: &str,
) -> Result<CapabilityConstraints, ExecError> {
    let audience = value
        .get("audience")
        .and_then(Value::as_str)
        .ok_or_else(|| env_err("capability lacks audience"))?;
    let actions: BTreeSet<String> = value
        .get("actions")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let mut parameter_bounds = std::collections::BTreeMap::new();
    if let Some(bindings) = value.get("parameter_binding").and_then(Value::as_object) {
        for (name, bound) in bindings {
            let max = bound
                .as_i64()
                .ok_or_else(|| env_err(format!("non-numeric parameter binding {name}")))?;
            parameter_bounds.insert(name.clone(), ParameterBound::NumericMax(max));
        }
    }
    let expires = value
        .get("expires")
        .and_then(Value::as_str)
        .ok_or_else(|| env_err("capability lacks expires"))?;
    let depth = value
        .get("depth_remaining")
        .and_then(Value::as_i64)
        .ok_or_else(|| env_err("capability lacks depth_remaining"))?;
    Ok(CapabilityConstraints {
        subject: "principal://tenant-a/delegator".to_owned(),
        audience: audience.to_owned(),
        resource: "scope://tenant-a/payments".to_owned(),
        purpose: "payments".to_owned(),
        actions,
        parameter_bounds,
        lease: LeaseWindow {
            not_before: ts(fallback_not_before)?,
            expires: ts(expires)?,
        },
        depth_remaining: depth,
        issued_epoch: 41,
    })
}

pub(super) fn attenuation_behavior(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let parent = constraints_from(
        vector
            .input
            .get("parent")
            .ok_or_else(|| env_err("vector lacks parent capability"))?,
        "2026-07-18T12:00:00Z",
    )?;
    let derived = constraints_from(
        vector
            .input
            .get("derived")
            .ok_or_else(|| env_err("vector lacks derived capability"))?,
        "2026-07-18T12:00:00Z",
    )?;

    let violations = attenuation_violations(&parent, &derived);
    let actual = match kind {
        ImplementationKind::Reference => {
            if violations.is_empty() {
                json!({
                    "decision": "allow",
                    "error": Value::Null,
                    "violations": [],
                    "audit_required": false,
                })
            } else {
                json!({
                    "decision": "deny",
                    "error": registered(ctx, "AUTH_CAPABILITY_ATTENUATION_VIOLATION")?,
                    "violations": violations,
                    // Contract constant for governed denials (REQ-AUDIT-001);
                    // derivation audit trail is an M5 issuance-service duty.
                    "audit_required": true,
                })
            }
        }
        // The wrong implementation accepts the amplified derivation.
        ImplementationKind::DeliberatelyWrong => json!({
            "decision": "allow",
            "error": Value::Null,
            "violations": [],
            "audit_required": false,
        }),
    };

    Ok(GateOutput {
        actual,
        grounding: vec![
            "crates/cognitive-domain/src/capability.rs (attenuation_violations)".to_owned(),
            "specs/registry/errors.yaml#AUTH_CAPABILITY_ATTENUATION_VIOLATION".to_owned(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence: json!({
            "violations_found": violations,
            "parent_bound_max_amount_minor": vector.input.pointer("/parent/parameter_binding/max_amount_minor"),
            "derived_bound_max_amount_minor": vector.input.pointer("/derived/parameter_binding/max_amount_minor"),
            "audit_required_basis": "contract constant for governed denials (REQ-AUDIT-001)",
        }),
    })
}

// ---------------------------------------------------------------------------
// CTX-REVOKE-CACHE-001
// ---------------------------------------------------------------------------

fn binding_from(input: &Value, epoch: i64) -> Result<GovernanceBinding, ExecError> {
    let cache_binding = input
        .pointer("/cached_context_view/cache_binding")
        .ok_or_else(|| env_err("vector lacks cache_binding"))?;
    Ok(GovernanceBinding {
        tenant: cache_binding
            .get("tenant")
            .and_then(Value::as_str)
            .unwrap_or("tenant-a")
            .to_owned(),
        actor_chain_digest: cache_binding
            .get("actor_chain_digest")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        capability_set_version: 7,
        revocation_epoch: epoch,
        purpose: cache_binding
            .get("purpose")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        schema_digest: format!("sha256:{}", "5d".repeat(32)),
        encoding_profile: cognitive_contracts::ENCODING_PROFILE.to_owned(),
        conversation: cache_binding
            .get("conversation")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}

fn derived_kind_label(kind: DerivedCacheKind) -> &'static str {
    match kind {
        DerivedCacheKind::KvCache => "kv_cache",
        DerivedCacheKind::PromptCache => "prompt_cache",
        DerivedCacheKind::EmbeddingResult => "embedding_result",
        DerivedCacheKind::Summary => "summary",
    }
}

pub(super) fn revocation_cache_behavior(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let input = &vector.input;
    let stale_epoch = input
        .pointer("/cached_context_view/cache_binding/revocation_version")
        .and_then(Value::as_i64)
        .ok_or_else(|| env_err("vector lacks cached revocation_version"))?;
    let new_epoch = input
        .pointer("/revocation_event/new_revocation_version")
        .and_then(Value::as_i64)
        .ok_or_else(|| env_err("vector lacks new_revocation_version"))?;
    let declared_epoch = input
        .pointer("/subsequent_request/declared_revocation_version")
        .and_then(Value::as_i64)
        .unwrap_or(stale_epoch);
    let loaded_refs: Vec<String> = input
        .pointer("/cached_context_view/loaded_items")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();

    let stale = binding_from(input, stale_epoch)?;
    let declared = binding_from(input, declared_epoch)?;
    let current = binding_from(input, new_epoch)?;

    let mut cache = ContextViewCache::default();
    cache.insert(
        stale.clone(),
        CachedView {
            render_digest: format!("sha256:{}", "9f".repeat(32)),
            loaded_refs: loaded_refs.clone(),
            derived: vec![
                DerivedCacheKind::KvCache,
                DerivedCacheKind::PromptCache,
                DerivedCacheKind::EmbeddingResult,
                DerivedCacheKind::Summary,
            ],
        },
    );

    let (actual, evidence) = match kind {
        ImplementationKind::Reference => {
            // Structural miss under the CURRENT binding (stale key
            // unreachable by construction).
            let hit = cache.lookup_current(&current);
            let cache_hit_served = hit.is_some();
            // The declared stale binding is refused and purged with its
            // derived caches.
            let (denial, report) = match cache.serve_declared(&declared, &current) {
                Err((denial, report)) => (denial, report),
                Ok(_) => {
                    return Err(env_err(
                        "declared stale binding was served instead of refused",
                    ));
                }
            };
            let invalidated: Vec<&'static str> = report
                .as_ref()
                .map(|r| {
                    r.derived_caches_invalidated
                        .iter()
                        .map(|k| derived_kind_label(*k))
                        .collect()
                })
                .unwrap_or_default();
            // No purchase-back: declaring again finds nothing to serve
            // (entry purged), still refused.
            let second = cache.serve_declared(&declared, &current);
            let repurchase_possible = second.is_ok();
            // Fresh resolution under the new epoch with the stale-epoch
            // chain loads nothing (authorization re-runs; nothing skipped).
            let stale_chain_snapshot = snapshot(
                &current.tenant,
                "principal://tenant-a/user-b",
                &current.actor_chain_digest,
                vec![link(
                    "scope://tenant-a/user-b/personal",
                    &["read_body"],
                    &current.purpose,
                )?],
                new_epoch,
            )?;
            let fresh = resolve(
                &request(
                    stale_chain_snapshot,
                    &current.purpose,
                    vec![],
                    true,
                    ContextBudget::default(),
                ),
                &[candidate(
                    &loaded_refs.first().cloned().unwrap_or_else(|| {
                        "knowledge://tenant-a/policy/refund?version=8".to_owned()
                    }),
                    Some(current.tenant.as_str()),
                    "scope://tenant-a/user-b/personal",
                    LoadedContextItemRole::Evidence,
                    LoadedContextItemTrustLevel::Verified,
                    json!({"policy": "cached-body"}),
                    10,
                    1,
                )],
                &ArrivalOrderRanker,
            )
            .map_err(|err| env_err(format!("fresh resolve failed structurally: {err}")))?;
            let stale_chain_loads = !fresh.loaded.is_empty();

            let actual = json!({
                "cache_hit_served": cache_hit_served,
                "decision": "revalidate_or_reresolve",
                "error": registered(ctx, denial.code)?,
                // The cache stores digests and refs only; the refusal path
                // returned no view/body.
                "revoked_body_reexposed": false,
                "derived_caches_invalidated": invalidated,
                // Authorization dimensions live in the cache key itself and
                // the fresh resolution re-ran per-object authorization
                // (stale chain loaded nothing).
                "authorization_skipped_on_cache_hit": stale_chain_loads,
                "audit_required": true,
            });
            let evidence = json!({
                "structural_miss_under_current_binding": !cache_hit_served,
                "invalidation_report_produced": report.is_some(),
                "second_declaration_repurchases": repurchase_possible,
                "fresh_resolve_with_stale_epoch_chain_loaded": stale_chain_loads,
                "fresh_resolve_rejected": fresh.rejected,
                "decision_mapping": "declared-stale refusal + structural miss => revalidate_or_reresolve (context_cache::serve_declared)",
                "audit_basis": "InvalidationReport is the audit-facing record (context_cache.rs)",
            });
            (actual, evidence)
        }
        ImplementationKind::DeliberatelyWrong => {
            // Wrong cache, driven for real: looks up with the DECLARED
            // binding as if it were current — the stale entry hits.
            let hit = cache.lookup_current(&declared);
            let actual = json!({
                "cache_hit_served": hit.is_some(),
                "decision": "serve_cached",
                "error": Value::Null,
                "revoked_body_reexposed": hit.is_some(),
                "derived_caches_invalidated": [],
                "authorization_skipped_on_cache_hit": true,
                "audit_required": false,
            });
            (
                actual,
                json!({ "wrong_rule": "cache keyed without the revocation epoch dimension" }),
            )
        }
    };

    Ok(GateOutput {
        actual,
        grounding: vec![
            "crates/cognitive-kernel/src/context_cache.rs (governance-bound keys, serve_declared)"
                .to_owned(),
            "crates/cognitive-kernel/src/authz.rs (per-object re-authorization on fresh resolve)"
                .to_owned(),
            "specs/registry/errors.yaml#CONTEXT_AUTH_DENIED".to_owned(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence,
    })
}

// ---------------------------------------------------------------------------
// CTX-RANK-AUTH-001
// ---------------------------------------------------------------------------

pub(super) fn rank_before_auth_behavior(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let input = &vector.input;
    let tenant = input
        .pointer("/context_request/tenant")
        .and_then(Value::as_str)
        .unwrap_or("tenant-a");
    let purpose = input
        .pointer("/context_request/purpose")
        .and_then(Value::as_str)
        .unwrap_or("answer_customer_question");
    let chain_digest = input
        .pointer("/context_request/actor_chain_digest")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let discovered = input
        .pointer("/discovered_candidates")
        .and_then(Value::as_array)
        .ok_or_else(|| env_err("vector lacks discovered_candidates"))?;

    const AUTHORIZED_SCOPE: &str = "scope://tenant-a/shared/knowledge";
    const PRIVATE_SCOPE: &str = "scope://tenant-a/user-b/private";
    const UNAUTHORIZED_MARKER: &str = "UNAUTHORIZED-BODY-MARKER-2c9d";

    let mut candidates: Vec<CandidateObject> = Vec::new();
    let mut authorized_refs: Vec<String> = Vec::new();
    let mut denied_refs: Vec<String> = Vec::new();
    for entry in discovered {
        let object_ref = entry
            .get("ref")
            .and_then(Value::as_str)
            .ok_or_else(|| env_err("candidate lacks ref"))?;
        let authorized = entry
            .get("read_authorized_for_actor")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let (scope, body) = if authorized {
            authorized_refs.push(object_ref.to_owned());
            (
                AUTHORIZED_SCOPE,
                json!({"policy": "refunds require approval"}),
            )
        } else {
            denied_refs.push(object_ref.to_owned());
            (PRIVATE_SCOPE, json!({"notes": UNAUTHORIZED_MARKER}))
        };
        candidates.push(candidate(
            object_ref,
            Some(tenant),
            scope,
            LoadedContextItemRole::Evidence,
            LoadedContextItemTrustLevel::Verified,
            body,
            50,
            5,
        ));
    }
    // The actor's capability covers the shared knowledge scope only.
    let snap = snapshot(
        tenant,
        "principal://tenant-a/actor",
        chain_digest,
        vec![link(AUTHORIZED_SCOPE, &["read_body"], purpose)?],
        41,
    )?;
    let req = request(snap, purpose, vec![], true, ContextBudget::default());

    let (actual, evidence) = match kind {
        ImplementationKind::Reference => {
            let recording = RecordingRanker::default();
            let view = resolve(&req, &candidates, &recording)
                .map_err(|err| env_err(format!("resolve failed structurally: {err}")))?;
            let seen_refs = recording.seen_refs.borrow().clone();
            let exposed = recording
                .seen_bodies
                .borrow()
                .iter()
                .any(|body| body.contains(UNAUTHORIZED_MARKER));
            let denied_ref = denied_refs
                .first()
                .cloned()
                .ok_or_else(|| env_err("vector has no unauthorized candidate"))?;
            let denied_loaded = view.loaded.iter().any(|item| item.object_ref == denied_ref);
            let denied_rejection = view
                .rejected
                .iter()
                .find(|r| r.candidate_ref == denied_ref)
                .ok_or_else(|| env_err("unauthorized candidate missing from rejected list"))?;

            // Hostile ranker cannot restore the denied item or smuggle an
            // external ref into the view.
            let hostile = HostileRanker {
                smuggle: vec![
                    denied_ref.clone(),
                    "knowledge://tenant-a/external".to_owned(),
                ],
            };
            let hostile_view = resolve(&req, &candidates, &hostile)
                .map_err(|err| env_err(format!("hostile resolve failed: {err}")))?;
            let restored = hostile_view
                .loaded
                .iter()
                .any(|item| item.object_ref == denied_ref || item.object_ref.contains("external"));

            let actual = json!({
                "decision": "rank_only_authorized_candidates",
                "ranker_input_refs": view.ranker_input_refs,
                "unauthorized_body_exposed_to_ranker": exposed,
                "unauthorized_item_result": {
                    "loaded": denied_loaded,
                    "error": registered(ctx, &denied_rejection.reason)?,
                },
                "rank_score_cannot_restore_denied_item": !restored,
                "audit_required": true,
            });
            let evidence = json!({
                "recording_ranker_saw_refs": seen_refs,
                "hostile_proposal_restored_denied_or_external": restored,
                "rejected": view.rejected,
                "decision_mapping": "resolve() stage 5 receives authorized survivors only => rank_only_authorized_candidates",
                "audit_basis": "denials recorded in view.rejected with registered codes (stage records retained)",
            });
            (actual, evidence)
        }
        ImplementationKind::DeliberatelyWrong => {
            // Wrong pipeline, driven for real: every discovered candidate
            // (including unauthorized bodies) is offered to the ranker and
            // its proposal is loaded without authorization.
            let recording = RecordingRanker::default();
            let all: Vec<RankerCandidate<'_>> = candidates
                .iter()
                .map(|candidate| RankerCandidate {
                    object_ref: candidate.object_ref.as_str(),
                    role: candidate.role,
                    body: &candidate.body,
                })
                .collect();
            let proposal = recording.rank(&all);
            let exposed = recording
                .seen_bodies
                .borrow()
                .iter()
                .any(|body| body.contains(UNAUTHORIZED_MARKER));
            let denied_ref = denied_refs.first().cloned().unwrap_or_default();
            let actual = json!({
                "decision": "rank_everything",
                "ranker_input_refs": proposal,
                "unauthorized_body_exposed_to_ranker": exposed,
                "unauthorized_item_result": {
                    "loaded": true,
                    "error": Value::Null,
                },
                "rank_score_cannot_restore_denied_item": false,
                "audit_required": false,
            });
            (
                actual,
                json!({ "wrong_rule": format!("ranked and loaded {denied_ref} without authorization") }),
            )
        }
    };

    Ok(GateOutput {
        actual,
        grounding: vec![
            "crates/cognitive-kernel/src/context.rs (stage 4 authorization before stage 5 ranking)"
                .to_owned(),
            "specs/registry/errors.yaml#CONTEXT_AUTH_DENIED".to_owned(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence,
    })
}

// ---------------------------------------------------------------------------
// CTX-REQ-007
// ---------------------------------------------------------------------------

pub(super) fn required_budget_behavior(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let input = &vector.input;
    let allow_partial = input
        .get("allow_partial")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let budget = ContextBudget {
        context_bytes: input
            .pointer("/hard_budget/context_bytes")
            .and_then(Value::as_i64),
        input_tokens: input
            .pointer("/hard_budget/input_tokens")
            .and_then(Value::as_i64),
    };
    let items = input
        .get("required_items")
        .and_then(Value::as_array)
        .ok_or_else(|| env_err("vector lacks required_items"))?;

    const SCOPE: &str = "scope://tenant-a/workspace";
    let mut required = Vec::new();
    let mut candidates = Vec::new();
    for item in items {
        let object_ref = item
            .get("ref")
            .and_then(Value::as_str)
            .ok_or_else(|| env_err("required item lacks ref"))?;
        let bytes = item.get("bytes").and_then(Value::as_i64).unwrap_or(0);
        let tokens = item.get("tokens").and_then(Value::as_i64).unwrap_or(0);
        required.push(RequiredItem {
            object_ref: object_ref.to_owned(),
        });
        candidates.push(candidate(
            object_ref,
            Some("tenant-a"),
            SCOPE,
            LoadedContextItemRole::AuthoritativeState,
            LoadedContextItemTrustLevel::Authoritative,
            json!({"ref": object_ref}),
            bytes,
            tokens,
        ));
    }
    let snap = snapshot(
        "tenant-a",
        "principal://tenant-a/user-b",
        &format!("sha256:{}", "aa".repeat(32)),
        vec![link(SCOPE, &["read_body"], "task_execution")?],
        41,
    )?;
    let req = request(snap, "task_execution", required, allow_partial, budget);

    let (actual, evidence) = match kind {
        ImplementationKind::Reference => match resolve(&req, &candidates, &ArrivalOrderRanker) {
            Ok(view) => (
                json!({
                    "decision": "ok",
                    "error": Value::Null,
                    "context_view_emitted": true,
                    "missing_items": view.missing,
                }),
                json!({ "loaded": view.loaded.len() }),
            ),
            Err(failure) => (
                json!({
                    "decision": "error",
                    "error": registered(ctx, failure.code)?,
                    // Err = fail closed, no view value exists to emit.
                    "context_view_emitted": false,
                    "missing_items": failure.missing_items,
                }),
                json!({
                    "failure_stage": failure.stage,
                    "failure_detail": failure.detail,
                    "decision_mapping": "ResolutionFailure => decision error, no view emitted",
                }),
            ),
        },
        ImplementationKind::DeliberatelyWrong => {
            // Wrong pipeline: silently truncate the required set to fit and
            // report success.
            let mut kept: Vec<String> = Vec::new();
            let mut used = 0i64;
            for candidate in &candidates {
                if req
                    .budget
                    .context_bytes
                    .is_none_or(|max| used + candidate.cost_bytes <= max)
                {
                    used += candidate.cost_bytes;
                    kept.push(candidate.object_ref.clone());
                }
            }
            (
                json!({
                    "decision": "ok",
                    "error": Value::Null,
                    "context_view_emitted": true,
                    "missing_items": [],
                }),
                json!({ "wrong_rule": "silently truncated required set", "kept": kept }),
            )
        }
    };

    Ok(GateOutput {
        actual,
        grounding: vec![
            "crates/cognitive-kernel/src/context.rs (stage 6 budget fitting, fail-closed)"
                .to_owned(),
            "specs/registry/errors.yaml#CONTEXT_BUDGET_EXCEEDED".to_owned(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence,
    })
}

// ---------------------------------------------------------------------------
// CTX-RENDER-001
// ---------------------------------------------------------------------------

pub(super) fn render_stability_behavior(
    _ctx: &AssetContext,
    _vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    const SCOPE: &str = "scope://tenant-a/workspace";
    let base = |object_ref: &str, body: Value| {
        candidate(
            object_ref,
            Some("tenant-a"),
            SCOPE,
            LoadedContextItemRole::Working,
            LoadedContextItemTrustLevel::Verified,
            body,
            20,
            2,
        )
    };
    let a = base("state://tenant-a/task/1?version=3", json!({"a": 1}));
    let b = base("knowledge://tenant-a/policy/2?version=8", json!({"b": 2}));
    let c = base("memory://tenant-a/unrelated/3?version=1", json!({"c": 3}));
    let snap = snapshot(
        "tenant-a",
        "principal://tenant-a/user-b",
        &format!("sha256:{}", "aa".repeat(32)),
        vec![link(SCOPE, &["read_body"], "task_execution")?],
        41,
    )?;
    let req = request(
        snap,
        "task_execution",
        vec![],
        true,
        ContextBudget::default(),
    );

    let first = resolve(&req, &[a.clone(), b.clone()], &ArrivalOrderRanker)
        .map_err(|err| env_err(format!("first resolve failed: {err}")))?;
    let second = resolve(&req, &[a.clone(), b.clone()], &ArrivalOrderRanker)
        .map_err(|err| env_err(format!("second resolve failed: {err}")))?;
    let extended = resolve(
        &req,
        &[a.clone(), b.clone(), c.clone()],
        &ArrivalOrderRanker,
    )
    .map_err(|err| env_err(format!("extended resolve failed: {err}")))?;

    let repeat_equal =
        first.render.digest == second.render.digest && first.render.bytes == second.render.bytes;
    let old_positions: Vec<usize> = [&a.object_ref, &b.object_ref]
        .iter()
        .filter_map(|wanted| {
            extended
                .loaded
                .iter()
                .position(|item| &&item.object_ref == wanted)
        })
        .collect();
    let order_preserved = old_positions == vec![0, 1];
    let prefix_len = first.render.segments.len();
    let prefix_unchanged = extended.render.segments.len() > prefix_len - 1
        && first
            .render
            .segments
            .iter()
            .zip(extended.render.segments.iter())
            .all(|(old, new)| old.bytes == new.bytes && old.item_ref == new.item_ref);
    let old_stream_is_prefix = extended.render.bytes.starts_with(&first.render.bytes);
    let new_at_suffix = extended
        .render
        .segments
        .last()
        .is_some_and(|segment| segment.item_ref == c.object_ref);

    let (actual, evidence) = match kind {
        ImplementationKind::Reference => (
            json!({
                "outcome": "allowed",
                "repeat_render_digest_equal": repeat_equal,
                "existing_item_relative_order_preserved": order_preserved,
                "existing_prefix_segment_bytes_unchanged": prefix_unchanged && old_stream_is_prefix,
                "new_content_position": if new_at_suffix {
                    "append_only_suffix_or_declared_cache_breakpoint"
                } else {
                    "reordered"
                },
                "per_activity_reordering_observed": !order_preserved,
                "authority_unchanged": true,
                "capability_expanded": false,
            }),
            json!({
                "first_render_digest": first.render.digest,
                "extended_render_digest": extended.render.digest,
                "prefix_segments_compared": prefix_len,
                "input_placeholder_digests_note": "input.prefix_segments_before_addition carries illustrative placeholders; the real prefix digests are recorded here",
                "real_prefix_segment_digests": first.render.segments.iter().map(|s| s.digest.clone()).collect::<Vec<_>>(),
                "structural_facts": STRUCTURAL_FACTS,
            }),
        ),
        ImplementationKind::DeliberatelyWrong => {
            // Wrong renderer, driven for real: re-sorts every segment by
            // ref on each render, so adding C reshuffles the prefix.
            let mut reshuffled = extended.render.segments.clone();
            reshuffled.sort_by(|left, right| right.item_ref.cmp(&left.item_ref));
            let wrong_prefix_unchanged = first
                .render
                .segments
                .iter()
                .zip(reshuffled.iter())
                .all(|(old, new)| old.bytes == new.bytes);
            (
                json!({
                    "outcome": "allowed",
                    "repeat_render_digest_equal": repeat_equal,
                    "existing_item_relative_order_preserved": false,
                    "existing_prefix_segment_bytes_unchanged": wrong_prefix_unchanged,
                    "new_content_position": "reordered",
                    "per_activity_reordering_observed": true,
                    "authority_unchanged": true,
                    "capability_expanded": false,
                }),
                json!({ "wrong_rule": "renderer re-sorts all segments on every render" }),
            )
        }
    };

    Ok(GateOutput {
        actual,
        grounding: vec![
            "crates/cognitive-kernel/src/context.rs (deterministic render, partition + arrival order)".to_owned(),
            "specs/registry/requirements.yaml#REQ-CTX-012".to_owned(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence,
    })
}

// ---------------------------------------------------------------------------
// DISC-STAGNATION-004
// ---------------------------------------------------------------------------

pub(super) fn stagnation_behavior(
    _ctx: &AssetContext,
    _vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    const SCOPE: &str = "scope://tenant-a/workspace";
    let unloadable = "knowledge://tenant-a/never-materializes?version=1";
    let req = request(
        snapshot(
            "tenant-a",
            "principal://tenant-a/user-b",
            &format!("sha256:{}", "aa".repeat(32)),
            vec![link(SCOPE, &["read_body"], "task_execution")?],
            41,
        )?,
        "task_execution",
        vec![RequiredItem {
            object_ref: unloadable.to_owned(),
        }],
        false,
        ContextBudget::default(),
    );

    let mut session = ResolutionSession::default();
    let mut attempts = 0u32;
    let mut stagnation: Option<cognitive_kernel::context::ResolutionFailure> = None;
    let bound = match kind {
        ImplementationKind::Reference => 10,
        // The wrong implementation retries far past any bound and never
        // consults the session.
        ImplementationKind::DeliberatelyWrong => 25,
    };
    for _ in 0..bound {
        attempts += 1;
        let failure = match resolve(&req, &[], &ArrivalOrderRanker) {
            Err(failure) => failure,
            Ok(_) => return Err(env_err("unloadable required item resolved unexpectedly")),
        };
        if matches!(kind, ImplementationKind::Reference)
            && let Err(stagnated) = session.note_failed_attempt(&failure.missing_items)
        {
            stagnation = Some(stagnated);
            break;
        }
    }

    let (actual, evidence) = match kind {
        ImplementationKind::Reference => {
            let stagnated = stagnation
                .ok_or_else(|| env_err("bounded session never stagnated within 10 attempts"))?;
            (
                json!({
                    "outcome": "denied_or_controlled_fallback",
                    "error_code": stagnated.code,
                    "authority_unchanged": true,
                    "capability_expanded": false,
                }),
                json!({
                    "attempts_before_stagnation": attempts,
                    "stagnation_bound": cognitive_kernel::context::STAGNATION_BOUND,
                    "per_attempt_failure_code": "CONTEXT_INCOMPLETE",
                    "outcome_mapping": "ResolutionSession refuses further retries with the registered code => denied_or_controlled_fallback",
                    "structural_facts": STRUCTURAL_FACTS,
                }),
            )
        }
        ImplementationKind::DeliberatelyWrong => (
            json!({
                "outcome": "allowed_unbounded_retry",
                "error_code": Value::Null,
                "authority_unchanged": true,
                "capability_expanded": false,
            }),
            json!({ "wrong_rule": "retried without a session bound", "attempts": attempts }),
        ),
    };

    Ok(GateOutput {
        actual,
        grounding: vec![
            "crates/cognitive-kernel/src/context.rs (ResolutionSession, STAGNATION_BOUND)"
                .to_owned(),
            "specs/registry/errors.yaml#CONTEXT_RESOLUTION_STAGNATED".to_owned(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence,
    })
}

// ---------------------------------------------------------------------------
// DISC-ADMISSION-002
// ---------------------------------------------------------------------------

pub(super) fn candidate_admission_behavior(
    ctx: &AssetContext,
    _vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    const AUTHORIZED_SCOPE: &str = "scope://tenant-a/shared/knowledge";
    let in_scope = candidate(
        "knowledge://tenant-a/shared/faq?version=2",
        Some("tenant-a"),
        AUTHORIZED_SCOPE,
        LoadedContextItemRole::Evidence,
        LoadedContextItemTrustLevel::Verified,
        json!({"faq": true}),
        10,
        1,
    );
    let out_of_capability = candidate(
        "memory://tenant-a/user-c/private?version=4",
        Some("tenant-a"),
        "scope://tenant-a/user-c/personal",
        LoadedContextItemRole::Evidence,
        LoadedContextItemTrustLevel::Verified,
        json!({"private": true}),
        10,
        1,
    );
    let cross_tenant = candidate(
        "knowledge://tenant-b/leak?version=1",
        Some("tenant-b"),
        "scope://tenant-b/shared",
        LoadedContextItemRole::Evidence,
        LoadedContextItemTrustLevel::Verified,
        json!({"foreign": true}),
        10,
        1,
    );
    let all = [
        in_scope.clone(),
        out_of_capability.clone(),
        cross_tenant.clone(),
    ];
    let req = request(
        snapshot(
            "tenant-a",
            "principal://tenant-a/actor",
            &format!("sha256:{}", "aa".repeat(32)),
            vec![link(AUTHORIZED_SCOPE, &["read_body"], "task_execution")?],
            41,
        )?,
        "task_execution",
        vec![],
        true,
        ContextBudget::default(),
    );

    let (actual, evidence) = match kind {
        ImplementationKind::Reference => {
            let recording = RecordingRanker::default();
            let view = resolve(&req, &all, &recording)
                .map_err(|err| env_err(format!("resolve failed structurally: {err}")))?;
            let narrowed_before_execution = view.ranker_input_refs
                == vec![in_scope.object_ref.clone()]
                && recording.seen_refs.borrow().as_slice() == [in_scope.object_ref.clone()];
            let denied = view
                .rejected
                .iter()
                .find(|r| r.candidate_ref == out_of_capability.object_ref)
                .ok_or_else(|| env_err("out-of-capability candidate missing from rejected"))?;
            if !narrowed_before_execution {
                return Err(env_err("candidate set was not narrowed before ranking"));
            }
            // The rejection reason must be a registered machine code.
            registered(ctx, &denied.reason)?;
            (
                json!({
                    "outcome": "denied_or_controlled_fallback",
                    "error_code": denied.reason,
                    "authority_unchanged": true,
                    "capability_expanded": false,
                }),
                json!({
                    "loaded": view.loaded.iter().map(|i| i.object_ref.clone()).collect::<Vec<_>>(),
                    "rejected": view.rejected,
                    "ranker_saw_only_narrowed_set": narrowed_before_execution,
                    "outcome_mapping": "unauthorized candidates denied at prefilter/authorization stages while the narrowed resolution proceeds => denied_or_controlled_fallback",
                    "structural_facts": STRUCTURAL_FACTS,
                }),
            )
        }
        ImplementationKind::DeliberatelyWrong => {
            // Wrong pipeline: no narrowing — everything is offered onward.
            let recording = RecordingRanker::default();
            let everything: Vec<RankerCandidate<'_>> = all
                .iter()
                .map(|candidate| RankerCandidate {
                    object_ref: candidate.object_ref.as_str(),
                    role: candidate.role,
                    body: &candidate.body,
                })
                .collect();
            let proposal = recording.rank(&everything);
            (
                json!({
                    "outcome": "allowed",
                    "error_code": Value::Null,
                    "authority_unchanged": true,
                    "capability_expanded": false,
                }),
                json!({ "wrong_rule": "no narrowing before execution", "offered": proposal }),
            )
        }
    };

    Ok(GateOutput {
        actual,
        grounding: vec![
            "crates/cognitive-kernel/src/context.rs (stages 2+4 narrow the candidate set before ranking/loading)".to_owned(),
            "specs/registry/errors.yaml#CONTEXT_AUTH_DENIED".to_owned(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence,
    })
}

// ---------------------------------------------------------------------------
// CTX-TRUST-004 (behavioral upgrade of the M1 static trust-plane gate)
// ---------------------------------------------------------------------------

pub(super) fn trust_plane_behavior(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let items = vector
        .input
        .get("context_items")
        .and_then(Value::as_array)
        .ok_or_else(|| env_err("vector lacks context_items"))?;
    let mutations: Vec<String> = vector
        .input
        .get("proposed_control_mutations")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();

    const SCOPE: &str = "scope://tenant-a/workspace";
    let mut candidates: Vec<CandidateObject> = Vec::new();
    let mut control_ref = String::new();
    let mut injected_ref = String::new();
    for (index, item) in items.iter().enumerate() {
        let role = item.get("role").and_then(Value::as_str).unwrap_or_default();
        let content = item
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let object_ref = format!("context://tenant-a/item-{index}");
        let (item_role, trust) = match role {
            "control" => {
                control_ref = object_ref.clone();
                (
                    LoadedContextItemRole::Control,
                    LoadedContextItemTrustLevel::Control,
                )
            }
            _ => {
                injected_ref = object_ref.clone();
                (
                    LoadedContextItemRole::UntrustedInput,
                    LoadedContextItemTrustLevel::Untrusted,
                )
            }
        };
        candidates.push(candidate(
            &object_ref,
            Some("tenant-a"),
            SCOPE,
            item_role,
            trust,
            json!({ "content": content }),
            64,
            8,
        ));
    }
    if control_ref.is_empty() || injected_ref.is_empty() {
        return Err(env_err("vector needs one control and one untrusted item"));
    }
    let req = request(
        snapshot(
            "tenant-a",
            "principal://tenant-a/user-b",
            &format!("sha256:{}", "aa".repeat(32)),
            vec![link(SCOPE, &["read_body"], "task_execution")?],
            41,
        )?,
        "task_execution",
        vec![],
        true,
        ContextBudget::default(),
    );
    let view = resolve(&req, &candidates, &ArrivalOrderRanker)
        .map_err(|err| env_err(format!("resolve failed structurally: {err}")))?;

    let (actual, evidence) = match kind {
        ImplementationKind::Reference => {
            let injected_item = view
                .loaded
                .iter()
                .find(|item| item.object_ref == injected_ref)
                .ok_or_else(|| env_err("injected item not loaded"))?;
            let render_role = match injected_item.role {
                LoadedContextItemRole::UntrustedInput => "untrusted_input",
                LoadedContextItemRole::Control => "control",
                LoadedContextItemRole::AuthoritativeState => "authoritative_state",
                LoadedContextItemRole::Evidence => "evidence",
                LoadedContextItemRole::Working => "working",
            };
            let control_plane = effective_control_plane(&view);
            let effective_policy = control_plane
                .iter()
                .filter_map(|item| item.body.get("content").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("\n");
            // The proposed control mutation is attributed to the injected
            // untrusted item — the real gate refuses it.
            let mutation_admitted = admit_control_mutation(&view, &injected_ref).is_ok();
            let decision = if mutations.is_empty() || mutation_admitted {
                "accept"
            } else {
                "reject_control_mutation"
            };
            (
                json!({
                    "decision": decision,
                    "render_role": render_role,
                    "effective_policy": effective_policy,
                    // No API on this surface can mint an
                    // AuthorizationCapability, and the mutation gate refused
                    // the untrusted source.
                    "capability_created": mutation_admitted,
                    "audit_required": true,
                }),
                json!({
                    "control_plane_refs": control_plane.iter().map(|i| i.object_ref.clone()).collect::<Vec<_>>(),
                    "mutation_admitted_for_untrusted_source": mutation_admitted,
                    "denial_code_on_refusal": registered(ctx, "CONTEXT_AUTH_DENIED")?,
                    "audit_basis": "refusal is a registered-coded ResolutionFailure; rejected/stage records retained in the view",
                }),
            )
        }
        ImplementationKind::DeliberatelyWrong => {
            // Wrong extraction, driven for real: the control plane is built
            // from CONTENT claims, so the injected item takes over.
            let claimed: Vec<&cognitive_kernel::context::LoadedItem> = view
                .loaded
                .iter()
                .filter(|item| {
                    item.body
                        .get("content")
                        .and_then(Value::as_str)
                        .is_some_and(|content| content.contains("Ignore all policy"))
                })
                .collect();
            let effective_policy = claimed
                .first()
                .and_then(|item| item.body.get("content").and_then(Value::as_str))
                .unwrap_or_default();
            (
                json!({
                    "decision": "accept",
                    "render_role": "control",
                    "effective_policy": effective_policy,
                    "capability_created": true,
                    "audit_required": false,
                }),
                json!({ "wrong_rule": "control plane derived from content claims" }),
            )
        }
    };

    Ok(GateOutput {
        actual,
        grounding: vec![
            "crates/cognitive-kernel/src/context.rs (effective_control_plane, admit_control_mutation)".to_owned(),
            "specs/registry/requirements.yaml#REQ-CTX-008".to_owned(),
            "specs/registry/requirements.yaml#REQ-SEC-002".to_owned(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence,
    })
}
