import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { isKnownRoute } from "../../data/normalize";
import { ACTIVITY_COVERAGE, ACTIVITY_ROW_CAP } from "../../data/projections/activity";
import { noteObservedTask } from "../../data/projections/home";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";

const TASK = "task://personal/web-ui/0193a3f9-1111-7000-8000-0000000000a1";

type RouteResponse = { status: number; body: unknown };
type RouteHandler = RouteResponse | ((call: { url: URL; body?: unknown }) => RouteResponse);

interface RecordedCall {
  method: string;
  path: string;
  query: URLSearchParams;
  body?: unknown;
}

function defaultRoute(path: string): RouteResponse {
  if (path === "/personal/health") {
    return { status: 200, body: { status: "ok" } };
  }
  if (path === "/personal/status") {
    return { status: 200, body: { status: "ok", overall: "ready", components: [] } };
  }
  if (path === "/management/alerts") {
    return { status: 200, body: { status: "ok", alerts: [] } };
  }
  if (path === "/management/audit") {
    return { status: 200, body: { status: "ok", events: [] } };
  }
  return { status: 404, body: { status: "error", code: "NOT_FOUND", message: "not found" } };
}

function installFetch(routes: Record<string, RouteHandler>): RecordedCall[] {
  const calls: RecordedCall[] = [];
  vi.stubGlobal(
    "fetch",
    vi.fn(async (input: unknown, init?: RequestInit) => {
      const url = new URL(String(input), "http://localhost");
      const method = (init?.method ?? "GET").toUpperCase();
      let body: unknown;
      if (typeof init?.body === "string") {
        try {
          body = JSON.parse(init.body);
        } catch {
          body = init.body;
        }
      }
      calls.push({ method, path: url.pathname, query: url.searchParams, body });
      const handler = routes[`${method} ${url.pathname}`];
      const resolved =
        typeof handler === "function"
          ? handler({ url, body })
          : (handler ?? defaultRoute(url.pathname));
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

async function flush(ticks = 12) {
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

function activityRoutes(overrides: Record<string, RouteHandler> = {}): Record<string, RouteHandler> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", components: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/audit": { status: 200, body: { status: "ok", events: [] } },
    ...overrides,
  };
}

async function renderActivity(
  overrides: Record<string, RouteHandler> = {},
  options: { task?: boolean } = {},
) {
  rememberBearer("management", "test-management-bearer");
  if (options.task) {
    rememberBearer("task", "test-task-bearer");
  }
  const calls = installFetch(activityRoutes(overrides));
  const view = renderAppAt("#/activity");
  await flush();
  return { ...view, calls };
}

const POPULATED: Record<string, RouteHandler> = {
  "GET /management/alerts": {
    status: 200,
    body: {
      status: "ok",
      alerts: [
        {
          alert_id: "al-live",
          budget_id: "bud-1",
          threshold_kind: "exceeded_80",
          issued_at_ms: Date.now() - 15_000,
          acknowledged_at_ms: null,
        },
      ],
    },
  },
  "GET /management/audit": {
    status: 200,
    body: {
      status: "ok",
      events: [
        { audit_id: "aud-2", action: "key.rotate", outcome: "ok", detail: "acct-1" },
        { audit_id: "aud-1", action: "account.create", outcome: "ok", detail: "acct-1" },
      ],
    },
  },
};

describe("Activity page (W8)", () => {
  afterEach(() => {
    clearSession();
    appProjections.clear();
    window.location.hash = "";
    vi.unstubAllGlobals();
  });

  it("keeps the BD-5 coverage banner on the empty stream and is not a placeholder", async () => {
    const { host, root, calls } = await renderActivity();
    expect(host.querySelector("main h2")?.textContent).toBe("Activity");
    expect(host.querySelector("[data-annotation='activity-coverage']")?.textContent).toContain(
      ACTIVITY_COVERAGE,
    );
    expect(host.textContent).toContain("Nothing recorded in this view yet");
    expect(host.textContent).not.toMatch(/Under reconstruction/);
    expect(host.textContent).not.toMatch(/AI activity/);
    expect(host.querySelector("input[type='search']")).toBeNull();
    expect(host.querySelector(".cp-chat")).toBeNull();
    expect(calls.some((call) => call.path === "/management/resource/v1/list")).toBe(false);
    expect(isKnownRoute("GET", "/management/audit")).toBe(true);
    expect(isKnownRoute("POST", "/management/alerts/acknowledge")).toBe(true);
    unmount(host, root);
  });

  it("renders a time-ordered stream with kinds labeled as text", async () => {
    const { host, root } = await renderActivity(POPULATED);
    const kinds = [...host.querySelectorAll("[data-kind]")].map((row) => row.getAttribute("data-kind"));
    expect(kinds).toContain("error");
    expect(kinds).toContain("change");
    expect(kinds).toContain("event");
    expect(host.textContent).toContain("Error");
    expect(host.textContent).toContain("Change");
    expect(host.textContent).toContain("Event");
    expect(host.textContent).toContain("age unknown (provider audit rows carry no timestamp)");
    const stream = host.querySelector("[aria-label='Activity stream']");
    expect(stream?.querySelector("[data-kind='error']")?.textContent).toContain("al-live");
    unmount(host, root);
  });

  it("acknowledges an alert inline and keeps the receipt after the refresh it triggers", async () => {
    let acknowledged = false;
    const { host, root } = await renderActivity({
      ...POPULATED,
      "GET /management/alerts": () => ({
        status: 200,
        body: {
          status: "ok",
          alerts: acknowledged
            ? []
            : [
                {
                  alert_id: "al-live",
                  budget_id: "bud-1",
                  threshold_kind: "exceeded_80",
                  issued_at_ms: Date.now() - 15_000,
                  acknowledged_at_ms: null,
                },
              ],
        },
      }),
      "POST /management/alerts/acknowledge": (call) => {
        expect(call.body).toEqual({ alert_id: "al-live" });
        acknowledged = true;
        return { status: 200, body: { status: "ok" } };
      },
    });
    const ack = [...host.querySelectorAll("button")].find(
      (button) => (button.textContent ?? "").trim() === "Acknowledge",
    );
    expect(ack).toBeDefined();
    await act(async () => {
      ack?.click();
    });
    await flush();
    expect(host.textContent).toContain("Alert al-live acknowledged.");
    expect(host.textContent).toContain("This session performed alert.acknowledge");
    expect(host.querySelector("[data-kind='intervention']")?.textContent).toContain("Intervention");
    unmount(host, root);
  });

  it("keeps a failed acknowledge named instead of toasting it away", async () => {
    const { host, root } = await renderActivity({
      ...POPULATED,
      "POST /management/alerts/acknowledge": {
        status: 503,
        body: { status: "error", code: "ALERT_ACK_UNAVAILABLE", message: "down" },
      },
    });
    const ack = [...host.querySelectorAll("button")].find(
      (button) => (button.textContent ?? "").trim() === "Acknowledge",
    );
    await act(async () => {
      ack?.click();
    });
    await flush();
    expect(host.textContent).toContain("HTTP 503 ALERT_ACK_UNAVAILABLE");
    expect(host.querySelector("[data-kind='error']")?.textContent).toContain("al-live");
    unmount(host, root);
  });

  it("names a failed audit source and still renders alerts", async () => {
    const { host, root } = await renderActivity({
      ...POPULATED,
      "GET /management/audit": {
        status: 503,
        body: { status: "error", code: "AUDIT_UNAVAILABLE", message: "down" },
      },
    });
    expect(host.textContent).toContain("provider-plane audit unavailable — AUDIT_UNAVAILABLE");
    expect(host.querySelector("[data-kind='error']")?.textContent).toContain("al-live");
    expect(host.textContent).not.toContain("Nothing recorded in this view yet");
    unmount(host, root);
  });

  it("renders the daemon 200-stub as STUB_ROUTE rather than as an empty stream", async () => {
    const { host, root } = await renderActivity({
      "GET /management/audit": {
        status: 200,
        body: {
          status: "ok",
          channel: "management",
          note: "authenticated personal front door; business routes deferred",
        },
      },
    });
    expect(host.textContent).toContain("STUB_ROUTE");
    expect(host.textContent).not.toContain("Nothing recorded in this view yet");
    unmount(host, root);
  });

  it("shows session-observed admission without fabricating evidence when there is no Task session", async () => {
    noteObservedTask(appProjections, {
      taskRef: TASK,
      objective: "search the workspace",
      observedAtMs: Date.now() - 30_000,
      origin: "task/admit",
    });
    const { host, root, calls } = await renderActivity(POPULATED);
    expect(host.textContent).toContain("Admitted this session");
    expect(host.textContent).toContain("Admission is not execution");
    expect(host.textContent).toContain("Task evidence/effects not-run");
    expect(calls.some((call) => call.path === "/task/evidence")).toBe(false);
    expect(calls.some((call) => call.path === "/task/effects")).toBe(false);
    unmount(host, root);
  });

  it("composes verification, acceptance and effect rows from observed task probes", async () => {
    noteObservedTask(appProjections, {
      taskRef: TASK,
      objective: "search the workspace",
      observedAtMs: Date.now() - 30_000,
      origin: "task/admit",
    });
    const { host, root, calls } = await renderActivity(
      {
        ...POPULATED,
        "GET /task/effects": {
          status: 200,
          body: {
            effects: [
              {
                effect_ref: "e-1",
                stage: "EXECUTED",
                outcome_class: "ok",
                reconcile_class: "closed",
              },
            ],
          },
        },
        "GET /task/evidence": {
          status: 200,
          body: {
            task_ref: TASK,
            lifecycle: { current_state: "COMPLETED" },
            latest_verification: {
              report_ref: "report://personal/r-881",
              status: "passed",
              completed_at: new Date(Date.now() - 60_000).toISOString(),
              current: true,
            },
            latest_acceptance: { current: true },
          },
        },
      },
      { task: true },
    );
    expect(host.textContent).toContain("Verification");
    expect(host.textContent).toContain("Acceptance");
    expect(host.textContent).toContain("Effect");
    expect(host.textContent).toContain("Terminal acceptance is recorded");
    expect(calls.some((call) => call.path === "/task/evidence")).toBe(true);
    expect(calls.some((call) => call.path === "/task/effects")).toBe(true);
    expect(calls.some((call) => call.path === "/management/resource/v1/list")).toBe(false);
    unmount(host, root);
  });

  it("names the bounded window when the stream exceeds the cap", async () => {
    const events = Array.from({ length: ACTIVITY_ROW_CAP + 4 }, (_, index) => ({
      audit_id: `aud-${String(index).padStart(3, "0")}`,
      action: "account.create",
      outcome: "ok",
    }));
    const { host, root } = await renderActivity({
      "GET /management/audit": { status: 200, body: { status: "ok", events } },
    });
    expect(host.textContent).toContain(`showing ${ACTIVITY_ROW_CAP} of ${ACTIVITY_ROW_CAP + 4}`);
    expect(host.textContent).toContain(`bounded window of ${ACTIVITY_ROW_CAP}`);
    expect(host.querySelectorAll("[data-kind]")).toHaveLength(ACTIVITY_ROW_CAP);
    unmount(host, root);
  });

  it("filters by kind without dropping the coverage banner", async () => {
    const { host, root } = await renderActivity(POPULATED);
    const kindFilter = host.querySelector("select[aria-label='Filter by kind']") as HTMLSelectElement;
    await act(async () => {
      kindFilter.value = "change";
      kindFilter.dispatchEvent(new Event("change", { bubbles: true }));
    });
    await flush();
    const kinds = [...host.querySelectorAll("[data-kind]")].map((row) => row.getAttribute("data-kind"));
    expect(kinds.every((value) => value === "change")).toBe(true);
    expect(host.querySelector("[data-annotation='activity-coverage']")?.textContent).toContain("BD-5");
    unmount(host, root);
  });

  it("inspects a row without dumping a chat transcript", async () => {
    const { host, root } = await renderActivity(POPULATED);
    const inspect = [...host.querySelectorAll("button")].find(
      (button) => (button.textContent ?? "").trim() === "Inspect",
    );
    await act(async () => {
      inspect?.click();
    });
    await flush();
    expect(host.querySelector(".cp-inspector")?.textContent).toContain("source");
    expect(host.querySelector(".cp-inspector")?.textContent).toContain("GET /management/");
    unmount(host, root);
  });
});
