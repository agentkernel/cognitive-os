/*
 * Work (W4) view models — docs/design/14, state grammar 22.
 *
 * Two jobs, one module, all pure:
 *
 * 1. The Tier-1 inventory. There is no task-list-with-state API. The only
 *    sources are the daemon's envelope list, the refs this browser session
 *    observed, and bounded per-ref evidence/effects for refs already known.
 *    Everything here says exactly that: nothing is inferred, and a task with
 *    no evidence reads `state not exposed` rather than "running".
 *
 * 2. The governed creation chain. The browser mints no authority: every step
 *    is a daemon call, the daemon derives `clarification_required` from the
 *    candidate's own material-ambiguity facts, and admit is bound to the exact
 *    preview digest and interpretation the operator reviewed.
 *
 * No fetching happens here. Views drive readJson/fetchProjection.
 */

import { readDomainState, type StateReading } from "../../state/stateMap";
import { asRecord } from "../projections";
import type { ProjectionStore } from "../store";
import {
  formatAge,
  mergeCurrentWork,
  type CurrentWorkRow,
  type EffectEntryView,
  type ObservedTask,
  type TaskEnvelopeView,
  type TaskEvidenceView,
} from "./home";

export {
  formatAge,
  shortTaskRef,
  noteObservedTask,
  OBSERVED_TASKS_KEY,
  TASK_LIST_LIMIT,
  projectTaskEffects,
  projectTaskEnvelopes,
  projectTaskEvidence,
  type EffectEntryView,
  type ObservedTask,
  type TaskEnvelopeView,
  type TaskEvidenceView,
} from "./home";

/* ---------- inventory ---------- */

export const WORK_TASKS_KEY = "work:tasks";
export const workEvidenceKey = (taskRef: string) => `work:evidence:${taskRef}`;
export const workEffectsKey = (taskRef: string) => `work:effects:${taskRef}`;

/**
 * Per-ref probes are bounded. The daemon has no task stream and no bulk
 * evidence read, so the page probes only the selected row plus a small
 * prefix of the loaded set — never a fan-out over an unbounded list.
 */
export const WORK_PROBE_LIMIT = 8;

export type WorkOriginFilter = "session" | "all";

export interface WorkRow extends CurrentWorkRow {
  /**
   * Lifecycle is only ever the daemon's own word from /task/evidence. The
   * envelope has none, so this stays undefined until a probe returns one.
   */
  lifecycleState?: string;
  /** Whether this ref was probed at all — an unprobed row is not "unknown". */
  probed: boolean;
}

/**
 * The inventory is the merge of the envelope list and this session's observed
 * refs, deduplicated by `task_ref`, with per-ref lifecycle attached only where
 * a probe actually returned one.
 */
export function buildWorkRows(input: {
  envelopes: TaskEnvelopeView[];
  observed: ObservedTask[];
  evidence: Map<string, TaskEvidenceView | undefined>;
  probed: Set<string>;
}): WorkRow[] {
  return mergeCurrentWork(input.envelopes, input.observed).map((row) => ({
    ...row,
    lifecycleState: input.evidence.get(row.taskRef)?.lifecycleState,
    probed: input.probed.has(row.taskRef),
  }));
}

/**
 * Total, deterministic order so the list never reshuffles under the pointer:
 * session-observed first (newest observation first), then envelope rows by
 * ref. `mergeCurrentWork` already emits exactly this order; sorting again with
 * the same comparator keeps the guarantee local and testable.
 */
export function sortWorkRows(rows: WorkRow[]): WorkRow[] {
  return [...rows].sort((a, b) => {
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

export function filterWorkRows(rows: WorkRow[], origin: WorkOriginFilter): WorkRow[] {
  if (origin === "all") {
    return rows;
  }
  return rows.filter((row) => row.origin === "session" || row.origin === "envelope+session");
}

/**
 * Selection survives a refresh only if the selected ref is still present.
 * Returning undefined (rather than silently selecting a neighbour) keeps the
 * inspector from describing an object the operator did not choose.
 */
export function preserveSelection(
  rows: WorkRow[],
  selected: string | undefined,
): string | undefined {
  if (selected == null) {
    return undefined;
  }
  return rows.some((row) => row.taskRef === selected) ? selected : undefined;
}

/** The resident honesty footer. Never a total — only what is loaded. */
export function inventoryFooter(count: number): string {
  return `Showing ${count} known task${count === 1 ? "" : "s"} · inventory is envelope-only (BD-3)`;
}

/**
 * A row's state cell. Only a real lifecycle word from /task/evidence produces
 * a lifecycle reading; anything else is an explicit S7. An unprobed row says
 * so rather than claiming the state is unknown to the daemon.
 */
export function workRowReading(row: WorkRow): StateReading {
  if (row.lifecycleState != null && row.lifecycleState !== "") {
    return readDomainState("task", row.lifecycleState);
  }
  if (!row.probed) {
    return { category: "unknown", label: "state not exposed", unmapped: false };
  }
  return { category: "unknown", label: "state not exposed", unmapped: false };
}

export const UNKNOWN_FACT = "unknown";

/** Absent facts render the word, never a blank, a zero or a guess. */
export function factOrUnknown(value: string | number | undefined | null): string {
  if (value == null || value === "") {
    return UNKNOWN_FACT;
  }
  return String(value);
}

export function workRowObjective(row: WorkRow): string {
  return row.objective ?? "not exposed by the daemon's task list";
}

export function workRowAge(row: WorkRow, nowMs: number): string {
  const age = formatAge(row.observedAtMs, nowMs);
  return age ?? "unknown (the envelope carries no timestamp)";
}

/** Effects rolled up for the inspector without inventing an aggregate state. */
export interface EffectRollup {
  total: number;
  byStage: { stage: string; count: number }[];
  needsReconcile: number;
}

export function rollUpEffects(effects: EffectEntryView[]): EffectRollup {
  const counts = new Map<string, number>();
  for (const effect of effects) {
    counts.set(effect.stage, (counts.get(effect.stage) ?? 0) + 1);
  }
  return {
    total: effects.length,
    byStage: [...counts.entries()]
      .map(([stage, count]) => ({ stage, count }))
      .sort((a, b) => a.stage.localeCompare(b.stage)),
    needsReconcile: effects.filter(
      (effect) =>
        effect.reconcileClass === "must_reconcile" ||
        effect.reconcileClass === "pending_reconciliation",
    ).length,
  };
}

/**
 * Class-C operations: the daemon exposes no HTTP route for them at all. They
 * are stated as unavailable with the reason — never rendered as a disabled
 * button, which would claim a capability that does not exist.
 */
export const UNSUPPORTED_TASK_OPERATIONS: readonly { operation: string; reason: string }[] = [
  {
    operation: "cancel",
    reason:
      "the daemon exposes no task cancel route; detaching an observation has never cancelled a task",
  },
  {
    operation: "pause",
    reason: "the daemon exposes no task pause route",
  },
  {
    operation: "retry",
    reason:
      "the daemon exposes no task retry route; a correction is a new governed task, not a re-run",
  },
] as const;

/* ---------- governed creation chain ---------- */

export type CreationStep =
  | "describe"
  | "interpretation"
  | "preview"
  | "admitted";

export interface AmbiguityDraft {
  id: string;
  question: string;
  /** Material ambiguities are what force `clarification_required`. */
  material: boolean;
  /** The operator's answer. A material ambiguity is resolved by answering it. */
  answer: string;
}

export interface IntentRecordView {
  userIntentRecordId: string;
  recordedAt?: string;
}

export function projectIntentRecord(body: unknown): IntentRecordView {
  const record = asRecord(body);
  return {
    userIntentRecordId: String(record.user_intent_record_id ?? ""),
    recordedAt: record.recorded_at == null ? undefined : String(record.recorded_at),
  };
}

export interface InterpretationView {
  interpretationId: string;
  interpretationDigest: string;
  materialAmbiguityCount: number;
  /** Verbatim: "candidate" | "clarification_required". */
  status: string;
}

export function projectInterpretation(body: unknown): InterpretationView {
  const record = asRecord(body);
  const count = Number(record.material_ambiguity_count);
  return {
    interpretationId: String(record.interpretation_id ?? ""),
    interpretationDigest: String(record.interpretation_digest ?? ""),
    materialAmbiguityCount: Number.isFinite(count) ? count : 0,
    status: String(record.status ?? "unknown"),
  };
}

export const CLARIFICATION_REQUIRED = "clarification_required";

/**
 * The daemon derives the status from the candidate's material-ambiguity facts
 * and refuses admission with `INTENT_CLARIFICATION_REQUIRED` while one stands.
 * The client mirrors that gate so the operator is stopped at review rather
 * than at a rejected admit.
 */
export function canPreview(interpretation: InterpretationView | undefined): boolean {
  return interpretation != null && interpretation.status !== CLARIFICATION_REQUIRED;
}

export function unresolvedMaterial(ambiguities: AmbiguityDraft[]): AmbiguityDraft[] {
  return ambiguities.filter((item) => item.material && item.answer.trim() === "");
}

/**
 * The candidate sent to the daemon. An answered material ambiguity is no
 * longer an open question, so it leaves the candidate and its answer becomes a
 * recorded assumption — the correction is visible, not silently dropped.
 */
export function candidateFacts(input: {
  objective: string;
  constraints: string[];
  forbidden: string[];
  assumptions: string[];
  ambiguities: AmbiguityDraft[];
  informationGaps: string[];
}): {
  objectives: string[];
  constraints: string[];
  forbidden: string[];
  assumptions: string[];
  ambiguities: { id: string; material: boolean; question: string }[];
  information_gaps: string[];
} {
  const answered = input.ambiguities.filter((item) => item.answer.trim() !== "");
  const open = input.ambiguities.filter((item) => item.answer.trim() === "");
  return {
    objectives: [input.objective],
    constraints: input.constraints,
    forbidden: input.forbidden,
    assumptions: [
      ...input.assumptions,
      ...answered.map((item) => `${item.question} → ${item.answer.trim()}`),
    ],
    ambiguities: open.map((item) => ({
      id: item.id,
      material: item.material,
      question: item.question,
    })),
    information_gaps: input.informationGaps,
  };
}

export interface PreviewView {
  previewDigest: string;
  taskRef: string;
  objective: string;
  conditionCount?: number;
  budget?: Record<string, unknown>;
}

export function projectPreview(body: unknown): PreviewView {
  const record = asRecord(body);
  const conditions = Number(record.condition_count);
  return {
    previewDigest: String(record.preview_digest ?? ""),
    taskRef: String(record.task_ref ?? ""),
    objective: String(record.objective ?? ""),
    conditionCount: Number.isFinite(conditions) ? conditions : undefined,
    budget: record.budget == null ? undefined : asRecord(record.budget),
  };
}

export interface AdmissionView {
  taskRef: string;
  contractEpoch?: number;
  contractDigest?: string;
  taskContractRef?: string;
}

export function projectAdmission(body: unknown): AdmissionView {
  const record = asRecord(body);
  const epoch = Number(record.contract_epoch);
  return {
    taskRef: String(record.task_ref ?? ""),
    contractEpoch: Number.isFinite(epoch) ? epoch : undefined,
    contractDigest: record.contract_digest == null ? undefined : String(record.contract_digest),
    taskContractRef:
      record.task_contract_ref == null ? undefined : String(record.task_contract_ref),
  };
}

export interface AdmitFailure {
  /** True when the operator must re-preview and reconfirm before retrying. */
  requiresFreshPreview: boolean;
  code: string;
  message: string;
}

/**
 * Admit failures are named by cause, and 409 never auto-retries: the daemon
 * rejected the exact digest/acceptance/epoch tuple that was reviewed, so the
 * only honest recovery is a fresh preview and an explicit reconfirmation.
 */
export function admitFailure(status: number, body: unknown): AdmitFailure {
  const record = asRecord(body);
  const nested = asRecord(record.error);
  const code = String(record.code ?? nested.code ?? `HTTP_${status}`);
  if (status === 403) {
    return {
      requiresFreshPreview: false,
      code,
      message:
        "The daemon refused this acceptance: `accepted_by` must equal the authenticated principal of this Task session. Re-issue the session as the intended principal, or change the principal on the Session page, then preview again.",
    };
  }
  if (status === 409) {
    return {
      requiresFreshPreview: true,
      code,
      message:
        "The daemon rejected the preview digest, the acceptance, or the epoch CAS. Nothing was admitted and nothing is retried automatically — generate a fresh preview and confirm it again.",
    };
  }
  return {
    requiresFreshPreview: false,
    code,
    message: `The daemon refused the admission with HTTP ${status}. Nothing was admitted.`,
  };
}

export function chainErrorMessage(step: string, status: number, body: unknown): string {
  const record = asRecord(body);
  const nested = asRecord(record.error);
  const code = String(record.code ?? nested.code ?? `HTTP_${status}`);
  return `${step} failed — HTTP ${status} ${code}. Nothing was admitted.`;
}

/* ---------- session chain memory (what W5 composes from) ---------- */

export const WORK_CHAIN_KEY = "work:session-chains";
export const WORK_CHAIN_SOURCE = "session-recorded chain (this browser session only)";

/**
 * The daemon keeps no queryable record of an intent record, an interpretation
 * or a preview once admission has happened — `/task/evidence` and
 * `/task/effects` start at the admitted contract. So this exact sentence is
 * what a detail view must say instead of implying a preview was lost.
 */
export const PREVIEW_EPHEMERAL_NOTE =
  "Previews are ephemeral by design; the admitted contract is the durable record.";

export interface SessionChainInterpretation {
  interpretationId: string;
  interpretationDigest: string;
  /** Verbatim daemon status at the moment of admission. */
  status: string;
  materialAmbiguityCount: number;
  /** Ambiguities still open when the contract was admitted. */
  openAmbiguities: { id: string; question: string; material: boolean }[];
  /** Answers the operator recorded, which became candidate assumptions. */
  recordedDecisions: string[];
  informationGaps: string[];
  /** Earlier interpretations this session superseded, oldest first. */
  supersededInterpretationIds: string[];
}

/**
 * One admitted task's governed chain, as this session actually ran it.
 *
 * This lives only in the in-memory projection store: it is this tab's own
 * record, it is never written to localStorage, sessionStorage or the URL, and
 * it dies with the tab. A detail view for a ref this session did not admit
 * therefore has no chain at all, and must say so rather than reconstruct one.
 */
export interface SessionTaskChain {
  taskRef: string;
  admittedAtMs: number;
  intent: {
    userIntentRecordId: string;
    rawExpression: string;
    recordedAt?: string;
  };
  interpretation: SessionChainInterpretation;
  /**
   * The preview digest the operator actually reviewed. Marked ephemeral
   * because there is no daemon route that can return it again.
   */
  preview: {
    previewDigest: string;
    objective: string;
    conditionCount?: number;
    ephemeral: true;
  };
  admission: AdmissionView & { acceptedBy: string };
}

export function readSessionChains(store: ProjectionStore): SessionTaskChain[] {
  return store.get<SessionTaskChain[]>(WORK_CHAIN_KEY)?.data ?? [];
}

/** The chain for one ref, or undefined when this session did not admit it. */
export function sessionChainFor(
  store: ProjectionStore,
  taskRef: string,
): SessionTaskChain | undefined {
  return readSessionChains(store).find((chain) => chain.taskRef === taskRef);
}

/** Deduplicated by ref, newest admission first. Memory only, by design. */
export function noteSessionChain(store: ProjectionStore, chain: SessionTaskChain): void {
  const existing = readSessionChains(store).filter((row) => row.taskRef !== chain.taskRef);
  const rows = [...existing, chain].sort((a, b) => b.admittedAtMs - a.admittedAtMs);
  store.set<SessionTaskChain[]>(WORK_CHAIN_KEY, {
    status: rows.length === 0 ? "empty" : "ready",
    data: rows,
    source: WORK_CHAIN_SOURCE,
    updatedAt: Date.now(),
  });
}
