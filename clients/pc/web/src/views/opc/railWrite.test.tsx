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

function railButtons(host: HTMLElement): string[] {
  return [
    ...(host.querySelector("[data-rail='assistant']")?.querySelectorAll("button, a.cp-button") ??
      []),
  ].map((node) => (node.textContent ?? "").trim());
}

const READY_LIST: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [{ project_id: "proj-1", state: "active", title_summary: "unknown", cost: "unknown" }],
  },
};

const EMPTY_LIST: RouteResponse = { status: 200, body: { status: "ok", projects: [] } };

const TURN_OK: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    candidate_id: "cand-1",
    candidate_digest: "digest-1",
    preview_id: "prev-1",
    object_kind: "charter",
    installed_agent: false,
    observation_only: true,
  },
};

const APPLY_OK: RouteResponse = {
  status: 200,
  body: { status: "ok", new_base_seq: 1, payload_digest: "payload-1" },
};

function railRoutes(extras: Record<string, RouteResponse> = {}): Record<string, RouteResponse> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/project/v1/list": READY_LIST,
    "GET /management/project/v1/pending-previews": {
      status: 200,
      body: { status: "ok", previews: [] },
    },
    "POST /management/project/v1/assistant.turn": TURN_OK,
    "POST /management/project/v1/draft.apply": APPLY_OK,
    ...extras,
  };
}

async function renderRail(
  hash = "#/projects",
  extras: Record<string, RouteResponse> = {},
  list: RouteResponse = READY_LIST,
) {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch({
    ...railRoutes(extras),
    "GET /management/project/v1/list": list,
  });
  const view = renderAppAt(hash);
  await flush();
  return { ...view, calls };
}

function fillWrite(host: HTMLElement, text = "charter body") {
  setInputValue(host.querySelector("input[name='draft_id']") as HTMLInputElement, "draft-1");
  setInputValue(host.querySelector("input[name='base_seq']") as HTMLInputElement, "0");
  setInputValue(host.querySelector("textarea[name='canvas_edit']") as HTMLTextAreaElement, text);
}

afterEach(() => {
  clearSession();
  appProjections.clear();
  window.location.hash = "";
  vi.unstubAllGlobals();
});

describe("P12-T09 right-rail canvas write", () => {
  it("whitelists owner management write and refuses archive / task-channel turn", () => {
    expect(isKnownRoute("POST", "/management/project/v1/assistant.turn")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/draft.apply")).toBe(true);
    expect(isKnownRoute("POST", "/task/project/v1/assistant.turn")).toBe(false);
    expect(isKnownRoute("POST", "/management/project/v1/conversation.archive")).toBe(false);
    expect(isKnownRoute("POST", "/management/project/v1/confirm")).toBe(true);
  });

  it("hides the rail on empty home so chat is not a fake Approve surface", async () => {
    const { host, root } = await renderRail("#/", {}, EMPTY_LIST);
    expect(host.querySelector("[data-rail='assistant']")).toBeNull();
    expect(railButtons(host)).not.toContain("Approve");
    unmount(host, root);
  });

  it("walks edit → review → write canvas without Approve or invented draft", async () => {
    const { host, root, calls } = await renderRail();
    expect(host.querySelector("[data-region='opc-rail-write']")).not.toBeNull();
    expect(railButtons(host)).toContain("Review write");
    expect(railButtons(host)).not.toContain("Approve");
    expect(host.querySelector("[data-region='opc-rail-review']")).toBeNull();
    fillWrite(host);
    clickButton(host, "Review write");
    await flush();
    expect(host.querySelector("[data-region='opc-rail-review']")).not.toBeNull();
    expect(host.textContent).toContain("Owner message (local, not archive)");
    expect(railButtons(host)).toContain("Write to canvas");
    expect(railButtons(host)).toContain("Discard");
    expect(railButtons(host)).not.toContain("Approve");
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    clickButton(host, "Write to canvas");
    await flush();
    const posts = calls.filter((call) => call.method === "POST");
    expect(posts.map((call) => call.pathname)).toEqual([
      "/management/project/v1/assistant.turn",
      "/management/project/v1/draft.apply",
    ]);
    expect(posts[0]?.body).toEqual({
      kind: "propose",
      draft_id: "draft-1",
      object_kind: "charter",
      payload: { text: "charter body" },
      provenance: { kind: "owner-stated" },
    });
    expect(posts[1]?.body).toEqual({
      draft_id: "draft-1",
      base_seq: 0,
      candidate_digest: "digest-1",
    });
    expect(host.querySelector("[data-region='opc-rail-written']")?.textContent).toMatch(
      /payload-1/,
    );
    expect(host.querySelector("[data-region='opc-rail-preview-announce']")?.textContent).toMatch(
      /announce-only/,
    );
    expect(
      host.querySelector("[data-region='opc-rail-preview-announce'] a")?.getAttribute("href"),
    ).toBe("#/projects?preview=prev-1");
    expect(calls.some((call) => call.pathname === "/management/project/v1/confirm")).toBe(false);
    expect(calls.some((call) => call.pathname === "/management/project/v1/draft.create")).toBe(
      false,
    );
    expect(
      calls.some((call) => call.pathname === "/management/project/v1/conversation.archive"),
    ).toBe(false);
    unmount(host, root);
  });

  it("uses Enter in the editor to open review, not to bypass it", async () => {
    const { host, root, calls } = await renderRail();
    fillWrite(host);
    const textarea = host.querySelector("textarea[name='canvas_edit']") as HTMLTextAreaElement;
    act(() => {
      textarea.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    });
    await flush();
    expect(host.querySelector("[data-region='opc-rail-review']")).not.toBeNull();
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    unmount(host, root);
  });

  it("discards review without posting", async () => {
    const { host, root, calls } = await renderRail();
    fillWrite(host);
    clickButton(host, "Review write");
    clickButton(host, "Discard");
    await flush();
    expect(host.querySelector("[data-region='opc-rail-review']")).toBeNull();
    expect(railButtons(host)).toContain("Review write");
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    unmount(host, root);
  });

  it("refuses empty identity and secret-shaped paste before any POST", async () => {
    const { host, root, calls } = await renderRail();
    clickButton(host, "Review write");
    await flush();
    expect(host.querySelector("[data-region='opc-rail-write-error']")?.textContent).toMatch(
      /draft_id required/i,
    );
    fillWrite(host, "sk-abcdefghijklmnopqrstuvwxyz");
    clickButton(host, "Review write");
    await flush();
    expect(host.querySelector("[data-region='opc-rail-write-error']")?.textContent).toMatch(
      /secret-shaped/i,
    );
    expect(host.querySelector("[data-region='opc-rail-review']")).toBeNull();
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    unmount(host, root);
  });

  it("does not apply a candidate when assistant.turn is refused", async () => {
    const { host, root, calls } = await renderRail("#/projects", {
      "POST /management/project/v1/assistant.turn": {
        status: 403,
        body: { status: "error", code: "FORBIDDEN", message: "management only" },
      },
    });
    fillWrite(host);
    clickButton(host, "Review write");
    clickButton(host, "Write to canvas");
    await flush();
    expect(calls.filter((call) => call.method === "POST").map((call) => call.pathname)).toEqual([
      "/management/project/v1/assistant.turn",
    ]);
    expect(host.querySelector("[data-region='opc-rail-written']")).toBeNull();
    expect(host.querySelector("[data-region='opc-rail-write-error']")?.textContent).toMatch(
      /Chat cannot Approve/,
    );
    unmount(host, root);
  });

  it("points at Settings and applies nothing when the daemon reports no bound Provider (P13-T03)", async () => {
    const { host, root, calls } = await renderRail("#/projects", {
      "POST /management/project/v1/assistant.turn": {
        status: 409,
        body: {
          status: "provider_unbound",
          code: "ASSISTANT_PROVIDER_UNBOUND",
          message: "no Provider is bound to the assistant; open Settings to connect one",
          chat_input: false,
          silent_bind: false,
          candidate_registered: false,
          settings_route: "#/settings",
        },
      },
    });
    fillWrite(host);
    clickButton(host, "Review write");
    clickButton(host, "Write to canvas");
    await flush();
    expect(calls.filter((call) => call.method === "POST").map((call) => call.pathname)).toEqual([
      "/management/project/v1/assistant.turn",
    ]);
    expect(host.querySelector("[data-region='opc-rail-written']")).toBeNull();
    const pointer = host.querySelector("[data-region='opc-rail-provider-unbound'] a");
    expect(pointer?.getAttribute("href")).toBe("#/settings");
    expect(pointer?.textContent).toMatch(/Open Settings to connect a Provider/);
    expect(railButtons(host)).not.toContain("Approve");
    expect(host.textContent).not.toMatch(/api[_ ]key/i);
    unmount(host, root);
  });

  it("renders the inferred reply and chain kinds after a real turn (P13-T03)", async () => {
    const { host, root } = await renderRail("#/projects", {
      "POST /management/project/v1/assistant.turn": {
        status: 200,
        body: {
          ...(TURN_OK.body as Record<string, unknown>),
          reply: "Candidate charter proposed; nothing is written until you confirm.",
          model_id: "deepseek-chat",
          provider_round_trips: 1,
          chain: [
            { object_kind: "business-brief", fields: { goal: { value: "x", provenance: { kind: "owner-stated" } } } },
            { object_kind: "charter", fields: { title: { value: "y", provenance: { kind: "owner-stated" } } } },
          ],
        },
      },
    });
    fillWrite(host);
    clickButton(host, "Review write");
    clickButton(host, "Write to canvas");
    await flush();
    const reply = host.querySelector("[data-region='opc-rail-assistant-reply']")?.textContent ?? "";
    expect(reply).toMatch(/Candidate charter proposed/);
    expect(reply).toMatch(/deepseek-chat/);
    expect(reply).toMatch(/1 Provider round trip/);
    expect(reply).toMatch(/business-brief → charter/);
    expect(host.querySelector("[data-region='opc-rail-written']")).not.toBeNull();
    unmount(host, root);
  });

  it("does not claim success when draft.apply is refused", async () => {
    const { host, root, calls } = await renderRail("#/projects", {
      "POST /management/project/v1/draft.apply": {
        status: 409,
        body: { status: "error", code: "DRAFT_CAS", message: "stale base_seq" },
      },
    });
    fillWrite(host);
    clickButton(host, "Review write");
    clickButton(host, "Write to canvas");
    await flush();
    expect(calls.filter((call) => call.method === "POST").map((call) => call.pathname)).toEqual([
      "/management/project/v1/assistant.turn",
      "/management/project/v1/draft.apply",
    ]);
    expect(host.querySelector("[data-region='opc-rail-written']")).toBeNull();
    expect(host.querySelector("[data-region='opc-rail-write-error']")?.textContent).toMatch(
      /does not Approve or confirm authority/,
    );
    unmount(host, root);
  });
});
