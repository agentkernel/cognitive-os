//! Registered machine error codes consumed by the M2 kernel gate, and the
//! single mapping from typed rejection kinds to those codes.
//!
//! Code discipline (`docs/standards/error-contract.md`): a governed failure
//! surfaces exactly one code registered in `specs/registry/errors.yaml`;
//! codes are never invented, reused for a different meaning, or collapsed.
//! The mapping below is the one place where kernel rejection kinds meet
//! registered codes:
//!
//! - Version/state CAS mismatch -> `STATE_CONFLICT`
//!   (`state-and-transition-contract.md` section 3: "Version mismatch MUST
//!   fail as STATE_CONFLICT"; vector `state-conflict.json`).
//! - Illegal transition (no matching row, disallowed reason, terminal
//!   source, unknown state, unsatisfied guard, missing evidence, stale
//!   `from`) -> `STATE_CONFLICT` (`.cursor/rules/10-rust-kernel.mdc`:
//!   registered code for illegal transitions; rejection carries the current
//!   state/version and safe exits per contract section 3).
//! - Illegal exit attempted from Effect `OUTCOME_UNKNOWN` ->
//!   `EFFECT_OUTCOME_UNKNOWN` (vector `effect-state-closure-008.json`
//!   expected deny code; `OUTCOME_UNKNOWN` is a first-class state whose only
//!   legal continuation is reconciliation, `.cursor/rules/13-effect-recovery.mdc`).
//! - Transition-table pin mismatch or corrupted table asset ->
//!   `DIGEST_MISMATCH` (canonical standard section 15 fail-closed digest
//!   verification; a consumer must pin the table digest it decides under).
//! - Hard budget exceeded -> `RESOURCE_BUDGET_EXHAUSTED` (deterministic
//!   hard limit, fail-closed, not retryable).
//! - Authoritative commit path cannot persist -> `STATE_STORE_UNAVAILABLE`
//!   (REQ-REC-003; ADR-0002 binding rule 4; vector
//!   `state-store-degradation.json`).

use cognitive_domain::{StateName, Version};

/// One registered error code with its registry-declared category and
/// retryability (`specs/registry/errors.yaml`; a unit test pins these
/// triples against the registry file).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisteredError {
    /// Registered machine code.
    pub code: &'static str,
    /// Registered category.
    pub category: &'static str,
    /// Registered retryability contract.
    pub retryable: bool,
}

/// `STATE_CONFLICT`: expected state version differs from authoritative
/// version (also the registered code for illegal-transition rejection).
pub const STATE_CONFLICT: RegisteredError = RegisteredError {
    code: "STATE_CONFLICT",
    category: "state",
    retryable: true,
};

/// `STATE_STORE_UNAVAILABLE`: the authoritative commit path cannot persist
/// state or events; governed writes fail closed instead of buffering.
pub const STATE_STORE_UNAVAILABLE: RegisteredError = RegisteredError {
    code: "STATE_STORE_UNAVAILABLE",
    category: "state",
    retryable: true,
};

/// `EFFECT_OUTCOME_UNKNOWN`: execution may have occurred and requires
/// reconciliation or quarantine.
pub const EFFECT_OUTCOME_UNKNOWN: RegisteredError = RegisteredError {
    code: "EFFECT_OUTCOME_UNKNOWN",
    category: "effect",
    retryable: true,
};

/// `RESOURCE_BUDGET_EXHAUSTED`: a deterministic hard resource limit was
/// reached.
pub const RESOURCE_BUDGET_EXHAUSTED: RegisteredError = RegisteredError {
    code: "RESOURCE_BUDGET_EXHAUSTED",
    category: "resource",
    retryable: false,
};

/// `DIGEST_MISMATCH`: recomputed canonical digest differs from the pinned
/// or declared digest.
pub const DIGEST_MISMATCH: RegisteredError = RegisteredError {
    code: "DIGEST_MISMATCH",
    category: "protocol",
    retryable: false,
};

/// `CONTEXT_AUTH_DENIED`: context item is not authorized for principal and
/// purpose (also the isomorphic protected-read denial shape, M3 gate).
pub const CONTEXT_AUTH_DENIED: RegisteredError = RegisteredError {
    code: "CONTEXT_AUTH_DENIED",
    category: "auth",
    retryable: false,
};

/// `CONTEXT_INCOMPLETE`: required context set cannot be closed.
pub const CONTEXT_INCOMPLETE: RegisteredError = RegisteredError {
    code: "CONTEXT_INCOMPLETE",
    category: "context",
    retryable: true,
};

/// `CONTEXT_BUDGET_EXCEEDED`: hard budget cannot contain required context.
pub const CONTEXT_BUDGET_EXCEEDED: RegisteredError = RegisteredError {
    code: "CONTEXT_BUDGET_EXCEEDED",
    category: "context",
    retryable: false,
};

/// `AUTH_CAPABILITY_ATTENUATION_VIOLATION`: derived capability expands a
/// parent bound.
pub const AUTH_CAPABILITY_ATTENUATION_VIOLATION: RegisteredError = RegisteredError {
    code: "AUTH_CAPABILITY_ATTENUATION_VIOLATION",
    category: "auth",
    retryable: false,
};

/// `AUTH_CAPABILITY_EXPIRED`: capability lease is not currently valid.
pub const AUTH_CAPABILITY_EXPIRED: RegisteredError = RegisteredError {
    code: "AUTH_CAPABILITY_EXPIRED",
    category: "auth",
    retryable: true,
};

/// `CONTEXT_RESOLUTION_STAGNATED`: bounded resolution attempts made no
/// admissible information gain.
pub const CONTEXT_RESOLUTION_STAGNATED: RegisteredError = RegisteredError {
    code: "CONTEXT_RESOLUTION_STAGNATED",
    category: "discovery",
    retryable: false,
};

/// `EFFECT_IDEMPOTENCY_CONFLICT`: an idempotency key was reused with
/// different parameters; the request must be rejected, not deduplicated or
/// executed.
pub const EFFECT_IDEMPOTENCY_CONFLICT: RegisteredError = RegisteredError {
    code: "EFFECT_IDEMPOTENCY_CONFLICT",
    category: "effect",
    retryable: false,
};

/// `EFFECT_RECOVERY_QUARANTINED`: recovery cannot safely determine or
/// compensate outcome.
pub const EFFECT_RECOVERY_QUARANTINED: RegisteredError = RegisteredError {
    code: "EFFECT_RECOVERY_QUARANTINED",
    category: "effect",
    retryable: false,
};

/// `NO_AUTHORIZED_OPERATION_CANDIDATE`: no operation candidate is both
/// visible and authorized (M4 admission matrix: an operation whose executor
/// is neither queryable nor idempotent has no safe recovery closure and is
/// not an admissible candidate for governed_external dispatch, F-023).
pub const NO_AUTHORIZED_OPERATION_CANDIDATE: RegisteredError = RegisteredError {
    code: "NO_AUTHORIZED_OPERATION_CANDIDATE",
    category: "catalog",
    retryable: false,
};

/// `INTENT_CLARIFICATION_REQUIRED`: material intent ambiguity must be
/// resolved by the intent authority (M5 deterministic admission gate:
/// a candidate carrying a material ambiguity is never promoted top-1,
/// REQ-INTENT-ADMISSION-001).
pub const INTENT_CLARIFICATION_REQUIRED: RegisteredError = RegisteredError {
    code: "INTENT_CLARIFICATION_REQUIRED",
    category: "intent",
    retryable: true,
};

/// `INTENT_VERSION_SUPERSEDED`: proposal references a superseded intent or
/// TaskContract epoch (M5 correction fencing: dispatches bound to an old
/// contract epoch are rejected, REQ-INTENT-SUPERSEDE-001).
pub const INTENT_VERSION_SUPERSEDED: RegisteredError = RegisteredError {
    code: "INTENT_VERSION_SUPERSEDED",
    category: "intent",
    retryable: true,
};

/// Typed cause of a rejected governed operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectionKind {
    /// The request did not pin the loaded table version and digest.
    TablePinMismatch,
    /// The embedded registered table asset failed to parse or validate
    /// (fail-closed recovery barrier, never a caller error).
    TableAssetInvalid,
    /// No governed object with the requested identity exists.
    ObjectNotFound,
    /// The object belongs to a different lifecycle domain.
    DomainMismatch,
    /// Request `from` differs from the authoritative current state.
    FromStateMismatch,
    /// Request `expected_version` differs from the authoritative version.
    VersionMismatch,
    /// `from` or `to` is not a state of the pinned table version.
    UnknownState,
    /// `from` is terminal: no legal outgoing transition exists.
    TerminalState,
    /// No transition row matches the requested `(from, to)` pair.
    IllegalTransition,
    /// A row matches the state pair but not the requested reason.
    ReasonNotAllowed,
    /// A guard of the selected row is not deterministically established.
    GuardUnsatisfied,
    /// A required evidence item is absent.
    EvidenceMissing,
    /// The referenced hard-budget ledger row does not exist.
    BudgetNotFound,
    /// The deterministic hard budget cannot admit the charge.
    BudgetExhausted,
    /// The store lost the commit race (CAS applied zero rows).
    StoreConflict,
    /// The authoritative commit path cannot persist (fail closed, no
    /// in-memory buffering of governed writes).
    StoreUnavailable,
    /// The command is internally invalid (for example version overflow).
    InvalidCommand,
}

impl RejectionKind {
    /// The registered code this kind surfaces when the rejected subject is
    /// NOT an Effect sitting in `OUTCOME_UNKNOWN` (see
    /// [`TransitionRejection::registered`] for that special case).
    fn base_error(&self) -> RegisteredError {
        match self {
            RejectionKind::TablePinMismatch | RejectionKind::TableAssetInvalid => DIGEST_MISMATCH,
            RejectionKind::StoreUnavailable => STATE_STORE_UNAVAILABLE,
            RejectionKind::BudgetExhausted => RESOURCE_BUDGET_EXHAUSTED,
            RejectionKind::ObjectNotFound
            | RejectionKind::DomainMismatch
            | RejectionKind::FromStateMismatch
            | RejectionKind::VersionMismatch
            | RejectionKind::UnknownState
            | RejectionKind::TerminalState
            | RejectionKind::IllegalTransition
            | RejectionKind::ReasonNotAllowed
            | RejectionKind::GuardUnsatisfied
            | RejectionKind::EvidenceMissing
            | RejectionKind::BudgetNotFound
            | RejectionKind::StoreConflict
            | RejectionKind::InvalidCommand => STATE_CONFLICT,
        }
    }
}

/// A rejected governed operation: fail closed, no side effects, current
/// authoritative state/version and safe available exits attached
/// (`state-and-transition-contract.md` section 3).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{detail} (code {})", self.registered().code)]
pub struct TransitionRejection {
    /// Typed cause.
    pub kind: RejectionKind,
    /// Human-readable detail (never a machine code).
    pub detail: String,
    /// Authoritative current state at decision time, when known.
    pub current_state: Option<StateName>,
    /// Authoritative current version at decision time, when known.
    pub current_version: Option<Version>,
    /// Safe available exits from the current state (sorted, distinct).
    pub available_exits: Vec<String>,
    /// True when the rejected subject is an `effect` object whose
    /// authoritative current state is `OUTCOME_UNKNOWN`.
    pub effect_outcome_unknown: bool,
}

impl TransitionRejection {
    /// The registered error surfaced by this rejection.
    ///
    /// Special case pinned by vector `effect-state-closure-008.json`: an
    /// illegal exit attempted from Effect `OUTCOME_UNKNOWN` surfaces
    /// `EFFECT_OUTCOME_UNKNOWN` — the request failed precisely because the
    /// outcome is unknown and only reconciliation may continue.
    pub fn registered(&self) -> RegisteredError {
        if self.effect_outcome_unknown
            && matches!(
                self.kind,
                RejectionKind::IllegalTransition | RejectionKind::ReasonNotAllowed
            )
        {
            return EFFECT_OUTCOME_UNKNOWN;
        }
        self.kind.base_error()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    /// Pin every code constant against `specs/registry/errors.yaml`:
    /// the code exists and its registered category/retryable match.
    #[test]
    fn code_constants_match_the_registry() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("specs")
            .join("registry")
            .join("errors.yaml");
        let registry = std::fs::read_to_string(path).unwrap();
        for expected in [
            STATE_CONFLICT,
            STATE_STORE_UNAVAILABLE,
            EFFECT_OUTCOME_UNKNOWN,
            RESOURCE_BUDGET_EXHAUSTED,
            DIGEST_MISMATCH,
            CONTEXT_AUTH_DENIED,
            CONTEXT_INCOMPLETE,
            CONTEXT_BUDGET_EXCEEDED,
            AUTH_CAPABILITY_ATTENUATION_VIOLATION,
            AUTH_CAPABILITY_EXPIRED,
            CONTEXT_RESOLUTION_STAGNATED,
            EFFECT_IDEMPOTENCY_CONFLICT,
            EFFECT_RECOVERY_QUARANTINED,
            NO_AUTHORIZED_OPERATION_CANDIDATE,
            INTENT_CLARIFICATION_REQUIRED,
            INTENT_VERSION_SUPERSEDED,
        ] {
            let marker = format!("- code: {}", expected.code);
            let start = registry
                .find(&marker)
                .unwrap_or_else(|| panic!("{} not registered", expected.code));
            let entry: String = registry[start..]
                .lines()
                .take(4)
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                entry.contains(&format!("category: {}", expected.category)),
                "{} category drifted: {entry}",
                expected.code
            );
            assert!(
                entry.contains(&format!("retryable: {}", expected.retryable)),
                "{} retryable drifted: {entry}",
                expected.code
            );
        }
    }

    #[test]
    fn outcome_unknown_special_case_only_covers_illegal_exits() {
        let rejection = |kind: RejectionKind, unknown: bool| TransitionRejection {
            kind,
            detail: String::new(),
            current_state: None,
            current_version: None,
            available_exits: Vec::new(),
            effect_outcome_unknown: unknown,
        };
        assert_eq!(
            rejection(RejectionKind::IllegalTransition, true).registered(),
            EFFECT_OUTCOME_UNKNOWN
        );
        assert_eq!(
            rejection(RejectionKind::ReasonNotAllowed, true).registered(),
            EFFECT_OUTCOME_UNKNOWN
        );
        // A stale version race in OUTCOME_UNKNOWN stays STATE_CONFLICT.
        assert_eq!(
            rejection(RejectionKind::VersionMismatch, true).registered(),
            STATE_CONFLICT
        );
        assert_eq!(
            rejection(RejectionKind::IllegalTransition, false).registered(),
            STATE_CONFLICT
        );
        assert_eq!(
            rejection(RejectionKind::StoreUnavailable, true).registered(),
            STATE_STORE_UNAVAILABLE
        );
        assert_eq!(
            rejection(RejectionKind::BudgetExhausted, false).registered(),
            RESOURCE_BUDGET_EXHAUSTED
        );
        assert_eq!(
            rejection(RejectionKind::TablePinMismatch, false).registered(),
            DIGEST_MISMATCH
        );
    }
}
