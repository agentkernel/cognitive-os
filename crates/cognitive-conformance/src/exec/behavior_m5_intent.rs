//! M5 Intent Authority behavioral vectors: INTENT-SUPERSEDE-002 and
//! INTENT-ACCEPTANCE-007.
//!
//! Driven against the real `cognitive-kernel` intent-chain / transition
//! surfaces over `cognitive-store::SqliteAuthorityStore` (SQLite WAL).
//! Oracle twins: `cognitive-store/tests/m5_intent_chain.rs`
//! (`user_correction_advances_epoch_and_fences_old_dispatch`) and the
//! acceptance gate discipline already proven by GW-REMOTE-COMPLETE-001 /
//! M4 criterion_4 — this module executes the **vector-specific** shapes
//! and must not claim GW pass as INTENT-ACCEPTANCE-007 pass.
//!
//! Deliberately wrong: admit old-epoch dispatch / skip reconcile; treat
//! agent-completed as authority COMPLETED without verification.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::{AssetContext, ExecError, GateOutput, ImplementationKind};
use crate::LoadedVector;
use cognitive_contracts::generated::common_defs::Budget;
use cognitive_contracts::generated::governed_object_header::GovernedObjectHeaderSensitivity;
use cognitive_contracts::generated::task_contract::ContractConditionKind;
use cognitive_domain::{
    table, EventId, LifecycleDomain, ObjectId, ReasonCode, StateName, UriRef, Version,
    WallTimestamp,
};
use cognitive_kernel::authz::{
    authorize, AccessRequest, ActorChainFacts, AuthorizationGrant, AuthzSnapshot, MembershipFacts,
    ObjectGovernance, PrincipalFacts,
};
use cognitive_kernel::effects::{
    mint_intent, EffectClass, EffectError, EffectProtocol, GovernanceCurrency, IntentCommand,
    OperationDescriptor, WriterLease,
};
use cognitive_kernel::executor::ExecutorCapabilities;
use cognitive_kernel::intent_chain::ConditionSpec;
use cognitive_kernel::ports::{
    AuthorityStore, Clock, EventDraft, IdGenerator, ObjectAdmission, PortFailure, StoredObject,
    TaskBinding,
};
use cognitive_kernel::{
    admit_interpretation, mint_task_contract, record_interpretation_candidate, record_user_intent,
    supersede_task_contract, AcceptanceCommand, AdmitCommand, AmbiguityFact, Causation,
    GovernanceSeed, InterpretationCandidate, PendingWorkDisposition, Reason, SupersedeCommand,
    TablePin, TaskContractCommand, TransitionCommand, TransitionEngine, UserIntentCommand,
};
use cognitive_store::faults::{ScriptedExecutor, ScriptedOutcome};
use cognitive_store::SqliteAuthorityStore;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};

const REFERENCE_IMPLEMENTATION: &str = "cognitive-kernel intent_chain/effects + TransitionEngine \
     over cognitive-store SqliteAuthorityStore (real M5 intent-authority surface)";
const WRONG_IMPLEMENTATION: &str = "intent-authority anti-pattern implementation (deliberately \
     wrong: old-epoch dispatch admitted / agent-completed treated as COMPLETED)";

fn env_err(what: impl Into<String>) -> ExecError {
    ExecError::Environment(what.into())
}

fn implementation_label(kind: ImplementationKind) -> Option<&'static str> {
    Some(match kind {
        ImplementationKind::Reference => REFERENCE_IMPLEMENTATION,
        ImplementationKind::DeliberatelyWrong => WRONG_IMPLEMENTATION,
    })
}

fn registered(ctx: &AssetContext, code: &str) -> Result<Value, ExecError> {
    ctx.registered_error(code)
        .ok_or_else(|| env_err(format!("code {code} not registered")))
}

struct FixedClock(WallTimestamp);

impl FixedClock {
    fn new() -> Result<Self, ExecError> {
        Ok(Self(
            WallTimestamp::parse("2026-07-21T12:00:00Z")
                .map_err(|err| env_err(format!("clock: {err}")))?,
        ))
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Result<WallTimestamp, PortFailure> {
        Ok(self.0.clone())
    }
}

struct SeqIds(AtomicU64);

impl SeqIds {
    fn from(start: u64) -> Self {
        Self(AtomicU64::new(start))
    }
}

impl IdGenerator for SeqIds {
    fn next_uuid_v7(&self) -> Result<String, PortFailure> {
        let n = self.0.fetch_add(1, Ordering::SeqCst);
        Ok(format!("00000000-0000-7000-8000-{n:012x}"))
    }
}

fn ts(text: &str) -> Result<WallTimestamp, ExecError> {
    WallTimestamp::parse(text).map_err(|err| env_err(format!("timestamp: {err}")))
}

fn uri(text: &str) -> Result<UriRef, ExecError> {
    UriRef::parse(text).map_err(|err| env_err(format!("uri `{text}`: {err}")))
}

fn oid(n: u64) -> Result<ObjectId, ExecError> {
    ObjectId::parse(&format!("00000000-0000-7000-b500-{n:012x}"))
        .map_err(|err| env_err(format!("object id: {err}")))
}

fn state_name(text: &str) -> Result<StateName, ExecError> {
    StateName::parse(text).map_err(|err| env_err(format!("state `{text}`: {err}")))
}

fn lease(epoch: i64) -> WriterLease {
    WriterLease { epoch }
}

fn seed() -> Result<GovernanceSeed, ExecError> {
    Ok(GovernanceSeed {
        owner: evidence_ref(9001)?,
        authority: evidence_ref(9002)?,
        resource_scope: evidence_ref(9003)?,
        tenant_id: Some("00000000-0000-7000-9000-0000000000f1".to_owned()),
        created_by: "principal://tenant-a/user-1".to_owned(),
        sensitivity: GovernedObjectHeaderSensitivity::Internal,
        purpose_constraints: vec!["task_execution".to_owned()],
        retention_policy: "standard".to_owned(),
    })
}

fn evidence_ref(
    n: u64,
) -> Result<cognitive_contracts::generated::object_reference::StrongReference, ExecError> {
    Ok(
        cognitive_contracts::generated::object_reference::StrongReference {
            content_digest: cognitive_contracts::generated::common_defs::Digest(format!(
                "sha256:{}",
                format!("{n:x}").repeat(64)[..64].to_owned()
            )),
            id: cognitive_contracts::generated::object_reference::UuidV7(format!(
                "00000000-0000-7000-a000-{n:012x}"
            )),
            kind: cognitive_contracts::generated::object_reference::StrongReferenceKind::Strong,
            object_version: 1,
        },
    )
}

fn user_intent_cmd(record_n: u64, expression: &str) -> Result<UserIntentCommand, ExecError> {
    Ok(UserIntentCommand {
        record_id: oid(record_n)?,
        actor_chain_digest: format!("sha256:{}", "aa11".repeat(16)),
        conversation_or_scope_ref: uri("conversation://tenant-a/thread-1")?,
        input_refs: vec![uri("state://tenant-a/attachments/spec-v1")?],
        raw_expression: expression.to_owned(),
        intent_authority_ref: uri("principal://tenant-a/user-1")?,
        governance: seed()?,
        correlation_id: uri("corr://tenant-a/cfr-m5-intent")?,
    })
}

fn clean_candidate(interp_n: u64) -> Result<InterpretationCandidate, ExecError> {
    Ok(InterpretationCandidate {
        interpretation_id: oid(interp_n)?,
        objectives: vec!["roll out service v2 to staging".to_owned()],
        constraints: vec!["no production changes".to_owned()],
        forbidden: vec!["deleting user data".to_owned()],
        assumptions: vec!["staging cluster is reachable".to_owned()],
        ambiguities: vec![AmbiguityFact {
            id: "amb-cosmetic".to_owned(),
            material: false,
            question: "prefer blue or green deployment naming?".to_owned(),
        }],
        information_gaps: vec![],
        supersedes: None,
    })
}

fn contract_cmd(contract_n: u64, task_ref: &str) -> Result<TaskContractCommand, ExecError> {
    Ok(TaskContractCommand {
        contract_id: oid(contract_n)?,
        task_ref: uri(task_ref)?,
        objective: "staging rollout of service v2".to_owned(),
        in_scope: vec!["staging deployment".to_owned()],
        out_of_scope: vec!["production".to_owned()],
        conditions: vec![ConditionSpec {
            id: "acc-1".to_owned(),
            kind: ContractConditionKind::Acceptance,
            description: "service v2 healthy in staging per verifier".to_owned(),
            verifier_ref: Some("verifier://tenant-a/http-health".to_owned()),
        }],
        budget: Budget {
            attention_slots: None,
            context_bytes: None,
            egress_bytes: None,
            input_tokens: None,
            money_microunits: None,
            output_tokens: None,
            semantic_calls: None,
            tool_calls: Some(50),
            wall_time_ms: None,
        },
        max_iterations: 8,
        max_retries: 3,
        allowed_state_domains: vec!["task".to_owned(), "effect".to_owned()],
        allowed_tools: vec!["operation://tenant-a/payments/refund".to_owned()],
        governance: seed()?,
        correlation_id: uri("corr://tenant-a/cfr-m5-intent")?,
    })
}

fn snapshot(actions: &[&str]) -> Result<AuthzSnapshot, ExecError> {
    Ok(AuthzSnapshot {
        tenant_id: "tenant-a".to_owned(),
        principal: PrincipalFacts {
            principal_ref: uri("principal://tenant-a/agent-1")?,
            authenticated: true,
            active: true,
            tenant_id: Some("tenant-a".to_owned()),
        },
        actor_chain: ActorChainFacts {
            chain_digest: format!("sha256:{}", "aa11".repeat(16)),
            resolved: true,
        },
        membership: Some(MembershipFacts {
            valid: true,
            roles: ["member".to_owned()].into(),
        }),
        capability_links: vec![cognitive_domain::capability::CapabilityConstraints {
            subject: "principal://tenant-a/agent-1".to_owned(),
            audience: "service://tenant-a/payments".to_owned(),
            resource: "scope://tenant-a/payments".to_owned(),
            purpose: "refund_processing".to_owned(),
            actions: actions.iter().map(|a| (*a).to_owned()).collect(),
            parameter_bounds: Default::default(),
            lease: cognitive_domain::capability::LeaseWindow {
                not_before: ts("2026-07-21T11:00:00Z")?,
                expires: ts("2026-07-21T14:00:00Z")?,
            },
            depth_remaining: 1,
            issued_epoch: 41,
        }],
        capability_set_version: 7,
        explicit_denies: vec![],
        revocation_epoch: 41,
        decided_at: ts("2026-07-21T12:00:00Z")?,
    })
}

fn grant_for(action: &str) -> Result<AuthorizationGrant, ExecError> {
    let target = ObjectGovernance {
        object_ref: "effect://tenant-a/refund-17".to_owned(),
        tenant_id: Some("tenant-a".to_owned()),
        owner_ref: "principal://tenant-a/agent-1".to_owned(),
        resource_scope: "scope://tenant-a/payments/refunds".to_owned(),
        conversation_ref: None,
    };
    authorize(
        &snapshot(&[action])?,
        &target,
        &AccessRequest {
            action: action.to_owned(),
            purpose: "refund_processing".to_owned(),
        },
    )
    .map_err(|denied| env_err(format!("harness grant denied: {}", denied.denial.code)))
}

fn currency() -> GovernanceCurrency {
    GovernanceCurrency {
        revocation_epoch: 41,
        capability_set_version: 7,
    }
}

fn descriptor(queryable: bool, idempotent: bool) -> OperationDescriptor {
    OperationDescriptor {
        operation_id: "op://tenant-a/payments/refund".to_owned(),
        action: "payments.refund".to_owned(),
        effect_class: EffectClass::GovernedExternal,
        executor: "executor://tenant-a/payments".to_owned(),
        capabilities: ExecutorCapabilities {
            queryable,
            idempotent,
        },
        descriptor_version: 1,
    }
}

fn protocol<'a>(
    store: &'a SqliteAuthorityStore,
    clock: &'a FixedClock,
    ids: &'a SeqIds,
) -> Result<EffectProtocol<'a, SqliteAuthorityStore, FixedClock, SeqIds>, ExecError> {
    Ok(EffectProtocol::new(
        store,
        clock,
        ids,
        uri("actor://tenant-a/agent-1")?,
        uri("authority://tenant-a/effect-authority")?,
        uri("corr://tenant-a/cfr-m5-intent")?,
    ))
}

fn admit(
    store: &SqliteAuthorityStore,
    clock: &FixedClock,
    ids: &SeqIds,
    object_id: &ObjectId,
    domain: LifecycleDomain,
) -> Result<(), ExecError> {
    let engine = TransitionEngine::new(store, clock, ids);
    engine
        .admit_object(&AdmitCommand {
            object_id: object_id.clone(),
            domain,
            subject_ref: uri(&format!("{}://tenant-a/{}", domain.as_str(), object_id))?,
            body: json!({"conformance_m5_intent": true}),
            actor_ref: uri("actor://tenant-a/agent-1")?,
            authority_ref: uri("authority://tenant-a/state-authority")?,
            correlation_id: uri("corr://tenant-a/cfr-m5-intent")?,
            outbox_destinations: vec![],
            fencing_epoch: None,
        })
        .map_err(|err| env_err(format!("admission rejected: {err}")))?;
    Ok(())
}

fn intent_command(
    intent_n: u64,
    effect_id: &ObjectId,
    key: &str,
    amount: i64,
    desc: OperationDescriptor,
    binding: Option<TaskBinding>,
) -> Result<IntentCommand, ExecError> {
    Ok(IntentCommand {
        intent_id: oid(intent_n)?,
        effect_object_id: effect_id.clone(),
        descriptor: desc,
        target: "https://payments.example/api/refunds".to_owned(),
        parameters: json!({"amount_minor": amount, "currency": "EUR"}),
        idempotency_key: key.to_owned(),
        expected_state_version: Version::INITIAL,
        grant_epoch: 41,
        capability_set_version: 7,
        actor_ref: uri("actor://tenant-a/agent-1")?,
        authority_ref: uri("authority://tenant-a/effect-authority")?,
        correlation_id: uri("corr://tenant-a/cfr-m5-intent")?,
        task_binding: binding,
    })
}

fn denial_code(err: &EffectError) -> Result<(&'static str, &'static str), ExecError> {
    match err {
        EffectError::Denied(denial) => Ok((denial.registered.code, denial.registered.category)),
        other => Err(env_err(format!("expected protocol denial, got {other:?}"))),
    }
}

fn chain_to_contract(
    store: &SqliteAuthorityStore,
    clock: &FixedClock,
    ids: &SeqIds,
    record_n: u64,
    interp_n: u64,
    contract_n: u64,
    task_ref: &str,
) -> Result<cognitive_kernel::ports::TaskContractRow, ExecError> {
    let record = record_user_intent(
        store,
        clock,
        ids,
        &lease(1),
        &user_intent_cmd(record_n, "deploy service v2 to staging")?,
    )
    .map_err(|err| env_err(format!("record_user_intent: {err}")))?;
    let interpretation = record_interpretation_candidate(
        store,
        clock,
        ids,
        &lease(1),
        &record.record_id,
        &clean_candidate(interp_n)?,
        &seed()?,
        &uri("corr://tenant-a/cfr-m5-intent")?,
    )
    .map_err(|err| env_err(format!("record_interpretation_candidate: {err}")))?;
    let admitted = admit_interpretation(
        store,
        &AcceptanceCommand {
            interpretation_id: interpretation.interpretation_id.clone(),
            accepted_by: uri("principal://tenant-a/user-1")?,
            accepted_digest: interpretation.interpretation_digest.clone(),
        },
    )
    .map_err(|err| env_err(format!("admit_interpretation: {err}")))?;
    mint_task_contract(
        store,
        clock,
        ids,
        &lease(1),
        &admitted,
        &contract_cmd(contract_n, task_ref)?,
        0,
    )
    .map_err(|err| env_err(format!("mint_task_contract: {err}")))
}

fn pending_action_label(disposition: PendingWorkDisposition) -> &'static str {
    match disposition {
        PendingWorkDisposition::MustReconcile => "reconcile_before_continue",
        PendingWorkDisposition::SafelyCancelled => "safely_cancelled",
        PendingWorkDisposition::MustComplete => "must_complete",
        PendingWorkDisposition::Compensate => "compensate",
        PendingWorkDisposition::Quarantine => "quarantine",
    }
}

/// INTENT-SUPERSEDE-002 — fence old work after user intent correction.
pub(super) fn intent_supersede_002_behavior(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let _ = registered(ctx, "INTENT_VERSION_SUPERSEDED")?;
    let vector_old = vector
        .input
        .get("old_contract_epoch")
        .and_then(Value::as_i64)
        .unwrap_or(1);
    let vector_new = vector
        .input
        .get("new_contract_epoch")
        .and_then(Value::as_i64)
        .unwrap_or(2);
    let _pending_declared = vector
        .input
        .get("pending_effect")
        .and_then(Value::as_bool)
        .unwrap_or(true);

    if matches!(kind, ImplementationKind::DeliberatelyWrong) {
        return Ok(GateOutput {
            actual: json!({
                "old_epoch_new_dispatch_rejected": false,
                "pending_effect_action": "continue_without_reconcile",
                "error": {"code": "OK", "category": "auth"}
            }),
            grounding: vec![
                "specs/registry/errors.yaml#INTENT_VERSION_SUPERSEDED".into(),
                "crates/cognitive-kernel/src/intent_chain.rs".into(),
            ],
            informative: vec![],
            implementation: implementation_label(kind),
            evidence: json!({
                "anti_pattern": "old-epoch dispatch admitted; pending effect not reconciled",
                "vector_declared_epochs": {"old": vector_old, "new": vector_new},
            }),
        });
    }

    let dir = tempfile::tempdir().map_err(|err| env_err(format!("tempdir: {err}")))?;
    let store = SqliteAuthorityStore::open(&dir.path().join("authority.db"))
        .map_err(|err| env_err(format!("open store: {err}")))?;
    let clock = FixedClock::new()?;
    let ids = SeqIds::from(1);
    let driver = protocol(&store, &clock, &ids)?;
    let grant = grant_for("payments.refund")?;
    let task_ref = "task://tenant-a/cfr-supersede-002";

    let contract_v1 = chain_to_contract(&store, &clock, &ids, 200, 210, 220, task_ref)?;
    assert_env(contract_v1.contract_epoch == 1, "initial epoch must be 1")?;

    let pending_effect = oid(230)?;
    admit(
        &store,
        &clock,
        &ids,
        &pending_effect,
        LifecycleDomain::Effect,
    )?;
    let executor = ScriptedExecutor::queryable(1);
    executor.script(&[ScriptedOutcome::ExecuteThenTimeout]);
    let pending_cmd = intent_command(
        231,
        &pending_effect,
        "cfr-supersede-step-1-attempt-1",
        1000,
        descriptor(true, false),
        Some(TaskBinding {
            task_ref: task_ref.to_owned(),
            contract_epoch: 1,
        }),
    )?;
    mint_intent(&store, &clock, &ids, &lease(1), &pending_cmd)
        .map_err(|err| env_err(format!("mint pending: {err}")))?;
    let authorized = driver
        .authorize_effect(
            &pending_effect,
            Version::INITIAL,
            &grant,
            &currency(),
            &lease(1),
        )
        .map_err(|err| env_err(format!("authorize pending: {err}")))?;
    driver
        .dispatch_effect(
            &pending_effect,
            authorized.after_version,
            &grant,
            &currency(),
            &executor,
            &lease(1),
        )
        .map_err(|err| env_err(format!("dispatch pending: {err}")))?;

    let undispatched_effect = oid(240)?;
    admit(
        &store,
        &clock,
        &ids,
        &undispatched_effect,
        LifecycleDomain::Effect,
    )?;
    let undispatched_cmd = intent_command(
        241,
        &undispatched_effect,
        "cfr-supersede-step-2-attempt-1",
        2000,
        descriptor(true, false),
        Some(TaskBinding {
            task_ref: task_ref.to_owned(),
            contract_epoch: 1,
        }),
    )?;
    mint_intent(&store, &clock, &ids, &lease(1), &undispatched_cmd)
        .map_err(|err| env_err(format!("mint undispatched: {err}")))?;
    let undispatched_authorized = driver
        .authorize_effect(
            &undispatched_effect,
            Version::INITIAL,
            &grant,
            &currency(),
            &lease(1),
        )
        .map_err(|err| env_err(format!("authorize undispatched: {err}")))?;

    let correction_record = record_user_intent(
        &store,
        &clock,
        &ids,
        &lease(1),
        &user_intent_cmd(250, "no - staging EU only")?,
    )
    .map_err(|err| env_err(format!("correction record: {err}")))?;
    let mut correction_candidate = clean_candidate(260)?;
    correction_candidate.objectives = vec!["roll out v2 to staging EU only".to_owned()];
    correction_candidate.supersedes = Some(contract_v1.interpretation_id.clone());
    let superseding = record_interpretation_candidate(
        &store,
        &clock,
        &ids,
        &lease(1),
        &correction_record.record_id,
        &correction_candidate,
        &seed()?,
        &uri("corr://tenant-a/cfr-m5-intent")?,
    )
    .map_err(|err| env_err(format!("superseding interpretation: {err}")))?;
    let mut correction_contract = contract_cmd(270, task_ref)?;
    correction_contract.objective = "staging EU rollout of service v2".to_owned();

    let report = supersede_task_contract(
        &store,
        &clock,
        &ids,
        &lease(1),
        &SupersedeCommand {
            acceptance: AcceptanceCommand {
                interpretation_id: oid(260)?,
                accepted_by: uri("principal://tenant-a/user-1")?,
                accepted_digest: superseding.interpretation_digest.clone(),
            },
            contract: correction_contract,
            expected_current_epoch: 1,
        },
    )
    .map_err(|err| env_err(format!("supersede_task_contract: {err}")))?;

    assert_env(
        report.new_contract.contract_epoch == 2,
        "epoch must advance",
    )?;
    let pending_disp = report
        .pending
        .iter()
        .find(|p| p.effect_object_id == pending_effect)
        .map(|p| p.disposition)
        .ok_or_else(|| env_err("pending EXECUTING effect missing from supersede report"))?;
    let pending_effect_action = pending_action_label(pending_disp);

    let late_effect = oid(280)?;
    admit(&store, &clock, &ids, &late_effect, LifecycleDomain::Effect)?;
    let late_cmd = intent_command(
        281,
        &late_effect,
        "cfr-supersede-step-3-attempt-1",
        3000,
        descriptor(true, false),
        Some(TaskBinding {
            task_ref: task_ref.to_owned(),
            contract_epoch: 1,
        }),
    )?;
    let fenced_mint = mint_intent(&store, &clock, &ids, &lease(1), &late_cmd)
        .expect_err("old-epoch mint must be refused");
    let (mint_code, mint_category) = denial_code(&fenced_mint)?;

    let sink_before = executor.dispatches().len();
    let fenced_dispatch = driver
        .dispatch_effect(
            &undispatched_effect,
            undispatched_authorized.after_version,
            &grant,
            &currency(),
            &executor,
            &lease(1),
        )
        .expect_err("old-epoch dispatch must be fenced");
    let (dispatch_code, _) = denial_code(&fenced_dispatch)?;
    let sink_after = executor.dispatches().len();
    let old_epoch_new_dispatch_rejected = mint_code == "INTENT_VERSION_SUPERSEDED"
        && dispatch_code == "INTENT_VERSION_SUPERSEDED"
        && sink_after == sink_before;

    let error = registered(ctx, mint_code)?;
    assert_env(
        mint_category == "intent",
        "INTENT_VERSION_SUPERSEDED category must be intent",
    )?;

    Ok(GateOutput {
        actual: json!({
            "old_epoch_new_dispatch_rejected": old_epoch_new_dispatch_rejected,
            "pending_effect_action": pending_effect_action,
            "error": error,
        }),
        grounding: vec![
            "crates/cognitive-kernel/src/intent_chain.rs#supersede_task_contract".into(),
            "crates/cognitive-kernel/src/effects.rs#mint_intent/dispatch_effect".into(),
            "crates/cognitive-store (SqliteAuthorityStore, WAL)".into(),
            "specs/registry/errors.yaml#INTENT_VERSION_SUPERSEDED".into(),
            "REQ-INTENT-SUPERSEDE-001".into(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence: json!({
            "vector_declared_epochs": {"old": vector_old, "new": vector_new},
            "observed_epochs": {
                "superseded": report.superseded_epoch,
                "new": report.new_contract.contract_epoch,
            },
            "pending_disposition": format!("{pending_disp:?}"),
            "mint_fence": {"code": mint_code, "category": mint_category},
            "dispatch_fence": {"code": dispatch_code},
            "sink_dispatches_delta": (sink_after as i64) - (sink_before as i64),
            "mode_note": "behavioral twin of m5_intent_chain::user_correction_advances_epoch_and_fences_old_dispatch; epochs 1→2 exercise the same fencing as vector-declared 4→5",
        }),
    })
}

fn assert_env(cond: bool, msg: &str) -> Result<(), ExecError> {
    if cond {
        Ok(())
    } else {
        Err(env_err(msg.to_owned()))
    }
}

// ---------------------------------------------------------------------------
// INTENT-ACCEPTANCE-007
// ---------------------------------------------------------------------------

struct AcceptHarness {
    _dir: tempfile::TempDir,
    store: SqliteAuthorityStore,
    clock: FixedClock,
    ids: SeqIds,
}

impl AcceptHarness {
    fn new() -> Result<Self, ExecError> {
        let dir = tempfile::tempdir().map_err(|err| env_err(format!("tempdir: {err}")))?;
        let store = SqliteAuthorityStore::open(&dir.path().join("authority.db"))
            .map_err(|err| env_err(format!("open store: {err}")))?;
        Ok(Self {
            _dir: dir,
            store,
            clock: FixedClock::new()?,
            ids: SeqIds::from(5000),
        })
    }

    fn engine(&self) -> TransitionEngine<'_, SqliteAuthorityStore, FixedClock, SeqIds> {
        TransitionEngine::new(&self.store, &self.clock, &self.ids)
    }

    /// Seed a task already at `at` through the store port (committed history
    /// model), matching the M2 behavioral harness pattern.
    fn seed(&self, object_id: &ObjectId, at: &str, subject_ref: &str) -> Result<(), ExecError> {
        let admitted_at = WallTimestamp::parse("2026-07-21T11:00:00Z")
            .map_err(|err| env_err(format!("seed timestamp: {err}")))?;
        let raw_event_id = self
            .ids
            .next_uuid_v7()
            .map_err(|err| env_err(format!("seed id: {}", err.detail)))?;
        let event_id = EventId::parse(&raw_event_id)
            .map_err(|err| env_err(format!("seed event id: {err}")))?;
        let state = StateName::parse(at).map_err(|err| env_err(format!("seed state: {err}")))?;
        let event_value = json!({
            "event_id": event_id.as_str(),
            "event_type": "cognitiveos.object.admitted",
            "domain": "task",
            "object_id": object_id.as_str(),
            "subject_ref": subject_ref,
            "after_state": at,
            "after_version": 1,
            "event_time": admitted_at.as_str(),
        });
        let canonical_json = serde_json::to_string(&event_value)
            .map_err(|err| env_err(format!("seed event json: {err}")))?;
        self.store
            .admit_object(&ObjectAdmission {
                object: StoredObject {
                    object_id: object_id.clone(),
                    domain: LifecycleDomain::Task,
                    state,
                    version: Version::INITIAL,
                    body: json!({ "seeded_by": "conformance-m5-intent-accept" }),
                },
                admitted_at,
                event: EventDraft {
                    event_id,
                    object_id: object_id.clone(),
                    domain: LifecycleDomain::Task,
                    object_version: Version::INITIAL,
                    event_type: "cognitiveos.object.admitted".to_owned(),
                    canonical_json,
                },
                outbox: vec![],
                fencing_epoch: None,
            })
            .map_err(|err| env_err(format!("seed admission: {err}")))?;
        Ok(())
    }

    fn load(&self, id: &ObjectId) -> Result<StoredObject, ExecError> {
        self.store
            .load_object(LifecycleDomain::Task, id)
            .map_err(|err| env_err(format!("load: {err}")))?
            .ok_or_else(|| env_err("object missing"))
    }
}

fn acceptance_command(
    object_id: &ObjectId,
    subject_ref: &str,
    from: &str,
    to: &str,
    reason: &str,
    expected_version: Version,
    include_all_guards: bool,
) -> Result<TransitionCommand, ExecError> {
    let loaded =
        table(LifecycleDomain::Task).map_err(|err| env_err(format!("task table: {err}")))?;
    let from_state = state_name(from)?;
    let to_state = state_name(to)?;
    let edge = loaded
        .find_edge(&from_state, &to_state, reason)
        .map_err(|err| env_err(format!("edge {from}->{to}/{reason}: {err:?}")))?;
    let mut guards: BTreeSet<String> = edge.guards.iter().cloned().collect();
    if !include_all_guards {
        guards.remove("verification_passed_and_current");
    }
    let evidence: BTreeMap<
        String,
        cognitive_contracts::generated::object_reference::StrongReference,
    > = edge
        .required_evidence
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let tag = index as u64 + 1;
            let reference = cognitive_contracts::generated::object_reference::StrongReference {
                content_digest: cognitive_contracts::generated::common_defs::Digest(format!(
                    "sha256:{}",
                    format!("{tag:x}").repeat(64)[..64].to_owned()
                )),
                id: cognitive_contracts::generated::object_reference::UuidV7(format!(
                    "00000000-0000-7000-a100-{tag:012x}"
                )),
                kind: cognitive_contracts::generated::object_reference::StrongReferenceKind::Strong,
                object_version: 1,
            };
            (item.clone(), reference)
        })
        .collect();
    Ok(TransitionCommand {
        request_id: uri(&format!(
            "request://conformance/m5-accept/{}/{from}-{to}",
            object_id.as_str()
        ))?,
        domain: LifecycleDomain::Task,
        object_id: object_id.clone(),
        subject_ref: uri(subject_ref)?,
        from: from_state,
        to: to_state,
        expected_version,
        reason: Reason {
            code: ReasonCode::parse(reason).map_err(|err| env_err(format!("reason: {err}")))?,
            detail: None,
        },
        causation: Causation {
            causation_id: uri("cause://conformance/m5-accept")?,
            correlation_id: uri("corr://conformance/m5-accept")?,
        },
        actor_ref: uri("actor://conformance/runner")?,
        authority_ref: uri("authority://conformance/state-authority")?,
        requested_at: WallTimestamp::parse("2026-07-21T12:01:00Z")
            .map_err(|err| env_err(format!("requested_at: {err}")))?,
        table_pin: TablePin::current(LifecycleDomain::Task)
            .map_err(|err| env_err(format!("table pin: {err}")))?,
        established_guards: guards,
        evidence,
        budget: None,
        outbox_destinations: vec![],
        fencing_epoch: None,
    })
}

/// INTENT-ACCEPTANCE-007 — agent-completed without verification/acceptance
/// must not display as completed; next gate remains verification_or_acceptance.
pub(super) fn intent_acceptance_007_behavior(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let _ = registered(ctx, "STATE_CONFLICT")?;
    let agent_reports = vector
        .input
        .get("agent_reports_completed")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let input_state = vector
        .input
        .get("task_state")
        .and_then(Value::as_str)
        .unwrap_or("CANDIDATE_COMPLETE");
    let verification = vector
        .input
        .get("verification")
        .and_then(Value::as_str)
        .unwrap_or("inconclusive");
    let acceptance_decision = vector
        .input
        .get("acceptance_decision")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    assert_env(agent_reports, "vector requires agent_reports_completed")?;
    assert_env(
        input_state == "CANDIDATE_COMPLETE",
        "vector requires CANDIDATE_COMPLETE",
    )?;
    assert_env(
        verification == "inconclusive",
        "vector requires inconclusive",
    )?;
    assert_env(
        !acceptance_decision,
        "vector requires acceptance_decision=false",
    )?;

    if matches!(kind, ImplementationKind::DeliberatelyWrong) {
        return Ok(GateOutput {
            actual: json!({
                "task_state": "COMPLETED",
                "display_completed": true,
                "next_gate": "none"
            }),
            grounding: vec![
                "specs/transitions/task.transitions.json".into(),
                "REQ-INTENT-ACCEPT-001".into(),
            ],
            informative: vec![],
            implementation: implementation_label(kind),
            evidence: json!({
                "anti_pattern": "agent completed report treated as authority COMPLETED without verification/acceptance",
            }),
        });
    }

    let harness = AcceptHarness::new()?;
    let subject = "task://conformance/intent-acceptance-007";
    let task_id = oid(0x707)?;
    harness.seed(&task_id, "CANDIDATE_COMPLETE", subject)?;
    let before = harness.load(&task_id)?;
    assert_env(
        before.state.as_str() == "CANDIDATE_COMPLETE",
        "seed must leave task in CANDIDATE_COMPLETE",
    )?;

    // Agent reports completed + inconclusive verification + no acceptance:
    // attempt ACCEPTANCE_GRANTED without verification_passed_and_current.
    let forced = acceptance_command(
        &task_id,
        subject,
        "CANDIDATE_COMPLETE",
        "COMPLETED",
        "ACCEPTANCE_GRANTED",
        before.version,
        false,
    )?;
    let rejection = harness
        .engine()
        .commit_transition(&forced)
        .expect_err("acceptance without verification must refuse");
    let after = harness.load(&task_id)?;
    let task_state = after.state.as_str();
    let display_completed = false;
    let next_gate = if task_state == "CANDIDATE_COMPLETE" {
        "verification_or_acceptance"
    } else {
        "unexpected"
    };

    Ok(GateOutput {
        actual: json!({
            "task_state": task_state,
            "display_completed": display_completed,
            "next_gate": next_gate,
        }),
        grounding: vec![
            "crates/cognitive-kernel (TransitionEngine centralized gate)".into(),
            "crates/cognitive-store (SqliteAuthorityStore, WAL)".into(),
            "specs/transitions/task.transitions.json#CANDIDATE_COMPLETE→COMPLETED".into(),
            "specs/registry/errors.yaml#STATE_CONFLICT".into(),
            "REQ-INTENT-ACCEPT-001".into(),
            "REQ-SHELL-STATUS-001".into(),
        ],
        informative: vec![],
        implementation: implementation_label(kind),
        evidence: json!({
            "mode_note": "vector-specific: agent_reports_completed stays projection-only; authority remains CANDIDATE_COMPLETE until verification_or_acceptance",
            "rejection_kind": format!("{:?}", rejection.kind),
            "registered_error": rejection.registered().code,
            "input": {
                "agent_reports_completed": agent_reports,
                "verification": verification,
                "acceptance_decision": acceptance_decision,
            },
            "distinct_from": "GW-REMOTE-COMPLETE-001 (different vector id/expected; not claimed as this pass)",
        }),
    })
}
