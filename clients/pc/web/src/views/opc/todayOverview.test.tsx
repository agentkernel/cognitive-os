import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";

type RouteResponse = { status: number; body: unknown };
type FetchCall = { method: string; path: string; pathname: string };

function installFetch(routes: Record<string, RouteResponse>): FetchCall[] {
  const calls: FetchCall[] = [];
  vi.stubGlobal(
    "fetch",
    vi.fn(async (input: unknown, init?: RequestInit) => {
      const url = new URL(String(input), "http://localhost");
      const method = (init?.method ?? "GET").toUpperCase();
      calls.push({ method, path: `${url.pathname}${url.search}`, pathname: url.pathname });
      const byPath = routes[`${method} ${url.pathname}${url.search}`];
      const handler = byPath ?? routes[`${method} ${url.pathname}`];
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

const FAKE_ACTION =
  /approve|create project|activate|new project|team|inbox|confirm|ingest|apply authority|^start\b|^run\b|complete/i;

function fakeActionLabels(host: HTMLElement): string[] {
  const labels: string[] = [];
  const main = host.querySelector("#main");
  if (!main) {
    return labels;
  }
  for (const node of main.querySelectorAll("button, a.cp-button")) {
    const label = (node.textContent ?? "").trim();
    if (node.closest("[data-region='opc-rail-write']")) {
      continue;
    }
    if (FAKE_ACTION.test(label)) {
      labels.push(label);
    }
  }
  return labels;
}

const LIVE_LIST: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [
      { project_id: "proj-1", state: "active", title_summary: "unknown", cost: "unknown" },
      { project_id: "proj-2", state: "paused", title_summary: "unknown", cost: "unknown" },
    ],
  },
};

const CREATING_LIST: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [{ project_id: "proj-draft", state: "creating", title_summary: "unknown", cost: "unknown" }],
  },
};

function overview(period: string, rows: unknown[], counts: Record<string, unknown>): RouteResponse {
  return {
    status: 200,
    body: {
      status: "ok",
      projection_id: "personal-private.today-overview/0.1",
      period,
      period_start_ms: 0,
      now_ms: 10,
      period_basis: "utc",
      counts,
      rows,
      kpi_wall: false,
      verification_status: "not-run",
      cost: "unknown",
    },
  };
}

const TODAY_ROWS = [
  {
    project_id: "proj-1",
    state: "active",
    status: "running",
    armed_routines: 1,
    paused_routines: 0,
    running_occurrence_id: "occ-9",
    queued_count: 1,
    missed_count: 2,
    attempts_total: 3,
    attempts_done: 2,
    attempts_failed: 1,
    attempts_unknown: 0,
    duration_ms: 65000,
    current_stage_id: "s1",
    current_stage_title: "Draft",
    cost: "unknown",
  },
  {
    project_id: "proj-2",
    state: "paused",
    status: "paused",
    armed_routines: 0,
    paused_routines: 1,
    running_occurrence_id: null,
    queued_count: 0,
    missed_count: 0,
    attempts_total: 0,
    attempts_done: 0,
    attempts_failed: 0,
    attempts_unknown: 0,
    duration_ms: null,
    current_stage_id: null,
    current_stage_title: null,
    cost: "unknown",
  },
];

function routes(list: RouteResponse, extras: Record<string, RouteResponse> = {}): Record<string, RouteResponse> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/project/v1/list": list,
    "GET /management/project/v1/pending-previews": { status: 200, body: { status: "ok", previews: [] } },
    "GET /management/project/v1/today.overview?period=today": overview("today", TODAY_ROWS, {
      created: 0,
      live: 2,
      blocked: 0,
    }),
    "GET /management/project/v1/today.overview?period=week": overview("week", [TODAY_ROWS[0]], {
      created: 1,
      live: 2,
      blocked: 1,
    }),
    ...extras,
  };
}

async function renderToday(list: RouteResponse, extras: Record<string, RouteResponse> = {}) {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch(routes(list, extras));
  const view = renderAppAt("#/");
  await flush();
  return { ...view, calls };
}

afterEach(() => {
  clearSession();
  appProjections.clear();
  window.location.hash = "";
  vi.unstubAllGlobals();
});

describe("P13-T05/D02 Today run overview", () => {
  it("renders one row per live Project with status, completed runs, current stage and duration, plus counts", async () => {
    const { host, root, calls } = await renderToday(LIVE_LIST);
    expect(host.querySelector("[data-surface='today']")).not.toBeNull();
    const block = host.querySelector("[data-region='opc-today-run-overview']");
    expect(block).not.toBeNull();
    expect(block?.getAttribute("data-period")).toBe("today");
    expect(host.querySelector("[data-count='created']")?.textContent).toBe("0");
    expect(host.querySelector("[data-count='live']")?.textContent).toBe("2");
    expect(host.querySelector("[data-count='blocked']")?.textContent).toBe("0");

    const running = host.querySelector("[data-overview-project='proj-1']");
    expect(running?.getAttribute("data-status")).toBe("running");
    expect(running?.textContent).toContain("running · active");
    expect(running?.textContent).toContain("s1");
    expect(running?.textContent).toContain("Draft");
    expect(running?.textContent).toContain("1 min 5 s");
    expect(running?.textContent).toContain("1 / 2");
    expect(host.querySelector("[data-overview-project='proj-1'] a[href='#/projects/proj-1/runs']")).not.toBeNull();

    const paused = host.querySelector("[data-overview-project='proj-2']");
    expect(paused?.getAttribute("data-status")).toBe("paused");
    expect(paused?.textContent).toContain("unknown");

    expect(host.textContent).toMatch(/not a KPI wall/i);
    expect(host.textContent).not.toMatch(/success rate/i);
    expect(fakeActionLabels(host)).toEqual([]);
    expect(calls.some((call) => call.path === "/management/project/v1/today.overview?period=today")).toBe(true);
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    unmount(host, root);
  });

  it("collapses the decision packet block when nothing is pending and keeps the overview", async () => {
    const { host, root } = await renderToday(LIVE_LIST);
    const packetBlock = host.querySelector("[data-region='opc-today-packet-block']");
    expect(packetBlock?.getAttribute("data-collapsed")).toBe("true");
    expect((packetBlock as HTMLDetailsElement | null)?.open).toBe(false);
    expect(packetBlock?.textContent).toMatch(/nothing pending/i);
    expect(host.querySelector("[data-packet]")).toBeNull();
    expect(host.querySelector("[data-region='opc-today-run-overview']")).not.toBeNull();
    unmount(host, root);
  });

  it("opens the packet block when a preview is pending and still shows the overview", async () => {
    const { host, root } = await renderToday(LIVE_LIST, {
      "GET /management/project/v1/pending-previews": {
        status: 200,
        body: {
          status: "ok",
          previews: [{ preview_id: "prev-1", subject_kind: "activation", subject_ref: "proj-1", status: "pending" }],
        },
      },
    });
    const packetBlock = host.querySelector("[data-region='opc-today-packet-block']");
    expect(packetBlock?.getAttribute("data-collapsed")).toBe("false");
    expect((packetBlock as HTMLDetailsElement | null)?.open).toBe(true);
    expect(host.querySelector("[data-packet='prev-1']")).not.toBeNull();
    expect(host.querySelector("[data-region='opc-today-run-overview']")).not.toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("switches the period and refetches with ?period=week without a POST", async () => {
    const { host, root, calls } = await renderToday(LIVE_LIST);
    const week = host.querySelector("[data-region='opc-today-period'] [data-period='week']") as HTMLButtonElement | null;
    expect(week).not.toBeNull();
    expect(week?.getAttribute("aria-pressed")).toBe("false");
    await act(async () => {
      week?.click();
    });
    await flush();
    expect(week?.getAttribute("aria-pressed")).toBe("true");
    expect(calls.some((call) => call.path === "/management/project/v1/today.overview?period=week")).toBe(true);
    const block = host.querySelector("[data-region='opc-today-run-overview']");
    expect(block?.getAttribute("data-period")).toBe("week");
    expect(host.querySelector("[data-count='created']")?.textContent).toBe("1");
    expect(host.querySelector("[data-count='blocked']")?.textContent).toBe("1");
    expect(host.querySelector("[data-overview-project='proj-2']")).toBeNull();
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    unmount(host, root);
  });

  it("never fetches the overview for a creating-only home and never turns missing counts into 0", async () => {
    const creating = await renderToday(CREATING_LIST);
    expect(creating.host.querySelector("[data-surface='today-incomplete']")).not.toBeNull();
    expect(creating.host.querySelector("[data-region='opc-today-overview']")).toBeNull();
    expect(creating.calls.some((call) => call.pathname === "/management/project/v1/today.overview")).toBe(false);
    unmount(creating.host, creating.root);
    appProjections.clear();

    const partial = await renderToday(LIVE_LIST, {
      "GET /management/project/v1/today.overview?period=today": overview("today", [], { created: 1 }),
    });
    expect(partial.host.querySelector("[data-count='created']")?.textContent).toBe("1");
    expect(partial.host.querySelector("[data-count='live']")?.textContent).toBe("unknown");
    expect(partial.host.querySelector("[data-count='blocked']")?.textContent).toBe("unknown");
    expect(partial.host.querySelector("[data-row-key='no-live-row']")).not.toBeNull();
    unmount(partial.host, partial.root);
    appProjections.clear();

    const denied = await renderToday(LIVE_LIST, {
      "GET /management/project/v1/today.overview?period=today": {
        status: 403,
        body: { status: "error", code: "ROUTINE_RUNS_CHANNEL_FORBIDDEN", message: "management only" },
      },
    });
    expect(denied.host.querySelector("[data-region='opc-today-overview']")?.textContent).toMatch(
      /session denied/i,
    );
    expect(denied.host.querySelector("[data-region='opc-today-run-overview']")).toBeNull();
    unmount(denied.host, denied.root);
  });
});
