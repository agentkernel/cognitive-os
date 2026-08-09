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

use crate::authz::{ActorChainFacts, DenyRule, MembershipFacts, ObjectGovernance, PrincipalFacts};
use crate::budget::BudgetState;
use crate::effects::OperationDescriptor;
use cognitive_contracts::generated::context_view::{
    LoadedContextItemRepresentation, LoadedContextItemRole, LoadedContextItemTrustLevel,
};
use cognitive_contracts::generated::governed_object_header::GovernedObjectHeader;
use cognitive_domain::capability::CapabilityConstraints;
use cognitive_domain::{
    BudgetId, EventId, LifecycleDomain, ObjectId, RecordId, StateName, Version, WallTimestamp,
};
use serde::Deserialize;
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
    /// Canonical charge admitted by the deterministic transition gate.
    /// Compound authority transactions use this immutable proof to bind a
    /// private authorization to exactly the fresh debit it permits.
    pub charge_canonical_json: String,
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

/// Binding of an Intent to one task's contract epoch (M5 intent chain,
/// REQ-INTENT-SUPERSEDE-001). A dispatch bound to an epoch older than the
/// task's current contract epoch is fenced with the registered
/// `INTENT_VERSION_SUPERSEDED` code — the correction-fencing analogue of
/// the F-014 writer lease.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskBinding {
    /// Task URI the intent works for.
    pub task_ref: String,
    /// Contract epoch the proposal was made under.
    pub contract_epoch: i64,
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
    /// Task/contract-epoch binding (M5). `None` = pre-M5 unbound intent.
    pub task_binding: Option<TaskBinding>,
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

    /// List immutable Intent rows bound to exactly one task contract epoch,
    /// in stable identity order. Scheduler authority uses this reverse index
    /// to resolve a fenced dispatch only from durable facts.
    fn list_intents_for_task_binding(
        &self,
        task_binding: &TaskBinding,
    ) -> Result<Vec<IntentRow>, StorePortError>;

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

    /// Load one committed event by identity (D-018: the M5 runtime
    /// envelope assembler resolves outbox rows to their committed event
    /// values without scanning the log).
    fn load_event_by_id(
        &self,
        event_id: &EventId,
    ) -> Result<Option<CommittedEvent>, StorePortError>;

    /// Current (highest) TaskContract epoch of one task; 0 = no contract
    /// (M5 correction fencing: the epoch-currency read the effect
    /// protocol consults at mint and dispatch, REQ-INTENT-SUPERSEDE-001).
    fn current_contract_epoch(&self, task_ref: &str) -> Result<i64, StorePortError>;
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

/// One persisted UserIntentRecord row (immutable once inserted, exactly
/// like the event log: REQ-INTENT-RECORD-001 — summaries, model output and
/// later corrections never overwrite the original record). The
/// `canonical_json` carries the `user-intent-record.schema.json` shape
/// composed from the generated binding; the flat columns are derived
/// copies for deterministic queries.
#[derive(Debug, Clone, PartialEq)]
pub struct UserIntentRecordRow {
    /// Record identity.
    pub record_id: ObjectId,
    /// Conversation or ResourceScope the expression arrived in.
    pub conversation_or_scope_ref: String,
    /// Canonical actor-chain digest of the expressing principal.
    pub actor_chain_digest: String,
    /// Raw user expression (never rewritten).
    pub raw_expression: String,
    /// Wall time the record was fixed.
    pub recorded_at: WallTimestamp,
    /// Intent authority whose acceptance decisions bind this record
    /// (deterministic admission comparison basis).
    pub intent_authority_ref: String,
    /// Canonical digest over the fixed expression facts.
    pub intent_digest: String,
    /// Canonical JSON of the schema-shaped record (evidence).
    pub canonical_json: String,
}

/// One persisted IntentInterpretation candidate row (immutable). The row
/// records the candidate AS PROPOSED: `recorded_status` is derived
/// deterministically from the material-ambiguity facts (schema
/// conditional), never chosen by the model. Acceptance and supersession
/// are separate facts (TaskContract rows and `supersedes_interpretation`),
/// not in-place status rewrites.
#[derive(Debug, Clone, PartialEq)]
pub struct InterpretationRow {
    /// Interpretation identity.
    pub interpretation_id: ObjectId,
    /// UserIntentRecord this interpretation was derived from.
    pub user_intent_record_id: ObjectId,
    /// `candidate` or `clarification_required` (deterministic derivation).
    pub recorded_status: String,
    /// Number of MATERIAL ambiguities the candidate declared.
    pub material_ambiguity_count: i64,
    /// Interpretation this candidate supersedes (user correction chains).
    pub supersedes_interpretation: Option<ObjectId>,
    /// Canonical digest of the candidate content (acceptance binding
    /// basis: the authority accepts exactly the digest it reviewed).
    pub interpretation_digest: String,
    /// Canonical JSON of the schema-shaped candidate (evidence).
    pub canonical_json: String,
}

/// One persisted TaskContract row (immutable; `task-contract.schema.json`
/// shape in `canonical_json` via the generated binding). Contract epochs
/// per task are monotonic: the adapter admits epoch N+1 only against the
/// caller's expected current epoch N (CAS inside the transaction).
#[derive(Debug, Clone, PartialEq)]
pub struct TaskContractRow {
    /// Contract identity.
    pub contract_id: ObjectId,
    /// Task URI this contract governs.
    pub task_ref: String,
    /// Monotonic contract epoch (starts at 1).
    pub contract_epoch: i64,
    /// UserIntentRecord bound by this contract.
    pub user_intent_record_id: ObjectId,
    /// Accepted interpretation bound by this contract.
    pub interpretation_id: ObjectId,
    /// Authority that accepted the interpretation.
    pub accepted_by: String,
    /// Canonical digest of the contract content.
    pub contract_digest: String,
    /// Canonical JSON of the schema-shaped contract (evidence).
    pub canonical_json: String,
}

/// One daemon-issued immutable ContextRequest. The request is the durable
/// Context input that a TaskContract v0.4 pins with a strong reference;
/// individual ContextViews remain request-linked resolution artifacts.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextRequestRow {
    /// Immutable ContextRequest identity from its governed header.
    pub request_id: ObjectId,
    /// Task URI from `perspective.task`, retained for fail-closed lookup.
    pub task_ref: String,
    /// Canonical governed-object content digest.
    pub request_digest: String,
    /// Canonical schema-shaped ContextRequest payload.
    pub canonical_json: String,
}

/// One daemon-issued immutable ContextView. A view binds one exact resolution
/// to its ContextRequest and is not a replacement for the TaskContract input.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextViewRow {
    /// Immutable ContextView identity from its governed header.
    pub view_id: ObjectId,
    /// Strongly referenced ContextRequest identity.
    pub request_id: ObjectId,
    /// Canonical governed-object content digest.
    pub view_digest: String,
    /// Canonical schema-shaped ContextView payload.
    pub canonical_json: String,
}

/// Daemon-admitted immutable workspace Context source. The canonical payload
/// remains private to the body-load path; discovery receives metadata only.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceContextSourceRow {
    pub source_id: ObjectId,
    pub source_digest: String,
    pub governance: ObjectGovernance,
    pub role: LoadedContextItemRole,
    pub trust_level: LoadedContextItemTrustLevel,
    pub representation: LoadedContextItemRepresentation,
    pub provenance_ref: String,
    pub content_bytes: i64,
    pub content_tokens: Option<i64>,
    pub canonical_json: String,
}

/// Immutable Memory proposal. It remains a proposal until a daemon-owned
/// deterministic decision admits it; producer-selected admission is absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryCandidateRow {
    pub candidate_id: ObjectId,
    pub candidate_digest: String,
    pub source_id: ObjectId,
    pub source_digest: String,
    pub source_provenance_ref: String,
    pub governance_scope: String,
    pub target_scope: String,
    pub purpose: String,
    pub retention_expires_at_unix_seconds: i64,
    pub observed_at_unix_seconds: i64,
    pub canonical_json: String,
}

/// Immutable daemon decision bound to one exact candidate digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryAdmissionDecisionRow {
    pub decision_id: ObjectId,
    pub candidate_id: ObjectId,
    pub candidate_digest: String,
    pub decision: String,
    pub policy_version: i64,
    pub reason_codes_json: String,
    pub canonical_json: String,
}

/// Immutable admitted Memory object. This row may only accompany an `admit`
/// decision in the same daemon-owned SQLite transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryObjectRow {
    pub memory_id: ObjectId,
    pub candidate_id: ObjectId,
    pub decision_id: ObjectId,
    pub canonical_json: String,
}

/// Authority-filtered FTS query for admitted Memory objects. This is a
/// daemon-private discovery request, not a client authorization grant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemorySearchQuery {
    pub governance_scope: String,
    pub purpose: String,
    pub observed_at_unix_seconds: i64,
    pub query_text: String,
    pub maximum_results: usize,
}

/// Metadata-only Memory retrieval candidate. Callers must still authorize and
/// revalidate the source before loading any source body or using its content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemorySearchCandidateRow {
    pub memory_id: ObjectId,
    pub source_id: ObjectId,
    pub source_digest: String,
}

/// Metadata-only Context discovery result. It deliberately excludes body
/// content so callers must authorize before materializing a candidate.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextCandidateMetadata {
    pub source_id: ObjectId,
    pub source_digest: String,
    /// Immutable source creation time extracted by the store from its
    /// governed header. The scheduler can apply freshness policy before it
    /// asks the body-load port for source content.
    pub created_at: WallTimestamp,
    pub governance: ObjectGovernance,
    pub role: LoadedContextItemRole,
    pub trust_level: LoadedContextItemTrustLevel,
    pub representation: LoadedContextItemRepresentation,
    pub provenance_ref: String,
    pub content_bytes: i64,
    pub content_tokens: Option<i64>,
}

/// Scope-only predicate applied before per-object authorization and body
/// loading. It is not an authorization grant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextCandidateQuery {
    pub tenant_id: String,
    pub resource_scope_prefix: String,
    pub conversation_ref: Option<String>,
    pub limit: usize,
}

/// Immutable daemon-admin-issued authorization inputs for Context body reads.
/// Possession of this record does not itself grant access: callers reconstruct
/// it with current revocation currency and still call the six-step gate.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ContextAuthorizationFactsRow {
    pub fact_set_id: ObjectId,
    pub subject_ref: String,
    pub tenant_id: String,
    pub principal: PrincipalFacts,
    pub actor_chain: ActorChainFacts,
    pub membership: Option<MembershipFacts>,
    pub capability_links: Vec<CapabilityConstraints>,
    pub explicit_denies: Vec<DenyRule>,
    pub capability_set_version: i64,
    pub issued_revocation_epoch: i64,
    pub canonical_json: String,
}

impl ContextAuthorizationFactsRow {
    /// Rebuild an authorization decision snapshot using revocation currency
    /// read at the point of body authorization, never the historical epoch
    /// carried when this fact set was admitted.
    pub fn reconstruct_snapshot(
        &self,
        current_revocation_epoch: i64,
        decided_at: WallTimestamp,
    ) -> Result<crate::authz::AuthzSnapshot, StorePortError> {
        if current_revocation_epoch < 1
            || self.capability_set_version < 1
            || self.issued_revocation_epoch < 1
            || self.subject_ref != self.principal.principal_ref.as_str()
            || self.principal.tenant_id.as_deref() != Some(self.tenant_id.as_str())
        {
            return Err(StorePortError::Unavailable {
                detail: "Context authorization facts are incomplete or inconsistent".to_owned(),
            });
        }
        Ok(crate::authz::AuthzSnapshot {
            tenant_id: self.tenant_id.clone(),
            principal: self.principal.clone(),
            actor_chain: self.actor_chain.clone(),
            membership: self.membership.clone(),
            capability_links: self.capability_links.clone(),
            capability_set_version: self.capability_set_version,
            explicit_denies: self.explicit_denies.clone(),
            revocation_epoch: current_revocation_epoch,
            decided_at,
        })
    }
}

/// Immutable revocation observation. A higher tenant epoch invalidates older
/// capability material during snapshot reconstruction.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ContextRevocationFactRow {
    pub revocation_fact_id: ObjectId,
    pub tenant_id: String,
    pub revocation_epoch: i64,
    pub revoked_subject_ref: Option<String>,
    pub revoked_capability_ref: Option<String>,
    pub canonical_json: String,
}

/// One persisted immutable operation candidate proposal. This row preserves
/// non-authority input for later daemon admission; it does not authorize an
/// operation, reserve budget, or schedule work.
#[derive(Debug, Clone, PartialEq)]
pub struct OperationCandidateProposalRow {
    /// Immutable candidate proposal identity.
    pub candidate_id: ObjectId,
    /// TaskContract task reference the proposal names.
    pub task_ref: String,
    /// Immutable TaskContract epoch the proposal was observed against.
    pub contract_epoch: i64,
    /// Provenance-only source reference supplied by the candidate producer.
    pub candidate_source_ref: String,
    /// Proposed registered tool reference.
    pub tool_ref: String,
    /// Proposed operation action.
    pub action: String,
    /// Proposed operation target.
    pub target: String,
    /// Digest of the proposed parameters; parameters themselves remain in
    /// their separately governed operation descriptor.
    pub parameters_digest: String,
    /// Target-state version observed by the non-authority producer.
    pub expected_state_version: i64,
    /// Immutable operation descriptor reference for daemon validation.
    pub operation_descriptor_ref: ObjectId,
    /// Canonical JSON of the schema-shaped proposal, retained for audit.
    pub canonical_json: String,
}

/// One daemon-owned immutable operation descriptor registry row. Descriptors
/// describe what an executor can do; they never grant a caller permission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonOperationDescriptorRow {
    /// Stable descriptor reference carried by candidate proposals.
    pub descriptor_id: ObjectId,
    /// Descriptor capability and recovery-closure metadata.
    pub descriptor: OperationDescriptor,
    /// Canonical daemon-issued descriptor evidence retained for audit.
    pub canonical_json: String,
}

/// One immutable daemon-only authorization snapshot. It records the current
/// authorization currency and a previously evaluated grant for one exact
/// subject, target, action, and purpose binding. Candidate producers cannot
/// supply or replace these authority facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonAuthorizationSnapshotRow {
    /// Immutable snapshot identity.
    pub snapshot_id: ObjectId,
    /// Authenticated subject the grant applies to.
    pub subject_ref: String,
    /// Exact governed target reference.
    pub target_ref: String,
    /// Exact authorized action.
    pub action: String,
    /// Exact authorized purpose.
    pub purpose: String,
    /// Revocation epoch under which the grant was evaluated.
    pub grant_epoch: i64,
    /// Capability set version under which the grant was evaluated.
    pub capability_set_version: i64,
    /// Current revocation epoch at snapshot issuance.
    pub revocation_epoch: i64,
    /// Canonical decision-time wall timestamp.
    pub observed_at: WallTimestamp,
    /// Canonical daemon-issued authorization evidence.
    pub canonical_json: String,
}

/// One immutable daemon-issued pre-dispatch worker authorization. The row
/// binds a selected candidate to exact Intent, Effect, Loop, budget, and
/// fencing facts. Its issuance must occur only inside an atomic admission
/// bundle; worker consumption is recorded separately.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkerIterationAuthorizationRow {
    /// Immutable authorization identity.
    pub authorization_id: ObjectId,
    /// Daemon-minted TaskContract identity defining this authorization namespace.
    pub worker_authorization_root_id: ObjectId,
    /// Task governed by this authorization.
    pub task_ref: String,
    /// Current immutable TaskContract epoch.
    pub contract_epoch: i64,
    /// Loop whose exact iteration is authorized.
    pub loop_object_id: ObjectId,
    /// Monotonic loop iteration authorized for one worker attempt.
    pub iteration: i64,
    /// Loop CAS version the worker authorization was issued against.
    pub expected_loop_version: Version,
    /// Immutable candidate selected by daemon admission.
    pub selected_candidate_id: ObjectId,
    /// Durable Intent created by the same admission transaction.
    pub intent_id: ObjectId,
    /// Durable Effect created at PROPOSED by the same transaction.
    pub effect_object_id: ObjectId,
    /// Hard budget charged by the authorized iteration.
    pub budget_id: BudgetId,
    /// Canonical BudgetCharge value.
    pub budget_charge_canonical_json: String,
    /// Stable action/retry identity.
    pub action_fingerprint: String,
    /// Writer fencing epoch at issuance.
    pub issued_fencing_epoch: i64,
    /// Generated schema-shaped WorkerIterationAuthorization evidence.
    pub canonical_json: String,
}

/// One immutable daemon-recorded consumption of a WIA by a worker attempt.
/// Consumption records only the authorization handoff; it does not prove an
/// external effect, progress, verification, or Task completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkerIterationAuthorizationConsumptionRow {
    /// Authorization consumed exactly once.
    pub authorization_id: ObjectId,
    /// Daemon-assigned worker attempt identity.
    pub worker_attempt_id: ObjectId,
    /// Fencing epoch rechecked when consumption commits.
    pub consumed_fencing_epoch: i64,
    /// Canonical timestamp of the durable handoff.
    pub consumed_at: WallTimestamp,
    /// Canonical daemon-issued consumption evidence.
    pub canonical_json: String,
}

/// Immutable daemon-owned post-state pin created before loop verification.
/// It is not worker output and cannot be rewritten after verification begins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedPostStateRow {
    pub fixed_post_state_id: ObjectId,
    pub task_binding: TaskBinding,
    pub loop_object_id: ObjectId,
    pub subject_domain: LifecycleDomain,
    pub subject_object_id: ObjectId,
    pub subject_version: Version,
    pub recorded_fencing_epoch: i64,
    pub canonical_json: String,
}

/// Immutable daemon-owned verification request tied to one fixed post-state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationRequestRow {
    pub verification_request_id: ObjectId,
    pub fixed_post_state_id: ObjectId,
    pub task_binding: TaskBinding,
    pub loop_object_id: ObjectId,
    pub expected_loop_version: Version,
    pub verifier_ref: String,
    pub verifier_version: String,
    pub criteria_canonical_json: String,
    pub issued_fencing_epoch: i64,
    pub canonical_json: String,
}

/// Immutable verifier result that the daemon reloads before it may continue a
/// loop. A stored `passed` status alone never accepts or completes a Task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationReportRow {
    pub verification_report_id: ObjectId,
    pub verification_request_id: ObjectId,
    pub fixed_post_state_id: ObjectId,
    pub verifier_ref: String,
    pub verifier_version: String,
    pub status: String,
    pub evidence_refs_canonical_json: String,
    pub completed_at: WallTimestamp,
    pub recorded_fencing_epoch: i64,
    pub canonical_json: String,
}

/// Private one-time authority to begin the next iteration after a verified
/// continuation. This is intentionally distinct from the public WIA, which
/// remains immutable pre-dispatch authority for `DECIDE -> ACT`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuationAuthorizationRow {
    pub continuation_authorization_id: ObjectId,
    pub task_binding: TaskBinding,
    pub loop_object_id: ObjectId,
    pub iteration: i64,
    pub expected_loop_version: Version,
    pub checkpoint_id: ObjectId,
    pub budget_id: BudgetId,
    pub budget_charge_canonical_json: String,
    pub verification_report_id: ObjectId,
    pub issued_fencing_epoch: i64,
    pub canonical_json: String,
}

/// One immutable daemon-recorded handoff of private continuation authority.
/// It records a recoverable authorization boundary only; it does not prove
/// execution, progress, verification, Task acceptance, or Task completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuationAuthorizationConsumptionRow {
    pub continuation_authorization_id: ObjectId,
    pub worker_attempt_id: ObjectId,
    pub consumed_fencing_epoch: i64,
    pub consumed_at: WallTimestamp,
    pub canonical_json: String,
}

/// Exact scheduler lease that must remain active while private continuation
/// authority is handed to the bounded harness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundContinuationAuthorizationConsumption {
    pub consumption: ContinuationAuthorizationConsumptionRow,
    pub scheduler_lease: SchedulerLeaseBinding,
}

/// Daemon-private persistence for the verified continuation boundary.
/// Implementations must keep every row append-only and recheck declared
/// fencing epochs inside the transaction that writes it.
pub trait ContinuationAuthorityStore {
    fn append_fixed_post_state(&self, row: &FixedPostStateRow) -> Result<(), StorePortError>;

    fn load_fixed_post_state(
        &self,
        fixed_post_state_id: &ObjectId,
    ) -> Result<Option<FixedPostStateRow>, StorePortError>;

    fn append_verification_request(
        &self,
        row: &VerificationRequestRow,
    ) -> Result<(), StorePortError>;

    fn load_verification_request(
        &self,
        verification_request_id: &ObjectId,
    ) -> Result<Option<VerificationRequestRow>, StorePortError>;

    fn append_verification_report(&self, row: &VerificationReportRow)
    -> Result<(), StorePortError>;

    fn load_verification_report(
        &self,
        verification_report_id: &ObjectId,
    ) -> Result<Option<VerificationReportRow>, StorePortError>;

    /// Issue one continuation authorization only after the adapter has
    /// revalidated current contract, verified report, checkpoint, exact loop
    /// version/state, and fencing in one transaction.
    fn issue_continuation_authorization(
        &self,
        row: &ContinuationAuthorizationRow,
    ) -> Result<(), StorePortError>;

    /// Consume continuation authority at most once and bind it to an exact
    /// active scheduler lease and atomically commit the supplied, already
    /// gate-validated continuation entry. No partial consumption, lease
    /// binding, state transition, or budget debit may persist.
    fn consume_continuation_authorization_bound_to_scheduler_lease(
        &self,
        request: &BoundContinuationAuthorizationConsumption,
        transition: &TransitionCommit,
    ) -> Result<CommitReceipt, StorePortError>;

    fn load_unconsumed_continuation_authorization(
        &self,
        task_binding: &TaskBinding,
    ) -> Result<Option<ContinuationAuthorizationRow>, StorePortError>;
}

/// Exact scheduler lease identity held when the daemon hands a WIA to one
/// worker. This is private recovery evidence, not worker-provided input and
/// not part of the public TaskContract or WIA schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerLeaseBinding {
    /// Immutable scheduler task identity.
    pub task_ref: String,
    /// TaskContract epoch fixed by the scheduler work key.
    pub contract_epoch: i64,
    /// Exact daemon scheduler lease owner.
    pub lease_owner: String,
    /// Exact owner fencing epoch of the durable scheduler lease.
    pub lease_epoch: i64,
}

/// One daemon-only WIA handoff requested against an already acquired exact
/// scheduler lease. The adapter validates both authorities in one durable
/// transaction before recording either handoff evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundWorkerAuthorizationConsumption {
    /// One-time WIA handoff record.
    pub consumption: WorkerIterationAuthorizationConsumptionRow,
    /// Exact scheduler lease that authorizes this handoff.
    pub scheduler_lease: SchedulerLeaseBinding,
}

/// Durable recovery input for a worker attempt that crossed the daemon's WIA
/// handoff boundary. Recovery must use this record and the corresponding
/// Effect state; it must not reconstruct a worker attempt from scheduler
/// callbacks, receipts, or process-local memory.
#[derive(Debug, Clone, PartialEq)]
pub struct ConsumedWorkerIterationAuthorization {
    /// Immutable daemon-issued authority that was handed to a worker.
    pub authorization: WorkerIterationAuthorizationRow,
    /// The one durable worker-attempt handoff for that authority.
    pub consumption: WorkerIterationAuthorizationConsumptionRow,
    /// Exact lease captured with the handoff. `None` denotes pre-binding
    /// evidence, which recovery must retain rather than use to release work.
    pub scheduler_lease: Option<SchedulerLeaseBinding>,
}

/// All-or-nothing daemon admission of one selected non-authority candidate.
/// The caller must derive every field from reloaded durable authority facts;
/// the store rechecks fencing and CAS preconditions in one transaction.
#[derive(Debug, Clone, PartialEq)]
pub struct CandidateAdmissionCommit {
    /// Candidate selected by daemon-only validation.
    pub selected_candidate_id: ObjectId,
    /// Durable Intent and its provenance event.
    pub intent: IntentRow,
    pub intent_event: EventDraft,
    /// New Effect in its registered initial state (`PROPOSED`, version 1).
    pub effect_admission: ObjectAdmission,
    /// Immutable pre-dispatch worker authority issued by this bundle.
    pub worker_authorization: WorkerIterationAuthorizationRow,
    /// Loop admission transition and optional exact budget debit.
    pub loop_transition: TransitionCommit,
    /// Current writer epoch checked inside the same SQLite transaction.
    pub fencing_epoch: i64,
}

/// Receipts from every event persisted by one candidate admission bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateAdmissionReceipt {
    /// Intent persistence event sequence.
    pub intent_event_sequence: i64,
    /// Effect admission event sequence.
    pub effect_admission_event_sequence: i64,
    /// Loop transition event sequence.
    pub loop_transition_event_sequence: i64,
    /// Immutable WIA identity issued by the transaction.
    pub authorization_id: ObjectId,
}

/// Durable append-only candidate input boundary for daemon-only worker
/// authorization. Persisting a candidate merely makes it auditable; a
/// separate daemon admission path must validate it before creating Intent,
/// Effect, WorkerIterationAuthorization, a budget debit, or scheduler work.
pub trait WorkerAuthorizationStore {
    /// Load one immutable daemon-issued WIA before a worker attempt. A
    /// missing authorization must fail closed; callers must never rebuild it
    /// from worker-provided fields.
    fn load_worker_iteration_authorization(
        &self,
        authorization_id: &ObjectId,
    ) -> Result<Option<WorkerIterationAuthorizationRow>, StorePortError>;

    /// Reconstruct the committed candidate-admission receipt from immutable
    /// WIA and event evidence. This permits an idempotent caller to recover
    /// a successful admission after losing its original response without
    /// repeating budget debits, Loop transitions, or WIA issuance.
    fn load_candidate_admission_receipt_by_selected_candidate_id(
        &self,
        selected_candidate_id: &ObjectId,
    ) -> Result<Option<CandidateAdmissionReceipt>, StorePortError>;

    /// Resolve the sole unconsumed daemon-issued WIA for one exact scheduler
    /// binding. Multiple matching authorities are ambiguous and must fail
    /// closed; consumed WIAs remain recovery-only evidence.
    fn load_unconsumed_worker_iteration_authorization_for_task_binding(
        &self,
        task_ref: &str,
        contract_epoch: i64,
    ) -> Result<Option<WorkerIterationAuthorizationRow>, StorePortError>;

    /// Enumerate only WIA records that a daemon has durably handed to a
    /// worker. This is the recovery discovery surface: an unconsumed WIA is
    /// an issued authorization, not an in-flight worker attempt.
    fn list_consumed_worker_iteration_authorizations(
        &self,
    ) -> Result<Vec<ConsumedWorkerIterationAuthorization>, StorePortError>;

    /// Consume one WIA at most once under the current fencing epoch. This
    /// records only a worker handoff, not execution, progress, or completion.
    fn consume_worker_iteration_authorization(
        &self,
        consumption: &WorkerIterationAuthorizationConsumptionRow,
    ) -> Result<(), StorePortError>;

    /// Atomically consume one WIA and bind that handoff to an exact currently
    /// leased scheduler row. D05 daemon worker dispatch must use this method;
    /// an unbound handoff never proves authority to release scheduler work.
    fn consume_worker_iteration_authorization_bound_to_scheduler_lease(
        &self,
        request: &BoundWorkerAuthorizationConsumption,
    ) -> Result<(), StorePortError>;

    /// Commit Intent, Effect admission, immutable WIA, Loop CAS, exact budget
    /// debit, events, records, and outbox rows as one authority transaction.
    /// Any failure must roll back the entire bundle.
    fn commit_candidate_admission(
        &self,
        commit: &CandidateAdmissionCommit,
    ) -> Result<CandidateAdmissionReceipt, StorePortError>;

    /// Append an immutable daemon-only authorization snapshot.
    fn append_daemon_authorization_snapshot(
        &self,
        snapshot: &DaemonAuthorizationSnapshotRow,
    ) -> Result<(), StorePortError>;

    /// Load the newest daemon authorization snapshot for an exact binding.
    /// A missing snapshot is not an authorization grant and must fail closed.
    fn load_latest_daemon_authorization_snapshot(
        &self,
        subject_ref: &str,
        target_ref: &str,
        action: &str,
        purpose: &str,
    ) -> Result<Option<DaemonAuthorizationSnapshotRow>, StorePortError>;

    /// Append a daemon-owned immutable descriptor. Non-authority clients,
    /// Pi, worker, and Provider components do not receive this write path.
    fn append_daemon_operation_descriptor(
        &self,
        descriptor: &DaemonOperationDescriptorRow,
    ) -> Result<(), StorePortError>;

    /// Resolve the exact immutable descriptor named by a candidate proposal.
    fn load_daemon_operation_descriptor(
        &self,
        descriptor_id: &ObjectId,
    ) -> Result<Option<DaemonOperationDescriptorRow>, StorePortError>;

    /// Append an immutable candidate proposal. A duplicate identity is a
    /// conflict and no replacement or mutable candidate status is allowed.
    fn append_operation_candidate_proposal(
        &self,
        proposal: &OperationCandidateProposalRow,
    ) -> Result<(), StorePortError>;

    /// Load one immutable candidate proposal by identity.
    fn load_operation_candidate_proposal(
        &self,
        candidate_id: &ObjectId,
    ) -> Result<Option<OperationCandidateProposalRow>, StorePortError>;
}

/// M5 intent-chain persistence port (UserIntentRecord →
/// IntentInterpretation candidate → TaskContract; REQ-INTENT-RECORD-001,
/// REQ-INTENT-ADMISSION-001, REQ-INTENT-SUPERSEDE-001). Implemented
/// alongside [`AuthorityStore`]/[`ProtocolStore`] by the store adapter.
/// All three families are append-only rows committed atomically with
/// their events.
pub trait IntentChainStore {
    /// Insert a UserIntentRecord row and append its event in ONE
    /// transaction. A duplicate `record_id` is a conflict.
    fn insert_user_intent(
        &self,
        record: &UserIntentRecordRow,
        event: &EventDraft,
    ) -> Result<CommitReceipt, StorePortError>;

    /// Load one UserIntentRecord by identity.
    fn load_user_intent(
        &self,
        record_id: &ObjectId,
    ) -> Result<Option<UserIntentRecordRow>, StorePortError>;

    /// List records fixed in one conversation/scope, in insertion order.
    fn list_user_intents_for_scope(
        &self,
        conversation_or_scope_ref: &str,
    ) -> Result<Vec<UserIntentRecordRow>, StorePortError>;

    /// Insert an interpretation candidate row and append its event in ONE
    /// transaction. A duplicate `interpretation_id` is a conflict.
    fn insert_interpretation(
        &self,
        interpretation: &InterpretationRow,
        event: &EventDraft,
    ) -> Result<CommitReceipt, StorePortError>;

    /// Load one interpretation candidate by identity.
    fn load_interpretation(
        &self,
        interpretation_id: &ObjectId,
    ) -> Result<Option<InterpretationRow>, StorePortError>;

    /// Insert a TaskContract row and append its event in ONE transaction.
    /// The adapter MUST verify INSIDE the transaction that the task's
    /// current epoch equals `expected_current_epoch` (0 = no contract yet)
    /// and that `contract.contract_epoch == expected_current_epoch + 1`;
    /// any mismatch is a conflict and nothing persists.
    fn insert_task_contract(
        &self,
        contract: &TaskContractRow,
        event: &EventDraft,
        expected_current_epoch: i64,
    ) -> Result<CommitReceipt, StorePortError>;

    /// Load one contract by task and epoch.
    fn load_task_contract(
        &self,
        task_ref: &str,
        contract_epoch: i64,
    ) -> Result<Option<TaskContractRow>, StorePortError>;

    /// Enumerate persisted intents bound to one task (supersede
    /// classification input), in insertion order.
    fn list_intents_for_task(&self, task_ref: &str) -> Result<Vec<IntentRow>, StorePortError>;
}

/// Append-only daemon persistence for the durable Context chain. This port
/// intentionally exposes immutable request/view records only; it grants no
/// authority to resolve, rank, admit operations, or change Task state.
pub trait ContextStore {
    /// Persist one daemon-issued immutable ContextRequest. Duplicate identity
    /// is a conflict; callers must never replace a request in place.
    fn append_context_request(&self, request: &ContextRequestRow) -> Result<(), StorePortError>;

    /// Load one immutable ContextRequest by identity.
    fn load_context_request(
        &self,
        request_id: &ObjectId,
    ) -> Result<Option<ContextRequestRow>, StorePortError>;

    /// Persist one daemon-issued immutable ContextView. Its request binding
    /// must already exist; duplicate identity is a conflict.
    fn append_context_view(&self, view: &ContextViewRow) -> Result<(), StorePortError>;

    /// Load one immutable ContextView by identity.
    fn load_context_view(
        &self,
        view_id: &ObjectId,
    ) -> Result<Option<ContextViewRow>, StorePortError>;

    fn append_workspace_context_source(
        &self,
        source: &WorkspaceContextSourceRow,
    ) -> Result<(), StorePortError>;

    fn query_context_candidate_metadata(
        &self,
        query: &ContextCandidateQuery,
    ) -> Result<Vec<ContextCandidateMetadata>, StorePortError>;

    fn load_workspace_context_source_body(
        &self,
        source_id: &ObjectId,
    ) -> Result<Option<WorkspaceContextSourceRow>, StorePortError>;
}

/// Daemon-private append-only persistence for Memory admission. This port has
/// no mutable update, retrieval, or client-authority operation.
pub trait MemoryStore {
    /// Atomically records a proposal and its daemon decision. An admitted
    /// object is permitted only with an exact `admit` decision; all other
    /// decisions leave no Memory object behind.
    fn append_memory_admission(
        &self,
        candidate: &MemoryCandidateRow,
        decision: &MemoryAdmissionDecisionRow,
        admitted_object: Option<&MemoryObjectRow>,
    ) -> Result<(), StorePortError>;

    /// Load an immutable admitted object by identity for later daemon-only
    /// consumers. It does not constitute search or public projection.
    fn load_memory_object(
        &self,
        memory_id: &ObjectId,
    ) -> Result<Option<MemoryObjectRow>, StorePortError>;

    /// Discover metadata-only candidates from the derived FTS index after
    /// filtering authoritative Memory metadata and current source bindings.
    /// The FTS index is disposable derived data; its rows never grant access
    /// to source bodies or supersede authoritative SQLite records.
    fn search_memory_candidates(
        &self,
        query: &MemorySearchQuery,
    ) -> Result<Vec<MemorySearchCandidateRow>, StorePortError>;

    /// Rebuild the disposable FTS index from current authoritative Memory
    /// objects and their bound Context-source bodies.
    fn rebuild_memory_search_index(&self) -> Result<(), StorePortError>;
}

/// Daemon-private, immutable execution inputs for one scheduler task binding.
///
/// This row closes the gap between a Context-bound TaskContract and candidate
/// admission: the scheduler must reload its Context query and daemon-created
/// admission facts from durable state rather than infer defaults from a worker
/// request or a display-oriented task projection. The canonical payload is a
/// private implementation record, not a public TaskContract extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulerExecutionPolicyRow {
    /// Exact task binding this policy may serve.
    pub task_ref: String,
    /// Exact immutable TaskContract epoch this policy may serve.
    pub contract_epoch: i64,
    /// Strong ContextRequest identity fixed by that contract.
    pub context_request_id: ObjectId,
    /// Daemon-issued canonical private policy document.
    pub canonical_json: String,
}

/// Immutable daemon-private policy persistence for pre-admission scheduling.
/// A missing, malformed, or mismatched policy is never an authorization
/// fallback: the scheduler must fail closed before it invokes Pi.
pub trait SchedulerExecutionPolicyStore {
    /// Append the policy created by daemon task admission. Duplicate task and
    /// epoch bindings are conflicts because policy cannot be replaced.
    fn append_scheduler_execution_policy(
        &self,
        policy: &SchedulerExecutionPolicyRow,
    ) -> Result<(), StorePortError>;

    /// Load the sole immutable policy for an exact scheduler task binding.
    fn load_scheduler_execution_policy(
        &self,
        task_ref: &str,
        contract_epoch: i64,
    ) -> Result<Option<SchedulerExecutionPolicyRow>, StorePortError>;
}

/// Durable source of the facts needed to reconstruct a current Context
/// authorization snapshot. Only the daemon-admin authority may append facts.
pub trait ContextAuthorizationFactStore {
    fn append_context_authorization_facts(
        &self,
        facts: &ContextAuthorizationFactsRow,
    ) -> Result<(), StorePortError>;

    fn append_context_revocation_fact(
        &self,
        fact: &ContextRevocationFactRow,
    ) -> Result<(), StorePortError>;

    fn load_latest_context_authorization_facts(
        &self,
        subject_ref: &str,
        tenant_id: &str,
    ) -> Result<Option<ContextAuthorizationFactsRow>, StorePortError>;

    fn load_current_context_revocation_epoch(
        &self,
        tenant_id: &str,
    ) -> Result<Option<i64>, StorePortError>;
}

/// Durable resolution port for the governance header carried by M5 governed
/// objects. Publication adapters consume this port instead of accepting an
/// unverified caller-supplied header (D-018).
pub trait GovernanceObjectStore {
    /// Return the immutable, schema-shaped header for exactly one persisted
    /// governed object. Missing identities return `None`; malformed or
    /// ambiguous durable data fails closed.
    fn load_governed_object_header(
        &self,
        object_id: &ObjectId,
    ) -> Result<Option<GovernedObjectHeader>, StorePortError>;
}

/// One persisted loop progress fact (REQ-RUN-007: progress is a verifiable
/// state difference, reduced uncertainty or satisfied precondition —
/// recorded as a typed durable fact, never a transcript-length heuristic).
/// Append-only; the stagnation and retry counters fold over these rows.
#[derive(Debug, Clone, PartialEq)]
pub struct ProgressFactRow {
    /// Loop the fact belongs to.
    pub loop_object_id: ObjectId,
    /// Iteration the fact was recorded for (monotonic from 1).
    pub iteration: i64,
    /// `advanced`, `none`, `uncertain` or `blocked` (schema progress set).
    pub status: String,
    /// Deterministic fingerprint of the action taken this iteration
    /// (REQ-RUN-008 retry accounting key).
    pub action_fingerprint: String,
    /// Canonical JSON array of evidence references.
    pub evidence_refs_json: String,
    /// Wall time the fact was recorded.
    pub recorded_at: WallTimestamp,
    /// Fencing epoch of the recording writer (verified in-transaction,
    /// same store-side sink discipline as checkpoints).
    pub fencing_epoch: i64,
}

/// M5 harness-loop fact persistence port (progress facts for stagnation
/// and retry accounting; REQ-RUN-005/007/008). Implemented by the store
/// adapter next to [`ProtocolStore`].
pub trait HarnessStore {
    /// Append one progress fact (append-only). The adapter MUST verify
    /// `fencing_epoch` inside the transaction and reject stale writers,
    /// and MUST reject a duplicate `(loop_object_id, iteration)` pair.
    fn append_progress_fact(&self, fact: &ProgressFactRow) -> Result<(), StorePortError>;

    /// List progress facts of one loop in iteration order.
    fn list_progress_facts(
        &self,
        loop_object_id: &ObjectId,
    ) -> Result<Vec<ProgressFactRow>, StorePortError>;
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
