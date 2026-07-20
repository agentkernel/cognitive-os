//! The M5 intent chain: UserIntentRecord → IntentInterpretation candidate
//! → deterministic admission → TaskContract, plus user-correction
//! supersession (`docs/standards/task-loop-verification.md` sections 2-3;
//! REQ-INTENT-RECORD-001, REQ-INTENT-ADMISSION-001,
//! REQ-INTENT-SUPERSEDE-001; RFC-0001 REQ-SHELL-CORRECTION-001 kernel
//! side).
//!
//! Determinism boundary: an [`InterpretationCandidate`] is the OUTPUT TYPE
//! of a probabilistic component (LLM/ranker) — nothing more than a
//! persisted proposal. Everything that decides is deterministic code in
//! this module:
//!
//! - the recorded candidate status derives from the material-ambiguity
//!   facts by the registered schema conditional (a candidate carrying a
//!   material ambiguity is `clarification_required`, never silently
//!   promoted);
//! - admission into a TaskContract requires an explicit acceptance by the
//!   record's registered intent authority, digest-bound to the exact
//!   candidate reviewed (`INTENT_CLARIFICATION_REQUIRED` /
//!   `CONTEXT_AUTH_DENIED` otherwise);
//! - contract epochs advance by CAS inside the store transaction; a user
//!   correction supersedes by minting a NEW record + candidate + contract
//!   (epoch N+1) and never rewrites prior rows;
//! - dispatches bound to an older epoch are fenced with the registered
//!   `INTENT_VERSION_SUPERSEDED` code (enforced in [`crate::effects`] at
//!   both mint and dispatch, vector `intent-supersede-002` semantics).

use crate::effects::{
    EVIDENCE_DIGEST_DOMAIN, EffectError, ProtocolDenial, WriterLease, canonical_text,
    port_rejection, store_rejection,
};
use crate::error::{
    CONTEXT_AUTH_DENIED, INTENT_CLARIFICATION_REQUIRED, INTENT_VERSION_SUPERSEDED, STATE_CONFLICT,
};
use crate::ports::{
    Clock, EventDraft, IdGenerator, IntentChainStore, InterpretationRow, PortFailure,
    ProtocolStore, TaskContractRow, UserIntentRecordRow,
};
use cognitive_contracts::canonical;
use cognitive_contracts::generated::common_defs::{
    Budget, Digest, Lineage, Provenance, Retention, ValidTime,
};
use cognitive_contracts::generated::governed_object_header::{
    GovernedObjectHeader, GovernedObjectHeaderScopeDomain, GovernedObjectHeaderSensitivity,
};
use cognitive_contracts::generated::object_reference::{StrongReference, StrongReferenceKind};
use cognitive_contracts::generated::task_contract::{
    ContractCondition, ContractConditionKind, TaskContract, TaskScope,
};
use cognitive_contracts::generated::user_intent_record::UserIntentRecord;
use cognitive_domain::{LifecycleDomain, ObjectId, UriRef, Version, WallTimestamp};
use serde_json::{Value, json};

// ---------------------------------------------------------------------
// Governance header composition
// ---------------------------------------------------------------------

/// Deterministic governance facts the caller supplies for one governed
/// chain object's header (the M5 runtime resolves these from persisted
/// governance objects; tests fix them as data). Everything else in the
/// header is derived here.
#[derive(Debug, Clone, PartialEq)]
pub struct GovernanceSeed {
    /// Owner strong reference.
    pub owner: StrongReference,
    /// Governing authority strong reference.
    pub authority: StrongReference,
    /// ResourceScope strong reference.
    pub resource_scope: StrongReference,
    /// Tenant UUID (None = platform-scoped object).
    pub tenant_id: Option<String>,
    /// Creating principal URI (provenance).
    pub created_by: String,
    /// Sensitivity classification.
    pub sensitivity: GovernedObjectHeaderSensitivity,
    /// Purpose constraints (registered header requires at least one).
    pub purpose_constraints: Vec<String>,
    /// Retention policy name.
    pub retention_policy: String,
}

fn denial(detail: String) -> ProtocolDenial {
    ProtocolDenial {
        registered: STATE_CONFLICT,
        detail,
    }
}

/// Compose a schema-shaped governed-object header. `content_digest` is a
/// placeholder; [`seal_content_digest`] computes the real one under the
/// REQ-GOBJ-REF-004 default projection.
#[allow(clippy::too_many_arguments)]
fn compose_header(
    id: &ObjectId,
    object_type: &str,
    schema_version: &str,
    seed: &GovernanceSeed,
    source_refs: Vec<String>,
    lineage_parents: Vec<String>,
    transform: &str,
    created_at: &WallTimestamp,
) -> Result<GovernedObjectHeader, ProtocolDenial> {
    if seed.purpose_constraints.is_empty() {
        return Err(denial(
            "governed header requires at least one purpose constraint".to_owned(),
        ));
    }
    Ok(GovernedObjectHeader {
        authority_ref: seed.authority.clone(),
        compartments: Vec::new(),
        content_digest: Digest(format!("sha256:{}", "0".repeat(64))),
        created_at: created_at.as_str().to_owned(),
        id: id.to_generated(),
        lineage: Lineage {
            parents: lineage_parents,
            transform: transform.to_owned(),
        },
        object_version: 1,
        owner_ref: seed.owner.clone(),
        policy_refs: Vec::new(),
        provenance: Provenance {
            created_by: seed.created_by.clone(),
            source_refs,
        },
        purpose_constraints: seed.purpose_constraints.clone(),
        resource_scope_ref: seed.resource_scope.clone(),
        retention: Retention {
            expires_at: None,
            legal_hold: false,
            policy: seed.retention_policy.clone(),
        },
        schema_version: schema_version.to_owned(),
        scope_domain: match seed.tenant_id {
            Some(_) => GovernedObjectHeaderScopeDomain::Tenant,
            None => GovernedObjectHeaderScopeDomain::Platform,
        },
        sensitivity: seed.sensitivity,
        tenant_id: seed
            .tenant_id
            .clone()
            .map(cognitive_contracts::generated::object_reference::UuidV7),
        valid_time: ValidTime {
            from: created_at.as_str().to_owned(),
            until: None,
        },
        r#type: object_type.to_owned(),
    })
}

/// REQ-GOBJ-REF-004 default digest projection: `content_digest` is the
/// domain-separated digest of the canonical bytes of the schema-valid
/// object with exactly `/header/content_digest` excluded. Returns the
/// sealed value and the digest.
fn seal_content_digest(mut value: Value) -> Result<(Value, String), ProtocolDenial> {
    let mut projected = value.clone();
    let removed = projected
        .get_mut("header")
        .and_then(Value::as_object_mut)
        .map(|header| header.remove("content_digest").is_some());
    if removed != Some(true) {
        return Err(denial(
            "content digest projection: /header/content_digest absent".to_owned(),
        ));
    }
    let bytes = canonical::canonical_bytes_of_value(&projected)
        .map_err(|err| denial(format!("canonical encoding failed: {err}")))?;
    let digest = canonical::digest(&bytes, EVIDENCE_DIGEST_DOMAIN)
        .map_err(|err| denial(format!("content digest failed: {err}")))?;
    if let Some(header) = value.get_mut("header").and_then(Value::as_object_mut) {
        header.insert("content_digest".to_owned(), json!(digest));
    }
    Ok((value, digest))
}

fn digest_of(value: &Value) -> Result<String, ProtocolDenial> {
    let bytes = canonical::canonical_bytes_of_value(value)
        .map_err(|err| denial(format!("canonical encoding failed: {err}")))?;
    canonical::digest(&bytes, EVIDENCE_DIGEST_DOMAIN)
        .map_err(|err| denial(format!("digest failed: {err}")))
}

fn strong_ref_to(id: &ObjectId, content_digest: &str) -> StrongReference {
    StrongReference {
        content_digest: Digest(content_digest.to_owned()),
        id: id.to_generated(),
        kind: StrongReferenceKind::Strong,
        object_version: 1,
    }
}

fn next_event_id<G: IdGenerator>(
    ids: &G,
) -> Result<cognitive_domain::EventId, crate::error::TransitionRejection> {
    let raw = ids
        .next_uuid_v7()
        .map_err(|err| port_rejection("id generator", err))?;
    cognitive_domain::EventId::parse(&raw).map_err(|err| {
        port_rejection(
            "id generator",
            PortFailure {
                detail: format!("non-canonical uuid: {err}"),
            },
        )
    })
}

fn verify_lease<S: ProtocolStore>(store: &S, lease: &WriterLease) -> Result<(), EffectError> {
    let current = store.current_fencing_epoch().map_err(store_rejection)?;
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

// ---------------------------------------------------------------------
// 1. UserIntentRecord fixing (REQ-INTENT-RECORD-001)
// ---------------------------------------------------------------------

/// One user-intent fixing request. The raw expression is fixed BEFORE any
/// semantic interpretation; nothing here came out of a model.
#[derive(Debug, Clone, PartialEq)]
pub struct UserIntentCommand {
    /// Record identity (UUIDv7).
    pub record_id: ObjectId,
    /// Canonical actor-chain digest of the expressing principal.
    pub actor_chain_digest: String,
    /// Conversation or ResourceScope reference.
    pub conversation_or_scope_ref: UriRef,
    /// Input references (attachments, prior turns).
    pub input_refs: Vec<UriRef>,
    /// The user's raw expression (stored verbatim, never rewritten).
    pub raw_expression: String,
    /// Intent authority whose acceptance decisions bind this record.
    pub intent_authority_ref: UriRef,
    /// Governance header facts.
    pub governance: GovernanceSeed,
    /// Correlation chain.
    pub correlation_id: UriRef,
}

/// Fix one UserIntentRecord durably, atomically with its provenance event
/// (REQ-INTENT-RECORD-001: the record precedes semantic interpretation and
/// is never overwritten — the row is append-only in the store).
pub fn record_user_intent<S, C, G>(
    store: &S,
    clock: &C,
    ids: &G,
    lease: &WriterLease,
    cmd: &UserIntentCommand,
) -> Result<UserIntentRecordRow, EffectError>
where
    S: ProtocolStore + IntentChainStore,
    C: Clock,
    G: IdGenerator,
{
    verify_lease(store, lease)?;
    if cmd.raw_expression.is_empty() {
        return Err(ProtocolDenial {
            registered: STATE_CONFLICT,
            detail: "raw expression must be non-empty".to_owned(),
        }
        .into());
    }
    let recorded_at = clock
        .now()
        .map_err(|err| port_rejection("clock", err))
        .map_err(EffectError::Rejected)?;

    // The REQ-INTENT-RECORD-001 fixing digest: subject, scope, inputs,
    // expression and time, canonicalized.
    let intent_digest = digest_of(&json!({
        "actor_chain_digest": cmd.actor_chain_digest,
        "conversation_or_scope_ref": cmd.conversation_or_scope_ref.as_str(),
        "input_refs": cmd.input_refs.iter().map(UriRef::as_str).collect::<Vec<_>>(),
        "raw_expression": cmd.raw_expression,
        "recorded_at": recorded_at.as_str(),
    }))
    .map_err(EffectError::Denied)?;

    let header = compose_header(
        &cmd.record_id,
        "UserIntentRecord",
        "cognitiveos.user-intent-record/0.1",
        &cmd.governance,
        cmd.input_refs
            .iter()
            .map(|r| r.as_str().to_owned())
            .collect(),
        Vec::new(),
        "user-intent-capture",
        &recorded_at,
    )
    .map_err(EffectError::Denied)?;
    let record = UserIntentRecord {
        actor_chain_digest: Digest(cmd.actor_chain_digest.clone()),
        conversation_or_scope_ref: cmd.conversation_or_scope_ref.as_str().to_owned(),
        header,
        input_refs: cmd
            .input_refs
            .iter()
            .map(|r| r.as_str().to_owned())
            .collect(),
        intent_digest: Digest(intent_digest.clone()),
        raw_expression: cmd.raw_expression.clone(),
        recorded_at: recorded_at.as_str().to_owned(),
    };
    let value = serde_json::to_value(&record)
        .map_err(|err| denial(format!("record serialization: {err}")))
        .map_err(EffectError::Denied)?;
    let (sealed, _content_digest) = seal_content_digest(value).map_err(EffectError::Denied)?;
    let canonical_json = canonical_text(&sealed).map_err(EffectError::Denied)?;

    let event_id = next_event_id(ids).map_err(EffectError::Rejected)?;
    let event_value = json!({
        "event_id": event_id.as_str(),
        "event_type": crate::replay::EVENT_TYPE_USER_INTENT_RECORDED,
        "domain": "task",
        // Provenance event keyed by the record's OWN identity.
        "object_id": cmd.record_id.as_str(),
        "conversation_or_scope_ref": cmd.conversation_or_scope_ref.as_str(),
        "actor_chain_digest": cmd.actor_chain_digest,
        "intent_digest": intent_digest,
        "intent_authority_ref": cmd.intent_authority_ref.as_str(),
        "fencing_epoch": lease.epoch,
        "causation": {
            "causation_id": cmd.correlation_id.as_str(),
            "correlation_id": cmd.correlation_id.as_str(),
        },
        "event_time": recorded_at.as_str(),
    });
    let event = EventDraft {
        event_id,
        object_id: cmd.record_id.clone(),
        domain: LifecycleDomain::Task,
        object_version: Version::INITIAL,
        event_type: crate::replay::EVENT_TYPE_USER_INTENT_RECORDED.to_owned(),
        canonical_json: canonical_text(&event_value).map_err(EffectError::Denied)?,
    };
    let row = UserIntentRecordRow {
        record_id: cmd.record_id.clone(),
        conversation_or_scope_ref: cmd.conversation_or_scope_ref.as_str().to_owned(),
        actor_chain_digest: cmd.actor_chain_digest.clone(),
        raw_expression: cmd.raw_expression.clone(),
        recorded_at,
        intent_authority_ref: cmd.intent_authority_ref.as_str().to_owned(),
        intent_digest,
        canonical_json,
    };
    store
        .insert_user_intent(&row, &event)
        .map_err(store_rejection)?;
    Ok(row)
}

// ---------------------------------------------------------------------
// 2. IntentInterpretation candidate (REQ-INTENT-ADMISSION-001)
// ---------------------------------------------------------------------

/// One declared ambiguity of a candidate (schema `ambiguities` item).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmbiguityFact {
    /// Ambiguity identity.
    pub id: String,
    /// True when the ambiguity affects objective, scope, risk, cost,
    /// egress, acceptance or an irreversible Effect.
    pub material: bool,
    /// The clarification question to put to the intent authority.
    pub question: String,
}

/// A PROBABILISTIC interpretation candidate — the output type of a
/// semantic component. Holding one grants nothing and decides nothing:
/// only [`admit_interpretation`] (deterministic) can turn it into a
/// TaskContract binding, and only with an explicit authority acceptance.
#[derive(Debug, Clone, PartialEq)]
pub struct InterpretationCandidate {
    /// Interpretation identity (UUIDv7).
    pub interpretation_id: ObjectId,
    /// Proposed objectives (at least one).
    pub objectives: Vec<String>,
    /// Proposed constraints.
    pub constraints: Vec<String>,
    /// Proposed forbidden items.
    pub forbidden: Vec<String>,
    /// Assumptions the candidate made.
    pub assumptions: Vec<String>,
    /// Declared ambiguities (material ones force clarification).
    pub ambiguities: Vec<AmbiguityFact>,
    /// Information gaps (URI references).
    pub information_gaps: Vec<UriRef>,
    /// Interpretation this candidate supersedes (correction chains).
    pub supersedes: Option<ObjectId>,
}

/// Recorded status of a candidate, derived DETERMINISTICALLY from its
/// material-ambiguity facts (the registered schema conditional): any
/// material ambiguity ⇒ `clarification_required`. The model proposes the
/// facts; it never picks the status.
pub fn derive_candidate_status(candidate: &InterpretationCandidate) -> &'static str {
    if candidate.ambiguities.iter().any(|a| a.material) {
        "clarification_required"
    } else {
        "candidate"
    }
}

/// Persist one interpretation candidate, atomically with its provenance
/// event. Fails closed if the UserIntentRecord is not durably fixed first
/// (REQ-INTENT-RECORD-001 ordering is structural, not conventional).
// Explicit deterministic inputs are the point of the gate surface.
#[allow(clippy::too_many_arguments)]
pub fn record_interpretation_candidate<S, C, G>(
    store: &S,
    clock: &C,
    ids: &G,
    lease: &WriterLease,
    user_intent_record_id: &ObjectId,
    candidate: &InterpretationCandidate,
    governance: &GovernanceSeed,
    correlation_id: &UriRef,
) -> Result<InterpretationRow, EffectError>
where
    S: ProtocolStore + IntentChainStore,
    C: Clock,
    G: IdGenerator,
{
    verify_lease(store, lease)?;
    let record = store
        .load_user_intent(user_intent_record_id)
        .map_err(store_rejection)?
        .ok_or_else(|| ProtocolDenial {
            registered: STATE_CONFLICT,
            detail: format!(
                "no durable UserIntentRecord {user_intent_record_id}: interpretation refused \
                 (fix the record before semantic interpretation)"
            ),
        })?;
    if candidate.objectives.is_empty() {
        return Err(ProtocolDenial {
            registered: STATE_CONFLICT,
            detail: "candidate must propose at least one objective".to_owned(),
        }
        .into());
    }
    if let Some(superseded) = &candidate.supersedes
        && store
            .load_interpretation(superseded)
            .map_err(store_rejection)?
            .is_none()
    {
        return Err(ProtocolDenial {
            registered: STATE_CONFLICT,
            detail: format!("superseded interpretation {superseded} is not persisted"),
        }
        .into());
    }
    let recorded_at = clock
        .now()
        .map_err(|err| port_rejection("clock", err))
        .map_err(EffectError::Rejected)?;
    let status = derive_candidate_status(candidate);
    let material_count = candidate.ambiguities.iter().filter(|a| a.material).count() as i64;

    let ambiguities_value: Vec<Value> = candidate
        .ambiguities
        .iter()
        .map(|a| json!({"id": a.id, "material": a.material, "question": a.question}))
        .collect();
    let gaps_value: Vec<String> = candidate
        .information_gaps
        .iter()
        .map(|r| r.as_str().to_owned())
        .collect();

    // The acceptance-binding digest: the candidate content the authority
    // reviews, canonicalized (comparison basis, like parameter digests).
    let interpretation_digest = digest_of(&json!({
        "intent_ref": record.record_id.as_str(),
        "objectives": candidate.objectives,
        "constraints": candidate.constraints,
        "forbidden": candidate.forbidden,
        "assumptions": candidate.assumptions,
        "ambiguities": ambiguities_value,
        "information_gaps": gaps_value,
        "supersedes_ref": candidate.supersedes.as_ref().map(|s| s.as_str().to_owned()),
    }))
    .map_err(EffectError::Denied)?;

    let header = compose_header(
        &candidate.interpretation_id,
        "IntentInterpretation",
        "cognitiveos.intent-interpretation/0.1",
        governance,
        vec![format!("state://task/user-intent/{}", record.record_id)],
        vec![format!("state://task/user-intent/{}", record.record_id)],
        "intent-interpretation-candidate",
        &recorded_at,
    )
    .map_err(EffectError::Denied)?;
    let header_value = serde_json::to_value(&header)
        .map_err(|err| denial(format!("header serialization: {err}")))
        .map_err(EffectError::Denied)?;

    // Schema-shaped candidate value (`intent-interpretation.schema.json`;
    // no generated binding exists for this schema yet — composed by hand
    // exactly like the pre-M4 record path, swap registered as a Lane-CTR
    // codegen request in the M5 handoff).
    let mut interpretation_value = json!({
        "header": header_value,
        "intent_ref": {
            "kind": "strong",
            "id": record.record_id.as_str(),
            "object_version": 1,
            "content_digest": record.intent_digest,
        },
        "status": status,
        "objectives": candidate.objectives,
        "constraints": candidate.constraints,
        "forbidden": candidate.forbidden,
        "assumptions": candidate.assumptions,
        "ambiguities": ambiguities_value,
        "information_gaps": gaps_value,
        "interpretation_digest": interpretation_digest,
    });
    if let Some(superseded) = &candidate.supersedes
        && let Some(object) = interpretation_value.as_object_mut()
    {
        object.insert(
            "supersedes_ref".to_owned(),
            json!(format!("state://task/interpretation/{superseded}")),
        );
    }
    let (sealed, _) = seal_content_digest(interpretation_value).map_err(EffectError::Denied)?;
    let canonical_json = canonical_text(&sealed).map_err(EffectError::Denied)?;

    let event_id = next_event_id(ids).map_err(EffectError::Rejected)?;
    let event_value = json!({
        "event_id": event_id.as_str(),
        "event_type": crate::replay::EVENT_TYPE_INTERPRETATION_RECORDED,
        "domain": "task",
        "object_id": candidate.interpretation_id.as_str(),
        "user_intent_record_id": record.record_id.as_str(),
        "recorded_status": status,
        "material_ambiguity_count": material_count,
        "interpretation_digest": interpretation_digest,
        "fencing_epoch": lease.epoch,
        "causation": {
            "causation_id": correlation_id.as_str(),
            "correlation_id": correlation_id.as_str(),
        },
        "event_time": recorded_at.as_str(),
    });
    let event = EventDraft {
        event_id,
        object_id: candidate.interpretation_id.clone(),
        domain: LifecycleDomain::Task,
        object_version: Version::INITIAL,
        event_type: crate::replay::EVENT_TYPE_INTERPRETATION_RECORDED.to_owned(),
        canonical_json: canonical_text(&event_value).map_err(EffectError::Denied)?,
    };
    let row = InterpretationRow {
        interpretation_id: candidate.interpretation_id.clone(),
        user_intent_record_id: record.record_id.clone(),
        recorded_status: status.to_owned(),
        material_ambiguity_count: material_count,
        supersedes_interpretation: candidate.supersedes.clone(),
        interpretation_digest,
        canonical_json,
    };
    store
        .insert_interpretation(&row, &event)
        .map_err(store_rejection)?;
    Ok(row)
}

// ---------------------------------------------------------------------
// 3. Deterministic admission (REQ-INTENT-ADMISSION-001)
// ---------------------------------------------------------------------

/// One explicit acceptance decision by the intent authority. This is
/// deterministic INPUT data (a user action relayed by deterministic code),
/// never a model output.
#[derive(Debug, Clone, PartialEq)]
pub struct AcceptanceCommand {
    /// Candidate the authority reviewed.
    pub interpretation_id: ObjectId,
    /// Who accepts (must equal the record's registered intent authority).
    pub accepted_by: UriRef,
    /// Digest of the candidate content the authority reviewed (must equal
    /// the persisted `interpretation_digest`: acceptance binds bytes, not
    /// a conversational impression).
    pub accepted_digest: String,
}

/// Proof that one candidate passed the deterministic admission gate. Only
/// [`admit_interpretation`] constructs this; TaskContract minting requires
/// it — there is no API path from a raw candidate to a contract.
#[derive(Debug, Clone, PartialEq)]
pub struct AdmittedInterpretation {
    interpretation: InterpretationRow,
    record: UserIntentRecordRow,
    accepted_by: String,
    acceptance_digest: String,
}

impl AdmittedInterpretation {
    /// The admitted interpretation row.
    pub fn interpretation(&self) -> &InterpretationRow {
        &self.interpretation
    }

    /// The user-intent record the interpretation binds.
    pub fn record(&self) -> &UserIntentRecordRow {
        &self.record
    }

    /// The accepting authority.
    pub fn accepted_by(&self) -> &str {
        &self.accepted_by
    }

    /// Canonical digest of the acceptance decision value.
    pub fn acceptance_digest(&self) -> &str {
        &self.acceptance_digest
    }
}

/// The deterministic admission gate. Fails closed:
///
/// - a candidate recorded `clarification_required` (material ambiguity) is
///   refused with `INTENT_CLARIFICATION_REQUIRED` — the gate never picks
///   top-1 over an unresolved material ambiguity;
/// - an acceptor other than the record's registered intent authority is
///   refused with `CONTEXT_AUTH_DENIED` (a model or agent narrating
///   "accepted" cannot pass this comparison);
/// - an acceptance digest differing from the persisted candidate digest is
///   refused with `STATE_CONFLICT` (the authority accepts exactly the
///   bytes it reviewed).
pub fn admit_interpretation<S>(
    store: &S,
    cmd: &AcceptanceCommand,
) -> Result<AdmittedInterpretation, EffectError>
where
    S: IntentChainStore,
{
    let interpretation = store
        .load_interpretation(&cmd.interpretation_id)
        .map_err(store_rejection)?
        .ok_or_else(|| ProtocolDenial {
            registered: STATE_CONFLICT,
            detail: format!(
                "no durable interpretation {}: nothing to admit",
                cmd.interpretation_id
            ),
        })?;
    let record = store
        .load_user_intent(&interpretation.user_intent_record_id)
        .map_err(store_rejection)?
        .ok_or_else(|| ProtocolDenial {
            registered: STATE_CONFLICT,
            detail: "interpretation references a missing UserIntentRecord".to_owned(),
        })?;

    if interpretation.recorded_status == "clarification_required" {
        return Err(ProtocolDenial {
            registered: INTENT_CLARIFICATION_REQUIRED,
            detail: format!(
                "candidate {} declares {} material ambiguit{}; the intent authority must \
                 clarify before any TaskContract binding (top-1 selection is not admission)",
                interpretation.interpretation_id,
                interpretation.material_ambiguity_count,
                if interpretation.material_ambiguity_count == 1 {
                    "y"
                } else {
                    "ies"
                }
            ),
        }
        .into());
    }
    if cmd.accepted_by.as_str() != record.intent_authority_ref {
        return Err(ProtocolDenial {
            registered: CONTEXT_AUTH_DENIED,
            detail: format!(
                "only the registered intent authority {} may accept an interpretation; \
                 {} is not it",
                record.intent_authority_ref, cmd.accepted_by
            ),
        }
        .into());
    }
    if cmd.accepted_digest != interpretation.interpretation_digest {
        return Err(ProtocolDenial {
            registered: STATE_CONFLICT,
            detail: format!(
                "acceptance binds digest {}, persisted candidate is {}: the authority must \
                 accept exactly the candidate it reviewed",
                cmd.accepted_digest, interpretation.interpretation_digest
            ),
        }
        .into());
    }

    let acceptance_digest = digest_of(&json!({
        "interpretation_id": interpretation.interpretation_id.as_str(),
        "interpretation_digest": interpretation.interpretation_digest,
        "user_intent_record_id": record.record_id.as_str(),
        "accepted_by": cmd.accepted_by.as_str(),
    }))
    .map_err(EffectError::Denied)?;
    Ok(AdmittedInterpretation {
        interpretation,
        record,
        accepted_by: cmd.accepted_by.as_str().to_owned(),
        acceptance_digest,
    })
}

// ---------------------------------------------------------------------
// 4. TaskContract minting and supersession (REQ-INTENT-SUPERSEDE-001)
// ---------------------------------------------------------------------

/// One contract condition (mirrors the generated `ContractCondition`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionSpec {
    /// Condition identity.
    pub id: String,
    /// `acceptance`, `stop`, `escalation`, `wait` or `constraint`.
    pub kind: ContractConditionKind,
    /// Human-auditable description.
    pub description: String,
    /// Verifier bound to the condition.
    pub verifier_ref: Option<String>,
}

/// Deterministic TaskContract composition input.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskContractCommand {
    /// Contract identity (UUIDv7).
    pub contract_id: ObjectId,
    /// Task URI the contract governs (the epoch chain key).
    pub task_ref: UriRef,
    /// Contract objective.
    pub objective: String,
    /// In-scope items (at least one).
    pub in_scope: Vec<String>,
    /// Out-of-scope items.
    pub out_of_scope: Vec<String>,
    /// Contract conditions (at least one `acceptance` kind: completion
    /// must be decidable, REQ-RUN-004).
    pub conditions: Vec<ConditionSpec>,
    /// Hard budget dimensions (registered dimension names).
    pub budget: Budget,
    /// Hard iteration ceiling.
    pub max_iterations: i64,
    /// Hard same-action retry ceiling (REQ-RUN-008).
    pub max_retries: i64,
    /// Authority-managed state domains the loop may touch (at least one).
    pub allowed_state_domains: Vec<String>,
    /// Allowed tool URIs.
    pub allowed_tools: Vec<String>,
    /// Governance header facts.
    pub governance: GovernanceSeed,
    /// Correlation chain.
    pub correlation_id: UriRef,
}

/// Mint the TaskContract binding an ADMITTED interpretation, atomically
/// with its provenance event. `expected_current_epoch` is the CAS token:
/// 0 mints the first contract (epoch 1); N supersedes to epoch N+1. The
/// store re-verifies the epoch inside the transaction.
pub fn mint_task_contract<S, C, G>(
    store: &S,
    clock: &C,
    ids: &G,
    lease: &WriterLease,
    admitted: &AdmittedInterpretation,
    cmd: &TaskContractCommand,
    expected_current_epoch: i64,
) -> Result<TaskContractRow, EffectError>
where
    S: ProtocolStore + IntentChainStore,
    C: Clock,
    G: IdGenerator,
{
    verify_lease(store, lease)?;
    if !cmd
        .conditions
        .iter()
        .any(|c| c.kind == ContractConditionKind::Acceptance)
    {
        return Err(ProtocolDenial {
            registered: STATE_CONFLICT,
            detail: "a TaskContract must declare at least one acceptance condition so \
                     completion is decidable (REQ-RUN-004)"
                .to_owned(),
        }
        .into());
    }
    if cmd.max_iterations < 1 || cmd.max_retries < 0 {
        return Err(ProtocolDenial {
            registered: STATE_CONFLICT,
            detail: "max_iterations must be >= 1 and max_retries >= 0".to_owned(),
        }
        .into());
    }
    let current = store
        .current_contract_epoch(cmd.task_ref.as_str())
        .map_err(store_rejection)?;
    if current != expected_current_epoch {
        return Err(ProtocolDenial {
            registered: STATE_CONFLICT,
            detail: format!(
                "contract epoch raced: expected current {expected_current_epoch}, \
                 authoritative is {current}"
            ),
        }
        .into());
    }
    let contract_epoch = expected_current_epoch + 1;
    let minted_at = clock
        .now()
        .map_err(|err| port_rejection("clock", err))
        .map_err(EffectError::Rejected)?;

    let header = compose_header(
        &cmd.contract_id,
        "TaskContract",
        "cognitiveos.task-contract/0.1",
        &cmd.governance,
        vec![format!(
            "state://task/interpretation/{}",
            admitted.interpretation().interpretation_id
        )],
        vec![format!(
            "state://task/user-intent/{}",
            admitted.record().record_id
        )],
        "task-contract-mint",
        &minted_at,
    )
    .map_err(EffectError::Denied)?;

    // Acceptance decision strong reference: the decision value is the
    // admission gate's digest-bound acceptance fact.
    let acceptance_ref = StrongReference {
        content_digest: Digest(admitted.acceptance_digest().to_owned()),
        id: admitted.interpretation().interpretation_id.to_generated(),
        kind: StrongReferenceKind::Strong,
        object_version: 1,
    };
    let contract = TaskContract {
        allowed_state_domains: cmd.allowed_state_domains.clone(),
        allowed_tools: cmd.allowed_tools.clone(),
        budget: cmd.budget.clone(),
        conditions: cmd
            .conditions
            .iter()
            .map(|c| ContractCondition {
                description: c.description.clone(),
                id: c.id.clone(),
                kind: c.kind,
                machine_expression: None,
                verifier_ref: c.verifier_ref.clone(),
            })
            .collect(),
        contract_epoch,
        header,
        human_gates: None,
        intent_acceptance_ref: acceptance_ref,
        intent_interpretation_ref: strong_ref_to(
            &admitted.interpretation().interpretation_id,
            &admitted.interpretation().interpretation_digest,
        ),
        max_iterations: cmd.max_iterations,
        max_retries: cmd.max_retries,
        objective: cmd.objective.clone(),
        scope: TaskScope {
            in_scope: cmd.in_scope.clone(),
            out_of_scope: cmd.out_of_scope.clone(),
        },
        task_ref: cmd.task_ref.as_str().to_owned(),
        user_intent_ref: strong_ref_to(
            &admitted.record().record_id,
            &admitted.record().intent_digest,
        ),
    };
    let value = serde_json::to_value(&contract)
        .map_err(|err| denial(format!("contract serialization: {err}")))
        .map_err(EffectError::Denied)?;
    let (sealed, _) = seal_content_digest(value).map_err(EffectError::Denied)?;
    let canonical_json = canonical_text(&sealed).map_err(EffectError::Denied)?;
    let contract_digest = digest_of(&sealed).map_err(EffectError::Denied)?;

    let event_id = next_event_id(ids).map_err(EffectError::Rejected)?;
    let event_value = json!({
        "event_id": event_id.as_str(),
        "event_type": crate::replay::EVENT_TYPE_TASK_CONTRACT_MINTED,
        "domain": "task",
        "object_id": cmd.contract_id.as_str(),
        "task_ref": cmd.task_ref.as_str(),
        "contract_epoch": contract_epoch,
        "user_intent_record_id": admitted.record().record_id.as_str(),
        "interpretation_id": admitted.interpretation().interpretation_id.as_str(),
        "accepted_by": admitted.accepted_by(),
        "contract_digest": contract_digest,
        "fencing_epoch": lease.epoch,
        "causation": {
            "causation_id": cmd.correlation_id.as_str(),
            "correlation_id": cmd.correlation_id.as_str(),
        },
        "event_time": minted_at.as_str(),
    });
    let event = EventDraft {
        event_id,
        object_id: cmd.contract_id.clone(),
        domain: LifecycleDomain::Task,
        object_version: Version::INITIAL,
        event_type: crate::replay::EVENT_TYPE_TASK_CONTRACT_MINTED.to_owned(),
        canonical_json: canonical_text(&event_value).map_err(EffectError::Denied)?,
    };
    let row = TaskContractRow {
        contract_id: cmd.contract_id.clone(),
        task_ref: cmd.task_ref.as_str().to_owned(),
        contract_epoch,
        user_intent_record_id: admitted.record().record_id.clone(),
        interpretation_id: admitted.interpretation().interpretation_id.clone(),
        accepted_by: admitted.accepted_by().to_owned(),
        contract_digest,
        canonical_json,
    };
    store
        .insert_task_contract(&row, &event, expected_current_epoch)
        .map_err(store_rejection)?;
    Ok(row)
}

/// Deterministic epoch-currency check for one task binding: the fencing
/// primitive behind `INTENT_VERSION_SUPERSEDED` (REQ-INTENT-SUPERSEDE-001,
/// REQ-AKP-INTENT-001 kernel side; vector `intent-supersede-002`).
pub fn verify_task_binding_current<S>(
    store: &S,
    binding: &crate::ports::TaskBinding,
) -> Result<(), ProtocolDenial>
where
    S: IntentChainStore,
{
    let current = store
        .current_contract_epoch(&binding.task_ref)
        .map_err(|err| ProtocolDenial {
            registered: STATE_CONFLICT,
            detail: format!("contract epoch unavailable: {err}"),
        })?;
    if binding.contract_epoch < current {
        return Err(ProtocolDenial {
            registered: INTENT_VERSION_SUPERSEDED,
            detail: format!(
                "proposal bound to contract epoch {} of {}, but the task was superseded to \
                 epoch {current}: old-epoch dispatch is fenced",
                binding.contract_epoch, binding.task_ref
            ),
        });
    }
    if binding.contract_epoch > current {
        return Err(ProtocolDenial {
            registered: STATE_CONFLICT,
            detail: format!(
                "proposal claims contract epoch {} of {}, but the authoritative epoch is \
                 only {current}",
                binding.contract_epoch, binding.task_ref
            ),
        });
    }
    Ok(())
}

/// RFC-0001 REQ-SHELL-CORRECTION-001 classification of one pending piece
/// of old-epoch work, derived deterministically from the effect's
/// authoritative state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingWorkDisposition {
    /// Intent persisted, dispatch never recorded: nothing ran; the epoch
    /// fence alone retires it.
    SafelyCancelled,
    /// Dispatch recorded, outcome open: MUST be reconciled with the
    /// ORIGINAL idempotency key before new-epoch work continues
    /// (`intent-supersede-002` expected `reconcile_before_continue`).
    MustReconcile,
    /// Execution confirmed: the effect must be closed out through
    /// verification/commit or governed compensation — the fact cannot be
    /// cancelled away.
    MustComplete,
    /// Verification failed: compensation (independently authorized) is the
    /// remaining path.
    Compensate,
    /// Quarantined: explicit recovery only.
    Quarantine,
}

/// One classified pending item in a supersede report.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingWork {
    /// Effect object.
    pub effect_object_id: ObjectId,
    /// Original idempotency key (reconciliation binds this key).
    pub idempotency_key: String,
    /// Authoritative effect state at classification time.
    pub effect_state: String,
    /// Deterministic disposition.
    pub disposition: PendingWorkDisposition,
}

/// Report of one completed supersession (evidence for the correction).
#[derive(Debug, Clone, PartialEq)]
pub struct SupersedeReport {
    /// The correction's own UserIntentRecord (the original is untouched).
    pub correction_record: UserIntentRecordRow,
    /// The superseding interpretation.
    pub superseding_interpretation: InterpretationRow,
    /// The new contract (epoch = superseded + 1).
    pub new_contract: TaskContractRow,
    /// The epoch that was superseded.
    pub superseded_epoch: i64,
    /// Old-epoch pending work, classified (empty when nothing in flight).
    pub pending: Vec<PendingWork>,
}

/// One user-correction supersession command. The correction's
/// UserIntentRecord and superseding candidate are fixed FIRST through
/// [`record_user_intent`] / [`record_interpretation_candidate`] (the
/// authority must review the persisted candidate's digest before it can
/// accept anything); this command then performs the deterministic
/// cutover.
#[derive(Debug, Clone, PartialEq)]
pub struct SupersedeCommand {
    /// Acceptance of the ALREADY-PERSISTED superseding candidate by the
    /// intent authority (digest-bound, same gate as first admission).
    pub acceptance: AcceptanceCommand,
    /// The new contract for the SAME task_ref.
    pub contract: TaskContractCommand,
    /// Epoch the caller believes is current (CAS token).
    pub expected_current_epoch: i64,
}

fn classify_effect_state(state: &str) -> Option<PendingWorkDisposition> {
    match state {
        "PROPOSED" | "AUTHORIZED" => Some(PendingWorkDisposition::SafelyCancelled),
        "EXECUTING" | "OUTCOME_UNKNOWN" => Some(PendingWorkDisposition::MustReconcile),
        "EXECUTED" | "RECONCILED" | "VERIFIED" => Some(PendingWorkDisposition::MustComplete),
        "VERIFY_FAILED" | "COMPENSATING" => Some(PendingWorkDisposition::Compensate),
        "QUARANTINED" => Some(PendingWorkDisposition::Quarantine),
        // Terminal closures are not pending work.
        _ => None,
    }
}

/// Execute one user correction (REQ-INTENT-SUPERSEDE-001, RFC-0001
/// REQ-SHELL-CORRECTION-001). Precondition (structural, fail closed): the
/// correction was fixed as a NEW UserIntentRecord and the superseding
/// candidate was persisted against it — the authority reviewed the
/// PERSISTED digest. This function is the deterministic cutover:
///
/// 1. the SAME admission gate as first admission (clarification /
///    authority / digest rules apply to corrections too);
/// 2. the superseding candidate MUST name the interpretation it
///    supersedes (supersession never dangles);
/// 3. the epoch-N+1 contract mints under the store's epoch CAS (a racing
///    correction loses, nothing persists);
/// 4. old-epoch pending work is classified from authoritative effect
///    state (safely-cancelled / must-reconcile / must-complete /
///    compensate / quarantine).
///
/// Prior records, interpretations and contracts are never rewritten —
/// supersession is new rows plus the epoch fence.
pub fn supersede_task_contract<S, C, G>(
    store: &S,
    clock: &C,
    ids: &G,
    lease: &WriterLease,
    cmd: &SupersedeCommand,
) -> Result<SupersedeReport, EffectError>
where
    S: crate::ports::AuthorityStore + ProtocolStore + IntentChainStore,
    C: Clock,
    G: IdGenerator,
{
    if cmd.expected_current_epoch < 1 {
        return Err(ProtocolDenial {
            registered: STATE_CONFLICT,
            detail: "supersession requires an existing contract epoch (mint the first \
                     contract with mint_task_contract)"
                .to_owned(),
        }
        .into());
    }

    // 1. The SAME deterministic admission gate.
    let admitted = admit_interpretation(store, &cmd.acceptance)?;

    // 2. A superseding candidate must chain to what it supersedes.
    if admitted
        .interpretation()
        .supersedes_interpretation
        .is_none()
    {
        return Err(ProtocolDenial {
            registered: STATE_CONFLICT,
            detail: "a superseding candidate must reference the interpretation it supersedes"
                .to_owned(),
        }
        .into());
    }

    // 3. Mint the superseding contract: epoch CAS advances N -> N+1.
    let new_contract = mint_task_contract(
        store,
        clock,
        ids,
        lease,
        &admitted,
        &cmd.contract,
        cmd.expected_current_epoch,
    )?;

    // 4. Classify old-epoch pending work from authoritative state.
    let mut pending = Vec::new();
    for intent in store
        .list_intents_for_task(&new_contract.task_ref)
        .map_err(store_rejection)?
    {
        let bound_epoch = match &intent.task_binding {
            Some(binding) => binding.contract_epoch,
            None => continue,
        };
        if bound_epoch >= new_contract.contract_epoch {
            continue;
        }
        let effect = store
            .load_object(LifecycleDomain::Effect, &intent.effect_object_id)
            .map_err(store_rejection)?;
        let Some(effect) = effect else { continue };
        if let Some(disposition) = classify_effect_state(effect.state.as_str()) {
            pending.push(PendingWork {
                effect_object_id: intent.effect_object_id.clone(),
                idempotency_key: intent.idempotency_key.clone(),
                effect_state: effect.state.as_str().to_owned(),
                disposition,
            });
        }
    }

    Ok(SupersedeReport {
        correction_record: admitted.record().clone(),
        superseding_interpretation: admitted.interpretation().clone(),
        new_contract,
        superseded_epoch: cmd.expected_current_epoch,
        pending,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn candidate(material: bool) -> InterpretationCandidate {
        InterpretationCandidate {
            interpretation_id: ObjectId::parse("00000000-0000-7000-9000-0000000000aa").unwrap(),
            objectives: vec!["roll out v2".to_owned()],
            constraints: vec![],
            forbidden: vec![],
            assumptions: vec![],
            ambiguities: vec![AmbiguityFact {
                id: "amb-1".to_owned(),
                material,
                question: "which environment?".to_owned(),
            }],
            information_gaps: vec![],
            supersedes: None,
        }
    }

    /// The status derivation is the registered schema conditional, not a
    /// model choice: material ⇒ clarification_required.
    #[test]
    fn candidate_status_is_derived_from_material_ambiguity_facts() {
        assert_eq!(
            derive_candidate_status(&candidate(true)),
            "clarification_required"
        );
        assert_eq!(derive_candidate_status(&candidate(false)), "candidate");
        let mut none = candidate(false);
        none.ambiguities.clear();
        assert_eq!(derive_candidate_status(&none), "candidate");
    }

    #[test]
    fn pending_work_classification_covers_the_rfc_categories() {
        assert_eq!(
            classify_effect_state("AUTHORIZED"),
            Some(PendingWorkDisposition::SafelyCancelled)
        );
        assert_eq!(
            classify_effect_state("EXECUTING"),
            Some(PendingWorkDisposition::MustReconcile)
        );
        assert_eq!(
            classify_effect_state("OUTCOME_UNKNOWN"),
            Some(PendingWorkDisposition::MustReconcile)
        );
        assert_eq!(
            classify_effect_state("EXECUTED"),
            Some(PendingWorkDisposition::MustComplete)
        );
        assert_eq!(
            classify_effect_state("VERIFY_FAILED"),
            Some(PendingWorkDisposition::Compensate)
        );
        assert_eq!(
            classify_effect_state("QUARANTINED"),
            Some(PendingWorkDisposition::Quarantine)
        );
        // Terminal closures are not pending.
        assert_eq!(classify_effect_state("COMMITTED"), None);
        assert_eq!(classify_effect_state("NOT_EXECUTED"), None);
        assert_eq!(classify_effect_state("ABORTED"), None);
    }
}
