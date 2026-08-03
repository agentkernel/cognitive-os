/**
 * Transport layer tests: the injectable boundary, the in-memory fake, and
 * the default HTTP/SSE binding (ADR-0003 mapping;
 * docs/standards/akp-envelope-and-http-profile.md §3/§4).
 *
 * Transport status is never an outcome: a 2xx reply proves nothing about
 * effect success (REQ-GW-002 analog at the protocol layer), and the two
 * channels use disjoint endpoint roots.
 */

import assert from "node:assert/strict";
import { test } from "node:test";

import { buildRequestEnvelope, buildResultEnvelope, serializeEnvelope } from "./envelope.js";
import { HttpSseTransport, InMemoryTransport, kernelServerPath } from "./transport.js";

const REQUEST = buildRequestEnvelope({
  operation: "shell.attach",
  kind: "read",
  sender: "principal://tenant-a/user-1",
  audience: "kernel://tenant-a/node-1",
  correlationId: "corr-1",
  deadline: "2026-07-20T12:00:00Z",
  schemaDigest: `sha256:${"ab".repeat(32)}`,
  payload: { task_ref: "task://tenant-a/task-1" },
  messageId: "msg-1",
});

const MGMT_REQUEST = buildRequestEnvelope({
  operation: "management.inspect",
  kind: "read",
  sender: "principal://tenant-a/alice",
  audience: "service://kernel/management",
  correlationId: "corr-m",
  deadline: "2026-07-21T01:00:00Z",
  schemaDigest: `sha256:${"cd".repeat(32)}`,
  payload: { target: "agent-execution://1" },
  messageId: "msg-m",
});

function buildWatchRequest(operation: "watch.open" | "watch.resume", payload: object) {
  return buildRequestEnvelope({
    operation,
    kind: "read",
    sender: "principal://tenant-a/user-1",
    audience: "kernel://tenant-a/node-1",
    correlationId: "corr-watch",
    deadline: "2026-07-20T12:00:00Z",
    schemaDigest: `sha256:${"ab".repeat(32)}`,
    payload,
    messageId: `msg-${operation}`,
  });
}

test("in-memory transport records request envelopes and replies from the script", async () => {
  const transport = new InMemoryTransport("task", (envelope) =>
    buildResultEnvelope({
      inReplyTo: envelope.message_id,
      correlationId: envelope.correlation_id,
      status: "ok",
      result: { attached: true },
    }),
  );
  const reply = await transport.request(serializeEnvelope(REQUEST));
  assert.equal(transport.requests.length, 1);
  assert.equal(transport.requests[0]?.operation, "shell.attach");
  assert.match(reply.body, /"in_reply_to":"msg-1"/);
});

test("in-memory transport streams scripted frames in order", async () => {
  const transport = new InMemoryTransport("task", () => {
    throw new Error("no request handler in this test");
  });
  transport.scriptStream(() => ['{"sequence":1}', '{"sequence":2}']);
  const frames: string[] = [];
  for await (const frame of transport.openStream(serializeEnvelope(REQUEST))) {
    frames.push(frame);
  }
  assert.deepEqual(frames, ['{"sequence":1}', '{"sequence":2}']);
});

test("kernel-server path map keeps management and task roots disjoint", () => {
  assert.equal(kernelServerPath("management", "management.inspect"), "/management/inspect");
  assert.equal(kernelServerPath("task", "shell.detach"), "/shell/detach");
  assert.equal(kernelServerPath("task", "shell.control"), "/shell/cancel");
  assert.equal(kernelServerPath("task", "intent.record"), "/task/intent/record");
  assert.equal(kernelServerPath("task", "intent.interpret"), "/task/intent/interpret");
  assert.equal(kernelServerPath("task", "task.preview"), "/task/preview");
  assert.equal(kernelServerPath("task", "task.admit"), "/task/admit");
  assert.throws(() => kernelServerPath("management", "shell.detach"), /refusing/);
  assert.throws(() => kernelServerPath("task", "management.inspect"), /unsupported/);
});

test("http transport posts to channel-disjoint kernel-server roots with bearer material", async () => {
  const seen: Array<{ url: string; init: RequestInit }> = [];
  const fetchStub: typeof fetch = (input, init) => {
    seen.push({ url: String(input), init: init ?? {} });
    return Promise.resolve(new Response('{"status":"error"}', { status: 503 }));
  };
  const transport = new HttpSseTransport({
    baseUrl: "https://kernel.local",
    channel: "management",
    bearer: "mgmt-secret",
    fetchImpl: fetchStub,
  });
  const reply = await transport.request(serializeEnvelope(MGMT_REQUEST));
  // Transport status is surfaced verbatim; outcome interpretation is the
  // envelope layer's job, and a non-2xx body is still returned for parsing.
  assert.equal(reply.transportStatus, 503);
  assert.equal(reply.body, '{"status":"error"}');
  assert.equal(seen.length, 1);
  assert.equal(seen[0]?.url, "https://kernel.local/management/inspect");
  const headers = seen[0]?.init.headers as Record<string, string>;
  assert.equal(headers["authorization"], "Bearer mgmt-secret");
});

test("http transport parses SSE data lines into frame texts from GET /task/watch", async () => {
  const sse = 'data: {"sequence":1}\n\ndata: {"sequence":2}\n\n: comment\n\ndata: {"sequence":3}\n\n';
  const seen: string[] = [];
  const fetchStub: typeof fetch = (input, init) => {
    seen.push(`${init?.method ?? "GET"} ${String(input)}`);
    return Promise.resolve(
      new Response(sse, { status: 200, headers: { "content-type": "text/event-stream" } }),
    );
  };
  const transport = new HttpSseTransport({
    baseUrl: "https://kernel.local",
    channel: "task",
    bearer: "task-secret",
    fetchImpl: fetchStub,
  });
  const watchRequest = buildWatchRequest("watch.open", { cursor: null });
  const frames: string[] = [];
  for await (const frame of transport.openStream(serializeEnvelope(watchRequest))) {
    frames.push(frame);
  }
  assert.deepEqual(frames, ['{"sequence":1}', '{"sequence":2}', '{"sequence":3}']);
  assert.equal(
    seen[0],
    `GET https://kernel.local/task/watch?request=${encodeURIComponent(serializeEnvelope(watchRequest))}`,
  );
});

test("task watch forwards its open envelope so the daemon can resume the authoritative cursor", async () => {
  const seen: string[] = [];
  const fetchStub: typeof fetch = (input, init) => {
    seen.push(`${init?.method ?? "GET"} ${String(input)}`);
    return Promise.resolve(new Response("", { status: 200 }));
  };
  const transport = new HttpSseTransport({
    baseUrl: "https://kernel.local",
    channel: "task",
    bearer: "task-secret",
    fetchImpl: fetchStub,
  });
  const watchRequest = buildWatchRequest("watch.resume", {
    cursor: 7,
    snapshot_version: 3,
    high_watermark: 7,
    dedupe_window: 32,
  });

  for await (const _ of transport.openStream(serializeEnvelope(watchRequest))) {
    // This fixture emits no frames.
  }

  assert.equal(
    seen[0],
    `GET https://kernel.local/task/watch?request=${encodeURIComponent(serializeEnvelope(watchRequest))}`,
  );
});

test("task watch rejects non-watch operations before opening a stream", async () => {
  const transport = new HttpSseTransport({
    baseUrl: "https://kernel.local",
    channel: "task",
    bearer: "task-secret",
    fetchImpl: () => Promise.reject(new Error("network should not be touched")),
  });

  await assert.rejects(
    async () => {
      for await (const _ of transport.openStream(serializeEnvelope(REQUEST))) {
        // A shell request must never be silently transformed into a watch.
      }
    },
    /watch operation/,
  );
});

test("management channel refuses to open a watch stream", async () => {
  const transport = new HttpSseTransport({
    baseUrl: "https://kernel.local",
    channel: "management",
    bearer: "mgmt-secret",
    fetchImpl: () => Promise.reject(new Error("network should not be touched")),
  });
  await assert.rejects(
    async () => {
      for await (const _ of transport.openStream(serializeEnvelope(MGMT_REQUEST))) {
        // never
      }
    },
    /task channel only/,
  );
});
