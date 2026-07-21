//! Recovery steps 6/7 consumption for the runtime (M5 RUN batch 2b).
//!
//! Kernel delivers durable facts via [`RecoveryReport`]. This module never
//! invents grants or context bindings — it only decides which obligations
//! may continue under current governance and whether a declared binding
//! matches the rebinding epoch.

use cognitive_domain::WallTimestamp;
use cognitive_kernel::authz::AuthorizationGrant;
use cognitive_kernel::effects::GovernanceCurrency;
use cognitive_kernel::recovery::{
    ContextRebinding, ReauthorizationObligation, RecoveryReport, reauthorization_satisfied,
};

/// One obligation after runtime step-6 evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObligationDecision {
    pub effect_object_id: String,
    pub idempotency_key: String,
    pub satisfied: bool,
}

/// Aggregate step-6/7 admission for post-recovery continuations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryContinuationPlan {
    pub obligations: Vec<ObligationDecision>,
    pub context_rebinding: ContextRebinding,
    /// True when a caller-declared binding epoch equals `new_epoch`.
    pub declared_binding_current: bool,
}

/// Evaluate which reauthorization obligations are satisfied by the
/// corresponding fresh grant (same index). Stale grants fail closed.
pub fn plan_recovery_continuations(
    report: &RecoveryReport,
    fresh_grants: &[AuthorizationGrant],
    currency: &GovernanceCurrency,
    now: &WallTimestamp,
    declared_binding_epoch: Option<i64>,
) -> RecoveryContinuationPlan {
    let obligations = report
        .reauthorization_obligations
        .iter()
        .enumerate()
        .map(|(idx, obligation)| {
            decide_obligation(obligation, fresh_grants.get(idx), currency, now)
        })
        .collect();
    let declared_binding_current = declared_binding_epoch
        .map(|epoch| epoch == report.context_rebinding.new_epoch)
        .unwrap_or(false);
    RecoveryContinuationPlan {
        obligations,
        context_rebinding: report.context_rebinding,
        declared_binding_current,
    }
}

fn decide_obligation(
    obligation: &ReauthorizationObligation,
    grant: Option<&AuthorizationGrant>,
    currency: &GovernanceCurrency,
    now: &WallTimestamp,
) -> ObligationDecision {
    let satisfied = grant
        .map(|g| reauthorization_satisfied(obligation, g, currency, now))
        .unwrap_or(false);
    ObligationDecision {
        effect_object_id: obligation.effect_object_id.as_str().to_owned(),
        idempotency_key: obligation.idempotency_key.clone(),
        satisfied,
    }
}

/// Pure helper: a pre-crash binding is never current after rebinding.
pub fn pre_crash_binding_is_stale(rebinding: &ContextRebinding, declared_epoch: i64) -> bool {
    declared_epoch == rebinding.fenced_epoch || declared_epoch < rebinding.new_epoch
}
