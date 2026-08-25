import { describe, expect, it, vi } from "vitest";
import { createProjectionStore, type Projection } from "./store";

describe("projection store (zero-dependency)", () => {
  it("sets and gets projections by key", () => {
    const store = createProjectionStore();
    const value: Projection<{ n: number }> = {
      status: "ready",
      data: { n: 1 },
      source: "/personal/status",
    };
    store.set("k", value);
    expect(store.get<{ n: number }>("k")).toBe(value);
    expect(store.get("absent")).toBeUndefined();
  });

  it("notifies subscribers on set and supports unsubscribe", () => {
    const store = createProjectionStore();
    const listener = vi.fn();
    const unsubscribe = store.subscribe(listener);
    store.set("a", { status: "loading" });
    store.set("a", { status: "ready", data: 1 });
    expect(listener).toHaveBeenCalledTimes(2);
    unsubscribe();
    store.set("a", { status: "stale" });
    expect(listener).toHaveBeenCalledTimes(2);
  });

  it("keeps snapshot references stable between writes (immutable replace)", () => {
    const store = createProjectionStore();
    store.set("k", { status: "ready", data: 1 });
    const first = store.get("k");
    store.set("other", { status: "loading" });
    expect(store.get("k")).toBe(first);
  });
});
