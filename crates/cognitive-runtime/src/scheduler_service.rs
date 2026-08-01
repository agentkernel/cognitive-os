//! Deterministic scheduler eligibility service (P2-T03).
//!
//! The service is deliberately not a worker or authority-state writer. It
//! normalizes untrusted wall-clock samples into a monotonic local floor and
//! asks the durable repository to atomically acquire a fenced lease.

use cognitive_domain::WallTimestamp;
use cognitive_store::scheduler::{SchedulerRepository, SchedulerRepositoryError};
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

/// A durable lease the caller may use to start a bounded worker attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerDispatch {
    pub task_ref: String,
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
        task_ref: &str,
        lease_epoch: i64,
        observed_wall_time: &str,
    ) -> Result<SchedulerDispatch, SchedulerServiceError> {
        let trusted_wall_time = self.observe_wall_time(observed_wall_time)?;
        let lease_expires = add_lease_ttl(&trusted_wall_time, self.lease_ttl_seconds)?;
        let row = repository.acquire_eligible_lease(
            task_ref,
            &self.owner,
            lease_epoch,
            &trusted_wall_time,
            &lease_expires,
        )?;
        Ok(SchedulerDispatch {
            task_ref: row.task_ref,
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
