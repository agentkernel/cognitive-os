import { describe, expect, it } from "vitest";
import {
  TOOL_READINESS_CAVEAT,
  allowedToolMutations,
  projectToolCatalog,
  readinessLabel,
  toolMutationBody,
} from "./tools";

describe("tool catalog projection", () => {
  it("reads overlay lifecycle and readiness without inventing agent exposure", () => {
    const view = projectToolCatalog({
      kind: "tool.lifecycle.projection",
      authority_source: "daemon-native-tool-registry",
      resources: [
        {
          operation_id: "native.workspace.read",
          risk: "read",
          lifecycle: "enabled",
          execution_readiness: "execution_ready",
          descriptor_digest: "sha256:read",
          agent_exposed: true,
        },
        {
          operation_id: "native.process.check",
          risk: "process",
          lifecycle: "quarantined",
          execution_readiness: "registered_only",
        },
      ],
    });
    expect(view.resources[0]?.operationId).toBe("native.workspace.read");
    expect(view.resources[0]?.lifecycle).toBe("enabled");
    expect(view.resources[1]?.lifecycle).toBe("quarantined");
    expect(view.resources[1]?.agentExposed).toBe(false);
    expect(readinessLabel("execution_ready")).toBe("execution-ready*");
  });
});

describe("tool overlay transitions", () => {
  it("refuses quarantined→enabled and treats revoke as terminal", () => {
    expect(allowedToolMutations("quarantined")).toEqual(["disable", "revoke"]);
    expect(allowedToolMutations("quarantined")).not.toContain("enable");
    expect(allowedToolMutations("revoked")).toEqual([]);
    expect(toolMutationBody("native.workspace.read")).toEqual({
      operation_id: "native.workspace.read",
    });
    expect(TOOL_READINESS_CAVEAT).toMatch(/production call chain/);
  });
});
