import { afterEach, describe, expect, it, vi } from "vitest";
import { clearSession, rememberBearer } from "./session";
import {
  applyWatchFrames,
  createWatchSession,
  displayWatchLabel,
  projectWatchFrame,
  readTaskWatch,
  watchPath,
  WATCH_RING_CAPACITY,
} from "./watchStream";
import { createWatchController } from "./watch";
import { parseSse } from "./watchSse";

const SNAPSHOT = `event: snapshot\ndata: {"kind":"snapshot","latest_sequence":1,"tasks":[]}\n\n`;
const DELTA = `id: 2\nevent: delta\ndata: {"kind":"delta","sequence":2,"event":{"kind":"task.admitted","body":{"task_ref":"task://personal/a","contract_epoch":3}}}\n\n`;

afterEach(() => {
  vi.unstubAllGlobals();
  clearSession();
});

describe("watch path and frames", () => {
  it("encodes resume_from without putting a bearer on the query", () => {
    expect(watchPath()).toBe("/task/watch");
    expect(watchPath(12)).toBe("/task/watch?resume_from=12");
    expect(watchPath(12)).not.toMatch(/Bearer|token/i);
  });

  it("projects a snapshot separately from a delta and never treats either as completion", () => {
    const frames = parseSse(SNAPSHOT + DELTA);
    expect(projectWatchFrame(frames[0]!)).toBe("snapshot");
    const delta = projectWatchFrame(frames[1]!);
    expect(delta).not.toBe("snapshot");
    if (delta === "snapshot") {
      throw new Error("expected delta");
    }
    expect(delta.kind).toBe("task.admitted");
    expect(delta.taskRef).toBe("task://personal/a");
    expect(delta.cursor).toBe("2");
    const watch = createWatchController();
    applyWatchFrames(watch, frames);
    expect(watch.state).toBe("live");
    expect(watch.inferCompletion("process-exit")).toBe("unknown");
    expect(watch.events).toHaveLength(1);
  });

  it("names the process-local ring bound", () => {
    expect(WATCH_RING_CAPACITY).toBe(128);
  });
});

describe("readTaskWatch", () => {
  it("sends the task bearer and Accept event-stream, never a query token", async () => {
    rememberBearer("task", "task-secret-token");
    const fetchMock = vi.fn(async (input: unknown, init?: RequestInit) => {
      const url = new URL(String(input), "http://localhost");
      expect(url.pathname).toBe("/task/watch");
      expect(url.search).toBe("");
      const headers = new Headers(init?.headers);
      expect(headers.get("Authorization")).toBe("Bearer task-secret-token");
      expect(headers.get("accept")).toMatch(/event-stream/i);
      return new Response(SNAPSHOT + DELTA, {
        status: 200,
        headers: { "content-type": "text/event-stream" },
      });
    });
    vi.stubGlobal("fetch", fetchMock);
    const result = await readTaskWatch();
    expect(result.kind).toBe("ok");
    if (result.kind === "ok") {
      expect(result.latest).toBe(2);
      expect(result.frames).toHaveLength(2);
    }
  });

  it("marks 409 TASK_WATCH_RESUME_STALE as a gap, not a terminal task state", async () => {
    rememberBearer("task", "task-secret-token");
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => {
        return new Response(JSON.stringify({ code: "TASK_WATCH_RESUME_STALE" }), {
          status: 409,
          headers: { "content-type": "application/json" },
        });
      }),
    );
    const result = await readTaskWatch(3);
    expect(result).toEqual({ kind: "stale" });
  });
});

describe("createWatchSession", () => {
  it("starts unattached as unknown, not live", () => {
    const session = createWatchSession({
      read: async () => ({ kind: "ok", frames: [], latest: undefined }),
    });
    expect(session.snapshot().phase).toBe("unattached");
    expect(session.snapshot().label).toBe("not attached");
    expect(session.snapshot().state).toBe("unknown");
    expect(displayWatchLabel("unattached", "live")).toBe("not attached");
    expect(session.detach()).toEqual({ cancelledTask: false, stoppedAgent: false });
    expect(session.snapshot().phase).toBe("unattached");
    expect(session.snapshot().label).toBe("not attached");
  });

  it("attach feeds frames, detach is client-only, and a stale resume requires a snapshot", async () => {
    const reads: Array<number | undefined> = [];
    let mode: "ok" | "stale" = "ok";
    const session = createWatchSession({
      read: async (resumeFrom) => {
        reads.push(resumeFrom);
        if (mode === "stale") {
          return { kind: "stale" };
        }
        return {
          kind: "ok",
          frames: parseSse(SNAPSHOT + DELTA),
          latest: 2,
        };
      },
    });

    const attached = await session.attach();
    expect(attached.phase).toBe("attached");
    expect(attached.state).toBe("live");
    expect(attached.label).toBe("live");
    expect(attached.events).toHaveLength(1);
    expect(attached.resumeFrom).toBe(2);
    expect(attached.delivery).toBe("bounded-poll");
    expect(reads).toEqual([undefined]);

    await session.poll();
    expect(reads).toEqual([undefined, 2]);
    expect(session.snapshot().state).toBe("live");

    mode = "stale";
    const stale = await session.poll();
    expect(stale.state).toBe("stale");
    expect(stale.lastError).toMatch(/TASK_WATCH_RESUME_STALE/);
    expect(stale.resumeFrom).toBeUndefined();

    const receipt = session.detach();
    expect(receipt).toEqual({ cancelledTask: false, stoppedAgent: false });
    expect(session.snapshot().phase).toBe("detached");
    expect(session.snapshot().label).toBe("disconnected");
    expect(session.snapshot().state).toBe("disconnected");
  });
});
