import { describe, expect, it } from "vitest";
import { rejectCallerHeaderInjection } from "./api";

describe("provider request policy", () => {
  it("rejects arbitrary header injection on account or probe bodies", () => {
    expect(() => rejectCallerHeaderInjection({ headers: { Authorization: "sk-x" } })).toThrow(
      /header injection/,
    );
    expect(() => rejectCallerHeaderInjection({ authorization: "Bearer x" })).toThrow(
      /header injection/,
    );
    expect(() =>
      rejectCallerHeaderInjection({ display_name: "ok", provider_kind: "openai" }),
    ).not.toThrow();
  });
});
