import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
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
  const list = {
    length: files.length,
    item: (index: number) => files[index] ?? null,
    ...files,
  };
  Object.defineProperty(input, "files", { configurable: true, value: list });
  act(() => {
    input.dispatchEvent(new Event("change", { bubbles: true }));
  });
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

describe("P14-T08 Knowledge v9 files / why / import IA", () => {
  it("locks Knowledge without painting ingest or why tabs when no Project exists", async () => {
    const { host, root, calls } = await renderKnowledge({
      "GET /management/project/v1/list": EMPTY_LIST,
    });
    expect(host.querySelector('[role="tablist"][aria-label="Knowledge"]')).toBeNull();
    expect(host.querySelector("[data-region='opc-vault-ingest']")).toBeNull();
    expect(host.querySelector("[data-region='opc-why-fragment']")).toBeNull();
    expect(host.textContent).toMatch(/locked|no Project/i);
    expect(calls.some((call) => call.pathname === "/management/project/v1/vault.import")).toBe(false);
    expect(host.textContent).not.toMatch(/obsidian/i);
    unmount(host, root);
  });

  it("exposes Files / Import / Why this fragment / Memory instead of an HTTP-paste-only admin form", async () => {
    const { host, root } = await renderKnowledge();
    const tabs = [...host.querySelectorAll('[role="tablist"][aria-label="Knowledge"] [role="tab"]')].map(
      (node) => (node.textContent ?? "").trim(),
    );
    expect(tabs).toEqual(["Files", "Import", "Why this fragment", "Memory"]);
    expect(host.querySelector("[data-region='opc-knowledge-files']")).not.toBeNull();
    expect(host.querySelector("[data-region='opc-vault-ingest']")).toBeNull();
    expect(host.querySelector("[data-region='opc-why-fragment']")).toBeNull();
    expect(host.textContent).toMatch(/No files yet/);
    expect(host.textContent).not.toMatch(/obsidian/i);
    clickButton(host, "Import files");
    expect(host.querySelector("[data-region='opc-vault-ingest']")).not.toBeNull();
    expect(host.querySelector('input[name="vault-files"]')).not.toBeNull();
    unmount(host, root);
  });

  it("reads Why this fragment on its own tab from daemon inject_order and excerpts", async () => {
    const { host, root } = await renderKnowledge();
    clickTab(host, "Why this fragment");
    expect(host.querySelector("[data-region='opc-why-fragment']")).not.toBeNull();
    expect(host.querySelector("[data-row-key='ent-1']")?.textContent).toMatch(/note/);
    expect(host.textContent).toContain("fixed-decision");
    expect(host.querySelector("[data-region='opc-vault-ingest']")).toBeNull();
    unmount(host, root);
  });

  it("imports a picked Markdown file through vault.import as markdown-file, not as Project authority", async () => {
    const { host, root, calls } = await renderKnowledge({
      "POST /management/project/v1/vault.import": {
        status: 200,
        body: {
          status: "ok",
          document_id: "doc-file",
          is_authority: false,
          host_fs_e2e: "not-run",
        },
      },
      "POST /management/project/v1/vault.index.rebuild": {
        status: 200,
        body: { status: "ok", written: 1, memory_fts: "untouched" },
      },
    });
    clickTab(host, "Import");
    const input = host.querySelector('input[name="vault-files"]') as HTMLInputElement;
    setFiles(input, [new File(["# picked note\n"], "brief.md", { type: "text/markdown" })]);
    await flush();
    clickButton(host, "Import to Vault");
    await flush();
    const imported = calls.find((call) => call.pathname === "/management/project/v1/vault.import");
    expect(imported?.body).toEqual({
      project_id: "proj-1",
      relative_path: "inbox/brief.md",
      rights_class: "owner-owned",
      provenance: { source_uri: "file:brief.md" },
      source_kind: "markdown-file",
      body: "# picked note\n",
    });
    expect(calls.some((call) => call.pathname === "/management/project/v1/vault.apply-authority")).toBe(
      false,
    );
    expect(host.querySelector("[data-ingest-receipt='doc-file']")).not.toBeNull();
    unmount(host, root);
  });

  it("refuses secret-shaped file ingest and keeps the original file selection", async () => {
    const { host, root, calls } = await renderKnowledge();
    clickTab(host, "Import");
    const input = host.querySelector('input[name="vault-files"]') as HTMLInputElement;
    const secretFile = new File(["api_key=sk-p14t08-fixture"], "secret.md", { type: "text/markdown" });
    setFiles(input, [secretFile]);
    await flush();
    clickButton(host, "Import to Vault");
    await flush();
    expect(calls.some((call) => call.pathname === "/management/project/v1/vault.import")).toBe(false);
    expect((host.querySelector('input[name="vault-files"]') as HTMLInputElement).files?.[0]?.name).toBe(
      "secret.md",
    );
    expect(host.querySelector("[data-ingest-error='true']")?.textContent).toMatch(/secret-shaped/i);
    unmount(host, root);
  });

  it("refuses a Vault file that claims is_authority and does not rebuild", async () => {
    const { host, root, calls } = await renderKnowledge({
      "POST /management/project/v1/vault.import": {
        status: 200,
        body: { status: "ok", document_id: "doc-auth", is_authority: true },
      },
    });
    clickTab(host, "Import");
    const input = host.querySelector('input[name="vault-files"]') as HTMLInputElement;
    setFiles(input, [new File(["not authority"], "charter.md", { type: "text/markdown" })]);
    await flush();
    clickButton(host, "Import to Vault");
    await flush();
    expect(calls.some((call) => call.pathname === "/management/project/v1/vault.import")).toBe(true);
    expect(calls.some((call) => call.pathname === "/management/project/v1/vault.index.rebuild")).toBe(
      false,
    );
    expect(host.querySelector("[data-ingest-error='true']")?.textContent).toMatch(/not Project authority/i);
    expect(host.querySelector("[data-ingest-receipt]")).toBeNull();
    unmount(host, root);
  });

  it("keeps Memory auto-admission honest with zero Admit buttons", async () => {
    const { host, root, calls } = await renderKnowledge();
    clickTab(host, "Memory");
    const auto = host.querySelector("[data-region='opc-knowledge-auto-admit']");
    expect(auto?.textContent).toMatch(/Requires-backend/);
    expect(
      [...host.querySelectorAll("button")].some((node) => /\bAdmit\b/.test(node.textContent ?? "")),
    ).toBe(false);
    expect(calls.some((call) => call.pathname === "/management/resource/v1/memory/auto-admit.chat")).toBe(
      false,
    );
    unmount(host, root);
  });
});
