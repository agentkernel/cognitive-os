import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { LinuxLegacyApp as App } from "../../App";
import { isKnownRoute } from "../../data/normalize";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";

type RouteResponse = { status: number; body: unknown };
type RouteHandler = RouteResponse | ((call: { url: URL }) => RouteResponse);

interface RecordedCall {
  method: string;
  path: string;
  query: URLSearchParams;
}

function defaultRoute(path: string): RouteResponse {
  if (path === "/personal/health") {
    return { status: 200, body: { status: "ok" } };
  }
  if (path === "/personal/status") {
    return { status: 200, body: { status: "ok", overall: "ready", components: [] } };
  }
  if (path === "/management/alerts") {
    return { status: 200, body: { status: "ok", alerts: [] } };
  }
  return { status: 404, body: { status: "error", code: "NOT_FOUND", message: "not found" } };
}

function installFetch(routes: Record<string, RouteHandler>): RecordedCall[] {
  const calls: RecordedCall[] = [];
  vi.stubGlobal(
    "fetch",
    vi.fn(async (input: unknown, init?: RequestInit) => {
      const url = new URL(String(input), "http://localhost");
      const method = (init?.method ?? "GET").toUpperCase();
      calls.push({ method, path: url.pathname, query: url.searchParams });
      const handler = routes[`${method} ${url.pathname}`];
      const resolved =
        typeof handler === "function"
          ? handler({ url })
          : (handler ?? defaultRoute(url.pathname));
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

function agentRoutes(overrides: Record<string, RouteHandler> = {}): Record<string, RouteHandler> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", components: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/agent-bindings": {
      status: 200,
      body: {
        status: "ok",
        bindings: [
          {
            agent: "pi",
            account_id: "acct-1",
            model_id: "deepseek-chat",
            revision: 4,
            status: "active",
          },
          {
            agent: "dsh",
            account_id: "acct-1",
            model_id: "deepseek-chat",
            revision: 2,
            status: "active",
          },
        ],
      },
    },
    "GET /management/providers/accounts": {
      status: 200,
      body: {
        status: "ok",
        accounts: [
          {
            id: "acct-1",
            display_name: "main",
            provider_kind: "deepseek",
            status: "active",
            secret_ref: "ss://provider/acct-1",
          },
        ],
      },
    },
    "GET /personal/dsh/runtime": {
      status: 200,
      body: {
        schema_version: 1,
        surface: "personal-dsh-runtime",
        state: "ACTIVE",
        session_count: 1,
        process_id: 4812,
        process_alive: true,
        candidate_only: true,
        dsh_response_is_not_task_completion: true,
        sessions: [
          {
            session_id: "sess-1",
            state: "Active",
            fencing_epoch: 3,
            task_ref: "task://personal/a3f9",
          },
        ],
      },
    },
    "GET /management/resource/v1/list": ({ url }) => {
      const family = url.searchParams.get("family");
      if (family === "runtime") {
        return {
          status: 200,
          body: {
            kind: "resource.manager.list",
            family: "runtime",
            authority_source: "projection-only",
            resources: [],
          },
        };
      }
      return { status: 200, body: { status: "ok", resources: [] } };
    },
    "GET /management/resource/v1/inspect": {
      status: 404,
      body: {
        status: "error",
        code: "RESOURCE_MANAGER_NOT_FOUND",
        message: "context and runtime have no authority-backed Resource Manager rows",
      },
    },
    ...overrides,
  };
}

async function renderAgents(hash = "#/agents", overrides: Record<string, RouteHandler> = {}) {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch(agentRoutes(overrides));
  const view = renderAppAt(hash);
  await flush();
  return { ...view, calls };
}

describe("Agents inventory and dossier (W6)", () => {
  afterEach(() => {
    clearSession();
    appProjections.clear();
    window.location.hash = "";
    vi.unstubAllGlobals();
  });

  it("renders pi and dsh from bindings plus dsh current work, and never draws lifecycle buttons", async () => {
    const { host, root, calls } = await renderAgents();
    expect(host.querySelector("main h2")?.textContent).toBe("Agents");
    expect(host.textContent).toContain("pi");
    expect(host.textContent).toContain("dsh");
    expect(host.textContent).toContain("callable");
    expect(host.textContent).toContain("task://personal/a3f9");
    expect(host.textContent).toMatch(/projection-only/);
    expect(host.textContent).not.toMatch(/Under reconstruction/);
    const labels = [...host.querySelectorAll("button")].map((button) =>
      (button.textContent ?? "").trim(),
    );
    for (const verb of ["Pause", "Resume", "Stop", "Restart", "Quarantine"]) {
      expect(labels).not.toContain(verb);
    }
    const paths = calls.map((call) => `${call.method} ${call.path}`);
    expect(paths).toContain("GET /management/agent-bindings");
    expect(paths).toContain("GET /personal/dsh/runtime");
    expect(paths).toContain("GET /management/resource/v1/list");
    expect(paths.some((path) => path.startsWith("POST /"))).toBe(false);
    expect(isKnownRoute("POST", "/management/agent/transition")).toBe(false);
    unmount(host, root);
  });

  it("opens the dsh dossier with identity unknowns, class-C CLI paths, and the runtime snapshot", async () => {
    rememberBearer("task", "test-task-bearer");
    const { host, root } = await renderAgents("#/agents/dsh");
    expect(host.textContent).toMatch(/Lifecycle control runs through/);
    expect(host.textContent).toMatch(/RESOURCE_MANAGER_NOT_FOUND|no authority-backed/);
    expect(host.textContent).toMatch(/process liveness is not task completion/i);
    expect(host.textContent).toMatch(/cognitive agent-pause/);
    expect(host.textContent).toMatch(/candidate_only/);
    expect(host.textContent).toMatch(/never Task completion|not task completion/i);
    expect(host.textContent).toContain("Installed ≠ permitted");
    expect([...host.querySelectorAll("button")].map((button) => button.textContent)).not.toContain(
      "Pause",
    );
    expect(host.querySelector("#section-runtime")).not.toBeNull();
    unmount(host, root);
  });

  it("renders a designed object-404 for an actor this HTTP surface cannot name", async () => {
    const { host, root } = await renderAgents("#/agents/nope");
    expect(host.textContent).toMatch(/No such agent/);
    expect(host.textContent).toContain("nope");
    expect(host.querySelector("#section-overview")).toBeNull();
    unmount(host, root);
  });

  it("keeps pi current work unavailable and does not infer it from dsh process liveness", async () => {
    const { host, root } = await renderAgents("#/agents/pi");
    expect(host.textContent).toMatch(/not observable over HTTP \(BD-2\/BD-3\)/);
    expect(host.querySelector("#section-runtime")).toBeNull();
    expect(host.textContent).toMatch(/no agent lifecycle route exists over HTTP/);
    unmount(host, root);
  });

  it("names an unbound agent as unable to call a model", async () => {
    const { host, root } = await renderAgents("#/agents", {
      "GET /management/agent-bindings": { status: 200, body: { status: "ok", bindings: [] } },
    });
    expect(host.textContent).toMatch(/no binding — this agent cannot call a model/);
    unmount(host, root);
  });

  it("keeps a labelled inventory, section landmarks on the dossier, and no pause control", async () => {
    const { host, root } = await renderAgents();
    expect(host.querySelector("table caption")?.textContent).toMatch(/HTTP-addressable actors/);
    expect(host.querySelector('a[href="#/agents/pi"]')).not.toBeNull();
    unmount(host, root);
    const dossier = await renderAgents("#/agents/dsh");
    expect(dossier.host.querySelector("nav[aria-label='Agent dossier sections']")).not.toBeNull();
    expect(dossier.host.querySelector("#section-overview")).not.toBeNull();
    const focused = [...dossier.host.querySelectorAll("button, a")].filter(
      (node) => (node as HTMLElement).tabIndex >= 0,
    );
    expect(focused.length).toBeGreaterThan(4);
    unmount(dossier.host, dossier.root);
  });
});
