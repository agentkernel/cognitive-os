/*
 * Home (attention surface) view models — docs/design/13, state grammar 22.
 *
 * Home composes five verified sources into three questions: is the system
 * ready, what needs me, what is in flight. This module owns every pure
 * transform so the views stay declarative and the honesty rules are testable:
 *
 * - unknown / unavailable is never coerced to 0, to "ready", or to absence;
 * - a daemon word is carried verbatim and categorised through stateMap;
 * - facts the daemon does not expose (task objective, audit timestamps,
 *   a unified system audit, a task inventory) are named as missing, not
 *   guessed.
 *
 * No fetching happens here. Views drive fetchProjection/appProjections.
 */

import { readDomainState, type StateCategory, type StateReading } from "../../state/stateMap";
import type { WatchState } from "../../watch";
import { asList, asRecord, projectReadiness, type ReadinessComponentView } from "../projections";
import type {
  AuditEventView,
  BindingView,
  ProviderAccount,
  ProviderAlertView,
} from "./providers";
import type { Projection, ProjectionStore } from "../store";

/* ---------- shared helpers ---------- */

/** Severity for triage: lower sorts first. Unknown is never as good as ready. */
const CATEGORY_SEVERITY: Record<StateCategory, number> = {
  blocked: 0,
  attention: 1,
  unknown: 2,
  waiting: 3,
  active: 4,
  completed: 5,
  ready: 6,
};

export function categorySeverity(category: StateCategory): number {
  return CATEGORY_SEVERITY[category];
}

/**
 * Coarse age, cause-first surfaces read it next to the fact. Returns
 * undefined when the timestamp is unknown — callers must render the unknown,
 * never a zero.
 */
export function formatAge(fromMs: number | undefined, nowMs: number): string | undefined {
  if (fromMs == null || !Number.isFinite(fromMs)) {
    return undefined;
  }
  const seconds = Math.max(0, Math.round((nowMs - fromMs) / 1000));
  if (seconds < 60) {
    return `${seconds}s ago`;
  }
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) {
    return `${minutes}m ago`;
  }
  const hours = Math.round(minutes / 60);
  if (hours < 48) {
    return `${hours}h ago`;
  }
  return `${Math.round(hours / 24)}d ago`;
}

/**
 * Short display form of a `task://…` ref: the last path segment, medially
 * truncated. The full ref stays available (title/copy) — never replaced.
 */
export function shortTaskRef(taskRef: string): string {
  const segments = taskRef.split("/").filter((part) => part !== "");
  const last = segments[segments.length - 1] ?? taskRef;
  return last.length <= 13 ? last : `${last.slice(0, 6)}…${last.slice(-4)}`;
}

/* ---------- R1 readiness (/personal/status ≡ /personal/readiness) ---------- */

/** readiness.rs evaluates exactly these six, in this order. */
export const READINESS_COMPONENTS = [
  "system",
  "database",
  "secret",
  "provider",
  "daemon",
  "pi",
] as const;

export interface HomeReadinessComponent extends ReadinessComponentView {
  required?: boolean;
  errorClass?: string;
  /** False when the daemon did not report this expected component at all. */
  reported: boolean;
}

export interface HomeReadinessView {
  overall: string;
  firstConversationReady?: boolean;
  /** Daemon-side evaluation time; undefined means the daemon did not say. */
  evaluatedAtMs?: number;
  components: HomeReadinessComponent[];
}

export function projectHomeReadiness(body: unknown): HomeReadinessView {
  const record = asRecord(body);
  const base = projectReadiness(body);
  const raw = Array.isArray(record.components) ? record.components.map(asRecord) : [];
  const evaluated = Number(record.evaluated_at_unix_ms);
  return {
    overall: base.overall,
    firstConversationReady: base.firstConversationReady,
    evaluatedAtMs:
      record.evaluated_at_unix_ms != null && Number.isFinite(evaluated) ? evaluated : undefined,
    components: base.components.map((component, index) => {
      const source = raw[index] ?? {};
      return {
        ...component,
        reported: true,
        required: typeof source.required === "boolean" ? source.required : undefined,
        errorClass: source.error_class == null ? undefined : String(source.error_class),
      };
    }),
  };
}

function componentOrder(name: string): number {
  const index = (READINESS_COMPONENTS as readonly string[]).indexOf(name);
  return index === -1 ? READINESS_COMPONENTS.length : index;
}

/**
 * The six expected components in canonical order. A component the daemon did
 * not report renders as an unreported (S7) row — never dropped, never
 * inferred ready. Extra components the daemon adds are appended verbatim.
 */
export function expandedReadinessComponents(
  view: HomeReadinessView,
): HomeReadinessComponent[] {
  const byName = new Map(view.components.map((component) => [component.name, component]));
  const canonical = READINESS_COMPONENTS.map(
    (name) =>
      byName.get(name) ?? {
        name,
        state: "not reported",
        reported: false,
      },
  );
  const extra = view.components.filter(
    (component) => !(READINESS_COMPONENTS as readonly string[]).includes(component.name),
  );
  return [...canonical, ...extra];
}

/**
 * The component that most needs the owner. Ready components never win; an
 * unmapped or unreported word ranks worse than ready, never better.
 * Undefined only when every reported component is ready.
 */
export function worstReadinessComponent(
  view: HomeReadinessView,
): HomeReadinessComponent | undefined {
  const ranked = expandedReadinessComponents(view)
    .map((component) => ({
      component,
      severity: component.reported
        ? categorySeverity(readDomainState("readiness", component.state).category)
        : CATEGORY_SEVERITY.unknown,
    }))
    .filter((entry) => entry.severity < CATEGORY_SEVERITY.ready)
    .sort(
      (a, b) =>
        a.severity - b.severity ||
        componentOrder(a.component.name) - componentOrder(b.component.name) ||
        a.component.name.localeCompare(b.component.name),
    );
  return ranked[0]?.component;
}

/** The chip reading for one component; unreported is client-honest S7. */
export function readinessComponentReading(component: HomeReadinessComponent): StateReading {
  if (!component.reported) {
    return { category: "unknown", label: "not reported", unmapped: false };
  }
  return readDomainState("readiness", component.state);
}

/* ---------- session-observed task refs (BD-3 honesty, session-local) ---------- */

export const OBSERVED_TASKS_KEY = "home:observed-tasks";
export const OBSERVED_TASKS_SOURCE = "session-observed (this browser session only)";

export interface ObservedTask {
  taskRef: string;
  /** The objective this session used, when the session itself minted it. */
  objective?: string;
  observedAtMs: number;
  /** Where this session saw the ref (e.g. "task/admit"). */
  origin: string;
}

export function readObservedTasks(store: ProjectionStore): ObservedTask[] {
  return store.get<ObservedTask[]>(OBSERVED_TASKS_KEY)?.data ?? [];
}

/**
 * Remember a task ref this session actually observed. This is explicitly not
 * an inventory: it is the client's own memory of refs it saw, and it dies
 * with the tab. Deduplicated by ref, newest observation wins.
 */
export function noteObservedTask(store: ProjectionStore, task: ObservedTask): void {
  const existing = readObservedTasks(store).filter((row) => row.taskRef !== task.taskRef);
  const rows = [...existing, task].sort((a, b) => b.observedAtMs - a.observedAtMs);
  store.set<ObservedTask[]>(OBSERVED_TASKS_KEY, {
    status: rows.length === 0 ? "empty" : "ready",
    data: rows,
    source: OBSERVED_TASKS_SOURCE,
    updatedAt: Date.now(),
  });
}

/* ---------- session mutation receipts (critical changes, session-local) ---------- */

export const SESSION_RECEIPTS_KEY = "home:session-receipts";
export const SESSION_RECEIPTS_SOURCE = "this session's mutation receipts";

export interface SessionMutationReceipt {
  id: string;
  /** Provider-plane audit vocabulary, verbatim (e.g. "key.rotate"). */
  action: string;
  objectRef?: string;
  atMs: number;
  detail?: string;
}

export function readSessionMutations(store: ProjectionStore): SessionMutationReceipt[] {
  return store.get<SessionMutationReceipt[]>(SESSION_RECEIPTS_KEY)?.data ?? [];
}

export function recordSessionMutation(
  store: ProjectionStore,
  receipt: SessionMutationReceipt,
): void {
  const rows = [...readSessionMutations(store), receipt].sort((a, b) => b.atMs - a.atMs);
  store.set<SessionMutationReceipt[]>(SESSION_RECEIPTS_KEY, {
    status: "ready",
    data: rows,
    source: SESSION_RECEIPTS_SOURCE,
    updatedAt: Date.now(),
  });
}

/* ---------- R2 needs attention ---------- */

export type AttentionPriority = "change" | "blocked" | "attention" | "waiting" | "stale";

export const ATTENTION_PRIORITY_ORDER: readonly AttentionPriority[] = [
  "change",
  "blocked",
  "attention",
  "waiting",
  "stale",
] as const;

export const ATTENTION_ROW_CAP = 5;

export interface AttentionAction {
  kind: "link" | "acknowledge";
  label: string;
  /** Router path for link actions. */
  to?: string;
  /** Alert id for the class-B acknowledge action. */
  alertId?: string;
}

export interface AttentionItem {
  /** Stable across refreshes: rows never reshuffle under the cursor. */
  id: string;
  priority: AttentionPriority;
  reading: StateReading;
  objectType: string;
  /** Short, mono-rendered object label. */
  objectLabel: string;
  /** Full object ref when one exists (title/copy only). */
  objectRef?: string;
  /** One cause-first sentence. */
  reason: string;
  atMs?: number;
  /** Why the age is unknown, when it is. */
  ageUnknownReason?: string;
  action?: AttentionAction;
}

/**
 * Priority sort: changes, then blocked/failed, attention, waiting-on-owner,
 * stale. Array.prototype.sort is stable, and every builder emits a
 * deterministic order, so a new arrival inserts into its rank without
 * reordering the rows already on screen.
 */
export function sortAttention(items: AttentionItem[]): AttentionItem[] {
  const rank = (priority: AttentionPriority) => ATTENTION_PRIORITY_ORDER.indexOf(priority);
  return [...items].sort((a, b) => rank(a.priority) - rank(b.priority));
}

/**
 * Consequential provider-plane mutations (S4/S5-relevant). Account creation,
 * updates and catalog probes are ordinary traffic and stay out of the queue.
 */
export const CONSEQUENTIAL_CHANGE_ACTIONS: readonly string[] = [
  "account.delete",
  "key.set",
  "key.rotate",
  "key.remove",
  "binding.set",
  "binding.remove",
  "tool.quarantine",
  "tool.revoke",
  "restore.apply",
] as const;

export function isConsequentialChange(action: string): boolean {
  return CONSEQUENTIAL_CHANGE_ACTIONS.includes(action);
}

/**
 * R6 folded into R2: consequential governance mutations render as the top
 * group of the queue. Provider-plane audit rows carry no timestamp, so their
 * age is rendered unknown; this session's own receipts do carry one.
 */
export function changeAttention(
  auditEvents: AuditEventView[],
  receipts: SessionMutationReceipt[],
): AttentionItem[] {
  const fromReceipts = receipts
    .filter((receipt) => isConsequentialChange(receipt.action))
    .map<AttentionItem>((receipt) => ({
      id: `change:session:${receipt.id}`,
      priority: "change",
      reading: { category: "attention", label: receipt.action, unmapped: false },
      objectType: "change",
      objectLabel: receipt.objectRef ?? receipt.action,
      objectRef: receipt.objectRef,
      reason: `This session performed ${receipt.action}${
        receipt.detail ? ` — ${receipt.detail}` : ""
      }. Confirm it was intended.`,
      atMs: receipt.atMs,
      action: { kind: "link", label: "Review in Providers", to: "/providers" },
    }));

  const fromAudit = auditEvents
    .filter((event) => isConsequentialChange(event.action))
    .sort((a, b) => b.id.localeCompare(a.id))
    .map<AttentionItem>((event) => ({
      id: `change:audit:${event.id}`,
      priority: "change",
      reading: {
        category: event.outcome === "ok" ? "attention" : "blocked",
        label: event.action,
        unmapped: false,
      },
      objectType: "change",
      objectLabel: event.detail ?? event.id,
      objectRef: event.detail,
      reason: `Provider-plane audit recorded ${event.action} (outcome ${event.outcome})${
        event.detail ? ` on ${event.detail}` : ""
      }.`,
      ageUnknownReason: "provider audit rows carry no timestamp",
      action: { kind: "link", label: "Review in Providers", to: "/providers" },
    }));

  return [...fromReceipts, ...fromAudit];
}

export function providerAttention(accounts: ProviderAccount[]): AttentionItem[] {
  return [...accounts]
    .sort((a, b) => a.id.localeCompare(b.id))
    .flatMap<AttentionItem>((account) => {
      const reading = readDomainState("provider", account.status);
      if (reading.category === "ready") {
        // A ready account with an unresolvable key is still an owner problem.
        if (account.secret !== "absent") {
          return [];
        }
        return [
          {
            id: `provider:secret:${account.id}`,
            priority: "blocked",
            reading: { category: "blocked", label: "secret absent", unmapped: false },
            objectType: "provider",
            objectLabel: account.name,
            objectRef: account.id,
            reason:
              "No key is present for this account — the daemon cannot resolve a SecretRef for it.",
            action: {
              kind: "link",
              label: "Hand over a key",
              to: `/providers/${encodeURIComponent(account.id)}`,
            },
          },
        ];
      }
      const priority: AttentionPriority =
        reading.category === "blocked" ? "blocked" : reading.category === "attention" ? "attention" : "waiting";
      return [
        {
          id: `provider:${account.id}`,
          priority,
          reading,
          objectType: "provider",
          objectLabel: account.name,
          objectRef: account.id,
          reason: account.lastDiscoveryError
            ? `Discovery failed — ${account.lastDiscoveryError}.`
            : `The daemon reports this account ${account.status}${
                reading.unmapped ? " (state word not in the client's table)" : ""
              }.`,
          action: {
            kind: "link",
            label: "Repair provider",
            to: `/providers/${encodeURIComponent(account.id)}`,
          },
        },
      ];
    });
}

export function bindingAttention(bindings: BindingView[]): AttentionItem[] {
  return [...bindings]
    .sort((a, b) => a.agent.localeCompare(b.agent) || a.accountId.localeCompare(b.accountId))
    .filter((binding) => binding.status !== "active")
    .map<AttentionItem>((binding) => {
      const reading = readDomainState("provider", binding.status);
      return {
        id: `binding:${binding.agent}:${binding.accountId}`,
        priority: reading.category === "blocked" ? "blocked" : "attention",
        reading,
        objectType: "binding",
        objectLabel: `${binding.agent} → ${binding.modelId}`,
        objectRef: binding.accountId,
        reason: `Agent ${binding.agent} has a ${binding.status} provider binding — it cannot dispatch until the binding is set again.`,
        action: {
          kind: "link",
          label: "Open binding",
          to: `/providers/${encodeURIComponent(binding.accountId)}`,
        },
      };
    });
}

export function alertAttention(alerts: ProviderAlertView[]): AttentionItem[] {
  return [...alerts]
    .filter((alert) => !alert.acknowledged)
    .sort((a, b) => a.id.localeCompare(b.id))
    .map<AttentionItem>((alert) => ({
      id: `alert:${alert.id}`,
      priority: alert.threshold === "exceeded_100" ? "blocked" : "attention",
      reading: {
        category: alert.threshold === "exceeded_100" ? "blocked" : "attention",
        label: alert.threshold,
        unmapped: false,
      },
      objectType: "alert",
      objectLabel: alert.id,
      objectRef: alert.budgetId,
      reason: `Budget ${alert.budgetId ?? "unknown"} raised ${alert.threshold} — advisory, it never blocks execution.`,
      atMs: alert.issuedAtMs,
      action: { kind: "acknowledge", label: "Acknowledge", alertId: alert.id },
    }));
}

export function readinessAttention(view: HomeReadinessView | undefined): AttentionItem[] {
  if (!view) {
    return [];
  }
  return expandedReadinessComponents(view)
    .flatMap<AttentionItem>((component) => {
      const reading = readinessComponentReading(component);
      if (reading.category === "ready") {
        return [];
      }
      const priority: AttentionPriority =
        reading.category === "blocked"
          ? "blocked"
          : reading.category === "attention"
            ? "attention"
            : "waiting";
      return [
        {
          id: `readiness:${component.name}`,
          priority,
          reading,
          objectType: "component",
          objectLabel: component.name,
          reason: component.reported
            ? `Readiness reports ${component.name} ${component.state}${
                component.errorClass ? ` (${component.errorClass})` : ""
              }${component.detail ? ` — ${component.detail}` : ""}.`
            : `The daemon's readiness projection did not report ${component.name}; its state is unknown, not ready.`,
          action: { kind: "link", label: "Open System", to: "/system" },
        },
      ];
    })
    .sort(
      (a, b) => componentOrder(a.objectLabel) - componentOrder(b.objectLabel),
    );
}

export interface TaskEffectSummary {
  taskRef: string;
  effects: EffectEntryView[];
}

/** Effect stages that need the owner now, verbatim from effect.transitions. */
const EFFECT_ESCALATION_STAGES = new Set([
  "OUTCOME_UNKNOWN",
  "VERIFY_FAILED",
  "DENIED",
  "ABORTED",
  "QUARANTINED",
]);

export function effectAttention(summaries: TaskEffectSummary[]): AttentionItem[] {
  return [...summaries]
    .sort((a, b) => a.taskRef.localeCompare(b.taskRef))
    .flatMap<AttentionItem>((summary) =>
      summary.effects
        .filter(
          (effect) =>
            EFFECT_ESCALATION_STAGES.has(effect.stage) ||
            effect.reconcileClass === "must_reconcile" ||
            effect.reconcileClass === "pending_reconciliation",
        )
        .map<AttentionItem>((effect) => {
          const reading = readDomainState("effect", effect.stage);
          const priority: AttentionPriority =
            reading.category === "blocked"
              ? "blocked"
              : effect.reconcileClass === "must_reconcile"
                ? "attention"
                : "waiting";
          return {
            id: `effect:${summary.taskRef}:${effect.effectRef}`,
            priority,
            reading,
            objectType: "task",
            objectLabel: shortTaskRef(summary.taskRef),
            objectRef: summary.taskRef,
            reason: `Effect ${effect.effectRef} is ${effect.stage} with reconcile class ${effect.reconcileClass} — the outcome is not settled.`,
            action: { kind: "link", label: "Open Work", to: "/work" },
          };
        }),
    );
}

/**
 * Watch is process-local and Home attaches no stream in this wave, so this
 * builder is fed an explicit state. `unknown` produces no row: the absence
 * is stated as copy in Current work instead of faked as an alert.
 */
export function watchAttention(state: WatchState | undefined): AttentionItem[] {
  if (state == null || state === "live" || state === "unknown") {
    return [];
  }
  const reading = readDomainState("watch", state);
  return [
    {
      id: `watch:${state}`,
      priority: state === "disconnected" ? "blocked" : "stale",
      reading,
      objectType: "watch",
      objectLabel: "task watch",
      reason:
        state === "disconnected"
          ? "The task watch stream detached — detaching never cancelled a task, and completion stays unknown."
          : "The task watch cursor has a gap — events between cursors were not observed; a snapshot reload is required.",
      action: { kind: "link", label: "Open Work", to: "/work" },
    },
  ];
}

/**
 * First run: the queue is the guide (docs/design/13 §5). Only an
 * authoritative empty triggers these — a denied or unavailable read never
 * pretends the operator has nothing configured.
 */
export function setupAttention(input: {
  providersAuthoritativelyEmpty: boolean;
  workAuthoritativelyEmpty: boolean;
}): AttentionItem[] {
  const rows: AttentionItem[] = [];
  if (input.providersAuthoritativelyEmpty) {
    rows.push({
      id: "setup:provider",
      priority: "waiting",
      reading: { category: "waiting", label: "no provider account", unmapped: false },
      objectType: "setup",
      objectLabel: "provider account",
      reason:
        "No provider account exists yet — an agent cannot reach a model until one is created and keyed.",
      action: { kind: "link", label: "Create a provider account", to: "/providers" },
    });
  }
  if (input.workAuthoritativelyEmpty) {
    rows.push({
      id: "setup:work",
      priority: "waiting",
      reading: { category: "waiting", label: "no work observed", unmapped: false },
      objectType: "setup",
      objectLabel: "governed task",
      reason:
        "The daemon lists no task contracts and this session has observed none — start one to see work here.",
      action: { kind: "link", label: "Create a task in Work", to: "/work" },
    });
  }
  return rows;
}

export interface AttentionInput {
  readiness?: HomeReadinessView;
  accounts: ProviderAccount[];
  bindings: BindingView[];
  alerts: ProviderAlertView[];
  auditEvents: AuditEventView[];
  receipts: SessionMutationReceipt[];
  effects: TaskEffectSummary[];
  watchState?: WatchState;
  providersAuthoritativelyEmpty: boolean;
  workAuthoritativelyEmpty: boolean;
}

/** One queue, priority-sorted, changes on top. Views apply the row cap. */
export function composeAttention(input: AttentionInput): AttentionItem[] {
  return sortAttention([
    ...changeAttention(input.auditEvents, input.receipts),
    ...providerAttention(input.accounts),
    ...bindingAttention(input.bindings),
    ...alertAttention(input.alerts),
    ...readinessAttention(input.readiness),
    ...effectAttention(input.effects),
    ...watchAttention(input.watchState),
    ...setupAttention({
      providersAuthoritativelyEmpty: input.providersAuthoritativelyEmpty,
      workAuthoritativelyEmpty: input.workAuthoritativelyEmpty,
    }),
  ]);
}

/* ---------- R3 current work ---------- */

/** resource_manager.rs LIST_LIMIT — the list is bounded, and says so. */
export const TASK_LIST_LIMIT = 64;
export const CURRENT_WORK_ROW_CAP = 4;

export interface TaskEnvelopeView {
  taskRef: string;
  /** contract_epoch; undefined when absent (never defaulted to 0). */
  contractEpoch?: number;
  /** Envelope health word, verbatim ("contracted"). Not a lifecycle state. */
  health?: string;
  revisionDigest?: string;
  blockedReason?: string;
}

export function projectTaskEnvelopes(body: unknown): TaskEnvelopeView[] {
  return asList(body, ["resources", "items"]).map((row) => {
    const record = asRecord(row);
    const epoch = Number(record.object_version);
    return {
      taskRef: String(record.id ?? "unknown"),
      contractEpoch:
        record.object_version != null && Number.isFinite(epoch) ? epoch : undefined,
      health: record.health == null ? undefined : String(record.health),
      revisionDigest:
        record.revision_digest == null ? undefined : String(record.revision_digest),
      blockedReason: record.blocked_reason == null ? undefined : String(record.blocked_reason),
    };
  });
}

/** True when the list is at the daemon's bound and may be hiding rows. */
export function taskListAtBound(rows: TaskEnvelopeView[]): boolean {
  return rows.length >= TASK_LIST_LIMIT;
}

export type WorkOrigin = "envelope" | "session" | "envelope+session";

export interface CurrentWorkRow {
  taskRef: string;
  shortRef: string;
  origin: WorkOrigin;
  contractEpoch?: number;
  health?: string;
  objective?: string;
  observedAtMs?: number;
  revisionDigest?: string;
}

/**
 * Merge the daemon's envelope list with this session's observed refs,
 * deduplicated by task ref. Session-observed rows sort first (newest first),
 * then envelope rows by ref — a total, stable order.
 */
export function mergeCurrentWork(
  envelopes: TaskEnvelopeView[],
  observed: ObservedTask[],
): CurrentWorkRow[] {
  const byRef = new Map<string, CurrentWorkRow>();
  for (const envelope of envelopes) {
    byRef.set(envelope.taskRef, {
      taskRef: envelope.taskRef,
      shortRef: shortTaskRef(envelope.taskRef),
      origin: "envelope",
      contractEpoch: envelope.contractEpoch,
      health: envelope.health,
      revisionDigest: envelope.revisionDigest,
    });
  }
  for (const task of observed) {
    const existing = byRef.get(task.taskRef);
    byRef.set(task.taskRef, {
      taskRef: task.taskRef,
      shortRef: shortTaskRef(task.taskRef),
      origin: existing ? "envelope+session" : "session",
      contractEpoch: existing?.contractEpoch,
      health: existing?.health,
      revisionDigest: existing?.revisionDigest,
      objective: task.objective,
      observedAtMs: task.observedAtMs,
    });
  }
  return [...byRef.values()].sort((a, b) => {
    if (a.observedAtMs != null && b.observedAtMs != null) {
      return b.observedAtMs - a.observedAtMs || a.taskRef.localeCompare(b.taskRef);
    }
    if (a.observedAtMs != null) {
      return -1;
    }
    if (b.observedAtMs != null) {
      return 1;
    }
    return a.taskRef.localeCompare(b.taskRef);
  });
}

/**
 * The envelope has no lifecycle state, so the row's state cell is an explicit
 * S7 with the envelope's own health word — never an invented "running".
 */
export function currentWorkReading(row: CurrentWorkRow): StateReading {
  if (row.health == null) {
    return { category: "unknown", label: "state not exposed", unmapped: false };
  }
  return { category: "unknown", label: row.health, unmapped: false };
}

/* ---------- effects (/task/effects?task_ref=) ---------- */

export interface EffectEntryView {
  effectRef: string;
  stage: string;
  outcomeClass: string;
  reconcileClass: string;
  reportRef?: string;
}

export function projectTaskEffects(body: unknown): EffectEntryView[] {
  return asList(body, ["effects", "items"]).map((row) => {
    const record = asRecord(row);
    return {
      effectRef: String(record.effect_ref ?? "unknown"),
      stage: String(record.stage ?? "unknown"),
      outcomeClass: String(record.outcome_class ?? "unknown"),
      reconcileClass: String(record.reconcile_class ?? "unknown"),
      reportRef: record.report_ref == null ? undefined : String(record.report_ref),
    };
  });
}

/* ---------- R5 recent evidence (/task/evidence?task_ref=) ---------- */

export const RECENT_EVIDENCE_ROW_CAP = 3;

export interface TaskEvidenceView {
  taskRef?: string;
  lifecycleState?: string;
  hasVerification: boolean;
  verificationStatus?: string;
  verificationCurrent?: boolean;
  reportRef?: string;
  reportDigest?: string;
  completedAt?: string;
  artifactsCurrent?: boolean;
  acceptancePresent: boolean;
  acceptanceCurrent?: boolean;
}

export function projectTaskEvidence(body: unknown): TaskEvidenceView {
  const record = asRecord(body);
  const lifecycle = asRecord(record.lifecycle);
  const verification = record.latest_verification == null ? undefined : asRecord(record.latest_verification);
  const acceptance = record.latest_acceptance == null ? undefined : asRecord(record.latest_acceptance);
  return {
    taskRef: record.task_ref == null ? undefined : String(record.task_ref),
    lifecycleState:
      lifecycle.current_state == null ? undefined : String(lifecycle.current_state),
    hasVerification: verification != null,
    verificationStatus:
      verification?.status == null ? undefined : String(verification.status),
    verificationCurrent:
      typeof verification?.current === "boolean" ? verification.current : undefined,
    reportRef: verification?.report_ref == null ? undefined : String(verification.report_ref),
    reportDigest:
      verification?.report_digest == null ? undefined : String(verification.report_digest),
    completedAt:
      verification?.completed_at == null ? undefined : String(verification.completed_at),
    artifactsCurrent:
      typeof verification?.artifacts_current === "boolean"
        ? verification.artifacts_current
        : undefined,
    acceptancePresent: acceptance != null,
    acceptanceCurrent:
      typeof acceptance?.current === "boolean" ? acceptance.current : undefined,
  };
}

export interface EvidenceDisposition {
  reading: StateReading;
  detail: string;
}

/**
 * The disposition, never a bare "completed": a completion renders only with
 * its verification report, and a report that is not current for the task's
 * fencing epoch is not proof of the current state.
 */
export function evidenceDisposition(view: TaskEvidenceView): EvidenceDisposition {
  if (!view.hasVerification) {
    return {
      reading: { category: "unknown", label: "no verification report", unmapped: false },
      detail: "The daemon holds no verification report for this task — nothing is proven yet.",
    };
  }
  const status = view.verificationStatus ?? "unknown";
  if (view.verificationCurrent === false) {
    return {
      reading: { category: "unknown", label: status, unmapped: false },
      detail:
        "The verification report is not current for the task's fencing epoch — it does not prove the current state.",
    };
  }
  if (status === "passed") {
    return view.acceptancePresent
      ? {
          reading: { category: "completed", label: "passed · accepted", unmapped: false },
          detail: "Verification passed and a terminal acceptance transition is recorded.",
        }
      : {
          reading: { category: "waiting", label: "passed", unmapped: false },
          detail: "Verification passed; no terminal acceptance transition is recorded yet.",
        };
  }
  if (status === "failed") {
    return {
      reading: { category: "blocked", label: "failed", unmapped: false },
      detail: "Verification failed — the task did not meet its acceptance conditions.",
    };
  }
  return {
    reading: { category: "unknown", label: status, unmapped: false },
    detail: `Verification status is ${status} — the outcome is not determined.`,
  };
}

export interface EvidenceRow {
  taskRef: string;
  shortRef: string
  view: TaskEvidenceView;
}

/**
 * Only task refs that actually carry a verification report appear here: this
 * region shows proof, not activity. Newest completion first.
 */
export function recentEvidenceRows(entries: EvidenceRow[]): EvidenceRow[] {
  return [...entries]
    .filter((entry) => entry.view.hasVerification)
    .sort(
      (a, b) =>
        (b.view.completedAt ?? "").localeCompare(a.view.completedAt ?? "") ||
        a.taskRef.localeCompare(b.taskRef),
    );
}

/* ---------- region source plumbing ---------- */

/** Projection statuses that carry data the view may render as content. */
export function hasRenderableData<T>(projection: Projection<T>): boolean {
  return projection.data !== undefined;
}

/** A failure class the region must name rather than silently drop. */
export function isFailedProjection<T>(projection: Projection<T>): boolean {
  return (
    projection.status === "denied" ||
    projection.status === "disconnected" ||
    projection.status === "unknown" ||
    projection.status === "not-run"
  );
}
