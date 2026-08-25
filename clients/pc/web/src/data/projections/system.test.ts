import { describe, expect, it } from "vitest";
import { projectDoctor } from "./system";

describe("System doctor projection (W9)", () => {
  it("names NOT_PROBED sub-sections as unprobed rather than omitting them", () => {
    const view = projectDoctor({
      overall: "ready",
      first_conversation_ready: true,
      evaluated_at_unix_ms: 1,
      components: [
        {
          component: "provider",
          status: "degraded",
          source: "readiness",
          facts: [{ key: "catalog", value: "stale" }],
        },
      ],
      guidance: ["refresh the provider catalog"],
      six_resource: { overall: "blocked", error_code: "RESOURCE_HEALTH_NOT_PROBED" },
      headless_vault: { overall: "unavailable", error_code: "VAULT_PATH_NOT_PROBED" },
      operability: { overall: "blocked", error_code: "OPERABILITY_TOPIC_NOT_PROBED" },
      static_check_is_not_runtime_ready: true,
      profile_claim: "not-claimed",
      gate_claim: "not-claimed",
    });
    expect(view.sixResource.probed).toBe(false);
    expect(view.headlessVault.probed).toBe(false);
    expect(view.operability.probed).toBe(false);
    expect(view.sixResource.errorCode).toContain("NOT_PROBED");
    expect(view.guidance).toEqual(["refresh the provider catalog"]);
    expect(view.components[0].facts[0].value).toBe("stale");
  });
});
