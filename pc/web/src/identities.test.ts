import { describe, expect, it } from "vitest";
import { AGENT_IDENTITY_KEYS, emptyIdentities, mergeIdentities } from "./identities";

describe("agent identity separation", () => {
  it("keeps package, installation, registration, instance, sidecar, execution, process, task, and shell distinct", () => {
    const merged = mergeIdentities({ instance: "inst-1", task: "task-1" });
    expect(AGENT_IDENTITY_KEYS).toHaveLength(9);
    expect(merged.instance).toBe("inst-1");
    expect(merged.task).toBe("task-1");
    expect(merged.package).toBe("unknown");
    expect(merged.process).not.toBe(merged.task);
    expect(Object.keys(emptyIdentities())).toEqual([...AGENT_IDENTITY_KEYS]);
  });
});
