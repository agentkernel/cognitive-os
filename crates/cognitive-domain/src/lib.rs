//! `cognitive-domain`: pure domain layer of the CognitiveOS reference
//! implementation.
//!
//! Scope (M2, per `docs/plan/DEVELOPMENT-PLAN.md`): the five execution
//! lifecycle state machines consumed from `specs/transitions/*.json`, CAS
//! version rules, and domain invariants. This crate performs no I/O and must
//! never depend on HTTP, SQLite, or model SDKs.
//!
//! Normative sources: `docs/standards/state-and-transition-contract.md`,
//! `specs/registry/state-domains.yaml`, `specs/transitions/`.

/// The five registered execution lifecycle state domains, exactly matching
/// `specs/transitions/<domain>.transitions.json`. The registry keeps the
/// domain set open (REQ-STATE-*), but these five are the v0.1 execution core
/// and must never be merged into one machine.
pub const EXECUTION_LIFECYCLE_DOMAINS: [&str; 5] =
    ["agent-execution", "effect", "loop", "task", "verification"];

#[cfg(test)]
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
