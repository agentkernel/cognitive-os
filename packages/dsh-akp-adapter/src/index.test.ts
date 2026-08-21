import test from "node:test";
import assert from "node:assert/strict";
import { PassThrough } from "node:stream";
import {
  BRIDGE_PROTOCOL,
  DshAdapterError,
  DshAkpAdapter,
  JsonlAkpTransport,
  PINNED_AKP_SCHEMA_DIGEST,
  PINNED_DSH_REVISION,
  attachDshCordisPlugin,
  decodeResponse,
  encodeRequest,
  type AkpTransport,
  type DshAdapterRequest,
  type DshAdapterResponse,
} from "./index.js";
import { applyDshAkpCordisPlugin, name as cordisPluginName } from "./plugin.js";

const schemaDigest = PINNED_AKP_SCHEMA_DIGEST;

const response = (request: DshAdapterRequest): DshAdapterResponse => ({
  accepted: true,
  sequence: request.sequence,
  candidateOnly: true,
});

const adapterOptions = (transport: AkpTransport) => ({
  dshVersion: PINNED_DSH_REVISION,
  schemaDigest,
  sessionId: "s1",
  pluginId: "p1",
  transport,
});

test("submits ordered candidate-only events and exposes bridge timing", async () => {
  const requests: DshAdapterRequest[] = [];
  const transport: AkpTransport = { send: async (request) => { requests.push(request); return response(request); } };
  const adapter = new DshAkpAdapter(adapterOptions(transport));
  adapter.activate();
  const result = await adapter.submit({ kind: "candidate", operation: "context.propose", payload: { text: "x" } });
  assert.equal(result.response.candidateOnly, true);
  assert.equal(requests[0]?.sequence, 1);
  assert.equal(requests[0]?.schemaDigest, schemaDigest);
  assert.equal(requests[0]?.fencingEpoch, 1);
  assert.equal(adapter.lastSequence, 1);
  assert.ok(result.timing.totalNanos >= result.timing.transportNanos);
});

test("encodes snake_case wire frames that the Rust bridge can parse", () => {
  const frame = JSON.parse(encodeRequest({
    bridgeProtocol: BRIDGE_PROTOCOL,
    dshVersion: PINNED_DSH_REVISION,
    schemaDigest,
    sessionId: "s1",
    fencingEpoch: 1,
    sequence: 3,
    pluginId: "p1",
    correlationId: "s1:3",
    deadline: "2030-01-01T00:00:00.000Z",
    event: { kind: "candidate", operation: "WorkspaceRead", payload: { target: "README.md" } },
  })) as Record<string, unknown>;
  assert.equal(frame["bridge_protocol"], BRIDGE_PROTOCOL);
  assert.equal(frame["dsh_version"], PINNED_DSH_REVISION);
  assert.equal(frame["schema_digest"], schemaDigest);
  assert.equal(frame["session_id"], "s1");
  assert.equal(frame["fencing_epoch"], 1);
  assert.equal(frame["op"], "event");
  assert.equal((frame["event"] as Record<string, unknown>)["authority_claim"], false);
});

test("decodes snake_case candidate-only responses and rejects sequence mismatch", () => {
  const decoded = decodeResponse({ accepted: true, sequence: 2, candidate_only: true });
  assert.equal(decoded.candidateOnly, true);
  assert.equal(decoded.sequence, 2);
  assert.throws(
    () => decodeResponse({ accepted: true, sequence: 2, candidateOnly: false }),
    (error: unknown) => error instanceof DshAdapterError && error.code === "RESPONSE_INVALID",
  );
});

test("rejects authority, secret-shaped, and forbidden fields before transport", async () => {
  let calls = 0;
  const adapter = new DshAkpAdapter(adapterOptions({ send: async (request) => { calls += 1; return response(request); } }));
  adapter.activate();
  await assert.rejects(() => adapter.submit({ kind: "candidate", operation: "x.y", payload: { text: "x" }, authorityClaim: true }), (error: unknown) => error instanceof DshAdapterError && error.code === "AUTHORITY_CLAIM_FORBIDDEN");
  await assert.rejects(() => adapter.submit({ kind: "candidate", operation: "x.y", payload: { api_key: "sk-test" } }), (error: unknown) => error instanceof DshAdapterError && error.code === "SECRET_SHAPED_PAYLOAD");
  await assert.rejects(() => adapter.submit({ kind: "candidate", operation: "x.y", payload: { task_ref: "task://forged" } }), (error: unknown) => error instanceof DshAdapterError && error.code === "FORBIDDEN_PAYLOAD_FIELD");
  assert.equal(calls, 0);
});

test("Cordis-style host events attach without exposing authority", async () => {
  let listener: ((payload: unknown) => void) | undefined;
  const results: number[] = [];
  const adapter = attachDshCordisPlugin({ on: (_event, callback) => { listener = callback; } }, {
    ...adapterOptions({ send: async (request) => response(request) }),
    onResult: (result) => { results.push(result.response.sequence); },
  });
  listener?.({ kind: "candidate", operation: "context.propose", payload: { text: "x" } });
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(adapter.lastSequence, 1);
  assert.deepEqual(results, [1]);
});

test("JSONL transport serializes concurrent events and preserves response pairing", async () => {
  const childInput = new PassThrough();
  const childOutput = new PassThrough();
  const transport = new JsonlAkpTransport(childInput, childOutput, 4096);
  const seen: number[] = [];
  childOutput.on("data", (chunk: Buffer) => {
    const request = JSON.parse(chunk.toString("utf8")) as { sequence: number; bridge_protocol: string };
    seen.push(request.sequence);
    assert.equal(request.bridge_protocol, BRIDGE_PROTOCOL);
    childInput.write(`${JSON.stringify({ accepted: true, sequence: request.sequence, candidate_only: true })}\n`);
  });
  const adapter = new DshAkpAdapter(adapterOptions(transport));
  adapter.activate();
  const results = await Promise.all([
    adapter.submit({ kind: "candidate", operation: "context.one", payload: { n: 1 } }),
    adapter.submit({ kind: "candidate", operation: "context.two", payload: { n: 2 } }),
  ]);
  assert.deepEqual(seen, [1, 2]);
  assert.deepEqual(results.map((item) => item.response.sequence), [1, 2]);
  transport.close();
});

test("rejects oversized frames and mismatched response sequences", async () => {
  const adapter = new DshAkpAdapter({
    ...adapterOptions({
      send: async (request) => ({ accepted: true, sequence: request.sequence + 1, candidateOnly: true }),
    }),
    maxFrameBytes: 64,
  });
  adapter.activate();
  await assert.rejects(
    () => adapter.submit({ kind: "candidate", operation: "context.huge", payload: { text: "x".repeat(200) } }),
    (error: unknown) => error instanceof DshAdapterError && error.code === "FRAME_TOO_LARGE",
  );
  const pairing = new DshAkpAdapter(adapterOptions({
    send: async (request) => ({ accepted: true, sequence: request.sequence + 1, candidateOnly: true }),
  }));
  pairing.activate();
  await assert.rejects(
    () => pairing.submit({ kind: "candidate", operation: "context.one", payload: { n: 1 } }),
    (error: unknown) => error instanceof DshAdapterError && error.code === "RESPONSE_INVALID",
  );
});

test("Cordis apply reads a bearer file and emits startup candidates", async () => {
  assert.equal(cordisPluginName, "cognitiveos-akp");
  const requests: DshAdapterRequest[] = [];
  const results: number[] = [];
  let hostListener: ((payload: unknown) => void) | undefined;
  const adapter = applyDshAkpCordisPlugin(
    { on: (_event, callback) => { hostListener = callback; } },
    {
      endpoint: "http://127.0.0.1:9/task/akp/dsh",
      bearerFile: "/tmp/p8-t09-bearer",
      sessionId: "dsh-cordis-test",
      startupEvents: [{ kind: "lifecycle", operation: "adapter.ready", payload: { ok: true } }],
    },
    {
      transport: { send: async (request) => { requests.push(request); return response(request); } },
      onResult: (result) => { results.push(result.response.sequence); },
    },
  );
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(adapter.lastSequence, 1);
  assert.equal(requests[0]?.event.operation, "adapter.ready");
  assert.deepEqual(results, [1]);
  hostListener?.({ kind: "observation", operation: "adapter.observe", payload: { n: 2 } });
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(adapter.lastSequence, 2);
});

test("Cordis apply fails closed without an endpoint or bearer file", () => {
  assert.throws(
    () => applyDshAkpCordisPlugin({ on: () => undefined }, { endpoint: " ", bearerFile: "/tmp/x" }),
    (error: unknown) => error instanceof DshAdapterError && error.code === "INVALID_EVENT",
  );
  assert.throws(
    () => applyDshAkpCordisPlugin(
      { on: () => undefined },
      { endpoint: "http://127.0.0.1:9/task/akp/dsh", bearerFile: "/tmp/x" },
      { readBearer: () => "" },
    ),
    (error: unknown) => error instanceof DshAdapterError && error.code === "INVALID_EVENT",
  );
});

test("aborts a stalled transport at the configured deadline", async () => {
  const adapter = new DshAkpAdapter({
    ...adapterOptions({
      send: async (_request, signal) => await new Promise((_resolve, reject) => {
        signal?.addEventListener("abort", () => reject(new Error("aborted")), { once: true });
      }),
    }),
    timeoutMs: 5,
  });
  adapter.activate();
  await assert.rejects(
    () => adapter.submit({ kind: "candidate", operation: "context.timeout", payload: { ok: true } }),
    (error: unknown) => error instanceof DshAdapterError && error.code === "TIMEOUT",
  );
});
