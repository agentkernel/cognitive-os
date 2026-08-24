import { afterEach, describe, expect, it, vi } from "vitest";
import { fetchProjection } from "./fetchProjection";
import { createProjectionStore } from "./store";
import { projectProviderAccounts, secretPresenceOf } from "./projections";
import { rememberBearer, clearSession } from "../session";

function mockFetch(status: number, body: unknown) {
  return vi.fn(
    async () =>
      new Response(typeof body === "string" ? body : JSON.stringify(body), {
        status,
        headers: { "content-type": "application/json" },
      }),
  );
}

describe("fetchProjection — honest status for every failure class", () => {
  afterEach(() => {
    clearSession();
    vi.unstubAllGlobals();
  });

  it("ready with projected data on success", async () => {
    rememberBearer("management", "test-bearer");
    vi.stubGlobal(
      "fetch",
      mockFetch(200, { status: "ok", accounts: [{ id: "a1", secret_ref: "ss://x" }] }),
    );
    const store = createProjectionStore();
    const projection = await fetchProjection(
      store,
      "accounts",
      "/management/providers/accounts",
      "management",
      projectProviderAccounts,
    );
    expect(projection.status).toBe("ready");
    expect(projection.data).toHaveLength(1);
    expect(projection.source).toBe("/management/providers/accounts");
    // secret_ref never reaches the view model — presence only
    expect(projection.data?.[0].secret).toBe("present");
    expect(JSON.stringify(projection)).not.toContain("ss://");
  });

  it("empty when the projected list is authoritative-empty", async () => {
    rememberBearer("management", "test-bearer");
    vi.stubGlobal("fetch", mockFetch(200, { status: "ok", accounts: [] }));
    const store = createProjectionStore();
    const projection = await fetchProjection(
      store,
      "accounts",
      "/management/providers/accounts",
      "management",
      projectProviderAccounts,
    );
    expect(projection.status).toBe("empty");
  });

  it("denied on 401/403", async () => {
    rememberBearer("management", "test-bearer");
    vi.stubGlobal(
      "fetch",
      mockFetch(403, {
        status: "error",
        error: { code: "LOCAL_SESSION_UNAUTHORIZED", message: "denied" },
      }),
    );
    const store = createProjectionStore();
    const projection = await fetchProjection(
      store,
      "accounts",
      "/management/providers/accounts",
      "management",
      projectProviderAccounts,
    );
    expect(projection.status).toBe("denied");
    expect(projection.error?.code).toBe("LOCAL_SESSION_UNAUTHORIZED");
  });

  it("disconnected when fetch throws", async () => {
    rememberBearer("management", "test-bearer");
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => {
        throw new Error("connection refused");
      }),
    );
    const store = createProjectionStore();
    const projection = await fetchProjection(
      store,
      "accounts",
      "/management/providers/accounts",
      "management",
      projectProviderAccounts,
    );
    expect(projection.status).toBe("disconnected");
  });

  it("not-run when the daemon returns its 200 stub", async () => {
    rememberBearer("management", "test-bearer");
    vi.stubGlobal(
      "fetch",
      mockFetch(200, {
        status: "ok",
        channel: "management",
        note: "authenticated personal front door; business routes deferred",
      }),
    );
    const store = createProjectionStore();
    const projection = await fetchProjection(
      store,
      "accounts",
      "/management/providers/accounts",
      "management",
      projectProviderAccounts,
    );
    expect(projection.status).toBe("not-run");
    expect(projection.error?.code).toBe("STUB_ROUTE");
  });

  it("not-run without fetching when the route is not whitelisted", async () => {
    rememberBearer("management", "test-bearer");
    const fetchMock = mockFetch(200, { status: "ok" });
    vi.stubGlobal("fetch", fetchMock);
    const store = createProjectionStore();
    const projection = await fetchProjection(
      store,
      "cancel",
      "/task/cancel",
      "task",
      projectProviderAccounts,
    );
    expect(projection.status).toBe("not-run");
    expect(projection.error?.code).toBe("ROUTE_NOT_WHITELISTED");
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("secretPresenceOf distinguishes present/absent/unknown without values", () => {
    expect(secretPresenceOf({ secret_ref: "ss://provider/x" })).toBe("present");
    expect(secretPresenceOf({ secret_ref: null })).toBe("absent");
    expect(secretPresenceOf({ secret_ref: "" })).toBe("absent");
    expect(secretPresenceOf({ id: "no-field" })).toBe("unknown");
  });
});
