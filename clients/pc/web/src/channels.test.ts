import { afterEach, describe, expect, it } from "vitest";
import { assertChannelBinding, authorizationHeader, classifyPath } from "./channels";
import { clearSession, rememberBearer } from "./session";

afterEach(() => {
  clearSession();
});

describe("channel isolation", () => {
  it("classifies Task routes as task and management routes as management", () => {
    expect(classifyPath("/task/v1/tasks")).toBe("task");
    expect(classifyPath("/management/provider/v1/accounts")).toBe("management");
    expect(classifyPath("/personal/status")).toBe("management");
    expect(classifyPath("/local/session")).toBe("none");
  });

  it("rejects a management bearer on a Task route and a Task bearer on a management route", () => {
    rememberBearer("management", "mgmt");
    rememberBearer("task", "task");
    expect(() => assertChannelBinding("/task/v1/watch", "management")).toThrow(
      "SHELL_CHANNEL_BINDING_MISMATCH",
    );
    expect(() => assertChannelBinding("/management/provider/v1/accounts", "task")).toThrow(
      "SHELL_CHANNEL_BINDING_MISMATCH",
    );
    expect(authorizationHeader("/personal/status", "management")).toBe("Bearer mgmt");
  });
});
