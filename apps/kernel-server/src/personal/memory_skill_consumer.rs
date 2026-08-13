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
            _: &str,
            _: i64,
        ) -> Result<Vec<MemoryConsumptionPin>, StorePortError> {
            Ok(self.memory_pins.clone())
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
            Ok(Some(self.prior.clone()))
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
        .err()
        .expect("a stale request digest must fail closed");

        assert!(
            matches!(
                error,
                SchedulerAuthorityError::ContextResolution(ref detail)
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
        .err()
        .expect("a durable record from another principal must fail closed");

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
        .err()
        .expect("a forged durable identity must fail closed");

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
        .err()
        .expect("a mismatched Memory scope must fail closed");

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
                consumption_id: object_id(8),
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
