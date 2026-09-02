/**
 * Hidden Pi assistant projections (P12-T09 rail write; P13-T03 real inference).
 * Edit → review → POST assistant.turn (daemon runs exact Pi, registers a
 * candidate object chain with typed provenance) then draft.apply (owner write).
 * Chat has no Approve. draft.apply is not authority-confirm. The assistant
 * plane cannot write SecretStore, archive, Memory, or confirm. When no Provider
 * is bound the daemon answers `ASSISTANT_PROVIDER_UNBOUND`; the client renders
 * a Settings pointer, never a fake chat box and never a silent bind.
 */

import { asRecord } from "../projections";
import { containsSecretMaterial } from "../../policy";

export const ASSISTANT_TURN_PATH = "/management/project/v1/assistant.turn";
export const ASSISTANT_STATUS_PATH = "/management/project/v1/assistant.status";
export const DRAFT_APPLY_PATH = "/management/project/v1/draft.apply";
export const ASSISTANT_PROVIDER_UNBOUND_CODE = "ASSISTANT_PROVIDER_UNBOUND";
/** Route the pointer targets; mirrors the daemon's `settings_route`. */
export const ASSISTANT_SETTINGS_ROUTE = "/settings";

export const ASSISTANT_TURN_KINDS = ["explain", "navigate", "research", "propose"] as const;
export type AssistantTurnKind = (typeof ASSISTANT_TURN_KINDS)[number];

export const ASSISTANT_OBJECT_KINDS = [
  "business-brief",
  "research-run",
  "charter",
  "axis",
  "roster",
  "recipe",
] as const;
export type AssistantObjectKind = (typeof ASSISTANT_OBJECT_KINDS)[number];

export interface AssistantChainField {
  name: string;
  value: string;
  /** `sources` | `owner-stated` | `assistant-assumption` (typed provenance). */
  provenanceKind: string;
  sourceUris: string[];
}

export interface AssistantChainObject {
  objectKind: string;
  summary: string;
  fields: AssistantChainField[];
}

export interface AssistantTurnRow {
  candidateId: string;
  candidateDigest: string;
  previewId: string;
  objectKind: string;
  /** Bounded assistant reply; empty when the daemon returned none (P12 shape). */
  reply: string;
  modelId: string;
  providerRoundTrips: number;
  chain: AssistantChainObject[];
  fetchedSources: string[];
  refusedSources: string[];
}

export type AssistantStatusKind = "ready" | "provider_unbound" | "pi_unavailable" | "unknown";

export interface AssistantStatusRow {
  status: AssistantStatusKind;
  chatInput: boolean;
  modelId: string;
  guidance: string;
  settingsRoute: string;
  piDetail: string;
}

export interface DraftApplyRow {
  newBaseSeq: string;
  payloadDigest: string;
}

function stringField(record: Record<string, unknown>, key: string): string {
  const value = record[key];
  return typeof value === "string" ? value : "";
}

function stringList(value: unknown): string[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value.filter((item): item is string => typeof item === "string" && item.length > 0);
}

function renderValue(value: unknown): string {
  if (typeof value === "string") {
    return value;
  }
  if (value === null || value === undefined) {
    return "";
  }
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  try {
    return JSON.stringify(value);
  } catch {
    return "";
  }
}

export function projectAssistantChain(value: unknown): AssistantChainObject[] {
  if (!Array.isArray(value)) {
    return [];
  }
  const objects: AssistantChainObject[] = [];
  for (const item of value) {
    const record = asRecord(item);
    const objectKind = stringField(record, "object_kind");
    if (objectKind.length === 0) {
      continue;
    }
    const fieldsRecord = asRecord(record.fields);
    const fields: AssistantChainField[] = [];
    for (const [name, raw] of Object.entries(fieldsRecord)) {
      const field = asRecord(raw);
      const provenance = asRecord(field.provenance);
      const provenanceKind =
        stringField(provenance, "kind") || (Array.isArray(field.provenance) ? "sources" : "");
      if (provenanceKind.length === 0) {
        // A field without typed provenance is not rendered as a candidate fact.
        continue;
      }
      const sources = Array.isArray(field.provenance)
        ? field.provenance
        : Array.isArray(provenance.sources)
          ? provenance.sources
          : [];
      const sourceUris = sources
        .map((source) => stringField(asRecord(source), "uri"))
        .filter((uri) => uri.length > 0);
      fields.push({ name, value: renderValue(field.value), provenanceKind, sourceUris });
    }
    objects.push({ objectKind, summary: stringField(record, "summary"), fields });
  }
  return objects;
}

export function projectAssistantTurn(body: unknown): AssistantTurnRow | undefined {
  const record = asRecord(body);
  if (typeof record.candidate_digest !== "string" || record.candidate_digest.length === 0) {
    return undefined;
  }
  const research = asRecord(record.research);
  const refused = Array.isArray(research.refused)
    ? research.refused.map((item) => stringField(asRecord(item), "uri")).filter((uri) => uri.length > 0)
    : [];
  const roundTrips = record.provider_round_trips;
  return {
    candidateId: stringField(record, "candidate_id"),
    candidateDigest: record.candidate_digest,
    previewId: stringField(record, "preview_id"),
    objectKind: stringField(record, "object_kind") || "unknown",
    reply: stringField(record, "reply"),
    modelId: stringField(record, "model_id"),
    providerRoundTrips: typeof roundTrips === "number" && roundTrips >= 0 ? roundTrips : 0,
    chain: projectAssistantChain(record.chain),
    fetchedSources: stringList(research.fetched),
    refusedSources: refused,
  };
}

export function projectAssistantStatus(body: unknown): AssistantStatusRow {
  const record = asRecord(body);
  const raw = stringField(record, "status");
  const status: AssistantStatusKind =
    raw === "ready" || raw === "provider_unbound" || raw === "pi_unavailable" ? raw : "unknown";
  return {
    status,
    // Only a daemon `ready` with an explicit true opens the input.
    chatInput: status === "ready" && record.chat_input === true,
    modelId: stringField(record, "model_id"),
    guidance: stringField(record, "guidance"),
    settingsRoute: ASSISTANT_SETTINGS_ROUTE,
    piDetail: stringField(record, "pi_detail"),
  };
}

/** True when the daemon refused the turn because no Provider is bound. */
export function isProviderUnbound(status: number, body: unknown): boolean {
  const record = asRecord(body);
  return (
    status === 409 &&
    (record.code === ASSISTANT_PROVIDER_UNBOUND_CODE || record.status === "provider_unbound")
  );
}

export function projectDraftApply(body: unknown): DraftApplyRow | undefined {
  const record = asRecord(body);
  const payloadDigest =
    typeof record.payload_digest === "string" ? record.payload_digest : "";
  if (payloadDigest.length === 0) {
    return undefined;
  }
  const seq = record.new_base_seq;
  return {
    newBaseSeq: typeof seq === "number" ? String(seq) : typeof seq === "string" ? seq : "unknown",
    payloadDigest,
  };
}

export function railWriteReady(input: {
  draftId: string;
  baseSeq: string;
  text: string;
}): { ok: true } | { ok: false; reason: string } {
  const draftId = input.draftId.trim();
  const baseSeq = input.baseSeq.trim();
  const text = input.text.trim();
  if (draftId.length === 0) {
    return { ok: false, reason: "draft_id required. This rail does not invent an open draft." };
  }
  if (!/^\d+$/.test(baseSeq)) {
    return { ok: false, reason: "base_seq required as a non-negative integer from the daemon draft." };
  }
  if (text.length === 0) {
    return { ok: false, reason: "canvas edit required before review." };
  }
  if (containsSecretMaterial(draftId) || containsSecretMaterial(text)) {
    return { ok: false, reason: "secret-shaped material is not posted from this rail." };
  }
  return { ok: true };
}

/** Owner text for a create-page turn: non-empty, secret-free, bounded. */
export function assistantTurnReady(text: string): { ok: true } | { ok: false; reason: string } {
  const trimmed = text.trim();
  if (trimmed.length === 0) {
    return { ok: false, reason: "type a question or request before sending." };
  }
  if (trimmed.length > 4000) {
    return { ok: false, reason: "message is too long for one bounded turn (4000 characters)." };
  }
  if (containsSecretMaterial(trimmed)) {
    return { ok: false, reason: "secret-shaped material is not posted to the assistant. Keys belong in Settings." };
  }
  return { ok: true };
}
