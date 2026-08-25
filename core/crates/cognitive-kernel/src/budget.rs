//! Deterministic hard-budget metering primitive (M2 scope).
//!
//! A hard budget is a governed ledger row: named non-negative integer
//! dimensions (the AttentionBudget dimension set of
//! `common-defs.schema.json#/$defs/budget`) with authoritative remaining
//! amounts. Admission is pure integer arithmetic executed by deterministic
//! code — never by a model — and fails closed: a charge that any governed
//! dimension cannot cover is rejected with `RESOURCE_BUDGET_EXHAUSTED`
//! before anything is written. The debit commits in the same store
//! transaction as the state transition it admits.

use cognitive_contracts::canonical::MAX_SAFE_INTEGER;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Registered budget dimension names (`common-defs.schema.json` budget).
pub const BUDGET_DIMENSIONS: [&str; 9] = [
    "attention_slots",
    "context_bytes",
    "egress_bytes",
    "input_tokens",
    "money_microunits",
    "output_tokens",
    "semantic_calls",
    "tool_calls",
    "wall_time_ms",
];

/// Validation failures for budget states and charges.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BudgetError {
    /// Dimension name is not in the registered dimension set.
    #[error("unknown-budget-dimension: {0}")]
    UnknownDimension(String),
    /// Amount is negative or outside the I-JSON safe integer range.
    #[error("invalid-budget-amount: {dimension}={amount}")]
    InvalidAmount {
        /// Offending dimension.
        dimension: String,
        /// Offending amount.
        amount: i64,
    },
}

/// A deterministic denial: one governed dimension cannot cover the charge.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("budget-exhausted: {dimension} remaining {remaining} < charge {charge}")]
pub struct BudgetExhausted {
    /// First (lexicographically) dimension that cannot cover the charge.
    pub dimension: String,
    /// Remaining amount on that dimension.
    pub remaining: i64,
    /// Charge requested on that dimension.
    pub charge: i64,
}

fn validate_amounts(map: &BTreeMap<String, i64>) -> Result<(), BudgetError> {
    for (dimension, amount) in map {
        if !BUDGET_DIMENSIONS.contains(&dimension.as_str()) {
            return Err(BudgetError::UnknownDimension(dimension.clone()));
        }
        if !(0..=MAX_SAFE_INTEGER).contains(amount) {
            return Err(BudgetError::InvalidAmount {
                dimension: dimension.clone(),
                amount: *amount,
            });
        }
    }
    Ok(())
}

/// Authoritative remaining amounts of one hard budget. A dimension absent
/// from the map is not governed by this budget and admits any charge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BudgetState {
    remaining: BTreeMap<String, i64>,
}

impl BudgetState {
    /// Build a validated budget state.
    pub fn new(remaining: BTreeMap<String, i64>) -> Result<Self, BudgetError> {
        validate_amounts(&remaining)?;
        Ok(Self { remaining })
    }

    /// Governed dimensions and their remaining amounts.
    pub fn remaining(&self) -> &BTreeMap<String, i64> {
        &self.remaining
    }

    /// Deterministic admission: every governed dimension must cover its
    /// charge, otherwise the FIRST insufficient dimension (deterministic
    /// BTreeMap order) is reported and nothing is debited. On admission the
    /// debited successor state is returned; `self` is never mutated.
    pub fn check_and_debit(&self, charge: &BudgetCharge) -> Result<BudgetState, BudgetExhausted> {
        let mut next = self.remaining.clone();
        for (dimension, amount) in &charge.amounts {
            if *amount == 0 {
                continue;
            }
            if let Some(remaining) = next.get_mut(dimension) {
                if *remaining < *amount {
                    return Err(BudgetExhausted {
                        dimension: dimension.clone(),
                        remaining: *remaining,
                        charge: *amount,
                    });
                }
                *remaining -= *amount;
            }
        }
        Ok(BudgetState { remaining: next })
    }
}

/// A non-negative charge against zero or more registered dimensions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BudgetCharge {
    amounts: BTreeMap<String, i64>,
}

impl BudgetCharge {
    /// Build a validated charge.
    pub fn new(amounts: BTreeMap<String, i64>) -> Result<Self, BudgetError> {
        validate_amounts(&amounts)?;
        Ok(Self { amounts })
    }

    /// Charged dimensions and amounts.
    pub fn amounts(&self) -> &BTreeMap<String, i64> {
        &self.amounts
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    fn state(pairs: &[(&str, i64)]) -> BudgetState {
        BudgetState::new(pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()).unwrap()
    }

    fn charge(pairs: &[(&str, i64)]) -> BudgetCharge {
        BudgetCharge::new(pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()).unwrap()
    }

    #[test]
    fn debit_is_pure_and_exact() {
        let start = state(&[("tool_calls", 3), ("input_tokens", 100)]);
        let after = start
            .check_and_debit(&charge(&[("tool_calls", 1), ("input_tokens", 40)]))
            .unwrap();
        assert_eq!(after.remaining()["tool_calls"], 2);
        assert_eq!(after.remaining()["input_tokens"], 60);
        // Original state is untouched (no partial mutation on any path).
        assert_eq!(start.remaining()["tool_calls"], 3);
    }

    #[test]
    fn over_budget_fails_closed_without_partial_debit() {
        let start = state(&[("tool_calls", 1), ("input_tokens", 10)]);
        let err = start
            .check_and_debit(&charge(&[("input_tokens", 11), ("tool_calls", 1)]))
            .unwrap_err();
        assert_eq!(err.dimension, "input_tokens");
        assert_eq!((err.remaining, err.charge), (10, 11));
        assert_eq!(start.remaining()["tool_calls"], 1, "nothing was debited");
    }

    #[test]
    fn exact_exhaustion_is_admitted_and_next_charge_denied() {
        let start = state(&[("semantic_calls", 2)]);
        let drained = start
            .check_and_debit(&charge(&[("semantic_calls", 2)]))
            .unwrap();
        assert_eq!(drained.remaining()["semantic_calls"], 0);
        assert!(
            drained
                .check_and_debit(&charge(&[("semantic_calls", 1)]))
                .is_err()
        );
    }

    #[test]
    fn ungoverned_dimensions_admit_any_charge() {
        let start = state(&[("tool_calls", 1)]);
        let after = start
            .check_and_debit(&charge(&[("egress_bytes", 1_000_000)]))
            .unwrap();
        assert_eq!(after, start);
    }

    #[test]
    fn unknown_dimensions_and_negative_amounts_are_rejected_at_construction() {
        assert!(matches!(
            BudgetState::new([("gpu_seconds".to_owned(), 1)].into()),
            Err(BudgetError::UnknownDimension(_))
        ));
        assert!(matches!(
            BudgetCharge::new([("tool_calls".to_owned(), -1)].into()),
            Err(BudgetError::InvalidAmount { .. })
        ));
        assert!(matches!(
            BudgetState::new([("tool_calls".to_owned(), MAX_SAFE_INTEGER + 1)].into()),
            Err(BudgetError::InvalidAmount { .. })
        ));
    }
}
