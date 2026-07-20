//! Deterministic nine-stage Context Resolution pipeline
//! (`docs/standards/context-resolution-and-cache.md` section 2;
//! REQ-CTX-001..012).
//!
//! Stage order (each stage emits a reason-coded record, REQ-CTX-007):
//! admission → governance pre-filter → candidate retrieval → per-object
//! authorization re-validation → ranking/selection → budget fitting →
//! loss declaration → deterministic rendering → view emission.
//!
//! Two orderings are load-bearing and hard-wired by construction:
//!
//! 1. tenant/scope/conversation filtering runs BEFORE anything ranks or
//!    reads content (REQ-CTX-002, vector `context-rank-before-auth.json`);
//! 2. per-object body authorization re-validation runs BEFORE the ranker
//!    and the renderer see any body (REQ-CTX-006 ordering).
//!
//! The ranker slot is the only probabilistic insertion point and it is
//! quarantined: it receives only authorized survivors, and its output is a
//! PROPOSAL that may only reorder or shrink that set — anything else is
//! discarded and recorded. Untrusted content never gains a control role
//! (REQ-CTX-008, REQ-SEC-002).

use crate::authz::{AccessRequest, AuthzSnapshot, ObjectGovernance, authorize};
use crate::context_cache::GovernanceBinding;
use crate::error::{
    CONTEXT_AUTH_DENIED, CONTEXT_BUDGET_EXCEEDED, CONTEXT_INCOMPLETE, CONTEXT_RESOLUTION_STAGNATED,
    RegisteredError,
};
use cognitive_contracts::canonical;
use cognitive_contracts::generated::context_view::{
    LoadedContextItemRole, LoadedContextItemTrustLevel,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// One discovered candidate entering resolution. Role and trust level come
/// from AUTHORITY metadata of the source object, never from its content.
#[derive(Debug, Clone, PartialEq)]
pub struct CandidateObject {
    /// Version-qualified object reference.
    pub object_ref: String,
    /// Object version pinned by this candidate.
    pub object_version: i64,
    /// Content digest of the pinned version.
    pub content_digest: String,
    /// Governance facts of the object.
    pub governance: ObjectGovernance,
    /// Authority-declared context role.
    pub role: LoadedContextItemRole,
    /// Authority-declared trust level.
    pub trust_level: LoadedContextItemTrustLevel,
    /// Body content (opaque to the pipeline).
    pub body: Value,
    /// Declared byte cost (budget dimension `context_bytes`).
    pub cost_bytes: i64,
    /// Declared token cost (budget dimension `input_tokens`).
    pub cost_tokens: i64,
}

/// One required item of the request (REQ-CTX-004 protected set).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredItem {
    /// Version-qualified object reference that must be loaded.
    pub object_ref: String,
}

/// Hard budget for the resolution (absent dimension = unbounded).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ContextBudget {
    /// Maximum total candidate bytes.
    pub context_bytes: Option<i64>,
    /// Maximum total candidate tokens.
    pub input_tokens: Option<i64>,
}

/// Renderer selection (deterministic renderer version is part of the
/// cache identity).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSpec {
    /// Renderer implementation version.
    pub renderer_version: String,
    /// Target profile label (for example `structured/v1`).
    pub target_profile: String,
}

/// One deterministic resolution request.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolutionRequest {
    /// Authorization snapshot (tenant, principal, chain, capabilities,
    /// denies, revocation epoch, decision instant).
    pub snapshot: AuthzSnapshot,
    /// Purpose binding of this resolution.
    pub purpose: String,
    /// Conversation this resolution is bound to (None = non-conversational
    /// activity scope).
    pub conversation_ref: Option<String>,
    /// Required item set (fail closed when it cannot be loaded).
    pub required: Vec<RequiredItem>,
    /// Allow a partial view (`complete:false` + missing list) instead of
    /// failing on unloadable required items (REQ-CTX-004 explicit partial).
    pub allow_partial: bool,
    /// Hard budget.
    pub budget: ContextBudget,
    /// Renderer selection.
    pub render: RenderSpec,
    /// Schema digest pin of the consuming payload (cache-key component).
    pub schema_digest: String,
}

/// Ranker port: the probabilistic slot between authorization and budget
/// fitting. Implementations see ONLY authorized survivors and return a
/// reordering proposal (refs, best first).
pub trait ProposalRanker {
    /// Rank the supplied authorized candidates; returns refs in preference
    /// order. Output is a proposal: refs not in the input are ignored,
    /// omitted refs keep their relative input order after the proposal.
    fn rank(&self, candidates: &[RankerCandidate<'_>]) -> Vec<String>;
}

/// What the ranker is allowed to see (authorized survivors only).
#[derive(Debug, Clone, PartialEq)]
pub struct RankerCandidate<'a> {
    /// Version-qualified object reference.
    pub object_ref: &'a str,
    /// Authority-declared role.
    pub role: LoadedContextItemRole,
    /// Authorized body (safe to rank: authorization already re-validated).
    pub body: &'a Value,
}

/// Identity ranker: keeps arrival order (fully deterministic default).
#[derive(Debug, Default, Clone, Copy)]
pub struct ArrivalOrderRanker;

impl ProposalRanker for ArrivalOrderRanker {
    fn rank(&self, candidates: &[RankerCandidate<'_>]) -> Vec<String> {
        candidates
            .iter()
            .map(|candidate| candidate.object_ref.to_owned())
            .collect()
    }
}

/// The nine stages (REQ-CTX-007 reason codes are per stage).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Admission,
    GovernancePrefilter,
    Retrieval,
    PerObjectAuthorization,
    Ranking,
    BudgetFitting,
    LossDeclaration,
    Rendering,
    ViewEmission,
}

/// Reason-coded record of one completed stage (REQ-CTX-007).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StageRecord {
    /// Stage that ran.
    pub stage: Stage,
    /// Machine reason code for the stage outcome.
    pub reason_code: String,
    /// Deterministic detail (counts, versions — never bodies).
    pub detail: String,
}

/// One rejected candidate with its machine reason (view `rejected` list).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RejectedCandidate {
    /// Candidate reference.
    pub candidate_ref: String,
    /// Machine reason (registered code or stage reason).
    pub reason: String,
}

/// Explicit loss declaration entry (REQ-CTX-005: silent omission is
/// forbidden).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LossEntry {
    /// Reference of the omitted/degraded item.
    pub source: String,
    /// What happened (`omitted_over_budget`, `omitted_unranked`, ...).
    pub transform: String,
    /// Classes omitted.
    pub omitted_classes: Vec<String>,
}

/// One loaded item of the resolved view.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedItem {
    /// Version-qualified reference.
    pub object_ref: String,
    /// Pinned object version.
    pub object_version: i64,
    /// Pinned content digest.
    pub content_digest: String,
    /// Authority-declared role (preserved through rendering, REQ-CTX-008).
    pub role: LoadedContextItemRole,
    /// Authority-declared trust level (preserved).
    pub trust_level: LoadedContextItemTrustLevel,
    /// Byte cost charged against the budget.
    pub cost_bytes: i64,
    /// Token cost charged against the budget.
    pub cost_tokens: i64,
    /// Body (bodies of loaded items only; rejected bodies never travel).
    pub body: Value,
}

/// One rendered segment (per item; prefix-stability unit).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderSegment {
    /// Item this segment renders (`header` for the leading segment).
    pub item_ref: String,
    /// Segment bytes (canonical JSON line).
    pub bytes: Vec<u8>,
    /// Segment digest.
    pub digest: String,
}

/// Deterministic render output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedView {
    /// Full rendered byte stream (concatenated segments).
    pub bytes: Vec<u8>,
    /// Digest of the full stream.
    pub digest: String,
    /// Per-item segments in render order.
    pub segments: Vec<RenderSegment>,
}

/// Implementation-scoped digest domain for rendered views (pending a
/// registered render contract — recorded as an M3 contract gap).
pub const RENDER_DIGEST_DOMAIN: &str = "cognitiveos.impl.context-render/0.1";

/// A successfully resolved, non-authority ContextView projection.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedContextView {
    /// True when every required item loaded within budget.
    pub complete: bool,
    /// Loaded items in deterministic render order.
    pub loaded: Vec<LoadedItem>,
    /// Rejected candidates with machine reasons.
    pub rejected: Vec<RejectedCandidate>,
    /// Required refs that could not be loaded (partial views only).
    pub missing: Vec<String>,
    /// Explicit loss declarations.
    pub loss_declaration: Vec<LossEntry>,
    /// Pinned versions per loaded ref (REQ-CTX-005 provenance).
    pub pinned_versions: BTreeMap<String, i64>,
    /// Stage records (nine entries on success).
    pub stage_records: Vec<StageRecord>,
    /// Exactly the refs the ranker saw (evidence for REQ-CTX-002).
    pub ranker_input_refs: Vec<String>,
    /// Deterministic render output.
    pub render: RenderedView,
    /// Governance binding this view was resolved under (cache identity).
    pub binding: GovernanceBinding,
}

/// A failed resolution: fail closed, no view emitted (REQ-CTX-006: context
/// errors never degrade silently into success).
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("context-resolution-failed: {code} at {stage:?}: {detail}")]
pub struct ResolutionFailure {
    /// Registered code (`CONTEXT_INCOMPLETE`, `CONTEXT_BUDGET_EXCEEDED`,
    /// `CONTEXT_AUTH_DENIED`, `CONTEXT_RESOLUTION_STAGNATED`).
    pub code: &'static str,
    /// Registered category.
    pub category: &'static str,
    /// Registered retryability.
    pub retryable: bool,
    /// Stage that failed.
    pub stage: Stage,
    /// Required refs that could not be satisfied.
    pub missing_items: Vec<String>,
    /// Deterministic detail.
    pub detail: String,
}

fn failure(
    registered: RegisteredError,
    stage: Stage,
    missing_items: Vec<String>,
    detail: String,
) -> ResolutionFailure {
    ResolutionFailure {
        code: registered.code,
        category: registered.category,
        retryable: registered.retryable,
        stage,
        missing_items,
        detail,
    }
}

/// Resolve one ContextRequest deterministically.
pub fn resolve(
    request: &ResolutionRequest,
    candidates: &[CandidateObject],
    ranker: &dyn ProposalRanker,
) -> Result<ResolvedContextView, ResolutionFailure> {
    let mut records: Vec<StageRecord> = Vec::with_capacity(9);
    let mut rejected: Vec<RejectedCandidate> = Vec::new();

    // Stage 1: admission — the request must carry a usable purpose and
    // schema pin; the purpose-to-capability binding itself is enforced per
    // object at stage 4 (decision order step 6).
    if request.purpose.is_empty() || request.schema_digest.is_empty() {
        return Err(failure(
            CONTEXT_AUTH_DENIED,
            Stage::Admission,
            vec![],
            "request misses purpose or schema digest pin".to_owned(),
        ));
    }
    records.push(StageRecord {
        stage: Stage::Admission,
        reason_code: "REQUEST_ADMITTED".to_owned(),
        detail: format!(
            "required={} allow_partial={}",
            request.required.len(),
            request.allow_partial
        ),
    });

    // Stage 2: governance pre-filter — tenant, scope domain, conversation.
    // Runs BEFORE retrieval/ranking ever touches content (REQ-CTX-002).
    let mut prefiltered: Vec<&CandidateObject> = Vec::new();
    for candidate in candidates {
        let same_tenant =
            candidate.governance.tenant_id.as_deref() == Some(request.snapshot.tenant_id.as_str());
        let conversation_ok = match (
            &candidate.governance.conversation_ref,
            &request.conversation_ref,
        ) {
            (None, _) => true, // not conversation-scoped
            (Some(bound), Some(requested)) => bound == requested,
            (Some(_), None) => false,
        };
        if !same_tenant {
            // Cross-tenant reference: fail closed, audit, never rank
            // (REQ-SEC-001). Rejection reason is the registered denial code.
            rejected.push(RejectedCandidate {
                candidate_ref: candidate.object_ref.clone(),
                reason: CONTEXT_AUTH_DENIED.code.to_owned(),
            });
        } else if !conversation_ok {
            // Cross-Conversation contamination is rejected structurally.
            rejected.push(RejectedCandidate {
                candidate_ref: candidate.object_ref.clone(),
                reason: "CONVERSATION_SCOPE_MISMATCH".to_owned(),
            });
        } else {
            prefiltered.push(candidate);
        }
    }
    records.push(StageRecord {
        stage: Stage::GovernancePrefilter,
        reason_code: "GOVERNANCE_PREFILTER_APPLIED".to_owned(),
        detail: format!(
            "candidates={} admitted={} rejected={}",
            candidates.len(),
            prefiltered.len(),
            rejected.len()
        ),
    });

    // Stage 3: candidate retrieval. The M3 kernel consumes an
    // already-discovered candidate set; discovery services slot in here in
    // later milestones and remain candidate producers only.
    records.push(StageRecord {
        stage: Stage::Retrieval,
        reason_code: "CANDIDATE_SET_FIXED".to_owned(),
        detail: format!("retrieved={}", prefiltered.len()),
    });

    // Stage 4: per-object authorization re-validation, BEFORE ranker or
    // renderer see any body (REQ-CTX-002/006).
    let mut authorized: Vec<&CandidateObject> = Vec::new();
    for candidate in prefiltered {
        let access = AccessRequest {
            action: "read_body".to_owned(),
            purpose: request.purpose.clone(),
        };
        match authorize(&request.snapshot, &candidate.governance, &access) {
            Ok(_) => authorized.push(candidate),
            Err(denied) => rejected.push(RejectedCandidate {
                candidate_ref: candidate.object_ref.clone(),
                reason: denied.denial.code.to_owned(),
            }),
        }
    }
    records.push(StageRecord {
        stage: Stage::PerObjectAuthorization,
        reason_code: "PER_OBJECT_AUTHORIZATION_REVALIDATED".to_owned(),
        detail: format!("authorized={}", authorized.len()),
    });

    // Stage 5: ranking. The ranker sees ONLY authorized survivors; its
    // output may only reorder or shrink them (REQ-CTX-011 records the
    // candidate digests and policy version).
    let ranker_input_refs: Vec<String> = authorized
        .iter()
        .map(|candidate| candidate.object_ref.clone())
        .collect();
    let ranker_view: Vec<RankerCandidate<'_>> = authorized
        .iter()
        .map(|candidate| RankerCandidate {
            object_ref: candidate.object_ref.as_str(),
            role: candidate.role,
            body: &candidate.body,
        })
        .collect();
    let proposal = ranker.rank(&ranker_view);
    let authorized_by_ref: BTreeMap<&str, &CandidateObject> = authorized
        .iter()
        .map(|candidate| (candidate.object_ref.as_str(), *candidate))
        .collect();
    let mut ranked: Vec<&CandidateObject> = Vec::new();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut discarded_proposals = 0usize;
    for proposed in &proposal {
        match authorized_by_ref.get(proposed.as_str()) {
            Some(candidate) if !seen.contains(proposed.as_str()) => {
                ranked.push(candidate);
                seen.insert(candidate.object_ref.as_str());
            }
            // A proposal outside the authorized set (or a duplicate) is
            // discarded: rank score cannot restore a denied item.
            _ => discarded_proposals += 1,
        }
    }
    for candidate in &authorized {
        if !seen.contains(candidate.object_ref.as_str()) {
            ranked.push(candidate);
        }
    }
    let candidate_digests: Vec<String> = ranked
        .iter()
        .map(|candidate| candidate.content_digest.clone())
        .collect();
    records.push(StageRecord {
        stage: Stage::Ranking,
        reason_code: "RANKING_PROPOSAL_APPLIED".to_owned(),
        detail: format!(
            "policy=arrival_or_injected ranked={} discarded_proposals={} candidate_digests={}",
            ranked.len(),
            discarded_proposals,
            candidate_digests.join(",")
        ),
    });

    // Stage 6: budget fitting. Required first, in declared order; a
    // required item that is missing or unauthorized fails closed
    // (CONTEXT_INCOMPLETE) and one that cannot fit the hard budget fails
    // closed (CONTEXT_BUDGET_EXCEEDED) — unless partial is explicit.
    let ranked_by_ref: BTreeMap<&str, &CandidateObject> = ranked
        .iter()
        .map(|candidate| (candidate.object_ref.as_str(), *candidate))
        .collect();
    let mut missing: Vec<String> = Vec::new();
    let mut over_budget: Vec<String> = Vec::new();
    let mut loaded_refs: Vec<&CandidateObject> = Vec::new();
    let mut used_bytes = 0i64;
    let mut used_tokens = 0i64;
    let fits = |bytes: i64, tokens: i64, budget: &ContextBudget| {
        budget.context_bytes.is_none_or(|max| bytes <= max)
            && budget.input_tokens.is_none_or(|max| tokens <= max)
    };
    for required in &request.required {
        match ranked_by_ref.get(required.object_ref.as_str()) {
            None => missing.push(required.object_ref.clone()),
            Some(candidate) => {
                let next_bytes = used_bytes + candidate.cost_bytes;
                let next_tokens = used_tokens + candidate.cost_tokens;
                if fits(next_bytes, next_tokens, &request.budget) {
                    loaded_refs.push(candidate);
                    used_bytes = next_bytes;
                    used_tokens = next_tokens;
                } else {
                    over_budget.push(required.object_ref.clone());
                }
            }
        }
    }
    if !over_budget.is_empty() && !request.allow_partial {
        return Err(failure(
            CONTEXT_BUDGET_EXCEEDED,
            Stage::BudgetFitting,
            over_budget,
            "hard budget cannot contain the required set".to_owned(),
        ));
    }
    if !missing.is_empty() && !request.allow_partial {
        return Err(failure(
            CONTEXT_INCOMPLETE,
            Stage::BudgetFitting,
            missing,
            "required context set cannot be closed".to_owned(),
        ));
    }
    // Optional (non-required) ranked candidates fill the remaining budget.
    let required_refs: BTreeSet<&str> = request
        .required
        .iter()
        .map(|item| item.object_ref.as_str())
        .collect();
    let mut losses: Vec<LossEntry> = Vec::new();
    for candidate in &ranked {
        if required_refs.contains(candidate.object_ref.as_str()) {
            continue;
        }
        let next_bytes = used_bytes + candidate.cost_bytes;
        let next_tokens = used_tokens + candidate.cost_tokens;
        if fits(next_bytes, next_tokens, &request.budget) {
            loaded_refs.push(candidate);
            used_bytes = next_bytes;
            used_tokens = next_tokens;
        } else {
            // Optional degradation is EXPLICIT (REQ-CTX-005).
            losses.push(LossEntry {
                source: candidate.object_ref.clone(),
                transform: "omitted_over_budget".to_owned(),
                omitted_classes: vec!["optional_candidate".to_owned()],
            });
        }
    }
    let missing_for_view = {
        let mut all = missing.clone();
        all.extend(over_budget.iter().cloned());
        all
    };
    for item in &missing_for_view {
        losses.push(LossEntry {
            source: item.clone(),
            transform: "required_missing_partial_allowed".to_owned(),
            omitted_classes: vec!["required_item".to_owned()],
        });
    }
    records.push(StageRecord {
        stage: Stage::BudgetFitting,
        reason_code: if missing_for_view.is_empty() {
            "BUDGET_FITTED".to_owned()
        } else {
            "BUDGET_FITTED_PARTIAL".to_owned()
        },
        detail: format!(
            "loaded={} bytes={used_bytes} tokens={used_tokens}",
            loaded_refs.len()
        ),
    });

    // Stage 7: loss declaration (explicit; silent omission is forbidden).
    records.push(StageRecord {
        stage: Stage::LossDeclaration,
        reason_code: "LOSS_DECLARED".to_owned(),
        detail: format!("losses={}", losses.len()),
    });

    // Stage 8: deterministic rendering (REQ-CTX-012): stable partition
    // order, stable within-partition arrival order, per-item segment bytes
    // independent of every other item.
    let loaded: Vec<LoadedItem> = partition_order(&loaded_refs)
        .into_iter()
        .map(|candidate| LoadedItem {
            object_ref: candidate.object_ref.clone(),
            object_version: candidate.object_version,
            content_digest: candidate.content_digest.clone(),
            role: candidate.role,
            trust_level: candidate.trust_level,
            cost_bytes: candidate.cost_bytes,
            cost_tokens: candidate.cost_tokens,
            body: candidate.body.clone(),
        })
        .collect();
    let render = render_view(
        &request.render,
        request.conversation_ref.as_deref(),
        &loaded,
    )
    .map_err(|detail| {
        failure(
            CONTEXT_INCOMPLETE,
            Stage::Rendering,
            vec![],
            format!("deterministic render failed: {detail}"),
        )
    })?;
    records.push(StageRecord {
        stage: Stage::Rendering,
        reason_code: "RENDERED_DETERMINISTIC".to_owned(),
        detail: format!(
            "renderer={} segments={} digest={}",
            request.render.renderer_version,
            render.segments.len(),
            render.digest
        ),
    });

    // Stage 9: view emission with provenance.
    let pinned_versions: BTreeMap<String, i64> = loaded
        .iter()
        .map(|item| (item.object_ref.clone(), item.object_version))
        .collect();
    let binding = GovernanceBinding {
        tenant: request.snapshot.tenant_id.clone(),
        actor_chain_digest: request.snapshot.actor_chain.chain_digest.clone(),
        capability_set_version: request.snapshot.capability_set_version,
        revocation_epoch: request.snapshot.revocation_epoch,
        purpose: request.purpose.clone(),
        schema_digest: request.schema_digest.clone(),
        encoding_profile: cognitive_contracts::ENCODING_PROFILE.to_owned(),
        conversation: request.conversation_ref.clone(),
    };
    let complete = missing_for_view.is_empty();
    records.push(StageRecord {
        stage: Stage::ViewEmission,
        reason_code: if complete {
            "VIEW_EMITTED_COMPLETE".to_owned()
        } else {
            "VIEW_EMITTED_PARTIAL".to_owned()
        },
        detail: format!("loaded={} rejected={}", loaded.len(), rejected.len()),
    });

    Ok(ResolvedContextView {
        complete,
        loaded,
        rejected,
        missing: missing_for_view,
        loss_declaration: losses,
        pinned_versions,
        stage_records: records,
        ranker_input_refs,
        render,
        binding,
    })
}

/// Deterministic partition order for rendering: control first, untrusted
/// input last; within a partition the (already deterministic) fitting
/// order is preserved so adding items appends within its partition
/// instead of reshuffling (prefix stability, REQ-CTX-012).
fn partition_order<'a>(loaded: &[&'a CandidateObject]) -> Vec<&'a CandidateObject> {
    let rank = |role: LoadedContextItemRole| match role {
        LoadedContextItemRole::Control => 0u8,
        LoadedContextItemRole::AuthoritativeState => 1,
        LoadedContextItemRole::Evidence => 2,
        LoadedContextItemRole::Working => 3,
        LoadedContextItemRole::UntrustedInput => 4,
    };
    let mut ordered: Vec<&CandidateObject> = loaded.to_vec();
    // Stable sort: equal partitions keep their fitting order.
    ordered.sort_by_key(|candidate| rank(candidate.role));
    ordered
}

fn role_label(role: LoadedContextItemRole) -> &'static str {
    match role {
        LoadedContextItemRole::Control => "control",
        LoadedContextItemRole::AuthoritativeState => "authoritative_state",
        LoadedContextItemRole::Evidence => "evidence",
        LoadedContextItemRole::Working => "working",
        LoadedContextItemRole::UntrustedInput => "untrusted_input",
    }
}

fn trust_label(trust: LoadedContextItemTrustLevel) -> &'static str {
    match trust {
        LoadedContextItemTrustLevel::Control => "control",
        LoadedContextItemTrustLevel::Authoritative => "authoritative",
        LoadedContextItemTrustLevel::Verified => "verified",
        LoadedContextItemTrustLevel::Untrusted => "untrusted",
    }
}

fn render_view(
    spec: &RenderSpec,
    conversation: Option<&str>,
    loaded: &[LoadedItem],
) -> Result<RenderedView, String> {
    let mut segments: Vec<RenderSegment> = Vec::with_capacity(loaded.len() + 1);
    // Header segment depends ONLY on stable bindings — never on item
    // counts or content, so it stays byte-identical when items are added.
    let header_value = json!({
        "renderer": spec.renderer_version,
        "target_profile": spec.target_profile,
        "conversation": conversation,
    });
    segments.push(segment("header", &header_value)?);
    for item in loaded {
        // Per-item bytes are a function of the item alone (REQ-CTX-012:
        // adding unrelated objects must not change existing segment bytes).
        let value = json!({
            "object_ref": item.object_ref,
            "object_version": item.object_version,
            "content_digest": item.content_digest,
            "role": role_label(item.role),
            "trust_level": trust_label(item.trust_level),
            "body": item.body,
        });
        segments.push(segment(&item.object_ref, &value)?);
    }
    let mut bytes: Vec<u8> = Vec::new();
    for seg in &segments {
        bytes.extend_from_slice(&seg.bytes);
    }
    let digest = canonical::digest(&bytes, RENDER_DIGEST_DOMAIN).map_err(|err| err.to_string())?;
    Ok(RenderedView {
        bytes,
        digest,
        segments,
    })
}

fn segment(item_ref: &str, value: &Value) -> Result<RenderSegment, String> {
    let mut bytes = canonical::canonical_bytes_of_value(value).map_err(|err| err.to_string())?;
    bytes.push(b'\n');
    let digest = canonical::digest(&bytes, RENDER_DIGEST_DOMAIN).map_err(|err| err.to_string())?;
    Ok(RenderSegment {
        item_ref: item_ref.to_owned(),
        bytes,
        digest,
    })
}

/// Control-plane extraction (REQ-CTX-008, REQ-SEC-002): control statements
/// come ONLY from items whose AUTHORITY-declared role is `control` with
/// control/authoritative trust. Content of untrusted items never enters,
/// no matter what it claims to be.
pub fn effective_control_plane(view: &ResolvedContextView) -> Vec<&LoadedItem> {
    view.loaded
        .iter()
        .filter(|item| {
            item.role == LoadedContextItemRole::Control
                && matches!(
                    item.trust_level,
                    LoadedContextItemTrustLevel::Control
                        | LoadedContextItemTrustLevel::Authoritative
                )
        })
        .collect()
}

/// Gate for a proposed control mutation attributed to a view item: only a
/// control-plane item may back a control change; anything else is denied
/// and no capability is created (vector `prompt-injection-isolation.json`).
pub fn admit_control_mutation<'v>(
    view: &'v ResolvedContextView,
    source_ref: &str,
) -> Result<&'v LoadedItem, ResolutionFailure> {
    let control_plane = effective_control_plane(view);
    control_plane
        .into_iter()
        .find(|item| item.object_ref == source_ref)
        .ok_or_else(|| {
            failure(
                CONTEXT_AUTH_DENIED,
                Stage::ViewEmission,
                vec![],
                "control mutation source is not a control-plane item".to_owned(),
            )
        })
}

/// Bounded re-resolution tracker (REQ-DISC-STAGNATION-001): repeated
/// attempts that make no admissible information gain surface
/// `CONTEXT_RESOLUTION_STAGNATED` instead of looping.
#[derive(Debug, Clone, Default)]
pub struct ResolutionSession {
    attempts_without_gain: u32,
    last_missing: Option<BTreeSet<String>>,
}

/// Attempts without gain tolerated before stagnation (the initial attempt
/// plus one retry).
pub const STAGNATION_BOUND: u32 = 2;

impl ResolutionSession {
    /// Record a failed attempt's missing set; errs when the bounded
    /// attempts made no admissible gain (missing set did not shrink).
    pub fn note_failed_attempt(&mut self, missing: &[String]) -> Result<(), ResolutionFailure> {
        let missing: BTreeSet<String> = missing.iter().cloned().collect();
        let gained = match &self.last_missing {
            Some(previous) => missing.len() < previous.len(),
            None => true, // first attempt establishes the baseline
        };
        if gained {
            self.attempts_without_gain = 1;
        } else {
            self.attempts_without_gain += 1;
        }
        self.last_missing = Some(missing.clone());
        if self.attempts_without_gain > STAGNATION_BOUND {
            return Err(failure(
                CONTEXT_RESOLUTION_STAGNATED,
                Stage::Admission,
                missing.into_iter().collect(),
                "bounded resolution attempts made no admissible information gain".to_owned(),
            ));
        }
        Ok(())
    }
}
