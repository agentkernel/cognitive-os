/**
 * Snapshot + cursor watch consumer (docs/standards/event-audit-watch.md §5;
 * REQ-SHELL-WATCH-001, REQ-AKP-SHELL-002, REQ-AKP-CONT-001,
 * REQ-AKP-STR-001..002).
 *
 * Contract behavior:
 * - one consistent snapshot, then ordered deltas with a resumable cursor;
 * - delivery is at-least-once; the consumer deduplicates by sequence;
 * - a disconnect resumes from the last acknowledged cursor
 *   (`watch.resume` carrying subscription, snapshot version, cursor and
 *   dedupe window);
 * - `WATCH_CURSOR_STALE` means continuity is impossible: the consumer
 *   discards its position and opens a fresh authorized snapshot
 *   (`watch.open`), and the new snapshot item tells the application to
 *   re-base — a gap is never silently skipped;
 * - recovery is bounded; exhaustion fails closed.
 *
 * Frames are stream fragments (AKP §8), parsed strictly; the payloads are
 * authority projections consumed verbatim.
 */

import { CanonicalError, parseStrict } from "@cognitiveos/contracts-ts";

import type { CallSpec } from "./client.js";
import { SHELL_SCHEMA_DIGESTS, type WatchSubscription } from "./views.js";

/** Minimal stream source: any channel client exposing `openStream`. */
export interface WatchStreamSource {
  openStream<P>(spec: CallSpec<P>): AsyncIterable<string>;
}

/** One stream fragment (AKP §8 shape, watch profile). */
export interface WatchStreamFrame {
  readonly stream_id: string;
  readonly sequence: number;
  readonly kind: "snapshot" | "delta" | "error";
  readonly payload?: unknown;
  /** Present on snapshot frames: the authority snapshot version. */
  readonly snapshot_version?: number;
  readonly final?: boolean;
}

/** Serialize a frame (test fakes and the M5 harness produce frames). */
export function frameText(frame: WatchStreamFrame): string {
  return JSON.stringify(frame);
}

export type WatchViolationReason =
  | "malformed-frame"
  | "missing-snapshot"
  | "stream-error"
  | "recovery-exhausted";

/** Fail-closed watch consumer error. */
export class WatchViolation extends Error {
  readonly reason: WatchViolationReason;
  /** Registered error code carried by a stream error frame, if any. */
  readonly code: string | undefined;

  constructor(reason: WatchViolationReason, detail: string, code?: string) {
    super(`${reason}: ${detail}`);
    this.name = "WatchViolation";
    this.reason = reason;
    this.code = code;
  }
}

/** Item delivered to the application. */
export interface WatchItem<T = unknown> {
  readonly kind: "snapshot" | "delta";
  /** Resumable cursor position of this item (frame sequence). */
  readonly cursor: number;
  /** Snapshot version this item is based on. */
  readonly snapshotVersion: number;
  readonly payload: T;
}

export interface WatchParams {
  readonly subscription: WatchSubscription;
  readonly deadline: string;
  /** Envelope schema pin; defaults to the watch-subscription schema digest. */
  readonly schemaDigest?: string;
  /** Bound on consecutive recoveries without progress. Default 3. */
  readonly maxRecoveryAttempts?: number;
  /** Reattach position retained across a detach (REQ-SHELL-ATTACH-001). */
  readonly resumeFrom?: { readonly snapshotVersion: number; readonly lastAckCursor: number };
}

function parseFrame(text: string): WatchStreamFrame {
  let plain: unknown;
  try {
    plain = strictToPlainJson(text);
  } catch (error) {
    throw new WatchViolation("malformed-frame", error instanceof Error ? error.message : String(error));
  }
  if (plain === null || typeof plain !== "object" || Array.isArray(plain)) {
    throw new WatchViolation("malformed-frame", "frame is not a JSON object");
  }
  const frame = plain as Record<string, unknown>;
  const sequence = frame["sequence"];
  const kind = frame["kind"];
  if (
    typeof frame["stream_id"] !== "string" ||
    typeof sequence !== "number" ||
    !Number.isInteger(sequence) ||
    sequence < 0 ||
    (kind !== "snapshot" && kind !== "delta" && kind !== "error")
  ) {
    throw new WatchViolation("malformed-frame", "frame requires stream_id, integer sequence, known kind");
  }
  return frame as unknown as WatchStreamFrame;
}

/** Strict parse (BOM/duplicate/unsafe-integer rejection) to a plain value. */
function strictToPlainJson(text: string): unknown {
  try {
    return toPlain(parseStrict(text));
  } catch (error) {
    if (error instanceof CanonicalError) {
      throw new Error(error.message);
    }
    throw error;
  }
}

function toPlain(value: ReturnType<typeof parseStrict>): unknown {
  if (value === null || typeof value !== "object") {
    return value;
  }
  if (Array.isArray(value)) {
    return value.map(toPlain);
  }
  const out: Record<string, unknown> = {};
  for (const [name, member] of value.members) {
    out[name] = toPlain(member);
  }
  return out;
}

function errorCodeOf(payload: unknown): string | undefined {
  if (payload !== null && typeof payload === "object" && !Array.isArray(payload)) {
    const code = (payload as Record<string, unknown>)["code"];
    if (typeof code === "string") {
      return code;
    }
  }
  return undefined;
}

/**
 * Consume a watch subscription as an async iterator of snapshot/delta
 * items with automatic bounded recovery.
 */
export async function* consumeWatch(
  source: WatchStreamSource,
  params: WatchParams,
): AsyncGenerator<WatchItem, void, undefined> {
  const maxRecovery = params.maxRecoveryAttempts ?? 3;
  const schemaDigest =
    params.schemaDigest ?? SHELL_SCHEMA_DIGESTS["watch-subscription.schema.json"];

  let snapshotVersion = params.resumeFrom?.snapshotVersion;
  let lastAck = params.resumeFrom?.lastAckCursor;
  let hasSnapshot = params.resumeFrom !== undefined;
  let mode: "open" | "resume" = params.resumeFrom !== undefined ? "resume" : "open";
  let recoveryAttempts = 0;

  const bumpRecovery = (detail: string): void => {
    recoveryAttempts += 1;
    if (recoveryAttempts > maxRecovery) {
      throw new WatchViolation("recovery-exhausted", detail);
    }
  };

  outer: for (;;) {
    const payload: WatchSubscription =
      mode === "open"
        ? params.subscription
        : {
            ...params.subscription,
            snapshot_version: snapshotVersion ?? 0,
            cursor: lastAck ?? 0,
            high_watermark: lastAck ?? 0,
          };
    const stream = source.openStream({
      operation: mode === "open" ? "watch.open" : "watch.resume",
      kind: "read",
      schemaDigest,
      deadline: params.deadline,
      payload,
    });

    for await (const text of stream) {
      const frame = parseFrame(text);

      if (frame.kind === "error") {
        const code = errorCodeOf(frame.payload);
        if (code === "WATCH_CURSOR_STALE") {
          // Continuity is gone; only a fresh authorized snapshot may
          // continue the watch (vector SHELL-WATCH-RESUME-006).
          bumpRecovery("WATCH_CURSOR_STALE re-snapshot budget exhausted");
          mode = "open";
          snapshotVersion = undefined;
          lastAck = undefined;
          hasSnapshot = false;
          continue outer;
        }
        throw new WatchViolation("stream-error", `stream failed with ${code ?? "unknown code"}`, code);
      }

      if (frame.kind === "snapshot") {
        snapshotVersion = frame.snapshot_version ?? 0;
        lastAck = frame.sequence;
        hasSnapshot = true;
        recoveryAttempts = 0;
        yield {
          kind: "snapshot",
          cursor: frame.sequence,
          snapshotVersion,
          payload: frame.payload,
        };
        if (frame.final === true) {
          return;
        }
        continue;
      }

      // Delta frame.
      if (!hasSnapshot) {
        throw new WatchViolation("missing-snapshot", "delta before any snapshot on a fresh stream");
      }
      if (lastAck !== undefined && frame.sequence <= lastAck) {
        // At-least-once replay: already delivered, drop.
        if (frame.final === true) {
          return;
        }
        continue;
      }
      if (lastAck !== undefined && frame.sequence > lastAck + 1) {
        // Sequence gap: never skip silently; go back to the last
        // acknowledged cursor and let the authority backfill or declare
        // the cursor stale.
        bumpRecovery(`sequence gap after cursor ${lastAck} (saw ${frame.sequence})`);
        mode = "resume";
        continue outer;
      }
      lastAck = frame.sequence;
      recoveryAttempts = 0;
      yield {
        kind: "delta",
        cursor: frame.sequence,
        snapshotVersion: snapshotVersion ?? 0,
        payload: frame.payload,
      };
      if (frame.final === true) {
        return;
      }
    }

    // Stream ended without a final frame: disconnect; resume from cursor.
    bumpRecovery("stream disconnected and resume budget exhausted");
    mode = hasSnapshot ? "resume" : "open";
  }
}
