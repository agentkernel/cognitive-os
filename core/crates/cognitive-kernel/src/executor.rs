//! Executor port: the boundary to external effect sinks
//! (`docs/standards/intent-effect-idempotency.md`; REQ-EFF-002/003).
//!
//! An executor is an ADAPTER to an external system. The kernel treats it as
//! untrusted for governance purposes: every call carries the stable
//! idempotency key, the canonical parameter digest, the authorization
//! digest and the current fencing epoch (REQ-EFF-002), and nothing the
//! executor returns is ever acceptance — a receipt is execution evidence
//! only (REQ-EFF-003).

use crate::ports::PortFailure;

/// Capability self-description of an executor (F-023 admission matrix
/// input): can its outcomes be queried after the fact, and is dispatch
/// idempotent under the same key?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutorCapabilities {
    /// Outcomes are queryable by idempotency key after dispatch.
    pub queryable: bool,
    /// Re-dispatch with the same idempotency key is absorbed exactly-once.
    pub idempotent: bool,
}

/// One dispatch call (REQ-EFF-002 required fields).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutorCall {
    /// Operation action name.
    pub action: String,
    /// Target URI.
    pub target: String,
    /// Stable idempotency key (never re-minted across retries).
    pub idempotency_key: String,
    /// Canonical parameter digest.
    pub parameters_digest: String,
    /// Digest of the authorization decision this dispatch rides on.
    pub authorization_digest: String,
    /// Fencing epoch of the dispatching writer; stale epochs MUST be
    /// rejected by the sink (F-014).
    pub fencing_epoch: i64,
}

/// Externally observed dispatch outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchOutcome {
    /// The sink confirmed execution and returned a receipt reference.
    Executed {
        /// Receipt reference (evidence only, never acceptance).
        receipt_ref: String,
    },
    /// The sink authoritatively confirmed non-execution.
    NotExecuted {
        /// Non-execution evidence reference.
        reason: String,
    },
    /// Timeout, lost connection, or missing receipt: execution MAY have
    /// occurred. This is a first-class uncertain outcome, never an error
    /// to retry blindly (REQ-EFF-004).
    Unknown {
        /// Uncertainty description.
        detail: String,
    },
    /// The sink rejected the dispatch because the fencing epoch was stale
    /// (F-014 sink-side enforcement).
    FencedStaleEpoch {
        /// Epoch the sink currently trusts.
        sink_epoch: i64,
    },
}

/// Result of an after-the-fact outcome query (queryable executors only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutorQueryResult {
    /// The external system holds a record for the original key.
    ExecutedWithOriginalKey,
    /// The external system authoritatively reports no execution.
    NotExecuted,
    /// The external system cannot determine the outcome.
    Indeterminate,
}

/// The executor port. Implementations are adapters (fakes in tests, real
/// protocol adapters from M5/M6); the kernel never trusts them with
/// authority decisions.
pub trait EffectExecutor {
    /// Declared capabilities (admission matrix input, F-023).
    fn capabilities(&self) -> ExecutorCapabilities;

    /// Dispatch one call to the external sink.
    fn dispatch(&self, call: &ExecutorCall) -> Result<DispatchOutcome, PortFailure>;

    /// Query the outcome for an idempotency key. Non-queryable executors
    /// return `Indeterminate` (they cannot help reconciliation).
    fn query_outcome(&self, idempotency_key: &str) -> Result<ExecutorQueryResult, PortFailure>;
}
