/*
 * Work detail (W5) view models — docs/design/15, state grammar 22.
 *
 * The detail view composes four different kinds of fact, and the whole point
 * of this module is that they never get mixed:
 *
 *   authority   — GET /task/evidence lifecycle transitions. These are state
 *                 changes the daemon durably recorded.
 *   observation — GET /task/observation (bounded o4/o5). These are samples.
 *                 An observation is NOT a state transition and must never be
 *                 rendered as one.
 *   effects     — GET /task/effects. External mutation attempts.
 *   context     — GET /task/resource/v1/consumption. Durable Memory/Skill pins.
 *
 * The session's own intent -> interpretation -> preview -> admission chain
 * comes from the W4 in-memory projection. The daemon has no route that returns
 * a preview after the fact, so a missing chain is stated, never reconstructed.
 *
 * No fetching happens here. Views drive readJson/fetchProjection.
 */

import { asList, asRecord } from "../projections";

/* ---------- keys ---------- */

export const detailEvidenceKey = (taskRef: string) => `detail:evidence:${taskRef}`;
export const detailEffectsKey = (taskRef: string) => `detail:effects:${taskRef}`;
export const detailObservationKey = (taskRef: string, family: string) =>
  `detail:observation:${family}:${taskRef}`;
export const detailConsumptionKey = (taskRef: string) => `detail:consumption:${taskRef}`;

/** The observation families this view reads. Nothing else is probed. */
export const DETAIL_OBSERVATION_FAMILIES = ["o4", "o5"] as const;
export type DetailObservationFamily = (typeof DETAIL_OBSERVATION_FAMILIES)[number];

/* ---------- sections ---------- */

export const DETAIL_SECTIONS = [
  { id: "overview", title: "Overview" },
  { id: "run", title: "Run" },
  { id: "effects", title: "Effects" },
  { id: "evidence", title: "Evidence" },
  { id: "intent", title: "Intent & Contract" },
  { id: "context", title: "Context" },
] as const;

export type DetailSectionId = (typeof DETAIL_SECTIONS)[number]["id"];

export function isDetailSection(value: string | null | undefined): value is DetailSectionId {
  return DETAIL_SECTIONS.some((section) => section.id === value);
}

/* ---------- copy that carries a boundary ---------- */

/** An interval the daemon recorded nothing for. Never "nothing happened". */
export const NO_RECORDED_FACTS = "no recorded facts";

/**
 * The daemon exposes no route that returns a preview after admission, so a
 * detail view for a ref this session did not admit has no preview to show.
 * This is the honest statement, not "preview lost" or "preview unavailable".
 */
export const PREVIEW_EPHEMERAL_STATEMENT =
  "Previews are ephemeral by design; the admitted contract is the durable record.";

/**
 * Capabilities the daemon has no HTTP surface for at all. Stated as facts with
 * their reason; never rendered as controls, not even disabled ones.
 */
export const DETAIL_UNAVAILABLE_VIEWS: readonly { subject: string; reason: string }[] = [
  {
    subject: "Loop / DECIDE trace",
    reason:
      "the daemon exposes no Loop iteration route, so the decision trace behind a run cannot be read over HTTP",
  },
  {
    subject: "WIA (work-in-attention) set",
    reason: "the daemon exposes no WIA route, so what the agent was attending to cannot be read",
  },
  {
    subject: "Context assembly detail",
    reason:
      "only the durable consumption pins are exposed; how the context was assembled is not readable over HTTP",
  },
] as const;

/* ---------- authority: GET /task/evidence ---------- */

export interface LifecycleTransition {
  sequence: number;
  eventRef: string;
  eventType: string;
  afterState?: string;
  afterVersion: number;
  reasonCode?: string;
  eventTime?: string;
}

export interface TaskEvidenceDetail {
  taskRef?: string;
  contractEpoch?: number;
  currentState?: string;
  currentVersion?: number;
  transitions: LifecycleTransition[];
  /** The daemon's own flag: earlier transitions exist but were not returned. */
  transitionsTruncated: boolean;
  intentRefs: string[];
  effectRefs: string[];
  reconcileClass?: string;
  verification?: {
    reportRef?: string;
    reportDigest?: string;
    status?: string;
    completedAt?: string;
    current?: boolean;
    artifactRefs: string[];
    artifactsCurrent?: boolean;
  };
  acceptance?: {
    terminalTransitionRef?: string;
    terminalTransitionDigest?: string;
    current?: boolean;
  };
  durableCursor?: {
    eventSequence?: number;
    taskVersion?: number;
    terminalTransitionSequence?: number;
  };
}

function optionalNumber(value: unknown): number | undefined {
  const parsed = Number(value);
  return value == null || !Number.isFinite(parsed) ? undefined : parsed;
}

function optionalString(value: unknown): string | undefined {
  return value == null ? undefined : String(value);
}

function optionalBoolean(value: unknown): boolean | undefined {
  return typeof value === "boolean" ? value : undefined;
}

export function projectEvidenceDetail(body: unknown): TaskEvidenceDetail {
  const record = asRecord(body);
  const lifecycle = asRecord(record.lifecycle);
  const verification =
    record.latest_verification == null ? undefined : asRecord(record.latest_verification);
  const acceptance =
    record.latest_acceptance == null ? undefined : asRecord(record.latest_acceptance);
  const cursor = record.durable_cursor == null ? undefined : asRecord(record.durable_cursor);
  return {
    taskRef: optionalString(record.task_ref),
    contractEpoch: optionalNumber(record.contract_epoch),
    currentState: optionalString(lifecycle.current_state),
    currentVersion: optionalNumber(lifecycle.current_version),
    transitions: asList(lifecycle, ["transitions"]).map((row) => {
      const entry = asRecord(row);
      return {
        sequence: optionalNumber(entry.sequence) ?? 0,
        eventRef: String(entry.event_ref ?? "unknown"),
        eventType: String(entry.event_type ?? "unknown"),
        afterState: optionalString(entry.after_state),
        afterVersion: optionalNumber(entry.after_version) ?? 0,
        reasonCode: optionalString(entry.reason_code),
        eventTime: optionalString(entry.event_time),
      };
    }),
    transitionsTruncated: lifecycle.transitions_truncated === true,
    intentRefs: asList(record, ["intent_refs"]).map((row) => String(row)),
    effectRefs: asList(record, ["effect_refs"]).map((row) => String(row)),
    reconcileClass: optionalString(record.reconcile_class),
    verification:
      verification == null
        ? undefined
        : {
            reportRef: optionalString(verification.report_ref),
            reportDigest: optionalString(verification.report_digest),
            status: optionalString(verification.status),
            completedAt: optionalString(verification.completed_at),
            current: optionalBoolean(verification.current),
            artifactRefs: asList(verification, ["artifact_refs"]).map((row) => String(row)),
            artifactsCurrent: optionalBoolean(verification.artifacts_current),
          },
    acceptance:
      acceptance == null
        ? undefined
        : {
            terminalTransitionRef: optionalString(acceptance.terminal_transition_ref),
            terminalTransitionDigest: optionalString(acceptance.terminal_transition_digest),
            current: optionalBoolean(acceptance.current),
          },
    durableCursor:
      cursor == null
        ? undefined
        : {
            eventSequence: optionalNumber(cursor.event_sequence),
            taskVersion: optionalNumber(cursor.task_version),
            terminalTransitionSequence: optionalNumber(cursor.terminal_transition_sequence),
          },
  };
}

/* ---------- the authority lane ---------- */

export type AuthorityLaneRow =
  | { kind: "bounded"; note: string }
  | { kind: "gap"; missingVersions: number; note: string }
  | { kind: "transition"; transition: LifecycleTransition }
  | { kind: "empty"; note: string };

/**
 * The authority lane, in recorded order, with its own honesty rows.
 *
 * Two things are made explicit rather than smoothed over: the daemon's own
 * `transitions_truncated` flag becomes a leading bounded row, and a jump in
 * `after_version` between two returned transitions becomes a gap row — the
 * versions in between existed and were not returned, so the interval carries
 * no recorded facts rather than appearing continuous.
 */
export function composeAuthorityLane(evidence: TaskEvidenceDetail): AuthorityLaneRow[] {
  const ordered = [...evidence.transitions].sort((a, b) => a.sequence - b.sequence);
  if (ordered.length === 0) {
    return [
      {
        kind: "empty",
        note: evidence.transitionsTruncated
          ? `${NO_RECORDED_FACTS} in the returned window — the daemon reports the transition scan was bounded`
          : `${NO_RECORDED_FACTS}: the daemon returned no lifecycle transition for this task`,
      },
    ];
  }
  const rows: AuthorityLaneRow[] = [];
  if (evidence.transitionsTruncated) {
    rows.push({
      kind: "bounded",
      note: "Earlier transitions exist but were not returned: the daemon reports this transition scan as bounded.",
    });
  }
  let previous: LifecycleTransition | undefined;
  for (const transition of ordered) {
    if (previous && transition.afterVersion - previous.afterVersion > 1) {
      rows.push({
        kind: "gap",
        missingVersions: transition.afterVersion - previous.afterVersion - 1,
        note: `${NO_RECORDED_FACTS} for this interval — the returned window skips versions ${
          previous.afterVersion + 1
        }–${transition.afterVersion - 1}.`,
      });
    }
    rows.push({ kind: "transition", transition });
    previous = transition;
  }
  return rows;
}

/* ---------- observation: GET /task/observation ---------- */

export interface ObservationCounter {
  name: string;
  denominator: number;
  observedZero: boolean;
  negativeControl: string;
  count: number;
}

export interface ObservationView {
  family: string;
  taskRef?: string;
  denominator: number;
  observedZero: boolean;
  samplesTruncated: boolean;
  negativeControl?: string;
  authoritySideEffects: boolean;
  counters: ObservationCounter[];
  /** o5 effect samples, kept separate from /task/effects authority rows. */
  effectSamples: { effectRef?: string; stage?: string; outcomeClass?: string }[];
}

export function projectObservation(body: unknown): ObservationView {
  const record = asRecord(body);
  const counters = asRecord(record.counters);
  return {
    family: String(record.family ?? "unknown"),
    taskRef: optionalString(record.task_ref),
    denominator: optionalNumber(record.denominator) ?? 0,
    observedZero: record.observed_zero === true,
    samplesTruncated: record.samples_truncated === true,
    negativeControl: optionalString(record.negative_control),
    authoritySideEffects: record.authority_side_effects === true,
    counters: Object.entries(counters).map(([name, value]) => {
      const entry = asRecord(value);
      return {
        name,
        denominator: optionalNumber(entry.denominator) ?? 0,
        observedZero: entry.observed_zero === true,
        negativeControl: String(entry.negative_control ?? "unknown"),
        count: optionalNumber(entry.count) ?? 0,
      };
    }),
    effectSamples: asList(record, ["effects"]).map((row) => {
      const entry = asRecord(row);
      return {
        effectRef: optionalString(entry.effect_ref),
        stage: optionalString(entry.stage),
        outcomeClass: optionalString(entry.outcome_class),
      };
    }),
  };
}

export type ObservationLaneRow =
  | { kind: "counter"; counter: ObservationCounter }
  | { kind: "sample"; label: string; detail: string }
  | { kind: "empty"; note: string }
  | { kind: "bounded"; note: string };

/**
 * The observation lane. Every row is explicitly a sample, never a transition:
 * a zero here is a named observed zero, and a bounded sample set says so.
 */
export function composeObservationLane(views: ObservationView[]): ObservationLaneRow[] {
  const rows: ObservationLaneRow[] = [];
  for (const view of views) {
    if (view.samplesTruncated) {
      rows.push({
        kind: "bounded",
        note: `The ${view.family} sample set is bounded: more samples exist than were returned.`,
      });
    }
    for (const counter of view.counters) {
      rows.push({ kind: "counter", counter });
    }
    for (const sample of view.effectSamples) {
      rows.push({
        kind: "sample",
        label: `${view.family} effect sample`,
        detail: `${sample.stage ?? "unknown"} · ${sample.outcomeClass ?? "unknown"} · ${
          sample.effectRef ?? "unknown"
        }`,
      });
    }
  }
  const hasFacts = rows.some((row) => row.kind === "counter" || row.kind === "sample");
  if (!hasFacts) {
    const empty: ObservationLaneRow = {
      kind: "empty",
      note: `${NO_RECORDED_FACTS}: the bounded observation families returned no sample for this task. An observed zero is a measurement, not a claim about progress.`,
    };
    rows.push(empty);
  }
  return rows;
}

/**
 * Watch is not attached from this view. W11 owns streaming; until then the
 * page must not imply live delivery, and detaching has never been a control
 * over the Task or the Agent.
 */
export const WATCH_NOT_ATTACHED = {
  state: "not attached",
  detail:
    "This view attaches no watch stream, so there is no live delivery here and progress is unknown rather than idle. Attaching, detaching or losing a watch has never cancelled a Task or stopped an Agent.",
} as const;

/* ---------- effects: GET /task/effects ---------- */

export interface EffectHistoryEntry {
  effectRef: string;
  stage: string;
  outcomeClass: string;
  reconcileClass: string;
  originalKeyDigest?: string;
  mutationCount?: number;
  fixedPostStateRef?: string;
  reportRef?: string;
}

export interface EffectHistoryView {
  taskRef?: string;
  contractEpoch?: number;
  entries: EffectHistoryEntry[];
  effectsTruncated: boolean;
  authoritySideEffects: boolean;
}

export function projectEffectHistory(body: unknown): EffectHistoryView {
  const record = asRecord(body);
  return {
    taskRef: optionalString(record.task_ref),
    contractEpoch: optionalNumber(record.contract_epoch),
    entries: asList(record, ["effects"]).map((row) => {
      const entry = asRecord(row);
      return {
        effectRef: String(entry.effect_ref ?? "unknown"),
        stage: String(entry.stage ?? "unknown"),
        outcomeClass: String(entry.outcome_class ?? "unknown"),
        reconcileClass: String(entry.reconcile_class ?? "unknown"),
        originalKeyDigest: optionalString(entry.original_key_digest),
        mutationCount: optionalNumber(entry.mutation_count),
        fixedPostStateRef: optionalString(entry.fixed_post_state_ref),
        reportRef: optionalString(entry.report_ref),
      };
    }),
    effectsTruncated: record.effects_truncated === true,
    authoritySideEffects: record.authority_side_effects === true,
  };
}

/**
 * Stages that need the operator first. `OUTCOME_UNKNOWN` means the daemon does
 * not know whether the external world changed, and `VERIFY_FAILED` means a
 * mutation was attempted and its verification failed.
 */
export const EFFECT_PRIORITY_STAGES: readonly string[] = ["OUTCOME_UNKNOWN", "VERIFY_FAILED"];

export function effectNeedsAttention(entry: EffectHistoryEntry): boolean {
  return (
    EFFECT_PRIORITY_STAGES.includes(entry.stage) ||
    entry.outcomeClass === "failed" ||
    entry.outcomeClass === "indeterminate" ||
    entry.reconcileClass === "must_reconcile" ||
    entry.reconcileClass === "pending_reconciliation"
  );
}

/** Failures and indeterminates first; stable within each group. */
export function sortEffectsByAttention(entries: EffectHistoryEntry[]): EffectHistoryEntry[] {
  const rank = (entry: EffectHistoryEntry) => {
    if (EFFECT_PRIORITY_STAGES.includes(entry.stage)) {
      return 0;
    }
    if (entry.outcomeClass === "failed") {
      return 1;
    }
    if (effectNeedsAttention(entry)) {
      return 2;
    }
    return 3;
  };
  return [...entries].sort((a, b) => rank(a) - rank(b));
}

/**
 * An empty effect history means no external mutation was recorded. It is not a
 * statement that the work succeeded, and this copy is what prevents that read.
 */
export const NO_EFFECTS_RECORDED =
  "The daemon recorded no external mutation for this task. That is an absence of recorded mutation, not a successful outcome.";

/* ---------- evidence completion ---------- */

export type CompletionDisposition =
  | "completed"
  | "verified-not-accepted"
  | "verification-not-current"
  | "verification-failed"
  | "no-verification"
  | "not-recorded";

export interface CompletionReading {
  disposition: CompletionDisposition;
  /** The word shown to the operator. Only one disposition may say completed. */
  label: string;
  detail: string;
}

/**
 * The single place the word "completed" may be produced.
 *
 * A passing verification report is not Task completion: completion requires a
 * terminal acceptance record that is current for the task's fencing epoch. A
 * report that is not current proves nothing about the current state, and a
 * missing report proves nothing at all.
 */
export function completionReading(
  evidence: TaskEvidenceDetail | undefined,
  evidencePresent: boolean,
): CompletionReading {
  if (!evidencePresent || evidence == null) {
    return {
      disposition: "not-recorded",
      label: "No terminal evidence recorded",
      detail:
        "The daemon holds no terminal evidence for this task ref. Nothing is proven, and nothing is claimed about whether the task ran.",
    };
  }
  const verification = evidence.verification;
  const acceptance = evidence.acceptance;
  if (verification == null) {
    return {
      disposition: "no-verification",
      label: "no verification report",
      detail:
        "The daemon holds no verification report for this task, so nothing has been independently verified yet.",
    };
  }
  if (verification.current === false) {
    return {
      disposition: "verification-not-current",
      label: `${verification.status ?? "unknown"} (not current)`,
      detail:
        "The verification report is not current for the task's fencing epoch, so it does not prove the current state and is never read as an acceptance.",
    };
  }
  if (verification.status !== "passed") {
    return {
      disposition: "verification-failed",
      label: verification.status ?? "unknown",
      detail:
        "The independent verification did not pass. The task is not complete and no acceptance follows from this report.",
    };
  }
  if (acceptance == null) {
    return {
      disposition: "verified-not-accepted",
      label: "verified, not accepted",
      detail:
        "Verification passed, but the daemon holds no terminal acceptance record. A passing verification is not Task completion.",
    };
  }
  if (acceptance.current === false) {
    return {
      disposition: "verified-not-accepted",
      label: "acceptance not current",
      detail:
        "A terminal acceptance exists but is not current for the task's fencing epoch, so it does not establish completion of the current contract.",
    };
  }
  return {
    disposition: "completed",
    label: "completed",
    detail:
      "Verification passed and is current, and a current terminal acceptance record exists. This is the only combination under which this view says completed.",
  };
}

/* ---------- context: GET /task/resource/v1/consumption ---------- */

export interface ConsumptionView {
  taskRef?: string;
  contractEpoch?: number;
  contextRequestId?: string;
  contextRequestDigest?: string;
  sessionRef?: string;
  reuseOf?: string;
  decisionClass?: string;
  authoritySideEffects: boolean;
  memoryPins: { memoryId: string; sourceId?: string; sourceDigest?: string }[];
  skillPins: {
    bindingId: string;
    revisionId?: string;
    packageId?: string;
    contentDigest?: string;
  }[];
}

export function projectConsumption(body: unknown): ConsumptionView {
  const record = asRecord(body);
  return {
    taskRef: optionalString(record.task_ref),
    contractEpoch: optionalNumber(record.contract_epoch),
    contextRequestId: optionalString(record.context_request_id),
    contextRequestDigest: optionalString(record.context_request_digest),
    sessionRef: optionalString(record.session_ref),
    reuseOf: optionalString(record.reuse_of),
    decisionClass: optionalString(record.decision_class),
    authoritySideEffects: record.authority_side_effects === true,
    memoryPins: asList(record, ["memory"]).map((row) => {
      const entry = asRecord(row);
      return {
        memoryId: String(entry.memory_id ?? "unknown"),
        sourceId: optionalString(entry.source_id),
        sourceDigest: optionalString(entry.source_digest),
      };
    }),
    skillPins: asList(record, ["skill"]).map((row) => {
      const entry = asRecord(row);
      return {
        bindingId: String(entry.binding_id ?? "unknown"),
        revisionId: optionalString(entry.revision_id),
        packageId: optionalString(entry.package_id),
        contentDigest: optionalString(entry.content_digest),
      };
    }),
  };
}

/**
 * The consumption route has several distinct refusals and they mean different
 * things. Collapsing them into "unavailable" would hide a real conflict, so
 * each one is named.
 */
export function consumptionRefusal(code: string | undefined): string {
  switch (code) {
    case "RESOURCE_CONSUMPTION_NOT_FOUND":
      return "The daemon holds no durable Memory/Skill consumption record for this task. Nothing was pinned, or the record was never written.";
    case "RESOURCE_TASK_NOT_FOUND":
      return "The daemon has no current contract for this task ref, so it has no context to report.";
    case "RESOURCE_TASK_CONTEXT_MISSING":
      return "The task's ContextRequest is unavailable, so the pins cannot be revalidated. This is a missing authority record, not an empty context.";
    case "RESOURCE_TASK_CONTEXT_MISMATCH":
      return "The task's ContextRequest does not match the current task contract — a real conflict, reported rather than resolved here.";
    case "RESOURCE_CONSUMPTION_NOT_ELIGIBLE":
      return "The durable consumption request digest differs from the current request, so the stored pins are stale for this contract.";
    case "RESOURCE_CONSUMPTION_UNAVAILABLE":
      return "The Memory/Skill consumption store is unavailable, so the pins could not be read.";
    case "RESOURCE_CONSUMPTION_RESTATEMENT_FORBIDDEN":
      return "The daemon refuses restated queries on this route; only durable pins may be read.";
    default:
      return code == null
        ? "The consumption record could not be read and the daemon returned no error class."
        : `The consumption record could not be read — ${code}.`;
  }
}
