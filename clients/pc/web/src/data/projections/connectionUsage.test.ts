import { describe, expect, it } from "vitest";
import { connectionUsageLabel } from "./connectionUsage";

describe("connectionUsageLabel (P13-T08 source-labelled)", () => {
  it("never renders unknown or cost_unavailable as 0", () => {
    expect(connectionUsageLabel("a", [])).toBe("unknown");
    expect(
      connectionUsageLabel("a", [
        { id: "e1", accountId: "a", costStatus: "cost_unavailable" },
      ]),
    ).toBe("unknown");
    expect(
      connectionUsageLabel("a", [{ id: "e2", accountId: "a", costStatus: "unknown" }]),
    ).toBe("unknown");
    expect(connectionUsageLabel("a", [])).not.toBe("0");
  });

  it("labels priced rows actual or estimated and keeps a priced zero as actual $0.000000", () => {
    expect(
      connectionUsageLabel("b", [
        {
          id: "e3",
          accountId: "b",
          costMicros: 0,
          costStatus: "priced",
          costLabel: "actual",
        },
      ]),
    ).toBe("actual $0.000000");
    expect(
      connectionUsageLabel("c", [
        {
          id: "e4",
          accountId: "c",
          costMicros: 1500,
          costStatus: "priced",
          costLabel: "estimated",
        },
      ]),
    ).toBe("estimated $0.001500");
  });
});
