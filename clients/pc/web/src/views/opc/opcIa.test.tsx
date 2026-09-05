import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { isKnownRoute } from "../../data/normalize";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";
import { SPACE_CHORDS } from "../../shell/keyboard";
import { PRIMARY_NAV } from "../../shell/PrimaryNav";
import { NO_PROJECT_EMPTY, TODAY_EMPTY_ONLY_CREATE } from "./ProjectAuthorityPanel";

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
      if (
        node.closest("[data-region='opc-hitl-actions']") ||
        node.closest("[data-region='opc-vault-ingest']") ||
        node.closest("[data-region='opc-standing-policies']") ||
        node.closest("[data-region='opc-close-background']") ||
        node.closest("[data-region='opc-rail-write']")
      ) {
        continue;
      }
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
    "GET /management/project/v1/vault.index": {
      status: 200,
      body: { status: "ok", is_authority: false, entries: [] },
    },
    "GET /management/project/v1/vault.conflicts": {
      status: 200,
      body: { status: "ok", conflicts: [] },
    },
    "GET /management/project/v1/standing-policies": {
      status: 200,
      body: { status: "ok", policies: [] },
    },
    "GET /management/providers/accounts": {
      status: 200,
      body: { status: "ok", accounts: [] },
    },
    "GET /management/usage": { status: 200, body: { status: "ok", events: [] } },
    "GET /management/resource/v1/list": {
      status: 200,
      body: { status: "ok", family: "memory", resources: [] },
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
const READY_LIST: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [{ project_id: "proj-1", state: "active", title_summary: "unknown", cost: "unknown" }],
  },
};

afterEach(() => {
  clearSession();
  appProjections.clear();
  window.location.hash = "";
  vi.unstubAllGlobals();
});

describe("P11-T13 OPC IA chrome", () => {
  it("keeps L1 as Today / Projects / Knowledge and never Team or Inbox", () => {
    expect(PRIMARY_NAV.map(([, label]) => label)).toEqual([
      "Today",
      "Projects",
      "Knowledge",
      "Settings",
    ]);
    const { host, root } = renderAppAt("#/session");
    const nav = host.querySelector('nav[aria-label="Primary"]');
    expect(nav?.textContent).toContain("Today");
    expect(nav?.textContent).toContain("Projects");
    expect(nav?.textContent).toContain("Knowledge");
    expect(nav?.textContent).toContain("Settings");
    expect(nav?.textContent).not.toMatch(/Team|Inbox/);
    expect(host.querySelector("nav[aria-label='Primary'] a[href='#/settings']")?.textContent).toBe(
      "Settings",
    );
    expect(host.querySelector("[data-rail='assistant']")?.textContent).toMatch(/candidate-only/i);
    expect(
      [...(host.querySelector("[data-rail='assistant']")?.querySelectorAll("button, a.cp-button") ?? [])].map(
        (node) => (node.textContent ?? "").trim(),
      ),
    ).not.toContain("Approve");
    const links = [...host.querySelectorAll("nav[aria-label='Primary'] a")];
    expect(links).toHaveLength(4);
    for (const link of links) {
      const href = link.getAttribute("href") ?? "";
      expect(href.startsWith("#")).toBe(true);
      expect(href).not.toMatch(/^\/ui\//);
    }
    unmount(host, root);
  });

  it("maps g-chords onto Today / Projects / Knowledge / Settings without stealing Work", () => {
    expect(SPACE_CHORDS.t).toBe("/");
    expect(SPACE_CHORDS.p).toBe("/projects");
    expect(SPACE_CHORDS.n).toBe("/knowledge");
    expect(SPACE_CHORDS.s).toBe("/settings");
    expect(SPACE_CHORDS.h).toBeUndefined();
    expect(SPACE_CHORDS.w).toBeUndefined();
    expect(SPACE_CHORDS.v).toBeUndefined();
  });

  it("whitelists the Project list route used by Dual Track", () => {
    expect(isKnownRoute("GET", "/management/project/v1/list")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/list")).toBe(false);
    expect(isKnownRoute("GET", "/management/project/v1/detail")).toBe(true);
    expect(isKnownRoute("GET", "/management/project/v1/axis")).toBe(true);
    expect(isKnownRoute("GET", "/management/project/v1/roster")).toBe(true);
    expect(isKnownRoute("GET", "/management/project/v1/pending-previews")).toBe(true);
    expect(isKnownRoute("GET", "/management/project/v1/vault.index")).toBe(true);
    expect(isKnownRoute("GET", "/management/project/v1/vault.conflicts")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/vault.import")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/vault.index.rebuild")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/vault.apply-authority")).toBe(false);
    expect(isKnownRoute("GET", "/management/project/v1/standing-policies")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/standing-policy.revoke")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/standing-policy.create")).toBe(false);
    expect(isKnownRoute("GET", "/management/host/v1/status")).toBe(true);
    expect(isKnownRoute("POST", "/management/host/v1/close.request")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/confirm")).toBe(true);
    expect(isKnownRoute("GET", "/management/project/v1/preview-detail")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/preview.reject")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/preview.narrow")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/draft.create")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/draft.apply")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/assistant.turn")).toBe(true);
    expect(isKnownRoute("POST", "/task/project/v1/assistant.turn")).toBe(false);
    expect(isKnownRoute("POST", "/management/project/v1/preview.request")).toBe(true);
    expect(isKnownRoute("GET", "/management/project/v1/employee.catalog")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/roster.register")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/employee.seat.request")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/employee.seat.confirm")).toBe(true);
  });
});

describe("P11-T13 Dual Track honesty (zero fake buttons)", () => {
  it("renders Today as empty, not a fake OPC chrome, when the daemon has no Project", async () => {
    const { host, root, calls } = await renderOpc("#/", EMPTY_LIST);
    expect(host.querySelector("[data-page='opc-today']")).not.toBeNull();
    expect(host.querySelector("main h2")?.textContent).toBe("Today");
    expect(host.textContent).toContain(TODAY_EMPTY_ONLY_CREATE);
    expect(host.querySelector("[data-page='opc-today'] .cp-region")).toBeNull();
    expect(host.querySelector("[data-region='opc-hitl']")).toBeNull();
    expect(host.querySelector("[data-rail='assistant']")).toBeNull();
    expect(host.querySelector("a[href='#/projects/new']")?.textContent).toMatch(/Start create/);
    expect(fakeActionLabels(host)).toEqual([]);
    expect(calls.some((call) => call.pathname === "/management/project/v1/pending-previews")).toBe(
      false,
    );
    expect(calls.some((call) => call.pathname === "/management/project/v1/vault.index")).toBe(false);
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    unmount(host, root);
  });

  it("keeps empty, denied, disconnected, stub, and unexpected as distinct on Projects", async () => {
    const empty = await renderOpc("#/projects", EMPTY_LIST);
    expect(empty.host.textContent).toContain("no Project");
    expect(empty.host.textContent).not.toContain("session denied");
    expect(fakeActionLabels(empty.host)).toEqual([]);
    unmount(empty.host, empty.root);

    const denied = await renderOpc("#/projects", {
      status: 403,
      body: { status: "error", error: { code: "LOCAL_ORIGIN_HEADER_REJECTED", message: "denied" } },
    });
    expect(denied.host.textContent).toMatch(/session denied/i);
    expect(denied.host.textContent).not.toContain(NO_PROJECT_EMPTY);
    expect(fakeActionLabels(denied.host)).toEqual([]);
    unmount(denied.host, denied.root);
  });

  it("does not convert a 503 into an empty Project list", async () => {
    const { host, root } = await renderOpc("#/projects", {
      status: 503,
      body: { status: "error", code: "HTTP_503", message: "unavailable" },
    });
    expect(host.textContent).toMatch(/unexpected|unavailable|HTTP_503|HTTP 503/i);
    expect(host.textContent).not.toContain(NO_PROJECT_EMPTY);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("does not convert the daemon 200-stub into an empty Project list", async () => {
    const { host, root } = await renderOpc("#/", {
      status: 200,
      body: {
        status: "ok",
        channel: "management",
        note: "authenticated personal front door; business routes deferred",
      },
    });
    expect(host.querySelector("[data-page='opc-today']")).not.toBeNull();
    expect(host.textContent).toMatch(/unavailable|not-run|STUB_ROUTE|Not available/i);
    expect(host.textContent).not.toContain(NO_PROJECT_EMPTY);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("lists daemon Project rows without minting Activate or Approve", async () => {
    const { host, root } = await renderOpc("#/projects", READY_LIST);
    expect(host.querySelector("[data-row-key='proj-1']")).not.toBeNull();
    expect(host.textContent).toContain("unknown");
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("gates Knowledge on Project authority and keeps Settings as a hub of real routes", async () => {
    const knowledge = await renderOpc("#/knowledge", EMPTY_LIST);
    expect(knowledge.host.querySelector("[data-page='opc-knowledge']")).not.toBeNull();
    expect(knowledge.host.textContent).toContain(NO_PROJECT_EMPTY);
    expect(fakeActionLabels(knowledge.host)).toEqual([]);
    unmount(knowledge.host, knowledge.root);

    const settings = await renderOpc("#/settings", EMPTY_LIST);
    expect(settings.host.querySelector("[data-page='opc-settings']")).not.toBeNull();
    expect(settings.host.querySelector("a[href='#/home']")).toBeNull();
    expect(settings.host.querySelector("a[href='#/work']")).toBeNull();
    expect(settings.host.querySelector("a[href='#/agents']")).toBeNull();
    expect(settings.host.querySelector("a[href='#/providers']")).toBeNull();
    const settingsLinks = [...settings.host.querySelectorAll("[data-page='opc-settings'] a")].map(
      (node) => (node.textContent ?? "").trim(),
    );
    expect(settingsLinks).not.toContain("Team");
    expect(settingsLinks).not.toContain("Inbox");
    expect(fakeActionLabels(settings.host)).toEqual([]);
    unmount(settings.host, settings.root);
  });

  it("shows the Today session gate without painting fake OPC chrome", () => {
    const { host, root } = renderAppAt("#/");
    expect(host.querySelector("[data-page='session-gate']")).not.toBeNull();
    expect(host.querySelector("main h2")?.textContent).toBe("Today");
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });
});

describe("P11-T13 disconnected is not empty", () => {
  it("names daemon unreachable on Today when fetch throws", async () => {
    rememberBearer("management", "test-management-bearer");
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => {
        throw new TypeError("Failed to fetch");
      }),
    );
    const { host, root } = renderAppAt("#/");
    await flush();
    expect(host.textContent).toMatch(/unreachable|Failed to fetch/i);
    expect(host.textContent).not.toContain(NO_PROJECT_EMPTY);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });
});

describe("P11-T13 Dual Track daemon reads (fail-closed)", () => {
  it("does not claim Vite as the product origin on L1 or Settings", async () => {
    for (const hash of ["#/", "#/projects", "#/knowledge", "#/settings"]) {
      const { host, root } = await renderOpc(hash, EMPTY_LIST);
      expect(host.textContent).toMatch(/daemon-served hash \/ui\//);
      expect(host.textContent).not.toMatch(/vite preview|vite dev server|localhost:5173/i);
      expect(fakeActionLabels(host)).toEqual([]);
      unmount(host, root);
    }
  });

  it("announces pending HITL without Confirm when a Project exists", async () => {
    const { host, root, calls } = await renderOpc("#/", READY_LIST, {
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
    expect(host.querySelector("[data-packet='prev-1']")).not.toBeNull();
    expect(host.querySelector("[data-row-key='prev-1']")).not.toBeNull();
    expect(host.textContent).toContain("activation");
    expect(host.textContent).not.toContain("must-not-render");
    expect(host.textContent).toMatch(/announce only/i);
    expect(host.querySelector("a[href='#/projects/proj-1?preview=prev-1']")?.textContent).toMatch(
      /Open this decision on the canvas/i,
    );
    expect(host.querySelector("a[href*='#/hitl']")).toBeNull();
    expect(host.querySelector("[data-region='opc-rail-hitl']")?.textContent).toMatch(
      /pending ApprovalPreview/i,
    );
    expect(
      host.querySelector("[data-rail='assistant'] a[href='#/projects?preview=prev-1']"),
    ).not.toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    expect(
      calls.some(
        (call) => call.path === "/management/project/v1/pending-previews?subject_ref=proj-1",
      ),
    ).toBe(true);
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    unmount(host, root);
  });

  it("does not invent HITL rows when pending-previews is 403", async () => {
    const { host, root } = await renderOpc("#/", READY_LIST, {
      "GET /management/project/v1/pending-previews": {
        status: 403,
        body: { status: "error", error: { code: "LOCAL_ORIGIN_HEADER_REJECTED", message: "denied" } },
      },
    });
    expect(host.querySelector("[data-region='opc-hitl']")?.textContent).toMatch(/session denied/i);
    expect(host.querySelector("[data-row-key='prev-1']")).toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("does not open Vault or Memory without a Project id", async () => {
    const { host, root, calls } = await renderOpc("#/knowledge", EMPTY_LIST);
    expect(host.textContent).toContain(NO_PROJECT_EMPTY);
    expect(host.querySelector("[data-region='opc-vault']")).toBeNull();
    expect(host.querySelector("[data-region='opc-memory']")).toBeNull();
    expect(calls.some((call) => call.pathname === "/management/project/v1/vault.index")).toBe(false);
    expect(
      calls.some(
        (call) =>
          call.pathname === "/management/resource/v1/list" && call.path.includes("family=memory"),
      ),
    ).toBe(false);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("reads Vault index and Memory envelope without posting ingest on load when a Project exists", async () => {
    const { host, root, calls } = await renderOpc("#/knowledge", READY_LIST, {
      "GET /management/project/v1/vault.index": {
        status: 200,
        body: {
          status: "ok",
          is_authority: false,
          entries: [{ entry_id: "ent-1", document_id: "doc-1", layer: "summaries", excerpt: "note" }],
        },
      },
      "GET /management/resource/v1/list": {
        status: 200,
        body: { status: "ok", family: "memory", resources: [{ id: "mem-1", family: "memory" }] },
      },
    });
    expect(host.querySelector("[data-row-key='ent-1']")).not.toBeNull();
    expect(host.querySelector("[data-row-key='mem-1']")).not.toBeNull();
    expect(host.textContent).toContain("mem-1");
    expect(fakeActionLabels(host)).toEqual([]);
    expect(
      calls.some(
        (call) =>
          call.path ===
          "/management/project/v1/vault.index?project_id=proj-1&caller_project_id=proj-1",
      ),
    ).toBe(true);
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    unmount(host, root);
  });

  it("does not invent Vault rows when vault.index is 403", async () => {
    const { host, root } = await renderOpc("#/knowledge", READY_LIST, {
      "GET /management/project/v1/vault.index": {
        status: 403,
        body: { status: "error", error: { code: "LOCAL_ORIGIN_HEADER_REJECTED", message: "denied" } },
      },
    });
    expect(host.querySelector("[data-region='opc-vault']")?.textContent).toMatch(/session denied/i);
    expect(host.querySelector("[data-row-key='ent-1']")).toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("lists StandingApprovalPolicy on Settings without mint, Team, Inbox, or member budget", async () => {
    const { host, root, calls } = await renderOpc("#/settings", EMPTY_LIST, {
      "GET /management/project/v1/standing-policies": {
        status: 200,
        body: {
          status: "ok",
          policies: [
            {
              policy_id: "pol-1",
              subject_class: "grant-expansion",
              subject_ref: "proj-1",
              expires_at: 1,
              active: true,
            },
          ],
        },
      },
    });
    expect(host.querySelector("[data-row-key='pol-1']")).not.toBeNull();
    expect(host.querySelector("[data-region='opc-standing-policies'] input")).toBeNull();
    expect(host.textContent).toMatch(/2\.1 \/ Deferred/);
    const settingsLinks = [...host.querySelectorAll("[data-page='opc-settings'] a")].map(
      (node) => (node.textContent ?? "").trim(),
    );
    expect(settingsLinks).not.toContain("Team");
    expect(settingsLinks).not.toContain("Inbox");
    expect(fakeActionLabels(host)).toEqual([]);
    expect(
      calls.some((call) => call.pathname === "/management/project/v1/standing-policies"),
    ).toBe(true);
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    unmount(host, root);
  });

  it("keeps Settings Advanced collapsed so Linux 1.0 is not default chrome", async () => {
    const { host, root } = await renderOpc("#/settings", EMPTY_LIST);
    const advanced = host.querySelector(
      "[data-region='opc-settings-advanced']",
    ) as HTMLDetailsElement | null;
    expect(advanced).not.toBeNull();
    expect(advanced?.open).toBe(false);
    expect(advanced?.querySelector("a[href='#/home']")).toBeNull();
    expect(advanced?.querySelector("a[href='#/work']")).toBeNull();
    expect(advanced?.querySelector("a[href='#/agents']")).toBeNull();
    expect(advanced?.querySelector("a[href='#/providers']")).toBeNull();
    expect(host.querySelector("nav[aria-label='Primary'] a[href='#/work']")).toBeNull();
    expect(host.querySelector("nav[aria-label='Primary']")?.textContent).not.toMatch(/Team|Inbox/);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("lands Today deep-links on the Projects canvas without making Inbox or #/hitl L1", async () => {
    const { host, root, calls } = await renderOpc("#/projects?preview=prev-1", READY_LIST, {
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
            },
          ],
        },
      },
    });
    expect(host.querySelector("[data-page='opc-projects']")).not.toBeNull();
    expect(host.querySelector("[data-row-key='proj-1']")).not.toBeNull();
    expect(host.querySelector("[data-row-key='prev-1']")?.getAttribute("data-canvas-focus")).toBe(
      "true",
    );
    expect(host.querySelector("nav[aria-label='Primary']")?.textContent).not.toMatch(/Team|Inbox/);
    expect(host.querySelector("a[href*='#/hitl']")).toBeNull();
    expect(host.querySelector("[data-region='opc-hitl-actions']")).not.toBeNull();
    expect(host.querySelector("[data-hitl-blocked='unknown']")).not.toBeNull();
    expect(
      calls.some((call) => call.pathname === "/management/project/v1/preview-detail"),
    ).toBe(true);
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("does not populate Projects from a preview query when the daemon list is empty", async () => {
    const { host, root, calls } = await renderOpc("#/projects?preview=prev-1", EMPTY_LIST);
    expect(host.textContent).toContain(NO_PROJECT_EMPTY);
    expect(host.querySelector("[data-row-key='proj-1']")).toBeNull();
    expect(host.querySelector("[data-region='opc-hitl']")).toBeNull();
    expect(calls.some((call) => call.pathname === "/management/project/v1/pending-previews")).toBe(
      false,
    );
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("treats #/hitl, #/inbox, and #/team as missing routes, not L1", () => {
    for (const hash of ["#/hitl/prev-1", "#/inbox", "#/team"]) {
      const { host, root } = renderAppAt(hash);
      expect(host.textContent).toContain("No such route");
      expect(host.querySelector("nav[aria-label='Primary'] a[aria-current='page']")).toBeNull();
      expect(host.querySelector("nav[aria-label='Primary']")?.textContent).not.toMatch(/Team|Inbox/);
      expect(fakeActionLabels(host)).toEqual([]);
      unmount(host, root);
    }
  });
});
