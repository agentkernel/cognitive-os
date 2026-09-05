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

function baseRoutes(extras: Record<string, RouteResponse> = {}): Record<string, RouteResponse> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", alerts: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/project/v1/list": { status: 200, body: { status: "ok", projects: [] } },
    "GET /management/project/v1/pending-previews": { status: 200, body: { status: "ok", previews: [] } },
    "GET /management/project/v1/standing-policies": { status: 200, body: { status: "ok", policies: [] } },
    "GET /management/providers/accounts": { status: 200, body: { status: "ok", accounts: [] } },
    "GET /management/usage": { status: 200, body: { status: "ok", events: [] } },
    "GET /management/settings/v1/diagnostics": {
      status: 200,
      body: { status: "ok", dsh: { facts: "empty" }, pi: { facts: "empty" } },
    },
    "GET /management/settings/v1/notifications": {
      status: 200,
      body: { status: "ok", missed: [], offline: [], resume: [] },
    },
    ...extras,
  };
}

async function renderSettings(hash: string, extras: Record<string, RouteResponse> = {}) {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch(baseRoutes(extras));
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

describe("P13-T08 Settings Model Connections / diagnostics / state-lab", () => {
  it("whitelists Settings connect and keeps task aliases and /providers writes off the client", () => {
    expect(isKnownRoute("POST", "/management/settings/v1/connection.connect")).toBe(true);
    expect(isKnownRoute("GET", "/management/settings/v1/diagnostics")).toBe(true);
    expect(isKnownRoute("GET", "/management/settings/v1/notifications")).toBe(true);
    expect(isKnownRoute("POST", "/task/settings/v1/connection.connect")).toBe(false);
    expect(isKnownRoute("GET", "/task/settings/v1/diagnostics")).toBe(false);
  });

  it("completes Model Connections inside Settings without a /providers detour or raw secret", async () => {
    const { host, root, calls } = await renderSettings("#/settings", {
      "POST /management/settings/v1/connection.connect": {
        status: 200,
        body: {
          status: "ok",
          connection: {
            id: "acct-1",
            display_name: "openai-work",
            provider_kind: "openai_official",
            connection_status: "connected",
            secret: "present",
            model_id: "gpt-4o",
          },
        },
      },
      "GET /management/providers/accounts": {
        status: 200,
        body: {
          status: "ok",
          accounts: [
            {
              id: "acct-1",
              display_name: "openai-work",
              provider_kind: "openai_official",
              status: "active",
              secret_ref: "ss://provider/acct-1",
            },
          ],
        },
      },
      "GET /management/usage": {
        status: 200,
        body: {
          status: "ok",
          events: [
            {
              event_id: "ev-1",
              account_id: "acct-1",
              cost_micros: 1500,
              cost_status: "priced",
              cost_label: "actual",
              metering_source: "provider_reported",
            },
          ],
        },
      },
    });

    expect(host.querySelector("a[href='#/providers']")).toBeNull();
    expect(host.textContent).not.toMatch(/Open Providers/i);

    const form = host.querySelector("[data-region='opc-model-connections']");
    expect(form).not.toBeNull();
    const template = form?.querySelector("select[name='template']") as HTMLSelectElement;
    const key = form?.querySelector("input[name='api_key']") as HTMLInputElement;
    const model = form?.querySelector("input[name='model']") as HTMLInputElement;
    expect(template).not.toBeUndefined();
    act(() => {
      template.value = "openai";
      template.dispatchEvent(new Event("change", { bubbles: true }));
      key.value = "sk-live-must-never-render";
      key.dispatchEvent(new Event("input", { bubbles: true }));
      model.value = "gpt-4o";
      model.dispatchEvent(new Event("input", { bubbles: true }));
    });
    const submit = [...form!.querySelectorAll("button")].find((button) =>
      (button.textContent ?? "").toLowerCase().includes("secretstore"),
    );
    expect(submit).not.toBeUndefined();
    act(() => {
      submit!.click();
    });
    await flush();

    const posted = calls.find((call) => call.pathname === "/management/settings/v1/connection.connect");
    expect(posted?.method).toBe("POST");
    expect(posted?.body).toMatchObject({
      template: "openai",
      model: "gpt-4o",
    });
    expect(host.querySelector("[data-region='opc-model-connections'] input[name='api_key']")).toHaveProperty(
      "value",
      "",
    );
    expect(host.textContent).not.toContain("sk-live-must-never-render");
    expect(host.querySelector("[data-connection-status='acct-1']")?.textContent).toMatch(/connected/i);
    expect(host.querySelector("[data-usage='acct-1']")?.textContent).toBe("actual $0.001500");
    expect(host.querySelector("[data-usage='acct-1']")?.textContent).not.toBe("0");
    expect(host.querySelector("[data-page='opc-settings'] a[href='#/providers']")).toBeNull();
    expect(host.querySelector("[data-page='opc-settings']")?.textContent).not.toMatch(
      /#\/providers|Open Providers/i,
    );
    unmount(host, root);
  });

  it("does not post a fake Connect when the key is empty and shows failed without leaking secret", async () => {
    const { host, root, calls } = await renderSettings("#/settings", {
      "POST /management/settings/v1/connection.connect": {
        status: 503,
        body: {
          status: "error",
          code: "PROVIDER_SECRET_STORE_UNAVAILABLE",
          message: "approved Secret Store is not available",
          connection_status: "failed",
        },
      },
    });
    const form = host.querySelector("[data-region='opc-model-connections']") as HTMLElement;
    const submit = [...form.querySelectorAll("button")].find((button) =>
      (button.textContent ?? "").toLowerCase().includes("secretstore"),
    ) as HTMLButtonElement;
    expect(submit.disabled).toBe(true);
    expect(calls.some((call) => call.pathname === "/management/settings/v1/connection.connect")).toBe(
      false,
    );

    const key = form.querySelector("input[name='api_key']") as HTMLInputElement;
    act(() => {
      key.value = "sk-live-failed-path";
      key.dispatchEvent(new Event("input", { bubbles: true }));
    });
    act(() => {
      submit.click();
    });
    await flush();
    expect(host.querySelector("[data-region='opc-connection-error']")?.textContent).toMatch(/failed|SecretStore/i);
    expect(host.textContent).not.toContain("sk-live-failed-path");
    expect((form.querySelector("input[name='api_key']") as HTMLInputElement).value).toBe("");
    unmount(host, root);
  });

  it("groups missed / offline / resume facts and keeps diagnostics and state-lab hidden by default", async () => {
    const { host, root, calls } = await renderSettings("#/settings?home=home-1", {
      "GET /management/host/v1/status": {
        status: 200,
        body: {
          status: "ok",
          home_id: "home-1",
          daemon_id: "daemon-1",
          daemon_state: "offline",
          can_honor_background: false,
          close_disposition: "pause",
          tray_proves_work: false,
          missed_segments: 2,
          resume_eligible: true,
        },
      },
      "GET /management/settings/v1/notifications": {
        status: 200,
        body: {
          status: "ok",
          missed: [{ kind: "missed", detail: "2 host segments", source: "host" }],
          offline: [{ kind: "offline", detail: "offline", source: "host" }],
          resume: [{ kind: "resume", detail: "resume-eligible-only", source: "host" }],
        },
      },
      "GET /management/settings/v1/diagnostics": {
        status: 200,
        body: {
          status: "ok",
          dsh: { facts: "empty", expected_revision: null, health: null, update: null, rollback: null },
          pi: { facts: "empty", exact_version: null, health: null },
        },
      },
    });

    expect(host.querySelector("[data-region='opc-settings-notifications']")?.textContent).toMatch(
      /missed|offline|resume/i,
    );
    const diagnostics = host.querySelector("[data-region='opc-settings-diagnostics']") as HTMLDetailsElement;
    const stateLab = host.querySelector("[data-region='opc-settings-state-lab']") as HTMLDetailsElement;
    expect(diagnostics).not.toBeNull();
    expect(stateLab).not.toBeNull();
    expect(diagnostics.open).toBe(false);
    expect(stateLab.open).toBe(false);
    expect(host.querySelectorAll("[data-state-lab-cell]").length).toBe(0);
    expect(host.querySelector("[data-region='opc-state-lab-grid']")).toBeNull();
    expect(host.querySelector("[data-page='opc-state-lab']")).toBeNull();
    expect(calls.some((call) => call.pathname === "/management/settings/v1/diagnostics")).toBe(true);
    expect(host.querySelector("nav a[href='#/state-lab']")).toBeNull();
    unmount(host, root);
  });

  it("renders the nine-state × nine-surface lab with real components when Advanced is opened", async () => {
    const { host, root } = await renderSettings("#/settings");
    const stateLab = host.querySelector("[data-region='opc-settings-state-lab']") as HTMLDetailsElement;
    act(() => {
      stateLab.open = true;
      stateLab.dispatchEvent(new Event("toggle"));
    });
    await flush();
    const grid = host.querySelector("[data-region='opc-state-lab-grid']");
    expect(grid).not.toBeNull();
    expect(grid?.querySelectorAll("[data-state-lab-cell]").length).toBe(81);
    expect(grid?.querySelector(".cp-stateview")).not.toBeNull();
    expect(grid?.querySelector(".cp-receipt")).not.toBeNull();
    expect(host.querySelector("[data-page='opc-state-lab']")).toBeNull();
    unmount(host, root);
  });
});
