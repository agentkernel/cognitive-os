import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";

type RouteResponse = { status: number; body: unknown };
type FetchCall = { method: string; path: string; pathname: string; body?: unknown };

function installFetch(routes: Record<string, RouteResponse> = {}): FetchCall[] {
  const calls: FetchCall[] = [];
  vi.stubGlobal(
    "fetch",
    vi.fn(async (input: unknown, init?: RequestInit) => {
      const url = new URL(String(input), "http://localhost");
      const method = (init?.method ?? "GET").toUpperCase();
      let parsed: unknown;
      if (typeof init?.body === "string" && init.body.length > 0) {
        try {
          parsed = JSON.parse(init.body) as unknown;
        } catch {
          parsed = init.body;
        }
      }
      calls.push({
        method,
        path: `${url.pathname}${url.search}`,
        pathname: url.pathname,
        body: parsed,
      });
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

async function flush(ticks = 24) {
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

function clickTab(host: HTMLElement, label: string) {
  const tab = [...host.querySelectorAll('[role="tab"]')].find(
    (node) => (node.textContent ?? "").trim() === label,
  );
  if (!tab) {
    throw new Error(`tab not found: ${label}`);
  }
  act(() => {
    (tab as HTMLButtonElement).click();
  });
}

function clickButton(host: HTMLElement, text: string) {
  const button = [...host.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === text,
  );
  if (!button) {
    throw new Error(`button not found: ${text}`);
  }
  act(() => {
    button.click();
  });
}

function setFiles(input: HTMLInputElement, files: File[]) {
  const list: FileList & Record<number, File> = {
    length: files.length,
    item: (index: number) => files[index] ?? null,
  } as FileList & Record<number, File>;
  files.forEach((file, index) => {
    list[index] = file;
  });
  Object.defineProperty(input, "files", { configurable: true, value: list });
  act(() => {
    input.dispatchEvent(new Event("change", { bubbles: true }));
  });
}

const FAKE_ACTION = /approve|create project|activate|new project|team|inbox|confirm|apply authority/i;
const TWITTER_HERO = /twitter|\bx\b hero|trending on x|tweet composer/i;
const HTTP_PASTE_WALL = /HTTP-paste-only|owner-paste Vault HTTP|Typed note as the only ingest/i;
const JARGON_WALL = /Vite is not the product origin|Dual Track wizard|Why this fragment/i;

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

const READY_LIST: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [{ project_id: "proj-1", state: "active", title_summary: "unknown", cost: "unknown" }],
  },
};

function knowledgeRoutes(extras: Record<string, RouteResponse> = {}): Record<string, RouteResponse> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/project/v1/list": READY_LIST,
    "GET /management/project/v1/vault.index": {
      status: 200,
      body: {
        status: "ok",
        is_authority: false,
        inject_order: ["task-contract", "fixed-decision", "sourced-excerpt"],
        entries: [{ entry_id: "ent-1", document_id: "doc-1", layer: "sourced-excerpt", excerpt: "note" }],
      },
    },
    "GET /management/project/v1/vault.conflicts": { status: 200, body: { status: "ok", conflicts: [] } },
    "GET /management/resource/v1/list": {
      status: 200,
      body: { status: "ok", family: "memory", resources: [] },
    },
    "GET /management/resource/v1/vault.labeled": {
      status: 200,
      body: { status: "ok", is_authority: false, entries: [] },
    },
    "GET /management/resource/v1/vault.documents": {
      status: 200,
      body: { status: "ok", is_authority: false, documents: [] },
    },
    "GET /management/resource/v1/memory/promotes": {
      status: 200,
      body: { status: "ok", promotes: [] },
    },
    ...extras,
  };
}

async function renderKnowledge(extras: Record<string, RouteResponse> = {}) {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch(knowledgeRoutes(extras));
  const view = renderAppAt("#/knowledge");
  await flush();
  return { ...view, calls };
}

afterEach(() => {
  clearSession();
  appProjections.clear();
  window.location.hash = "";
  vi.unstubAllGlobals();
});

describe("P15-T04 Knowledge vs v9 Owner copy", () => {
  it("fail-closes Knowledge without a session: no ingest, no fake Activate, no Twitter hero", async () => {
    const calls = installFetch({});
    const { host, root } = renderAppAt("#/knowledge");
    await flush();
    expect(host.textContent).toMatch(/session denied|Open Session/i);
    expect(host.querySelector("[data-page='opc-knowledge']")).toBeNull();
    expect(host.querySelector("[data-region='opc-vault-ingest']")).toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    expect(host.textContent).not.toMatch(TWITTER_HERO);
    expect(host.textContent).not.toMatch(/\bActivate\b/);
    expect(calls.some((call) => call.pathname === "/management/project/v1/vault.import")).toBe(false);
    unmount(host, root);
  });

  it("leads Knowledge with v9 Owner language, not an HTTP-paste or developer honesty wall", async () => {
    const { host, root } = await renderKnowledge();
    expect(host.querySelector(".cp-shell")?.getAttribute("data-visual")).toBe("v9-target");
    expect(host.querySelector("[data-page='opc-knowledge']")).not.toBeNull();
    expect(host.querySelector(".cp-page-head h2")?.textContent).toBe("当前项目资料");
    expect(host.querySelector(".cp-lede")?.textContent).toMatch(/不必另装笔记应用/);
    const tabs = [...host.querySelectorAll('[role="tablist"][aria-label="知识"] [role="tab"]')].map(
      (node) => (node.textContent ?? "").trim(),
    );
    expect(tabs).toEqual(["项目资料", "导入", "为什么用这段", "记忆"]);
    expect(host.querySelector("[data-region='opc-knowledge-files']")).not.toBeNull();
    expect(host.textContent).toMatch(/还没资料/);
    expect(host.querySelector("[data-region='opc-vault-ingest']")).toBeNull();
    expect(host.querySelector("textarea[name='vault-body']")).toBeNull();
    expect(primaryHonesty(host)).toEqual([]);
    expect(host.querySelector("#main")?.textContent).not.toMatch(JARGON_WALL);
    expect(host.textContent).not.toMatch(HTTP_PASTE_WALL);
    expect(host.textContent).not.toMatch(/obsidian/i);
    expect(host.textContent).not.toMatch(TWITTER_HERO);
    expect(host.textContent).not.toMatch(/\bActivate\b/);
    expect(fakeActionLabels(host)).toEqual([]);
    clickButton(host, "导入资料");
    expect(host.querySelector("[data-region='opc-vault-ingest']")).not.toBeNull();
    expect(host.querySelector('input[name="vault-files"]')).not.toBeNull();
    expect(host.querySelector("h3")?.textContent).toBe("导入资料");
    unmount(host, root);
  });

  it("refuses secret ingest and a file that claims authority, with no Activate / Twitter P0", async () => {
    const { host, root, calls } = await renderKnowledge({
      "POST /management/project/v1/vault.import": {
        status: 200,
        body: { status: "ok", document_id: "doc-auth", is_authority: true },
      },
    });
    clickTab(host, "导入");
    const input = host.querySelector('input[name="vault-files"]') as HTMLInputElement;
    setFiles(input, [new File(["api_key=sk-p15t04-fixture"], "secret.md", { type: "text/markdown" })]);
    await flush();
    clickButton(host, "开始导入");
    await flush();
    expect(calls.some((call) => call.pathname === "/management/project/v1/vault.import")).toBe(false);
    expect(host.querySelector("[data-ingest-error='true']")?.textContent).toMatch(/secret-shaped|密钥/i);
    expect(host.textContent).not.toMatch(/\bActivate\b/);
    expect(host.textContent).not.toMatch(TWITTER_HERO);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });
});
