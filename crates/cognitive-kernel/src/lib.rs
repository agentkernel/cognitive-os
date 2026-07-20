//! `cognitive-kernel`: deterministic governance core of the CognitiveOS
//! reference implementation.
//!
//! M2 scope (per `docs/plan/DEVELOPMENT-PLAN.md`): the centralized
//! transition gate over the five registered lifecycle tables (CAS, guard
//! and evidence verification, registered-error rejection), deterministic
//! hard-budget metering, projection replay over the committed event
//! history, and the port traits adapters implement (`cognitive-store`).
//! M3/M4 add capabilities, the Context gate, Intent/Effect and recovery.
//!
//! Hard rule (architecture invariant): authorization, CAS, state
//! transitions, budgets, idempotency, fencing and final commits are
//! executed by deterministic code in this crate. LLMs, retrieval and
//! rankers only ever produce candidates or proposals upstream; nothing in
//! this crate calls a probabilistic component.
//!
//! REQ coverage: REQ-STATE-001/002/003, REQ-EVT-002/004 (port contract),
//! REQ-REC-003 (fail-closed mapping). Registered code mapping: [`error`].

pub mod budget;
pub mod engine;
pub mod error;
pub mod ports;
pub mod replay;

pub use budget::{BudgetCharge, BudgetError, BudgetExhausted, BudgetState};
pub use engine::{
    AdmitCommand, AdmittedObject, BudgetChargeCommand, Causation, CommittedTransition, Reason,
    TablePin, TransitionCommand, TransitionEngine,
};
pub use error::{RegisteredError, RejectionKind, TransitionRejection};
pub use ports::{AuthorityStore, Clock, IdGenerator, PortFailure, StorePortError};
pub use replay::{ReplayError, ReplayedProjection, replay_projection};

/// Port capability surface defined by this crate and implemented by
/// adapters. `event-log` and `outbox` are capabilities of the
/// [`ports::AuthorityStore`] trait (they share its atomic commit unit and
/// must never be split into independently committing stores); `clock` and
/// `id-generator` are standalone port traits.
pub const KERNEL_PORTS: [&str; 5] = [
    "authority-store",
    "event-log",
    "outbox",
    "clock",
    "id-generator",
];

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn kernel_sits_above_domain_only() {
        assert!(cognitive_domain::EXECUTION_LIFECYCLE_DOMAINS.contains(&"effect"));
        assert_eq!(KERNEL_PORTS.len(), 5);
        assert!(KERNEL_PORTS.contains(&"authority-store"));
    }

    #[test]
    fn table_pins_are_available_for_all_five_domains() {
        for domain in cognitive_domain::LifecycleDomain::ALL {
            let pin = TablePin::current(domain).unwrap();
            assert!(pin.digest.starts_with("sha256:"));
            assert!(!pin.version.is_empty());
        }
    }
}
