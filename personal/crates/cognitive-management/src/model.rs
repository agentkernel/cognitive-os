//! Experimental model seam of the management plane.
//!
//! The Intelligent Management Shell (whitepaper §12; profile
//! `intelligent_management_shell`) stays EXPERIMENTAL: it may one day
//! propose management actions through this trait, but the deterministic
//! management, recovery and stop paths MUST NEVER call it
//! (REQ-MGMT-FALLBACK-001, vector `management-deterministic-fallback.json`).
//! The trait exists so tests can wire a counting/panicking probe into the
//! slot and prove the deterministic verbs perform zero model calls.

use cognitive_kernel::ports::PortFailure;

/// A probabilistic completion provider (LLM). Anything it returns is at
/// most a PROPOSAL; it can never authorize, commit, transition state or
/// decide completion (architecture invariant: five-duty separation).
pub trait ModelProvider {
    /// Produce a completion for `prompt`. Deterministic management paths
    /// MUST NOT call this; only the experimental shell (not part of this
    /// batch) may, and only to produce candidates.
    fn complete(&self, prompt: &str) -> Result<String, PortFailure>;
}
