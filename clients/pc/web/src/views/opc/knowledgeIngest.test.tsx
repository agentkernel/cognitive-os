import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { isKnownRoute } from "../../data/normalize";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";

type RouteResponse = { status: number; body: unknown };
type FetchCall = { method: string; path: string; pathname: string; body?: unknown };

function installFetch(routes: Record<string, RouteResponse>): FetchCall[] {
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
          : { status: 404, body: { status: "error", code: "NOT_FOUND", message: "not found" } });
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

function setInputValue(input: HTMLInputElement | HTMLTextAreaElement, value: string) {
  const proto =
    input.tagName === "TEXTAREA"
      ? window.HTMLTextAreaElement.prototype
      : window.HTMLInputElement.prototype;
  const setter = Object.getOwnPropertyDescriptor(proto, "value")?.set;
  setter?.call(input, value);
  act(() => {
    input.dispatchEvent(new Event("input", { bubbles: true }));
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

const FAKE_ACTION = /approve|create project|activate|new project|team|inbox|confirm|apply authority/i;

function fakeActionLabels(host: HTMLElement): string[] {
  const scopes = [
    host.querySelector("#main"),
    host.querySelector("[data-rail='assistant']"),
    host.querySelector("nav[aria-label='Primary']"),
  ].filter((node): node is Element => node !== null);
  const labels: string[] = [];
  for (const scope of scopes) {
    for (const node of scope.querySelectorAll("button, a.cp-button")) {
      if (node.closest("[data-region='opc-hitl-actions']") || node.closest("[data-region='opc-vault-ingest']")) {
        continue;
      }
      const label = (node.textContent ?? "").trim();
      if (FAKE_ACTION.test(label)) {
        labels.push(label);
      }
    }
  }
  return labels;
}

const EMPTY_LIST: RouteResponse = {
  status: 200,
  body: { status: "ok", projects: [] },
};

const READY_LIST: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [{ project_id: "proj-1", state: "active", title_summary: "unknown", cost: "unknown" }],
  },
};

const INDEX_OK: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    is_authority: false,
    inject_order: ["task-contract", "fixed-decision", "sourced-excerpt", "summary", "older-narrative"],
    entries: [{ entry_id: "ent-1", document_id: "doc-1", layer: "sourced-excerpt", excerpt: "note" }],
  },
};

function knowledgeRoutes(extras: Record<string, RouteResponse> = {}): Record<string, RouteResponse> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/project/v1/list": READY_LIST,
    "GET /management/project/v1/vault.index": INDEX_OK,
    "GET /management/project/v1/vault.conflicts": { status: 200, body: { status: "ok", conflicts: [] } },
    "GET /management/resource/v1/list": {
      status: 200,
      body: { status: "ok", family: "memory", resources: [] },
    },
    ...extras,
  };
}

async function renderKnowledge(hash: string, extras: Record<string, RouteResponse> = {}) {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch(knowledgeRoutes(extras));
  const view = renderAppAt(hash);
  await flush();
  return { ...view, calls };
}

afterEach(() => {
  clearSession();
  appProjections.clear();
  window.location.hash = "";
  vi.unstubAllGlobals();
});

describe("P12-T07 Knowledge ingest + Why this fragment", () => {
  it("whitelists vault.import and rebuild and keeps apply-authority off the client", () => {
    expect(isKnownRoute("POST", "/management/project/v1/vault.import")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/vault.index.rebuild")).toBe(true);
    expect(isKnownRoute("GET", "/management/project/v1/vault.conflicts")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/vault.apply-authority")).toBe(false);
    expect(isKnownRoute("POST", "/task/project/v1/vault.import")).toBe(false);
  });

  it("does not offer ingest without a Project id", async () => {
    const { host, root, calls } = await renderKnowledge("#/knowledge", {
      "GET /management/project/v1/list": EMPTY_LIST,
    });
    expect(host.querySelector("[data-region='opc-vault-ingest']")).toBeNull();
    expect(host.querySelector("[data-region='opc-why-fragment']")).toBeNull();
    expect(calls.some((call) => call.pathname === "/management/project/v1/vault.import")).toBe(false);
    expect(fakeActionLabels(host)).toEqual([]);
    expect(host.querySelector("[data-region='opc-vault-ingest']")).toBeNull();
    unmount(host, root);
  });

  it("shows Why this fragment from daemon inject_order and excerpts", async () => {
    const { host, root } = await renderKnowledge("#/knowledge");
    expect(host.querySelector("[data-region='opc-why-fragment']")).not.toBeNull();
    expect(host.querySelector("[data-row-key='ent-1']")?.textContent).toMatch(/sourced-excerpt/);
    expect(host.textContent).toContain("fixed-decision");
    expect(host.textContent).toContain("sourced-excerpt");
    expect(host.querySelector("[data-region='opc-vault-ingest']")).not.toBeNull();
    expect(host.querySelector("[data-region='opc-vault-ingest']")?.textContent).not.toMatch(
      /obsidian/i,
    );
    expect(host.textContent).toMatch(/Import to Vault/);
    expect(host.textContent).not.toMatch(/apply authority/i);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("posts vault.import then index.rebuild and does not treat the file as authority", async () => {
    const { host, root, calls } = await renderKnowledge("#/knowledge", {
      "POST /management/project/v1/vault.import": {
        status: 200,
        body: {
          status: "ok",
          document_id: "doc-imported",
          is_authority: false,
          host_fs_e2e: "not-run",
        },
      },
      "POST /management/project/v1/vault.index.rebuild": {
        status: 200,
        body: { status: "ok", written: 1, memory_fts: "untouched" },
      },
    });
    setInputValue(host.querySelector("textarea[name='vault-body']") as HTMLTextAreaElement, "version one");
    clickButton(host, "Import to Vault");
    await flush();
    const imported = calls.find((call) => call.pathname === "/management/project/v1/vault.import");
    expect(imported?.body).toEqual({
      project_id: "proj-1",
      relative_path: "notes/note.md",
      rights_class: "owner-owned",
      provenance: { source_uri: "owner-paste" },
      source_kind: "owner-paste",
      body: "version one",
    });
    expect(calls.some((call) => call.pathname === "/management/project/v1/vault.index.rebuild")).toBe(
      true,
    );
    expect(calls.some((call) => call.pathname === "/management/project/v1/vault.apply-authority")).toBe(
      false,
    );
    expect(host.querySelector("[data-ingest-receipt='doc-imported']")).not.toBeNull();
    expect(host.textContent).toMatch(/is_authority remains false/i);
    unmount(host, root);
  });

  it("keeps the original fields when vault.import is rejected", async () => {
    const { host, root, calls } = await renderKnowledge("#/knowledge", {
      "POST /management/project/v1/vault.import": {
        status: 422,
        body: {
          status: "error",
          error: { code: "INVALID", message: "last-write-wins without a conflict record is rejected" },
        },
      },
    });
    setInputValue(host.querySelector("input[name='relative_path']") as HTMLInputElement, "notes/keep.md");
    setInputValue(host.querySelector("textarea[name='vault-body']") as HTMLTextAreaElement, "keep this body");
    clickButton(host, "Import to Vault");
    await flush();
    expect(calls.some((call) => call.pathname === "/management/project/v1/vault.import")).toBe(true);
    expect(calls.some((call) => call.pathname === "/management/project/v1/vault.index.rebuild")).toBe(
      false,
    );
    expect((host.querySelector("input[name='relative_path']") as HTMLInputElement).value).toBe(
      "notes/keep.md",
    );
    expect((host.querySelector("textarea[name='vault-body']") as HTMLTextAreaElement).value).toBe(
      "keep this body",
    );
    expect(host.querySelector("[data-ingest-error='true']")?.textContent).toMatch(/last-write-wins/i);
    expect(host.querySelector("[data-ingest-receipt]")).toBeNull();
    unmount(host, root);
  });

  it("does not POST secret-shaped paste and keeps the original", async () => {
    const { host, root, calls } = await renderKnowledge("#/knowledge");
    setInputValue(
      host.querySelector("textarea[name='vault-body']") as HTMLTextAreaElement,
      "api_key=sk-p12t07-fixture",
    );
    clickButton(host, "Import to Vault");
    await flush();
    expect(calls.some((call) => call.pathname === "/management/project/v1/vault.import")).toBe(false);
    expect((host.querySelector("textarea[name='vault-body']") as HTMLTextAreaElement).value).toBe(
      "api_key=sk-p12t07-fixture",
    );
    expect(host.querySelector("[data-ingest-error='true']")?.textContent).toMatch(/secret-shaped/i);
    unmount(host, root);
  });

  it("does not invent Why this fragment when vault.index is 403", async () => {
    const { host, root, calls } = await renderKnowledge("#/knowledge", {
      "GET /management/project/v1/vault.index": {
        status: 403,
        body: { status: "error", error: { code: "LOCAL_ORIGIN_HEADER_REJECTED", message: "denied" } },
      },
    });
    expect(host.querySelector("[data-region='opc-why-fragment']")?.textContent).toMatch(/session denied/i);
    expect(host.querySelector("[data-row-key='ent-1']")).toBeNull();
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });
});
