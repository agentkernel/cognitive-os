import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
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

const BASE: Record<string, RouteResponse> = {
  "GET /personal/health": { status: 200, body: { status: "ok" } },
  "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
  "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
  "GET /management/project/v1/list": {
    status: 200,
    body: {
      status: "ok",
      projects: [{ project_id: "proj-1", state: "active", title_summary: "unknown", cost: "unknown" }],
    },
  },
  "GET /management/project/v1/detail": {
    status: 200,
    body: {
      status: "ok",
      project: {
        project_id: "proj-1",
        state: "active",
        charter_status: "confirmed",
        charter_digest: "d".repeat(64),
        plan_revision_id: "plan-1",
        cost: "unknown",
        pending_preview_count: 0,
      },
    },
  },
  "GET /management/project/v1/axis": {
    status: 200,
    body: { status: "ok", stages: [] },
  },
  "GET /management/project/v1/pending-previews": {
    status: 200,
    body: { status: "ok", previews: [] },
  },
  "GET /management/project/v1/lifecycle": {
    status: 200,
    body: {
      status: "ok",
      project_id: "proj-1",
      state: "active",
      data_dir: "projects/proj-1",
      logically_deleted: false,
      is_disaster_backup: false,
      events: [],
      restore_points: [],
    },
  },
  "POST /management/project/v1/copy": {
    status: 200,
    body: { status: "ok", copy_project_id: "proj-2", state: "inactive" },
  },
  "POST /management/project/v1/archive": {
    status: 200,
    body: { status: "ok", project_id: "proj-1", state: "archived", triggers_stopped: true },
  },
};

async function renderAt(hash: string) {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch(BASE);
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

describe("P13-T09 project lifecycle UI", () => {
  it("copies as an inactive 副本 from the Projects list without Approve", async () => {
    const { host, root, calls } = await renderAt("#/projects");
    const copy = Array.from(host.querySelectorAll("button")).find((node) =>
      (node.textContent ?? "").includes("副本"),
    );
    expect(copy).toBeTruthy();
    const listActions = Array.from(host.querySelectorAll("button")).map(
      (node) => node.textContent ?? "",
    );
    expect(listActions.some((label) => /\bApprove\b|\bPublish\b/.test(label))).toBe(false);
    await act(async () => {
      copy?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await flush();
    expect(
      calls.some(
        (call) => call.method === "POST" && call.pathname === "/management/project/v1/copy",
      ),
    ).toBe(true);
    unmount(host, root);
  });

  it("offers archive / delete preview / restore / export on detail and not chat Approve", async () => {
    const { host, root, calls } = await renderAt("#/projects/proj-1");
    const panel = host.querySelector("[data-region='opc-project-lifecycle']");
    expect(panel).not.toBeNull();
    expect(panel?.textContent).toMatch(/Archive \(stop triggers\)/);
    expect(panel?.textContent).toMatch(/Preview delete impact/);
    expect(panel?.textContent).toMatch(/Apply logical delete/);
    expect(panel?.textContent).toMatch(/Record local restore point/);
    expect(panel?.textContent).toMatch(/Export without secrets/);
    expect(panel?.textContent).toMatch(/not a disaster backup/);
    const detailActions = Array.from(host.querySelectorAll("button")).map(
      (node) => node.textContent ?? "",
    );
    expect(detailActions.some((label) => /\bApprove\b|\bPublish\b/.test(label))).toBe(false);
    const archive = Array.from(host.querySelectorAll("button")).find((node) =>
      (node.textContent ?? "").includes("Archive (stop triggers)"),
    );
    await act(async () => {
      archive?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    await flush();
    expect(
      calls.some(
        (call) => call.method === "POST" && call.pathname === "/management/project/v1/archive",
      ),
    ).toBe(true);
    expect(calls.some((call) => call.pathname.includes("/task/project/v1/"))).toBe(false);
    unmount(host, root);
  });

  it("shows no copy or archive chrome when the daemon has no Project", async () => {
    rememberBearer("management", "test-management-bearer");
    installFetch({
      ...BASE,
      "GET /management/project/v1/list": {
        status: 200,
        body: { status: "ok", projects: [] },
      },
    });
    const { host, root } = renderAppAt("#/projects");
    await flush();
    expect(host.textContent).toMatch(/no Project/);
    expect(host.textContent).not.toMatch(/副本/);
    expect(host.textContent).not.toMatch(/Archive \(stop triggers\)/);
    expect(host.textContent).not.toMatch(/Preview delete impact/);
    expect(host.querySelector("[data-region='opc-project-lifecycle']")).toBeNull();
    unmount(host, root);
  });
});
