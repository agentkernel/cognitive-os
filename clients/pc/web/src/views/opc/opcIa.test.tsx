import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { isKnownRoute } from "../../data/normalize";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";
import { SPACE_CHORDS } from "../../shell/keyboard";
import { PRIMARY_NAV } from "../../shell/PrimaryNav";
import { NO_PROJECT_EMPTY } from "./ProjectAuthorityPanel";

type RouteResponse = { status: number; body: unknown };

function installFetch(routes: Record<string, RouteResponse>): void {
  vi.stubGlobal(
    "fetch",
    vi.fn(async (input: unknown, init?: RequestInit) => {
      const url = new URL(String(input), "http://localhost");
      const method = (init?.method ?? "GET").toUpperCase();
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

const FAKE_ACTION = /approve|create project|activate|new project|team|inbox/i;

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

function opcRoutes(list: RouteResponse): Record<string, RouteResponse> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/project/v1/list": list,
  };
}

async function renderOpc(hash: string, list: RouteResponse) {
  rememberBearer("management", "test-management-bearer");
  installFetch(opcRoutes(list));
  const view = renderAppAt(hash);
  await flush();
  return view;
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
    expect(PRIMARY_NAV.map(([, label]) => label)).toEqual(["Today", "Projects", "Knowledge"]);
    const { host, root } = renderAppAt("#/session");
    const nav = host.querySelector('nav[aria-label="Primary"]');
    expect(nav?.textContent).toContain("Today");
    expect(nav?.textContent).toContain("Projects");
    expect(nav?.textContent).toContain("Knowledge");
    expect(nav?.textContent).not.toMatch(/Team|Inbox/);
    expect(host.querySelector(".cp-side-foot a[href='#/settings']")?.textContent).toBe("Settings");
    expect(host.querySelector("[data-rail='assistant']")?.textContent).toMatch(/candidate-only/i);
    expect(
      [...(host.querySelector("[data-rail='assistant']")?.querySelectorAll("button, a.cp-button") ?? [])].map(
        (node) => (node.textContent ?? "").trim(),
      ),
    ).not.toContain("Approve");
    const links = [...host.querySelectorAll("nav[aria-label='Primary'] a")];
    expect(links).toHaveLength(3);
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
    expect(SPACE_CHORDS.h).toBe("/home");
    expect(SPACE_CHORDS.w).toBe("/work");
    expect(SPACE_CHORDS.v).toBe("/providers");
  });

  it("whitelists the Project list route used by Dual Track", () => {
    expect(isKnownRoute("GET", "/management/project/v1/list")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/list")).toBe(false);
  });
});

describe("P11-T13 Dual Track honesty (zero fake buttons)", () => {
  it("renders Today as empty, not a fake OPC chrome, when the daemon has no Project", async () => {
    const { host, root } = await renderOpc("#/", EMPTY_LIST);
    expect(host.querySelector("[data-page='opc-today']")).not.toBeNull();
    expect(host.querySelector("main h2")?.textContent).toBe("Today");
    expect(host.textContent).toContain(NO_PROJECT_EMPTY);
    expect(host.querySelector("[data-page='opc-today'] .cp-region")).toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
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
    expect(settings.host.querySelector("a[href='#/home']")?.textContent).toBe("Linux 1.0 Home");
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
