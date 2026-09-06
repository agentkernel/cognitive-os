import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";

const tokensCss = readFileSync(
  join(dirname(fileURLToPath(import.meta.url)), "../../tokens.css"),
  "utf8",
);

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
  return [...host.querySelectorAll("#main .cp-honesty")].filter(
    (node) => !node.closest("details[data-honesty='secondary']"),
  );
}

afterEach(() => {
  clearSession();
  appProjections.clear();
  window.location.hash = "";
  vi.unstubAllGlobals();
});

describe("P15-T01 shell + Today Owner chrome (v9-target)", () => {
  it("fail-closes Today without a session: no fake Activate, no Twitter hero, no live packets", async () => {
    installFetch({});
    const { host, root } = renderAppAt("#/");
    await flush();
    expect(host.textContent).toMatch(/session denied|Open Session/i);
    expect(host.querySelector("[data-surface='today']")).toBeNull();
    expect(host.querySelector("[data-packet]")).toBeNull();
    const labels = [...host.querySelectorAll("#main button, #main a.cp-button")].map(
      (node) => (node.textContent ?? "").trim(),
    );
    expect(labels.filter((label) => FAKE_ACTION.test(label))).toEqual([]);
    expect(host.textContent).not.toMatch(TWITTER_HERO);
    unmount(host, root);
  });

  it("does not lead Today with a developer honesty wall, Vite origin lecture, or authority-writer subtitle", async () => {
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
    expect(host.querySelector(".cp-brand")?.textContent).not.toMatch(/not an authority writer/i);
    expect(host.querySelector("#main")?.textContent).not.toMatch(/Vite is not the product origin/i);
    expect(host.querySelector("main h2")?.textContent).toBe("今日");
    expect(host.querySelector("a[href='#/projects/new']")?.textContent).toMatch(/创建项目/);
    expect(host.textContent).not.toMatch(TWITTER_HERO);
    unmount(host, root);
  });

  it("applies the v9-target type scale and no-stack shell token", () => {
    expect(tokensCss).toMatch(/--cp-size-body:\s*0\.875rem/);
    expect(tokensCss).toMatch(/--cp-size-headline:\s*0\.9375rem/);
    expect(tokensCss).toMatch(/--cp-shell-min-width:\s*1100px/);
  });
});
