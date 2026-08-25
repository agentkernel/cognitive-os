import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
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

function listBody(
  family: string,
  resources: unknown[],
  extra: Record<string, unknown> = {},
): RouteResponse {
  return {
    status: 200,
    body: {
      kind: "resource.manager.list",
      family,
      truncated: false,
      resources,
      ...extra,
    },
  };
}

function resourceRoutes(overrides: Record<string, RouteHandler> = {}): Record<string, RouteHandler> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", components: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/resource/v1/list": ({ url }) => {
      const family = url.searchParams.get("family");
      if (family === "memory") {
        return listBody("memory", [{ id: "mem-1", family: "memory", health: "admitted" }], {
          authority_source: "sqlite-authority-memory-objects",
        });
      }
      if (family === "skill") {
        return listBody(
          "skill",
          [
            { id: "bind-1", family: "skill", health: "bound" },
            { id: "bind-2", family: "skill", health: "revoked" },
          ],
          { authority_source: "sqlite-authority-skill-bindings" },
        );
      }
      if (family === "tool") {
        return listBody(
          "tool",
          [
            { id: "native.workspace.read", family: "tool", health: "enabled" },
            { id: "native.process.check", family: "tool", health: "quarantined" },
          ],
          { authority_source: "daemon-native-tool-registry" },
        );
      }
      if (family === "context") {
        return listBody("context", [], { authority_source: "projection-only" });
      }
      return { status: 200, body: { status: "ok", resources: [] } };
    },
    ...overrides,
  };
}

async function renderResources(overrides: Record<string, RouteHandler> = {}) {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch(resourceRoutes(overrides));
  const view = renderAppAt("#/resources");
  await flush();
  return { ...view, calls };
}

describe("Resources family hub (W7)", () => {
  afterEach(() => {
    clearSession();
    appProjections.clear();
    window.location.hash = "";
    vi.unstubAllGlobals();
  });

  it("renders four family rows from real list reads and is not the Wave 7 placeholder", async () => {
    const { host, root, calls } = await renderResources();
    expect(host.querySelector("main h2")?.textContent).toBe("Resources");
    expect(host.textContent).not.toMatch(/Under reconstruction/);
    expect(host.textContent).not.toMatch(/Lands in Wave 7/);
    const families = [...host.querySelectorAll("[data-family]")].map((row) =>
      row.getAttribute("data-family"),
    );
    expect(families).toEqual(["memory", "skill", "tool", "context"]);
    expect(host.textContent).toContain("1 admitted");
    expect(host.textContent).toContain("tombstones are not in this list");
    expect(host.textContent).toContain("1 bound");
    expect(host.textContent).toContain("list is skill bindings, not packages");
    expect(host.textContent).toContain("1 quarantined");
    expect(host.textContent).toContain("envelope limit 64");
    expect(host.textContent).toMatch(/no standalone HTTP browser/);
    expect(host.textContent).toMatch(/not a card wall/);

    const labels = [...host.querySelectorAll("button, a")].map((node) =>
      (node.textContent ?? "").trim(),
    );
    for (const verb of ["Remember", "Import", "Enable", "Forget", "Quarantine"]) {
      expect(labels).not.toContain(verb);
    }

    const familiesQueried = calls
      .filter((call) => call.path === "/management/resource/v1/list")
      .map((call) => call.query.get("family"))
      .sort();
    expect(familiesQueried).toEqual(["context", "memory", "skill", "tool"]);
    expect(calls.some((call) => call.method !== "GET")).toBe(false);
    expect(isKnownRoute("GET", "/management/resource/v1/list?family=memory")).toBe(true);
    unmount(host, root);
  });

  it("points Context at Work and does not treat projection-only as an empty family", async () => {
    const { host, root } = await renderResources();
    const context = host.querySelector("[data-family='context']");
    expect(context?.getAttribute("data-family-kind")).toBe("projection-only");
    const work = context?.querySelector("a");
    expect(work?.getAttribute("href")).toMatch(/#\/work$/);
    expect(work?.textContent).toBe("Work");
    expect(host.querySelector("[data-family='memory']")?.getAttribute("data-family-kind")).toBe(
      "ready",
    );
    const browse = host.querySelector("[data-family='memory'] a");
    expect(browse?.textContent).toBe("browse");
    expect(browse?.getAttribute("href")).toMatch(/#\/resources\/memory$/);
    const skills = host.querySelector("[data-family='skill'] a");
    expect(skills?.textContent).toBe("browse");
    expect(skills?.getAttribute("href")).toMatch(/#\/resources\/skill$/);
    unmount(host, root);
  });

  it("keeps denied, stub, empty and projection-only as four distinct statements", async () => {
    const { host, root } = await renderResources({
      "GET /management/resource/v1/list": ({ url }) => {
        const family = url.searchParams.get("family");
        if (family === "memory") {
          return {
            status: 401,
            body: { status: "error", code: "UNAUTHORIZED", message: "no session" },
          };
        }
        if (family === "skill") {
          return {
            status: 200,
            body: { status: "ok", note: "business routes deferred" },
          };
        }
        if (family === "tool") {
          return listBody("tool", [], { authority_source: "daemon-native-tool-registry" });
        }
        return listBody("context", [], { authority_source: "projection-only" });
      },
    });
    expect(host.querySelector("[data-family='memory']")?.getAttribute("data-family-kind")).toBe(
      "denied",
    );
    expect(host.querySelector("[data-family='memory']")?.textContent).toMatch(/session denied/);
    expect(host.querySelector("[data-family='memory']")?.textContent).not.toMatch(/admitted/);
    expect(host.querySelector("[data-family='skill']")?.getAttribute("data-family-kind")).toBe(
      "stub",
    );
    expect(host.querySelector("[data-family='skill']")?.textContent).toContain("STUB_ROUTE");
    expect(host.querySelector("[data-family='tool']")?.getAttribute("data-family-kind")).toBe(
      "empty",
    );
    expect(host.querySelector("[data-family='context']")?.getAttribute("data-family-kind")).toBe(
      "projection-only",
    );
    unmount(host, root);
  });

  it("keeps a labelled family index and a Work link as a real focusable control", async () => {
    const { host, root } = await renderResources();
    const table = host.querySelector("table.cp-family-index");
    expect(table?.querySelector("caption")?.textContent).toBe("Resource families");
    const work = host.querySelector("[data-family='context'] a");
    expect(work).not.toBeNull();
    expect((work as HTMLElement).tabIndex).toBeGreaterThanOrEqual(0);
    unmount(host, root);
  });
});
