import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { appProjections } from "../../data/store";
import {
  admitFailure,
  buildWorkRows,
  candidateFacts,
  canPreview,
  filterWorkRows,
  inventoryFooter,
  preserveSelection,
  projectAdmission,
  projectInterpretation,
  projectPreview,
  sessionChainFor,
  sortWorkRows,
  workRowReading,
  WORK_CHAIN_KEY,
  type ObservedTask,
  type TaskEnvelopeView,
} from "../../data/projections/work";
import { clearSession, rememberBearer } from "../../session";

/* ---------- harness (same shape as the W2/W3 suites) ---------- */

type RouteResponse = { status: number; body: unknown };
type RouteHandler = RouteResponse | ((call: { body?: any; url: URL }) => RouteResponse);

interface RecordedCall {
  method: string;
  path: string;
  query: URLSearchParams;
  body?: any;
}

const TASK_A = "task://personal/aaaa-1111";
const TASK_B = "task://personal/bbbb-2222";
const TASK_C = "task://personal/cccc-3333";

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
    let body: unknown;
    if (typeof init?.body === "string") {
      try {
        body = JSON.parse(init.body);
      } catch {
        body = init.body;
      }
    }
    calls.push({ method, path: url.pathname, query: url.searchParams, body });
    const handler = routes[`${method} ${url.pathname}`];
    const resolved =
      typeof handler === "function"
        ? handler({ body, url })
        : (handler ?? defaultRoute(url.pathname));
    return new Response(JSON.stringify(resolved.body), {
      status: resolved.status,
      headers: { "content-type": "application/json" },
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

async function flush(ticks = 12) {
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

function setInputValue(input: HTMLInputElement | HTMLTextAreaElement, value: string) {
  const proto =
    input.tagName === "TEXTAREA"
      ? window.HTMLTextAreaElement.prototype
      : window.HTMLInputElement.prototype;
  const setter = Object.getOwnPropertyDescriptor(proto, "value")?.set;
  setter?.call(input, value);
  input.dispatchEvent(new Event("input", { bubbles: true }));
}

function findButton(host: HTMLElement, text: string): HTMLButtonElement {
  const button = [...host.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === text,
  );
  if (!button) {
    throw new Error(`button not found: ${text}`);
  }
  return button;
}

function clickButton(host: HTMLElement, text: string) {
  const button = findButton(host, text);
  act(() => {
    button.click();
  });
}

/** The inventory scope control is a radio group, not a button. */
function selectScope(host: HTMLElement, value: "session" | "all") {
  const radio = host.querySelector(
    `input[name="work_origin"][value="${value}"]`,
  ) as HTMLInputElement;
  if (!radio) {
    throw new Error(`scope radio not found: ${value}`);
  }
  act(() => {
    radio.click();
  });
}

function submitForm(form: HTMLFormElement) {
  act(() => {
    form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
  });
}

function text(host: HTMLElement): string {
  return (host.textContent ?? "").replace(/\s+/g, " ");
}

const INTERPRETATION_DIGEST = `sha256:${"b".repeat(64)}`;
const PREVIEW_DIGEST = `sha256:${"c".repeat(64)}`;

function creationRoutes(overrides: Record<string, RouteHandler> = {}): Record<string, RouteHandler> {
  return {
    "GET /management/resource/v1/list": { status: 200, body: { status: "ok", resources: [] } },
    "POST /task/intent.record": {
      status: 200,
      body: { status: "ok", user_intent_record_id: "rec-1", recorded_at: "2026-08-25T00:00:00Z" },
    },
    "POST /task/intent.interpret": ({ body }) => ({
      status: 200,
      body: {
        status:
          (body?.candidate?.ambiguities ?? []).some((a: any) => a.material)
            ? "clarification_required"
            : "candidate",
        interpretation_id: "int-1",
        interpretation_digest: INTERPRETATION_DIGEST,
        material_ambiguity_count: (body?.candidate?.ambiguities ?? []).filter(
          (a: any) => a.material,
        ).length,
      },
    }),
    "POST /task/preview": {
      status: 200,
      body: {
        status: "ok",
        preview_digest: PREVIEW_DIGEST,
        task_ref: TASK_C,
        objective: "search the workspace for needle",
        condition_count: 1,
        budget: { semantic_calls: 4, tool_calls: 4 },
      },
    },
    "POST /task/admit": {
      status: 200,
      body: {
        status: "ok",
        task_ref: TASK_C,
        contract_epoch: 1,
        contract_digest: `sha256:${"d".repeat(64)}`,
        task_contract_ref: "task-contract://tc-1",
      },
    },
    ...overrides,
  };
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

/* ---------- inventory projection logic ---------- */

describe("Work inventory projection", () => {
  const envelopes: TaskEnvelopeView[] = [
    { taskRef: TASK_B, contractEpoch: 1, health: "contracted" },
    { taskRef: TASK_A, contractEpoch: 2, health: "contracted" },
  ];
  const observed: ObservedTask[] = [
    { taskRef: TASK_A, objective: "find needle", observedAtMs: 2000, origin: "task/admit" },
    { taskRef: TASK_C, objective: "second", observedAtMs: 3000, origin: "task/admit" },
  ];

  it("merges envelope and session refs and de-duplicates by task_ref", () => {
    const rows = buildWorkRows({
      envelopes,
      observed,
      evidence: new Map(),
      probed: new Set(),
    });
    expect(rows).toHaveLength(3);
    expect(rows.filter((row) => row.taskRef === TASK_A)).toHaveLength(1);
    const merged = rows.find((row) => row.taskRef === TASK_A);
    // A ref present in both keeps the envelope's authority facts AND the
    // session's objective — neither source silently wins.
    expect(merged?.origin).toBe("envelope+session");
    expect(merged?.contractEpoch).toBe(2);
    expect(merged?.objective).toBe("find needle");
  });

  it("sorts stably: session-observed newest first, then envelope refs by ref", () => {
    const rows = sortWorkRows(
      buildWorkRows({ envelopes, observed, evidence: new Map(), probed: new Set() }),
    );
    expect(rows.map((row) => row.taskRef)).toEqual([TASK_C, TASK_A, TASK_B]);
    // Re-running the same projection must not reshuffle anything.
    const again = sortWorkRows(
      buildWorkRows({ envelopes, observed, evidence: new Map(), probed: new Set() }),
    );
    expect(again.map((row) => row.taskRef)).toEqual(rows.map((row) => row.taskRef));
  });

  it("keeps a selection that still exists and drops one that does not", () => {
    const rows = sortWorkRows(
      buildWorkRows({ envelopes, observed, evidence: new Map(), probed: new Set() }),
    );
    expect(preserveSelection(rows, TASK_B)).toBe(TASK_B);
    // Never silently re-point the inspector at a neighbouring object.
    expect(preserveSelection(rows, "task://personal/gone")).toBeUndefined();
    expect(preserveSelection(rows, undefined)).toBeUndefined();
  });

  it("defaults the scope to refs this session observed", () => {
    const rows = buildWorkRows({ envelopes, observed, evidence: new Map(), probed: new Set() });
    const session = filterWorkRows(rows, "session");
    expect(session.map((row) => row.taskRef).sort()).toEqual([TASK_A, TASK_C].sort());
    expect(filterWorkRows(rows, "all")).toHaveLength(3);
  });

  it("never invents a lifecycle state for an envelope row", () => {
    const rows = buildWorkRows({ envelopes, observed, evidence: new Map(), probed: new Set() });
    for (const row of rows) {
      const reading = workRowReading(row);
      expect(reading.label).toBe("state not exposed");
      expect(reading.category).toBe("unknown");
    }
  });

  it("shows a lifecycle state only when a real evidence read returned one", () => {
    const evidence = new Map([
      [TASK_A, { hasVerification: true, lifecycleState: "COMPLETED", acceptancePresent: true }],
    ]);
    const rows = buildWorkRows({
      envelopes,
      observed,
      evidence: evidence as never,
      probed: new Set([TASK_A, TASK_B]),
    });
    const a = rows.find((row) => row.taskRef === TASK_A);
    const b = rows.find((row) => row.taskRef === TASK_B);
    expect(workRowReading(a!)).toMatchObject({ label: "COMPLETED", category: "completed" });
    // Probed but with no lifecycle word is still "not exposed", not "unknown state".
    expect(workRowReading(b!).label).toBe("state not exposed");
  });

  it("states the count as what is loaded, with the BD-3 qualifier", () => {
    expect(inventoryFooter(3)).toBe(
      "Showing 3 known tasks · inventory is envelope-only (BD-3)",
    );
    expect(inventoryFooter(1)).toBe("Showing 1 known task · inventory is envelope-only (BD-3)");
  });
});

/* ---------- creation chain logic ---------- */

describe("Governed creation chain projections", () => {
  it("blocks preview while the daemon says clarification_required", () => {
    expect(
      canPreview(
        projectInterpretation({
          status: "clarification_required",
          interpretation_id: "int-1",
          interpretation_digest: INTERPRETATION_DIGEST,
          material_ambiguity_count: 1,
        }),
      ),
    ).toBe(false);
    expect(
      canPreview(
        projectInterpretation({
          status: "candidate",
          interpretation_id: "int-1",
          interpretation_digest: INTERPRETATION_DIGEST,
          material_ambiguity_count: 0,
        }),
      ),
    ).toBe(true);
    expect(canPreview(undefined)).toBe(false);
  });

  it("moves an answered ambiguity out of the candidate and into a recorded decision", () => {
    const facts = candidateFacts({
      objective: "do the thing",
      constraints: ["only read"],
      forbidden: ["write"],
      assumptions: [],
      ambiguities: [
        { id: "amb-1", question: "which repo?", material: true, answer: "this one" },
        { id: "amb-2", question: "which branch?", material: true, answer: "" },
      ],
      informationGaps: ["memory://personal/unknown"],
    });
    expect(facts.ambiguities).toEqual([
      { id: "amb-2", material: true, question: "which branch?" },
    ]);
    expect(facts.assumptions).toContain("which repo? → this one");
    expect(facts.information_gaps).toEqual(["memory://personal/unknown"]);
    expect(facts.objectives).toEqual(["do the thing"]);
  });

  it("names a 403 as a principal mismatch and does not ask for a new preview", () => {
    const failure = admitFailure(403, { code: "TASK_ACCEPTANCE_PRINCIPAL_MISMATCH" });
    expect(failure.code).toBe("TASK_ACCEPTANCE_PRINCIPAL_MISMATCH");
    expect(failure.requiresFreshPreview).toBe(false);
    expect(failure.message).toMatch(/authenticated principal/);
  });

  it("requires a fresh preview after a 409 and never retries automatically", () => {
    const failure = admitFailure(409, { code: "TASK_ADMISSION_REJECTED" });
    expect(failure.requiresFreshPreview).toBe(true);
    expect(failure.message).toMatch(/nothing is retried automatically/);
  });

  it("reads the daemon's own preview and admission facts without defaulting", () => {
    expect(projectPreview({})).toMatchObject({ previewDigest: "", conditionCount: undefined });
    expect(projectAdmission({ task_ref: TASK_C })).toMatchObject({
      taskRef: TASK_C,
      contractEpoch: undefined,
      contractDigest: undefined,
    });
  });
});

/* ---------- rendered inventory ---------- */

describe("Work space", () => {
  it("lists merged refs, keeps the BD-3 footer, and calls no invented task API", async () => {
    const calls = installFetch({
      "GET /management/resource/v1/list": {
        status: 200,
        body: { status: "ok", resources: [envelope(TASK_A, 2), envelope(TASK_B, 1)] },
      },
      "GET /task/evidence": { status: 200, body: { status: "ok", task_ref: TASK_A } },
      "GET /task/effects": { status: 200, body: { status: "ok", effects: [] } },
    });
    const { host, root } = renderAppAt("#/work");
    await flush();

    selectScope(host, "all");
    await flush(4);

    const body = text(host);
    expect(body).toContain("Showing 2 known tasks · inventory is envelope-only (BD-3)");
    expect(body).toContain("state not exposed");
    expect(body).toContain("not exposed by the daemon's task list");

    const paths = calls.map((call) => `${call.method} ${call.path}`);
    // The only task-shaped reads that exist on this daemon.
    expect(paths).toContain("GET /management/resource/v1/list");
    expect(calls.some((call) => call.path === "/task/list")).toBe(false);
    expect(calls.some((call) => call.path === "/task/detail")).toBe(false);
    expect(calls.some((call) => call.path === "/task/cancel")).toBe(false);
    expect(calls.some((call) => call.path.includes("/task/control"))).toBe(false);
    unmount(host, root);
  });

  it("asks for family=task and probes evidence and effects per known ref only", async () => {
    const calls = installFetch({
      "GET /management/resource/v1/list": {
        status: 200,
        body: { status: "ok", resources: [envelope(TASK_A, 2)] },
      },
      "GET /task/evidence": { status: 200, body: { status: "ok", task_ref: TASK_A } },
      "GET /task/effects": { status: 200, body: { status: "ok", effects: [] } },
    });
    const { host, root } = renderAppAt("#/work");
    await flush();

    const listCall = calls.find((call) => call.path === "/management/resource/v1/list");
    expect(listCall?.query.get("family")).toBe("task");
    const evidenceCall = calls.find((call) => call.path === "/task/evidence");
    expect(evidenceCall?.query.get("task_ref")).toBe(TASK_A);
    const effectsCall = calls.find((call) => call.path === "/task/effects");
    expect(effectsCall?.query.get("task_ref")).toBe(TASK_A);
    unmount(host, root);
  });

  it("opens an inspector on select and preserves the selection across a refresh", async () => {
    installFetch({
      "GET /management/resource/v1/list": {
        status: 200,
        body: { status: "ok", resources: [envelope(TASK_A, 2), envelope(TASK_B, 1)] },
      },
      "GET /task/evidence": {
        status: 200,
        body: {
          status: "ok",
          task_ref: TASK_A,
          lifecycle: { current_state: "ACTIVE" },
          latest_verification: null,
        },
      },
      "GET /task/effects": { status: 200, body: { status: "ok", effects: [] } },
    });
    const { host, root } = renderAppAt("#/work");
    await flush();
    selectScope(host, "all");
    await flush(4);

    const inspectButtons = [...host.querySelectorAll("button")].filter(
      (button) => button.textContent === "Inspect",
    );
    act(() => {
      inspectButtons[0].click();
    });
    await flush(4);

    expect(host.querySelector("aside.cp-inspector")).not.toBeNull();
    const selectedBefore = host.querySelector('tr[aria-selected="true"]')?.getAttribute("data-row-key");
    expect(selectedBefore).toBe(TASK_A);

    clickButton(host, "Refresh");
    await flush(6);
    expect(
      host.querySelector('tr[aria-selected="true"]')?.getAttribute("data-row-key"),
    ).toBe(TASK_A);
    unmount(host, root);
  });

  it("states unsupported cancel/pause/retry as facts, never as clickable controls", async () => {
    installFetch({
      "GET /management/resource/v1/list": {
        status: 200,
        body: { status: "ok", resources: [envelope(TASK_A, 2)] },
      },
      "GET /task/evidence": { status: 200, body: { status: "ok", task_ref: TASK_A } },
      "GET /task/effects": { status: 200, body: { status: "ok", effects: [] } },
    });
    const { host, root } = renderAppAt("#/work");
    await flush();
    selectScope(host, "all");
    await flush(4);
    act(() => {
      findButton(host, "Inspect").click();
    });
    await flush(4);

    const inspector = host.querySelector("aside.cp-inspector") as HTMLElement;
    expect(text(inspector)).toContain("not available over HTTP");
    for (const label of ["cancel", "pause", "retry"]) {
      const control = [...inspector.querySelectorAll("button")].find(
        (button) => (button.textContent ?? "").trim().toLowerCase() === label,
      );
      expect(control).toBeUndefined();
    }
    // The detail link is a real route and carries the ref, so returning lands
    // on the same row rather than a reset list.
    const detailHref = [...inspector.querySelectorAll("a")]
      .map((a) => a.getAttribute("href") ?? "")
      .find((href) => href.includes(`/work/${encodeURIComponent(TASK_A)}`));
    expect(detailHref).toBeDefined();
    expect(detailHref).toContain(`task=${encodeURIComponent(TASK_A)}`);
    unmount(host, root);
  });

  it("names a denied task list and still lists session refs without inferring absence", async () => {
    installFetch({
      "GET /management/resource/v1/list": {
        status: 401,
        body: { status: "error", error: { code: "UNAUTHORIZED", message: "denied" } },
      },
    });
    const { host, root } = renderAppAt("#/work");
    await flush();
    const body = text(host);
    expect(body).toContain("could not be read");
    expect(body).toContain("UNAUTHORIZED");
    expect(body).toContain("no task is assumed to exist or not exist");
    unmount(host, root);
  });

  it("treats the daemon's 200 stub as not-run rather than an empty inventory", async () => {
    installFetch({
      "GET /management/resource/v1/list": {
        status: 200,
        body: { status: "ok", note: "business routes deferred to the daemon front door" },
      },
    });
    const { host, root } = renderAppAt("#/work");
    await flush();
    expect(text(host)).toContain("could not be read");
    unmount(host, root);
  });

  it("distinguishes a pending read from an authoritative empty inventory", async () => {
    let release: (() => void) | undefined;
    const held = new Promise<void>((resolve) => {
      release = resolve;
    });
    // The list read is held open, so the projection stays `loading` with no
    // last-good data — the real first-load case.
    vi.stubGlobal(
      "fetch",
      vi.fn(async (input: unknown) => {
        const url = new URL(String(input), "http://localhost");
        if (url.pathname === "/management/resource/v1/list") {
          await held;
        }
        return new Response(JSON.stringify({ status: "ok", resources: [] }), {
          status: 200,
          headers: { "content-type": "application/json" },
        });
      }),
    );

    const { host, root } = renderAppAt("#/work");
    await flush(4);
    selectScope(host, "all");
    await flush(2);

    const pending = text(host);
    expect(pending).toContain("Reading the daemon task list");
    expect(pending).not.toContain("No task refs in this scope");
    expect(pending).toContain("read in flight, not a statement that no task exists");

    release?.();
    await flush(6);

    const answered = text(host);
    expect(answered).toContain("No task refs in this scope");
    expect(answered).not.toContain("Reading the daemon task list");
    unmount(host, root);
  });

  it("never reports a failed task-list read as an empty inventory", async () => {
    installFetch({
      "GET /management/resource/v1/list": {
        status: 401,
        body: { status: "error", error: { code: "UNAUTHORIZED", message: "denied" } },
      },
    });
    const { host, root } = renderAppAt("#/work");
    await flush();
    selectScope(host, "all");
    await flush(4);

    const body = text(host);
    expect(body).toContain("The task list could not be read");
    expect(body).toContain("Nothing is claimed about whether tasks exist");
    expect(body).not.toContain("This page knows of no task in the selected scope");
    unmount(host, root);
  });

  it("never reports the daemon 200-stub as an empty inventory", async () => {
    installFetch({
      "GET /management/resource/v1/list": {
        status: 200,
        body: { status: "ok", note: "business routes deferred to the daemon front door" },
      },
    });
    const { host, root } = renderAppAt("#/work");
    await flush();
    selectScope(host, "all");
    await flush(4);

    const body = text(host);
    expect(body).toContain("The task list could not be read");
    expect(body).not.toContain("This page knows of no task in the selected scope");
    unmount(host, root);
  });

  it("keeps landmarks, a labelled table and focusable rows", async () => {
    installFetch({
      "GET /management/resource/v1/list": {
        status: 200,
        body: { status: "ok", resources: [envelope(TASK_A, 2)] },
      },
      "GET /task/evidence": { status: 200, body: { status: "ok", task_ref: TASK_A } },
      "GET /task/effects": { status: 200, body: { status: "ok", effects: [] } },
    });
    const { host, root } = renderAppAt("#/work");
    await flush();
    selectScope(host, "all");
    await flush(4);

    expect(host.querySelector("main")).not.toBeNull();
    const table = host.querySelector("table.cp-table");
    expect(table?.querySelector("caption")?.textContent).toBe("Known tasks");
    expect(host.querySelector('[role="group"][aria-label="Inventory scope"]')).not.toBeNull();
    const focusable = [...host.querySelectorAll("main button, main a, main input")];
    expect(focusable.length).toBeGreaterThan(0);
    for (const element of focusable) {
      expect(element.getAttribute("tabindex")).not.toBe("-1");
    }
    unmount(host, root);
  });
});

/* ---------- rendered creation flow ---------- */

describe("Governed task creation", () => {
  it("runs record → interpret → preview → admit in exactly that order", async () => {
    const calls = installFetch(creationRoutes());
    const { host, root } = renderAppAt("#/work/new");
    await flush();

    submitForm(host.querySelector("form") as HTMLFormElement);
    await flush(6);
    clickButton(host, "Preview the contract");
    await flush(6);
    clickButton(host, "Confirm and admit");
    await flush(6);

    const chain = calls
      .filter((call) => call.method === "POST" && call.path.startsWith("/task/"))
      .map((call) => call.path);
    expect(chain).toEqual([
      "/task/intent.record",
      "/task/intent.interpret",
      "/task/preview",
      "/task/admit",
    ]);
    unmount(host, root);
  });

  it("records the raw expression before anything interprets it", async () => {
    const calls = installFetch(creationRoutes());
    const { host, root } = renderAppAt("#/work/new");
    await flush();
    submitForm(host.querySelector("form") as HTMLFormElement);
    await flush(6);

    const record = calls.find((call) => call.path === "/task/intent.record");
    expect(record?.body.schema_version).toBe("cognitiveos.task-intent-record-request/0.1");
    expect(record?.body.raw_expression).toBe("search the workspace for needle");
    const interpret = calls.find((call) => call.path === "/task/intent.interpret");
    expect(interpret?.body.user_intent_record_id).toBe("rec-1");
    unmount(host, root);
  });

  it("surfaces ambiguities and information gaps as first-class review content", async () => {
    installFetch(creationRoutes());
    const { host, root } = renderAppAt("#/work/new");
    await flush();

    clickButton(host, "Add an ambiguity");
    await flush(2);
    setInputValue(
      host.querySelector('input[name="ambiguity_question_0"]') as HTMLInputElement,
      "which workspace?",
    );
    setInputValue(
      host.querySelector('textarea[name="gaps"]') as HTMLTextAreaElement,
      "memory://personal/missing",
    );
    await flush(2);
    submitForm(host.querySelector("form") as HTMLFormElement);
    await flush(6);

    const body = text(host);
    expect(body).toContain("which workspace?");
    expect(body).toContain("memory://personal/missing");
    expect(body).toContain("material");
    unmount(host, root);
  });

  it("treats clarification_required as a normal branch and blocks preview and admit", async () => {
    const calls = installFetch(creationRoutes());
    const { host, root } = renderAppAt("#/work/new");
    await flush();

    clickButton(host, "Add an ambiguity");
    await flush(2);
    setInputValue(
      host.querySelector('input[name="ambiguity_question_0"]') as HTMLInputElement,
      "which workspace?",
    );
    await flush(2);
    submitForm(host.querySelector("form") as HTMLFormElement);
    await flush(6);

    const body = text(host);
    expect(body).toContain("clarification_required");
    expect(body).toContain("This is the expected path, not a failure");
    // The gate is real: preview is unavailable and no preview/admit was issued.
    expect(findButton(host, "Preview the contract").disabled).toBe(true);
    expect(calls.some((call) => call.path === "/task/preview")).toBe(false);
    expect(calls.some((call) => call.path === "/task/admit")).toBe(false);
    unmount(host, root);
  });

  it("lets an answered material ambiguity supersede the candidate and unblock preview", async () => {
    const calls = installFetch(creationRoutes());
    const { host, root } = renderAppAt("#/work/new");
    await flush();

    clickButton(host, "Add an ambiguity");
    await flush(2);
    setInputValue(
      host.querySelector('input[name="ambiguity_question_0"]') as HTMLInputElement,
      "which workspace?",
    );
    await flush(2);
    submitForm(host.querySelector("form") as HTMLFormElement);
    await flush(6);

    const answer = [...host.querySelectorAll("input")].find((input) =>
      input.getAttribute("name")?.startsWith("answer_"),
    ) as HTMLInputElement;
    setInputValue(answer, "the personal workspace");
    await flush(2);
    clickButton(host, "Re-interpret with these decisions");
    await flush(6);

    const second = calls.filter((call) => call.path === "/task/intent.interpret")[1];
    expect(second?.body.candidate.ambiguities).toEqual([]);
    expect(second?.body.candidate.assumptions).toContain(
      "which workspace? → the personal workspace",
    );
    // A correction supersedes rather than rewriting the prior candidate.
    expect(second?.body.candidate.supersedes_interpretation_id).toBe("int-1");
    expect(findButton(host, "Preview the contract").disabled).toBe(false);
    unmount(host, root);
  });

  it("shows the contract facts the operator is accepting", async () => {
    installFetch(creationRoutes());
    const { host, root } = renderAppAt("#/work/new");
    await flush();
    submitForm(host.querySelector("form") as HTMLFormElement);
    await flush(6);
    clickButton(host, "Preview the contract");
    await flush(6);

    const body = text(host);
    expect(body).toContain("search the workspace for needle");
    expect(body).toContain("native.workspace.search");
    expect(body).toContain("workspace search");
    expect(body).toContain("independent fixed-effect verification");
    expect(body).toContain("2027-12-31T00:00:00Z");
    expect(body).toContain("semantic_calls=4");
    expect(body).toContain("int-1");
    expect(body).toContain(TASK_C);
    unmount(host, root);
  });

  it("admits with the same preview digest, interpretation and current principal", async () => {
    const calls = installFetch(creationRoutes());
    const { host, root } = renderAppAt("#/work/new");
    await flush();
    submitForm(host.querySelector("form") as HTMLFormElement);
    await flush(6);
    clickButton(host, "Preview the contract");
    await flush(6);
    clickButton(host, "Confirm and admit");
    await flush(6);

    const admit = calls.find((call) => call.path === "/task/admit");
    expect(admit?.body.preview_digest).toBe(PREVIEW_DIGEST);
    expect(admit?.body.acceptance.interpretation_id).toBe("int-1");
    expect(admit?.body.acceptance.accepted_digest).toBe(INTERPRETATION_DIGEST);
    expect(admit?.body.acceptance.accepted_by).toBe("principal://local/owner");
    const previewCall = calls.find((call) => call.path === "/task/preview");
    // The admitted draft is byte-identical to the previewed one.
    expect(admit?.body.task_contract_draft).toEqual(previewCall?.body.task_contract_draft);
    unmount(host, root);
  });

  it("explains a 403 as a principal or session cause", async () => {
    installFetch(
      creationRoutes({
        "POST /task/admit": {
          status: 403,
          body: { status: "error", code: "TASK_ACCEPTANCE_PRINCIPAL_MISMATCH" },
        },
      }),
    );
    const { host, root } = renderAppAt("#/work/new");
    await flush();
    submitForm(host.querySelector("form") as HTMLFormElement);
    await flush(6);
    clickButton(host, "Preview the contract");
    await flush(6);
    clickButton(host, "Confirm and admit");
    await flush(6);

    const alert = host.querySelector('p[role="alert"]');
    expect(alert?.textContent).toContain("TASK_ACCEPTANCE_PRINCIPAL_MISMATCH");
    expect(alert?.textContent).toContain("authenticated principal");
    expect(text(host)).not.toContain("Admitted");
    unmount(host, root);
  });

  it("never auto-retries a 409: it regenerates a preview and asks for reconfirmation", async () => {
    let admitAttempts = 0;
    const calls = installFetch(
      creationRoutes({
        "POST /task/admit": () => {
          admitAttempts += 1;
          return {
            status: 409,
            body: { status: "error", code: "TASK_ADMISSION_REJECTED" },
          };
        },
        "POST /task/preview": () => ({
          status: 200,
          body: {
            status: "ok",
            preview_digest: `sha256:${String(calls.filter((c) => c.path === "/task/preview").length).repeat(1)}${"e".repeat(63)}`,
            task_ref: TASK_C,
            objective: "search the workspace for needle",
            condition_count: 1,
          },
        }),
      }),
    );
    const { host, root } = renderAppAt("#/work/new");
    await flush();
    submitForm(host.querySelector("form") as HTMLFormElement);
    await flush(6);
    clickButton(host, "Preview the contract");
    await flush(6);
    clickButton(host, "Confirm and admit");
    await flush(8);

    // Exactly one admit attempt — the failure is reported, not retried.
    expect(admitAttempts).toBe(1);
    const previews = calls.filter((call) => call.path === "/task/preview");
    expect(previews).toHaveLength(2);
    const body = text(host);
    expect(body).toContain("TASK_ADMISSION_REJECTED");
    expect(body).toContain("nothing is retried automatically");
    expect(body).toContain("needs its own explicit confirmation");
    unmount(host, root);
  });

  it("shows an admission receipt that never claims the task ran", async () => {
    installFetch(creationRoutes());
    const { host, root } = renderAppAt("#/work/new");
    await flush();
    submitForm(host.querySelector("form") as HTMLFormElement);
    await flush(6);
    clickButton(host, "Preview the contract");
    await flush(6);
    clickButton(host, "Confirm and admit");
    await flush(6);

    const body = text(host);
    expect(body).toContain("Admitted");
    expect(body).toContain(TASK_C);
    expect(body).toContain("contract epoch 1");
    expect(body).toContain("not a claim that the task has started, progressed or completed");
    expect(body).not.toMatch(/\brunning\b/);
    unmount(host, root);
  });

  it("remembers the admitted ref for this session so Work can show it", async () => {
    installFetch(creationRoutes());
    const { host, root } = renderAppAt("#/work/new");
    await flush();
    submitForm(host.querySelector("form") as HTMLFormElement);
    await flush(6);
    clickButton(host, "Preview the contract");
    await flush(6);
    clickButton(host, "Confirm and admit");
    await flush(6);

    // The production caller for noteObservedTask that W3 registered and W4 wires.
    const observed = appProjections.get<ObservedTask[]>("home:observed-tasks");
    expect(observed?.data?.map((row) => row.taskRef)).toEqual([TASK_C]);
    expect(observed?.data?.[0].origin).toBe("task/admit");
    expect(observed?.source).toContain("session");
    unmount(host, root);
  });

  it("retains the session chain for W5 and marks the preview ephemeral, never persisted", async () => {
    installFetch(creationRoutes());
    const { host, root } = renderAppAt("#/work/new");
    await flush();
    submitForm(host.querySelector("form") as HTMLFormElement);
    await flush(6);
    clickButton(host, "Preview the contract");
    await flush(6);
    clickButton(host, "Confirm and admit");
    await flush(6);

    const chain = sessionChainFor(appProjections, TASK_C);
    expect(chain?.intent.userIntentRecordId).toBe("rec-1");
    expect(chain?.intent.rawExpression).toBe("search the workspace for needle");
    expect(chain?.interpretation.interpretationId).toBe("int-1");
    expect(chain?.interpretation.interpretationDigest).toBe(INTERPRETATION_DIGEST);
    expect(chain?.interpretation.status).toBe("candidate");
    // The reviewed digest is the link between preview and admission.
    expect(chain?.preview.previewDigest).toBe(PREVIEW_DIGEST);
    expect(chain?.preview.ephemeral).toBe(true);
    expect(chain?.admission.contractEpoch).toBe(1);
    expect(chain?.admission.acceptedBy).toBe("principal://local/owner");
    expect(appProjections.get(WORK_CHAIN_KEY)?.source).toContain("this browser session only");

    // Memory only: nothing about the chain reaches web storage or the URL.
    expect(window.localStorage.length).toBe(0);
    expect(window.sessionStorage.length).toBe(0);
    expect(window.location.href).not.toContain(PREVIEW_DIGEST);
    unmount(host, root);
  });

  it("records the superseded interpretation when a decision forces a re-interpretation", async () => {
    installFetch(creationRoutes());
    const { host, root } = renderAppAt("#/work/new");
    await flush();

    clickButton(host, "Add an ambiguity");
    await flush(2);
    setInputValue(
      host.querySelector('input[name="ambiguity_question_0"]') as HTMLInputElement,
      "which workspace?",
    );
    await flush(2);
    submitForm(host.querySelector("form") as HTMLFormElement);
    await flush(6);
    const answer = [...host.querySelectorAll("input")].find((input) =>
      input.getAttribute("name")?.startsWith("answer_"),
    ) as HTMLInputElement;
    setInputValue(answer, "the personal workspace");
    await flush(2);
    clickButton(host, "Re-interpret with these decisions");
    await flush(6);
    clickButton(host, "Preview the contract");
    await flush(6);
    clickButton(host, "Confirm and admit");
    await flush(6);

    const chain = sessionChainFor(appProjections, TASK_C);
    expect(chain?.interpretation.supersededInterpretationIds).toEqual(["int-1"]);
    expect(chain?.interpretation.recordedDecisions).toContain(
      "which workspace? → the personal workspace",
    );
    // The answered ambiguity is no longer open on the admitted candidate.
    expect(chain?.interpretation.openAmbiguities).toEqual([]);
    unmount(host, root);
  });

  it("surfaces the newly admitted ref in the Work inventory session scope", async () => {
    installFetch(creationRoutes());
    const first = renderAppAt("#/work/new");
    await flush();
    submitForm(first.host.querySelector("form") as HTMLFormElement);
    await flush(6);
    clickButton(first.host, "Preview the contract");
    await flush(6);
    clickButton(first.host, "Confirm and admit");
    await flush(6);
    unmount(first.host, first.root);

    const second = renderAppAt("#/work");
    await flush(8);
    const body = text(second.host);
    expect(body).toContain("Showing 1 known task · inventory is envelope-only (BD-3)");
    expect(body).toContain("observed this session");
    expect(body).toContain("state not exposed");
    unmount(second.host, second.root);
  });

  it("reports a chain failure without admitting anything", async () => {
    const calls = installFetch(
      creationRoutes({
        "POST /task/intent.record": {
          status: 409,
          body: { status: "error", code: "TASK_INTENT_RECORD_REJECTED" },
        },
      }),
    );
    const { host, root } = renderAppAt("#/work/new");
    await flush();
    submitForm(host.querySelector("form") as HTMLFormElement);
    await flush(6);

    expect(host.querySelector('p[role="alert"]')?.textContent).toContain(
      "TASK_INTENT_RECORD_REJECTED",
    );
    expect(text(host)).toContain("Nothing was admitted");
    expect(calls.some((call) => call.path === "/task/intent.interpret")).toBe(false);
    unmount(host, root);
  });
});

describe("legacy Tasks retirement", () => {
  it("redirects #/tasks to Work and does not mount the retired diagnostics page", async () => {
    installFetch({
      "GET /management/resource/v1/list": { status: 200, body: { status: "ok", resources: [] } },
    });
    const { host, root } = renderAppAt("#/tasks");
    await flush();
    expect(window.location.hash).toBe("#/work");
    expect(host.querySelector("h2")?.textContent).toBe("Work");
    expect(text(host)).not.toContain("Tasks, Effects, Evidence");
    expect(text(host)).not.toContain("Watch poll");
    expect(text(host)).not.toContain("Simulate cursor gap");
    expect(host.querySelector('a[href="#/tasks"]')).toBeNull();
    unmount(host, root);
  });
});
