/*
 * Memory family (W7) view models — docs/design/18 §2.
 *
 * List is the Resource Manager envelope (non-tombstoned, limit 64). Explain
 * is GET /management/resource/v1/memory/object. Content search is BD-6.
 * Remember/Forget are class-A daemon mutations, never inferred locally.
 */

import { asRecord } from "../projections";
import type { ResourceEnvelope } from "./resources";

export const MEMORY_SEARCH_UNAVAILABLE =
  "Content search is not available over HTTP (BD-6).";

export const MEMORY_RETENTION_CAP_DAYS = 365;
export const MEMORY_RETENTION_CAP_SECONDS = 31_536_000;
export const MEMORY_FORGET_CONSEQUENCE =
  "Forget creates a durable tombstone; stale copies cannot resurrect it.";

export function memoryObjectPath(id: string): string {
  return `/management/resource/v1/memory/object?id=${encodeURIComponent(id)}`;
}

export function memoryInspectPath(id: string): string {
  return `/management/resource/v1/inspect?family=memory&id=${encodeURIComponent(id)}`;
}

export function memoryObjectKey(id: string): string {
  return `resources:memory:object:${id}`;
}

export function memoryInspectKey(id: string): string {
  return `resources:memory:inspect:${id}`;
}

export interface MemoryExplainView {
  memoryId: string;
  candidateId?: string;
  decisionId?: string;
  canonicalJson?: string;
}

export function projectMemoryExplain(body: unknown): MemoryExplainView {
  const record = asRecord(body);
  const memory = asRecord(record.memory ?? record);
  return {
    memoryId: String(memory.memory_id ?? memory.id ?? "unknown"),
    candidateId: memory.candidate_id == null ? undefined : String(memory.candidate_id),
    decisionId: memory.decision_id == null ? undefined : String(memory.decision_id),
    canonicalJson:
      typeof memory.canonical_json === "string" ? memory.canonical_json : undefined,
  };
}

export function projectMemoryInspect(body: unknown): ResourceEnvelope {
  const record = asRecord(body);
  const resource = asRecord(record.resource ?? record);
  return {
    id: String(resource.id ?? resource.memory_id ?? "unknown"),
    family: String(resource.family ?? "memory"),
    health: resource.health == null ? undefined : String(resource.health),
  };
}

export function memoryMasterFooter(count: number, atBound: boolean): string {
  const bound = atBound
    ? `envelope at bound (limit 64)`
    : `envelope limit 64`;
  return `Showing ${count} admitted object${count === 1 ? "" : "s"} · ${bound} · tombstones are not in this list`;
}

export function retentionExpiryUnix(
  days: number,
  nowSec: number,
): { ok: true; unix: number } | { ok: false; reason: string } {
  if (!Number.isFinite(days) || days <= 0) {
    return { ok: false, reason: "retention days must be a positive number" };
  }
  if (days > MEMORY_RETENTION_CAP_DAYS) {
    return { ok: false, reason: "retention is capped at 365 days" };
  }
  const seconds = Math.round(days * 86_400);
  if (seconds > MEMORY_RETENTION_CAP_SECONDS) {
    return { ok: false, reason: "retention is capped at 365 days" };
  }
  return { ok: true, unix: nowSec + seconds };
}
