import { afterEach, describe, expect, it } from "vitest";
import { bearer, clearSession, exportClientState, rememberBearer } from "./session";

afterEach(() => {
  clearSession();
  window.localStorage.clear();
  window.sessionStorage.clear();
});

describe("memory-only session", () => {
  it("keeps bearers in memory and exports no session material", () => {
    rememberBearer("management", "mgmt-token");
    rememberBearer("task", "task-token");
    expect(bearer("management")).toBe("mgmt-token");
    expect(exportClientState()).toEqual({});
    expect(window.localStorage.length).toBe(0);
    expect(window.sessionStorage.length).toBe(0);
  });

  it("refuses to keep working if storage already holds session material", () => {
    window.localStorage.setItem("cognitiveos.token", "leaked");
    expect(() => rememberBearer("management", "mgmt-token")).toThrow(/must not persist/);
  });
});
