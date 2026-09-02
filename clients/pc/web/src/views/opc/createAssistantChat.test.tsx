import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { isKnownRoute } from "../../data/normalize";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";
import { defaultObjectKind } from "./CreateAssistantChat";

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

function setSelectValue(select: HTMLSelectElement, value: string) {
  const setter = Object.getOwnPropertyDescriptor(window.HTMLSelectElement.prototype, "value")?.set;
  setter?.call(select, value);
  act(() => {
    select.dispatchEvent(new Event("change", { bubbles: true }));
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

const FAKE_ACTION = /approve|create project|activate|new project|team|inbox|confirm|ingest|apply authority/i;

function fakeActionLabels(host: HTMLElement): string[] {
  return [...host.querySelectorAll("button, a.cp-button")]
    .map((node) => (node.textContent ?? "").trim())
    .filter((label) => FAKE_ACTION.test(label));
}

const STATUS_UNBOUND: RouteResponse = {
  status: 200,
  body: {
    status: "provider_unbound",
    chat_input: false,
    silent_bind: false,
    candidate_registered: false,
    settings_route: "#/settings",
    guidance: "No model is bound to the assistant yet. Open Settings to connect a Provider.",
    installed_agent: false,
  },
};

const STATUS_READY: RouteResponse = {
  status: 200,
  body: {
    status: "ready",
    chat_input: true,
    model_id: "deepseek-chat",
    binding_source: "agent-binding",
    installed_agent: false,
  },
};

const STATUS_PI_UNAVAILABLE: RouteResponse = {
  status: 200,
  body: {
    status: "pi_unavailable",
    chat_input: false,
    model_id: "deepseek-chat",
    pi_detail: "Pi runtime is not configured on this daemon",
  },
};

const TURN_OK: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    candidate_id: "cand-1",
    candidate_digest: "a".repeat(64),
    preview_id: null,
    object_kind: "business-brief",
    model_id: "deepseek-chat",
    provider_round_trips: 1,
    reply: "A weekly client report is a recurring one-page brief; here is a candidate brief.",
    chain: [
      {
        object_kind: "business-brief",
        fields: {
          goal: { value: "weekly client report", provenance: { kind: "owner-stated" } },
          cadence: { value: "weekly", provenance: { kind: "assistant-assumption" } },
        },
      },
    ],
    research: { fetch_family: "HttpFetchReadOnly", fetched: [], refused: [] },
    installed_agent: false,
    observation_only: true,
  },
};

const TURN_UNBOUND: RouteResponse = {
  status: 409,
  body: {
    status: "provider_unbound",
    code: "ASSISTANT_PROVIDER_UNBOUND",
    message: "no Provider is bound to the assistant; open Settings to connect one",
    chat_input: false,
    settings_route: "#/settings",
  },
};

const DRAFT_OK: RouteResponse = {
  status: 200,
  body: { status: "ok", draft_id: "draft-research-1", payload_digest: "p", charter_digest: "c" },
};

function wizardRoutes(extras: Record<string, RouteResponse> = {}): Record<string, RouteResponse> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/project/v1/list": { status: 200, body: { status: "ok", projects: [] } },
    ...extras,
  };
}

async function renderWizard(extras: Record<string, RouteResponse> = {}) {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch(wizardRoutes(extras));
  const view = renderAppAt("#/projects/new");
  await flush();
  return { ...view, calls };
}

afterEach(() => {
  clearSession();
  appProjections.clear();
  window.location.hash = "";
  vi.unstubAllGlobals();
});

describe("P13-T03 create-page assistant chat", () => {
  it("whitelists assistant.status on management only", () => {
    expect(isKnownRoute("GET", "/management/project/v1/assistant.status")).toBe(true);
    expect(isKnownRoute("GET", "/task/project/v1/assistant.status")).toBe(false);
  });

  it("maps wizard steps onto the closed object kinds", () => {
    expect(defaultObjectKind("create-init")).toBe("business-brief");
    expect(defaultObjectKind("create-process")).toBe("axis");
    expect(defaultObjectKind("create-members")).toBe("roster");
    expect(defaultObjectKind("create-test")).toBe("recipe");
    expect(defaultObjectKind("create-joint")).toBe("charter");
  });

  it("renders a Settings pointer instead of a chat box when no Provider is bound", async () => {
    const { host, root, calls } = await renderWizard({
      "GET /management/project/v1/assistant.status": STATUS_UNBOUND,
    });
    const region = host.querySelector("[data-region='opc-create-assistant']");
    expect(region?.getAttribute("data-assistant-state")).toBe("provider-unbound");
    expect(host.querySelector("[data-region='opc-create-assistant-unbound']")).not.toBeNull();
    const pointer = host.querySelector("[data-region='opc-create-assistant-unbound'] a");
    expect(pointer?.getAttribute("href")).toBe("#/settings");
    expect(pointer?.textContent).toMatch(/Open Settings to connect a Provider/);
    expect(host.querySelector("textarea[name='assistant_text']")).toBeNull();
    expect(host.querySelector("[data-region='opc-create-assistant-form']")).toBeNull();
    expect(host.textContent).not.toMatch(/api[_ ]key/i);
    expect(fakeActionLabels(host)).toEqual([]);
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    unmount(host, root);
  });

  it("states the Pi gap honestly when bound but Pi is unavailable", async () => {
    const { host, root } = await renderWizard({
      "GET /management/project/v1/assistant.status": STATUS_PI_UNAVAILABLE,
    });
    expect(
      host.querySelector("[data-region='opc-create-assistant']")?.getAttribute("data-assistant-state"),
    ).toBe("pi-unavailable");
    expect(host.querySelector("[data-region='opc-create-assistant-pi-unavailable']")?.textContent).toMatch(
      /Pi runtime is not configured/,
    );
    expect(host.querySelector("textarea[name='assistant_text']")).toBeNull();
    unmount(host, root);
  });

  it("shows no chat box when assistant.status is unavailable", async () => {
    const { host, root } = await renderWizard();
    expect(
      host.querySelector("[data-region='opc-create-assistant']")?.getAttribute("data-assistant-state"),
    ).toBe("unavailable");
    expect(host.querySelector("textarea[name='assistant_text']")).toBeNull();
    unmount(host, root);
  });

  it("sends an explain turn on a lazily created research draft and renders the typed chain", async () => {
    const { host, root, calls } = await renderWizard({
      "GET /management/project/v1/assistant.status": STATUS_READY,
      "POST /management/project/v1/draft.create": DRAFT_OK,
      "POST /management/project/v1/assistant.turn": TURN_OK,
    });
    expect(
      host.querySelector("[data-region='opc-create-assistant']")?.getAttribute("data-assistant-state"),
    ).toBe("ready");
    expect(host.textContent).toMatch(/deepseek-chat/);
    setInputValue(host.querySelector("input[name='title']") as HTMLInputElement, "Q3 client reports");
    setInputValue(
      host.querySelector("textarea[name='assistant_text']") as HTMLTextAreaElement,
      "what is a weekly client report?",
    );
    clickButton(host, "Send to assistant");
    await flush();
    const posts = calls.filter((call) => call.method === "POST");
    expect(posts.map((call) => call.pathname)).toEqual([
      "/management/project/v1/draft.create",
      "/management/project/v1/assistant.turn",
    ]);
    expect(posts[1]?.body).toEqual({
      kind: "explain",
      draft_id: "draft-research-1",
      object_kind: "business-brief",
      payload: { text: "what is a weekly client report?" },
      provenance: { kind: "owner-stated" },
    });
    expect(host.querySelector("[data-region='opc-create-assistant-reply']")?.textContent).toMatch(
      /candidate brief/,
    );
    expect(host.querySelector("[data-region='opc-create-assistant-reply']")?.textContent).toMatch(
      /1 Provider round trip/,
    );
    const chain = host.querySelector("[data-region='opc-create-assistant-chain']");
    expect(chain?.querySelector("[data-object-kind='business-brief']")).not.toBeNull();
    expect(chain?.querySelector("[data-provenance='owner-stated']")?.textContent).toMatch(/goal/);
    expect(chain?.querySelector("[data-provenance='assistant-assumption']")?.textContent).toMatch(
      /cadence/,
    );
    expect(host.textContent).toMatch(/draft-research-1/);
    expect(fakeActionLabels(host)).toEqual([]);
    expect(calls.some((call) => call.pathname === "/management/project/v1/confirm")).toBe(false);
    expect(calls.some((call) => call.pathname === "/management/project/v1/draft.apply")).toBe(false);

    setInputValue(
      host.querySelector("textarea[name='assistant_text']") as HTMLTextAreaElement,
      "and a second question",
    );
    clickButton(host, "Send to assistant");
    await flush();
    expect(
      calls.filter((call) => call.pathname === "/management/project/v1/draft.create").length,
    ).toBe(1);
    expect(host.querySelectorAll("[data-region='opc-create-assistant-reply']").length).toBe(2);
    unmount(host, root);
  });

  it("posts research targets with the read-only fetch family only for research turns", async () => {
    const { host, root, calls } = await renderWizard({
      "GET /management/project/v1/assistant.status": STATUS_READY,
      "POST /management/project/v1/draft.create": DRAFT_OK,
      "POST /management/project/v1/assistant.turn": TURN_OK,
    });
    setSelectValue(host.querySelector("select[name='assistant_kind']") as HTMLSelectElement, "research");
    await flush(2);
    setInputValue(
      host.querySelector("input[name='assistant_research_targets']") as HTMLInputElement,
      "https://example.invalid/report-format https://other.invalid/x",
    );
    setInputValue(
      host.querySelector("textarea[name='assistant_text']") as HTMLTextAreaElement,
      "how are weekly client reports formatted?",
    );
    clickButton(host, "Send to assistant");
    await flush();
    const turn = calls.find((call) => call.pathname === "/management/project/v1/assistant.turn");
    expect(turn?.body).toMatchObject({
      kind: "research",
      tools: ["HttpFetchReadOnly"],
      research_targets: ["https://example.invalid/report-format", "https://other.invalid/x"],
    });
    unmount(host, root);
  });

  it("refuses empty and secret-shaped text before any POST", async () => {
    const { host, root, calls } = await renderWizard({
      "GET /management/project/v1/assistant.status": STATUS_READY,
    });
    clickButton(host, "Send to assistant");
    await flush();
    expect(host.querySelector("[data-region='opc-create-assistant-error']")?.textContent).toMatch(
      /type a question/i,
    );
    setInputValue(
      host.querySelector("textarea[name='assistant_text']") as HTMLTextAreaElement,
      "sk-abcdefghijklmnopqrstuvwxyz",
    );
    clickButton(host, "Send to assistant");
    await flush();
    expect(host.querySelector("[data-region='opc-create-assistant-error']")?.textContent).toMatch(
      /secret-shaped/i,
    );
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    unmount(host, root);
  });

  it("switches to the Settings pointer when the daemon refuses a turn as provider-unbound", async () => {
    const { host, root, calls } = await renderWizard({
      "GET /management/project/v1/assistant.status": STATUS_READY,
      "POST /management/project/v1/draft.create": DRAFT_OK,
      "POST /management/project/v1/assistant.turn": TURN_UNBOUND,
    });
    setInputValue(
      host.querySelector("textarea[name='assistant_text']") as HTMLTextAreaElement,
      "propose a charter",
    );
    clickButton(host, "Send to assistant");
    await flush();
    expect(
      host.querySelector("[data-region='opc-create-assistant']")?.getAttribute("data-assistant-state"),
    ).toBe("provider-unbound");
    expect(host.querySelector("[data-region='opc-create-assistant-unbound'] a")?.getAttribute("href")).toBe(
      "#/settings",
    );
    expect(host.querySelectorAll("[data-region='opc-create-assistant-reply']").length).toBe(0);
    expect(calls.some((call) => call.pathname === "/management/project/v1/draft.apply")).toBe(false);
    unmount(host, root);
  });

  it("does not render a reply when the daemon refuses the candidate", async () => {
    const { host, root } = await renderWizard({
      "GET /management/project/v1/assistant.status": STATUS_READY,
      "POST /management/project/v1/draft.create": DRAFT_OK,
      "POST /management/project/v1/assistant.turn": {
        status: 422,
        body: {
          status: "error",
          code: "ASSISTANT_CANDIDATE_REFUSED",
          message: "assistant chain field without provenance is refused",
        },
      },
    });
    setInputValue(
      host.querySelector("textarea[name='assistant_text']") as HTMLTextAreaElement,
      "propose a charter",
    );
    clickButton(host, "Send to assistant");
    await flush();
    expect(host.querySelectorAll("[data-region='opc-create-assistant-reply']").length).toBe(0);
    expect(host.querySelector("[data-region='opc-create-assistant-error']")?.textContent).toMatch(
      /ASSISTANT_CANDIDATE_REFUSED/,
    );
    expect(host.querySelector("[data-region='opc-create-assistant-error']")?.textContent).toMatch(
      /No candidate was registered/,
    );
    unmount(host, root);
  });
});
