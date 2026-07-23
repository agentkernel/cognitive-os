//! The deterministic management plane: inspect / stop / revoke /
//! reconcile with NO model dependency (REQ-MGMT-FALLBACK-001, vector
//! `management-deterministic-fallback.json`).
//!
//! Every verb is gated by a valid [`PrivilegedManagementSession`]
//! (REQ-MGMT-SESSION-002/003, REQ-MGMT-GATE-001) and executes exclusively
//! through deterministic kernel APIs: the central transition gate
//! (`TransitionEngine`), the M3 revocation arithmetic
//! (`revalidate_grant` currency via [`crate::GovernanceLedger`]) and the
//! M4 recovery sequence (`run_recovery`). Probabilistic components are
//! structurally absent: the only model seam is the experimental shell
//! slot, which no verb reads (tests pin zero calls on a probe).

use crate::audit::{
    AuditedInspectError, ManagementAuditPort, PRIVILEGED_READ_REQUEST_DOMAIN,
    PRIVILEGED_READ_RESULT_DOMAIN, PrivilegedReadDecision, PrivilegedReadOutcome,
    ResultReleaseGate, digest_value,
};
use crate::error::{ManagementDenial, ManagementError};
use crate::governance::GovernanceLedger;
use crate::model::ModelProvider;
use crate::session::{ManagementAction, PrivilegedManagementSession, RiskClass};
use cognitive_contracts::generated::error_registry::RegisteredErrorCode;
use cognitive_contracts::generated::object_reference::{StrongReference, StrongReferenceKind};
use cognitive_domain::{LifecycleDomain, ObjectId, ReasonCode, StateName, UriRef, WallTimestamp};
use cognitive_kernel::effects::{EffectProtocol, WriterLease};
use cognitive_kernel::executor::EffectExecutor;
use cognitive_kernel::ports::{AuthorityStore, Clock, IdGenerator, ProtocolStore, StoredObject};
use cognitive_kernel::recovery::{EffectDisposition, RecoveryStep, run_recovery};
use cognitive_kernel::{Causation, Reason, TablePin, TransitionCommand, TransitionEngine};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// Digest domain for management evidence bodies (same registered domain
/// the effect protocol uses for evidence references).
const EVIDENCE_DIGEST_DOMAIN: &str = "governed-object-content/0.1";

/// Upper bound on events scanned per inspect (single-node admin tool).
const EVENT_SCAN_LIMIT: usize = 1_000_000;

/// Effect states that still carry open work or open uncertainty. Derived
/// from `specs/transitions/effect.transitions.json`: everything that is
/// not one of the five terminal states.
const PENDING_EFFECT_STATES: [&str; 9] = [
    "PROPOSED",
    "AUTHORIZED",
    "EXECUTING",
    "EXECUTED",
    "OUTCOME_UNKNOWN",
    "RECONCILED",
    "VERIFIED",
    "VERIFY_FAILED",
    "COMPENSATING",
];

/// The four deterministic fallback verbs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackVerb {
    /// Read authority state.
    Inspect,
    /// Deterministically terminate an execution.
    Stop,
    /// Advance the revocation epoch (capability revocation).
    Revoke,
    /// Reconcile in-flight effects through the M4 recovery path.
    Reconcile,
}

impl FallbackVerb {
    /// Gate facts of this verb: action name, management domain, risk.
    /// Inspect is read-only (R0); the mutating verbs are R1.
    pub fn action_name(self) -> &'static str {
        match self {
            Self::Inspect => "status.inspect",
            Self::Stop => "execution.stop",
            Self::Revoke => "capability.revoke",
            Self::Reconcile => "effect.reconcile",
        }
    }

    fn domain(self) -> &'static str {
        match self {
            Self::Inspect => "cognitiveos.management.status",
            Self::Stop => "cognitiveos.management.execution",
            Self::Revoke => "cognitiveos.management.capability",
            Self::Reconcile => "cognitiveos.management.effect",
        }
    }

    fn risk(self) -> RiskClass {
        match self {
            Self::Inspect => RiskClass::R0,
            Self::Stop | Self::Revoke | Self::Reconcile => RiskClass::R1,
        }
    }

    fn parse(action: &str) -> Option<Self> {
        match action {
            "status.inspect" => Some(Self::Inspect),
            "execution.stop" => Some(Self::Stop),
            "capability.revoke" => Some(Self::Revoke),
            "effect.reconcile" => Some(Self::Reconcile),
            _ => None,
        }
    }
}

/// Inspect request: one governed object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectRequest {
    /// Lifecycle domain of the object.
    pub domain: LifecycleDomain,
    /// Object identity.
    pub object_id: ObjectId,
}

/// Stop request: one agent-execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopRequest {
    /// Execution to terminate.
    pub execution_id: ObjectId,
}

/// Inspect result (authority facts read from the durable store).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InspectReport {
    /// Lifecycle domain.
    pub domain: String,
    /// Object identity.
    pub object_id: String,
    /// Authoritative current state.
    pub state: String,
    /// Authoritative logical version.
    pub version: i64,
    /// Committed events of this object.
    pub event_count: u64,
    /// Type of the newest committed event, if any.
    pub last_event_type: Option<String>,
    /// Log sequence of the newest committed event, if any.
    pub last_event_sequence: Option<i64>,
    /// Current store fencing epoch.
    pub fencing_epoch: i64,
}

/// Stop result (facts of the committed termination).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StopReport {
    /// Terminated execution.
    pub object_id: String,
    /// State the execution held before the stop.
    pub from_state: String,
    /// Terminal state (always `TERMINATED`).
    pub to_state: String,
    /// Authoritative version after the commit.
    pub after_version: i64,
    /// Log sequence of the termination event.
    pub event_sequence: i64,
    /// Fencing epoch the stop committed under.
    pub fencing_epoch: i64,
}

/// Revoke result (durable governance-currency facts).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RevokeReport {
    /// Epoch before the revocation.
    pub previous_revocation_epoch: i64,
    /// Epoch after the revocation (grants under older epochs are stale).
    pub revocation_epoch: i64,
    /// Capability set version (unchanged by revocation).
    pub capability_set_version: i64,
}

/// One reconciled in-flight effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconciledEffect {
    /// Effect identity.
    pub effect_id: String,
    /// Disposition name (`ready_to_redispatch_original_key`,
    /// `reconciled_executed`, `reconciled_not_executed`, `quarantined`).
    pub disposition: String,
    /// Original idempotency key (redispatch-ready effects only).
    pub idempotency_key: Option<String>,
    /// Registered code surfaced by the disposition (quarantine only).
    pub error_code: Option<String>,
}

/// Reconcile result (the M4 recovery report, management view).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileReport {
    /// Epoch installed by the recovery.
    pub new_epoch: i64,
    /// Epoch of the fenced (pre-recovery) writer generation.
    pub fenced_epoch: i64,
    /// Committed events replayed.
    pub replayed_events: u64,
    /// Projection digest after replay (byte-stable).
    pub projection_digest: String,
    /// Steps in execution order (must equal the registered eight).
    pub step_order: Vec<RecoveryStep>,
    /// Per-effect dispositions.
    pub reconciled: Vec<ReconciledEffect>,
    /// Loops whose checkpoints validated.
    pub resumable_loops: Vec<String>,
}

impl ReconcileReport {
    /// JSON view (steps as registered step names).
    pub fn to_json_value(&self) -> Value {
        json!({
            "new_epoch": self.new_epoch,
            "fenced_epoch": self.fenced_epoch,
            "replayed_events": self.replayed_events,
            "projection_digest": self.projection_digest,
            "step_order": self.step_order.iter().map(|step| step_name(*step)).collect::<Vec<_>>(),
            "reconciled": self.reconciled.iter().map(|entry| json!({
                "effect_id": entry.effect_id,
                "disposition": entry.disposition,
                "idempotency_key": entry.idempotency_key,
                "error_code": entry.error_code,
            })).collect::<Vec<_>>(),
            "resumable_loops": self.resumable_loops,
        })
    }
}

fn step_name(step: RecoveryStep) -> &'static str {
    match step {
        RecoveryStep::Barrier => "barrier",
        RecoveryStep::IdentityAndEpoch => "identity_and_epoch",
        RecoveryStep::FenceOldWriter => "fence_old_writer",
        RecoveryStep::ReplayHistory => "replay_history",
        RecoveryStep::ReconcileEffects => "reconcile_effects",
        RecoveryStep::Reauthorize => "reauthorize",
        RecoveryStep::ReresolveContext => "reresolve_context",
        RecoveryStep::ResumeLoop => "resume_loop",
    }
}

/// The deterministic management plane over one authority store.
pub struct ManagementPlane<'a, S, C, G> {
    store: &'a S,
    clock: &'a C,
    ids: &'a G,
    /// Experimental Intelligent Management Shell slot. NEVER read by the
    /// deterministic verbs; present so tests can prove exactly that.
    experimental_shell_model: Option<&'a dyn ModelProvider>,
}

impl<'a, S, C, G> ManagementPlane<'a, S, C, G>
where
    S: AuthorityStore + ProtocolStore,
    C: Clock,
    G: IdGenerator,
{
    /// Build the plane with NO model anywhere (the fallback construction
    /// path: works with `model_available: false`).
    pub fn deterministic(store: &'a S, clock: &'a C, ids: &'a G) -> Self {
        Self {
            store,
            clock,
            ids,
            experimental_shell_model: None,
        }
    }

    /// Wire the EXPERIMENTAL shell model slot. Deterministic verbs must
    /// never consult it; wiring it changes nothing about their behavior.
    pub fn with_experimental_shell_model(mut self, model: &'a dyn ModelProvider) -> Self {
        self.experimental_shell_model = Some(model);
        self
    }

    /// Whether the experimental slot is wired (informational only).
    pub fn experimental_shell_wired(&self) -> bool {
        self.experimental_shell_model.is_some()
    }

    fn now(&self) -> Result<WallTimestamp, ManagementError> {
        Ok(self.clock.now()?)
    }

    /// Session gate for one verb against one resource. Batch-1 policy
    /// requires no step-up for the fallback verbs; the step-up primitive
    /// is exercised through [`Self::gate_with_step_up`].
    fn gate(
        &self,
        session: &PrivilegedManagementSession,
        verb: FallbackVerb,
        resource: &str,
    ) -> Result<(), ManagementError> {
        let now = self.now()?;
        session.authorize(
            &ManagementAction {
                action: verb.action_name().to_owned(),
                domain: verb.domain().to_owned(),
                resource: resource.to_owned(),
                risk: verb.risk(),
                step_up_required: false,
                step_up_satisfied: false,
            },
            &now,
        )?;
        Ok(())
    }

    /// The gate primitive with explicit step-up facts (policy wiring is a
    /// Management-API-batch concern; the primitive challenges with the
    /// registered code, MGMT-GATE-DENY-003 `step_up_unsatisfied` shape).
    pub fn gate_with_step_up(
        &self,
        session: &PrivilegedManagementSession,
        action: &str,
        step_up_required: bool,
        step_up_satisfied: bool,
    ) -> Result<(), ManagementError> {
        let verb = FallbackVerb::parse(action).ok_or_else(|| {
            ManagementDenial::new(
                RegisteredErrorCode::ManagementScopeMismatch,
                format!("unknown management action `{action}`"),
            )
        })?;
        let now = self.now()?;
        session.authorize(
            &ManagementAction {
                action: verb.action_name().to_owned(),
                domain: verb.domain().to_owned(),
                resource: generic_resource(verb).to_owned(),
                risk: verb.risk(),
                step_up_required,
                step_up_satisfied,
            },
            &now,
        )?;
        Ok(())
    }

    /// Internal inspect primitive: read the authority facts of one governed object
    /// (current state, version, committed events, fencing epoch). Pure
    /// read; a missing object surfaces the same registered denial as an
    /// unauthorized one (M3 protected-read isomorphism).
    fn inspect(
        &self,
        session: &PrivilegedManagementSession,
        request: &InspectRequest,
    ) -> Result<InspectReport, ManagementError> {
        let resource = format!("{}://{}", request.domain.as_str(), request.object_id);
        self.gate(session, FallbackVerb::Inspect, &resource)?;

        let object = self
            .store
            .load_object(request.domain, &request.object_id)?
            .ok_or_else(|| {
                ManagementDenial::new(
                    RegisteredErrorCode::ContextAuthDenied,
                    "management read denied",
                )
            })?;
        let events = self.store.read_events(0, EVENT_SCAN_LIMIT)?;
        let mut event_count: u64 = 0;
        let mut last_event: Option<(&str, i64)> = None;
        for event in &events {
            if event.object_id == request.object_id && event.domain == request.domain {
                event_count += 1;
                last_event = Some((event.event_type.as_str(), event.sequence));
            }
        }
        Ok(InspectReport {
            domain: request.domain.as_str().to_owned(),
            object_id: request.object_id.as_str().to_owned(),
            state: object.state.as_str().to_owned(),
            version: object.version.get(),
            event_count,
            last_event_type: last_event.map(|(event_type, _)| event_type.to_owned()),
            last_event_sequence: last_event.map(|(_, sequence)| sequence),
            fencing_epoch: self.store.current_fencing_epoch()?,
        })
    }

    /// Ordinary Core tracer — inspect authority state and release the report
    /// only after a matching privileged-read audit commit receipt.
    ///
    /// The candidate audit carrier is intentionally internal until final-byte
    /// review/registration. Its deterministic behavior is implemented now so
    /// contract freeze can consume real implementation feedback (ADR-0014).
    pub fn inspect_with_audit<A: ManagementAuditPort>(
        &self,
        session: &PrivilegedManagementSession,
        request: &InspectRequest,
        audit: &A,
    ) -> Result<InspectReport, AuditedInspectError> {
        let observed_at = self.now()?;
        let record_id = self.ids.next_uuid_v7().map_err(ManagementError::from)?;
        let request_digest = digest_value(
            &json!({
                "domain": request.domain.as_str(),
                "object_id": request.object_id.as_str(),
            }),
            PRIVILEGED_READ_REQUEST_DOMAIN,
        )?;

        let result = self.inspect(session, request);
        let (outcome, safe_reason, result_digest) = match &result {
            Ok(report) => (
                PrivilegedReadOutcome::Success,
                None,
                Some(digest_value(
                    &serde_json::to_value(report).map_err(|err| {
                        crate::AuditPortFailure::new(format!("serialize inspect report: {err}"))
                    })?,
                    PRIVILEGED_READ_RESULT_DOMAIN,
                )?),
            ),
            Err(error) => {
                let parts = error.registered_parts();
                let outcome = if matches!(error, ManagementError::Denied(_)) {
                    PrivilegedReadOutcome::Denied
                } else {
                    PrivilegedReadOutcome::Error
                };
                (outcome, Some(parts.code), None)
            }
        };

        let record = PrivilegedReadDecision {
            record_kind: "privileged_read_decision".to_owned(),
            record_id,
            request_digest,
            outcome,
            safe_reason,
            result_digest,
            observed_at,
        };
        let record_digest = record.canonical_digest()?;
        let receipt = audit.commit_privileged_read_decision(&record, &record_digest)?;
        ResultReleaseGate::validate(&record, &record_digest, &receipt)?;

        result.map_err(AuditedInspectError::Management)
    }

    /// Verb 2 — stop: deterministically terminate an agent-execution
    /// through the CENTRAL transition gate (reason `TERMINATION_REQUESTED`).
    /// Guards are derived, never asserted: `writer_fenced` is made true by
    /// advancing the fencing epoch, `pending_effects_closed_or_quarantined`
    /// is derived from the durable effect table (conservative global
    /// check). An illegal target state is rejected by the gate with the
    /// registered code and no state change.
    pub fn stop(
        &self,
        session: &PrivilegedManagementSession,
        request: &StopRequest,
    ) -> Result<StopReport, ManagementError> {
        let resource = format!("agent-execution://{}", request.execution_id);
        self.gate(session, FallbackVerb::Stop, &resource)?;

        let object = self
            .store
            .load_object(LifecycleDomain::AgentExecution, &request.execution_id)?
            .ok_or_else(|| {
                ManagementDenial::new(
                    RegisteredErrorCode::ContextAuthDenied,
                    "management stop denied",
                )
            })?;
        let from_state = object.state.as_str().to_owned();
        let pending = self.pending_effects()?;
        let now = self.now()?;

        // Pending-effect precondition of the registered edges
        // (specs/transitions/agent-execution.transitions.json): while any
        // effect is still open the guard fact does not hold, so the stop
        // is refused deterministically — with the guard named — before a
        // doomed command is submitted or any writer is fenced. Reconcile
        // (or quarantine) first, then stop.
        let pending_guard = match from_state.as_str() {
            "RUNNABLE" | "WAITING" | "SUSPENDED" => Some("pending_effects_closed_or_quarantined"),
            "ADMITTED" => Some("pending_effects_closed_or_transferred"),
            _ => None,
        };
        if let Some(guard) = pending_guard
            && !pending.is_empty()
        {
            return Err(ManagementDenial::new(
                RegisteredErrorCode::StateConflict,
                format!(
                    "guard {guard} cannot be established: {} effect(s) still \
                     pending; reconcile or quarantine them first",
                    pending.len()
                ),
            )
            .into());
        }

        // Guard derivation per registered edge. Guards are only attested
        // when their fact holds; a missing fact leaves the guard out and
        // the central gate rejects (fail closed).
        let mut established: BTreeSet<String> = BTreeSet::new();
        let mut fencing_epoch = self.store.current_fencing_epoch()?;
        match from_state.as_str() {
            "RUNNABLE" | "WAITING" => {
                // The fencing act itself: advancing the epoch makes every
                // older writer lease provably stale at all sinks.
                fencing_epoch = self.store.advance_fencing_epoch()?;
                established.insert("writer_fenced".to_owned());
                established.insert("pending_effects_closed_or_quarantined".to_owned());
            }
            "ADMITTED" => {
                established.insert("pending_effects_closed_or_transferred".to_owned());
            }
            "SUSPENDED" => {
                established.insert("pending_effects_closed_or_quarantined".to_owned());
            }
            "CHECKPOINTED" => {
                // Retention disposition is part of the recorded decision.
                established.insert("checkpoint_retention_decided".to_owned());
            }
            // Any other state has no TERMINATION_REQUESTED edge: submit
            // unmodified and let the central gate reject with the
            // registered code.
            _ => {}
        }

        let decision = json!({
            "checkpoint_retention": "retain",
            "decided_by": session.human_principal,
            "pending_effects": pending.len(),
            "requested_at": now.as_str(),
            "session": session.session_id,
            "target": request.execution_id.as_str(),
            "verb": "stop",
        });
        let decision_id = self.next_object_id()?;
        let mut evidence: BTreeMap<String, StrongReference> = BTreeMap::new();
        evidence.insert(
            "termination_decision".to_owned(),
            self.strong_ref(&decision_id, &decision)?,
        );

        let command = TransitionCommand {
            request_id: self.management_uri(session, "stop/request")?,
            domain: LifecycleDomain::AgentExecution,
            object_id: request.execution_id.clone(),
            subject_ref: parse_uri(&resource)?,
            from: parse_state(&from_state)?,
            to: parse_state("TERMINATED")?,
            expected_version: object.version,
            reason: Reason {
                code: parse_reason("TERMINATION_REQUESTED")?,
                detail: Some("deterministic management stop".to_owned()),
            },
            causation: Causation {
                causation_id: self.management_uri(session, "stop")?,
                correlation_id: self.management_uri(session, "stop")?,
            },
            actor_ref: parse_session_uri(&session.human_principal)?,
            authority_ref: parse_session_uri(&session.session_authority)?,
            requested_at: now,
            table_pin: TablePin::current(LifecycleDomain::AgentExecution)?,
            established_guards: established,
            evidence,
            budget: None,
            outbox_destinations: vec![],
            fencing_epoch: Some(fencing_epoch),
        };
        let engine = TransitionEngine::new(self.store, self.clock, self.ids);
        let committed = engine.commit_transition(&command)?;
        Ok(StopReport {
            object_id: request.execution_id.as_str().to_owned(),
            from_state,
            to_state: "TERMINATED".to_owned(),
            after_version: committed.after_version.get(),
            event_sequence: committed.event_sequence,
            fencing_epoch,
        })
    }

    /// Verb 3 — revoke: advance the revocation epoch in the durable
    /// governance ledger. From this instant every grant decided under the
    /// previous epoch fails the M3 revalidation
    /// (`cognitive_kernel::authz::revalidate_grant`) and cannot authorize
    /// dispatch or commit (F-007 race points).
    pub fn revoke(
        &self,
        session: &PrivilegedManagementSession,
        ledger: &mut GovernanceLedger,
    ) -> Result<RevokeReport, ManagementError> {
        self.gate(
            session,
            FallbackVerb::Revoke,
            "governance://revocation-epoch",
        )?;
        let now = self.now()?;
        let (previous, next) = ledger.advance_revocation_epoch(&now)?;
        Ok(RevokeReport {
            previous_revocation_epoch: previous,
            revocation_epoch: next,
            capability_set_version: ledger.currency().capability_set_version,
        })
    }

    /// Verb 4 — reconcile: close every in-flight effect through the M4
    /// eight-step recovery (`cognitive_kernel::recovery::run_recovery`):
    /// fence the current writer generation, replay committed history,
    /// reconcile EXECUTING/OUTCOME_UNKNOWN with the ORIGINAL idempotency
    /// keys (never re-dispatching), confirm AUTHORIZED intents undispatched
    /// and quarantine what stays unknown. The executor is consulted for
    /// outcome queries only.
    pub fn reconcile(
        &self,
        session: &PrivilegedManagementSession,
        executor: &dyn EffectExecutor,
    ) -> Result<ReconcileReport, ManagementError> {
        self.gate(session, FallbackVerb::Reconcile, "effect://")?;
        let crashed = WriterLease {
            epoch: self.store.current_fencing_epoch()?,
        };
        let protocol = EffectProtocol::new(
            self.store,
            self.clock,
            self.ids,
            parse_session_uri(&session.human_principal)?,
            parse_session_uri(&session.session_authority)?,
            self.management_uri(session, "reconcile")?,
        );
        let report = run_recovery(self.store, crashed, executor, &protocol)?;
        let reconciled = report
            .reconciled
            .iter()
            .map(|(effect_id, disposition)| {
                let (name, key, code) = match disposition {
                    EffectDisposition::ReadyToRedispatchOriginalKey { idempotency_key } => (
                        "ready_to_redispatch_original_key",
                        Some(idempotency_key.clone()),
                        None,
                    ),
                    EffectDisposition::ReconciledExecuted => ("reconciled_executed", None, None),
                    EffectDisposition::ReconciledNotExecuted => {
                        ("reconciled_not_executed", None, None)
                    }
                    EffectDisposition::Quarantined { code } => {
                        ("quarantined", None, Some(code.code.to_owned()))
                    }
                };
                ReconciledEffect {
                    effect_id: effect_id.as_str().to_owned(),
                    disposition: name.to_owned(),
                    idempotency_key: key,
                    error_code: code,
                }
            })
            .collect();
        Ok(ReconcileReport {
            new_epoch: report.new_epoch,
            fenced_epoch: report.fenced_epoch,
            replayed_events: report.replayed_events,
            projection_digest: report.projection_digest,
            step_order: report.step_order,
            reconciled,
            resumable_loops: report
                .resumable_loops
                .iter()
                .map(|id| id.as_str().to_owned())
                .collect(),
        })
    }

    /// Effects still carrying open work or uncertainty (durable read).
    fn pending_effects(&self) -> Result<Vec<StoredObject>, ManagementError> {
        let mut states = Vec::with_capacity(PENDING_EFFECT_STATES.len());
        for name in PENDING_EFFECT_STATES {
            states.push(parse_state(name)?);
        }
        Ok(self
            .store
            .list_objects_in_states(LifecycleDomain::Effect, &states)?)
    }

    fn next_object_id(&self) -> Result<ObjectId, ManagementError> {
        let raw = self.ids.next_uuid_v7()?;
        ObjectId::parse(&raw).map_err(|err| internal(format!("id generator: {err}")))
    }

    fn strong_ref(
        &self,
        id: &ObjectId,
        content: &Value,
    ) -> Result<StrongReference, ManagementError> {
        let bytes = cognitive_contracts::canonical::canonical_bytes_of_value(content)
            .map_err(|err| internal(format!("evidence not canonicalizable: {err}")))?;
        let digest = cognitive_contracts::canonical::digest(&bytes, EVIDENCE_DIGEST_DOMAIN)
            .map_err(|err| internal(format!("evidence digest failed: {err}")))?;
        Ok(StrongReference {
            content_digest: cognitive_contracts::generated::common_defs::Digest(digest),
            id: id.to_generated(),
            kind: StrongReferenceKind::Strong,
            object_version: 1,
        })
    }

    fn management_uri(
        &self,
        session: &PrivilegedManagementSession,
        suffix: &str,
    ) -> Result<UriRef, ManagementError> {
        parse_uri(&format!("management://{}/{suffix}", session.session_id))
    }
}

fn generic_resource(verb: FallbackVerb) -> &'static str {
    match verb {
        FallbackVerb::Inspect => "agent-execution://",
        FallbackVerb::Stop => "agent-execution://",
        FallbackVerb::Revoke => "governance://revocation-epoch",
        FallbackVerb::Reconcile => "effect://",
    }
}

/// Internal invariant failure (never caller-reachable): fail closed with
/// store-unavailable semantics.
fn internal(detail: String) -> ManagementError {
    ManagementError::Ledger(detail)
}

fn parse_uri(text: &str) -> Result<UriRef, ManagementError> {
    UriRef::parse(text).map_err(|err| internal(format!("uri `{text}`: {err}")))
}

/// Session-supplied references: a session whose references do not parse is
/// invalid session material (fail closed with the registered auth denial).
fn parse_session_uri(text: &str) -> Result<UriRef, ManagementError> {
    UriRef::parse(text).map_err(|_| {
        ManagementError::Denied(ManagementDenial::new(
            RegisteredErrorCode::ContextAuthDenied,
            "management session rejected: reference is not a valid URI reference",
        ))
    })
}

fn parse_state(name: &str) -> Result<StateName, ManagementError> {
    StateName::parse(name).map_err(|err| internal(format!("state `{name}`: {err}")))
}

fn parse_reason(name: &str) -> Result<ReasonCode, ManagementError> {
    ReasonCode::parse(name).map_err(|err| internal(format!("reason `{name}`: {err}")))
}
