import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { buildCommandCatalog } from "../../data/commands";
import { createProjectionStore } from "../../data/store";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";
import { PRIMARY_NAV } from "../../shell/PrimaryNav";
import { SPACE_CHORDS } from "../../shell/keyboard";

type RouteResponse = { status: number; body: unknown };

function installFetch(routes: Record<string, RouteResponse> = {}): void {
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
          : url.pathname === "/management/alerts"
            ? { status: 200, body: { status: "ok", alerts: [] } }
            : url.pathname === "/management/project/v1/list"
              ? { status: 200, body: { status: "ok", projects: [] } }
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

async function flush(ticks = 16) {
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

async function renderOwner(hash: string) {
  rememberBearer("management", "test-management-bearer");
  installFetch({
    "GET /management/providers/accounts": { status: 200, body: { status: "ok", accounts: [] } },
    "GET /management/usage": { status: 200, body: { status: "ok", events: [] } },
    "GET /management/project/v1/standing-policies": { status: 200, body: { status: "ok", policies: [] } },
    "GET /management/settings/v1/diagnostics": {
      status: 200,
      body: { status: "ok", dsh: {}, pi: {} },
    },
    "GET /management/settings/v1/notifications": {
      status: 200,
      body: { status: "ok", missed: [], offline: [], resume: [] },
    },
  });
  const view = renderAppAt(hash);
  await flush();
  return view;
}

afterEach(() => {
  clearSession();
  appProjections.clear();
  window.location.hash = "";
  vi.unstubAllGlobals();
});

describe("P14-T07 Owner chrome IA (failure-first vs EVAL-016 J6/J8/J12/J20)", () => {
  it("makes Settings a real L1 role=link beside Today / Projects / Knowledge", async () => {
    expect(PRIMARY_NAV.map(([, label]) => label)).toEqual([
      "Today",
      "Projects",
      "Knowledge",
      "Settings",
    ]);
    const { host, root } = await renderOwner("#/");
    const nav = host.querySelector('nav[aria-label="Primary"]');
    const settings = [...(nav?.querySelectorAll("a") ?? [])].find(
      (node) => (node.textContent ?? "").trim() === "Settings",
    );
    expect(settings).not.toBeNull();
    expect(settings?.getAttribute("href")).toBe("#/settings");
    expect(settings?.getAttribute("role") === "link" || settings?.tagName === "A").toBe(true);
    expect(nav?.textContent).not.toMatch(/Team|Inbox|Home|Work|Agents|Providers/);
    unmount(host, root);
  });

  it("does not dump the 9×9 state-lab into the default Settings tree", async () => {
    const { host, root } = await renderOwner("#/settings");
    expect(host.querySelector("[data-page='opc-settings']")).not.toBeNull();
    expect(host.querySelectorAll("[data-state-lab-cell]").length).toBe(0);
    expect(host.querySelector("[data-region='opc-state-lab-grid']")).toBeNull();
    expect(host.querySelector("[data-page='opc-state-lab']")).toBeNull();
    const lab = host.querySelector("[data-region='opc-settings-state-lab']") as HTMLDetailsElement | null;
    expect(lab).not.toBeNull();
    expect(lab?.open).toBe(false);
    unmount(host, root);
  });

  it("renders retired Linux 1.0 hashes as No such route, same as #/inbox", async () => {
    for (const hash of [
      "#/inbox",
      "#/home",
      "#/work",
      "#/work/new",
      "#/agents",
      "#/providers",
      "#/bindings",
      "#/tasks",
    ]) {
      const { host, root } = await renderOwner(hash);
      expect(host.textContent).toContain("No such route");
      expect(host.querySelector("[data-page='opc-settings']")).toBeNull();
      unmount(host, root);
    }
  });

  it("does not advertise Home / Work / Agents / Providers as 2.0 palette destinations", () => {
    const items = buildCommandCatalog(createProjectionStore());
    expect(items.some((item) => item.href === "/home")).toBe(false);
    expect(items.some((item) => item.href === "/work")).toBe(false);
    expect(items.some((item) => item.href === "/work/new")).toBe(false);
    expect(items.some((item) => item.href === "/agents")).toBe(false);
    expect(items.some((item) => item.href === "/providers")).toBe(false);
    expect(items.some((item) => item.label === "Work" && item.kind === "destination")).toBe(false);
    expect(items.some((item) => item.href === "/settings")).toBe(true);
    expect(items.some((item) => item.href === "/settings/model-connections")).toBe(true);
  });

  it("does not bind g-chords onto retired Linux 1.0 hashes", () => {
    expect(SPACE_CHORDS.s).toBe("/settings");
    expect(SPACE_CHORDS.h).toBeUndefined();
    expect(SPACE_CHORDS.w).toBeUndefined();
    expect(SPACE_CHORDS.a).toBeUndefined();
    expect(SPACE_CHORDS.v).toBeUndefined();
  });

  it("reaches Model Connections on #/settings/model-connections without a fake Connect or Vite origin", async () => {
    const { host, root } = await renderOwner("#/settings/model-connections");
    const form = host.querySelector("[data-region='opc-model-connections']");
    expect(form).not.toBeNull();
    const submit = form?.querySelector("button[type='submit']") as HTMLButtonElement | null;
    expect(submit?.disabled).toBe(true);
    expect(submit?.textContent).toMatch(/SecretStore/i);
    expect([...form!.querySelectorAll("button")].map((node) => (node.textContent ?? "").trim())).not.toContain(
      "Connect",
    );
    expect(host.textContent).toMatch(/daemon-served hash \/ui\//);
    expect(host.textContent).not.toMatch(/vite preview|vite dev server|localhost:5173/i);
    expect(host.querySelector("input[name='api_key']")?.getAttribute("type")).toBe("password");
    unmount(host, root);
  });

  it("closes the command palette on Escape so a later page click is not intercepted", async () => {
    const { host, root } = await renderOwner("#/settings");
    const open = host.querySelector('button[aria-label="Open command palette"]') as HTMLButtonElement;
    await act(async () => {
      open.click();
    });
    await flush(4);
    expect(host.querySelector('[role="dialog"][aria-label="Command palette"]')).not.toBeNull();
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }));
    });
    await flush(4);
    expect(host.querySelector('[role="dialog"][aria-label="Command palette"]')).toBeNull();
    const settings = [...host.querySelectorAll('nav[aria-label="Primary"] a')].find(
      (node) => (node.textContent ?? "").trim() === "Settings",
    ) as HTMLAnchorElement | undefined;
    await act(async () => {
      settings?.click();
    });
    await flush(4);
    expect(host.querySelector("[data-page='opc-settings']")).not.toBeNull();
    unmount(host, root);
  });
});
