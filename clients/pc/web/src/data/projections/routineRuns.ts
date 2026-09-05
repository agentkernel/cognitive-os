/**
 * Personal-private Routine runs projection (P13-T05/D02 Dual Track).
 * Sources are GET /management/project/v1/routine.runs (armings + occurrence
 * ledger written by the daemon scheduler tick) and
 * GET /management/project/v1/dsh.hosted.attempt.list (real Attempt history).
 * This file never invents an occurrence, a Start control, or a completion:
 * every row carries the daemon's `completion_claimed = false` and
 * `verification_status = not-run` as stated facts.
 */

import { asList, asRecord } from "../projections";

export const ROUTINE_RUNS_PATH = "/management/project/v1/routine.runs";
export const ATTEMPT_LIST_PATH = "/management/project/v1/dsh.hosted.attempt.list";
export const ATTEMPT_RUN_PATH = "/management/project/v1/dsh.hosted.attempt.run";
export const ROUTINE_TRIGGER_PATH = "/management/project/v1/routine.trigger";

export function routineRunsPath(projectId: string): string {
  return `${ROUTINE_RUNS_PATH}?project_id=${encodeURIComponent(projectId)}`;
}

export function attemptListPath(projectId: string): string {
  return `${ATTEMPT_LIST_PATH}?project_id=${encodeURIComponent(projectId)}`;
}

export function routineRunsKey(projectId: string): string {
  return `opc:routine-runs:${projectId}`;
}

export function attemptHistoryKey(projectId: string): string {
  return `opc:attempt-history:${projectId}`;
}

export interface RoutineArmingRow {
  armingId: string;
  routineId: string;
  revisionId: string;
  stageId: string;
  employeeId: string;
  cadenceKind: string;
  intervalMs: string;
  state: string;
  applyMode: string;
  armedAfter: string;
  nextDueAt: string;
  lastFiredAt: string;
}

export interface RoutineOccurrenceRow {
  occurrenceId: string;
  routineId: string;
  revisionId: string;
  triggerKind: string;
  triggerSource: string;
  requestedAt: string;
  disposition: string;
  dispatchState: string;
  coalescedBy: string;
  missReason: string;
  attemptId: string;
  attemptOutcome: string;
  outcomeDetail: string;
  elapsedMs: string;
  terminalAt: string;
  /** Daemon-stated. A `true` here would be a contract violation, shown as-is. */
  completionClaimed: string;
  verificationStatus: string;
}

export interface RoutineRunsSummary {
  active: string;
  running: string;
  queued: string;
  missed: string;
  coalesced: string;
  attempted: string;
  done: string;
  failed: string;
  unknown: string;
}

export interface RoutineRunsView {
  projectId: string;
  hostAvailable: string;
  hostReason: string;
  scheduler: string;
  armings: RoutineArmingRow[];
  occurrences: RoutineOccurrenceRow[];
  summary: RoutineRunsSummary;
  attemptHistoryPath: string;
  manualTriggerPath: string;
  receiptIsNotCompletion: string;
  verificationStatus: string;
  clockSleepRestartHostE2e: string;
}

export interface AttemptHistoryRow {
  attemptId: string;
  employeeId: string;
  taskRef: string;
  state: string;
  terminalKind: string;
  exitCode: string;
  responseStatus: string;
  completionClaimed: string;
  verificationStatus: string;
  elapsedMs: string;
  createdAt: string;
  terminalAt: string;
}

function stated(value: unknown, fallback = "unknown"): string {
  if (typeof value === "string" && value.length > 0) {
    return value;
  }
  if (typeof value === "number" && Number.isFinite(value)) {
    return String(value);
  }
  if (typeof value === "boolean") {
    return String(value);
  }
  return fallback;
}

/** Counts are daemon-stated integers; anything else is `unknown`, never 0. */
function statedCount(value: unknown): string {
  return typeof value === "number" && Number.isInteger(value) && value >= 0
    ? String(value)
    : "unknown";
}

function projectArming(item: unknown): RoutineArmingRow | undefined {
  const record = asRecord(item);
  if (typeof record.arming_id !== "string" || record.arming_id.length === 0) {
    return undefined;
  }
  return {
    armingId: record.arming_id,
    routineId: stated(record.routine_id),
    revisionId: stated(record.revision_id),
    stageId: stated(record.stage_id),
    employeeId: stated(record.employee_id),
    cadenceKind: stated(record.cadence_kind),
    intervalMs: stated(record.interval_ms, "—"),
    state: stated(record.state),
    applyMode: stated(record.apply_mode),
    armedAfter: stated(record.armed_after),
    nextDueAt: stated(record.next_due_at, "—"),
    lastFiredAt: stated(record.last_fired_at, "—"),
  };
}

function projectOccurrence(item: unknown): RoutineOccurrenceRow | undefined {
  const record = asRecord(item);
  if (typeof record.occurrence_id !== "string" || record.occurrence_id.length === 0) {
    return undefined;
  }
  return {
    occurrenceId: record.occurrence_id,
    routineId: stated(record.routine_id),
    revisionId: stated(record.revision_id),
    triggerKind: stated(record.trigger_kind),
    triggerSource: stated(record.trigger_source),
    requestedAt: stated(record.requested_at),
    disposition: stated(record.disposition),
    dispatchState: stated(record.dispatch_state),
    coalescedBy: stated(record.coalesced_by, "—"),
    missReason: stated(record.miss_reason, "—"),
    attemptId: stated(record.attempt_id, "—"),
    attemptOutcome: stated(record.attempt_outcome, "—"),
    outcomeDetail: stated(record.outcome_detail, "—"),
    elapsedMs: stated(record.elapsed_ms, "—"),
    terminalAt: stated(record.terminal_at, "—"),
    completionClaimed: stated(record.completion_claimed),
    verificationStatus: stated(record.verification_status),
  };
}

function projectSummary(value: unknown): RoutineRunsSummary {
  const record = asRecord(value);
  return {
    active: statedCount(record.active),
    running: statedCount(record.running),
    queued: statedCount(record.queued),
    missed: statedCount(record.missed),
    coalesced: statedCount(record.coalesced),
    attempted: statedCount(record.attempted),
    done: statedCount(record.done),
    failed: statedCount(record.failed),
    unknown: statedCount(record.unknown),
  };
}

/**
 * One view row (not a list) so a Project with zero occurrences still reads
 * `ready`: the honest empty ledger is the daemon's answer, not a missing one.
 */
export function projectRoutineRuns(body: unknown): RoutineRunsView[] {
  const record = asRecord(body);
  const projectId = stated(record.project_id, "");
  if (projectId.length === 0) {
    return [];
  }
  const host = asRecord(record.host);
  return [
    {
      projectId,
      hostAvailable: stated(host.available),
      hostReason: stated(host.reason, "—"),
      scheduler: stated(record.scheduler),
      armings: asList(body, ["armings"])
        .map(projectArming)
        .filter((row): row is RoutineArmingRow => row !== undefined),
      occurrences: asList(body, ["occurrences"])
        .map(projectOccurrence)
        .filter((row): row is RoutineOccurrenceRow => row !== undefined),
      summary: projectSummary(record.summary),
      attemptHistoryPath: stated(record.attempt_history_path, ATTEMPT_LIST_PATH),
      manualTriggerPath: stated(record.manual_trigger_path, ROUTINE_TRIGGER_PATH),
      receiptIsNotCompletion: stated(record.receipt_is_not_completion),
      verificationStatus: stated(record.verification_status),
      clockSleepRestartHostE2e: stated(record.clock_sleep_restart_host_e2e, "not-run"),
    },
  ];
}

export function projectAttemptHistory(body: unknown): AttemptHistoryRow[] {
  const rows: AttemptHistoryRow[] = [];
  for (const item of asList(body, ["attempts"])) {
    const record = asRecord(item);
    if (typeof record.attempt_id !== "string" || record.attempt_id.length === 0) {
      continue;
    }
    rows.push({
      attemptId: record.attempt_id,
      employeeId: stated(record.employee_id),
      taskRef: stated(record.task_ref),
      state: stated(record.state),
      terminalKind: stated(record.terminal_kind, "—"),
      exitCode: stated(record.exit_code, "—"),
      responseStatus: stated(record.response_status),
      completionClaimed: stated(record.completion_claimed),
      verificationStatus: stated(record.verification_status),
      elapsedMs: stated(record.elapsed_ms, "—"),
      createdAt: stated(record.created_at),
      terminalAt: stated(record.terminal_at, "—"),
    });
  }
  return rows;
}

/** Occurrences whose ledger fact is a queue / miss / coalesce decision. */
export function ledgerDecisionRows(view: RoutineRunsView): RoutineOccurrenceRow[] {
  return view.occurrences.filter(
    (row) =>
      row.disposition === "queued" ||
      row.disposition === "missed" ||
      row.disposition === "coalesced",
  );
}
