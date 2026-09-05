import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { LinuxLegacyApp as App } from "../../App";
import { isKnownRoute } from "../../data/normalize";
import {
  composeAttention,
  evidenceDisposition,
  expandedReadinessComponents,
  mergeCurrentWork,
  noteObservedTask,
  projectHomeReadiness,
  projectTaskEnvelopes,
  projectTaskEvidence,
  readinessComponentReading,
  recentEvidenceRows,
  recordSessionMutation,
  shortTaskRef,
  sortAttention,
  worstReadinessComponent,
  type AttentionItem,
} from "../../data/projections/home";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";

/* ---------- test harness (same shape as views/providers/providers.test.tsx) ---------- */

type RouteResponse = { status: number; body: unknown };
type RouteHandler = RouteResponse | ((call: { body?: any; url: URL }) => RouteResponse);

interface RecordedCall {
  method: string;
  path: string;
  query: URLSearchParams;
  body?: any;
}

function defaultRoute(): RouteResponse {
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
        typeof handler === "function" ? handler({ body, url }) : (handler ?? defaultRoute());
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

function findButton(scope: HTMLElement, text: string): HTMLButtonElement {
  const button = [...scope.querySelectorAll("button")].find(
    (candidate) => (candidate.textContent ?? "").trim() === text,
  );
  if (!button) {
    throw new Error(`button not found: ${text}`);
  }
  return button;
}

function region(host: HTMLElement, id: string): HTMLElement {
  const found = host.querySelector<HTMLElement>(`section[aria-labelledby="${id}"]`);
  if (!found) {
    throw new Error(`region not found: ${id}`);
  }
  return found;
}

/* ---------- fixtures ---------- */

const TASK_A = "task://personal/web-ui/0193a3f9-1111-7000-8000-0000000000a1";
const TASK_B = "task://personal/web-ui/0193b71c-2222-7000-8000-0000000000b2";

const READY_COMPONENTS = [
  { component: "system", status: "ready", required: true, error_class: null },
  { component: "database", status: "ready", required: true, error_class: null },
  { component: "secret", status: "ready", required: true, error_class: null },
  { component: "provider", status: "ready", required: true, error_class: null },
  { component: "daemon", status: "ready", required: true, error_class: null },
  { component: "pi", status: "ready", required: false, error_class: null },
];

function statusBody(overrides: Record<string, unknown> = {}): unknown {
  return {
    status: "ok",
    schema_version: 1,
    surface: "personal-status",
    overall: "ready",
    first_conversation_ready: true,
    evaluated_at_unix_ms: Date.now() - 12_000,
    components: READY_COMPONENTS,
    ...overrides,
  };
}

function homeRoutes(overrides: Record<string, RouteHandler> = {}): Record<string, RouteHandler> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: statusBody() },
    "GET /management/providers/accounts": { status: 200, body: { status: "ok", accounts: [] } },
    "GET /management/agent-bindings": { status: 200, body: { status: "ok", bindings: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/audit": { status: 200, body: { status: "ok", events: [] } },
    "GET /management/resource/v1/list": { status: 200, body: { status: "ok", resources: [] } },
    ...overrides,
  };
}

async function renderHome(overrides: Record<string, RouteHandler> = {}, withTask = false) {
  rememberBearer("management", "test-management-bearer");
  if (withTask) {
    rememberBearer("task", "test-task-bearer");
  }
  const calls = installFetch(homeRoutes(overrides));
  const view = renderAppAt("#/home");
  await flush();
  return { ...view, calls };
}

function taskEnvelope(taskRef: string, epoch: number | null) {
  return {
    id: taskRef,
    family: "task",
    object_version: epoch,
    projection_version: "personal-resource-manager/1",
    health: "contracted",
    owner: "owner://local",
    scope: taskRef,
    revision_digest: "sha256:contract-digest",
    blocked_reason: null,
    allowed_actions: ["inspect"],
    typed_bindings: [],
  };
}

function evidenceBody(taskRef: string, overrides: Record<string, unknown> = {}) {
  return {
    schema_version: 1,
    task_ref: taskRef,
    contract_epoch: 1,
    lifecycle: {
      current_state: "COMPLETED",
      current_version: 4,
      transitions: [],
      transitions_truncated: false,
    },
    intent_refs: [],
    effect_refs: [],
    reconcile_class: "closed",
    latest_verification: {
      report_ref: "report://personal/r-881",
      report_digest: "sha256:0011223344556677",
      status: "passed",
      completed_at: new Date(Date.now() - 26 * 60 * 1000).toISOString(),
      current: true,
      artifact_refs: [],
      artifacts_current: true,
    },
    latest_acceptance: {
      terminal_transition_ref: "transition://personal/t-9",
      terminal_transition_digest: "sha256:aabbccdd",
      current: true,
    },
    durable_cursor: { event_sequence: 9, task_version: 4, terminal_transition_sequence: 9 },
    ...overrides,
  };
}

afterEach(() => {
  clearSession();
  appProjections.clear();
  window.location.hash = "";
  vi.unstubAllGlobals();
});

/* ---------- projection unit tests ---------- */

describe("Home readiness projection", () => {
  it("selects the worst component and never ranks unknown as ready", () => {
    const view = projectHomeReadiness(
      statusBody({
        overall: "blocked",
        first_conversation_ready: false,
        components: [
          { component: "system", status: "ready", required: true },
          { component: "database", status: "degraded", required: true, error_class: "IO_SLOW" },
          { component: "secret", status: "blocked", required: true, error_class: "VAULT_LOCKED" },
          { component: "provider", status: "not_configured", required: true },
          { component: "daemon", status: "ready", required: true },
          { component: "pi", status: "wobbly", required: false },
        ],
      }),
    );
    const worst = worstReadinessComponent(view);
    expect(worst?.name).toBe("secret");
    expect(worst?.state).toBe("blocked");
    expect(worst?.errorClass).toBe("VAULT_LOCKED");

    // A word the client does not know is unknown — never promoted to ready.
    const unmapped = view.components.find((component) => component.name === "pi");
    expect(unmapped?.state).toBe("wobbly");
    const reading = readinessComponentReading(unmapped!);
    expect(reading.category).toBe("unknown");
    expect(reading.unmapped).toBe(true);
    expect(reading.label).toBe("wobbly");
    // …and with the blocked component gone the next-worst reported one wins;
    // a ready component is never selected as the thing that needs the owner.
    expect(
      worstReadinessComponent({
        ...view,
        components: view.components.filter((component) => component.name !== "secret"),
      })?.name,
    ).toBe("database");
  });

  it("returns no worst component when every reported component is ready", () => {
    expect(worstReadinessComponent(projectHomeReadiness(statusBody()))).toBeUndefined();
  });

  it("renders a component the daemon did not report as unreported, never as ready", () => {
    const view = projectHomeReadiness(
      statusBody({
        components: [
          { component: "system", status: "ready", required: true },
          { component: "database", status: "ready", required: true },
        ],
      }),
    );
    const expanded = expandedReadinessComponents(view);
    expect(expanded).toHaveLength(6);
    const pi = expanded.find((component) => component.name === "pi");
    expect(pi?.reported).toBe(false);
    expect(pi?.state).toBe("not reported");
    // An unreported component is the worst thing on the surface, not the best.
    expect(worstReadinessComponent(view)?.reported).toBe(false);
  });

  it("keeps an absent evaluation timestamp undefined instead of zero", () => {
    const view = projectHomeReadiness(statusBody({ evaluated_at_unix_ms: null }));
    expect(view.evaluatedAtMs).toBeUndefined();
  });
});

describe("Home attention ordering", () => {
  function item(id: string, priority: AttentionItem["priority"]): AttentionItem {
    return {
      id,
      priority,
      reading: { category: "unknown", label: id, unmapped: false },
      objectType: "test",
      objectLabel: id,
      reason: "reason",
    };
  }

  it("sorts change, blocked, attention, waiting, stale", () => {
    const sorted = sortAttention([
      item("s", "stale"),
      item("w", "waiting"),
      item("a", "attention"),
      item("b", "blocked"),
      item("c", "change"),
    ]);
    expect(sorted.map((row) => row.id)).toEqual(["c", "b", "a", "w", "s"]);
  });

  it("inserts a new arrival in rank without reshuffling the rows already on screen", () => {
    const before = sortAttention([
      item("b1", "blocked"),
      item("b2", "blocked"),
      item("a1", "attention"),
    ]);
    expect(before.map((row) => row.id)).toEqual(["b1", "b2", "a1"]);
    const after = sortAttention([
      item("b1", "blocked"),
      item("b2", "blocked"),
      item("b3", "blocked"),
      item("a1", "attention"),
    ]);
    expect(after.map((row) => row.id)).toEqual(["b1", "b2", "b3", "a1"]);
  });

  it("puts consequential changes at the top of the queue and leaves routine audit rows out", () => {
    const items = composeAttention({
      readiness: undefined,
      accounts: [
        {
          id: "acct-1",
          name: "deepseek-main",
          kind: "openai_compatible",
          status: "revoked",
          secret: "present",
        },
      ],
      bindings: [],
      alerts: [],
      auditEvents: [
        { id: "aud-1", action: "account.create", outcome: "ok", detail: "acct-1" },
        { id: "aud-2", action: "key.rotate", outcome: "ok", detail: "acct-1" },
      ],
      receipts: [],
      effects: [],
      providersAuthoritativelyEmpty: false,
      workAuthoritativelyEmpty: false,
    });
    expect(items[0].priority).toBe("change");
    expect(items[0].reading.label).toBe("key.rotate");
    // Account creation and catalog probes are ordinary traffic, not changes.
    expect(items.some((row) => row.reading.label === "account.create")).toBe(false);
    // Provider-plane audit rows have no timestamp; the age says so.
    expect(items[0].ageUnknownReason).toContain("no timestamp");
    expect(items[1].priority).toBe("blocked");
  });
});

describe("Home current-work merge", () => {
  it("deduplicates a task ref present in both the envelope list and this session", () => {
    const envelopes = projectTaskEnvelopes({
      resources: [taskEnvelope(TASK_A, 3), taskEnvelope(TASK_B, 1)],
    });
    const rows = mergeCurrentWork(envelopes, [
      { taskRef: TASK_A, objective: "search the workspace", observedAtMs: 1000, origin: "task/admit" },
    ]);
    expect(rows).toHaveLength(2);
    const merged = rows.find((row) => row.taskRef === TASK_A);
    expect(merged?.origin).toBe("envelope+session");
    expect(merged?.contractEpoch).toBe(3);
    expect(merged?.objective).toBe("search the workspace");
    // Session-observed rows lead; envelope-only rows follow by ref.
    expect(rows[0].taskRef).toBe(TASK_A);
    expect(rows[1].origin).toBe("envelope");
  });

  it("never fabricates a contract epoch the envelope did not carry", () => {
    const rows = projectTaskEnvelopes({ resources: [taskEnvelope(TASK_A, null)] });
    expect(rows[0].contractEpoch).toBeUndefined();
    expect(rows[0].health).toBe("contracted");
  });
});

describe("Home evidence disposition", () => {
  it("reports passed with acceptance, failed, and not-current distinctly", () => {
    const passed = projectTaskEvidence(evidenceBody(TASK_A));
    expect(evidenceDisposition(passed).reading.label).toBe("passed · accepted");
    expect(evidenceDisposition(passed).reading.category).toBe("completed");

    const failed = projectTaskEvidence(
      evidenceBody(TASK_A, {
        latest_verification: { ...(evidenceBody(TASK_A) as any).latest_verification, status: "failed" },
        latest_acceptance: null,
      }),
    );
    expect(evidenceDisposition(failed).reading.category).toBe("blocked");

    const notCurrent = projectTaskEvidence(
      evidenceBody(TASK_A, {
        latest_verification: { ...(evidenceBody(TASK_A) as any).latest_verification, current: false },
      }),
    );
    expect(evidenceDisposition(notCurrent).reading.category).toBe("unknown");
    expect(evidenceDisposition(notCurrent).detail).toContain("not current");
  });

  it("keeps a task without a verification report out of the evidence region", () => {
    const withReport = projectTaskEvidence(evidenceBody(TASK_A));
    const withoutReport = projectTaskEvidence(
      evidenceBody(TASK_B, { latest_verification: null, latest_acceptance: null }),
    );
    const rows = recentEvidenceRows([
      { taskRef: TASK_A, shortRef: shortTaskRef(TASK_A), view: withReport },
      { taskRef: TASK_B, shortRef: shortTaskRef(TASK_B), view: withoutReport },
    ]);
    expect(rows.map((row) => row.taskRef)).toEqual([TASK_A]);
    // A completion is never rendered bare: no report, no claim.
    expect(evidenceDisposition(withoutReport).reading.label).toBe("no verification report");
  });
});

/* ---------- rendered surface ---------- */

describe("Home surface structure", () => {
  it("renders the four regions in reading order with headings and no dashboard furniture", async () => {
    const { host, root } = await renderHome();
    expect(host.querySelector("main h2")?.textContent).toBe("Home");
    const titles = [...host.querySelectorAll("main section.cp-region > h3")].map(
      (heading) => heading.textContent,
    );
    expect(titles).toEqual(["Readiness", "Needs attention", "Current work", "Recent evidence"]);
    for (const id of [
      "home-readiness-title",
      "home-attention-title",
      "home-work-title",
      "home-evidence-title",
    ]) {
      expect(region(host, id)).not.toBeNull();
    }
    // No KPI/score/percentage theatre.
    const text = host.querySelector("main")?.textContent ?? "";
    expect(text).not.toMatch(/system score|health score|\d+%/i);
    // Explicit refresh only — the surface never claims to poll.
    expect(text).toContain("Nothing on this page polls the daemon");
    unmount(host, root);
  });

  it("keeps landmarks, labelled controls, and a focusable action on every queue row", async () => {
    const { host, root } = await renderHome({
      "GET /management/providers/accounts": {
        status: 200,
        body: {
          status: "ok",
          accounts: [
            {
              id: "acct-1",
              display_name: "deepseek-main",
              provider_kind: "openai_compatible",
              secret_ref: "ss://provider/acct-1",
              status: "degraded",
              last_discovery_error: "discovery failed",
            },
          ],
        },
      },
    });
    expect(host.querySelector("#main")).not.toBeNull();
    expect(host.querySelectorAll("h1").length).toBe(1);
    const queue = region(host, "home-attention-title");
    const rows = [...queue.querySelectorAll("li.cp-queue-row")];
    expect(rows.length).toBeGreaterThan(0);
    for (const row of rows) {
      // Exactly one next action per row, and it is keyboard reachable.
      const actions = row.querySelectorAll("a[href], button");
      expect(actions.length).toBe(1);
      expect((actions[0].textContent ?? "").trim().length).toBeGreaterThan(0);
    }
    // Every state signal carries its verbatim text label, never colour alone.
    for (const chip of host.querySelectorAll("main .cp-chip")) {
      expect((chip.textContent ?? "").trim().length).toBeGreaterThan(0);
    }
    // Readiness expands to the six components, each linking to System.
    act(() => {
      findButton(region(host, "home-readiness-title"), "Show components").click();
    });
    const chips = region(host, "home-readiness-title").querySelectorAll(
      '.cp-region-chips a[href="#/system"]',
    );
    expect(chips.length).toBe(6);
    unmount(host, root);
  });

  it("issues no request to a route outside the client whitelist", async () => {
    noteObservedTask(appProjections, {
      taskRef: TASK_A,
      observedAtMs: Date.now(),
      origin: "test",
    });
    const { host, root, calls } = await renderHome(
      {
        "GET /management/resource/v1/list": {
          status: 200,
          body: { status: "ok", resources: [taskEnvelope(TASK_B, 1)] },
        },
        "GET /task/effects": { status: 200, body: { status: "ok", effects: [] } },
        "GET /task/evidence": ({ url }) => ({
          status: 200,
          body: evidenceBody(url.searchParams.get("task_ref") ?? TASK_A),
        }),
      },
      true,
    );
    expect(calls.length).toBeGreaterThan(0);
    for (const call of calls) {
      expect(
        isKnownRoute(call.method, call.path),
        `${call.method} ${call.path} is not in KNOWN_ROUTES`,
      ).toBe(true);
    }
    // Specifically: no task list/detail/cancel/pause route was invented.
    for (const forbidden of ["/task/cancel", "/task/pause", "/task/list", "/task/tasks"]) {
      expect(calls.some((call) => call.path === forbidden)).toBe(false);
    }
    unmount(host, root);
  });
});

describe("Home needs-attention queue", () => {
  const CROWDED = {
    "GET /personal/status": {
      status: 200,
      body: statusBody({
        overall: "degraded",
        first_conversation_ready: false,
        components: [
          { component: "system", status: "ready", required: true },
          { component: "database", status: "ready", required: true },
          { component: "secret", status: "blocked", required: true, error_class: "VAULT_LOCKED" },
          { component: "provider", status: "degraded", required: true },
          { component: "daemon", status: "ready", required: true },
          { component: "pi", status: "not_configured", required: false },
        ],
      }),
    },
    "GET /management/providers/accounts": {
      status: 200,
      body: {
        status: "ok",
        accounts: [
          {
            id: "acct-1",
            display_name: "deepseek-main",
            provider_kind: "openai_compatible",
            secret_ref: "ss://provider/acct-1",
            status: "revoked",
          },
          {
            id: "acct-2",
            display_name: "grok-side",
            provider_kind: "openai_compatible",
            secret_ref: "ss://provider/acct-2",
            status: "degraded",
            last_discovery_error: "auth rejected",
          },
        ],
      },
    },
    "GET /management/alerts": {
      status: 200,
      body: {
        status: "ok",
        alerts: [
          {
            alert_id: "al-1",
            budget_id: "b1",
            threshold_kind: "warning_80",
            issued_at_ms: Date.now() - 3 * 60 * 1000,
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
  } satisfies Record<string, RouteHandler>;

  it("orders the queue by priority with critical changes as the top group inside it", async () => {
    const { host, root } = await renderHome(CROWDED);
    const queue = region(host, "home-attention-title");

    // Critical changes are a labelled group inside the queue, not a card.
    expect(queue.querySelector("#home-attention-changes")?.textContent).toBe("Critical changes");
    expect(host.querySelectorAll("main .cp-panel").length).toBe(0);
    expect(queue.textContent).toContain("not");
    expect(queue.textContent).toContain("unified system-wide audit");

    const priorities = [...queue.querySelectorAll("li.cp-queue-row")].map((row) =>
      row.getAttribute("data-priority"),
    );
    expect(priorities[0]).toBe("change");
    const rank = ["change", "blocked", "attention", "waiting", "stale"];
    const ranks = priorities.map((priority) => rank.indexOf(priority ?? ""));
    expect(ranks).toEqual([...ranks].sort((a, b) => a - b));
    unmount(host, root);
  });

  it("caps the queue at five rows and discloses the rest", async () => {
    const { host, root } = await renderHome(CROWDED);
    const queue = region(host, "home-attention-title");
    expect(queue.querySelectorAll("li.cp-queue-row").length).toBe(5);
    const more = [...queue.querySelectorAll("button")].find((button) =>
      /\d+ more/.test(button.textContent ?? ""),
    );
    expect(more).not.toBeUndefined();
    act(() => {
      (more as HTMLButtonElement).click();
    });
    expect(
      region(host, "home-attention-title").querySelectorAll("li.cp-queue-row").length,
    ).toBeGreaterThan(5);
    unmount(host, root);
  });

  it("acknowledges an alert inline and keeps the receipt after the refresh it triggers", async () => {
    let posted: any;
    const { host, root } = await renderHome({
      ...CROWDED,
      "POST /management/alerts/acknowledge": (call) => {
        posted = call.body;
        return { status: 200, body: { status: "ok" } };
      },
    });
    const queue = region(host, "home-attention-title");
    act(() => {
      findButton(queue, "Acknowledge").click();
    });
    await flush();
    expect(posted).toEqual({ alert_id: "al-1" });
    const after = region(host, "home-attention-title");
    expect(after.textContent).toContain("Alert al-1 acknowledged");
    expect(after.querySelector(".cp-receipt")).not.toBeNull();
    unmount(host, root);
  });

  it("shows the error class when acknowledge fails, and never a silent success", async () => {
    const { host, root } = await renderHome({
      ...CROWDED,
      "POST /management/alerts/acknowledge": {
        status: 503,
        body: { status: "error", code: "PROVIDER_STORE_LOCKED", message: "locked" },
      },
    });
    act(() => {
      findButton(region(host, "home-attention-title"), "Acknowledge").click();
    });
    await flush();
    const after = region(host, "home-attention-title");
    expect(after.textContent).toContain("HTTP 503");
    expect(after.textContent).toContain("PROVIDER_STORE_LOCKED");
    expect(after.querySelector('[role="alert"]')).not.toBeNull();
    expect(after.textContent).not.toContain("acknowledged.");
    unmount(host, root);
  });

  it("renders a session mutation receipt as a change row at the top of the queue", async () => {
    recordSessionMutation(appProjections, {
      id: "key-rotate-1",
      action: "key.rotate",
      objectRef: "acct-1",
      atMs: Date.now() - 90 * 1000,
      detail: "provider key rotated",
    });
    const { host, root } = await renderHome();
    const rows = [...region(host, "home-attention-title").querySelectorAll("li.cp-queue-row")];
    expect(rows[0].getAttribute("data-priority")).toBe("change");
    expect(rows[0].textContent).toContain("key.rotate");
    expect(rows[0].textContent).toContain("2m ago");
    unmount(host, root);
  });

  it("uses the queue itself to guide a first run, with no onboarding wizard", async () => {
    const { host, root } = await renderHome();
    const queue = region(host, "home-attention-title");
    expect(queue.textContent).toContain("No provider account exists yet");
    expect(queue.querySelector('a[href="#/providers"]')).not.toBeNull();
    expect(queue.textContent).toContain("The daemon lists no task contracts");
    expect(queue.querySelector('a[href="#/work"]')).not.toBeNull();
    expect(host.textContent).not.toMatch(/get started|welcome|step 1 of/i);
    // The designed empty states are present alongside the guidance.
    expect(region(host, "home-work-title").textContent).toContain("No work observed yet");
    unmount(host, root);
  });
});

describe("Home current work", () => {
  it("merges the envelope list with session-observed refs and caps the region at four rows", async () => {
    const observedRefs = [1, 2, 3].map(
      (index) => `task://personal/web-ui/0193c${index}00-0000-7000-8000-00000000000${index}`,
    );
    observedRefs.forEach((taskRef, index) => {
      noteObservedTask(appProjections, {
        taskRef,
        observedAtMs: Date.now() - index * 60 * 1000,
        origin: "task/admit",
      });
    });
    const { host, root } = await renderHome({
      "GET /management/resource/v1/list": {
        status: 200,
        body: {
          status: "ok",
          resources: [
            taskEnvelope(TASK_A, 3),
            taskEnvelope(TASK_B, 1),
            taskEnvelope(observedRefs[0], 2),
          ],
        },
      },
    });
    const work = region(host, "home-work-title");
    const rows = [...work.querySelectorAll("li.cp-queue-row")];
    expect(rows.length).toBe(4);
    // The ref observed in both places appears once, marked as both.
    const both = rows.filter((row) => row.getAttribute("data-origin") === "envelope+session");
    expect(both).toHaveLength(1);
    expect(work.textContent).toContain("contract epoch 2");
    // The envelope carries no lifecycle state, and the rows say exactly that —
    // none of them invents a lifecycle word the daemon never sent.
    const rowText = rows.map((row) => row.textContent ?? "").join(" ");
    expect(rowText).toContain("contracted");
    expect(rowText).toContain("state not exposed");
    expect(rowText).not.toMatch(/\brunning\b|\bcompleted\b|\bactive\b/i);
    expect(work.textContent).toContain("more in Work");
    unmount(host, root);
  });

  it("carries the envelope-only inventory honesty line permanently", async () => {
    const { host, root } = await renderHome({
      "GET /management/resource/v1/list": {
        status: 200,
        body: { status: "ok", resources: [taskEnvelope(TASK_A, 1)] },
      },
    });
    const work = region(host, "home-work-title");
    expect(work.textContent).toContain(
      "Inventory is partial — daemon task listing is envelope-only.",
    );
    expect(work.textContent).toContain(
      "The task watch snapshot does not represent current task inventory",
    );
    expect(work.querySelector('[role="note"]')).not.toBeNull();
    unmount(host, root);
  });
});

describe("Home recent evidence", () => {
  it("shows only task refs that carry a verification report, with their disposition", async () => {
    const { host, root } = await renderHome(
      {
        "GET /management/resource/v1/list": {
          status: 200,
          body: { status: "ok", resources: [taskEnvelope(TASK_A, 1), taskEnvelope(TASK_B, 1)] },
        },
        "GET /task/effects": { status: 200, body: { status: "ok", effects: [] } },
        "GET /task/evidence": ({ url }) =>
          url.searchParams.get("task_ref") === TASK_A
            ? { status: 200, body: evidenceBody(TASK_A) }
            : {
                status: 404,
                body: {
                  status: "error",
                  code: "TASK_EVIDENCE_NOT_FOUND",
                  message: "no evidence",
                },
              },
      },
      true,
    );
    const evidence = region(host, "home-evidence-title");
    const rows = [...evidence.querySelectorAll("li.cp-queue-row")];
    expect(rows).toHaveLength(1);
    expect(rows[0].textContent).toContain(shortTaskRef(TASK_A));
    expect(rows[0].textContent).toContain("passed · accepted");
    expect(rows[0].textContent).toContain("report digest");
    expect(rows[0].textContent).toContain("26m ago");
    // The task without evidence is named as unread, never as completed.
    expect(evidence.textContent).toContain("TASK_EVIDENCE_NOT_FOUND");
    expect(evidence.textContent).toContain("not shown as verified");
    unmount(host, root);
  });

  it("states the task-channel dependency instead of inferring evidence", async () => {
    const { host, root } = await renderHome({
      "GET /management/resource/v1/list": {
        status: 200,
        body: { status: "ok", resources: [taskEnvelope(TASK_A, 1)] },
      },
    });
    const evidence = region(host, "home-evidence-title");
    expect(evidence.textContent).toContain("Not run");
    expect(evidence.textContent).toContain("management session only");
    expect(evidence.querySelectorAll("li.cp-queue-row")).toHaveLength(0);
    unmount(host, root);
  });
});

describe("Home failure honesty", () => {
  it("keeps the other regions rendered when one projection fails", async () => {
    const { host, root } = await renderHome({
      "GET /management/providers/accounts": {
        status: 403,
        body: {
          status: "error",
          error: { code: "LOCAL_SESSION_UNAUTHORIZED", message: "denied" },
        },
      },
      "GET /management/resource/v1/list": {
        status: 200,
        body: { status: "ok", resources: [taskEnvelope(TASK_A, 1)] },
      },
    });
    const queue = region(host, "home-attention-title");
    expect(queue.textContent).toContain("provider accounts could not be read");
    expect(queue.textContent).toContain("LOCAL_SESSION_UNAUTHORIZED");
    expect(queue.querySelector('a[href="#/session"]')).not.toBeNull();
    expect(queue.textContent).toContain("the other regions are unaffected");
    // Readiness and current work still render their real content.
    expect(region(host, "home-readiness-title").textContent).toContain("ready");
    expect(region(host, "home-work-title").querySelectorAll("li.cp-queue-row")).toHaveLength(1);
    // A denied read is never converted into an authoritative empty.
    expect(queue.textContent).not.toContain("No provider account exists yet");
    unmount(host, root);
  });

  it("renders the daemon 200-stub as not-run rather than as success", async () => {
    const { host, root } = await renderHome({
      "GET /management/audit": {
        status: 200,
        body: {
          status: "ok",
          channel: "management",
          note: "authenticated personal front door; business routes deferred",
        },
      },
    });
    const queue = region(host, "home-attention-title");
    expect(queue.textContent).toContain("the provider-plane audit could not be read");
    expect(queue.textContent).toContain("STUB_ROUTE");
    unmount(host, root);
  });

  it("keeps the last good read on screen with its age and source when the daemon goes away", async () => {
    rememberBearer("management", "test-management-bearer");
    let offline = false;
    const routes = homeRoutes({
      "GET /management/resource/v1/list": {
        status: 200,
        body: { status: "ok", resources: [taskEnvelope(TASK_A, 7)] },
      },
    });
    vi.stubGlobal(
      "fetch",
      vi.fn(async (input: unknown, init?: RequestInit) => {
        if (offline) {
          throw new Error("connection refused");
        }
        const url = new URL(String(input), "http://localhost");
        const method = (init?.method ?? "GET").toUpperCase();
        const handler = routes[`${method} ${url.pathname}`];
        const resolved =
          typeof handler === "function" ? handler({ url }) : (handler ?? defaultRoute());
        return new Response(JSON.stringify(resolved.body), {
          status: resolved.status,
          headers: { "content-type": "application/json" },
        });
      }),
    );
    const { host, root } = renderAppAt("#/home");
    await flush();
    expect(region(host, "home-work-title").textContent).toContain("contract epoch 7");

    offline = true;
    act(() => {
      findButton(host, "Refresh").click();
    });
    await flush();

    const work = region(host, "home-work-title");
    expect(work.textContent).toContain("DISCONNECTED");
    expect(work.textContent).toContain("Showing the last known the task list, as of");
    expect(work.textContent).toContain("/management/resource/v1/list?family=task");
    expect(work.textContent).toContain("not claimed as current");
    // The last-good row is still there, and still not claimed as fresh.
    expect(work.querySelectorAll("li.cp-queue-row")).toHaveLength(1);
    expect(work.textContent).toContain("contract epoch 7");
    unmount(host, root);
  });

  it("never converts an unknown or unmapped state into ready or zero", async () => {
    const { host, root } = await renderHome({
      "GET /personal/status": {
        status: 200,
        body: statusBody({
          overall: "degraded",
          first_conversation_ready: false,
          components: [
            { component: "system", status: "ready", required: true },
            { component: "database", status: "wobbly", required: true },
          ],
        }),
      },
      "GET /management/resource/v1/list": {
        status: 200,
        body: { status: "ok", resources: [taskEnvelope(TASK_A, null)] },
      },
    });
    const readiness = region(host, "home-readiness-title");
    // First run opens the component row; unknown words stay verbatim + unmapped.
    expect(readiness.textContent).toContain("wobbly");
    expect(readiness.textContent).toContain("(unmapped state)");
    expect(readiness.textContent).toContain("not reported");
    // Missing components are unknown, never counted as ready and never zeroed.
    const work = region(host, "home-work-title");
    expect(work.textContent).not.toContain("contract epoch 0");
    expect(work.textContent).toContain("contracted");
    unmount(host, root);
  });
});
