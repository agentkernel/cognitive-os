/**
 * Watch consumer tests (docs/standards/event-audit-watch.md §5;
 * REQ-SHELL-WATCH-001, REQ-AKP-SHELL-002, REQ-AKP-STR-001; vector
 * `shell-watch-resume-006.json`).
 *
 * Semantics under test: one snapshot then ordered deltas; at-least-once
 * delivery deduplicated by sequence; disconnect resumes from the last
 * acknowledged cursor; `WATCH_CURSOR_STALE` forces a fresh authorized
 * snapshot; gaps are never silently skipped; recovery is bounded.
 */

import assert from "node:assert/strict";
import { test } from "node:test";

import { taskCredential } from "./channel.js";
import { TaskChannelClient } from "./client.js";
import { buildResultEnvelope, type RequestEnvelope } from "./envelope.js";
import { sampleWatchSubscription } from "./fixtures.js";
import { InMemoryTransport } from "./transport.js";
import { consumeWatch, frameText, WatchViolation, type WatchItem } from "./watch.js";

const CRED = taskCredential({
  credentialId: "cred-task-1",
  principalRef: "principal://tenant-a/user-1",
  secret: "task-secret",
});

const CONTEXT = {
  sender: "principal://tenant-a/user-1",
  audience: "kernel://tenant-a/node-1",
} as const;

function harness(streams: Array<(envelope: RequestEnvelope) => string[]>) {
  const queue = [...streams];
  const transport = new InMemoryTransport(
    "task",
    (envelope) =>
      buildResultEnvelope({
        inReplyTo: envelope.message_id,
        correlationId: envelope.correlation_id,
        status: "ok",
      }),
    (envelope) => {
      const handler = queue.shift();
      if (!handler) {
        throw new Error("no scripted stream left");
      }
      return handler(envelope);
    },
  );
  const client = new TaskChannelClient(CRED, CONTEXT, { transport });
  return { transport, client };
}

const snap = (sequence: number, snapshotVersion: number, payload: unknown = { s: sequence }) =>
  frameText({ stream_id: "st-1", sequence, kind: "snapshot", snapshot_version: snapshotVersion, payload });
const delta = (sequence: number, payload: unknown = { d: sequence }, final = false) =>
  frameText({ stream_id: "st-1", sequence, kind: "delta", payload, final });
const errorFrame = (code: string, retryable: boolean) =>
  frameText({
    stream_id: "st-1",
    sequence: 0,
    kind: "error",
    payload: { code, category: "watch", retryable, stage: "resume" },
  });

async function drain(items: AsyncIterable<WatchItem>): Promise<WatchItem[]> {
  const out: WatchItem[] = [];
  for await (const item of items) {
    out.push(item);
  }
  return out;
}

const PARAMS = {
  subscription: sampleWatchSubscription(),
  deadline: "2026-07-20T12:00:00Z",
} as const;

test("watch delivers one snapshot then ordered deltas and completes on final", async () => {
  const { client, transport } = harness([
    () => [snap(10, 8), delta(11), delta(12, { d: 12 }, true)],
  ]);
  const items = await drain(consumeWatch(client, PARAMS));
  assert.deepEqual(
    items.map((item) => [item.kind, item.cursor]),
    [
      ["snapshot", 10],
      ["delta", 11],
      ["delta", 12],
    ],
  );
  assert.equal(items[0]?.snapshotVersion, 8);
  assert.equal(transport.streamOpens[0]?.operation, "watch.open");
});

test("at-least-once duplicates are dropped by sequence", async () => {
  const { client } = harness([
    () => [snap(10, 8), delta(11), delta(11), delta(12, { d: 12 }, true)],
  ]);
  const items = await drain(consumeWatch(client, PARAMS));
  assert.deepEqual(
    items.map((item) => item.cursor),
    [10, 11, 12],
  );
});

test("disconnect resumes from the last acknowledged cursor with replay dedupe", async () => {
  const { client, transport } = harness([
    () => [snap(10, 8), delta(11)], // ends without final: disconnect
    () => [delta(11), delta(12), delta(13, { d: 13 }, true)], // replay from ack
  ]);
  const items = await drain(consumeWatch(client, PARAMS));
  assert.deepEqual(
    items.map((item) => item.cursor),
    [10, 11, 12, 13],
    "replayed delta 11 must not be delivered twice",
  );
  assert.equal(transport.streamOpens[1]?.operation, "watch.resume");
  const resumePayload = transport.streamOpens[1]?.payload as Record<string, unknown>;
  assert.equal(resumePayload["cursor"], 11, "resume carries the last acknowledged cursor");
  assert.equal(resumePayload["snapshot_version"], 8);
  assert.equal(resumePayload["high_watermark"], 11);
  assert.equal(typeof resumePayload["dedupe_window"], "number");
});

test("WATCH_CURSOR_STALE forces a fresh authorized snapshot, never a silent gap", async () => {
  const { client, transport } = harness([
    () => [snap(10, 8), delta(11)], // disconnect
    () => [errorFrame("WATCH_CURSOR_STALE", true)], // compacted: resume denied
    () => [snap(50, 9), delta(51, { d: 51 }, true)], // fresh snapshot
  ]);
  const items = await drain(consumeWatch(client, PARAMS));
  assert.deepEqual(
    items.map((item) => [item.kind, item.cursor]),
    [
      ["snapshot", 10],
      ["delta", 11],
      ["snapshot", 50], // consumer re-bases on the new snapshot
      ["delta", 51],
    ],
  );
  assert.equal(items[2]?.snapshotVersion, 9);
  assert.deepEqual(
    transport.streamOpens.map((envelope) => envelope.operation),
    ["watch.open", "watch.resume", "watch.open"],
    "stale cursor requires a new authorized snapshot (required_action of SHELL-WATCH-RESUME-006)",
  );
});

test("a sequence gap is never skipped: the consumer resumes and backfills", async () => {
  const { client, transport } = harness([
    () => [snap(10, 8), delta(11), delta(13)], // gap: 12 missing
    () => [delta(12), delta(13, { d: 13 }, true)], // backfill from ack 11
  ]);
  const items = await drain(consumeWatch(client, PARAMS));
  assert.deepEqual(
    items.map((item) => item.cursor),
    [10, 11, 12, 13],
    "13 must not be delivered before 12",
  );
  const resumePayload = transport.streamOpens[1]?.payload as Record<string, unknown>;
  assert.equal(resumePayload["cursor"], 11);
});

test("persistent gaps exhaust bounded recovery and fail closed", async () => {
  const gapStream = () => [delta(13)];
  const { client } = harness([
    () => [snap(10, 8), delta(11), delta(13)],
    gapStream,
    gapStream,
    gapStream,
  ]);
  await assert.rejects(
    drain(consumeWatch(client, { ...PARAMS, maxRecoveryAttempts: 3 })),
    (error: unknown) => error instanceof WatchViolation && error.reason === "recovery-exhausted",
  );
});

test("a delta before any snapshot on a fresh open fails closed", async () => {
  const { client } = harness([() => [delta(11)]]);
  await assert.rejects(
    drain(consumeWatch(client, PARAMS)),
    (error: unknown) => error instanceof WatchViolation && error.reason === "missing-snapshot",
  );
});

test("a non-stale stream error is surfaced with its registered code", async () => {
  const { client } = harness([
    () => [snap(10, 8), errorFrame("CONTEXT_AUTH_DENIED", false)],
  ]);
  await assert.rejects(
    drain(consumeWatch(client, PARAMS)),
    (error: unknown) =>
      error instanceof WatchViolation &&
      error.reason === "stream-error" &&
      error.code === "CONTEXT_AUTH_DENIED",
  );
});

test("reattach with a retained cursor starts in resume mode (detach kept the position)", async () => {
  const { client, transport } = harness([
    () => [delta(12), delta(13, { d: 13 }, true)],
  ]);
  const items = await drain(
    consumeWatch(client, { ...PARAMS, resumeFrom: { snapshotVersion: 8, lastAckCursor: 11 } }),
  );
  assert.deepEqual(
    items.map((item) => item.cursor),
    [12, 13],
  );
  assert.equal(transport.streamOpens[0]?.operation, "watch.resume");
  const resumePayload = transport.streamOpens[0]?.payload as Record<string, unknown>;
  assert.equal(resumePayload["cursor"], 11);
});

test("malformed frames fail closed", async () => {
  const { client } = harness([() => [snap(10, 8), '{"sequence":11,"sequence":12}']]);
  await assert.rejects(
    drain(consumeWatch(client, PARAMS)),
    (error: unknown) => error instanceof WatchViolation && error.reason === "malformed-frame",
  );
});
