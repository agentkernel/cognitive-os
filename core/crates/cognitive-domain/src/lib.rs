//! `cognitive-domain`: pure domain layer of the CognitiveOS reference
//! implementation.
//!
//! Scope (M2, per `docs/plan/DEVELOPMENT-PLAN.md`): the five execution
//! lifecycle state machines consumed from `specs/transitions/*.json`
//! (embedded registered assets, digest-pinned), logical-version CAS rules,
//! and validated identifier newtypes. This crate performs no I/O and never
//! depends on HTTP, SQLite, or model SDKs.
//!
//! Normative sources: `docs/standards/state-and-transition-contract.md`,
//! `specs/registry/state-domains.yaml`, `specs/transitions/`, ADR-0005.
//! REQ coverage: REQ-STATE-001/002/003, REQ-GOBJ-ID-001 (format layer),
//! REQ-CAP-001/002 (capability constraint arithmetic,
//! `docs/standards/authn-authz-capability.md`).

pub mod capability;
pub mod error;
pub mod ids;
pub mod transitions;
pub mod version;

pub use capability::{
    CapabilityConstraints, EffectiveRights, LeaseWindow, ParameterBound, attenuation_violations,
    intersect_chain, resource_within,
};
pub use error::DomainError;
pub use ids::{
    BudgetId, EventId, ObjectId, ReasonCode, RecordId, StateName, TimestampInstant, UriRef,
    WallTimestamp,
};
pub use transitions::{
    EdgeLookupError, LifecycleDomain, LoadedTable, TableAssetError, TransitionEdge,
    TransitionTable, table,
};
pub use version::Version;

/// The five registered execution lifecycle state domain names, exactly
/// matching `specs/transitions/<domain>.transitions.json`. The registry
/// keeps the domain set open (REQ-STATE-001); these five are the v0.1
/// execution core and must never be merged into one machine.
pub const EXECUTION_LIFECYCLE_DOMAINS: [&str; 5] =
    ["agent-execution", "effect", "loop", "task", "verification"];

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_domains_are_sorted_unique_and_match_transition_tables() {
        let mut sorted = EXECUTION_LIFECYCLE_DOMAINS;
        sorted.sort_unstable();
        assert_eq!(sorted, EXECUTION_LIFECYCLE_DOMAINS, "keep the list sorted");
        for pair in EXECUTION_LIFECYCLE_DOMAINS.windows(2) {
            assert_ne!(pair[0], pair[1], "domains must be unique");
        }
        for (name, domain) in EXECUTION_LIFECYCLE_DOMAINS.iter().zip(LifecycleDomain::ALL) {
            assert_eq!(*name, domain.as_str());
            assert_eq!(LifecycleDomain::parse(name).unwrap(), domain);
        }
        assert!(LifecycleDomain::parse("world").is_err());
    }

    #[test]
    fn depends_only_on_contracts_layer() {
        // Compile-time witness of the allowed dependency direction.
        assert_eq!(
            cognitive_contracts::ENCODING_PROFILE,
            "cognitiveos.canonical-json/0.1"
        );
    }
}
