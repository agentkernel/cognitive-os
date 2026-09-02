//! Hidden hosted DSH engine (P11-T07).
//!
//! Durable managed-child identity (pid / digest / artifact) bound onto an
//! Employee `runtime_binding_ref`. Not Installed Agent chrome, not native DSH
//! UI, not an engine store, and not the Member execution engine for Pi.
//! Secrets stay on the daemon Provider proxy (`POST /provider/v1/dsh/chat/completions`).
//! Isolated spawn fail-closes on Windows GNU; Windows OPC E2E is not claimed.

use crate::employee::{EmployeeStore, reject_installed_agent_chrome, reject_pi_member_engine};
use crate::project_aggregate::{ConfirmCaller, ProjectAggregateError};
use crate::sqlite::SqliteAuthorityStore;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::Value;
use std::sync::{Arc, Mutex};

/// Hidden engine identity. Not an Installed Agent.
pub const HOSTED_DSH_ENGINE_ID: &str = "cognitiveos.personal.hidden-hosted-dsh/0.1";
/// Exact DeepSeek Harness source pin (git object). Must match
/// `cognitive_runtime::dsh_agent::DSH_PACKAGE_REVISION`.
pub const HOSTED_DSH_ARTIFACT_DIGEST: &str = "528c682e061696f5a160f363f236ecbf53cbd006";
/// Path B AKP transport profile. Not a new protocol.
pub const HOSTED_DSH_PROTOCOL: &str = "akp-http-json-sse";
/// Daemon-owned secret-bearing path. Children never receive the key.
pub const HOSTED_DSH_PROVIDER_PROXY: &str = "POST /provider/v1/dsh/chat/completions";
/// Path B agent identity reused for proxy routing only — never as Employee chrome.
pub const HOSTED_DSH_PATH_B_AGENT: &str = "agent://personal/dsh";
/// Windows GNU cannot host an isolated DSH child on this toolchain.
pub const HOSTED_DSH_WIN_GNU_FENCE: &str =
    "isolated DSH spawn is fenced on DEV-WIN-GNU-01; DEV-WINDOWS-NATIVE-OPC-01 remains not-run";

/// Authority migration v31: managed hosted-DSH child identity.
pub const HOSTED_DSH_SCHEMA_V31: &str = "
CREATE TABLE p11_hosted_dsh_child (
  child_id TEXT PRIMARY KEY,
  employee_id TEXT NOT NULL REFERENCES p11_employee(employee_id),
  employee_revision_id TEXT NOT NULL,
  task_ref TEXT NOT NULL,
  artifact_digest TEXT NOT NULL CHECK (length(artifact_digest) = 40 OR length(artifact_digest) = 64),
  protocol TEXT NOT NULL,
  pid INTEGER,
  spawn_kind TEXT NOT NULL CHECK (spawn_kind = 'identity-bound'),
  state TEXT NOT NULL CHECK (state IN ('bound','exited')),
  env_keys_json TEXT NOT NULL,
  argv_redacted_json TEXT NOT NULL,
  terminal_kind TEXT NOT NULL CHECK (terminal_kind IN ('started','exited')),
  provider_proxy TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
) STRICT;
CREATE INDEX p11_hosted_dsh_child_employee
  ON p11_hosted_dsh_child(employee_id, created_at);
";

/// v31 migration entry.
pub fn hosted_dsh_migration_entry() -> crate::migration::MigrationPlanEntry {
    crate::migration::MigrationPlanEntry::new(31, HOSTED_DSH_SCHEMA_V31)
}

/// Attempt-runner start input. Full stdio broker is out of this skeleton.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostedDshStartSpec<'a> {
    pub employee_id: &'a str,
    pub employee_revision_id: &'a str,
    pub task_ref: &'a str,
    pub bounded_context: &'a str,
    pub artifact_digest: &'a str,
    pub protocol: &'a str,
    pub engine_id: &'a str,
    pub observed_pid: Option<u32>,
    pub argv: &'a [&'a str],
    pub env_pairs: &'a [(&'a str, &'a str)],
    pub child_output: Option<&'a str>,
    pub now_ms: i64,
}

/// Durable managed-child observation. `terminal_kind` is never `success`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostedDshObservation {
    pub child_id: String,
    pub employee_id: String,
    pub runtime_binding_ref: String,
    pub artifact_digest: String,
    pub protocol: String,
    pub pid: Option<u32>,
    pub spawn_kind: String,
    pub state: String,
    pub terminal_kind: String,
    pub provider_proxy: String,
    pub path_b_agent: String,
    pub secret_bearer: String,
    pub installed_agent: bool,
    pub pi_member_engine: bool,
}

/// Hidden hosted DSH plane over the daemon-owned writer.
#[derive(Clone)]
pub struct HostedDshPlane {
    conn: Arc<Mutex<Connection>>,
    employees: EmployeeStore,
}

impl HostedDshPlane {
    /// Share the daemon-owned authority writer.
    pub fn from_authority_store(store: &SqliteAuthorityStore) -> Self {
        Self {
            conn: Arc::clone(&store.conn),
            employees: EmployeeStore::from_authority_store(store),
        }
    }

    /// Open the authority database path (tests / CLI-free helpers).
    pub fn open_path(path: &std::path::Path) -> Result<Self, ProjectAggregateError> {
        let employees = EmployeeStore::open_path(path)?;
        Ok(Self {
            conn: employees.conn_arc(),
            employees,
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, ProjectAggregateError> {
        self.conn
            .lock()
            .map_err(|_| ProjectAggregateError::Unavailable {
                detail: "authority writer lock poisoned".to_owned(),
            })
    }

    /// Windows GNU isolated spawn is fenced. Other hosts still do not claim OPC E2E.
    pub fn isolated_spawn_is_fenced() -> bool {
        cfg!(all(windows, target_env = "gnu"))
    }

    /// Attempt-runner `start(...)`: persist managed-child identity and bind Employee.
    pub fn start(
        &self,
        caller: ConfirmCaller,
        spec: &HostedDshStartSpec<'_>,
    ) -> Result<HostedDshObservation, ProjectAggregateError> {
        EmployeeStore::require_owner(caller)?;
        if Self::isolated_spawn_is_fenced() {
            return Err(ProjectAggregateError::Rejected {
                detail: HOSTED_DSH_WIN_GNU_FENCE,
            });
        }
        reject_pi_member_engine(spec.engine_id)?;
        reject_installed_agent_chrome(spec.engine_id)?;
        reject_installed_agent_chrome(spec.task_ref)?;
        if spec.engine_id != HOSTED_DSH_ENGINE_ID {
            return Err(ProjectAggregateError::Rejected {
                detail: "hosted DSH engine identity mismatch",
            });
        }
        if spec.artifact_digest != HOSTED_DSH_ARTIFACT_DIGEST {
            return Err(ProjectAggregateError::Rejected {
                detail: "hosted DSH artifact digest mismatch",
            });
        }
        if spec.protocol != HOSTED_DSH_PROTOCOL {
            return Err(ProjectAggregateError::Rejected {
                detail: "hosted DSH protocol mismatch",
            });
        }
        reject_secret_material(spec.argv, spec.env_pairs)?;
        reject_native_harness_escape(spec.argv, spec.env_pairs)?;
        reject_unknown_child_output_as_success(spec.child_output)?;
        if spec.bounded_context.trim().is_empty() {
            return Err(ProjectAggregateError::Invalid {
                detail: "bounded_context required",
            });
        }
        if spec.task_ref.starts_with("task://") && spec.task_ref.len() < 8 {
            return Err(ProjectAggregateError::Invalid {
                detail: "task_ref required",
            });
        }

        let employee = self.employees.get_employee(spec.employee_id)?.ok_or(
            ProjectAggregateError::NotFound {
                detail: "employee not found",
            },
        )?;
        let Some(revision_id) = self.employees.latest_revision_id(spec.employee_id)? else {
            return Err(ProjectAggregateError::NotFound {
                detail: "employee revision not found",
            });
        };
        if revision_id != spec.employee_revision_id {
            return Err(ProjectAggregateError::Rejected {
                detail: "employee_revision_id mismatch",
            });
        }
        if employee.state != "seated" {
            return Err(ProjectAggregateError::Rejected {
                detail: "employee must be seated before hosted DSH bind",
            });
        }

        let child_id = next_child_id()?;
        let runtime_binding_ref = format!("hosted-dsh:{}:{child_id}", spec.artifact_digest);
        let env_keys: Vec<&str> = spec.env_pairs.iter().map(|(key, _)| *key).collect();
        let env_keys_json =
            serde_json::to_string(&env_keys).map_err(|_| ProjectAggregateError::Unavailable {
                detail: "serialize env keys".to_owned(),
            })?;
        let argv_redacted: Vec<String> =
            spec.argv.iter().map(|arg| redact_argv_token(arg)).collect();
        let argv_redacted_json = serde_json::to_string(&argv_redacted).map_err(|_| {
            ProjectAggregateError::Unavailable {
                detail: "serialize argv".to_owned(),
            }
        })?;
        let pid = spec.observed_pid.map(i64::from);

        {
            let conn = self.lock()?;
            conn.execute(
                "INSERT INTO p11_hosted_dsh_child (
                    child_id, employee_id, employee_revision_id, task_ref, artifact_digest,
                    protocol, pid, spawn_kind, state, env_keys_json, argv_redacted_json,
                    terminal_kind, provider_proxy, created_at, updated_at
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,'identity-bound','bound',?8,?9,'started',?10,?11,?11)",
                params![
                    child_id,
                    spec.employee_id,
                    spec.employee_revision_id,
                    spec.task_ref,
                    spec.artifact_digest,
                    spec.protocol,
                    pid,
                    env_keys_json,
                    argv_redacted_json,
                    HOSTED_DSH_PROVIDER_PROXY,
                    spec.now_ms
                ],
            )
            .map_err(unavailable("insert hosted dsh child"))?;
        }

        self.employees
            .bind_runtime(caller, spec.employee_id, &runtime_binding_ref, spec.now_ms)?;

        Ok(HostedDshObservation {
            child_id,
            employee_id: spec.employee_id.to_owned(),
            runtime_binding_ref,
            artifact_digest: spec.artifact_digest.to_owned(),
            protocol: spec.protocol.to_owned(),
            pid: spec.observed_pid,
            spawn_kind: "identity-bound".to_owned(),
            state: "bound".to_owned(),
            terminal_kind: "started".to_owned(),
            provider_proxy: HOSTED_DSH_PROVIDER_PROXY.to_owned(),
            path_b_agent: HOSTED_DSH_PATH_B_AGENT.to_owned(),
            secret_bearer: "daemon-proxy-only".to_owned(),
            installed_agent: false,
            pi_member_engine: false,
        })
    }

    /// Spawn observer (P13-T02): the broker reports the OS pid of an
    /// identity-bound child after the process exists. Only a `bound` child
    /// without a pid may take one; nothing else about the row changes.
    pub fn observe_spawn(
        &self,
        child_id: &str,
        pid: u32,
        now_ms: i64,
    ) -> Result<(), ProjectAggregateError> {
        let conn = self.lock()?;
        let updated = conn
            .execute(
                "UPDATE p11_hosted_dsh_child SET pid = ?1, updated_at = ?2
                  WHERE child_id = ?3 AND state = 'bound' AND pid IS NULL",
                params![i64::from(pid), now_ms, child_id],
            )
            .map_err(unavailable("observe hosted dsh spawn"))?;
        if updated == 0 {
            return Err(ProjectAggregateError::Conflict {
                detail: "hosted DSH child is not a bound, pid-less identity",
            });
        }
        Ok(())
    }

    /// Process-death observer: record child exit without deleting authority rows.
    pub fn observe_exit(
        &self,
        employee_id: &str,
    ) -> Result<Option<HostedDshObservation>, ProjectAggregateError> {
        self.employees.observe_attempt_process_exit(employee_id)?;
        self.latest_child(employee_id)
    }

    /// Latest managed child for an Employee, if any.
    pub fn latest_child(
        &self,
        employee_id: &str,
    ) -> Result<Option<HostedDshObservation>, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT child_id, employee_id, artifact_digest, protocol, pid, spawn_kind, state,
                    terminal_kind, provider_proxy
               FROM p11_hosted_dsh_child
              WHERE employee_id = ?1
              ORDER BY created_at DESC LIMIT 1",
            [employee_id],
            map_child_row,
        )
        .optional()
        .map_err(unavailable("latest hosted dsh child"))
    }
}

fn map_child_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<HostedDshObservation> {
    let child_id: String = row.get(0)?;
    let artifact_digest: String = row.get(2)?;
    let pid: Option<i64> = row.get(4)?;
    Ok(HostedDshObservation {
        runtime_binding_ref: format!("hosted-dsh:{artifact_digest}:{child_id}"),
        child_id,
        employee_id: row.get(1)?,
        artifact_digest,
        protocol: row.get(3)?,
        pid: pid.and_then(|value| u32::try_from(value).ok()),
        spawn_kind: row.get(5)?,
        state: row.get(6)?,
        terminal_kind: row.get(7)?,
        provider_proxy: row.get(8)?,
        path_b_agent: HOSTED_DSH_PATH_B_AGENT.to_owned(),
        secret_bearer: "daemon-proxy-only".to_owned(),
        installed_agent: false,
        pi_member_engine: false,
    })
}

fn reject_secret_material(
    argv: &[&str],
    env_pairs: &[(&str, &str)],
) -> Result<(), ProjectAggregateError> {
    for (key, value) in env_pairs {
        if secret_shaped_key(key) || secret_shaped_value(value) {
            return Err(ProjectAggregateError::Invalid {
                detail: "secret must not enter hosted DSH child env",
            });
        }
    }
    for arg in argv {
        if secret_shaped_key(arg) || secret_shaped_value(arg) {
            return Err(ProjectAggregateError::Invalid {
                detail: "secret must not enter hosted DSH child argv",
            });
        }
    }
    Ok(())
}

fn reject_native_harness_escape(
    argv: &[&str],
    env_pairs: &[(&str, &str)],
) -> Result<(), ProjectAggregateError> {
    let haystack = argv
        .iter()
        .copied()
        .chain(env_pairs.iter().map(|(key, _)| *key))
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>();
    for item in &haystack {
        if item.contains("--mcp")
            || item.contains("native-mcp")
            || item.contains("base-tool")
            || item.contains("hmr")
            || item.contains("home-patch")
        {
            return Err(ProjectAggregateError::Rejected {
                detail: "native MCP/base tool/HMR/home patch is not hosted DSH",
            });
        }
    }
    Ok(())
}

fn reject_unknown_child_output_as_success(
    child_output: Option<&str>,
) -> Result<(), ProjectAggregateError> {
    let Some(raw) = child_output
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(());
    };
    let lowered = raw.to_ascii_lowercase();
    if lowered == "ok"
        || lowered == "success"
        || lowered.contains("agent_end")
        || lowered == "complete"
        || lowered == "completed"
    {
        return Err(ProjectAggregateError::Rejected {
            detail: "unknown child output is not success",
        });
    }
    if let Ok(parsed) = serde_json::from_str::<Value>(raw) {
        if parsed.get("kind").and_then(Value::as_str) == Some("candidate")
            || parsed.get("observation").and_then(Value::as_str) == Some("candidate")
        {
            return Ok(());
        }
        if parsed.get("status").and_then(Value::as_str) == Some("success")
            || parsed.get("ok").and_then(Value::as_bool) == Some(true)
        {
            return Err(ProjectAggregateError::Rejected {
                detail: "unknown child output is not success",
            });
        }
    }
    Err(ProjectAggregateError::Rejected {
        detail: "unknown child output is not success",
    })
}

pub(crate) fn secret_shaped_key(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.contains("secret")
        || lowered.contains("token")
        || lowered.contains("password")
        || lowered.contains("authorization")
        || lowered.contains("api_key")
        || lowered.contains("apikey")
        || lowered.contains("api-key")
}

pub(crate) fn secret_shaped_value(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.contains("sk-")
        || lowered.contains("bearer ")
        || lowered.contains("api_key")
        || lowered.contains("x-api-key")
        || lowered.contains("ssv1:")
}

fn redact_argv_token(arg: &str) -> String {
    if arg.starts_with('-') {
        arg.chars().take(32).collect()
    } else {
        "<arg>".to_owned()
    }
}

fn next_child_id() -> Result<String, ProjectAggregateError> {
    let generated = uuid::Uuid::now_v7().as_hyphenated().to_string();
    Ok(format!("dshchild-{generated}"))
}

fn unavailable(operation: &'static str) -> impl Fn(rusqlite::Error) -> ProjectAggregateError {
    move |source| ProjectAggregateError::Unavailable {
        detail: format!("{operation}: {source}"),
    }
}
