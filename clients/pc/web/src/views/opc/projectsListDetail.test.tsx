import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { isKnownRoute } from "../../data/normalize";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";

type RouteResponse = { status: number; body: unknown };

function installFetch(routes: Record<string, RouteResponse> = {}) {
  vi.stubGlobal(
    "fetch",
    vi.fn(async (input: unknown, init?: RequestInit) => {
      const url = new URL(String(input), "http://localhost");
      const method = (init?.method ?? "GET").toUpperCase();
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

const FAKE_ACTION = /approve|create project|activate|new project|team|inbox|confirm|ingest|apply authority/i;
const TWITTER_HERO = /twitter|\bx\b hero|trending on x|tweet composer/i;
const JARGON_WALL = /Daemon Project aggregate|Not a renamed Task|Vite is not the product origin/i;

function primaryHonesty(host: HTMLElement): Element[] {
  return [...host.querySelectorAll("[data-page] .cp-honesty")].filter(
    (node) =>
      !node.closest("details[data-honesty='secondary']") &&
      !node.closest("[data-region='opc-project-lifecycle']") &&
      !node.closest("[data-region='opc-hitl']"),
  );
}

function fakeActionLabels(host: HTMLElement): string[] {
  return [...host.querySelectorAll("#main button, #main a.cp-button")]
    .map((node) => (node.textContent ?? "").trim())
    .filter((label) => FAKE_ACTION.test(label));
}

function enabledRunControls(host: HTMLElement): string[] {
  return [...host.querySelectorAll("#main button, #main a.cp-button")]
    .filter((node) => {
      const label = (node.textContent ?? "").trim();
      if (!/^(Run|Run now|Start|Trigger)$/i.test(label)) {
        return false;
      }
      return !(node instanceof HTMLButtonElement && node.disabled);
    })
    .map((node) => (node.textContent ?? "").trim());
}

const LIVE_LIST: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [
      { project_id: "proj-1", state: "active", title_summary: "周报与客户跟进", cost: "unknown" },
    ],
  },
};

const CREATING_LIST: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projects: [
      { project_id: "proj-draft", state: "creating", title_summary: "周报与客户跟进", cost: "unknown" },
    ],
  },
};

const EMPTY_LIST: RouteResponse = {
  status: 200,
  body: { status: "ok", projects: [] },
};

const LIVE_DETAIL: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projection: "personal-private",
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

const LIVE_AXIS: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projection: "personal-private",
    plan_revision_id: "plan-1",
    stages: [
      {
        stage_id: "st-1",
        position: 0,
        title: "收集",
        confirm_status: "confirmed",
        ready: true,
        seated: true,
        output_contract: {
          digest: "out-1",
          deliverable_type: "unknown",
          save_format: "unknown",
          open_with: "unknown",
        },
        gaps: [],
      },
    ],
  },
};

const EMPTY_AXIS: RouteResponse = {
  status: 200,
  body: {
    status: "ok",
    projection: "personal-private",
    plan_revision_id: "unknown",
    stages: [],
  },
};

const EMPTY_ROSTER: RouteResponse = {
  status: 200,
  body: { status: "ok", authority_note: "empty-roster", roster: [] },
};

function authedRoutes(extras: Record<string, RouteResponse> = {}): Record<string, RouteResponse> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/project/v1/list": LIVE_LIST,
    "GET /management/project/v1/detail": LIVE_DETAIL,
    "GET /management/project/v1/axis": LIVE_AXIS,
    "GET /management/project/v1/roster": EMPTY_ROSTER,
    "GET /management/project/v1/pending-previews": {
      status: 200,
      body: { status: "ok", previews: [] },
    },
    "GET /management/project/v1/routine.runs": {
      status: 200,
      body: {
        status: "ok",
        projection_id: "personal-private.routine-arming/0.1",
        project_id: "proj-1",
        host: { available: true, reason: "ok" },
        scheduler: "daemon-tick-only",
        armings: [],
        occurrences: [],
      },
    },
    "GET /management/project/v1/dsh.hosted.attempt.list": {
      status: 200,
      body: { status: "ok", attempts: [] },
    },
    ...extras,
  };
}

async function renderAuthed(hash: string, extras: Record<string, RouteResponse> = {}) {
  rememberBearer("management", "test-management-bearer");
  installFetch(authedRoutes(extras));
  const view = renderAppAt(hash);
  await flush();
  return view;
}

afterEach(() => {
  clearSession();
  appProjections.clear();
  window.location.hash = "";
  vi.unstubAllGlobals();
});

describe("P15-T03 Projects list/detail vs v9", () => {
  it("fail-closes Projects without a session: no list, no fake Activate, no Twitter hero", async () => {
    installFetch({});
    const { host, root } = renderAppAt("#/projects");
    await flush();
    expect(host.textContent).toMatch(/session denied|Open Session/i);
    expect(host.querySelector("[data-page='opc-projects']")).toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    expect(host.textContent).not.toMatch(TWITTER_HERO);
    unmount(host, root);
  });

  it("does not treat Linux 1.0 #/work as 2.0 Projects chrome", async () => {
    expect(isKnownRoute("GET", "/management/project/v1/list")).toBe(true);
    const { host, root } = await renderAuthed("#/work");
    expect(host.querySelector("[data-page='opc-projects']")).toBeNull();
    expect(host.querySelector("[data-page='opc-project-detail']")).toBeNull();
    expect(host.querySelector("nav[aria-label='Project sections']")).toBeNull();
    expect(host.querySelector(".cp-page-head h2")?.textContent).not.toBe("项目列表");
    expect(host.querySelector(".cp-page-head h2")?.textContent).not.toBe("项目详情");
    expect(window.location.hash).toBe("#/work");
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("does not offer a clickable Run without live Project authority", async () => {
    const { host, root } = await renderAuthed("#/projects/proj-1/runs");
    const write = host.querySelector("[data-write-attempt]");
    expect(write).not.toBeNull();
    expect(write?.getAttribute("data-write-attempt")).toBe("blocked");
    expect((write as HTMLButtonElement).disabled).toBe(true);
    expect(enabledRunControls(host)).toEqual([]);
    expect(host.querySelector("a[href='#/work']")).toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("leads the live list with v9 Owner copy and four-submenu density, not a jargon wall", async () => {
    const { host, root } = await renderAuthed("#/projects");
    expect(host.querySelector(".cp-shell")?.getAttribute("data-visual")).toBe("v9-target");
    expect(host.querySelector("[data-page='opc-projects']")).not.toBeNull();
    expect(host.querySelector(".cp-page-head h2")?.textContent).toBe("项目列表");
    expect(host.querySelector(".cp-lede")?.textContent).toMatch(/详情 \/ 成员 \/ 运行 \/ 产出/);
    const row = host.querySelector("[data-row-key='proj-1']");
    expect(row).not.toBeNull();
    expect(row?.textContent).toContain("周报与客户跟进");
    expect(row?.textContent).toMatch(/已上线/);
    expect(host.querySelector("a[href='#/projects/proj-1']")?.textContent).toBe("打开");
    expect(host.querySelector("a[href='#/projects/proj-1/members']")?.textContent).toBe("成员");
    expect(host.querySelector("a[href='#/projects/proj-1/runs']")?.textContent).toBe("运行");
    expect(host.querySelector("a[href='#/projects/proj-1/outputs']")?.textContent).toBe("产出");
    expect(primaryHonesty(host)).toEqual([]);
    expect(host.querySelector("#main")?.textContent).not.toMatch(JARGON_WALL);
    expect(fakeActionLabels(host)).toEqual([]);
    expect(host.textContent).not.toMatch(TWITTER_HERO);
    expect(host.textContent).not.toMatch(/\bActivate\b/);
    unmount(host, root);
  });

  it("keeps creating drafts off detail/members/runs/outputs and continues create instead", async () => {
    const { host, root } = await renderAuthed("#/projects", {
      "GET /management/project/v1/list": CREATING_LIST,
    });
    expect(host.querySelector(".cp-page-head h2")?.textContent).toBe("未完成的创建");
    expect(host.querySelector("a[href='#/projects/new']")?.textContent).toMatch(/继续/);
    expect(host.querySelector("a[href='#/projects/proj-draft']")).toBeNull();
    expect(host.querySelector("a[href='#/projects/proj-draft/runs']")).toBeNull();
    expect(enabledRunControls(host)).toEqual([]);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("keeps an empty list honest, with 回今日 and no demo project or Activate", async () => {
    const { host, root } = await renderAuthed("#/projects", {
      "GET /management/project/v1/list": EMPTY_LIST,
    });
    expect(host.querySelector(".cp-page-head h2")?.textContent).toBe("还没有项目");
    expect(host.textContent).toMatch(/no Project/);
    expect(
      [...host.querySelectorAll("[data-page='opc-projects'] a[href='#/']")].some((node) =>
        (node.textContent ?? "").includes("回今日"),
      ),
    ).toBe(true);
    expect(host.querySelector("[data-row-key]")).toBeNull();
    expect(fakeActionLabels(host)).toEqual([]);
    expect(host.textContent).not.toMatch(/\bActivate\b/);
    unmount(host, root);
  });

  it("leads Project detail with v9 Owner copy, four submenus, and honest empty axis", async () => {
    const live = await renderAuthed("#/projects/proj-1");
    expect(live.host.querySelector(".cp-page-head h2")?.textContent).toBe("项目详情");
    expect(live.host.querySelector(".cp-lede")?.textContent).toMatch(/只读章程与流程轴/);
    const nav = live.host.querySelector('nav[aria-label="Project sections"]');
    expect(nav?.textContent).toContain("详情");
    expect(nav?.textContent).toContain("成员");
    expect(nav?.textContent).toContain("运行");
    expect(nav?.textContent).toContain("产出");
    expect(nav?.textContent).not.toMatch(/Team|Inbox|Work/);
    expect(live.host.textContent).toMatch(/已上线/);
    expect(live.host.querySelector("[data-region='opc-project-axis']")?.textContent).toMatch(/收集/);
    expect(primaryHonesty(live.host)).toEqual([]);
    expect(live.host.querySelector("#main")?.textContent).not.toMatch(JARGON_WALL);
    expect(fakeActionLabels(live.host)).toEqual([]);
    unmount(live.host, live.root);
    clearSession();
    appProjections.clear();

    const emptyAxis = await renderAuthed("#/projects/proj-1", {
      "GET /management/project/v1/axis": EMPTY_AXIS,
    });
    expect(emptyAxis.host.textContent).toMatch(/无轴|没有流程轴|no PlanRevision axis/i);
    expect(emptyAxis.host.querySelector("[data-row-key='st-1']")).toBeNull();
    expect(enabledRunControls(emptyAxis.host)).toEqual([]);
    expect(fakeActionLabels(emptyAxis.host)).toEqual([]);
    unmount(emptyAxis.host, emptyAxis.root);
  });
});
