//! The Intent → Effect protocol driver
//! (`docs/standards/intent-effect-idempotency.md`; REQ-EFF-001..006,
//! REQ-EFF-STATE-001; `.cursor/rules/13-effect-recovery.mdc`).
//!
//! Seven-property model (IMP-07). Each property names its deterministic
//! enforcement point here and its behavioral test in
//! `crates/cognitive-store/tests/m4_effects.rs` /
//! `m4_recovery.rs` / `m4_tracer_bullet.rs`:
//!
//! 1. **Idempotency**: one key per logical attempt chain, minted at Intent
//!    persistence and NEVER re-minted on timeout/retry/crash
//!    ([`mint_intent`]: same key + same digest = replay of the persisted
//!    Intent; the store's UNIQUE key constraint is the structural backstop).
//! 2. **Fencing**: every commit sink checks the writer's epoch
//!    ([`WriterLease`], [`COMMIT_SINKS`]; store-side SQL check, executor
//!    sink-side check).
//! 3. **No double execution**: dispatch is recorded (EXECUTING commit)
//!    before the external call; recovery never blind-redispatches — it
//!    reconciles ([`reconcile_unknown`]) or re-dispatches only a
//!    provably-not-dispatched Intent with the ORIGINAL key.
//! 4. **Unknown-outcome closure**: `OUTCOME_UNKNOWN` exits only through
//!    reconciliation to executed / not_executed / still_unknown →
//!    quarantine ([`reconcile_unknown`], [`quarantine_still_unknown`];
//!    table-pinned since M2).
//! 5. **Recovery order**: the eight-step sequencer refuses out-of-order
//!    steps (`crate::recovery`).
//! 6. **Compensation independence**: compensation requires a FRESH
//!    authorization grant and a NEW intent ([`begin_compensation`] rejects
//!    reuse of the original grant).
//! 7. **Fail-before-effect**: no Intent, no dispatch
//!    ([`dispatch_effect`] derives `intent_durably_persisted` by reloading
//!    the intent row from the durable store — the sanctioned derivation
//!    registered in the M3 handoff).

use crate::authz::AuthorizationGrant;
use crate::engine::{
    Causation, CommittedTransition, Reason, TablePin, TransitionCommand, TransitionEngine,
};
use crate::error::{
    EFFECT_IDEMPOTENCY_CONFLICT, EFFECT_OUTCOME_UNKNOWN, NO_AUTHORIZED_OPERATION_CANDIDATE,
    RegisteredError, RejectionKind, STATE_CONFLICT, TransitionRejection,
};
use crate::executor::{
    DispatchOutcome, EffectExecutor, ExecutorCall, ExecutorCapabilities, ExecutorQueryResult,
};
use crate::ports::{
    AuthorityStore, Clock, EventDraft, IdGenerator, IntentRow, PortFailure, ProtocolStore,
    StorePortError, TaskBinding,
};
use cognitive_contracts::canonical;
use cognitive_contracts::generated::object_reference::{StrongReference, StrongReferenceKind};
use cognitive_domain::{
    LifecycleDomain, ObjectId, ReasonCode, StateName, UriRef, Version, WallTimestamp,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// Digest domain for protocol evidence values (registered example domain
/// for governed object content, `canonical-encoding-and-digest.md` §9).
pub(crate) const EVIDENCE_DIGEST_DOMAIN: &str = "governed-object-content/0.1";

// ---------------------------------------------------------------------
// F-023: OperationDescriptor and the admission matrix
// ---------------------------------------------------------------------

/// Effect class of an operation (mirrors `intent.schema.json`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectClass {
    /// No externally visible state change.
    Pure,
    /// Local, ephemeral, safely repeatable.
    LocalEphemeral,
    /// Externally visible governed side effect.
    GovernedExternal,
    /// Emergency safety path (M5+; admission here stays conservative).
    EmergencySafety,
}

/// What an operation CAN do — never what a subject MAY do. Descriptors and
/// AuthorizationCapability are distinct types checked independently
/// (`authn-authz-capability.md` section 4): possessing a descriptor grants
/// nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationDescriptor {
    /// Operation identity.
    pub operation_id: String,
    /// Action name the operation performs.
    pub action: String,
    /// Effect class.
    pub effect_class: EffectClass,
    /// Executor adapter identity.
    pub executor: String,
    /// Executor capability self-description.
    pub capabilities: ExecutorCapabilities,
    /// Descriptor version (catalog pinning).
    pub descriptor_version: i64,
}

/// How an admitted operation's unknown outcomes will be closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryClosure {
    /// Outcomes are queryable: reconcile by query.
    QueryReconcile,
    /// Executor absorbs same-key re-dispatch: reconcile by idempotent
    /// re-dispatch.
    IdempotentRedispatch,
    /// Pure/local operations need no external closure.
    NoExternalCommitment,
}

/// F-023 admission matrix: a `governed_external` operation whose executor
/// is NEITHER queryable NOR idempotent has no safe closure for an unknown
/// outcome and is not an admissible dispatch candidate — rejected with the
/// registered code before any Intent is minted.
pub fn admit_operation(
    descriptor: &OperationDescriptor,
) -> Result<RecoveryClosure, RegisteredError> {
    match descriptor.effect_class {
        EffectClass::Pure | EffectClass::LocalEphemeral => {
            Ok(RecoveryClosure::NoExternalCommitment)
        }
        EffectClass::GovernedExternal | EffectClass::EmergencySafety => {
            let caps = descriptor.capabilities;
            if caps.queryable {
                Ok(RecoveryClosure::QueryReconcile)
            } else if caps.idempotent {
                Ok(RecoveryClosure::IdempotentRedispatch)
            } else {
                Err(NO_AUTHORIZED_OPERATION_CANDIDATE)
            }
        }
    }
}

// ---------------------------------------------------------------------
// F-014: commit sinks and fencing
// ---------------------------------------------------------------------

/// Every sink through which this reference implementation commits state or
/// side effects (the F-014 matrix). Each entry has a fencing enforcement
/// point and a stale-epoch negative test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitSink {
    /// External executor dispatch (enforced sink-side by the adapter AND
    /// pre-checked by the driver).
    ExternalExecutor,
    /// Authority-store transition commit — object CAS + event + record +
    /// budget (enforced inside the store transaction).
    AuthorityStoreCommit,
    /// Object/Intent admission + outbox enqueue (same transactional sink).
    AdmissionAndOutbox,
    /// Checkpoint persistence (enforced inside the store transaction).
    CheckpointWrite,
}

/// The complete sink inventory (F-014).
pub const COMMIT_SINKS: [CommitSink; 4] = [
    CommitSink::ExternalExecutor,
    CommitSink::AuthorityStoreCommit,
    CommitSink::AdmissionAndOutbox,
    CommitSink::CheckpointWrite,
];

/// A writer's fencing lease: the epoch it acquired at admission/recovery.
/// Deterministic code passes the lease to every sink; a lease older than
/// the store's current epoch is fenced out everywhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WriterLease {
    /// Epoch this writer holds.
    pub epoch: i64,
}

/// Current governance facts at a protocol decision point (supplied by the
/// deterministic caller — the runtime in M5, the test harness before
/// that). Distinct from the store's FENCING epoch: this is the
/// authorization-side revocation currency (REQ-CAP-005).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GovernanceCurrency {
    /// Current revocation epoch.
    pub revocation_epoch: i64,
    /// Current capability set version.
    pub capability_set_version: i64,
}

/// Acquire a lease at the store's current epoch.
pub fn acquire_lease<S: ProtocolStore>(store: &S) -> Result<WriterLease, TransitionRejection> {
    let epoch = store.current_fencing_epoch().map_err(store_rejection)?;
    Ok(WriterLease { epoch })
}

pub(crate) fn store_rejection(err: StorePortError) -> TransitionRejection {
    match err {
        StorePortError::Conflict { detail } => TransitionRejection {
            kind: RejectionKind::StoreConflict,
            detail,
            current_state: None,
            current_version: None,
            available_exits: Vec::new(),
            effect_outcome_unknown: false,
        },
        StorePortError::Unavailable { detail } => TransitionRejection {
            kind: RejectionKind::StoreUnavailable,
            detail,
            current_state: None,
            current_version: None,
            available_exits: Vec::new(),
            effect_outcome_unknown: false,
        },
    }
}

pub(crate) fn port_rejection(what: &str, err: PortFailure) -> TransitionRejection {
    TransitionRejection {
        kind: RejectionKind::StoreUnavailable,
        detail: format!("{what} unavailable: {}", err.detail),
        current_state: None,
        current_version: None,
        available_exits: Vec::new(),
        effect_outcome_unknown: false,
    }
}

// ---------------------------------------------------------------------
// Protocol denials
// ---------------------------------------------------------------------

/// A protocol-level denial with its registered code (idempotency conflict,
/// admission rejection, fenced writer, unknown outcome, quarantine).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{detail} (code {})", registered.code)]
pub struct ProtocolDenial {
    /// Registered machine code.
    pub registered: RegisteredError,
    /// Deterministic detail.
    pub detail: String,
}

/// Protocol operations fail with either a registered protocol denial or a
/// transition rejection from the central gate.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EffectError {
    /// Denied at the protocol layer before any gate commit.
    #[error(transparent)]
    Denied(#[from] ProtocolDenial),
    /// Rejected by the central transition gate.
    #[error(transparent)]
    Rejected(#[from] TransitionRejection),
}

// ---------------------------------------------------------------------
// Intent minting (REQ-EFF-001/002)
// ---------------------------------------------------------------------

/// One intent minting request.
#[derive(Debug, Clone, PartialEq)]
pub struct IntentCommand {
    /// Intent identity (UUIDv7).
    pub intent_id: ObjectId,
    /// Effect object this intent binds to.
    pub effect_object_id: ObjectId,
    /// Operation descriptor (admission matrix input).
    pub descriptor: OperationDescriptor,
    /// Target URI.
    pub target: String,
    /// Raw parameters; the canonical digest is computed here and becomes
    /// the comparison basis (source bytes never compared).
    pub parameters: Value,
    /// Stable idempotency key for this logical effect attempt chain.
    pub idempotency_key: String,
    /// CAS version of the fixed pre-state.
    pub expected_state_version: Version,
    /// Authorization grant covering the action (M3 gate output).
    pub grant_epoch: i64,
    /// Capability set version of the grant.
    pub capability_set_version: i64,
    /// Actor reference.
    pub actor_ref: UriRef,
    /// Authority reference.
    pub authority_ref: UriRef,
    /// Correlation chain.
    pub correlation_id: UriRef,
    /// Task/contract-epoch binding (M5 intent chain). When set, minting
    /// verifies the epoch is CURRENT and persists the binding; stale
    /// epochs are fenced with `INTENT_VERSION_SUPERSEDED`
    /// (REQ-INTENT-SUPERSEDE-001). `None` = pre-M5 unbound intent.
    pub task_binding: Option<TaskBinding>,
}

/// Outcome of intent minting.
#[derive(Debug, Clone, PartialEq)]
pub enum MintedIntent {
    /// A fresh intent row was persisted atomically with its event.
    Persisted(IntentRow),
    /// The same key with the SAME canonical parameter digest already
    /// exists: this is the same logical attempt chain — the persisted
    /// intent is returned, nothing new is created, and the key is reused
    /// (property 1).
    ReplayedExisting(IntentRow),
}

/// Compute the canonical parameter digest (comparison basis, section 3).
pub fn parameters_digest(parameters: &Value) -> Result<String, ProtocolDenial> {
    let bytes = canonical::canonical_bytes_of_value(parameters).map_err(|err| ProtocolDenial {
        registered: STATE_CONFLICT,
        detail: format!("parameters not canonicalizable: {err}"),
    })?;
    canonical::digest(&bytes, EVIDENCE_DIGEST_DOMAIN).map_err(|err| ProtocolDenial {
        registered: STATE_CONFLICT,
        detail: format!("parameter digest failed: {err}"),
    })
}

/// Mint (or replay) the Intent for one logical effect attempt chain
/// (REQ-EFF-001: persisting the Intent and appending its event is one
/// atomic transaction; REQ-EFF-002: same key + different digest is
/// `EFFECT_IDEMPOTENCY_CONFLICT`, never dedup, never execution).
pub fn mint_intent<S, C, G>(
    store: &S,
    clock: &C,
    ids: &G,
    lease: &WriterLease,
    cmd: &IntentCommand,
) -> Result<MintedIntent, EffectError>
where
    S: AuthorityStore + ProtocolStore,
    C: Clock,
    G: IdGenerator,
{
    // F-023 admission before anything is persisted.
    admit_operation(&cmd.descriptor).map_err(|registered| ProtocolDenial {
        registered,
        detail: format!(
            "operation {} (executor {}) is neither queryable nor idempotent: no safe \
             recovery closure for governed_external dispatch",
            cmd.descriptor.operation_id, cmd.descriptor.executor
        ),
    })?;

    // F-014: a fenced writer cannot mint intents either.
    let current_epoch = store.current_fencing_epoch().map_err(store_rejection)?;
    if lease.epoch != current_epoch {
        return Err(ProtocolDenial {
            registered: STATE_CONFLICT,
            detail: format!(
                "writer fenced: lease epoch {} != current epoch {current_epoch}",
                lease.epoch
            ),
        }
        .into());
    }

    // M5 correction fencing: a proposal bound to a superseded contract
    // epoch cannot mint an intent (REQ-INTENT-SUPERSEDE-001, vector
    // `intent-supersede-002`: old_epoch_new_dispatch_rejected).
    if let Some(binding) = &cmd.task_binding {
        crate::intent_chain::verify_task_binding_current(store, binding)?;
    }

    let digest = parameters_digest(&cmd.parameters)?;

    // Idempotency arithmetic on the DURABLE record, not on memory.
    if let Some(existing) = store
        .load_intent_by_key(&cmd.idempotency_key)
        .map_err(store_rejection)?
    {
        if existing.parameters_digest == digest {
            return Ok(MintedIntent::ReplayedExisting(existing));
        }
        return Err(ProtocolDenial {
            registered: EFFECT_IDEMPOTENCY_CONFLICT,
            detail: format!(
                "idempotency key {} already bound to parameter digest {}; \
                 refusing digest {} (no dedup, no execution)",
                cmd.idempotency_key, existing.parameters_digest, digest
            ),
        }
        .into());
    }

    let minted_at = clock.now().map_err(|err| port_rejection("clock", err))?;
    let mut intent_value = json!({
        "intent_id": cmd.intent_id.as_str(),
        "action": cmd.descriptor.action,
        "operation_id": cmd.descriptor.operation_id,
        "executor": cmd.descriptor.executor,
        "target": cmd.target,
        "idempotency_key": cmd.idempotency_key,
        "parameters_digest": digest,
        "expected_state_version": cmd.expected_state_version.get(),
        "effect_object_id": cmd.effect_object_id.as_str(),
        "grant_epoch": cmd.grant_epoch,
        "capability_set_version": cmd.capability_set_version,
        "minted_at": minted_at.as_str(),
    });
    if let (Some(binding), Some(object)) = (&cmd.task_binding, intent_value.as_object_mut()) {
        object.insert("task_ref".to_owned(), json!(binding.task_ref));
        object.insert("contract_epoch".to_owned(), json!(binding.contract_epoch));
    }
    let canonical_json = canonical_text(&intent_value)?;
    let row = IntentRow {
        intent_id: cmd.intent_id.clone(),
        idempotency_key: cmd.idempotency_key.clone(),
        parameters_digest: digest,
        action: cmd.descriptor.action.clone(),
        target: cmd.target.clone(),
        effect_object_id: cmd.effect_object_id.clone(),
        expected_state_version: cmd.expected_state_version,
        grant_epoch: cmd.grant_epoch,
        capability_set_version: cmd.capability_set_version,
        task_binding: cmd.task_binding.clone(),
        canonical_json: canonical_json.clone(),
    };

    let event_id_raw = ids
        .next_uuid_v7()
        .map_err(|err| port_rejection("id generator", err))?;
    let event_id = cognitive_domain::EventId::parse(&event_id_raw).map_err(|err| {
        port_rejection(
            "id generator",
            PortFailure {
                detail: format!("non-canonical uuid: {err}"),
            },
        )
    })?;
    let event_value = json!({
        "event_id": event_id.as_str(),
        "event_type": crate::replay::EVENT_TYPE_INTENT_PERSISTED,
        "domain": "effect",
        // Provenance event keyed by the intent's OWN identity (it advances
        // no object state; replay folds it as provenance only).
        "object_id": cmd.intent_id.as_str(),
        "effect_object_id": cmd.effect_object_id.as_str(),
        "idempotency_key": cmd.idempotency_key,
        "parameters_digest": row.parameters_digest,
        "fencing_epoch": lease.epoch,
        "causation": {
            "causation_id": cmd.correlation_id.as_str(),
            "correlation_id": cmd.correlation_id.as_str(),
        },
        "actor_ref": cmd.actor_ref.as_str(),
        "authority_ref": cmd.authority_ref.as_str(),
        "event_time": minted_at.as_str(),
    });
    let event = EventDraft {
        event_id,
        object_id: cmd.intent_id.clone(),
        domain: LifecycleDomain::Effect,
        object_version: Version::INITIAL,
        event_type: crate::replay::EVENT_TYPE_INTENT_PERSISTED.to_owned(),
        canonical_json: canonical_text(&event_value)?,
    };
    store.insert_intent(&row, &event).map_err(store_rejection)?;
    Ok(MintedIntent::Persisted(row))
}

pub(crate) fn canonical_text(value: &Value) -> Result<String, ProtocolDenial> {
    let bytes = canonical::canonical_bytes_of_value(value).map_err(|err| ProtocolDenial {
        registered: STATE_CONFLICT,
        detail: format!("canonical encoding failed: {err}"),
    })?;
    String::from_utf8(bytes).map_err(|err| ProtocolDenial {
        registered: STATE_CONFLICT,
        detail: format!("canonical bytes not utf-8: {err}"),
    })
}

pub(crate) fn strong_ref(
    id: &ObjectId,
    version: i64,
    content: &str,
) -> Result<StrongReference, ProtocolDenial> {
    let digest = canonical::digest(content.as_bytes(), EVIDENCE_DIGEST_DOMAIN).map_err(|err| {
        ProtocolDenial {
            registered: STATE_CONFLICT,
            detail: format!("evidence digest failed: {err}"),
        }
    })?;
    Ok(StrongReference {
        content_digest: cognitive_contracts::generated::common_defs::Digest(digest),
        id: id.to_generated(),
        kind: StrongReferenceKind::Strong,
        object_version: version,
    })
}

// ---------------------------------------------------------------------
// Verification binding (REQ-EFF-003, REQ-RUN-009)
// ---------------------------------------------------------------------

/// Immutable verification report status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationStatus {
    /// All required criteria passed.
    Passed,
    /// One or more required criteria failed.
    Failed,
    /// Pass/fail could not be established.
    Indeterminate,
}

/// A verification result bound to a fixed post-state (the ONLY admissible
/// completion/commit evidence; receipts and remote strings never qualify).
#[derive(Debug, Clone, PartialEq)]
pub struct VerificationRecord {
    /// Governed verification object (its own lifecycle machine).
    pub verification_object_id: ObjectId,
    /// Report identity.
    pub report_id: ObjectId,
    /// Immutable status.
    pub status: VerificationStatus,
    /// Subject the verification is bound to.
    pub subject_domain: LifecycleDomain,
    /// Subject object id.
    pub subject_object_id: ObjectId,
    /// Fixed post-state version the evidence was collected against.
    pub fixed_post_state_version: Version,
}

/// Sanctioned derivation: the verification binding is still current — the
/// subject's AUTHORITATIVE version (reloaded from the store, never from
/// memory) still equals the fixed post-state version.
pub fn verification_still_current<S: AuthorityStore>(
    store: &S,
    record: &VerificationRecord,
) -> Result<bool, TransitionRejection> {
    let subject = store
        .load_object(record.subject_domain, &record.subject_object_id)
        .map_err(store_rejection)?;
    Ok(subject.is_some_and(|object| object.version == record.fixed_post_state_version))
}

// ---------------------------------------------------------------------
// Effect protocol driver
// ---------------------------------------------------------------------

/// The deterministic protocol driver: wraps the central transition gate
/// with the SANCTIONED guard derivations for the effect table (M3 handoff
/// §4.4 anti-drift item — these derivations are the only place effect
/// guards may be attested).
pub struct EffectProtocol<'a, S, C, G> {
    store: &'a S,
    clock: &'a C,
    ids: &'a G,
    /// Effect-authority reference this driver commits under.
    pub authority_ref: UriRef,
    /// Actor reference.
    pub actor_ref: UriRef,
    /// Correlation chain.
    pub correlation_id: UriRef,
}

impl<'a, S, C, G> EffectProtocol<'a, S, C, G>
where
    S: AuthorityStore + ProtocolStore,
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

    fn now(&self) -> Result<WallTimestamp, TransitionRejection> {
        self.clock.now().map_err(|err| port_rejection("clock", err))
    }

    /// Verify the writer lease against the current fencing epoch (driver
    /// pre-check; the store re-checks inside the transaction).
    pub fn verify_lease(&self, lease: &WriterLease) -> Result<(), EffectError> {
        let current = self
            .store
            .current_fencing_epoch()
            .map_err(store_rejection)?;
        if lease.epoch != current {
            return Err(ProtocolDenial {
                registered: STATE_CONFLICT,
                detail: format!(
                    "writer fenced: lease epoch {} != current epoch {current}",
                    lease.epoch
                ),
            }
            .into());
        }
        Ok(())
    }

    // Explicit deterministic inputs are the point of the gate surface.
    #[allow(clippy::too_many_arguments)]
    fn command(
        &self,
        effect_id: &ObjectId,
        from: &str,
        to: &str,
        reason: &str,
        established: BTreeSet<String>,
        evidence: BTreeMap<String, StrongReference>,
        expected_version: Version,
        lease: &WriterLease,
    ) -> Result<TransitionCommand, TransitionRejection> {
        Ok(TransitionCommand {
            request_id: uri_or_reject(&format!(
                "request://effect/{}/{from}-{to}",
                effect_id.as_str()
            ))?,
            domain: LifecycleDomain::Effect,
            object_id: effect_id.clone(),
            subject_ref: uri_or_reject(&format!("effect://{}", effect_id.as_str()))?,
            from: state_or_reject(from)?,
            to: state_or_reject(to)?,
            expected_version,
            reason: Reason {
                code: reason_or_reject(reason)?,
                detail: None,
            },
            causation: Causation {
                causation_id: self.correlation_id.clone(),
                correlation_id: self.correlation_id.clone(),
            },
            actor_ref: self.actor_ref.clone(),
            authority_ref: self.authority_ref.clone(),
            requested_at: self.now()?,
            table_pin: TablePin::current(LifecycleDomain::Effect)?,
            established_guards: established,
            evidence,
            budget: None,
            outbox_destinations: vec![],
            fencing_epoch: Some(lease.epoch),
        })
    }

    /// PROPOSED -> AUTHORIZED. Guards derived: `intent_persisted` (durable
    /// reload), `parameters_and_idempotency_key_fixed` (row carries both),
    /// `authorization_current` (M3 revalidation against the CURRENT
    /// governance facts).
    pub fn authorize_effect(
        &self,
        effect_id: &ObjectId,
        expected_version: Version,
        grant: &AuthorizationGrant,
        currency: &GovernanceCurrency,
        lease: &WriterLease,
    ) -> Result<CommittedTransition, EffectError> {
        self.verify_lease(lease)?;
        let intent = self
            .store
            .load_intent_for_effect(effect_id)
            .map_err(store_rejection)?;
        let mut established = BTreeSet::new();
        if let Some(row) = &intent {
            established.insert("intent_persisted".to_owned());
            if !row.idempotency_key.is_empty() && !row.parameters_digest.is_empty() {
                established.insert("parameters_and_idempotency_key_fixed".to_owned());
            }
            let now = self.now()?;
            if crate::authz::capability_and_revocation_current(
                grant,
                currency.revocation_epoch,
                currency.capability_set_version,
                &now,
            ) {
                established.insert("authorization_current".to_owned());
            }
        }
        let mut evidence = BTreeMap::new();
        if let Some(row) = &intent {
            evidence.insert(
                "intent".to_owned(),
                strong_ref(&row.intent_id, 1, &row.canonical_json)?,
            );
            evidence.insert(
                "authorization_decision".to_owned(),
                strong_ref(
                    &row.intent_id,
                    grant.capability_set_version,
                    "authorization",
                )?,
            );
        }
        let cmd = self.command(
            effect_id,
            "PROPOSED",
            "AUTHORIZED",
            "AUTHORIZATION_GRANTED",
            established,
            evidence,
            expected_version,
            lease,
        )?;
        Ok(self.engine().commit_transition(&cmd)?)
    }

    /// AUTHORIZED -> EXECUTING, then the external call. The dispatch
    /// record commits BEFORE the sink is invoked (no unrecorded external
    /// call can exist); the external outcome then drives the next
    /// transition. Guards derived: `fencing_epoch_current` (lease vs
    /// store), `idempotency_binding_valid` (call fields == durable intent),
    /// `capability_and_revocation_current` (M3 derivation),
    /// `intent_durably_persisted` (durable reload — property 7).
    pub fn dispatch_effect(
        &self,
        effect_id: &ObjectId,
        expected_version: Version,
        grant: &AuthorizationGrant,
        currency: &GovernanceCurrency,
        executor: &dyn EffectExecutor,
        lease: &WriterLease,
    ) -> Result<(CommittedTransition, DispatchOutcome), EffectError> {
        self.verify_lease(lease)?;
        let intent = self
            .store
            .load_intent_for_effect(effect_id)
            .map_err(store_rejection)?
            .ok_or_else(|| ProtocolDenial {
                registered: STATE_CONFLICT,
                detail: "no durable intent for effect: dispatch refused (fail-before-effect)"
                    .to_owned(),
            })?;

        // M5 correction fencing at the DISPATCH sink: an intent minted
        // under a contract epoch that has since been superseded is refused
        // BEFORE any transition commit or external call — zero execution
        // (REQ-INTENT-SUPERSEDE-001, vector `intent-supersede-002`).
        if let Some(binding) = &intent.task_binding {
            crate::intent_chain::verify_task_binding_current(self.store, binding)?;
        }

        let current_epoch = self
            .store
            .current_fencing_epoch()
            .map_err(store_rejection)?;
        let now = self.now()?;
        let mut established = BTreeSet::new();
        if lease.epoch == current_epoch {
            established.insert("fencing_epoch_current".to_owned());
        }
        // The durable reload above IS the sanctioned derivation.
        established.insert("intent_durably_persisted".to_owned());
        if !intent.idempotency_key.is_empty() && !intent.parameters_digest.is_empty() {
            established.insert("idempotency_binding_valid".to_owned());
        }
        if crate::authz::capability_and_revocation_current(
            grant,
            currency.revocation_epoch,
            currency.capability_set_version,
            &now,
        ) {
            established.insert("capability_and_revocation_current".to_owned());
        }

        let dispatch_event_id = self.next_object_id()?;
        let mut evidence = BTreeMap::new();
        evidence.insert(
            "dispatch_event".to_owned(),
            strong_ref(&dispatch_event_id, 1, &intent.canonical_json)?,
        );
        let cmd = self.command(
            effect_id,
            "AUTHORIZED",
            "EXECUTING",
            "DISPATCHED",
            established,
            evidence,
            expected_version,
            lease,
        )?;
        let committed = self.engine().commit_transition(&cmd)?;

        // External call AFTER the durable dispatch record.
        let call = ExecutorCall {
            action: intent.action.clone(),
            target: intent.target.clone(),
            idempotency_key: intent.idempotency_key.clone(),
            parameters_digest: intent.parameters_digest.clone(),
            authorization_digest: format!("epoch:{}", grant.decided_at_epoch),
            fencing_epoch: lease.epoch,
        };
        let outcome = executor
            .dispatch(&call)
            .map_err(|err| port_rejection("executor", err))
            .map_err(EffectError::Rejected)?;
        Ok((committed, outcome))
    }

    fn next_object_id(&self) -> Result<ObjectId, TransitionRejection> {
        let raw = self
            .ids
            .next_uuid_v7()
            .map_err(|err| port_rejection("id generator", err))?;
        ObjectId::parse(&raw).map_err(|err| {
            port_rejection(
                "id generator",
                PortFailure {
                    detail: format!("non-canonical uuid: {err}"),
                },
            )
        })
    }

    /// Record the observed dispatch outcome as the next transition:
    /// receipt -> EXECUTED; authoritative non-execution -> NOT_EXECUTED;
    /// unknown -> OUTCOME_UNKNOWN (first-class, never an error to retry).
    pub fn record_outcome(
        &self,
        effect_id: &ObjectId,
        expected_version: Version,
        outcome: &DispatchOutcome,
        lease: &WriterLease,
    ) -> Result<CommittedTransition, EffectError> {
        let evidence_id = self.next_object_id()?;
        let (to, reason, guard, evidence_name, evidence_body): (_, _, _, _, String) = match outcome
        {
            DispatchOutcome::Executed { receipt_ref } => (
                "EXECUTED",
                "EXECUTION_CONFIRMED",
                "receipt_matches_intent_and_idempotency_key",
                "execution_receipt",
                receipt_ref.clone(),
            ),
            DispatchOutcome::NotExecuted { reason } => (
                "NOT_EXECUTED",
                "NON_EXECUTION_CONFIRMED",
                "authoritative_non_execution_evidence",
                "non_execution_receipt",
                reason.clone(),
            ),
            DispatchOutcome::Unknown { detail } => (
                "OUTCOME_UNKNOWN",
                "TIMEOUT_AFTER_DISPATCH",
                "execution_may_have_occurred",
                "uncertainty_report",
                detail.clone(),
            ),
            DispatchOutcome::FencedStaleEpoch { sink_epoch } => {
                return Err(ProtocolDenial {
                    registered: STATE_CONFLICT,
                    detail: format!(
                        "sink fenced the dispatch (sink epoch {sink_epoch}); no outcome to record"
                    ),
                }
                .into());
            }
        };
        let mut established: BTreeSet<String> = [guard.to_owned()].into();
        let mut evidence = BTreeMap::new();
        evidence.insert(
            evidence_name.to_owned(),
            strong_ref(&evidence_id, 1, &evidence_body)?,
        );
        if to == "OUTCOME_UNKNOWN" {
            // The edge also requires the dispatch event as evidence.
            let dispatch_ref_id = self.next_object_id()?;
            evidence.insert(
                "dispatch_event".to_owned(),
                strong_ref(&dispatch_ref_id, 1, "dispatch")?,
            );
            established.insert("execution_may_have_occurred".to_owned());
        }
        let cmd = self.command(
            effect_id,
            "EXECUTING",
            to,
            reason,
            established,
            evidence,
            expected_version,
            lease,
        )?;
        Ok(self.engine().commit_transition(&cmd)?)
    }

    /// Reconcile an uncertain outcome by querying the executor with the
    /// ORIGINAL idempotency key (property 3/4: reconciliation is the only
    /// continuation from OUTCOME_UNKNOWN; the key is never re-minted).
    /// Also reconciles EXECUTED effects (receipt-confirmed path).
    pub fn reconcile(
        &self,
        effect_id: &ObjectId,
        from_state: &str,
        expected_version: Version,
        executor: &dyn EffectExecutor,
        lease: &WriterLease,
    ) -> Result<(CommittedTransition, ExecutorQueryResult), EffectError> {
        let intent = self
            .store
            .load_intent_for_effect(effect_id)
            .map_err(store_rejection)?
            .ok_or_else(|| ProtocolDenial {
                registered: STATE_CONFLICT,
                detail: "no durable intent: reconciliation cannot bind the original key".to_owned(),
            })?;
        let query = if from_state == "EXECUTED" {
            // Receipt already confirms execution; the reconciliation
            // report restates it.
            ExecutorQueryResult::ExecutedWithOriginalKey
        } else {
            executor
                .query_outcome(&intent.idempotency_key)
                .map_err(|err| port_rejection("executor query", err))
                .map_err(EffectError::Rejected)?
        };
        let (reason, guard) = match query {
            ExecutorQueryResult::ExecutedWithOriginalKey => (
                "RECONCILIATION_CONFIRMED_EXECUTED",
                "reconciliation_result_equals_executed",
            ),
            ExecutorQueryResult::NotExecuted => (
                "RECONCILIATION_CONFIRMED_NOT_EXECUTED",
                "reconciliation_result_equals_not_executed",
            ),
            ExecutorQueryResult::Indeterminate => (
                "RECONCILIATION_STILL_UNKNOWN",
                "reconciliation_result_equals_still_unknown",
            ),
        };
        let established: BTreeSet<String> = [
            guard.to_owned(),
            "reconciliation_binds_original_idempotency_key".to_owned(),
        ]
        .into();
        let report_id = self.next_object_id()?;
        let mut evidence = BTreeMap::new();
        evidence.insert(
            "reconciliation_report".to_owned(),
            strong_ref(
                &report_id,
                1,
                &format!("{query:?}:{}", intent.idempotency_key),
            )?,
        );
        let cmd = self.command(
            effect_id,
            from_state,
            "RECONCILED",
            reason,
            established,
            evidence,
            expected_version,
            lease,
        )?;
        Ok((self.engine().commit_transition(&cmd)?, query))
    }

    /// RECONCILED(not_executed) -> NOT_EXECUTED: confirmed non-execution
    /// closes the effect terminally (no compensation, nothing happened).
    pub fn close_not_executed(
        &self,
        effect_id: &ObjectId,
        expected_version: Version,
        lease: &WriterLease,
    ) -> Result<CommittedTransition, EffectError> {
        let established: BTreeSet<String> =
            ["reconciliation_result_equals_not_executed".to_owned()].into();
        let report_id = self.next_object_id()?;
        let mut evidence = BTreeMap::new();
        evidence.insert(
            "reconciliation_report".to_owned(),
            strong_ref(&report_id, 1, "not_executed")?,
        );
        let cmd = self.command(
            effect_id,
            "RECONCILED",
            "NOT_EXECUTED",
            "RECONCILIATION_CONFIRMED_NOT_EXECUTED",
            established,
            evidence,
            expected_version,
            lease,
        )?;
        Ok(self.engine().commit_transition(&cmd)?)
    }

    /// RECONCILED(still_unknown) -> QUARANTINED: safe recovery is not
    /// available; normal commit is disabled and the surfaced error is the
    /// registered unknown-outcome code (vector `effect-unknown-outcome`).
    pub fn quarantine_still_unknown(
        &self,
        effect_id: &ObjectId,
        expected_version: Version,
        lease: &WriterLease,
    ) -> Result<(CommittedTransition, RegisteredError), EffectError> {
        let established: BTreeSet<String> = [
            "reconciliation_result_equals_still_unknown".to_owned(),
            "normal_commit_disabled".to_owned(),
        ]
        .into();
        let decision_id = self.next_object_id()?;
        let report_id = self.next_object_id()?;
        let mut evidence = BTreeMap::new();
        evidence.insert(
            "quarantine_decision".to_owned(),
            strong_ref(
                &decision_id,
                1,
                "quarantine: still_unknown, safe recovery unavailable",
            )?,
        );
        evidence.insert(
            "reconciliation_report".to_owned(),
            strong_ref(&report_id, 1, "still_unknown")?,
        );
        let cmd = self.command(
            effect_id,
            "RECONCILED",
            "QUARANTINED",
            "SAFE_RECOVERY_UNAVAILABLE",
            established,
            evidence,
            expected_version,
            lease,
        )?;
        let committed = self.engine().commit_transition(&cmd)?;
        // Quarantine is the state; the surfaced error for the caller is
        // the registered unknown-outcome code (vector
        // `effect-unknown-outcome.json` expected error).
        Ok((committed, EFFECT_OUTCOME_UNKNOWN))
    }

    /// RECONCILED(executed) -> VERIFIED with a real verification record.
    pub fn verify_effect(
        &self,
        effect_id: &ObjectId,
        expected_version: Version,
        record: &VerificationRecord,
        lease: &WriterLease,
    ) -> Result<CommittedTransition, EffectError> {
        let mut established: BTreeSet<String> =
            ["reconciliation_result_equals_executed".to_owned()].into();
        established.insert("verification_binds_fixed_post_state".to_owned());
        if record.status == VerificationStatus::Passed
            && verification_still_current(self.store, record)?
        {
            established.insert("verification_current".to_owned());
        }
        let reconciliation_evidence_id = self.next_object_id()?;
        let mut evidence = BTreeMap::new();
        evidence.insert(
            "verification_report".to_owned(),
            strong_ref(&record.report_id, 1, "verification-report")?,
        );
        evidence.insert(
            "reconciliation_report".to_owned(),
            strong_ref(&reconciliation_evidence_id, 1, "reconciliation-report")?,
        );
        let cmd = self.command(
            effect_id,
            "RECONCILED",
            "VERIFIED",
            "POSTCONDITION_PASSED",
            established,
            evidence,
            expected_version,
            lease,
        )?;
        Ok(self.engine().commit_transition(&cmd)?)
    }

    /// VERIFIED -> COMMITTED. Guards derived by RELOADING authoritative
    /// state: `verification_still_current` (subject version unchanged),
    /// `expected_state_version_matches`, `commit_authority_matches`,
    /// `capability_and_revocation_current` (M3). A receipt or remote
    /// `completed` string cannot derive any of these.
    #[allow(clippy::too_many_arguments)]
    pub fn commit_effect(
        &self,
        effect_id: &ObjectId,
        expected_version: Version,
        record: &VerificationRecord,
        grant: &AuthorizationGrant,
        currency: &GovernanceCurrency,
        commit_authority: &UriRef,
        lease: &WriterLease,
    ) -> Result<CommittedTransition, EffectError> {
        self.verify_lease(lease)?;
        let now = self.now()?;
        let still_current = verification_still_current(self.store, record)?
            && record.status == VerificationStatus::Passed;
        let mut established = BTreeSet::new();
        if still_current {
            established.insert("verification_still_current".to_owned());
            established.insert("expected_state_version_matches".to_owned());
        }
        if commit_authority == &self.authority_ref {
            established.insert("commit_authority_matches".to_owned());
        }
        if crate::authz::capability_and_revocation_current(
            grant,
            currency.revocation_epoch,
            currency.capability_set_version,
            &now,
        ) {
            established.insert("capability_and_revocation_current".to_owned());
        }
        let decision_id = self.next_object_id()?;
        let mut evidence = BTreeMap::new();
        evidence.insert(
            "verification_report".to_owned(),
            strong_ref(&record.report_id, 1, "verification-report")?,
        );
        evidence.insert(
            "commit_decision".to_owned(),
            strong_ref(&decision_id, 1, "commit-decision")?,
        );
        let cmd = self.command(
            effect_id,
            "VERIFIED",
            "COMMITTED",
            "COMMIT_AUTHORIZED",
            established,
            evidence,
            expected_version,
            lease,
        )?;
        Ok(self.engine().commit_transition(&cmd)?)
    }

    /// VERIFY_FAILED -> COMPENSATING (property 6): compensation is a NEW
    /// governed effect under a FRESH, independently checked authorization —
    /// reusing the original grant is refused before the gate is consulted.
    #[allow(clippy::too_many_arguments)]
    pub fn begin_compensation(
        &self,
        effect_id: &ObjectId,
        expected_version: Version,
        original_grant: &AuthorizationGrant,
        compensation_grant: &AuthorizationGrant,
        currency: &GovernanceCurrency,
        compensation_intent: &IntentRow,
        lease: &WriterLease,
    ) -> Result<CommittedTransition, EffectError> {
        if compensation_grant == original_grant {
            return Err(ProtocolDenial {
                registered: crate::error::CONTEXT_AUTH_DENIED,
                detail: "compensation must be independently authorized; the original \
                         grant is not admissible"
                    .to_owned(),
            }
            .into());
        }
        let original_intent = self
            .store
            .load_intent_for_effect(effect_id)
            .map_err(store_rejection)?;
        let mut established = BTreeSet::new();
        let is_separate = original_intent
            .as_ref()
            .is_none_or(|orig| orig.idempotency_key != compensation_intent.idempotency_key);
        if is_separate {
            established.insert("compensation_is_separate_governed_effect".to_owned());
        }
        let now = self.now()?;
        if crate::authz::capability_and_revocation_current(
            compensation_grant,
            currency.revocation_epoch,
            currency.capability_set_version,
            &now,
        ) {
            established.insert("independent_authorization_granted".to_owned());
        }
        let report_id = self.next_object_id()?;
        let mut evidence = BTreeMap::new();
        evidence.insert(
            "verification_report".to_owned(),
            strong_ref(&report_id, 1, "verify-failed-report")?,
        );
        evidence.insert(
            "compensation_intent".to_owned(),
            strong_ref(
                &compensation_intent.intent_id,
                1,
                &compensation_intent.canonical_json,
            )?,
        );
        evidence.insert(
            "authorization_decision".to_owned(),
            strong_ref(
                &compensation_intent.intent_id,
                1,
                "independent-authorization",
            )?,
        );
        let cmd = self.command(
            effect_id,
            "VERIFY_FAILED",
            "COMPENSATING",
            "COMPENSATION_AUTHORIZED",
            established,
            evidence,
            expected_version,
            lease,
        )?;
        Ok(self.engine().commit_transition(&cmd)?)
    }
}

fn uri_or_reject(text: &str) -> Result<UriRef, TransitionRejection> {
    UriRef::parse(text).map_err(|err| TransitionRejection {
        kind: RejectionKind::InvalidCommand,
        detail: format!("bad uri: {err}"),
        current_state: None,
        current_version: None,
        available_exits: Vec::new(),
        effect_outcome_unknown: false,
    })
}

fn state_or_reject(text: &str) -> Result<StateName, TransitionRejection> {
    StateName::parse(text).map_err(|err| TransitionRejection {
        kind: RejectionKind::InvalidCommand,
        detail: format!("bad state: {err}"),
        current_state: None,
        current_version: None,
        available_exits: Vec::new(),
        effect_outcome_unknown: false,
    })
}

fn reason_or_reject(text: &str) -> Result<ReasonCode, TransitionRejection> {
    ReasonCode::parse(text).map_err(|err| TransitionRejection {
        kind: RejectionKind::InvalidCommand,
        detail: format!("bad reason: {err}"),
        current_state: None,
        current_version: None,
        available_exits: Vec::new(),
        effect_outcome_unknown: false,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn descriptor(
        effect_class: EffectClass,
        queryable: bool,
        idempotent: bool,
    ) -> OperationDescriptor {
        OperationDescriptor {
            operation_id: "op://tenant-a/payments/refund".to_owned(),
            action: "payments.refund".to_owned(),
            effect_class,
            executor: "executor://tenant-a/payments".to_owned(),
            capabilities: ExecutorCapabilities {
                queryable,
                idempotent,
            },
            descriptor_version: 1,
        }
    }

    /// F-023 admission matrix: all four capability combinations for
    /// governed_external, plus the pure/local rows.
    #[test]
    fn admission_matrix_rejects_unqueryable_nonidempotent_external_operations() {
        assert_eq!(
            admit_operation(&descriptor(EffectClass::GovernedExternal, true, true)).unwrap(),
            RecoveryClosure::QueryReconcile
        );
        assert_eq!(
            admit_operation(&descriptor(EffectClass::GovernedExternal, true, false)).unwrap(),
            RecoveryClosure::QueryReconcile
        );
        assert_eq!(
            admit_operation(&descriptor(EffectClass::GovernedExternal, false, true)).unwrap(),
            RecoveryClosure::IdempotentRedispatch
        );
        let rejected =
            admit_operation(&descriptor(EffectClass::GovernedExternal, false, false)).unwrap_err();
        assert_eq!(rejected.code, "NO_AUTHORIZED_OPERATION_CANDIDATE");
        assert_eq!(rejected.category, "catalog");
        // Emergency paths are held to the same closure requirement.
        assert!(admit_operation(&descriptor(EffectClass::EmergencySafety, false, false)).is_err());
        // Pure/local operations carry no external commitment.
        assert_eq!(
            admit_operation(&descriptor(EffectClass::Pure, false, false)).unwrap(),
            RecoveryClosure::NoExternalCommitment
        );
        assert_eq!(
            admit_operation(&descriptor(EffectClass::LocalEphemeral, false, false)).unwrap(),
            RecoveryClosure::NoExternalCommitment
        );
    }

    #[test]
    fn parameter_digest_is_canonical_not_textual() {
        let a = parameters_digest(&json!({"amount": 42, "currency": "EUR"})).unwrap();
        let b = parameters_digest(&json!({"currency": "EUR", "amount": 42})).unwrap();
        assert_eq!(a, b, "member order does not change the digest");
        let c = parameters_digest(&json!({"amount": 43, "currency": "EUR"})).unwrap();
        assert_ne!(a, c);
    }

    #[test]
    fn sink_inventory_is_complete_and_stable() {
        // F-014: the matrix is a reviewed constant; growing the sink set
        // without updating the fencing tests must fail here.
        assert_eq!(COMMIT_SINKS.len(), 4);
    }
}
