//! Graded Skill/rule loading by context cost — private MVP (P8-T04/D03).
//!
//! Daemon-owned planner admits digest-bound Skill/rule candidates only within
//! a remaining context-cost budget. Overflow and undeclared candidates fail
//! closed. This path does not grant authority, complete Tasks, or bypass
//! admission.

use thiserror::Error;

/// Relative load preference for a digest-bound Skill/rule candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GradedLoadTier {
    /// Must load when declared; fails closed if budget cannot cover cost.
    Required,
    /// Load when remaining budget allows after required items.
    Opportunistic,
}

/// Digest-bound graded load candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GradedLoadCandidate {
    pub item_id: String,
    pub item_digest: String,
    pub cost_units: u64,
    pub tier: GradedLoadTier,
}

/// Deterministic load plan for one Context assembly step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GradedLoadPlan {
    pub admitted: Vec<GradedLoadCandidate>,
    pub deferred: Vec<GradedLoadCandidate>,
    pub remaining_budget: u64,
}

/// Fail-closed graded load errors.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum GradedLoadError {
    #[error("graded load candidate is missing identity or digest material")]
    UndeclaredCandidate,
    #[error("required graded load exceeds remaining context-cost budget")]
    BudgetOverflow,
}

/// Plan graded Skill/rule loads under a remaining context-cost budget.
pub fn plan_graded_context_load(
    candidates: &[GradedLoadCandidate],
    remaining_budget: u64,
) -> Result<GradedLoadPlan, GradedLoadError> {
    for candidate in candidates {
        if candidate.item_id.trim().is_empty() || candidate.item_digest.trim().is_empty() {
            return Err(GradedLoadError::UndeclaredCandidate);
        }
    }

    let mut required: Vec<&GradedLoadCandidate> = candidates
        .iter()
        .filter(|candidate| candidate.tier == GradedLoadTier::Required)
        .collect();
    required.sort_by(|left, right| {
        left.cost_units
            .cmp(&right.cost_units)
            .then_with(|| left.item_id.cmp(&right.item_id))
    });

    let mut opportunistic: Vec<&GradedLoadCandidate> = candidates
        .iter()
        .filter(|candidate| candidate.tier == GradedLoadTier::Opportunistic)
        .collect();
    opportunistic.sort_by(|left, right| {
        left.cost_units
            .cmp(&right.cost_units)
            .then_with(|| left.item_id.cmp(&right.item_id))
    });

    let mut remaining = remaining_budget;
    let mut admitted = Vec::new();
    for candidate in required {
        if candidate.cost_units > remaining {
            return Err(GradedLoadError::BudgetOverflow);
        }
        remaining = remaining.saturating_sub(candidate.cost_units);
        admitted.push(candidate.clone());
    }

    let mut deferred = Vec::new();
    for candidate in opportunistic {
        if candidate.cost_units <= remaining {
            remaining = remaining.saturating_sub(candidate.cost_units);
            admitted.push(candidate.clone());
        } else {
            deferred.push(candidate.clone());
        }
    }

    Ok(GradedLoadPlan {
        admitted,
        deferred,
        remaining_budget: remaining,
    })
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn candidate(id: &str, cost: u64, tier: GradedLoadTier) -> GradedLoadCandidate {
        GradedLoadCandidate {
            item_id: id.to_owned(),
            item_digest: format!("sha256:{}", "a".repeat(64)),
            cost_units: cost,
            tier,
        }
    }

    #[test]
    fn admits_required_then_opportunistic_within_budget() {
        let plan = plan_graded_context_load(
            &[
                candidate("skill.expensive", 5, GradedLoadTier::Opportunistic),
                candidate("rule.required", 2, GradedLoadTier::Required),
                candidate("skill.cheap", 1, GradedLoadTier::Opportunistic),
            ],
            4,
        )
        .expect("plan");
        assert_eq!(
            plan.admitted
                .iter()
                .map(|item| item.item_id.as_str())
                .collect::<Vec<_>>(),
            vec!["rule.required", "skill.cheap"]
        );
        assert_eq!(plan.deferred.len(), 1);
        assert_eq!(plan.deferred[0].item_id, "skill.expensive");
        assert_eq!(plan.remaining_budget, 1);
    }

    #[test]
    fn rejects_required_budget_overflow_and_undeclared() {
        assert_eq!(
            plan_graded_context_load(
                &[candidate("rule.required", 3, GradedLoadTier::Required)],
                2
            )
            .unwrap_err(),
            GradedLoadError::BudgetOverflow
        );

        let mut undeclared = candidate("rule.x", 1, GradedLoadTier::Required);
        undeclared.item_digest.clear();
        assert_eq!(
            plan_graded_context_load(&[undeclared], 10).unwrap_err(),
            GradedLoadError::UndeclaredCandidate
        );
    }
}
