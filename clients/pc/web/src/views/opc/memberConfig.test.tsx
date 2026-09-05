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

const FAKE_ACTION = /approve|create project|activate|new project|team|inbox|confirm|ingest|apply authority|install/i;

function fakeActionLabels(host: HTMLElement): string[] {
  const scopes = [
    host.querySelector("#main"),
    host.querySelector("[data-rail='assistant']"),
    host.querySelector("nav[aria-label='Primary']"),
    host.querySelector("nav[aria-label='Project sections']"),
  ].filter((node): node is Element => node !== null);
  const labels: string[] = [];
  for (const scope of scopes) {
    for (const node of scope.querySelectorAll("button, a.cp-button")) {
      const label = (node.textContent ?? "").trim();
      if (node.closest("[data-region='opc-rail-write']")) {
        continue;
      }
      if (FAKE_ACTION.test(label)) {
        labels.push(label);
      }
    }
  }
  return labels;
}

const READY_LIST: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [{ project_id: "proj-1", state: "active", title_summary: "unknown", cost: "unknown" }],
  },
};

const READY_DETAIL: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    project: {
      project_id: "proj-1",
      state: "active",
      created_at: "t0",
      activated_at: "t1",
      accepted_at: null,
    },
    charter: { status: "confirmed", content_digest: "dig-1" },
    plan: { plan_revision_id: "plan-1" },
    pending_preview_count: 0,
    cost: "unknown",
  },
};

const SLOTTED_AXIS: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    plan_revision_id: "plan-1",
    stages: [
      {
        stage_id: "st-1",
        position: 0,
        title: "Intake",
        responsible_slot: "manager",
        confirm_status: "confirmed",
        ready: false,
        seated: false,
        output_contract: {
          digest: "unknown",
          deliverable_type: "unknown",
          save_format: "unknown",
          open_with: "unknown",
        },
        gaps: [],
      },
    ],
  },
};

const EMPTY_ROSTER: RouteResponse = {
  status: 200,
  body: { status: "ok", authority_note: "empty-roster", roster: [] },
};

const READY_ROSTER: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    authority_note: "employee",
    roster: [
      {
        employee_id: "emp-1",
        state: "seated",
        model_bound: true,
        is_current_manager: true,
        runtime_binding_ref: "run-1",
        responsible_stage_ids: ["st-1"],
      },
    ],
  },
};

const EMPTY_CATALOG: RouteResponse = {
  status: 200,
  body: { status: "ok", catalog: [] },
};

function memberRoutes(extras: Record<string, RouteResponse> = {}): Record<string, RouteResponse> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/project/v1/list": READY_LIST,
    "GET /management/project/v1/detail": READY_DETAIL,
    "GET /management/project/v1/axis": SLOTTED_AXIS,
    "GET /management/project/v1/roster": EMPTY_ROSTER,
    "GET /management/project/v1/employee.catalog": EMPTY_CATALOG,
    "GET /management/project/v1/pending-previews": { status: 200, body: { status: "ok", previews: [] } },
    ...extras,
  };
}

async function renderMember(hash: string, extras: Record<string, RouteResponse> = {}) {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch(memberRoutes(extras));
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

describe("P12-T04 select-then-configure + add member", () => {
  it("whitelists catalog and seating Intent routes", () => {
    expect(isKnownRoute("GET", "/management/project/v1/employee.catalog")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/roster.register")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/employee.seat.request")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/employee.seat.confirm")).toBe(true);
  });

  it("does not configure a member that is not on the current roster", async () => {
    const { host, root } = await renderMember("#/projects/proj-1/members/emp-missing");
    expect(host.querySelector("[data-page='opc-member-config']")).not.toBeNull();
    expect(host.textContent).toMatch(/not on this roster/i);
    expect(host.querySelector("[role='tablist']")).toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("opens duty through reflection tabs after a roster row is selected and does not Install", async () => {
    const { host, root, calls } = await renderMember("#/projects/proj-1/members/emp-1", {
      "GET /management/project/v1/roster": READY_ROSTER,
    });
    expect(host.querySelector("[data-page='opc-member-config']")).not.toBeNull();
    const tabs = [...host.querySelectorAll("[role='tab']")].map((node) => (node.textContent ?? "").trim());
    expect(tabs).toEqual([
      "Duty",
      "Input",
      "Output",
      "Skills",
      "Tools",
      "Brief",
      "Loop",
      "Perms",
      "Reflection",
    ]);
    expect(host.querySelector("input[name='budget']")).toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    clickButton(host, "Skills");
    await flush();
    expect(host.textContent).toMatch(/recipe mention/i);
    expect(calls.some((call) => call.pathname === "/management/project/v1/employee.catalog")).toBe(
      true,
    );
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("refuses an unreviewed acquire and does not mint a local grant", async () => {
    const { host, root, calls } = await renderMember("#/projects/proj-1/members/emp-1", {
      "GET /management/project/v1/roster": READY_ROSTER,
    });
    clickButton(host, "Skills");
    await flush();
    expect(host.querySelector("[data-region='opc-capability-acquire']")).not.toBeNull();
    clickButton(host, "Request acquire preview");
    await flush();
    expect(host.querySelector("[data-acquire-error='true']")?.textContent).toMatch(/unreviewed/i);
    expect(calls.some((call) => call.pathname === "/management/project/v1/capability.acquire")).toBe(
      false,
    );
    expect(host.querySelector("[data-region='opc-acquire-previewed']")).toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("posts capability.acquire then deep-links the canvas without Activate", async () => {
    const { host, root, calls } = await renderMember("#/projects/proj-1/members/emp-1", {
      "GET /management/project/v1/roster": READY_ROSTER,
      "POST /management/project/v1/capability.acquire": {
        status: 200,
        body: {
          status: "ok",
          preview_id: "prev-grant-1",
          preview_digest: "digest-1",
          granted: false,
          install_is_not_grant: true,
        },
      },
    });
    clickButton(host, "Skills");
    await flush();
    const panel = host.querySelector("[data-region='opc-capability-acquire']");
    expect(panel).not.toBeNull();
    for (const input of panel?.querySelectorAll("input") ?? []) {
      const current = (input as HTMLInputElement).value;
      if (current.length === 0) {
        act(() => {
          const native = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value");
          native?.set?.call(input, "https://example.invalid/skill/research");
          input.dispatchEvent(new Event("input", { bubbles: true }));
        });
      }
    }
    clickButton(host, "Request acquire preview");
    await flush();
    const acquire = calls.find((call) => call.pathname === "/management/project/v1/capability.acquire");
    expect(acquire?.body).toMatchObject({
      project_id: "proj-1",
      employee_id: "emp-1",
      phase: "install",
    });
    expect(host.querySelector("[data-region='opc-acquire-previewed']")?.textContent).toMatch(
      /prev-grant-1/,
    );
    expect(host.querySelector("a[href*='preview=prev-grant-1']")).not.toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("posts reflection.generate then runtime preview and never Admit", async () => {
    const { host, root, calls } = await renderMember("#/projects/proj-1/members/emp-1", {
      "GET /management/project/v1/roster": READY_ROSTER,
      "POST /management/project/v1/reflection.generate": {
        status: 200,
        body: {
          status: "ok",
          generated: [
            {
              candidate_id: "cand-1",
              kind: "incident",
              source: "attempt-terminal",
              employee_id: "emp-1",
              completion_claimed: false,
            },
          ],
        },
      },
      "GET /management/project/v1/reflection.list": {
        status: 200,
        body: {
          status: "ok",
          candidates: [
            {
              candidate_id: "cand-1",
              kind: "incident",
              source: "attempt-terminal",
              employee_id: "emp-1",
              completion_claimed: false,
            },
          ],
        },
      },
      "POST /management/project/v1/reflection.improve.propose": {
        status: 200,
        body: {
          status: "ok",
          preview_id: "prev-runtime-1",
          preview_digest: "digest-runtime",
          improvement_id: "improve-1",
          state: "preview",
          granted: false,
        },
      },
    });
    clickButton(host, "Reflection");
    await flush();
    expect(host.querySelector("[data-region='opc-member-reflection']")).not.toBeNull();
    expect(host.textContent).toMatch(/cannot apply a revision/i);
    clickButton(host, "Generate from facts");
    await flush();
    expect(
      calls.some((call) => call.pathname === "/management/project/v1/reflection.generate"),
    ).toBe(true);
    const prompt = host.querySelector("[data-region='opc-member-reflection'] input:not([type='radio'])");
    expect(prompt).not.toBeNull();
    act(() => {
      const native = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value");
      native?.set?.call(prompt, "tighten research");
      prompt?.dispatchEvent(new Event("input", { bubbles: true }));
    });
    clickButton(host, "Request runtime preview");
    await flush();
    const propose = calls.find(
      (call) => call.pathname === "/management/project/v1/reflection.improve.propose",
    );
    expect(propose?.body).toMatchObject({
      candidate_id: "cand-1",
      proposed_prompt: "tighten research",
    });
    expect(host.querySelector("[data-region='opc-reflection-previewed']")?.textContent).toMatch(
      /prev-runtime-1/,
    );
    expect(host.querySelector("a[href*='preview=prev-runtime-1']")).not.toBeNull();
    expect(host.textContent).not.toMatch(/\bAdmit\b/);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("posts register then seat.request then seat.confirm on Write join", async () => {
    const { host, root, calls } = await renderMember("#/projects/proj-1/members/new", {
      "POST /management/project/v1/roster.register": {
        status: 200,
        body: { status: "ok", employee_ids: ["emp-new"] },
      },
      "POST /management/project/v1/employee.seat.request": {
        status: 200,
        body: { status: "ok", state: "seating" },
      },
      "POST /management/project/v1/employee.seat.confirm": {
        status: 200,
        body: { status: "ok", state: "seated" },
      },
    });
    expect(host.querySelector("[data-page='opc-add-member']")).not.toBeNull();
    expect(host.textContent).toMatch(/empty-roster/);
    expect(fakeActionLabels(host)).toEqual([]);
    clickButton(host, "Write join");
    await flush();
    const register = calls.find((call) => call.pathname === "/management/project/v1/roster.register");
    expect(register?.body).toEqual({
      project_id: "proj-1",
      plan_revision_id: "plan-1",
      proposals: [{ slot: "manager", specialization: "member", prompt: "", tools_declared: [] }],
    });
    expect(calls.some((call) => call.pathname === "/management/project/v1/employee.seat.request")).toBe(
      true,
    );
    const confirm = calls.find(
      (call) => call.pathname === "/management/project/v1/employee.seat.confirm",
    );
    expect(confirm?.body).toEqual({ employee_id: "emp-new", accept: true });
    expect(host.querySelector("[data-region='opc-join-written']")?.textContent).toContain("emp-new");
    unmount(host, root);
  });

  it("does not mint a member when roster.register is rejected", async () => {
    const { host, root, calls } = await renderMember("#/projects/proj-1/members/new", {
      "POST /management/project/v1/roster.register": {
        status: 409,
        body: {
          status: "error",
          error: {
            code: "PROJECT_REJECTED",
            message: "roster has surplus member without a slot",
          },
        },
      },
    });
    clickButton(host, "Write join");
    await flush();
    expect(host.querySelector("[data-join-error='true']")?.textContent).toMatch(/PROJECT_REJECTED/);
    expect(host.querySelector("[data-region='opc-join-written']")).toBeNull();
    expect(calls.some((call) => call.pathname === "/management/project/v1/employee.seat.request")).toBe(
      false,
    );
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("does not POST when Refuse join is used before a register", async () => {
    const { host, root, calls } = await renderMember("#/projects/proj-1/members/new");
    clickButton(host, "Refuse join");
    await flush();
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    expect(host.querySelector("[data-region='opc-join-written']")).toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("does not POST register when the axis has no responsible slots", async () => {
    const { host, root, calls } = await renderMember("#/projects/proj-1/members/new", {
      "GET /management/project/v1/axis": {
        status: 200,
        body: {
          status: "ok",
          plan_revision_id: "plan-1",
          stages: [{ stage_id: "st-1", title: "Intake", gaps: [] }],
        },
      },
    });
    clickButton(host, "Write join");
    await flush();
    expect(host.querySelector("[data-join-error='true']")?.textContent).toMatch(/responsible slots/i);
    expect(calls.some((call) => call.pathname === "/management/project/v1/roster.register")).toBe(
      false,
    );
    unmount(host, root);
  });
});
