import { describe, expect, it } from "vitest";
import {
  EFFECT_STAGE_CATEGORY,
  TASK_STATE_CATEGORY,
  readBindingState,
  readDomainState,
} from "./stateMap";

describe("state system — verbatim domain vocabularies", () => {
  it("covers all 9 registered Task states", () => {
    const states = [
      "DRAFT",
      "READY",
      "ACTIVE",
      "BLOCKED",
      "CANDIDATE_COMPLETE",
      "COMPLETED",
      "FAILED",
      "CANCELLED",
      "ESCALATED",
    ];
    for (const state of states) {
      expect(TASK_STATE_CATEGORY[state], `task state ${state}`).toBeDefined();
    }
    expect(Object.keys(TASK_STATE_CATEGORY)).toHaveLength(9);
    expect(readDomainState("task", "COMPLETED").category).toBe("completed");
    expect(readDomainState("task", "ACTIVE").category).toBe("active");
    expect(readDomainState("task", "DRAFT").category).toBe("waiting");
  });

  it("covers all 14 registered Effect stages", () => {
    const stages = [
      "PROPOSED",
      "AUTHORIZED",
      "DENIED",
      "EXECUTING",
      "EXECUTED",
      "OUTCOME_UNKNOWN",
      "RECONCILED",
      "VERIFIED",
      "VERIFY_FAILED",
      "COMPENSATING",
      "NOT_EXECUTED",
      "COMMITTED",
      "ABORTED",
      "QUARANTINED",
    ];
    for (const stage of stages) {
      expect(EFFECT_STAGE_CATEGORY[stage], `effect stage ${stage}`).toBeDefined();
    }
    expect(Object.keys(EFFECT_STAGE_CATEGORY)).toHaveLength(14);
    expect(readDomainState("effect", "OUTCOME_UNKNOWN").category).toBe("blocked");
    expect(readDomainState("effect", "NOT_EXECUTED").category).toBe("unknown");
  });

  it("keeps the verbatim label and never guesses a color for unmapped words", () => {
    const reading = readDomainState("task", "SOME_FUTURE_STATE");
    expect(reading.category).toBe("unknown");
    expect(reading.label).toBe("SOME_FUTURE_STATE");
    expect(reading.unmapped).toBe(true);
  });

  it("treats missing/empty state as unknown, not as ready", () => {
    expect(readDomainState("provider", undefined).category).toBe("unknown");
    expect(readDomainState("provider", "").category).toBe("unknown");
    expect(readDomainState("provider", null).label).toBe("unknown");
  });

  it("maps readiness/provider/tool/dsh/watch vocabularies", () => {
    expect(readDomainState("readiness", "degraded").category).toBe("attention");
    expect(readDomainState("readiness", "not_configured").category).toBe("unknown");
    expect(readDomainState("provider", "revoked").category).toBe("blocked");
    expect(readDomainState("tool", "quarantined").category).toBe("blocked");
    expect(readDomainState("dsh", "CRASHED").category).toBe("blocked");
    expect(readDomainState("watch", "stale").category).toBe("attention");
  });

  it("binding state requires dispatchability, not just status", () => {
    expect(readBindingState("active", true).category).toBe("ready");
    expect(readBindingState("active", false).category).toBe("blocked");
    expect(readBindingState("active", undefined).category).toBe("attention");
    expect(readBindingState("revoked", undefined).category).toBe("unknown");
  });
});
