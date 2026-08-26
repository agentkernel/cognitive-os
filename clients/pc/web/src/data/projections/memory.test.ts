import { describe, expect, it } from "vitest";
import {
  MEMORY_RETENTION_CAP_DAYS,
  memoryMasterFooter,
  projectMemoryExplain,
  retentionExpiryUnix,
} from "./memory";

describe("memory explain projection", () => {
  it("reads provenance ids from memory.explain and does not invent missing ones", () => {
    const view = projectMemoryExplain({
      kind: "memory.explain",
      memory: {
        memory_id: "mem-1",
        candidate_id: "cand-1",
        decision_id: "dec-1",
        canonical_json: "{\"text\":\"hello\"}",
      },
    });
    expect(view.memoryId).toBe("mem-1");
    expect(view.candidateId).toBe("cand-1");
    expect(view.decisionId).toBe("dec-1");
    expect(view.canonicalJson).toBe("{\"text\":\"hello\"}");
  });

  it("leaves candidate and decision unknown when the explain envelope omits them", () => {
    const view = projectMemoryExplain({ memory: { memory_id: "mem-2" } });
    expect(view.candidateId).toBeUndefined();
    expect(view.decisionId).toBeUndefined();
    expect(view.canonicalJson).toBeUndefined();
  });
});

describe("memory retention and footer", () => {
  it("refuses retention above the 365-day cap instead of sending it", () => {
    expect(retentionExpiryUnix(MEMORY_RETENTION_CAP_DAYS + 1, 1_000)).toEqual({
      ok: false,
      reason: "retention is capped at 365 days",
    });
    expect(retentionExpiryUnix(90, 1_000)).toEqual({ ok: true, unix: 1_000 + 90 * 86_400 });
  });

  it("labels the list as envelope-only and never invents a tombstone count", () => {
    expect(memoryMasterFooter(2, false)).toBe(
      "Showing 2 admitted objects · envelope limit 64 · tombstones are not in this list",
    );
    expect(memoryMasterFooter(1, true)).toContain("envelope at bound");
  });
});
