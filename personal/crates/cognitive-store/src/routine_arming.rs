//! Personal-private Routine arming + scheduler-driven occurrence ledger
//! (P13-T05, authority migration v37).
//!
//! P11-T08 (v33) gave a Routine its revisions, Trigger admission, and the
//! no-overlap / queue-latest / missed / coalesced ledger, and parked each
//! `active` occurrence as a `task://personal/routine/<occurrence>` row in the
//! daemon `scheduler_entries` table. Nothing consumed those rows. This module
//! closes that loop without a second scheduler:
//!
//! - an **arming** binds one current Routine revision to one confirmed plan
//!   stage and its seated responsible Member **after G2** (the Project is
//!   `active` with an acceptance fact). The ③「周期与触发」declaration lives in
//!   the Routine revision body (`cadence`, `interval_ms`, `bounded_context`,
//!   `attempt_timeout_ms`);
//! - the daemon scheduler tick (kernel-server) fires due schedule triggers,
//!   leases each `active` occurrence row through the fenced
//!   `scheduler_entries` CAS, drives one hosted Attempt (P13-T02) for it, and
//!   writes the daemon-observed Attempt terminal back as the occurrence
//!   outcome. An occurrence is never "completed": it reaches `attempted` with
//!   an outcome fact and `completion_claimed = 0`;
//! - a new Owner instruction is a new Routine revision applied at a safe point
//!   by `continue` / `pause` / `restart`. The running Attempt keeps its context
//!   digest; nothing is injected into a running prompt;
//! - the P11-T02 host daemon state (`paused` after a close-window pause,
//!   `offline` segments) makes schedule firings land as visible `missed`
//!   facts with the host reason.

use crate::clock::format_canonical_utc;
use crate::migration::MigrationPlanEntry;
use crate::project_aggregate::{ConfirmCaller, ProjectAggregateError};
use crate::routine::{
    RoutineOccurrence, RoutineStore, RoutineTriggerSpec, routine_scheduler_task_ref,
};
use crate::sqlite::SqliteAuthorityStore;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

/// Personal-private Routine arming envelope (P13-T05).
pub const ROUTINE_ARMING_PROJECTION_ID: &str = "cognitiveos.personal.routine-arming/0.1";
/// Personal-private Today live-Project overview envelope (P13-T05).
pub const TODAY_OVERVIEW_PROJECTION_ID: &str = "cognitiveos.personal.today-overview/0.1";
/// Scheduler task_ref prefix of Routine occurrences (P11-T08 shape).
pub const ROUTINE_TASK_REF_PREFIX: &str = "task://personal/routine/";
/// Daemon scheduler identity that leases Routine occurrence rows.
pub const ROUTINE_SCHEDULER_LEASE_OWNER: &str = "personal-daemon-scheduler";
/// Shortest schedule period accepted (short periods are for live E2E only).
pub const ROUTINE_ARMING_MIN_INTERVAL_MS: i64 = 1_000;
/// Default hosted Attempt budget for one occurrence.
pub const ROUTINE_ATTEMPT_DEFAULT_TIMEOUT_MS: i64 = 120_000;
/// Hosted Attempt budget ceiling (mirrors the broker's 30 minute ceiling).
pub const ROUTINE_ATTEMPT_MAX_TIMEOUT_MS: i64 = 30 * 60 * 1000;
/// Outcome facts an occurrence may carry. There is deliberately no `success`.
pub const ROUTINE_ATTEMPT_OUTCOMES: [&str; 8] = [
    "done",
    "failed",
    "blocked",
    "unknown",
    "timed-out",
    "signaled",
    "spawn-failed",
    "unknown-outcome",
];

/// Authority migration v37: Routine arming + occurrence dispatch/outcome
/// columns. `p11_routine_occurrence` is rebuilt (v30 precedent) so its
/// disposition CHECK can name `attempted`; existing rows are preserved.
pub const ROUTINE_ARMING_SCHEMA_V37: &str = "
CREATE TABLE p13_routine_arming (
  arming_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  routine_id TEXT NOT NULL REFERENCES p11_routine(routine_id),
  revision_id TEXT NOT NULL REFERENCES p11_routine_revision(revision_id),
  plan_revision_id TEXT NOT NULL,
  stage_id TEXT NOT NULL,
  employee_id TEXT NOT NULL REFERENCES p11_employee(employee_id),
  seq INTEGER NOT NULL CHECK (seq >= 1),
  cadence_kind TEXT NOT NULL CHECK (cadence_kind IN ('manual','interval')),
  interval_ms INTEGER CHECK (interval_ms IS NULL OR interval_ms >= 1000),
  bounded_context TEXT NOT NULL CHECK (length(bounded_context) > 0 AND length(bounded_context) <= 65536),
  attempt_timeout_ms INTEGER NOT NULL CHECK (attempt_timeout_ms > 0 AND attempt_timeout_ms <= 1800000),
  declaration_digest TEXT NOT NULL CHECK (length(declaration_digest) = 64),
  armed_after TEXT NOT NULL CHECK (armed_after = 'G2'),
  state TEXT NOT NULL CHECK (state IN ('armed','paused','superseded')),
  apply_mode TEXT NOT NULL CHECK (apply_mode IN ('arm','continue','pause','restart','resume')),
  next_due_at INTEGER,
  last_fired_at INTEGER,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  UNIQUE (routine_id, seq)
) STRICT;
CREATE INDEX p13_routine_arming_project ON p13_routine_arming(project_id, state);
CREATE INDEX p13_routine_arming_due ON p13_routine_arming(state, cadence_kind, next_due_at);
CREATE TABLE p11_routine_occurrence_v37 (
  occurrence_id TEXT PRIMARY KEY,
  routine_id TEXT NOT NULL,
  revision_id TEXT NOT NULL,
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  trigger_kind TEXT NOT NULL CHECK (trigger_kind IN (
    'manual','schedule','qualified-event'
  )),
  trigger_source TEXT NOT NULL,
  requested_at INTEGER NOT NULL,
  disposition TEXT NOT NULL CHECK (disposition IN (
    'active','queued','coalesced','missed','cancelled','attempted'
  )),
  coalesced_by TEXT,
  miss_reason TEXT,
  policy_digest TEXT NOT NULL CHECK (length(policy_digest) = 64),
  scheduler_task_ref TEXT,
  checkpoint_json TEXT,
  recorded_at INTEGER NOT NULL,
  arming_id TEXT,
  attempt_id TEXT,
  lease_epoch INTEGER,
  started_at INTEGER,
  attempt_outcome TEXT CHECK (attempt_outcome IS NULL OR attempt_outcome IN (
    'done','failed','blocked','unknown','timed-out','signaled','spawn-failed','unknown-outcome'
  )),
  outcome_detail TEXT,
  elapsed_ms INTEGER,
  terminal_at INTEGER,
  completion_claimed INTEGER NOT NULL DEFAULT 0 CHECK (completion_claimed = 0),
  CHECK ((disposition = 'attempted') = (attempt_outcome IS NOT NULL))
) STRICT;
INSERT INTO p11_routine_occurrence_v37 (
  occurrence_id, routine_id, revision_id, project_id, trigger_kind, trigger_source,
  requested_at, disposition, coalesced_by, miss_reason, policy_digest,
  scheduler_task_ref, checkpoint_json, recorded_at
)
SELECT occurrence_id, routine_id, revision_id, project_id, trigger_kind, trigger_source,
       requested_at, disposition, coalesced_by, miss_reason, policy_digest,
       scheduler_task_ref, checkpoint_json, recorded_at
  FROM p11_routine_occurrence;
DROP INDEX IF EXISTS p11_routine_occurrence_scope;
DROP TABLE p11_routine_occurrence;
ALTER TABLE p11_routine_occurrence_v37 RENAME TO p11_routine_occurrence;
CREATE INDEX p11_routine_occurrence_scope
  ON p11_routine_occurrence(routine_id, disposition, recorded_at);
CREATE INDEX p11_routine_occurrence_project_terminal
  ON p11_routine_occurrence(project_id, disposition, terminal_at);
";

/// v37 migration entry.
pub fn routine_arming_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(37, ROUTINE_ARMING_SCHEMA_V37)
}

/// Canonical RFC 3339 UTC text for a millisecond epoch instant (scheduler
/// `next_eligible` / lease expiry columns).
pub fn canonical_timestamp_from_ms(now_ms: i64) -> String {
    let clamped = now_ms.max(0);
    let seconds = u64::try_from(clamped / 1000).unwrap_or(0);
    let millis = u32::try_from(clamped % 1000).unwrap_or(0);
    format_canonical_utc(seconds, millis)
}

/// The ③「周期与触发」declaration parsed from a Routine revision body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutineDeclaration {
    /// `manual` | `interval`.
    pub cadence_kind: String,
    pub interval_ms: Option<i64>,
    pub bounded_context: String,
    pub attempt_timeout_ms: i64,
    /// SHA-256 of the canonical declaration fields.
    pub declaration_digest: String,
}

impl RoutineDeclaration {
    /// Parse and validate a Routine revision body. Fails closed on an
    /// unknown cadence, a too-short interval, an empty / oversize /
    /// secret-shaped context or an out-of-range Attempt budget.
    pub fn from_body_json(
        body_json: &str,
        default_context: &str,
    ) -> Result<Self, ProjectAggregateError> {
        let body: Value =
            serde_json::from_str(body_json).map_err(|_| ProjectAggregateError::Invalid {
                detail: "Routine body must be a JSON object",
            })?;
        let object = body.as_object().ok_or(ProjectAggregateError::Invalid {
            detail: "Routine body must be a JSON object",
        })?;
        let cadence_kind = object
            .get("cadence")
            .and_then(Value::as_str)
            .unwrap_or("manual");
        if cadence_kind != "manual" && cadence_kind != "interval" {
            return Err(ProjectAggregateError::Invalid {
                detail: "declaration cadence must be manual or interval",
            });
        }
        let interval_ms = object.get("interval_ms").and_then(Value::as_i64);
        if cadence_kind == "interval" {
            match interval_ms {
                Some(value) if value >= ROUTINE_ARMING_MIN_INTERVAL_MS => {}
                _ => {
                    return Err(ProjectAggregateError::Invalid {
                        detail: "interval cadence requires interval_ms >= 1000",
                    });
                }
            }
        }
        let bounded_context = object
            .get("bounded_context")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| default_context.to_owned());
        if bounded_context.trim().is_empty() {
            return Err(ProjectAggregateError::Invalid {
                detail: "declaration bounded_context required",
            });
        }
        if bounded_context.len() > crate::hosted_dsh_attempt::HOSTED_ATTEMPT_CONTEXT_MAX_BYTES {
            return Err(ProjectAggregateError::Invalid {
                detail: "declaration bounded_context exceeds the 64 KiB ceiling",
            });
        }
        reject_secret_shape(&bounded_context)?;
        let attempt_timeout_ms = object
            .get("attempt_timeout_ms")
            .and_then(Value::as_i64)
            .unwrap_or(ROUTINE_ATTEMPT_DEFAULT_TIMEOUT_MS);
        if attempt_timeout_ms <= 0 || attempt_timeout_ms > ROUTINE_ATTEMPT_MAX_TIMEOUT_MS {
            return Err(ProjectAggregateError::Invalid {
                detail: "attempt_timeout_ms must be within (0, 30m]",
            });
        }
        let canonical = format!(
            "{cadence_kind}\n{}\n{bounded_context}\n{attempt_timeout_ms}",
            interval_ms.map(|v| v.to_string()).unwrap_or_default()
        );
        Ok(Self {
            cadence_kind: cadence_kind.to_owned(),
            interval_ms: if cadence_kind == "interval" {
                interval_ms
            } else {
                None
            },
            bounded_context,
            attempt_timeout_ms,
            declaration_digest: digest_hex(canonical.as_bytes()),
        })
    }
}

/// Arming request (owner management only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoutineArmSpec<'a> {
    pub project_id: &'a str,
    pub routine_id: &'a str,
    pub revision_id: &'a str,
    pub stage_id: &'a str,
    pub employee_id: &'a str,
    pub now_ms: i64,
}

/// New instruction applied at a safe point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoutineInstructionSpec<'a> {
    pub arming_id: &'a str,
    /// Routine revision to apply; must be the Routine's current revision.
    pub revision_id: &'a str,
    /// `continue` | `pause` | `restart`.
    pub apply: &'a str,
    pub now_ms: i64,
}

/// One arming row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutineArming {
    pub arming_id: String,
    pub project_id: String,
    pub routine_id: String,
    pub revision_id: String,
    pub plan_revision_id: String,
    pub stage_id: String,
    pub employee_id: String,
    pub seq: i64,
    pub cadence_kind: String,
    pub interval_ms: Option<i64>,
    pub bounded_context: String,
    pub attempt_timeout_ms: i64,
    pub declaration_digest: String,
    pub armed_after: String,
    pub state: String,
    pub apply_mode: String,
    pub next_due_at: Option<i64>,
    pub last_fired_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Instruction outcome: the new arming plus what happened to occurrences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutineInstructionOutcome {
    pub arming: RoutineArming,
    /// The active occurrence that keeps running untouched, if any.
    pub active_occurrence_id: Option<String>,
    /// `restart` only: the new-revision occurrence queued behind the active one
    /// (or started at once when nothing was active).
    pub restart_occurrence: Option<RoutineOccurrence>,
}

/// Ledger row with the P13-T05 dispatch / outcome columns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutineLedgerRow {
    pub occurrence: RoutineOccurrence,
    pub arming_id: Option<String>,
    pub attempt_id: Option<String>,
    pub lease_epoch: Option<i64>,
    pub started_at: Option<i64>,
    pub attempt_outcome: Option<String>,
    pub outcome_detail: Option<String>,
    pub elapsed_ms: Option<i64>,
    pub terminal_at: Option<i64>,
    pub completion_claimed: bool,
}

/// Whether the daemon may dispatch right now, from P11-T02 host facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostDispatchAvailability {
    pub available: bool,
    /// `close-paused` | `offline:<cause>` when unavailable.
    pub reason: Option<String>,
}

/// One live-Project overview row (Today). Counts are daemon facts over the
/// requested period; `attempts_done` is Attempts whose child answered `done`
/// — verification is `not-run` for every Attempt, so this is never completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodayProjectRow {
    pub project_id: String,
    pub state: String,
    pub armed_routines: i64,
    pub paused_routines: i64,
    pub running_occurrence_id: Option<String>,
    pub running_since: Option<i64>,
    pub queued_count: i64,
    pub missed_count: i64,
    pub attempts_total: i64,
    pub attempts_done: i64,
    pub attempts_failed: i64,
    pub attempts_unknown: i64,
    /// Sum of daemon-observed `elapsed_ms` over the period; `None` when no
    /// Attempt in the period carried an elapsed fact.
    pub duration_ms: Option<i64>,
    pub current_stage_id: Option<String>,
    pub current_stage_title: Option<String>,
    pub last_terminal_at: Option<i64>,
}

/// Today overview: created / live / blocked counts + one row per live Project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodayOverview {
    pub period: String,
    pub period_start_ms: i64,
    pub now_ms: i64,
    pub created_count: i64,
    pub live_count: i64,
    pub blocked_count: i64,
    pub rows: Vec<TodayProjectRow>,
}

/// Personal-private Routine arming store on the authority writer.
#[derive(Clone)]
pub struct RoutineArmingStore {
    conn: Arc<Mutex<Connection>>,
    routines: RoutineStore,
}

impl RoutineArmingStore {
    /// Share the daemon-owned authority writer.
    pub fn from_authority_store(store: &SqliteAuthorityStore) -> Self {
        Self {
            conn: Arc::clone(&store.conn),
            routines: RoutineStore::from_authority_store(store),
        }
    }

    /// Open the authority database path (tests). The embedded P11-T08 store
    /// opens its own connection on the same WAL file; the daemon path shares
    /// the single writer through [`Self::from_authority_store`].
    pub fn open_path(path: &std::path::Path) -> Result<Self, ProjectAggregateError> {
        let conn = Connection::open(path).map_err(unavailable("open"))?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;",
        )
        .map_err(unavailable("pragma"))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            routines: RoutineStore::open_path(path)?,
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, ProjectAggregateError> {
        self.conn
            .lock()
            .map_err(|_| ProjectAggregateError::Unavailable {
                detail: "authority writer lock poisoned".to_owned(),
            })
    }

    /// The P11-T08 Routine store sharing this writer.
    pub fn routines(&self) -> &RoutineStore {
        &self.routines
    }

    /// Arm one Routine revision after G2 for one confirmed stage and its
    /// seated responsible Member. Fails closed before G2, on a stale
    /// revision, an unseated / non-responsible Member, or an invalid
    /// declaration. One Routine has at most one live arming; revise it
    /// through [`Self::apply_instruction`].
    pub fn arm(
        &self,
        caller: ConfirmCaller,
        spec: &RoutineArmSpec<'_>,
    ) -> Result<RoutineArming, ProjectAggregateError> {
        require_owner(caller)?;
        let conn = self.lock()?;
        let (project_state, accepted_at, plan_revision_id) =
            load_project_gate(&conn, spec.project_id)?;
        if project_state != "active" || accepted_at.is_none() {
            return Err(ProjectAggregateError::Unconfirmed {
                detail: "ROUTINE_ARM_BEFORE_G2: Routine arming requires the Project to have passed G2 joint acceptance",
            });
        }
        let Some(plan_revision_id) = plan_revision_id else {
            return Err(ProjectAggregateError::Rejected {
                detail: "arming requires a current plan revision",
            });
        };
        let (routine_project, current_revision_id, body_json) =
            load_routine_current(&conn, spec.routine_id)?;
        if routine_project != spec.project_id {
            return Err(ProjectAggregateError::Forbidden {
                detail: "cross-project Routine arming rejected",
            });
        }
        if current_revision_id != spec.revision_id {
            return Err(ProjectAggregateError::Stale {
                detail: "stale Routine revision cannot be armed",
            });
        }
        let stage = load_stage(&conn, &plan_revision_id, spec.stage_id)?;
        require_responsible_seated(&conn, spec.project_id, spec.employee_id, spec.stage_id)?;
        if live_arming_id(&conn, spec.routine_id)?.is_some() {
            return Err(ProjectAggregateError::Conflict {
                detail: "Routine is already armed; apply an instruction instead of arming twice",
            });
        }
        let declaration = RoutineDeclaration::from_body_json(
            &body_json,
            &format!("Routine occurrence for stage {} — {}", stage.0, stage.2),
        )?;
        let seq = next_arming_seq(&conn, spec.routine_id)?;
        insert_arming(
            &conn,
            ArmingWrite {
                project_id: spec.project_id,
                routine_id: spec.routine_id,
                revision_id: spec.revision_id,
                plan_revision_id: &plan_revision_id,
                stage_id: spec.stage_id,
                employee_id: spec.employee_id,
                seq,
                declaration: &declaration,
                state: "armed",
                apply_mode: "arm",
                now_ms: spec.now_ms,
            },
        )
        .and_then(|arming_id| load_arming(&conn, &arming_id))
    }

    /// Apply a new Owner instruction (a Routine revision) at a safe point.
    /// `continue` takes effect from the next occurrence, `pause` stops new
    /// occurrences, `restart` queues a new-revision occurrence behind the
    /// active one. The active occurrence and its Attempt are never touched.
    pub fn apply_instruction(
        &self,
        caller: ConfirmCaller,
        spec: &RoutineInstructionSpec<'_>,
    ) -> Result<RoutineInstructionOutcome, ProjectAggregateError> {
        require_owner(caller)?;
        if !matches!(spec.apply, "continue" | "pause" | "restart") {
            return Err(ProjectAggregateError::Invalid {
                detail: "apply must be continue, pause, or restart",
            });
        }
        let (previous, routine_id, stage_title, stage_objective) = {
            let conn = self.lock()?;
            let previous = load_arming(&conn, spec.arming_id)?;
            if previous.state == "superseded" {
                return Err(ProjectAggregateError::Stale {
                    detail: "arming already superseded; instruct the live arming",
                });
            }
            let (_, current_revision_id, _) = load_routine_current(&conn, &previous.routine_id)?;
            if current_revision_id != spec.revision_id {
                return Err(ProjectAggregateError::Stale {
                    detail: "instruction must apply the Routine's current revision",
                });
            }
            let stage = load_stage(&conn, &previous.plan_revision_id, &previous.stage_id)?;
            (
                previous.clone(),
                previous.routine_id.clone(),
                stage.0,
                stage.2,
            )
        };
        let body_json = {
            let conn = self.lock()?;
            load_revision_body(&conn, spec.revision_id)?
        };
        let declaration = RoutineDeclaration::from_body_json(
            &body_json,
            &format!("Routine occurrence for stage {stage_title} — {stage_objective}"),
        )?;
        let restart_occurrence = if spec.apply == "restart" {
            // Queue-latest through the P11-T08 Intent path: nothing running is
            // touched; the new revision starts at the next safe point.
            Some(self.routines.admit_trigger(
                caller,
                &RoutineTriggerSpec {
                    routine_id: &routine_id,
                    revision_id: spec.revision_id,
                    trigger_kind: "manual",
                    trigger_source: "instruction-restart",
                    force_parallel: false,
                    host_unavailable: false,
                    now_ms: spec.now_ms,
                },
            )?)
        } else {
            None
        };
        let conn = self.lock()?;
        let state = if spec.apply == "pause" {
            "paused"
        } else {
            "armed"
        };
        conn.execute(
            "UPDATE p13_routine_arming SET state = 'superseded', updated_at = ?1 WHERE arming_id = ?2",
            params![spec.now_ms, previous.arming_id],
        )
        .map_err(unavailable("supersede arming"))?;
        let seq = next_arming_seq(&conn, &routine_id)?;
        let arming_id = insert_arming(
            &conn,
            ArmingWrite {
                project_id: &previous.project_id,
                routine_id: &routine_id,
                revision_id: spec.revision_id,
                plan_revision_id: &previous.plan_revision_id,
                stage_id: &previous.stage_id,
                employee_id: &previous.employee_id,
                seq,
                declaration: &declaration,
                state,
                apply_mode: spec.apply,
                now_ms: spec.now_ms,
            },
        )?;
        if let Some(occurrence) = restart_occurrence.as_ref() {
            conn.execute(
                "UPDATE p11_routine_occurrence SET arming_id = ?1 WHERE occurrence_id = ?2",
                params![arming_id, occurrence.occurrence_id],
            )
            .map_err(unavailable("bind restart occurrence"))?;
        }
        let active_occurrence_id = load_one_disposition(&conn, &routine_id, "active")?;
        Ok(RoutineInstructionOutcome {
            arming: load_arming(&conn, &arming_id)?,
            active_occurrence_id,
            restart_occurrence,
        })
    }

    /// Resume a paused arming (owner). Same declaration, new seq.
    pub fn resume_arming(
        &self,
        caller: ConfirmCaller,
        arming_id: &str,
        now_ms: i64,
    ) -> Result<RoutineArming, ProjectAggregateError> {
        require_owner(caller)?;
        let conn = self.lock()?;
        let previous = load_arming(&conn, arming_id)?;
        if previous.state != "paused" {
            return Err(ProjectAggregateError::Invalid {
                detail: "only a paused arming may resume",
            });
        }
        conn.execute(
            "UPDATE p13_routine_arming SET state = 'superseded', updated_at = ?1 WHERE arming_id = ?2",
            params![now_ms, previous.arming_id],
        )
        .map_err(unavailable("supersede paused arming"))?;
        let declaration = RoutineDeclaration {
            cadence_kind: previous.cadence_kind.clone(),
            interval_ms: previous.interval_ms,
            bounded_context: previous.bounded_context.clone(),
            attempt_timeout_ms: previous.attempt_timeout_ms,
            declaration_digest: previous.declaration_digest.clone(),
        };
        let seq = next_arming_seq(&conn, &previous.routine_id)?;
        insert_arming(
            &conn,
            ArmingWrite {
                project_id: &previous.project_id,
                routine_id: &previous.routine_id,
                revision_id: &previous.revision_id,
                plan_revision_id: &previous.plan_revision_id,
                stage_id: &previous.stage_id,
                employee_id: &previous.employee_id,
                seq,
                declaration: &declaration,
                state: "armed",
                apply_mode: "resume",
                now_ms,
            },
        )
        .and_then(|id| load_arming(&conn, &id))
    }

    /// One arming by id.
    pub fn get_arming(&self, arming_id: &str) -> Result<RoutineArming, ProjectAggregateError> {
        let conn = self.lock()?;
        load_arming(&conn, arming_id)
    }

    /// The live (`armed` / `paused`) arming of a Routine, if any.
    pub fn live_arming(
        &self,
        routine_id: &str,
    ) -> Result<Option<RoutineArming>, ProjectAggregateError> {
        let conn = self.lock()?;
        match live_arming_id(&conn, routine_id)? {
            Some(id) => load_arming(&conn, &id).map(Some),
            None => Ok(None),
        }
    }

    /// All armings of a Project, newest first (superseded included so the
    /// instruction history is visible).
    pub fn list_armings(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<RoutineArming>, ProjectAggregateError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(&format!(
                "{ARMING_SELECT} WHERE project_id = ?1 ORDER BY created_at DESC, seq DESC LIMIT ?2"
            ))
            .map_err(unavailable("prepare armings"))?;
        let rows = statement
            .query_map(params![project_id, limit.clamp(1, 128)], map_arming)
            .map_err(unavailable("query armings"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("collect armings"))
    }

    // ------------------------------------------------------------------
    // Daemon scheduler tick helpers (no caller: the daemon is the authority).
    // ------------------------------------------------------------------

    /// P11-T02 host facts: a close-window `pause` or an open offline segment
    /// means the daemon must not dispatch; schedule firings become `missed`.
    pub fn host_dispatch_availability(
        &self,
    ) -> Result<HostDispatchAvailability, ProjectAggregateError> {
        let conn = self.lock()?;
        let daemon: Option<(String, String)> = conn
            .query_row(
                "SELECT state, home_id FROM p11_windows_host_daemon ORDER BY updated_at DESC LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(unavailable("host daemon state"))?;
        let Some((state, home_id)) = daemon else {
            return Ok(HostDispatchAvailability {
                available: true,
                reason: None,
            });
        };
        if state == "paused" {
            return Ok(HostDispatchAvailability {
                available: false,
                reason: Some("close-paused".to_owned()),
            });
        }
        if state == "offline" || state == "recovering" {
            let cause: Option<String> = conn
                .query_row(
                    "SELECT cause FROM p11_windows_host_offline_segment
                      WHERE home_id = ?1 AND ended_at IS NULL ORDER BY started_at DESC LIMIT 1",
                    [home_id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(unavailable("offline segment"))?;
            return Ok(HostDispatchAvailability {
                available: false,
                reason: Some(format!(
                    "offline:{}",
                    cause.unwrap_or_else(|| state.clone())
                )),
            });
        }
        Ok(HostDispatchAvailability {
            available: true,
            reason: None,
        })
    }

    /// Armed interval armings whose `next_due_at` has passed.
    pub fn due_schedule_armings(
        &self,
        now_ms: i64,
    ) -> Result<Vec<RoutineArming>, ProjectAggregateError> {
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(&format!(
                "{ARMING_SELECT} WHERE state = 'armed' AND cadence_kind = 'interval'
                   AND next_due_at IS NOT NULL AND next_due_at <= ?1
                 ORDER BY next_due_at, arming_id"
            ))
            .map_err(unavailable("prepare due armings"))?;
        let rows = statement
            .query_map([now_ms], map_arming)
            .map_err(unavailable("query due armings"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("collect due armings"))
    }

    /// Fire one due schedule trigger through the P11-T08 admission path
    /// (no-overlap / queue-latest / missed) and advance `next_due_at`.
    pub fn fire_schedule(
        &self,
        arming: &RoutineArming,
        host: &HostDispatchAvailability,
        now_ms: i64,
    ) -> Result<RoutineOccurrence, ProjectAggregateError> {
        let source = format!("arming:{}", arming.arming_id);
        let occurrence = self.routines.admit_trigger(
            ConfirmCaller::OwnerManagement,
            &RoutineTriggerSpec {
                routine_id: &arming.routine_id,
                revision_id: &arming.revision_id,
                trigger_kind: "schedule",
                trigger_source: &source,
                force_parallel: false,
                host_unavailable: !host.available,
                now_ms,
            },
        )?;
        let conn = self.lock()?;
        conn.execute(
            "UPDATE p11_routine_occurrence SET arming_id = ?1 WHERE occurrence_id = ?2",
            params![arming.arming_id, occurrence.occurrence_id],
        )
        .map_err(unavailable("bind arming to occurrence"))?;
        if let Some(reason) = host.reason.as_deref() {
            conn.execute(
                "UPDATE p11_routine_occurrence SET miss_reason = ?1
                  WHERE occurrence_id = ?2 AND disposition = 'missed'",
                params![
                    format!("host-unavailable:{reason}"),
                    occurrence.occurrence_id
                ],
            )
            .map_err(unavailable("record miss reason"))?;
        }
        let interval = arming.interval_ms.unwrap_or(ROUTINE_ARMING_MIN_INTERVAL_MS);
        let mut next_due = arming.next_due_at.unwrap_or(now_ms) + interval;
        if next_due <= now_ms {
            next_due = now_ms + interval;
        }
        conn.execute(
            "UPDATE p13_routine_arming SET next_due_at = ?1, last_fired_at = ?2, updated_at = ?2
              WHERE arming_id = ?3",
            params![next_due, now_ms, arming.arming_id],
        )
        .map_err(unavailable("advance next_due"))?;
        load_ledger_row(&conn, &occurrence.occurrence_id).map(|row| row.occurrence)
    }

    /// `active` occurrences without an Attempt yet (the tick dispatches these).
    pub fn dispatchable_occurrences(&self) -> Result<Vec<RoutineLedgerRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        query_ledger(
            &conn,
            "WHERE disposition = 'active' AND attempt_id IS NULL ORDER BY recorded_at, occurrence_id",
            &[],
        )
    }

    /// `active` occurrences with a dispatched Attempt (the tick reconciles these).
    pub fn in_flight_occurrences(&self) -> Result<Vec<RoutineLedgerRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        query_ledger(
            &conn,
            "WHERE disposition = 'active' AND attempt_id IS NOT NULL ORDER BY recorded_at, occurrence_id",
            &[],
        )
    }

    /// Bind the persisted Attempt to an `active` occurrence under the exact
    /// scheduler lease the daemon holds. A stale or foreign lease is refused,
    /// so a second dispatcher can never bind a second Attempt (N1).
    pub fn bind_attempt(
        &self,
        occurrence_id: &str,
        arming_id: &str,
        attempt_id: &str,
        lease_epoch: i64,
        now_ms: i64,
    ) -> Result<RoutineLedgerRow, ProjectAggregateError> {
        let conn = self.lock()?;
        let row = load_ledger_row(&conn, occurrence_id)?;
        if row.occurrence.disposition != "active" || row.attempt_id.is_some() {
            return Err(ProjectAggregateError::Conflict {
                detail: "overlap rejected: occurrence already has an Attempt or is not active",
            });
        }
        let task_ref = routine_scheduler_task_ref(occurrence_id);
        let lease: Option<(Option<String>, i64, String)> = conn
            .query_row(
                "SELECT lease_owner, lease_epoch, state FROM scheduler_entries
                  WHERE task_ref = ?1 AND contract_epoch = 1",
                [&task_ref],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(unavailable("scheduler lease for bind"))?;
        match lease {
            Some((Some(owner), epoch, state))
                if owner == ROUTINE_SCHEDULER_LEASE_OWNER
                    && epoch == lease_epoch
                    && state == "leased" => {}
            _ => {
                return Err(ProjectAggregateError::Conflict {
                    detail: "second scheduler rejected: the daemon scheduler lease does not match",
                });
            }
        }
        conn.execute(
            "UPDATE p11_routine_occurrence
                SET arming_id = ?1, attempt_id = ?2, lease_epoch = ?3, started_at = ?4
              WHERE occurrence_id = ?5 AND disposition = 'active' AND attempt_id IS NULL",
            params![arming_id, attempt_id, lease_epoch, now_ms, occurrence_id],
        )
        .map_err(unavailable("bind attempt"))?;
        load_ledger_row(&conn, occurrence_id)
    }

    /// A manual trigger on a Routine that was never armed cannot dispatch: it
    /// stays visible as `missed` (`not-armed`) and may resume once armed (N3).
    pub fn mark_not_armed(
        &self,
        occurrence_id: &str,
        now_ms: i64,
    ) -> Result<RoutineLedgerRow, ProjectAggregateError> {
        let conn = self.lock()?;
        let row = load_ledger_row(&conn, occurrence_id)?;
        if row.occurrence.disposition != "active" || row.attempt_id.is_some() {
            return Err(ProjectAggregateError::Invalid {
                detail: "only an undispatched active occurrence can be marked not-armed",
            });
        }
        conn.execute(
            "UPDATE p11_routine_occurrence
                SET disposition = 'missed', miss_reason = 'not-armed', scheduler_task_ref = NULL,
                    recorded_at = ?1
              WHERE occurrence_id = ?2",
            params![now_ms, occurrence_id],
        )
        .map_err(unavailable("mark not-armed"))?;
        retire_scheduler_row(&conn, &routine_scheduler_task_ref(occurrence_id))?;
        load_ledger_row(&conn, occurrence_id)
    }

    /// Record the daemon-observed Attempt terminal on an occurrence. The
    /// occurrence becomes `attempted` with an outcome fact; `success` is not
    /// an outcome and `completion_claimed` stays 0 (N4 / N7).
    pub fn record_attempt_terminal(
        &self,
        occurrence_id: &str,
        outcome: &str,
        detail: Option<&str>,
        elapsed_ms: Option<i64>,
        now_ms: i64,
    ) -> Result<RoutineLedgerRow, ProjectAggregateError> {
        if !ROUTINE_ATTEMPT_OUTCOMES.contains(&outcome) {
            return Err(ProjectAggregateError::Invalid {
                detail: "attempt outcome must be a daemon-observed terminal fact, never success or completion",
            });
        }
        if let Some(detail) = detail {
            reject_secret_shape(detail)?;
        }
        let conn = self.lock()?;
        let row = load_ledger_row(&conn, occurrence_id)?;
        if row.occurrence.disposition != "active" {
            return Err(ProjectAggregateError::Invalid {
                detail: "only an active occurrence can reach an Attempt terminal",
            });
        }
        let bounded_detail = detail.map(|d| crate::hosted_dsh_attempt::redact_bounded(d, 512));
        conn.execute(
            "UPDATE p11_routine_occurrence
                SET disposition = 'attempted', attempt_outcome = ?1, outcome_detail = ?2,
                    elapsed_ms = ?3, terminal_at = ?4, recorded_at = ?4
              WHERE occurrence_id = ?5",
            params![outcome, bounded_detail, elapsed_ms, now_ms, occurrence_id],
        )
        .map_err(unavailable("record attempt terminal"))?;
        load_ledger_row(&conn, occurrence_id)
    }

    /// Queue-latest promotion: once nothing is `active` for the Routine, the
    /// latest `queued` occurrence becomes `active` with a fresh scheduler row.
    pub fn promote_queued(
        &self,
        routine_id: &str,
        now_ms: i64,
    ) -> Result<Option<RoutineLedgerRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        if load_one_disposition(&conn, routine_id, "active")?.is_some() {
            return Ok(None);
        }
        let Some(queued_id) = load_one_disposition(&conn, routine_id, "queued")? else {
            return Ok(None);
        };
        let task_ref = routine_scheduler_task_ref(&queued_id);
        conn.execute(
            "UPDATE p11_routine_occurrence
                SET disposition = 'active', scheduler_task_ref = ?1, recorded_at = ?2
              WHERE occurrence_id = ?3 AND disposition = 'queued'",
            params![task_ref, now_ms, queued_id],
        )
        .map_err(unavailable("promote queued"))?;
        conn.execute(
            "INSERT INTO scheduler_entries (
               task_ref, contract_epoch, state, lease_owner, lease_epoch, lease_expires,
               next_eligible, attempt_count, cancel_requested
             ) VALUES (?1, 1, 'runnable', NULL, 0, NULL, ?2, 0, 0)
             ON CONFLICT(task_ref, contract_epoch) DO UPDATE SET
               state = excluded.state,
               lease_owner = NULL,
               lease_expires = NULL,
               next_eligible = excluded.next_eligible",
            params![task_ref, canonical_timestamp_from_ms(now_ms)],
        )
        .map_err(unavailable("upsert scheduler row for promoted occurrence"))?;
        load_ledger_row(&conn, &queued_id).map(Some)
    }

    /// One ledger row.
    pub fn get_ledger_row(
        &self,
        occurrence_id: &str,
    ) -> Result<RoutineLedgerRow, ProjectAggregateError> {
        let conn = self.lock()?;
        load_ledger_row(&conn, occurrence_id)
    }

    /// Newest-first occurrence ledger of one Project across all its Routines
    /// (the `runs` data source).
    pub fn list_project_ledger(
        &self,
        project_id: &str,
        limit: i64,
    ) -> Result<Vec<RoutineLedgerRow>, ProjectAggregateError> {
        let conn = self.lock()?;
        let limit = limit.clamp(1, 256);
        query_ledger(
            &conn,
            "WHERE project_id = ?1 ORDER BY recorded_at DESC, occurrence_id DESC LIMIT ?2",
            &[&project_id, &limit],
        )
    }

    /// Today overview over `period` (`today` = since UTC day start, `week` =
    /// last 7 days, `month` = last 30 days). Created = `creating`, live =
    /// `active`, blocked = `attention` / `paused`.
    pub fn today_overview(
        &self,
        period: &str,
        now_ms: i64,
    ) -> Result<TodayOverview, ProjectAggregateError> {
        let period_start_ms = match period {
            "today" => now_ms - now_ms.rem_euclid(86_400_000),
            "week" => now_ms - 7 * 86_400_000,
            "month" => now_ms - 30 * 86_400_000,
            _ => {
                return Err(ProjectAggregateError::Invalid {
                    detail: "period must be today, week, or month",
                });
            }
        };
        let conn = self.lock()?;
        let mut projects = conn
            .prepare(
                "SELECT project_id, state, current_plan_revision_id FROM p11_project
                  ORDER BY created_at, project_id",
            )
            .map_err(unavailable("prepare projects"))?;
        let project_rows = projects
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .map_err(unavailable("query projects"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("collect projects"))?;
        drop(projects);
        let mut overview = TodayOverview {
            period: period.to_owned(),
            period_start_ms,
            now_ms,
            created_count: 0,
            live_count: 0,
            blocked_count: 0,
            rows: Vec::new(),
        };
        for (project_id, state, plan_revision_id) in project_rows {
            match state.as_str() {
                "creating" => overview.created_count += 1,
                "active" => overview.live_count += 1,
                "attention" | "paused" => overview.blocked_count += 1,
                _ => {}
            }
            if state != "active" {
                continue;
            }
            overview.rows.push(today_row(
                &conn,
                &project_id,
                &state,
                plan_revision_id.as_deref(),
                period_start_ms,
                now_ms,
            )?);
        }
        Ok(overview)
    }
}

// ----------------------------------------------------------------------
// SQL helpers
// ----------------------------------------------------------------------

const ARMING_SELECT: &str =
    "SELECT arming_id, project_id, routine_id, revision_id, plan_revision_id,
       stage_id, employee_id, seq, cadence_kind, interval_ms, bounded_context, attempt_timeout_ms,
       declaration_digest, armed_after, state, apply_mode, next_due_at, last_fired_at, created_at,
       updated_at FROM p13_routine_arming";

const LEDGER_SELECT: &str =
    "SELECT occurrence_id, routine_id, revision_id, project_id, trigger_kind,
       trigger_source, requested_at, disposition, coalesced_by, miss_reason, policy_digest,
       scheduler_task_ref, checkpoint_json, recorded_at, arming_id, attempt_id, lease_epoch,
       started_at, attempt_outcome, outcome_detail, elapsed_ms, terminal_at, completion_claimed
  FROM p11_routine_occurrence";

struct ArmingWrite<'a> {
    project_id: &'a str,
    routine_id: &'a str,
    revision_id: &'a str,
    plan_revision_id: &'a str,
    stage_id: &'a str,
    employee_id: &'a str,
    seq: i64,
    declaration: &'a RoutineDeclaration,
    state: &'a str,
    apply_mode: &'a str,
    now_ms: i64,
}

fn insert_arming(
    conn: &Connection,
    write: ArmingWrite<'_>,
) -> Result<String, ProjectAggregateError> {
    let arming_id = next_id("arming")?;
    let next_due_at = if write.state == "armed" && write.declaration.cadence_kind == "interval" {
        write
            .declaration
            .interval_ms
            .map(|interval| write.now_ms + interval)
    } else {
        None
    };
    conn.execute(
        "INSERT INTO p13_routine_arming (
           arming_id, project_id, routine_id, revision_id, plan_revision_id, stage_id, employee_id,
           seq, cadence_kind, interval_ms, bounded_context, attempt_timeout_ms, declaration_digest,
           armed_after, state, apply_mode, next_due_at, last_fired_at, created_at, updated_at
         ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,'G2',?14,?15,?16,NULL,?17,?17)",
        params![
            arming_id,
            write.project_id,
            write.routine_id,
            write.revision_id,
            write.plan_revision_id,
            write.stage_id,
            write.employee_id,
            write.seq,
            write.declaration.cadence_kind,
            write.declaration.interval_ms,
            write.declaration.bounded_context,
            write.declaration.attempt_timeout_ms,
            write.declaration.declaration_digest,
            write.state,
            write.apply_mode,
            next_due_at,
            write.now_ms,
        ],
    )
    .map_err(unavailable("insert arming"))?;
    Ok(arming_id)
}

fn load_project_gate(
    conn: &Connection,
    project_id: &str,
) -> Result<(String, Option<i64>, Option<String>), ProjectAggregateError> {
    conn.query_row(
        "SELECT state, accepted_at, current_plan_revision_id FROM p11_project WHERE project_id = ?1",
        [project_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
    .optional()
    .map_err(unavailable("project gate"))?
    .ok_or(ProjectAggregateError::NotFound {
        detail: "project not found",
    })
}

fn load_routine_current(
    conn: &Connection,
    routine_id: &str,
) -> Result<(String, String, String), ProjectAggregateError> {
    conn.query_row(
        "SELECT r.project_id, r.current_revision_id, v.body_json
           FROM p11_routine r
           JOIN p11_routine_revision v ON v.revision_id = r.current_revision_id
          WHERE r.routine_id = ?1",
        [routine_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
    .optional()
    .map_err(unavailable("routine current"))?
    .ok_or(ProjectAggregateError::NotFound {
        detail: "routine not found",
    })
}

fn load_revision_body(
    conn: &Connection,
    revision_id: &str,
) -> Result<String, ProjectAggregateError> {
    conn.query_row(
        "SELECT body_json FROM p11_routine_revision WHERE revision_id = ?1",
        [revision_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(unavailable("revision body"))?
    .ok_or(ProjectAggregateError::NotFound {
        detail: "routine revision not found",
    })
}

/// (title, responsible_slot, objective) of a stage in a plan revision.
fn load_stage(
    conn: &Connection,
    plan_revision_id: &str,
    stage_id: &str,
) -> Result<(String, String, String), ProjectAggregateError> {
    conn.query_row(
        "SELECT title, responsible_slot, objective FROM p11_stage
          WHERE plan_revision_id = ?1 AND stage_id = ?2",
        params![plan_revision_id, stage_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
    .optional()
    .map_err(unavailable("stage"))?
    .ok_or(ProjectAggregateError::NotFound {
        detail: "stage not found in the current plan revision",
    })
}

fn require_responsible_seated(
    conn: &Connection,
    project_id: &str,
    employee_id: &str,
    stage_id: &str,
) -> Result<(), ProjectAggregateError> {
    let employee: Option<(String, String, String)> = conn
        .query_row(
            "SELECT project_id, state, responsible_stage_ids_json FROM p11_employee
              WHERE employee_id = ?1",
            [employee_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(unavailable("employee for arming"))?;
    let Some((employee_project, state, stage_ids_json)) = employee else {
        return Err(ProjectAggregateError::NotFound {
            detail: "employee not found",
        });
    };
    if employee_project != project_id {
        return Err(ProjectAggregateError::Forbidden {
            detail: "cross-project Member arming rejected",
        });
    }
    if state != "seated" {
        return Err(ProjectAggregateError::Rejected {
            detail: "arming requires the responsible Member to be seated",
        });
    }
    let responsible: Vec<String> = serde_json::from_str(&stage_ids_json).unwrap_or_default();
    if !responsible.iter().any(|id| id == stage_id) {
        return Err(ProjectAggregateError::Rejected {
            detail: "arming requires the Member responsible for this stage",
        });
    }
    Ok(())
}

fn live_arming_id(
    conn: &Connection,
    routine_id: &str,
) -> Result<Option<String>, ProjectAggregateError> {
    conn.query_row(
        "SELECT arming_id FROM p13_routine_arming
          WHERE routine_id = ?1 AND state IN ('armed','paused')
          ORDER BY seq DESC LIMIT 1",
        [routine_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(unavailable("live arming"))
}

fn next_arming_seq(conn: &Connection, routine_id: &str) -> Result<i64, ProjectAggregateError> {
    let last: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(seq), 0) FROM p13_routine_arming WHERE routine_id = ?1",
            [routine_id],
            |row| row.get(0),
        )
        .map_err(unavailable("arming seq"))?;
    Ok(last + 1)
}

fn load_arming(conn: &Connection, arming_id: &str) -> Result<RoutineArming, ProjectAggregateError> {
    conn.query_row(
        &format!("{ARMING_SELECT} WHERE arming_id = ?1"),
        [arming_id],
        map_arming,
    )
    .optional()
    .map_err(unavailable("load arming"))?
    .ok_or(ProjectAggregateError::NotFound {
        detail: "arming not found",
    })
}

fn map_arming(row: &rusqlite::Row<'_>) -> rusqlite::Result<RoutineArming> {
    Ok(RoutineArming {
        arming_id: row.get(0)?,
        project_id: row.get(1)?,
        routine_id: row.get(2)?,
        revision_id: row.get(3)?,
        plan_revision_id: row.get(4)?,
        stage_id: row.get(5)?,
        employee_id: row.get(6)?,
        seq: row.get(7)?,
        cadence_kind: row.get(8)?,
        interval_ms: row.get(9)?,
        bounded_context: row.get(10)?,
        attempt_timeout_ms: row.get(11)?,
        declaration_digest: row.get(12)?,
        armed_after: row.get(13)?,
        state: row.get(14)?,
        apply_mode: row.get(15)?,
        next_due_at: row.get(16)?,
        last_fired_at: row.get(17)?,
        created_at: row.get(18)?,
        updated_at: row.get(19)?,
    })
}

fn load_one_disposition(
    conn: &Connection,
    routine_id: &str,
    disposition: &str,
) -> Result<Option<String>, ProjectAggregateError> {
    conn.query_row(
        "SELECT occurrence_id FROM p11_routine_occurrence
          WHERE routine_id = ?1 AND disposition = ?2
          ORDER BY recorded_at DESC, occurrence_id DESC LIMIT 1",
        params![routine_id, disposition],
        |row| row.get(0),
    )
    .optional()
    .map_err(unavailable("load disposition"))
}

fn retire_scheduler_row(conn: &Connection, task_ref: &str) -> Result<(), ProjectAggregateError> {
    conn.execute(
        "UPDATE scheduler_entries SET state = 'failed', lease_owner = NULL, lease_expires = NULL
          WHERE task_ref = ?1 AND contract_epoch = 1",
        [task_ref],
    )
    .map_err(unavailable("retire scheduler row"))?;
    Ok(())
}

fn query_ledger(
    conn: &Connection,
    clause: &str,
    bind: &[&dyn rusqlite::ToSql],
) -> Result<Vec<RoutineLedgerRow>, ProjectAggregateError> {
    let mut statement = conn
        .prepare(&format!("{LEDGER_SELECT} {clause}"))
        .map_err(unavailable("prepare ledger"))?;
    let rows = statement
        .query_map(bind, map_ledger_row)
        .map_err(unavailable("query ledger"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(unavailable("collect ledger"))
}

fn load_ledger_row(
    conn: &Connection,
    occurrence_id: &str,
) -> Result<RoutineLedgerRow, ProjectAggregateError> {
    conn.query_row(
        &format!("{LEDGER_SELECT} WHERE occurrence_id = ?1"),
        [occurrence_id],
        map_ledger_row,
    )
    .optional()
    .map_err(unavailable("load ledger row"))?
    .ok_or(ProjectAggregateError::NotFound {
        detail: "occurrence not found",
    })
}

fn map_ledger_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RoutineLedgerRow> {
    let completion_claimed: i64 = row.get(22)?;
    Ok(RoutineLedgerRow {
        occurrence: RoutineOccurrence {
            occurrence_id: row.get(0)?,
            routine_id: row.get(1)?,
            revision_id: row.get(2)?,
            project_id: row.get(3)?,
            trigger_kind: row.get(4)?,
            trigger_source: row.get(5)?,
            requested_at: row.get(6)?,
            disposition: row.get(7)?,
            coalesced_by: row.get(8)?,
            miss_reason: row.get(9)?,
            policy_digest: row.get(10)?,
            scheduler_task_ref: row.get(11)?,
            checkpoint_json: row.get(12)?,
            recorded_at: row.get(13)?,
        },
        arming_id: row.get(14)?,
        attempt_id: row.get(15)?,
        lease_epoch: row.get(16)?,
        started_at: row.get(17)?,
        attempt_outcome: row.get(18)?,
        outcome_detail: row.get(19)?,
        elapsed_ms: row.get(20)?,
        terminal_at: row.get(21)?,
        completion_claimed: completion_claimed != 0,
    })
}

fn today_row(
    conn: &Connection,
    project_id: &str,
    state: &str,
    plan_revision_id: Option<&str>,
    period_start_ms: i64,
    now_ms: i64,
) -> Result<TodayProjectRow, ProjectAggregateError> {
    let (armed_routines, paused_routines): (i64, i64) = conn
        .query_row(
            "SELECT COALESCE(SUM(state = 'armed'), 0), COALESCE(SUM(state = 'paused'), 0)
               FROM p13_routine_arming WHERE project_id = ?1",
            [project_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(unavailable("arming counts"))?;
    let running: Option<(String, Option<i64>)> = conn
        .query_row(
            "SELECT occurrence_id, started_at FROM p11_routine_occurrence
              WHERE project_id = ?1 AND disposition = 'active' AND attempt_id IS NOT NULL
              ORDER BY started_at DESC LIMIT 1",
            [project_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(unavailable("running occurrence"))?;
    let (queued_count, missed_count): (i64, i64) = conn
        .query_row(
            "SELECT COALESCE(SUM(disposition = 'queued'), 0),
                    COALESCE(SUM(disposition = 'missed'), 0)
               FROM p11_routine_occurrence WHERE project_id = ?1",
            [project_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(unavailable("queue counts"))?;
    let (attempts_total, attempts_done, attempts_failed, attempts_unknown, duration_sum, duration_rows, last_terminal_at): (
        i64,
        i64,
        i64,
        i64,
        Option<i64>,
        i64,
        Option<i64>,
    ) = conn
        .query_row(
            "SELECT COUNT(*),
                    COALESCE(SUM(attempt_outcome = 'done'), 0),
                    COALESCE(SUM(attempt_outcome IN ('failed','blocked','timed-out','signaled','spawn-failed')), 0),
                    COALESCE(SUM(attempt_outcome IN ('unknown','unknown-outcome')), 0),
                    SUM(elapsed_ms),
                    COALESCE(SUM(elapsed_ms IS NOT NULL), 0),
                    MAX(terminal_at)
               FROM p11_routine_occurrence
              WHERE project_id = ?1 AND disposition = 'attempted'
                AND terminal_at >= ?2 AND terminal_at <= ?3",
            params![project_id, period_start_ms, now_ms],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )
        .map_err(unavailable("attempt counts"))?;
    let duration_ms = if duration_rows > 0 {
        duration_sum
    } else {
        None
    };
    let mut current_stage_id: Option<String> = None;
    let mut current_stage_title: Option<String> = None;
    let stage_from_arming: Option<String> = conn
        .query_row(
            "SELECT stage_id FROM p13_routine_arming
              WHERE project_id = ?1 AND state IN ('armed','paused')
              ORDER BY updated_at DESC LIMIT 1",
            [project_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(unavailable("current stage from arming"))?;
    if let (Some(plan), Some(stage_id)) = (plan_revision_id, stage_from_arming) {
        let title: Option<String> = conn
            .query_row(
                "SELECT title FROM p11_stage WHERE plan_revision_id = ?1 AND stage_id = ?2",
                params![plan, stage_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("stage title"))?;
        current_stage_title = title;
        current_stage_id = Some(stage_id);
    }
    Ok(TodayProjectRow {
        project_id: project_id.to_owned(),
        state: state.to_owned(),
        armed_routines,
        paused_routines,
        running_occurrence_id: running.as_ref().map(|(id, _)| id.clone()),
        running_since: running.and_then(|(_, since)| since),
        queued_count,
        missed_count,
        attempts_total,
        attempts_done,
        attempts_failed,
        attempts_unknown,
        duration_ms,
        current_stage_id,
        current_stage_title,
        last_terminal_at,
    })
}

fn require_owner(caller: ConfirmCaller) -> Result<(), ProjectAggregateError> {
    match caller {
        ConfirmCaller::OwnerManagement => Ok(()),
        ConfirmCaller::TaskChannel | ConfirmCaller::Assistant => {
            Err(ProjectAggregateError::Forbidden {
                detail: "only owner management session may arm or instruct a Routine",
            })
        }
    }
}

fn reject_secret_shape(body: &str) -> Result<(), ProjectAggregateError> {
    let lowered = body.to_ascii_lowercase();
    if lowered.contains("sk-")
        || lowered.contains("bearer ")
        || lowered.contains("api_key")
        || lowered.contains("x-api-key")
        || lowered.contains("ssv1:")
    {
        return Err(ProjectAggregateError::Invalid {
            detail: "secret-shaped material is rejected at registration",
        });
    }
    Ok(())
}

fn digest_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn next_id(prefix: &str) -> Result<String, ProjectAggregateError> {
    let generated = uuid::Uuid::now_v7().as_hyphenated().to_string();
    Ok(format!("{prefix}-{generated}"))
}

fn unavailable(operation: &'static str) -> impl Fn(rusqlite::Error) -> ProjectAggregateError {
    move |source| ProjectAggregateError::Unavailable {
        detail: format!("{operation}: {source}"),
    }
}
