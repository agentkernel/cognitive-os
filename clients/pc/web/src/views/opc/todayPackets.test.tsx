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

const FAKE_ACTION = /approve|create project|activate|new project|team|inbox|confirm|ingest|apply authority/i;

function fakeActionLabels(host: HTMLElement): string[] {
  const scopes = [
    host.querySelector("#main"),
    host.querySelector("[data-rail='assistant']"),
    host.querySelector("nav[aria-label='Primary']"),
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

function opcRoutes(
  list: RouteResponse,
  extras: Record<string, RouteResponse> = {},
): Record<string, RouteResponse> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/project/v1/list": list,
    "GET /management/project/v1/pending-previews": {
      status: 200,
      body: { status: "ok", previews: [] },
    },
    ...extras,
  };
}

async function renderOpc(
  hash: string,
  list: RouteResponse,
  extras: Record<string, RouteResponse> = {},
) {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch(opcRoutes(list, extras));
  const view = renderAppAt(hash);
  await flush();
  return { ...view, calls };
}

const EMPTY_LIST: RouteResponse = { status: 200, body: { status: "ok", projects: [] } };
const CREATING_LIST: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [
      { project_id: "proj-draft", state: "creating", title_summary: "unknown", cost: "unknown" },
    ],
  },
};
const LIVE_LIST: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [{ project_id: "proj-1", state: "active", title_summary: "unknown", cost: "unknown" }],
  },
};
const MIXED_LIST: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [
      { project_id: "proj-draft", state: "creating", title_summary: "unknown", cost: "unknown" },
      { project_id: "proj-1", state: "attention", title_summary: "unknown", cost: "unknown" },
    ],
  },
};

afterEach(() => {
  clearSession();
  appProjections.clear();
  window.location.hash = "";
  vi.unstubAllGlobals();
});

describe("P12-T05 Today decision packets (Dual Track)", () => {
  it("keeps empty home as only-create, not a packet and not T13 empty chrome pretending packets are accepted", async () => {
    const { host, root, calls } = await renderOpc("#/", EMPTY_LIST);
    expect(host.querySelector("[data-surface='today']")).toBeNull();
    expect(host.querySelector("[data-surface='today-incomplete']")).toBeNull();
    expect(host.querySelector("[data-packet]")).toBeNull();
    expect(host.textContent).toContain(TODAY_EMPTY_ONLY_CREATE);
    expect(host.textContent).not.toMatch(/weekly report/i);
    expect(host.querySelector("a[href='#/projects/new']")?.textContent).toMatch(/Start create/);
    expect(host.querySelector("[data-rail='assistant']")).toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    expect(calls.some((call) => call.pathname === "/management/project/v1/pending-previews")).toBe(
      false,
    );
    unmount(host, root);
  });

  it("shows creating-only Today as continue-create and does not fetch pending-previews", async () => {
    const { host, root, calls } = await renderOpc("#/", CREATING_LIST);
    expect(host.querySelector("[data-surface='today-incomplete']")).not.toBeNull();
    expect(host.querySelector("[data-surface='today']")).toBeNull();
    expect(host.querySelector("[data-packet]")).toBeNull();
    expect(host.querySelector("[data-region='opc-hitl']")).toBeNull();
    expect(host.textContent).toContain(TODAY_INCOMPLETE_ONLY_CREATE);
    expect(host.querySelector("[data-row-key='proj-draft']")).not.toBeNull();
    expect(host.querySelector("a[href='#/projects/new']")?.textContent).toMatch(/Continue create/);
    expect(host.querySelector("[data-rail='assistant']")).toBeNull();
    expect(host.textContent).not.toMatch(/weekly report/i);
    expect(fakeActionLabels(host)).toEqual([]);
    expect(calls.some((call) => call.pathname === "/management/project/v1/pending-previews")).toBe(
      false,
    );
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    unmount(host, root);
  });

  it("does not invent daily packets when a live Project has no pending-previews", async () => {
    const { host, root, calls } = await renderOpc("#/", LIVE_LIST);
    expect(host.querySelector("[data-surface='today']")).not.toBeNull();
    expect(host.querySelector("[data-packet]")).toBeNull();
    expect(host.querySelector("[data-region='opc-hitl']")?.textContent).toMatch(
      /no pending ApprovalPreview/i,
    );
    expect(host.textContent).toMatch(/not a KPI/i);
    expect(host.textContent).not.toMatch(/weekly report/i);
    expect(fakeActionLabels(host)).toEqual([]);
    expect(
      calls.some(
        (call) => call.path === "/management/project/v1/pending-previews?subject_ref=proj-1",
      ),
    ).toBe(true);
    unmount(host, root);
  });

  it("renders live packets from pending-previews and deep-links to the HITL canvas, never chat Approve", async () => {
    const { host, root, calls } = await renderOpc("#/", LIVE_LIST, {
      "GET /management/project/v1/pending-previews": {
        status: 200,
        body: {
          status: "ok",
          previews: [
            {
              preview_id: "prev-1",
              subject_kind: "activation",
              subject_ref: "proj-1",
              status: "pending",
              preview_digest: "must-not-render",
            },
          ],
        },
      },
    });
    expect(host.querySelector("[data-surface='today']")).not.toBeNull();
    expect(host.querySelector("[data-packet='prev-1']")).not.toBeNull();
    expect(host.textContent).not.toContain("must-not-render");
    expect(host.querySelector("a[href='#/projects/proj-1?preview=prev-1']")?.textContent).toMatch(
      /Open this decision on the canvas/i,
    );
    expect(host.querySelector("a[href*='#/hitl']")).toBeNull();
    expect(host.querySelector("[data-rail='assistant']")).not.toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    unmount(host, root);
  });

  it("keeps creating rows as continue-create when mixed with a live Project, and packets use the live subject_ref", async () => {
    const { host, root, calls } = await renderOpc("#/", MIXED_LIST, {
      "GET /management/project/v1/pending-previews": {
        status: 200,
        body: { status: "ok", previews: [] },
      },
    });
    expect(host.querySelector("[data-surface='today']")).not.toBeNull();
    expect(host.querySelector("[data-surface='today-incomplete']")).toBeNull();
    expect(host.textContent).toMatch(/Continue create/i);
    expect(host.querySelector("a[href='#/projects/new']")).not.toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    expect(
      calls.some(
        (call) => call.path === "/management/project/v1/pending-previews?subject_ref=proj-1",
      ),
    ).toBe(true);
    expect(
      calls.some((call) => call.path.includes("subject_ref=proj-draft")),
    ).toBe(false);
    unmount(host, root);
  });
});
