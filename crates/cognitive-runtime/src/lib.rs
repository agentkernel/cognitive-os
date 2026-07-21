//! `cognitive-runtime`: execution layer of the CognitiveOS reference
//! implementation.
//!
//! Scope (M4-M6, per `docs/plan/DEVELOPMENT-PLAN.md`): the Operation
//! executor (OperationDescriptor is never an AuthorizationCapability),
//! sandbox and adapter ports for C0/C1 agent integration, and the bounded
//! Harness Loop with progress/stagnation judgment. Dispatch goes through the
//! kernel's Effect protocol; this crate never commits authority state
//! directly.

pub mod event_envelope;
pub use event_envelope::{EventEnvelopeError, assemble_event};
/// Placeholder marker for the executor pipeline (implemented from M4).
pub const RUNTIME_ROLE: &str = "operation-executor+harness-loop (planned, M4)";

#[cfg(test)]
mod tests {
    #[test]
    fn depends_on_kernel_layer() {
        assert!(cognitive_kernel::KERNEL_PORTS.contains(&"outbox"));
    }
}
