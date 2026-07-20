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
import { HttpSseTransport, InMemoryTransport } from "./transport.js";

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

test("http transport posts to channel-disjoint endpoint roots with bearer material", async () => {
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
  const reply = await transport.request('{"x":1}');
  // Transport status is surfaced verbatim; outcome interpretation is the
  // envelope layer's job, and a non-2xx body is still returned for parsing.
  assert.equal(reply.transportStatus, 503);
  assert.equal(reply.body, '{"status":"error"}');
  assert.equal(seen.length, 1);
  assert.ok(seen[0]?.url.startsWith("https://kernel.local/akp/management/"));
  const headers = seen[0]?.init.headers as Record<string, string>;
  assert.equal(headers["authorization"], "Bearer mgmt-secret");
});

test("http transport parses SSE data lines into frame texts", async () => {
  const sse = 'data: {"sequence":1}\n\ndata: {"sequence":2}\n\n: comment\n\ndata: {"sequence":3}\n\n';
  const fetchStub: typeof fetch = () =>
    Promise.resolve(
      new Response(sse, { status: 200, headers: { "content-type": "text/event-stream" } }),
    );
  const transport = new HttpSseTransport({
    baseUrl: "https://kernel.local",
    channel: "task",
    bearer: "task-secret",
    fetchImpl: fetchStub,
  });
  const frames: string[] = [];
  for await (const frame of transport.openStream('{"x":1}')) {
    frames.push(frame);
  }
  assert.deepEqual(frames, ['{"sequence":1}', '{"sequence":2}', '{"sequence":3}']);
});
