/**
 * AKP envelope build/parse tests over the generated wire bindings
 * (akp-request-envelope / akp-result-envelope, D-013; specs/akp/README.md
 * §3/§5/§6; docs/standards/akp-envelope-and-http-profile.md §2;
 * REQ-AKP-ENV-001, REQ-AKP-ENV-002, REQ-AKP-VER-001, REQ-AKP-CAN-001,
 * REQ-AKP-IDEM-001).
 *
 * Gate order under test (fail closed before payload processing): strict
 * parse → shape → version → critical extensions → payload digest. Schema
 * conditionals enforced eagerly client-side: payload XOR payload_ref,
 * error status ⇒ machine error, partial status ⇒ continuation.
 */

import assert from "node:assert/strict";
import { test } from "node:test";

import { canonicalize, digest } from "@cognitiveos/contracts-ts";

import {
  AKP_PAYLOAD_DIGEST_DOMAIN,
  AKP_PROTOCOL_VERSION,
  buildRequestEnvelope,
  buildResultEnvelope,
  EnvelopeViolation,
  parseRequestEnvelope,
  parseResultEnvelope,
  serializeEnvelope,
} from "./envelope.js";

const BASE_REQUEST = {
  operation: "shell.submit",
  kind: "effecting",
  sender: "principal://tenant-a/user-1",
  audience: "kernel://tenant-a/node-1",
  correlationId: "corr-0001",
  deadline: "2026-07-20T12:00:00Z",
  schemaDigest: `sha256:${"ab".repeat(32)}`,
  idempotencyKey: "idem-key-0001",
  payload: { action: "approve", proposal: "mp-1" },
  messageId: "msg-0001",
} as const;

test("request round-trip: build, serialize, parse back with verified payload digest", () => {
  const envelope = buildRequestEnvelope(BASE_REQUEST);
  assert.equal(envelope.protocol_version, AKP_PROTOCOL_VERSION);
  assert.equal(envelope.message_id, "msg-0001");
  assert.equal(envelope.idempotency_key, "idem-key-0001");
  // Payload digest must be the contracts-ts domain-separated digest, pinned
  // by the akp-payload/0.2 golden fixture.
  const expected = digest(
    canonicalize(JSON.stringify(BASE_REQUEST.payload)),
    AKP_PAYLOAD_DIGEST_DOMAIN,
  );
  assert.equal(envelope.payload_digest, expected);

  const text = serializeEnvelope(envelope);
  const parsed = parseRequestEnvelope(text);
  assert.equal(parsed.operation, "shell.submit");
  assert.deepEqual(parsed.payload, BASE_REQUEST.payload);
  assert.equal(parsed.payload_digest, expected);
});

test("an effecting request without an idempotency key fails at build time", () => {
  assert.throws(
    () => buildRequestEnvelope({ ...BASE_REQUEST, idempotencyKey: undefined }),
    (error: unknown) =>
      error instanceof EnvelopeViolation && error.reason === "missing-idempotency-key",
  );
});

test("a read request needs no idempotency key", () => {
  const envelope = buildRequestEnvelope({
    ...BASE_REQUEST,
    operation: "shell.attach",
    kind: "read",
    idempotencyKey: undefined,
  });
  assert.equal(envelope.idempotency_key, undefined);
});

test("payload and payload_ref together are rejected (schema oneOf, both sides)", () => {
  assert.throws(
    () =>
      buildRequestEnvelope({ ...BASE_REQUEST, payloadRef: "artifact://tenant-a/blob-1" }),
    (error: unknown) => error instanceof EnvelopeViolation && error.reason === "ambiguous-payload",
  );
  const envelope = buildRequestEnvelope(BASE_REQUEST);
  const tampered = JSON.parse(serializeEnvelope(envelope)) as Record<string, unknown>;
  tampered["payload_ref"] = "artifact://tenant-a/blob-1";
  assert.throws(
    () => parseRequestEnvelope(JSON.stringify(tampered)),
    (error: unknown) => error instanceof EnvelopeViolation && error.reason === "ambiguous-payload",
  );
});

test("a partial result without a continuation is rejected (schema conditional)", () => {
  assert.throws(
    () =>
      buildResultEnvelope({
        inReplyTo: "msg-0001",
        correlationId: "corr-0001",
        status: "partial",
        result: { part: 1 },
      }),
    (error: unknown) =>
      error instanceof EnvelopeViolation && error.reason === "missing-continuation",
  );
  assert.throws(
    () => parseResultEnvelope(okResultText({ status: "partial" })),
    (error: unknown) =>
      error instanceof EnvelopeViolation && error.reason === "missing-continuation",
  );
  const parsed = parseResultEnvelope(
    okResultText({ status: "partial", continuation: { token: "c1", high_watermark: 3 } }),
  );
  assert.equal(parsed.status, "partial");
});

test("non-canonical deadline (offset form) is rejected at build time", () => {
  assert.throws(
    () => buildRequestEnvelope({ ...BASE_REQUEST, deadline: "2026-07-20T12:00:00+08:00" }),
    (error: unknown) => error instanceof EnvelopeViolation && error.reason === "invalid-timestamp",
  );
});

test("malformed schema digest is rejected at build time", () => {
  assert.throws(
    () => buildRequestEnvelope({ ...BASE_REQUEST, schemaDigest: "sha256:XYZ" }),
    (error: unknown) => error instanceof EnvelopeViolation && error.reason === "invalid-digest",
  );
});

test("parse rejects duplicate members and BOM via the strict contracts parser", () => {
  assert.throws(
    () => parseResultEnvelope('{"status":"ok","status":"error"}'),
    (error: unknown) => error instanceof EnvelopeViolation && error.reason === "malformed-json",
  );
  assert.throws(
    () => parseResultEnvelope('\uFEFF{"status":"ok"}'),
    (error: unknown) => error instanceof EnvelopeViolation && error.reason === "malformed-json",
  );
});

function okResultText(overrides: Record<string, unknown> = {}): string {
  const result = buildResultEnvelope({
    inReplyTo: "msg-0001",
    correlationId: "corr-0001",
    status: "ok",
    result: { view: "v" },
  });
  return JSON.stringify({ ...(JSON.parse(serializeEnvelope(result)) as object), ...overrides });
}

test("result round-trip parses status and result verbatim", () => {
  const parsed = parseResultEnvelope(okResultText());
  assert.equal(parsed.status, "ok");
  assert.deepEqual(parsed.result, { view: "v" });
  assert.equal(parsed.in_reply_to, "msg-0001");
});

test("unsupported protocol major/minor fails with VERSION_UNSUPPORTED semantics", () => {
  assert.throws(
    () => parseResultEnvelope(okResultText({ protocol_version: "cognitiveos.akp/1.0" })),
    (error: unknown) => error instanceof EnvelopeViolation && error.code === "VERSION_UNSUPPORTED",
  );
});

test("unknown critical extension fails closed before payload digest verification", () => {
  // Both violations present: the critical-extension gate must win because it
  // runs before any payload processing (REQ-AKP-ENV-002).
  const text = okResultText({
    extensions: [{ id: "x-unknown", critical: true }],
    result_digest: `sha256:${"00".repeat(32)}`,
  });
  assert.throws(
    () => parseResultEnvelope(text),
    (error: unknown) =>
      error instanceof EnvelopeViolation && error.code === "CRITICAL_EXTENSION_UNKNOWN",
  );
});

test("non-critical unknown extension is tolerated", () => {
  const parsed = parseResultEnvelope(okResultText({ extensions: [{ id: "x-soft", critical: false }] }));
  assert.equal(parsed.status, "ok");
});

test("declared result digest is recomputed and a mismatch fails closed", () => {
  const good = digest(canonicalize(JSON.stringify({ view: "v" })), AKP_PAYLOAD_DIGEST_DOMAIN);
  const parsed = parseResultEnvelope(okResultText({ result_digest: good }));
  assert.equal(parsed.status, "ok");
  assert.throws(
    () => parseResultEnvelope(okResultText({ result_digest: `sha256:${"00".repeat(32)}` })),
    (error: unknown) => error instanceof EnvelopeViolation && error.code === "DIGEST_MISMATCH",
  );
});

test("request parse verifies the inline payload digest and rejects tampering", () => {
  const envelope = buildRequestEnvelope(BASE_REQUEST);
  const tampered = JSON.parse(serializeEnvelope(envelope)) as Record<string, unknown>;
  tampered["payload"] = { action: "approve", proposal: "mp-2" };
  assert.throws(
    () => parseRequestEnvelope(JSON.stringify(tampered)),
    (error: unknown) => error instanceof EnvelopeViolation && error.code === "DIGEST_MISMATCH",
  );
});

test("an error result must carry the registered error envelope shape", () => {
  const withError = buildResultEnvelope({
    inReplyTo: "msg-0001",
    correlationId: "corr-0001",
    status: "error",
    error: {
      code: "STATE_CONFLICT",
      category: "state",
      retryable: true,
      stage: "commit",
    },
  });
  const parsed = parseResultEnvelope(serializeEnvelope(withError));
  assert.equal(parsed.status, "error");
  assert.equal(parsed.error?.code, "STATE_CONFLICT");

  assert.throws(
    () => parseResultEnvelope(okResultText({ status: "error" })),
    (error: unknown) => error instanceof EnvelopeViolation && error.reason === "missing-error",
  );
});

test("unknown result status fails closed", () => {
  assert.throws(
    () => parseResultEnvelope(okResultText({ status: "finished" })),
    (error: unknown) => error instanceof EnvelopeViolation && error.reason === "unknown-status",
  );
});

test("accepted / receipt-like statuses parse verbatim and are never rewritten", () => {
  // REQ-AKP-RES-001: the SDK surfaces `accepted` as-is; nothing maps it to
  // ok/verified/committed. The absence of any such helper is part of the
  // contract; here we pin that parsing does not rewrite the status.
  for (const status of ["accepted", "cancel_pending", "outcome_unknown"] as const) {
    const parsed = parseResultEnvelope(okResultText({ status }));
    assert.equal(parsed.status, status);
  }
});
