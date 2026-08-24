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

  it("keeps last-good data as stale while a refetch is in flight, then settles", async () => {
    rememberBearer("management", "test-bearer");
    vi.stubGlobal(
      "fetch",
      mockFetch(200, { status: "ok", accounts: [{ id: "a1", secret_ref: "ss://x" }] }),
    );
    const store = createProjectionStore();
    const first = await fetchProjection(
      store,
      "accounts",
      "/management/providers/accounts",
      "management",
      projectProviderAccounts,
    );
    expect(first.status).toBe("ready");

    let release: (response: Response) => void = () => {};
    vi.stubGlobal(
      "fetch",
      vi.fn(
        () =>
          new Promise<Response>((resolve) => {
            release = resolve;
          }),
      ),
    );
    const pending = fetchProjection(
      store,
      "accounts",
      "/management/providers/accounts",
      "management",
      projectProviderAccounts,
    );

    // The in-flight state carries the previous data forward, labelled stale
    // with its own age and source — a refresh never blanks the surface.
    const inFlight = store.get<unknown[]>("accounts");
    expect(inFlight?.status).toBe("stale");
    expect(inFlight?.data).toHaveLength(1);
    expect(inFlight?.updatedAt).toBe(first.updatedAt);
    expect(inFlight?.source).toBe("/management/providers/accounts");
    expect(inFlight?.error).toBeUndefined();

    release(
      new Response(
        JSON.stringify({
          status: "ok",
          accounts: [{ id: "a1", secret_ref: "ss://x" }, { id: "a2", secret_ref: null }],
        }),
        { status: 200, headers: { "content-type": "application/json" } },
      ),
    );
    const settled = await pending;
    expect(settled.status).toBe("ready");
    expect(settled.data).toHaveLength(2);
    expect(store.get<unknown[]>("accounts")?.status).toBe("ready");
  });

  it("a failed refetch reports its failure class and never inherits last-good data", async () => {
    const load = async (store: ReturnType<typeof createProjectionStore>) => {
      vi.stubGlobal(
        "fetch",
        mockFetch(200, { status: "ok", accounts: [{ id: "a1", secret_ref: "ss://x" }] }),
      );
      const ready = await fetchProjection(
        store,
        "accounts",
        "/management/providers/accounts",
        "management",
        projectProviderAccounts,
      );
      expect(ready.status).toBe("ready");
    };
    const refetch = (store: ReturnType<typeof createProjectionStore>, path: string) =>
      fetchProjection(store, "accounts", path, "management", projectProviderAccounts);

    rememberBearer("management", "test-bearer");

    // denied (403)
    const denied = createProjectionStore();
    await load(denied);
    vi.stubGlobal(
      "fetch",
      mockFetch(403, {
        status: "error",
        error: { code: "LOCAL_SESSION_UNAUTHORIZED", message: "denied" },
      }),
    );
    let projection = await refetch(denied, "/management/providers/accounts");
    expect(projection.status).toBe("denied");
    expect(projection.error?.code).toBe("LOCAL_SESSION_UNAUTHORIZED");
    expect(projection.data).toBeUndefined();

    // disconnected (throw)
    const disconnected = createProjectionStore();
    await load(disconnected);
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => {
        throw new Error("connection refused");
      }),
    );
    projection = await refetch(disconnected, "/management/providers/accounts");
    expect(projection.status).toBe("disconnected");
    expect(projection.data).toBeUndefined();

    // not-run (daemon 200 stub)
    const stub = createProjectionStore();
    await load(stub);
    vi.stubGlobal(
      "fetch",
      mockFetch(200, {
        status: "ok",
        channel: "management",
        note: "authenticated personal front door; business routes deferred",
      }),
    );
    projection = await refetch(stub, "/management/providers/accounts");
    expect(projection.status).toBe("not-run");
    expect(projection.error?.code).toBe("STUB_ROUTE");
    expect(projection.data).toBeUndefined();

    // not-run (route not whitelisted — still no request issued)
    const offRoute = createProjectionStore();
    await load(offRoute);
    const fetchMock = mockFetch(200, { status: "ok" });
    vi.stubGlobal("fetch", fetchMock);
    projection = await refetch(offRoute, "/task/cancel");
    expect(projection.status).toBe("not-run");
    expect(projection.error?.code).toBe("ROUTE_NOT_WHITELISTED");
    expect(projection.data).toBeUndefined();
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("secretPresenceOf distinguishes present/absent/unknown without values", () => {
    expect(secretPresenceOf({ secret_ref: "ss://provider/x" })).toBe("present");
    expect(secretPresenceOf({ secret_ref: null })).toBe("absent");
    expect(secretPresenceOf({ secret_ref: "" })).toBe("absent");
    expect(secretPresenceOf({ id: "no-field" })).toBe("unknown");
  });
});
