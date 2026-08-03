//! Daemon-only durable scheduler authority reads (P2-T03).
//!
//! This module deliberately performs no lease acquisition or worker dispatch.
//! It reloads the immutable TaskContract and the identities it binds, deriving
//! ceiling inputs solely from authority-store facts.

#![allow(dead_code, clippy::items_after_test_module)] // Activated only after the fenced quiescence protocol exists.

use cognitive_contracts::generated::task_contract::TaskContract;
use cognitive_domain::{BudgetId, LifecycleDomain, ObjectId, Version, WallTimestamp};
use cognitive_kernel::effects::WriterLease;
use cognitive_kernel::engine::CommittedTransition;
use cognitive_kernel::harness::LoopDriver;
use cognitive_kernel::ports::{
    AuthorityStore, Clock, HarnessStore, IdGenerator, IntentChainStore, ProtocolStore, TaskBinding,
};
use cognitive_runtime::{
    SchedulerCeilingDispatch, SchedulerCeilingDispatchError, SchedulerCeilingFacts,
    SchedulerDispatch, SchedulerService, SchedulerServiceError,
};
use cognitive_store::scheduler::{SchedulerRepository, SchedulerRepositoryError, SchedulerState};
use serde::Deserialize;
use thiserror::Error;

const TASK_CONTRACT_EXECUTION_SCHEMA_VERSION: &str = "cognitiveos.task-contract/0.2";

#[derive(Deserialize)]
struct TaskContractVersionEnvelope {
    header: TaskContractVersionHeader,
}

#[derive(Deserialize)]
struct TaskContractVersionHeader {
    schema_version: String,
}

/// Exact identities fixed by an immutable task contract epoch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SchedulerAuthorityBinding {
    pub task_ref: String,
    pub action_fingerprint: String,
}

/// Durable authority inputs required to decide one scheduler admission.
///
/// These facts are reloaded from the current immutable TaskContract and the
/// authority store for every attempt. They are never taken from the worker or
/// a prior scheduler projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SchedulerAuthoritySnapshot {
    pub ceiling_facts: SchedulerCeilingFacts,
    pub loop_object_id: ObjectId,
    pub budget_id: BudgetId,
}

/// One daemon-owned scheduler admission result.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SchedulerDispatchAdmission {
    Stopped(CommittedTransition),
    Leased(SchedulerDispatch),
}

/// The only scheduler-facing result a daemon worker may accept from Effect
/// processing. The worker callback must derive either state from the durable
/// Effect protocol; an external receipt is not a closed Effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SchedulerEffectClosure {
    Closed,
    PendingReconciliation,
}

/// The single durable Effect resolved for one scheduler TaskContract epoch.
///
/// A later worker integration must use this object identity to derive an
/// outcome from the authority store; it must not substitute a receipt or
/// process-local result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SchedulerEffectResolution {
    pub effect_object_id: ObjectId,
    pub closure: SchedulerEffectClosure,
}

/// Daemon-owned outcome after scheduler admission reaches the Effect boundary.
///
/// This remains distinct from Task acceptance. A closed Effect only permits a
/// later fenced scheduler release; independent verification still decides
/// whether a Task may complete.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SchedulerWorkerAttempt {
    Stopped(CommittedTransition),
    EffectClosed(SchedulerDispatch),
    AwaitingReconciliation(SchedulerDispatch),
}

/// Fail-closed authority-read failures before scheduler lease acquisition.
#[derive(Debug, Error)]
pub(crate) enum SchedulerAuthorityError {
    #[error("scheduler task reference must not be empty")]
    EmptyTaskReference,
    #[error("scheduler action fingerprint must not be empty")]
    EmptyActionFingerprint,
    #[error("scheduler authority store failed: {0}")]
    Store(String),
    #[error("scheduler task has no current contract: {0}")]
    MissingContract(String),
    #[error("scheduler contract is not execution-bound: {0}")]
    LegacyContract(String),
    #[error("scheduler contract is malformed: {0}")]
    MalformedContract(String),
    #[error("scheduler bound loop is unavailable or not dispatchable: {0}")]
    LoopUnavailable(String),
    #[error("scheduler bound budget is unavailable or inconsistent: {0}")]
    BudgetUnavailable(String),
    #[error("scheduler task contract epoch must be positive: {0}")]
    InvalidContractEpoch(i64),
    #[error(
        "scheduler task contract epoch has no durable Effect binding: {task_ref} at {contract_epoch}"
    )]
    MissingEffectBinding {
        task_ref: String,
        contract_epoch: i64,
    },
    #[error(
        "scheduler task contract epoch has ambiguous durable Effect bindings: {task_ref} at {contract_epoch}"
    )]
    AmbiguousEffectBindings {
        task_ref: String,
        contract_epoch: i64,
    },
    #[error("scheduler durable Intent binding is inconsistent: {0}")]
    InconsistentEffectBinding(String),
    #[error("scheduler durable Effect is unavailable: {0}")]
    MissingEffect(String),
    #[error("scheduler durable Effect state is unsupported: {0}")]
    UnsupportedEffectState(String),
    #[error("scheduler dispatch does not match the resolved TaskContract binding: {0}")]
    DispatchBindingMismatch(String),
    #[error("scheduler lease release time is invalid: {0}")]
    InvalidReleaseTime(String),
    #[error(transparent)]
    Repository(#[from] SchedulerRepositoryError),
    #[error(transparent)]
    Scheduler(#[from] SchedulerServiceError),
    #[error(transparent)]
    CeilingStop(#[from] SchedulerCeilingDispatchError),
}

/// Parse only a current execution-bound contract before reading its bindings.
///
/// The explicit version check preserves old contract rows for audit while
/// preventing their deserialization from becoming a scheduler admission path.
fn parse_execution_bound_contract(
    canonical_json: &str,
) -> Result<TaskContract, SchedulerAuthorityError> {
    let version_envelope: TaskContractVersionEnvelope = serde_json::from_str(canonical_json)
        .map_err(|error| SchedulerAuthorityError::MalformedContract(error.to_string()))?;
    if version_envelope.header.schema_version != TASK_CONTRACT_EXECUTION_SCHEMA_VERSION {
        return Err(SchedulerAuthorityError::LegacyContract(
            version_envelope.header.schema_version,
        ));
    }

    serde_json::from_str(canonical_json)
        .map_err(|error| SchedulerAuthorityError::MalformedContract(error.to_string()))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{
        SchedulerAuthorityError, SchedulerDispatchAdmission, SchedulerEffectClosure,
        SchedulerWorkerAttempt, classify_scheduler_effect_closure,
        complete_resolved_effect_and_release, complete_scheduler_admission,
        complete_scheduler_worker_attempt, parse_execution_bound_contract,
        release_closed_effect_dispatch, select_single_effect_intent,
    };
    use cognitive_domain::{EventId, ObjectId, RecordId, Version, WallTimestamp};
    use cognitive_kernel::engine::CommittedTransition;
    use cognitive_kernel::ports::{IntentRow, TaskBinding};
    use cognitive_runtime::{SchedulerCeilingDispatch, SchedulerDispatch};
    use cognitive_store::scheduler::{SchedulerRepository, SchedulerRow, SchedulerState};
    use std::cell::Cell;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn scheduler_row(task_ref: &str) -> SchedulerRow {
        SchedulerRow {
            task_ref: task_ref.to_owned(),
            state: SchedulerState::Runnable.as_str().to_owned(),
            lease_owner: None,
            lease_epoch: 0,
            lease_expires: None,
            next_eligible: "2026-08-03T00:00:00Z".to_owned(),
            attempt_count: 0,
            cancel_requested: false,
        }
    }

    fn temporary_scheduler_database_path() -> std::path::PathBuf {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "cognitiveos-scheduler-authority-{}-{unique_suffix}.db",
            std::process::id()
        ))
    }

    fn committed_ceiling_stop() -> CommittedTransition {
        CommittedTransition {
            record_id: RecordId::parse("00000000-0000-7000-8000-000000000001").unwrap(),
            event_id: EventId::parse("00000000-0000-7000-8000-000000000002").unwrap(),
            event_sequence: 1,
            after_version: Version::new(2).unwrap(),
            committed_at: WallTimestamp::parse("2026-08-02T00:00:00Z").unwrap(),
        }
    }

    fn task_binding() -> TaskBinding {
        TaskBinding {
            task_ref: "task://personal/durable-effect-resolution".to_owned(),
            contract_epoch: 4,
        }
    }

    fn effect_intent(intent_suffix: u64, binding: Option<TaskBinding>) -> IntentRow {
        IntentRow {
            intent_id: ObjectId::parse(&format!("00000000-0000-7000-8000-{intent_suffix:012x}"))
                .unwrap(),
            idempotency_key: format!("scheduler-effect-{intent_suffix}"),
            parameters_digest: format!("sha256:{}", "ab".repeat(32)),
            action: "scheduler.effect".to_owned(),
            target: "effect://personal/scheduler".to_owned(),
            effect_object_id: ObjectId::parse(&format!(
                "00000000-0000-7000-9000-{intent_suffix:012x}"
            ))
            .unwrap(),
            expected_state_version: Version::INITIAL,
            grant_epoch: 1,
            capability_set_version: 1,
            task_binding: binding,
            canonical_json: "{}".to_owned(),
        }
    }

    #[test]
    fn legacy_contract_is_rejected_before_execution_binding_deserialization() {
        let legacy_contract = r#"{
            "header": {
                "schema_version": "cognitiveos.task-contract/0.1"
            }
        }"#;

        assert!(matches!(parse_execution_bound_contract(legacy_contract),
            Err(SchedulerAuthorityError::LegacyContract(version))
            if version == "cognitiveos.task-contract/0.1"));
    }

    #[test]
    fn execution_schema_without_required_bindings_is_rejected_as_malformed() {
        let incomplete_execution_contract = r#"{
            "header": {
                "schema_version": "cognitiveos.task-contract/0.2"
            }
        }"#;

        assert!(matches!(
            parse_execution_bound_contract(incomplete_execution_contract),
            Err(SchedulerAuthorityError::MalformedContract(_))
        ));
    }

    #[test]
    fn effect_resolution_rejects_missing_ambiguous_and_inconsistent_bindings() {
        let binding = task_binding();

        assert!(matches!(
            select_single_effect_intent(&binding, &[]),
            Err(SchedulerAuthorityError::MissingEffectBinding {
                task_ref,
                contract_epoch: 4,
            }) if task_ref == binding.task_ref
        ));

        let first_intent = effect_intent(11, Some(binding.clone()));
        let second_intent = effect_intent(12, Some(binding.clone()));
        assert!(matches!(
            select_single_effect_intent(&binding, &[first_intent, second_intent]),
            Err(SchedulerAuthorityError::AmbiguousEffectBindings {
                task_ref,
                contract_epoch: 4,
            }) if task_ref == binding.task_ref
        ));

        let inconsistent_intent = effect_intent(
            13,
            Some(TaskBinding {
                task_ref: binding.task_ref.clone(),
                contract_epoch: 5,
            }),
        );
        assert!(matches!(
            select_single_effect_intent(&binding, &[inconsistent_intent]),
            Err(SchedulerAuthorityError::InconsistentEffectBinding(_))
        ));
    }

    #[test]
    fn reached_ceiling_returns_durable_stop_without_attempting_a_scheduler_lease() {
        let lease_acquisition_attempted = Cell::new(false);

        let admission = complete_scheduler_admission(
            SchedulerCeilingDispatch::Stopped(committed_ceiling_stop()),
            || {
                lease_acquisition_attempted.set(true);
                unreachable!("a reached ceiling must not acquire a scheduler lease")
            },
        )
        .unwrap();

        assert!(matches!(admission, SchedulerDispatchAdmission::Stopped(_)));
        assert!(
            !lease_acquisition_attempted.get(),
            "a terminal ceiling STOP must precede every lease attempt"
        );
    }

    #[test]
    fn clear_ceiling_acquires_exactly_one_scheduler_lease() {
        let lease_acquisition_count = Cell::new(0);

        let admission = complete_scheduler_admission(SchedulerCeilingDispatch::Proceed, || {
            lease_acquisition_count.set(lease_acquisition_count.get() + 1);
            Ok(SchedulerDispatch {
                task_ref: "task://personal/admission-order".to_owned(),
                lease_owner: "scheduler-worker".to_owned(),
                lease_epoch: 3,
                lease_expires: "2026-08-02T00:01:00Z".to_owned(),
                attempt_count: 1,
            })
        })
        .unwrap();

        assert!(matches!(admission, SchedulerDispatchAdmission::Leased(_)));
        assert_eq!(lease_acquisition_count.get(), 1);
    }

    #[test]
    fn ceiling_stop_skips_the_effect_closure_callback() {
        let effect_closure_attempted = Cell::new(false);

        let attempt = complete_scheduler_worker_attempt(
            SchedulerDispatchAdmission::Stopped(committed_ceiling_stop()),
            |_| {
                effect_closure_attempted.set(true);
                unreachable!("a stopped scheduler attempt must not process an Effect")
            },
        )
        .unwrap();

        assert!(matches!(attempt, SchedulerWorkerAttempt::Stopped(_)));
        assert!(
            !effect_closure_attempted.get(),
            "a durable ceiling STOP must precede every Effect-closure callback"
        );
    }

    #[test]
    fn unresolved_effect_keeps_the_fenced_dispatch_for_reconciliation() {
        let dispatch = SchedulerDispatch {
            task_ref: "task://personal/effect-reconciliation".to_owned(),
            lease_owner: "scheduler-worker".to_owned(),
            lease_epoch: 7,
            lease_expires: "2026-08-02T00:01:00Z".to_owned(),
            attempt_count: 1,
        };

        let attempt = complete_scheduler_worker_attempt(
            SchedulerDispatchAdmission::Leased(dispatch.clone()),
            |received_dispatch| {
                assert_eq!(received_dispatch, dispatch);
                Ok(SchedulerEffectClosure::PendingReconciliation)
            },
        )
        .unwrap();

        assert_eq!(
            attempt,
            SchedulerWorkerAttempt::AwaitingReconciliation(dispatch),
            "an unresolved Effect must not be converted into a scheduler success"
        );
    }

    #[test]
    fn only_a_closed_effect_can_release_the_exact_fenced_scheduler_dispatch() {
        let dispatch = SchedulerDispatch {
            task_ref: "task://personal/closed-effect".to_owned(),
            lease_owner: "scheduler-worker".to_owned(),
            lease_epoch: 8,
            lease_expires: "2026-08-02T00:01:00Z".to_owned(),
            attempt_count: 1,
        };
        let release_count = Cell::new(0);

        let released = release_closed_effect_dispatch(
            SchedulerWorkerAttempt::EffectClosed(dispatch.clone()),
            |received_dispatch| {
                release_count.set(release_count.get() + 1);
                assert_eq!(received_dispatch, dispatch);
                Ok(())
            },
        )
        .unwrap();

        assert_eq!(released, SchedulerWorkerAttempt::EffectClosed(dispatch));
        assert_eq!(release_count.get(), 1);
    }

    #[test]
    fn pending_effect_reconciliation_does_not_release_its_scheduler_lease() {
        let dispatch = SchedulerDispatch {
            task_ref: "task://personal/pending-effect".to_owned(),
            lease_owner: "scheduler-worker".to_owned(),
            lease_epoch: 9,
            lease_expires: "2026-08-02T00:01:00Z".to_owned(),
            attempt_count: 1,
        };
        let release_attempted = Cell::new(false);

        let retained = release_closed_effect_dispatch(
            SchedulerWorkerAttempt::AwaitingReconciliation(dispatch.clone()),
            |_| {
                release_attempted.set(true);
                unreachable!("a pending Effect must retain its fenced scheduler lease")
            },
        )
        .unwrap();

        assert_eq!(
            retained,
            SchedulerWorkerAttempt::AwaitingReconciliation(dispatch)
        );
        assert!(
            !release_attempted.get(),
            "a pending Effect must not release its scheduler lease"
        );
    }

    #[test]
    fn closed_effect_releases_the_matching_durable_lease_without_completing_the_task() {
        let database_path = temporary_scheduler_database_path();
        let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
        let task_ref = "task://personal/durable-effect-closure";
        scheduler_repository
            .upsert(&scheduler_row(task_ref))
            .unwrap();
        scheduler_repository
            .acquire_lease(task_ref, "scheduler-worker", 10, "2026-08-03T00:00:00Z")
            .unwrap();
        let dispatch = SchedulerDispatch {
            task_ref: task_ref.to_owned(),
            lease_owner: "scheduler-worker".to_owned(),
            lease_epoch: 10,
            lease_expires: "2026-08-03T00:01:00Z".to_owned(),
            attempt_count: 1,
        };

        let completed_attempt = complete_resolved_effect_and_release(
            SchedulerWorkerAttempt::EffectClosed(dispatch.clone()),
            &mut scheduler_repository,
            "2026-08-03T00:00:30Z",
        )
        .unwrap();

        assert_eq!(
            completed_attempt,
            SchedulerWorkerAttempt::EffectClosed(dispatch),
            "a closed Effect ends this scheduler attempt, not Task acceptance"
        );
        let durable_row = scheduler_repository.load(task_ref).unwrap().unwrap();
        assert_eq!(durable_row.state, SchedulerState::Succeeded.as_str());
        assert_eq!(durable_row.lease_owner, None);
        drop(scheduler_repository);
        std::fs::remove_file(database_path).unwrap();
    }

    #[test]
    fn malformed_release_time_preserves_the_closed_effects_fenced_lease() {
        let database_path = temporary_scheduler_database_path();
        let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
        let task_ref = "task://personal/malformed-release-time";
        scheduler_repository
            .upsert(&scheduler_row(task_ref))
            .unwrap();
        scheduler_repository
            .acquire_lease(task_ref, "scheduler-worker", 11, "2026-08-03T00:00:00Z")
            .unwrap();
        let dispatch = SchedulerDispatch {
            task_ref: task_ref.to_owned(),
            lease_owner: "scheduler-worker".to_owned(),
            lease_epoch: 11,
            lease_expires: "2026-08-03T00:01:00Z".to_owned(),
            attempt_count: 1,
        };

        assert!(matches!(
            complete_resolved_effect_and_release(
                SchedulerWorkerAttempt::EffectClosed(dispatch),
                &mut scheduler_repository,
                "not-a-timestamp",
            ),
            Err(SchedulerAuthorityError::InvalidReleaseTime(value)) if value == "not-a-timestamp"
        ));

        let durable_row = scheduler_repository.load(task_ref).unwrap().unwrap();
        assert_eq!(durable_row.state, SchedulerState::Leased.as_str());
        assert_eq!(durable_row.lease_owner.as_deref(), Some("scheduler-worker"));
        assert_eq!(durable_row.lease_epoch, 11);
        drop(scheduler_repository);
        std::fs::remove_file(database_path).unwrap();
    }

    #[test]
    fn stale_closed_effect_release_preserves_a_successor_fenced_lease() {
        let database_path = temporary_scheduler_database_path();
        let mut scheduler_repository = SchedulerRepository::open(&database_path).unwrap();
        let task_ref = "task://personal/stale-closed-effect";
        scheduler_repository
            .upsert(&scheduler_row(task_ref))
            .unwrap();
        scheduler_repository
            .acquire_eligible_lease(
                task_ref,
                "scheduler-worker",
                12,
                "2026-08-03T00:00:00Z",
                "2026-08-03T00:00:30Z",
            )
            .unwrap();
        scheduler_repository
            .acquire_eligible_lease(
                task_ref,
                "scheduler-worker",
                13,
                "2026-08-03T00:00:30Z",
                "2026-08-03T00:01:30Z",
            )
            .unwrap();
        let stale_dispatch = SchedulerDispatch {
            task_ref: task_ref.to_owned(),
            lease_owner: "scheduler-worker".to_owned(),
            lease_epoch: 12,
            lease_expires: "2026-08-03T00:01:00Z".to_owned(),
            attempt_count: 1,
        };

        assert!(matches!(
            complete_resolved_effect_and_release(
                SchedulerWorkerAttempt::EffectClosed(stale_dispatch),
                &mut scheduler_repository,
                "2026-08-03T00:01:30Z",
            ),
            Err(SchedulerAuthorityError::Repository(_))
        ));

        let durable_row = scheduler_repository.load(task_ref).unwrap().unwrap();
        assert_eq!(durable_row.state, SchedulerState::Leased.as_str());
        assert_eq!(durable_row.lease_owner.as_deref(), Some("scheduler-worker"));
        assert_eq!(durable_row.lease_epoch, 13);
        drop(scheduler_repository);
        std::fs::remove_file(database_path).unwrap();
    }

    #[test]
    fn only_durable_terminal_effect_states_close_a_scheduler_attempt() {
        assert_eq!(
            classify_scheduler_effect_closure("RECONCILED").unwrap(),
            SchedulerEffectClosure::Closed
        );
        assert_eq!(
            classify_scheduler_effect_closure("EXECUTING").unwrap(),
            SchedulerEffectClosure::PendingReconciliation
        );
        assert!(matches!(
            classify_scheduler_effect_closure("UNRECOGNIZED"),
            Err(SchedulerAuthorityError::UnsupportedEffectState(state)) if state == "UNRECOGNIZED"
        ));
    }
}

/// Resolve the exact durable Effect bound to one scheduler TaskContract epoch.
///
/// This is deliberately a read-only authority boundary. It rejects zero or
/// multiple bindings, missing objects, adapter-inconsistent rows, and unknown
/// states before any worker can turn a process result into a scheduler outcome.
pub(crate) fn resolve_scheduler_effect_for_task_binding<S>(
    store: &S,
    task_binding: &TaskBinding,
) -> Result<SchedulerEffectResolution, SchedulerAuthorityError>
where
    S: AuthorityStore + ProtocolStore,
{
    if task_binding.task_ref.is_empty() {
        return Err(SchedulerAuthorityError::EmptyTaskReference);
    }
    if task_binding.contract_epoch <= 0 {
        return Err(SchedulerAuthorityError::InvalidContractEpoch(
            task_binding.contract_epoch,
        ));
    }

    let intent_rows = store
        .list_intents_for_task_binding(task_binding)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    let intent_row = select_single_effect_intent(task_binding, &intent_rows)?;

    let effect_object = store
        .load_object(LifecycleDomain::Effect, &intent_row.effect_object_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::MissingEffect(intent_row.effect_object_id.as_str().to_owned())
        })?;
    let closure = classify_scheduler_effect_closure(effect_object.state.as_str())?;

    Ok(SchedulerEffectResolution {
        effect_object_id: intent_row.effect_object_id.clone(),
        closure,
    })
}

/// Select exactly one immutable Intent and verify that its stored binding
/// agrees with the reverse-index query used to find it.
fn select_single_effect_intent<'intent>(
    task_binding: &TaskBinding,
    intent_rows: &'intent [cognitive_kernel::ports::IntentRow],
) -> Result<&'intent cognitive_kernel::ports::IntentRow, SchedulerAuthorityError> {
    let intent_row = match intent_rows {
        [] => {
            return Err(SchedulerAuthorityError::MissingEffectBinding {
                task_ref: task_binding.task_ref.clone(),
                contract_epoch: task_binding.contract_epoch,
            });
        }
        [intent_row] => intent_row,
        _ => {
            return Err(SchedulerAuthorityError::AmbiguousEffectBindings {
                task_ref: task_binding.task_ref.clone(),
                contract_epoch: task_binding.contract_epoch,
            });
        }
    };
    if intent_row.task_binding.as_ref() != Some(task_binding) {
        return Err(SchedulerAuthorityError::InconsistentEffectBinding(
            intent_row.intent_id.as_str().to_owned(),
        ));
    }

    Ok(intent_row)
}

/// Classify an Effect only from its durable lifecycle state.
///
/// The states accepted as closed mirror the fail-closed checkpoint inventory:
/// reconciliation or verification has reached a terminal disposition. Every
/// in-flight state retains the fenced dispatch for reconciliation; unknown
/// values are rejected rather than treated as a successful closure.
fn classify_scheduler_effect_closure(
    state: &str,
) -> Result<SchedulerEffectClosure, SchedulerAuthorityError> {
    match state {
        "RECONCILED" | "VERIFIED" | "VERIFY_FAILED" => Ok(SchedulerEffectClosure::Closed),
        "PROPOSED" | "AUTHORIZED" | "EXECUTING" | "OUTCOME_UNKNOWN" | "EXECUTED"
        | "COMPENSATING" | "QUARANTINED" => Ok(SchedulerEffectClosure::PendingReconciliation),
        _ => Err(SchedulerAuthorityError::UnsupportedEffectState(
            state.to_owned(),
        )),
    }
}

/// Reload durable facts that must precede a scheduler dispatch decision.
pub(crate) fn load_scheduler_ceiling_facts<S>(
    store: &S,
    binding: &SchedulerAuthorityBinding,
) -> Result<SchedulerCeilingFacts, SchedulerAuthorityError>
where
    S: AuthorityStore + HarnessStore + IntentChainStore + ProtocolStore,
{
    Ok(load_scheduler_authority_snapshot(store, binding)?.ceiling_facts)
}

/// Reload the full durable input set required before scheduler admission.
pub(crate) fn load_scheduler_authority_snapshot<S>(
    store: &S,
    binding: &SchedulerAuthorityBinding,
) -> Result<SchedulerAuthoritySnapshot, SchedulerAuthorityError>
where
    S: AuthorityStore + HarnessStore + IntentChainStore + ProtocolStore,
{
    if binding.task_ref.is_empty() {
        return Err(SchedulerAuthorityError::EmptyTaskReference);
    }
    if binding.action_fingerprint.is_empty() {
        return Err(SchedulerAuthorityError::EmptyActionFingerprint);
    }

    let contract_epoch = store
        .current_contract_epoch(&binding.task_ref)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    if contract_epoch == 0 {
        return Err(SchedulerAuthorityError::MissingContract(
            binding.task_ref.clone(),
        ));
    }
    let contract_row = store
        .load_task_contract(&binding.task_ref, contract_epoch)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| SchedulerAuthorityError::MissingContract(binding.task_ref.clone()))?;
    let contract = parse_execution_bound_contract(&contract_row.canonical_json)?;

    let deadline = contract.deadline.ok_or_else(|| {
        SchedulerAuthorityError::MalformedContract("v0.2 contract has no deadline".to_owned())
    })?;
    let loop_object_id = ObjectId::parse(
        &contract
            .loop_object_id
            .ok_or_else(|| {
                SchedulerAuthorityError::MalformedContract(
                    "v0.2 contract has no loop object identity".to_owned(),
                )
            })?
            .0,
    )
    .map_err(|error| SchedulerAuthorityError::MalformedContract(error.to_string()))?;
    let budget_id = BudgetId::parse(
        &contract
            .budget_id
            .ok_or_else(|| {
                SchedulerAuthorityError::MalformedContract(
                    "v0.2 contract has no budget identity".to_owned(),
                )
            })?
            .0,
    )
    .map_err(|error| SchedulerAuthorityError::MalformedContract(error.to_string()))?;
    let loop_object = store
        .load_object(LifecycleDomain::Loop, &loop_object_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::LoopUnavailable(loop_object_id.as_str().to_owned())
        })?;
    if !matches!(loop_object.state.as_str(), "START" | "CONTINUE") {
        return Err(SchedulerAuthorityError::LoopUnavailable(format!(
            "{} is {}",
            loop_object_id.as_str(),
            loop_object.state.as_str()
        )));
    }

    let progress_facts = store
        .list_progress_facts(&loop_object_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?;
    let retry_count = progress_facts
        .iter()
        .filter(|fact| {
            fact.action_fingerprint == binding.action_fingerprint && fact.status != "advanced"
        })
        .count() as i64;
    let completed_steps = progress_facts.len() as i64;
    let stored_budget = store
        .load_budget(&budget_id)
        .map_err(|error| SchedulerAuthorityError::Store(error.to_string()))?
        .ok_or_else(|| SchedulerAuthorityError::BudgetUnavailable(budget_id.as_str().to_owned()))?;
    let configured_cost = contract.budget.money_microunits;
    let (spent_cost_microunits, cost_ceiling_microunits) = match configured_cost {
        Some(cost_ceiling) => {
            let remaining_cost = stored_budget
                .state
                .remaining()
                .get("money_microunits")
                .copied()
                .ok_or_else(|| {
                    SchedulerAuthorityError::BudgetUnavailable(budget_id.as_str().to_owned())
                })?;
            if remaining_cost > cost_ceiling {
                return Err(SchedulerAuthorityError::BudgetUnavailable(
                    budget_id.as_str().to_owned(),
                ));
            }
            (cost_ceiling - remaining_cost, cost_ceiling)
        }
        None => (0, i64::MAX),
    };

    Ok(SchedulerAuthoritySnapshot {
        ceiling_facts: SchedulerCeilingFacts {
            deadline: Some(deadline),
            retry_count,
            retry_ceiling: contract.max_retries,
            completed_steps,
            step_ceiling: contract.max_iterations,
            spent_cost_microunits,
            cost_ceiling_microunits,
        },
        loop_object_id,
        budget_id,
    })
}

/// Finish daemon admission after the runtime has evaluated the fresh ceiling
/// snapshot. A committed STOP is terminal for this attempt: the lease closure
/// remains uncalled, ensuring no scheduler worker is admitted after a ceiling.
fn complete_scheduler_admission(
    ceiling_dispatch: SchedulerCeilingDispatch,
    acquire_lease: impl FnOnce() -> Result<SchedulerDispatch, SchedulerServiceError>,
) -> Result<SchedulerDispatchAdmission, SchedulerAuthorityError> {
    match ceiling_dispatch {
        SchedulerCeilingDispatch::Stopped(transition) => {
            Ok(SchedulerDispatchAdmission::Stopped(transition))
        }
        SchedulerCeilingDispatch::Proceed => {
            Ok(SchedulerDispatchAdmission::Leased(acquire_lease()?))
        }
    }
}

/// Invoke the daemon-owned Effect-closure boundary only after a fenced lease.
///
/// A persisted ceiling STOP never reaches the callback. A callback error or a
/// pending reconciliation leaves the durable lease untouched so a stale or
/// uncertain Effect cannot be reported as a scheduler or Task success.
fn complete_scheduler_worker_attempt(
    admission: SchedulerDispatchAdmission,
    complete_effect: impl FnOnce(
        SchedulerDispatch,
    ) -> Result<SchedulerEffectClosure, SchedulerAuthorityError>,
) -> Result<SchedulerWorkerAttempt, SchedulerAuthorityError> {
    match admission {
        SchedulerDispatchAdmission::Stopped(transition) => {
            Ok(SchedulerWorkerAttempt::Stopped(transition))
        }
        SchedulerDispatchAdmission::Leased(dispatch) => match complete_effect(dispatch.clone())? {
            SchedulerEffectClosure::Closed => Ok(SchedulerWorkerAttempt::EffectClosed(dispatch)),
            SchedulerEffectClosure::PendingReconciliation => {
                Ok(SchedulerWorkerAttempt::AwaitingReconciliation(dispatch))
            }
        },
    }
}

/// Release a fenced scheduler lease only after durable Effect closure.
///
/// The supplied release operation must call `SchedulerRepository::release_lease`
/// with the exact dispatch task reference, owner, and epoch. This boundary
/// intentionally retains stopped and reconciliation-pending attempts: neither
/// state proves that an Effect is closed, and neither may become scheduler or
/// Task success through lease release.
fn release_closed_effect_dispatch(
    worker_attempt: SchedulerWorkerAttempt,
    release_lease: impl FnOnce(SchedulerDispatch) -> Result<(), SchedulerAuthorityError>,
) -> Result<SchedulerWorkerAttempt, SchedulerAuthorityError> {
    match worker_attempt {
        SchedulerWorkerAttempt::EffectClosed(dispatch) => {
            release_lease(dispatch.clone())?;
            Ok(SchedulerWorkerAttempt::EffectClosed(dispatch))
        }
        SchedulerWorkerAttempt::Stopped(transition) => {
            Ok(SchedulerWorkerAttempt::Stopped(transition))
        }
        SchedulerWorkerAttempt::AwaitingReconciliation(dispatch) => {
            Ok(SchedulerWorkerAttempt::AwaitingReconciliation(dispatch))
        }
    }
}

/// Resolve a dispatch's durable Effect and close only the matching scheduler
/// lease. This is the concrete worker closure boundary: it accepts neither a
/// process receipt nor a caller-provided Effect state.
///
/// `task_binding` is fixed before worker entry and must match the leased task.
/// The repository release retains the dispatch owner and epoch, while the
/// durable Effect resolver supplies the only closure disposition. A scheduler
/// `Succeeded` row means this dispatch's Effect reached a terminal durable
/// state; it does not accept or complete the Task.
fn complete_durable_scheduler_effect_closure<S>(
    admission: SchedulerDispatchAdmission,
    store: &S,
    task_binding: &TaskBinding,
    scheduler_repository: &mut SchedulerRepository,
    released_at: &str,
) -> Result<SchedulerWorkerAttempt, SchedulerAuthorityError>
where
    S: AuthorityStore + ProtocolStore,
{
    let worker_attempt = complete_scheduler_worker_attempt(admission, |dispatch| {
        if dispatch.task_ref != task_binding.task_ref {
            return Err(SchedulerAuthorityError::DispatchBindingMismatch(format!(
                "leased task {} does not match bound task {}",
                dispatch.task_ref, task_binding.task_ref
            )));
        }
        Ok(resolve_scheduler_effect_for_task_binding(store, task_binding)?.closure)
    })?;
    complete_resolved_effect_and_release(worker_attempt, scheduler_repository, released_at)
}

/// Release an already resolved closed Effect through the real scheduler
/// repository. Pending reconciliation and durable ceiling STOP attempts keep
/// their leases untouched.
fn complete_resolved_effect_and_release(
    worker_attempt: SchedulerWorkerAttempt,
    scheduler_repository: &mut SchedulerRepository,
    released_at: &str,
) -> Result<SchedulerWorkerAttempt, SchedulerAuthorityError> {
    WallTimestamp::parse(released_at)
        .map_err(|_| SchedulerAuthorityError::InvalidReleaseTime(released_at.to_owned()))?;
    release_closed_effect_dispatch(worker_attempt, |dispatch| {
        scheduler_repository.release_lease(
            &dispatch.task_ref,
            &dispatch.lease_owner,
            dispatch.lease_epoch,
            SchedulerState::Succeeded,
            released_at,
        )?;
        Ok(())
    })
}

/// Commit a reached ceiling STOP before a worker can obtain a scheduler lease.
///
/// This is the daemon composition boundary: it reloads the current authority
/// snapshot, delegates the fenced STOP commit to the kernel, and only calls
/// the scheduler repository when no hard ceiling was reached. It deliberately
/// stops before external worker or Effect dispatch.
#[allow(clippy::too_many_arguments)]
pub(crate) fn admit_scheduler_dispatch<S, C, G>(
    authority_store: &S,
    scheduler_repository: &mut SchedulerRepository,
    scheduler_service: &mut SchedulerService,
    driver: &LoopDriver<'_, S, C, G>,
    binding: &SchedulerAuthorityBinding,
    lease_epoch: i64,
    observed_wall_time: &str,
    expected_loop_version: Version,
    writer_lease: &WriterLease,
) -> Result<SchedulerDispatchAdmission, SchedulerAuthorityError>
where
    S: AuthorityStore + HarnessStore + IntentChainStore + ProtocolStore,
    C: Clock,
    G: IdGenerator,
{
    let snapshot = load_scheduler_authority_snapshot(authority_store, binding)?;
    let ceiling_dispatch = scheduler_service.stop_before_dispatch_when_ceiling_reached(
        &snapshot.ceiling_facts,
        observed_wall_time,
        driver,
        &snapshot.loop_object_id,
        expected_loop_version,
        &binding.task_ref,
        &snapshot.budget_id,
        writer_lease,
    )?;
    complete_scheduler_admission(ceiling_dispatch, || {
        scheduler_service.claim_eligible(
            scheduler_repository,
            &binding.task_ref,
            lease_epoch,
            observed_wall_time,
        )
    })
}
