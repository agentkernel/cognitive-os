//! `cognitive-kernel`: deterministic governance core of the CognitiveOS
//! reference implementation.
//!
//! Scope (M2-M4, per `docs/plan/DEVELOPMENT-PLAN.md`): authority and CAS,
//! capability intersection and monotone attenuation, hard budgets, the
//! deterministic Context gate, the Intent/Effect protocol with stable
//! idempotency keys, checkpointing, and the eight-step recovery order of
//! whitepaper section 16.6.
//!
//! Hard rule: authorization, CAS, state transitions, budgets, idempotency,
//! fencing and final commits are executed by deterministic code in this
//! crate. LLMs, retrieval and rankers only ever produce candidates or
//! proposals upstream. This crate defines port traits; adapters implement
//! them (`cognitive-store`, `cognitive-runtime`).

/// Port trait families this crate will define (named after ADR-0001 and the
/// whitepaper adapter families). Placeholder list until M2 lands the traits.
pub const KERNEL_PORTS: [&str; 5] = ["object-store", "event-log", "outbox", "clock", "checkpoint"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernel_sits_above_domain_only() {
        assert!(cognitive_domain::EXECUTION_LIFECYCLE_DOMAINS.contains(&"effect"));
        assert_eq!(KERNEL_PORTS.len(), 5);
    }
}
