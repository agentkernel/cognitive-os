import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";
import { TODAY_EMPTY_ONLY_CREATE, TODAY_INCOMPLETE_ONLY_CREATE } from "./ProjectAuthorityPanel";

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

function overview(
  period: string,
  rows: unknown[],
  counts: Record<string, unknown>,
  extra: Record<string, unknown> = {},
): RouteResponse {
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
      ...extra,
    },
  };
}

const ALPHA_ROW = {
  project_id: "proj-alpha",
  state: "active",
  status: "running",
  armed_routines: 1,
  paused_routines: 0,
  running_occurrence_id: "occ-1",
  queued_count: 0,
  missed_count: 0,
  attempts_total: 1,
  attempts_done: 1,
  attempts_failed: 0,
  attempts_unknown: 0,
  duration_ms: 5000,
  current_stage_id: "collect",
  current_stage_title: "Collect",
  cost: "unknown",
};

const BETA_ROW = {
  ...ALPHA_ROW,
  project_id: "proj-beta",
  status: "paused",
  current_stage_id: "analyze",
  current_stage_title: "Analyze",
};

function routes(list: RouteResponse, extras: Record<string, RouteResponse> = {}): Record<string, RouteResponse> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/project/v1/list": list,
    "GET /management/project/v1/pending-previews": { status: 200, body: { status: "ok", previews: [] } },
    "GET /management/project/v1/today.overview?period=today": overview("today", [ALPHA_ROW], {
      created: 0,
      live: 1,
      blocked: 0,
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

const EMPTY_LIST: RouteResponse = { status: 200, body: { status: "ok", projects: [] } };
const TITLED_CREATING: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [
      { project_id: "proj-draft", state: "creating", title_summary: "Almost Alpha", cost: "unknown" },
    ],
  },
};
const TITLED_LIVE: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [
      { project_id: "proj-alpha", state: "active", title_summary: "Owner Alpha", cost: "unknown" },
    ],
  },
};
const TWO_LIVE: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [
      { project_id: "proj-alpha", state: "active", title_summary: "Owner Alpha", cost: "unknown" },
      { project_id: "proj-beta", state: "attention", title_summary: "Owner Beta", cost: "unknown" },
    ],
  },
};
const MIXED_AFTER_ACTIVATION: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [
      { project_id: "proj-draft", state: "creating", title_summary: "unknown", cost: "unknown" },
      { project_id: "proj-alpha", state: "active", title_summary: "Owner Alpha", cost: "unknown" },
    ],
  },
};

afterEach(() => {
  clearSession();
  appProjections.clear();
  window.location.hash = "";
  vi.unstubAllGlobals();
});

describe("P14-T06/D01 Today live packets after activation (Dual Track)", () => {
  it("does not treat T13 empty chrome as packet acceptance", async () => {
    const { host, root, calls } = await renderToday(EMPTY_LIST);
    expect(host.querySelector("[data-surface='today']")).toBeNull();
    expect(host.querySelector("[data-packet]")).toBeNull();
    expect(host.textContent).toContain(TODAY_EMPTY_ONLY_CREATE);
    expect(host.textContent).not.toMatch(/packet accepted|packets are accepted/i);
    expect(host.querySelector("a[href='#/projects/new']")?.textContent).toMatch(/Start create/);
    expect(calls.some((call) => call.pathname === "/management/project/v1/pending-previews")).toBe(
      false,
    );
    expect(calls.some((call) => call.pathname === "/management/project/v1/today.overview")).toBe(
      false,
    );
    unmount(host, root);
  });

  it("does not paint live packets for an unactivated titled Project", async () => {
    const { host, root, calls } = await renderToday(TITLED_CREATING, {
      "GET /management/project/v1/pending-previews?subject_ref=proj-draft": {
        status: 200,
        body: {
          status: "ok",
          previews: [
            {
              preview_id: "prev-draft",
              subject_kind: "activation",
              subject_ref: "proj-draft",
              status: "pending",
            },
          ],
        },
      },
    });
    expect(host.querySelector("[data-surface='today-incomplete']")).not.toBeNull();
    expect(host.querySelector("[data-surface='today']")).toBeNull();
    expect(host.querySelector("[data-packet]")).toBeNull();
    expect(host.querySelector("[data-region='opc-today-run-overview']")).toBeNull();
    expect(host.textContent).toContain(TODAY_INCOMPLETE_ONLY_CREATE);
    expect(host.textContent).toContain("Almost Alpha");
    expect(host.querySelector("a[href='#/projects/new']")?.textContent).toMatch(/Continue create/);
    expect(fakeActionLabels(host)).toEqual([]);
    expect(calls.some((call) => call.pathname === "/management/project/v1/pending-previews")).toBe(
      false,
    );
    expect(calls.some((call) => call.pathname === "/management/project/v1/today.overview")).toBe(
      false,
    );
    unmount(host, root);
  });

  it("refuses a KPI wall and never renders success rate as Today chrome", async () => {
    const { host, root } = await renderToday(TITLED_LIVE, {
      "GET /management/project/v1/today.overview?period=today": overview(
        "today",
        [ALPHA_ROW],
        { created: 0, live: 1, blocked: 0 },
        { kpi_wall: true, success_rate: 0.99, weekly_report: "shipped" },
      ),
    });
    expect(host.querySelector("[data-kpi-wall='refused']")).not.toBeNull();
    expect(host.querySelector("[data-region='opc-today-counts']")).toBeNull();
    expect(host.textContent).not.toMatch(/success rate/i);
    expect(host.textContent).not.toMatch(/weekly report/i);
    expect(host.querySelector("[data-overview-project='proj-alpha']")).not.toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("after a titled live Project, Today is packets plus per-Project overview, not continue-create", async () => {
    const { host, root, calls } = await renderToday(TITLED_LIVE, {
      "GET /management/project/v1/pending-previews?subject_ref=proj-alpha": {
        status: 200,
        body: {
          status: "ok",
          previews: [
            {
              preview_id: "prev-alpha",
              subject_kind: "activation",
              subject_ref: "proj-alpha",
              status: "pending",
            },
          ],
        },
      },
    });
    expect(host.querySelector("[data-surface='today']")).not.toBeNull();
    expect(host.querySelector("[data-surface='today-incomplete']")).toBeNull();
    expect(host.textContent).not.toContain(TODAY_INCOMPLETE_ONLY_CREATE);
    expect(host.textContent).not.toMatch(/Continue create/i);
    expect(host.textContent).toContain("Owner Alpha");
    expect(host.querySelector("[data-packet='prev-alpha']")).not.toBeNull();
    expect(
      host.querySelector("a[href='#/projects/proj-alpha?preview=prev-alpha']")?.textContent,
    ).toMatch(/Open this decision on the canvas/i);
    expect(host.querySelector("[data-region='opc-today-run-overview']")).not.toBeNull();
    expect(host.querySelector("[data-overview-project='proj-alpha']")).not.toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    expect(
      calls.some(
        (call) => call.path === "/management/project/v1/pending-previews?subject_ref=proj-alpha",
      ),
    ).toBe(true);
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    unmount(host, root);
  });

  it("loads a packet for every live Project and deep-links that subject, not the first live id", async () => {
    const { host, root, calls } = await renderToday(TWO_LIVE, {
      "GET /management/project/v1/today.overview?period=today": overview(
        "today",
        [ALPHA_ROW, BETA_ROW],
        { created: 0, live: 2, blocked: 0 },
      ),
      "GET /management/project/v1/pending-previews?subject_ref=proj-alpha": {
        status: 200,
        body: { status: "ok", previews: [] },
      },
      "GET /management/project/v1/pending-previews?subject_ref=proj-beta": {
        status: 200,
        body: {
          status: "ok",
          previews: [
            {
              preview_id: "prev-beta",
              subject_kind: "run-acceptance",
              subject_ref: "proj-beta",
              status: "pending",
            },
          ],
        },
      },
    });
    expect(host.querySelector("[data-packet='prev-beta']")).not.toBeNull();
    expect(host.querySelector("a[href='#/projects/proj-beta?preview=prev-beta']")).not.toBeNull();
    expect(host.querySelector("a[href='#/projects/proj-alpha?preview=prev-beta']")).toBeNull();
    expect(host.querySelector("[data-overview-project='proj-alpha']")).not.toBeNull();
    expect(host.querySelector("[data-overview-project='proj-beta']")).not.toBeNull();
    expect(
      calls.some(
        (call) => call.path === "/management/project/v1/pending-previews?subject_ref=proj-alpha",
      ),
    ).toBe(true);
    expect(
      calls.some(
        (call) => call.path === "/management/project/v1/pending-previews?subject_ref=proj-beta",
      ),
    ).toBe(true);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("after activation, leftover creating drafts do not keep Today on continue-create", async () => {
    const { host, root, calls } = await renderToday(MIXED_AFTER_ACTIVATION);
    expect(host.querySelector("[data-surface='today']")).not.toBeNull();
    expect(host.querySelector("[data-surface='today-incomplete']")).toBeNull();
    expect(host.textContent).not.toMatch(/Continue create/i);
    expect(host.querySelector("[data-region='opc-today-leftover-drafts']")).not.toBeNull();
    expect(host.querySelector("[data-region='opc-today-run-overview']")).not.toBeNull();
    expect(host.textContent).toContain("Owner Alpha");
    expect(fakeActionLabels(host)).toEqual([]);
    expect(
      calls.some((call) => call.path.includes("subject_ref=proj-draft")),
    ).toBe(false);
    expect(
      calls.some(
        (call) => call.path === "/management/project/v1/pending-previews?subject_ref=proj-alpha",
      ),
    ).toBe(true);
    unmount(host, root);
  });
});
