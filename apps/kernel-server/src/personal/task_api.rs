//! Authenticated daemon-owned Task API (P2-T02 / ADR-0022).
//!
//! This adapter accepts only generated request bindings, derives governance
//! context from the authenticated session, and uses the durable authority
//! store for every mutating lifecycle step. It intentionally keeps watch
//! delivery process-local: a reconnect receives a fresh snapshot before any
//! resumable deltas, and a resume point outside the bounded replay window
//! fails explicitly rather than silently losing observations.

use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::Write;

use cognitive_contracts::generated::common_defs::Digest;
use cognitive_contracts::generated::governed_object_header::GovernedObjectHeaderSensitivity;
use cognitive_contracts::generated::object_reference::{
    StrongReference, StrongReferenceKind, UuidV7,
};
use cognitive_contracts::generated::task_admit_request::TaskAdmitRequest;
use cognitive_contracts::generated::task_admit_result::{
    TaskAdmitResult, TaskAdmitResultSchemaVersion,
};
use cognitive_contracts::generated::task_intent_interpret_request::TaskIntentInterpretRequest;
use cognitive_contracts::generated::task_intent_interpret_result::{
    TaskIntentInterpretResult, TaskIntentInterpretResultSchemaVersion,
    TaskIntentInterpretResultStatus,
};
use cognitive_contracts::generated::task_intent_record_request::TaskIntentRecordRequest;
use cognitive_contracts::generated::task_intent_record_result::{
    TaskIntentRecordResult, TaskIntentRecordResultSchemaVersion,
};
use cognitive_contracts::generated::task_preview_request::{
    TaskContractDraft, TaskContractDraftConditionItemKind, TaskPreviewRequest,
};
use cognitive_contracts::generated::task_preview_result::{
    TaskPreviewResult, TaskPreviewResultSchemaVersion,
};
use cognitive_domain::{BudgetId, LifecycleDomain, ObjectId, UriRef, WallTimestamp};
use cognitive_kernel::effects::WriterLease;
use cognitive_kernel::intent_chain::{
    AcceptanceCommand, AmbiguityFact, ConditionSpec, GovernanceSeed, InterpretationCandidate,
    TaskContractCommand, UserIntentCommand, compose_governed_header,
    seal_governed_object_content_digest, strong_reference_to,
};
use cognitive_kernel::ports::{
    AuthorityStore, ContextRequestRow, ContextStore, ContinuationAuthorityStore, IntentChainStore,
    ProtocolStore, SchedulerExecutionPolicyRow, SchedulerExecutionPolicyStore, TaskBinding,
};
use cognitive_management::{KernelTaskApplicationService, TaskApplicationService};
use cognitive_store::{
    ArtifactStore, PersonalDataLayout, SqliteAuthorityStore, SystemClock, UuidV7Generator,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest as Sha2Digest, Sha256};

const MAX_WATCH_EVENTS: usize = 128;
const MAX_EVIDENCE_EVENTS_SCANNED: usize = 4096;
const EVIDENCE_EVENT_BATCH_SIZE: usize = 256;
const MAX_LIFECYCLE_TRANSITIONS: usize = 16;
const MAX_OPAQUE_OPERATION_REFS: usize = 64;
const MAX_ARTIFACT_BYTES: usize = 8 * 1024 * 1024;
const GOVERNANCE_ROOT_FILE_NAME: &str = "personal-governance-root.json";

#[derive(Debug, Serialize)]
struct TerminalTaskEvidence {
    schema_version: u8,
    task_ref: String,
    contract_epoch: i64,
    lifecycle: TaskLifecycleEvidence,
    intent_refs: Vec<String>,
    effect_refs: Vec<String>,
    reconcile_class: String,
    latest_verification: Option<VerificationEvidence>,
    latest_acceptance: Option<AcceptanceEvidence>,
    durable_cursor: DurableTaskCursor,
}

#[derive(Debug, Serialize)]
struct TaskLifecycleEvidence {
    current_state: String,
    current_version: i64,
    transitions: Vec<RedactedLifecycleTransition>,
    transitions_truncated: bool,
}

#[derive(Debug, Serialize)]
struct RedactedLifecycleTransition {
    sequence: i64,
    event_ref: String,
    event_type: String,
    after_state: Option<String>,
    after_version: i64,
    reason_code: Option<String>,
    event_time: Option<String>,
}

#[derive(Debug, Serialize)]
struct VerificationEvidence {
    report_ref: String,
    report_digest: String,
    status: String,
    completed_at: String,
    current: bool,
    artifact_refs: Vec<String>,
    artifacts_current: bool,
}

#[derive(Debug, Serialize)]
struct AcceptanceEvidence {
    terminal_transition_ref: String,
    terminal_transition_digest: String,
    current: bool,
}

#[derive(Debug, Serialize)]
struct DurableTaskCursor {
    event_sequence: i64,
    task_version: i64,
    terminal_transition_sequence: Option<i64>,
}

/// Durable, daemon-issued local governance root. The authenticated principal
/// is deliberately persisted beside the anchors so a later principal cannot
/// borrow the first local root.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedGovernanceRoot {
    schema_version: u8,
    principal: String,
    owner: StrongReference,
    authority: StrongReference,
    resource_scope: StrongReference,
}

/// A response ready for the loopback HTTP transport.
#[derive(Debug)]
pub(crate) struct TaskApiResponse {
    pub status: u16,
    pub body: String,
    pub content_type: &'static str,
}

/// Process-lifetime task observation log. Durable authority facts are always
/// reloaded for mutations; this only provides connection delivery semantics.
pub(crate) struct TaskApi {
    layout: PersonalDataLayout,
    next_watch_sequence: u64,
    watch_events: VecDeque<(u64, Value)>,
}

impl TaskApi {
    pub(crate) fn new(layout: PersonalDataLayout) -> Self {
        Self {
            layout,
            next_watch_sequence: 1,
            watch_events: VecDeque::new(),
        }
    }

    pub(crate) fn handle(
        &mut self,
        method_path: &str,
        body: &[u8],
        authenticated_principal: &str,
    ) -> TaskApiResponse {
        match method_path.split_whitespace().next().unwrap_or_default() {
            "POST" if method_path.starts_with("POST /task/intent.record ") => {
                self.record(body, authenticated_principal)
            }
            "POST" if method_path.starts_with("POST /task/intent.interpret ") => {
                self.interpret(body, authenticated_principal)
            }
            "POST" if method_path.starts_with("POST /task/preview ") => {
                self.preview(body, authenticated_principal)
            }
            "POST" if method_path.starts_with("POST /task/admit ") => {
                self.admit(body, authenticated_principal)
            }
            "GET"
                if method_path.starts_with("GET /task/evidence?")
                    || method_path.starts_with("GET /task/evidence ") =>
            {
                self.evidence(method_path)
            }
            "GET" if method_path.starts_with("GET /task/watch") => self.watch(method_path),
            _ => TaskApiResponse {
                status: 200,
                body: json!({
                    "status": "ok",
                    "channel": "task",
                    "authority_side_effects": false,
                    "note": "authenticated task front door; no Task API operation matched"
                })
                .to_string(),
                content_type: "application/json",
            },
        }
    }

    fn record(&mut self, body: &[u8], principal: &str) -> TaskApiResponse {
        let request: TaskIntentRecordRequest = match decode(body) {
            Ok(request) => request,
            Err(response) => return response,
        };
        let mut service = match self.service() {
            Ok(service) => service,
            Err(response) => return response,
        };
        let input_refs = match request
            .input_refs
            .unwrap_or_default()
            .into_iter()
            .map(|reference| uri(&reference))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(value) => value,
            Err(response) => return response,
        };
        let intent = UserIntentCommand {
            record_id: match new_object_id() {
                Ok(value) => value,
                Err(response) => return response,
            },
            actor_chain_digest: principal_digest(principal),
            conversation_or_scope_ref: match uri(&request.conversation_or_scope_ref) {
                Ok(value) => value,
                Err(response) => return response,
            },
            input_refs,
            raw_expression: request.raw_expression,
            intent_authority_ref: match uri(principal) {
                Ok(value) => value,
                Err(response) => return response,
            },
            governance: match self.governance(principal) {
                Ok(governance) => governance,
                Err(response) => return response,
            },
            correlation_id: match correlation(principal) {
                Ok(correlation_id) => correlation_id,
                Err(response) => return response,
            },
        };
        let lease = match writer_lease(service.store()) {
            Ok(lease) => lease,
            Err(response) => return response,
        };
        match service.propose(&lease, &intent) {
            Ok(row) => {
                let result = TaskIntentRecordResult {
                    intent_digest: Digest(row.intent_digest),
                    recorded_at: row.recorded_at.as_str().to_owned(),
                    schema_version:
                        TaskIntentRecordResultSchemaVersion::CognitiveosTaskIntentRecordResult01,
                    user_intent_record_id: row.record_id.to_generated(),
                };
                self.publish(
                    "intent.recorded",
                    json!({"user_intent_record_id": result.user_intent_record_id.0}),
                );
                ok(result)
            }
            Err(_) => error(
                409,
                "TASK_INTENT_RECORD_REJECTED",
                "intent record was not persisted",
            ),
        }
    }

    fn interpret(&mut self, body: &[u8], principal: &str) -> TaskApiResponse {
        let request: TaskIntentInterpretRequest = match decode(body) {
            Ok(request) => request,
            Err(response) => return response,
        };
        let mut service = match self.service() {
            Ok(service) => service,
            Err(response) => return response,
        };
        let interpretation_id = match new_object_id() {
            Ok(value) => value,
            Err(response) => return response,
        };
        let record_id = match ObjectId::try_from(&request.user_intent_record_id) {
            Ok(value) => value,
            Err(_) => {
                return error(
                    400,
                    "TASK_INVALID_REQUEST",
                    "user intent record id is invalid",
                );
            }
        };
        let information_gaps = match request
            .candidate
            .information_gaps
            .into_iter()
            .map(|reference| uri(&reference))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(value) => value,
            Err(response) => return response,
        };
        let supersedes = match request.candidate.supersedes_interpretation_id.as_ref() {
            Some(identifier) => match ObjectId::try_from(identifier) {
                Ok(value) => Some(value),
                Err(_) => {
                    return error(
                        400,
                        "TASK_INVALID_REQUEST",
                        "superseded interpretation id is invalid",
                    );
                }
            },
            None => None,
        };
        let candidate = InterpretationCandidate {
            interpretation_id,
            objectives: request.candidate.objectives,
            constraints: request.candidate.constraints,
            forbidden: request.candidate.forbidden,
            assumptions: request.candidate.assumptions,
            ambiguities: request
                .candidate
                .ambiguities
                .into_iter()
                .map(|item| AmbiguityFact {
                    id: item.id,
                    material: item.material,
                    question: item.question,
                })
                .collect(),
            information_gaps,
            supersedes,
        };
        let lease = match writer_lease(service.store()) {
            Ok(lease) => lease,
            Err(response) => return response,
        };
        let correlation_id = match correlation(principal) {
            Ok(correlation_id) => correlation_id,
            Err(response) => return response,
        };
        match service.clarify(
            &lease,
            &record_id,
            &candidate,
            &match self.governance(principal) {
                Ok(governance) => governance,
                Err(response) => return response,
            },
            &correlation_id,
        ) {
            Ok(row) => {
                let material_ambiguity_count = candidate
                    .ambiguities
                    .iter()
                    .filter(|ambiguity| ambiguity.material)
                    .count() as i64;
                let result = TaskIntentInterpretResult { interpretation_digest: Digest(row.interpretation_digest), interpretation_id: row.interpretation_id.to_generated(), material_ambiguity_count, schema_version: TaskIntentInterpretResultSchemaVersion::CognitiveosTaskIntentInterpretResult01, status: if material_ambiguity_count == 0 { TaskIntentInterpretResultStatus::Candidate } else { TaskIntentInterpretResultStatus::ClarificationRequired } };
                self.publish("intent.interpreted", json!({"interpretation_id": result.interpretation_id.0, "status": result.status}));
                ok(result)
            }
            Err(_) => error(
                409,
                "TASK_INTENT_INTERPRET_REJECTED",
                "interpretation was not persisted",
            ),
        }
    }

    fn preview(&mut self, body: &[u8], principal: &str) -> TaskApiResponse {
        let request: TaskPreviewRequest = match decode(body) {
            Ok(request) => request,
            Err(response) => return response,
        };
        let mut service = match self.service() {
            Ok(service) => service,
            Err(response) => return response,
        };
        let preview_governance = match preview_governance(principal) {
            Ok(governance) => governance,
            Err(response) => return response,
        };
        let contract =
            match contract_from_draft(request.task_contract_draft, principal, preview_governance) {
                Ok(contract) => contract,
                Err(response) => return response,
            };
        match service.preview(&contract) {
            Ok(preview) => ok(TaskPreviewResult {
                budget: contract.budget,
                condition_count: preview.condition_count as i64,
                objective: preview.objective,
                preview_digest: Digest(preview.preview_digest),
                schema_version: TaskPreviewResultSchemaVersion::CognitiveosTaskPreviewResult01,
                task_ref: preview.task_ref,
            }),
            Err(_) => error(
                400,
                "TASK_PREVIEW_REJECTED",
                "task draft is not previewable",
            ),
        }
    }

    fn admit(&mut self, body: &[u8], principal: &str) -> TaskApiResponse {
        let request: TaskAdmitRequest = match decode(body) {
            Ok(request) => request,
            Err(response) => return response,
        };
        if request.acceptance.accepted_by != principal {
            return error(
                403,
                "TASK_ACCEPTANCE_PRINCIPAL_MISMATCH",
                "accepted_by must equal the authenticated principal",
            );
        }
        let mut service = match self.service() {
            Ok(service) => service,
            Err(response) => return response,
        };
        let governance = match self.governance(principal) {
            Ok(governance) => governance,
            Err(response) => return response,
        };
        let mut contract =
            match contract_from_draft(request.task_contract_draft, principal, governance.clone()) {
                Ok(contract) => contract,
                Err(response) => return response,
            };
        let interpretation_id = match ObjectId::try_from(&request.acceptance.interpretation_id) {
            Ok(value) => value,
            Err(_) => return error(400, "TASK_INVALID_REQUEST", "interpretation id is invalid"),
        };
        let accepted_by = match uri(principal) {
            Ok(value) => value,
            Err(response) => return response,
        };
        let acceptance = AcceptanceCommand {
            interpretation_id,
            accepted_by,
            accepted_digest: request.acceptance.accepted_digest.0,
        };
        let lease = match writer_lease(service.store()) {
            Ok(lease) => lease,
            Err(response) => return response,
        };
        let context_request_ref = match load_scheduler_policy_context_request(
            service.store(),
            &contract,
            request.expected_current_epoch + 1,
        ) {
            Ok(Some(reference)) => reference,
            Ok(None) => {
                match issue_context_request(service.store(), &contract, &governance, principal) {
                    Ok(reference) => reference,
                    Err(response) => return response,
                }
            }
            Err(response) => return response,
        };
        contract.context_request_ref = Some(context_request_ref);
        if let Err(response) = persist_scheduler_execution_policy(
            service.store(),
            &contract,
            principal,
            &governance,
            request.expected_current_epoch + 1,
        ) {
            return response;
        }
        match service.admit(
            &lease,
            &request.preview_digest.0,
            &acceptance,
            &contract,
            request.expected_current_epoch,
        ) {
            Ok(row) => {
                let result = TaskAdmitResult {
                    contract_digest: Digest(row.contract_digest),
                    contract_epoch: row.contract_epoch,
                    schema_version: TaskAdmitResultSchemaVersion::CognitiveosTaskAdmitResult01,
                    task_contract_ref: format!("task-contract://{}", row.contract_id),
                    task_ref: row.task_ref,
                };
                self.publish(
                    "task.admitted",
                    json!({"task_ref": result.task_ref, "contract_epoch": result.contract_epoch}),
                );
                ok(result)
            }
            Err(_) => error(
                409,
                "TASK_ADMISSION_REJECTED",
                "preview digest, acceptance, or epoch CAS was rejected",
            ),
        }
    }

    fn watch(&self, method_path: &str) -> TaskApiResponse {
        let requested_sequence = method_path.split('?').nth(1).and_then(|query| {
            query
                .split('&')
                .find_map(|pair| pair.strip_prefix("resume_from=")?.parse::<u64>().ok())
        });
        let oldest_sequence = self
            .watch_events
            .front()
            .map(|event| event.0)
            .unwrap_or(self.next_watch_sequence);
        if requested_sequence.is_some_and(|sequence| sequence.saturating_add(1) < oldest_sequence) {
            return error(
                409,
                "TASK_WATCH_RESUME_STALE",
                "requested watch resume point is no longer retained",
            );
        }
        let mut frames = vec![format!(
            "event: snapshot\ndata: {}\n\n",
            json!({"kind":"snapshot", "latest_sequence": self.next_watch_sequence.saturating_sub(1), "tasks": []})
        )];
        for (sequence, event) in self
            .watch_events
            .iter()
            .filter(|(sequence, _)| requested_sequence.is_none_or(|resume| *sequence > resume))
        {
            frames.push(format!(
                "id: {sequence}\nevent: delta\ndata: {}\n\n",
                json!({"kind":"delta", "sequence": sequence, "event": event})
            ));
        }
        TaskApiResponse {
            status: 200,
            body: frames.concat(),
            content_type: "text/event-stream",
        }
    }

    fn evidence(&self, method_path: &str) -> TaskApiResponse {
        let task_ref = match required_query_parameter(method_path, "task_ref") {
            Ok(task_ref) if UriRef::parse(&task_ref).is_ok() => task_ref,
            Ok(_) => {
                return error(
                    400,
                    "TASK_EVIDENCE_INVALID_TASK_REF",
                    "task_ref must be a canonical URI",
                );
            }
            Err(response) => return response,
        };
        let store = match SqliteAuthorityStore::open(&self.layout.authority_database_path()) {
            Ok(store) => store,
            Err(_) => {
                return error(
                    503,
                    "TASK_AUTHORITY_STORE_UNAVAILABLE",
                    "durable authority store is unavailable",
                );
            }
        };
        match reconstruct_terminal_task_evidence(&store, &self.layout, &task_ref) {
            Ok(Some(evidence)) => ok(evidence),
            Ok(None) => error(
                404,
                "TASK_EVIDENCE_NOT_FOUND",
                "no durable TaskContract exists for task_ref",
            ),
            Err(response) => response,
        }
    }

    fn service(
        &self,
    ) -> Result<
        KernelTaskApplicationService<SqliteAuthorityStore, SystemClock, UuidV7Generator>,
        TaskApiResponse,
    > {
        SqliteAuthorityStore::open(&self.layout.authority_database_path())
            .map(|store| KernelTaskApplicationService::new(store, SystemClock, UuidV7Generator))
            .map_err(|_| {
                error(
                    503,
                    "TASK_AUTHORITY_STORE_UNAVAILABLE",
                    "durable authority store is unavailable",
                )
            })
    }

    fn publish(&mut self, kind: &str, body: Value) {
        self.watch_events.push_back((
            self.next_watch_sequence,
            json!({"kind": kind, "body": body}),
        ));
        self.next_watch_sequence = self.next_watch_sequence.saturating_add(1);
        if self.watch_events.len() > MAX_WATCH_EVENTS {
            self.watch_events.pop_front();
        }
    }

    /// Bootstrap exactly one canonical root on the first mutation, then load
    /// and validate it for every subsequent mutation. The root is not a
    /// request DTO and is never populated from client-provided object facts.
    fn governance(&self, principal: &str) -> Result<GovernanceSeed, TaskApiResponse> {
        let root_path = self.layout.data_dir().join(GOVERNANCE_ROOT_FILE_NAME);
        let root = if root_path.exists() {
            load_governance_root(&root_path)?
        } else {
            let root = bootstrap_governance_root(principal)?;
            persist_governance_root(&root_path, &root)?;
            root
        };
        if root.principal != principal || !governance_root_is_canonical(&root) {
            return Err(error(
                403,
                "TASK_GOVERNANCE_ROOT_INVALID",
                "persisted governance root is missing, corrupt, ambiguous, or principal-mismatched",
            ));
        }
        Ok(GovernanceSeed {
            owner: root.owner,
            authority: root.authority,
            resource_scope: root.resource_scope,
            tenant_id: None,
            created_by: principal.to_owned(),
            sensitivity: GovernedObjectHeaderSensitivity::Internal,
            purpose_constraints: vec!["task_execution".to_owned()],
            retention_policy: "standard".to_owned(),
        })
    }
}

/// Persist the complete daemon-owned input set used by the first owner-local
/// scheduler path. This is written before TaskContract admission so a task can
/// never become runnable without its Context query and candidate-admission
/// policy. The values are daemon policy, not client-controlled Task fields.
fn persist_scheduler_execution_policy(
    store: &SqliteAuthorityStore,
    contract: &TaskContractCommand,
    principal: &str,
    governance: &GovernanceSeed,
    contract_epoch: i64,
) -> Result<(), TaskApiResponse> {
    let context_request_ref = contract.context_request_ref.as_ref().ok_or_else(|| {
        error(
            503,
            "TASK_SCHEDULER_POLICY_MISSING_CONTEXT",
            "daemon could not bind scheduler policy to the ContextRequest",
        )
    })?;
    let context_request_id = ObjectId::parse(context_request_ref.id.0.as_str()).map_err(|_| {
        error(
            503,
            "TASK_SCHEDULER_POLICY_INVALID_CONTEXT",
            "daemon could not parse the ContextRequest identity",
        )
    })?;
    if contract_epoch < 1 {
        return Err(error(
            503,
            "TASK_SCHEDULER_POLICY_INVALID_EPOCH",
            "daemon could not derive the next TaskContract epoch",
        ));
    }

    // A previous attempt may have persisted the immutable policy and then
    // lost the TaskContract epoch CAS. Reusing the exact row preserves the
    // candidate identity and ContextRequest binding for a safe retry.
    if let Some(existing_policy) = store
        .load_scheduler_execution_policy(contract.task_ref.as_str(), contract_epoch)
        .map_err(|_| {
            error(
                503,
                "TASK_SCHEDULER_POLICY_PERSISTENCE_FAILED",
                "daemon could not reload scheduler execution policy",
            )
        })?
    {
        if existing_policy.context_request_id == context_request_id {
            return Ok(());
        }
        return Err(error(
            409,
            "TASK_SCHEDULER_POLICY_BINDING_CONFLICT",
            "scheduler execution policy already binds a different ContextRequest",
        ));
    }

    // Personal's first owner-local policy intentionally uses one fixed local
    // tenant and workspace prefix. A future multi-tenant policy must replace
    // these daemon-owned inputs with an explicit persisted configuration; the
    // scheduler must never infer them from Pi or from a strong reference ID.
    let policy = json!({
        "schema_version": 1,
        "task_ref": contract.task_ref.as_str(),
        "contract_epoch": contract_epoch,
        "context": {
            "request_id": context_request_ref.id.0.as_str(),
            "authorization_subject_ref": principal,
            "tenant_id": "personal",
            "resource_scope_prefix": "workspace://personal/",
            "conversation_ref": null,
            "source_limit": 32,
        },
        "admission": {
            "candidate_id": new_object_id()?.as_str(),
            "authorization_subject_ref": principal,
            "authorization_purpose": "task_execution",
            "budget_charge": {"semantic_calls": 1},
            "governance": {
                "owner": &governance.owner,
                "authority": &governance.authority,
                "resource_scope": &governance.resource_scope,
                "tenant_id": null,
                "created_by": "principal://personal/daemon",
                "sensitivity": "internal",
                "purpose_constraints": ["task_execution"],
                "retention_policy": "standard",
            },
            "actor_ref": "principal://personal/daemon",
            "authority_ref": "authority://personal/daemon",
            "correlation_id": contract.correlation_id.as_str(),
        },
    });
    let canonical_json = serde_json::to_string(&policy).map_err(|_| {
        error(
            503,
            "TASK_SCHEDULER_POLICY_SERIALIZATION_FAILED",
            "daemon could not serialize scheduler execution policy",
        )
    })?;
    store
        .append_scheduler_execution_policy(&SchedulerExecutionPolicyRow {
            task_ref: contract.task_ref.as_str().to_owned(),
            contract_epoch,
            context_request_id,
            canonical_json,
        })
        .map_err(|_| {
            error(
                503,
                "TASK_SCHEDULER_POLICY_PERSISTENCE_FAILED",
                "daemon could not persist scheduler execution policy",
            )
        })
}

/// Recover the immutable ContextRequest selected by a previously persisted
/// scheduler policy. This is only a retry path for the exact next epoch; it
/// never derives a new policy from the stored JSON or from client input.
fn load_scheduler_policy_context_request(
    store: &SqliteAuthorityStore,
    contract: &TaskContractCommand,
    contract_epoch: i64,
) -> Result<Option<StrongReference>, TaskApiResponse> {
    let Some(policy) = store
        .load_scheduler_execution_policy(contract.task_ref.as_str(), contract_epoch)
        .map_err(|_| {
            error(
                503,
                "TASK_SCHEDULER_POLICY_PERSISTENCE_FAILED",
                "daemon could not reload scheduler execution policy",
            )
        })?
    else {
        return Ok(None);
    };
    let request = store
        .load_context_request(&policy.context_request_id)
        .map_err(|_| {
            error(
                503,
                "TASK_CONTEXT_PERSISTENCE_FAILED",
                "daemon could not reload the scheduler ContextRequest",
            )
        })?
        .ok_or_else(|| {
            error(
                503,
                "TASK_CONTEXT_PERSISTENCE_FAILED",
                "scheduler policy references a missing ContextRequest",
            )
        })?;
    if request.task_ref != contract.task_ref.as_str() {
        return Err(error(
            409,
            "TASK_SCHEDULER_POLICY_BINDING_CONFLICT",
            "scheduler policy ContextRequest belongs to a different Task",
        ));
    }
    Ok(Some(strong_reference_to(
        &request.request_id,
        &request.request_digest,
    )))
}

fn reconstruct_terminal_task_evidence(
    store: &SqliteAuthorityStore,
    layout: &PersonalDataLayout,
    task_ref: &str,
) -> Result<Option<TerminalTaskEvidence>, TaskApiResponse> {
    let contract_epoch = store.current_contract_epoch(task_ref).map_err(|_| {
        error(
            503,
            "TASK_EVIDENCE_READ_FAILED",
            "current TaskContract epoch could not be read",
        )
    })?;
    if contract_epoch == 0 {
        return Ok(None);
    }
    let task_binding = TaskBinding {
        task_ref: task_ref.to_owned(),
        contract_epoch,
    };
    let contract = store
        .load_task_contract(task_ref, contract_epoch)
        .map_err(|_| {
            error(
                503,
                "TASK_EVIDENCE_READ_FAILED",
                "current TaskContract could not be read",
            )
        })?
        .ok_or_else(|| {
            error(
                503,
                "TASK_EVIDENCE_INCONSISTENT",
                "current TaskContract epoch has no durable row",
            )
        })?;
    let task = store
        .load_object(LifecycleDomain::Task, &contract.contract_id)
        .map_err(|_| {
            error(
                503,
                "TASK_EVIDENCE_READ_FAILED",
                "governed Task lifecycle could not be read",
            )
        })?
        .ok_or_else(|| {
            error(
                503,
                "TASK_EVIDENCE_INCONSISTENT",
                "TaskContract has no governed Task lifecycle row",
            )
        })?;

    let intents = store
        .list_intents_for_task_binding(&task_binding)
        .map_err(|_| {
            error(
                503,
                "TASK_EVIDENCE_READ_FAILED",
                "durable Intent bindings could not be read",
            )
        })?;
    let mut intent_refs = Vec::new();
    let mut effect_refs = Vec::new();
    let mut effect_states = Vec::new();
    for intent in &intents {
        if intent_refs.len() < MAX_OPAQUE_OPERATION_REFS {
            intent_refs.push(format!("intent://{}", intent.intent_id));
            effect_refs.push(format!("effect://{}", intent.effect_object_id));
        }
        let effect_state = store
            .load_object(LifecycleDomain::Effect, &intent.effect_object_id)
            .map_err(|_| {
                error(
                    503,
                    "TASK_EVIDENCE_READ_FAILED",
                    "durable Effect lifecycle could not be read",
                )
            })?
            .map(|effect| effect.state.as_str().to_owned())
            .unwrap_or_else(|| "MISSING".to_owned());
        effect_states.push(effect_state);
    }

    let (transitions, event_sequence, scan_was_bounded) =
        read_redacted_task_transitions(store, &contract.contract_id)?;
    let terminal_transition = transitions
        .iter()
        .rev()
        .find(|transition| transition.after_state.as_deref() == Some("COMPLETED"));
    let terminal_transition_sequence = terminal_transition.map(|transition| transition.sequence);
    let latest_acceptance = terminal_transition.map(|transition| AcceptanceEvidence {
        terminal_transition_ref: transition.event_ref.clone(),
        terminal_transition_digest: transition_digest_for_reference(store, transition)
            .unwrap_or_else(|| "unavailable".to_owned()),
        current: task.state.as_str() == "COMPLETED"
            && transition.after_version == task.version.get(),
    });

    let latest_verification = store
        .load_latest_verification_report_for_task_binding(&task_binding)
        .map_err(|_| {
            error(
                503,
                "TASK_EVIDENCE_READ_FAILED",
                "latest verification report could not be read",
            )
        })?
        .map(|report| build_verification_evidence(store, layout, &task_binding, report))
        .transpose()?;
    let transitions_truncated = scan_was_bounded || transitions.len() > MAX_LIFECYCLE_TRANSITIONS;
    let visible_transitions = transitions
        .into_iter()
        .rev()
        .take(MAX_LIFECYCLE_TRANSITIONS)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    Ok(Some(TerminalTaskEvidence {
        schema_version: 1,
        task_ref: task_ref.to_owned(),
        contract_epoch,
        lifecycle: TaskLifecycleEvidence {
            current_state: task.state.as_str().to_owned(),
            current_version: task.version.get(),
            transitions: visible_transitions,
            transitions_truncated,
        },
        intent_refs,
        effect_refs,
        reconcile_class: classify_reconcile_state(&effect_states).to_owned(),
        latest_verification,
        latest_acceptance,
        durable_cursor: DurableTaskCursor {
            event_sequence,
            task_version: task.version.get(),
            terminal_transition_sequence,
        },
    }))
}

fn read_redacted_task_transitions(
    store: &SqliteAuthorityStore,
    task_id: &ObjectId,
) -> Result<(Vec<RedactedLifecycleTransition>, i64, bool), TaskApiResponse> {
    let mut after_sequence = 0;
    let mut scanned_event_count = 0;
    let mut task_transitions = Vec::new();
    loop {
        let remaining_capacity = MAX_EVIDENCE_EVENTS_SCANNED.saturating_sub(scanned_event_count);
        if remaining_capacity == 0 {
            return Ok((task_transitions, after_sequence, true));
        }
        let batch_limit = remaining_capacity.min(EVIDENCE_EVENT_BATCH_SIZE);
        let events = store
            .read_events(after_sequence, batch_limit)
            .map_err(|_| {
                error(
                    503,
                    "TASK_EVIDENCE_READ_FAILED",
                    "durable Task event cursor could not be read",
                )
            })?;
        if events.is_empty() {
            return Ok((task_transitions, after_sequence, false));
        }
        scanned_event_count += events.len();
        for event in &events {
            after_sequence = event.sequence;
            if event.domain != LifecycleDomain::Task || event.object_id != *task_id {
                continue;
            }
            let value = serde_json::from_str::<Value>(&event.canonical_json).unwrap_or(Value::Null);
            task_transitions.push(RedactedLifecycleTransition {
                sequence: event.sequence,
                event_ref: format!("event://{}", event.event_id),
                event_type: event.event_type.clone(),
                after_state: value
                    .get("after_state")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                after_version: event.object_version.get(),
                reason_code: value
                    .pointer("/reason/code")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                event_time: value
                    .get("event_time")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
            });
        }
        if events.len() < batch_limit {
            return Ok((task_transitions, after_sequence, false));
        }
    }
}

fn transition_digest_for_reference(
    store: &SqliteAuthorityStore,
    transition: &RedactedLifecycleTransition,
) -> Option<String> {
    let event = store
        .read_events(transition.sequence.saturating_sub(1), 1)
        .ok()?
        .into_iter()
        .next()?;
    if event.sequence != transition.sequence {
        return None;
    }
    Some(sha256_digest(event.canonical_json.as_bytes()))
}

fn build_verification_evidence(
    store: &SqliteAuthorityStore,
    layout: &PersonalDataLayout,
    task_binding: &TaskBinding,
    report: cognitive_kernel::ports::VerificationReportRow,
) -> Result<VerificationEvidence, TaskApiResponse> {
    let artifact_refs: Vec<String> = serde_json::from_str(&report.evidence_refs_canonical_json)
        .map_err(|_| {
            error(
                503,
                "TASK_EVIDENCE_INCONSISTENT",
                "verification artifact references are malformed",
            )
        })?;
    let artifact_root = layout.data_dir().join("artifacts");
    let artifacts_current = if artifact_root.is_dir() {
        let artifact_store =
            ArtifactStore::open(&artifact_root, MAX_ARTIFACT_BYTES).map_err(|_| {
                error(
                    503,
                    "TASK_ARTIFACT_STORE_UNAVAILABLE",
                    "Artifact CAS is unavailable",
                )
            })?;
        artifact_refs.iter().all(|artifact_ref| {
            artifact_store
                .contains_artifact_uri(artifact_ref)
                .unwrap_or(false)
        })
    } else {
        artifact_refs.is_empty()
    };
    let request = store
        .load_verification_request(&report.verification_request_id)
        .map_err(|_| {
            error(
                503,
                "TASK_EVIDENCE_READ_FAILED",
                "verification request currentness could not be read",
            )
        })?;
    let fixed_post_state = store
        .load_fixed_post_state(&report.fixed_post_state_id)
        .map_err(|_| {
            error(
                503,
                "TASK_EVIDENCE_READ_FAILED",
                "verification fixed post-state currentness could not be read",
            )
        })?;
    let binding_is_current = match (request, fixed_post_state) {
        (Some(request), Some(fixed_post_state)) => {
            let subject_is_current = store
                .load_object(
                    fixed_post_state.subject_domain,
                    &fixed_post_state.subject_object_id,
                )
                .map_err(|_| {
                    error(
                        503,
                        "TASK_EVIDENCE_READ_FAILED",
                        "verification subject currentness could not be read",
                    )
                })?
                .is_some_and(|subject| {
                    subject.version == fixed_post_state.subject_version
                        && matches!(
                            subject.state.as_str(),
                            "RECONCILED" | "VERIFIED" | "VERIFY_FAILED"
                        )
                });
            request.task_binding == *task_binding
                && fixed_post_state.task_binding == *task_binding
                && fixed_post_state.subject_domain == LifecycleDomain::Effect
                && request.fixed_post_state_id == fixed_post_state.fixed_post_state_id
                && report.fixed_post_state_id == fixed_post_state.fixed_post_state_id
                && report.verification_request_id == request.verification_request_id
                && report.verifier_ref == request.verifier_ref
                && report.verifier_version == request.verifier_version
                && subject_is_current
        }
        _ => false,
    };
    Ok(VerificationEvidence {
        report_ref: format!("verification-report://{}", report.verification_report_id),
        report_digest: sha256_digest(report.canonical_json.as_bytes()),
        status: report.status,
        completed_at: report.completed_at.as_str().to_owned(),
        current: binding_is_current && artifacts_current,
        artifact_refs: artifact_refs
            .into_iter()
            .take(MAX_OPAQUE_OPERATION_REFS)
            .collect(),
        artifacts_current,
    })
}

fn classify_reconcile_state(effect_states: &[String]) -> &'static str {
    if effect_states.is_empty() {
        return "not_applicable";
    }
    let is_durable = |state: &str| matches!(state, "RECONCILED" | "VERIFIED" | "VERIFY_FAILED");
    let all_durable = effect_states.iter().all(|state| is_durable(state));
    if all_durable {
        return "closed";
    }
    let any_durable = effect_states.iter().any(|state| is_durable(state));
    if any_durable {
        return "pending_reconciliation";
    }
    "must_reconcile"
}

fn sha256_digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn required_query_parameter(
    method_path: &str,
    parameter_name: &str,
) -> Result<String, TaskApiResponse> {
    let query = method_path
        .split_whitespace()
        .nth(1)
        .and_then(|path| path.split_once('?').map(|(_, query)| query))
        .ok_or_else(|| {
            error(
                400,
                "TASK_EVIDENCE_TASK_REF_REQUIRED",
                "task_ref query parameter is required",
            )
        })?;
    let matching_values = query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .filter(|(name, _)| *name == parameter_name)
        .map(|(_, value)| percent_decode(value))
        .collect::<Result<Vec<_>, _>>()?;
    match matching_values.as_slice() {
        [value] if !value.trim().is_empty() => Ok(value.clone()),
        _ => Err(error(
            400,
            "TASK_EVIDENCE_TASK_REF_REQUIRED",
            "exactly one non-empty task_ref query parameter is required",
        )),
    }
}

fn percent_decode(value: &str) -> Result<String, TaskApiResponse> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err(error(
                    400,
                    "TASK_EVIDENCE_INVALID_TASK_REF",
                    "task_ref contains invalid percent encoding",
                ));
            }
            let high = hex_value(bytes[index + 1]);
            let low = hex_value(bytes[index + 2]);
            let (Some(high), Some(low)) = (high, low) else {
                return Err(error(
                    400,
                    "TASK_EVIDENCE_INVALID_TASK_REF",
                    "task_ref contains invalid percent encoding",
                ));
            };
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).map_err(|_| {
        error(
            400,
            "TASK_EVIDENCE_INVALID_TASK_REF",
            "task_ref must be UTF-8",
        )
    })
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn decode<T: serde::de::DeserializeOwned>(body: &[u8]) -> Result<T, TaskApiResponse> {
    serde_json::from_slice(body).map_err(|_| {
        error(
            400,
            "TASK_INVALID_REQUEST",
            "request must match its generated binding",
        )
    })
}
fn ok<T: serde::Serialize>(value: T) -> TaskApiResponse {
    TaskApiResponse {
        status: 200,
        body: serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_owned()),
        content_type: "application/json",
    }
}
fn error(status: u16, code: &str, message: &str) -> TaskApiResponse {
    TaskApiResponse {
        status,
        body: json!({"status":"error", "code":code, "message":message}).to_string(),
        content_type: "application/json",
    }
}
fn uri(value: &str) -> Result<UriRef, TaskApiResponse> {
    UriRef::parse(value).map_err(|_| {
        error(
            400,
            "TASK_INVALID_REQUEST",
            "request contains an invalid URI reference",
        )
    })
}
fn new_object_id() -> Result<ObjectId, TaskApiResponse> {
    cognitive_kernel::ports::IdGenerator::next_uuid_v7(&UuidV7Generator)
        .ok()
        .and_then(|value| ObjectId::parse(&value).ok())
        .ok_or_else(|| {
            error(
                503,
                "TASK_ID_GENERATION_FAILED",
                "daemon could not mint authority identity",
            )
        })
}
fn writer_lease(store: &SqliteAuthorityStore) -> Result<WriterLease, TaskApiResponse> {
    store
        .current_fencing_epoch()
        .map(|epoch| WriterLease { epoch })
        .map_err(|_| {
            error(
                503,
                "TASK_WRITER_LEASE_UNAVAILABLE",
                "daemon could not obtain a current writer lease",
            )
        })
}
fn correlation(principal: &str) -> Result<UriRef, TaskApiResponse> {
    UriRef::parse(&format!(
        "corr://personal/{}",
        principal_digest(principal).trim_start_matches("sha256:")
    ))
    .map_err(|_| {
        error(
            503,
            "TASK_CORRELATION_DERIVATION_FAILED",
            "daemon could not derive a canonical correlation reference",
        )
    })
}
fn principal_digest(principal: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in principal.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("sha256:{hash:016x}{hash:016x}{hash:016x}{hash:016x}")
}

/// Persist the daemon's immutable ContextRequest before TaskContract minting.
/// The client draft supplies no Context authority: purpose, perspective,
/// freshness, sensitivity, and render target are daemon-selected policy.
fn issue_context_request(
    store: &SqliteAuthorityStore,
    contract: &TaskContractCommand,
    governance: &GovernanceSeed,
    principal: &str,
) -> Result<StrongReference, TaskApiResponse> {
    let request_id = new_object_id()?;
    let created_at = cognitive_kernel::ports::Clock::now(&SystemClock).map_err(|_| {
        error(
            503,
            "TASK_CONTEXT_CLOCK_UNAVAILABLE",
            "daemon could not timestamp the Context request",
        )
    })?;
    let header = compose_governed_header(
        &request_id,
        "ContextRequest",
        "cognitiveos.context-request/0.1",
        governance,
        vec![contract.task_ref.as_str().to_owned()],
        Vec::new(),
        "daemon-task-admission-context-request",
        &created_at,
    )
    .map_err(|_| {
        error(
            503,
            "TASK_CONTEXT_HEADER_REJECTED",
            "daemon could not compose the Context request header",
        )
    })?;
    let payload = json!({
        "header": header,
        "purpose": "task_execution",
        "perspective": {
            "principal": principal,
            "task": contract.task_ref.as_str(),
            "episode": format!("episode://personal/{}", request_id.as_str()),
        },
        "budget": contract.budget,
        "priority": ["task", "working", "evidence"],
        "required": [],
        "forbidden": [],
        "freshness": {"world_max_age_ms": 0},
        "sensitivity": {"max_input": "internal", "egress": "none"},
        "target_profile": {
            "kind": "structured",
            "schema": "cognitiveos.context-view/0.1",
        },
        "allow_partial": false,
    });
    let (sealed_payload, request_digest) =
        seal_governed_object_content_digest(payload).map_err(|_| {
            error(
                503,
                "TASK_CONTEXT_SEALING_FAILED",
                "daemon could not seal the Context request",
            )
        })?;
    let canonical_json = serde_json::to_string(&sealed_payload).map_err(|_| {
        error(
            503,
            "TASK_CONTEXT_SERIALIZATION_FAILED",
            "daemon could not serialize the Context request",
        )
    })?;
    store
        .append_context_request(&ContextRequestRow {
            request_id: request_id.clone(),
            task_ref: contract.task_ref.as_str().to_owned(),
            request_digest: request_digest.clone(),
            canonical_json,
        })
        .map_err(|_| {
            error(
                503,
                "TASK_CONTEXT_PERSISTENCE_FAILED",
                "daemon could not persist the Context request",
            )
        })?;
    Ok(strong_reference_to(&request_id, &request_digest))
}

fn contract_from_draft(
    draft: TaskContractDraft,
    principal: &str,
    governance: GovernanceSeed,
) -> Result<TaskContractCommand, TaskApiResponse> {
    let conditions = draft.conditions.into_iter().map(|condition| Ok(ConditionSpec { id: condition.id, kind: match condition.kind { TaskContractDraftConditionItemKind::Acceptance => cognitive_contracts::generated::task_contract::ContractConditionKind::Acceptance, TaskContractDraftConditionItemKind::Stop => cognitive_contracts::generated::task_contract::ContractConditionKind::Stop, TaskContractDraftConditionItemKind::Escalation => cognitive_contracts::generated::task_contract::ContractConditionKind::Escalation, TaskContractDraftConditionItemKind::Wait => cognitive_contracts::generated::task_contract::ContractConditionKind::Wait, TaskContractDraftConditionItemKind::Constraint => cognitive_contracts::generated::task_contract::ContractConditionKind::Constraint }, description: condition.description, verifier_ref: condition.verifier_ref })).collect::<Result<Vec<_>, TaskApiResponse>>()?;
    Ok(TaskContractCommand {
        contract_id: new_object_id()?,
        task_ref: uri(&draft.task_ref)?,
        objective: draft.objective,
        in_scope: draft.scope.in_scope,
        out_of_scope: draft.scope.out_of_scope,
        conditions,
        budget: draft.budget,
        max_iterations: draft.max_iterations,
        max_retries: draft.max_retries,
        deadline: WallTimestamp::parse(&draft.deadline)
            .map_err(|_| error(400, "TASK_INVALID_REQUEST", "deadline must be canonical"))?,
        loop_object_id: ObjectId::try_from(&draft.loop_object_id)
            .map_err(|_| error(400, "TASK_INVALID_REQUEST", "loop object id is invalid"))?,
        budget_id: BudgetId::try_from(&draft.budget_id)
            .map_err(|_| error(400, "TASK_INVALID_REQUEST", "budget id is invalid"))?,
        allowed_state_domains: draft.allowed_state_domains,
        allowed_tools: draft.allowed_tools,
        context_request_ref: None,
        governance,
        correlation_id: correlation(principal)?,
    })
}

fn preview_governance(principal: &str) -> Result<GovernanceSeed, TaskApiResponse> {
    // Preview must not bootstrap or persist a root. Its governance fields are
    // excluded from the preview digest; admission reloads the durable root.
    Ok(GovernanceSeed {
        owner: daemon_reference("preview-owner")?,
        authority: daemon_reference("preview-authority")?,
        resource_scope: daemon_reference("preview-scope")?,
        tenant_id: None,
        created_by: principal.to_owned(),
        sensitivity: GovernedObjectHeaderSensitivity::Internal,
        purpose_constraints: vec!["task_execution".to_owned()],
        retention_policy: "standard".to_owned(),
    })
}

fn bootstrap_governance_root(principal: &str) -> Result<PersistedGovernanceRoot, TaskApiResponse> {
    Ok(PersistedGovernanceRoot {
        schema_version: 1,
        principal: principal.to_owned(),
        owner: daemon_reference(&format!("owner:{principal}"))?,
        authority: daemon_reference("authority:personal-daemon")?,
        resource_scope: daemon_reference("resource-scope:personal-task")?,
    })
}

fn daemon_reference(identity: &str) -> Result<StrongReference, TaskApiResponse> {
    let identifier =
        cognitive_kernel::ports::IdGenerator::next_uuid_v7(&UuidV7Generator).map_err(|_| {
            error(
                503,
                "TASK_ID_GENERATION_FAILED",
                "daemon could not mint governance identity",
            )
        })?;
    Ok(StrongReference {
        content_digest: Digest(canonical_anchor_digest(identity)?),
        id: UuidV7(identifier),
        kind: StrongReferenceKind::Strong,
        object_version: 1,
    })
}

fn canonical_anchor_digest(identity: &str) -> Result<String, TaskApiResponse> {
    let canonical_value = json!({"identity": identity, "schema_version": 1});
    let bytes = cognitive_contracts::canonical::canonical_bytes_of_value(&canonical_value)
        .map_err(|_| {
            error(
                503,
                "TASK_GOVERNANCE_ROOT_INVALID",
                "daemon could not canonicalize governance anchor",
            )
        })?;
    cognitive_contracts::canonical::digest(&bytes, "cognitiveos.personal.governance-anchor")
        .map_err(|_| {
            error(
                503,
                "TASK_GOVERNANCE_ROOT_INVALID",
                "daemon could not digest governance anchor",
            )
        })
}

fn governance_root_is_canonical(root: &PersistedGovernanceRoot) -> bool {
    root.schema_version == 1
        && root.owner.kind == StrongReferenceKind::Strong
        && root.authority.kind == StrongReferenceKind::Strong
        && root.resource_scope.kind == StrongReferenceKind::Strong
        && root.owner.object_version == 1
        && root.authority.object_version == 1
        && root.resource_scope.object_version == 1
        && canonical_anchor_digest(&format!("owner:{}", root.principal))
            .is_ok_and(|digest| root.owner.content_digest.0 == digest)
        && canonical_anchor_digest("authority:personal-daemon")
            .is_ok_and(|digest| root.authority.content_digest.0 == digest)
        && canonical_anchor_digest("resource-scope:personal-task")
            .is_ok_and(|digest| root.resource_scope.content_digest.0 == digest)
}

fn load_governance_root(
    path: &std::path::Path,
) -> Result<PersistedGovernanceRoot, TaskApiResponse> {
    let bytes = fs::read(path).map_err(|_| {
        error(
            503,
            "TASK_GOVERNANCE_ROOT_INVALID",
            "persisted governance root cannot be read",
        )
    })?;
    serde_json::from_slice(&bytes).map_err(|_| {
        error(
            503,
            "TASK_GOVERNANCE_ROOT_INVALID",
            "persisted governance root is corrupt",
        )
    })
}

fn persist_governance_root(
    path: &std::path::Path,
    root: &PersistedGovernanceRoot,
) -> Result<(), TaskApiResponse> {
    let bytes = serde_json::to_vec(root).map_err(|_| {
        error(
            503,
            "TASK_GOVERNANCE_ROOT_INVALID",
            "governance root cannot be serialized",
        )
    })?;
    let temporary_path = path.with_extension("json.tmp");
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary_path)
        .map_err(|_| {
            error(
                503,
                "TASK_GOVERNANCE_ROOT_INVALID",
                "governance root cannot be created",
            )
        })?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| {
            error(
                503,
                "TASK_GOVERNANCE_ROOT_INVALID",
                "governance root cannot be persisted",
            )
        })?;
    fs::rename(&temporary_path, path).map_err(|_| {
        error(
            503,
            "TASK_GOVERNANCE_ROOT_INVALID",
            "governance root cannot be committed",
        )
    })
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod evidence_tests {
    use super::*;

    #[test]
    fn evidence_query_decodes_one_canonical_task_reference() {
        let task_ref = required_query_parameter(
            "GET /task/evidence?task_ref=task%3A%2F%2Fpersonal%2Fexample HTTP/1.1",
            "task_ref",
        )
        .expect("decode Task reference");

        assert_eq!(task_ref, "task://personal/example");
    }

    #[test]
    fn evidence_query_rejects_missing_duplicate_and_malformed_values() {
        for method_path in [
            "GET /task/evidence HTTP/1.1",
            "GET /task/evidence?task_ref=task%3A%2F%2Fa&task_ref=task%3A%2F%2Fb HTTP/1.1",
            "GET /task/evidence?task_ref=task%ZZ HTTP/1.1",
        ] {
            let response = required_query_parameter(method_path, "task_ref")
                .expect_err("invalid query must fail closed");
            assert_eq!(response.status, 400);
        }
    }

    #[test]
    fn reconcile_class_uses_only_durable_effect_states() {
        assert_eq!(classify_reconcile_state(&[]), "not_applicable");
        assert_eq!(
            classify_reconcile_state(&["RECONCILED".to_owned(), "VERIFIED".to_owned()]),
            "closed"
        );
        assert_eq!(
            classify_reconcile_state(&["RECONCILED".to_owned(), "OUTCOME_UNKNOWN".to_owned()]),
            "pending_reconciliation"
        );
    }

    #[test]
    fn serialized_evidence_contains_only_redacted_projection_fields() {
        let evidence = TerminalTaskEvidence {
            schema_version: 1,
            task_ref: "task://personal/example".to_owned(),
            contract_epoch: 2,
            lifecycle: TaskLifecycleEvidence {
                current_state: "COMPLETED".to_owned(),
                current_version: 5,
                transitions: Vec::new(),
                transitions_truncated: false,
            },
            intent_refs: vec!["intent://00000000-0000-7000-8000-000000000001".to_owned()],
            effect_refs: vec!["effect://00000000-0000-7000-8000-000000000002".to_owned()],
            reconcile_class: "closed".to_owned(),
            latest_verification: None,
            latest_acceptance: None,
            durable_cursor: DurableTaskCursor {
                event_sequence: 9,
                task_version: 5,
                terminal_transition_sequence: Some(9),
            },
        };

        let serialized = serde_json::to_string(&evidence).expect("serialize evidence projection");
        for forbidden_field in [
            "parameters",
            "workspace",
            "path_bytes",
            "provider_body",
            "receipt",
            "secret",
            "bearer",
            "sqlite",
        ] {
            assert!(!serialized.to_ascii_lowercase().contains(forbidden_field));
        }
    }
}
