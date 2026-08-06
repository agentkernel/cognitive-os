//! Deterministic scheduler eligibility service (P2-T03).
//!
//! The service is deliberately not a worker or authority-state writer. It
//! normalizes untrusted wall-clock samples into a monotonic local floor and
//! asks the durable repository to atomically acquire a fenced lease.

use cognitive_domain::{BudgetId, ObjectId, Version, WallTimestamp};
use cognitive_kernel::effects::{EffectError, WriterLease};
use cognitive_kernel::engine::CommittedTransition;
use cognitive_kernel::harness::{CeilingStopReason, LoopDriver};
use cognitive_kernel::ports::{
    AuthorityStore, Clock, HarnessStore, IdGenerator, IntentChainStore, ProtocolStore,
};
use cognitive_store::scheduler::{SchedulerRepository, SchedulerRepositoryError, SchedulerWorkKey};
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

/// A durable lease the caller may use to start a bounded worker attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerDispatch {
    pub task_ref: String,
    pub contract_epoch: i64,
    pub lease_owner: String,
    pub lease_epoch: i64,
    pub lease_expires: String,
    pub attempt_count: i64,
}

/// Durable authority facts that bound one scheduler dispatch attempt.
///
/// The daemon must reload this snapshot from its durable task, loop, and
/// budget records before asking the scheduler to lease work. It is not a
/// worker-owned counter and must never be derived from a model response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerCeilingFacts {
    pub deadline: Option<String>,
    pub retry_count: i64,
    pub retry_ceiling: i64,
    pub completed_steps: i64,
    pub step_ceiling: i64,
    pub spent_cost_microunits: i64,
    pub cost_ceiling_microunits: i64,
}

/// The first inclusive hard ceiling that refuses a new scheduler dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerStopReason {
    DeadlineReached,
    RetryCeilingReached,
    StepCeilingReached,
    CostCeilingReached,
}

/// Fail-closed result of evaluating one worker's dispatch ceiling.
///
/// A reached ceiling is not merely a local scheduler refusal. The runtime must
/// persist the corresponding terminal loop transition before it can consider
/// a worker dispatch. `Stopped` carries the committed kernel transition so
/// callers cannot substitute an in-memory flag for durable authority state.
#[derive(Debug, Clone, PartialEq)]
pub enum SchedulerCeilingDispatch {
    Proceed,
    Stopped(CommittedTransition),
}

/// Scheduler-service validation and persistence failures.
#[derive(Debug, Error)]
pub enum SchedulerServiceError {
    #[error("scheduler owner must not be empty")]
    EmptyOwner,
    #[error("scheduler lease TTL must be positive")]
    InvalidLeaseTtl,
    #[error("scheduler clock sample is invalid: {0}")]
    InvalidClockSample(String),
    #[error("scheduler timestamp arithmetic failed: {0}")]
    TimestampArithmetic(String),
    #[error("scheduler authority fact is invalid: {0}")]
    InvalidAuthorityFact(String),
    #[error(transparent)]
    Repository(#[from] SchedulerRepositoryError),
}

/// Errors while converting a scheduler ceiling into a durable loop STOP.
#[derive(Debug, Error)]
pub enum SchedulerCeilingDispatchError {
    #[error(transparent)]
    Scheduler(#[from] SchedulerServiceError),
    #[error(transparent)]
    Effect(#[from] EffectError),
}

/// Scheduler eligibility policy for one worker identity.
///
/// A service instance never moves its trusted wall-clock floor backward.
/// Callers must create a new instance only after restoring its worker's
/// persisted scheduler state; a restarted worker still relies on the durable
/// lease expiry and epoch fencing enforced by [`SchedulerRepository`].
pub struct SchedulerService {
    owner: String,
    lease_ttl_seconds: i64,
    trusted_wall_time: Option<WallTimestamp>,
}

impl SchedulerService {
    /// Create a scheduler policy for one non-empty worker owner.
    pub fn new(
        owner: impl Into<String>,
        lease_ttl_seconds: i64,
    ) -> Result<Self, SchedulerServiceError> {
        let owner = owner.into();
        if owner.is_empty() {
            return Err(SchedulerServiceError::EmptyOwner);
        }
        if lease_ttl_seconds <= 0 {
            return Err(SchedulerServiceError::InvalidLeaseTtl);
        }
        Ok(Self {
            owner,
            lease_ttl_seconds,
            trusted_wall_time: None,
        })
    }

    /// Observe a wall-clock value, clamping backwards movement to the last
    /// trusted instant so a rollback cannot produce a second dispatch.
    pub fn observe_wall_time(
        &mut self,
        observed_wall_time: &str,
    ) -> Result<String, SchedulerServiceError> {
        let observed = WallTimestamp::parse(observed_wall_time).map_err(|_| {
            SchedulerServiceError::InvalidClockSample(observed_wall_time.to_owned())
        })?;
        if self
            .trusted_wall_time
            .as_ref()
            .is_none_or(|trusted| observed.instant_key() > trusted.instant_key())
        {
            self.trusted_wall_time = Some(observed);
        }
        match self.trusted_wall_time.as_ref() {
            Some(trusted_wall_time) => Ok(trusted_wall_time.as_str().to_owned()),
            None => Err(SchedulerServiceError::InvalidClockSample(
                observed_wall_time.to_owned(),
            )),
        }
    }

    /// Claim work eligible at the monotonic wall-time floor. Expired leases
    /// are reclaimed only by a higher epoch inside the durable repository.
    pub fn claim_eligible(
        &mut self,
        repository: &mut SchedulerRepository,
        work_key: &SchedulerWorkKey,
        lease_epoch: i64,
        observed_wall_time: &str,
    ) -> Result<SchedulerDispatch, SchedulerServiceError> {
        let trusted_wall_time = self.observe_wall_time(observed_wall_time)?;
        let lease_expires = add_lease_ttl(&trusted_wall_time, self.lease_ttl_seconds)?;
        let row = repository.acquire_eligible_lease(
            work_key,
            &self.owner,
            lease_epoch,
            &trusted_wall_time,
            &lease_expires,
        )?;
        Ok(SchedulerDispatch {
            task_ref: row.task_ref,
            contract_epoch: row.contract_epoch,
            lease_owner: row.lease_owner.unwrap_or_default(),
            lease_epoch: row.lease_epoch,
            lease_expires: row.lease_expires.unwrap_or_default(),
            attempt_count: row.attempt_count,
        })
    }

    /// Evaluate durable authority ceilings before a caller can acquire a new
    /// lease. Boundaries are inclusive: reaching a ceiling stops the next
    /// dispatch rather than allowing one unaccounted extra attempt.
    pub fn evaluate_authority_ceilings(
        &mut self,
        facts: &SchedulerCeilingFacts,
        observed_wall_time: &str,
    ) -> Result<Option<SchedulerStopReason>, SchedulerServiceError> {
        let trusted_wall_time = self.observe_wall_time(observed_wall_time)?;
        validate_non_negative("retry_count", facts.retry_count)?;
        validate_non_negative("retry_ceiling", facts.retry_ceiling)?;
        validate_non_negative("completed_steps", facts.completed_steps)?;
        validate_non_negative("step_ceiling", facts.step_ceiling)?;
        validate_non_negative("spent_cost_microunits", facts.spent_cost_microunits)?;
        validate_non_negative("cost_ceiling_microunits", facts.cost_ceiling_microunits)?;

        if let Some(deadline) = facts.deadline.as_deref() {
            let parsed_deadline = WallTimestamp::parse(deadline)
                .map_err(|_| SchedulerServiceError::InvalidAuthorityFact("deadline".to_owned()))?;
            let parsed_trusted_time = WallTimestamp::parse(&trusted_wall_time).map_err(|_| {
                SchedulerServiceError::InvalidClockSample(trusted_wall_time.clone())
            })?;
            if parsed_deadline.instant_key() <= parsed_trusted_time.instant_key() {
                return Ok(Some(SchedulerStopReason::DeadlineReached));
            }
        }
        if facts.retry_count >= facts.retry_ceiling {
            return Ok(Some(SchedulerStopReason::RetryCeilingReached));
        }
        if facts.completed_steps >= facts.step_ceiling {
            return Ok(Some(SchedulerStopReason::StepCeilingReached));
        }
        if facts.spent_cost_microunits >= facts.cost_ceiling_microunits {
            return Ok(Some(SchedulerStopReason::CostCeilingReached));
        }
        Ok(None)
    }

    /// Evaluate fresh daemon-owned ceiling facts and durably STOP the loop
    /// before a worker can acquire a dispatch lease.
    ///
    /// The caller supplies facts it has just reloaded from the TaskContract,
    /// loop progress, and budget authority records. When a boundary is
    /// reached, this method invokes the kernel's fenced STOP transition rather
    /// than returning a retryable local refusal. A clear result performs no
    /// worker lease acquisition; that remains a later caller operation.
    #[allow(clippy::too_many_arguments)]
    pub fn stop_before_dispatch_when_ceiling_reached<S, C, G>(
        &mut self,
        facts: &SchedulerCeilingFacts,
        observed_wall_time: &str,
        driver: &LoopDriver<'_, S, C, G>,
        loop_id: &ObjectId,
        expected_version: Version,
        task_ref: &str,
        budget_id: &BudgetId,
        writer_lease: &WriterLease,
    ) -> Result<SchedulerCeilingDispatch, SchedulerCeilingDispatchError>
    where
        S: AuthorityStore + ProtocolStore + IntentChainStore + HarnessStore,
        C: Clock,
        G: IdGenerator,
    {
        let Some(stop_reason) = self.evaluate_authority_ceilings(facts, observed_wall_time)? else {
            return Ok(SchedulerCeilingDispatch::Proceed);
        };
        let transition = driver.stop_for_ceiling(
            loop_id,
            expected_version,
            task_ref,
            budget_id,
            kernel_stop_reason(stop_reason),
            writer_lease,
        )?;
        Ok(SchedulerCeilingDispatch::Stopped(transition))
    }
}

fn kernel_stop_reason(stop_reason: SchedulerStopReason) -> CeilingStopReason {
    match stop_reason {
        SchedulerStopReason::DeadlineReached => CeilingStopReason::DeadlineReached,
        SchedulerStopReason::RetryCeilingReached => CeilingStopReason::RetryCeilingReached,
        SchedulerStopReason::StepCeilingReached => CeilingStopReason::StepCeilingReached,
        SchedulerStopReason::CostCeilingReached => CeilingStopReason::CostCeilingReached,
    }
}

fn validate_non_negative(field_name: &str, value: i64) -> Result<(), SchedulerServiceError> {
    if value < 0 {
        return Err(SchedulerServiceError::InvalidAuthorityFact(format!(
            "{field_name} must not be negative"
        )));
    }
    Ok(())
}

fn add_lease_ttl(wall_time: &str, lease_ttl_seconds: i64) -> Result<String, SchedulerServiceError> {
    let timestamp = OffsetDateTime::parse(wall_time, &Rfc3339)
        .map_err(|_| SchedulerServiceError::InvalidClockSample(wall_time.to_owned()))?;
    timestamp
        .checked_add(Duration::seconds(lease_ttl_seconds))
        .ok_or_else(|| SchedulerServiceError::TimestampArithmetic(wall_time.to_owned()))?
        .format(&Rfc3339)
        .map_err(|error| SchedulerServiceError::TimestampArithmetic(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{SchedulerStopReason, kernel_stop_reason};
    use cognitive_kernel::harness::CeilingStopReason;

    /// Regression coverage for the scheduler-to-kernel reason boundary:
    /// every evaluated ceiling must map to the registered terminal STOP edge.
    #[test]
    fn maps_each_scheduler_ceiling_to_its_registered_kernel_stop_reason() {
        assert_eq!(
            kernel_stop_reason(SchedulerStopReason::DeadlineReached),
            CeilingStopReason::DeadlineReached
        );
        assert_eq!(
            kernel_stop_reason(SchedulerStopReason::RetryCeilingReached),
            CeilingStopReason::RetryCeilingReached
        );
        assert_eq!(
            kernel_stop_reason(SchedulerStopReason::StepCeilingReached),
            CeilingStopReason::StepCeilingReached
        );
        assert_eq!(
            kernel_stop_reason(SchedulerStopReason::CostCeilingReached),
            CeilingStopReason::CostCeilingReached
        );
    }
}
