//! Port traits the deterministic kernel depends on, implemented by
//! adapters (`cognitive-store` for persistence; test fakes in unit tests).
//!
//! Dependency rule (`.cursor/rules/10-rust-kernel.mdc`): these traits and
//! DTOs carry no SQLite, HTTP, or runtime types. The store adapter maps
//! them onto its own technology (ADR-0002: SQLite WAL) and surfaces exactly
//! two failure classes: CAS conflict and fail-closed unavailability.
//!
//! Atomicity contract (ADR-0002 binding rule 1, REQ-EVT-002): one
//! [`TransitionCommit`] or [`ObjectAdmission`] is one authoritative commit —
//! object CAS + event append + transition record + optional budget debit +
//! outbox rows commit together or not at all. An adapter MUST NOT apply any
//! subset, MUST NOT buffer a failed commit in memory (REQ-REC-003), and
//! MUST keep the event log append-only (REQ-EVT-004).

use crate::budget::BudgetState;
use cognitive_domain::{
    BudgetId, EventId, LifecycleDomain, ObjectId, RecordId, StateName, Version, WallTimestamp,
};
use serde_json::Value;

/// Failure classes an adapter may surface on the authority path.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StorePortError {
    /// A compare-and-set applied zero rows (version raced) or a uniqueness
    /// constraint rejected a duplicate identity. No side effects persist.
    #[error("store-conflict: {detail}")]
    Conflict {
        /// What raced.
        detail: String,
    },
    /// The authoritative commit path cannot persist. Governed writes fail
    /// closed (`STATE_STORE_UNAVAILABLE`); nothing may be buffered in
    /// memory as if committed (REQ-REC-003).
    #[error("store-unavailable: {detail}")]
    Unavailable {
        /// Underlying failure description.
        detail: String,
    },
}

/// Authoritative current row of one governed object.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredObject {
    /// Stable identity.
    pub object_id: ObjectId,
    /// Lifecycle domain whose table governs this object.
    pub domain: LifecycleDomain,
    /// Authoritative current state.
    pub state: StateName,
    /// Authoritative logical version.
    pub version: Version,
    /// Opaque object body (header/payload as provided at admission).
    pub body: Value,
}

/// Admission of a new governed object at its table's initial state,
/// committed atomically with its admission event.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectAdmission {
    /// The object row to insert (version [`Version::INITIAL`]).
    pub object: StoredObject,
    /// Wall time of admission.
    pub admitted_at: WallTimestamp,
    /// The admission event to append in the same transaction.
    pub event: EventDraft,
    /// Outbox rows to insert in the same transaction.
    pub outbox: Vec<OutboxDraft>,
    /// Writer fencing epoch (F-014): when set, the adapter MUST verify it
    /// against the current epoch INSIDE the transaction and reject stale
    /// writers with a conflict. `None` = unfenced M2 path.
    pub fencing_epoch: Option<i64>,
}

/// Compare-and-set update of one governed object row.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectCas {
    /// Target object.
    pub object_id: ObjectId,
    /// Lifecycle domain (part of the row identity check).
    pub domain: LifecycleDomain,
    /// State the row must currently hold.
    pub from_state: StateName,
    /// State the row moves to.
    pub to_state: StateName,
    /// Version the row must currently hold (`WHERE version = ?`).
    pub expected_version: Version,
    /// Version the row advances to (exactly `expected + 1`).
    pub next_version: Version,
    /// Wall time of the commit.
    pub committed_at: WallTimestamp,
}

/// One event to append to the append-only log.
#[derive(Debug, Clone, PartialEq)]
pub struct EventDraft {
    /// Event identity.
    pub event_id: EventId,
    /// Object this event belongs to.
    pub object_id: ObjectId,
    /// Lifecycle domain of the object.
    pub domain: LifecycleDomain,
    /// Object logical version after this event.
    pub object_version: Version,
    /// Event type (`^[a-z][a-z0-9_.-]+$`).
    pub event_type: String,
    /// Canonical JSON bytes of the event value (RFC 8785, UTF-8).
    pub canonical_json: String,
}

/// One committed state-transition record to append (append-only, like the
/// event log; `state-transition-record.schema.json` shape).
#[derive(Debug, Clone, PartialEq)]
pub struct RecordDraft {
    /// Record identity.
    pub record_id: RecordId,
    /// Subject object.
    pub object_id: ObjectId,
    /// Lifecycle domain.
    pub domain: LifecycleDomain,
    /// Object logical version after the transition.
    pub object_version: Version,
    /// Canonical JSON bytes of the record value.
    pub canonical_json: String,
}

/// Compare-and-set update of one hard-budget ledger row.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetCas {
    /// Target budget row.
    pub budget_id: BudgetId,
    /// Version the row must currently hold.
    pub expected_version: Version,
    /// Version the row advances to.
    pub next_version: Version,
    /// Canonical JSON bytes of the debited [`BudgetState`].
    pub next_state_canonical_json: String,
}

/// One outbox row to insert with the commit (at-least-once delivery seed).
#[derive(Debug, Clone, PartialEq)]
pub struct OutboxDraft {
    /// Event the outbox row delivers.
    pub event_id: EventId,
    /// Logical destination (consumer channel name).
    pub destination: String,
}

/// One authoritative transition commit (single atomic unit).
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionCommit {
    /// Object CAS.
    pub cas: ObjectCas,
    /// Event append.
    pub event: EventDraft,
    /// Transition record append.
    pub record: RecordDraft,
    /// Optional hard-budget debit (same transaction).
    pub budget: Option<BudgetCas>,
    /// Outbox rows (same transaction).
    pub outbox: Vec<OutboxDraft>,
    /// Writer fencing epoch (F-014): when set, the adapter MUST verify it
    /// against the current epoch INSIDE the transaction and reject stale
    /// writers with a conflict. `None` = unfenced M2 path.
    pub fencing_epoch: Option<i64>,
}

/// Receipt of one committed admission or transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitReceipt {
    /// Global append-only log sequence of the committed event.
    pub event_sequence: i64,
}

/// One committed event read back from the log.
#[derive(Debug, Clone, PartialEq)]
pub struct CommittedEvent {
    /// Global log sequence (authoritative order, ADR-0005
    /// `logical_version` domain).
    pub sequence: i64,
    /// Event identity.
    pub event_id: EventId,
    /// Object the event belongs to.
    pub object_id: ObjectId,
    /// Lifecycle domain.
    pub domain: LifecycleDomain,
    /// Object logical version after the event.
    pub object_version: Version,
    /// Event type.
    pub event_type: String,
    /// Canonical JSON bytes of the event value.
    pub canonical_json: String,
}

/// Authoritative current row of one hard budget.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredBudget {
    /// Budget identity.
    pub budget_id: BudgetId,
    /// Remaining amounts.
    pub state: BudgetState,
    /// Ledger row version (CAS token).
    pub version: Version,
}

/// One pending outbox row.
#[derive(Debug, Clone, PartialEq)]
pub struct OutboxEntry {
    /// Outbox sequence.
    pub outbox_sequence: i64,
    /// Event to deliver.
    pub event_id: EventId,
    /// Logical destination.
    pub destination: String,
    /// True once delivery bookkeeping marked this row dispatched.
    pub dispatched: bool,
}

/// Persistence port for the authoritative store (implemented by
/// `cognitive-store`; ADR-0002 binds the reference adapter to SQLite WAL).
pub trait AuthorityStore {
    /// Read the authoritative current row of one object.
    fn load_object(
        &self,
        domain: LifecycleDomain,
        object_id: &ObjectId,
    ) -> Result<Option<StoredObject>, StorePortError>;

    /// Admit a new object atomically with its admission event. A duplicate
    /// identity is a [`StorePortError::Conflict`].
    fn admit_object(&self, admission: &ObjectAdmission) -> Result<CommitReceipt, StorePortError>;

    /// Apply one transition commit atomically. Zero-row CAS (object or
    /// budget) is a [`StorePortError::Conflict`] and nothing persists.
    fn commit_transition(&self, commit: &TransitionCommit)
    -> Result<CommitReceipt, StorePortError>;

    /// Read the authoritative current row of one budget.
    fn load_budget(&self, budget_id: &BudgetId) -> Result<Option<StoredBudget>, StorePortError>;

    /// Create a hard-budget ledger row. Duplicate identity is a conflict.
    fn create_budget(
        &self,
        budget_id: &BudgetId,
        state_canonical_json: &str,
        created_at: &WallTimestamp,
    ) -> Result<(), StorePortError>;

    /// Read committed events in log order, strictly after `after_sequence`
    /// (0 reads from the beginning), up to `limit` rows.
    fn read_events(
        &self,
        after_sequence: i64,
        limit: usize,
    ) -> Result<Vec<CommittedEvent>, StorePortError>;

    /// Read outbox rows not yet marked dispatched, in outbox order.
    fn pending_outbox(&self, limit: usize) -> Result<Vec<OutboxEntry>, StorePortError>;

    /// Delivery bookkeeping: mark one outbox row dispatched. This never
    /// touches the event log.
    fn mark_outbox_dispatched(
        &self,
        outbox_sequence: i64,
        dispatched_at: &WallTimestamp,
    ) -> Result<(), StorePortError>;
}

/// One persisted Intent row (immutable once inserted; the storage layer
/// forbids UPDATE/DELETE exactly like the event log). The idempotency key
/// is unique across the store: key stability and same-key conflict
/// detection are structural (REQ-EFF-001/002,
/// `docs/standards/intent-effect-idempotency.md` sections 2-3).
#[derive(Debug, Clone, PartialEq)]
pub struct IntentRow {
    /// Intent identity.
    pub intent_id: ObjectId,
    /// Stable idempotency key of the logical effect attempt chain.
    pub idempotency_key: String,
    /// Canonical parameter digest (comparison basis, never source bytes).
    pub parameters_digest: String,
    /// Operation action name.
    pub action: String,
    /// Target URI.
    pub target: String,
    /// Effect object this intent is bound to.
    pub effect_object_id: ObjectId,
    /// CAS version of the fixed pre-state.
    pub expected_state_version: Version,
    /// Revocation epoch of the authorization binding.
    pub grant_epoch: i64,
    /// Capability set version of the authorization binding.
    pub capability_set_version: i64,
    /// Canonical JSON of the full intent value (evidence).
    pub canonical_json: String,
}

/// M4 protocol persistence port: intents, fencing epochs and in-flight
/// enumeration. Implemented alongside [`AuthorityStore`] by the store
/// adapter; the intent insert commits atomically with its event
/// (REQ-EFF-001: no Intent, no dispatch).
pub trait ProtocolStore {
    /// Insert an intent row and append its event in ONE transaction. A
    /// duplicate `intent_id`/`effect_object_id` is a conflict; a duplicate
    /// `idempotency_key` is a conflict the caller maps to idempotent-replay
    /// or `EFFECT_IDEMPOTENCY_CONFLICT` per parameter digest.
    fn insert_intent(
        &self,
        intent: &IntentRow,
        event: &EventDraft,
    ) -> Result<CommitReceipt, StorePortError>;

    /// Load the intent bound to an idempotency key.
    fn load_intent_by_key(&self, key: &str) -> Result<Option<IntentRow>, StorePortError>;

    /// Load the intent bound to an effect object.
    fn load_intent_for_effect(
        &self,
        effect_object_id: &ObjectId,
    ) -> Result<Option<IntentRow>, StorePortError>;

    /// Current fencing epoch of this authority store (starts at 1).
    fn current_fencing_epoch(&self) -> Result<i64, StorePortError>;

    /// Advance the fencing epoch by exactly one and return the new value
    /// (recovery step 2; old-epoch writers are fenced from that instant).
    fn advance_fencing_epoch(&self) -> Result<i64, StorePortError>;

    /// Enumerate governed objects of `domain` currently in any of `states`
    /// (recovery step 5: find in-flight Effects to reconcile).
    fn list_objects_in_states(
        &self,
        domain: LifecycleDomain,
        states: &[StateName],
    ) -> Result<Vec<StoredObject>, StorePortError>;

    /// Append one checkpoint row (append-only, like events). The adapter
    /// MUST verify `fencing_epoch` against the current epoch INSIDE the
    /// transaction and reject stale writers (F-014 checkpoint sink).
    fn append_checkpoint(&self, checkpoint: &CheckpointRow) -> Result<(), StorePortError>;

    /// Load the newest checkpoint of one loop object.
    fn latest_checkpoint(
        &self,
        loop_object_id: &ObjectId,
    ) -> Result<Option<CheckpointRow>, StorePortError>;
}

/// One persisted loop checkpoint (recovery-stable facts of
/// `loop-checkpoint.schema.json`: event high-watermark, fencing epoch,
/// version pins — REQ-RUN-006, F-010).
#[derive(Debug, Clone, PartialEq)]
pub struct CheckpointRow {
    /// Checkpoint identity.
    pub checkpoint_id: ObjectId,
    /// Loop object this checkpoint belongs to.
    pub loop_object_id: ObjectId,
    /// Event-log high watermark consumed at checkpoint time.
    pub event_high_watermark: i64,
    /// Fencing epoch the checkpoint was taken under.
    pub fencing_epoch: i64,
    /// Canonical JSON of the checkpoint value (pins and pending effects).
    pub canonical_json: String,
}

/// Failure of an infrastructure port (clock, ID generation). The kernel
/// fails closed on these; they never degrade into guesses.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("port-failure: {detail}")]
pub struct PortFailure {
    /// Failure description.
    pub detail: String,
}

/// Wall-clock port (`wall_clock` domain, ADR-0005). Readings are canonical
/// RFC 3339 UTC timestamps; a clock that cannot produce a trusted reading
/// fails instead of guessing.
pub trait Clock {
    /// Current wall-clock time.
    fn now(&self) -> Result<WallTimestamp, PortFailure>;
}

/// Identifier source: lowercase canonical UUIDv7 (RFC 9562, ADR-0005),
/// cryptographically random, monotonicity-preserving within the generator.
pub trait IdGenerator {
    /// Generate the next UUIDv7 in lowercase canonical text form.
    fn next_uuid_v7(&self) -> Result<String, PortFailure>;
}
