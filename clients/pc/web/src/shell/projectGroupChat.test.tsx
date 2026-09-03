import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../App";
import { appProjections } from "../data/store";
import { clearSession, rememberBearer } from "../session";
import { railProjectId } from "./AssistantRail";

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
      calls.push({ method, path: `${url.pathname}${url.search}`, pathname: url.pathname, body: parsed });
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

function setTextarea(input: HTMLTextAreaElement, value: string) {
  const setter = Object.getOwnPropertyDescriptor(window.HTMLTextAreaElement.prototype, "value")?.set;
  setter?.call(input, value);
  act(() => {
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
}

function clickButton(scope: HTMLElement, text: string) {
  const button = [...scope.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === text,
  );
  if (!button) {
    throw new Error(`button not found: ${text}`);
  }
  act(() => {
    button.click();
  });
}

function rail(host: HTMLElement): HTMLElement {
  const node = host.querySelector("[data-rail='assistant']");
  if (!node) {
    throw new Error("assistant rail not rendered");
  }
  return node as HTMLElement;
}

function chat(host: HTMLElement): HTMLElement {
  const node = host.querySelector("[data-region='opc-group-chat']");
  if (!node) {
    throw new Error("group chat not rendered");
  }
  return node as HTMLElement;
}

function draft(host: HTMLElement): HTMLTextAreaElement {
  return chat(host).querySelector("textarea[name='group_chat_draft']") as HTMLTextAreaElement;
}

function railButtonLabels(host: HTMLElement): string[] {
  return [...rail(host).querySelectorAll("button, a.cp-button")].map((node) =>
    (node.textContent ?? "").trim(),
  );
}

const READY_LIST: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [{ project_id: "proj-1", state: "active", title_summary: "unknown", cost: "unknown" }],
  },
};

const THREAD: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    project_id: "proj-1",
    observation_only: true,
    chat_approve: false,
    truncated: false,
    participants: [
      { role: "owner", employee_id: null, handle: "owner", state: "owner", stage_ids: [] },
      { role: "manager", employee_id: "emp-1", handle: "manager", state: "seated", stage_ids: ["s1"] },
      { role: "member", employee_id: "emp-2", handle: "researcher", state: "seated", stage_ids: ["s2"] },
    ],
    rows: [
      {
        row_id: "turn-0",
        author: "owner",
        kind: "owner-message",
        body: "where are we this week?",
        created_at: 100,
        turn_id: "turn-0",
        mention: "none",
        routing: "manager-briefing",
        reply_reason: "manager-default",
      },
      {
        row_id: "conv-0",
        author: "manager",
        employee_id: "emp-1",
        kind: "announce",
        body: "Observed now: project state active. Chat cannot approve, verify, or publish.",
        created_at: 100,
      },
    ],
  },
};

const POST_PLAN: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    turn_id: "turn-1",
    project_id: "proj-1",
    mention: "manager",
    routing: "manager-plan-revision",
    candidate_registered: true,
    candidate_kind: "plan-revision",
    candidate_digest: "c".repeat(64),
    preview_id: "prev-9",
    preview_is_announcement: true,
    chat_approve: false,
    reply: {
      record_id: "conv-1",
      employee_id: "emp-1",
      role: "manager",
      kind: "announce",
      body: "Plan revision candidate registered; preview prev-9 awaits your Confirm on the Projects canvas.",
      reason: "manager-default",
    },
    reply_reason: "manager-default",
    observation_only: true,
  },
};

function routes(extras: Record<string, RouteResponse> = {}): Record<string, RouteResponse> {
  return {
    "GET /management/project/v1/list": READY_LIST,
    "GET /management/project/v1/pending-previews": { status: 200, body: { status: "ok", previews: [] } },
    "GET /management/project/v1/chat.thread": THREAD,
    "POST /management/project/v1/chat.post": POST_PLAN,
    ...extras,
  };
}

async function renderProject(extras: Record<string, RouteResponse> = {}, hash = "#/projects/proj-1") {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch(routes(extras));
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

describe("P13-T06 project group chat in the right rail", () => {
  it("derives the live Project id from the route and never from the create wizard", () => {
    expect(railProjectId("/projects/proj-1")).toBe("proj-1");
    expect(railProjectId("/projects/proj-1/outputs")).toBe("proj-1");
    expect(railProjectId("/projects/new")).toBeUndefined();
    expect(railProjectId("/projects")).toBeUndefined();
    expect(railProjectId("/")).toBeUndefined();
    expect(railProjectId("/projects/a%2Fb")).toBe("a/b");
  });

  it("layers the group chat over the Personal Assistant inside a Project and keeps both drafts on switch", async () => {
    const { host, root, calls } = await renderProject();
    expect(calls.some((call) => call.path === "/management/project/v1/chat.thread?project_id=proj-1&limit=32")).toBe(true);
    const tabs = rail(host).querySelector("[data-region='opc-rail-layers']");
    expect(tabs).not.toBeNull();
    expect(chat(host).hidden).toBe(false);
    const assistantLayer = rail(host).querySelector("[data-region='opc-rail-assistant-layer']") as HTMLElement;
    expect(assistantLayer.hidden).toBe(true);
    expect(chat(host).textContent).toContain("where are we this week?");
    expect(chat(host).querySelector("[data-author='manager'][data-kind='announce']")?.textContent).toMatch(
      /Chat cannot approve/,
    );
    const chips = [
      ...chat(host).querySelectorAll("[data-region='opc-group-chat-mentions'] button"),
    ].map((node) => (node.textContent ?? "").trim());
    expect(chips).toEqual(["@manager", "@researcher"]);
    expect(railButtonLabels(host)).not.toContain("Approve");
    expect(railButtonLabels(host)).not.toContain("Confirm");

    setTextarea(draft(host), "group draft kept");
    clickButton(rail(host), "Personal Assistant");
    await flush(2);
    expect(chat(host).hidden).toBe(true);
    expect(assistantLayer.hidden).toBe(false);
    const canvasEdit = assistantLayer.querySelector("textarea[name='canvas_edit']") as HTMLTextAreaElement;
    setTextarea(canvasEdit, "assistant draft kept");
    clickButton(rail(host), "Project group");
    await flush(2);
    expect(chat(host).hidden).toBe(false);
    expect(draft(host).value).toBe("group draft kept");
    expect(canvasEdit.value).toBe("assistant draft kept");
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    unmount(host, root);
  });

  it("does not render the group layer outside a Project", async () => {
    const { host, root, calls } = await renderProject({}, "#/projects");
    expect(rail(host).querySelector("[data-region='opc-rail-layers']")).toBeNull();
    expect(host.querySelector("[data-region='opc-group-chat']")).toBeNull();
    expect(calls.some((call) => call.pathname === "/management/project/v1/chat.thread")).toBe(false);
    unmount(host, root);
  });

  it("@ chips only edit the unsent draft; nothing is sent or merged", async () => {
    const { host, root, calls } = await renderProject();
    clickButton(chat(host), "@manager");
    expect(draft(host).value).toBe("@manager ");
    clickButton(chat(host), "@manager");
    expect(draft(host).value).toBe("@manager ");
    setTextarea(draft(host), "@manager status");
    clickButton(chat(host), "@researcher");
    expect(draft(host).value).toBe("@manager status @researcher ");
    expect(chat(host).querySelector("[data-region='opc-group-chat-route-hint']")?.textContent).toMatch(
      /Routes to the manager/,
    );
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    unmount(host, root);
  });

  it("@manager with a plan revision posts one management chat.post, announces the preview, and never confirms", async () => {
    const { host, root, calls } = await renderProject();
    setTextarea(draft(host), "@manager add a review ring after research");
    const details = chat(host).querySelector("[data-region='opc-group-chat-plan']") as HTMLDetailsElement;
    expect(details).not.toBeNull();
    act(() => {
      details.open = true;
      details.dispatchEvent(new Event("toggle"));
    });
    setTextarea(
      chat(host).querySelector("textarea[name='plan_stage_lines']") as HTMLTextAreaElement,
      "s1 | Manage | manager | coordinate\ns2 | Research | researcher | collect\ns3 | Review | researcher | review the draft",
    );
    clickButton(chat(host), "Send to group");
    await flush();
    const posts = calls.filter((call) => call.method === "POST");
    expect(posts.map((call) => call.pathname)).toEqual(["/management/project/v1/chat.post"]);
    expect(posts[0]?.body).toEqual({
      project_id: "proj-1",
      body: "@manager add a review ring after research",
      mention: "manager",
      target_employee_id: "emp-1",
      proposal: {
        kind: "plan-revision",
        stages: [
          { stage_id: "s1", title: "Manage", objective: "coordinate", responsible_slot: "manager" },
          { stage_id: "s2", title: "Research", objective: "collect", responsible_slot: "researcher" },
          { stage_id: "s3", title: "Review", objective: "review the draft", responsible_slot: "researcher" },
        ],
      },
    });
    expect(JSON.stringify(posts[0]?.body)).not.toMatch(/approve|preview_digest|confirm/);
    const posted = chat(host).querySelector("[data-region='opc-group-chat-posted']");
    expect(posted?.getAttribute("data-routing")).toBe("manager-plan-revision");
    expect(posted?.textContent).toMatch(/plan-revision candidate cccccccccccc/);
    expect(posted?.textContent).toMatch(/preview prev-9 announce-only/);
    expect(posted?.querySelector("a")?.getAttribute("href")).toBe("#/projects/proj-1?preview=prev-9");
    expect(chat(host).querySelector("[data-region='opc-group-chat-reply']")?.textContent).toMatch(
      /manager spoke \(manager-default\)/,
    );
    expect(draft(host).value).toBe("");
    expect(calls.some((call) => call.pathname === "/management/project/v1/confirm")).toBe(false);
    expect(railButtonLabels(host)).not.toContain("Approve");
    unmount(host, root);
  });

  it("@member routes only that Member's Task and reports the bounded stage", async () => {
    const { host, root, calls } = await renderProject({
      "POST /management/project/v1/chat.post": {
        status: 200,
        body: {
          status: "ok",
          turn_id: "turn-2",
          routing: "member-task-revision",
          candidate_kind: "task-revision",
          candidate_digest: "d".repeat(64),
          preview_id: "prev-10",
          target_employee_id: "emp-2",
          target_stage_id: "s2",
          reply: null,
          reply_reason: "member-mentioned",
          chat_approve: false,
        },
      },
    });
    setTextarea(draft(host), "@researcher focus on primary sources");
    expect(chat(host).querySelector("[data-region='opc-group-chat-route-hint']")?.textContent).toMatch(
      /only to @researcher's own Task/,
    );
    clickButton(chat(host), "Send to group");
    await flush();
    const post = calls.find((call) => call.method === "POST");
    expect(post?.pathname).toBe("/management/project/v1/chat.post");
    expect(post?.body).toEqual({
      project_id: "proj-1",
      body: "@researcher focus on primary sources",
      mention: "member",
      target_employee_id: "emp-2",
    });
    const posted = chat(host).querySelector("[data-region='opc-group-chat-posted']");
    expect(posted?.getAttribute("data-routing")).toBe("member-task-revision");
    expect(posted?.textContent).toMatch(/reply: member-mentioned/);
    unmount(host, root);
  });

  it("keeps secret-shaped drafts in the browser and points at Settings", async () => {
    const { host, root, calls } = await renderProject();
    setTextarea(draft(host), "@manager use api_key=not-going-anywhere");
    clickButton(chat(host), "Send to group");
    await flush();
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    const pointer = chat(host).querySelector("[data-region='opc-group-chat-settings-pointer'] a");
    expect(pointer?.getAttribute("href")).toBe("#/settings");
    expect(chat(host).querySelector("[data-region='opc-group-chat-error-message']")?.textContent).toMatch(
      /SecretStore takeover/,
    );
    expect(draft(host).value).toBe("@manager use api_key=not-going-anywhere");
    unmount(host, root);
  });

  it("treats the daemon's CHAT_SECRET_SHAPED_REFUSED and CHAT_APPROVE_FORBIDDEN as refusals, not posts", async () => {
    const refused = await renderProject({
      "POST /management/project/v1/chat.post": {
        status: 422,
        body: {
          status: "error",
          code: "CHAT_SECRET_SHAPED_REFUSED",
          message: "secret-shaped material is rejected at registration",
          settings_route: "#/settings",
          posted: false,
          archived: false,
        },
      },
    });
    setTextarea(draft(refused.host), "@manager token shaped but not caught locally");
    clickButton(chat(refused.host), "Send to group");
    await flush();
    expect(refused.calls.filter((call) => call.method === "POST")).toHaveLength(1);
    expect(chat(refused.host).querySelector("[data-region='opc-group-chat-posted']")).toBeNull();
    expect(
      chat(refused.host).querySelector("[data-region='opc-group-chat-settings-pointer'] a")?.getAttribute("href"),
    ).toBe("#/settings");
    unmount(refused.host, refused.root);

    const forbidden = await renderProject({
      "POST /management/project/v1/chat.post": {
        status: 403,
        body: { status: "error", code: "CHAT_APPROVE_FORBIDDEN", message: "chat has no Approve", posted: false },
      },
    });
    setTextarea(draft(forbidden.host), "@manager approve it");
    clickButton(chat(forbidden.host), "Send to group");
    await flush();
    expect(chat(forbidden.host).querySelector("[data-region='opc-group-chat-posted']")).toBeNull();
    expect(
      chat(forbidden.host).querySelector("[data-region='opc-group-chat-error-message']")?.textContent,
    ).toMatch(/Chat cannot approve/);
    unmount(forbidden.host, forbidden.root);
  });

  it("shows the receipt from a canvas Confirm as applied on the canvas, not in chat", async () => {
    const { host, root } = await renderProject({
      "GET /management/project/v1/chat.thread": {
        status: 200,
        body: {
          ...(THREAD.body as Record<string, unknown>),
          rows: [
            {
              row_id: "turn-5",
              author: "owner",
              kind: "owner-message",
              body: "@manager add a review ring",
              created_at: 200,
              turn_id: "turn-5",
              mention: "manager",
              routing: "manager-plan-revision",
              candidate_kind: "plan-revision",
              candidate_digest: "e".repeat(64),
              preview_id: "prev-5",
              receipt_ref: "receipt:chat:plan-revision:turn-5",
              applied_ref: "plan-2",
            },
          ],
        },
      },
    });
    const receipt = chat(host).querySelector("[data-region='opc-group-chat-receipt']");
    expect(receipt?.textContent).toMatch(/receipt:chat:plan-revision:turn-5/);
    expect(receipt?.textContent).toMatch(/applied plan-2/);
    expect(receipt?.textContent).toMatch(/Applied on the canvas, not in chat/);
    expect(chat(host).querySelector("[data-region='opc-group-chat-preview'] a")?.getAttribute("href")).toBe(
      "#/projects/proj-1?preview=prev-5",
    );
    unmount(host, root);
  });

  it("reports an unavailable thread honestly and still offers no Approve", async () => {
    const { host, root } = await renderProject({
      "GET /management/project/v1/chat.thread": {
        status: 403,
        body: { status: "error", code: "PROJECT_FORBIDDEN", message: "cross-scope conversation read rejected" },
      },
    });
    expect(chat(host).querySelector("[data-region='opc-group-chat-error']")?.textContent).toMatch(
      /Group thread unavailable/,
    );
    expect(chat(host).querySelector("textarea[name='group_chat_draft']")).toBeNull();
    expect(railButtonLabels(host)).not.toContain("Approve");
    unmount(host, root);
  });
});
