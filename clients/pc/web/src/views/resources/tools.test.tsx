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
  body?: unknown;
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
      calls.push({ method, path: url.pathname, body });
      const handler = routes[`${method} ${url.pathname}`];
      const resolved =
        typeof handler === "function"
          ? handler({ url, body })
          : (handler ?? {
              status: pathStatus(url.pathname),
              body: { status: "ok" },
            });
      return new Response(JSON.stringify(resolved.body), {
        status: resolved.status,
        headers: { "content-type": "application/json" },
      });
    }),
  );
  return calls;
}

function pathStatus(path: string): number {
  if (
    path === "/personal/health" ||
    path === "/personal/status" ||
    path === "/management/alerts"
  ) {
    return 200;
  }
  return 404;
}

function catalog(overrides: Record<string, unknown>[] = []) {
  return {
    status: 200,
    body: {
      kind: "tool.lifecycle.projection",
      authority_source: "daemon-native-tool-registry",
      resources: [
        {
          operation_id: "native.workspace.read",
          action: "read",
          family: "workspace",
          risk: "read",
          lifecycle: "enabled",
          execution_readiness: "execution_ready",
          descriptor_digest: "sha256:read",
          agent_exposed: true,
        },
        {
          operation_id: "native.process.check",
          action: "check",
          family: "process",
          risk: "process",
          lifecycle: "quarantined",
          execution_readiness: "registered_only",
          descriptor_digest: "sha256:check",
          agent_exposed: false,
        },
        ...overrides,
      ],
    },
  };
}

function toolRoutes(overrides: Record<string, RouteHandler> = {}): Record<string, RouteHandler> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", components: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/resource/v1/tool": catalog(),
    "POST /management/resource/v1/tool/disable": {
      status: 200,
      body: {
        kind: "tool.lifecycle.mutation",
        operation_id: "native.workspace.read",
        lifecycle: "disabled",
      },
    },
    ...overrides,
  };
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

async function renderTools(overrides: Record<string, RouteHandler> = {}) {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch(toolRoutes(overrides));
  const view = renderAppAt("#/resources/tool");
  await flush();
  return { ...view, calls };
}

describe("Tools family page (W7)", () => {
  afterEach(() => {
    clearSession();
    appProjections.clear();
    window.location.hash = "";
    vi.unstubAllGlobals();
  });

  it("renders a catalog table, names the readiness caveat, and is not a card wall", async () => {
    const { host, root, calls } = await renderTools();
    expect(host.querySelector("main h2")?.textContent).toBe("Tools");
    expect(host.textContent).toContain("native.workspace.read");
    expect(host.textContent).toContain("quarantined");
    expect(host.textContent).toContain("registered/enabled ≠ production call chain wired");
    expect(host.querySelectorAll("[data-annotation='tool-readiness']")).toHaveLength(1);
    expect(host.textContent).toContain("not a card wall");
    expect(calls.map((call) => `${call.method} ${call.path}`)).toContain(
      "GET /management/resource/v1/tool",
    );
    expect(isKnownRoute("POST", "/management/resource/v1/tool/quarantine")).toBe(true);
    unmount(host, root);
  });

  it("does not offer enable on a quarantined row and states the one-way rule", async () => {
    const { host, root } = await renderTools();
    const inspect = [...host.querySelectorAll("button")].find((button) => {
      const row = button.closest("tr");
      return (
        (button.textContent ?? "").trim() === "Inspect" &&
        row?.textContent?.includes("native.process.check")
      );
    });
    await act(async () => {
      inspect?.click();
    });
    await flush();
    expect(host.textContent).toContain("cannot be enabled");
    expect(host.textContent).not.toMatch(/Preview enable/);
    expect([...host.querySelectorAll("button")].map((button) => (button.textContent ?? "").trim())).not.toContain(
      "Enable",
    );
    unmount(host, root);
  });

  it("posts only operation_id after class-A confirm", async () => {
    const { host, root, calls } = await renderTools();
    const inspect = [...host.querySelectorAll("button")].find((button) => {
      const row = button.closest("tr");
      return (
        (button.textContent ?? "").trim() === "Inspect" &&
        row?.textContent?.includes("native.workspace.read")
      );
    });
    await act(async () => {
      inspect?.click();
    });
    await flush();
    const preview = [...host.querySelectorAll("button")].find(
      (button) => (button.textContent ?? "").trim() === "Preview disable",
    );
    await act(async () => {
      preview?.click();
    });
    await flush();
    expect(host.textContent).toContain("Confirm disable");
    const confirm = [...host.querySelectorAll(".cp-confirm")]
      .find((node) => node.textContent?.includes("Confirm disable"))
      ?.querySelector("input[type='checkbox']") as HTMLInputElement;
    await act(async () => {
      confirm.click();
    });
    const disable = [...host.querySelectorAll(".cp-confirm button")].find(
      (button) => (button.textContent ?? "").trim() === "Disable",
    ) as HTMLButtonElement;
    await act(async () => {
      disable.click();
    });
    await flush();
    const posted = calls.find((call) => call.path === "/management/resource/v1/tool/disable");
    expect(posted?.body).toEqual({ operation_id: "native.workspace.read" });
    unmount(host, root);
  });
});
