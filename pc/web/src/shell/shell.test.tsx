import { createRoot } from "react-dom/client";
import { act } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../App";
import { clearSession, rememberBearer } from "../session";

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

describe("shell accessibility and structure (W1)", () => {
  afterEach(() => {
    clearSession();
    window.location.hash = "";
    vi.unstubAllGlobals();
  });

  it("has a skip link targeting #main, one h1, and a labeled status contentinfo", () => {
    const { host, root } = renderApp("#/session");
    const skip = host.querySelector("a.skip");
    expect(skip?.getAttribute("href")).toBe("#main");
    expect(host.querySelector("#main")?.getAttribute("tabindex")).toBe("-1");
    expect(host.querySelectorAll("h1").length).toBe(1);
    expect(host.querySelector('[role="contentinfo"]')?.getAttribute("aria-label")).toBe(
      "System status",
    );
    act(() => root.unmount());
    host.remove();
  });

  it("marks the current nav item with aria-current and others without", () => {
    const { host, root } = renderApp("#/providers");
    const current = host.querySelector('nav[aria-label="Primary"] a[aria-current="page"]');
    expect(current?.textContent).toBe("Providers");
    const all = [...host.querySelectorAll('nav[aria-label="Primary"] a')];
    expect(all.length).toBe(7);
    expect(all.filter((a) => a.hasAttribute("aria-current")).length).toBe(1);
    act(() => root.unmount());
    host.remove();
  });

  it("state chips carry text labels (state is never color-only)", () => {
    rememberBearer("management", "mgmt-bearer");
    vi.stubGlobal(
      "fetch",
      vi.fn(
        async () =>
          new Response(JSON.stringify({ status: "ok", overall: "ready", alerts: [] }), {
            status: 200,
            headers: { "content-type": "application/json" },
          }),
      ),
    );
    const { host, root } = renderApp("#/");
    // Strip cells always render text next to dots.
    const strip = host.querySelector('[role="contentinfo"]');
    expect(strip?.textContent).toContain("daemon");
    const dots = [...host.querySelectorAll(".cp-dot")];
    for (const dot of dots) {
      expect(dot.getAttribute("aria-hidden")).toBe("true");
    }
    act(() => root.unmount());
    host.remove();
  });

  it("designed 404 exists and links home", () => {
    const { host, root } = renderApp("#/no/such/route");
    expect(host.textContent).toContain("No such route");
    const home = host.querySelector('a[href="#/"]');
    expect(home).not.toBeNull();
    act(() => root.unmount());
    host.remove();
  });

  it("unknown hash routes never render a blank main", () => {
    const { host, root } = renderApp("#/definitely-not-real");
    const main = host.querySelector("#main");
    expect(main?.textContent?.length).toBeGreaterThan(0);
    act(() => root.unmount());
    host.remove();
  });
});
