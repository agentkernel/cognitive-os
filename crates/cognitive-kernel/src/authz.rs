//! Deterministic six-step authorization gate
//! (`docs/standards/authn-authz-capability.md` section 2; REQ-AUTH-001,
//! REQ-CAP-001..005, REQ-SEC-001).
//!
//! Decision order, failing closed at the first violated step:
//!
//! 1. authenticate principal, resolve the full ActorChain (identity never
//!    comes from natural language);
//! 2. resolve tenant and membership — tenant match alone grants nothing
//!    (REQ-GOBJ-TENANT-001; vector `tenant-lateral-read-denial.json`);
//! 3. intersect every applicable capability link (never union) and check
//!    revocation currency;
//! 4. apply explicit deny (beats any allow; default deny with no grant);
//! 5. check lease validity (`AUTH_CAPABILITY_EXPIRED`; expired leases are
//!    never extended, ADR-0005);
//! 6. check scope, purpose, action and parameter binding
//!    (`CONTEXT_AUTH_DENIED` on mismatch).
//!
//! Denial isomorphism (error-contract standard section 5): where existence
//! is protected, a denial and a not-found MUST be the same shape, same
//! code, same detail. [`protected_read`] produces byte-identical denials
//! for "exists but denied" and "does not exist".
//!
//! Every decision here is pure data arithmetic: the caller supplies the
//! governance snapshot (facts current at the decision instant) and this
//! module never consults a store, clock, or model.

use crate::error::{AUTH_CAPABILITY_EXPIRED, CONTEXT_AUTH_DENIED, RegisteredError};
use cognitive_domain::capability::{CapabilityConstraints, EffectiveRights, intersect_chain};
use cognitive_domain::{UriRef, WallTimestamp, capability::resource_within};
use serde::Serialize;
use std::collections::BTreeSet;

/// Authenticated principal facts at decision time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrincipalFacts {
    /// Principal reference.
    pub principal_ref: UriRef,
    /// True only when authentication completed against the registered
    /// authentication authority. Claims in natural language never set this.
    pub authenticated: bool,
    /// Principal status is `active` (suspended/revoked principals fail).
    pub active: bool,
    /// Tenant the principal belongs to (None = platform principal).
    pub tenant_id: Option<String>,
}

/// Resolved actor-chain facts (immutable ordered chain, REQ-GOBJ-BIND-002).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorChainFacts {
    /// Canonical chain digest (cache-key component).
    pub chain_digest: String,
    /// True when the chain resolved completely (no unresolved link).
    pub resolved: bool,
}

/// Membership facts of the principal within the request tenant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipFacts {
    /// Membership is currently valid.
    pub valid: bool,
    /// Roles held (informative for audit; roles alone grant nothing).
    pub roles: BTreeSet<String>,
}

/// One explicit deny rule (step 4). Explicit deny beats any allow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenyRule {
    /// Resource scope URI prefix the deny covers.
    pub resource_prefix: String,
    /// Actions denied (empty set = all actions).
    pub actions: BTreeSet<String>,
}

/// Deterministic governance snapshot for one authorization decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthzSnapshot {
    /// Tenant the request executes in.
    pub tenant_id: String,
    /// Authenticated principal facts.
    pub principal: PrincipalFacts,
    /// Resolved actor chain facts.
    pub actor_chain: ActorChainFacts,
    /// Membership facts for the request tenant (None = no membership).
    pub membership: Option<MembershipFacts>,
    /// Applicable capability chain links (already schema-verified upstream).
    pub capability_links: Vec<CapabilityConstraints>,
    /// Capability set version (cache-key component, REQ-CAP-005).
    pub capability_set_version: i64,
    /// Explicit deny rules in force.
    pub explicit_denies: Vec<DenyRule>,
    /// Current revocation epoch (advances on every revocation).
    pub revocation_epoch: i64,
    /// Decision instant (wall clock supplied by the deterministic caller).
    pub decided_at: WallTimestamp,
}

/// Governance facts of the target object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectGovernance {
    /// Object reference (URI, version-qualified where applicable).
    pub object_ref: String,
    /// Tenant owning the object (None = platform object).
    pub tenant_id: Option<String>,
    /// Lifecycle owner.
    pub owner_ref: String,
    /// Resource scope URI the object lives in.
    pub resource_scope: String,
    /// Conversation the object is bound to, when conversation-scoped.
    pub conversation_ref: Option<String>,
}

/// One requested access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessRequest {
    /// Action requested (for example `read_body`, `govern`).
    pub action: String,
    /// Purpose binding of the request.
    pub purpose: String,
}

/// Denial stages for audit (never leaked into the public response shape).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStage {
    Authentication,
    TenantMembership,
    CapabilityIntersection,
    ExplicitDeny,
    Lease,
    ScopePurposeBinding,
}

/// Audit-facing record of a denial (REQ-SEC-001: fail closed AND audit).
/// Carries no object body, no secret, and no cross-tenant content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DenialAudit {
    /// Stage that failed.
    pub stage: DecisionStage,
    /// Registered code surfaced.
    pub code: &'static str,
    /// Principal that was denied.
    pub principal_ref: String,
    /// Action requested.
    pub action: String,
    /// Purpose requested.
    pub purpose: String,
}

/// The public, existence-safe denial shape. Two denials with the same code
/// are byte-identical regardless of whether the target exists, is foreign,
/// or is simply out of scope (`existence_leak_beyond_policy: false`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AccessDenial {
    /// Registered machine code.
    pub code: &'static str,
    /// Registered category.
    pub category: &'static str,
    /// Registered retryability.
    pub retryable: bool,
    /// Fixed, existence-safe detail text (never names the object).
    pub detail: &'static str,
}

/// A denial with its audit record (audit stays server-side; the public
/// response is [`AccessDenial`] alone). The audit record is boxed to keep
/// the hot `Result` path small.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeniedAccess {
    /// Public response shape.
    pub denial: AccessDenial,
    /// Server-side audit record.
    pub audit: Box<DenialAudit>,
}

const DENIAL_DETAIL: &str = "not available for this principal and purpose";

fn denial(
    registered: RegisteredError,
    stage: DecisionStage,
    snapshot: &AuthzSnapshot,
    request: &AccessRequest,
) -> DeniedAccess {
    DeniedAccess {
        denial: AccessDenial {
            code: registered.code,
            category: registered.category,
            retryable: registered.retryable,
            detail: DENIAL_DETAIL,
        },
        audit: Box::new(DenialAudit {
            stage,
            code: registered.code,
            principal_ref: snapshot.principal.principal_ref.as_str().to_owned(),
            action: request.action.clone(),
            purpose: request.purpose.clone(),
        }),
    }
}

/// A granted decision: what was effectively allowed and under which
/// governance versions (feeds cache keys and dispatch/commit revalidation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationGrant {
    /// Effective rights after chain intersection.
    pub effective: EffectiveRights,
    /// Revocation epoch the decision was made under.
    pub decided_at_epoch: i64,
    /// Capability set version the decision was made under.
    pub capability_set_version: i64,
}

/// Evaluate the six-step decision order for one object access.
pub fn authorize(
    snapshot: &AuthzSnapshot,
    object: &ObjectGovernance,
    request: &AccessRequest,
) -> Result<AuthorizationGrant, DeniedAccess> {
    // Step 1: authentication and chain resolution.
    if !snapshot.principal.authenticated
        || !snapshot.principal.active
        || !snapshot.actor_chain.resolved
        || snapshot.actor_chain.chain_digest.is_empty()
    {
        return Err(denial(
            CONTEXT_AUTH_DENIED,
            DecisionStage::Authentication,
            snapshot,
            request,
        ));
    }

    // Step 2: tenant and membership. Cross-tenant references fail closed
    // (REQ-SEC-001); platform objects are out of tenant reach here.
    let same_tenant = object.tenant_id.as_deref() == Some(snapshot.tenant_id.as_str())
        && snapshot.principal.tenant_id.as_deref() == Some(snapshot.tenant_id.as_str());
    let membership_valid = snapshot
        .membership
        .as_ref()
        .is_some_and(|membership| membership.valid);
    if !same_tenant || !membership_valid {
        return Err(denial(
            CONTEXT_AUTH_DENIED,
            DecisionStage::TenantMembership,
            snapshot,
            request,
        ));
    }
    // Tenant match and membership alone grant NOTHING: fall through to the
    // capability steps (`membership_alone_grants_read: false`).

    // Step 3: capability chain intersection + revocation currency. A chain
    // whose stalest link predates the current revocation epoch is not
    // current material; deciding from it would be the cached-material
    // defect of REQ-CAP-005. An explicit ShareGrant from the scope
    // authority enters as a regular capability link — grants are governed
    // objects, not a bypass of this arithmetic.
    let effective = intersect_chain(&snapshot.capability_links);
    if effective.is_empty() || effective.oldest_issued_epoch < snapshot.revocation_epoch {
        return Err(denial(
            CONTEXT_AUTH_DENIED,
            DecisionStage::CapabilityIntersection,
            snapshot,
            request,
        ));
    }

    // Step 4: explicit deny beats any allow.
    for deny in &snapshot.explicit_denies {
        let action_covered = deny.actions.is_empty() || deny.actions.contains(&request.action);
        if action_covered && resource_within(&object.resource_scope, &deny.resource_prefix) {
            return Err(denial(
                CONTEXT_AUTH_DENIED,
                DecisionStage::ExplicitDeny,
                snapshot,
                request,
            ));
        }
    }

    // Step 5: lease validity at the decision instant.
    let lease_ok = effective
        .lease
        .as_ref()
        .is_some_and(|window| window.contains(&snapshot.decided_at));
    if !lease_ok {
        return Err(denial(
            AUTH_CAPABILITY_EXPIRED,
            DecisionStage::Lease,
            snapshot,
            request,
        ));
    }

    // Step 6: scope, purpose and action binding against the object.
    let scope_ok = effective
        .resource
        .as_ref()
        .is_some_and(|resource| resource_within(&object.resource_scope, resource));
    let purpose_ok = effective
        .purpose
        .as_ref()
        .is_some_and(|purpose| *purpose == request.purpose);
    let action_ok = effective.actions.contains(&request.action);
    if !scope_ok || !purpose_ok || !action_ok {
        return Err(denial(
            CONTEXT_AUTH_DENIED,
            DecisionStage::ScopePurposeBinding,
            snapshot,
            request,
        ));
    }

    Ok(AuthorizationGrant {
        effective,
        decided_at_epoch: snapshot.revocation_epoch,
        capability_set_version: snapshot.capability_set_version,
    })
}

/// Protected read gate with denial/not-found isomorphism: `None` (no such
/// object) and a denied existing object produce byte-identical denials —
/// same code, same shape, same fixed detail (error-contract section 5,
/// vector `tenant-lateral-read-denial.json`).
pub fn protected_read<'a, T>(
    snapshot: &AuthzSnapshot,
    target: Option<(&ObjectGovernance, &'a T)>,
    request: &AccessRequest,
) -> Result<&'a T, DeniedAccess> {
    match target {
        None => Err(denial(
            CONTEXT_AUTH_DENIED,
            DecisionStage::ScopePurposeBinding,
            snapshot,
            request,
        )),
        Some((object, body)) => {
            authorize(snapshot, object, request)?;
            Ok(body)
        }
    }
}

/// Revalidate a previously issued grant against the CURRENT governance
/// snapshot (F-007 race points: after resolution before dispatch, and
/// after dispatch before commit). A grant decided under an older
/// revocation epoch or capability set version is stale material and MUST
/// NOT authorize dispatch or commit.
pub fn revalidate_grant(
    grant: &AuthorizationGrant,
    current_revocation_epoch: i64,
    current_capability_set_version: i64,
    now: &WallTimestamp,
) -> Result<(), AccessDenial> {
    if grant.decided_at_epoch != current_revocation_epoch
        || grant.capability_set_version != current_capability_set_version
    {
        return Err(AccessDenial {
            code: CONTEXT_AUTH_DENIED.code,
            category: CONTEXT_AUTH_DENIED.category,
            retryable: CONTEXT_AUTH_DENIED.retryable,
            detail: DENIAL_DETAIL,
        });
    }
    let lease_ok = grant
        .effective
        .lease
        .as_ref()
        .is_some_and(|window| window.contains(now));
    if !lease_ok {
        return Err(AccessDenial {
            code: AUTH_CAPABILITY_EXPIRED.code,
            category: AUTH_CAPABILITY_EXPIRED.category,
            retryable: AUTH_CAPABILITY_EXPIRED.retryable,
            detail: DENIAL_DETAIL,
        });
    }
    Ok(())
}

/// Compute the engine guard attestation `capability_and_revocation_current`
/// deterministically from a revalidation result. This is the ONLY sanctioned
/// way to attest that guard (effect transition guards,
/// `specs/transitions/effect.transitions.json`).
pub fn capability_and_revocation_current(
    grant: &AuthorizationGrant,
    current_revocation_epoch: i64,
    current_capability_set_version: i64,
    now: &WallTimestamp,
) -> bool {
    revalidate_grant(
        grant,
        current_revocation_epoch,
        current_capability_set_version,
        now,
    )
    .is_ok()
}
