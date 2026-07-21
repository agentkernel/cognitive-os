//! Intent-chain admission orchestration for the Shell submit path (M5 RUN).
//!
//! Probability components may only produce [`InterpretationCandidate`].
//! This module runs the deterministic record → admit → mint sequence.

use cognitive_kernel::effects::{EffectError, WriterLease};
use cognitive_kernel::intent_chain::{
    AcceptanceCommand, AdmittedInterpretation, GovernanceSeed, InterpretationCandidate,
    SupersedeCommand, SupersedeReport, TaskContractCommand, UserIntentCommand,
    admit_interpretation, mint_task_contract, record_interpretation_candidate, record_user_intent,
    supersede_task_contract,
};
use cognitive_kernel::ports::{
    AuthorityStore, Clock, IdGenerator, IntentChainStore, InterpretationRow, ProtocolStore,
    TaskContractRow, UserIntentRecordRow,
};

/// Deterministic intent admission flow used after a shell submit receipt.
#[allow(clippy::too_many_arguments)]
pub fn admit_and_mint_contract<S, C, G>(
    store: &S,
    clock: &C,
    ids: &G,
    lease: &WriterLease,
    intent: &UserIntentCommand,
    candidate: &InterpretationCandidate,
    governance: &GovernanceSeed,
    acceptance: &AcceptanceCommand,
    contract: &TaskContractCommand,
    expected_current_epoch: i64,
) -> Result<(UserIntentRecordRow, InterpretationRow, TaskContractRow), EffectError>
where
    S: AuthorityStore + ProtocolStore + IntentChainStore,
    C: Clock,
    G: IdGenerator,
{
    let record = record_user_intent(store, clock, ids, lease, intent)?;
    let interpretation = record_interpretation_candidate(
        store,
        clock,
        ids,
        lease,
        &record.record_id,
        candidate,
        governance,
        &intent.correlation_id,
    )?;
    let admitted: AdmittedInterpretation = admit_interpretation(store, acceptance)?;
    let task = mint_task_contract(
        store,
        clock,
        ids,
        lease,
        &admitted,
        contract,
        expected_current_epoch,
    )?;
    Ok((record, interpretation, task))
}

/// Correction path: new record + candidate + admit + epoch CAS supersede.
pub fn correct_and_supersede<S, C, G>(
    store: &S,
    clock: &C,
    ids: &G,
    lease: &WriterLease,
    command: &SupersedeCommand,
) -> Result<SupersedeReport, EffectError>
where
    S: AuthorityStore + ProtocolStore + IntentChainStore,
    C: Clock,
    G: IdGenerator,
{
    supersede_task_contract(store, clock, ids, lease, command)
}
