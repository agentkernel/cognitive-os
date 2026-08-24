import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";

/* ---------- test harness ---------- */

type RouteResponse = { status: number; body: unknown };
type RouteHandler = RouteResponse | ((call: { body?: any; url: URL }) => RouteResponse);

interface RecordedCall {
  method: string;
  path: string;
  query: URLSearchParams;
  body?: any;
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
  return { status: 404, body: { status: "error", code: "NOT_FOUND", message: "not found" } };
}

function installFetch(routes: Record<string, RouteHandler>): RecordedCall[] {
  const calls: RecordedCall[] = [];
  const fetchMock = vi.fn(async (input: unknown, init?: RequestInit) => {
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
      typeof handler === "function" ? handler({ body, url }) : (handler ?? defaultRoute(url.pathname));
    return new Response(JSON.stringify(resolved.body), {
      status: resolved.status,
      headers: { "content-type": "application/json" },
    });
  });
  vi.stubGlobal("fetch", fetchMock);
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

async function flush(ticks = 10) {
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

function setInputValue(input: HTMLInputElement, value: string) {
  const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value")?.set;
  setter?.call(input, value);
  input.dispatchEvent(new Event("input", { bubbles: true }));
}

function setSelectValue(select: HTMLSelectElement, value: string) {
  const setter = Object.getOwnPropertyDescriptor(window.HTMLSelectElement.prototype, "value")?.set;
  setter?.call(select, value);
  select.dispatchEvent(new Event("change", { bubbles: true }));
}

function submitForm(form: HTMLFormElement) {
  act(() => {
    form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
  });
}

function findButton(host: HTMLElement, text: string): HTMLButtonElement {
  const button = [...host.querySelectorAll("button")].find(
    (candidate) => candidate.textContent === text,
  );
  if (!button) {
    throw new Error(`button not found: ${text}`);
  }
  return button;
}

/* ---------- fixtures ---------- */

const ACCOUNT = {
  id: "acct-1",
  display_name: "deepseek-main",
  provider_kind: "openai_compatible",
  endpoint: "https://deepseek.local/v1",
  secret_ref: "ss://provider/acct-1",
  status: "active",
  catalog_revision: 12,
  last_discovery_error: "",
  allow_private_network: true,
  allow_insecure_http: false,
  network_scope: "private",
};

const MODELS = [
  {
    account_id: "acct-1",
    model_id: "deepseek-chat",
    source: "discovered",
    pricing_version: "v1",
    price_input_per_million: "0.27",
    price_output_per_million: "1.10",
  },
  {
    account_id: "acct-1",
    model_id: "grok-beta",
    source: "manual",
    pricing_version: "manual",
    price_input_per_million: null,
    price_output_per_million: null,
  },
];

function detailRoutes(overrides: Record<string, RouteHandler> = {}): Record<string, RouteHandler> {
  return {
    "GET /management/providers/accounts/inspect": {
      status: 200,
      body: { status: "ok", account: ACCOUNT },
    },
    "GET /management/providers/models": {
      status: 200,
      body: { status: "ok", models: MODELS },
    },
    "GET /management/agent-bindings": { status: 200, body: { status: "ok", bindings: [] } },
    "GET /management/usage": { status: 200, body: { status: "ok", events: [] } },
    "GET /management/budgets": { status: 200, body: { status: "ok", budgets: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/audit": { status: 200, body: { status: "ok", events: [] } },
    "GET /personal/dsh/runtime": {
      status: 200,
      body: { status: "ok", state: "INACTIVE", process_alive: false },
    },
    "GET /provider/v1/dsh/selected-model": {
      status: 200,
      body: { status: "ok", selected_model: "unset" },
    },
    ...overrides,
  };
}

async function renderDetail(routes: Record<string, RouteHandler> = {}) {
  rememberBearer("management", "test-bearer");
  const calls = installFetch(detailRoutes(routes));
  const view = renderAppAt("#/providers/acct-1");
  await flush();
  return { ...view, calls };
}

afterEach(() => {
  clearSession();
  appProjections.clear();
  window.location.hash = "";
  vi.unstubAllGlobals();
});

/* ---------- Providers master ---------- */

describe("Providers master page", () => {
  it("renders state chips with text labels in triage order (revoked/degraded before active)", async () => {
    rememberBearer("management", "test-bearer");
    installFetch({
      "GET /management/providers/accounts": {
        status: 200,
        body: {
          status: "ok",
          accounts: [
            { id: "a-active", display_name: "active-one", provider_kind: "openai_official", secret_ref: "ss://x", status: "active" },
            { id: "a-revoked", display_name: "revoked-one", provider_kind: "openai_official", secret_ref: null, status: "revoked" },
            { id: "a-degraded", display_name: "degraded-one", provider_kind: "openai_official", secret_ref: "ss://y", status: "degraded", last_discovery_error: "auth" },
          ],
        },
      },
    });
    const { host, root } = renderAppAt("#/providers");
    await flush();

    const rows = [...host.querySelectorAll("tbody tr")];
    expect(rows.length).toBe(3);
    expect(rows[0].textContent).toContain("revoked-one");
    expect(rows[1].textContent).toContain("degraded-one");
    expect(rows[2].textContent).toContain("active-one");

    // State chips carry verbatim text labels — state is never color-only.
    const text = host.textContent ?? "";
    expect(text).toContain("revoked");
    expect(text).toContain("degraded");
    expect(text).toContain("active");
    const chips = [...host.querySelectorAll(".cp-chip")];
    expect(chips.length).toBeGreaterThan(0);
    for (const chip of chips) {
      expect((chip.textContent ?? "").trim().length).toBeGreaterThan(0);
    }

    // secret_ref renders as presence only
    expect(text).toContain("secret present");
    expect(text).toContain("secret absent");
    expect(text).not.toContain("ss://");

    // Inspector opens for the selected row without navigating.
    const inspectButtons = [...host.querySelectorAll("button")].filter(
      (button) => button.textContent === "Inspect",
    );
    act(() => {
      (inspectButtons[0] as HTMLButtonElement).click();
    });
    const inspector = host.querySelector('aside[aria-label="Provider account inspector"]');
    expect(inspector?.textContent).toContain("revoked-one");

    unmount(host, root);
  });

  it("renders the honest not-run state when the daemon returns its 200 stub", async () => {
    rememberBearer("management", "test-bearer");
    installFetch({
      "GET /management/providers/accounts": {
        status: 200,
        body: {
          status: "ok",
          channel: "management",
          note: "authenticated personal front door; business routes deferred",
        },
      },
    });
    const { host, root } = renderAppAt("#/providers");
    await flush();
    expect(host.textContent).toContain("Not available over HTTP");
    expect(host.textContent).toContain("front-door stub");
    unmount(host, root);
  });
});

/* ---------- create flow ---------- */

describe("Account create flow", () => {
  it("gates private/HTTP creation behind trust confirmation, then persists", async () => {
    rememberBearer("management", "test-bearer");
    const calls = installFetch({
      "GET /management/providers/accounts": { status: 200, body: { status: "ok", accounts: [] } },
      "POST /management/providers/accounts": {
        status: 200,
        body: { status: "ok", account: { id: "acct-new" } },
      },
    });
    const { host, root } = renderAppAt("#/providers");
    await flush();

    const form = host.querySelector("form") as HTMLFormElement;
    (form.querySelector('input[name="display_name"]') as HTMLInputElement).value =
      "private-deepseek";
    act(() => {
      setSelectValue(
        form.querySelector('select[name="provider_kind"]') as HTMLSelectElement,
        "openai_compatible",
      );
    });
    act(() => {
      (form.querySelector('input[name="allow_private_network"]') as HTMLInputElement).click();
    });

    // Trust confirmation required but not given → blocked before any POST.
    submitForm(form);
    await flush(3);
    expect(host.textContent).toContain("Trust confirmation is required");
    expect(
      calls.filter((call) => call.method === "POST" && call.path === "/management/providers/accounts"),
    ).toHaveLength(0);

    // Confirm trust → the documented order proceeds to persist.
    act(() => {
      (form.querySelector('input[name="trust_confirmed"]') as HTMLInputElement).click();
    });
    submitForm(form);
    await flush();
    const posts = calls.filter(
      (call) => call.method === "POST" && call.path === "/management/providers/accounts",
    );
    expect(posts).toHaveLength(1);
    expect(posts[0].body.display_name).toBe("private-deepseek");
    expect(posts[0].body.provider_kind).toBe("openai_compatible");
    expect(posts[0].body.allow_private_network).toBe(true);
    // Key handoff pointer to the detail page; the key is never in this form.
    expect(host.querySelector('a[href="#/providers/acct-new"]')).not.toBeNull();
    expect(host.textContent).toContain("bounded probe");
    unmount(host, root);
  });

  it("shows the HTTP error class when creation fails", async () => {
    rememberBearer("management", "test-bearer");
    installFetch({
      "GET /management/providers/accounts": { status: 200, body: { status: "ok", accounts: [] } },
      "POST /management/providers/accounts": {
        status: 400,
        body: { status: "error", code: "PROVIDER_ENDPOINT_UNTRUSTED", message: "untrusted" },
      },
    });
    const { host, root } = renderAppAt("#/providers");
    await flush();
    const form = host.querySelector("form") as HTMLFormElement;
    (form.querySelector('input[name="display_name"]') as HTMLInputElement).value = "bad-endpoint";
    submitForm(form);
    await flush();
    expect(host.textContent).toContain("HTTP 400");
    expect(host.textContent).toContain("PROVIDER_ENDPOINT_UNTRUSTED");
    unmount(host, root);
  });
});

/* ---------- key handoff ---------- */

describe("Key handoff", () => {
  it("hands the key to the daemon without leaking it (op=rotate when present)", async () => {
    const { host, root, calls } = await renderDetail({
      "POST /management/providers/accounts/key": { status: 200, body: { status: "ok" } },
    });

    const keyInput = host.querySelector('input[type="password"]') as HTMLInputElement;
    expect(keyInput).not.toBeNull();
    setInputValue(keyInput, "sk-test-never-persist");
    submitForm(keyInput.closest("form") as HTMLFormElement);
    await flush();

    const keyPosts = calls.filter((call) => call.path === "/management/providers/accounts/key");
    expect(keyPosts).toHaveLength(1);
    // The POST body is the sanctioned handoff path; op chosen by presence.
    expect(keyPosts[0].body.op).toBe("rotate");
    expect(keyPosts[0].body.api_key).toBe("sk-test-never-persist");

    // Field cleared on submit; key never in DOM, URLs, or the projection store.
    expect((host.querySelector('input[type="password"]') as HTMLInputElement).value).toBe("");
    expect(host.innerHTML).not.toContain("sk-test-never-persist");
    expect(document.body.innerHTML).not.toContain("sk-test-never-persist");
    expect(
      calls.every(
        (call) => !call.path.includes("sk-test") && !String(call.query).includes("sk-test"),
      ),
    ).toBe(true);
    for (const key of [
      "provider:acct-1:account",
      "provider:acct-1:models",
      "bindings:all",
      "usage:all",
      "budgets:all",
      "alerts:all",
      "audit:all",
      "dsh:runtime",
      "dsh:selected",
    ]) {
      expect(JSON.stringify(appProjections.get(key) ?? {})).not.toContain("sk-test");
    }
    expect(host.textContent).toContain("SecretStore");
    unmount(host, root);
  });

  it("chooses op=set when no key is present", async () => {
    const { root, host, calls } = await renderDetail({
      "GET /management/providers/accounts/inspect": {
        status: 200,
        body: { status: "ok", account: { ...ACCOUNT, secret_ref: null } },
      },
      "POST /management/providers/accounts/key": { status: 200, body: { status: "ok" } },
    });
    const keyInput = host.querySelector('input[type="password"]') as HTMLInputElement;
    setInputValue(keyInput, "sk-first-set");
    submitForm(keyInput.closest("form") as HTMLFormElement);
    await flush();
    const keyPosts = calls.filter((call) => call.path === "/management/providers/accounts/key");
    expect(keyPosts).toHaveLength(1);
    expect(keyPosts[0].body.op).toBe("set");
    expect(host.textContent).toContain("secret absent");
    unmount(host, root);
  });
});

/* ---------- models ---------- */

describe("Models section", () => {
  it("keeps the last catalog when the bounded probe fails", async () => {
    const { host, root } = await renderDetail({
      "POST /management/providers/models/refresh": {
        status: 500,
        body: { status: "error", code: "PROVIDER_DISCOVERY_FAILED", message: "discovery_failed" },
      },
    });
    expect(host.textContent).toContain("deepseek-chat");
    act(() => {
      findButton(host, "Refresh catalog (bounded probe)").click();
    });
    await flush();
    expect(host.textContent).toContain("preserved");
    expect(host.textContent).toContain("deepseek-chat");
    expect(host.textContent).toContain("grok-beta");
    // Unknown prices stay text, never 0.
    expect(host.textContent).toContain("unknown");
    unmount(host, root);
  });

  it("adds a manual model and sets prices; failures show the error class", async () => {
    const { host, root, calls } = await renderDetail({
      "POST /management/providers/models/add": { status: 200, body: { status: "ok", model: {} } },
      "POST /management/providers/models/set-price": { status: 200, body: { status: "ok" } },
    });

    const addForm = [...host.querySelectorAll("form")].find((form) =>
      form.querySelector('input[name="model_id"]'),
    ) as HTMLFormElement;
    (addForm.querySelector('input[name="model_id"]') as HTMLInputElement).value = "deepseek-v3";
    submitForm(addForm);
    await flush();
    const adds = calls.filter((call) => call.path === "/management/providers/models/add");
    expect(adds).toHaveLength(1);
    expect(adds[0].body).toEqual({ account_id: "acct-1", model_id: "deepseek-v3" });
    expect(host.textContent).toContain("Manual model deepseek-v3 stored");

    const priceForm = [...host.querySelectorAll("form")].find((form) =>
      form.querySelector('input[name="price_input"]'),
    ) as HTMLFormElement;
    (priceForm.querySelector('input[name="price_model_id"]') as HTMLInputElement).value =
      "deepseek-chat";
    (priceForm.querySelector('input[name="price_input"]') as HTMLInputElement).value = "0.27";
    (priceForm.querySelector('input[name="price_output"]') as HTMLInputElement).value = "1.10";
    submitForm(priceForm);
    await flush();
    const prices = calls.filter((call) => call.path === "/management/providers/models/set-price");
    expect(prices).toHaveLength(1);
    expect(prices[0].body).toEqual({
      account_id: "acct-1",
      model_id: "deepseek-chat",
      price_input_per_million: "0.27",
      price_output_per_million: "1.10",
    });
    expect(host.textContent).toContain("Price stored for deepseek-chat");
    unmount(host, root);
  });

  it("surfaces set-price failure with its error code", async () => {
    const { host, root } = await renderDetail({
      "POST /management/providers/models/set-price": {
        status: 400,
        body: { status: "error", code: "PROVIDER_MODEL_NOT_FOUND", message: "no such model" },
      },
    });
    const priceForm = [...host.querySelectorAll("form")].find((form) =>
      form.querySelector('input[name="price_input"]'),
    ) as HTMLFormElement;
    (priceForm.querySelector('input[name="price_model_id"]') as HTMLInputElement).value = "ghost";
    (priceForm.querySelector('input[name="price_input"]') as HTMLInputElement).value = "0.01";
    submitForm(priceForm);
    await flush();
    expect(host.textContent).toContain("HTTP 400");
    expect(host.textContent).toContain("PROVIDER_MODEL_NOT_FOUND");
    unmount(host, root);
  });
});

/* ---------- bindings ---------- */

describe("Bindings section", () => {
  async function selectModelAndConfirm(host: HTMLElement) {
    act(() => {
      setSelectValue(
        host.querySelector('select[name="binding_model"]') as HTMLSelectElement,
        "deepseek-chat",
      );
    });
    const confirmBox = [...host.querySelectorAll('input[type="checkbox"]')].find((box) =>
      box.closest("label")?.textContent?.includes("Confirm this exact"),
    ) as HTMLInputElement;
    act(() => {
      confirmBox.click();
    });
  }

  it("previews expected revision 0 for a revoked binding and the live revision for an active one", async () => {
    const revoked = await renderDetail({
      "GET /management/agent-bindings": {
        status: 200,
        body: {
          status: "ok",
          bindings: [
            { agent: "pi", account_id: "acct-1", model_id: "deepseek-chat", revision: 7, status: "revoked" },
          ],
        },
      },
    });
    act(() => {
      setSelectValue(
        revoked.host.querySelector('select[name="binding_model"]') as HTMLSelectElement,
        "deepseek-chat",
      );
    });
    expect(revoked.host.querySelector('[data-testid="binding-preview"]')?.textContent).toContain(
      "expected revision 0",
    );
    unmount(revoked.host, revoked.root);

    const active = await renderDetail({
      "GET /management/agent-bindings": {
        status: 200,
        body: {
          status: "ok",
          bindings: [
            { agent: "pi", account_id: "acct-1", model_id: "deepseek-chat", revision: 4, status: "active" },
          ],
        },
      },
    });
    act(() => {
      setSelectValue(
        active.host.querySelector('select[name="binding_model"]') as HTMLSelectElement,
        "deepseek-chat",
      );
    });
    expect(active.host.querySelector('[data-testid="binding-preview"]')?.textContent).toContain(
      "expected revision 4",
    );
    unmount(active.host, active.root);
  });

  it("posts only the fixed tuple — never fallback or per-request override fields", async () => {
    let posted: any;
    const { host, root } = await renderDetail({
      "POST /management/agent-bindings": (call) => {
        posted = call.body;
        return { status: 200, body: { status: "ok" } };
      },
    });
    await selectModelAndConfirm(host);
    act(() => {
      findButton(host, "Set binding").click();
    });
    await flush();
    expect(posted).toEqual({
      agent: "pi",
      account_id: "acct-1",
      model_id: "deepseek-chat",
      expected_revision: 0,
    });
    expect("fallback" in posted).toBe(false);
    expect("per_request" in posted).toBe(false);
    expect("perRequestOverride" in posted).toBe(false);
    // Policy is stated as text, not offered as rejectable checkboxes.
    expect(host.textContent).toContain("Fallback and per-request override are forbidden");
    expect(host.textContent).toContain("Binding stored");
    unmount(host, root);
  });

  it("409 stale re-reads authority state and re-previews — never a silent retry", async () => {
    let bindingRevision = 4;
    let postCount = 0;
    const { host, root, calls } = await renderDetail({
      "GET /management/agent-bindings": () => ({
        status: 200,
        body: {
          status: "ok",
          bindings: [
            {
              agent: "pi",
              account_id: "acct-1",
              model_id: "deepseek-chat",
              revision: bindingRevision,
              status: "active",
            },
          ],
        },
      }),
      "POST /management/agent-bindings": () => {
        postCount += 1;
        bindingRevision = 5; // the authority state moved under us
        return {
          status: 409,
          body: { status: "error", code: "PROVIDER_BINDING_REVISION_STALE", message: "stale" },
        };
      },
    });
    await selectModelAndConfirm(host);
    expect(host.querySelector('[data-testid="binding-preview"]')?.textContent).toContain(
      "expected revision 4",
    );
    act(() => {
      findButton(host, "Set binding").click();
    });
    await flush();

    expect(postCount).toBe(1); // no automatic second POST
    const gets = calls.filter(
      (call) => call.method === "GET" && call.path === "/management/agent-bindings",
    );
    expect(gets.length).toBe(2); // initial load + re-read after the 409
    expect(host.textContent).toContain("changed under you");
    // Fresh preview presents the re-read revision for re-confirmation.
    expect(host.querySelector('[data-testid="binding-preview"]')?.textContent).toContain(
      "expected revision 5",
    );
    unmount(host, root);
  });
});

/* ---------- dsh apply ---------- */

describe("dsh apply gate", () => {
  const dshBinding = {
    agent: "dsh",
    account_id: "acct-1",
    model_id: "deepseek-chat",
    revision: 4,
    status: "active",
  };

  it("fails closed when the runtime is not ACTIVE", async () => {
    const { host, root } = await renderDetail({
      "GET /management/agent-bindings": {
        status: 200,
        body: { status: "ok", bindings: [dshBinding] },
      },
      "GET /personal/dsh/runtime": {
        status: 200,
        body: { status: "ok", state: "INACTIVE", process_alive: false },
      },
    });
    const apply = findButton(host, "Apply to running dsh");
    expect(apply.disabled).toBe(true);
    expect(host.textContent).toContain("not ACTIVE");
    unmount(host, root);
  });

  it("fails closed when the dsh binding is revoked", async () => {
    const { host, root } = await renderDetail({
      "GET /management/agent-bindings": {
        status: 200,
        body: { status: "ok", bindings: [{ ...dshBinding, status: "revoked" }] },
      },
      "GET /personal/dsh/runtime": {
        status: 200,
        body: { status: "ok", state: "ACTIVE", process_alive: true },
      },
    });
    const apply = findButton(host, "Apply to running dsh");
    expect(apply.disabled).toBe(true);
    expect(host.textContent).toContain("No active dsh binding");
    unmount(host, root);
  });

  it("fails closed when the bound model is not in this account catalog", async () => {
    const { host, root } = await renderDetail({
      "GET /management/agent-bindings": {
        status: 200,
        body: { status: "ok", bindings: [{ ...dshBinding, model_id: "ghost-model" }] },
      },
      "GET /personal/dsh/runtime": {
        status: 200,
        body: { status: "ok", state: "ACTIVE", process_alive: true },
      },
    });
    const apply = findButton(host, "Apply to running dsh");
    expect(apply.disabled).toBe(true);
    expect(host.textContent).toContain("not in this account catalog");
    unmount(host, root);
  });

  it("posts op:apply with the active binding revision when the gate passes", async () => {
    let posted: any;
    const { host, root } = await renderDetail({
      "GET /management/agent-bindings": {
        status: 200,
        body: { status: "ok", bindings: [dshBinding] },
      },
      "GET /personal/dsh/runtime": {
        status: 200,
        body: { status: "ok", state: "ACTIVE", process_alive: true },
      },
      "POST /personal/dsh/runtime": (call) => {
        posted = call.body;
        return { status: 200, body: { status: "ok", applied_model: "deepseek-chat", restart_performed: true } };
      },
    });
    const apply = findButton(host, "Apply to running dsh");
    expect(apply.disabled).toBe(false);
    act(() => {
      apply.click();
    });
    await flush();
    expect(posted.op).toBe("apply");
    expect(posted.expected_revision).toBe(4);
    expect(host.textContent).toContain("Applied deepseek-chat");
    unmount(host, root);
  });
});

/* ---------- usage / alerts / audit ---------- */

describe("Usage, alerts, audit", () => {
  it("renders cost_unavailable as text, never as 0", async () => {
    const { host, root } = await renderDetail({
      "GET /management/usage": {
        status: 200,
        body: {
          status: "ok",
          events: [
            { event_id: "ev-1", account_id: "acct-1", cost_micros: null, cost_status: "cost_unavailable" },
            { event_id: "ev-2", account_id: "acct-1", cost_micros: 1940000, cost_status: "priced" },
          ],
        },
      },
    });
    const usage = host.querySelector("#provider-usage") as HTMLElement;
    expect(usage.textContent).toContain("cost_unavailable");
    expect(usage.textContent).toContain("$1.940000");
    expect(usage.textContent).not.toContain("$0.000000");
    expect(usage.textContent).toContain("observe-only");
    unmount(host, root);
  });

  it("acknowledges an alert with a receipt", async () => {
    let posted: any;
    const { host, root } = await renderDetail({
      "GET /management/budgets": {
        status: 200,
        body: {
          status: "ok",
          budgets: [
            { budget_id: "b1", scope_kind: "account", scope_id: "acct-1", token_limit: 1000, amount_micros_limit: 10000000 },
          ],
        },
      },
      "GET /management/alerts": {
        status: 200,
        body: {
          status: "ok",
          alerts: [
            { alert_id: "al-1", budget_id: "b1", threshold_kind: "warning_80", issued_at_ms: 1, acknowledged_at_ms: null },
          ],
        },
      },
      "POST /management/alerts/acknowledge": (call) => {
        posted = call.body;
        return { status: 200, body: { status: "ok" } };
      },
    });
    expect(host.textContent).toContain("al-1");
    act(() => {
      findButton(host, "Acknowledge").click();
    });
    await flush();
    expect(posted).toEqual({ alert_id: "al-1" });
    expect(host.textContent).toContain("Alert al-1 acknowledged");
    unmount(host, root);
  });

  it("shows an error state when acknowledge fails", async () => {
    const { host, root } = await renderDetail({
      "GET /management/budgets": {
        status: 200,
        body: {
          status: "ok",
          budgets: [
            { budget_id: "b1", scope_kind: "account", scope_id: "acct-1", token_limit: 1000, amount_micros_limit: 10000000 },
          ],
        },
      },
      "GET /management/alerts": {
        status: 200,
        body: {
          status: "ok",
          alerts: [
            { alert_id: "al-1", budget_id: "b1", threshold_kind: "warning_80", issued_at_ms: 1, acknowledged_at_ms: null },
          ],
        },
      },
      "POST /management/alerts/acknowledge": {
        status: 500,
        body: { status: "error", code: "PROVIDER_STORE_LOCKED", message: "locked" },
      },
    });
    act(() => {
      findButton(host, "Acknowledge").click();
    });
    await flush();
    expect(host.textContent).toContain("HTTP 500");
    expect(host.textContent).toContain("PROVIDER_STORE_LOCKED");
    unmount(host, root);
  });

  it("renders the provider-plane audit coverage note and filtered rows", async () => {
    const { host, root } = await renderDetail({
      "GET /management/audit": {
        status: 200,
        body: {
          status: "ok",
          events: [
            { audit_id: "aud-1", action: "account.created", outcome: "ok", detail: "acct-1" },
            { audit_id: "aud-2", action: "key.rotated", outcome: "ok", detail: "acct-1" },
            { audit_id: "aud-3", action: "binding.set", outcome: "ok", detail: "other-account" },
          ],
        },
      },
    });
    const audit = host.querySelector("#provider-audit") as HTMLElement;
    expect(audit.textContent).toContain(
      "Provider-plane audit only — not a complete system event stream (BD-5).",
    );
    expect(audit.textContent).toContain("key.rotated");
    expect(audit.textContent).not.toContain("binding.set");
    unmount(host, root);
  });
});

/* ---------- routing + structure ---------- */

describe("routing and structure", () => {
  it("redirects #/bindings to #/providers", async () => {
    rememberBearer("management", "test-bearer");
    installFetch({
      "GET /management/providers/accounts": { status: 200, body: { status: "ok", accounts: [] } },
    });
    const { host, root } = renderAppAt("#/bindings");
    await flush();
    expect(window.location.hash).toBe("#/providers");
    expect(host.querySelector("h2")?.textContent).toBe("Providers");
    unmount(host, root);
  });

  it("keeps landmarks, labelled tables, and focusable actions on the detail page", async () => {
    const { host, root } = await renderDetail();
    expect(host.querySelector("h2")?.textContent).toBe("deepseek-main");
    expect(host.querySelector('nav[aria-label="Account sections"]')).not.toBeNull();
    for (const id of [
      "provider-overview",
      "provider-models",
      "provider-bindings",
      "provider-usage",
      "provider-audit",
    ]) {
      expect(host.querySelector(`#${id}`)).not.toBeNull();
    }
    const tables = [...host.querySelectorAll("table")];
    expect(tables.length).toBeGreaterThan(0);
    for (const table of tables) {
      expect(table.querySelector("caption")).not.toBeNull();
    }
    // Actions are real labelled buttons (no icon-only / unlabeled controls).
    const buttons = [...host.querySelectorAll("button")];
    expect(buttons.length).toBeGreaterThan(0);
    for (const button of buttons) {
      expect((button.textContent ?? "").trim().length).toBeGreaterThan(0);
    }
    // Raw projection is collapsed and redacted.
    const details = host.querySelector("details");
    expect(details?.textContent).toContain("Raw projection");
    expect(details?.textContent).not.toContain("ss://");
    unmount(host, root);
  });
});
