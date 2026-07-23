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
use cognitive_management::executor_port::{
    DispatchOutcome, EffectExecutor, ExecutorCall, ExecutorCapabilities, ExecutorQueryResult,
    PortFailure,
};
use cognitive_management::{
    AuditPortFailure, AuditedInspectError, FileManagementAuditLog, GovernanceLedger,
    InspectRequest, ManagementError, ManagementPlane, PrivilegedManagementSession, StopRequest,
};
use cognitive_store::{SqliteAuthorityStore, SystemClock, UuidV7Generator};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const USAGE: &str = "admin-cli — deterministic management fallback (no model dependency)

USAGE:
  admin-cli inspect   --store <db> --session <session.json> --domain <lifecycle-domain> --object <object-id> [--audit <journal>]
  admin-cli stop      --store <db> --session <session.json> --execution <object-id>
  admin-cli revoke    --store <db> --session <session.json> --ledger <governance.json>
  admin-cli reconcile --store <db> --session <session.json>

Verbs run against the SQLite WAL authority store; every mutation goes
through the central deterministic transition gate. Errors are registered
codes (specs/registry/errors.yaml) as JSON on stderr.";

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
