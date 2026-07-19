/**
 * Digest projections, content-digest verification, canonical timestamp and
 * digest-string validation, and the unknown-critical-extension gate.
 * TypeScript twin of `crates/cognitive-contracts/src/projection.rs`
 * (docs/standards/canonical-encoding-and-digest.md sections 3, 6, 8, 10,
 * 15); cross-language behavior pinned by
 * `tests/golden/digest-and-projection-fixtures.json`.
 */

import { canonicalize, digest } from "./canonical.js";

export type ProjectionErrorCategory =
  | "invalid-digest"
  | "invalid-timestamp"
  | "invalid-pointer"
  | "missing-digest"
  | "digest-mismatch"
  | "critical-extension-unknown";

/** Rejection categories shared with the Rust twin and the golden fixtures. */
export class ProjectionError extends Error {
  constructor(
    readonly category: ProjectionErrorCategory,
    detail: string,
  ) {
    super(`${category}: ${detail}`);
    this.name = "ProjectionError";
  }
}

/** Validate the machine digest string form (section 8). */
export function validateDigestString(value: string): void {
  if (!/^sha256:[0-9a-f]{64}$/.test(value)) {
    throw new ProjectionError("invalid-digest", value);
  }
}

/**
 * Validate the canonical RFC 3339 UTC timestamp FORM (section 6):
 * `YYYY-MM-DDTHH:MM:SS[.fraction]Z`, uppercase `T`/`Z`, no offset or local
 * time, no leap second, fraction 1-9 digits, no trailing zeros, zero
 * fraction omitted.
 */
export function validateCanonicalTimestamp(value: string): void {
  const match = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.(\d{1,9}))?Z$/.exec(value);
  if (!match) {
    throw new ProjectionError("invalid-timestamp", value);
  }
  const month = Number(match[2]);
  const day = Number(match[3]);
  const hour = Number(match[4]);
  const minute = Number(match[5]);
  const second = Number(match[6]);
  const fraction = match[7];
  const rangesOk =
    month >= 1 && month <= 12 && day >= 1 && day <= 31 && hour <= 23 && minute <= 59 && second <= 59;
  if (!rangesOk) {
    throw new ProjectionError("invalid-timestamp", value);
  }
  if (fraction !== undefined && (fraction.endsWith("0") || /^0+$/.test(fraction))) {
    throw new ProjectionError("invalid-timestamp", value);
  }
}

type JsonValue = null | boolean | number | string | JsonValue[] | { [key: string]: JsonValue };

const unescapeSegment = (segment: string): string => segment.replaceAll("~1", "/").replaceAll("~0", "~");

/**
 * Remove one JSON Pointer path if present (declared exclusions that do not
 * exist are a no-op — self fields are excluded "if present", section 10).
 */
function removePointer(value: JsonValue, pointer: string): void {
  if (pointer.length === 0 || !pointer.startsWith("/")) {
    throw new ProjectionError("invalid-pointer", pointer);
  }
  const segments = pointer.split("/").slice(1).map(unescapeSegment);
  const last = segments.pop();
  if (last === undefined) {
    throw new ProjectionError("invalid-pointer", pointer);
  }
  let current: JsonValue = value;
  for (const segment of segments) {
    if (Array.isArray(current)) {
      if (!/^\d+$/.test(segment)) {
        throw new ProjectionError("invalid-pointer", pointer);
      }
      const next: JsonValue | undefined = current[Number(segment)];
      if (next === undefined) {
        return;
      }
      current = next;
    } else if (current !== null && typeof current === "object") {
      const next: JsonValue | undefined = current[segment];
      if (next === undefined) {
        return;
      }
      current = next;
    } else {
      return;
    }
  }
  if (Array.isArray(current)) {
    throw new ProjectionError("invalid-pointer", pointer);
  }
  if (current !== null && typeof current === "object") {
    delete current[last];
  }
}

/**
 * Digest projection (section 10): the value with ONLY the declared
 * `digest_excluded` paths removed. No other path may be dropped.
 */
export function digestProjection(value: unknown, digestExcluded: ReadonlyArray<string>): unknown {
  const projected = JSON.parse(JSON.stringify(value)) as JsonValue;
  for (const pointer of digestExcluded) {
    removePointer(projected, pointer);
  }
  return projected;
}

/** Canonical bytes of the digest projection. */
export function projectionCanonicalBytes(
  value: unknown,
  digestExcluded: ReadonlyArray<string>,
): Uint8Array {
  return canonicalize(JSON.stringify(digestProjection(value, digestExcluded)));
}

/** Content digest of the projection under the contract's domain. */
export function projectionDigest(
  value: unknown,
  digestExcluded: ReadonlyArray<string>,
  domain: string,
): string {
  return digest(projectionCanonicalBytes(value, digestExcluded), domain);
}

function pointerGet(value: unknown, pointer: string): unknown {
  if (pointer.length === 0 || !pointer.startsWith("/")) {
    throw new ProjectionError("invalid-pointer", pointer);
  }
  let current: unknown = value;
  for (const segment of pointer.split("/").slice(1).map(unescapeSegment)) {
    if (Array.isArray(current)) {
      current = current[Number(segment)];
    } else if (current !== null && typeof current === "object") {
      current = (current as Record<string, unknown>)[segment];
    } else {
      return undefined;
    }
  }
  return current;
}

/**
 * Verify a self-referential content digest: read the declared digest at
 * `digestPointer`, recompute the projection digest from the received
 * semantic value, and fail closed on any mismatch (sections 10 and 15).
 */
export function verifyContentDigest(
  value: unknown,
  digestExcluded: ReadonlyArray<string>,
  domain: string,
  digestPointer: string,
): void {
  const declared = pointerGet(value, digestPointer);
  if (typeof declared !== "string") {
    throw new ProjectionError("missing-digest", digestPointer);
  }
  validateDigestString(declared);
  const computed = projectionDigest(value, digestExcluded, domain);
  if (declared !== computed) {
    throw new ProjectionError("digest-mismatch", `declared ${declared}, computed ${computed}`);
  }
}

/**
 * Reject unknown critical extensions before any payload processing
 * (section 3; AKP envelope `extensions`; CRITICAL_EXTENSION_UNKNOWN). An
 * extension entry is `{id: string, critical: boolean}`; a malformed entry
 * cannot be verified and therefore fails closed as critical.
 */
export function assertNoUnknownCriticalExtensions(
  value: unknown,
  supportedIds: ReadonlyArray<string>,
): void {
  if (value === null || typeof value !== "object" || !("extensions" in value)) {
    return;
  }
  const extensions = (value as { extensions: unknown }).extensions;
  if (!Array.isArray(extensions)) {
    throw new ProjectionError("critical-extension-unknown", "extensions is not an array");
  }
  for (const item of extensions) {
    const id = item !== null && typeof item === "object" ? (item as { id?: unknown }).id : undefined;
    const critical =
      item !== null && typeof item === "object" ? (item as { critical?: unknown }).critical : undefined;
    if (typeof id !== "string" || typeof critical !== "boolean") {
      throw new ProjectionError("critical-extension-unknown", "malformed extension entry");
    }
    if (critical && !supportedIds.includes(id)) {
      throw new ProjectionError("critical-extension-unknown", id);
    }
  }
}
