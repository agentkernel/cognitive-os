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
const JARGON_WALL = /① Charter|Dual Track wizard|Charter\/Process|Vite is not the product origin/i;

function primaryHonesty(host: HTMLElement): Element[] {
  return [...host.querySelectorAll("#main .cp-honesty")].filter(
    (node) => !node.closest("details[data-honesty='secondary']"),
  );
}

function fakeActionLabels(host: HTMLElement): string[] {
  return [...host.querySelectorAll("#main button, #main a.cp-button")]
    .map((node) => (node.textContent ?? "").trim())
    .filter((label) => FAKE_ACTION.test(label));
}

async function renderAuthedWizard() {
  rememberBearer("management", "test-management-bearer");
  installFetch({
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/project/v1/list": { status: 200, body: { status: "ok", projects: [] } },
  });
  const view = renderAppAt("#/projects/new");
  await flush();
  return view;
}

afterEach(() => {
  clearSession();
  appProjections.clear();
  window.location.hash = "";
  vi.unstubAllGlobals();
});

describe("P15-T02 Write/①–⑤ Owner copy vs v9", () => {
  it("fail-closes Write without a session: no wizard, no fake Activate, no Twitter hero", async () => {
    installFetch({});
    const { host, root } = renderAppAt("#/projects/new");
    await flush();
    expect(host.textContent).toMatch(/session denied|Open Session/i);
    expect(host.querySelector("[data-page='opc-create-wizard']")).toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    expect(host.textContent).not.toMatch(TWITTER_HERO);
    unmount(host, root);
  });

  it("leads ① with v9 Owner step language, not Charter / Dual Track jargon walls", async () => {
    const { host, root } = await renderAuthedWizard();
    expect(host.querySelector(".cp-shell")?.getAttribute("data-visual")).toBe("v9-target");
    expect(host.querySelector("[data-page='opc-create-wizard']")).not.toBeNull();
    const steps = [...host.querySelectorAll("[data-step-item]")].map((node) =>
      (node.textContent ?? "").trim(),
    );
    expect(steps).toEqual([
      "① 项目初始化",
      "② 流程初始化",
      "③ 成员初始化",
      "④ 分环节测试",
      "⑤ 联合调试",
    ]);
    expect(host.querySelector(".cp-page-head h2")?.textContent).toBe("创建项目 · ① 项目初始化");
    expect(host.querySelector("[data-wizard-heading]")?.textContent).toBe("① 逐项确认这件事");
    expect(host.querySelector(".cp-lede")?.textContent).toMatch(/右侧助手是主入口/);
    expect(host.textContent).not.toMatch(JARGON_WALL);
    expect(host.querySelector("label.cp-field")?.textContent).toMatch(/标题/);
    expect(host.querySelector("textarea[name='charter']")?.closest("label")?.textContent).toMatch(
      /这件事/,
    );
    expect(primaryHonesty(host)).toEqual([]);
    expect(fakeActionLabels(host)).toEqual([]);
    expect(host.textContent).not.toMatch(TWITTER_HERO);
    expect(host.textContent).toMatch(/进入 ②/);
    unmount(host, root);
  });

  it("keeps persist-before-dispatch Write and never offers Activate / Twitter P0", async () => {
    const { host, root } = await renderAuthedWizard();
    expect(host.textContent).not.toMatch(/\bActivate\b/);
    expect(host.textContent).not.toMatch(TWITTER_HERO);
    expect(fakeActionLabels(host)).toEqual([]);
    expect(host.querySelector("[data-step='create-init']")).not.toBeNull();
    unmount(host, root);
  });
});
