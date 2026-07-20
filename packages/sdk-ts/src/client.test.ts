/**
 * Channel client tests: request pipeline, contract-driven retry
 * (error-contract §3), idempotency-stable resends (REQ-AKP-IDEM-001), and
 * channel binding enforcement (vector `shell-channel-isolation-003.json`).
 */

import assert from "node:assert/strict";
import { test } from "node:test";

import { ChannelBindingViolation, managementCredential, taskCredential } from "./channel.js";
import {
  ManagementChannelClient,
  OutcomeUnknownError,
  TaskChannelClient,
} from "./client.js";
import { buildResultEnvelope, type ErrorEnvelope, type RequestEnvelope } from "./envelope.js";
import { InMemoryTransport } from "./transport.js";

const TASK_CRED = taskCredential({
  credentialId: "cred-task-1",
  principalRef: "principal://tenant-a/user-1",
  secret: "task-secret",
});

const MGMT_CRED = managementCredential({
  credentialId: "cred-mgmt-1",
  principalRef: "principal://tenant-a/admin-1",
  secret: "mgmt-secret",
});

const CONTEXT = {
  sender: "principal://tenant-a/user-1",
  audience: "kernel://tenant-a/node-1",
} as const;

const SCHEMA_DIGEST = `sha256:${"ab".repeat(32)}`;

let idCounter = 0;
const deps = (transport: InMemoryTransport) => ({
  transport,
  newMessageId: () => `msg-${++idCounter}`,
});

function callSpec(overrides: Record<string, unknown> = {}) {
  return {
    operation: "shell.submit",
    kind: "effecting",
    schemaDigest: SCHEMA_DIGEST,
    deadline: "2026-07-20T12:00:00Z",
    idempotencyKey: "idem-0001",
    payload: { attempt: "one" },
    ...overrides,
  } as const;
}

function ok(envelope: RequestEnvelope) {
  return buildResultEnvelope({
    inReplyTo: envelope.message_id,
    correlationId: envelope.correlation_id,
    status: "ok",
    result: { fine: true },
  });
}

function wireError(envelope: RequestEnvelope, error: ErrorEnvelope) {
  return buildResultEnvelope({
    inReplyTo: envelope.message_id,
    correlationId: envelope.correlation_id,
    status: "error",
    error,
  });
}

const ERR = (code: string, retryable: boolean, category = "state"): ErrorEnvelope => ({
  code,
  category: category as ErrorEnvelope["category"],
  retryable,
  stage: "commit",
});

test("a non-retryable denial is surfaced verbatim after exactly one attempt", async () => {
  const transport = new InMemoryTransport("task", (envelope) =>
    wireError(envelope, ERR("CONTEXT_AUTH_DENIED", false, "auth")),
  );
  const client = new TaskChannelClient(TASK_CRED, CONTEXT, deps(transport));
  const result = await client.call(callSpec());
  assert.equal(result.status, "error");
  assert.equal(result.error?.code, "CONTEXT_AUTH_DENIED");
  assert.equal(transport.requests.length, 1);
});

test("retryable failures are retried with a stable idempotency key and fresh message ids", async () => {
  let attempt = 0;
  const transport = new InMemoryTransport("task", (envelope) => {
    attempt += 1;
    return attempt < 3 ? wireError(envelope, ERR("STATE_STORE_UNAVAILABLE", true)) : ok(envelope);
  });
  const client = new TaskChannelClient(TASK_CRED, CONTEXT, deps(transport));
  const result = await client.call(callSpec(), { maxAttempts: 3 });
  assert.equal(result.status, "ok");
  assert.equal(transport.requests.length, 3);
  const keys = new Set(transport.requests.map((request) => request.idempotency_key));
  assert.deepEqual([...keys], ["idem-0001"], "idempotency key must stay stable across retries");
  const ids = new Set(transport.requests.map((request) => request.message_id));
  assert.equal(ids.size, 3, "each delivery attempt gets a fresh message id");
});

test("retry is bounded and exhaustion returns the last error envelope, never a fabricated success", async () => {
  const transport = new InMemoryTransport("task", (envelope) =>
    wireError(envelope, ERR("STATE_STORE_UNAVAILABLE", true)),
  );
  const client = new TaskChannelClient(TASK_CRED, CONTEXT, deps(transport));
  const result = await client.call(callSpec(), { maxAttempts: 3 });
  assert.equal(result.status, "error");
  assert.equal(result.error?.code, "STATE_STORE_UNAVAILABLE");
  assert.equal(transport.requests.length, 3);
});

test("EFFECT_OUTCOME_UNKNOWN is never blindly re-dispatched", async () => {
  const transport = new InMemoryTransport("task", (envelope) =>
    wireError(envelope, ERR("EFFECT_OUTCOME_UNKNOWN", true, "effect")),
  );
  const client = new TaskChannelClient(TASK_CRED, CONTEXT, deps(transport));
  const result = await client.call(callSpec(), { maxAttempts: 5 });
  assert.equal(result.error?.code, "EFFECT_OUTCOME_UNKNOWN");
  assert.equal(transport.requests.length, 1, "reconcile-only: no automatic resend");
});

test("an outcome_unknown result status is returned verbatim without retry", async () => {
  const transport = new InMemoryTransport("task", (envelope) =>
    buildResultEnvelope({
      inReplyTo: envelope.message_id,
      correlationId: envelope.correlation_id,
      status: "outcome_unknown",
    }),
  );
  const client = new TaskChannelClient(TASK_CRED, CONTEXT, deps(transport));
  const result = await client.call(callSpec(), { maxAttempts: 5 });
  assert.equal(result.status, "outcome_unknown");
  assert.equal(transport.requests.length, 1);
});

test("STATE_CONFLICT is not auto-retried without a refresh hook", async () => {
  const transport = new InMemoryTransport("task", (envelope) =>
    wireError(envelope, ERR("STATE_CONFLICT", true)),
  );
  const client = new TaskChannelClient(TASK_CRED, CONTEXT, deps(transport));
  const result = await client.call(callSpec(), { maxAttempts: 3 });
  assert.equal(result.error?.code, "STATE_CONFLICT");
  assert.equal(transport.requests.length, 1, "stale guards must not be resent");
});

test("STATE_CONFLICT retries only through the caller's authoritative re-read", async () => {
  let attempt = 0;
  const transport = new InMemoryTransport("task", (envelope) => {
    attempt += 1;
    return attempt === 1 ? wireError(envelope, ERR("STATE_CONFLICT", true)) : ok(envelope);
  });
  const client = new TaskChannelClient(TASK_CRED, CONTEXT, deps(transport));
  const seen: string[] = [];
  const result = await client.call(callSpec(), {
    maxAttempts: 3,
    refreshOnStateConflict: (error) => {
      seen.push(error.code);
      // Re-read authoritative state: fresh guards, new params, new key.
      return callSpec({ payload: { attempt: "two" }, idempotencyKey: "idem-0002" });
    },
  });
  assert.equal(result.status, "ok");
  assert.deepEqual(seen, ["STATE_CONFLICT"]);
  assert.equal(transport.requests.length, 2);
  assert.deepEqual(transport.requests[1]?.payload, { attempt: "two" });
  assert.equal(transport.requests[1]?.idempotency_key, "idem-0002");
});

test("transport failure on an effecting call resends the same key, then signals reconciliation", async () => {
  const transport = new InMemoryTransport("task", () => {
    throw new Error("connection reset");
  });
  const client = new TaskChannelClient(TASK_CRED, CONTEXT, deps(transport));
  await assert.rejects(
    client.call(callSpec(), { maxAttempts: 2 }),
    (error: unknown) =>
      error instanceof OutcomeUnknownError &&
      error.code === "EFFECT_OUTCOME_UNKNOWN" &&
      error.idempotencyKey === "idem-0001",
  );
  assert.equal(transport.requests.length, 2);
  assert.deepEqual(
    new Set(transport.requests.map((request) => request.idempotency_key)),
    new Set(["idem-0001"]),
  );
});

test("transport failure on a read call rethrows after bounded retries", async () => {
  const transport = new InMemoryTransport("task", () => {
    throw new Error("connection reset");
  });
  const client = new TaskChannelClient(TASK_CRED, CONTEXT, deps(transport));
  await assert.rejects(
    client.call(callSpec({ operation: "shell.attach", kind: "read", idempotencyKey: undefined }), {
      maxAttempts: 2,
    }),
    /connection reset/,
  );
  assert.equal(transport.requests.length, 2);
});

test("the task client refuses non-task operations before anything is sent", async () => {
  const transport = new InMemoryTransport("task", (envelope) => ok(envelope));
  const client = new TaskChannelClient(TASK_CRED, CONTEXT, deps(transport));
  await assert.rejects(
    client.call(callSpec({ operation: "system.configure" })),
    (error: unknown) =>
      error instanceof ChannelBindingViolation && error.code === "SHELL_CHANNEL_BINDING_MISMATCH",
  );
  assert.equal(transport.requests.length, 0, "the mixed-channel request must never leave the client");
});

test("the management client refuses task-channel operations before anything is sent", async () => {
  const transport = new InMemoryTransport("management", (envelope) => ok(envelope));
  const client = new ManagementChannelClient(MGMT_CRED, CONTEXT, deps(transport));
  await assert.rejects(
    client.call(callSpec({ operation: "shell.submit" })),
    (error: unknown) => error instanceof ChannelBindingViolation,
  );
  assert.equal(transport.requests.length, 0);
});

test("an authority channel-binding denial surfaces verbatim and is not retried", async () => {
  const transport = new InMemoryTransport("task", (envelope) =>
    wireError(envelope, ERR("SHELL_CHANNEL_BINDING_MISMATCH", false, "auth")),
  );
  const client = new TaskChannelClient(TASK_CRED, CONTEXT, deps(transport));
  const result = await client.call(callSpec(), { maxAttempts: 3 });
  assert.equal(result.error?.code, "SHELL_CHANNEL_BINDING_MISMATCH");
  assert.equal(transport.requests.length, 1);
});

test("client construction fails closed on credential/transport channel mismatch", () => {
  const mgmtTransport = new InMemoryTransport("management", (envelope) => ok(envelope));
  assert.throws(
    // @ts-expect-error task client cannot take a management credential
    () => new TaskChannelClient(MGMT_CRED, CONTEXT, deps(mgmtTransport)),
    (error: unknown) => error instanceof ChannelBindingViolation,
  );
  const taskTransport = new InMemoryTransport("task", (envelope) => ok(envelope));
  assert.throws(
    () => new ManagementChannelClient(MGMT_CRED, CONTEXT, deps(taskTransport)),
    (error: unknown) => error instanceof ChannelBindingViolation,
  );
});

test("a 2xx transport reply is not success: the enveloped error decides", async () => {
  const transport = new InMemoryTransport("task", (envelope) =>
    wireError(envelope, ERR("RESOURCE_BUDGET_EXHAUSTED", false, "resource")),
  );
  const client = new TaskChannelClient(TASK_CRED, CONTEXT, deps(transport));
  // InMemoryTransport always answers transportStatus 200.
  const result = await client.call(callSpec());
  assert.equal(result.status, "error");
  assert.equal(result.error?.code, "RESOURCE_BUDGET_EXHAUSTED");
});

test("each client instance keeps its own channel-partitioned projection store", () => {
  const taskClient = new TaskChannelClient(
    TASK_CRED,
    CONTEXT,
    deps(new InMemoryTransport("task", (envelope) => ok(envelope))),
  );
  const mgmtClient = new ManagementChannelClient(
    MGMT_CRED,
    CONTEXT,
    deps(new InMemoryTransport("management", (envelope) => ok(envelope))),
  );
  taskClient.projections.ingest("task://tenant-a/task-1", 1, { status: "runnable" });
  assert.equal(mgmtClient.projections.get("task://tenant-a/task-1"), undefined);
});
