//! `admin-cli`: deterministic management CLI of the CognitiveOS reference
//! implementation (M5 batch 1).
//!
//! Hard rule: this binary must never depend on a model SDK. It is the
//! emergency path that keeps inspect / stop / revoke / reconcile available
//! when no model is reachable (REQ-MGMT-FALLBACK-001, vector
//! `management-deterministic-fallback.json`). All logic lives in the
//! `cognitive-management` library; this binary is argument parsing (std
//! only — zero new dependencies), the session file gate and canonical JSON
//! output.
//!
//! Exit codes: `0` success, `1` registered denial/rejection (error JSON on
//! stderr), `2` usage error.

use cognitive_domain::{LifecycleDomain, ObjectId};
use cognitive_kernel::Clock;
use cognitive_management::executor_port::{
    DispatchOutcome, EffectExecutor, ExecutorCall, ExecutorCapabilities, ExecutorQueryResult,
    PortFailure,
};
use cognitive_management::{
    AuditPortFailure, AuditedInspectError, FileManagementAuditLog, GovernanceLedger,
    InspectRequest, ManagementAction, ManagementError, ManagementPlane,
    PrivilegedManagementSession, RiskClass, StopRequest,
};
use cognitive_runtime::{
    AcceptingOfficialPiAcquisitionLockVerifier, CUSTOM_USER_PROVIDED_RISK_NOTICE,
    CustomInstallationAcknowledgement, CustomUserProvidedProjectVerifier,
    DurableInstallationAuthority, InstallerError, OFFICIAL_NPM_ORIGIN,
    OFFICIAL_PI_INSTALLATION_ROOT, OFFICIAL_PI_PACKAGE, OFFICIAL_PI_VERSION,
    OfficialPiAcquisitionRequest, OfficialPiAgentActivationRequest,
    OfficialPiAgentLifecycleRequest, OfficialPiAgentRegistrationRequest, PackageInstallRequest,
    PiInstallationLifecyclePrecondition, PiInstallationRootActivationRequest,
    PiInstallationUninstallRequest, acquire_official_pi_durable,
    activate_official_pi_agent_durable, activate_official_pi_root_durable, install_package_durable,
    observe_official_pi_agent_health_durable, package_artifact_digest,
    pause_official_pi_agent_durable, recover_official_pi_agent_durable,
    register_official_pi_agent_durable, resume_official_pi_agent_durable,
    rollback_official_pi_root_durable, stop_official_pi_agent_durable,
    uninstall_official_pi_root_durable,
};
use cognitive_store::{SqliteAuthorityStore, SystemClock, UuidV7Generator};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

const USAGE: &str = "admin-cli — deterministic management fallback (no model dependency)

USAGE:
  admin-cli inspect   --store <db> --session <session.json> --domain <lifecycle-domain> --object <object-id> [--audit <journal>]
  admin-cli stop      --store <db> --session <session.json> --execution <object-id>
  admin-cli revoke    --store <db> --session <session.json> --ledger <governance.json>
  admin-cli reconcile --store <db> --session <session.json>
  admin-cli install   --mode custom --session <session.json> --installation-store <db> --project <dir> --package-id <ref> --adapter-digest <sha256> --sandbox-digest <sha256> --compatibility-digest <sha256> --confirm-custom-source yes
  admin-cli install   --mode official --session <session.json> --installation-store <db> --staged-artifact <file> --dependency-lock <file> --node-version <semver> --signed-lock-ref <ref> --adapter-digest <sha256> --sandbox-digest <sha256> --compatibility-digest <sha256>
  admin-cli register  --session <session.json> --installation-store <db> --installation-root <root> --expected-activation-version <u64> --adapter-digest <sha256> --protocol-digest <sha256> --policy-digest <sha256>
  admin-cli activate  --session <session.json> --installation-store <db> --installation-root <root> --expected-fencing-epoch <u64> --protocol-digest <sha256>
  admin-cli activate-root --session <session.json> --installation-store <db> --installation-root <root> --package-ref <pkg://...> [--expected-activation-version <u64>] --compatibility-accepted yes --health-accepted yes
  admin-cli rollback --session <session.json> --installation-store <db> --installation-root <root> --expected-activation-version <u64> --target-activation-version <u64>
  admin-cli agent-pause   --session <session.json> --installation-store <db> --installation-root <root> --expected-fencing-epoch <u64> --protocol-digest <sha256>
  admin-cli agent-resume  --session <session.json> --installation-store <db> --installation-root <root> --expected-fencing-epoch <u64> --protocol-digest <sha256>
  admin-cli agent-stop    --session <session.json> --installation-store <db> --installation-root <root> --expected-fencing-epoch <u64> --protocol-digest <sha256>
  admin-cli agent-recover --session <session.json> --installation-store <db> --installation-root <root> --expected-fencing-epoch <u64> --protocol-digest <sha256>
  admin-cli agent-health  --session <session.json> --installation-store <db> --installation-root <root>
  admin-cli uninstall --store <db> --session <session.json> --installation-store <db> --installation-root <root> --expected-activation-version <u64> --lifecycle-precondition stopped|absent

Verbs run against the SQLite WAL authority store; every mutation goes
through the central deterministic transition gate. Errors are registered
codes (core/specs/registry/errors.yaml) as JSON on stderr.";

/// Executor used when no external adapter is configured: outcome queries
/// answer `Indeterminate` (still-unknown effects quarantine — fail safe),
/// and any dispatch attempt fails loudly (reconciliation never dispatches).
struct UnconfiguredExecutor;

impl EffectExecutor for UnconfiguredExecutor {
    fn capabilities(&self) -> ExecutorCapabilities {
        ExecutorCapabilities {
            queryable: false,
            idempotent: false,
        }
    }

    fn dispatch(&self, _call: &ExecutorCall) -> Result<DispatchOutcome, PortFailure> {
        Err(PortFailure {
            detail: "no external executor adapter is configured; dispatch is not possible"
                .to_owned(),
        })
    }

    fn query_outcome(&self, _idempotency_key: &str) -> Result<ExecutorQueryResult, PortFailure> {
        Ok(ExecutorQueryResult::Indeterminate)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(run(&args));
}

fn run(args: &[String]) -> i32 {
    let Some((verb, rest)) = args.split_first() else {
        eprintln!("{USAGE}");
        return 2;
    };
    let flags = match parse_flags(rest) {
        Ok(flags) => flags,
        Err(message) => return usage_error(&message),
    };
    match verb.as_str() {
        "inspect" => dispatch_inspect(&flags),
        "stop" => dispatch_stop(&flags),
        "revoke" => dispatch_revoke(&flags),
        "reconcile" => dispatch_reconcile(&flags),
        "install" => dispatch_install(&flags),
        "register" => dispatch_register(&flags),
        "activate" => dispatch_activate(&flags),
        "activate-root" => dispatch_activate_root(&flags),
        "rollback" => dispatch_rollback(&flags),
        "agent-pause" => dispatch_agent_lifecycle(&flags, "agent.pause"),
        "agent-resume" => dispatch_agent_lifecycle(&flags, "agent.resume"),
        "agent-stop" => dispatch_agent_lifecycle(&flags, "agent.stop"),
        "agent-recover" => dispatch_agent_lifecycle(&flags, "agent.recover"),
        "agent-health" => dispatch_agent_health(&flags),
        "uninstall" => dispatch_uninstall(&flags),
        other => usage_error(&format!("unknown verb `{other}`")),
    }
}

fn parse_flags(args: &[String]) -> Result<BTreeMap<String, String>, String> {
    let mut flags = BTreeMap::new();
    let mut iter = args.iter();
    while let Some(flag) = iter.next() {
        let Some(name) = flag.strip_prefix("--") else {
            return Err(format!("unexpected argument `{flag}`"));
        };
        let Some(value) = iter.next() else {
            return Err(format!("flag --{name} requires a value"));
        };
        if flags.insert(name.to_owned(), value.clone()).is_some() {
            return Err(format!("flag --{name} given twice"));
        }
    }
    Ok(flags)
}

fn usage_error(message: &str) -> i32 {
    eprintln!("error: {message}\n\n{USAGE}");
    2
}

fn required<'f>(flags: &'f BTreeMap<String, String>, name: &str) -> Result<&'f str, String> {
    flags
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| format!("missing required flag --{name}"))
}

/// Load and shape-validate the session document; open the store. Both are
/// preconditions of every verb.
fn open_gate(
    flags: &BTreeMap<String, String>,
) -> Result<(PrivilegedManagementSession, SqliteAuthorityStore), GateFailure> {
    let session_path = required(flags, "session").map_err(GateFailure::Usage)?;
    let store_path = required(flags, "store").map_err(GateFailure::Usage)?;
    let session_text = std::fs::read_to_string(session_path).map_err(|err| {
        GateFailure::Management(ManagementError::Ledger(format!(
            "read session {session_path}: {err}"
        )))
    })?;
    let session_value: Value = serde_json::from_str(&session_text).map_err(|err| {
        GateFailure::Management(ManagementError::Ledger(format!(
            "parse session {session_path}: {err}"
        )))
    })?;
    let session = PrivilegedManagementSession::from_json_value(&session_value)
        .map_err(|denial| GateFailure::Management(ManagementError::Denied(denial)))?;
    let store = SqliteAuthorityStore::open(std::path::Path::new(store_path))
        .map_err(|err| GateFailure::Management(ManagementError::Store(err)))?;
    Ok((session, store))
}

enum GateFailure {
    Usage(String),
    Management(ManagementError),
}

fn fail(error: &ManagementError) -> i32 {
    let parts = error.registered_parts();
    let value = json!({
        "error": {
            "category": parts.category,
            "code": parts.code,
            "detail": parts.detail,
            "retryable": parts.retryable,
        }
    });
    match canonical_line(&value) {
        Ok(line) => eprintln!("{line}"),
        Err(message) => eprintln!(
            "{{\"error\":{{\"code\":\"STATE_STORE_UNAVAILABLE\",\"detail\":\"{message}\"}}}}"
        ),
    }
    1
}

fn emit(value: &Value) -> i32 {
    match canonical_line(value) {
        Ok(line) => {
            println!("{line}");
            0
        }
        Err(message) => {
            eprintln!("error: canonical output failed: {message}");
            1
        }
    }
}

fn canonical_line(value: &Value) -> Result<String, String> {
    let bytes = cognitive_contracts::canonical::canonical_bytes_of_value(value)
        .map_err(|err| err.to_string())?;
    String::from_utf8(bytes).map_err(|err| err.to_string())
}

fn gate_failure(failure: GateFailure) -> i32 {
    match failure {
        GateFailure::Usage(message) => usage_error(&message),
        GateFailure::Management(error) => fail(&error),
    }
}

fn fail_audit(error: AuditPortFailure) -> i32 {
    fail(&ManagementError::Ledger(format!(
        "management audit: {}",
        error.detail
    )))
}

fn fail_audited_inspect(error: AuditedInspectError) -> i32 {
    match error {
        AuditedInspectError::Management(error) => fail(&error),
        AuditedInspectError::Audit(error) => fail_audit(error),
    }
}

fn fail_installer(error: InstallerError) -> i32 {
    let value = json!({
        "error": {
            "category": "installation",
            "code": error.code,
            "detail": error.detail,
            "retryable": false,
        }
    });
    match canonical_line(&value) {
        Ok(line) => eprintln!("{line}"),
        Err(message) => eprintln!("error: installer failure serialization: {message}"),
    }
    1
}

fn confirmation_required() -> i32 {
    let value = json!({
        "notice": CUSTOM_USER_PROVIDED_RISK_NOTICE,
        "confirmation_required": "pass --confirm-custom-source yes after reviewing this notice",
    });
    match canonical_line(&value) {
        Ok(line) => eprintln!("{line}"),
        Err(message) => eprintln!("error: custom-source notice serialization: {message}"),
    }
    1
}

fn inspect_audit_path(flags: &BTreeMap<String, String>) -> Result<std::path::PathBuf, String> {
    if let Some(path) = flags.get("audit") {
        return Ok(std::path::PathBuf::from(path));
    }
    let store = required(flags, "store")?;
    Ok(std::path::PathBuf::from(format!(
        "{store}.management-audit.jsonl"
    )))
}

fn dispatch_inspect(flags: &BTreeMap<String, String>) -> i32 {
    let domain = match required(flags, "domain") {
        Ok(text) => match LifecycleDomain::parse(text) {
            Ok(domain) => domain,
            Err(err) => return usage_error(&format!("--domain: {err}")),
        },
        Err(message) => return usage_error(&message),
    };
    let object_id = match required(flags, "object") {
        Ok(text) => match ObjectId::parse(text) {
            Ok(id) => id,
            Err(err) => return usage_error(&format!("--object: {err}")),
        },
        Err(message) => return usage_error(&message),
    };
    let (session, store) = match open_gate(flags) {
        Ok(gate) => gate,
        Err(failure) => return gate_failure(failure),
    };
    let audit_path = match inspect_audit_path(flags) {
        Ok(path) => path,
        Err(message) => return usage_error(&message),
    };
    let clock = SystemClock;
    let ids = UuidV7Generator;
    let audit = match FileManagementAuditLog::open(&audit_path, clock) {
        Ok(audit) => audit,
        Err(error) => return fail_audit(error),
    };
    let plane = ManagementPlane::deterministic(&store, &clock, &ids);
    match plane.inspect_with_audit(&session, &InspectRequest { domain, object_id }, &audit) {
        Ok(report) => match serde_json::to_value(&report) {
            Ok(value) => emit(&value),
            Err(err) => {
                eprintln!("error: report serialization failed: {err}");
                1
            }
        },
        Err(error) => fail_audited_inspect(error),
    }
}

fn dispatch_stop(flags: &BTreeMap<String, String>) -> i32 {
    let execution_id = match required(flags, "execution") {
        Ok(text) => match ObjectId::parse(text) {
            Ok(id) => id,
            Err(err) => return usage_error(&format!("--execution: {err}")),
        },
        Err(message) => return usage_error(&message),
    };
    let (session, store) = match open_gate(flags) {
        Ok(gate) => gate,
        Err(failure) => return gate_failure(failure),
    };
    let clock = SystemClock;
    let ids = UuidV7Generator;
    let plane = ManagementPlane::deterministic(&store, &clock, &ids);
    match plane.stop(&session, &StopRequest { execution_id }) {
        Ok(report) => match serde_json::to_value(&report) {
            Ok(value) => emit(&value),
            Err(err) => {
                eprintln!("error: report serialization failed: {err}");
                1
            }
        },
        Err(error) => fail(&error),
    }
}

fn dispatch_revoke(flags: &BTreeMap<String, String>) -> i32 {
    let ledger_path = match required(flags, "ledger") {
        Ok(text) => text.to_owned(),
        Err(message) => return usage_error(&message),
    };
    let (session, store) = match open_gate(flags) {
        Ok(gate) => gate,
        Err(failure) => return gate_failure(failure),
    };
    let mut ledger = match GovernanceLedger::load(std::path::Path::new(&ledger_path)) {
        Ok(ledger) => ledger,
        Err(error) => return fail(&error),
    };
    let clock = SystemClock;
    let ids = UuidV7Generator;
    let plane = ManagementPlane::deterministic(&store, &clock, &ids);
    match plane.revoke(&session, &mut ledger) {
        Ok(report) => match serde_json::to_value(&report) {
            Ok(value) => emit(&value),
            Err(err) => {
                eprintln!("error: report serialization failed: {err}");
                1
            }
        },
        Err(error) => fail(&error),
    }
}

fn dispatch_reconcile(flags: &BTreeMap<String, String>) -> i32 {
    let (session, store) = match open_gate(flags) {
        Ok(gate) => gate,
        Err(failure) => return gate_failure(failure),
    };
    let clock = SystemClock;
    let ids = UuidV7Generator;
    let plane = ManagementPlane::deterministic(&store, &clock, &ids);
    match plane.reconcile(&session, &UnconfiguredExecutor) {
        Ok(report) => emit(&report.to_json_value()),
        Err(error) => fail(&error),
    }
}

fn dispatch_install(flags: &BTreeMap<String, String>) -> i32 {
    let mode = match required(flags, "mode") {
        Ok(mode) => mode,
        Err(message) => return usage_error(&message),
    };
    if mode != "custom" && mode != "official" {
        return usage_error("--mode must be custom or official");
    }
    if mode == "custom" && flags.get("confirm-custom-source").map(String::as_str) != Some("yes") {
        return confirmation_required();
    }

    let session_path = match required(flags, "session") {
        Ok(path) => path,
        Err(message) => return usage_error(&message),
    };
    let session_text = match std::fs::read_to_string(session_path) {
        Ok(text) => text,
        Err(err) => {
            return fail(&ManagementError::Ledger(format!(
                "read session {session_path}: {err}"
            )));
        }
    };
    let session_value: Value = match serde_json::from_str(&session_text) {
        Ok(value) => value,
        Err(err) => {
            return fail(&ManagementError::Ledger(format!(
                "parse session {session_path}: {err}"
            )));
        }
    };
    let session = match PrivilegedManagementSession::from_json_value(&session_value) {
        Ok(session) => session,
        Err(denial) => return fail(&ManagementError::Denied(denial)),
    };
    let package_id = match required(flags, "package-id") {
        Ok(value) if value.starts_with("pkg://") => value,
        Ok(_) => return usage_error("--package-id must be a pkg:// immutable package reference"),
        Err(message) => return usage_error(&message),
    };
    let clock = SystemClock;
    let now = match clock.now() {
        Ok(now) => now,
        Err(err) => {
            return fail(&ManagementError::Ledger(format!(
                "read management clock: {err}"
            )));
        }
    };
    let action = ManagementAction {
        action: "agent.install".to_owned(),
        domain: "cognitiveos.management".to_owned(),
        resource: format!("agent-installation://{package_id}"),
        risk: RiskClass::R1,
        step_up_required: false,
        step_up_satisfied: false,
    };
    if let Err(denial) = session.authorize(&action, &now) {
        return fail(&ManagementError::Denied(denial));
    }

    if mode == "official" {
        return dispatch_official_install(flags);
    }

    let project = match required(flags, "project") {
        Ok(path) => Path::new(path),
        Err(message) => return usage_error(&message),
    };
    let prepared = match prepare_locked_project(project) {
        Ok(prepared) => prepared,
        Err(error) => return fail_installer(error),
    };
    let adapter_digest = match required_digest(flags, "adapter-digest") {
        Ok(value) => value,
        Err(message) => return usage_error(&message),
    };
    let sandbox_digest = match required_digest(flags, "sandbox-digest") {
        Ok(value) => value,
        Err(message) => return usage_error(&message),
    };
    let compatibility_digest = match required_digest(flags, "compatibility-digest") {
        Ok(value) => value,
        Err(message) => return usage_error(&message),
    };
    let acknowledgement = match CustomInstallationAcknowledgement::after_risk_review(
        prepared.bundle_digest.clone(),
        session.human_principal.clone(),
        prepared.project_ref.clone(),
    ) {
        Ok(acknowledgement) => acknowledgement,
        Err(error) => return fail_installer(error),
    };
    let request = PackageInstallRequest {
        package_id: package_id.to_owned(),
        publisher: "custom-user-provided".to_owned(),
        package_version: "bundle".to_owned(),
        artifact: prepared.bundle,
        declared_artifact_digest: prepared.bundle_digest.clone(),
        signature_ref: format!(
            "custom-user-provided://operator/{}",
            session.human_principal
        ),
        provenance_ref: format!("custom-user-provided://project/{}", prepared.project_ref),
        adapter_digest: adapter_digest.to_owned(),
        sandbox_digest: sandbox_digest.to_owned(),
        compatibility_digest: compatibility_digest.to_owned(),
        lockfile_digest: prepared.lockfile_digest.clone(),
        expected_adapter_digest: adapter_digest.to_owned(),
        expected_sandbox_digest: sandbox_digest.to_owned(),
        expected_compatibility_digest: compatibility_digest.to_owned(),
    };
    let store_path = match required(flags, "installation-store") {
        Ok(path) => Path::new(path),
        Err(message) => return usage_error(&message),
    };
    let authority = match DurableInstallationAuthority::open(store_path) {
        Ok(authority) => authority,
        Err(error) => return fail_installer(error),
    };
    let manager = match authority.acquire_installation_manager() {
        Ok(manager) => manager,
        Err(error) => return fail_installer(error),
    };
    let verifier = CustomUserProvidedProjectVerifier::new(acknowledgement);
    if let Err(error) = install_package_durable(&manager, &request, &verifier) {
        return fail_installer(error);
    }
    let committed = match manager.committed_installation(package_id) {
        Ok(Some(committed)) => committed,
        Ok(None) => {
            return fail_installer(InstallerError {
                code: "STATE_STORE_UNAVAILABLE",
                detail: "installation commit is not queryable through the manager".to_owned(),
            });
        }
        Err(error) => return fail_installer(error),
    };
    let evidence = match committed.evidence() {
        Some(evidence) => evidence,
        None => {
            return fail_installer(InstallerError {
                code: "STATE_STORE_UNAVAILABLE",
                detail: "Custom installation acknowledgement evidence was not durably committed"
                    .to_owned(),
            });
        }
    };
    emit(&json!({
        "source_mode": evidence.source_mode(),
        "operator_ref": evidence.operator_ref(),
        "project_ref": evidence.project_ref(),
        "bundle_digest": prepared.bundle_digest,
        "lockfile_digest": evidence.lockfile_digest(),
        "adapter_digest": adapter_digest,
        "sandbox_digest": sandbox_digest,
        "compatibility_digest": compatibility_digest,
        "verification": evidence.verification_result(),
        "capability_grants": authority.capability_grants(),
        "effects_created": 0,
        "tasks_completed": 0,
    }))
}

fn dispatch_register(flags: &BTreeMap<String, String>) -> i32 {
    let root = match required(flags, "installation-root") {
        Ok(root) if root == OFFICIAL_PI_INSTALLATION_ROOT => root,
        Ok(_) => return usage_error("--installation-root must be the versioned official Pi root"),
        Err(message) => return usage_error(&message),
    };
    let expected_version = match required(flags, "expected-activation-version") {
        Ok(value) => match value.parse::<u64>() {
            Ok(version) => version,
            Err(_) => return usage_error("--expected-activation-version must be a u64"),
        },
        Err(message) => return usage_error(&message),
    };
    let adapter_digest = match required_digest(flags, "adapter-digest") {
        Ok(value) => value,
        Err(message) => return usage_error(&message),
    };
    let protocol_digest = match required_digest(flags, "protocol-digest") {
        Ok(value) => value,
        Err(message) => return usage_error(&message),
    };
    let policy_digest = match required_digest(flags, "policy-digest") {
        Ok(value) => value,
        Err(message) => return usage_error(&message),
    };
    let session_path = match required(flags, "session") {
        Ok(path) => path,
        Err(message) => return usage_error(&message),
    };
    let session_text = match std::fs::read_to_string(session_path) {
        Ok(text) => text,
        Err(err) => {
            return fail(&ManagementError::Ledger(format!(
                "read session {session_path}: {err}"
            )));
        }
    };
    let session_value: Value = match serde_json::from_str(&session_text) {
        Ok(value) => value,
        Err(err) => {
            return fail(&ManagementError::Ledger(format!(
                "parse session {session_path}: {err}"
            )));
        }
    };
    let session = match PrivilegedManagementSession::from_json_value(&session_value) {
        Ok(session) => session,
        Err(denial) => return fail(&ManagementError::Denied(denial)),
    };
    let clock = SystemClock;
    let now = match clock.now() {
        Ok(now) => now,
        Err(err) => {
            return fail(&ManagementError::Ledger(format!(
                "read management clock: {err}"
            )));
        }
    };
    let action = ManagementAction {
        action: "agent.register".to_owned(),
        domain: "cognitiveos.management".to_owned(),
        resource: format!("agent-installation://{root}"),
        risk: RiskClass::R1,
        step_up_required: false,
        step_up_satisfied: false,
    };
    if let Err(denial) = session.authorize(&action, &now) {
        return fail(&ManagementError::Denied(denial));
    }
    let installation_store = match required(flags, "installation-store") {
        Ok(path) => Path::new(path),
        Err(message) => return usage_error(&message),
    };
    let authority = match DurableInstallationAuthority::open(installation_store) {
        Ok(authority) => authority,
        Err(error) => return fail_installer(error),
    };
    let manager = match authority.acquire_installation_manager() {
        Ok(manager) => manager,
        Err(error) => return fail_installer(error),
    };
    match register_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentRegistrationRequest {
            installation_root: root.to_owned(),
            expected_activation_version: expected_version,
            expected_adapter_digest: adapter_digest.to_owned(),
            protocol_digest: protocol_digest.to_owned(),
            policy_digest: policy_digest.to_owned(),
        },
    ) {
        Ok(record) => emit(&json!({
            "registration_id": record.registration_id(),
            "instance_id": record.instance_id(),
            "installation_root": record.installation_root(),
            "activation_version": record.activation_version(),
            "package_ref": record.package_ref(),
            "adapter_digest": record.adapter_digest(),
            "protocol_digest": record.protocol_digest(),
            "policy_digest": record.policy_digest(),
            "lifecycle_state": record.lifecycle_state(),
            "fencing_epoch": record.fencing_epoch(),
            "capability_grants": authority.capability_grants(),
            "sidecar_sessions": 0,
            "effects_created": 0,
            "tasks_completed": 0,
        })),
        Err(error) => fail_installer(error),
    }
}

fn dispatch_activate(flags: &BTreeMap<String, String>) -> i32 {
    let root = match required(flags, "installation-root") {
        Ok(root) if root == OFFICIAL_PI_INSTALLATION_ROOT => root,
        Ok(_) => return usage_error("--installation-root must be the versioned official Pi root"),
        Err(message) => return usage_error(&message),
    };
    let expected_epoch = match required(flags, "expected-fencing-epoch") {
        Ok(value) => match value.parse::<u64>() {
            Ok(version) => version,
            Err(_) => return usage_error("--expected-fencing-epoch must be a u64"),
        },
        Err(message) => return usage_error(&message),
    };
    let protocol_digest = match required_digest(flags, "protocol-digest") {
        Ok(value) => value,
        Err(message) => return usage_error(&message),
    };
    let session_path = match required(flags, "session") {
        Ok(path) => path,
        Err(message) => return usage_error(&message),
    };
    let session_text = match std::fs::read_to_string(session_path) {
        Ok(text) => text,
        Err(err) => {
            return fail(&ManagementError::Ledger(format!(
                "read session {session_path}: {err}"
            )));
        }
    };
    let session_value: Value = match serde_json::from_str(&session_text) {
        Ok(value) => value,
        Err(err) => {
            return fail(&ManagementError::Ledger(format!(
                "parse session {session_path}: {err}"
            )));
        }
    };
    let session = match PrivilegedManagementSession::from_json_value(&session_value) {
        Ok(session) => session,
        Err(denial) => return fail(&ManagementError::Denied(denial)),
    };
    let clock = SystemClock;
    let now = match clock.now() {
        Ok(now) => now,
        Err(err) => {
            return fail(&ManagementError::Ledger(format!(
                "read management clock: {err}"
            )));
        }
    };
    let action = ManagementAction {
        action: "agent.activate".to_owned(),
        domain: "cognitiveos.management".to_owned(),
        resource: format!("agent-installation://{root}"),
        risk: RiskClass::R1,
        step_up_required: false,
        step_up_satisfied: false,
    };
    if let Err(denial) = session.authorize(&action, &now) {
        return fail(&ManagementError::Denied(denial));
    }
    let installation_store = match required(flags, "installation-store") {
        Ok(path) => Path::new(path),
        Err(message) => return usage_error(&message),
    };
    let authority = match DurableInstallationAuthority::open(installation_store) {
        Ok(authority) => authority,
        Err(error) => return fail_installer(error),
    };
    let manager = match authority.acquire_installation_manager() {
        Ok(manager) => manager,
        Err(error) => return fail_installer(error),
    };
    match activate_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentActivationRequest {
            installation_root: root.to_owned(),
            expected_fencing_epoch: expected_epoch,
            protocol_digest: protocol_digest.to_owned(),
        },
    ) {
        Ok((record, sidecar)) => emit(&json!({
            "registration_id": record.registration_id(),
            "instance_id": record.instance_id(),
            "installation_root": record.installation_root(),
            "lifecycle_state": record.lifecycle_state(),
            "fencing_epoch": record.fencing_epoch(),
            "sidecar_session_id": sidecar.session_id(),
            "sidecar_lifecycle_state": sidecar.lifecycle_state(),
            "sidecar_fencing_epoch": sidecar.fencing_epoch(),
            "protocol_digest": sidecar.protocol_digest(),
            "capability_grants": authority.capability_grants(),
            "effects_created": 0,
            "tasks_completed": 0,
        })),
        Err(error) => fail_installer(error),
    }
}

fn dispatch_activate_root(flags: &BTreeMap<String, String>) -> i32 {
    let root = match required(flags, "installation-root") {
        Ok(root) if root == OFFICIAL_PI_INSTALLATION_ROOT => root,
        Ok(_) => return usage_error("--installation-root must be the versioned official Pi root"),
        Err(message) => return usage_error(&message),
    };
    let official_package = format!("pkg://{OFFICIAL_PI_PACKAGE}@{OFFICIAL_PI_VERSION}");
    let package_ref = match required(flags, "package-ref") {
        Ok(value) if value == official_package => value,
        Ok(_) => return usage_error("--package-ref must be the official Pi package reference"),
        Err(message) => return usage_error(&message),
    };
    let expected_activation_version = match flags.get("expected-activation-version") {
        Some(value) => match value.parse::<u64>() {
            Ok(version) => Some(version),
            Err(_) => return usage_error("--expected-activation-version must be a u64"),
        },
        None => None,
    };
    if flags.get("compatibility-accepted").map(String::as_str) != Some("yes") {
        return usage_error("--compatibility-accepted must be yes");
    }
    if flags.get("health-accepted").map(String::as_str) != Some("yes") {
        return usage_error("--health-accepted must be yes");
    }
    if let Err(code) = authorize_agent_management(flags, "agent.install", root) {
        return code;
    }
    let authority = match open_installation_authority(flags) {
        Ok(authority) => authority,
        Err(code) => return code,
    };
    let manager = match authority.acquire_installation_manager() {
        Ok(manager) => manager,
        Err(error) => return fail_installer(error),
    };
    match activate_official_pi_root_durable(
        &manager,
        &PiInstallationRootActivationRequest {
            installation_root: root.to_owned(),
            package_ref: package_ref.to_owned(),
            expected_activation_version,
            compatibility_accepted: true,
            health_accepted: true,
        },
    ) {
        Ok(binding) => emit(&json!({
            "installation_root": binding.installation_root(),
            "activation_version": binding.activation_version(),
            "package_ref": binding.package_ref(),
            "capability_grants": authority.capability_grants(),
            "effects_created": 0,
            "tasks_completed": 0,
        })),
        Err(error) => fail_installer(error),
    }
}

fn dispatch_rollback(flags: &BTreeMap<String, String>) -> i32 {
    let root = match required(flags, "installation-root") {
        Ok(root) if root == OFFICIAL_PI_INSTALLATION_ROOT => root,
        Ok(_) => return usage_error("--installation-root must be the versioned official Pi root"),
        Err(message) => return usage_error(&message),
    };
    let expected_version = match required(flags, "expected-activation-version") {
        Ok(value) => match value.parse::<u64>() {
            Ok(version) => version,
            Err(_) => return usage_error("--expected-activation-version must be a u64"),
        },
        Err(message) => return usage_error(&message),
    };
    let target_version = match required(flags, "target-activation-version") {
        Ok(value) => match value.parse::<u64>() {
            Ok(version) => version,
            Err(_) => return usage_error("--target-activation-version must be a u64"),
        },
        Err(message) => return usage_error(&message),
    };
    if let Err(code) = authorize_agent_management(flags, "agent.rollback", root) {
        return code;
    }
    let authority = match open_installation_authority(flags) {
        Ok(authority) => authority,
        Err(code) => return code,
    };
    let manager = match authority.acquire_installation_manager() {
        Ok(manager) => manager,
        Err(error) => return fail_installer(error),
    };
    match rollback_official_pi_root_durable(&manager, root, expected_version, target_version) {
        Ok(binding) => emit(&json!({
            "installation_root": binding.installation_root(),
            "activation_version": binding.activation_version(),
            "package_ref": binding.package_ref(),
            "capability_grants": authority.capability_grants(),
            "effects_created": 0,
            "tasks_completed": 0,
        })),
        Err(error) => fail_installer(error),
    }
}

fn authorize_agent_management(
    flags: &BTreeMap<String, String>,
    action_name: &str,
    root: &str,
) -> Result<(), i32> {
    let session_path = match required(flags, "session") {
        Ok(path) => path,
        Err(message) => return Err(usage_error(&message)),
    };
    let session_text = match std::fs::read_to_string(session_path) {
        Ok(text) => text,
        Err(err) => {
            return Err(fail(&ManagementError::Ledger(format!(
                "read session {session_path}: {err}"
            ))));
        }
    };
    let session_value: Value = match serde_json::from_str(&session_text) {
        Ok(value) => value,
        Err(err) => {
            return Err(fail(&ManagementError::Ledger(format!(
                "parse session {session_path}: {err}"
            ))));
        }
    };
    let session = match PrivilegedManagementSession::from_json_value(&session_value) {
        Ok(session) => session,
        Err(denial) => return Err(fail(&ManagementError::Denied(denial))),
    };
    let clock = SystemClock;
    let now = match clock.now() {
        Ok(now) => now,
        Err(err) => {
            return Err(fail(&ManagementError::Ledger(format!(
                "read management clock: {err}"
            ))));
        }
    };
    let action = ManagementAction {
        action: action_name.to_owned(),
        domain: "cognitiveos.management".to_owned(),
        resource: format!("agent-installation://{root}"),
        risk: RiskClass::R1,
        step_up_required: false,
        step_up_satisfied: false,
    };
    if let Err(denial) = session.authorize(&action, &now) {
        return Err(fail(&ManagementError::Denied(denial)));
    }
    Ok(())
}

fn open_installation_authority(
    flags: &BTreeMap<String, String>,
) -> Result<DurableInstallationAuthority, i32> {
    let installation_store = match required(flags, "installation-store") {
        Ok(path) => Path::new(path),
        Err(message) => return Err(usage_error(&message)),
    };
    match DurableInstallationAuthority::open(installation_store) {
        Ok(authority) => Ok(authority),
        Err(error) => Err(fail_installer(error)),
    }
}

fn dispatch_agent_lifecycle(flags: &BTreeMap<String, String>, action_name: &str) -> i32 {
    let root = match required(flags, "installation-root") {
        Ok(root) if root == OFFICIAL_PI_INSTALLATION_ROOT => root,
        Ok(_) => return usage_error("--installation-root must be the versioned official Pi root"),
        Err(message) => return usage_error(&message),
    };
    let expected_epoch = match required(flags, "expected-fencing-epoch") {
        Ok(value) => match value.parse::<u64>() {
            Ok(version) => version,
            Err(_) => return usage_error("--expected-fencing-epoch must be a u64"),
        },
        Err(message) => return usage_error(&message),
    };
    let protocol_digest = match required_digest(flags, "protocol-digest") {
        Ok(value) => value,
        Err(message) => return usage_error(&message),
    };
    if let Err(code) = authorize_agent_management(flags, action_name, root) {
        return code;
    }
    let authority = match open_installation_authority(flags) {
        Ok(authority) => authority,
        Err(code) => return code,
    };
    let manager = match authority.acquire_installation_manager() {
        Ok(manager) => manager,
        Err(error) => return fail_installer(error),
    };
    let request = OfficialPiAgentLifecycleRequest {
        installation_root: root.to_owned(),
        expected_fencing_epoch: expected_epoch,
        protocol_digest: protocol_digest.to_owned(),
    };
    match action_name {
        "agent.pause" => match pause_official_pi_agent_durable(&manager, &request) {
            Ok(record) => emit(&json!({
                "registration_id": record.registration_id(),
                "instance_id": record.instance_id(),
                "lifecycle_state": record.lifecycle_state(),
                "fencing_epoch": record.fencing_epoch(),
                "capability_grants": authority.capability_grants(),
                "current_sidecar_session": false,
                "effects_created": 0,
                "tasks_completed": 0,
            })),
            Err(error) => fail_installer(error),
        },
        "agent.stop" => match stop_official_pi_agent_durable(&manager, &request) {
            Ok(record) => emit(&json!({
                "registration_id": record.registration_id(),
                "instance_id": record.instance_id(),
                "lifecycle_state": record.lifecycle_state(),
                "fencing_epoch": record.fencing_epoch(),
                "capability_grants": authority.capability_grants(),
                "current_sidecar_session": false,
                "effects_created": 0,
                "tasks_completed": 0,
            })),
            Err(error) => fail_installer(error),
        },
        "agent.resume" => match resume_official_pi_agent_durable(&manager, &request) {
            Ok((record, sidecar)) => emit(&json!({
                "registration_id": record.registration_id(),
                "instance_id": record.instance_id(),
                "lifecycle_state": record.lifecycle_state(),
                "fencing_epoch": record.fencing_epoch(),
                "sidecar_session_id": sidecar.session_id(),
                "sidecar_lifecycle_state": sidecar.lifecycle_state(),
                "sidecar_fencing_epoch": sidecar.fencing_epoch(),
                "capability_grants": authority.capability_grants(),
                "effects_created": 0,
                "tasks_completed": 0,
            })),
            Err(error) => fail_installer(error),
        },
        "agent.recover" => match recover_official_pi_agent_durable(&manager, &request) {
            Ok((record, sidecar)) => emit(&json!({
                "registration_id": record.registration_id(),
                "instance_id": record.instance_id(),
                "lifecycle_state": record.lifecycle_state(),
                "fencing_epoch": record.fencing_epoch(),
                "sidecar_session_id": sidecar.session_id(),
                "sidecar_lifecycle_state": sidecar.lifecycle_state(),
                "sidecar_fencing_epoch": sidecar.fencing_epoch(),
                "capability_grants": authority.capability_grants(),
                "effects_created": 0,
                "tasks_completed": 0,
            })),
            Err(error) => fail_installer(error),
        },
        other => usage_error(&format!("unsupported agent lifecycle action `{other}`")),
    }
}

fn dispatch_agent_health(flags: &BTreeMap<String, String>) -> i32 {
    let root = match required(flags, "installation-root") {
        Ok(root) if root == OFFICIAL_PI_INSTALLATION_ROOT => root,
        Ok(_) => return usage_error("--installation-root must be the versioned official Pi root"),
        Err(message) => return usage_error(&message),
    };
    if let Err(code) = authorize_agent_management(flags, "agent.health", root) {
        return code;
    }
    let authority = match open_installation_authority(flags) {
        Ok(authority) => authority,
        Err(code) => return code,
    };
    let manager = match authority.acquire_installation_manager() {
        Ok(manager) => manager,
        Err(error) => return fail_installer(error),
    };
    match observe_official_pi_agent_health_durable(&manager, root) {
        Ok(health) => emit(&json!({
            "instance_id": health.instance_id(),
            "lifecycle_state": health.lifecycle_state(),
            "fencing_epoch": health.fencing_epoch(),
            "current_sidecar_session": health.current_sidecar_session(),
            "sidecar_fencing_epoch": health.sidecar_fencing_epoch(),
            "process_bound": health.process_bound(),
            "capability_grants": authority.capability_grants(),
            "effects_created": 0,
            "tasks_completed": 0,
        })),
        Err(error) => fail_installer(error),
    }
}

fn dispatch_uninstall(flags: &BTreeMap<String, String>) -> i32 {
    let root = match required(flags, "installation-root") {
        Ok(root) if root == OFFICIAL_PI_INSTALLATION_ROOT => root,
        Ok(_) => return usage_error("--installation-root must be the versioned official Pi root"),
        Err(message) => return usage_error(&message),
    };
    let expected_version = match required(flags, "expected-activation-version") {
        Ok(value) => match value.parse::<u64>() {
            Ok(version) => version,
            Err(_) => return usage_error("--expected-activation-version must be a u64"),
        },
        Err(message) => return usage_error(&message),
    };
    let lifecycle_precondition = match required(flags, "lifecycle-precondition") {
        Ok("stopped") => PiInstallationLifecyclePrecondition::Stopped,
        Ok("absent") => PiInstallationLifecyclePrecondition::Absent,
        Ok(_) => return usage_error("--lifecycle-precondition must be stopped or absent"),
        Err(message) => return usage_error(&message),
    };
    let (session, _authority_store) = match open_gate(flags) {
        Ok(gate) => gate,
        Err(failure) => return gate_failure(failure),
    };
    let clock = SystemClock;
    let now = match clock.now() {
        Ok(now) => now,
        Err(error) => {
            return fail(&ManagementError::Ledger(format!(
                "read management clock: {error}"
            )));
        }
    };
    let action = ManagementAction {
        action: "agent.uninstall".to_owned(),
        domain: "cognitiveos.management".to_owned(),
        resource: format!("agent-installation://{root}"),
        risk: RiskClass::R1,
        step_up_required: false,
        step_up_satisfied: false,
    };
    if let Err(denial) = session.authorize(&action, &now) {
        return fail(&ManagementError::Denied(denial));
    }
    let installation_store = match required(flags, "installation-store") {
        Ok(path) => Path::new(path),
        Err(message) => return usage_error(&message),
    };
    let authority = match DurableInstallationAuthority::open(installation_store) {
        Ok(authority) => authority,
        Err(error) => return fail_installer(error),
    };
    let manager = match authority.acquire_installation_manager() {
        Ok(manager) => manager,
        Err(error) => return fail_installer(error),
    };
    match uninstall_official_pi_root_durable(
        &manager,
        &PiInstallationUninstallRequest {
            installation_root: root.to_owned(),
            expected_activation_version: expected_version,
            lifecycle_precondition: Some(lifecycle_precondition),
        },
    ) {
        Ok(receipt) => emit(&json!({
            "installation_root": receipt.installation_root,
            "activation_version": receipt.activation_version,
            "package_ref": receipt.package_ref,
            "quarantine": receipt.quarantine,
            "capability_grants": authority.capability_grants(),
            "effects_created": 0,
            "tasks_completed": 0,
        })),
        Err(error) => fail_installer(error),
    }
}

/// Acquire an already staged official Pi artifact. This CLI deliberately has
/// no downloader: callers must provide local bytes and a detached lock ref.
fn dispatch_official_install(flags: &BTreeMap<String, String>) -> i32 {
    let staged_artifact = match required(flags, "staged-artifact") {
        Ok(path) => match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) => {
                return fail_installer(installer_failure(format!("read staged artifact: {error}")));
            }
        },
        Err(message) => return usage_error(&message),
    };
    let dependency_lock = match required(flags, "dependency-lock") {
        Ok(path) => match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) => {
                return fail_installer(installer_failure(format!("read dependency lock: {error}")));
            }
        },
        Err(message) => return usage_error(&message),
    };
    let adapter_digest = match required_digest(flags, "adapter-digest") {
        Ok(value) => value,
        Err(message) => return usage_error(&message),
    };
    let sandbox_digest = match required_digest(flags, "sandbox-digest") {
        Ok(value) => value,
        Err(message) => return usage_error(&message),
    };
    let compatibility_digest = match required_digest(flags, "compatibility-digest") {
        Ok(value) => value,
        Err(message) => return usage_error(&message),
    };
    let node_version = match required(flags, "node-version") {
        Ok(value) => value,
        Err(message) => return usage_error(&message),
    };
    let signed_lock_ref = match required(flags, "signed-lock-ref") {
        Ok(value) => value,
        Err(message) => return usage_error(&message),
    };
    let artifact_digest = match package_artifact_digest(&staged_artifact) {
        Ok(value) => value,
        Err(error) => return fail_installer(error),
    };
    let dependency_lock_digest = match package_artifact_digest(&dependency_lock) {
        Ok(value) => value,
        Err(error) => return fail_installer(error),
    };
    let sri_sha512 = cognitive_runtime::package_sri_sha512(&staged_artifact);
    let package_sha256 = cognitive_runtime::package_sha256_digest(&staged_artifact);
    let request = OfficialPiAcquisitionRequest {
        install: PackageInstallRequest {
            package_id: format!("pkg://{OFFICIAL_PI_PACKAGE}@{OFFICIAL_PI_VERSION}"),
            publisher: OFFICIAL_PI_PACKAGE.to_owned(),
            package_version: OFFICIAL_PI_VERSION.to_owned(),
            artifact: staged_artifact,
            declared_artifact_digest: artifact_digest,
            signature_ref: "official-pi-acquisition-lock".to_owned(),
            provenance_ref: OFFICIAL_NPM_ORIGIN.to_owned(),
            adapter_digest: adapter_digest.to_owned(),
            sandbox_digest: sandbox_digest.to_owned(),
            compatibility_digest: compatibility_digest.to_owned(),
            lockfile_digest: dependency_lock_digest.clone(),
            expected_adapter_digest: adapter_digest.to_owned(),
            expected_sandbox_digest: sandbox_digest.to_owned(),
            expected_compatibility_digest: compatibility_digest.to_owned(),
        },
        registry_origin: OFFICIAL_NPM_ORIGIN.to_owned(),
        resolved_origin: OFFICIAL_NPM_ORIGIN.to_owned(),
        sri_sha512,
        declared_package_sha256: package_sha256,
        dependency_lock,
        declared_dependency_lock_digest: dependency_lock_digest,
        node_version: node_version.to_owned(),
        signed_acquisition_lock_ref: signed_lock_ref.to_owned(),
    };
    let store_path = match required(flags, "installation-store") {
        Ok(path) => Path::new(path),
        Err(message) => return usage_error(&message),
    };
    let authority = match DurableInstallationAuthority::open(store_path) {
        Ok(authority) => authority,
        Err(error) => return fail_installer(error),
    };
    let manager = match authority.acquire_installation_manager() {
        Ok(manager) => manager,
        Err(error) => return fail_installer(error),
    };
    if let Err(error) = acquire_official_pi_durable(
        &manager,
        &request,
        &AcceptingOfficialPiAcquisitionLockVerifier,
    ) {
        return fail_installer(error);
    }
    emit(
        &json!({"source_mode":"official_pi","package":OFFICIAL_PI_PACKAGE,"version":OFFICIAL_PI_VERSION,"capability_grants":authority.capability_grants(),"effects_created":0,"tasks_completed":0}),
    )
}

struct PreparedProject {
    project_ref: String,
    bundle: Vec<u8>,
    bundle_digest: String,
    lockfile_digest: String,
}

fn required_digest<'f>(flags: &'f BTreeMap<String, String>, name: &str) -> Result<&'f str, String> {
    let value = required(flags, name)?;
    let valid = value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit());
    if valid {
        Ok(value)
    } else {
        Err(format!("--{name} must be a sha256 digest"))
    }
}

fn installer_failure(detail: impl Into<String>) -> InstallerError {
    InstallerError {
        code: "AGENT_PACKAGE_VERIFICATION_FAILED",
        detail: detail.into(),
    }
}

fn prepare_locked_project(project: &Path) -> Result<PreparedProject, InstallerError> {
    let canonical = std::fs::canonicalize(project)
        .map_err(|err| installer_failure(format!("canonicalize project: {err}")))?;
    if !canonical.is_dir() {
        return Err(installer_failure("project must be a directory"));
    }
    let package_json = canonical.join("package.json");
    let lockfile = canonical.join("package-lock.json");
    let package: Value = serde_json::from_slice(
        &std::fs::read(&package_json)
            .map_err(|err| installer_failure(format!("read package.json: {err}")))?,
    )
    .map_err(|err| installer_failure(format!("parse package.json: {err}")))?;
    reject_floating_dependencies(&package)?;
    let lockfile_bytes = std::fs::read(&lockfile).map_err(|_| {
        installer_failure("package-lock.json is required; unlocked projects are refused")
    })?;
    let _: Value = serde_json::from_slice(&lockfile_bytes)
        .map_err(|err| installer_failure(format!("parse package-lock.json: {err}")))?;
    let before = deterministic_bundle(&canonical)?;
    prepare_dependencies(&canonical)?;
    let bundle = deterministic_bundle(&canonical)?;
    if bundle != before {
        return Err(installer_failure(
            "project changed during dependency preparation",
        ));
    }
    Ok(PreparedProject {
        project_ref: format!("file://{}", canonical.to_string_lossy().replace('\\', "/")),
        bundle_digest: package_artifact_digest(&bundle)?,
        lockfile_digest: package_artifact_digest(&lockfile_bytes)?,
        bundle,
    })
}

fn reject_floating_dependencies(package: &Value) -> Result<(), InstallerError> {
    for field in [
        "dependencies",
        "devDependencies",
        "optionalDependencies",
        "peerDependencies",
    ] {
        let Some(entries) = package.get(field).and_then(Value::as_object) else {
            continue;
        };
        for (name, version) in entries {
            let Some(version) = version.as_str() else {
                return Err(installer_failure(format!(
                    "{field}.{name} must be a fixed string"
                )));
            };
            if version.is_empty()
                || version.starts_with(['^', '~', '>', '<', '=', '*'])
                || version.contains('*')
                || version.eq_ignore_ascii_case("latest")
            {
                return Err(installer_failure(format!(
                    "floating dependency {field}.{name}={version} is refused"
                )));
            }
        }
    }
    Ok(())
}

fn prepare_dependencies(project: &Path) -> Result<(), InstallerError> {
    #[cfg(windows)]
    let npm = "npm.cmd";
    #[cfg(not(windows))]
    let npm = "npm";
    let status = Command::new(npm)
        .current_dir(project)
        .args([
            "ci",
            "--ignore-scripts",
            "--offline",
            "--no-audit",
            "--no-fund",
        ])
        .env_remove("NPM_TOKEN")
        .env_remove("NODE_AUTH_TOKEN")
        .env_remove("DEEPSEEK_API_KEY")
        .env_remove("OPENAI_API_KEY")
        .env_remove("ANTHROPIC_API_KEY")
        .status()
        .map_err(|err| installer_failure(format!("start npm ci --ignore-scripts: {err}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(installer_failure("npm ci --ignore-scripts failed"))
    }
}

fn deterministic_bundle(project: &Path) -> Result<Vec<u8>, InstallerError> {
    let mut files = Vec::new();
    collect_project_files(project, project, &mut files)?;
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let mut bundle = Vec::new();
    for (relative, content) in files {
        bundle.extend_from_slice(&(relative.len() as u64).to_be_bytes());
        bundle.extend_from_slice(relative.as_bytes());
        bundle.extend_from_slice(&(content.len() as u64).to_be_bytes());
        bundle.extend_from_slice(&content);
    }
    Ok(bundle)
}

fn collect_project_files(
    root: &Path,
    current: &Path,
    files: &mut Vec<(String, Vec<u8>)>,
) -> Result<(), InstallerError> {
    for entry in std::fs::read_dir(current)
        .map_err(|err| installer_failure(format!("read project directory: {err}")))?
    {
        let entry = entry.map_err(|err| installer_failure(format!("read project entry: {err}")))?;
        let file_type = entry
            .file_type()
            .map_err(|err| installer_failure(format!("read project entry type: {err}")))?;
        let name = entry.file_name();
        if name == ".git" || name == "node_modules" {
            continue;
        }
        if file_type.is_symlink() {
            return Err(installer_failure("project symlinks are refused"));
        }
        let path = entry.path();
        if file_type.is_dir() {
            collect_project_files(root, &path, files)?;
        } else if file_type.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| installer_failure("project path escaped root"))?
                .to_string_lossy()
                .replace('\\', "/");
            files.push((
                relative,
                std::fs::read(&path)
                    .map_err(|err| installer_failure(format!("read project file: {err}")))?,
            ));
        } else {
            return Err(installer_failure("project contains a non-regular file"));
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn verbs_come_from_management_crate() {
        assert!(cognitive_management::DETERMINISTIC_FALLBACK_VERBS.contains(&"reconcile"));
    }

    #[test]
    fn flag_parsing_is_strict() {
        let parsed = parse_flags(&["--store".to_owned(), "a.db".to_owned()]).unwrap();
        assert_eq!(parsed.get("store").map(String::as_str), Some("a.db"));
        assert!(
            parse_flags(&["--store".to_owned()]).is_err(),
            "missing value"
        );
        assert!(
            parse_flags(&["loose".to_owned()]).is_err(),
            "no positional args"
        );
        assert!(
            parse_flags(&[
                "--store".to_owned(),
                "a".to_owned(),
                "--store".to_owned(),
                "b".to_owned()
            ])
            .is_err(),
            "duplicate flag"
        );
    }
}
