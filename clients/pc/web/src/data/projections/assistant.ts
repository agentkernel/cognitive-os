/**
 * Right-rail canvas write (P12-T09 Dual Track).
 * Edit → review → POST assistant.turn (candidate) then draft.apply (owner write).
 * Chat has no Approve. draft.apply is not authority-confirm. The assistant
 * plane cannot write SecretStore, archive, or confirm.
 */

import { asRecord } from "../projections";
import { containsSecretMaterial } from "../../policy";

export const ASSISTANT_TURN_PATH = "/management/project/v1/assistant.turn";
export const DRAFT_APPLY_PATH = "/management/project/v1/draft.apply";

export interface AssistantTurnRow {
  candidateId: string;
  candidateDigest: string;
  previewId: string;
  objectKind: string;
}

export interface DraftApplyRow {
  newBaseSeq: string;
  payloadDigest: string;
}

export function projectAssistantTurn(body: unknown): AssistantTurnRow | undefined {
  const record = asRecord(body);
  if (typeof record.candidate_digest !== "string" || record.candidate_digest.length === 0) {
    return undefined;
  }
  return {
    candidateId: typeof record.candidate_id === "string" ? record.candidate_id : "",
    candidateDigest: record.candidate_digest,
    previewId: typeof record.preview_id === "string" ? record.preview_id : "",
    objectKind: typeof record.object_kind === "string" ? record.object_kind : "unknown",
  };
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
