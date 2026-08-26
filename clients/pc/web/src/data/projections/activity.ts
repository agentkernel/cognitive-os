/*
 * Activity (evidence stream) view models — docs/design/19, wave 8 in 39.
 *
 * Activity is time-ordered investigation, not Home's attention queue. Every
 * row is one of seven labeled kinds mapped onto a real source. There is no
 * unified authority feed (BD-5): this module composes provider-plane audit,
 * budget alerts, this session's mutation receipts, and this session's
 * observed task evidence/effects. Memory, skill, tool-lifecycle and backup
 * mutations are not events here.
 *
 * No fetching happens here. Views drive fetchProjection/appProjections.
 */

import { readDomainState, type StateReading } from "../../state/stateMap";
import {
  evidenceDisposition,
  isConsequentialChange,
  shortTaskRef,
  type ObservedTask,
  type SessionMutationReceipt,
  type TaskEvidenceView,
  type TaskEffectSummary,
} from "./home";
import type { AuditEventView, ProviderAlertView } from "./providers";
import type { Projection } from "../store";

export const ACTIVITY_KINDS = [
  "event",
  "change",
  "effect",
  "error",
  "intervention",
  "verification",
  "acceptance",
] as const;

export type ActivityKind = (typeof ACTIVITY_KINDS)[number];

export const ACTIVITY_KIND_LABEL: Record<ActivityKind, string> = {
  event: "Event",
  change: "Change",
  effect: "Effect",
  error: "Error",
  intervention: "Intervention",
  verification: "Verification",
  acceptance: "Acceptance",
};

export const ACTIVITY_OBJECT_TYPES = ["task", "provider", "alert", "session"] as const;
export type ActivityObjectType = (typeof ACTIVITY_OBJECT_TYPES)[number];

export const ACTIVITY_ROW_CAP = 50;
export const ACTIVITY_TASK_PROBE_LIMIT = 6;

export const ACTIVITY_COVERAGE =
  "Coverage: provider-plane audit + this session's observed task events. Not a complete authority event log (BD-5). Memory, skill, tool-lifecycle and backup mutations are not emitted as events.";

export const ACTIVITY_SINCE_NOTE =
  "Rows without a timestamp (provider-plane audit, effects) stay visible — they cannot be filtered by time.";

/** Share Home's projection keys so an acknowledge on either surface updates both. */
export const ACTIVITY_ALERTS_KEY = "home:alerts";
export const ACTIVITY_AUDIT_KEY = "home:audit";
export const STRIP_ALERTS_KEY = "strip:alerts";

export const activityEffectsKey = (taskRef: string) => `home:effects:${taskRef}`;
export const activityEvidenceKey = (taskRef: string) => `home:evidence:${taskRef}`;

/** Same stages Home escalates; here they render as Error rather than Effect. */
const ERROR_EFFECT_STAGES = new Set([
  "OUTCOME_UNKNOWN",
  "VERIFY_FAILED",
  "DENIED",
  "ABORTED",
  "QUARANTINED",
]);

export type ActivityKindFilter = "all" | ActivityKind;
export type ActivityObjectFilter = "all" | ActivityObjectType;
export type ActivitySinceFilter = "all" | "hour" | "day";

export interface ActivityRow {
  id: string;
  kind: ActivityKind;
  objectType: ActivityObjectType;
  objectLabel: string;
  objectRef?: string;
  fact: string;
  source: string;
  atMs?: number;
  ageUnknownReason?: string;
  href?: string;
  alertId?: string;
  reading: StateReading;
}

export interface ActivityComposeInput {
  alerts: ProviderAlertView[];
  auditEvents: AuditEventView[];
  receipts: SessionMutationReceipt[];
  observed: ObservedTask[];
  effects: TaskEffectSummary[];
  evidence: { taskRef: string; view: TaskEvidenceView }[];
}

function taskHref(taskRef: string, section: "run" | "evidence"): string {
  return `/work/${encodeURIComponent(taskRef)}?section=${section}`;
}

function providerHref(detail?: string): string {
  if (detail && !detail.includes(" ") && !detail.includes("://")) {
    return `/providers/${encodeURIComponent(detail)}`;
  }
  return "/providers";
}

export function activityObjectHref(row: ActivityRow): string | undefined {
  if (row.href) {
    return row.href;
  }
  if (row.objectType === "task" && row.objectRef) {
    return taskHref(
      row.objectRef,
      row.kind === "verification" || row.kind === "acceptance" ? "evidence" : "run",
    );
  }
  if (row.objectType === "provider") {
    return providerHref(row.objectRef);
  }
  return undefined;
}

function alertRows(alerts: ProviderAlertView[]): ActivityRow[] {
  return alerts
    .filter((alert) => !alert.acknowledged)
    .map((alert) => ({
      id: `alert:${alert.id}`,
      kind: "error" as const,
      objectType: "alert" as const,
      objectLabel: alert.id,
      objectRef: alert.budgetId,
      fact: `Budget ${alert.budgetId ?? "unknown"} raised ${alert.threshold} — advisory, it never blocks execution.`,
      source: "GET /management/alerts",
      atMs: alert.issuedAtMs,
      alertId: alert.id,
      reading: {
        category: alert.threshold === "exceeded_100" ? "blocked" : "attention",
        label: alert.threshold,
        unmapped: false,
      },
    }));
}

function auditRows(events: AuditEventView[]): ActivityRow[] {
  return events.map((event) => {
    const failed = event.outcome !== "ok";
    const change = isConsequentialChange(event.action);
    const kind: ActivityKind = failed ? "error" : change ? "change" : "event";
    return {
      id: `audit:${event.id}`,
      kind,
      objectType: "provider" as const,
      objectLabel: event.detail ?? event.id,
      objectRef: event.detail,
      fact: `Provider-plane audit recorded ${event.action} (outcome ${event.outcome})${
        event.detail ? ` on ${event.detail}` : ""
      }.`,
      source: "GET /management/audit",
      ageUnknownReason: "provider audit rows carry no timestamp",
      href: providerHref(event.detail),
      reading: {
        category: failed ? "blocked" : change ? "attention" : "ready",
        label: event.action,
        unmapped: false,
      },
    };
  });
}

function receiptRows(receipts: SessionMutationReceipt[]): ActivityRow[] {
  return receipts.map((receipt) => {
    const ack = receipt.action === "alert.acknowledge";
    const change = isConsequentialChange(receipt.action);
    const kind: ActivityKind = ack || !change ? "intervention" : "change";
    const objectType: ActivityObjectType = ack
      ? "alert"
      : receipt.objectRef?.startsWith("task://")
        ? "task"
        : "session";
    return {
      id: `receipt:${receipt.id}`,
      kind,
      objectType,
      objectLabel: receipt.objectRef ?? receipt.action,
      objectRef: receipt.objectRef,
      fact: `This session performed ${receipt.action}${
        receipt.detail ? ` — ${receipt.detail}` : ""
      }.`,
      source: "this session's mutation receipts",
      atMs: receipt.atMs,
      href: objectType === "task" && receipt.objectRef ? taskHref(receipt.objectRef, "run") : "/providers",
      reading: {
        category: "attention",
        label: receipt.action,
        unmapped: false,
      },
    };
  });
}

function observedRows(observed: ObservedTask[]): ActivityRow[] {
  return observed.map((task) => {
    const admitted = task.origin.includes("admit");
    return {
      id: `observed:${task.taskRef}`,
      kind: (admitted ? "intervention" : "event") as ActivityKind,
      objectType: "task" as const,
      objectLabel: shortTaskRef(task.taskRef),
      objectRef: task.taskRef,
      fact: admitted
        ? `Admitted this session${task.objective ? ` — ${task.objective}` : ""}. Admission is not execution.`
        : `Observed this session at ${task.origin}.`,
      source: "session-observed (this browser session only)",
      atMs: task.observedAtMs,
      href: taskHref(task.taskRef, "run"),
      reading: {
        category: admitted ? "waiting" : "ready",
        label: admitted ? "admitted" : task.origin,
        unmapped: false,
      },
    };
  });
}

function effectRows(summaries: TaskEffectSummary[]): ActivityRow[] {
  return summaries.flatMap((summary) =>
    summary.effects.map((effect) => {
      const failed =
        ERROR_EFFECT_STAGES.has(effect.stage) ||
        effect.reconcileClass === "must_reconcile" ||
        effect.reconcileClass === "pending_reconciliation";
      const reading = readDomainState("effect", effect.stage);
      return {
        id: `effect:${summary.taskRef}:${effect.effectRef}`,
        kind: (failed ? "error" : "effect") as ActivityKind,
        objectType: "task" as const,
        objectLabel: shortTaskRef(summary.taskRef),
        objectRef: summary.taskRef,
        fact: `Effect ${effect.effectRef} is ${effect.stage} with reconcile class ${effect.reconcileClass}.`,
        source: "GET /task/effects",
        ageUnknownReason: "task effects carry no timestamp",
        href: taskHref(summary.taskRef, "run"),
        reading,
      };
    }),
  );
}

function evidenceRows(entries: { taskRef: string; view: TaskEvidenceView }[]): ActivityRow[] {
  return entries.flatMap((entry) => {
    const rows: ActivityRow[] = [];
    const completedMs = entry.view.completedAt ? Date.parse(entry.view.completedAt) : Number.NaN;
    const atMs = Number.isFinite(completedMs) ? completedMs : undefined;
    if (entry.view.hasVerification) {
      const disposition = evidenceDisposition(entry.view);
      const failed = entry.view.verificationStatus === "failed" && entry.view.verificationCurrent !== false;
      rows.push({
        id: `verification:${entry.taskRef}`,
        kind: failed ? "error" : "verification",
        objectType: "task",
        objectLabel: shortTaskRef(entry.taskRef),
        objectRef: entry.taskRef,
        fact: `${disposition.detail}${
          entry.view.reportRef ? ` Report ${entry.view.reportRef}.` : ""
        }`,
        source: "GET /task/evidence",
        atMs,
        ageUnknownReason: atMs == null ? "verification completion time unknown" : undefined,
        href: taskHref(entry.taskRef, "evidence"),
        reading: disposition.reading,
      });
    }
    if (entry.view.acceptancePresent && entry.view.verificationCurrent !== false) {
      rows.push({
        id: `acceptance:${entry.taskRef}`,
        kind: "acceptance",
        objectType: "task",
        objectLabel: shortTaskRef(entry.taskRef),
        objectRef: entry.taskRef,
        fact: entry.view.acceptanceCurrent === false
          ? "A terminal acceptance record is present but is not current for the fencing epoch."
          : "Terminal acceptance is recorded.",
        source: "GET /task/evidence",
        atMs,
        ageUnknownReason:
          atMs == null ? "acceptance records carry no independent timestamp" : undefined,
        href: taskHref(entry.taskRef, "evidence"),
        reading: {
          category: entry.view.acceptanceCurrent === false ? "unknown" : "completed",
          label: "accepted",
          unmapped: false,
        },
      });
    }
    return rows;
  });
}

/**
 * Newest known time first. Rows the daemon did not timestamp sort after
 * timed rows, by id — their order is not a clock.
 */
export function sortActivity(rows: ActivityRow[]): ActivityRow[] {
  return [...rows].sort((a, b) => {
    const aTime = a.atMs;
    const bTime = b.atMs;
    const aKnown = aTime != null && Number.isFinite(aTime);
    const bKnown = bTime != null && Number.isFinite(bTime);
    if (aKnown && bKnown && aTime !== bTime) {
      return (bTime as number) - (aTime as number);
    }
    if (aKnown !== bKnown) {
      return aKnown ? -1 : 1;
    }
    return a.id.localeCompare(b.id);
  });
}

export function composeActivity(input: ActivityComposeInput): ActivityRow[] {
  return sortActivity([
    ...alertRows(input.alerts),
    ...auditRows(input.auditEvents),
    ...receiptRows(input.receipts),
    ...observedRows(input.observed),
    ...effectRows(input.effects),
    ...evidenceRows(input.evidence),
  ]);
}

export function filterActivityRows(
  rows: ActivityRow[],
  filters: {
    kind: ActivityKindFilter;
    objectType: ActivityObjectFilter;
    since: ActivitySinceFilter;
    nowMs: number;
  },
): ActivityRow[] {
  const sinceMs =
    filters.since === "hour"
      ? filters.nowMs - 60 * 60 * 1000
      : filters.since === "day"
        ? filters.nowMs - 24 * 60 * 60 * 1000
        : undefined;
  return rows.filter((row) => {
    if (filters.kind !== "all" && row.kind !== filters.kind) {
      return false;
    }
    if (filters.objectType !== "all" && row.objectType !== filters.objectType) {
      return false;
    }
    if (sinceMs != null && row.atMs != null && row.atMs < sinceMs) {
      return false;
    }
    return true;
  });
}

export function boundActivityRows(
  rows: ActivityRow[],
  cap: number = ACTIVITY_ROW_CAP,
): { shown: ActivityRow[]; total: number; truncated: boolean } {
  return {
    shown: rows.slice(0, cap),
    total: rows.length,
    truncated: rows.length > cap,
  };
}

export function probeObservedRefs(observed: ObservedTask[]): {
  refs: string[];
  truncated: boolean;
  total: number;
} {
  const unique = [...new Map(observed.map((task) => [task.taskRef, task])).values()].sort(
    (a, b) => b.observedAtMs - a.observedAtMs,
  );
  return {
    refs: unique.slice(0, ACTIVITY_TASK_PROBE_LIMIT).map((task) => task.taskRef),
    truncated: unique.length > ACTIVITY_TASK_PROBE_LIMIT,
    total: unique.length,
  };
}

export function namedSourceFailure(projection: Projection<unknown>, label: string): string | undefined {
  if (
    projection.status === "loading" ||
    projection.status === "stale" ||
    projection.status === "ready" ||
    projection.status === "empty"
  ) {
    return undefined;
  }
  const code = projection.error?.code ?? projection.status;
  return `${label} unavailable — ${code}`;
}
