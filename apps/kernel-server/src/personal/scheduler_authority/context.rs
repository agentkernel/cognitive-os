#![allow(dead_code, unused_imports)]

use cognitive_contracts::{
    canonical,
    generated::governed_object_header::GovernedObjectHeaderSensitivity,
    generated::worker_iteration_authorization::WorkerIterationAuthorization,
    generated::{
        context_request::ContextRequest,
        context_view::{
            ContextView, ContextViewPinnedVersionsValue, ItemCost, LoadedContextItem,
            LoadedContextItemRepresentation, LoadedContextItemRole, LoadedContextItemTrustLevel,
            LossDeclaration, RejectedCandidate as PersistedRejectedCandidate, ResolutionCost,
        },
        object_reference::StrongReferenceKind,
        operation_candidate_proposal::OperationCandidateProposal,
        task_contract::TaskContract,
    },
};
use cognitive_domain::{
    BudgetId, EventId, LifecycleDomain, ObjectId, RecordId, UriRef, Version, WallTimestamp,
};
use cognitive_kernel::authz::ObjectGovernance;
use cognitive_kernel::budget::BudgetCharge;
use cognitive_kernel::candidate_admission::{
    CandidateAdmissionFacts, CandidateAdmissionIdentities, CandidateAdmissionInputs,
    compose_candidate_admission,
};
use cognitive_kernel::context::{
    ArrivalOrderRanker, CandidateObject, ContextBudget, LossEntry, RejectedCandidate, RenderSpec,
    RequiredItem, ResolutionRequest, ResolvedContextView, resolve,
};
use cognitive_kernel::effects::{WriterLease, admit_operation};
use cognitive_kernel::engine::CommittedTransition;
use cognitive_kernel::harness::LoopDriver;
use cognitive_kernel::intent_chain::{
    GovernanceSeed, compose_governed_header, seal_governed_object_content_digest,
    strong_reference_to,
};
use cognitive_kernel::{
    ContextCacheEntry, ContextCacheKey, ContextCacheLookup, ContextSourceDigest, DerivedCacheKind,
    GovernedContextCache,
};
use cognitive_kernel::{
    authz::{AccessRequest, authorize},
    ports::{
        AuthorityStore, BoundContinuationAuthorizationConsumption,
        BoundWorkerAuthorizationConsumption, CandidateAdmissionReceipt, Clock,
        ContextAuthorizationFactStore, ContextCandidateQuery, ContextRequestRow, ContextStore,
        ContextViewRow, ContinuationAuthorityStore, ContinuationAuthorizationConsumptionRow,
        ContinuationAuthorizationRow, HarnessStore, IdGenerator, IntentChainStore, ProtocolStore,
        SchedulerExecutionPolicyRow, SchedulerExecutionPolicyStore, SchedulerLeaseBinding,
        TaskBinding, WorkerAuthorizationStore, WorkerIterationAuthorizationConsumptionRow,
        WorkerIterationAuthorizationRow,
    },
    resolve_persisted_native_descriptor,
};
use cognitive_runtime::{
    SchedulerCeilingDispatch, SchedulerCeilingDispatchError, SchedulerCeilingFacts,
    SchedulerDispatch, SchedulerService, SchedulerServiceError,
};
use cognitive_store::{
    SqliteAuthorityStore, SystemClock, UuidV7Generator,
    scheduler::{SchedulerRepository, SchedulerRepositoryError, SchedulerState, SchedulerWorkKey},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;
use thiserror::Error;

use crate::personal::pi_runtime::{
    PrivatePiCandidateProcess, PrivatePiCandidateRequest, PrivatePiCandidateResponse,
    load_pi_config,
};

use super::*;

/// Reconstruct a Context authorization snapshot immediately before a body
/// access. Re-reading both fact material and revocation currency prevents an
/// earlier metadata discovery from authorizing a later body read with stale
/// policy or capability facts.
pub(crate) fn load_current_context_authorization_snapshot<S>(
    store: &S,
    command: &ContextResolutionCommand,
) -> Result<cognitive_kernel::authz::AuthzSnapshot, SchedulerAuthorityError>
where
    S: ContextAuthorizationFactStore,
{
    let authorization_facts = store
        .load_latest_context_authorization_facts(
            &command.authorization_subject_ref,
            &command.tenant_id,
        )
        .map_err(|error| {
            SchedulerAuthorityError::ContextAuthorizationUnavailable(error.to_string())
        })?
        .ok_or_else(|| {
            SchedulerAuthorityError::ContextAuthorizationUnavailable(
                "no durable authorization facts for Context body read".to_owned(),
            )
        })?;
    let current_revocation_epoch = store
        .load_current_context_revocation_epoch(&command.tenant_id)
        .map_err(|error| {
            SchedulerAuthorityError::ContextAuthorizationUnavailable(error.to_string())
        })?
        .ok_or_else(|| {
            SchedulerAuthorityError::ContextAuthorizationUnavailable(
                "no durable Context revocation epoch".to_owned(),
            )
        })?;
    authorization_facts
        .reconstruct_snapshot(current_revocation_epoch, command.decided_at.clone())
        .map_err(|error| {
            SchedulerAuthorityError::ContextAuthorizationUnavailable(error.to_string())
        })
}

/// Build the daemon-owned System and Task fragments that every task-bound
/// resolution needs. These fragments are derived solely from the immutable
/// ContextRequest and TaskContract; Pi and workspace sources cannot supply or
/// modify them. They deliberately use the current capability scope so the
/// normal resolver revalidates their access alongside every other body.
pub(crate) fn build_required_task_fragments(
    authorization_snapshot: &cognitive_kernel::authz::AuthzSnapshot,
    command: &ContextResolutionCommand,
    request_row: &ContextRequestRow,
    context_request: &ContextRequest,
    contract_row: &cognitive_kernel::ports::TaskContractRow,
    contract: &TaskContract,
) -> Result<Vec<CandidateObject>, SchedulerAuthorityError> {
    let resource_scope = authorization_snapshot
        .capability_links
        .first()
        .map(|capability| capability.resource.clone())
        .filter(|scope| !scope.trim().is_empty())
        .ok_or_else(|| {
            SchedulerAuthorityError::ContextAuthorizationUnavailable(
                "Context Builder requires a current capability resource scope".to_owned(),
            )
        })?;
    let fragment_governance = ObjectGovernance {
        object_ref: request_row.request_id.to_string(),
        tenant_id: Some(command.tenant_id.clone()),
        owner_ref: command.authorization_subject_ref.clone(),
        resource_scope,
        conversation_ref: command.conversation_ref.clone(),
    };
    let system_body = json!({
        "fragment": "system",
        "task_ref": command.task_ref,
        "purpose": context_request.purpose,
        "context_budget": context_request.budget,
        "authority": "daemon_observational_only",
    });
    let task_body = json!({
        "fragment": "task",
        "task_ref": contract.task_ref,
        "contract_epoch": contract.contract_epoch,
        "objective": contract.objective,
        "max_iterations": contract.max_iterations,
        "max_retries": contract.max_retries,
    });
    let candidate_cost = |body: &Value| {
        canonical::canonical_bytes_of_value(body)
            .map(|bytes| (bytes.len() as i64, (bytes.len() as i64 + 3) / 4))
            .map_err(|error| SchedulerAuthorityError::ContextResolution(error.to_string()))
    };
    let (system_bytes, system_tokens) = candidate_cost(&system_body)?;
    let (task_bytes, task_tokens) = candidate_cost(&task_body)?;
    Ok(vec![
        CandidateObject {
            object_ref: request_row.request_id.to_string(),
            object_version: 1,
            content_digest: context_request.header.content_digest.0.clone(),
            governance: fragment_governance.clone(),
            role: LoadedContextItemRole::Control,
            trust_level: LoadedContextItemTrustLevel::Verified,
            representation: LoadedContextItemRepresentation::Structured,
            body: system_body,
            cost_bytes: system_bytes,
            cost_tokens: system_tokens,
        },
        CandidateObject {
            object_ref: contract_row.contract_id.to_string(),
            object_version: contract.header.object_version,
            content_digest: contract.header.content_digest.0.clone(),
            governance: ObjectGovernance {
                object_ref: contract_row.contract_id.to_string(),
                ..fragment_governance
            },
            role: LoadedContextItemRole::AuthoritativeState,
            trust_level: LoadedContextItemTrustLevel::Verified,
            representation: LoadedContextItemRepresentation::Structured,
            body: task_body,
            cost_bytes: task_bytes,
            cost_tokens: task_tokens,
        },
    ])
}

/// Return the daemon-supported source family for an externally admitted
/// source role. Control and authoritative state are daemon-built fragments,
/// never workspace source families.
pub(crate) fn source_family_for_role(role: LoadedContextItemRole) -> Option<&'static str> {
    match role {
        LoadedContextItemRole::Working => Some("working"),
        LoadedContextItemRole::Evidence => Some("evidence"),
        LoadedContextItemRole::UntrustedInput => Some("shell"),
        LoadedContextItemRole::Control | LoadedContextItemRole::AuthoritativeState => None,
    }
}

/// Calculate a millisecond wall-clock instant from the repository's canonical
/// UTC timestamp form without treating a UUIDv7 identity as a time source.
pub(crate) fn timestamp_milliseconds(timestamp: &WallTimestamp) -> Option<i64> {
    let value = timestamp.as_str();
    let year = value.get(0..4)?.parse::<i64>().ok()?;
    let month = value.get(5..7)?.parse::<usize>().ok()?;
    let day = value.get(8..10)?.parse::<i64>().ok()?;
    let hour = value.get(11..13)?.parse::<i64>().ok()?;
    let minute = value.get(14..16)?.parse::<i64>().ok()?;
    let second = value.get(17..19)?.parse::<i64>().ok()?;
    let fraction = value
        .get(19..value.len().checked_sub(1)?)
        .unwrap_or_default()
        .strip_prefix('.')
        .unwrap_or_default();
    let milliseconds = format!("{fraction:0<3}").get(0..3)?.parse::<i64>().ok()?;
    let is_leap_year = |candidate_year: i64| {
        candidate_year % 4 == 0 && (candidate_year % 100 != 0 || candidate_year % 400 == 0)
    };
    let days_before_month = [0i64, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    if !(1..=12).contains(&month) || day < 1 || hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    let leap_days = ((year - 1) / 4) - ((year - 1) / 100) + ((year - 1) / 400);
    let leap_day_after_february = if month > 2 && is_leap_year(year) {
        1
    } else {
        0
    };
    let days_since_epoch = 365 * (year - 1970)
        + (leap_days - 477)
        + days_before_month[month - 1]
        + leap_day_after_february
        + day
        - 1;
    Some((((days_since_epoch * 24 + hour) * 60 + minute) * 60 + second) * 1_000 + milliseconds)
}

/// Determine whether immutable source metadata meets the request's
/// role-specific freshness rule before its body becomes eligible to load.
pub(crate) fn source_freshness_reason(
    role: LoadedContextItemRole,
    created_at: &WallTimestamp,
    decided_at: &WallTimestamp,
    context_request: &ContextRequest,
) -> Option<&'static str> {
    let maximum_age_milliseconds = match role {
        LoadedContextItemRole::Working | LoadedContextItemRole::UntrustedInput => {
            context_request.freshness.world_max_age_ms
        }
        LoadedContextItemRole::Evidence => context_request
            .freshness
            .knowledge_max_age_s
            .and_then(|seconds| seconds.checked_mul(1_000)),
        LoadedContextItemRole::Control | LoadedContextItemRole::AuthoritativeState => None,
    }?;
    let created_milliseconds = timestamp_milliseconds(created_at)?;
    let decided_milliseconds = timestamp_milliseconds(decided_at)?;
    let age_milliseconds = decided_milliseconds.checked_sub(created_milliseconds)?;
    (age_milliseconds < 0 || age_milliseconds > maximum_age_milliseconds)
        .then_some("CONTEXT_SOURCE_STALE")
}

/// Resolve daemon-admitted Context for a TaskContract before Pi can make a
/// non-authoritative candidate proposal. Metadata is queried first, each
/// source is authorized with current revocation currency, and only authorized
/// sources have their durable body materialized. This function neither calls
/// Pi nor persists a candidate, Intent, Effect, WIA, budget debit, progress,
/// evidence, verification, acceptance, or Task completion.
pub(crate) fn resolve_authorized_task_context<S>(
    store: &S,
    command: &ContextResolutionCommand,
) -> Result<ResolvedContextView, SchedulerAuthorityError>
where
    S: AuthorityStore
        + ContextStore
        + ContextAuthorizationFactStore
        + IntentChainStore
        + ProtocolStore,
{
    resolve_authorized_task_context_after_metadata(store, command, || Ok(()))
}

/// Resolve Context with a daemon-owned digest-only cache coordinator.
///
/// This intentionally does not permit the cache to return render bytes or
/// source bodies. The resolver re-runs metadata discovery, freshness checks,
/// current authorization, and body/digest matching first. Only then can an
/// exact key confirm that stable-prefix/delta metadata is reusable.
pub(crate) fn resolve_authorized_task_context_with_cache<S>(
    store: &S,
    command: &ContextResolutionCommand,
    context_cache: &mut GovernedContextCache,
) -> Result<GovernedContextResolution, SchedulerAuthorityError>
where
    S: AuthorityStore
        + ContextStore
        + ContextAuthorizationFactStore
        + IntentChainStore
        + ProtocolStore,
{
    let initial_contract_epoch = store
        .current_contract_epoch(&command.task_ref)
        .map_err(|error| SchedulerAuthorityError::ContextRequestUnavailable(error.to_string()))?;
    let initial_contract = store
        .load_task_contract(&command.task_ref, initial_contract_epoch)
        .map_err(|error| SchedulerAuthorityError::ContextRequestUnavailable(error.to_string()))?
        .ok_or_else(|| SchedulerAuthorityError::MissingContract(command.task_ref.clone()))?;
    let initial_request = store
        .load_context_request(&command.request_id)
        .map_err(|error| SchedulerAuthorityError::ContextRequestUnavailable(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::ContextRequestUnavailable(command.request_id.to_string())
        })?;
    let resolved_view = resolve_authorized_task_context(store, command)?;
    let current_contract_epoch = store
        .current_contract_epoch(&command.task_ref)
        .map_err(|error| SchedulerAuthorityError::ContextRequestUnavailable(error.to_string()))?;
    let current_contract = store
        .load_task_contract(&command.task_ref, current_contract_epoch)
        .map_err(|error| SchedulerAuthorityError::ContextRequestUnavailable(error.to_string()))?
        .ok_or_else(|| SchedulerAuthorityError::MissingContract(command.task_ref.clone()))?;
    let current_request = store
        .load_context_request(&command.request_id)
        .map_err(|error| SchedulerAuthorityError::ContextRequestUnavailable(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::ContextRequestUnavailable(command.request_id.to_string())
        })?;
    if initial_contract.contract_epoch != current_contract.contract_epoch
        || initial_contract.contract_digest != current_contract.contract_digest
        || initial_request.request_digest != current_request.request_digest
    {
        return Err(SchedulerAuthorityError::ContextRequestUnavailable(
            "TaskContract or ContextRequest changed during Context cache resolution".to_owned(),
        ));
    }

    let cache_key = context_cache_key(&resolved_view, command, &current_request, &current_contract);
    let cache_entry = context_cache_entry(&resolved_view);
    let cache_telemetry = match context_cache.lookup_current(&cache_key) {
        ContextCacheLookup::Hit(cached_entry) => {
            if cached_entry != cache_entry {
                return Err(SchedulerAuthorityError::ContextResolution(
                    "governed Context cache metadata differs from the freshly authorized view"
                        .to_owned(),
                ));
            }
            ContextCacheTelemetry {
                cache_hit: true,
                stable_prefix_segment_count: cached_entry.stable_prefix_segment_digests.len(),
                delta_segment_count: cached_entry.delta_segment_digests.len(),
            }
        }
        ContextCacheLookup::MissResolveFresh => {
            context_cache.insert(cache_key, cache_entry.clone());
            ContextCacheTelemetry {
                cache_hit: false,
                stable_prefix_segment_count: cache_entry.stable_prefix_segment_digests.len(),
                delta_segment_count: cache_entry.delta_segment_digests.len(),
            }
        }
    };

    Ok(GovernedContextResolution {
        resolved_view,
        cache_telemetry,
    })
}

pub(crate) fn context_cache_key(
    resolved_view: &ResolvedContextView,
    command: &ContextResolutionCommand,
    request_row: &ContextRequestRow,
    contract_row: &cognitive_kernel::ports::TaskContractRow,
) -> ContextCacheKey {
    let mut ordered_source_digests = resolved_view
        .loaded
        .iter()
        .map(|item| ContextSourceDigest {
            source_ref: item.object_ref.clone(),
            content_digest: item.content_digest.clone(),
        })
        .collect::<Vec<_>>();
    ordered_source_digests.sort();

    ContextCacheKey {
        governance: resolved_view.binding.clone(),
        context_request_id: request_row.request_id.to_string(),
        context_request_digest: request_row.request_digest.clone(),
        task_ref: command.task_ref.clone(),
        task_contract_epoch: contract_row.contract_epoch,
        task_contract_digest: contract_row.contract_digest.clone(),
        ordered_source_digests,
        renderer_version: "personal-context-render/1".to_owned(),
        // Context is built before an untrusted Pi candidate exists. Tool-bound
        // delta caching may only be added at a later daemon-validated boundary.
        validated_tool_descriptor_digest: None,
    }
}

pub(crate) fn context_cache_entry(resolved_view: &ResolvedContextView) -> ContextCacheEntry {
    let stable_prefix_segment_count = resolved_view
        .loaded
        .iter()
        .take_while(|item| {
            matches!(
                item.role,
                LoadedContextItemRole::Control | LoadedContextItemRole::AuthoritativeState
            )
        })
        .count()
        + 1; // The renderer header depends only on stable bindings.
    let segment_digests = resolved_view
        .render
        .segments
        .iter()
        .map(|segment| segment.digest.clone())
        .collect::<Vec<_>>();
    ContextCacheEntry {
        render_digest: resolved_view.render.digest.clone(),
        stable_prefix_segment_digests: segment_digests
            .iter()
            .take(stable_prefix_segment_count)
            .cloned()
            .collect(),
        delta_segment_digests: segment_digests
            .into_iter()
            .skip(stable_prefix_segment_count)
            .collect(),
        derived: vec![DerivedCacheKind::KvCache, DerivedCacheKind::PromptCache],
    }
}

/// Resolve Context after the metadata-only discovery stage. Production calls
/// this through [`resolve_authorized_task_context`] with a no-op observer; the
/// private observer makes the discovery-to-body authorization boundary
/// deterministically testable without exposing a runtime control surface.
pub(crate) fn resolve_authorized_task_context_after_metadata<S, F>(
    store: &S,
    command: &ContextResolutionCommand,
    after_metadata: F,
) -> Result<ResolvedContextView, SchedulerAuthorityError>
where
    S: AuthorityStore
        + ContextStore
        + ContextAuthorizationFactStore
        + IntentChainStore
        + ProtocolStore,
    F: FnOnce() -> Result<(), SchedulerAuthorityError>,
{
    let current_contract_epoch = store
        .current_contract_epoch(&command.task_ref)
        .map_err(|error| SchedulerAuthorityError::ContextRequestUnavailable(error.to_string()))?;
    let contract_row = store
        .load_task_contract(&command.task_ref, current_contract_epoch)
        .map_err(|error| SchedulerAuthorityError::ContextRequestUnavailable(error.to_string()))?
        .ok_or_else(|| SchedulerAuthorityError::MissingContract(command.task_ref.clone()))?;
    let contract = parse_execution_bound_contract(&contract_row.canonical_json)?;
    let contract_request_reference = contract.context_request_ref.as_ref().ok_or_else(|| {
        SchedulerAuthorityError::ContextRequestUnavailable(
            "current TaskContract has no ContextRequest binding".to_owned(),
        )
    })?;
    let request_row = store
        .load_context_request(&command.request_id)
        .map_err(|error| SchedulerAuthorityError::ContextRequestUnavailable(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::ContextRequestUnavailable(command.request_id.to_string())
        })?;
    if contract_request_reference.id.0.as_str() != command.request_id.as_str()
        || contract_request_reference.kind != StrongReferenceKind::Strong
        || contract_request_reference.object_version != 1
        || contract_request_reference.content_digest.0 != request_row.request_digest
    {
        return Err(SchedulerAuthorityError::ContextRequestUnavailable(
            "current TaskContract ContextRequest reference differs from durable request".to_owned(),
        ));
    }
    if request_row.task_ref != command.task_ref {
        return Err(SchedulerAuthorityError::ContextRequestUnavailable(
            "ContextRequest task binding differs from scheduler task".to_owned(),
        ));
    }
    let context_request: ContextRequest = serde_json::from_str(&request_row.canonical_json)
        .map_err(|error| SchedulerAuthorityError::ContextRequestUnavailable(error.to_string()))?;
    if context_request.perspective.task != command.task_ref {
        return Err(SchedulerAuthorityError::ContextRequestUnavailable(
            "ContextRequest payload task differs from durable task binding".to_owned(),
        ));
    }
    if context_request.perspective.principal != command.authorization_subject_ref {
        return Err(SchedulerAuthorityError::ContextAuthorizationUnavailable(
            "ContextRequest principal differs from scheduler authorization subject".to_owned(),
        ));
    }
    let authorization_snapshot = load_current_context_authorization_snapshot(store, command)?;
    let mut required_fragment_candidates = build_required_task_fragments(
        &authorization_snapshot,
        command,
        &request_row,
        &context_request,
        &contract_row,
        &contract,
    )?;
    let required_fragment_refs = required_fragment_candidates
        .iter()
        .map(|candidate| candidate.object_ref.clone())
        .collect::<Vec<_>>();
    let metadata = store
        .query_context_candidate_metadata(&ContextCandidateQuery {
            tenant_id: command.tenant_id.clone(),
            resource_scope_prefix: command.resource_scope_prefix.clone(),
            conversation_ref: command.conversation_ref.clone(),
            limit: command.source_limit,
        })
        .map_err(|error| SchedulerAuthorityError::ContextRequestUnavailable(error.to_string()))?;
    after_metadata()?;

    let mut authorized_candidates =
        Vec::with_capacity(metadata.len() + required_fragment_candidates.len());
    authorized_candidates.append(&mut required_fragment_candidates);
    let mut excluded_source_records = Vec::new();
    let mut authorization_denied_after_discovery = false;
    for source_metadata in metadata {
        let Some(source_family) = source_family_for_role(source_metadata.role) else {
            excluded_source_records.push((
                RejectedCandidate {
                    candidate_ref: source_metadata.source_id.to_string(),
                    reason: "SOURCE_FAMILY_EXCLUDED".to_owned(),
                },
                LossEntry {
                    source: source_metadata.source_id.to_string(),
                    transform: "omitted_source_family".to_owned(),
                    omitted_classes: vec!["unsupported_source_role".to_owned()],
                    verification: Some(source_metadata.source_digest.clone()),
                },
            ));
            continue;
        };
        if !context_request
            .priority
            .iter()
            .any(|priority| priority == source_family)
        {
            excluded_source_records.push((
                RejectedCandidate {
                    candidate_ref: source_metadata.source_id.to_string(),
                    reason: "SOURCE_FAMILY_EXCLUDED".to_owned(),
                },
                LossEntry {
                    source: source_metadata.source_id.to_string(),
                    transform: "omitted_source_family".to_owned(),
                    omitted_classes: vec![source_family.to_owned()],
                    verification: Some(source_metadata.source_digest.clone()),
                },
            ));
            continue;
        }
        if let Some(reason) = source_freshness_reason(
            source_metadata.role,
            &source_metadata.created_at,
            &command.decided_at,
            &context_request,
        ) {
            excluded_source_records.push((
                RejectedCandidate {
                    candidate_ref: source_metadata.source_id.to_string(),
                    reason: reason.to_owned(),
                },
                LossEntry {
                    source: source_metadata.source_id.to_string(),
                    transform: "omitted_stale_source".to_owned(),
                    omitted_classes: vec![source_family.to_owned()],
                    verification: Some(source_metadata.source_digest.clone()),
                },
            ));
            continue;
        }
        // Discovery is metadata-only. Re-read durable authorization state for
        // every body, so a revocation that lands after discovery cannot reach
        // body materialization, ranking, rendering, or the Pi boundary.
        let current_authorization_snapshot =
            load_current_context_authorization_snapshot(store, command)?;
        if authorize(
            &current_authorization_snapshot,
            &source_metadata.governance,
            &AccessRequest {
                action: "read_body".to_owned(),
                purpose: context_request.purpose.clone(),
            },
        )
        .is_err()
        {
            authorization_denied_after_discovery = true;
            continue;
        }
        let source = store
            .load_workspace_context_source_body(&source_metadata.source_id)
            .map_err(|error| SchedulerAuthorityError::ContextBodyUnavailable(error.to_string()))?
            .ok_or_else(|| {
                SchedulerAuthorityError::ContextBodyUnavailable(
                    source_metadata.source_id.to_string(),
                )
            })?;
        if source.source_digest != source_metadata.source_digest
            || source.governance != source_metadata.governance
            || source.role != source_metadata.role
            || source.trust_level != source_metadata.trust_level
            || source.content_bytes != source_metadata.content_bytes
            || source.content_tokens != source_metadata.content_tokens
        {
            return Err(SchedulerAuthorityError::ContextBodyUnavailable(
                "Context body metadata no longer matches its discovery record".to_owned(),
            ));
        }
        let source_payload: Value = serde_json::from_str(&source.canonical_json)
            .map_err(|error| SchedulerAuthorityError::ContextBodyUnavailable(error.to_string()))?;
        let body = source_payload.get("body").cloned().ok_or_else(|| {
            SchedulerAuthorityError::ContextBodyUnavailable(
                "WorkspaceContextSource payload has no body".to_owned(),
            )
        })?;
        let source_governance = source.governance;
        authorized_candidates.push(CandidateObject {
            object_ref: source_governance.object_ref.clone(),
            object_version: 1,
            content_digest: source.source_digest,
            governance: source_governance,
            role: source.role,
            trust_level: source.trust_level,
            representation: source.representation,
            body,
            cost_bytes: source.content_bytes,
            cost_tokens: source.content_tokens.unwrap_or(0),
        });
    }

    // A post-discovery denial invalidates the source set selected for this
    // resolution. Required daemon fragments must not allow that stale source
    // authorization boundary to be bypassed before Pi receives a view.
    if authorization_denied_after_discovery {
        return Err(SchedulerAuthorityError::ContextAuthorizationUnavailable(
            "Context source was denied before body materialization".to_owned(),
        ));
    }

    let resolution_request = ResolutionRequest {
        snapshot: authorization_snapshot,
        purpose: context_request.purpose,
        conversation_ref: command.conversation_ref.clone(),
        required: required_fragment_refs
            .into_iter()
            .chain(
                context_request
                    .required
                    .into_iter()
                    .map(|required| required.r#ref),
            )
            .map(|object_ref| RequiredItem { object_ref })
            .collect(),
        allow_partial: context_request.allow_partial,
        budget: ContextBudget {
            context_bytes: context_request.budget.context_bytes,
            input_tokens: context_request.budget.input_tokens,
        },
        render: RenderSpec {
            renderer_version: "personal-context-render/1".to_owned(),
            target_profile: context_request.target_profile.schema,
        },
        schema_digest: cognitive_contracts::generated::context_request::SCHEMA_DIGEST.to_owned(),
    };
    let mut resolved_view = resolve(
        &resolution_request,
        &authorized_candidates,
        &ArrivalOrderRanker,
    )
    .map_err(|error| SchedulerAuthorityError::ContextResolution(error.to_string()))?;
    for (rejected, loss) in excluded_source_records {
        resolved_view.rejected.push(rejected);
        resolved_view.loss_declaration.push(loss);
    }
    Ok(resolved_view)
}

/// Persist the exact immutable ContextView that is about to become input to a
/// private candidate producer. The durable view intentionally contains only
/// source metadata and strong references; source bodies remain confined to the
/// already-authorized resolver and the bounded rendered transport.
pub(crate) fn persist_resolved_context_view<S, C, G>(
    store: &S,
    clock: &C,
    identifiers: &G,
    request_row: &ContextRequestRow,
    resolved_view: &ResolvedContextView,
    governance: &GovernanceSeed,
) -> Result<ContextViewRow, SchedulerAuthorityError>
where
    S: ContextStore,
    C: Clock,
    G: IdGenerator,
{
    let view_id = next_object_id(identifiers)?;
    let resolved_at = clock
        .now()
        .map_err(|error| SchedulerAuthorityError::ContextResolution(error.detail))?;
    let header = compose_governed_header(
        &view_id,
        "ContextView",
        "cognitiveos.context-view/0.1",
        governance,
        vec![format!(
            "activity://personal/context/{}",
            request_row.request_id
        )],
        vec![request_row.request_id.to_string()],
        "daemon-persisted-context-resolution",
        &resolved_at,
    )
    .map_err(|error| SchedulerAuthorityError::ContextResolution(error.to_string()))?;
    let loaded = resolved_view
        .loaded
        .iter()
        .map(|item| {
            let source_id = ObjectId::parse(&item.object_ref).map_err(|_| {
                SchedulerAuthorityError::ContextResolution(
                    "resolved Context item identity is not a governed object identifier".to_owned(),
                )
            })?;
            Ok(LoadedContextItem {
                item_id: item.object_ref.clone(),
                object_ref: strong_reference_to(&source_id, &item.content_digest),
                representation: item.representation,
                trust_level: item.trust_level,
                role: item.role,
                cost: ItemCost {
                    bytes: item.cost_bytes,
                    tokens: Some(item.cost_tokens),
                },
            })
        })
        .collect::<Result<Vec<_>, SchedulerAuthorityError>>()?;
    let pinned_versions = resolved_view
        .pinned_versions
        .iter()
        .map(|(object_ref, version)| {
            (
                object_ref.clone(),
                ContextViewPinnedVersionsValue::Integer(*version),
            )
        })
        .collect();
    let payload = ContextView {
        activity_bound: format!("activity://personal/context/{}", request_row.request_id),
        complete: resolved_view.complete,
        cost: ResolutionCost {
            bytes: resolved_view
                .loaded
                .iter()
                .map(|item| item.cost_bytes)
                .sum(),
            money_microunits: None,
            resolve_ms: 0,
            tokens: Some(
                resolved_view
                    .loaded
                    .iter()
                    .map(|item| item.cost_tokens)
                    .sum(),
            ),
        },
        header,
        loaded,
        loss_declaration: resolved_view
            .loss_declaration
            .iter()
            .map(|loss| LossDeclaration {
                omitted_classes: loss.omitted_classes.clone(),
                source: loss.source.clone(),
                transform: loss.transform.clone(),
                verification: loss
                    .verification
                    .clone()
                    .unwrap_or_else(|| resolved_view.render.digest.clone()),
            })
            .collect(),
        missing: (!resolved_view.missing.is_empty()).then(|| resolved_view.missing.clone()),
        pinned_versions,
        rejected: resolved_view
            .rejected
            .iter()
            .map(|rejected| PersistedRejectedCandidate {
                candidate_ref: rejected.candidate_ref.clone(),
                reason: rejected.reason.clone(),
            })
            .collect(),
        request_ref: strong_reference_to(&request_row.request_id, &request_row.request_digest),
    };
    let payload_value = serde_json::to_value(payload)
        .map_err(|error| SchedulerAuthorityError::ContextResolution(error.to_string()))?;
    let (sealed_payload, view_digest) = seal_governed_object_content_digest(payload_value)
        .map_err(|error| SchedulerAuthorityError::ContextResolution(error.to_string()))?;
    let canonical_json = String::from_utf8(
        canonical::canonical_bytes_of_value(&sealed_payload)
            .map_err(|error| SchedulerAuthorityError::ContextResolution(error.to_string()))?,
    )
    .map_err(|error| SchedulerAuthorityError::ContextResolution(error.to_string()))?;
    let view_row = ContextViewRow {
        view_id,
        request_id: request_row.request_id.clone(),
        view_digest,
        canonical_json,
    };
    store
        .append_context_view(&view_row)
        .map_err(|error| SchedulerAuthorityError::ContextResolution(error.to_string()))?;
    Ok(view_row)
}
