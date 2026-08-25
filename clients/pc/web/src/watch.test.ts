import { describe, expect, it } from "vitest";
import { createWatchController } from "./watch";

describe("watch cursor and detach", () => {
  it("marks a cursor gap stale and never forges a completed state", () => {
    const watch = createWatchController("c0");
    watch.accept({ id: "e1", cursor: "c1", state: "running" });
    watch.noteGap();
    expect(watch.state).toBe("stale");
    expect(watch.inferCompletion("process-exit")).toBe("unknown");
    expect(watch.inferCompletion("provider")).toBe("unknown");
  });

  it("dedupes events and treats detach as client-only", () => {
    const watch = createWatchController();
    watch.accept({ id: "e1", cursor: "c1" });
    watch.accept({ id: "e1", cursor: "c1" });
    expect(watch.events).toHaveLength(1);
    expect(watch.detach()).toEqual({ cancelledTask: false, stoppedAgent: false });
    expect(watch.state).toBe("disconnected");
  });
});
