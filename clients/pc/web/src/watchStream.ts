/**
 * Authenticated Task watch stream.
 *
 * Design 36/39 say "wire real EventSource". Native `EventSource` cannot set
 * `Authorization`, and the task bearer must not enter the URL, so this module
 * opens the same `GET /task/watch` SSE through `daemonFetch` (task channel)
 * and feeds the kept watch controller. The daemon's watch is snapshot-first,
 * process-local (128-event ring, empty `tasks:[]`), and `Connection: close`
 * after the snapshot — so an attached watch follows with bounded polling.
 *
 * Completion is never inferred from frames, process exit, or stream close.
 */

import { daemonFetch } from "./channels";
import { redactSecrets } from "./policy";
import {
  createWatchController,
  type WatchController,
  type WatchEvent,
  type WatchState,
} from "./watch";
import { isWatchResumeStale, latestSequence, parseSse, type WatchFrame } from "./watchSse";

/** Named bound: the daemon closes after each snapshot, so attach polls at this interval. */
export const WATCH_POLL_INTERVAL_MS = 15_000;

/** Process-local ring capacity from `task_api.rs`. */
export const WATCH_RING_CAPACITY = 128;

export const WATCH_TRANSPORT_NOTE =
  "Attach opens GET /task/watch as an authenticated event stream. The browser EventSource constructor cannot carry the task bearer, and the bearer must not enter the URL.";

export const WATCH_RING_NOTE = `GET /task/watch is a process-local ${WATCH_RING_CAPACITY}-event ring. Its snapshot list is always empty. A live watch is not a claim that this task is progressing, and a delta is never a state transition.`;

export const WATCH_DETACH_NOTE =
  "Detaching never cancelled a Task or stopped an Agent. Completion stays unknown.";

export type WatchRead =
  | { kind: "ok"; frames: WatchFrame[]; latest: number | undefined }
  | { kind: "stale" }
  | { kind: "error"; status: number; body: unknown };

export type WatchDelivery = "unattached" | "stream" | "bounded-poll";
export type WatchPhase = "unattached" | "attached" | "detached";

export type WatchSessionSnapshot = {
  phase: WatchPhase;
  state: WatchState;
  label: string;
  cursor: string | undefined;
  events: WatchEvent[];
  resumeFrom: number | undefined;
  delivery: WatchDelivery;
  lastError?: string;
};

export function watchPath(resumeFrom?: number): string {
  if (resumeFrom == null) {
    return "/task/watch";
  }
  return `/task/watch?resume_from=${encodeURIComponent(String(resumeFrom))}`;
}

export async function readTaskWatch(
  resumeFrom?: number,
  init: RequestInit = {},
): Promise<WatchRead> {
  const headers = new Headers(init.headers);
  headers.set("accept", "text/event-stream");
  const response = await daemonFetch(watchPath(resumeFrom), "task", {
    ...init,
    method: "GET",
    headers,
  });
  const text = await response.text();
  let parsed: unknown;
  try {
    parsed = text.length > 0 ? JSON.parse(text) : undefined;
  } catch {
    parsed = undefined;
  }
  if (isWatchResumeStale(response.status, parsed ?? text)) {
    return { kind: "stale" };
  }
  if (!response.ok) {
    return {
      kind: "error",
      status: response.status,
      body: redactSecrets(parsed ?? { raw: text }),
    };
  }
  const frames = parseSse(text);
  return { kind: "ok", frames, latest: latestSequence(frames) };
}

export function projectWatchFrame(frame: WatchFrame): WatchEvent | "snapshot" {
  if (frame.event === "snapshot") {
    return "snapshot";
  }
  const data = asRecord(frame.data);
  const inner = asRecord(data.event);
  const body = asRecord(inner.body ?? data.body);
  const kind = String(inner.kind ?? data.kind ?? frame.event ?? "delta");
  const taskRef = typeof body.task_ref === "string" ? body.task_ref : undefined;
  const sequence =
    typeof data.sequence === "number"
      ? String(data.sequence)
      : typeof inner.sequence === "number"
        ? String(inner.sequence)
        : undefined;
  const id = frame.id ?? sequence ?? JSON.stringify(frame.data);
  const cursor = frame.id ?? sequence ?? id;
  return {
    id,
    cursor,
    kind,
    taskRef,
    detail: summarizeWatchBody(kind, body),
  };
}

export function applyWatchFrames(watch: WatchController, frames: WatchFrame[]): number | undefined {
  let sawSnapshot = false;
  for (const frame of frames) {
    const projected = projectWatchFrame(frame);
    if (projected === "snapshot") {
      sawSnapshot = true;
      watch.reconnect();
      continue;
    }
    watch.accept(projected);
  }
  if (sawSnapshot && watch.state === "unknown") {
    watch.reconnect();
  }
  return latestSequence(frames);
}

export function displayWatchLabel(phase: WatchPhase, state: WatchState): string {
  if (phase === "unattached") {
    return "not attached";
  }
  return state;
}

export function createWatchSession(options: {
  read?: typeof readTaskWatch;
  onChange?: (snapshot: WatchSessionSnapshot) => void;
} = {}): {
  snapshot(): WatchSessionSnapshot;
  attach(): Promise<WatchSessionSnapshot>;
  detach(opts?: { silent?: boolean }): { cancelledTask: false; stoppedAgent: false };
  reconnect(): Promise<WatchSessionSnapshot>;
  poll(): Promise<WatchSessionSnapshot>;
} {
  const read = options.read ?? readTaskWatch;
  let watch = createWatchController();
  let phase: WatchPhase = "unattached";
  let delivery: WatchDelivery = "unattached";
  let resumeFrom: number | undefined;
  let lastError: string | undefined;
  let inFlight = false;
  let generation = 0;

  function snapshot(): WatchSessionSnapshot {
    const state: WatchState =
      phase === "unattached" ? "unknown" : phase === "detached" ? "disconnected" : watch.state;
    return {
      phase,
      state,
      label: displayWatchLabel(phase, state),
      cursor: watch.cursor,
      events: watch.events.slice(),
      resumeFrom,
      delivery,
      lastError,
    };
  }

  function notify(): WatchSessionSnapshot {
    const next = snapshot();
    options.onChange?.(next);
    return next;
  }

  async function pull(): Promise<WatchSessionSnapshot> {
    if (phase !== "attached" || inFlight) {
      return snapshot();
    }
    inFlight = true;
    delivery = "stream";
    notify();
    const token = generation;
    try {
      const result = await read(resumeFrom);
      if (token !== generation || phase !== "attached") {
        return snapshot();
      }
      if (result.kind === "stale") {
        watch.noteGap();
        resumeFrom = undefined;
        lastError = "TASK_WATCH_RESUME_STALE — snapshot reload required. Completion stays unknown.";
        delivery = "stream";
        return notify();
      }
      if (result.kind === "error") {
        lastError = `watch HTTP ${result.status}`;
        return notify();
      }
      lastError = undefined;
      const latest = applyWatchFrames(watch, result.frames);
      if (latest != null) {
        resumeFrom = latest;
      }
      delivery = "bounded-poll";
      return notify();
    } catch (error) {
      if (token !== generation || phase !== "attached") {
        return snapshot();
      }
      lastError = error instanceof Error ? error.message : "watch read failed";
      return notify();
    } finally {
      inFlight = false;
    }
  }

  return {
    snapshot,
    async attach() {
      if (phase === "attached") {
        return snapshot();
      }
      generation += 1;
      watch = createWatchController();
      phase = "attached";
      delivery = "stream";
      resumeFrom = undefined;
      lastError = undefined;
      notify();
      return pull();
    },
    detach(opts?: { silent?: boolean }) {
      generation += 1;
      inFlight = false;
      delivery = "unattached";
      if (phase === "attached") {
        const receipt = watch.detach();
        phase = "detached";
        if (!opts?.silent) {
          notify();
        }
        return receipt;
      }
      if (!opts?.silent) {
        notify();
      }
      return { cancelledTask: false, stoppedAgent: false };
    },
    async reconnect() {
      if (phase !== "attached" && phase !== "detached") {
        return snapshot();
      }
      generation += 1;
      watch.reconnect();
      phase = "attached";
      resumeFrom = undefined;
      lastError = undefined;
      notify();
      return pull();
    },
    poll() {
      return pull();
    },
  };
}

function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

function summarizeWatchBody(kind: string, body: Record<string, unknown>): string {
  const parts = [kind];
  if (typeof body.task_ref === "string") {
    parts.push(body.task_ref);
  }
  if (typeof body.user_intent_record_id === "string") {
    parts.push(`intent ${body.user_intent_record_id}`);
  }
  if (typeof body.interpretation_id === "string") {
    parts.push(`interpretation ${body.interpretation_id}`);
  }
  if (typeof body.status === "string") {
    parts.push(body.status);
  }
  if (body.contract_epoch != null) {
    parts.push(`epoch ${String(body.contract_epoch)}`);
  }
  return parts.join(" · ");
}
