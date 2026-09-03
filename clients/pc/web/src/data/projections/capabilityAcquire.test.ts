import { describe, expect, it } from "vitest";
import {
  CAPABILITY_ACQUIRE_PATH,
  EMPTY_REVIEW,
  acquireBody,
  reviewIsComplete,
} from "./capabilityAcquire";

describe("P13-T10 capability acquire Dual Track", () => {
  it("refuses an incomplete review as not ready to acquire", () => {
    expect(reviewIsComplete("mcp", EMPTY_REVIEW)).toBe(false);
    expect(
      reviewIsComplete("mcp", {
        ...EMPTY_REVIEW,
        source: "https://example.invalid/mcp/search",
        license: "MIT",
        sources: "https://example.invalid/mcp/search",
        toolPermissions: "search",
        supplyChain: "pinned-origin",
      }),
    ).toBe(true);
  });

  it("builds an acquire body that never claims a grant", () => {
    const body = acquireBody({
      projectId: "proj-1",
      employeeId: "emp-1",
      capabilityRef: "mcp:search",
      versionPin: "1.0.0",
      kind: "mcp",
      scope: "project-a",
      phase: "install",
      draft: {
        ...EMPTY_REVIEW,
        source: "https://example.invalid/mcp/search",
        license: "MIT",
        sources: "https://example.invalid/mcp/search",
        networkIntent: "declared",
        toolPermissions: "search",
        supplyChain: "pinned-origin",
      },
    });
    expect(CAPABILITY_ACQUIRE_PATH).toBe("/management/project/v1/capability.acquire");
    expect(body.phase).toBe("install");
    expect(body.granted).toBeUndefined();
    expect(body.scope).toBe("project-a");
  });
});
