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

const READY_LIST: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [{ project_id: "proj-1", state: "active", title_summary: "unknown", cost: "unknown" }],
  },
};

const PENDING: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    previews: [
      {
        preview_id: "prev-1",
        subject_kind: "activation",
        subject_ref: "proj-1",
        status: "pending",
        preview_digest: "must-not-land",
      },
    ],
  },
};

function detail(status: string, digest = "digest-1"): RouteResponse {
  return {
    status: 200,
    body: {
      preview_id: "prev-1",
      subject_kind: "activation",
      preview_digest: digest,
      status,
      receipt_ref: null,
      superseded_by: null,
    },
  };
}

async function renderCanvas(extras: Record<string, RouteResponse> = {}, hash = "#/projects?preview=prev-1") {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch({
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/project/v1/list": READY_LIST,
    "GET /management/project/v1/pending-previews": PENDING,
    "GET /management/project/v1/preview-detail": detail("pending"),
    "GET /management/project/v1/vault.index": {
      status: 200,
      body: { status: "ok", is_authority: false, entries: [] },
    },
    "GET /management/project/v1/standing-policies": {
      status: 200,
      body: { status: "ok", policies: [] },
    },
    ...extras,
  });
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

describe("P12-T06 HITL canvas Confirm", () => {
  it("posts digest-bound confirm from preview-detail, never from the list digest", async () => {
    const { host, root, calls } = await renderCanvas({
      "POST /management/project/v1/confirm": {
        status: 200,
        body: { status: "ok", receipt_ref: "receipt-1", result: "activated", new_ref: "proj-1" },
      },
    });
    expect(host.querySelector("[data-region='opc-hitl-actions']")).not.toBeNull();
    expect(host.textContent).not.toContain("must-not-land");
    expect(
      [...(host.querySelector("[data-rail='assistant']")?.querySelectorAll("button, a.cp-button") ?? [])].map(
        (node) => (node.textContent ?? "").trim(),
      ),
    ).not.toContain("Approve");
    clickButton(host, "Confirm preview");
    await flush();
    const confirm = calls.find((call) => call.pathname === "/management/project/v1/confirm");
    expect(confirm?.body).toEqual({ preview_id: "prev-1", preview_digest: "digest-1" });
    expect(JSON.stringify(confirm?.body)).not.toContain("must-not-land");
    expect(host.querySelector("[data-region='opc-hitl-written']")?.textContent).toContain("receipt-1");
    expect(host.querySelector("a[href*='#/hitl']")).toBeNull();
    unmount(host, root);
  });

  it("does not confirm a stale preview and does not treat unknown as success", async () => {
    const stale = await renderCanvas({
      "GET /management/project/v1/preview-detail": detail("stale"),
    });
    expect(stale.host.querySelector("[data-hitl-blocked='stale']")).not.toBeNull();
    clickButton(stale.host, "Confirm preview");
    await flush();
    expect(stale.calls.some((call) => call.method === "POST")).toBe(false);
    expect(stale.host.querySelector("[data-region='opc-hitl-written']")).toBeNull();
    unmount(stale.host, stale.root);

    const unknown = await renderCanvas({
      "GET /management/project/v1/preview-detail": {
        status: 404,
        body: { status: "error", error: { code: "PREVIEW_NOT_FOUND", message: "preview not found" } },
      },
    });
    expect(unknown.host.querySelector("[data-hitl-blocked='unknown']")).not.toBeNull();
    clickButton(unknown.host, "Confirm preview");
    await flush();
    expect(unknown.calls.some((call) => call.method === "POST")).toBe(false);
    unmount(unknown.host, unknown.root);
  });

  it("does not confirm when preview-detail is denied", async () => {
    const { host, root, calls } = await renderCanvas({
      "GET /management/project/v1/preview-detail": {
        status: 403,
        body: {
          status: "error",
          error: { code: "LOCAL_ORIGIN_HEADER_REJECTED", message: "denied" },
        },
      },
    });
    expect(host.querySelector("[data-hitl-blocked='denied']")).not.toBeNull();
    clickButton(host, "Confirm preview");
    await flush();
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    unmount(host, root);
  });

  it("keeps a 409 stale confirm as failure, not success", async () => {
    const { host, root, calls } = await renderCanvas({
      "POST /management/project/v1/confirm": {
        status: 409,
        body: {
          status: "error",
          error: { code: "PROJECT_STALE", message: "preview_digest does not match" },
        },
      },
    });
    clickButton(host, "Confirm preview");
    await flush();
    expect(calls.some((call) => call.pathname === "/management/project/v1/confirm")).toBe(true);
    expect(host.querySelector("[data-hitl-error='true']")?.textContent).toMatch(/PROJECT_STALE|409/);
    expect(host.querySelector("[data-region='opc-hitl-written']")).toBeNull();
    unmount(host, root);
  });

  it("posts reject and narrow with the preview-detail digest", async () => {
    const { host, root, calls } = await renderCanvas({
      "POST /management/project/v1/preview.reject": {
        status: 200,
        body: { status: "ok", result: "rejected", receipt_ref: "reject-1" },
      },
      "POST /management/project/v1/preview.narrow": {
        status: 200,
        body: {
          status: "ok",
          preview_id: "prev-2",
          preview_digest: "digest-2",
          superseded_preview_id: "prev-1",
        },
      },
    });
    clickButton(host, "Reject preview");
    await flush();
    expect(calls.find((call) => call.pathname === "/management/project/v1/preview.reject")?.body).toEqual({
      preview_id: "prev-1",
      preview_digest: "digest-1",
    });
    const textarea = host.querySelector("textarea[name='narrow-bytes']") as HTMLTextAreaElement;
    const setter = Object.getOwnPropertyDescriptor(window.HTMLTextAreaElement.prototype, "value")?.set;
    act(() => {
      setter?.call(textarea, "narrowed charter");
      textarea.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await flush();
    clickButton(host, "Narrow preview");
    await flush();
    expect(calls.find((call) => call.pathname === "/management/project/v1/preview.narrow")?.body).toEqual({
      preview_id: "prev-1",
      preview_digest: "digest-1",
      preview_bytes: "narrowed charter",
    });
    const stop = [...host.querySelectorAll("button")].find(
      (candidate) => (candidate.textContent ?? "").trim() === "Stop execution",
    ) as HTMLButtonElement;
    expect(stop.disabled).toBe(true);
    unmount(host, root);
  });

  it("does not offer Confirm on Today packets or chat", async () => {
    const { host, root, calls } = await renderCanvas({}, "#/");
    expect(host.querySelector("[data-page='opc-today']")).not.toBeNull();
    expect(host.querySelector("[data-packet='prev-1']")).not.toBeNull();
    expect(host.querySelector("[data-region='opc-hitl-actions']")).toBeNull();
    expect(host.textContent).toMatch(/announce only/i);
    expect(
      [...(host.querySelector("[data-rail='assistant']")?.querySelectorAll("button") ?? [])].map(
        (node) => (node.textContent ?? "").trim(),
      ),
    ).not.toContain("Approve");
    expect(calls.some((call) => call.pathname === "/management/project/v1/preview-detail")).toBe(
      false,
    );
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    unmount(host, root);
  });
});
