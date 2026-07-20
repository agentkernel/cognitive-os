//! Deterministic kernel ports for the bounded Harness Loop
//! (`docs/standards/task-loop-verification.md` section 4; REQ-RUN-004,
//! REQ-RUN-005, REQ-RUN-007, REQ-RUN-008; loop table
//! `specs/transitions/loop.transitions.json`).
//!
//! Division of labor (frozen for Lane-RUN): the OODA orchestration —
//! which phase to enter, what to observe, when to diagnose — is the M5
//! runtime's. What lives HERE is everything that must be deterministic
//! and durable-fact-derived:
//!
//! - loop boundary transitions with sanctioned guard derivations
//!   ([`LoopDriver::start_loop`], [`LoopDriver::begin_iteration`],
//!   [`LoopDriver::end_iteration`]): contract pinning, hard budget
//!   admission + same-transaction debit, checkpoint-bound continuation;
//! - typed progress facts ([`LoopDriver::record_progress`]): progress is
//!   a verifiable difference with evidence references, never a transcript
//!   length or a model self-report (REQ-RUN-007);
//! - stagnation and retry arithmetic ([`LoopDriver::stagnation`],
//!   [`LoopDriver::retry_count`], [`LoopDriver::admit_retry`]): pure
//!   folds over durable facts, bounded by the contract's registered
//!   ceilings (REQ-RUN-008);
//! - iteration ceilings: exceeding `max_iterations` is a deterministic
//!   `RESOURCE_BUDGET_EXHAUSTED` denial, not a policy suggestion.
//!
//! Checkpoint persistence itself is the M4 port
//! ([`crate::ports::ProtocolStore::append_checkpoint`]); recovery-side
//! validation is [`crate::recovery::validate_checkpoint`].

use crate::budget::{BudgetCharge, BudgetState};
use crate::effects::{
    EffectError, ProtocolDenial, WriterLease, canonical_text, port_rejection, store_rejection,
    strong_ref,
};
use crate::engine::{
    BudgetChargeCommand, Causation, CommittedTransition, Reason, TablePin, TransitionCommand,
    TransitionEngine,
};
use crate::error::{RESOURCE_BUDGET_EXHAUSTED, STATE_CONFLICT};
use crate::ports::{
    AuthorityStore, Clock, HarnessStore, IdGenerator, IntentChainStore, ProgressFactRow,
    ProtocolStore,
};
use cognitive_contracts::generated::object_reference::StrongReference;
use cognitive_contracts::generated::task_contract::TaskContract;
use cognitive_domain::{
    BudgetId, LifecycleDomain, ObjectId, ReasonCode, StateName, UriRef, Version, WallTimestamp,
};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

/// Typed progress status (schema `loop-checkpoint.schema.json` progress
/// set). The variant is a recorded FACT; what counts as progress is fixed
/// by REQ-RUN-007, not by the recorder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressStatus {
    /// A verifiable state difference, reduced uncertainty or satisfied
    /// precondition — requires evidence references.
    Advanced,
    /// No progress this iteration.
    NoProgress,
    /// Progress could not be established.
    Uncertain,
    /// Blocked on a dependency or gate.
    Blocked,
}

impl ProgressStatus {
    /// Registered schema token.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProgressStatus::Advanced => "advanced",
            ProgressStatus::NoProgress => "none",
            ProgressStatus::Uncertain => "uncertain",
            ProgressStatus::Blocked => "blocked",
        }
    }
}

/// Deterministic stagnation facts folded from durable progress rows.
/// The kernel supplies the arithmetic; whether the loop stops, waits or
/// escalates on these facts is the runtime's contract-driven decision
/// (loop table `STAGNATION_DETECTED` edge).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagnationFacts {
    /// Trailing iterations without an `advanced` fact.
    pub consecutive_without_progress: u64,
    /// Last iteration that recorded verifiable progress.
    pub last_advanced_iteration: Option<i64>,
    /// Total iterations with recorded facts.
    pub recorded_iterations: u64,
}

/// Sanctioned derivation of one task's CURRENT contract facts: reloaded
/// from the durable contract row (never from memory) and parsed through
/// the generated `task-contract` binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractFacts {
    /// Task the contract governs.
    pub task_ref: String,
    /// Current contract epoch.
    pub contract_epoch: i64,
    /// Hard iteration ceiling.
    pub max_iterations: i64,
    /// Hard same-action retry ceiling.
    pub max_retries: i64,
    /// Canonical digest of the contract row.
    pub contract_digest: String,
}

fn denial(registered: crate::error::RegisteredError, detail: String) -> ProtocolDenial {
    ProtocolDenial { registered, detail }
}

/// The deterministic loop-boundary driver (one store, clock, ID source;
/// mirrors [`crate::effects::EffectProtocol`]).
pub struct LoopDriver<'a, S, C, G> {
    store: &'a S,
    clock: &'a C,
    ids: &'a G,
    /// Loop-authority reference this driver commits under.
    pub authority_ref: UriRef,
    /// Actor reference.
    pub actor_ref: UriRef,
    /// Correlation chain.
    pub correlation_id: UriRef,
}

impl<'a, S, C, G> LoopDriver<'a, S, C, G>
where
    S: AuthorityStore + ProtocolStore + IntentChainStore + HarnessStore,
    C: Clock,
    G: IdGenerator,
{
    /// Build a driver bound to actor/authority/correlation references.
    pub fn new(
        store: &'a S,
        clock: &'a C,
        ids: &'a G,
        actor_ref: UriRef,
        authority_ref: UriRef,
        correlation_id: UriRef,
    ) -> Self {
        Self {
            store,
            clock,
            ids,
            authority_ref,
            actor_ref,
            correlation_id,
        }
    }

    fn engine(&self) -> TransitionEngine<'a, S, C, G> {
        TransitionEngine::new(self.store, self.clock, self.ids)
    }

    fn now(&self) -> Result<WallTimestamp, EffectError> {
        self.clock
            .now()
            .map_err(|err| port_rejection("clock", err))
            .map_err(EffectError::Rejected)
    }

    fn verify_lease(&self, lease: &WriterLease) -> Result<(), EffectError> {
        let current = self
            .store
            .current_fencing_epoch()
            .map_err(store_rejection)?;
        if lease.epoch != current {
            return Err(denial(
                STATE_CONFLICT,
                format!(
                    "writer fenced: lease epoch {} != current epoch {current}",
                    lease.epoch
                ),
            )
            .into());
        }
        Ok(())
    }

    /// Sanctioned derivation: the task's CURRENT contract facts, reloaded
    /// durably and parsed through the generated binding (REQ-RUN-004: no
    /// loop boundary decision is made from a remembered contract).
    pub fn contract_facts(&self, task_ref: &str) -> Result<ContractFacts, EffectError> {
        let epoch = self
            .store
            .current_contract_epoch(task_ref)
            .map_err(store_rejection)?;
        if epoch == 0 {
            return Err(denial(
                STATE_CONFLICT,
                format!("no TaskContract for {task_ref}: a loop cannot run uncontracted"),
            )
            .into());
        }
        let row = self
            .store
            .load_task_contract(task_ref, epoch)
            .map_err(store_rejection)?
            .ok_or_else(|| {
                denial(
                    STATE_CONFLICT,
                    format!("contract epoch {epoch} of {task_ref} unreadable"),
                )
            })?;
        let contract: TaskContract = serde_json::from_str(&row.canonical_json).map_err(|err| {
            denial(
                STATE_CONFLICT,
                format!("stored contract does not parse through the generated binding: {err}"),
            )
        })?;
        Ok(ContractFacts {
            task_ref: row.task_ref,
            contract_epoch: row.contract_epoch,
            max_iterations: contract.max_iterations,
            max_retries: contract.max_retries,
            contract_digest: row.contract_digest,
        })
    }

    /// Sanctioned derivation: the loop's hard budget still admits work —
    /// the ledger row loads and no governed dimension is exhausted.
    fn budget_remaining(&self, budget_id: &BudgetId) -> Result<Option<BudgetState>, EffectError> {
        let stored = self.store.load_budget(budget_id).map_err(store_rejection)?;
        Ok(stored.and_then(|row| {
            if row.state.remaining().values().all(|amount| *amount >= 1) {
                Some(row.state)
            } else {
                None
            }
        }))
    }

    fn last_recorded_iteration(&self, loop_id: &ObjectId) -> Result<i64, EffectError> {
        let facts = self
            .store
            .list_progress_facts(loop_id)
            .map_err(store_rejection)?;
        Ok(facts.last().map_or(0, |fact| fact.iteration))
    }

    // Explicit deterministic inputs are the point of the gate surface.
    #[allow(clippy::too_many_arguments)]
    fn command(
        &self,
        loop_id: &ObjectId,
        from: &str,
        to: &str,
        reason: &str,
        established: BTreeSet<String>,
        evidence: BTreeMap<String, StrongReference>,
        expected_version: Version,
        budget: Option<BudgetChargeCommand>,
        lease: &WriterLease,
    ) -> Result<TransitionCommand, EffectError> {
        Ok(TransitionCommand {
            request_id: uri(&format!("request://loop/{}/{from}-{to}", loop_id.as_str()))?,
            domain: LifecycleDomain::Loop,
            object_id: loop_id.clone(),
            subject_ref: uri(&format!("loop://{}", loop_id.as_str()))?,
            from: state(from)?,
            to: state(to)?,
            expected_version,
            reason: Reason {
                code: reason_code(reason)?,
                detail: None,
            },
            causation: Causation {
                causation_id: self.correlation_id.clone(),
                correlation_id: self.correlation_id.clone(),
            },
            actor_ref: self.actor_ref.clone(),
            authority_ref: self.authority_ref.clone(),
            requested_at: self.now()?,
            table_pin: TablePin::current(LifecycleDomain::Loop).map_err(EffectError::Rejected)?,
            established_guards: established,
            evidence,
            budget,
            outbox_destinations: vec![],
            fencing_epoch: Some(lease.epoch),
        })
    }

    /// START -> OBSERVE (`LOOP_STARTED`). Guards derived: `task_contract_pinned`
    /// (the task's current contract reloaded durably), `loop_budget_available`
    /// (ledger row loads, no governed dimension exhausted). Evidence:
    /// `loop_checkpoint_or_start_record` = strong reference to the contract
    /// row (the start record IS the pinned contract).
    pub fn start_loop(
        &self,
        loop_id: &ObjectId,
        expected_version: Version,
        task_ref: &str,
        budget_id: &BudgetId,
        lease: &WriterLease,
    ) -> Result<CommittedTransition, EffectError> {
        self.verify_lease(lease)?;
        let mut established = BTreeSet::new();
        let facts = self.contract_facts(task_ref)?;
        established.insert("task_contract_pinned".to_owned());
        if self.budget_remaining(budget_id)?.is_some() {
            established.insert("loop_budget_available".to_owned());
        }
        let contract_row = self
            .store
            .load_task_contract(task_ref, facts.contract_epoch)
            .map_err(store_rejection)?
            .ok_or_else(|| denial(STATE_CONFLICT, "contract row vanished".to_owned()))?;
        let mut evidence = BTreeMap::new();
        evidence.insert(
            "loop_checkpoint_or_start_record".to_owned(),
            strong_ref(&contract_row.contract_id, 1, &contract_row.canonical_json)
                .map_err(EffectError::Denied)?,
        );
        let cmd = self.command(
            loop_id,
            "START",
            "OBSERVE",
            "LOOP_STARTED",
            established,
            evidence,
            expected_version,
            None,
            lease,
        )?;
        Ok(self.engine().commit_transition(&cmd)?)
    }

    /// CONTINUE -> OBSERVE (`NEXT_ITERATION`): the deterministic iteration
    /// gate (REQ-RUN-005 hard preconditions, fail closed):
    ///
    /// - `iteration` must be exactly the successor of the last recorded
    ///   iteration (no skipped or replayed accounting);
    /// - `iteration` must not exceed the contract's `max_iterations`
    ///   ceiling (`RESOURCE_BUDGET_EXHAUSTED` — a deterministic hard
    ///   limit, never "one more try");
    /// - the loop's hard budget must admit `charge`; the debit commits in
    ///   the SAME transaction as the transition (REQ-RES-001 consumption
    ///   point);
    /// - a durable checkpoint must exist (`loop_checkpoint` evidence:
    ///   continuation binds recovery-stable facts, REQ-RUN-006).
    #[allow(clippy::too_many_arguments)]
    pub fn begin_iteration(
        &self,
        loop_id: &ObjectId,
        expected_version: Version,
        task_ref: &str,
        iteration: i64,
        budget_id: &BudgetId,
        charge: &BudgetCharge,
        lease: &WriterLease,
    ) -> Result<CommittedTransition, EffectError> {
        self.verify_lease(lease)?;
        let facts = self.contract_facts(task_ref)?;
        let last = self.last_recorded_iteration(loop_id)?;
        if iteration != last + 1 {
            return Err(denial(
                STATE_CONFLICT,
                format!(
                    "iteration accounting must be monotonic: last recorded {last}, \
                     requested {iteration}"
                ),
            )
            .into());
        }
        if iteration > facts.max_iterations {
            return Err(denial(
                RESOURCE_BUDGET_EXHAUSTED,
                format!(
                    "iteration {iteration} exceeds the contract ceiling max_iterations={} \
                     of {} (epoch {}): the loop stops or escalates, it does not spin",
                    facts.max_iterations, facts.task_ref, facts.contract_epoch
                ),
            )
            .into());
        }
        let mut established = BTreeSet::new();
        if self.budget_remaining(budget_id)?.is_some() {
            established.insert("loop_budget_remaining".to_owned());
        }
        let checkpoint = self
            .store
            .latest_checkpoint(loop_id)
            .map_err(store_rejection)?
            .ok_or_else(|| {
                denial(
                    STATE_CONFLICT,
                    "no durable checkpoint: a continuing loop must bind recovery-stable \
                     facts before the next iteration"
                        .to_owned(),
                )
            })?;
        let mut evidence = BTreeMap::new();
        evidence.insert(
            "loop_checkpoint".to_owned(),
            strong_ref(&checkpoint.checkpoint_id, 1, &checkpoint.canonical_json)
                .map_err(EffectError::Denied)?,
        );
        let cmd = self.command(
            loop_id,
            "CONTINUE",
            "OBSERVE",
            "NEXT_ITERATION",
            established,
            evidence,
            expected_version,
            Some(BudgetChargeCommand {
                budget_id: budget_id.clone(),
                charge: charge.clone(),
            }),
            lease,
        )?;
        Ok(self.engine().commit_transition(&cmd)?)
    }

    /// Record one typed progress fact (REQ-RUN-007). Deterministic rules,
    /// fail closed:
    ///
    /// - `advanced` REQUIRES at least one evidence reference — a claim of
    ///   progress without a verifiable difference is not recordable;
    /// - iteration accounting is monotonic (exactly last + 1);
    /// - the writer's fencing epoch is re-verified inside the store
    ///   transaction (stale writers cannot poison stagnation counters).
    pub fn record_progress(
        &self,
        loop_id: &ObjectId,
        iteration: i64,
        status: ProgressStatus,
        action_fingerprint: &str,
        evidence_refs: &[String],
        lease: &WriterLease,
    ) -> Result<ProgressFactRow, EffectError> {
        self.verify_lease(lease)?;
        if status == ProgressStatus::Advanced && evidence_refs.is_empty() {
            return Err(denial(
                STATE_CONFLICT,
                "progress=advanced requires evidence references: a verifiable state \
                 difference, reduced uncertainty or satisfied precondition (REQ-RUN-007); \
                 a bare claim is not progress"
                    .to_owned(),
            )
            .into());
        }
        if action_fingerprint.is_empty() {
            return Err(denial(
                STATE_CONFLICT,
                "action fingerprint must be non-empty (REQ-RUN-008 retry accounting key)"
                    .to_owned(),
            )
            .into());
        }
        let last = self.last_recorded_iteration(loop_id)?;
        if iteration != last + 1 {
            return Err(denial(
                STATE_CONFLICT,
                format!(
                    "progress accounting must be monotonic: last recorded {last}, \
                     requested {iteration}"
                ),
            )
            .into());
        }
        let recorded_at = self.now()?;
        let evidence_refs_json =
            canonical_text(&json!(evidence_refs)).map_err(EffectError::Denied)?;
        let fact = ProgressFactRow {
            loop_object_id: loop_id.clone(),
            iteration,
            status: status.as_str().to_owned(),
            action_fingerprint: action_fingerprint.to_owned(),
            evidence_refs_json,
            recorded_at,
            fencing_epoch: lease.epoch,
        };
        self.store
            .append_progress_fact(&fact)
            .map_err(store_rejection)?;
        Ok(fact)
    }

    /// Deterministic stagnation arithmetic over durable progress facts:
    /// how many trailing iterations produced no `advanced` fact. The
    /// runtime feeds this to the loop table's `STAGNATION_DETECTED` /
    /// escalation edges — a stagnating loop stops or escalates, it never
    /// spins on "one more model call".
    pub fn stagnation(&self, loop_id: &ObjectId) -> Result<StagnationFacts, EffectError> {
        let facts = self
            .store
            .list_progress_facts(loop_id)
            .map_err(store_rejection)?;
        let last_advanced_iteration = facts
            .iter()
            .filter(|fact| fact.status == "advanced")
            .map(|fact| fact.iteration)
            .max();
        let consecutive_without_progress = facts
            .iter()
            .rev()
            .take_while(|fact| fact.status != "advanced")
            .count() as u64;
        Ok(StagnationFacts {
            consecutive_without_progress,
            last_advanced_iteration,
            recorded_iterations: facts.len() as u64,
        })
    }

    /// Deterministic retry accounting (REQ-RUN-008): non-advancing
    /// attempts of the SAME action fingerprint.
    pub fn retry_count(
        &self,
        loop_id: &ObjectId,
        action_fingerprint: &str,
    ) -> Result<u64, EffectError> {
        let facts = self
            .store
            .list_progress_facts(loop_id)
            .map_err(store_rejection)?;
        Ok(facts
            .iter()
            .filter(|fact| {
                fact.action_fingerprint == action_fingerprint && fact.status != "advanced"
            })
            .count() as u64)
    }

    /// Deterministic retry admission (REQ-RUN-008): a further attempt of
    /// an action whose non-advancing count already reached the contract's
    /// `max_retries` is refused with the registered hard-limit code.
    pub fn admit_retry(
        &self,
        facts: &ContractFacts,
        prior_failed_attempts: u64,
    ) -> Result<(), ProtocolDenial> {
        if prior_failed_attempts >= facts.max_retries as u64 {
            return Err(denial(
                RESOURCE_BUDGET_EXHAUSTED,
                format!(
                    "retry bound reached: {prior_failed_attempts} non-advancing attempts, \
                     contract max_retries={} ({} epoch {})",
                    facts.max_retries, facts.task_ref, facts.contract_epoch
                ),
            ));
        }
        Ok(())
    }

    /// VERIFY -> CONTINUE (`PROGRESS_VERIFIED`): the iteration closes with
    /// a verification report. Guards derived: `loop_budget_remaining`
    /// (durable ledger reload), `task_not_accepted` (the governed task
    /// object reloaded — a task already COMPLETED admits no further
    /// iterations). Evidence: `verification_report`.
    #[allow(clippy::too_many_arguments)]
    pub fn end_iteration(
        &self,
        loop_id: &ObjectId,
        expected_version: Version,
        task_object_id: &ObjectId,
        verification_report_id: &ObjectId,
        verification_report_content: &str,
        budget_id: &BudgetId,
        lease: &WriterLease,
    ) -> Result<CommittedTransition, EffectError> {
        self.verify_lease(lease)?;
        let mut established = BTreeSet::new();
        if self.budget_remaining(budget_id)?.is_some() {
            established.insert("loop_budget_remaining".to_owned());
        }
        let task = self
            .store
            .load_object(LifecycleDomain::Task, task_object_id)
            .map_err(store_rejection)?;
        if task.is_some_and(|task| task.state.as_str() != "COMPLETED") {
            established.insert("task_not_accepted".to_owned());
        }
        let mut evidence = BTreeMap::new();
        evidence.insert(
            "verification_report".to_owned(),
            strong_ref(verification_report_id, 1, verification_report_content)
                .map_err(EffectError::Denied)?,
        );
        let cmd = self.command(
            loop_id,
            "VERIFY",
            "CONTINUE",
            "PROGRESS_VERIFIED",
            established,
            evidence,
            expected_version,
            None,
            lease,
        )?;
        Ok(self.engine().commit_transition(&cmd)?)
    }
}

fn uri(text: &str) -> Result<UriRef, EffectError> {
    UriRef::parse(text)
        .map_err(|err| denial(STATE_CONFLICT, format!("bad uri: {err}")))
        .map_err(EffectError::Denied)
}

fn state(text: &str) -> Result<StateName, EffectError> {
    StateName::parse(text)
        .map_err(|err| denial(STATE_CONFLICT, format!("bad state: {err}")))
        .map_err(EffectError::Denied)
}

fn reason_code(text: &str) -> Result<ReasonCode, EffectError> {
    ReasonCode::parse(text)
        .map_err(|err| denial(STATE_CONFLICT, format!("bad reason: {err}")))
        .map_err(EffectError::Denied)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn progress_status_tokens_match_the_registered_schema_set() {
        assert_eq!(ProgressStatus::Advanced.as_str(), "advanced");
        assert_eq!(ProgressStatus::NoProgress.as_str(), "none");
        assert_eq!(ProgressStatus::Uncertain.as_str(), "uncertain");
        assert_eq!(ProgressStatus::Blocked.as_str(), "blocked");
    }
}
