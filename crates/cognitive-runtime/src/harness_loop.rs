//! Bounded Harness Loop runtime (M5 RUN batch 2b).
//!
//! Consumes the frozen kernel [`LoopDriver`] ports. This module owns the
//! OODA-side *orchestration* decisions that the kernel deliberately left
//! open: when stagnation is detected, escalate or stop — never spin.

use cognitive_domain::{BudgetId, ObjectId, Version};
use cognitive_kernel::budget::BudgetCharge;
use cognitive_kernel::effects::{EffectError, WriterLease};
use cognitive_kernel::engine::CommittedTransition;
use cognitive_kernel::harness::{LoopDriver, ProgressStatus, StagnationFacts};
use cognitive_kernel::ports::{
    AuthorityStore, Clock, HarnessStore, IdGenerator, IntentChainStore, ProtocolStore,
};

/// Outcome of one bounded harness drive cycle after progress is recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarnessDecision {
    /// Iteration advanced with evidence of progress.
    Advanced { iteration: i64 },
    /// No progress; still within stagnation tolerance.
    Waiting { consecutive_without_progress: u64 },
    /// Stagnation ceiling hit — stop the loop (no unbounded spin).
    StoppedForStagnation { consecutive_without_progress: u64 },
    /// Stagnation ceiling hit — escalate to a human/management surface.
    Escalated { consecutive_without_progress: u64 },
}

/// What to do when stagnation consecutive count reaches the ceiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StagnationPolicy {
    Stop,
    Escalate,
}

/// Runtime harness over a [`LoopDriver`].
pub struct BoundedHarness<'a, S, C, G> {
    driver: &'a LoopDriver<'a, S, C, G>,
    stagnation_ceiling: u64,
    policy: StagnationPolicy,
}

impl<'a, S, C, G> BoundedHarness<'a, S, C, G>
where
    S: AuthorityStore + ProtocolStore + IntentChainStore + HarnessStore,
    C: Clock,
    G: IdGenerator,
{
    pub fn new(
        driver: &'a LoopDriver<'a, S, C, G>,
        stagnation_ceiling: u64,
        policy: StagnationPolicy,
    ) -> Self {
        Self {
            driver,
            stagnation_ceiling: stagnation_ceiling.max(1),
            policy,
        }
    }

    pub fn start(
        &self,
        loop_id: &ObjectId,
        expected_version: Version,
        task_ref: &str,
        budget_id: &BudgetId,
        lease: &WriterLease,
    ) -> Result<CommittedTransition, EffectError> {
        self.driver
            .start_loop(loop_id, expected_version, task_ref, budget_id, lease)
    }

    /// One iteration: begin → record progress → fold stagnation → decide.
    /// Hard ceilings (`RESOURCE_BUDGET_EXHAUSTED`, etc.) propagate as
    /// [`EffectError`] — the caller must not retry unboundedly.
    #[allow(clippy::too_many_arguments)]
    pub fn drive_iteration(
        &self,
        loop_id: &ObjectId,
        expected_version: Version,
        task_ref: &str,
        iteration: i64,
        budget_id: &BudgetId,
        charge: &BudgetCharge,
        progress: ProgressStatus,
        action_fingerprint: &str,
        evidence_refs: &[String],
        lease: &WriterLease,
    ) -> Result<(CommittedTransition, HarnessDecision), EffectError> {
        let committed = self.driver.begin_iteration(
            loop_id,
            expected_version,
            task_ref,
            iteration,
            budget_id,
            charge,
            lease,
        )?;
        self.driver.record_progress(
            loop_id,
            iteration,
            progress,
            action_fingerprint,
            evidence_refs,
            lease,
        )?;
        let facts = self.driver.stagnation(loop_id)?;
        Ok((
            committed,
            decide_stagnation(
                &facts,
                progress,
                iteration,
                self.stagnation_ceiling,
                self.policy,
            ),
        ))
    }

    pub fn stagnation(&self, loop_id: &ObjectId) -> Result<StagnationFacts, EffectError> {
        self.driver.stagnation(loop_id)
    }
}

/// Pure stagnation decision (unit-tested; used by [`BoundedHarness`]).
pub fn decide_stagnation(
    facts: &StagnationFacts,
    progress: ProgressStatus,
    iteration: i64,
    stagnation_ceiling: u64,
    policy: StagnationPolicy,
) -> HarnessDecision {
    if matches!(progress, ProgressStatus::Advanced) {
        return HarnessDecision::Advanced { iteration };
    }
    let consecutive = facts.consecutive_without_progress;
    if consecutive >= stagnation_ceiling.max(1) {
        return match policy {
            StagnationPolicy::Stop => HarnessDecision::StoppedForStagnation {
                consecutive_without_progress: consecutive,
            },
            StagnationPolicy::Escalate => HarnessDecision::Escalated {
                consecutive_without_progress: consecutive,
            },
        };
    }
    HarnessDecision::Waiting {
        consecutive_without_progress: consecutive,
    }
}
