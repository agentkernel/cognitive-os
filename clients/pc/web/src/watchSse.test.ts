import { describe, expect, it } from "vitest";
import { isWatchResumeStale, latestSequence, parseSse } from "./watchSse";

describe("watch SSE", () => {
  it("parses snapshot then delta and never treats them as completion", () => {
    const frames = parseSse(
      [
        "event: snapshot\ndata: {\"kind\":\"snapshot\",\"latest_sequence\":1}\n\n",
        "id: 2\nevent: delta\ndata: {\"kind\":\"delta\",\"sequence\":2,\"event\":{\"type\":\"intent.recorded\"}}\n\n",
      ].join(""),
    );
    expect(frames[0]?.event).toBe("snapshot");
    expect(frames[1]?.id).toBe("2");
    expect(latestSequence(frames)).toBe(2);
  });

  it("marks a stale resume as a gap, not a terminal Task state", () => {
    expect(
      isWatchResumeStale(409, { code: "TASK_WATCH_RESUME_STALE" }),
    ).toBe(true);
    expect(isWatchResumeStale(200, { kind: "snapshot" })).toBe(false);
  });
});
