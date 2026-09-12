import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";

type RouteResponse = { status: number; body: unknown };

function installFetch(routes: Record<string, RouteResponse> = {}) {
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
          : { status: 401, body: { status: "error", code: "UNAUTHENTICATED", message: "no session" } });
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
const TWITTER_HERO = /twitter|\bx\b hero|trending on x|tweet composer/i;
const JARGON_WALL = /Hand key to SecretStore|don't-ask-this-week|Vite is not the product origin/i;

function primaryHonesty(host: HTMLElement): Element[] {
  return [...host.querySelectorAll("#main .cp-honesty")].filter(
    (node) => !node.closest("details[data-honesty='secondary']"),
  );
}

function primaryMainText(host: HTMLElement): string {
  const main = host.querySelector("#main");
  if (!main) {
    return "";
  }
  const clone = main.cloneNode(true) as HTMLElement;
  for (const node of clone.querySelectorAll("details[data-honesty='secondary']")) {
    node.remove();
  }
  return clone.textContent ?? "";
}

function fakeActionLabels(host: HTMLElement): string[] {
  return [...host.querySelectorAll("#main button, #main a.cp-button")]
    .map((node) => (node.textContent ?? "").trim())
    .filter((label) => FAKE_ACTION.test(label));
}

function settingsRoutes(): Record<string, RouteResponse> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/project/v1/list": { status: 200, body: { status: "ok", projects: [] } },
    "GET /management/project/v1/standing-policies": { status: 200, body: { status: "ok", policies: [] } },
    "GET /management/providers/accounts": { status: 200, body: { status: "ok", accounts: [] } },
    "GET /management/usage": { status: 200, body: { status: "ok", events: [] } },
    "GET /management/settings/v1/diagnostics": {
      status: 200,
      body: { status: "ok", dsh: { facts: "empty" }, pi: { facts: "empty" } },
    },
    "GET /management/settings/v1/notifications": {
      status: 200,
      body: { status: "ok", missed: [], offline: [], resume: [] },
    },
  };
}

async function renderAuthedSettings(hash: string) {
  rememberBearer("management", "test-management-bearer");
  installFetch(settingsRoutes());
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

describe("P15-T05 Settings / Model Connections Owner copy vs v9", () => {
  it("fail-closes Settings without a session: no hub, no fake Connect, no Twitter P0", async () => {
    installFetch({});
    const { host, root } = renderAppAt("#/settings");
    await flush();
    expect(host.textContent).toMatch(/session denied|Open Session/i);
    expect(host.querySelector("[data-page='opc-settings']")).toBeNull();
    expect(host.querySelector("[data-region='opc-model-connections']")).toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    expect(host.textContent).not.toMatch(TWITTER_HERO);
    expect([...host.querySelectorAll("button")].map((node) => (node.textContent ?? "").trim())).not.toContain(
      "Connect",
    );
    unmount(host, root);
  });

  it("leads Settings with v9 Owner language and an honest Model Connections hub", async () => {
    const { host, root } = await renderAuthedSettings("#/settings");
    expect(host.querySelector("[data-page='opc-settings']")).not.toBeNull();
    expect(host.querySelector(".cp-page-head h2")?.textContent).toBe("设置");
    expect(host.querySelector(".cp-lede")?.textContent).toMatch(
      /连接模型 · 收回本周不再问 · 通知恢复/,
    );
    const form = host.querySelector("[data-region='opc-model-connections']");
    expect(form).not.toBeNull();
    expect(form?.querySelector("h2")?.textContent).toBe("模型连接");
    expect(form?.querySelector("label.cp-field span")?.textContent).toBe("供应商模板");
    expect(form?.querySelector("input[name='api_key']")?.closest("label")?.textContent).toMatch(
      /密钥（一次性交接）/,
    );
    expect(form?.querySelector("input[name='api_key']")?.getAttribute("type")).toBe("password");
    const submit = form?.querySelector("button[type='submit']") as HTMLButtonElement | null;
    expect(submit?.disabled).toBe(true);
    expect(submit?.textContent).toMatch(/交接密钥/);
    expect(submit?.textContent).toMatch(/SecretStore/i);
    expect([...form!.querySelectorAll("button")].map((node) => (node.textContent ?? "").trim())).not.toContain(
      "Connect",
    );
    expect(host.querySelector("[data-region='opc-settings-notifications'] h2")?.textContent).toBe(
      "通知与恢复",
    );
    expect(host.textContent).toMatch(/本周不再问/);
    expect(primaryHonesty(host)).toEqual([]);
    expect(primaryMainText(host)).not.toMatch(JARGON_WALL);
    expect(host.querySelectorAll("[data-state-lab-cell]").length).toBe(0);
    expect(host.querySelector("[data-region='opc-settings-state-lab']")).toHaveProperty("open", false);
    expect(fakeActionLabels(host)).toEqual([]);
    expect(host.textContent).not.toMatch(TWITTER_HERO);
    expect(host.textContent).not.toMatch(/\bActivate\b/);
    unmount(host, root);
  });

  it("keeps #/settings/model-connections honest: no fake key, no Provider secret, no Connect", async () => {
    const { host, root } = await renderAuthedSettings("#/settings/model-connections");
    const form = host.querySelector("[data-region='opc-model-connections']") as HTMLElement;
    expect(form).not.toBeNull();
    expect(host.querySelector(".cp-page-head h2")?.textContent).toBe("设置");
    const key = form.querySelector("input[name='api_key']") as HTMLInputElement;
    act(() => {
      key.value = "sk-live-must-never-render";
      key.dispatchEvent(new Event("input", { bubbles: true }));
    });
    expect(host.textContent).not.toContain("sk-live-must-never-render");
    expect(key.getAttribute("type")).toBe("password");
    expect(host.textContent).not.toMatch(/ss:\/\/provider\//);
    expect([...form.querySelectorAll("button")].map((node) => (node.textContent ?? "").trim())).not.toContain(
      "Connect",
    );
    expect(host.textContent).not.toMatch(TWITTER_HERO);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });
});
