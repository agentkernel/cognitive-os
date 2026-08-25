import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "../../App";
import { isKnownRoute } from "../../data/normalize";
import { appProjections } from "../../data/store";
import { clearSession, rememberBearer } from "../../session";

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

function skillRoutes(overrides: Record<string, RouteHandler> = {}): Record<string, RouteHandler> {
  return {
    "GET /personal/health": { status: 200, body: { status: "ok" } },
    "GET /personal/status": { status: 200, body: { status: "ok", overall: "ready", components: [] } },
    "GET /management/alerts": { status: 200, body: { status: "ok", alerts: [] } },
    "GET /management/resource/v1/list": ({ url }) => {
      if (url.searchParams.get("family") === "skill") {
        return {
          status: 200,
          body: {
            kind: "resource.manager.list",
            family: "skill",
            authority_source: "sqlite-authority-skill-bindings",
            truncated: false,
            resources: [
              { id: "bind-1", family: "skill", health: "bound" },
              { id: "bind-2", family: "skill", health: "revoked" },
            ],
          },
        };
      }
      return { status: 200, body: { status: "ok", resources: [] } };
    },
    "GET /management/resource/v1/skill/binding/explain": {
      status: 200,
      body: {
        kind: "skill.binding.explain",
        authority_source: "daemon-skill-store",
        binding: {
          binding_id: "bind-1",
          revision_id: "rev-1",
          workspace_scope: "workspace://personal",
          target_kind: "workspace",
          target_ref: "workspace://personal/skills",
          status: "active",
          package_id: "pkg-1",
          manifest_digest: "sha256:manifest",
          content_digest: "sha256:content",
        },
      },
    },
    "POST /management/resource/v1/skill/import": {
      status: 201,
      body: { status: "imported", package_id: "pkg-new", revision_id: "rev-new" },
    },
    "POST /management/resource/v1/skill/bind": {
      status: 201,
      body: { status: "bound", binding_id: "bind-new" },
    },
    "POST /management/resource/v1/skill/binding/revoke": {
      status: 201,
      body: { status: "revoked", binding_id: "bind-1" },
    },
    ...overrides,
  };
}

async function renderSkills(hash = "#/resources/skill", overrides: Record<string, RouteHandler> = {}) {
  rememberBearer("management", "test-management-bearer");
  const calls = installFetch(skillRoutes(overrides));
  const view = renderAppAt(hash);
  await flush();
  return { ...view, calls };
}

describe("Skills family page (W7)", () => {
  afterEach(() => {
    clearSession();
    appProjections.clear();
    window.location.hash = "";
    vi.unstubAllGlobals();
  });

  it("lists bindings, names content≠permission once, and does not treat the list as packages", async () => {
    const { host, root, calls } = await renderSkills();
    expect(host.querySelector("main h2")?.textContent).toBe("Skills");
    expect(host.textContent).toContain("bind-1");
    expect(host.textContent).toContain("list is bindings, not packages");
    expect(host.textContent).toContain("grants no tool, filesystem, network, or model capability");
    expect(host.querySelectorAll("[data-annotation='skill-permission']")).toHaveLength(1);
    expect(host.querySelector("input[type='search']")).toBeNull();
    expect(calls.map((call) => `${call.method} ${call.path}`)).toContain(
      "GET /management/resource/v1/list",
    );
    expect(isKnownRoute("GET", "/management/resource/v1/skill/binding/explain")).toBe(true);
    expect(isKnownRoute("POST", "/management/resource/v1/skill/import")).toBe(true);
    unmount(host, root);
  });

  it("explains a selected binding and never fabricates compatibility", async () => {
    const { host, root, calls } = await renderSkills();
    const inspect = [...host.querySelectorAll("button")].find(
      (button) => (button.textContent ?? "").trim() === "Inspect",
    );
    expect(inspect).toBeDefined();
    await act(async () => {
      inspect?.click();
    });
    await flush();
    expect(host.textContent).toContain("rev-1");
    expect(host.textContent).toContain("pkg-1");
    expect(host.textContent).toContain("sha256:manifest");
    expect(host.textContent).toContain("unknown (binding explain does not carry it)");
    expect(host.textContent).toMatch(/durable revocation/);
    expect(calls.map((call) => `${call.method} ${call.path}`)).toContain(
      "GET /management/resource/v1/skill/binding/explain",
    );
    unmount(host, root);
  });

  it("names explain 404 as a gap rather than an empty family", async () => {
    const { host, root } = await renderSkills("#/resources/skill", {
      "GET /management/resource/v1/skill/binding/explain": {
        status: 404,
        body: {
          status: "error",
          code: "RESOURCE_SKILL_BINDING_NOT_FOUND",
          message: "Skill binding not found",
        },
      },
    });
    const inspect = [...host.querySelectorAll("button")].find(
      (button) => (button.textContent ?? "").trim() === "Inspect",
    );
    await act(async () => {
      inspect?.click();
    });
    await flush();
    expect(host.textContent).toContain("RESOURCE_SKILL_BINDING_NOT_FOUND");
    expect(host.textContent).not.toMatch(/No skill bindings/);
    unmount(host, root);
  });

  it("previews import and posts operator-supplied path and digests without previous_revision_id", async () => {
    const { host, root, calls } = await renderSkills();
    const form = host.querySelector("form[data-skill-form='import']") as HTMLFormElement;
    await act(async () => {
      (form.querySelector("input[name='workspace_scope']") as HTMLInputElement).value =
        "workspace://personal";
      (form.querySelector("input[name='local_source_path']") as HTMLInputElement).value =
        "skills/example";
      (form.querySelector("input[name='manifest_digest']") as HTMLInputElement).value =
        "sha256:manifest";
      (form.querySelector("input[name='content_digest']") as HTMLInputElement).value =
        "sha256:content";
      form.requestSubmit();
    });
    await flush();
    expect(host.textContent).toContain("Confirm import");
    expect(host.textContent).toContain("does not read the local path");
    const confirm = host.querySelector(".cp-confirm input[type='checkbox']") as HTMLInputElement;
    await act(async () => {
      confirm.click();
    });
    const importButton = [...host.querySelectorAll(".cp-confirm button")].find(
      (button) => (button.textContent ?? "").trim() === "Import",
    ) as HTMLButtonElement;
    await act(async () => {
      importButton.click();
    });
    await flush();
    const posted = calls.find((call) => call.path === "/management/resource/v1/skill/import");
    expect(posted?.method).toBe("POST");
    const body = posted?.body as Record<string, unknown>;
    expect(body.previous_revision_id).toBeUndefined();
    expect(body.local_source_path).toBe("skills/example");
    expect(body.manifest_digest).toBe("sha256:manifest");
    expect(typeof body.package_id).toBe("string");
    expect(typeof body.revision_id).toBe("string");
    unmount(host, root);
  });

  it("does not post import before confirmation", async () => {
    const { host, root, calls } = await renderSkills();
    const form = host.querySelector("form[data-skill-form='import']") as HTMLFormElement;
    await act(async () => {
      (form.querySelector("input[name='workspace_scope']") as HTMLInputElement).value =
        "workspace://personal";
      (form.querySelector("input[name='local_source_path']") as HTMLInputElement).value = "skills/x";
      (form.querySelector("input[name='manifest_digest']") as HTMLInputElement).value = "sha256:m";
      (form.querySelector("input[name='content_digest']") as HTMLInputElement).value = "sha256:c";
      form.requestSubmit();
    });
    await flush();
    expect(host.textContent).toContain("Confirm import");
    expect(calls.some((call) => call.path === "/management/resource/v1/skill/import")).toBe(false);
    unmount(host, root);
  });

  it("previews bind and posts minted binding_id with the operator revision", async () => {
    const { host, root, calls } = await renderSkills();
    const form = host.querySelector("form[data-skill-form='bind']") as HTMLFormElement;
    await act(async () => {
      (form.querySelector("input[name='revision_id']") as HTMLInputElement).value = "rev-1";
      (form.querySelector("input[name='workspace_scope']") as HTMLInputElement).value =
        "workspace://personal";
      (form.querySelector("input[name='target_kind']") as HTMLInputElement).value = "workspace";
      (form.querySelector("input[name='target_ref']") as HTMLInputElement).value =
        "workspace://personal";
      form.requestSubmit();
    });
    await flush();
    expect(host.textContent).toContain("Confirm bind");
    const confirm = [...host.querySelectorAll(".cp-confirm")]
      .find((node) => node.textContent?.includes("Confirm bind"))
      ?.querySelector("input[type='checkbox']") as HTMLInputElement;
    await act(async () => {
      confirm.click();
    });
    const bind = [...host.querySelectorAll(".cp-confirm button")].find(
      (button) => (button.textContent ?? "").trim() === "Bind",
    ) as HTMLButtonElement;
    await act(async () => {
      bind.click();
    });
    await flush();
    const posted = calls.find((call) => call.path === "/management/resource/v1/skill/bind");
    const body = posted?.body as Record<string, unknown>;
    expect(body.revision_id).toBe("rev-1");
    expect(typeof body.binding_id).toBe("string");
    unmount(host, root);
  });

  it("refuses revoke without a reason and posts revocation_id only after confirm", async () => {
    const { host, root, calls } = await renderSkills();
    const inspect = [...host.querySelectorAll("button")].find(
      (button) => (button.textContent ?? "").trim() === "Inspect",
    );
    await act(async () => {
      inspect?.click();
    });
    await flush();
    const form = host.querySelector("form[data-skill-form='revoke']") as HTMLFormElement;
    expect(form).not.toBeNull();
    expect(calls.some((call) => call.path === "/management/resource/v1/skill/binding/revoke")).toBe(
      false,
    );
    await act(async () => {
      form.requestSubmit();
    });
    await flush();
    expect(host.textContent).not.toContain("Confirm revoke");
    await act(async () => {
      (form.querySelector("input[name='reason']") as HTMLInputElement).value = "operator revoke";
      form.requestSubmit();
    });
    await flush();
    expect(host.textContent).toContain("Confirm revoke");
    const confirm = [...host.querySelectorAll(".cp-confirm")]
      .find((node) => node.textContent?.includes("Confirm revoke"))
      ?.querySelector("input[type='checkbox']") as HTMLInputElement;
    await act(async () => {
      confirm.click();
    });
    const revoke = [...host.querySelectorAll(".cp-confirm button")].find(
      (button) => (button.textContent ?? "").trim() === "Revoke",
    ) as HTMLButtonElement;
    await act(async () => {
      revoke.click();
    });
    await flush();
    const posted = calls.find((call) => call.path === "/management/resource/v1/skill/binding/revoke");
    const body = posted?.body as Record<string, unknown>;
    expect(body.binding_id).toBe("bind-1");
    expect(body.reason).toBe("operator revoke");
    expect(typeof body.revocation_id).toBe("string");
    unmount(host, root);
  });
});
