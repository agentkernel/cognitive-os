import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../App";
import { COMMAND_INDEX_HONESTY, COMMAND_NO_RESULTS, resetCommandRecents } from "../data/commands";
import { WORK_TASKS_KEY } from "../data/projections/work";
import { appProjections } from "../data/store";
import { clearSession, rememberBearer } from "../session";

type RouteResponse = { status: number; body: unknown };

const TASK_REF = "task://personal/web-ui/abc";

function installFetch(routes: Record<string, RouteResponse> = {}) {
  vi.stubGlobal(
    "fetch",
    vi.fn(async (input: unknown) => {
      const url = new URL(String(input), "http://localhost");
      const handler = routes[`${(url as URL).pathname}`];
      const resolved = handler ?? {
        status:
          url.pathname === "/personal/health" ||
          url.pathname === "/personal/status" ||
          url.pathname === "/management/alerts"
            ? 200
            : 404,
        body: { status: "ok", alerts: [] },
      };
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

async function flush(ticks = 8) {
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

async function openPalette(host: HTMLDivElement) {
  const button = host.querySelector('button[aria-label="Open command palette"]');
  await act(async () => {
    (button as HTMLButtonElement | null)?.click();
  });
  await flush(4);
}

function typeQuery(input: HTMLInputElement, value: string) {
  const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value")?.set;
  setter?.call(input, value);
  input.dispatchEvent(new Event("input", { bubbles: true }));
}

describe("command palette (W10)", () => {
  afterEach(() => {
    clearSession();
    appProjections.clear();
    resetCommandRecents();
    window.location.hash = "";
    vi.unstubAllGlobals();
  });

  it("opens from the footer, names BD-6, and has no class-C verbs", async () => {
    rememberBearer("management", "test-management-bearer");
    installFetch();
    const { host, root } = renderAppAt("#/");
    await flush();
    await openPalette(host);
    const dialog = host.querySelector('[role="dialog"][aria-label="Command palette"]');
    expect(dialog).not.toBeNull();
    expect(dialog?.textContent).toContain(COMMAND_INDEX_HONESTY);
    expect(dialog?.textContent).toContain("Cancel, pause, retry");
    expect(dialog?.textContent).not.toMatch(/\bCancel task\b/);
    expect(dialog?.textContent).toContain("ACTIONS");
    expect(dialog?.textContent).toContain("DESTINATIONS");
    const input = host.querySelector(".cp-palette-input") as HTMLInputElement;
    expect(input.type).toBe("text");
    expect(input.name).toBe("command-palette-query");
    expect(host.querySelector('input[type="password"]')).toBeNull();
    unmount(host, root);
  });

  it("opens from Control+K", async () => {
    rememberBearer("management", "test-management-bearer");
    installFetch();
    const { host, root } = renderAppAt("#/");
    await flush();
    await act(async () => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "k", ctrlKey: true, bubbles: true }));
    });
    await flush(2);
    expect(host.querySelector('[role="dialog"][aria-label="Command palette"]')).not.toBeNull();
    unmount(host, root);
  });

  it("navigates a destination and closes", async () => {
    rememberBearer("management", "test-management-bearer");
    installFetch();
    const { host, root } = renderAppAt("#/");
    await flush();
    await openPalette(host);
    const work = [...host.querySelectorAll('[role="option"]')].find(
      (node) => (node.textContent ?? "").includes("Work") && (node.textContent ?? "").includes("destination"),
    );
    await act(async () => {
      (work as HTMLElement | undefined)?.click();
    });
    await flush();
    expect(window.location.hash).toMatch(/#\/work/);
    expect(host.querySelector('[role="dialog"][aria-label="Command palette"]')).toBeNull();
    unmount(host, root);
  });

  it("searches loaded objects and names the empty result as partial", async () => {
    rememberBearer("management", "test-management-bearer");
    rememberBearer("task", "test-task-bearer");
    installFetch();
    appProjections.set(WORK_TASKS_KEY, {
      status: "ready",
      data: [{ taskRef: TASK_REF }],
    });
    const { host, root } = renderAppAt("#/");
    await flush();
    await openPalette(host);
    const input = host.querySelector(".cp-palette-input") as HTMLInputElement;
    await act(async () => {
      typeQuery(input, TASK_REF);
    });
    await flush();
    expect(host.textContent).toContain(TASK_REF);
    await act(async () => {
      typeQuery(input, "zzzz-no-such-object");
    });
    await flush();
    expect(host.textContent).toContain(COMMAND_NO_RESULTS);
    unmount(host, root);
  });

  it("copies the current task ref as class-B without leaving the palette", async () => {
    rememberBearer("management", "test-management-bearer");
    rememberBearer("task", "test-task-bearer");
    installFetch();
    const writeText = vi.fn(async () => undefined);
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: { writeText },
    });
    const { host, root } = renderAppAt(`#/work/${encodeURIComponent(TASK_REF)}`);
    await flush();
    await openPalette(host);
    const copy = [...host.querySelectorAll('[role="option"]')].find((node) =>
      (node.textContent ?? "").includes("Copy task ref"),
    );
    await act(async () => {
      (copy as HTMLElement | undefined)?.click();
    });
    await flush();
    expect(writeText).toHaveBeenCalledWith(TASK_REF);
    expect(host.textContent).toContain(`Copied ${TASK_REF}`);
    expect(host.querySelector('[role="dialog"]')).not.toBeNull();
    unmount(host, root);
  });

  it("acknowledges an unacked alert inline as class-B", async () => {
    rememberBearer("management", "test-management-bearer");
    const calls: string[] = [];
    vi.stubGlobal(
      "fetch",
      vi.fn(async (input: unknown, init?: RequestInit) => {
        const url = new URL(String(input), "http://localhost");
        const method = (init?.method ?? "GET").toUpperCase();
        calls.push(`${method} ${url.pathname}`);
        if (method === "POST" && url.pathname === "/management/alerts/acknowledge") {
          return new Response(JSON.stringify({ status: "ok" }), {
            status: 200,
            headers: { "content-type": "application/json" },
          });
        }
        if (method === "GET" && url.pathname === "/management/alerts") {
          return new Response(
            JSON.stringify({
              alerts: [
                {
                  alert_id: "al-live",
                  threshold_kind: "exceeded_80",
                  acknowledged_at_ms: null,
                },
              ],
            }),
            { status: 200, headers: { "content-type": "application/json" } },
          );
        }
        return new Response(JSON.stringify({ status: "ok", alerts: [] }), {
          status: 200,
          headers: { "content-type": "application/json" },
        });
      }),
    );
    const { host, root } = renderAppAt("#/");
    await flush();
    await openPalette(host);
    const input = host.querySelector(".cp-palette-input") as HTMLInputElement;
    await act(async () => {
      typeQuery(input, "al-live");
    });
    await flush();
    const ack = [...host.querySelectorAll('[role="option"]')].find((node) =>
      (node.textContent ?? "").includes("Acknowledge al-live"),
    );
    await act(async () => {
      (ack as HTMLElement | undefined)?.click();
    });
    await flush();
    expect(calls).toContain("POST /management/alerts/acknowledge");
    expect(host.textContent).toContain("Alert al-live acknowledged.");
    expect(host.querySelector('[role="dialog"]')).not.toBeNull();
    unmount(host, root);
  });
});
