import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { isKnownRoute } from "../../data/normalize";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";

type RouteResponse = { status: number; body: unknown };
type RouteHandler = RouteResponse | ((call: { url: URL; body?: unknown }) => RouteResponse);

interface RecordedCall {
  method: string;
  path: string;
  query: URLSearchParams;
  body?: unknown;
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
      let body: unknown;
      if (typeof init?.body === "string") {
        try {
          body = JSON.parse(init.body);
        } catch {
          body = init.body;
        }
      }
      calls.push({ method, path: url.pathname, query: url.searchParams, body });
      const handler = routes[`${method} ${url.pathname}`];
      const resolved =
        typeof handler === "function"
          ? handler({ url, body })
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

function memoryRoutes(overrides: Record<string, RouteHandler> = {}): Record<string, RouteHandler> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", components: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/resource/v1/list": ({ url }) => {
      if (url.searchParams.get("family") === "memory") {
        return {
          status: 200,
          body: {
            kind: "resource.manager.list",
            family: "memory",
            authority_source: "sqlite-authority-memory-objects",
            truncated: false,
            resources: [{ id: "mem-1", family: "memory", health: "admitted" }],
          },
        };
      }
      return { status: 200, body: { status: "ok", resources: [] } };
    },
    "GET /management/resource/v1/inspect": {
      status: 200,
      body: {
        kind: "resource.manager.inspect",
        resource: { id: "mem-1", family: "memory", health: "admitted" },
      },
    },
    "GET /management/resource/v1/memory/object": {
      status: 200,
      body: {
        kind: "memory.explain",
        memory: {
          memory_id: "mem-1",
          candidate_id: "cand-1",
          decision_id: "dec-1",
          canonical_json: "{\"text\":\"admitted procedure\"}",
        },
      },
    },
    "POST /management/resource/v1/memory/remember": {
      status: 201,
      body: { status: "ok", memory_id: "mem-2" },
    },
    "POST /management/resource/v1/memory/forget": {
      status: 201,
      body: { status: "forgotten", memory_id: "mem-1" },
    },
    ...overrides,
  };
}

async function renderMemory(hash = "#/resources/memory", overrides: Record<string, RouteHandler> = {}) {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch(memoryRoutes(overrides));
  const view = renderAppAt(hash);
  await flush();
  return { ...view, calls };
}

describe("Memory family page (W7)", () => {
  afterEach(() => {
    clearSession();
    appProjections.clear();
    window.location.hash = "";
    vi.unstubAllGlobals();
  });

  it("lists admitted envelopes, names BD-6, and does not invent tombstones or a search index", async () => {
    const { host, root, calls } = await renderMemory();
    expect(host.querySelector("main h2")?.textContent).toBe("Memory");
    expect(host.textContent).toContain("mem-1");
    expect(host.textContent).toContain("tombstones are not in this list");
    expect(host.textContent).toMatch(/BD-6/);
    expect(host.textContent).toContain("envelope limit 64");
    expect(host.querySelector("input[type='search']")).toBeNull();
    expect(calls.map((call) => `${call.method} ${call.path}`)).toContain(
      "GET /management/resource/v1/list",
    );
    expect(isKnownRoute("POST", "/management/resource/v1/memory/remember")).toBe(true);
    unmount(host, root);
  });

  it("explains a selected object from inspect plus memory.object and never fabricates provenance", async () => {
    const { host, root, calls } = await renderMemory();
    const inspect = [...host.querySelectorAll("button")].find(
      (button) => (button.textContent ?? "").trim() === "Inspect",
    );
    expect(inspect).toBeDefined();
    await act(async () => {
      inspect?.click();
    });
    await flush();
    expect(host.textContent).toContain("cand-1");
    expect(host.textContent).toContain("dec-1");
    expect(host.textContent).toContain("admitted procedure");
    expect(host.textContent).toMatch(/durable tombstone/);
    const paths = calls.map((call) => `${call.method} ${call.path}`);
    expect(paths).toContain("GET /management/resource/v1/inspect");
    expect(paths).toContain("GET /management/resource/v1/memory/object");
    unmount(host, root);
  });

  it("names inspect 404 as a gap rather than an empty family", async () => {
    const { host, root } = await renderMemory("#/resources/memory", {
      "GET /management/resource/v1/inspect": {
        status: 404,
        body: {
          status: "error",
          code: "RESOURCE_MANAGER_NOT_FOUND",
          message: "Memory object was not found or is tombstoned",
        },
      },
      "GET /management/resource/v1/memory/object": {
        status: 404,
        body: { status: "error", code: "RESOURCE_MEMORY_NOT_FOUND", message: "Memory object not found" },
      },
    });
    const inspect = [...host.querySelectorAll("button")].find(
      (button) => (button.textContent ?? "").trim() === "Inspect",
    );
    await act(async () => {
      inspect?.click();
    });
    await flush();
    expect(host.textContent).toMatch(/RESOURCE_MANAGER_NOT_FOUND|RESOURCE_MEMORY_NOT_FOUND/);
    expect(host.textContent).not.toMatch(/No admitted Memory objects/);
    unmount(host, root);
  });

  it("previews remember with the 365-day cap and posts unsealed public fields only", async () => {
    const { host, root, calls } = await renderMemory();
    const form = host.querySelector("form.cp-form") as HTMLFormElement;
    const text = form.querySelector("textarea[name='text']") as HTMLTextAreaElement;
    const scope = form.querySelector("input[name='governance_scope']") as HTMLInputElement;
    const days = form.querySelector("input[name='retention_days']") as HTMLInputElement;
    await act(async () => {
      text.value = "remembered procedure";
      scope.value = "workspace://personal/control-plane";
      days.value = "90";
      form.requestSubmit();
    });
    await flush();
    expect(host.textContent).toContain("Confirm remember");
    const confirm = host.querySelector(".cp-confirm input[type='checkbox']") as HTMLInputElement;
    await act(async () => {
      confirm.click();
    });
    const remember = [...host.querySelectorAll(".cp-confirm button")].find(
      (button) => (button.textContent ?? "").trim() === "Remember",
    ) as HTMLButtonElement;
    await act(async () => {
      remember.click();
    });
    await flush();
    const posted = calls.find((call) => call.path === "/management/resource/v1/memory/remember");
    expect(posted?.method).toBe("POST");
    const body = posted?.body as Record<string, unknown>;
    expect(body.source).toBeUndefined();
    expect(body.candidate).toBeUndefined();
    expect(body.text).toBe("remembered procedure");
    expect(body.governance_scope).toBe("workspace://personal/control-plane");
    expect(body.retention_expires_at_unix_seconds).toEqual(expect.any(Number));
    unmount(host, root);
  });

  it("refuses retention above 365 days before calling remember", async () => {
    const { host, root, calls } = await renderMemory();
    const form = host.querySelector("form.cp-form") as HTMLFormElement;
    const text = form.querySelector("textarea[name='text']") as HTMLTextAreaElement;
    const scope = form.querySelector("input[name='governance_scope']") as HTMLInputElement;
    const days = form.querySelector("input[name='retention_days']") as HTMLInputElement;
    await act(async () => {
      text.value = "too long";
      scope.value = "workspace://personal/control-plane";
      days.value = "400";
      form.requestSubmit();
    });
    await flush();
    expect(host.textContent).not.toContain("Confirm remember");
    expect(calls.some((call) => call.path === "/management/resource/v1/memory/remember")).toBe(
      false,
    );
    unmount(host, root);
  });
});
