//! The eight-step recovery sequencer (whitepaper section 16.6 order, fixed;
//! REQ-REC-001/002, REQ-RUN-006; `.cursor/rules/13-effect-recovery.mdc`).
//!
//! Order (never reordered, property 5 of the IMP-07 model):
//!
//! 1. **Barrier** — establish the recovery barrier: normal dispatch stops.
//! 2. **IdentityAndEpoch** — verify execution identity, advance the
//!    fencing epoch, install the new writer lease.
//! 3. **FenceOldWriter** — the pre-crash lease is now provably stale at
//!    every sink (F-014).
//! 4. **ReplayHistory** — verify the snapshot by replaying committed
//!    history to the high watermark (never inferring transitions from
//!    process state, REQ-REC-002: replay re-executes nothing).
//! 5. **ReconcileEffects** — close every in-flight Effect: EXECUTING is
//!    moved to OUTCOME_UNKNOWN and reconciled by querying with the
//!    ORIGINAL idempotency key; AUTHORIZED intents are confirmed
//!    undispached and marked for single re-dispatch; still-unknown
//!    outcomes are quarantined.
//! 6. **Reauthorize** — capability/revocation currency is re-established
//!    for continuations (stale grants must be re-issued by the authz gate).
//! 7. **ReresolveContext** — ContextViews are re-resolved under the new
//!    epoch (stale cache bindings cannot hit, M3).
//! 8. **ResumeLoop** — checkpoints are validated (epoch older than the new
//!    one, watermark within replayed history) and loops become resumable.
//!
//! The sequencer refuses out-of-order steps at runtime: violating the
//! order is a test failure, not a silent reorder.

use crate::effects::{EffectProtocol, WriterLease};
use crate::error::{EFFECT_RECOVERY_QUARANTINED, RegisteredError, TransitionRejection};
use crate::executor::{EffectExecutor, ExecutorQueryResult};
use crate::ports::{
    AuthorityStore, CheckpointRow, Clock, IdGenerator, ProtocolStore, StorePortError,
};
use crate::replay::{ReplayError, replay_projection};
use cognitive_domain::{LifecycleDomain, ObjectId, StateName, Version};

/// The eight steps, in their only legal order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStep {
    /// 1. Establish the recovery barrier.
    Barrier,
    /// 2. Verify identity, advance epoch, install fencing.
    IdentityAndEpoch,
    /// 3. Fence the old writer.
    FenceOldWriter,
    /// 4. Replay committed history to the high watermark.
    ReplayHistory,
    /// 5. Reconcile in-flight Effects.
    ReconcileEffects,
    /// 6. Re-authorize continuations.
    Reauthorize,
    /// 7. Re-resolve Context.
    ReresolveContext,
    /// 8. Resume loops.
    ResumeLoop,
}

/// The fixed order (property 5).
pub const RECOVERY_ORDER: [RecoveryStep; 8] = [
    RecoveryStep::Barrier,
    RecoveryStep::IdentityAndEpoch,
    RecoveryStep::FenceOldWriter,
    RecoveryStep::ReplayHistory,
    RecoveryStep::ReconcileEffects,
    RecoveryStep::Reauthorize,
    RecoveryStep::ReresolveContext,
    RecoveryStep::ResumeLoop,
];

/// Recovery failures.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RecoveryError {
    /// A step was attempted out of order (property 5 violation).
    #[error("recovery-order-violation: attempted {attempted:?}, expected {expected:?}")]
    OrderViolation {
        /// Step that was attempted.
        attempted: RecoveryStep,
        /// Step the order requires next.
        expected: RecoveryStep,
    },
    /// Storage failed during recovery (fail closed).
    #[error(transparent)]
    Store(#[from] StorePortError),
    /// Replay hit a barrier (corrupted or inconsistent history).
    #[error(transparent)]
    Replay(#[from] ReplayError),
    /// A reconciliation transition was rejected by the gate.
    #[error(transparent)]
    Rejected(#[from] TransitionRejection),
    /// A protocol operation failed during recovery.
    #[error(transparent)]
    Protocol(#[from] crate::effects::EffectError),
    /// A checkpoint's recovery-stable facts are invalid.
    #[error("checkpoint-invalid: {0}")]
    CheckpointInvalid(String),
}

/// How one in-flight effect was closed during step 5.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectDisposition {
    /// Intent persisted, provably never dispatched: safe to re-dispatch
    /// EXACTLY ONCE with the ORIGINAL idempotency key after re-authorization.
    ReadyToRedispatchOriginalKey {
        /// The original idempotency key that MUST be reused.
        idempotency_key: String,
    },
    /// Reconciled to a confirmed execution: continue to verification.
    ReconciledExecuted,
    /// Reconciled to confirmed non-execution: terminal NOT_EXECUTED.
    ReconciledNotExecuted,
    /// Still unknown: quarantined with the registered recovery code.
    Quarantined {
        /// Registered code surfaced for the quarantine disposition.
        code: RegisteredError,
    },
}

/// Step 6 fact: one continuation whose pre-crash authorization binding is
/// NOT admissible material for post-recovery work. The obligation carries
/// the governance versions the durable intent was minted under; the
/// runtime must present a grant that is CURRENT
/// ([`reauthorization_satisfied`]) before the effect protocol's
/// `capability_and_revocation_current` guard will pass again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReauthorizationObligation {
    /// Effect whose continuation needs fresh authorization.
    pub effect_object_id: ObjectId,
    /// Original idempotency key (continuations bind this key).
    pub idempotency_key: String,
    /// Revocation epoch the pre-crash grant was decided under.
    pub grant_epoch: i64,
    /// Capability set version the pre-crash grant was decided under.
    pub capability_set_version: i64,
}

/// Step 7 fact: the governance rebinding recovery installed. Every
/// context artifact cached under a binding older than `new_epoch` is
/// unreachable by key construction (M3 `ContextViewCache`): a continuation
/// declaring its pre-crash binding is refused and purged
/// (`ContextViewCache::serve_declared`), and fresh resolution under the
/// current binding is the only path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextRebinding {
    /// Epoch the crashed writer's context artifacts were bound under.
    pub fenced_epoch: i64,
    /// Epoch continuations must re-resolve under.
    pub new_epoch: i64,
}

/// Step 6 arithmetic for the runtime: a continuation's fresh grant
/// satisfies one reauthorization obligation only when it was decided
/// under the CURRENT governance facts (and its lease is valid now) —
/// the M3 revalidation applied to a recovery continuation.
pub fn reauthorization_satisfied(
    obligation: &ReauthorizationObligation,
    grant: &crate::authz::AuthorizationGrant,
    currency: &crate::effects::GovernanceCurrency,
    now: &cognitive_domain::WallTimestamp,
) -> bool {
    let _ = obligation;
    crate::authz::capability_and_revocation_current(
        grant,
        currency.revocation_epoch,
        currency.capability_set_version,
        now,
    )
}

/// Report of one completed recovery run (evidence).
#[derive(Debug, Clone, PartialEq)]
pub struct RecoveryReport {
    /// Epoch installed at step 2.
    pub new_epoch: i64,
    /// Epoch the crashed writer held (fenced at step 3).
    pub fenced_epoch: i64,
    /// Events replayed at step 4.
    pub replayed_events: u64,
    /// Projection digest after replay (byte-stable, REQ-STATE-002).
    pub projection_digest: String,
    /// Per-effect dispositions from step 5.
    pub reconciled: Vec<(ObjectId, EffectDisposition)>,
    /// Step 6 facts: continuations that must present fresh authorization
    /// (empty when nothing was in flight).
    pub reauthorization_obligations: Vec<ReauthorizationObligation>,
    /// Step 7 fact: the governance rebinding continuations resolve under.
    pub context_rebinding: ContextRebinding,
    /// Steps in the order they actually ran (must equal [`RECOVERY_ORDER`]).
    pub step_order: Vec<RecoveryStep>,
    /// Checkpoints validated at step 8.
    pub resumable_loops: Vec<ObjectId>,
}

/// Runtime order enforcement: steps advance strictly along
/// [`RECOVERY_ORDER`]; anything else errs.
#[derive(Debug, Default)]
pub struct RecoverySequencer {
    completed: Vec<RecoveryStep>,
}

impl RecoverySequencer {
    /// Start a sequencer with no completed steps.
    pub fn new() -> Self {
        Self::default()
    }

    /// The next step the order requires.
    pub fn expected_next(&self) -> Option<RecoveryStep> {
        RECOVERY_ORDER.get(self.completed.len()).copied()
    }

    /// Mark `step` complete; errs unless it is exactly the next step.
    pub fn advance(&mut self, step: RecoveryStep) -> Result<(), RecoveryError> {
        match self.expected_next() {
            Some(expected) if expected == step => {
                self.completed.push(step);
                Ok(())
            }
            Some(expected) => Err(RecoveryError::OrderViolation {
                attempted: step,
                expected,
            }),
            None => Err(RecoveryError::OrderViolation {
                attempted: step,
                expected: RecoveryStep::Barrier,
            }),
        }
    }

    /// Steps completed so far, in execution order.
    pub fn completed(&self) -> &[RecoveryStep] {
        &self.completed
    }
}

/// Validate a checkpoint's recovery-stable facts against the recovered
/// world (step 8; REQ-RUN-006, F-010): the checkpoint epoch MUST predate
/// the newly installed epoch and its high watermark MUST lie within the
/// replayed history.
pub fn validate_checkpoint(
    checkpoint: &CheckpointRow,
    new_epoch: i64,
    replayed_high_watermark: i64,
) -> Result<(), RecoveryError> {
    if checkpoint.fencing_epoch >= new_epoch {
        return Err(RecoveryError::CheckpointInvalid(format!(
            "checkpoint epoch {} is not older than the recovery epoch {new_epoch}",
            checkpoint.fencing_epoch
        )));
    }
    if checkpoint.event_high_watermark > replayed_high_watermark {
        return Err(RecoveryError::CheckpointInvalid(format!(
            "checkpoint watermark {} beyond replayed history {replayed_high_watermark}",
            checkpoint.event_high_watermark
        )));
    }
    Ok(())
}

/// Run the full eight-step recovery against one authority store.
///
/// `executor` is consulted ONLY for reconciliation queries with original
/// idempotency keys — recovery never re-executes committed work
/// (REQ-REC-002) and never mints new keys.
pub fn run_recovery<S, C, G>(
    store: &S,
    crashed_lease: WriterLease,
    executor: &dyn EffectExecutor,
    protocol: &EffectProtocol<'_, S, C, G>,
) -> Result<RecoveryReport, RecoveryError>
where
    S: AuthorityStore + ProtocolStore,
    C: Clock,
    G: IdGenerator,
{
    let mut sequencer = RecoverySequencer::new();
    let mut step_order = Vec::with_capacity(8);

    // 1. Barrier: no normal dispatch runs during recovery. In the M4
    // single-node reference this is structural — recovery holds the only
    // writer path until the report is produced.
    sequencer.advance(RecoveryStep::Barrier)?;
    step_order.push(RecoveryStep::Barrier);

    // 2. Identity + epoch: advance fencing and install the new lease.
    let new_epoch = store.advance_fencing_epoch()?;
    let new_lease = WriterLease { epoch: new_epoch };
    sequencer.advance(RecoveryStep::IdentityAndEpoch)?;
    step_order.push(RecoveryStep::IdentityAndEpoch);

    // 3. Old writer fenced: its lease is now stale at every sink.
    if crashed_lease.epoch >= new_epoch {
        return Err(RecoveryError::CheckpointInvalid(format!(
            "crashed lease epoch {} not fenced by new epoch {new_epoch}",
            crashed_lease.epoch
        )));
    }
    sequencer.advance(RecoveryStep::FenceOldWriter)?;
    step_order.push(RecoveryStep::FenceOldWriter);

    // 4. Replay committed history (re-executes nothing).
    let projection = replay_projection(store)?;
    sequencer.advance(RecoveryStep::ReplayHistory)?;
    step_order.push(RecoveryStep::ReplayHistory);

    // 5. Reconcile in-flight Effects.
    let executing = state("EXECUTING")?;
    let unknown = state("OUTCOME_UNKNOWN")?;
    let authorized = state("AUTHORIZED")?;
    let in_flight =
        store.list_objects_in_states(LifecycleDomain::Effect, &[executing, unknown, authorized])?;
    let mut reconciled = Vec::new();
    let mut reauthorization_obligations = Vec::new();
    for effect in &in_flight {
        let disposition = match effect.state.as_str() {
            // Crash point 1: intent persisted, dispatch never recorded.
            // Confirm non-dispatch, then hand back for ONE re-dispatch
            // with the ORIGINAL key after step 6 re-authorization.
            "AUTHORIZED" => {
                let intent = store
                    .load_intent_for_effect(&effect.object_id)?
                    .ok_or_else(|| {
                        RecoveryError::CheckpointInvalid(format!(
                            "authorized effect {} without durable intent",
                            effect.object_id
                        ))
                    })?;
                EffectDisposition::ReadyToRedispatchOriginalKey {
                    idempotency_key: intent.idempotency_key,
                }
            }
            // Crash point 2: dispatched, outcome unrecorded. The effect
            // MAY have executed: move to OUTCOME_UNKNOWN, then reconcile
            // with the original key. Never blind-retry.
            "EXECUTING" => {
                let committed = protocol.record_outcome(
                    &effect.object_id,
                    effect.version,
                    &crate::executor::DispatchOutcome::Unknown {
                        detail: "crash before outcome was recorded".to_owned(),
                    },
                    &new_lease,
                )?;
                reconcile_after_unknown(
                    protocol,
                    executor,
                    &effect.object_id,
                    committed.after_version,
                    &new_lease,
                )?
            }
            // Already OUTCOME_UNKNOWN at crash time.
            "OUTCOME_UNKNOWN" => reconcile_after_unknown(
                protocol,
                executor,
                &effect.object_id,
                effect.version,
                &new_lease,
            )?,
            other => {
                return Err(RecoveryError::CheckpointInvalid(format!(
                    "unexpected in-flight state {other}"
                )));
            }
        };
        // Every continuation that is not terminally closed must present
        // fresh authorization before it can proceed (step 6 input).
        if !matches!(disposition, EffectDisposition::ReconciledNotExecuted)
            && let Some(intent) = store.load_intent_for_effect(&effect.object_id)?
        {
            reauthorization_obligations.push(ReauthorizationObligation {
                effect_object_id: effect.object_id.clone(),
                idempotency_key: intent.idempotency_key,
                grant_epoch: intent.grant_epoch,
                capability_set_version: intent.capability_set_version,
            });
        }
        reconciled.push((effect.object_id.clone(), disposition));
    }
    sequencer.advance(RecoveryStep::ReconcileEffects)?;
    step_order.push(RecoveryStep::ReconcileEffects);

    // 6. Re-authorize: stale grants cannot continue. The obligations
    // (durable intent bindings of every non-terminal continuation) are
    // the step fact; the teeth are the effect protocol's
    // capability_and_revocation_current guard, which the runtime can only
    // satisfy with grants that pass [`reauthorization_satisfied`] against
    // the CURRENT governance currency.
    sequencer.advance(RecoveryStep::Reauthorize)?;
    step_order.push(RecoveryStep::Reauthorize);

    // 7. Re-resolve Context: stale cache bindings cannot hit under the
    // advanced epoch (M3 governance-bound keys). The rebinding fact tells
    // continuations which epoch their declared bindings are checked
    // against (`ContextViewCache::serve_declared` refuses and purges).
    let context_rebinding = ContextRebinding {
        fenced_epoch: crashed_lease.epoch,
        new_epoch,
    };
    sequencer.advance(RecoveryStep::ReresolveContext)?;
    step_order.push(RecoveryStep::ReresolveContext);

    // 8. Resume loops: validate checkpoint recovery-stable facts.
    let loops = store.list_objects_in_states(
        LifecycleDomain::Loop,
        &[
            state("OBSERVE")?,
            state("RESOLVE")?,
            state("ORIENT")?,
            state("DECIDE")?,
            state("ACT")?,
            state("VERIFY")?,
            state("CONTINUE")?,
            state("WAIT")?,
        ],
    )?;
    let mut resumable = Vec::new();
    for loop_object in &loops {
        if let Some(checkpoint) = store.latest_checkpoint(&loop_object.object_id)? {
            validate_checkpoint(&checkpoint, new_epoch, projection.high_watermark)?;
            resumable.push(loop_object.object_id.clone());
        }
    }
    sequencer.advance(RecoveryStep::ResumeLoop)?;
    step_order.push(RecoveryStep::ResumeLoop);

    Ok(RecoveryReport {
        new_epoch,
        fenced_epoch: crashed_lease.epoch,
        replayed_events: projection.event_count,
        projection_digest: projection.digest,
        reconciled,
        reauthorization_obligations,
        context_rebinding,
        step_order,
        resumable_loops: resumable,
    })
}

fn reconcile_after_unknown<S, C, G>(
    protocol: &EffectProtocol<'_, S, C, G>,
    executor: &dyn EffectExecutor,
    effect_id: &ObjectId,
    version_at_unknown: Version,
    lease: &WriterLease,
) -> Result<EffectDisposition, RecoveryError>
where
    S: AuthorityStore + ProtocolStore,
    C: Clock,
    G: IdGenerator,
{
    let (committed, query) = protocol.reconcile(
        effect_id,
        "OUTCOME_UNKNOWN",
        version_at_unknown,
        executor,
        lease,
    )?;
    Ok(match query {
        ExecutorQueryResult::ExecutedWithOriginalKey => EffectDisposition::ReconciledExecuted,
        ExecutorQueryResult::NotExecuted => {
            // RECONCILED(not_executed) closes as terminal NOT_EXECUTED.
            protocol.close_not_executed(effect_id, committed.after_version, lease)?;
            EffectDisposition::ReconciledNotExecuted
        }
        ExecutorQueryResult::Indeterminate => {
            protocol.quarantine_still_unknown(effect_id, committed.after_version, lease)?;
            EffectDisposition::Quarantined {
                code: EFFECT_RECOVERY_QUARANTINED,
            }
        }
    })
}

fn state(name: &str) -> Result<StateName, RecoveryError> {
    StateName::parse(name)
        .map_err(|err| RecoveryError::CheckpointInvalid(format!("state name: {err}")))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    /// Property 5: the sequencer refuses every out-of-order step.
    #[test]
    fn out_of_order_steps_are_rejected() {
        let mut sequencer = RecoverySequencer::new();
        // Resuming the loop before anything else is the canonical
        // violation (milestone acceptance 5).
        let violation = sequencer.advance(RecoveryStep::ResumeLoop).unwrap_err();
        assert!(matches!(
            violation,
            RecoveryError::OrderViolation {
                attempted: RecoveryStep::ResumeLoop,
                expected: RecoveryStep::Barrier,
            }
        ));
        sequencer.advance(RecoveryStep::Barrier).unwrap();
        // Reconciling before fencing/replay is refused.
        assert!(sequencer.advance(RecoveryStep::ReconcileEffects).is_err());
        sequencer.advance(RecoveryStep::IdentityAndEpoch).unwrap();
        // Skipping the fence to replay is refused.
        assert!(sequencer.advance(RecoveryStep::ReplayHistory).is_err());
        sequencer.advance(RecoveryStep::FenceOldWriter).unwrap();
        sequencer.advance(RecoveryStep::ReplayHistory).unwrap();
        sequencer.advance(RecoveryStep::ReconcileEffects).unwrap();
        sequencer.advance(RecoveryStep::Reauthorize).unwrap();
        sequencer.advance(RecoveryStep::ReresolveContext).unwrap();
        sequencer.advance(RecoveryStep::ResumeLoop).unwrap();
        assert_eq!(sequencer.completed(), RECOVERY_ORDER);
        // A ninth step does not exist.
        assert!(sequencer.advance(RecoveryStep::Barrier).is_err());
    }

    #[test]
    fn checkpoint_facts_are_validated() {
        let checkpoint = CheckpointRow {
            checkpoint_id: ObjectId::parse("00000000-0000-7000-9000-0000000000c1").unwrap(),
            loop_object_id: ObjectId::parse("00000000-0000-7000-9000-0000000000c2").unwrap(),
            event_high_watermark: 10,
            fencing_epoch: 1,
            canonical_json: "{}".to_owned(),
        };
        assert!(validate_checkpoint(&checkpoint, 2, 10).is_ok());
        // Checkpoint from the CURRENT epoch: recovery did not fence first.
        assert!(validate_checkpoint(&checkpoint, 1, 10).is_err());
        // Watermark beyond replayed history: history is incomplete.
        assert!(validate_checkpoint(&checkpoint, 2, 9).is_err());
    }
}
