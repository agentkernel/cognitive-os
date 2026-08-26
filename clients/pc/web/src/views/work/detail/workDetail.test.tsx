import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "../../../App";
import { noteObservedTask, noteSessionChain } from "../../../data/projections/work";
import {
  NO_RECORDED_FACTS,
  PREVIEW_EPHEMERAL_STATEMENT,
  completionReading,
  composeAuthorityLane,
  composeObservationLane,
  composeWatchObservation,
  consumptionRefusal,
  effectNeedsAttention,
  projectEffectHistory,
  projectEvidenceDetail,
  projectObservation,
  sortEffectsByAttention,
} from "../../../data/projections/workDetail";
import { appProjections } from "../../../data/store";
import { clearSession, rememberBearer } from "../../../session";

/* ---------- harness ---------- */

type RouteResponse = { status: number; body: unknown; contentType?: string };
type RouteHandler = RouteResponse | ((call: { url: URL }) => RouteResponse);

interface RecordedCall {
  method: string;
  path: string;
  query: URLSearchParams;
  authorization: string | null;
  accept: string | null;
}

const TASK_A = "task://personal/web-ui/0193c100-0000-7000-8000-000000000001";
const TASK_MISSING = "task://personal/web-ui/0193cfff-0000-7000-8000-00000000ffff";
const REPORT_DIGEST = `sha256:${"f".repeat(64)}`;
const INTERPRETATION_DIGEST = `sha256:${"b".repeat(64)}`;
const PREVIEW_DIGEST = `sha256:${"c".repeat(64)}`;

function watchSse(taskRef = TASK_A): string {
  return [
    `event: snapshot\ndata: {"kind":"snapshot","latest_sequence":2,"tasks":[]}\n\n`,
    `id: 2\nevent: delta\ndata: {"kind":"delta","sequence":2,"event":{"kind":"task.admitted","body":{"task_ref":${JSON.stringify(taskRef)},"contract_epoch":3}}}\n\n`,
  ].join("");
}

function envelope(taskRef: string, epoch: number): Record<string, unknown> {
  return {
    id: taskRef,
    family: "task",
    object_version: epoch,
    health: "contracted",
    revision_digest: `sha256:${"a".repeat(64)}`,
    blocked_reason: null,
  };
}

function transition(
  sequence: number,
  afterVersion: number,
  afterState: string,
  reasonCode?: string,
): Record<string, unknown> {
  return {
    sequence,
    event_ref: `event://ev-${sequence}`,
    event_type: "task.transition",
    after_state: afterState,
    after_version: afterVersion,
    reason_code: reasonCode ?? null,
    event_time: `2026-08-25T0${sequence}:00:00Z`,
  };
}

function evidenceBody(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    schema_version: 1,
    task_ref: TASK_A,
    contract_epoch: 3,
    lifecycle: {
      current_state: "COMPLETED",
      current_version: 4,
      transitions: [
        transition(11, 1, "ADMITTED"),
        transition(12, 2, "ACTIVE"),
        transition(15, 4, "COMPLETED", "verified"),
      ],
      transitions_truncated: false,
    },
    intent_refs: ["intent://i-1"],
    effect_refs: ["effect://e-1"],
    reconcile_class: "closed",
    latest_verification: {
      report_ref: "report://r-1",
      report_digest: REPORT_DIGEST,
      status: "passed",
      completed_at: "2026-08-25T05:00:00Z",
      current: true,
      artifact_refs: ["artifact://a-1"],
      artifacts_current: true,
    },
    latest_acceptance: {
      terminal_transition_ref: "event://ev-15",
      terminal_transition_digest: `sha256:${"e".repeat(64)}`,
      current: true,
    },
    durable_cursor: { event_sequence: 15, task_version: 4, terminal_transition_sequence: 15 },
    ...overrides,
  };
}

function effectsBody(): Record<string, unknown> {
  return {
    schema_version: 1,
    task_ref: TASK_A,
    contract_epoch: 3,
    effects: [
      {
        effect_ref: "effect://ok-1",
        stage: "RECONCILED",
        outcome_class: "executed",
        reconcile_class: "reconciled",
        original_key_digest: `sha256:${"1".repeat(64)}`,
        mutation_count: 1,
        fixed_post_state_ref: "state://post-1",
        report_ref: "report://eff-1",
      },
      {
        effect_ref: "effect://unknown-1",
        stage: "OUTCOME_UNKNOWN",
        outcome_class: "indeterminate",
        reconcile_class: "must_reconcile",
        original_key_digest: `sha256:${"2".repeat(64)}`,
        mutation_count: null,
        fixed_post_state_ref: null,
        report_ref: null,
      },
    ],
    effects_truncated: true,
    authority_side_effects: false,
  };
}

function observationBody(family: string): Record<string, unknown> {
  if (family === "o5") {
    return {
      schema_version: 1,
      kind: "observation.plane",
      family: "o5",
      task_ref: TASK_A,
      contract_epoch: 3,
      denominator: 1,
      observed_zero: false,
      samples_truncated: false,
      authority_side_effects: false,
      negative_control: "effect_history_recorded",
      effects: [{ effect_ref: "effect://ok-1", stage: "RECONCILED", outcome_class: "executed" }],
    };
  }
  return {
    schema_version: 1,
    kind: "observation.plane",
    family: "o4",
    task_ref: TASK_A,
    denominator: 2,
    observed_zero: false,
    samples_truncated: false,
    authority_side_effects: false,
    negative_control: "scheduler_probe_recorded",
    counters: {
      queue_wait: {
        denominator: 1,
        observed_zero: false,
        negative_control: "queue_wait_recorded",
        count: 3,
      },
      fairness: {
        denominator: 0,
        observed_zero: true,
        negative_control: "no_fairness_sample",
        count: 0,
      },
    },
  };
}

function consumptionBody(): Record<string, unknown> {
  return {
    kind: "task.resource.consumption",
    authority_source: "daemon-memory-skill-consumption",
    task_ref: TASK_A,
    contract_epoch: 3,
    context_request_id: "ctx-1",
    context_request_digest: `sha256:${"9".repeat(64)}`,
    session_ref: "session://task/1",
    reuse_of: null,
    decision_class: "authorized_exact_pin",
    memory: [{ memory_id: "mem-1", source_id: "src-1", source_digest: `sha256:${"7".repeat(64)}` }],
    skill: [
      {
        binding_id: "bind-1",
        revision_id: "rev-1",
        package_id: "pkg-1",
        content_digest: `sha256:${"8".repeat(64)}`,
      },
    ],
    authority_side_effects: false,
  };
}

function detailRoutes(overrides: Record<string, RouteHandler> = {}): Record<string, RouteHandler> {
  return {
    "GET /management/resource/v1/list": {
      status: 200,
      body: { status: "ok", resources: [envelope(TASK_A, 3)] },
    },
    // Ref-aware: a ref the daemon does not know has no evidence at all.
    "GET /task/evidence": ({ url }) =>
      url.searchParams.get("task_ref") === TASK_A
        ? { status: 200, body: evidenceBody() }
        : { status: 404, body: { status: "error", code: "TASK_EVIDENCE_NOT_FOUND" } },
    "GET /task/effects": ({ url }) =>
      url.searchParams.get("task_ref") === TASK_A
        ? { status: 200, body: effectsBody() }
        : { status: 404, body: { status: "error", code: "TASK_EFFECT_HISTORY_NOT_FOUND" } },
    "GET /task/observation": ({ url }) => ({
      status: 200,
      body: observationBody(url.searchParams.get("family") ?? "o4"),
    }),
    "GET /task/resource/v1/consumption": { status: 200, body: consumptionBody() },
    "GET /task/watch": {
      status: 200,
      contentType: "text/event-stream",
      body: watchSse(),
    },
    ...overrides,
  };
}

function defaultRoute(path: string): RouteResponse {
  if (path === "/personal/health") {
    return { status: 200, body: { status: "ok" } };
  }
  if (path === "/personal/status") {
    return { status: 200, body: { status: "ok", overall: "ready", components: [] } };
  }
  return { status: 404, body: { status: "error", code: "NOT_FOUND", message: "not found" } };
}

function installFetch(routes: Record<string, RouteHandler>): RecordedCall[] {
  const calls: RecordedCall[] = [];
  const fetchMock = vi.fn(async (input: unknown, init?: RequestInit) => {
    const url = new URL(String(input), "http://localhost");
    const method = (init?.method ?? "GET").toUpperCase();
    const headers = new Headers(init?.headers);
    calls.push({
      method,
      path: url.pathname,
      query: url.searchParams,
      authorization: headers.get("Authorization"),
      accept: headers.get("accept") ?? headers.get("Accept"),
    });
    const handler = routes[`${method} ${url.pathname}`];
    const resolved =
      typeof handler === "function" ? handler({ url }) : (handler ?? defaultRoute(url.pathname));
    const isStream =
      resolved.contentType?.includes("event-stream") === true || typeof resolved.body === "string";
    return new Response(isStream ? String(resolved.body) : JSON.stringify(resolved.body), {
      status: resolved.status,
      headers: { "content-type": resolved.contentType ?? "application/json" },
    });
  });
  vi.stubGlobal("fetch", fetchMock);
  return calls;
}

function renderAppAt(hash: string) {
  window.location.hash = hash;
  const host = document.createElement("div");
  document.body.appendChild(host);
  const root = createRoot(host);
  act(() => {
    root.render(<App />);
  });
  return { host, root };
}

async function flush(ticks = 14) {
  for (let i = 0; i < ticks; i += 1) {
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 0));
    });
  }
}

function unmount(host: HTMLDivElement, root: ReturnType<typeof createRoot>) {
  act(() => {
    root.unmount();
  });
  host.remove();
}

function text(host: HTMLElement): string {
  return (host.textContent ?? "").replace(/\s+/g, " ");
}

function clickNamed(host: HTMLElement, label: string): void {
  const match = [...host.querySelectorAll("button")].find(
    (el) => (el.textContent ?? "").trim() === label,
  );
  expect(match).toBeTruthy();
  act(() => {
    (match as HTMLButtonElement).click();
  });
}

function detailHash(taskRef: string, search = ""): string {
  return `#/work/${encodeURIComponent(taskRef)}${search}`;
}

/**
 * The disposition the page actually asserts, read from the Evidence chip.
 * Scoped deliberately: the honesty copy explains when "completed" may be used,
 * so searching the page text for that word proves nothing either way.
 */
function dispositionLabel(host: HTMLElement): string {
  const section = host.querySelector("#section-evidence");
  return (section?.querySelector(".cp-chip")?.textContent ?? "").trim();
}

beforeEach(() => {
  appProjections.clear();
  clearSession();
  rememberBearer("management", "mgmt-token");
  rememberBearer("task", "task-token");
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
  appProjections.clear();
  clearSession();
  window.location.hash = "";
});

/* ---------- lane composition ---------- */

describe("Run timeline lanes", () => {
  it("keeps authority transitions and observation samples in separate lanes", async () => {
    installFetch(detailRoutes());
    const { host, root } = renderAppAt(detailHash(TASK_A));
    await flush();

    const authority = host.querySelector(".cp-lane--authority") as HTMLElement;
    const observation = host.querySelector(".cp-lane--observation") as HTMLElement;
    expect(authority).not.toBeNull();
    expect(observation).not.toBeNull();

    // A transition never appears in the observation lane, and a sampled
    // counter never appears in the authority lane.
    expect(text(authority)).toContain("task.transition");
    expect(text(authority)).not.toContain("queue_wait");
    expect(text(observation)).toContain("queue_wait");
    expect(text(observation)).not.toContain("task.transition");
    expect(text(observation)).toContain("never means the task moved");
    unmount(host, root);
  });

  it("orders transitions by sequence and marks a version gap as no recorded facts", () => {
    const evidence = projectEvidenceDetail(evidenceBody());
    const lane = composeAuthorityLane(evidence);
    const kinds = lane.map((row) => row.kind);
    expect(kinds).toEqual(["transition", "transition", "gap", "transition"]);
    const gap = lane.find((row) => row.kind === "gap");
    expect(gap && gap.kind === "gap" ? gap.missingVersions : 0).toBe(1);
    expect(gap && gap.kind === "gap" ? gap.note : "").toContain(NO_RECORDED_FACTS);
  });

  it("marks a truncated transition scan as bounded rather than complete", () => {
    const evidence = projectEvidenceDetail(
      evidenceBody({
        lifecycle: {
          current_state: "ACTIVE",
          current_version: 9,
          transitions: [transition(20, 9, "ACTIVE")],
          transitions_truncated: true,
        },
      }),
    );
    const lane = composeAuthorityLane(evidence);
    expect(lane[0].kind).toBe("bounded");
    expect(lane[0].kind === "bounded" ? lane[0].note : "").toContain("were not returned");
  });

  it("says no recorded facts when the daemon returned no transition at all", () => {
    const evidence = projectEvidenceDetail(
      evidenceBody({
        lifecycle: { current_state: "ADMITTED", current_version: 1, transitions: [] },
      }),
    );
    const lane = composeAuthorityLane(evidence);
    expect(lane).toHaveLength(1);
    expect(lane[0].kind).toBe("empty");
    expect(lane[0].kind === "empty" ? lane[0].note : "").toContain(NO_RECORDED_FACTS);
  });

  it("reports an observed zero as a measurement and a bounded sample set as bounded", () => {
    const view = projectObservation({
      family: "o4",
      denominator: 0,
      observed_zero: true,
      samples_truncated: true,
      counters: {
        budget_stop: {
          denominator: 0,
          observed_zero: true,
          negative_control: "no_budget_stop_sample",
          count: 0,
        },
      },
    });
    const lane = composeObservationLane([view]);
    expect(lane.some((row) => row.kind === "bounded")).toBe(true);
    const counter = lane.find((row) => row.kind === "counter");
    expect(counter && counter.kind === "counter" ? counter.counter.observedZero : false).toBe(true);
    expect(
      counter && counter.kind === "counter" ? counter.counter.negativeControl : "",
    ).toBe("no_budget_stop_sample");
  });

  it("never claims an unattached watch is live and never implies detach is a control", async () => {
    const calls = installFetch(detailRoutes());
    const { host, root } = renderAppAt(detailHash(TASK_A));
    await flush();
    const body = text(host);
    expect(body).toContain("not attached");
    expect(body).toContain("never cancelled a Task or stopped an Agent");
    expect(body).not.toContain("live delivery arrives with W11");
    expect(body).not.toMatch(/Watch is live/);
    expect(calls.some((call) => call.path === "/task/watch")).toBe(false);
    unmount(host, root);
  });

  it("attach opens the task watch stream and puts deltas only on the observation lane", async () => {
    const calls = installFetch(detailRoutes());
    const { host, root } = renderAppAt(detailHash(TASK_A));
    await flush();
    clickNamed(host, "Attach watch");
    await flush();
    const watchCalls = calls.filter((call) => call.path === "/task/watch");
    expect(watchCalls.length).toBeGreaterThan(0);
    expect(watchCalls[0]?.authorization).toBe("Bearer task-token");
    expect(watchCalls[0]?.accept).toMatch(/event-stream/i);
    expect(watchCalls[0]?.query.get("resume_from")).toBeNull();

    const body = text(host);
    expect(body).toContain("Watch is live");
    expect(body).toContain("15 s bounded poll");
    const observation = host.querySelector(".cp-lane--observation") as HTMLElement;
    const authority = host.querySelector(".cp-lane--authority") as HTMLElement;
    expect(text(observation)).toContain("task.admitted");
    expect(text(observation)).toContain("obs");
    expect(text(authority)).not.toContain("task.admitted");
    expect(text(authority)).toContain("task.transition");
    expect(body).toContain("never cancelled a Task or stopped an Agent");
    unmount(host, root);
  });

  it("detach is observation-only and a stale resume is a gap, not completion", async () => {
    installFetch(
      detailRoutes({
        "GET /task/watch": { status: 409, body: { code: "TASK_WATCH_RESUME_STALE" } },
      }),
    );
    const { host, root } = renderAppAt(detailHash(TASK_A));
    await flush();
    clickNamed(host, "Attach watch");
    await flush();
    expect(text(host)).toContain("stale");
    expect(text(host)).toContain("TASK_WATCH_RESUME_STALE");
    expect(text(host)).toContain("Completion stays unknown");
    expect(dispositionLabel(host)).not.toBe("stale");
    clickNamed(host, "Detach watch");
    await flush();
    expect(text(host)).toContain("disconnected");
    expect(text(host)).toContain("never cancelled a Task or stopped an Agent");
    unmount(host, root);
  });

  it("composes watch deltas as observation rows that are never transitions", () => {
    const rows = composeWatchObservation(
      [{ cursor: "2", kind: "task.admitted", detail: "task.admitted · task://x", taskRef: "task://x" }],
      "task://x",
    );
    expect(rows).toEqual([
      {
        kind: "watch",
        sequence: "2",
        eventKind: "task.admitted",
        detail: "task.admitted · task://x",
        scopedToPageTask: true,
      },
    ]);
  });
});

/* ---------- effects ---------- */

describe("Effects section", () => {
  it("surfaces OUTCOME_UNKNOWN and VERIFY_FAILED before settled effects", () => {
    const view = projectEffectHistory(effectsBody());
    const sorted = sortEffectsByAttention(view.entries);
    expect(sorted[0].stage).toBe("OUTCOME_UNKNOWN");
    expect(effectNeedsAttention(sorted[0])).toBe(true);
    expect(effectNeedsAttention(sorted[1])).toBe(false);
  });

  it("shows stage, outcome, reconcile, mutation count and refs, and marks truncation", async () => {
    installFetch(detailRoutes());
    const { host, root } = renderAppAt(detailHash(TASK_A, "?section=effects"));
    await flush();

    const section = host.querySelector("#section-effects") as HTMLElement;
    const body = text(section);
    expect(body).toContain("OUTCOME_UNKNOWN");
    expect(body).toContain("indeterminate");
    expect(body).toContain("must_reconcile");
    expect(body).toContain("unknown (the daemon reports no mutation count for this stage)");
    // The digest chip truncates its value, so the label plus the tail is what
    // is actually on screen.
    expect(body).toContain("fixed post-state ref");
    expect(body).toContain("post-1");
    expect(body).toContain("reconciliation report ref");
    expect(body).toContain("Bounded.");
    expect(body).toContain("more effects exist than were returned");
    unmount(host, root);
  });

  it("treats an empty effect history as no recorded mutation, never as success", async () => {
    installFetch(
      detailRoutes({
        "GET /task/effects": {
          status: 200,
          body: { status: "ok", task_ref: TASK_A, effects: [], effects_truncated: false },
        },
      }),
    );
    const { host, root } = renderAppAt(detailHash(TASK_A, "?section=effects"));
    await flush();
    const section = text(host.querySelector("#section-effects") as HTMLElement);
    expect(section).toContain("No external mutation recorded");
    expect(section).toContain("not a successful outcome");
    unmount(host, root);
  });
});

/* ---------- evidence and completion ---------- */

describe("Evidence completion honesty", () => {
  it("says completed only when a current acceptance record exists", () => {
    const complete = completionReading(projectEvidenceDetail(evidenceBody()), true);
    expect(complete.disposition).toBe("completed");
    expect(complete.label).toBe("completed");
  });

  it("never treats a passing verification alone as task completion", () => {
    const reading = completionReading(
      projectEvidenceDetail(evidenceBody({ latest_acceptance: null })),
      true,
    );
    expect(reading.disposition).toBe("verified-not-accepted");
    expect(reading.label).not.toContain("completed");
    expect(reading.detail).toContain("passing verification is not Task completion");
  });

  it("labels a non-current verification and refuses to read it as acceptance", () => {
    const reading = completionReading(
      projectEvidenceDetail(
        evidenceBody({
          latest_verification: { status: "passed", current: false, report_digest: REPORT_DIGEST },
        }),
      ),
      true,
    );
    expect(reading.disposition).toBe("verification-not-current");
    expect(reading.label).toContain("not current");
    expect(reading.detail).toContain("does not prove the current state");
  });

  it("refuses completion when the acceptance itself is not current", () => {
    const reading = completionReading(
      projectEvidenceDetail(
        evidenceBody({
          latest_acceptance: { terminal_transition_ref: "event://x", current: false },
        }),
      ),
      true,
    );
    expect(reading.disposition).toBe("verified-not-accepted");
    expect(reading.label).toBe("acceptance not current");
  });

  it("reads a 404 as no terminal evidence recorded", async () => {
    installFetch(
      detailRoutes({
        "GET /task/evidence": {
          status: 404,
          body: { status: "error", code: "TASK_EVIDENCE_NOT_FOUND" },
        },
      }),
    );
    const { host, root } = renderAppAt(detailHash(TASK_A, "?section=evidence"));
    await flush();
    const body = text(host);
    expect(body).toContain("No terminal evidence recorded");
    expect(body).toContain("not a claim that it never ran");
    expect(dispositionLabel(host)).toBe("No terminal evidence recorded");
    unmount(host, root);
  });
});

/* ---------- intent chain ---------- */

describe("Intent and contract chain", () => {
  it("states the ephemeral-preview rule when this session did not admit the ref", async () => {
    installFetch(detailRoutes());
    const { host, root } = renderAppAt(detailHash(TASK_A, "?section=intent"));
    await flush();
    const section = text(host.querySelector("#section-intent") as HTMLElement);
    expect(section).toContain(PREVIEW_EPHEMERAL_STATEMENT);
    expect(section).toContain("No chain recorded in this session");
    expect(section).toContain("no daemon route returns a UserIntentRecord by task ref");
    unmount(host, root);
  });

  it("renders the session chain when this session admitted the ref", async () => {
    noteSessionChain(appProjections, {
      taskRef: TASK_A,
      admittedAtMs: 1000,
      intent: {
        userIntentRecordId: "uir-1",
        rawExpression: "search the workspace for needle",
        recordedAt: "2026-08-25T02:00:00Z",
      },
      interpretation: {
        interpretationId: "int-1",
        interpretationDigest: INTERPRETATION_DIGEST,
        status: "candidate",
        materialAmbiguityCount: 0,
        openAmbiguities: [],
        recordedDecisions: ["which workspace? → the personal one"],
        informationGaps: ["memory://personal/missing"],
        supersededInterpretationIds: ["int-0"],
      },
      preview: {
        previewDigest: PREVIEW_DIGEST,
        objective: "search the workspace for needle",
        conditionCount: 1,
        ephemeral: true,
      },
      admission: {
        taskRef: TASK_A,
        contractEpoch: 3,
        contractDigest: `sha256:${"d".repeat(64)}`,
        taskContractRef: "task-contract://tc-1",
        acceptedBy: "principal://local/owner",
      },
    });
    installFetch(detailRoutes());
    const { host, root } = renderAppAt(detailHash(TASK_A, "?section=intent"));
    await flush();
    const section = text(host.querySelector("#section-intent") as HTMLElement);
    expect(section).toContain("uir-1");
    expect(section).toContain("int-1");
    expect(section).toContain("int-0");
    expect(section).toContain("which workspace? → the personal one");
    expect(section).toContain("memory://personal/missing");
    // The preview is present but still labelled ephemeral.
    expect(section).toContain(PREVIEW_EPHEMERAL_STATEMENT);
    expect(section).toContain("principal://local/owner");
    unmount(host, root);
  });
});

/* ---------- context ---------- */

describe("Context section", () => {
  it("shows the real consumption pins", async () => {
    installFetch(detailRoutes());
    const { host, root } = renderAppAt(detailHash(TASK_A, "?section=context"));
    await flush();
    const section = text(host.querySelector("#section-context") as HTMLElement);
    expect(section).toContain("mem-1");
    expect(section).toContain("bind-1");
    expect(section).toContain("authorized_exact_pin");
    expect(section).toContain("ctx-1");
    unmount(host, root);
  });

  it("names each consumption refusal instead of collapsing them into unavailable", () => {
    expect(consumptionRefusal("RESOURCE_CONSUMPTION_NOT_FOUND")).toContain(
      "no durable Memory/Skill consumption record",
    );
    expect(consumptionRefusal("RESOURCE_TASK_CONTEXT_MISMATCH")).toContain("a real conflict");
    expect(consumptionRefusal("RESOURCE_CONSUMPTION_NOT_ELIGIBLE")).toContain("stale");
    expect(consumptionRefusal("RESOURCE_TASK_CONTEXT_MISSING")).toContain(
      "missing authority record",
    );
    expect(consumptionRefusal(undefined)).toContain("no error class");
  });

  it("reports a consumption conflict on screen rather than as an empty context", async () => {
    installFetch(
      detailRoutes({
        "GET /task/resource/v1/consumption": {
          status: 409,
          body: { status: "error", code: "RESOURCE_TASK_CONTEXT_MISMATCH" },
        },
      }),
    );
    const { host, root } = renderAppAt(detailHash(TASK_A, "?section=context"));
    await flush();
    const section = text(host.querySelector("#section-context") as HTMLElement);
    expect(section).toContain("a real conflict");
    expect(section).not.toContain("No Memory pin was consumed");
    unmount(host, root);
  });

  it("names Loop, WIA and context assembly as unavailable rather than guessing", async () => {
    installFetch(detailRoutes());
    const { host, root } = renderAppAt(detailHash(TASK_A));
    await flush();
    const body = text(host);
    expect(body).toContain("Loop / DECIDE trace");
    expect(body).toContain("WIA (work-in-attention) set");
    expect(body).toContain("Context assembly detail");
    expect(body).toContain("unavailable");
    unmount(host, root);
  });
});

/* ---------- structure, routing and controls ---------- */

describe("Work detail structure and routing", () => {
  it("renders all six sections continuously, with no tab or accordion", async () => {
    installFetch(detailRoutes());
    const { host, root } = renderAppAt(detailHash(TASK_A));
    await flush();

    for (const id of ["overview", "run", "effects", "evidence", "intent", "context"]) {
      expect(host.querySelector(`#section-${id}`)).not.toBeNull();
    }
    expect(host.querySelector('[role="tab"]')).toBeNull();
    expect(host.querySelector('[role="tablist"]')).toBeNull();
    expect(host.querySelector("details")).toBeNull();
    unmount(host, root);
  });

  it("honours a section deep link and keeps every section rendered", async () => {
    installFetch(detailRoutes());
    const { host, root } = renderAppAt(detailHash(TASK_A, "?section=evidence"));
    await flush();
    const current = host.querySelector('.cp-sectionnav-link[aria-current="true"]');
    expect(current?.textContent).toBe("Evidence");
    expect(host.querySelector("#section-overview")).not.toBeNull();
    unmount(host, root);
  });

  it("returns to Work carrying the inventory selection and filter", async () => {
    installFetch(detailRoutes());
    const { host, root } = renderAppAt(
      detailHash(TASK_A, `?task=${encodeURIComponent(TASK_A)}&scope=all&q=0193c1`),
    );
    await flush();
    const back = [...host.querySelectorAll("a")].find(
      (anchor) => (anchor.textContent ?? "").trim() === "Back to Work",
    );
    const href = back?.getAttribute("href") ?? "";
    expect(href).toContain(`task=${encodeURIComponent(TASK_A)}`);
    expect(href).toContain("scope=all");
    expect(href).toContain("q=0193c1");
    unmount(host, root);
  });

  it("shows a designed object-404 for an unknown ref and fabricates no detail", async () => {
    installFetch(detailRoutes());
    const { host, root } = renderAppAt(detailHash(TASK_MISSING));
    await flush();
    const body = text(host);
    expect(body).toContain("No such task on this daemon");
    expect(body).toContain("not a claim that the task never existed");
    // No fabricated sections behind the 404, so there is no disposition at all.
    expect(host.querySelector("#section-run")).toBeNull();
    expect(host.querySelector("#section-evidence")).toBeNull();
    expect(host.querySelector("#section-overview")).toBeNull();
    expect(host.querySelector(".cp-lanes")).toBeNull();
    expect(dispositionLabel(host)).toBe("");
    unmount(host, root);
  });

  it("accounts for a ref this session observed even when the envelope list omits it", async () => {
    noteObservedTask(appProjections, {
      taskRef: TASK_MISSING,
      objective: "session-only ref",
      observedAtMs: 2000,
      origin: "task/admit",
    });
    installFetch(
      detailRoutes({
        "GET /task/evidence": {
          status: 404,
          body: { status: "error", code: "TASK_EVIDENCE_NOT_FOUND" },
        },
      }),
    );
    const { host, root } = renderAppAt(detailHash(TASK_MISSING));
    await flush();
    const body = text(host);
    expect(body).not.toContain("No such task on this daemon");
    expect(body).toContain("No terminal evidence recorded");
    expect(body).toContain("session-only ref");
    unmount(host, root);
  });

  it("offers no cancel, pause or retry control anywhere", async () => {
    installFetch(detailRoutes());
    const { host, root } = renderAppAt(detailHash(TASK_A));
    await flush();
    const controls = [...host.querySelectorAll("main button, main a")].map((node) =>
      (node.textContent ?? "").trim().toLowerCase(),
    );
    for (const label of ["cancel", "pause", "retry", "stop", "kill"]) {
      expect(controls).not.toContain(label);
    }
    expect(text(host)).toContain("Not available over HTTP");
    expect(text(host)).toContain("the daemon exposes no task cancel route");
    unmount(host, root);
  });

  it("calls only the six real daemon reads and no invented task route", async () => {
    const calls = installFetch(detailRoutes());
    const { host, root } = renderAppAt(detailHash(TASK_A));
    await flush();

    const paths = [...new Set(calls.map((call) => `${call.method} ${call.path}`))];
    expect(paths).toContain("GET /management/resource/v1/list");
    expect(paths).toContain("GET /task/evidence");
    expect(paths).toContain("GET /task/effects");
    expect(paths).toContain("GET /task/observation");
    expect(paths).toContain("GET /task/resource/v1/consumption");
    for (const invented of [
      "/task/detail",
      "/task/run",
      "/task/timeline",
      "/task/cancel",
      "/task/pause",
      "/task/retry",
      "/task/loop",
      "/task/wia",
      "/task/context",
    ]) {
      expect(calls.some((call) => call.path === invented)).toBe(false);
    }
    // Only the two registered observation families are probed.
    const families = calls
      .filter((call) => call.path === "/task/observation")
      .map((call) => call.query.get("family"));
    expect([...new Set(families)].sort()).toEqual(["o4", "o5"]);
    unmount(host, root);
  });

  it("names a denied read without inventing content", async () => {
    installFetch(
      detailRoutes({
        "GET /task/evidence": {
          status: 401,
          body: { status: "error", error: { code: "UNAUTHORIZED", message: "denied" } },
        },
        "GET /task/effects": {
          status: 401,
          body: { status: "error", error: { code: "UNAUTHORIZED", message: "denied" } },
        },
      }),
    );
    const { host, root } = renderAppAt(detailHash(TASK_A));
    await flush();
    const body = text(host);
    expect(body).toContain("UNAUTHORIZED");
    expect(body).toContain("neither verified nor failed");
    expect(dispositionLabel(host)).toBe("No terminal evidence recorded");
    unmount(host, root);
  });

  it("treats the daemon 200-stub on observation as a missing measurement", async () => {
    installFetch(
      detailRoutes({
        "GET /task/observation": {
          status: 200,
          body: { status: "ok", note: "no Task API operation matched" },
        },
      }),
    );
    const { host, root } = renderAppAt(detailHash(TASK_A));
    await flush();
    const observation = text(host.querySelector(".cp-lane--observation") as HTMLElement);
    expect(observation).toContain("could not be read");
    expect(observation).toContain("not an observed zero");
    unmount(host, root);
  });

  it("keeps landmarks, section labelling and focusable controls", async () => {
    installFetch(detailRoutes());
    const { host, root } = renderAppAt(detailHash(TASK_A));
    await flush();

    expect(host.querySelector("main")).not.toBeNull();
    expect(
      host.querySelector('nav[aria-label="Task detail sections"]'),
    ).not.toBeNull();
    for (const id of ["overview", "run", "effects", "evidence", "intent", "context"]) {
      const section = host.querySelector(`#section-${id}`);
      expect(section?.getAttribute("aria-labelledby")).toBeTruthy();
    }
    const focusable = [...host.querySelectorAll("main button, main a")];
    expect(focusable.length).toBeGreaterThan(6);
    for (const element of focusable) {
      expect(element.getAttribute("tabindex")).not.toBe("-1");
    }
    unmount(host, root);
  });

  it("bracket keys walk the section navigator to Evidence", async () => {
    installFetch(detailRoutes());
    const { host, root } = renderAppAt(detailHash(TASK_A));
    await flush();
    expect(
      (host.querySelector('.cp-sectionnav-link[aria-current="true"]')?.textContent ?? "").trim(),
    ).toBe("Overview");
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "]", bubbles: true }));
    });
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "]", bubbles: true }));
    });
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "]", bubbles: true }));
    });
    expect(
      (host.querySelector('.cp-sectionnav-link[aria-current="true"]')?.textContent ?? "").trim(),
    ).toBe("Evidence");
    unmount(host, root);
  });
});
