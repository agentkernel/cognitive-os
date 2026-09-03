/**
 * Personal-private Today run overview projection (P13-T05/D02 Dual Track).
 * Source is GET /management/project/v1/today.overview?period=today|week|month.
 * One row per live Project (state · today's completed Attempts · current
 * stage · duration) plus created / live / blocked counts. Counts are daemon
 * integers; anything missing reads `unknown`, never 0. No KPI wall, no
 * success rate, no completion claim: `attempts_done` is the daemon-observed
 * `done` terminal count with `verification_status = not-run`.
 */

import { asList, asRecord } from "../projections";

export const TODAY_OVERVIEW_PATH = "/management/project/v1/today.overview";
export const TODAY_OVERVIEW_KEY = "opc:today-overview";

export const TODAY_PERIODS = ["today", "week", "month"] as const;
export type TodayPeriod = (typeof TODAY_PERIODS)[number];

export function isTodayPeriod(value: string): value is TodayPeriod {
  return (TODAY_PERIODS as readonly string[]).includes(value);
}

export function todayOverviewPath(period: TodayPeriod): string {
  return `${TODAY_OVERVIEW_PATH}?period=${encodeURIComponent(period)}`;
}

export interface TodayOverviewRow {
  projectId: string;
  state: string;
  status: string;
  armedRoutines: string;
  pausedRoutines: string;
  runningOccurrenceId: string;
  queuedCount: string;
  missedCount: string;
  attemptsTotal: string;
  attemptsDone: string;
  attemptsFailed: string;
  attemptsUnknown: string;
  durationMs: string;
  currentStageId: string;
  currentStageTitle: string;
  lastTerminalAt: string;
  cost: string;
}

export interface TodayOverviewView {
  period: string;
  periodBasis: string;
  created: string;
  live: string;
  blocked: string;
  rows: TodayOverviewRow[];
  kpiWall: string;
  verificationStatus: string;
  cost: string;
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

function statedCount(value: unknown): string {
  return typeof value === "number" && Number.isInteger(value) && value >= 0
    ? String(value)
    : "unknown";
}

/** Duration is shown as stated milliseconds; `unknown` stays `unknown`. */
export function formatDuration(durationMs: string): string {
  if (!/^\d+$/.test(durationMs)) {
    return durationMs;
  }
  const total = Number(durationMs);
  if (total < 1000) {
    return `${total} ms`;
  }
  const seconds = Math.floor(total / 1000);
  if (seconds < 60) {
    return `${seconds} s`;
  }
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return `${minutes} min ${seconds % 60} s`;
  }
  const hours = Math.floor(minutes / 60);
  return `${hours} h ${minutes % 60} min`;
}

function projectRow(item: unknown): TodayOverviewRow | undefined {
  const record = asRecord(item);
  if (typeof record.project_id !== "string" || record.project_id.length === 0) {
    return undefined;
  }
  return {
    projectId: record.project_id,
    state: stated(record.state),
    status: stated(record.status),
    armedRoutines: statedCount(record.armed_routines),
    pausedRoutines: statedCount(record.paused_routines),
    runningOccurrenceId: stated(record.running_occurrence_id, "—"),
    queuedCount: statedCount(record.queued_count),
    missedCount: statedCount(record.missed_count),
    attemptsTotal: statedCount(record.attempts_total),
    attemptsDone: statedCount(record.attempts_done),
    attemptsFailed: statedCount(record.attempts_failed),
    attemptsUnknown: statedCount(record.attempts_unknown),
    durationMs: statedCount(record.duration_ms),
    currentStageId: stated(record.current_stage_id, "—"),
    currentStageTitle: stated(record.current_stage_title, "—"),
    lastTerminalAt: stated(record.last_terminal_at, "—"),
    cost: stated(record.cost),
  };
}

/**
 * One view row (not a list) so zero live Projects still reads `ready` with
 * the daemon's counts: the overview is never confused with a missing answer.
 */
export function projectTodayOverview(body: unknown): TodayOverviewView[] {
  const record = asRecord(body);
  const period = stated(record.period, "");
  if (!isTodayPeriod(period)) {
    return [];
  }
  const counts = asRecord(record.counts);
  return [
    {
      period,
      periodBasis: stated(record.period_basis),
      created: statedCount(counts.created),
      live: statedCount(counts.live),
      blocked: statedCount(counts.blocked),
      rows: asList(body, ["rows"])
        .map(projectRow)
        .filter((row): row is TodayOverviewRow => row !== undefined),
      kpiWall: stated(record.kpi_wall),
      verificationStatus: stated(record.verification_status),
      cost: stated(record.cost),
    },
  ];
}
