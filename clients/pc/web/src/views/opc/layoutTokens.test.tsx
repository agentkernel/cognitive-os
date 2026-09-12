import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";

const here = dirname(fileURLToPath(import.meta.url));
const tokensCss = readFileSync(join(here, "../../tokens.css"), "utf8");
const appCss = readFileSync(join(here, "../../app.css"), "utf8");

type RouteResponse = { status: number; body: unknown };

function installFetch(routes: Record<string, RouteResponse>) {
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

function primaryHonesty(host: HTMLElement): Element[] {
  return [...host.querySelectorAll(".cp-honesty")].filter(
    (node) => !node.closest("details[data-honesty='secondary']"),
  );
}

afterEach(() => {
  clearSession();
  appProjections.clear();
  window.location.hash = "";
  vi.unstubAllGlobals();
});

describe("P15-T06 remaining layout tokens (v9 176/22/44/~1100)", () => {
  it("fail-closes Today without a session: no stacked product chrome, no fake Activate, no Twitter hero", async () => {
    installFetch({});
    const { host, root } = renderAppAt("#/");
    await flush();
    expect(host.textContent).toMatch(/session denied|Open Session/i);
    expect(host.querySelector("[data-surface='today']")).toBeNull();
    const labels = [...host.querySelectorAll("#main button, #main a.cp-button")].map(
      (node) => (node.textContent ?? "").trim(),
    );
    expect(labels.filter((label) => FAKE_ACTION.test(label))).toEqual([]);
    expect(host.textContent).not.toMatch(TWITTER_HERO);
    expect(host.textContent).not.toMatch(/Vite is the product origin/i);
    unmount(host, root);
  });

  it("locks the v9 three-column tokens and does not revert T01 min-width or stack columns", () => {
    expect(tokensCss).toMatch(/--cp-shell-min-width:\s*1100px/);
    expect(tokensCss).toMatch(/--cp-nav-width:\s*176px/);
    expect(tokensCss).toMatch(/--cp-main-min:\s*576px/);
    expect(tokensCss).toMatch(/--cp-rail-width:\s*348px/);
    expect(tokensCss).toMatch(/--cp-size-title1:\s*1\.375rem/);
    expect(tokensCss).toMatch(/--cp-target-min:\s*44px/);
    expect(appCss).toMatch(
      /grid-template-columns:\s*var\(--cp-nav-width\)\s+minmax\(var\(--cp-main-min\),\s*1fr\)\s+var\(--cp-rail-width\)/,
    );
    expect(appCss).not.toMatch(/grid-template-columns:\s*232px/);
    expect(appCss).not.toMatch(/grid-template-columns:\s*200px/);
    expect(appCss).not.toMatch(/grid-template-areas:\s*"side"\s+"main"\s+"rail"/);
    expect(appCss).toMatch(/min-height:\s*var\(--cp-target-min\)/);
    expect(appCss).not.toMatch(/Vite is the product origin/);
  });

  it("demotes remaining Owner honesty on Today / Settings / Assistant rail to secondary details", async () => {
    rememberBearer("management", "test-management-bearer");
    installFetch({
      "GET /personal/health": { status: 200, body: { status: "ok" } },
      "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
      "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
      "GET /management/project/v1/list": { status: 200, body: { status: "ok", projects: [] } },
    });
    const { host, root } = renderAppAt("#/");
    await flush();
    expect(host.querySelector(".cp-shell")?.getAttribute("data-visual")).toBe("v9-target");
    expect(primaryHonesty(host)).toEqual([]);
    expect(host.textContent).not.toMatch(TWITTER_HERO);
    expect(host.querySelector("#main")?.textContent).not.toMatch(/Vite is not the product origin/i);
    unmount(host, root);

    const settings = renderAppAt("#/settings");
    await flush();
    expect(primaryHonesty(settings.host)).toEqual([]);
    expect(settings.host.textContent).not.toMatch(TWITTER_HERO);
    unmount(settings.host, settings.root);
  });
});
