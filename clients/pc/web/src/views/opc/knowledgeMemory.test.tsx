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

async function flush(ticks = 24) {
  for (let i = 0; i < ticks; i += 1) {
    await act(async () => {
      await Promise.resolve();
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

const LABELED_OK: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    is_authority: false,
    entries: [
      {
        entry_id: "ent-owned",
        document_id: "doc-owned",
        relative_path: "notes/owned.md",
        excerpt: "Owned excerpt",
        layer: "sourced-excerpt",
        provenance_source_uri: "owner-paste:owned",
        rights_class: "owner-owned",
        freshness: "current",
        exclusion: "included",
        exclusion_reason: "",
        untrusted_observation: false,
        is_authority: false,
      },
      {
        entry_id: "ent-cite",
        document_id: "doc-cite",
        relative_path: "notes/cite.md",
        excerpt: "Cite excerpt",
        layer: "sourced-excerpt",
        provenance_source_uri: "https://example.invalid/cite",
        rights_class: "citation-only",
        freshness: "current",
        exclusion: "excluded",
        exclusion_reason: "citation-only",
        untrusted_observation: true,
        is_authority: false,
      },
    ],
  },
};

function knowledgeRoutes(extras: Record<string, RouteResponse> = {}): Record<string, RouteResponse> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/project/v1/list": {
      status: 200,
      body: {
        status: "ok",
        projects: [
          { project_id: "proj-1", state: "active", title_summary: "one", cost: "unknown" },
          { project_id: "proj-2", state: "active", title_summary: "two", cost: "unknown" },
        ],
      },
    },
    "GET /management/project/v1/vault.index": {
      status: 200,
      body: { status: "ok", is_authority: false, inject_order: [], entries: [] },
    },
    "GET /management/project/v1/vault.conflicts": { status: 200, body: { status: "ok", conflicts: [] } },
    "GET /management/resource/v1/list": {
      status: 200,
      body: {
        status: "ok",
        family: "memory",
        resources: [{ id: "mem-1", family: "memory", health: "ok" }],
      },
    },
    "GET /management/resource/v1/vault.labeled": LABELED_OK,
    "GET /management/resource/v1/vault.documents": {
      status: 200,
      body: {
        status: "ok",
        is_authority: false,
        documents: [
          {
            document_id: "doc-pending",
            relative_path: "notes/pending.md",
            provenance_source_uri: "owner-paste:pending",
            index_status: "not-indexed",
          },
        ],
      },
    },
    "GET /management/resource/v1/memory/promotes": { status: 200, body: { status: "ok", promotes: [] } },
    "GET /management/resource/v1/inspect": {
      status: 200,
      body: { status: "ok", resource: { id: "mem-1", family: "memory", health: "ok" } },
    },
    "GET /management/resource/v1/memory/object": {
      status: 200,
      body: { memory_id: "mem-1", canonical_json: "{\"text\":\"admitted note\"}" },
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

describe("P13-T07 Knowledge labels and Memory authority", () => {
  it("does not require new labeled routes on KNOWN_ROUTES", () => {
    expect(isKnownRoute("GET", "/management/resource/v1/vault.labeled")).toBe(false);
    expect(isKnownRoute("GET", "/management/resource/v1/vault.documents")).toBe(false);
    expect(isKnownRoute("POST", "/management/resource/v1/memory/promote.request")).toBe(false);
    expect(isKnownRoute("POST", "/management/resource/v1/memory/forget")).toBe(true);
  });

  it("renders provenance/rights/freshness/exclusion and not-indexed originals", async () => {
    const { host, root } = await renderKnowledge();
    const labels = host.querySelector("[data-region='opc-knowledge-labels']");
    expect(labels?.textContent).toMatch(/owner-owned/);
    expect(labels?.textContent).toMatch(/citation-only/);
    expect(labels?.textContent).toMatch(/excluded/);
    expect(labels?.textContent).toMatch(/untrusted/i);
    expect(host.querySelector("[data-region='opc-knowledge-documents']")?.textContent).toMatch(
      /not-indexed/,
    );
    expect(host.textContent).not.toMatch(/apply authority/i);
    unmount(host, root);
  });

  it("keeps chat auto-admission honest empty with no Admit button", async () => {
    const { host, root, calls } = await renderKnowledge();
    const auto = host.querySelector("[data-region='opc-knowledge-auto-admit']");
    expect(auto?.textContent).toMatch(/Requires-backend/);
    expect(auto?.querySelector("button")).toBeNull();
    expect(auto?.textContent).not.toMatch(/\bAdmit\b/);
    expect(
      [...host.querySelectorAll("button")].some((node) => /\bAdmit\b/.test(node.textContent ?? "")),
    ).toBe(false);
    expect(calls.some((call) => call.pathname === "/management/resource/v1/memory/auto-admit.chat")).toBe(
      false,
    );
    unmount(host, root);
  });

  it("inspects Memory then posts correct on management HTTP", async () => {
    const { host, root, calls } = await renderKnowledge({
      "POST /management/resource/v1/memory/correct": {
        status: 200,
        body: { status: "ok", memory_id: "mem-2" },
      },
    });
    const select = host.querySelector("select[name='memory_id']") as HTMLSelectElement;
    act(() => {
      select.value = "mem-1";
      select.dispatchEvent(new Event("change", { bubbles: true }));
    });
    await flush();
    expect(host.querySelector("[data-memory-canonical='true']")?.textContent).toMatch(/admitted note/);
    setInputValue(host.querySelector("input[name='employee_id']") as HTMLInputElement, "emp-1");
    setInputValue(host.querySelector("textarea[name='correct_text']") as HTMLTextAreaElement, "fixed");
    clickButton(host, "Correct Memory");
    await flush();
    const posted = calls.find((call) => call.pathname === "/management/resource/v1/memory/correct");
    expect(posted?.body).toEqual({
      memory_id: "mem-1",
      project_id: "proj-1",
      employee_id: "emp-1",
      text: "fixed",
    });
    unmount(host, root);
  });

  it("requests a cross-Project promote preview without inventing a copy", async () => {
    const { host, root, calls } = await renderKnowledge({
      "POST /management/resource/v1/memory/promote.request": {
        status: 201,
        body: {
          status: "pending",
          promote_id: "prm-1",
          memory_id: "mem-1",
          from_project_id: "proj-1",
          to_project_id: "proj-2",
          preview_digest: "deadbeef",
        },
      },
    });
    const memory = host.querySelector("select[name='memory_id']") as HTMLSelectElement;
    act(() => {
      memory.value = "mem-1";
      memory.dispatchEvent(new Event("change", { bubbles: true }));
    });
    const target = host.querySelector("select[name='to_project_id']") as HTMLSelectElement;
    act(() => {
      target.value = "proj-2";
      target.dispatchEvent(new Event("change", { bubbles: true }));
    });
    setInputValue(host.querySelector("input[name='to_employee_id']") as HTMLInputElement, "emp-2");
    clickButton(host, "Request promote preview");
    await flush();
    const posted = calls.find((call) => call.pathname === "/management/resource/v1/memory/promote.request");
    expect(posted?.body).toEqual({
      memory_id: "mem-1",
      from_project_id: "proj-1",
      to_project_id: "proj-2",
      to_employee_id: "emp-2",
    });
    expect(host.textContent).toMatch(/does not yet have a copy/i);
    expect(host.querySelector("[data-region='opc-knowledge-memory']")?.textContent).toMatch(
      /deadbeef/,
    );
    unmount(host, root);
  });
});
