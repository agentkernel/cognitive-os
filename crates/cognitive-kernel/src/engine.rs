//! The centralized deterministic transition gate (REQ-STATE-003,
//! `docs/standards/state-and-transition-contract.md` sections 3 and 5).
//!
//! Every write of authority state goes through [`TransitionEngine`]. The
//! gate performs, in fixed order: table digest pinning, authoritative
//! state/version comparison (CAS), table row lookup with reason matching,
//! guard verification, required-evidence verification, deterministic hard
//! budget admission, and the atomic commit (object CAS + event append +
//! transition record + budget debit + outbox, one transaction).
//!
//! Determinism boundary (architecture invariant): every input to this gate
//! is data — guard attestations and evidence references are established by
//! deterministic upstream code, never by a model. Probabilistic components
//! may PROPOSE a target state or reason upstream; nothing in this module
//! calls or trusts them. Rejections fail closed with the registered error
//! codes mapped in [`crate::error`] and leave state unchanged.

use crate::budget::{BudgetCharge, BudgetState};
use crate::error::{RejectionKind, TransitionRejection};
use crate::ports::{
    AuthorityStore, BudgetCas, Clock, CommitReceipt, EventDraft, IdGenerator, ObjectAdmission,
    ObjectCas, OutboxDraft, PortFailure, RecordDraft, StorePortError, StoredObject,
    TransitionCommit,
};
use cognitive_contracts::canonical;
use cognitive_contracts::generated::object_reference::StrongReference;
use cognitive_contracts::generated::state_transition_record as generated_record;
use cognitive_contracts::generated::state_transition_request as generated_request;
use cognitive_domain::transitions::EdgeLookupError;
use cognitive_domain::{
    BudgetId, EventId, LifecycleDomain, LoadedTable, ObjectId, ReasonCode, RecordId, StateName,
    UriRef, Version, WallTimestamp, table,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// Event type appended for an accepted state transition.
pub const EVENT_TYPE_TRANSITION_COMMITTED: &str = "cognitiveos.state.transition.committed";

/// Event type appended when a new governed object is admitted.
pub const EVENT_TYPE_OBJECT_ADMITTED: &str = "cognitiveos.object.admitted";

/// Structured transition reason (`state-transition-request.schema.json`
/// `$defs/reason`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reason {
    /// Reason code matched against the table row's `reason_codes`.
    pub code: ReasonCode,
    /// Free-text detail (never a machine code).
    pub detail: Option<String>,
}

/// Causation binding (`state-transition-request.schema.json`
/// `$defs/causation`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Causation {
    /// What caused this request.
    pub causation_id: UriRef,
    /// Correlation chain identity.
    pub correlation_id: UriRef,
}

/// The transition-table version + canonical digest a requester decided
/// under. The gate rejects a request whose pin does not match the loaded
/// registered asset (`state-and-transition-contract.md` section 2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TablePin {
    /// Table `version` field.
    pub version: String,
    /// Canonical digest of the table asset (domain `spec-set/0.1`).
    pub digest: String,
}

impl TablePin {
    /// Pin the currently loaded registered table of `domain`.
    pub fn current(domain: LifecycleDomain) -> Result<TablePin, TransitionRejection> {
        let loaded = load_table(domain)?;
        Ok(TablePin {
            version: loaded.table.version.clone(),
            digest: loaded.digest.clone(),
        })
    }
}

/// Optional hard-budget charge admitted and debited with the transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BudgetChargeCommand {
    /// Ledger row to debit.
    pub budget_id: BudgetId,
    /// Charge per dimension.
    pub charge: BudgetCharge,
}

/// One governed transition request, fully deterministic input.
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionCommand {
    /// Request identity (URI reference).
    pub request_id: UriRef,
    /// Lifecycle domain of the subject.
    pub domain: LifecycleDomain,
    /// Subject object identity.
    pub object_id: ObjectId,
    /// Subject URI reference recorded in record and event.
    pub subject_ref: UriRef,
    /// State the requester observed.
    pub from: StateName,
    /// Requested target state.
    pub to: StateName,
    /// Authoritative version the requester decided against.
    pub expected_version: Version,
    /// Structured reason.
    pub reason: Reason,
    /// Causation binding.
    pub causation: Causation,
    /// Acting principal chain reference.
    pub actor_ref: UriRef,
    /// Deciding authority reference.
    pub authority_ref: UriRef,
    /// Wall time of the request.
    pub requested_at: WallTimestamp,
    /// Pinned table version and digest.
    pub table_pin: TablePin,
    /// Guard names established true by deterministic upstream code. A guard
    /// missing from this set is indeterminate and fails closed.
    pub established_guards: BTreeSet<String>,
    /// Evidence by required-evidence item name.
    pub evidence: BTreeMap<String, StrongReference>,
    /// Optional hard-budget charge.
    pub budget: Option<BudgetChargeCommand>,
    /// Outbox destinations for the committed event.
    pub outbox_destinations: Vec<String>,
    /// Writer fencing epoch (F-014): when set, the store verifies it
    /// against the current epoch inside the commit transaction and fences
    /// stale writers out. `None` = unfenced path (M2 compatibility).
    pub fencing_epoch: Option<i64>,
}

/// Admission of a new governed object at the table's initial state.
#[derive(Debug, Clone, PartialEq)]
pub struct AdmitCommand {
    /// New object identity.
    pub object_id: ObjectId,
    /// Lifecycle domain.
    pub domain: LifecycleDomain,
    /// Subject URI reference.
    pub subject_ref: UriRef,
    /// Opaque object body.
    pub body: Value,
    /// Acting principal chain reference.
    pub actor_ref: UriRef,
    /// Deciding authority reference.
    pub authority_ref: UriRef,
    /// Correlation chain identity.
    pub correlation_id: UriRef,
    /// Outbox destinations for the admission event.
    pub outbox_destinations: Vec<String>,
    /// Writer fencing epoch (F-014); `None` = unfenced path.
    pub fencing_epoch: Option<i64>,
}

/// Receipt of one accepted, durably committed transition.
#[derive(Debug, Clone, PartialEq)]
pub struct CommittedTransition {
    /// Committed record identity.
    pub record_id: RecordId,
    /// Committed event identity.
    pub event_id: EventId,
    /// Global event-log sequence.
    pub event_sequence: i64,
    /// Authoritative version after the commit.
    pub after_version: Version,
    /// Commit wall time.
    pub committed_at: WallTimestamp,
}

/// Receipt of one accepted object admission.
#[derive(Debug, Clone, PartialEq)]
pub struct AdmittedObject {
    /// Committed admission event identity.
    pub event_id: EventId,
    /// Global event-log sequence.
    pub event_sequence: i64,
    /// Initial state assigned from the table.
    pub initial_state: StateName,
    /// Initial authoritative version (1).
    pub version: Version,
    /// Admission wall time.
    pub admitted_at: WallTimestamp,
}

/// The deterministic kernel gate over one store, clock and ID source.
pub struct TransitionEngine<'a, S, C, G> {
    store: &'a S,
    clock: &'a C,
    ids: &'a G,
}

fn reject(kind: RejectionKind, detail: impl Into<String>) -> TransitionRejection {
    TransitionRejection {
        kind,
        detail: detail.into(),
        current_state: None,
        current_version: None,
        available_exits: Vec::new(),
        effect_outcome_unknown: false,
    }
}

fn reject_at(
    kind: RejectionKind,
    detail: impl Into<String>,
    table: &LoadedTable,
    current: &StoredObject,
) -> TransitionRejection {
    TransitionRejection {
        kind,
        detail: detail.into(),
        current_state: Some(current.state.clone()),
        current_version: Some(current.version),
        available_exits: table
            .legal_exits(&current.state)
            .into_iter()
            .map(str::to_owned)
            .collect(),
        effect_outcome_unknown: current.domain == LifecycleDomain::Effect
            && current.state.as_str() == "OUTCOME_UNKNOWN",
    }
}

fn load_table(domain: LifecycleDomain) -> Result<&'static LoadedTable, TransitionRejection> {
    table(domain).map_err(|err| {
        reject(
            RejectionKind::TableAssetInvalid,
            format!("registered table asset unusable: {err}"),
        )
    })
}

fn store_error(err: StorePortError) -> TransitionRejection {
    match err {
        StorePortError::Conflict { detail } => reject(RejectionKind::StoreConflict, detail),
        StorePortError::Unavailable { detail } => reject(RejectionKind::StoreUnavailable, detail),
    }
}

fn port_failure(what: &str, err: PortFailure) -> TransitionRejection {
    reject(
        RejectionKind::StoreUnavailable,
        format!("{what} unavailable: {}", err.detail),
    )
}

fn canonical_text(value: &Value) -> Result<String, TransitionRejection> {
    let bytes = canonical::canonical_bytes_of_value(value).map_err(|err| {
        reject(
            RejectionKind::InvalidCommand,
            format!("canonical encoding failed: {err}"),
        )
    })?;
    String::from_utf8(bytes).map_err(|err| {
        reject(
            RejectionKind::InvalidCommand,
            format!("canonical bytes not UTF-8: {err}"),
        )
    })
}

impl<'a, S, C, G> TransitionEngine<'a, S, C, G>
where
    S: AuthorityStore,
    C: Clock,
    G: IdGenerator,
{
    /// Build a gate over the given adapters.
    pub fn new(store: &'a S, clock: &'a C, ids: &'a G) -> Self {
        Self { store, clock, ids }
    }

    fn next_event_id(&self) -> Result<EventId, TransitionRejection> {
        let raw = self
            .ids
            .next_uuid_v7()
            .map_err(|err| port_failure("id generator", err))?;
        EventId::parse(&raw).map_err(|err| {
            reject(
                RejectionKind::StoreUnavailable,
                format!("id generator produced non-canonical uuid: {err}"),
            )
        })
    }

    fn next_record_id(&self) -> Result<RecordId, TransitionRejection> {
        let raw = self
            .ids
            .next_uuid_v7()
            .map_err(|err| port_failure("id generator", err))?;
        RecordId::parse(&raw).map_err(|err| {
            reject(
                RejectionKind::StoreUnavailable,
                format!("id generator produced non-canonical uuid: {err}"),
            )
        })
    }

    /// Admit a new governed object at its table's initial state, version 1,
    /// atomically with its admission event.
    pub fn admit_object(&self, cmd: &AdmitCommand) -> Result<AdmittedObject, TransitionRejection> {
        let loaded = load_table(cmd.domain)?;
        let initial_state = StateName::parse(&loaded.table.initial_state).map_err(|err| {
            reject(
                RejectionKind::TableAssetInvalid,
                format!("initial state grammar: {err}"),
            )
        })?;
        let admitted_at = self.clock.now().map_err(|err| port_failure("clock", err))?;
        let event_id = self.next_event_id()?;

        let event_value = json!({
            "event_id": event_id.as_str(),
            "event_type": EVENT_TYPE_OBJECT_ADMITTED,
            "domain": cmd.domain.as_str(),
            "object_id": cmd.object_id.as_str(),
            "subject_ref": cmd.subject_ref.as_str(),
            "after_state": initial_state.as_str(),
            "after_version": Version::INITIAL.get(),
            "actor_ref": cmd.actor_ref.as_str(),
            "authority_ref": cmd.authority_ref.as_str(),
            "causation": {
                "causation_id": cmd.correlation_id.as_str(),
                "correlation_id": cmd.correlation_id.as_str(),
            },
            "event_time": admitted_at.as_str(),
            "table_version": loaded.table.version,
            "table_digest": loaded.digest,
        });
        let canonical_json = canonical_text(&event_value)?;

        let admission = ObjectAdmission {
            object: StoredObject {
                object_id: cmd.object_id.clone(),
                domain: cmd.domain,
                state: initial_state.clone(),
                version: Version::INITIAL,
                body: cmd.body.clone(),
            },
            admitted_at: admitted_at.clone(),
            event: EventDraft {
                event_id: event_id.clone(),
                object_id: cmd.object_id.clone(),
                domain: cmd.domain,
                object_version: Version::INITIAL,
                event_type: EVENT_TYPE_OBJECT_ADMITTED.to_owned(),
                canonical_json,
            },
            outbox: cmd
                .outbox_destinations
                .iter()
                .map(|destination| OutboxDraft {
                    event_id: event_id.clone(),
                    destination: destination.clone(),
                })
                .collect(),
            fencing_epoch: cmd.fencing_epoch,
        };
        let receipt: CommitReceipt = self.store.admit_object(&admission).map_err(store_error)?;
        Ok(AdmittedObject {
            event_id,
            event_sequence: receipt.event_sequence,
            initial_state,
            version: Version::INITIAL,
            admitted_at,
        })
    }

    /// Create a hard-budget ledger row (version 1).
    pub fn create_budget(
        &self,
        budget_id: &BudgetId,
        state: &BudgetState,
    ) -> Result<(), TransitionRejection> {
        let value = serde_json::to_value(state).map_err(|err| {
            reject(
                RejectionKind::InvalidCommand,
                format!("budget state serialization: {err}"),
            )
        })?;
        let canonical_json = canonical_text(&value)?;
        let created_at = self.clock.now().map_err(|err| port_failure("clock", err))?;
        self.store
            .create_budget(budget_id, &canonical_json, &created_at)
            .map_err(store_error)
    }

    /// Apply one governed transition through the full deterministic gate.
    /// On rejection nothing is written and the authoritative state is
    /// unchanged; the rejection carries the registered error code, current
    /// state/version and safe available exits.
    pub fn commit_transition(
        &self,
        cmd: &TransitionCommand,
    ) -> Result<CommittedTransition, TransitionRejection> {
        // 1. Registered table, pinned by version + canonical digest.
        let loaded = load_table(cmd.domain)?;
        if cmd.table_pin.version != loaded.table.version || cmd.table_pin.digest != loaded.digest {
            return Err(reject(
                RejectionKind::TablePinMismatch,
                format!(
                    "request pinned table {}/{}, loaded registered table is {}/{}",
                    cmd.table_pin.version,
                    cmd.table_pin.digest,
                    loaded.table.version,
                    loaded.digest
                ),
            ));
        }

        // 2. Authoritative current row.
        let current = self
            .store
            .load_object(cmd.domain, &cmd.object_id)
            .map_err(store_error)?
            .ok_or_else(|| {
                reject(
                    RejectionKind::ObjectNotFound,
                    format!("no governed object {} in {}", cmd.object_id, cmd.domain),
                )
            })?;
        if current.domain != cmd.domain {
            return Err(reject(
                RejectionKind::DomainMismatch,
                format!(
                    "object {} belongs to {}, not {}",
                    cmd.object_id, current.domain, cmd.domain
                ),
            ));
        }

        // 3. State currency: the requester's `from` must be the
        //    authoritative current state.
        if cmd.from != current.state {
            return Err(reject_at(
                RejectionKind::FromStateMismatch,
                format!(
                    "request from {} but authoritative state is {}",
                    cmd.from, current.state
                ),
                loaded,
                &current,
            ));
        }

        // 4. CAS: expected version must equal the authoritative version
        //    (STATE_CONFLICT on mismatch, never last-write-wins).
        if cmd.expected_version != current.version {
            return Err(reject_at(
                RejectionKind::VersionMismatch,
                format!(
                    "expected_version {} but authoritative version is {}",
                    cmd.expected_version, current.version
                ),
                loaded,
                &current,
            ));
        }
        let next_version = current.version.next().map_err(|err| {
            reject_at(
                RejectionKind::InvalidCommand,
                format!("version overflow: {err}"),
                loaded,
                &current,
            )
        })?;

        // 5. Table row lookup: exactly one row for (from, to, reason).
        let edge = match loaded.find_edge(&cmd.from, &cmd.to, cmd.reason.code.as_str()) {
            Ok(edge) => edge,
            Err(err) => {
                let (kind, what) = match err {
                    EdgeLookupError::UnknownFromState | EdgeLookupError::UnknownToState => {
                        (RejectionKind::UnknownState, "state not in pinned table")
                    }
                    EdgeLookupError::TerminalFrom => (
                        RejectionKind::TerminalState,
                        "no legal transition leaves a terminal state",
                    ),
                    EdgeLookupError::NoMatchingEdge => (
                        RejectionKind::IllegalTransition,
                        "no registered transition row matches the state pair",
                    ),
                    EdgeLookupError::ReasonNotAllowed => (
                        RejectionKind::ReasonNotAllowed,
                        "no matching row allows the requested reason",
                    ),
                };
                return Err(reject_at(
                    kind,
                    format!(
                        "{what}: {} -> {} reason {}",
                        cmd.from, cmd.to, cmd.reason.code
                    ),
                    loaded,
                    &current,
                ));
            }
        };

        // 6. Guards: every guard of the row must be deterministically
        //    established; absent or false is indeterminate and fails closed.
        for guard in &edge.guards {
            if !cmd.established_guards.contains(guard) {
                return Err(reject_at(
                    RejectionKind::GuardUnsatisfied,
                    format!("guard {guard} not deterministically established"),
                    loaded,
                    &current,
                ));
            }
        }

        // 7. Required evidence: every declared item must be present.
        for item in &edge.required_evidence {
            if !cmd.evidence.contains_key(item) {
                return Err(reject_at(
                    RejectionKind::EvidenceMissing,
                    format!("required evidence {item} absent"),
                    loaded,
                    &current,
                ));
            }
        }

        // 8. Deterministic hard budget admission (fail closed BEFORE any
        //    write; the debit itself joins the atomic commit below).
        let budget_cas = match &cmd.budget {
            None => None,
            Some(budget_cmd) => {
                let stored = self
                    .store
                    .load_budget(&budget_cmd.budget_id)
                    .map_err(store_error)?
                    .ok_or_else(|| {
                        reject_at(
                            RejectionKind::BudgetNotFound,
                            format!("no budget ledger row {}", budget_cmd.budget_id),
                            loaded,
                            &current,
                        )
                    })?;
                let debited =
                    stored
                        .state
                        .check_and_debit(&budget_cmd.charge)
                        .map_err(|exhausted| {
                            reject_at(
                                RejectionKind::BudgetExhausted,
                                exhausted.to_string(),
                                loaded,
                                &current,
                            )
                        })?;
                let next_budget_version = stored.version.next().map_err(|err| {
                    reject_at(
                        RejectionKind::InvalidCommand,
                        format!("budget version overflow: {err}"),
                        loaded,
                        &current,
                    )
                })?;
                let state_value = serde_json::to_value(&debited).map_err(|err| {
                    reject(
                        RejectionKind::InvalidCommand,
                        format!("budget state serialization: {err}"),
                    )
                })?;
                Some(BudgetCas {
                    budget_id: budget_cmd.budget_id.clone(),
                    expected_version: stored.version,
                    next_version: next_budget_version,
                    next_state_canonical_json: canonical_text(&state_value)?,
                })
            }
        };

        // 9. Compose the committed record and event values.
        let committed_at = self.clock.now().map_err(|err| port_failure("clock", err))?;
        let event_id = self.next_event_id()?;
        let record_id = self.next_record_id()?;

        let mut evidence_refs: Vec<&StrongReference> = cmd.evidence.values().collect();
        evidence_refs.sort_by(|a, b| {
            (a.id.0.as_str(), a.object_version).cmp(&(b.id.0.as_str(), b.object_version))
        });
        evidence_refs.dedup_by(|a, b| a == b);

        // Committed record via the schema-generated binding
        // (`state-transition-record.schema.json`, contracts-codegen v0.2;
        // the canonical member set is identical to the pre-binding
        // hand-rolled composition — replay/record bytes unchanged).
        let reason_value = generated_request::Reason {
            code: cmd.reason.code.as_str().to_owned(),
            detail: cmd.reason.detail.clone(),
        };
        let record = generated_record::CommittedStateTransitionRecord {
            actor_ref: cmd.actor_ref.as_str().to_owned(),
            after_state: cmd.to.as_str().to_owned(),
            after_version: next_version.get(),
            authority_ref: cmd.authority_ref.as_str().to_owned(),
            before_state: cmd.from.as_str().to_owned(),
            before_version: cmd.expected_version.get(),
            causation: generated_request::Causation {
                causation_id: cmd.causation.causation_id.as_str().to_owned(),
                correlation_id: cmd.causation.correlation_id.as_str().to_owned(),
                request_ref: Some(cmd.request_id.as_str().to_owned()),
            },
            committed_at: committed_at.as_str().to_owned(),
            domain: cmd.domain.as_str().to_owned(),
            event_ref: None,
            evidence_refs: evidence_refs.into_iter().cloned().collect(),
            expected_version: cmd.expected_version.get(),
            metadata: edge
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.as_object().cloned()),
            reason: reason_value.clone(),
            record_id: record_id.as_str().to_owned(),
            request_ref: cmd.request_id.as_str().to_owned(),
            requested_at: cmd.requested_at.as_str().to_owned(),
            subject_ref: cmd.subject_ref.as_str().to_owned(),
            table_digest: loaded.digest.clone(),
            table_version: loaded.table.version.clone(),
        };
        let record_value = serde_json::to_value(&record).map_err(|err| {
            reject(
                RejectionKind::InvalidCommand,
                format!("record serialization: {err}"),
            )
        })?;
        let record_json = canonical_text(&record_value)?;

        let event_value = json!({
            "event_id": event_id.as_str(),
            "event_type": EVENT_TYPE_TRANSITION_COMMITTED,
            "domain": cmd.domain.as_str(),
            "object_id": cmd.object_id.as_str(),
            "subject_ref": cmd.subject_ref.as_str(),
            "before_state": cmd.from.as_str(),
            "after_state": cmd.to.as_str(),
            "before_version": cmd.expected_version.get(),
            "after_version": next_version.get(),
            "reason": reason_value,
            "causation": {
                "causation_id": cmd.causation.causation_id.as_str(),
                "correlation_id": cmd.causation.correlation_id.as_str(),
                "request_ref": cmd.request_id.as_str(),
            },
            "record_ref": record_id.as_str(),
            "event_time": committed_at.as_str(),
            "table_version": loaded.table.version,
            "table_digest": loaded.digest,
        });
        let event_json = canonical_text(&event_value)?;

        // 10. One atomic authoritative commit.
        let commit = TransitionCommit {
            cas: ObjectCas {
                object_id: cmd.object_id.clone(),
                domain: cmd.domain,
                from_state: cmd.from.clone(),
                to_state: cmd.to.clone(),
                expected_version: cmd.expected_version,
                next_version,
                committed_at: committed_at.clone(),
            },
            event: EventDraft {
                event_id: event_id.clone(),
                object_id: cmd.object_id.clone(),
                domain: cmd.domain,
                object_version: next_version,
                event_type: EVENT_TYPE_TRANSITION_COMMITTED.to_owned(),
                canonical_json: event_json,
            },
            record: RecordDraft {
                record_id: record_id.clone(),
                object_id: cmd.object_id.clone(),
                domain: cmd.domain,
                object_version: next_version,
                canonical_json: record_json,
            },
            budget: budget_cas,
            outbox: cmd
                .outbox_destinations
                .iter()
                .map(|destination| OutboxDraft {
                    event_id: event_id.clone(),
                    destination: destination.clone(),
                })
                .collect(),
            fencing_epoch: cmd.fencing_epoch,
        };
        let receipt = self
            .store
            .commit_transition(&commit)
            .map_err(|err| match err {
                StorePortError::Conflict { detail } => {
                    reject_at(RejectionKind::StoreConflict, detail, loaded, &current)
                }
                other => store_error(other),
            })?;

        Ok(CommittedTransition {
            record_id,
            event_id,
            event_sequence: receipt.event_sequence,
            after_version: next_version,
            committed_at,
        })
    }
}
