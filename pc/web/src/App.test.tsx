import { createRoot } from "react-dom/client";
import { act } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { redactSecrets } from "./policy";
import { clearSession, exportClientState, rememberBearer } from "./session";

describe("DOM and export redaction", () => {
  it("never writes api_key or SecretRef values into the document", () => {
    const host = document.createElement("div");
    document.body.appendChild(host);
    const root = createRoot(host);
    const projection = redactSecrets({
      id: "acct-1",
      api_key: "sk-live-secret",
      secret_ref: "ss://provider/acct-1",
    });
    act(() => {
      root.render(<pre>{JSON.stringify(projection)}</pre>);
    });
    expect(host.textContent).not.toMatch(/sk-live|ss:\/\//);
    expect(host.textContent).toMatch(/"api_key":"present"/);
    expect(exportClientState()).toEqual({});
    act(() => {
      root.unmount();
    });
    host.remove();
  });
});

function renderApp(hash: string) {
  window.location.hash = hash;
  const host = document.createElement("div");
  document.body.appendChild(host);
  const root = createRoot(host);
  act(() => {
    root.render(<App />);
  });
  return { host, root };
}

describe("Shell identity and navigation", () => {
  afterEach(() => {
    clearSession();
    window.location.hash = "";
  });

  it("renders one product identity heading and the primary navigation landmark with every route", () => {
    const { host, root } = renderApp("#/session");
    const headings = host.querySelectorAll("h1");
    expect(headings.length).toBe(1);
    expect(headings[0].textContent).toBe("CognitiveOS Personal");
    const nav = host.querySelector('nav[aria-label="Primary"]');
    expect(nav).not.toBeNull();
    for (const label of [
      "Home",
      "Agents",
      "Providers",
      "Bindings",
      "Tasks",
      "Activity",
      "Resources",
      "Session",
    ]) {
      expect(nav?.textContent).toContain(label);
    }
    act(() => {
      root.unmount();
    });
    host.remove();
  });

  it("shows an in-place session gate for the dashboard when unauthenticated", () => {
    const { host, root } = renderApp("#/");
    expect(host.querySelector("[data-page='session-gate']")).not.toBeNull();
    expect(host.querySelector("main h2")?.textContent).toBe("Home");
    expect(host.textContent).toMatch(/not a Provider LLM API key/i);
    expect(host.textContent).toMatch(/local-bootstrap\.secret/);
    act(() => {
      root.unmount();
    });
    host.remove();
  });

  it("emits hash hrefs, not pathname routes that the daemon 404s", () => {
    const { host, root } = renderApp("#/session");
    const links = [...host.querySelectorAll("nav a")];
    expect(links.length).toBeGreaterThanOrEqual(8);
    for (const link of links) {
      const href = link.getAttribute("href") ?? "";
      expect(href.startsWith("#"), `href ${href}`).toBe(true);
      expect(href).not.toMatch(/^\/ui\//);
    }
    act(() => {
      root.unmount();
    });
    host.remove();
  });
});

describe("Provider hierarchy and authoritative-empty state", () => {
  afterEach(() => {
    clearSession();
    window.location.hash = "";
    vi.unstubAllGlobals();
  });

  it("shows the create/list hierarchy with an authoritative-empty table and never renders the bearer", async () => {
    const bearerValue = "mgmt-bearer-must-not-render-in-dom";
    rememberBearer("management", bearerValue);
    const fetchMock = vi.fn(async () =>
      new Response(JSON.stringify({ accounts: [] }), {
        status: 200,
        headers: { "content-type": "application/json" },
      }),
    );
    vi.stubGlobal("fetch", fetchMock);

    const { host, root } = renderApp("#/providers");
    await act(async () => {
      await Promise.resolve();
      await Promise.resolve();
    });

    expect(host.querySelector("h2")?.textContent).toBe("Providers");
    expect(host.querySelector('h3')?.textContent).toBe("Create named account");
    expect(host.textContent).toContain("No provider accounts yet");
    expect(host.textContent).not.toContain(bearerValue);
    expect(host.innerHTML).not.toContain(bearerValue);

    act(() => {
      root.unmount();
    });
    host.remove();
  });
});
