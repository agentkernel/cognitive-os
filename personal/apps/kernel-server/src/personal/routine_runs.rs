//! Personal-private Routine arming, occurrence ledger and Today overview
//! routes plus the daemon scheduler tick step that drives hosted Attempts
//! (P13-T05).
//!
//! Real caller chain: `routine.arm` (after G2) → the periodic daemon scheduler
//! tick ([`run_routine_tick`], the **only** dispatcher of
//! `task://personal/routine/*` rows) fires due schedule triggers through the
//! P11-T08 Intent path, leases each `active` occurrence through the fenced
//! `scheduler_entries` CAS, launches one hosted Attempt (P13-T02
//! persist-before-dispatch), and writes the daemon-observed Attempt terminal
//! back as the occurrence outcome. `routine.runs` and `today.overview` are the
//! `runs` / Today reads. Manual triggers stay on the existing `routine.trigger`
//! Intent route. Management-channel only; task-channel aliases are 403. There
//! is no Start button, no second scheduler, and no completion: every outcome
//! carries `completion_claimed = false` and `verification_status = not-run`.

use std::time::Duration;

use cognitive_store::{
    ConfirmCaller, HostDispatchAvailability, HostedAttemptRow, HostedDshAttemptStore,
    ProjectAggregateError, ROUTINE_ARMING_PROJECTION_ID, ROUTINE_SCHEDULER_LEASE_OWNER,
    RoutineArmSpec, RoutineArming, RoutineArmingStore, RoutineInstructionSpec, RoutineLedgerRow,
    SqliteAuthorityStore, TODAY_OVERVIEW_PROJECTION_ID, TodayOverview, TodayProjectRow,
    canonical_timestamp_from_ms,
    scheduler::{SchedulerRepository, SchedulerRepositoryError, SchedulerState, SchedulerWorkKey},
};
use serde_json::{Value, json};

use super::hosted_dsh_attempt::{HostedAttemptHost, HostedAttemptLaunch, launch_hosted_attempt};
use super::resource_api::ResourceApiResponse;

const ROUTE_LITERALS: &[&str] = &[
    "POST /management/project/v1/routine.arm",
    "POST /management/project/v1/routine.instruction",
    "POST /management/project/v1/routine.arming.resume",
    "GET /management/project/v1/routine.armings",
    "GET /management/project/v1/routine.runs",
    "GET /management/project/v1/today.overview",
    "POST /task/project/v1/routine.arm",
    "POST /task/project/v1/routine.instruction",
    "POST /task/project/v1/routine.arming.resume",
    "GET /task/project/v1/routine.armings",
    "GET /task/project/v1/routine.runs",
    "GET /task/project/v1/today.overview",
];

/// Lease slack past the Attempt budget before another daemon may reclaim.
const LEASE_SLACK_MS: i64 = 60_000;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Channel {
    Management,
    Task,
}

pub(crate) fn matches(method_path: &str) -> bool {
    parse_route(method_path).is_some()
}

pub(crate) fn is_task_channel(method_path: &str) -> bool {
    parse_route(method_path).is_some_and(|(channel, _)| channel == Channel::Task)
}

pub(crate) fn channel_forbidden() -> ResourceApiResponse {
    error(
        403,
        "ROUTINE_RUNS_CHANNEL_FORBIDDEN",
        "Routine arming / runs operations are management-channel only",
    )
}

pub(crate) fn handle(
    method_path: &str,
    body: &[u8],
    store: &SqliteAuthorityStore,
) -> ResourceApiResponse {
    let Some((channel, literal)) = parse_route(method_path) else {
        return error(
            404,
            "ROUTINE_RUNS_ROUTE_NOT_FOUND",
            "no Routine runs route matched",
        );
    };
    if channel == Channel::Task {
        return channel_forbidden();
    }
    let armings = RoutineArmingStore::from_authority_store(store);
    match literal {
        "POST /management/project/v1/routine.arm" => routine_arm(body, &armings),
        "POST /management/project/v1/routine.instruction" => routine_instruction(body, &armings),
        "POST /management/project/v1/routine.arming.resume" => {
            routine_arming_resume(body, &armings)
        }
        "GET /management/project/v1/routine.armings" => routine_armings(method_path, &armings),
        "GET /management/project/v1/routine.runs" => routine_runs(method_path, &armings),
        "GET /management/project/v1/today.overview" => today_overview(method_path, &armings),
        _ => error(
            404,
            "ROUTINE_RUNS_ROUTE_NOT_FOUND",
            "no Routine runs route matched",
        ),
    }
}

fn parse_route(method_path: &str) -> Option<(Channel, &'static str)> {
    for literal in ROUTE_LITERALS {
        if method_path.starts_with(literal) {
            let channel = if literal.contains("/task/") {
                Channel::Task
            } else {
                Channel::Management
            };
            return Some((channel, *literal));
        }
    }
    None
}

// ----------------------------------------------------------------------
// HTTP handlers
// ----------------------------------------------------------------------

fn routine_arm(body: &[u8], armings: &RoutineArmingStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "ROUTINE_JSON_REQUIRED", "JSON body required");
    };
    let mut fields = Vec::new();
    for name in [
        "project_id",
        "routine_id",
        "revision_id",
        "stage_id",
        "employee_id",
    ] {
        match document.get(name).and_then(Value::as_str) {
            Some(value) if !value.is_empty() => fields.push(value),
            _ => {
                return error(
                    400,
                    "ROUTINE_ARM_FIELD_REQUIRED",
                    &format!("{name} required"),
                );
            }
        }
    }
    match armings.arm(
        ConfirmCaller::OwnerManagement,
        &RoutineArmSpec {
            project_id: fields[0],
            routine_id: fields[1],
            revision_id: fields[2],
            stage_id: fields[3],
            employee_id: fields[4],
            now_ms: now_ms(),
        },
    ) {
        Ok(arming) => ok(json!({
            "status": "ok",
            "projection_id": ROUTINE_ARMING_PROJECTION_ID,
            "arming": arming_json(&arming),
            "scheduler": "daemon-tick-only",
            "manual_trigger_path": "/management/project/v1/routine.trigger",
            "is_authority": true,
        })),
        Err(err) => store_error(err),
    }
}

fn routine_instruction(body: &[u8], armings: &RoutineArmingStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "ROUTINE_JSON_REQUIRED", "JSON body required");
    };
    let Some(arming_id) = document.get("arming_id").and_then(Value::as_str) else {
        return error(400, "ARMING_ID_REQUIRED", "arming_id required");
    };
    let Some(revision_id) = document.get("revision_id").and_then(Value::as_str) else {
        return error(400, "REVISION_ID_REQUIRED", "revision_id required");
    };
    let Some(apply) = document.get("apply").and_then(Value::as_str) else {
        return error(
            400,
            "APPLY_REQUIRED",
            "apply required: continue, pause, or restart",
        );
    };
    match armings.apply_instruction(
        ConfirmCaller::OwnerManagement,
        &RoutineInstructionSpec {
            arming_id,
            revision_id,
            apply,
            now_ms: now_ms(),
        },
    ) {
        Ok(outcome) => ok(json!({
            "status": "ok",
            "projection_id": ROUTINE_ARMING_PROJECTION_ID,
            "arming": arming_json(&outcome.arming),
            "active_occurrence_id": outcome.active_occurrence_id,
            "active_occurrence_untouched": true,
            "running_prompt_injected": false,
            "restart_occurrence_id": outcome
                .restart_occurrence
                .as_ref()
                .map(|occurrence| occurrence.occurrence_id.clone()),
            "restart_disposition": outcome
                .restart_occurrence
                .as_ref()
                .map(|occurrence| occurrence.disposition.clone()),
            "is_authority": true,
        })),
        Err(err) => store_error(err),
    }
}

fn routine_arming_resume(body: &[u8], armings: &RoutineArmingStore) -> ResourceApiResponse {
    let Some(document) = parse_json(body) else {
        return error(400, "ROUTINE_JSON_REQUIRED", "JSON body required");
    };
    let Some(arming_id) = document.get("arming_id").and_then(Value::as_str) else {
        return error(400, "ARMING_ID_REQUIRED", "arming_id required");
    };
    match armings.resume_arming(ConfirmCaller::OwnerManagement, arming_id, now_ms()) {
        Ok(arming) => ok(json!({
            "status": "ok",
            "projection_id": ROUTINE_ARMING_PROJECTION_ID,
            "arming": arming_json(&arming),
            "is_authority": true,
        })),
        Err(err) => store_error(err),
    }
}

fn routine_armings(method_path: &str, armings: &RoutineArmingStore) -> ResourceApiResponse {
    let Some(project_id) = query_parameter(method_path, "project_id").filter(|v| !v.is_empty())
    else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    let limit = query_parameter(method_path, "limit")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(32);
    match armings.list_armings(&project_id, limit) {
        Ok(rows) => ok(json!({
            "status": "ok",
            "projection_id": ROUTINE_ARMING_PROJECTION_ID,
            "project_id": project_id,
            "armings": rows.iter().map(arming_json).collect::<Vec<_>>(),
        })),
        Err(err) => store_error(err),
    }
}

/// The `runs` read: live armings, the occurrence ledger across the Project's
/// Routines with a derived dispatch state, host availability, and the pointer
/// to the real Attempt history (`dsh.hosted.attempt.list`).
fn routine_runs(method_path: &str, armings: &RoutineArmingStore) -> ResourceApiResponse {
    let Some(project_id) = query_parameter(method_path, "project_id").filter(|v| !v.is_empty())
    else {
        return error(400, "PROJECT_ID_REQUIRED", "project_id required");
    };
    let limit = query_parameter(method_path, "limit")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(64);
    let host = match armings.host_dispatch_availability() {
        Ok(host) => host,
        Err(err) => return store_error(err),
    };
    let arming_rows = match armings.list_armings(&project_id, 128) {
        Ok(rows) => rows,
        Err(err) => return store_error(err),
    };
    let ledger = match armings.list_project_ledger(&project_id, limit) {
        Ok(rows) => rows,
        Err(err) => return store_error(err),
    };
    let mut summary = json!({
        "active": 0, "running": 0, "queued": 0, "missed": 0, "coalesced": 0,
        "attempted": 0, "done": 0, "failed": 0, "unknown": 0, "cancelled": 0,
    });
    let occurrences: Vec<Value> = ledger
        .iter()
        .map(|row| {
            let dispatch_state = dispatch_state(row, &arming_rows, &host);
            bump(&mut summary, &row.occurrence.disposition);
            if dispatch_state == "running" {
                bump(&mut summary, "running");
            }
            match row.attempt_outcome.as_deref() {
                Some("done") => bump(&mut summary, "done"),
                Some("unknown" | "unknown-outcome") => bump(&mut summary, "unknown"),
                Some(_) => bump(&mut summary, "failed"),
                None => {}
            }
            ledger_json(row, &dispatch_state)
        })
        .collect();
    redacted_ok(json!({
        "status": "ok",
        "projection_id": ROUTINE_ARMING_PROJECTION_ID,
        "project_id": project_id,
        "host": { "available": host.available, "reason": host.reason },
        "scheduler": "daemon-tick-only",
        "armings": arming_rows.iter().map(arming_json).collect::<Vec<_>>(),
        "occurrences": occurrences,
        "summary": summary,
        "attempt_history_path": format!(
            "/management/project/v1/dsh.hosted.attempt.list?project_id={project_id}"
        ),
        "attempt_detail_path": "/management/project/v1/dsh.hosted.attempt.detail?attempt_id=",
        "manual_trigger_path": "/management/project/v1/routine.trigger",
        "receipt_is_not_completion": true,
        "verification_status": "not-run",
        "clock_sleep_restart_host_e2e": "not-run",
    }))
}

fn today_overview(method_path: &str, armings: &RoutineArmingStore) -> ResourceApiResponse {
    let period = query_parameter(method_path, "period").unwrap_or_else(|| "today".to_owned());
    match armings.today_overview(&period, now_ms()) {
        Ok(overview) => ok(overview_json(&overview)),
        Err(err) => store_error(err),
    }
}

fn dispatch_state(
    row: &RoutineLedgerRow,
    armings: &[RoutineArming],
    host: &HostDispatchAvailability,
) -> String {
    match row.occurrence.disposition.as_str() {
        "active" if row.attempt_id.is_some() => "running".to_owned(),
        "active" => {
            let live = armings.iter().find(|arming| {
                arming.routine_id == row.occurrence.routine_id
                    && (arming.state == "armed" || arming.state == "paused")
            });
            match live {
                None => "waiting-arming".to_owned(),
                Some(arming) if arming.state == "paused" => "waiting-paused".to_owned(),
                Some(_) if !host.available => "waiting-host".to_owned(),
                Some(_) => "waiting-dispatch".to_owned(),
            }
        }
        other => other.to_owned(),
    }
}

fn bump(summary: &mut Value, key: &str) {
    if let Some(count) = summary.get(key).and_then(Value::as_i64) {
        summary[key] = json!(count + 1);
    }
}

fn arming_json(arming: &RoutineArming) -> Value {
    json!({
        "arming_id": arming.arming_id,
        "project_id": arming.project_id,
        "routine_id": arming.routine_id,
        "revision_id": arming.revision_id,
        "plan_revision_id": arming.plan_revision_id,
        "stage_id": arming.stage_id,
        "employee_id": arming.employee_id,
        "seq": arming.seq,
        "cadence_kind": arming.cadence_kind,
        "interval_ms": arming.interval_ms,
        "attempt_timeout_ms": arming.attempt_timeout_ms,
        "declaration_digest": arming.declaration_digest,
        "context_bytes": arming.bounded_context.len(),
        "armed_after": arming.armed_after,
        "state": arming.state,
        "apply_mode": arming.apply_mode,
        "next_due_at": arming.next_due_at,
        "last_fired_at": arming.last_fired_at,
        "created_at": arming.created_at,
        "updated_at": arming.updated_at,
    })
}

fn ledger_json(row: &RoutineLedgerRow, dispatch_state: &str) -> Value {
    let occurrence = &row.occurrence;
    json!({
        "occurrence_id": occurrence.occurrence_id,
        "routine_id": occurrence.routine_id,
        "revision_id": occurrence.revision_id,
        "project_id": occurrence.project_id,
        "trigger_kind": occurrence.trigger_kind,
        "trigger_source": occurrence.trigger_source,
        "requested_at": occurrence.requested_at,
        "disposition": occurrence.disposition,
        "dispatch_state": dispatch_state,
        "coalesced_by": occurrence.coalesced_by,
        "miss_reason": occurrence.miss_reason,
        "scheduler_task_ref": occurrence.scheduler_task_ref,
        "has_checkpoint": occurrence.checkpoint_json.is_some(),
        "recorded_at": occurrence.recorded_at,
        "arming_id": row.arming_id,
        "attempt_id": row.attempt_id,
        "lease_epoch": row.lease_epoch,
        "started_at": row.started_at,
        "attempt_outcome": row.attempt_outcome,
        "outcome_detail": row.outcome_detail,
        "elapsed_ms": row.elapsed_ms,
        "terminal_at": row.terminal_at,
        "completion_claimed": row.completion_claimed,
        "verification_status": "not-run",
    })
}

fn overview_json(overview: &TodayOverview) -> Value {
    json!({
        "status": "ok",
        "projection_id": TODAY_OVERVIEW_PROJECTION_ID,
        "period": overview.period,
        "period_start_ms": overview.period_start_ms,
        "now_ms": overview.now_ms,
        "period_basis": "utc",
        "counts": {
            "created": overview.created_count,
            "live": overview.live_count,
            "blocked": overview.blocked_count,
        },
        "rows": overview.rows.iter().map(today_row_json).collect::<Vec<_>>(),
        "kpi_wall": false,
        "verification_status": "not-run",
        "cost": "unknown",
    })
}

fn today_row_json(row: &TodayProjectRow) -> Value {
    json!({
        "project_id": row.project_id,
        "state": row.state,
        "status": if row.running_occurrence_id.is_some() {
            "running"
        } else if row.paused_routines > 0 && row.armed_routines == 0 {
            "paused"
        } else if row.armed_routines > 0 {
            "armed"
        } else {
            "idle"
        },
        "armed_routines": row.armed_routines,
        "paused_routines": row.paused_routines,
        "running_occurrence_id": row.running_occurrence_id,
        "running_since": row.running_since,
        "queued_count": row.queued_count,
        "missed_count": row.missed_count,
        "attempts_total": row.attempts_total,
        "attempts_done": row.attempts_done,
        "attempts_failed": row.attempts_failed,
        "attempts_unknown": row.attempts_unknown,
        "duration_ms": row.duration_ms,
        "current_stage_id": row.current_stage_id,
        "current_stage_title": row.current_stage_title,
        "last_terminal_at": row.last_terminal_at,
        "cost": "unknown",
    })
}

// ----------------------------------------------------------------------
// Daemon scheduler tick step
// ----------------------------------------------------------------------

/// What one tick pass did. Counts only; never a completion claim.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct RoutineTickReport {
    pub reconciled: usize,
    pub fired: usize,
    pub missed: usize,
    pub dispatched: usize,
    pub refused: usize,
    pub not_armed: usize,
    pub waiting: usize,
    pub fenced: usize,
    pub promoted: usize,
}

impl RoutineTickReport {
    pub(crate) fn is_quiet(&self) -> bool {
        *self == Self::default()
    }
}

/// One Routine pass inside the daemon scheduler tick. Order matters: observed
/// terminals are written first (so a queued successor can be promoted), then
/// due schedules fire, then undispatched `active` occurrences are leased and
/// launched. Every step is durable before the next; nothing is retried
/// blindly and nothing is dropped silently.
pub(crate) fn run_routine_tick(
    store: &SqliteAuthorityStore,
    host: &HostedAttemptHost,
    repository: &mut SchedulerRepository,
    now_ms: i64,
) -> Result<RoutineTickReport, String> {
    let armings = RoutineArmingStore::from_authority_store(store);
    let attempts = HostedDshAttemptStore::from_authority_store(store);
    let mut report = RoutineTickReport::default();

    // 1. Reconcile in-flight occurrences against daemon-observed Attempt facts.
    for row in armings.in_flight_occurrences().map_err(stringify)? {
        let Some(attempt_id) = row.attempt_id.as_deref() else {
            continue;
        };
        let attempt = attempts.get_attempt(attempt_id).map_err(stringify)?;
        let (outcome, detail, elapsed) = match attempt {
            None => (
                "unknown-outcome".to_owned(),
                "attempt row missing".to_owned(),
                None,
            ),
            Some(ref attempt)
                if attempt.state == "terminal" || attempt.state == "unknown-outcome" =>
            {
                (
                    outcome_from_attempt(attempt).to_owned(),
                    attempt_detail(attempt),
                    attempt.elapsed_ms,
                )
            }
            Some(_) => continue,
        };
        let occurrence_id = row.occurrence.occurrence_id.clone();
        armings
            .record_attempt_terminal(&occurrence_id, &outcome, Some(&detail), elapsed, now_ms)
            .map_err(stringify)?;
        release_lease(
            repository,
            &occurrence_id,
            row.lease_epoch.unwrap_or(0),
            if outcome == "done" {
                SchedulerState::Succeeded
            } else {
                SchedulerState::Failed
            },
            now_ms,
        );
        report.reconciled += 1;
        if armings
            .promote_queued(&row.occurrence.routine_id, now_ms)
            .map_err(stringify)?
            .is_some()
        {
            report.promoted += 1;
        }
    }

    // 2. Fire due schedule triggers through the P11-T08 admission path.
    let host_state = armings.host_dispatch_availability().map_err(stringify)?;
    for arming in armings.due_schedule_armings(now_ms).map_err(stringify)? {
        match armings.fire_schedule(&arming, &host_state, now_ms) {
            Ok(occurrence) if occurrence.disposition == "missed" => report.missed += 1,
            Ok(_) => report.fired += 1,
            Err(ProjectAggregateError::Stale { .. }) => {
                // The Routine moved to a newer revision without an instruction
                // on this arming; nothing fires from a stale declaration.
                report.refused += 1;
            }
            Err(err) => return Err(stringify(err)),
        }
    }

    // 3. Lease and launch undispatched active occurrences.
    for row in armings.dispatchable_occurrences().map_err(stringify)? {
        let occurrence = &row.occurrence;
        let Some(arming) = armings
            .live_arming(&occurrence.routine_id)
            .map_err(stringify)?
        else {
            armings
                .mark_not_armed(&occurrence.occurrence_id, now_ms)
                .map_err(stringify)?;
            report.not_armed += 1;
            continue;
        };
        if arming.state == "paused" || !host_state.available {
            report.waiting += 1;
            continue;
        }
        let Some(task_ref) = occurrence.scheduler_task_ref.clone() else {
            report.waiting += 1;
            continue;
        };
        let key = SchedulerWorkKey {
            task_ref,
            contract_epoch: 1,
        };
        let Some(scheduler_row) = repository.load(&key).map_err(|err| err.to_string())? else {
            armings
                .record_attempt_terminal(
                    &occurrence.occurrence_id,
                    "spawn-failed",
                    Some("scheduler row missing for the occurrence"),
                    None,
                    now_ms,
                )
                .map_err(stringify)?;
            report.refused += 1;
            continue;
        };
        let lease_epoch = scheduler_row.lease_epoch.saturating_add(1);
        let expires_ms = now_ms
            .saturating_add(arming.attempt_timeout_ms)
            .saturating_add(LEASE_SLACK_MS);
        match repository.acquire_eligible_lease(
            &key,
            ROUTINE_SCHEDULER_LEASE_OWNER,
            lease_epoch,
            &canonical_timestamp_from_ms(now_ms),
            &canonical_timestamp_from_ms(expires_ms),
        ) {
            Ok(_) => {}
            Err(SchedulerRepositoryError::LeaseConflict(_)) => {
                report.fenced += 1;
                continue;
            }
            Err(err) => return Err(err.to_string()),
        }
        let context = format!(
            "{}\n\n[routine occurrence {} · routine revision {} · trigger {}/{} · stage {}]",
            arming.bounded_context,
            occurrence.occurrence_id,
            occurrence.revision_id,
            occurrence.trigger_kind,
            occurrence.trigger_source,
            arming.stage_id
        );
        let timeout_ms = u64::try_from(arming.attempt_timeout_ms).unwrap_or(120_000);
        match launch_hosted_attempt(
            store,
            host,
            &HostedAttemptLaunch {
                employee_id: arming.employee_id.clone(),
                employee_revision_id: None,
                task_ref: key.task_ref.clone(),
                bounded_context: context,
                timeout: Duration::from_millis(timeout_ms),
            },
        ) {
            Ok(launched) => {
                drop(launched.handle);
                armings
                    .bind_attempt(
                        &occurrence.occurrence_id,
                        &arming.arming_id,
                        &launched.row.attempt_id,
                        lease_epoch,
                        now_ms,
                    )
                    .map_err(stringify)?;
                report.dispatched += 1;
            }
            Err(response) => {
                let detail = refusal_detail(&response);
                armings
                    .record_attempt_terminal(
                        &occurrence.occurrence_id,
                        "spawn-failed",
                        Some(&detail),
                        None,
                        now_ms,
                    )
                    .map_err(stringify)?;
                release_lease(
                    repository,
                    &occurrence.occurrence_id,
                    lease_epoch,
                    SchedulerState::Failed,
                    now_ms,
                );
                report.refused += 1;
                if armings
                    .promote_queued(&occurrence.routine_id, now_ms)
                    .map_err(stringify)?
                    .is_some()
                {
                    report.promoted += 1;
                }
            }
        }
    }
    Ok(report)
}

fn release_lease(
    repository: &mut SchedulerRepository,
    occurrence_id: &str,
    lease_epoch: i64,
    state: SchedulerState,
    now_ms: i64,
) {
    let key = SchedulerWorkKey {
        task_ref: cognitive_store::routine_scheduler_task_ref(occurrence_id),
        contract_epoch: 1,
    };
    if let Err(err) = repository.release_lease(
        &key,
        ROUTINE_SCHEDULER_LEASE_OWNER,
        lease_epoch,
        state,
        &canonical_timestamp_from_ms(now_ms),
    ) {
        eprintln!(
            "kernel-server personal routine tick: lease release for {occurrence_id} refused: {err}"
        );
    }
}

fn outcome_from_attempt(attempt: &HostedAttemptRow) -> &'static str {
    match attempt.terminal_kind.as_str() {
        "exited" => match attempt.response_status.as_str() {
            "done" => "done",
            "failed" => "failed",
            "blocked" => "blocked",
            _ => "unknown",
        },
        "timed-out" => "timed-out",
        "signaled" => "signaled",
        "spawn-failed" => "spawn-failed",
        "unknown-outcome" => "unknown-outcome",
        _ => "unknown",
    }
}

fn attempt_detail(attempt: &HostedAttemptRow) -> String {
    format!(
        "attempt {} terminal_kind={} exit_code={} response_status={} completion_claimed={} verification_status={}",
        attempt.attempt_id,
        attempt.terminal_kind,
        attempt
            .exit_code
            .map(|code| code.to_string())
            .unwrap_or_else(|| "none".to_owned()),
        attempt.response_status,
        attempt.completion_claimed,
        attempt.verification_status
    )
}

fn refusal_detail(response: &ResourceApiResponse) -> String {
    let parsed: Option<Value> = serde_json::from_str(&response.body).ok();
    let code = parsed
        .as_ref()
        .and_then(|value| value.get("code"))
        .and_then(Value::as_str)
        .unwrap_or("HOSTED_ATTEMPT_REFUSED");
    let message = parsed
        .as_ref()
        .and_then(|value| value.get("message"))
        .and_then(Value::as_str)
        .unwrap_or("");
    format!("http {} {code}: {message}", response.status)
}

fn stringify(err: ProjectAggregateError) -> String {
    err.to_string()
}

// ----------------------------------------------------------------------
// Shared helpers
// ----------------------------------------------------------------------

fn parse_json(body: &[u8]) -> Option<Value> {
    serde_json::from_slice(body).ok()
}

fn query_parameter(method_path: &str, name: &str) -> Option<String> {
    let (_, query) = method_path.split_once('?')?;
    for pair in query.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        if key == name {
            return Some(value.split_whitespace().next().unwrap_or(value).to_owned());
        }
    }
    None
}

fn now_ms() -> i64 {
    cognitive_store::now_ms()
}

fn ok(body: Value) -> ResourceApiResponse {
    ResourceApiResponse {
        status: 200,
        body: body.to_string(),
        content_type: "application/json",
    }
}

fn redacted_ok(body: Value) -> ResourceApiResponse {
    let serialized = body.to_string();
    let lowered = serialized.to_ascii_lowercase();
    if lowered.contains("sk-live")
        || lowered.contains("\"sess-")
        || lowered.contains("boot-")
        || lowered.contains("ssv1:")
        || lowered.contains("secretref:")
    {
        return error(
            500,
            "ROUTINE_RUNS_REDACTION",
            "runs projection redaction failed",
        );
    }
    ResourceApiResponse {
        status: 200,
        body: serialized,
        content_type: "application/json",
    }
}

fn error(status: u16, code: &str, message: &str) -> ResourceApiResponse {
    ResourceApiResponse {
        status,
        body: json!({"status":"error","code": code, "message": message}).to_string(),
        content_type: "application/json",
    }
}

fn store_error(err: ProjectAggregateError) -> ResourceApiResponse {
    match err {
        ProjectAggregateError::Forbidden { detail } => {
            error(403, "ROUTINE_ARMING_FORBIDDEN", detail)
        }
        ProjectAggregateError::NotFound { detail } => {
            error(404, "ROUTINE_ARMING_NOT_FOUND", detail)
        }
        ProjectAggregateError::Conflict { detail } => error(409, "ROUTINE_ARMING_CONFLICT", detail),
        ProjectAggregateError::Stale { detail } => error(409, "ROUTINE_ARMING_STALE", detail),
        ProjectAggregateError::Unconfirmed { detail } => {
            error(409, "ROUTINE_ARM_BEFORE_G2", detail)
        }
        ProjectAggregateError::Rejected { detail } => error(422, "ROUTINE_ARMING_REJECTED", detail),
        ProjectAggregateError::Invalid { detail } => error(422, "ROUTINE_ARMING_INVALID", detail),
        ProjectAggregateError::Unavailable { .. } => {
            error(503, "ROUTINE_ARMING_UNAVAILABLE", "store unavailable")
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use cognitive_runtime::{HOSTED_DSH_CONFIG_FILE_NAME, HOSTED_DSH_REVISION_FILE_NAME};
    use cognitive_store::{
        EmployeeStore, HOSTED_DSH_ARTIFACT_DIGEST, HostedDshPlane, PersonalDataLayout,
        ProjectAggregateStore, RosterProposal, RoutineRevisionSpec, RoutineStore,
        RoutineTriggerSpec, SeatingFacts, StageSpec, StageTestOracle, prepare_personal_databases,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::thread;
    use std::time::{Duration, Instant};
    use tempfile::TempDir;

    struct Harness {
        _tmp: TempDir,
        store: SqliteAuthorityStore,
        host: HostedAttemptHost,
        authority_path: PathBuf,
        project_id: String,
        employee_id: String,
    }

    fn stage(id: &str, title: &str, slot: &str) -> StageSpec {
        StageSpec {
            stage_id: id.to_owned(),
            title: title.to_owned(),
            objective: format!("{title} objective"),
            output_contract_digest: ProjectAggregateStore::digest_hex(
                format!("out-{id}").as_bytes(),
            ),
            acceptance_spec_ref: Some(format!("cas:spec-{id}")),
            cadence_json: Some(r#"{"kind":"manual"}"#.to_owned()),
            responsible_slot: slot.to_owned(),
            blocking_gap: None,
        }
    }

    /// Fake exact-artifact child: reads the request frame and answers `done`
    /// (or crashes) after echoing the context digest.
    const FAKE_CHILD: &str = r#"
let data = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => { data += chunk; });
process.stdin.on("end", () => {
  const request = JSON.parse(data.trim().split("\n")[0]);
  const emit = (frame) => process.stdout.write(JSON.stringify(frame) + "\n");
  emit({ frame: "observation", text: "child.started digest:" + request.context_digest });
  emit({ frame: "heartbeat" });
  emit({ frame: "candidate", operation: "DeliverableDraft", payload: { text: "Summary: " + request.context.slice(0, 40) } });
  const mode = (request.context.match(/mode=(\w+)/) || [])[1] || "done";
  if (mode === "crash") { process.exit(9); }
  emit({ frame: "response", status: mode === "fail" ? "failed" : "done" });
  process.exit(0);
});
"#;

    fn pass_stage(store: &ProjectAggregateStore, project_id: &str, plan_id: &str, stage_id: &str) {
        let ring = store.get_stage(plan_id, stage_id).unwrap().unwrap();
        store
            .confirm_stage(
                ConfirmCaller::OwnerManagement,
                project_id,
                plan_id,
                stage_id,
                &ring.stage_digest,
            )
            .unwrap();
        store
            .derive_stage_test_passed(&StageTestOracle {
                project_id: project_id.to_owned(),
                plan_revision_id: plan_id.to_owned(),
                stage_id: stage_id.to_owned(),
                task_ref: format!("task://personal/{stage_id}"),
                seating: SeatingFacts { seated: true },
                verification_current: true,
                verification_report_ref: format!("cas:report-{stage_id}"),
                openable: true,
                checks_passed: true,
                effects_closed: true,
                now_ms: 60,
            })
            .unwrap();
    }

    fn harness(accept: bool) -> Harness {
        let temporary = TempDir::new().unwrap();
        let root = temporary.path();
        let layout = PersonalDataLayout::from_xdg_roots(
            root.join("config"),
            root.join("data"),
            root.join("state"),
            root.join("cache"),
            root.join("runtime"),
        );
        prepare_personal_databases(&layout).unwrap();
        let authority_path = layout.authority_database_path();
        let store = SqliteAuthorityStore::open(&authority_path).unwrap();
        let dsh_root = root.join("dsh");
        let adapter_root = root.join("adapter");
        fs::create_dir_all(&dsh_root).unwrap();
        fs::create_dir_all(adapter_root.join("scripts")).unwrap();
        fs::write(
            dsh_root.join(HOSTED_DSH_REVISION_FILE_NAME),
            format!("{HOSTED_DSH_ARTIFACT_DIGEST}\n"),
        )
        .unwrap();
        fs::write(
            adapter_root.join("scripts/hosted-attempt-child.mjs"),
            FAKE_CHILD,
        )
        .unwrap();
        fs::create_dir_all(layout.config_dir()).unwrap();
        fs::write(
            layout.config_dir().join(HOSTED_DSH_CONFIG_FILE_NAME),
            json!({
                "schema_version": 1,
                "surface": "personal-dsh-config",
                "dsh_root": dsh_root.display().to_string(),
                "adapter_root": adapter_root.display().to_string(),
                "revision": HOSTED_DSH_ARTIFACT_DIGEST,
                "adapter_id": "deepseek.dsh.akp",
                "candidate_only": true,
            })
            .to_string(),
        )
        .unwrap();
        fs::create_dir_all(layout.state_dir()).unwrap();
        fs::write(
            layout.state_dir().join("daemon-endpoint.json"),
            json!({"schema_version": 1, "endpoint": "127.0.0.1:48181", "surface": "personal-daemon-endpoint"}).to_string(),
        )
        .unwrap();
        let bootstrap = layout.local_bootstrap_secret_path();
        fs::create_dir_all(bootstrap.parent().unwrap()).unwrap();
        fs::write(&bootstrap, "boot-test-not-real\n").unwrap();
        let host = HostedAttemptHost::from_layout(&layout);

        let projects = ProjectAggregateStore::from_authority_store(&store);
        let employees = EmployeeStore::from_authority_store(&store);
        let (draft_id, _) = projects.create_draft(b"charter-v1", 10).unwrap();
        projects
            .put_draft_charter(&draft_id, b"charter-body-v1", 11)
            .unwrap();
        let (preview_id, preview_digest) = projects
            .request_preview("activation", &draft_id, b"activation-preview", 12)
            .unwrap();
        let project_id = projects
            .confirm_preview(
                ConfirmCaller::OwnerManagement,
                &preview_id,
                &preview_digest,
                13,
            )
            .unwrap()
            .new_ref;
        let plan_id = projects
            .apply_plan_revision(
                &project_id,
                &project_id,
                &[
                    stage("s1", "Manage", "manager"),
                    stage("s2", "Research", "researcher"),
                ],
                20,
            )
            .unwrap();
        let ids = employees
            .register_roster(
                ConfirmCaller::OwnerManagement,
                &project_id,
                &plan_id,
                &[
                    RosterProposal {
                        slot: "manager".to_owned(),
                        specialization: "project-manager".to_owned(),
                        prompt: "coordinate".to_owned(),
                        tools_declared: vec!["workspace-write".to_owned()],
                    },
                    RosterProposal {
                        slot: "researcher".to_owned(),
                        specialization: "member".to_owned(),
                        prompt: "research".to_owned(),
                        tools_declared: vec!["workspace-write".to_owned()],
                    },
                ],
                21,
            )
            .unwrap();
        employees
            .request_seating(ConfirmCaller::OwnerManagement, &ids[0], 30)
            .unwrap();
        employees
            .confirm_seating(
                ConfirmCaller::OwnerManagement,
                &ids[0],
                Some("flash"),
                true,
                31,
            )
            .unwrap();
        if accept {
            pass_stage(&projects, &project_id, &plan_id, "s1");
            pass_stage(&projects, &project_id, &plan_id, "s2");
            let (preview_id, preview_digest) = projects
                .request_preview("acceptance", &project_id, b"g2-ok", 80)
                .unwrap();
            projects
                .confirm_preview(
                    ConfirmCaller::OwnerManagement,
                    &preview_id,
                    &preview_digest,
                    81,
                )
                .unwrap();
        }
        Harness {
            _tmp: temporary,
            store,
            host,
            authority_path,
            project_id,
            employee_id: ids[0].clone(),
        }
    }

    fn publish(harness: &Harness, body: &str) -> (String, String) {
        let published = RoutineStore::from_authority_store(&harness.store)
            .publish_revision(
                ConfirmCaller::OwnerManagement,
                &RoutineRevisionSpec {
                    project_id: &harness.project_id,
                    routine_id: None,
                    body_json: body,
                    risk_class: "internal",
                    now_ms: now_ms(),
                },
            )
            .unwrap();
        (published.routine_id, published.revision_id)
    }

    fn trigger(harness: &Harness, routine_id: &str, revision_id: &str) -> String {
        RoutineStore::from_authority_store(&harness.store)
            .admit_trigger(
                ConfirmCaller::OwnerManagement,
                &RoutineTriggerSpec {
                    routine_id,
                    revision_id,
                    trigger_kind: "manual",
                    trigger_source: "owner-run",
                    force_parallel: false,
                    host_unavailable: false,
                    now_ms: now_ms(),
                },
            )
            .unwrap()
            .occurrence_id
    }

    fn arm_body(harness: &Harness, routine_id: &str, revision_id: &str) -> Vec<u8> {
        json!({
            "project_id": harness.project_id,
            "routine_id": routine_id,
            "revision_id": revision_id,
            "stage_id": "s1",
            "employee_id": harness.employee_id,
        })
        .to_string()
        .into_bytes()
    }

    fn tick(harness: &Harness, repository: &mut SchedulerRepository) -> RoutineTickReport {
        run_routine_tick(&harness.store, &harness.host, repository, now_ms()).unwrap()
    }

    fn runs(harness: &Harness) -> Value {
        let response = handle(
            &format!(
                "GET /management/project/v1/routine.runs?project_id={}",
                harness.project_id
            ),
            b"",
            &harness.store,
        );
        assert_eq!(response.status, 200, "{}", response.body);
        serde_json::from_str(&response.body).unwrap()
    }

    #[test]
    fn routine_runs_task_channel_forbidden_arm_before_g2_refused_and_reads_are_honest_when_empty() {
        let harness = harness(false);
        for route in [
            "POST /task/project/v1/routine.arm",
            "POST /task/project/v1/routine.instruction",
            "GET /task/project/v1/routine.runs?project_id=x",
            "GET /task/project/v1/today.overview",
        ] {
            let forbidden = handle(route, b"{}", &harness.store);
            assert_eq!(forbidden.status, 403, "{route}");
            assert!(forbidden.body.contains("ROUTINE_RUNS_CHANNEL_FORBIDDEN"));
        }
        let (routine_id, revision_id) = publish(&harness, r#"{"cadence":"manual"}"#);
        let before = handle(
            "POST /management/project/v1/routine.arm",
            &arm_body(&harness, &routine_id, &revision_id),
            &harness.store,
        );
        assert_eq!(before.status, 409, "{}", before.body);
        assert!(before.body.contains("ROUTINE_ARM_BEFORE_G2"));
        let missing = handle(
            "POST /management/project/v1/routine.arm",
            br#"{"project_id":"p"}"#,
            &harness.store,
        );
        assert_eq!(missing.status, 400);

        // A manual trigger before arming is Intent, not a run: the tick keeps
        // it visible as missed/not-armed and never dispatches.
        let orphan = trigger(&harness, &routine_id, &revision_id);
        let mut repository = SchedulerRepository::open(&harness.authority_path).unwrap();
        let report = tick(&harness, &mut repository);
        assert_eq!(report.not_armed, 1);
        assert_eq!(report.dispatched, 0);
        let runs = runs(&harness);
        let occurrences = runs["occurrences"].as_array().unwrap();
        assert_eq!(occurrences.len(), 1);
        assert_eq!(occurrences[0]["occurrence_id"], orphan);
        assert_eq!(occurrences[0]["disposition"], "missed");
        assert_eq!(occurrences[0]["miss_reason"], "not-armed");
        assert_eq!(runs["summary"]["missed"], 1);
        assert_eq!(runs["scheduler"], "daemon-tick-only");
        assert_eq!(runs["receipt_is_not_completion"], true);
        assert!(runs["armings"].as_array().unwrap().is_empty());
        assert!(
            HostedDshAttemptStore::from_authority_store(&harness.store)
                .list_attempts(&harness.project_id, 10)
                .unwrap()
                .is_empty(),
            "no Attempt may exist for an un-armed Routine"
        );

        let overview = handle(
            "GET /management/project/v1/today.overview?period=today",
            b"",
            &harness.store,
        );
        assert_eq!(overview.status, 200, "{}", overview.body);
        let overview: Value = serde_json::from_str(&overview.body).unwrap();
        assert_eq!(overview["counts"]["created"], 1);
        assert_eq!(overview["counts"]["live"], 0);
        assert!(overview["rows"].as_array().unwrap().is_empty());
        assert_eq!(overview["kpi_wall"], false);
        assert_eq!(overview["cost"], "unknown");
        let bad_period = handle(
            "GET /management/project/v1/today.overview?period=year",
            b"",
            &harness.store,
        );
        assert_eq!(bad_period.status, 422);
    }

    #[test]
    fn routine_tick_drives_a_hosted_attempt_and_writes_the_occurrence_ledger() {
        let harness = harness(true);
        let (routine_id, revision_id) = publish(
            &harness,
            r#"{"cadence":"manual","bounded_context":"mode=done summarize README","attempt_timeout_ms":20000}"#,
        );
        let armed = handle(
            "POST /management/project/v1/routine.arm",
            &arm_body(&harness, &routine_id, &revision_id),
            &harness.store,
        );
        assert_eq!(armed.status, 200, "{}", armed.body);
        let armed: Value = serde_json::from_str(&armed.body).unwrap();
        assert_eq!(armed["arming"]["state"], "armed");
        assert_eq!(armed["arming"]["armed_after"], "G2");
        let arming_id = armed["arming"]["arming_id"].as_str().unwrap().to_owned();

        let first = trigger(&harness, &routine_id, &revision_id);
        let mut repository = SchedulerRepository::open(&harness.authority_path).unwrap();
        let armings = RoutineArmingStore::from_authority_store(&harness.store);
        let attempts = HostedDshAttemptStore::from_authority_store(&harness.store);

        if HostedDshPlane::isolated_spawn_is_fenced() {
            let report = tick(&harness, &mut repository);
            assert_eq!(report.refused, 1);
            let row = armings.get_ledger_row(&first).unwrap();
            assert_eq!(row.occurrence.disposition, "attempted");
            assert_eq!(row.attempt_outcome.as_deref(), Some("spawn-failed"));
            assert!(
                row.outcome_detail
                    .as_deref()
                    .unwrap()
                    .contains("DEV-WIN-GNU-01")
            );
            assert!(!row.completion_claimed);
            return;
        }

        // Tick 1: lease + launch. The occurrence is bound to a real Attempt.
        let report = tick(&harness, &mut repository);
        assert_eq!(report.dispatched, 1, "{report:?}");
        let running = armings.get_ledger_row(&first).unwrap();
        assert_eq!(running.occurrence.disposition, "active");
        let attempt_id = running.attempt_id.clone().expect("attempt bound");
        assert_eq!(running.arming_id.as_deref(), Some(arming_id.as_str()));
        let lease_epoch = running.lease_epoch.unwrap();
        let scheduler_row = repository
            .load(&SchedulerWorkKey {
                task_ref: cognitive_store::routine_scheduler_task_ref(&first),
                contract_epoch: 1,
            })
            .unwrap()
            .unwrap();
        assert_eq!(scheduler_row.state, "leased");
        assert_eq!(scheduler_row.lease_epoch, lease_epoch);
        assert_eq!(
            scheduler_row.lease_owner.as_deref(),
            Some(ROUTINE_SCHEDULER_LEASE_OWNER)
        );
        let runs_now = runs(&harness);
        let running_json = &runs_now["occurrences"][0];
        assert_eq!(running_json["dispatch_state"], "running");
        assert_eq!(running_json["attempt_id"], attempt_id);

        // A second trigger while running queues; a second tick never launches
        // a second Attempt for the same Routine (overlap).
        let second = trigger(&harness, &routine_id, &revision_id);
        assert_eq!(
            armings
                .get_ledger_row(&second)
                .unwrap()
                .occurrence
                .disposition,
            "queued"
        );
        let second_pass = tick(&harness, &mut repository);
        assert_eq!(second_pass.dispatched, 0);
        assert_eq!(
            attempts
                .list_attempts(&harness.project_id, 10)
                .unwrap()
                .len(),
            1
        );

        // Wait for the daemon-observed terminal, then reconcile.
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            let attempt = attempts.get_attempt(&attempt_id).unwrap().unwrap();
            if attempt.state == "terminal" {
                assert_eq!(attempt.response_status, "done");
                assert!(!attempt.completion_claimed);
                assert_eq!(attempt.verification_status, "not-run");
                assert_eq!(
                    attempt.task_ref,
                    cognitive_store::routine_scheduler_task_ref(&first)
                );
                break;
            }
            assert!(Instant::now() < deadline, "attempt never reached terminal");
            thread::sleep(Duration::from_millis(100));
        }
        let reconcile = tick(&harness, &mut repository);
        assert_eq!(reconcile.reconciled, 1, "{reconcile:?}");
        assert_eq!(reconcile.promoted, 1, "{reconcile:?}");
        assert_eq!(reconcile.dispatched, 1, "{reconcile:?}");
        let attempted = armings.get_ledger_row(&first).unwrap();
        assert_eq!(attempted.occurrence.disposition, "attempted");
        assert_eq!(attempted.attempt_outcome.as_deref(), Some("done"));
        assert!(!attempted.completion_claimed);
        assert!(attempted.elapsed_ms.is_some());
        let released = repository
            .load(&SchedulerWorkKey {
                task_ref: cognitive_store::routine_scheduler_task_ref(&first),
                contract_epoch: 1,
            })
            .unwrap()
            .unwrap();
        assert_eq!(released.state, "succeeded");
        assert!(released.lease_owner.is_none());
        let promoted = armings.get_ledger_row(&second).unwrap();
        assert_eq!(promoted.occurrence.disposition, "active");
        assert!(
            promoted.attempt_id.is_some(),
            "promoted occurrence dispatched"
        );
        assert_eq!(
            attempts
                .list_attempts(&harness.project_id, 10)
                .unwrap()
                .len(),
            2
        );

        // Pause instruction while the second runs: the running Attempt keeps its
        // context digest; a third trigger waits behind the pause.
        let second_attempt_id = promoted.attempt_id.clone().unwrap();
        let digest_before = attempts
            .get_attempt(&second_attempt_id)
            .unwrap()
            .unwrap()
            .context_digest;
        let paused = handle(
            "POST /management/project/v1/routine.instruction",
            &json!({
                "arming_id": arming_id,
                "revision_id": revision_id,
                "apply": "pause",
            })
            .to_string()
            .into_bytes(),
            &harness.store,
        );
        assert_eq!(paused.status, 200, "{}", paused.body);
        let paused: Value = serde_json::from_str(&paused.body).unwrap();
        assert_eq!(paused["arming"]["state"], "paused");
        assert_eq!(paused["running_prompt_injected"], false);
        assert_eq!(paused["active_occurrence_id"], second);
        let second_attempt = attempts.get_attempt(&second_attempt_id).unwrap().unwrap();
        assert_eq!(
            second_attempt.context_digest, digest_before,
            "an instruction never rewrites the running Attempt's context"
        );
        assert_eq!(
            armings
                .get_ledger_row(&second)
                .unwrap()
                .occurrence
                .revision_id,
            revision_id
        );
        let deadline = Instant::now() + Duration::from_secs(30);
        while attempts
            .get_attempt(&second_attempt_id)
            .unwrap()
            .unwrap()
            .state
            != "terminal"
        {
            assert!(Instant::now() < deadline, "second attempt never terminal");
            thread::sleep(Duration::from_millis(100));
        }
        let third = trigger(&harness, &routine_id, &revision_id);
        let after_pause = tick(&harness, &mut repository);
        assert_eq!(after_pause.reconciled, 1, "{after_pause:?}");
        assert_eq!(after_pause.dispatched, 0, "paused arming must not dispatch");
        assert!(after_pause.waiting >= 1, "{after_pause:?}");
        let runs_paused = runs(&harness);
        let third_json = runs_paused["occurrences"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["occurrence_id"] == third)
            .unwrap()
            .clone();
        assert_eq!(third_json["dispatch_state"], "waiting-paused");
        assert_eq!(runs_paused["summary"]["attempted"], 2);
        assert_eq!(runs_paused["summary"]["done"], 2);
        assert!(
            runs_paused["attempt_history_path"]
                .as_str()
                .unwrap()
                .contains("dsh.hosted.attempt.list")
        );
        assert!(!runs_paused.to_string().contains("boot-test-not-real"));

        // Resume → the waiting occurrence dispatches on the next tick.
        let resumed = handle(
            "POST /management/project/v1/routine.arming.resume",
            &json!({"arming_id": paused["arming"]["arming_id"]})
                .to_string()
                .into_bytes(),
            &harness.store,
        );
        assert_eq!(resumed.status, 200, "{}", resumed.body);
        let after_resume = tick(&harness, &mut repository);
        assert_eq!(after_resume.dispatched, 1, "{after_resume:?}");

        // Today overview: one live row, attempts counted, never completion.
        let overview = handle(
            "GET /management/project/v1/today.overview?period=today",
            b"",
            &harness.store,
        );
        let overview: Value = serde_json::from_str(&overview.body).unwrap();
        assert_eq!(overview["counts"]["live"], 1);
        let row = &overview["rows"][0];
        assert_eq!(row["project_id"], harness.project_id);
        assert_eq!(row["attempts_done"], 2);
        assert_eq!(row["status"], "running");
        assert_eq!(row["current_stage_id"], "s1");
        assert_eq!(overview["verification_status"], "not-run");
    }
}
