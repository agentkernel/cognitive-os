//! Adaptive Context fragment budgets — private MVP (P8-T05/D02).
//!
//! Daemon-owned budget adjustment consumes durable telemetry facts only and
//! never authorizes skipping body reauthorization. Compaction benefit
//! observation lives in `compaction_benefit`.

use thiserror::Error;

/// Durable telemetry snapshot used to adapt fragment budgets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptiveBudgetTelemetry {
    pub observed_fragment_tokens: u64,
    pub observed_loss_events: u64,
    pub body_reauthorization_required: bool,
}

/// Adaptive budget decision for one Context assembly step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptiveBudgetDecision {
    pub fragment_budget: u64,
    pub body_reauthorization_required: bool,
}

/// Fail-closed adaptive budget errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AdaptiveBudgetError {
    #[error("adaptive budget refuses to skip body reauthorization")]
    ReauthorizationRequired,
    #[error("adaptive budget ceiling would become unbounded")]
    UnboundedBudget,
}

/// Adapt a fragment budget from durable telemetry without skipping reauth.
pub fn adapt_fragment_budget(
    baseline_budget: u64,
    ceiling: u64,
    telemetry: &AdaptiveBudgetTelemetry,
) -> Result<AdaptiveBudgetDecision, AdaptiveBudgetError> {
    if !telemetry.body_reauthorization_required {
        return Err(AdaptiveBudgetError::ReauthorizationRequired);
    }
    if ceiling == 0 || baseline_budget == 0 {
        return Err(AdaptiveBudgetError::UnboundedBudget);
    }

    let mut adapted = baseline_budget;
    if telemetry.observed_fragment_tokens > baseline_budget {
        adapted = adapted.saturating_add(telemetry.observed_loss_events.saturating_add(1));
    }
    if adapted > ceiling {
        return Err(AdaptiveBudgetError::UnboundedBudget);
    }

    Ok(AdaptiveBudgetDecision {
        fragment_budget: adapted,
        body_reauthorization_required: true,
    })
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn adapts_within_ceiling_while_keeping_reauthorization() {
        let decision = adapt_fragment_budget(
            8,
            16,
            &AdaptiveBudgetTelemetry {
                observed_fragment_tokens: 10,
                observed_loss_events: 2,
                body_reauthorization_required: true,
            },
        )
        .expect("adapt");
        assert_eq!(decision.fragment_budget, 11);
        assert!(decision.body_reauthorization_required);
    }

    #[test]
    fn rejects_skip_reauth_and_unbounded_ceiling() {
        assert_eq!(
            adapt_fragment_budget(
                8,
                16,
                &AdaptiveBudgetTelemetry {
                    observed_fragment_tokens: 10,
                    observed_loss_events: 0,
                    body_reauthorization_required: false,
                },
            )
            .unwrap_err(),
            AdaptiveBudgetError::ReauthorizationRequired
        );
        assert_eq!(
            adapt_fragment_budget(
                8,
                9,
                &AdaptiveBudgetTelemetry {
                    observed_fragment_tokens: 20,
                    observed_loss_events: 5,
                    body_reauthorization_required: true,
                },
            )
            .unwrap_err(),
            AdaptiveBudgetError::UnboundedBudget
        );
    }
}
