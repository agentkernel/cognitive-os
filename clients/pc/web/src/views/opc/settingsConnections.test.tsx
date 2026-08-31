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

const FAKE_ACTION = /approve|create project|activate|new project|team|inbox|confirm|ingest|apply authority/i;

function fakeActionLabels(host: HTMLElement): string[] {
  const scopes = [
    host.querySelector("#main"),
    host.querySelector("[data-rail='assistant']"),
    host.querySelector("nav[aria-label='Primary']"),
  ].filter((node): node is Element => node !== null);
  const labels: string[] = [];
  for (const scope of scopes) {
    for (const node of scope.querySelectorAll("button, a.cp-button")) {
      if (
        node.closest("[data-region='opc-hitl-actions']") ||
        node.closest("[data-region='opc-vault-ingest']") ||
        node.closest("[data-region='opc-standing-policies']") ||
        node.closest("[data-region='opc-close-background']")
      ) {
        continue;
      }
      const label = (node.textContent ?? "").trim();
      if (FAKE_ACTION.test(label)) {
        labels.push(label);
      }
    }
  }
  return labels;
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

describe("P12-T08 Settings connections / retract / CloseBackground", () => {
  it("whitelists Settings write routes and keeps mint/task aliases off the client", () => {
    expect(isKnownRoute("POST", "/management/project/v1/standing-policy.revoke")).toBe(true);
    expect(isKnownRoute("POST", "/management/host/v1/close.request")).toBe(true);
    expect(isKnownRoute("GET", "/management/host/v1/status")).toBe(true);
    expect(isKnownRoute("POST", "/management/project/v1/standing-policy.create")).toBe(false);
    expect(isKnownRoute("POST", "/task/project/v1/standing-policy.revoke")).toBe(false);
    expect(isKnownRoute("POST", "/task/host/v1/close.request")).toBe(false);
  });

  it("shows an honest empty connection table without inventing accounts or posting create", async () => {
    const { host, root, calls } = await renderSettings("#/settings");
    expect(host.querySelector("[data-region='opc-connections']")?.textContent).toMatch(
      /no model connection/i,
    );
    expect(host.textContent).toMatch(/never 0/);
    expect(host.querySelector("[data-row-key]")).toBeNull();
    expect(calls.some((call) => call.pathname === "/management/providers/accounts")).toBe(true);
    expect(calls.some((call) => call.method === "POST")).toBe(false);
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("renders daemon accounts with unknown usage not zeroed and secret presence only", async () => {
    const { host, root } = await renderSettings("#/settings", {
      "GET /management/providers/accounts": {
        status: 200,
        body: {
          status: "ok",
          accounts: [
            {
              id: "acct-1",
              display_name: "flash",
              provider_kind: "openai_compatible",
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
              cost_micros: null,
              cost_status: "cost_unavailable",
            },
          ],
        },
      },
    });
    expect(host.querySelector("[data-row-key='acct-1']")).not.toBeNull();
    expect(host.querySelector("[data-usage='acct-1']")?.textContent).toBe("cost_unavailable");
    expect(host.querySelector("[data-usage='acct-1']")?.textContent).not.toBe("0");
    expect(host.textContent).toContain("present");
    expect(host.textContent).not.toContain("ss://provider/acct-1");
    expect(fakeActionLabels(host)).toEqual([]);
    unmount(host, root);
  });

  it("retracts a time-box via standing-policy.revoke and does not mint", async () => {
    const { host, root, calls } = await renderSettings("#/settings", {
      "GET /management/project/v1/standing-policies": {
        status: 200,
        body: {
          status: "ok",
          policies: [
            {
              policy_id: "pol-1",
              subject_class: "grant-expansion",
              subject_ref: "proj-1",
              expires_at: 1,
              active: true,
            },
          ],
        },
      },
      "POST /management/project/v1/standing-policy.revoke": {
        status: 200,
        body: { status: "ok", result: "revoked", policy_id: "pol-1" },
      },
    });
    expect(host.querySelector("[data-row-key='pol-1']")).not.toBeNull();
    clickButton(host, "Retract this week");
    await flush();
    const revoked = calls.find((call) => call.pathname === "/management/project/v1/standing-policy.revoke");
    expect(revoked?.method).toBe("POST");
    expect(revoked?.body).toEqual({ policy_id: "pol-1" });
    expect(calls.some((call) => call.pathname === "/management/project/v1/standing-policy.create")).toBe(
      false,
    );
    unmount(host, root);
  });

  it("keeps the original policy row when revoke is rejected", async () => {
    const { host, root, calls } = await renderSettings("#/settings", {
      "GET /management/project/v1/standing-policies": {
        status: 200,
        body: {
          status: "ok",
          policies: [
            {
              policy_id: "pol-1",
              subject_class: "grant-expansion",
              subject_ref: "proj-1",
              expires_at: 1,
              active: true,
            },
          ],
        },
      },
      "POST /management/project/v1/standing-policy.revoke": {
        status: 409,
        body: { status: "error", code: "POLICY_NOT_FOUND", message: "gone" },
      },
    });
    clickButton(host, "Retract this week");
    await flush();
    expect(host.querySelector("[data-row-key='pol-1']")).not.toBeNull();
    expect(host.querySelector("[data-region='opc-standing-revoke-error']")?.textContent).toMatch(
      /original policy list stays/i,
    );
    expect(calls.some((call) => call.pathname === "/management/project/v1/standing-policy.revoke")).toBe(
      true,
    );
    unmount(host, root);
  });

  it("does not post close.request when no home_id is on the Settings session", async () => {
    const { host, root, calls } = await renderSettings("#/settings");
    expect(host.querySelector("[data-region='opc-close-background']")?.textContent).toMatch(
      /Requires-environment|not-run/i,
    );
    expect(
      [...host.querySelectorAll("[data-region='opc-close-background'] button")].some((node) =>
        (node.textContent ?? "").includes("Continue in background"),
      ),
    ).toBe(false);
    expect(calls.some((call) => call.pathname === "/management/host/v1/close.request")).toBe(false);
    expect(calls.some((call) => call.pathname === "/management/host/v1/status")).toBe(false);
    unmount(host, root);
  });

  it("posts background close only when the daemon can honor background", async () => {
    const { host, root, calls } = await renderSettings("#/settings?home=home-1", {
      "GET /management/host/v1/status": {
        status: 200,
        body: {
          status: "ok",
          home_id: "home-1",
          daemon_id: "daemon-1",
          daemon_state: "bound",
          can_honor_background: true,
          close_disposition: "unknown",
          tray_proves_work: false,
        },
      },
      "POST /management/host/v1/close.request": {
        status: 200,
        body: {
          status: "ok",
          home_id: "home-1",
          daemon_state: "bound",
          close_disposition: "background-honored",
          can_honor_background: true,
          tray_proves_work: false,
        },
      },
    });
    expect(host.querySelector("[data-row-key='home-1']")?.textContent).toMatch(/false/);
    clickButton(host, "Continue in background");
    await flush();
    const closed = calls.find((call) => call.pathname === "/management/host/v1/close.request");
    expect(closed?.body).toEqual({ home_id: "home-1", choice: "background" });
    unmount(host, root);
  });

  it("does not post a fake background when the daemon cannot honor it", async () => {
    const { host, root, calls } = await renderSettings("#/settings?home=home-1", {
      "GET /management/host/v1/status": {
        status: 200,
        body: {
          status: "ok",
          home_id: "home-1",
          daemon_id: "daemon-1",
          daemon_state: "bound",
          can_honor_background: false,
          close_disposition: "unknown",
          tray_proves_work: false,
        },
      },
    });
    const background = [...host.querySelectorAll("button")].find(
      (node) => (node.textContent ?? "").trim() === "Continue in background",
    ) as HTMLButtonElement | undefined;
    expect(background?.disabled).toBe(true);
    expect(calls.some((call) => call.pathname === "/management/host/v1/close.request")).toBe(false);
    unmount(host, root);
  });

  it("posts pause and keeps the original status when close.request is rejected", async () => {
    const { host, root, calls } = await renderSettings("#/settings?home=home-1", {
      "GET /management/host/v1/status": {
        status: 200,
        body: {
          status: "ok",
          home_id: "home-1",
          daemon_id: "daemon-1",
          daemon_state: "bound",
          can_honor_background: true,
          close_disposition: "unknown",
          tray_proves_work: false,
        },
      },
      "POST /management/host/v1/close.request": {
        status: 409,
        body: { status: "error", code: "HOST_REJECTED", message: "fake background rejected" },
      },
    });
    clickButton(host, "Pause");
    await flush();
    expect(calls.find((call) => call.pathname === "/management/host/v1/close.request")?.body).toEqual({
      home_id: "home-1",
      choice: "pause",
    });
    expect(host.querySelector("[data-row-key='home-1']")).not.toBeNull();
    expect(host.querySelector("[data-region='opc-close-error']")?.textContent).toMatch(
      /original close status stays/i,
    );
    unmount(host, root);
  });
});
