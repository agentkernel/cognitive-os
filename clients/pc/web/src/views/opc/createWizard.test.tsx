import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";
import { PROCESS_STAGES, RUNTIME_SLOTS } from "./createWizardModel";

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
  act(() => {
    select.value = value;
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

function button(host: HTMLElement, text: string): HTMLButtonElement | undefined {
  return [...host.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === text,
  ) as HTMLButtonElement | undefined;
}

const FAKE_ACTION = /approve|create project|activate|new project|team|inbox|confirm|ingest|apply authority/i;

function fakeActionLabels(host: HTMLElement): string[] {
  return [...host.querySelectorAll("button, a.cp-button")]
    .map((node) => (node.textContent ?? "").trim())
    .filter((label) => FAKE_ACTION.test(label));
}

const EMPTY_LIST: RouteResponse = { status: 200, body: { status: "ok", projects: [] } };

function wizardRoutes(extras: Record<string, RouteResponse> = {}): Record<string, RouteResponse> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/project/v1/list": EMPTY_LIST,
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

function fillCharter(host: HTMLElement) {
  setInputValue(host.querySelector("input[name='title']") as HTMLInputElement, "Q3 charter");
  setInputValue(
    host.querySelector("textarea[name='charter']") as HTMLTextAreaElement,
    "owner charter body",
  );
  clickButton(host, "Continue");
}

function confirmProcessAxis(host: HTMLElement) {
  for (let i = 0; i < PROCESS_STAGES.length; i += 1) {
    clickButton(host, "确认这一环");
  }
  clickButton(host, "确认总目标与项目触发");
  clickButton(host, "进入 ③");
}

function seatAllMembers(host: HTMLElement) {
  clickButton(host, "创建岗位");
  for (let i = 0; i < PROCESS_STAGES.length; i += 1) {
    const select = host.querySelector("select[name='member-model']") as HTMLSelectElement;
    setSelectValue(select, "draft-bound");
    for (const slot of RUNTIME_SLOTS) {
      const box = host.querySelector(`input[name='slot-${slot.id}']`) as HTMLInputElement;
      if (!box.checked) {
        act(() => {
          box.click();
        });
      }
    }
    clickButton(host, "确认就位");
  }
  clickButton(host, "进入 ④");
}

function passAllStageTests(host: HTMLElement) {
  PROCESS_STAGES.forEach((_stage, index) => {
    clickButton(host, "开始测");
    clickButton(host, "记录通过");
    const last = index === PROCESS_STAGES.length - 1;
    clickButton(host, last ? "末环通过，进入 ⑤" : "通过，下一环");
  });
}

function walkToJoint(host: HTMLElement) {
  fillCharter(host);
  confirmProcessAxis(host);
  seatAllMembers(host);
  passAllStageTests(host);
  clickButton(host, "开始联调");
  clickButton(host, "核对通过");
}

afterEach(() => {
  clearSession();
  appProjections.clear();
  window.location.hash = "";
  vi.unstubAllGlobals();
});

describe("P14-T02 Dual Track create wizard", () => {
  it("replaces note textareas with process axis / seating / openable tests and has no fake Activate chrome", async () => {
    const { host, root } = await renderWizard();
    expect(host.querySelector("[data-page='opc-create-wizard']")).not.toBeNull();
    expect(host.querySelector("[data-step='create-init']")).not.toBeNull();
    expect(host.querySelector("[data-rail='assistant']")).toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    fillCharter(host);
    expect(host.querySelector("[data-step='create-process']")).not.toBeNull();
    expect(host.querySelector("textarea[name='process']")).toBeNull();
    expect(host.querySelector("[data-process-axis]")).not.toBeNull();
    expect(host.querySelectorAll("[data-ring]")).toHaveLength(PROCESS_STAGES.length);
    confirmProcessAxis(host);
    expect(host.querySelector("[data-step='create-members']")).not.toBeNull();
    expect(host.querySelector("textarea[name='members']")).toBeNull();
    seatAllMembers(host);
    expect(host.querySelector("[data-step='create-test']")).not.toBeNull();
    expect(host.querySelector("textarea[name='verification']")).toBeNull();
    passAllStageTests(host);
    expect(host.querySelector("[data-step='create-joint']")).not.toBeNull();
    expect(host.querySelector("[data-openable-result]")).not.toBeNull();
    expect(host.textContent).toMatch(/Request preview/);
    expect(host.textContent).toMatch(/Write Project/);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("does not enter ③ until every ring is resolved and total goal is confirmed", async () => {
    const { host, root } = await renderWizard();
    fillCharter(host);
    expect(button(host, "进入 ③")).toBeUndefined();
    clickButton(host, "确认这一环");
    expect(host.querySelector("[data-step='create-process']")).not.toBeNull();
    expect(button(host, "进入 ③")).toBeUndefined();
    unmount(host, root);
  });

  it("refuses seating without a model and blocks ④ until every member is seated", async () => {
    const { host, root } = await renderWizard();
    fillCharter(host);
    confirmProcessAxis(host);
    clickButton(host, "创建岗位");
    const seat = button(host, "确认就位");
    expect(seat?.disabled).toBe(true);
    expect(button(host, "进入 ④")?.disabled).toBe(true);
    unmount(host, root);
  });

  it("blocks start/pass when the responsible member is not seated, and unknown cannot pass", async () => {
    const { host, root } = await renderWizard();
    fillCharter(host);
    confirmProcessAxis(host);
    expect(host.querySelector("[data-step='create-test']")).toBeNull();
    seatAllMembers(host);
    clickButton(host, "开始测");
    clickButton(host, "记录说不清");
    expect(button(host, "通过，下一环")?.disabled).toBe(true);
    expect(host.textContent).toMatch(/说不清|不能通过/);
    unmount(host, root);
  });

  it("posts draft.create then preview.request then confirm, and does not mint locally on daemon reject", async () => {
    const { host, root, calls } = await renderWizard({
      "POST /management/project/v1/draft.create": {
        status: 200,
        body: { status: "ok", draft_id: "draft-1", payload_digest: "p", charter_digest: "c" },
      },
      "POST /management/project/v1/preview.request": {
        status: 200,
        body: { status: "ok", preview_id: "prev-1", preview_digest: "digest-1" },
      },
      "POST /management/project/v1/confirm": {
        status: 200,
        body: { status: "ok", new_ref: "proj-new", receipt_ref: "r1", result: "activation" },
      },
    });
    walkToJoint(host);
    expect(button(host, "Write Project")?.disabled).toBe(true);
    clickButton(host, "Request preview");
    await flush();
    expect(calls.some((call) => call.pathname === "/management/project/v1/draft.create")).toBe(
      true,
    );
    expect(calls.some((call) => call.pathname === "/management/project/v1/preview.request")).toBe(
      true,
    );
    const preview = calls.find((call) => call.pathname === "/management/project/v1/preview.request");
    expect(preview?.body).toEqual({ subject_kind: "activation", subject_ref: "draft-1" });
    const created = calls.find((call) => call.pathname === "/management/project/v1/draft.create");
    expect(String((created?.body as { charter?: string })?.charter ?? "")).toMatch(/收集本周事实/);
    clickButton(host, "Write Project");
    await flush();
    expect(calls.some((call) => call.pathname === "/management/project/v1/confirm")).toBe(true);
    const confirm = calls.find((call) => call.pathname === "/management/project/v1/confirm");
    expect(confirm?.body).toEqual({ preview_id: "prev-1", preview_digest: "digest-1" });
    unmount(host, root);
  });

  it("keeps the page honest when draft.create is rejected and never posts confirm", async () => {
    const { host, root, calls } = await renderWizard({
      "POST /management/project/v1/draft.create": {
        status: 422,
        body: {
          status: "error",
          error: { code: "PROJECT_INVALID", message: "secret-shaped material is rejected at registration" },
        },
      },
    });
    walkToJoint(host);
    clickButton(host, "Request preview");
    await flush();
    expect(host.querySelector("[data-wizard-error='true']")?.textContent).toMatch(/PROJECT_INVALID/);
    expect(host.querySelector("[data-page='opc-create-wizard']")).not.toBeNull();
    expect(calls.some((call) => call.pathname === "/management/project/v1/confirm")).toBe(false);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("does not leave create-init without a charter", async () => {
    const { host, root } = await renderWizard();
    clickButton(host, "Continue");
    expect(host.querySelector("[data-step='create-init']")).not.toBeNull();
    expect(host.querySelector("[data-wizard-error='true']")?.textContent).toMatch(/Charter/);
    unmount(host, root);
  });
});
