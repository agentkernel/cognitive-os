//! Fault-injection framework for effect/recovery testing (M4), shared by
//! the in-repo acceptance suites and the Lane-CFR conformance runner's
//! behavioral execution batches.
//!
//! Provides:
//! - [`CrashHarness`]: process-crash simulation at the three canonical
//!   crash points (`eff-crash-001..003`). A "crash" drops every in-memory
//!   handle at the chosen point and reopens the WAL database fresh —
//!   exactly what a `kill -9` leaves behind: committed transactions only.
//! - [`ScriptedExecutor`]: a deterministic external-sink double that
//!   records every dispatch/query (idempotency evidence), scripts outcomes
//!   (executed / not executed / unknown / timeout), enforces sink-side
//!   fencing (F-014: stale-epoch dispatch is rejected AT THE SINK), and
//!   can simulate idempotent absorption of same-key re-dispatch.
//! - Storage degradation reuse: read-only reopen (`SqliteAuthorityStore::
//!   open_read_only`) and mid-transaction collision injection are M2
//!   facilities; this module adds the process-crash dimension.

use crate::sqlite::SqliteAuthorityStore;
use cognitive_kernel::executor::{
    DispatchOutcome, EffectExecutor, ExecutorCall, ExecutorCapabilities, ExecutorQueryResult,
};
use cognitive_kernel::ports::{PortFailure, StorePortError};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// The three canonical crash points (REQ-EFF-006).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrashPoint {
    /// After the Intent persisted, before dispatch was recorded.
    AfterIntentBeforeDispatch,
    /// After external execution, before the receipt/outcome persisted.
    AfterExecutionBeforeReceipt,
    /// After verification passed, before the local commit.
    AfterVerificationBeforeCommit,
}

/// Process-crash simulation over one authority database file. The harness
/// owns the path; `crash()` drops the current store handle (all in-memory
/// state dies with it) and `recover_handle()` reopens the same durable
/// file, which contains exactly the committed history.
#[derive(Debug)]
pub struct CrashHarness {
    path: PathBuf,
}

impl CrashHarness {
    /// Create a harness over a database path.
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }

    /// Open the pre-crash store handle.
    pub fn open(&self) -> Result<SqliteAuthorityStore, StorePortError> {
        SqliteAuthorityStore::open(&self.path)
    }

    /// Simulate the crash: drop the handle. Everything not committed to
    /// the WAL is gone; nothing buffered in memory survives.
    pub fn crash(&self, store: SqliteAuthorityStore) {
        drop(store);
    }

    /// Reopen the database after the crash (the recovery process's view).
    pub fn recover_handle(&self) -> Result<SqliteAuthorityStore, StorePortError> {
        SqliteAuthorityStore::open(&self.path)
    }
}

/// One recorded external call (idempotency evidence).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedDispatch {
    /// Idempotency key the call carried.
    pub idempotency_key: String,
    /// Parameter digest the call carried.
    pub parameters_digest: String,
    /// Fencing epoch the call carried.
    pub fencing_epoch: i64,
    /// Whether the sink accepted (fencing) and executed the call.
    pub accepted: bool,
}

/// Scripted behavior for the next dispatches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptedOutcome {
    /// Execute and return a receipt.
    Execute,
    /// Authoritatively refuse (nothing happened externally).
    Refuse,
    /// Execute externally but return Unknown (timeout after the side
    /// effect happened) — the crash-point-2 shape.
    ExecuteThenTimeout,
    /// Do nothing and return Unknown (timeout before anything happened).
    VanishWithoutExecution,
}

#[derive(Debug, Default)]
struct ExecutorState {
    /// Keys the external system has ACTUALLY executed (its ledger).
    executed_keys: Vec<String>,
    /// Every dispatch attempt seen by the sink.
    dispatches: Vec<RecordedDispatch>,
    /// Every outcome query seen by the sink.
    queries: Vec<String>,
    /// Scripted outcomes consumed in order; empty = Execute.
    script: Vec<ScriptedOutcome>,
    /// Whether queries can see the ledger (queryable capability).
    queryable: bool,
    /// Whether same-key re-dispatch is absorbed (idempotent capability).
    idempotent: bool,
    /// Epoch the sink currently trusts (F-014 sink-side fencing).
    trusted_epoch: i64,
}

/// Deterministic external-sink double (see module docs).
#[derive(Debug)]
pub struct ScriptedExecutor {
    state: Mutex<ExecutorState>,
}

impl ScriptedExecutor {
    /// A queryable, non-idempotent executor trusting `epoch`.
    pub fn queryable(epoch: i64) -> Self {
        Self {
            state: Mutex::new(ExecutorState {
                queryable: true,
                idempotent: false,
                trusted_epoch: epoch,
                ..Default::default()
            }),
        }
    }

    /// An idempotent, non-queryable executor trusting `epoch`.
    pub fn idempotent(epoch: i64) -> Self {
        Self {
            state: Mutex::new(ExecutorState {
                queryable: false,
                idempotent: true,
                trusted_epoch: epoch,
                ..Default::default()
            }),
        }
    }

    /// Script the outcomes of upcoming dispatches (consumed in order).
    pub fn script(&self, outcomes: &[ScriptedOutcome]) {
        if let Ok(mut state) = self.state.lock() {
            state.script = outcomes.to_vec();
        }
    }

    /// Update the epoch the sink trusts (recovery advances it).
    pub fn trust_epoch(&self, epoch: i64) {
        if let Ok(mut state) = self.state.lock() {
            state.trusted_epoch = epoch;
        }
    }

    /// All dispatch attempts the sink has seen.
    pub fn dispatches(&self) -> Vec<RecordedDispatch> {
        self.state
            .lock()
            .map(|state| state.dispatches.clone())
            .unwrap_or_default()
    }

    /// Keys the external system actually executed (its ledger).
    pub fn executed_keys(&self) -> Vec<String> {
        self.state
            .lock()
            .map(|state| state.executed_keys.clone())
            .unwrap_or_default()
    }

    /// Outcome queries the sink has seen.
    pub fn queries(&self) -> Vec<String> {
        self.state
            .lock()
            .map(|state| state.queries.clone())
            .unwrap_or_default()
    }
}

impl EffectExecutor for ScriptedExecutor {
    fn capabilities(&self) -> ExecutorCapabilities {
        self.state
            .lock()
            .map(|state| ExecutorCapabilities {
                queryable: state.queryable,
                idempotent: state.idempotent,
            })
            .unwrap_or(ExecutorCapabilities {
                queryable: false,
                idempotent: false,
            })
    }

    fn dispatch(&self, call: &ExecutorCall) -> Result<DispatchOutcome, PortFailure> {
        let mut state = self.state.lock().map_err(|_| PortFailure {
            detail: "executor poisoned".to_owned(),
        })?;
        // F-014: sink-side fencing beats everything else.
        if call.fencing_epoch != state.trusted_epoch {
            let sink_epoch = state.trusted_epoch;
            state.dispatches.push(RecordedDispatch {
                idempotency_key: call.idempotency_key.clone(),
                parameters_digest: call.parameters_digest.clone(),
                fencing_epoch: call.fencing_epoch,
                accepted: false,
            });
            return Ok(DispatchOutcome::FencedStaleEpoch { sink_epoch });
        }
        // Idempotent absorption: a key already executed is not re-executed.
        if state.idempotent
            && state
                .executed_keys
                .iter()
                .any(|k| k == &call.idempotency_key)
        {
            state.dispatches.push(RecordedDispatch {
                idempotency_key: call.idempotency_key.clone(),
                parameters_digest: call.parameters_digest.clone(),
                fencing_epoch: call.fencing_epoch,
                accepted: true,
            });
            return Ok(DispatchOutcome::Executed {
                receipt_ref: format!("receipt://absorbed/{}", call.idempotency_key),
            });
        }
        let outcome = if state.script.is_empty() {
            ScriptedOutcome::Execute
        } else {
            state.script.remove(0)
        };
        state.dispatches.push(RecordedDispatch {
            idempotency_key: call.idempotency_key.clone(),
            parameters_digest: call.parameters_digest.clone(),
            fencing_epoch: call.fencing_epoch,
            accepted: true,
        });
        Ok(match outcome {
            ScriptedOutcome::Execute => {
                state.executed_keys.push(call.idempotency_key.clone());
                DispatchOutcome::Executed {
                    receipt_ref: format!("receipt://executed/{}", call.idempotency_key),
                }
            }
            ScriptedOutcome::Refuse => DispatchOutcome::NotExecuted {
                reason: "sink refused deterministically".to_owned(),
            },
            ScriptedOutcome::ExecuteThenTimeout => {
                // The side effect HAPPENS, but the caller only sees a
                // timeout: the canonical unknown-outcome shape.
                state.executed_keys.push(call.idempotency_key.clone());
                DispatchOutcome::Unknown {
                    detail: "timeout after dispatch; receipt lost".to_owned(),
                }
            }
            ScriptedOutcome::VanishWithoutExecution => DispatchOutcome::Unknown {
                detail: "connection lost; nothing observable".to_owned(),
            },
        })
    }

    fn query_outcome(&self, idempotency_key: &str) -> Result<ExecutorQueryResult, PortFailure> {
        let mut state = self.state.lock().map_err(|_| PortFailure {
            detail: "executor poisoned".to_owned(),
        })?;
        state.queries.push(idempotency_key.to_owned());
        if !state.queryable {
            return Ok(ExecutorQueryResult::Indeterminate);
        }
        if state.executed_keys.iter().any(|k| k == idempotency_key) {
            Ok(ExecutorQueryResult::ExecutedWithOriginalKey)
        } else {
            Ok(ExecutorQueryResult::NotExecuted)
        }
    }
}
