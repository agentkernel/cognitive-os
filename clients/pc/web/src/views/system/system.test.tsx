import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { isKnownRoute } from "../../data/normalize";
import { SYSTEM_CLAIM_CEILING, SYSTEM_RESTORE_409 } from "../../data/projections/system";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";

type RouteResponse = { status: number; body: unknown };
type RouteHandler = RouteResponse | ((call: { url: URL; body?: unknown }) => RouteResponse);

function installFetch(routes: Record<string, RouteHandler>) {
  const calls: { method: string; path: string; body?: unknown }[] = [];
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
              status:
                url.pathname === "/personal/health" ||
                url.pathname === "/personal/status" ||
                url.pathname === "/management/alerts"
                  ? 200
                  : 404,
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

function systemRoutes(overrides: Record<string, RouteHandler> = {}): Record<string, RouteHandler> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": {
      status: 200,
      body: {
        status: "ok",
        overall: "degraded",
        first_conversation_ready: false,
        evaluated_at_unix_ms: Date.now() - 12_000,
        components: [
          { name: "system", state: "ready" },
          { name: "database", state: "ready" },
          { name: "secret", state: "ready" },
          { name: "provider", state: "degraded", detail: "catalog stale" },
          { name: "daemon", state: "ready" },
          { name: "pi", state: "ready" },
        ],
      },
    },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /personal/doctor": {
      status: 200,
      body: {
        overall: "degraded",
        first_conversation_ready: false,
        evaluated_at_unix_ms: Date.now() - 12_000,
        components: [
          {
            component: "provider",
            status: "degraded",
            source: "readiness",
            facts: [{ key: "catalog", value: "stale" }],
          },
        ],
        guidance: ["refresh the provider catalog"],
        six_resource: { overall: "blocked", error_code: "RESOURCE_HEALTH_NOT_PROBED" },
        headless_vault: { overall: "unavailable", error_code: "VAULT_PATH_NOT_PROBED" },
        operability: { overall: "blocked", error_code: "OPERABILITY_TOPIC_NOT_PROBED" },
        static_check_is_not_runtime_ready: true,
        profile_claim: "not-claimed",
        gate_claim: "not-claimed",
      },
    },
    "POST /management/resource/v1/backup": {
      status: 200,
      body: {
        archive_id: "arch-1",
        archive_path: "/tmp/arch-1",
        manifest_digest: "sha256:backup",
        excluded_secret_count: 2,
        sqlite_copied: false,
      },
    },
    "POST /management/resource/v1/backup/preflight": {
      status: 200,
      body: { preflight_only: true },
    },
    "POST /management/resource/v1/restore": {
      status: 409,
      body: { status: "error", code: "RESOURCE_BACKUP_TAMPERED" },
    },
    ...overrides,
  };
}

async function renderSystem(hash = "#/system", overrides: Record<string, RouteHandler> = {}) {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch(systemRoutes(overrides));
  const view = renderAppAt(hash);
  await flush();
  return { ...view, calls };
}

async function openSection(host: HTMLDivElement, title: string) {
  const button = [...host.querySelectorAll("button")].find(
    (node) => (node.textContent ?? "").trim() === title,
  );
  await act(async () => {
    button?.click();
  });
  await flush();
}

describe("System page (W9)", () => {
  afterEach(() => {
    clearSession();
    appProjections.clear();
    window.location.hash = "";
    vi.unstubAllGlobals();
  });

  it("replaces the placeholder with readiness and the claim ceiling", async () => {
    const { host, root, calls } = await renderSystem();
    expect(host.querySelector("main h2")?.textContent).toBe("System");
    expect(host.textContent).toContain(SYSTEM_CLAIM_CEILING);
    expect(host.textContent).not.toMatch(/Under reconstruction/);
    expect(host.textContent).toContain("provider");
    expect(host.textContent).toContain("degraded");
    expect(host.textContent).toContain("not ready");
    expect(calls.map((call) => `${call.method} ${call.path}`)).toContain("GET /personal/status");
    expect(calls.map((call) => `${call.method} ${call.path}`)).toContain("GET /personal/doctor");
    unmount(host, root);
  });

  it("names doctor sub-sections that were not probed", async () => {
    const { host, root } = await renderSystem();
    await openSection(host, "Doctor");
    expect(host.textContent).toContain("refresh the provider catalog");
    expect(host.querySelector("[data-subsection='six-resource']")?.textContent).toContain(
      "RESOURCE_HEALTH_NOT_PROBED",
    );
    expect(host.querySelector("[data-subsection='six-resource']")?.textContent).toContain(
      "not probed over HTTP",
    );
    expect(host.textContent).toContain("never contains secrets");
    unmount(host, root);
  });

  it("runs backup class-A and names restore 409 instead of retrying", async () => {
    const { host, root, calls } = await renderSystem();
    await openSection(host, "Stewardship");
    expect(host.textContent).toContain("Secrets and raw SQLite are never included");
    expect(host.textContent).toContain(SYSTEM_RESTORE_409);
    const backupBox = [...host.querySelectorAll("input[type='checkbox']")][0] as HTMLInputElement;
    await act(async () => {
      backupBox.click();
    });
    const create = [...host.querySelectorAll("button")].find(
      (button) => (button.textContent ?? "").trim() === "Create backup",
    );
    await act(async () => {
      create?.click();
    });
    await flush();
    expect(host.textContent).toContain("archive arch-1");
    expect(host.textContent).toContain("excluded secrets 2");
    expect(host.textContent).toContain("sqlite copied false");
    const archive = host.querySelector("input:not([type='checkbox'])") as HTMLInputElement;
    await act(async () => {
      archive.value = "arch-1";
      archive.dispatchEvent(new Event("input", { bubbles: true }));
    });
    const restoreBox = [...host.querySelectorAll("input[type='checkbox']")][1] as HTMLInputElement;
    await act(async () => {
      restoreBox.click();
    });
    const restore = [...host.querySelectorAll("button")].find(
      (button) => (button.textContent ?? "").trim() === "Restore now",
    );
    await act(async () => {
      restore?.click();
    });
    await flush();
    expect(host.textContent).toContain("HTTP 409 RESOURCE_BACKUP_TAMPERED");
    expect(calls.some((call) => call.path === "/management/resource/v1/backup" && call.method === "POST")).toBe(
      true,
    );
    expect(isKnownRoute("POST", "/management/resource/v1/restore")).toBe(true);
    unmount(host, root);
  });

  it("states session expiry as unknown under BD-7", async () => {
    const { host, root } = await renderSystem();
    await openSection(host, "Session");
    expect(host.textContent).toContain("BD-7");
    expect(host.textContent).toContain("unknown (BD-7)");
    expect(host.textContent).toContain("principal://local/owner");
    unmount(host, root);
  });

  it("keeps about copy from claiming a Gate", async () => {
    const { host, root } = await renderSystem();
    await openSection(host, "About");
    expect(host.textContent).toContain("not-claimed");
    expect(host.textContent).toContain("cognitive doctor --bundle");
    expect(host.textContent).not.toMatch(/Gate pass/);
    unmount(host, root);
  });
});
