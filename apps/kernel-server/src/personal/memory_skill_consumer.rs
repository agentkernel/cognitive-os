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
    EligibleMemoryConsumption, MemoryConsumptionPin, MemorySkillConsumptionRecord,
    MemorySkillConsumptionStore, SkillConsumptionPin,
};
use cognitive_kernel::ports::{ContextStore, StorePortError, WorkspaceContextSourceRow};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

struct ConsumptionRecordInput<'a> {
    command: &'a ContextResolutionCommand,
    contract_epoch: i64,
    context_request_digest: &'a str,
    purpose: &'a str,
    session_ref: &'a str,
    reuse_of: Option<ObjectId>,
    memory: &'a [MemoryConsumptionPin],
    skill: &'a [SkillConsumptionPin],
}

struct ConsumptionIdentityInput<'a> {
    task_ref: &'a str,
    contract_epoch: i64,
    context_request_id: &'a ObjectId,
    context_request_digest: &'a str,
    principal_ref: &'a str,
    tenant_id: &'a str,
    resource_scope: &'a str,
    purpose: &'a str,
    session_ref: &'a str,
    reuse_of: Option<&'a ObjectId>,
    memory: &'a [MemoryConsumptionPin],
    skill: &'a [SkillConsumptionPin],
}

struct ConsumptionGovernanceBinding {
    principal_ref: String,
    tenant_id: String,
    resource_scope: String,
    purpose: String,
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
    let (eligible_memory, skill_pins, reuse_of) = if let Some(record) = prior {
        let eligible_memory =
            revalidate_consumption_pins(store, &record, command, context_request_digest, purpose)?;
        (
            eligible_memory,
            record.skill.clone(),
            Some(record.consumption_id.clone()),
        )
    } else {
        let eligible_memory = store
            .list_eligible_memory_pins(
                &command.resource_scope_prefix,
                &command.task_ref,
                purpose,
                timestamp_unix_seconds(&command.decided_at),
            )
            .map_err(consumption_store_error)?;
        for eligible in &eligible_memory {
            validate_memory_metadata(eligible, command, purpose)?;
        }
        let skill_pins = store
            .list_eligible_skill_pins(&command.resource_scope_prefix, &command.task_ref)
            .map_err(consumption_store_error)?;
        (eligible_memory, skill_pins, None)
    };
    let memory_pins = eligible_memory
        .iter()
        .map(|eligible| eligible.pin.clone())
        .collect::<Vec<_>>();

    let mut candidates = Vec::new();
    for eligible in &eligible_memory {
        candidates.push(load_memory_candidate(store, command, eligible)?);
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
                purpose,
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
    context_request_digest: &str,
    purpose: &str,
) -> Result<Vec<EligibleMemoryConsumption>, SchedulerAuthorityError>
where
    S: MemorySkillConsumptionStore,
{
    if record.task_ref != command.task_ref || record.context_request_id != command.request_id {
        return Err(SchedulerAuthorityError::ContextResolution(
            "durable Memory/Skill consumption no longer matches the current Task request"
                .to_owned(),
        ));
    }
    if record.context_request_digest != context_request_digest {
        return Err(SchedulerAuthorityError::ContextAuthorizationUnavailable(
            "durable Memory/Skill consumption request digest differs from the current request"
                .to_owned(),
        ));
    }
    let binding = consumption_governance_binding(record)?;
    if binding.principal_ref != command.authorization_subject_ref {
        return Err(SchedulerAuthorityError::ContextAuthorizationUnavailable(
            "durable Memory/Skill consumption principal differs from the current authenticated principal"
                .to_owned(),
        ));
    }
    if binding.tenant_id != command.tenant_id {
        return Err(SchedulerAuthorityError::ContextAuthorizationUnavailable(
            "durable Memory/Skill consumption tenant differs from the current authenticated tenant"
                .to_owned(),
        ));
    }
    if binding.resource_scope != command.resource_scope_prefix || binding.purpose != purpose {
        return Err(SchedulerAuthorityError::ContextAuthorizationUnavailable(
            "durable Memory/Skill consumption scope or purpose differs from the current request"
                .to_owned(),
        ));
    }
    let expected_identity = consumption_identity(ConsumptionIdentityInput {
        task_ref: &record.task_ref,
        contract_epoch: record.contract_epoch,
        context_request_id: &record.context_request_id,
        context_request_digest: &record.context_request_digest,
        principal_ref: &binding.principal_ref,
        tenant_id: &binding.tenant_id,
        resource_scope: &binding.resource_scope,
        purpose: &binding.purpose,
        session_ref: &record.session_ref,
        reuse_of: record.reuse_of.as_ref(),
        memory: &record.memory,
        skill: &record.skill,
    })?;
    if record.consumption_id != expected_identity {
        return Err(SchedulerAuthorityError::ContextResolution(
            "durable Memory/Skill consumption identity does not match its exact bindings"
                .to_owned(),
        ));
    }
    let live_memory = store
        .list_eligible_memory_pins(
            &command.resource_scope_prefix,
            &command.task_ref,
            purpose,
            timestamp_unix_seconds(&command.decided_at),
        )
        .map_err(consumption_store_error)?;
    let mut selected_memory = Vec::with_capacity(record.memory.len());
    for pin in &record.memory {
        let Some(live) = live_memory.iter().find(|live| live.pin == *pin) else {
            return Err(SchedulerAuthorityError::ContextAuthorizationUnavailable(
                "forgotten, expired, or digest-drifted Memory cannot be reused".to_owned(),
            ));
        };
        validate_memory_metadata(live, command, purpose)?;
        selected_memory.push(live.clone());
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
    Ok(selected_memory)
}

fn consumption_governance_binding(
    record: &MemorySkillConsumptionRecord,
) -> Result<ConsumptionGovernanceBinding, SchedulerAuthorityError> {
    let document: Value = serde_json::from_str(&record.canonical_json).map_err(|error| {
        SchedulerAuthorityError::ContextResolution(format!(
            "durable Memory/Skill consumption payload is malformed: {error}"
        ))
    })?;
    let expected = json!({
        "memory": record.memory.iter().map(|pin| json!({
            "memory_id": pin.memory_id.to_string(),
            "source_id": pin.source_id.to_string(),
            "source_digest": pin.source_digest,
        })).collect::<Vec<_>>(),
        "skill": record.skill.iter().map(|pin| json!({
            "binding_id": pin.binding_id.to_string(),
            "revision_id": pin.revision_id.to_string(),
            "package_id": pin.package_id.to_string(),
            "content_digest": pin.content_digest,
        })).collect::<Vec<_>>(),
    });
    if document.get("memory") != expected.get("memory")
        || document.get("skill") != expected.get("skill")
    {
        return Err(SchedulerAuthorityError::ContextResolution(
            "durable Memory/Skill consumption canonical pins differ from the loaded record"
                .to_owned(),
        ));
    }
    Ok(ConsumptionGovernanceBinding {
        principal_ref: required_consumption_field(&document, "principal_ref")?,
        tenant_id: required_consumption_field(&document, "tenant_id")?,
        resource_scope: required_consumption_field(&document, "resource_scope")?,
        purpose: required_consumption_field(&document, "purpose")?,
    })
}

fn required_consumption_field(
    document: &Value,
    field: &str,
) -> Result<String, SchedulerAuthorityError> {
    document
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            SchedulerAuthorityError::ContextResolution(format!(
                "durable Memory/Skill consumption {field} binding is missing"
            ))
        })
}

fn validate_memory_metadata(
    eligible: &EligibleMemoryConsumption,
    command: &ContextResolutionCommand,
    purpose: &str,
) -> Result<(), SchedulerAuthorityError> {
    if eligible.tenant_id != command.tenant_id {
        return Err(SchedulerAuthorityError::ContextAuthorizationUnavailable(
            "Memory tenant differs from the current authenticated tenant".to_owned(),
        ));
    }
    if eligible.owner_ref.trim().is_empty() {
        return Err(SchedulerAuthorityError::ContextAuthorizationUnavailable(
            "Memory source owner is missing".to_owned(),
        ));
    }
    if !scope_is_authorized(&eligible.resource_scope, &command.resource_scope_prefix) {
        return Err(SchedulerAuthorityError::ContextAuthorizationUnavailable(
            "Memory scope differs from the current authorized scope".to_owned(),
        ));
    }
    if eligible.target_scope != command.task_ref || eligible.purpose != purpose {
        return Err(SchedulerAuthorityError::ContextAuthorizationUnavailable(
            "Memory Task target or purpose differs from the current request".to_owned(),
        ));
    }
    if eligible.source_provenance_ref.trim().is_empty() {
        return Err(SchedulerAuthorityError::ContextAuthorizationUnavailable(
            "Memory source provenance is missing".to_owned(),
        ));
    }
    Ok(())
}

fn load_memory_candidate<S>(
    store: &S,
    command: &ContextResolutionCommand,
    eligible: &EligibleMemoryConsumption,
) -> Result<CandidateObject, SchedulerAuthorityError>
where
    S: ContextStore,
{
    let pin = &eligible.pin;
    let source = store
        .load_workspace_context_source_body(&pin.source_id)
        .map_err(|error| SchedulerAuthorityError::ContextBodyUnavailable(error.to_string()))?
        .ok_or_else(|| {
            SchedulerAuthorityError::ContextBodyUnavailable(pin.source_id.to_string())
        })?;
    if source.source_digest != pin.source_digest
        || source.governance.tenant_id.as_deref() != Some(eligible.tenant_id.as_str())
        || source.governance.owner_ref != eligible.owner_ref
        || source.governance.resource_scope != eligible.resource_scope
        || source.provenance_ref != eligible.source_provenance_ref
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
        "principal_ref": input.command.authorization_subject_ref,
        "tenant_id": input.command.tenant_id,
        "resource_scope": input.command.resource_scope_prefix,
        "purpose": input.purpose,
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
    let consumption_id = consumption_identity(ConsumptionIdentityInput {
        task_ref: &input.command.task_ref,
        contract_epoch: input.contract_epoch,
        context_request_id: &input.command.request_id,
        context_request_digest: input.context_request_digest,
        principal_ref: &input.command.authorization_subject_ref,
        tenant_id: &input.command.tenant_id,
        resource_scope: &input.command.resource_scope_prefix,
        purpose: input.purpose,
        session_ref: input.session_ref,
        reuse_of: input.reuse_of.as_ref(),
        memory: input.memory,
        skill: input.skill,
    })?;
    let record = MemorySkillConsumptionRecord {
        consumption_id,
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
        Err(StorePortError::Conflict { detail }) => {
            let persisted = store
                .load_memory_skill_consumption(&record.consumption_id)
                .map_err(consumption_store_error)?;
            if persisted.as_ref() == Some(&record) {
                Ok(())
            } else {
                Err(SchedulerAuthorityError::ContextResolution(format!(
                    "durable Memory/Skill consumption conflict is not an exact idempotent replay: {detail}"
                )))
            }
        }
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
    input: ConsumptionIdentityInput<'_>,
) -> Result<ObjectId, SchedulerAuthorityError> {
    let mut hasher = Sha256::new();
    hash_identity_field(&mut hasher, b"cognitiveos.memory-skill-consumption/1");
    hash_identity_field(&mut hasher, input.task_ref.as_bytes());
    hash_identity_field(&mut hasher, &input.contract_epoch.to_be_bytes());
    hash_identity_field(&mut hasher, input.context_request_id.as_str().as_bytes());
    hash_identity_field(&mut hasher, input.context_request_digest.as_bytes());
    hash_identity_field(&mut hasher, input.principal_ref.as_bytes());
    hash_identity_field(&mut hasher, input.tenant_id.as_bytes());
    hash_identity_field(&mut hasher, input.resource_scope.as_bytes());
    hash_identity_field(&mut hasher, input.purpose.as_bytes());
    hash_identity_field(&mut hasher, input.session_ref.as_bytes());
    match input.reuse_of {
        Some(prior) => {
            hash_identity_field(&mut hasher, b"some");
            hash_identity_field(&mut hasher, prior.as_str().as_bytes());
        }
        None => hash_identity_field(&mut hasher, b"none"),
    }
    hash_identity_field(
        &mut hasher,
        &u64::try_from(input.memory.len())
            .unwrap_or(u64::MAX)
            .to_be_bytes(),
    );
    for pin in input.memory {
        hash_identity_field(&mut hasher, b"memory");
        hash_identity_field(&mut hasher, pin.memory_id.as_str().as_bytes());
        hash_identity_field(&mut hasher, pin.source_id.as_str().as_bytes());
        hash_identity_field(&mut hasher, pin.source_digest.as_bytes());
    }
    hash_identity_field(
        &mut hasher,
        &u64::try_from(input.skill.len())
            .unwrap_or(u64::MAX)
            .to_be_bytes(),
    );
    for pin in input.skill {
        hash_identity_field(&mut hasher, b"skill");
        hash_identity_field(&mut hasher, pin.binding_id.as_str().as_bytes());
        hash_identity_field(&mut hasher, pin.revision_id.as_str().as_bytes());
        hash_identity_field(&mut hasher, pin.package_id.as_str().as_bytes());
        hash_identity_field(&mut hasher, pin.content_digest.as_bytes());
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

fn hash_identity_field(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update(u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_be_bytes());
    hasher.update(bytes);
}

fn timestamp_unix_seconds(timestamp: &cognitive_domain::WallTimestamp) -> i64 {
    crate::personal::scheduler_authority::timestamp_milliseconds(timestamp)
        .map(|milliseconds| milliseconds / 1000)
        .unwrap_or(1)
}

fn consumption_store_error(error: StorePortError) -> SchedulerAuthorityError {
    SchedulerAuthorityError::ContextResolution(error.to_string())
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use cognitive_contracts::generated::context_view::{
        LoadedContextItemRepresentation, LoadedContextItemRole, LoadedContextItemTrustLevel,
    };
    use cognitive_domain::WallTimestamp;
    use cognitive_kernel::ports::{
        ContextCandidateMetadata, ContextCandidateQuery, ContextRequestRow, ContextViewRow,
    };
    use std::cell::{Cell, RefCell};

    struct ConsumerTestStore {
        source: WorkspaceContextSourceRow,
        memory_pins: Vec<MemoryConsumptionPin>,
        skill_pins: Vec<SkillConsumptionPin>,
        prior: MemorySkillConsumptionRecord,
        initial_prior: bool,
        latest_reads: Cell<usize>,
        append_conflict: bool,
        append_attempts: Cell<usize>,
        skill_payload: (String, String),
        body_loads: Cell<usize>,
        skill_payload_loads: Cell<usize>,
        appended: RefCell<Vec<MemorySkillConsumptionRecord>>,
    }

    impl ContextStore for ConsumerTestStore {
        fn append_context_request(&self, _: &ContextRequestRow) -> Result<(), StorePortError> {
            Err(unsupported("append ContextRequest"))
        }

        fn load_context_request(
            &self,
            _: &ObjectId,
        ) -> Result<Option<ContextRequestRow>, StorePortError> {
            Err(unsupported("load ContextRequest"))
        }

        fn append_context_view(&self, _: &ContextViewRow) -> Result<(), StorePortError> {
            Err(unsupported("append ContextView"))
        }

        fn load_context_view(
            &self,
            _: &ObjectId,
        ) -> Result<Option<ContextViewRow>, StorePortError> {
            Err(unsupported("load ContextView"))
        }

        fn append_workspace_context_source(
            &self,
            _: &WorkspaceContextSourceRow,
        ) -> Result<(), StorePortError> {
            Err(unsupported("append Context source"))
        }

        fn query_context_candidate_metadata(
            &self,
            _: &ContextCandidateQuery,
        ) -> Result<Vec<ContextCandidateMetadata>, StorePortError> {
            Err(unsupported("query Context metadata"))
        }

        fn load_workspace_context_source_body(
            &self,
            source_id: &ObjectId,
        ) -> Result<Option<WorkspaceContextSourceRow>, StorePortError> {
            self.body_loads.set(self.body_loads.get() + 1);
            Ok((source_id == &self.source.source_id).then(|| self.source.clone()))
        }
    }

    impl MemorySkillConsumptionStore for ConsumerTestStore {
        fn list_eligible_memory_pins(
            &self,
            _: &str,
            task_ref: &str,
            purpose: &str,
            _: i64,
        ) -> Result<Vec<EligibleMemoryConsumption>, StorePortError> {
            Ok(self
                .memory_pins
                .iter()
                .cloned()
                .map(|pin| EligibleMemoryConsumption {
                    pin,
                    tenant_id: self
                        .source
                        .governance
                        .tenant_id
                        .clone()
                        .expect("test source has tenant governance"),
                    owner_ref: self.source.governance.owner_ref.clone(),
                    resource_scope: self.source.governance.resource_scope.clone(),
                    target_scope: task_ref.to_owned(),
                    purpose: purpose.to_owned(),
                    source_provenance_ref: self.source.provenance_ref.clone(),
                })
                .collect())
        }

        fn list_eligible_skill_pins(
            &self,
            _: &str,
            _: &str,
        ) -> Result<Vec<SkillConsumptionPin>, StorePortError> {
            Ok(self.skill_pins.clone())
        }

        fn append_memory_skill_consumption(
            &self,
            record: &MemorySkillConsumptionRecord,
        ) -> Result<(), StorePortError> {
            self.append_attempts.set(self.append_attempts.get() + 1);
            if self.append_conflict {
                return Err(StorePortError::Conflict {
                    detail: "competing durable consumption binding".to_owned(),
                });
            }
            self.appended.borrow_mut().push(record.clone());
            Ok(())
        }

        fn load_memory_skill_consumption(
            &self,
            consumption_id: &ObjectId,
        ) -> Result<Option<MemorySkillConsumptionRecord>, StorePortError> {
            Ok((consumption_id == &self.prior.consumption_id).then(|| self.prior.clone()))
        }

        fn load_latest_memory_skill_consumption(
            &self,
            _: &str,
            _: i64,
            _: &ObjectId,
        ) -> Result<Option<MemorySkillConsumptionRecord>, StorePortError> {
            let read = self.latest_reads.get();
            self.latest_reads.set(read + 1);
            Ok((self.initial_prior || read > 0).then(|| self.prior.clone()))
        }

        fn load_skill_revision_payload(
            &self,
            _: &ObjectId,
        ) -> Result<Option<(String, String)>, StorePortError> {
            self.skill_payload_loads
                .set(self.skill_payload_loads.get() + 1);
            Ok(Some(self.skill_payload.clone()))
        }
    }

    #[test]
    fn durable_request_digest_mismatch_fails_before_any_body_access() {
        let command = command("principal://tenant-a/owner");
        let mut store = store_for(&command);
        store.prior.context_request_digest = digest('0');

        let error = load_governed_memory_skill_candidates(
            &store,
            &command,
            1,
            &digest('1'),
            "task_execution",
        )
        .expect_err("a stale request digest must fail closed");

        assert!(
            matches!(
                error,
                SchedulerAuthorityError::ContextAuthorizationUnavailable(ref detail)
                    if detail.contains("request digest")
            ),
            "request-digest drift must have a distinguishable error: {error:?}"
        );
        assert_eq!(store.body_loads.get(), 0);
        assert_eq!(store.skill_payload_loads.get(), 0);
        assert!(store.appended.borrow().is_empty());
    }

    #[test]
    fn durable_record_cannot_cross_an_authenticated_principal() {
        let original = command("principal://tenant-a/owner");
        let store = store_for(&original);
        let crossed = command("principal://tenant-a/other");

        let error = load_governed_memory_skill_candidates(
            &store,
            &crossed,
            1,
            &original_digest(),
            "task_execution",
        )
        .expect_err("a durable record from another principal must fail closed");

        assert!(
            matches!(
                error,
                SchedulerAuthorityError::ContextAuthorizationUnavailable(ref detail)
                    if detail.contains("principal")
            ),
            "cross-principal reuse must have a distinguishable error: {error:?}"
        );
        assert_eq!(
            store.body_loads.get(),
            0,
            "principal filtering must precede Memory body access"
        );
        assert_eq!(store.skill_payload_loads.get(), 0);
        assert!(store.appended.borrow().is_empty());
    }

    #[test]
    fn forged_durable_record_identity_fails_before_replay() {
        let command = command("principal://tenant-a/owner");
        let mut store = store_for(&command);
        store.prior.consumption_id = object_id(99);

        let error = load_governed_memory_skill_candidates(
            &store,
            &command,
            1,
            &original_digest(),
            "task_execution",
        )
        .expect_err("a forged durable identity must fail closed");

        assert!(
            matches!(
                error,
                SchedulerAuthorityError::ContextResolution(ref detail)
                    if detail.contains("identity")
            ),
            "forged durable identity must have a distinguishable error: {error:?}"
        );
        assert_eq!(store.body_loads.get(), 0);
        assert_eq!(store.skill_payload_loads.get(), 0);
        assert!(store.appended.borrow().is_empty());
    }

    #[test]
    fn mismatched_memory_scope_is_rejected_before_body_materialization() {
        let command = command("principal://tenant-a/owner");
        let mut store = store_for(&command);
        store.source.governance.resource_scope = "workspace://tenant-a/other".to_owned();

        let error = load_governed_memory_skill_candidates(
            &store,
            &command,
            1,
            &original_digest(),
            "task_execution",
        )
        .expect_err("a mismatched Memory scope must fail closed");

        assert!(
            matches!(
                error,
                SchedulerAuthorityError::ContextAuthorizationUnavailable(ref detail)
                    if detail.contains("scope")
            ),
            "scope mismatch must have a distinguishable error: {error:?}"
        );
        assert_eq!(
            store.body_loads.get(),
            0,
            "scope filtering must precede Memory body access"
        );
        assert_eq!(store.skill_payload_loads.get(), 0);
        assert!(store.appended.borrow().is_empty());
    }

    #[test]
    fn conflicting_durable_record_is_not_accepted_as_idempotent_replay() {
        let command = command("principal://tenant-a/owner");
        let mut store = store_for(&command);
        store.initial_prior = false;
        store.append_conflict = true;
        store.prior.consumption_id = object_id(98);
        store.prior.session_ref = "conversation://tenant-a/competitor".to_owned();

        let error = load_governed_memory_skill_candidates(
            &store,
            &command,
            1,
            &original_digest(),
            "task_execution",
        )
        .expect_err("a competing durable record must not be reported as idempotent success");

        assert!(
            matches!(
                error,
                SchedulerAuthorityError::ContextResolution(ref detail)
                    if detail.contains("conflict")
            ),
            "a competing append must have a distinguishable conflict: {error:?}"
        );
        assert_eq!(store.append_attempts.get(), 1);
        assert!(store.appended.borrow().is_empty());
    }

    fn store_for(command: &ContextResolutionCommand) -> ConsumerTestStore {
        let memory_pin = MemoryConsumptionPin {
            memory_id: object_id(2),
            source_id: object_id(1),
            source_digest: digest('a'),
        };
        let skill_pin = SkillConsumptionPin {
            binding_id: object_id(5),
            revision_id: object_id(4),
            package_id: object_id(3),
            content_digest: digest('b'),
        };
        let canonical_json = json!({
            "principal_ref": command.authorization_subject_ref,
            "tenant_id": command.tenant_id,
            "resource_scope": command.resource_scope_prefix,
            "purpose": "task_execution",
            "memory": [{
                "memory_id": memory_pin.memory_id.to_string(),
                "source_id": memory_pin.source_id.to_string(),
                "source_digest": memory_pin.source_digest,
            }],
            "skill": [{
                "binding_id": skill_pin.binding_id.to_string(),
                "revision_id": skill_pin.revision_id.to_string(),
                "package_id": skill_pin.package_id.to_string(),
                "content_digest": skill_pin.content_digest,
            }],
        })
        .to_string();
        ConsumerTestStore {
            source: WorkspaceContextSourceRow {
                source_id: memory_pin.source_id.clone(),
                source_digest: memory_pin.source_digest.clone(),
                governance: ObjectGovernance {
                    object_ref: memory_pin.source_id.to_string(),
                    tenant_id: Some(command.tenant_id.clone()),
                    owner_ref: command.authorization_subject_ref.clone(),
                    resource_scope: command.resource_scope_prefix.clone(),
                    conversation_ref: command.conversation_ref.clone(),
                },
                role: LoadedContextItemRole::Working,
                trust_level: LoadedContextItemTrustLevel::Verified,
                representation: LoadedContextItemRepresentation::Text,
                provenance_ref: "file://workspace/memory.txt".to_owned(),
                content_bytes: 16,
                content_tokens: Some(4),
                canonical_json: json!({
                    "header": {"content_digest": memory_pin.source_digest},
                    "body": {"text": "governed memory"},
                })
                .to_string(),
            },
            memory_pins: vec![memory_pin.clone()],
            skill_pins: vec![skill_pin.clone()],
            prior: MemorySkillConsumptionRecord {
                consumption_id: ObjectId::parse("aef81138-8f8d-7000-9000-9aaf056cd908").unwrap(),
                task_ref: command.task_ref.clone(),
                contract_epoch: 1,
                context_request_id: command.request_id.clone(),
                context_request_digest: original_digest(),
                session_ref: command
                    .conversation_ref
                    .clone()
                    .expect("test command has a session"),
                reuse_of: None,
                memory: vec![memory_pin],
                skill: vec![skill_pin.clone()],
                canonical_json,
            },
            initial_prior: true,
            latest_reads: Cell::new(0),
            append_conflict: false,
            append_attempts: Cell::new(0),
            skill_payload: (
                skill_pin.content_digest.clone(),
                json!({
                    "content_digest": skill_pin.content_digest,
                    "instructions": "use only the exact reviewed revision",
                })
                .to_string(),
            ),
            body_loads: Cell::new(0),
            skill_payload_loads: Cell::new(0),
            appended: RefCell::new(Vec::new()),
        }
    }

    fn command(principal: &str) -> ContextResolutionCommand {
        ContextResolutionCommand {
            task_ref: "task://tenant-a/consumer-test".to_owned(),
            request_id: object_id(6),
            authorization_subject_ref: principal.to_owned(),
            tenant_id: "tenant-a".to_owned(),
            resource_scope_prefix: "workspace://tenant-a/project".to_owned(),
            conversation_ref: Some("conversation://tenant-a/one".to_owned()),
            source_limit: 8,
            decided_at: WallTimestamp::parse("2026-08-14T00:00:00Z").unwrap(),
        }
    }

    fn original_digest() -> String {
        digest('c')
    }

    fn digest(fill: char) -> String {
        format!("sha256:{}", fill.to_string().repeat(64))
    }

    fn object_id(serial: u64) -> ObjectId {
        ObjectId::parse(&format!("00000000-0000-7000-9000-{serial:012x}")).unwrap()
    }

    fn unsupported(operation: &str) -> StorePortError {
        StorePortError::Unavailable {
            detail: format!("consumer test store does not support {operation}"),
        }
    }
}
