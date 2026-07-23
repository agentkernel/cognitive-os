//! Vector execution: M1 static-contract gates plus M2 kernel-behavioral
//! gates.
//!
//! Execution discipline (`docs/standards/conformance-evidence.md` section 2):
//! a vector is reported `pass` only when its stated `input` was actually
//! executed against an implementation and the observable result matched
//! `expected`. Static gates are grounded in registered machine assets
//! (schema files, registries, transition tables) — never in the vector's
//! own `expected` document. The M2 behavioral gates (`behavior` module)
//! execute against the real `cognitive-kernel` transition engine over the
//! `cognitive-store` SQLite WAL adapter. Vectors whose expectations require
//! runtime behavior that does not exist yet are honestly reported `not-run`
//! with a recorded reason.
//!
//! The deliberately wrong implementation (`ImplementationKind::
//! DeliberatelyWrong`) exists for the runner self-check demanded by
//! `docs/standards/conformance-evidence.md` section 3 and DEVELOPMENT-PLAN
//! M1 acceptance 2: its outputs are schema-shaped but behaviorally wrong —
//! static side: bridges legacy shapes, accepts incomplete benefit claims,
//! promotes untrusted content to the control plane; behavioral side (M2): a
//! gate-bypassing direct store writer. The runner MUST fail it;
//! "schema-valid alone is never pass".

use crate::LoadedVector;
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

/// M2 behavioral execution against the real kernel/store authority path.
mod behavior;
/// Ordinary Core AUDIT behavioral execution through the audited public
/// `status.inspect` consumer and durable file adapter.
mod behavior_audit;
/// M3 behavioral execution against the governance/context kernel surface.
mod behavior_m3;
/// M4 behavioral execution: effect protocol and crash recovery through the
/// public fault-injection framework.
mod behavior_m4;
/// M5 behavioral execution: management approval (F-011), shell cancel/detach,
/// watch cursor stale — against RUN public surfaces.
mod behavior_m5;
/// M5 Intent Authority: supersede fencing + acceptance gate (KRN/store).
mod behavior_m5_intent;
/// M6 behavioral execution: install verification, adapter/sandbox bypass,
/// OOB reconciliation — against RUN public surfaces.
mod behavior_m6;

/// Implementation selector for vector execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImplementationKind {
    /// Deterministic reference gates grounded in registered machine assets.
    Reference,
    /// Self-check implementation: schema-valid outputs, wrong behavior.
    DeliberatelyWrong,
}

impl ImplementationKind {
    fn label(self) -> &'static str {
        match self {
            ImplementationKind::Reference => "reference-static-contract-gates",
            ImplementationKind::DeliberatelyWrong => {
                "deliberately-wrong-implementation (self-check)"
            }
        }
    }
}

/// How a vector is executed by the runner (M1 static-contract gates plus
/// the M2 kernel-behavioral gates).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionMode {
    /// Draft 2020-12 validation of `input.object` against the schema named
    /// by `input.validate_against` (wire-schema negatives).
    SchemaGate,
    /// Registry-backed contract traceability: requirement registered, status
    /// and owner_spec match, bidirectional test mapping intact.
    TraceabilityGate,
    /// Whole-registry pairwise coverage (`spec-contract-coverage`).
    CoverageGate,
    /// Performance-report contract gate (REQ-PERF-002/004/005;
    /// `performance-report.schema.json`; `PERFORMANCE_REPORT_INCOMPLETE`).
    PerfContractGate,
    /// M2 behavioral: stale compare-and-swap write against the real kernel
    /// gate over a SQLite WAL authority store (REQ-STATE-003; supersedes
    /// the M1 static CAS comparator).
    CasBehavior,
    /// M2 behavioral: illegal Effect `OUTCOME_UNKNOWN` exit against the
    /// real kernel gate, with still-unknown continuations committed for
    /// real (REQ-EFF-STATE-001; supersedes the M1 static table lookup).
    EffectClosureBehavior,
    /// M2 behavioral: forced remote-completed acceptance against the real
    /// kernel gate over the registered task table (REQ-GW-002,
    /// REQ-INTENT-ACCEPT-001).
    TaskAcceptanceBehavior,
    /// M3 behavioral: same-tenant lateral read against the six-step
    /// authorization gate with denial/not-found isomorphism
    /// (REQ-CTX-002; supersedes nothing — left not-run before M3).
    LateralReadBehavior,
    /// M3 behavioral: capability amplification against the monotone
    /// attenuation arithmetic (REQ-CAP-002).
    AttenuationBehavior,
    /// M3 behavioral: revocation-stale cache reuse against the
    /// governance-bound context cache (REQ-CAP-005, REQ-PROFILE-CVM-001).
    RevocationCacheBehavior,
    /// M3 behavioral: filter-before-rank against the nine-stage pipeline
    /// (REQ-CTX-002).
    RankBeforeAuthBehavior,
    /// M3 behavioral: required-over-budget fail-closed against the
    /// pipeline's budget-fitting stage (REQ-CTX-004, REQ-RES-001).
    RequiredBudgetBehavior,
    /// M3 behavioral: deterministic prefix-stable render (REQ-CTX-012,
    /// IMP-02).
    RenderStabilityBehavior,
    /// M3 behavioral: bounded no-gain resolution stagnation
    /// (REQ-DISC-STAGNATION-001).
    StagnationBehavior,
    /// M3 behavioral: candidate narrowing before execution
    /// (REQ-DISC-ADMIT-001).
    CandidateAdmissionBehavior,
    /// M3 behavioral upgrade of the M1 static trust-plane gate: prompt
    /// injection isolated by the real pipeline and control-plane gates
    /// (REQ-CTX-008, REQ-SEC-002).
    TrustPlaneBehavior,
    /// M4 behavioral: crash after intent persisted, before dispatch —
    /// recover to a single original-key dispatch (REQ-EFF-006).
    CrashPoint1Behavior,
    /// M4 behavioral: crash after external execution, before the receipt —
    /// reconcile, never blind-retry (REQ-EFF-006).
    CrashPoint2Behavior,
    /// M4 behavioral: crash after verification, before commit — commit
    /// from evidence without re-execution (REQ-EFF-006).
    CrashPoint3Behavior,
    /// M4 behavioral: the three crash points aggregated
    /// (REQ-EFF-006, REQ-RUN-006).
    CrashRecoveryBehavior,
    /// M4 behavioral: unknown outcome reconciles under the original key and
    /// quarantines when unresolvable (REQ-EFF-004).
    UnknownOutcomeBehavior,
    /// M4 behavioral: same key + different canonical parameters is refused
    /// with the registered conflict code (REQ-EFF-002).
    IdempotencyConflictBehavior,
    /// M4 behavioral: pending effects are reconciled before any loop
    /// resume (REQ-AGENT-RECOVERY-001).
    RecoveryReconciliationBehavior,
    /// M5 behavioral: R1 approval without structured confirmation (F-011).
    ApprovalR1MissingBehavior,
    /// M5 behavioral: self-authorization / chain entanglement denied (F-011).
    ApprovalSelfBehavior,
    /// M5 behavioral: expired/replayed/mismatched/burst approvals (F-011).
    ApprovalFatigueBehavior,
    /// M5 behavioral: cancel yields CANCEL_PENDING, not authority cancel.
    ShellCancelBehavior,
    /// M5 behavioral: detach does not cancel; reattach restores no privilege.
    ShellDetachBehavior,
    /// M5 behavioral: stale watch cursor forces authorized new snapshot.
    ShellWatchResumeBehavior,
    /// M5 behavioral: task/management channel binding mismatch deny.
    ShellChannelIsolationBehavior,
    /// M5 behavioral: TargetSelector multi-candidate ambiguity deny
    /// (`SHELL_TARGET_AMBIGUOUS`, dispatch=false).
    ShellTargetAmbiguityBehavior,
    /// M5 Intent Authority: user correction supersedes epoch and fences
    /// old-epoch dispatch (REQ-INTENT-SUPERSEDE-001).
    IntentSupersedeBehavior,
    /// M5 Intent Authority: agent-completed without verification/acceptance
    /// does not complete the task (REQ-INTENT-ACCEPT-001).
    IntentAcceptanceBehavior,
    /// M6 behavioral: invalid package signature prevents install commit.
    AgentInstallBehavior,
    /// M6 behavioral: adapter/sandbox/self-completion bypass denied.
    AgentBypassBehavior,
    /// M6 behavioral: OOB digest drift ingests candidate (no silent adopt).
    AgentOobBehavior,
    /// Ordinary Core behavioral AUDIT: audited `status.inspect`, durable
    /// decision readback, formal receipt binding, and release withholding.
    OrdinaryCoreAuditInspectBehavior,
}

/// Execution plan for one vector, decided by structural classification.
#[derive(Debug, Clone)]
pub enum ExecutionPlan {
    Execute(ExecutionMode),
    /// Not statically decidable from registered machine assets.
    NotRun {
        reason: String,
    },
}

/// One field-level difference between `expected` and the observed result.
#[derive(Debug, Clone, Serialize)]
pub struct Mismatch {
    pub path: String,
    pub expected: Value,
    pub actual: Value,
}

/// Evidence record for one executed vector.
#[derive(Debug, Clone, Serialize)]
pub struct ExecutionRecord {
    pub mode: ExecutionMode,
    pub implementation: &'static str,
    /// Registered machine assets this execution was grounded in.
    pub grounding: Vec<String>,
    /// Number of machine-compared leaf fields of `expected`.
    pub compared_fields: usize,
    /// Prose rationale fields recorded as evidence but not machine-compared.
    pub informative_fields: Vec<String>,
    pub mismatches: Vec<Mismatch>,
    /// Observed gate output and auxiliary evidence.
    pub evidence: Value,
}

/// Result of executing (or honestly skipping) one vector.
#[derive(Debug, Clone, Serialize)]
pub struct VectorOutcome {
    pub id: String,
    pub file: String,
    pub layer_slug: String,
    pub profiles: Vec<String>,
    pub requirement_ids: Vec<String>,
    /// One of the five report states.
    pub result: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_run_reason: Option<String>,
    /// Partial contract assertions recorded for plan-named behavioral
    /// vectors that cannot be fully executed yet (M1: static side,
    /// DEVELOPMENT-PLAN M1 acceptance 4; M2: the real read-only degradation
    /// subset). Never a pass.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_contract_assertions: Option<Value>,
}

#[derive(Debug, thiserror::Error)]
pub enum ExecError {
    #[error("i/o error at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("registry {path} failed to parse: {reason}")]
    Registry { path: PathBuf, reason: String },
    #[error("schema {name} failed to compile: {reason}")]
    SchemaCompile { name: String, reason: String },
    #[error("execution environment invalid: {0}")]
    Environment(String),
}

// ---------------------------------------------------------------------------
// Registered machine assets (registries, schemas, transition tables)
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
struct RequirementsFile {
    requirements: Vec<Requirement>,
}

#[derive(Debug, serde::Deserialize)]
struct Requirement {
    id: String,
    owner_spec: String,
    status: String,
    #[serde(default)]
    tests: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ErrorsFile {
    errors: Vec<ErrorEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct ErrorEntry {
    code: String,
    category: String,
    #[serde(default)]
    description: String,
}

#[derive(Debug, serde::Deserialize)]
struct TransitionTable {
    transitions: Vec<TransitionEdge>,
}

#[derive(Debug, serde::Deserialize)]
struct TransitionEdge {
    from: String,
    to: String,
    #[serde(default)]
    guards: Vec<String>,
}

/// Loaded, parsed machine assets shared by all gates.
pub struct AssetContext {
    repo_root: PathBuf,
    requirements: HashMap<String, Requirement>,
    errors: HashMap<String, ErrorEntry>,
    schemas: HashMap<String, Value>,
    effect_transitions: TransitionTable,
}

/// Resolves any retrieval URI to the schema whose file name matches the last
/// path segment ($id == file name policy, `conformance/README.md`).
struct FileNameRetriever {
    schemas: HashMap<String, Value>,
}

impl jsonschema::Retrieve for FileNameRetriever {
    fn retrieve(
        &self,
        uri: &jsonschema::Uri<String>,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let path = uri.path().as_str();
        let file_name = path.rsplit('/').next().unwrap_or(path);
        self.schemas
            .get(file_name)
            .cloned()
            .ok_or_else(|| format!("schema not found for retrieval URI {uri}").into())
    }
}

impl AssetContext {
    pub fn load(repo_root: &Path) -> Result<Self, ExecError> {
        let read = |path: &Path| -> Result<String, ExecError> {
            fs::read_to_string(path).map_err(|source| ExecError::Io {
                path: path.to_path_buf(),
                source,
            })
        };

        let req_path = repo_root
            .join("specs")
            .join("registry")
            .join("requirements.yaml");
        let requirements: RequirementsFile =
            serde_yaml::from_str(&read(&req_path)?).map_err(|err| ExecError::Registry {
                path: req_path.clone(),
                reason: err.to_string(),
            })?;
        let err_path = repo_root.join("specs").join("registry").join("errors.yaml");
        let errors: ErrorsFile =
            serde_yaml::from_str(&read(&err_path)?).map_err(|err| ExecError::Registry {
                path: err_path.clone(),
                reason: err.to_string(),
            })?;

        let schema_dir = repo_root.join("specs").join("schemas");
        let mut schemas = HashMap::new();
        let entries = fs::read_dir(&schema_dir).map_err(|source| ExecError::Io {
            path: schema_dir.clone(),
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| ExecError::Io {
                path: schema_dir.clone(),
                source,
            })?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                let doc: Value =
                    serde_json::from_str(&read(&path)?).map_err(|err| ExecError::Registry {
                        path: path.clone(),
                        reason: err.to_string(),
                    })?;
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                schemas.insert(name, doc);
            }
        }

        let eff_path = repo_root
            .join("specs")
            .join("transitions")
            .join("effect.transitions.json");
        let effect_transitions: TransitionTable =
            serde_json::from_str(&read(&eff_path)?).map_err(|err| ExecError::Registry {
                path: eff_path.clone(),
                reason: err.to_string(),
            })?;

        Ok(Self {
            repo_root: repo_root.to_path_buf(),
            requirements: requirements
                .requirements
                .into_iter()
                .map(|r| (r.id.clone(), r))
                .collect(),
            errors: errors
                .errors
                .into_iter()
                .map(|e| (e.code.clone(), e))
                .collect(),
            schemas,
            effect_transitions,
        })
    }

    fn validator(&self, schema_name: &str) -> Result<jsonschema::Validator, ExecError> {
        let schema = self
            .schemas
            .get(schema_name)
            .ok_or_else(|| ExecError::SchemaCompile {
                name: schema_name.to_owned(),
                reason: "schema not found under specs/schemas/".to_owned(),
            })?;
        jsonschema::options()
            .with_retriever(FileNameRetriever {
                schemas: self.schemas.clone(),
            })
            .should_validate_formats(true)
            .build(schema)
            .map_err(|err| ExecError::SchemaCompile {
                name: schema_name.to_owned(),
                reason: err.to_string(),
            })
    }

    /// Look up a registered error code; the registry stays the single truth
    /// for code/category pairs used in gate outputs.
    fn registered_error(&self, code: &str) -> Option<Value> {
        self.errors
            .get(code)
            .map(|entry| json!({ "code": entry.code, "category": entry.category }))
    }
}

// ---------------------------------------------------------------------------
// Classification
// ---------------------------------------------------------------------------

/// Vector ids of the singleton execution paths. The M2 behavioral trio is
/// executed against the real `cognitive-kernel`/`cognitive-store` authority
/// path (KRN M2 handoff candidate list); pinning by id keeps future vectors
/// defaulting to `not-run` instead of silently acquiring an unsound
/// execution path.
const CAS_VECTOR_ID: &str = "STATE-CAS-002";
const TRANSITION_VECTOR_ID: &str = "EFFECT-STATE-CLOSURE-008";
const TASK_ACCEPTANCE_VECTOR_ID: &str = "GW-REMOTE-COMPLETE-001";
const PERF_VECTOR_ID: &str = "PERF-REPORT-CONTRACT-001";
const TRUST_VECTOR_ID: &str = "CTX-TRUST-004";
const COVERAGE_VECTOR_ID: &str = "SPEC-CONTRACT-COVERAGE-001";
/// M3 governance/context behavioral executions (KRN M3 handoff candidate
/// list; kernel behavioral twins exist for each).
const LATERAL_VECTOR_ID: &str = "GOBJ-TENANT-LATERAL-001";
const ATTENUATION_VECTOR_ID: &str = "CAP-ATTEN-004";
const REVOCATION_CACHE_VECTOR_ID: &str = "CTX-REVOKE-CACHE-001";
const RANK_BEFORE_AUTH_VECTOR_ID: &str = "CTX-RANK-AUTH-001";
const REQUIRED_BUDGET_VECTOR_ID: &str = "CTX-REQ-007";
const RENDER_STABILITY_VECTOR_ID: &str = "CTX-RENDER-001";
const STAGNATION_VECTOR_ID: &str = "DISC-STAGNATION-004";
const CANDIDATE_ADMISSION_VECTOR_ID: &str = "DISC-ADMISSION-002";
/// Delta consumption is an M5 runtime path: no kernel API exists to
/// execute `context-delta-scope` against, so it stays not-run honestly.
const DELTA_SCOPE_VECTOR_ID: &str = "DISC-DELTA-SCOPE-003";
/// M4 effect/recovery behavioral executions (KRN M4 handoff candidate
/// list; the fault-injection framework is public for exactly this reuse).
const CRASH_1_VECTOR_ID: &str = "EFF-CRASH-001";
const CRASH_2_VECTOR_ID: &str = "EFF-CRASH-002";
const CRASH_3_VECTOR_ID: &str = "EFF-CRASH-003";
const CRASH_RECOVERY_VECTOR_ID: &str = "RECOVERY-CRASH-006";
const UNKNOWN_OUTCOME_VECTOR_ID: &str = "EFF-UNK-003";
const IDEMPOTENCY_CONFLICT_VECTOR_ID: &str = "EFF-IDEM-CONFLICT-001";
const RECOVERY_RECONCILIATION_VECTOR_ID: &str = "AGENT-RECOVERY-003";
/// M5 management/shell/watch behavioral executions (RUN M5 public APIs).
const APPROVAL_R1_VECTOR_ID: &str = "MGMT-APPROVAL-R1-009";
const APPROVAL_SELF_VECTOR_ID: &str = "MGMT-APPROVAL-SELF-010";
const APPROVAL_FATIGUE_VECTOR_ID: &str = "MGMT-APPROVAL-FATIGUE-011";
const SHELL_CANCEL_VECTOR_ID: &str = "SHELL-CANCEL-SEMANTICS-005";
const SHELL_DETACH_VECTOR_ID: &str = "SHELL-DETACH-ATTACH-004";
const SHELL_WATCH_VECTOR_ID: &str = "SHELL-WATCH-RESUME-006";
const SHELL_CHANNEL_VECTOR_ID: &str = "SHELL-CHANNEL-ISOLATION-003";
const SHELL_TARGET_AMBIGUITY_VECTOR_ID: &str = "SHELL-TARGET-AMBIGUITY-001";
/// M5 Intent Authority behavioral executions (KRN intent_chain / acceptance).
const INTENT_SUPERSEDE_VECTOR_ID: &str = "INTENT-SUPERSEDE-002";
const INTENT_ACCEPTANCE_VECTOR_ID: &str = "INTENT-ACCEPTANCE-007";
/// M6 install/adapter/OOB behavioral executions (RUN M6 public APIs).
const AGENT_INSTALL_VECTOR_ID: &str = "AGENT-INSTALL-001";
const AGENT_BYPASS_VECTOR_ID: &str = "AGENT-BYPASS-002";
const AGENT_OOB_VECTOR_ID: &str = "AGENT-OOB-001";
/// Ordinary Core audited public-read execution.
const ORDINARY_CORE_AUDIT_INSPECT_VECTOR_ID: &str = "ORDINARY-CORE-AUDIT-INSPECT-001";
/// Behavioral vector that receives recorded partial contract assertions
/// (M1 static side + M2 real read-only degradation subset; never a pass —
/// disk-full and dispatch/stop/revoke expectations are M4/M5 behavior).
const STORE_DEGRADATION_VECTOR_ID: &str = "STATE-STORE-DEGRADE-001";

/// Reason strings for honestly-not-run layers: behavioral execution arrives
/// with the owning subsystem milestone (docs/plan/DEVELOPMENT-PLAN.md).
fn not_run_reason(vector: &LoadedVector) -> String {
    let milestone = match vector.layer_slug.as_str() {
        "effect-recovery" => "kernel Effect/recovery behavior (M4)",
        "state-machine" => "kernel state-machine behavior (M2)",
        // Remaining after the M3 batch: knowledge poisoning isolation.
        "security-negative" => "knowledge-plane isolation behavior (M11)",
        // Remaining after the M3 batch: knowledge invalidation (M11) and
        // embodied observation staleness (M10).
        "context-semantic" => "knowledge/embodied runtime behavior (M10/M11)",
        "shell-intent-lifecycle" => "Shell/intent runtime behavior (M5)",
        "management-shell" => "management session behavior (M5)",
        "harness-loop" => "harness/loop runtime behavior (M5+)",
        "agent-installation" => "installation/adapter/sandbox behavior (M6)",
        "governed-memory" => "governed memory behavior (M7)",
        "cognitive-discovery" => "discovery runtime behavior (M8)",
        "operation-catalog" => "operation catalog runtime behavior (M8)",
        "semantic-mediation" => "semantic mediation runtime behavior (M8)",
        "wire-schema" => "profile runtime behavior (M10)",
        other => {
            return format!(
                "not statically decidable from registered machine assets \
                 (unmapped layer slug `{other}`)"
            );
        }
    };
    format!("not statically decidable from registered machine assets; requires {milestone}")
}

/// Decide how a vector is handled. Structural shapes (schema-validation
/// inputs, traceability inputs) classify generically; the four singleton
/// gates are pinned by vector id so that future vectors default to `not-run`
/// instead of silently acquiring an unsound execution path.
pub fn classify(vector: &LoadedVector) -> ExecutionPlan {
    let input = &vector.input;
    if input.get("validate_against").is_some() && input.get("object").is_some() {
        return ExecutionPlan::Execute(ExecutionMode::SchemaGate);
    }
    if vector.layer_slug == "contract-traceability"
        && input.get("owner_spec").is_some()
        && input.get("requirement_status").is_some()
    {
        return ExecutionPlan::Execute(ExecutionMode::TraceabilityGate);
    }
    match vector.id.as_str() {
        COVERAGE_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::CoverageGate),
        CAS_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::CasBehavior),
        TRANSITION_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::EffectClosureBehavior),
        TASK_ACCEPTANCE_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::TaskAcceptanceBehavior),
        PERF_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::PerfContractGate),
        TRUST_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::TrustPlaneBehavior),
        LATERAL_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::LateralReadBehavior),
        ATTENUATION_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::AttenuationBehavior),
        REVOCATION_CACHE_VECTOR_ID => {
            ExecutionPlan::Execute(ExecutionMode::RevocationCacheBehavior)
        }
        RANK_BEFORE_AUTH_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::RankBeforeAuthBehavior),
        REQUIRED_BUDGET_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::RequiredBudgetBehavior),
        RENDER_STABILITY_VECTOR_ID => {
            ExecutionPlan::Execute(ExecutionMode::RenderStabilityBehavior)
        }
        STAGNATION_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::StagnationBehavior),
        CANDIDATE_ADMISSION_VECTOR_ID => {
            ExecutionPlan::Execute(ExecutionMode::CandidateAdmissionBehavior)
        }
        DELTA_SCOPE_VECTOR_ID => ExecutionPlan::NotRun {
            reason: "delta consumption is an M5 runtime path; no kernel API exists to \
                     execute this scenario against (KRN M3 handoff; honest not-run)"
                .to_owned(),
        },
        CRASH_1_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::CrashPoint1Behavior),
        CRASH_2_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::CrashPoint2Behavior),
        CRASH_3_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::CrashPoint3Behavior),
        CRASH_RECOVERY_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::CrashRecoveryBehavior),
        UNKNOWN_OUTCOME_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::UnknownOutcomeBehavior),
        IDEMPOTENCY_CONFLICT_VECTOR_ID => {
            ExecutionPlan::Execute(ExecutionMode::IdempotencyConflictBehavior)
        }
        RECOVERY_RECONCILIATION_VECTOR_ID => {
            ExecutionPlan::Execute(ExecutionMode::RecoveryReconciliationBehavior)
        }
        APPROVAL_R1_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::ApprovalR1MissingBehavior),
        APPROVAL_SELF_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::ApprovalSelfBehavior),
        APPROVAL_FATIGUE_VECTOR_ID => {
            ExecutionPlan::Execute(ExecutionMode::ApprovalFatigueBehavior)
        }
        SHELL_CANCEL_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::ShellCancelBehavior),
        SHELL_DETACH_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::ShellDetachBehavior),
        SHELL_WATCH_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::ShellWatchResumeBehavior),
        SHELL_CHANNEL_VECTOR_ID => {
            ExecutionPlan::Execute(ExecutionMode::ShellChannelIsolationBehavior)
        }
        SHELL_TARGET_AMBIGUITY_VECTOR_ID => {
            ExecutionPlan::Execute(ExecutionMode::ShellTargetAmbiguityBehavior)
        }
        INTENT_SUPERSEDE_VECTOR_ID => {
            ExecutionPlan::Execute(ExecutionMode::IntentSupersedeBehavior)
        }
        INTENT_ACCEPTANCE_VECTOR_ID => {
            ExecutionPlan::Execute(ExecutionMode::IntentAcceptanceBehavior)
        }
        AGENT_INSTALL_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::AgentInstallBehavior),
        AGENT_BYPASS_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::AgentBypassBehavior),
        AGENT_OOB_VECTOR_ID => ExecutionPlan::Execute(ExecutionMode::AgentOobBehavior),
        ORDINARY_CORE_AUDIT_INSPECT_VECTOR_ID => {
            ExecutionPlan::Execute(ExecutionMode::OrdinaryCoreAuditInspectBehavior)
        }
        STORE_DEGRADATION_VECTOR_ID => ExecutionPlan::NotRun {
            reason: "disk-full fault mode and management stop/revoke expectations are \
                     M4-deferred (no portable disk-full injection) and M5 management plane; \
                     the read-only degradation and fencing subsets are executed for real and \
                     recorded under partial_contract_assertions"
                .to_owned(),
        },
        _ => ExecutionPlan::NotRun {
            reason: not_run_reason(vector),
        },
    }
}

// ---------------------------------------------------------------------------
// Expected-vs-actual comparison
// ---------------------------------------------------------------------------

/// Compare `expected` against the observed `actual` document. Prose
/// rationale fields listed in `informative` (dotted paths from the root of
/// `expected`) are recorded as evidence but not machine-compared: the
/// registered contract fixes codes, decisions and structural outcomes, not
/// human-readable phrasing.
fn compare_expected(
    expected: &Value,
    actual: &Value,
    informative: &[&str],
) -> (usize, Vec<Mismatch>) {
    let mut mismatches = Vec::new();
    let mut compared = 0usize;
    walk_compare(
        expected,
        actual,
        String::new(),
        informative,
        &mut compared,
        &mut mismatches,
    );
    (compared, mismatches)
}

fn walk_compare(
    expected: &Value,
    actual: &Value,
    path: String,
    informative: &[&str],
    compared: &mut usize,
    mismatches: &mut Vec<Mismatch>,
) {
    if informative.contains(&path.as_str()) {
        return;
    }
    match expected {
        Value::Object(map) => {
            for (key, exp_child) in map {
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                let act_child = actual.get(key).unwrap_or(&Value::Null);
                walk_compare(
                    exp_child,
                    act_child,
                    child_path,
                    informative,
                    compared,
                    mismatches,
                );
            }
        }
        Value::Array(items) => {
            let actual_items = actual.as_array().cloned().unwrap_or_default();
            if items.len() != actual_items.len() {
                *compared += 1;
                mismatches.push(Mismatch {
                    path: format!("{path}.length"),
                    expected: json!(items.len()),
                    actual: json!(actual_items.len()),
                });
                return;
            }
            for (index, exp_child) in items.iter().enumerate() {
                let act_child = actual_items.get(index).cloned().unwrap_or(Value::Null);
                walk_compare(
                    exp_child,
                    &act_child,
                    format!("{path}[{index}]"),
                    informative,
                    compared,
                    mismatches,
                );
            }
        }
        leaf => {
            *compared += 1;
            if leaf != actual {
                mismatches.push(Mismatch {
                    path,
                    expected: leaf.clone(),
                    actual: actual.clone(),
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Gates
// ---------------------------------------------------------------------------

struct GateOutput {
    actual: Value,
    grounding: Vec<String>,
    informative: Vec<&'static str>,
    /// Overrides the implementation label of the execution record (used by
    /// the behavioral gates, whose implementation under test is the real
    /// kernel/store path, not the runner's static reference gates).
    implementation: Option<&'static str>,
    evidence: Value,
}

fn schema_gate(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let schema_rel = vector
        .input
        .get("validate_against")
        .and_then(Value::as_str)
        .ok_or_else(|| ExecError::Environment("missing input.validate_against".to_owned()))?;
    let schema_name = schema_rel.rsplit('/').next().unwrap_or(schema_rel);
    let object = vector.input.get("object").cloned().unwrap_or(Value::Null);
    let validator = ctx.validator(schema_name)?;

    let truly_valid = validator.is_valid(&object);
    let errors: Vec<String> = validator
        .iter_errors(&object)
        .take(8)
        .map(|e| format!("{}: {}", e.instance_path(), e))
        .collect();

    // The wrong implementation silently bridges the legacy shape: it reports
    // the object as valid regardless of the registered single-track contract.
    let reported_valid = match kind {
        ImplementationKind::Reference => truly_valid,
        ImplementationKind::DeliberatelyWrong => true,
    };
    let bridged = matches!(kind, ImplementationKind::DeliberatelyWrong) && !truly_valid;

    let error = if reported_valid {
        Value::Null
    } else {
        ctx.registered_error("SCHEMA_MISMATCH")
            .ok_or_else(|| ExecError::Environment("SCHEMA_MISMATCH not registered".to_owned()))?
    };

    // Structural derivations from the registered schemas themselves.
    let header_required = ctx
        .schemas
        .get(schema_name)
        .and_then(|s| s.get("required"))
        .and_then(Value::as_array)
        .is_some_and(|req| req.iter().any(|v| v == "header"));
    let strong_ref = ctx
        .schemas
        .get("object-reference.schema.json")
        .and_then(|s| s.pointer("/$defs/strongReference"));
    let strong_ref_fixed = strong_ref.is_some_and(|def| {
        let required: BTreeSet<&str> = def
            .get("required")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        let closed = def.get("additionalProperties") == Some(&Value::Bool(false));
        required == BTreeSet::from(["kind", "id", "object_version", "content_digest"]) && closed
    });

    let mut actual = json!({
        "schema_valid": reported_valid,
        "decision": if reported_valid { "allow" } else { "deny" },
        "error": error,
    });
    // Mirror the vector-specific derived assertions.
    if vector
        .expected
        .get("governed_object_header_required")
        .is_some()
    {
        actual["governed_object_header_required"] = json!(match kind {
            ImplementationKind::Reference => header_required,
            ImplementationKind::DeliberatelyWrong => false,
        });
    }
    if vector
        .expected
        .get("strong_reference_contract_required")
        .is_some()
    {
        actual["strong_reference_contract_required"] = json!(match kind {
            ImplementationKind::Reference => strong_ref_fixed,
            ImplementationKind::DeliberatelyWrong => false,
        });
    }
    if vector
        .expected
        .get("legacy_envelope_bridged_silently")
        .is_some()
    {
        actual["legacy_envelope_bridged_silently"] = json!(bridged);
    }
    if vector
        .expected
        .get("legacy_reference_bridged_silently")
        .is_some()
    {
        actual["legacy_reference_bridged_silently"] = json!(bridged);
    }

    Ok(GateOutput {
        actual,
        grounding: vec![
            format!("specs/schemas/{schema_name}"),
            "specs/schemas/object-reference.schema.json#/$defs/strongReference".to_owned(),
            "specs/registry/errors.yaml#SCHEMA_MISMATCH".to_owned(),
        ],
        informative: vec!["rejection_reasons"],
        implementation: None,
        evidence: json!({
            "validator": "jsonschema draft 2020-12, relative $refs from containing file",
            "schema_validation_errors": errors,
            "rejection_reasons_recorded_not_compared": vector.expected.get("rejection_reasons"),
        }),
    })
}

fn traceability_gate(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let input_owner = vector
        .input
        .get("owner_spec")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let input_status = vector
        .input
        .get("requirement_status")
        .and_then(Value::as_str)
        .unwrap_or_default();

    let mut checks: Vec<Value> = Vec::new();
    let mut all_ok = true;
    let mut owner_schema_required: Option<Value> = None;
    for req_id in &vector.requirement_ids {
        match ctx.requirements.get(req_id) {
            None => {
                all_ok = false;
                checks.push(json!({ "requirement": req_id, "registered": false }));
            }
            Some(req) => {
                let status_ok = req.status == input_status;
                let owner_ok = req.owner_spec == input_owner;
                let owner_file = req.owner_spec.split('#').next().unwrap_or("");
                let owner_exists = ctx.repo_root.join(owner_file).exists();
                let mapped_back = req.tests.iter().any(|t| t == &vector.id);
                all_ok = all_ok && status_ok && owner_ok && owner_exists && mapped_back;
                // For schema-owned requirements, record the compiled
                // contract's required members as citable static evidence
                // (used by the findings-ledger M1 re-verification entries).
                if owner_file.starts_with("specs/schemas/") {
                    let name = owner_file.rsplit('/').next().unwrap_or(owner_file);
                    let compiles = ctx.validator(name).is_ok();
                    all_ok = all_ok && compiles;
                    owner_schema_required = Some(json!({
                        "schema": name,
                        "compiles": compiles,
                        "required": ctx
                            .schemas
                            .get(name)
                            .and_then(|s| s.get("required"))
                            .cloned()
                            .unwrap_or(Value::Null),
                    }));
                }
                checks.push(json!({
                    "requirement": req_id,
                    "registered": true,
                    "status_matches_input": status_ok,
                    "owner_spec_matches_input": owner_ok,
                    "owner_spec_file_exists": owner_exists,
                    "registry_maps_test_back_to_vector": mapped_back,
                }));
            }
        }
    }

    // The wrong implementation is intentionally NOT wired into this gate:
    // the expected booleans are runner-discipline constants, so a lazy
    // implementation that skips the registry lookups is not observable
    // through expected-comparison. That honesty gap is closed by the report
    // invariants (every pass carries an execution record) and the CI
    // assertions, and is recorded in the self-check report.
    let enforced = match kind {
        ImplementationKind::Reference | ImplementationKind::DeliberatelyWrong => all_ok,
    };

    Ok(GateOutput {
        actual: json!({
            // True only when every registry linkage check passed.
            "requirement_semantics_enforced": enforced,
            // Runner discipline constants, enforced by construction: pass
            // requires an execution record with evidence, and parsing or
            // schema-validating a vector file never sets pass.
            "evidence_required_for_pass": true,
            "schema_parse_alone_is_not_pass": true,
        }),
        grounding: vec![
            "specs/registry/requirements.yaml".to_owned(),
            input_owner.to_owned(),
        ],
        informative: vec![],
        implementation: None,
        evidence: json!({
            "registry_checks": checks,
            "owner_schema": owner_schema_required,
            "scope": "contract-traceability layer only; no behavioral claim for the mapped requirements",
        }),
    })
}

fn coverage_gate(
    ctx: &AssetContext,
    vector: &LoadedVector,
    _kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let docs: Vec<&str> = vector
        .input
        .get("normative_documents")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let ids = &vector.requirement_ids;

    let mut unregistered: Vec<String> = Vec::new();
    let mut owner_mismatches: Vec<String> = Vec::new();
    let mut missing_files: BTreeSet<String> = BTreeSet::new();
    let length_match = ids.len() == docs.len();
    for (index, req_id) in ids.iter().enumerate() {
        match ctx.requirements.get(req_id) {
            None => unregistered.push(req_id.clone()),
            Some(req) => {
                let owner_file = req.owner_spec.split('#').next().unwrap_or("").to_owned();
                if Some(owner_file.as_str()) != docs.get(index).copied() {
                    owner_mismatches.push(req_id.clone());
                }
                if !ctx.repo_root.join(&owner_file).exists() {
                    missing_files.insert(owner_file);
                }
            }
        }
    }

    Ok(GateOutput {
        actual: json!({
            "every_requirement_registered": length_match && unregistered.is_empty(),
            "owner_spec_resolves": owner_mismatches.is_empty() && missing_files.is_empty(),
            // Discipline constant: the runner derives no implementation or
            // execution claim from the registry `status` field; `specified`
            // feeds enumeration only.
            "status_does_not_imply_implementation": true,
        }),
        grounding: vec!["specs/registry/requirements.yaml".to_owned()],
        informative: vec![],
        implementation: None,
        evidence: json!({
            "pairs_checked": ids.len(),
            "unregistered": unregistered,
            "owner_spec_mismatches": owner_mismatches,
            "owner_spec_files_missing": missing_files,
        }),
    })
}

fn perf_contract_gate(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    let schema_name = "performance-report.schema.json";
    let schema = ctx
        .schemas
        .get(schema_name)
        .cloned()
        .ok_or_else(|| ExecError::Environment(format!("{schema_name} missing")))?;
    let validator = ctx.validator(schema_name)?;
    let report = vector.input.get("report").cloned().unwrap_or(Value::Null);
    let fragment = vector
        .input
        .pointer("/negative_case/report_fragment")
        .cloned()
        .unwrap_or(Value::Null);

    let report_valid = validator.is_valid(&report);
    let fragment_valid = validator.is_valid(&fragment);

    let required_top: Vec<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let metrics_required: Vec<&str> = schema
        .pointer("/properties/metrics/items/required")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let comparison_required: Vec<&str> = schema
        .pointer("/properties/comparison/required")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();

    let arm_kinds = |doc: &Value| -> BTreeSet<String> {
        doc.pointer("/comparison/arms")
            .and_then(Value::as_array)
            .map(|arms| {
                arms.iter()
                    .filter_map(|arm| arm.get("arm_kind").and_then(Value::as_str))
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    };
    let report_arms = arm_kinds(&report);
    let fragment_arms = arm_kinds(&fragment);
    let fragment_prereg = fragment
        .pointer("/comparison/preregistration_ref")
        .is_some();

    // Negative fragment: reject unless it is schema-valid AND carries both
    // required comparison arms AND a preregistration reference.
    let fragment_complete = fragment_valid
        && fragment_arms.contains("native_baseline")
        && fragment_arms.contains("governance_only")
        && fragment_prereg;
    let accept_negative = match kind {
        ImplementationKind::Reference => fragment_complete,
        // The wrong implementation happily accepts the benefit claim.
        ImplementationKind::DeliberatelyWrong => true,
    };
    let negative_case_result = if accept_negative {
        json!({
            "decision": "accept",
            "error": Value::Null,
            "benefit_claim_accepted": true,
        })
    } else {
        json!({
            "decision": "reject",
            "error": ctx.registered_error("PERFORMANCE_REPORT_INCOMPLETE").ok_or_else(|| {
                ExecError::Environment("PERFORMANCE_REPORT_INCOMPLETE not registered".to_owned())
            })?,
            "benefit_claim_accepted": false,
        })
    };

    let metrics_have_tails =
        report
            .get("metrics")
            .and_then(Value::as_array)
            .is_some_and(|metrics| {
                !metrics.is_empty()
                    && metrics.iter().all(|m| {
                        m.get("p50").is_some() && m.get("p95").is_some() && m.get("p99").is_some()
                    })
            });

    let actual = json!({
        "schema_valid": match kind {
            ImplementationKind::Reference => report_valid,
            ImplementationKind::DeliberatelyWrong => true,
        },
        // The registered schema does not require a universal composite score
        // (top-level `required` has no `composite_score`), REQ-PERF-002.
        "universal_composite_required": required_top.contains(&"composite_score"),
        "tail_percentiles_preserved":
            metrics_required.contains(&"p50")
                && metrics_required.contains(&"p95")
                && metrics_required.contains(&"p99")
                && metrics_have_tails,
        "mechanism_latency_separated": report
            .pointer("/benchmark_manifest/latency_boundaries/mechanism")
            .is_some()
            && report
                .pointer("/benchmark_manifest/latency_boundaries/model_tool_network")
                .is_some(),
        "governance_overhead_reported": report.get("governance_overhead").is_some(),
        "ungoverned_baseline_declared": report
            .pointer("/governance_overhead/ungoverned_baseline")
            .is_some(),
        "comparison_arms_include_native_and_governance_only": report_arms
            .contains("native_baseline")
            && report_arms.contains("governance_only"),
        "claim_level_bound_to_preregistered_thresholds": comparison_required
            .contains(&"claim_level")
            && comparison_required.contains(&"preregistration_ref")
            && report.pointer("/comparison/preregistration_ref").is_some(),
        "negative_case_result": negative_case_result,
    });

    Ok(GateOutput {
        actual,
        grounding: vec![
            format!("specs/schemas/{schema_name}"),
            "specs/registry/errors.yaml#PERFORMANCE_REPORT_INCOMPLETE".to_owned(),
        ],
        informative: vec!["negative_case_result.reason"],
        implementation: None,
        evidence: json!({
            "report_schema_valid": report_valid,
            "fragment_schema_valid": fragment_valid,
            "fragment_arm_kinds": fragment_arms,
            "fragment_has_preregistration_ref": fragment_prereg,
            "reason_recorded_not_compared": vector
                .expected
                .pointer("/negative_case_result/reason"),
        }),
    })
}

/// Static contract-side assertions for the plan-named behavioral vector
/// `state-store-degradation` (F-008). Recorded as evidence only; the vector
/// result stays `not-run` until the M2/M4 behavioral execution.
fn store_degradation_assertions(ctx: &AssetContext) -> Value {
    let entry = ctx.errors.get("STATE_STORE_UNAVAILABLE");
    let dispatch_guard = ctx.effect_transitions.transitions.iter().any(|edge| {
        edge.from == "AUTHORIZED"
            && edge.to == "EXECUTING"
            && edge.guards.iter().any(|g| g == "intent_durably_persisted")
    });
    json!({
        "scope": "static contract side only (DEVELOPMENT-PLAN M1 acceptance 4); behavioral fail-closed execution is M2/M4 evidence",
        "error_registered": entry.map(|e| json!({
            "code": e.code,
            "category": e.category,
            "fail_closed_description": e.description.contains("fail closed"),
        })),
        "dispatch_requires_durable_intent_guard_in_transition_table": dispatch_guard,
    })
}

// ---------------------------------------------------------------------------
// Vector execution driver
// ---------------------------------------------------------------------------

fn execute_gate(
    ctx: &AssetContext,
    vector: &LoadedVector,
    mode: ExecutionMode,
    kind: ImplementationKind,
) -> Result<GateOutput, ExecError> {
    match mode {
        ExecutionMode::SchemaGate => schema_gate(ctx, vector, kind),
        ExecutionMode::TraceabilityGate => traceability_gate(ctx, vector, kind),
        ExecutionMode::CoverageGate => coverage_gate(ctx, vector, kind),
        ExecutionMode::PerfContractGate => perf_contract_gate(ctx, vector, kind),
        ExecutionMode::CasBehavior => behavior::cas_behavior(ctx, vector, kind),
        ExecutionMode::EffectClosureBehavior => {
            behavior::effect_closure_behavior(ctx, vector, kind)
        }
        ExecutionMode::TaskAcceptanceBehavior => {
            behavior::task_acceptance_behavior(ctx, vector, kind)
        }
        ExecutionMode::LateralReadBehavior => behavior_m3::lateral_read_behavior(ctx, vector, kind),
        ExecutionMode::AttenuationBehavior => behavior_m3::attenuation_behavior(ctx, vector, kind),
        ExecutionMode::RevocationCacheBehavior => {
            behavior_m3::revocation_cache_behavior(ctx, vector, kind)
        }
        ExecutionMode::RankBeforeAuthBehavior => {
            behavior_m3::rank_before_auth_behavior(ctx, vector, kind)
        }
        ExecutionMode::RequiredBudgetBehavior => {
            behavior_m3::required_budget_behavior(ctx, vector, kind)
        }
        ExecutionMode::RenderStabilityBehavior => {
            behavior_m3::render_stability_behavior(ctx, vector, kind)
        }
        ExecutionMode::StagnationBehavior => behavior_m3::stagnation_behavior(ctx, vector, kind),
        ExecutionMode::CandidateAdmissionBehavior => {
            behavior_m3::candidate_admission_behavior(ctx, vector, kind)
        }
        ExecutionMode::TrustPlaneBehavior => behavior_m3::trust_plane_behavior(ctx, vector, kind),
        ExecutionMode::CrashPoint1Behavior => behavior_m4::eff_crash_1_behavior(ctx, vector, kind),
        ExecutionMode::CrashPoint2Behavior => behavior_m4::eff_crash_2_behavior(ctx, vector, kind),
        ExecutionMode::CrashPoint3Behavior => behavior_m4::eff_crash_3_behavior(ctx, vector, kind),
        ExecutionMode::CrashRecoveryBehavior => {
            behavior_m4::crash_recovery_behavior(ctx, vector, kind)
        }
        ExecutionMode::UnknownOutcomeBehavior => {
            behavior_m4::unknown_outcome_behavior(ctx, vector, kind)
        }
        ExecutionMode::IdempotencyConflictBehavior => {
            behavior_m4::idempotency_conflict_behavior(ctx, vector, kind)
        }
        ExecutionMode::RecoveryReconciliationBehavior => {
            behavior_m4::recovery_reconciliation_behavior(ctx, vector, kind)
        }
        ExecutionMode::ApprovalR1MissingBehavior => {
            behavior_m5::approval_r1_009_behavior(ctx, vector, kind)
        }
        ExecutionMode::ApprovalSelfBehavior => {
            behavior_m5::approval_self_010_behavior(ctx, vector, kind)
        }
        ExecutionMode::ApprovalFatigueBehavior => {
            behavior_m5::approval_fatigue_011_behavior(ctx, vector, kind)
        }
        ExecutionMode::ShellCancelBehavior => {
            behavior_m5::shell_cancel_005_behavior(ctx, vector, kind)
        }
        ExecutionMode::ShellDetachBehavior => {
            behavior_m5::shell_detach_004_behavior(ctx, vector, kind)
        }
        ExecutionMode::ShellWatchResumeBehavior => {
            behavior_m5::shell_watch_006_behavior(ctx, vector, kind)
        }
        ExecutionMode::ShellChannelIsolationBehavior => {
            behavior_m5::shell_channel_isolation_003_behavior(ctx, vector, kind)
        }
        ExecutionMode::ShellTargetAmbiguityBehavior => {
            behavior_m5::shell_target_ambiguity_001_behavior(ctx, vector, kind)
        }
        ExecutionMode::IntentSupersedeBehavior => {
            behavior_m5_intent::intent_supersede_002_behavior(ctx, vector, kind)
        }
        ExecutionMode::IntentAcceptanceBehavior => {
            behavior_m5_intent::intent_acceptance_007_behavior(ctx, vector, kind)
        }
        ExecutionMode::AgentInstallBehavior => {
            behavior_m6::agent_install_001_behavior(ctx, vector, kind)
        }
        ExecutionMode::AgentBypassBehavior => {
            behavior_m6::agent_bypass_002_behavior(ctx, vector, kind)
        }
        ExecutionMode::AgentOobBehavior => behavior_m6::agent_oob_001_behavior(ctx, vector, kind),
        ExecutionMode::OrdinaryCoreAuditInspectBehavior => {
            behavior_audit::ordinary_core_audit_inspect_001_behavior(ctx, vector, kind)
        }
    }
}

/// Execute one vector under the selected implementation.
pub fn execute_vector(
    ctx: &AssetContext,
    vector: &LoadedVector,
    kind: ImplementationKind,
) -> VectorOutcome {
    let base = |result: &'static str| VectorOutcome {
        id: vector.id.clone(),
        file: vector.file.clone(),
        layer_slug: vector.layer_slug.clone(),
        profiles: vector.profiles.clone(),
        requirement_ids: vector.requirement_ids.clone(),
        result,
        execution: None,
        not_run_reason: None,
        partial_contract_assertions: None,
    };

    match classify(vector) {
        ExecutionPlan::NotRun { reason } => {
            let mut outcome = base("not-run");
            outcome.not_run_reason = Some(reason);
            if vector.id == STORE_DEGRADATION_VECTOR_ID {
                outcome.partial_contract_assertions = Some(json!({
                    "static_contract": store_degradation_assertions(ctx),
                    "m2_behavioral_read_only_subset":
                        behavior::store_degradation_behavioral_subset(),
                    "m4_behavioral_fencing_subset":
                        behavior_m4::store_degradation_m4_fencing_subset(),
                }));
            }
            outcome
        }
        ExecutionPlan::Execute(mode) => match execute_gate(ctx, vector, mode, kind) {
            Err(err) => {
                let mut outcome = base("fail");
                outcome.execution = Some(ExecutionRecord {
                    mode,
                    implementation: kind.label(),
                    grounding: vec![],
                    compared_fields: 0,
                    informative_fields: vec![],
                    mismatches: vec![Mismatch {
                        path: "(execution)".to_owned(),
                        expected: json!("gate executed against registered machine assets"),
                        actual: json!(err.to_string()),
                    }],
                    evidence: json!({ "execution_error": err.to_string() }),
                });
                outcome
            }
            Ok(gate) => {
                let (compared, mismatches) =
                    compare_expected(&vector.expected, &gate.actual, &gate.informative);
                let result: &'static str = if mismatches.is_empty() {
                    "pass"
                } else {
                    "fail"
                };
                let mut outcome = base(result);
                outcome.execution = Some(ExecutionRecord {
                    mode,
                    implementation: gate.implementation.unwrap_or_else(|| kind.label()),
                    grounding: gate.grounding,
                    compared_fields: compared,
                    informative_fields: gate.informative.iter().map(|s| (*s).to_owned()).collect(),
                    mismatches,
                    evidence: gate.evidence,
                });
                outcome
            }
        },
    }
}

/// Execute every vector under the selected implementation.
pub fn execute_all(
    repo_root: &Path,
    vectors: &[LoadedVector],
    kind: ImplementationKind,
) -> Result<Vec<VectorOutcome>, ExecError> {
    let ctx = AssetContext::load(repo_root)?;
    Ok(vectors
        .iter()
        .map(|vector| execute_vector(&ctx, vector, kind))
        .collect())
}

// ---------------------------------------------------------------------------
// Runner self-check (conformance-evidence.md section 3)
// ---------------------------------------------------------------------------

/// Machine report of the wrong-implementation self-check.
#[derive(Debug, Serialize)]
pub struct SelfCheckReport {
    pub report: &'static str,
    pub wrong_implementation: &'static str,
    /// Gates whose corruption is observable through expected-comparison.
    pub corrupted_gates: Vec<&'static str>,
    /// Vector ids that must flip pass -> fail under the wrong implementation.
    pub must_flip: Vec<String>,
    pub flipped_to_fail: Vec<String>,
    /// Vectors that stayed pass under the wrong implementation although
    /// their gate is corrupted — MUST be empty for a conforming runner.
    pub corrupted_but_still_passing: Vec<String>,
    /// Traceability/coverage gates emit runner-discipline constants, so a
    /// lazy implementation is not observable through expected-comparison;
    /// that gap is guarded by report invariants and CI assertions instead.
    pub unobservable_gates: Vec<&'static str>,
    pub verdict: &'static str,
}

/// Gates the deliberately wrong implementation corrupts observably. The M2
/// behavioral modes are corrupted by a gate-bypassing direct store writer;
/// the M3 modes by governance anti-patterns (membership-alone reads,
/// rank-before-auth, stale cache serving, silent truncation, unbounded
/// retry, reshuffling render, content-claimed control plane, accepted
/// amplification).
const CORRUPTED_MODES: [ExecutionMode; 35] = [
    ExecutionMode::SchemaGate,
    ExecutionMode::PerfContractGate,
    ExecutionMode::CasBehavior,
    ExecutionMode::EffectClosureBehavior,
    ExecutionMode::TaskAcceptanceBehavior,
    ExecutionMode::LateralReadBehavior,
    ExecutionMode::AttenuationBehavior,
    ExecutionMode::RevocationCacheBehavior,
    ExecutionMode::RankBeforeAuthBehavior,
    ExecutionMode::RequiredBudgetBehavior,
    ExecutionMode::RenderStabilityBehavior,
    ExecutionMode::StagnationBehavior,
    ExecutionMode::CandidateAdmissionBehavior,
    ExecutionMode::TrustPlaneBehavior,
    ExecutionMode::CrashPoint1Behavior,
    ExecutionMode::CrashPoint2Behavior,
    ExecutionMode::CrashPoint3Behavior,
    ExecutionMode::CrashRecoveryBehavior,
    ExecutionMode::UnknownOutcomeBehavior,
    ExecutionMode::IdempotencyConflictBehavior,
    ExecutionMode::RecoveryReconciliationBehavior,
    ExecutionMode::ApprovalR1MissingBehavior,
    ExecutionMode::ApprovalSelfBehavior,
    ExecutionMode::ApprovalFatigueBehavior,
    ExecutionMode::ShellCancelBehavior,
    ExecutionMode::ShellDetachBehavior,
    ExecutionMode::ShellWatchResumeBehavior,
    ExecutionMode::ShellChannelIsolationBehavior,
    ExecutionMode::ShellTargetAmbiguityBehavior,
    ExecutionMode::IntentSupersedeBehavior,
    ExecutionMode::IntentAcceptanceBehavior,
    ExecutionMode::AgentInstallBehavior,
    ExecutionMode::AgentBypassBehavior,
    ExecutionMode::AgentOobBehavior,
    ExecutionMode::OrdinaryCoreAuditInspectBehavior,
];

/// Run the self-check: the deliberately wrong implementation must fail every
/// vector whose gate it corrupts, otherwise the runner itself is
/// non-conforming ("schema-valid alone is never pass").
pub fn self_check(
    repo_root: &Path,
    vectors: &[LoadedVector],
) -> Result<SelfCheckReport, ExecError> {
    let ctx = AssetContext::load(repo_root)?;
    let mut must_flip: Vec<String> = Vec::new();
    let mut flipped: Vec<String> = Vec::new();
    let mut still_passing: Vec<String> = Vec::new();

    for vector in vectors {
        let ExecutionPlan::Execute(mode) = classify(vector) else {
            continue;
        };
        if !CORRUPTED_MODES.contains(&mode) {
            continue;
        }
        let reference = execute_vector(&ctx, vector, ImplementationKind::Reference);
        if reference.result != "pass" {
            // Only vectors the reference implementation passes can prove the
            // flip; anything else is already visible in the main report.
            continue;
        }
        must_flip.push(vector.id.clone());
        let wrong = execute_vector(&ctx, vector, ImplementationKind::DeliberatelyWrong);
        if wrong.result == "fail" {
            flipped.push(vector.id.clone());
        } else {
            still_passing.push(vector.id.clone());
        }
    }

    let verdict = if still_passing.is_empty() && !must_flip.is_empty() {
        "self-check-passed: the deliberately wrong implementation fails every corrupted vector"
    } else {
        "self-check-FAILED: schema-valid wrong behavior was not caught"
    };
    Ok(SelfCheckReport {
        report: "cognitiveos-conformance-self-check",
        wrong_implementation: "schema-valid outputs, wrong behavior: bridges legacy governed-object \
             shapes, accepts incomplete benefit claims; (behavioral, M2) writes authority state \
             through a gate-bypassing direct store writer — blind last-write-wins over stale \
             CAS, commits OUTCOME_UNKNOWN->COMMITTED, force-completes an ACTIVE task; \
             (behavioral, M3) governance anti-patterns — membership-alone body reads, ranks \
             unauthorized bodies, serves revocation-stale cache entries, silently truncates \
             over-budget required sets, retries without bound, reshuffles rendered prefixes, \
             builds the control plane from content claims, accepts amplified capability \
             derivations; (behavioral, M4) effect/recovery anti-patterns — re-mints under a \
             fresh idempotency key after a crash, blind-re-dispatches on unknown outcome, \
             re-executes external actions during commit recovery, treats an idempotency \
             conflict as dedup success, resumes loops without reconciling in-flight effects",
        corrupted_gates: vec![
            "schema-gate",
            "perf-contract-gate",
            "cas-behavior",
            "effect-closure-behavior",
            "task-acceptance-behavior",
            "lateral-read-behavior",
            "attenuation-behavior",
            "revocation-cache-behavior",
            "rank-before-auth-behavior",
            "required-budget-behavior",
            "render-stability-behavior",
            "stagnation-behavior",
            "candidate-admission-behavior",
            "trust-plane-behavior",
            "crash-point-1-behavior",
            "crash-point-2-behavior",
            "crash-point-3-behavior",
            "crash-recovery-behavior",
            "unknown-outcome-behavior",
            "idempotency-conflict-behavior",
            "recovery-reconciliation-behavior",
            "ordinary-core-audit-inspect-behavior",
        ],
        must_flip,
        flipped_to_fail: flipped,
        corrupted_but_still_passing: still_passing,
        unobservable_gates: vec!["traceability-gate", "coverage-gate"],
        verdict,
    })
}

pub fn self_check_passed(report: &SelfCheckReport) -> bool {
    report.corrupted_but_still_passing.is_empty() && !report.must_flip.is_empty()
}
