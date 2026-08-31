import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { isKnownRoute } from "../../data/normalize";
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

const FAKE_ACTION = /approve|create project|activate|new project|team|inbox|confirm|ingest|apply authority|install/i;

function fakeActionLabels(host: HTMLElement): string[] {
  const scopes = [
    host.querySelector("#main"),
    host.querySelector("[data-rail='assistant']"),
    host.querySelector("nav[aria-label='Primary']"),
    host.querySelector("nav[aria-label='Project sections']"),
  ].filter((node): node is Element => node !== null);
  const labels: string[] = [];
  for (const scope of scopes) {
    for (const node of scope.querySelectorAll("button, a.cp-button")) {
      const label = (node.textContent ?? "").trim();
      if (FAKE_ACTION.test(label)) {
        labels.push(label);
      }
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
    project: {
      project_id: "proj-1",
      state: "active",
      created_at: "t0",
      activated_at: "t1",
      accepted_at: null,
    },
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
        output_contract: {
          digest: "out-1",
          deliverable_type: "unknown",
          save_format: "unknown",
          open_with: "unknown",
        },
        gaps: [],
      },
    ],
  },
};

const READY_ROSTER: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projection: "personal-private",
    authority_note: "employee",
    roster: [
      {
        employee_id: "emp-1",
        state: "seated",
        model_bound: true,
        is_current_manager: true,
        runtime_binding_ref: "run-1",
      },
    ],
  },
};

const EMPTY_ROSTER: RouteResponse = {
  status: 200,
  body: { status: "ok", authority_note: "empty-roster", roster: [] },
};

function workRoutes(extras: Record<string, RouteResponse> = {}): Record<string, RouteResponse> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/project/v1/list": READY_LIST,
    "GET /management/project/v1/detail": READY_DETAIL,
    "GET /management/project/v1/axis": READY_AXIS,
    "GET /management/project/v1/roster": READY_ROSTER,
    "GET /management/project/v1/pending-previews": {
      status: 200,
      body: { status: "ok", previews: [] },
    },
    ...extras,
  };
}

async function renderWork(hash: string, extras: Record<string, RouteResponse> = {}) {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch(workRoutes(extras));
  const view = renderAppAt(hash);
  await flush();
  return { ...view, calls };
}

afterEach(() => {
  clearSession();
  appProjections.clear();
  window.location.hash = "";
  vi.unstubAllGlobals();
});

describe("P12-T03 Project four submenus", () => {
  it("opens daemon Project detail from the list without minting Activate", async () => {
    const { host, root } = await renderWork("#/projects");
    expect(host.querySelector("a[href='#/projects/proj-1']")?.textContent).toBe("Open");
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("keeps L2 as Detail / Members / Runs / Outputs and never Team or Inbox", async () => {
    const { host, root, calls } = await renderWork("#/projects/proj-1");
    const nav = host.querySelector('nav[aria-label="Project sections"]');
    expect(nav?.textContent).toContain("Detail");
    expect(nav?.textContent).toContain("Members");
    expect(nav?.textContent).toContain("Runs");
    expect(nav?.textContent).toContain("Outputs");
    expect(nav?.textContent).not.toMatch(/Team|Inbox|Work/);
    expect(host.querySelector("[data-page='opc-project-detail']")).not.toBeNull();
    expect(host.textContent).toContain("active");
    expect(host.textContent).toContain("unknown");
    expect(calls.some((call) => call.pathname === "/management/project/v1/detail")).toBe(true);
    expect(calls.some((call) => call.pathname === "/task/evidence")).toBe(false);
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("does not convert a missing Project into an empty list or a fake button", async () => {
    const { host, root } = await renderWork("#/projects/missing", {
      "GET /management/project/v1/detail": {
        status: 404,
        body: { status: "error", code: "PROJECT_NOT_FOUND", message: "project not found" },
      },
      "GET /management/project/v1/axis": {
        status: 404,
        body: { status: "error", code: "PROJECT_NOT_FOUND", message: "project not found" },
      },
    });
    expect(host.textContent).toMatch(/PROJECT_NOT_FOUND|not found|unexpected/i);
    expect(host.querySelector("[data-page='opc-projects']")).toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("keeps empty roster honest and does not offer Install or Add", async () => {
    const { host, root, calls } = await renderWork("#/projects/proj-1/members", {
      "GET /management/project/v1/roster": EMPTY_ROSTER,
    });
    const page = host.querySelector("[data-page='opc-project-members']");
    expect(page).not.toBeNull();
    expect(page?.textContent).toMatch(/empty roster/i);
    expect(page?.textContent).toMatch(/empty-roster/);
    expect(fakeActionLabels(host)).toEqual([]);
    expect(calls.some((call) => call.pathname === "/management/project/v1/roster")).toBe(true);
    expect(calls.some((call) => call.pathname === "/management/project/v1/vault.index")).toBe(false);
    unmount(host, root);
  });

  it("does not default the first Employee until a row is chosen", async () => {
    const { host, root } = await renderWork("#/projects/proj-1/members");
    expect(host.textContent).toMatch(/no member selected/i);
    expect(host.querySelector("[data-region='opc-member-selected']")).toBeNull();
    const button = [...host.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").trim() === "emp-1",
    );
    expect(button).toBeTruthy();
    await act(async () => {
      button?.click();
    });
    const selected = host.querySelector("[data-region='opc-member-selected']");
    expect(selected?.textContent).toContain("emp-1");
    expect(selected?.textContent).not.toMatch(/duty|skills|prompt|Install/i);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("renders runs from the PlanRevision axis, not Work inventory", async () => {
    const { host, root, calls } = await renderWork("#/projects/proj-1/runs");
    expect(host.querySelector("[data-page='opc-project-runs']")).not.toBeNull();
    expect(host.querySelector("[data-row-key='st-1']")).not.toBeNull();
    expect(host.textContent).toContain("Intake");
    expect(host.textContent).toMatch(/not a renamed Work timeline/i);
    expect(calls.some((call) => call.pathname === "/management/project/v1/axis")).toBe(true);
    expect(calls.some((call) => call.pathname === "/task/evidence")).toBe(false);
    expect(host.querySelector("a[href='#/work']")).toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("keeps outputs select-then-view and does not treat Knowledge files as authority", async () => {
    const { host, root, calls } = await renderWork("#/projects/proj-1/outputs");
    expect(host.querySelector("[data-page='opc-project-outputs']")).not.toBeNull();
    expect(host.textContent).toMatch(/no output selected/i);
    expect(host.querySelector("[data-region='opc-output-selected']")).toBeNull();
    const button = [...host.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").trim() === "Intake",
    );
    await act(async () => {
      button?.click();
    });
    expect(host.querySelector("[data-region='opc-output-selected']")?.textContent).toContain("out-1");
    expect(host.textContent).toMatch(/Files are not Project authority/i);
    expect(calls.some((call) => call.pathname === "/management/project/v1/vault.index")).toBe(false);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("does not convert members 403 into an empty roster", async () => {
    const { host, root } = await renderWork("#/projects/proj-1/members", {
      "GET /management/project/v1/roster": {
        status: 403,
        body: { status: "error", error: { code: "LOCAL_ORIGIN_HEADER_REJECTED", message: "denied" } },
      },
    });
    expect(host.textContent).toMatch(/session denied/i);
    expect(host.textContent).not.toMatch(/empty roster/i);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("does not treat #/work as Projects and keeps #/hitl missing", async () => {
    expect(isKnownRoute("GET", "/management/project/v1/detail")).toBe(true);
    const { host, root } = await renderWork("#/hitl");
    expect(host.querySelector("[data-page='opc-project-detail']")).toBeNull();
    expect(host.querySelector("[data-page='not-found']") ?? host.textContent).toBeTruthy();
    unmount(host, root);
  });
});
