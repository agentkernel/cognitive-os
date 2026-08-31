import { describe, expect, it } from "vitest";
import { connectionUsageLabel } from "./connectionUsage";

describe("connectionUsageLabel (P12-T08)", () => {
  it("never renders unknown or cost_unavailable as 0", () => {
    expect(connectionUsageLabel("a", [])).toBe("unknown");
    expect(
      connectionUsageLabel("a", [
        { id: "e1", accountId: "a", costStatus: "cost_unavailable" },
      ]),
    ).toBe("cost_unavailable");
    expect(
      connectionUsageLabel("a", [{ id: "e2", accountId: "a", costStatus: "unknown" }]),
    ).toBe("unknown");
    expect(
      connectionUsageLabel("b", [{ id: "e3", accountId: "b", costMicros: 0, costStatus: "priced" }]),
    ).toBe("$0.000000");
  });
});
