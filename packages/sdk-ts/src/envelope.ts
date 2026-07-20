/**
 * AKP envelope construction and parsing (specs/akp/README.md §3/§5/§9;
 * docs/standards/akp-envelope-and-http-profile.md §2/§3; REQ-AKP-ENV-001,
 * REQ-AKP-ENV-002, REQ-AKP-VER-001, REQ-AKP-CAN-001, REQ-AKP-IDEM-001,
 * REQ-AKP-RES-001).
 *
 * Canonicalization and digests go exclusively through
 * `@cognitiveos/contracts-ts` (REQ-AKP-CAN-001). The receive path fails
 * closed in contract order: strict parse → shape → version → unknown
 * critical extensions → payload digest, so no payload byte is interpreted
 * behind a failed gate.
 *
 * NOTE (registered contract gap, 20260720 lane-tsc handoff): there is no
 * machine schema for the AKP envelope yet, so the member names below encode
 * specs/akp/README.md §3 prose. They are confined to this module; when
 * Lane-CTR lands an envelope schema + codegen, this module consumes the
 * generated binding instead. `result_digest` mirrors the standard's inline
 * `payload_digest` semantics for result values and is provisional.
 *
 * This SDK is a client: it never interprets `accepted`, receipts, or remote
 * `completed` as success, and it exposes no helper that collapses statuses
 * (REQ-AKP-RES-001, REQ-GW-002).
 */

import {
  CanonicalError,
  ProjectionError,
  assertNoUnknownCriticalExtensions,
  canonicalize,
  digest,
  parseStrict,
  validateCanonicalTimestamp,
  validateDigestString,
  type StrictValue,
  type commonDefs,
} from "@cognitiveos/contracts-ts";

/** Protocol version implemented by this SDK (specs/akp/README.md). */
export const AKP_PROTOCOL_VERSION = "cognitiveos.akp/0.2";

/**
 * Digest domain for inline payload/result values, pinned by the
 * `signature-preimage` golden fixture (tests/golden/canonical-json-fixtures).
 */
export const AKP_PAYLOAD_DIGEST_DOMAIN = "akp-payload/0.2";

/** Extension IDs this client understands; unknown critical ones fail closed. */
export const SUPPORTED_EXTENSION_IDS: ReadonlyArray<string> = [];

/** Client-side envelope violation reasons (not authority error responses). */
export type EnvelopeViolationReason =
  | "malformed-json"
  | "invalid-shape"
  | "missing-field"
  | "missing-idempotency-key"
  | "invalid-timestamp"
  | "invalid-digest"
  | "version-unsupported"
  | "critical-extension-unknown"
  | "digest-mismatch"
  | "missing-error"
  | "unknown-status";

const REASON_TO_CODE: Partial<Record<EnvelopeViolationReason, string>> = {
  "version-unsupported": "VERSION_UNSUPPORTED",
  "critical-extension-unknown": "CRITICAL_EXTENSION_UNKNOWN",
  "digest-mismatch": "DIGEST_MISMATCH",
};

/**
 * Local fail-closed rejection raised by this client before/while processing
 * an envelope. Distinct from a wire `ErrorEnvelope`: the client never
 * fabricates authority responses. `code` carries the registered code whose
 * semantics the rejection mirrors, where one exists.
 */
export class EnvelopeViolation extends Error {
  readonly reason: EnvelopeViolationReason;
  readonly code: string | undefined;

  constructor(reason: EnvelopeViolationReason, detail: string) {
    super(`${reason}: ${detail}`);
    this.name = "EnvelopeViolation";
    this.reason = reason;
    this.code = REASON_TO_CODE[reason];
  }
}

/** AKP extension entry ({@link assertNoUnknownCriticalExtensions} shape). */
export interface ExtensionEntry {
  readonly id: string;
  readonly critical: boolean;
}

/** Wire error envelope: the generated registered-error shape. */
export type ErrorEnvelope = commonDefs.Error;

/** Request envelope (specs/akp/README.md §3). */
export interface RequestEnvelope<P = unknown> {
  readonly message_id: string;
  readonly operation: string;
  readonly protocol_version: string;
  readonly schema_digest: string;
  readonly sender: string;
  readonly audience: string;
  readonly correlation_id: string;
  readonly causation_id?: string;
  readonly deadline: string;
  readonly idempotency_key?: string;
  readonly authorization_ref?: string;
  readonly budget?: commonDefs.Budget;
  readonly payload?: P;
  readonly payload_ref?: string;
  readonly payload_digest?: string;
  readonly extensions?: ReadonlyArray<ExtensionEntry>;
}

/** Result statuses (specs/akp/README.md §5 base set + §10.1 additions). */
export const RESULT_STATUSES = [
  "ok",
  "accepted",
  "partial",
  "cancel_pending",
  "error",
  "outcome_unknown",
  "verified",
  "committed",
] as const;
export type ResultStatus = (typeof RESULT_STATUSES)[number];

/** Result envelope (specs/akp/README.md §3/§5). */
export interface ResultEnvelope<R = unknown> {
  readonly in_reply_to: string;
  readonly correlation_id: string;
  readonly protocol_version: string;
  readonly status: ResultStatus;
  readonly result?: R;
  readonly result_ref?: string;
  readonly result_digest?: string;
  readonly error?: ErrorEnvelope;
  readonly observed_versions?: Readonly<Record<string, number>>;
  readonly cost?: commonDefs.Budget;
  /** Opaque continuation token/object; forwarded verbatim, never decoded. */
  readonly continuation?: unknown;
  readonly audit_ref?: string;
  readonly extensions?: ReadonlyArray<ExtensionEntry>;
}

/** Operation effect declaration: effecting operations require idempotency. */
export type OperationKind = "effecting" | "read";

export interface RequestSpec<P = unknown> {
  readonly operation: string;
  readonly kind: OperationKind;
  readonly sender: string;
  readonly audience: string;
  readonly correlationId: string;
  readonly causationId?: string | undefined;
  readonly deadline: string;
  readonly schemaDigest: string;
  readonly idempotencyKey?: string | undefined;
  readonly authorizationRef?: string | undefined;
  readonly budget?: commonDefs.Budget | undefined;
  readonly payload?: P | undefined;
  readonly payloadRef?: string | undefined;
  readonly extensions?: ReadonlyArray<ExtensionEntry> | undefined;
  readonly messageId: string;
}

/** Canonical digest of an inline JSON value under the AKP payload domain. */
export function payloadDigest(value: unknown): string {
  return digest(canonicalize(JSON.stringify(value)), AKP_PAYLOAD_DIGEST_DOMAIN);
}

/**
 * Build a request envelope. Fails closed on: effecting operation without an
 * idempotency key (REQ-AKP-IDEM-001), non-canonical deadline, malformed
 * schema digest, and missing payload and payload_ref alike.
 */
export function buildRequestEnvelope<P>(spec: RequestSpec<P>): RequestEnvelope<P> {
  try {
    validateDigestString(spec.schemaDigest);
  } catch {
    throw new EnvelopeViolation("invalid-digest", `schema_digest ${spec.schemaDigest}`);
  }
  try {
    validateCanonicalTimestamp(spec.deadline);
  } catch {
    throw new EnvelopeViolation("invalid-timestamp", `deadline ${spec.deadline}`);
  }
  if (spec.kind === "effecting" && !spec.idempotencyKey) {
    throw new EnvelopeViolation(
      "missing-idempotency-key",
      `effecting operation ${spec.operation} requires an idempotency key`,
    );
  }
  if (spec.payload === undefined && spec.payloadRef === undefined) {
    throw new EnvelopeViolation("missing-field", "payload or payload_ref is required");
  }
  return {
    message_id: spec.messageId,
    operation: spec.operation,
    protocol_version: AKP_PROTOCOL_VERSION,
    schema_digest: spec.schemaDigest,
    sender: spec.sender,
    audience: spec.audience,
    correlation_id: spec.correlationId,
    ...(spec.causationId !== undefined ? { causation_id: spec.causationId } : {}),
    deadline: spec.deadline,
    ...(spec.idempotencyKey !== undefined ? { idempotency_key: spec.idempotencyKey } : {}),
    ...(spec.authorizationRef !== undefined ? { authorization_ref: spec.authorizationRef } : {}),
    ...(spec.budget !== undefined ? { budget: spec.budget } : {}),
    ...(spec.payload !== undefined
      ? { payload: spec.payload, payload_digest: payloadDigest(spec.payload) }
      : {}),
    ...(spec.payloadRef !== undefined ? { payload_ref: spec.payloadRef } : {}),
    ...(spec.extensions !== undefined ? { extensions: spec.extensions } : {}),
  };
}

export interface ResultSpec<R = unknown> {
  readonly inReplyTo: string;
  readonly correlationId: string;
  readonly status: ResultStatus;
  readonly result?: R | undefined;
  readonly resultRef?: string | undefined;
  readonly error?: ErrorEnvelope | undefined;
  readonly observedVersions?: Readonly<Record<string, number>> | undefined;
  readonly cost?: commonDefs.Budget | undefined;
  readonly continuation?: unknown;
  readonly auditRef?: string | undefined;
}

/**
 * Build a result envelope. Client SDKs receive results rather than produce
 * them; this producer exists for in-memory transport fakes and M5 test
 * harnesses.
 */
export function buildResultEnvelope<R>(spec: ResultSpec<R>): ResultEnvelope<R> {
  if (spec.status === "error" && spec.error === undefined) {
    throw new EnvelopeViolation("missing-error", "error status requires an error envelope");
  }
  return {
    in_reply_to: spec.inReplyTo,
    correlation_id: spec.correlationId,
    protocol_version: AKP_PROTOCOL_VERSION,
    status: spec.status,
    ...(spec.result !== undefined ? { result: spec.result } : {}),
    ...(spec.resultRef !== undefined ? { result_ref: spec.resultRef } : {}),
    ...(spec.error !== undefined ? { error: spec.error } : {}),
    ...(spec.observedVersions !== undefined ? { observed_versions: spec.observedVersions } : {}),
    ...(spec.cost !== undefined ? { cost: spec.cost } : {}),
    ...(spec.continuation !== undefined ? { continuation: spec.continuation } : {}),
    ...(spec.auditRef !== undefined ? { audit_ref: spec.auditRef } : {}),
  };
}

/** Serialize an envelope to wire JSON text (digests are computed separately). */
export function serializeEnvelope(envelope: RequestEnvelope<unknown> | ResultEnvelope<unknown>): string {
  return JSON.stringify(envelope);
}

/** Convert a strict-parsed value into a plain JS value for field access. */
function strictToPlain(value: StrictValue): unknown {
  if (value === null || typeof value !== "object") {
    return value;
  }
  if (Array.isArray(value)) {
    return value.map(strictToPlain);
  }
  const out: Record<string, unknown> = {};
  for (const [name, member] of value.members) {
    out[name] = strictToPlain(member);
  }
  return out;
}

function parseEnvelopeObject(text: string): Record<string, unknown> {
  let plain: unknown;
  try {
    plain = strictToPlain(parseStrict(text));
  } catch (error) {
    if (error instanceof CanonicalError) {
      throw new EnvelopeViolation("malformed-json", error.message);
    }
    throw error;
  }
  if (plain === null || typeof plain !== "object" || Array.isArray(plain)) {
    throw new EnvelopeViolation("invalid-shape", "envelope is not a JSON object");
  }
  return plain as Record<string, unknown>;
}

function requireString(obj: Record<string, unknown>, field: string): string {
  const value = obj[field];
  if (typeof value !== "string") {
    throw new EnvelopeViolation("missing-field", `${field} must be a string`);
  }
  return value;
}

/** Version gate (REQ-AKP-VER-001): exact pinned version under SemVer 0.x. */
function checkVersion(obj: Record<string, unknown>): void {
  const version = requireString(obj, "protocol_version");
  if (version !== AKP_PROTOCOL_VERSION) {
    throw new EnvelopeViolation("version-unsupported", version);
  }
}

/** Critical-extension gate, before any payload processing (REQ-AKP-ENV-002). */
function checkExtensions(obj: Record<string, unknown>): void {
  try {
    assertNoUnknownCriticalExtensions(obj, SUPPORTED_EXTENSION_IDS);
  } catch (error) {
    if (error instanceof ProjectionError) {
      throw new EnvelopeViolation("critical-extension-unknown", error.message);
    }
    throw error;
  }
}

function verifyInlineDigest(value: unknown, declared: unknown, field: string): void {
  if (declared === undefined) {
    return;
  }
  if (typeof declared !== "string") {
    throw new EnvelopeViolation("invalid-digest", `${field} must be a string`);
  }
  try {
    validateDigestString(declared);
  } catch {
    throw new EnvelopeViolation("invalid-digest", `${field} ${declared}`);
  }
  if (value === undefined) {
    return;
  }
  const computed = payloadDigest(value);
  if (computed !== declared) {
    throw new EnvelopeViolation("digest-mismatch", `${field}: declared ${declared}, computed ${computed}`);
  }
}

/** Parse and gate a request envelope (receiver side; used by test fakes). */
export function parseRequestEnvelope<P = unknown>(text: string): RequestEnvelope<P> {
  const obj = parseEnvelopeObject(text);
  checkVersion(obj);
  checkExtensions(obj);
  const envelope: RequestEnvelope<P> = {
    message_id: requireString(obj, "message_id"),
    operation: requireString(obj, "operation"),
    protocol_version: requireString(obj, "protocol_version"),
    schema_digest: requireString(obj, "schema_digest"),
    sender: requireString(obj, "sender"),
    audience: requireString(obj, "audience"),
    correlation_id: requireString(obj, "correlation_id"),
    deadline: requireString(obj, "deadline"),
    ...(obj["causation_id"] !== undefined ? { causation_id: requireString(obj, "causation_id") } : {}),
    ...(obj["idempotency_key"] !== undefined
      ? { idempotency_key: requireString(obj, "idempotency_key") }
      : {}),
    ...(obj["authorization_ref"] !== undefined
      ? { authorization_ref: requireString(obj, "authorization_ref") }
      : {}),
    ...(obj["budget"] !== undefined ? { budget: obj["budget"] as commonDefs.Budget } : {}),
    ...(obj["payload"] !== undefined ? { payload: obj["payload"] as P } : {}),
    ...(obj["payload_ref"] !== undefined ? { payload_ref: requireString(obj, "payload_ref") } : {}),
    ...(obj["payload_digest"] !== undefined
      ? { payload_digest: requireString(obj, "payload_digest") }
      : {}),
    ...(obj["extensions"] !== undefined
      ? { extensions: obj["extensions"] as ReadonlyArray<ExtensionEntry> }
      : {}),
  };
  if (envelope.payload === undefined && envelope.payload_ref === undefined) {
    throw new EnvelopeViolation("missing-field", "payload or payload_ref is required");
  }
  verifyInlineDigest(envelope.payload, obj["payload_digest"], "payload_digest");
  return envelope;
}

function parseErrorEnvelope(value: unknown): ErrorEnvelope {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new EnvelopeViolation("missing-error", "error member is not an object");
  }
  const error = value as Record<string, unknown>;
  if (
    typeof error["code"] !== "string" ||
    typeof error["category"] !== "string" ||
    typeof error["retryable"] !== "boolean" ||
    typeof error["stage"] !== "string"
  ) {
    throw new EnvelopeViolation(
      "missing-error",
      "error envelope requires code, category, retryable, stage",
    );
  }
  return error as unknown as ErrorEnvelope;
}

/** Parse and gate a result envelope (client receive path). */
export function parseResultEnvelope<R = unknown>(text: string): ResultEnvelope<R> {
  const obj = parseEnvelopeObject(text);
  checkVersion(obj);
  checkExtensions(obj);
  const status = requireString(obj, "status");
  if (!(RESULT_STATUSES as ReadonlyArray<string>).includes(status)) {
    throw new EnvelopeViolation("unknown-status", status);
  }
  const envelope: ResultEnvelope<R> = {
    in_reply_to: requireString(obj, "in_reply_to"),
    correlation_id: requireString(obj, "correlation_id"),
    protocol_version: requireString(obj, "protocol_version"),
    status: status as ResultStatus,
    ...(obj["result"] !== undefined ? { result: obj["result"] as R } : {}),
    ...(obj["result_ref"] !== undefined ? { result_ref: requireString(obj, "result_ref") } : {}),
    ...(obj["result_digest"] !== undefined
      ? { result_digest: requireString(obj, "result_digest") }
      : {}),
    ...(obj["error"] !== undefined ? { error: parseErrorEnvelope(obj["error"]) } : {}),
    ...(obj["observed_versions"] !== undefined
      ? { observed_versions: obj["observed_versions"] as Readonly<Record<string, number>> }
      : {}),
    ...(obj["cost"] !== undefined ? { cost: obj["cost"] as commonDefs.Budget } : {}),
    ...(obj["continuation"] !== undefined ? { continuation: obj["continuation"] } : {}),
    ...(obj["audit_ref"] !== undefined ? { audit_ref: requireString(obj, "audit_ref") } : {}),
    ...(obj["extensions"] !== undefined
      ? { extensions: obj["extensions"] as ReadonlyArray<ExtensionEntry> }
      : {}),
  };
  if (envelope.status === "error" && envelope.error === undefined) {
    throw new EnvelopeViolation("missing-error", "error status requires an error envelope");
  }
  verifyInlineDigest(envelope.result, obj["result_digest"], "result_digest");
  return envelope;
}
