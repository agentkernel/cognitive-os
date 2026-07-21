//! Graded readiness: MANAGEMENT_READY → USER_READY → OPERATIONAL (IMP-06 / M6-A5).
//!
//! No registered readiness carrier (D-021). Evidence is milestone e2e/fault only.

use serde_json::{Value, json};

/// Ordered readiness grades (whitepaper / DEVELOPMENT-PLAN).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReadinessGrade {
    ManagementReady = 0,
    UserReady = 1,
    Operational = 2,
}

impl ReadinessGrade {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ManagementReady => "MANAGEMENT_READY",
            Self::UserReady => "USER_READY",
            Self::Operational => "OPERATIONAL",
        }
    }

    pub const ORDER: [ReadinessGrade; 3] = [
        Self::ManagementReady,
        Self::UserReady,
        Self::Operational,
    ];
}

#[derive(Debug, Clone, Default)]
pub struct ReadinessFacts {
    pub authority_store_available: bool,
    pub audit_available: bool,
    pub management_api_available: bool,
    pub recovery_available: bool,
    pub intent_channel_available: bool,
    pub task_shell_available: bool,
    pub watch_available: bool,
    pub agent_installed_committed: bool,
    pub sandbox_evidence_ok: bool,
    pub harness_ready: bool,
    pub model_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadinessError {
    pub detail: String,
}

/// Deterministic evaluator: never skip grades; never expand capability on downgrade.
#[derive(Debug, Clone)]
pub struct ReadinessEvaluator {
    pub current: ReadinessGrade,
    pub facts: ReadinessFacts,
}

impl ReadinessEvaluator {
    pub fn new(facts: ReadinessFacts) -> Self {
        Self {
            current: ReadinessGrade::ManagementReady,
            facts,
        }
    }

    pub fn compute_ceiling(&self) -> ReadinessGrade {
        if !(self.facts.authority_store_available
            && self.facts.audit_available
            && self.facts.management_api_available
            && self.facts.recovery_available)
        {
            // Below MANAGEMENT_READY is not modeled; stay at management only if
            // management deps hold — otherwise still report management as the
            // lowest declared grade but refuse elevation.
            return ReadinessGrade::ManagementReady;
        }
        let user_ok = self.facts.intent_channel_available
            && self.facts.task_shell_available
            && self.facts.watch_available;
        if !user_ok {
            return ReadinessGrade::ManagementReady;
        }
        let op_ok = self.facts.agent_installed_committed
            && self.facts.sandbox_evidence_ok
            && self.facts.harness_ready;
        // Model unavailability must NOT block MANAGEMENT or USER.
        if op_ok {
            ReadinessGrade::Operational
        } else {
            ReadinessGrade::UserReady
        }
    }

    pub fn try_elevate(&mut self, target: ReadinessGrade) -> Result<(), ReadinessError> {
        let ceiling = self.compute_ceiling();
        if target > ceiling {
            return Err(ReadinessError {
                detail: format!(
                    "refuse elevate to {} (ceiling={})",
                    target.as_str(),
                    ceiling.as_str()
                ),
            });
        }
        // Strict order: cannot jump over intermediate grades.
        let next = match self.current {
            ReadinessGrade::ManagementReady => ReadinessGrade::UserReady,
            ReadinessGrade::UserReady => ReadinessGrade::Operational,
            ReadinessGrade::Operational => ReadinessGrade::Operational,
        };
        if target > next {
            return Err(ReadinessError {
                detail: format!(
                    "refuse out-of-order jump from {} to {}",
                    self.current.as_str(),
                    target.as_str()
                ),
            });
        }
        if target < self.current {
            return Err(ReadinessError {
                detail: "use degrade() to lower readiness".into(),
            });
        }
        self.current = target;
        Ok(())
    }

    pub fn degrade(&mut self, target: ReadinessGrade) -> Result<(), ReadinessError> {
        if target > self.current {
            return Err(ReadinessError {
                detail: "degrade cannot raise readiness".into(),
            });
        }
        self.current = target;
        Ok(())
    }

    pub fn management_available_without_model(&self) -> bool {
        self.facts.management_api_available
            && self.facts.recovery_available
            && !self.facts.model_available
    }

    pub fn snapshot(&self) -> Value {
        json!({
            "current": self.current.as_str(),
            "ceiling": self.compute_ceiling().as_str(),
            "model_required_for_management": false,
            "grades": ReadinessGrade::ORDER.map(|g| g.as_str()),
        })
    }
}

/// R0 thin-path: legal degradations vs non-degradable boundaries (IMP-06).
#[derive(Debug, Clone, Copy)]
pub struct R0ThinPath;

impl R0ThinPath {
    pub fn may_degrade_intelligent_shell(&self) -> bool {
        true
    }

    pub fn may_skip_deterministic_management(&self) -> bool {
        false
    }

    pub fn may_skip_effect_reconcile_before_loop_resume(&self) -> bool {
        false
    }

    pub fn may_skip_capability_check(&self) -> bool {
        false
    }

    pub fn may_treat_receipt_as_acceptance(&self) -> bool {
        false
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn base_facts() -> ReadinessFacts {
        ReadinessFacts {
            authority_store_available: true,
            audit_available: true,
            management_api_available: true,
            recovery_available: true,
            intent_channel_available: true,
            task_shell_available: true,
            watch_available: true,
            agent_installed_committed: false,
            sandbox_evidence_ok: false,
            harness_ready: false,
            model_available: false,
        }
    }

    #[test]
    fn management_works_without_model() {
        let facts = base_facts();
        let ev = ReadinessEvaluator::new(facts);
        assert!(ev.management_available_without_model());
        assert_eq!(ev.compute_ceiling(), ReadinessGrade::UserReady);
    }

    #[test]
    fn refuse_operational_without_sandbox_evidence() {
        let mut ev = ReadinessEvaluator::new(base_facts());
        ev.try_elevate(ReadinessGrade::UserReady).unwrap();
        let err = ev.try_elevate(ReadinessGrade::Operational).unwrap_err();
        assert!(err.detail.contains("OPERATIONAL"));
    }

    #[test]
    fn refuse_out_of_order_jump() {
        let mut ev = ReadinessEvaluator::new(base_facts());
        let err = ev.try_elevate(ReadinessGrade::Operational).unwrap_err();
        assert!(err.detail.contains("out-of-order") || err.detail.contains("ceiling"));
    }

    #[test]
    fn r0_non_degradable_boundaries() {
        let r0 = R0ThinPath;
        assert!(r0.may_degrade_intelligent_shell());
        assert!(!r0.may_skip_deterministic_management());
        assert!(!r0.may_skip_effect_reconcile_before_loop_resume());
        assert!(!r0.may_skip_capability_check());
        assert!(!r0.may_treat_receipt_as_acceptance());
    }
}
