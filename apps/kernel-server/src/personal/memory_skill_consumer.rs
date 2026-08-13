//! 生产 Memory/Skill 受治理消费方。
//!
//! 在装入 Context 前按当前权威事实复核 scope/pin/digest；遗忘、撤销或 digest
//! 漂移一律失败闭合。跨会话复用只重放精确钉，不信任旧正文。

use crate::personal::scheduler_authority::{ContextResolutionCommand, SchedulerAuthorityError};
use cognitive_contracts::generated::context_view::{
    LoadedContextItemRepresentation, LoadedContextItemRole, LoadedContextItemTrustLevel,
};
use cognitive_domain::ObjectId;
use cognitive_kernel::authz::ObjectGovernance;
use cognitive_kernel::context::CandidateObject;
use cognitive_kernel::memory_skill_consumption::{
    MemoryConsumptionPin, MemorySkillConsumptionRecord, MemorySkillConsumptionStore,
    SkillConsumptionPin,
};
use cognitive_kernel::ports::{ContextStore, StorePortError, WorkspaceContextSourceRow};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

struct ConsumptionRecordInput<'a> {
    command: &'a ContextResolutionCommand,
    contract_epoch: i64,
    context_request_digest: &'a str,
    session_ref: &'a str,
    reuse_of: Option<ObjectId>,
    memory: &'a [MemoryConsumptionPin],
    skill: &'a [SkillConsumptionPin],
}

/// 从当前权威事实装载可进入 Context 的 Memory/Skill 片段，并写入只追加消费记录。
pub(crate) fn load_governed_memory_skill_candidates<S>(
    store: &S,
    command: &ContextResolutionCommand,
    contract_epoch: i64,
    context_request_digest: &str,
    purpose: &str,
) -> Result<Vec<CandidateObject>, SchedulerAuthorityError>
where
    S: ContextStore + MemorySkillConsumptionStore,
{
    let session_ref = command
        .conversation_ref
        .clone()
        .unwrap_or_else(|| format!("session://task/{}", command.task_ref));
    let prior = store
        .load_latest_memory_skill_consumption(
            &command.task_ref,
            contract_epoch,
            &command.request_id,
        )
        .map_err(consumption_store_error)?;
    let (memory_pins, skill_pins, reuse_of) = if let Some(record) = prior {
        revalidate_consumption_pins(store, &record, command, purpose)?;
        (
            record.memory.clone(),
            record.skill.clone(),
            Some(record.consumption_id.clone()),
        )
    } else {
        let memory_pins = store
            .list_eligible_memory_pins(
                &command.resource_scope_prefix,
                purpose,
                timestamp_unix_seconds(&command.decided_at),
            )
            .map_err(consumption_store_error)?;
        let skill_pins = store
            .list_eligible_skill_pins(&command.resource_scope_prefix, &command.task_ref)
            .map_err(consumption_store_error)?;
        (memory_pins, skill_pins, None)
    };

    let mut candidates = Vec::new();
    for pin in &memory_pins {
        candidates.push(load_memory_candidate(store, command, pin)?);
    }
    for pin in &skill_pins {
        candidates.push(load_skill_candidate(store, command, pin)?);
    }

    if !memory_pins.is_empty() || !skill_pins.is_empty() {
        persist_consumption_record(
            store,
            ConsumptionRecordInput {
                command,
                contract_epoch,
                context_request_digest,
                session_ref: &session_ref,
                reuse_of,
                memory: &memory_pins,
                skill: &skill_pins,
            },
        )?;
    }
    Ok(candidates)
}

fn revalidate_consumption_pins<S>(
    store: &S,
    record: &MemorySkillConsumptionRecord,
    command: &ContextResolutionCommand,
    purpose: &str,
) -> Result<(), SchedulerAuthorityError>
where
    S: MemorySkillConsumptionStore,
{
    if record.task_ref != command.task_ref
        || record.context_request_id != command.request_id
        || record.context_request_digest.trim().is_empty()
    {
        return Err(SchedulerAuthorityError::ContextResolution(
            "durable Memory/Skill consumption no longer matches the current Task request"
                .to_owned(),
        ));
    }
    let live_memory = store
        .list_eligible_memory_pins(
            &command.resource_scope_prefix,
            purpose,
            timestamp_unix_seconds(&command.decided_at),
        )
        .map_err(consumption_store_error)?;
    for pin in &record.memory {
        if !live_memory.iter().any(|live| {
            live.memory_id == pin.memory_id
                && live.source_id == pin.source_id
                && live.source_digest == pin.source_digest
        }) {
            return Err(SchedulerAuthorityError::ContextAuthorizationUnavailable(
                "forgotten, expired, or digest-drifted Memory cannot be reused".to_owned(),
            ));
        }
    }
    let live_skill = store
        .list_eligible_skill_pins(&command.resource_scope_prefix, &command.task_ref)
        .map_err(consumption_store_error)?;
    for pin in &record.skill {
        if !live_skill.iter().any(|live| {
            live.binding_id == pin.binding_id
                && live.revision_id == pin.revision_id
                && live.content_digest == pin.content_digest
        }) {
            return Err(SchedulerAuthorityError::ContextAuthorizationUnavailable(
                "revoked or digest-drifted Skill cannot be reused".to_owned(),
            ));
        }
    }
    Ok(())
}

fn load_memory_candidate<S>(
    store: &S,
    command: &ContextResolutionCommand,
    pin: &MemoryConsumptionPin,
) -> Result<CandidateObject, SchedulerAuthorityError>
where
    S: ContextStore,
{
    let source = store
        .load_workspace_context_source_body(&pin.source_id)
        .map_err(|error| SchedulerAuthorityError::ContextBodyUnavailable(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::ContextBodyUnavailable(pin.source_id.to_string())
        })?;
    if source.source_digest != pin.source_digest
        || !scope_is_authorized(
            &source.governance.resource_scope,
            &command.resource_scope_prefix,
        )
    {
        return Err(SchedulerAuthorityError::ContextBodyUnavailable(
            "Memory source pin, digest, or scope no longer matches the current authority fact"
                .to_owned(),
        ));
    }
    verify_source_digest(&source)?;
    let source_payload: Value = serde_json::from_str(&source.canonical_json)
        .map_err(|error| SchedulerAuthorityError::ContextBodyUnavailable(error.to_string()))?;
    let body = source_payload.get("body").cloned().ok_or_else(|| {
        SchedulerAuthorityError::ContextBodyUnavailable(
            "Memory source payload has no body".to_owned(),
        )
    })?;
    let mut governance = source.governance;
    governance.object_ref = pin.memory_id.to_string();
    // 跨会话复用只重放精确钉；当前会话 conversation 不得把已准入 Memory 卡在旧会话上。
    governance.conversation_ref = command.conversation_ref.clone();
    Ok(CandidateObject {
        object_ref: pin.memory_id.to_string(),
        object_version: 1,
        content_digest: pin.source_digest.clone(),
        governance,
        role: LoadedContextItemRole::Working,
        trust_level: LoadedContextItemTrustLevel::Verified,
        representation: LoadedContextItemRepresentation::Text,
        body,
        cost_bytes: source.content_bytes,
        cost_tokens: source.content_tokens.unwrap_or(0),
    })
}

fn load_skill_candidate<S>(
    store: &S,
    command: &ContextResolutionCommand,
    pin: &SkillConsumptionPin,
) -> Result<CandidateObject, SchedulerAuthorityError>
where
    S: MemorySkillConsumptionStore,
{
    let (content_digest, canonical_json) = store
        .load_skill_revision_payload(&pin.revision_id)
        .map_err(consumption_store_error)?
        .ok_or_else(|| {
            SchedulerAuthorityError::ContextBodyUnavailable(pin.revision_id.to_string())
        })?;
    if content_digest != pin.content_digest
        || !canonical_json_contains_digest(&canonical_json, &pin.content_digest)
    {
        return Err(SchedulerAuthorityError::ContextBodyUnavailable(
            "Skill revision digest no longer matches its exact pin".to_owned(),
        ));
    }
    let body: Value = serde_json::from_str(&canonical_json)
        .map_err(|error| SchedulerAuthorityError::ContextBodyUnavailable(error.to_string()))?;
    Ok(CandidateObject {
        object_ref: pin.binding_id.to_string(),
        object_version: 1,
        content_digest: pin.content_digest.clone(),
        governance: ObjectGovernance {
            object_ref: pin.binding_id.to_string(),
            tenant_id: Some(command.tenant_id.clone()),
            owner_ref: command.authorization_subject_ref.clone(),
            resource_scope: command.resource_scope_prefix.clone(),
            conversation_ref: command.conversation_ref.clone(),
        },
        role: LoadedContextItemRole::Working,
        trust_level: LoadedContextItemTrustLevel::Verified,
        representation: LoadedContextItemRepresentation::Text,
        body,
        cost_bytes: i64::try_from(canonical_json.len()).unwrap_or(i64::MAX),
        cost_tokens: 1,
    })
}

fn persist_consumption_record<S>(
    store: &S,
    input: ConsumptionRecordInput<'_>,
) -> Result<(), SchedulerAuthorityError>
where
    S: MemorySkillConsumptionStore,
{
    let existing = store
        .load_latest_memory_skill_consumption(
            &input.command.task_ref,
            input.contract_epoch,
            &input.command.request_id,
        )
        .map_err(consumption_store_error)?;
    if let Some(record) = existing
        && record.session_ref == input.session_ref
        && record.memory == input.memory
        && record.skill == input.skill
        && record.context_request_digest == input.context_request_digest
    {
        return Ok(());
    }
    let canonical = json!({
        "memory": input.memory.iter().map(|pin| json!({
            "memory_id": pin.memory_id.to_string(),
            "source_id": pin.source_id.to_string(),
            "source_digest": pin.source_digest,
        })).collect::<Vec<_>>(),
        "skill": input.skill.iter().map(|pin| json!({
            "binding_id": pin.binding_id.to_string(),
            "revision_id": pin.revision_id.to_string(),
            "package_id": pin.package_id.to_string(),
            "content_digest": pin.content_digest,
        })).collect::<Vec<_>>(),
    });
    let record = MemorySkillConsumptionRecord {
        consumption_id: consumption_identity(
            input.command,
            input.contract_epoch,
            input.session_ref,
            input.reuse_of.as_ref(),
        )?,
        task_ref: input.command.task_ref.clone(),
        contract_epoch: input.contract_epoch,
        context_request_id: input.command.request_id.clone(),
        context_request_digest: input.context_request_digest.to_owned(),
        session_ref: input.session_ref.to_owned(),
        reuse_of: input.reuse_of,
        memory: input.memory.to_vec(),
        skill: input.skill.to_vec(),
        canonical_json: canonical.to_string(),
    };
    match store.append_memory_skill_consumption(&record) {
        Ok(()) => Ok(()),
        Err(StorePortError::Conflict { .. }) => Ok(()),
        Err(error) => Err(consumption_store_error(error)),
    }
}

fn verify_source_digest(source: &WorkspaceContextSourceRow) -> Result<(), SchedulerAuthorityError> {
    let payload: Value = serde_json::from_str(&source.canonical_json)
        .map_err(|error| SchedulerAuthorityError::ContextBodyUnavailable(error.to_string()))?;
    let header_digest = payload
        .pointer("/header/content_digest")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            SchedulerAuthorityError::ContextBodyUnavailable(
                "Memory source header digest is missing".to_owned(),
            )
        })?;
    if header_digest != source.source_digest {
        return Err(SchedulerAuthorityError::ContextBodyUnavailable(
            "Memory source digest does not match its sealed header".to_owned(),
        ));
    }
    Ok(())
}

fn canonical_json_contains_digest(canonical_json: &str, expected_digest: &str) -> bool {
    serde_json::from_str::<Value>(canonical_json)
        .ok()
        .and_then(|value| {
            value
                .get("content_digest")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .as_deref()
        == Some(expected_digest)
}

fn scope_is_authorized(scope: &str, prefix: &str) -> bool {
    scope == prefix || scope.starts_with(&format!("{prefix}/"))
}

fn consumption_identity(
    command: &ContextResolutionCommand,
    contract_epoch: i64,
    session_ref: &str,
    reuse_of: Option<&ObjectId>,
) -> Result<ObjectId, SchedulerAuthorityError> {
    let mut hasher = Sha256::new();
    hasher.update(command.task_ref.as_bytes());
    hasher.update(contract_epoch.to_be_bytes());
    hasher.update(command.request_id.as_str().as_bytes());
    hasher.update(session_ref.as_bytes());
    if let Some(prior) = reuse_of {
        hasher.update(prior.as_str().as_bytes());
    }
    let digest = hasher.finalize();
    let first = u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]]);
    let second = u16::from_be_bytes([digest[4], digest[5]]);
    let last = u64::from_be_bytes([
        digest[8], digest[9], digest[10], digest[11], digest[12], digest[13], digest[14],
        digest[15],
    ]) & 0x0000_ffff_ffff_ffff;
    ObjectId::parse(&format!("{first:08x}-{second:04x}-7000-9000-{last:012x}")).map_err(|error| {
        SchedulerAuthorityError::ContextResolution(format!(
            "deterministic consumption identity is invalid: {error}"
        ))
    })
}

fn timestamp_unix_seconds(timestamp: &cognitive_domain::WallTimestamp) -> i64 {
    crate::personal::scheduler_authority::timestamp_milliseconds(timestamp)
        .map(|milliseconds| milliseconds / 1000)
        .unwrap_or(1)
}

fn consumption_store_error(error: StorePortError) -> SchedulerAuthorityError {
    SchedulerAuthorityError::ContextResolution(error.to_string())
}
