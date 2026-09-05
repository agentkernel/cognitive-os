import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { isKnownRoute } from "../../data/normalize";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";

type RouteResponse = { status: number; body: unknown };
type FetchCall = { method: string; path: string; pathname: string; body?: string };

function installFetch(routes: Record<string, RouteResponse>): FetchCall[] {
  const calls: FetchCall[] = [];
  vi.stubGlobal(
    "fetch",
    vi.fn(async (input: unknown, init?: RequestInit) => {
      const url = new URL(String(input), "http://localhost");
      const method = (init?.method ?? "GET").toUpperCase();
      calls.push({
        method,
        path: `${url.pathname}${url.search}`,
        pathname: url.pathname,
        body: typeof init?.body === "string" ? init.body : undefined,
      });
      const handler = routes[`${method} ${url.pathname}`];
      const resolved =
        handler ??
        (url.pathname === "/personal/health" || url.pathname === "/personal/status"
          ? { status: 200, body: { status: "ok", overall: "ready", alerts: [] } }
          : { status: 404, body: { status: "error", code: "NOT_FOUND", message: "not found" } });
      return new Response(JSON.stringify(resolved.body), {
        status: resolved.status,
        headers: { "content-type": "application/json" },
      });
    }),
  );
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

async function flush(ticks = 20) {
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

function clickButton(host: HTMLElement, text: string) {
  const button = [...host.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === text,
  );
  if (!button) {
    throw new Error(`button not found: ${text}`);
  }
  act(() => {
    button.click();
  });
}

function workHashes(host: HTMLElement): string[] {
  return [...host.querySelectorAll("a")]
    .map((node) => node.getAttribute("href") ?? "")
    .filter((href) => href.includes("/work"));
}

/** P13-T05 drift negatives: fake Start, Approve, Complete, Retry-as-run. */
const FAKE_ACTION =
  /^(start|run now|trigger|approve|complete|mark complete|cancel run|retry run|arm|pause|resume|restart)\b/i;

function fakeActionLabels(host: HTMLElement): string[] {
  const labels: string[] = [];
  const main = host.querySelector("#main");
  if (!main) {
    return labels;
  }
  for (const node of main.querySelectorAll("button, a.cp-button")) {
    const label = (node.textContent ?? "").trim();
    if (FAKE_ACTION.test(label)) {
      labels.push(label);
    }
  }
  return labels;
}

const READY_LIST: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [{ project_id: "proj-1", state: "active", title_summary: "unknown", cost: "unknown" }],
  },
};

const READY_DETAIL: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projection: "personal-private",
    project: { project_id: "proj-1", state: "active", created_at: "t0", activated_at: "t1", accepted_at: "t2" },
    charter: { status: "confirmed", content_digest: "dig-1" },
    plan: { plan_revision_id: "plan-1" },
    pending_preview_count: 0,
    cost: "unknown",
  },
};

const READY_AXIS: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projection: "personal-private",
    plan_revision_id: "plan-1",
    stages: [
      {
        stage_id: "st-1",
        position: 0,
        title: "Intake",
        confirm_status: "confirmed",
        ready: false,
        seated: true,
        output_contract: { digest: "out-1", deliverable_type: "unknown", save_format: "unknown", open_with: "unknown" },
        gaps: [],
      },
    ],
  },
};

const READY_ROSTER: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    authority_note: "employee",
    roster: [
      {
        employee_id: "emp-1",
        state: "seated",
        model_bound: true,
        is_current_manager: false,
        runtime_binding_ref: "run-1",
        responsible_stage_ids: ["st-1"],
      },
    ],
  },
};

const RUNS: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projection_id: "personal-private.routine-arming/0.1",
    project_id: "proj-1",
    host: { available: false, reason: "close-paused" },
    scheduler: "daemon-tick-only",
    armings: [
      {
        arming_id: "arm-1",
        project_id: "proj-1",
        routine_id: "rt-1",
        revision_id: "rev-1",
        stage_id: "st-1",
        employee_id: "emp-1",
        cadence_kind: "interval",
        interval_ms: 5000,
        state: "paused",
        apply_mode: "pause",
        armed_after: "G2",
        next_due_at: null,
      },
    ],
    occurrences: [
      {
        occurrence_id: "occ-running",
        routine_id: "rt-1",
        revision_id: "rev-1",
        trigger_kind: "schedule",
        trigger_source: "daemon-tick",
        requested_at: 900,
        disposition: "active",
        dispatch_state: "running",
        attempt_id: "att-1",
        attempt_outcome: null,
        completion_claimed: false,
        verification_status: "not-run",
      },
      {
        occurrence_id: "occ-queued",
        routine_id: "rt-1",
        revision_id: "rev-1",
        trigger_kind: "manual",
        trigger_source: "owner-run",
        requested_at: 950,
        disposition: "queued",
        dispatch_state: "queued",
        completion_claimed: false,
        verification_status: "not-run",
      },
      {
        occurrence_id: "occ-old",
        routine_id: "rt-1",
        revision_id: "rev-1",
        trigger_kind: "manual",
        trigger_source: "owner-run",
        requested_at: 940,
        disposition: "coalesced",
        dispatch_state: "coalesced",
        coalesced_by: "occ-queued",
        completion_claimed: false,
        verification_status: "not-run",
      },
      {
        occurrence_id: "occ-missed",
        routine_id: "rt-1",
        revision_id: "rev-1",
        trigger_kind: "schedule",
        trigger_source: "daemon-tick",
        requested_at: 500,
        disposition: "missed",
        dispatch_state: "missed",
        miss_reason: "host-unavailable:close-paused",
        completion_claimed: false,
        verification_status: "not-run",
      },
      {
        occurrence_id: "occ-done",
        routine_id: "rt-1",
        revision_id: "rev-1",
        trigger_kind: "schedule",
        trigger_source: "daemon-tick",
        requested_at: 100,
        disposition: "attempted",
        dispatch_state: "attempted",
        attempt_id: "att-0",
        attempt_outcome: "done",
        elapsed_ms: 1500,
        completion_claimed: false,
        verification_status: "not-run",
      },
    ],
    summary: {
      active: 1,
      running: 1,
      queued: 1,
      missed: 1,
      coalesced: 1,
      attempted: 1,
      done: 1,
      failed: 0,
      unknown: 0,
    },
    attempt_history_path: "/management/project/v1/dsh.hosted.attempt.list?project_id=proj-1",
    manual_trigger_path: "/management/project/v1/routine.trigger",
    receipt_is_not_completion: true,
    verification_status: "not-run",
    clock_sleep_restart_host_e2e: "not-run",
  },
};

const ATTEMPTS: RouteResponse = {
  status: 200,
  body: {
    projection: "personal-private.hosted-attempt/0.1",
    project_id: "proj-1",
    attempts: [
      {
        attempt_id: "att-1",
        employee_id: "emp-1",
        task_ref: "task://personal/routine/occ-running",
        state: "running",
        terminal_kind: null,
        exit_code: null,
        response_status: "working",
        completion_claimed: false,
        verification_status: "not-run",
        elapsed_ms: null,
        created_at: 900,
      },
      {
        attempt_id: "att-0",
        employee_id: "emp-1",
        task_ref: "task://personal/routine/occ-done",
        state: "terminal",
        terminal_kind: "exited",
        exit_code: 0,
        response_status: "done",
        completion_claimed: false,
        verification_status: "not-run",
        elapsed_ms: 1500,
        created_at: 100,
        terminal_at: 1600,
      },
    ],
    receipt_is_not_completion: true,
  },
};

function routes(extras: Record<string, RouteResponse> = {}): Record<string, RouteResponse> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/project/v1/list": READY_LIST,
    "GET /management/project/v1/detail": READY_DETAIL,
    "GET /management/project/v1/axis": READY_AXIS,
    "GET /management/project/v1/roster": READY_ROSTER,
    "GET /management/project/v1/pending-previews": { status: 200, body: { status: "ok", previews: [] } },
    "GET /management/project/v1/routine.runs": RUNS,
    "GET /management/project/v1/dsh.hosted.attempt.list": ATTEMPTS,
    ...extras,
  };
}

async function renderRuns(extras: Record<string, RouteResponse> = {}) {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch(routes(extras));
  const view = renderAppAt("#/projects/proj-1/runs");
  await flush();
  return { ...view, calls };
}

afterEach(() => {
  clearSession();
  appProjections.clear();
  window.location.hash = "";
  vi.unstubAllGlobals();
});

describe("P13-T05/D02 Project runs: occurrence ledger + Attempt history", () => {
  it("renders the daemon occurrence ledger with no-overlap, queue-latest, missed and coalesced facts", async () => {
    const { host, root, calls } = await renderRuns();
    const page = host.querySelector("[data-page='opc-project-runs']");
    expect(page).not.toBeNull();
    expect(host.querySelector("[data-region='opc-routine-ledger']")).not.toBeNull();

    const running = host.querySelector("[data-occurrence='occ-running']");
    expect(running?.getAttribute("data-dispatch-state")).toBe("running");
    expect(running?.textContent).toContain("att-1");

    const queued = host.querySelector("[data-occurrence='occ-queued']");
    expect(queued?.getAttribute("data-disposition")).toBe("queued");

    const coalesced = host.querySelector("[data-occurrence='occ-old']");
    expect(coalesced?.getAttribute("data-disposition")).toBe("coalesced");
    expect(coalesced?.textContent).toMatch(/coalesced by/);
    expect(coalesced?.textContent).toContain("occ-queued");

    const missed = host.querySelector("[data-occurrence='occ-missed']");
    expect(missed?.getAttribute("data-disposition")).toBe("missed");
    expect(missed?.textContent).toContain("host-unavailable:close-paused");

    const done = host.querySelector("[data-occurrence='occ-done']");
    expect(done?.textContent).toContain("done");
    expect(done?.textContent).toContain("1500 ms");
    expect(done?.textContent).not.toMatch(/contract violation/);

    const summary = host.querySelector("[data-region='opc-routine-runs-summary']")?.textContent ?? "";
    expect(summary).toContain("daemon-tick-only");
    expect(summary).toMatch(/host available false/);
    expect(summary).toContain("close-paused");
    expect(summary).toMatch(/missed 1/);
    expect(summary).toMatch(/coalesced 1/);
    expect(summary).toMatch(/clock \/ sleep \/ restart host E2E not-run/);

    expect(host.querySelector("[data-arming='arm-1']")?.textContent).toMatch(/paused/);
    expect(host.querySelector("[data-arming='arm-1']")?.textContent).toContain("5000 ms");

    expect(calls.some((call) => call.path === "/management/project/v1/routine.runs?project_id=proj-1")).toBe(true);
    expect(
      calls.some((call) => call.path === "/management/project/v1/dsh.hosted.attempt.list?project_id=proj-1"),
    ).toBe(true);
    expect(calls.some((call) => call.pathname.startsWith("/task/"))).toBe(false);
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    unmount(host, root);
  });

  it("renders real Attempt history as receipts, never completion, and keeps the PlanRevision axis", async () => {
    const { host, root } = await renderRuns();
    const history = host.querySelector("[data-region='opc-attempt-history-table']");
    expect(history).not.toBeNull();
    const done = host.querySelector("[data-attempt='att-0']");
    expect(done?.textContent).toContain("terminal");
    expect(done?.textContent).toContain("exited / exit 0");
    expect(done?.textContent).toContain("done");
    expect(done?.textContent).toContain("false");
    expect(done?.textContent).toContain("not-run");
    const running = host.querySelector("[data-attempt='att-1']");
    expect(running?.textContent).toContain("working");
    const page = host.querySelector("[data-page='opc-project-runs']");
    expect(page?.textContent).toMatch(/receipt is not completion/i);
    expect(page?.textContent).not.toMatch(/\bcompleted\b/i);
    expect(host.querySelector("[data-row-key='st-1']")).not.toBeNull();
    expect(host.textContent).toContain("Intake");
    unmount(host, root);
  });

  it("offers no Start, Approve, Complete, arm, pause, resume or restart control", async () => {
    const { host, root } = await renderRuns();
    expect(fakeActionLabels(host)).toEqual([]);
    expect(host.querySelector("#main form")).toBeNull();
    expect(host.textContent).not.toMatch(/start button/i);
    unmount(host, root);
  });

  it("keeps an empty ledger honest: no armed Routine, no occurrence, no Attempt, and no fake run", async () => {
    const { host, root } = await renderRuns({
      "GET /management/project/v1/routine.runs": {
        status: 200,
        body: {
          status: "ok",
          project_id: "proj-1",
          host: { available: true, reason: null },
          scheduler: "daemon-tick-only",
          armings: [],
          occurrences: [],
          summary: { active: 0, running: 0, queued: 0, missed: 0, coalesced: 0, attempted: 0, done: 0, failed: 0, unknown: 0 },
          receipt_is_not_completion: true,
          verification_status: "not-run",
        },
      },
      "GET /management/project/v1/dsh.hosted.attempt.list": {
        status: 200,
        body: { projection: "personal-private.hosted-attempt/0.1", project_id: "proj-1", attempts: [] },
      },
    });
    expect(host.querySelector("[data-row-key='no-arming']")?.textContent).toMatch(/missed \/ not-armed, never vanish/);
    expect(host.querySelector("[data-row-key='no-occurrence']")).not.toBeNull();
    expect(host.querySelector("[data-region='opc-attempt-history']")?.textContent).toMatch(
      /no hosted Attempt yet/i,
    );
    expect(host.querySelector("[data-occurrence]")).toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("does not convert a denied ledger or missing counts into zeros", async () => {
    const denied = await renderRuns({
      "GET /management/project/v1/routine.runs": {
        status: 403,
        body: { status: "error", code: "ROUTINE_RUNS_CHANNEL_FORBIDDEN", message: "management only" },
      },
    });
    expect(denied.host.querySelector("[data-region='opc-routine-runs']")?.textContent).toMatch(
      /session denied/i,
    );
    expect(denied.host.querySelector("[data-region='opc-routine-ledger']")).toBeNull();
    expect(denied.host.querySelector("[data-row-key='no-occurrence']")).toBeNull();
    unmount(denied.host, denied.root);
    appProjections.clear();

    const partial = await renderRuns({
      "GET /management/project/v1/routine.runs": {
        status: 200,
        body: {
          status: "ok",
          project_id: "proj-1",
          armings: [],
          occurrences: [],
          summary: {},
        },
      },
    });
    const summary =
      partial.host.querySelector("[data-region='opc-routine-runs-summary']")?.textContent ?? "";
    expect(summary).toMatch(/missed unknown/);
    expect(summary).not.toMatch(/missed 0/);
    unmount(partial.host, partial.root);
  });
});

describe("P14-T05/D01 Project chrome Write Attempt (not Linux 1.0 Work)", () => {
  it("does not treat #/work as 2.0 chrome and never advertises Vite as product origin", async () => {
    const { host, root } = await renderRuns();
    expect(workHashes(host)).toEqual([]);
    expect(host.textContent).toMatch(/daemon-served hash \/ui\//i);
    expect(host.textContent).toMatch(/Vite preview is not the product origin/);
    expect(host.textContent).toMatch(/not Linux 1\.0 #\/work/);
    expect(host.querySelector("[data-page='opc-project-runs']")).not.toBeNull();
    unmount(host, root);
  });

  it("does not offer a clickable Run or Write Attempt without a live Project and seated Member", async () => {
    const creating = await renderRuns({
      "GET /management/project/v1/detail": {
        status: 200,
        body: {
          status: "ok",
          projection: "personal-private",
          project: { project_id: "proj-1", state: "creating", created_at: "t0" },
          charter: { status: "draft", content_digest: "dig-1" },
          plan: { plan_revision_id: "unknown" },
          pending_preview_count: 0,
        },
      },
      "GET /management/project/v1/roster": {
        status: 200,
        body: { status: "ok", authority_note: "empty-roster", roster: [] },
      },
    });
    const creatingButton = creating.host.querySelector(
      "button[data-write-attempt]",
    ) as HTMLButtonElement | null;
    expect(creatingButton?.disabled).toBe(true);
    expect(creatingButton?.getAttribute("data-write-attempt")).toBe("blocked");
    expect(creating.host.querySelector("[data-region='opc-write-attempt']")?.textContent).toMatch(
      /Nothing was dispatched/,
    );
    expect(fakeActionLabels(creating.host)).toEqual([]);
    creatingButton?.click();
    await flush();
    expect(
      creating.calls.some((call) => call.pathname === "/management/project/v1/dsh.hosted.attempt.run"),
    ).toBe(false);
    unmount(creating.host, creating.root);
    appProjections.clear();

    const unseated = await renderRuns({
      "GET /management/project/v1/roster": {
        status: 200,
        body: {
          status: "ok",
          authority_note: "employee",
          roster: [{ employee_id: "emp-1", state: "registered", model_bound: false }],
        },
      },
    });
    const unseatedButton = unseated.host.querySelector(
      "button[data-write-attempt]",
    ) as HTMLButtonElement | null;
    expect(unseatedButton?.disabled).toBe(true);
    expect(unseatedButton?.getAttribute("data-write-attempt")).toBe("blocked");
    expect(fakeActionLabels(unseated.host)).toEqual([]);
    expect(
      unseated.calls.some((call) => call.method === "POST" && call.pathname.includes("attempt.run")),
    ).toBe(false);
    unmount(unseated.host, unseated.root);
  });

  it("posts management dsh.hosted.attempt.run from Write Attempt, never the task channel or #/work", async () => {
    const { host, root, calls } = await renderRuns({
      "POST /management/project/v1/dsh.hosted.attempt.run": {
        status: 200,
        body: {
          status: "ok",
          attempt: {
            attempt_id: "dshattempt-new",
            employee_id: "emp-1",
            task_ref: "task://personal/project/proj-1",
            state: "working",
            completion_claimed: false,
            verification_status: "not-run",
          },
          receipt_is_not_completion: true,
        },
      },
    });
    expect(isKnownRoute("POST", "/management/project/v1/dsh.hosted.attempt.run")).toBe(true);
    expect(isKnownRoute("POST", "/task/project/v1/dsh.hosted.attempt.run")).toBe(false);
    const button = host.querySelector("button[data-write-attempt='ready']") as HTMLButtonElement | null;
    expect(button?.disabled).toBe(false);
    expect(fakeActionLabels(host)).toEqual([]);
    clickButton(host, "Write Attempt");
    await flush();
    const posted = calls.find(
      (call) => call.method === "POST" && call.pathname === "/management/project/v1/dsh.hosted.attempt.run",
    );
    expect(posted).toBeDefined();
    expect(posted?.body).toContain("\"employee_id\":\"emp-1\"");
    expect(posted?.body).toContain("\"task_ref\":\"task://personal/project/proj-1\"");
    expect(posted?.body).toContain("\"wait\":false");
    expect(calls.some((call) => call.pathname.startsWith("/task/"))).toBe(false);
    expect(workHashes(host)).toEqual([]);
    unmount(host, root);
  });
});
