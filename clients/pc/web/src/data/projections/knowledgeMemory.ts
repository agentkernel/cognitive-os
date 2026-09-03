/**
 * P13-T07 Knowledge/Memory projections. New GETs use readJson, not
 * fetchProjection (T06 owns KNOWN_ROUTES). Chat auto-admission stays
 * honest-empty / Requires-backend while the T06 archive source is absent.
 */

import { asList, asRecord } from "../projections";

export const MEMORY_PROMOTES_PATH = "/management/resource/v1/memory/promotes";
export const MEMORY_PROMOTE_REQUEST_PATH = "/management/resource/v1/memory/promote.request";
export const MEMORY_PROMOTE_CONFIRM_PATH = "/management/resource/v1/memory/promote.confirm";
export const MEMORY_AUTO_ADMIT_PATH = "/management/resource/v1/memory/auto-admit.chat";
export const MEMORY_CORRECT_PATH = "/management/resource/v1/memory/correct";
export const MEMORY_FORGET_PATH = "/management/resource/v1/memory/forget";
export const MEMORY_REMEMBER_PATH = "/management/resource/v1/memory/remember";

export const CHAT_AUTO_ADMIT_REQUIRES_BACKEND =
  "Chat auto-admission Requires-backend until the conversation archive (P13-T06) is present. This list stays empty. No admission control.";

export function memoryPromotesPath(projectId: string): string {
  return `${MEMORY_PROMOTES_PATH}?project_id=${encodeURIComponent(projectId)}`;
}

export interface MemoryPromoteView {
  promoteId: string;
  memoryId: string;
  fromProjectId: string;
  toProjectId: string;
  previewDigest: string;
  status: string;
  promotedMemoryId?: string;
}

export function projectMemoryPromotes(body: unknown): MemoryPromoteView[] {
  const rows: MemoryPromoteView[] = [];
  for (const item of asList(body, ["promotes"])) {
    const record = asRecord(item);
    if (typeof record.promote_id !== "string" || record.promote_id.length === 0) {
      continue;
    }
    rows.push({
      promoteId: record.promote_id,
      memoryId: typeof record.memory_id === "string" ? record.memory_id : "unknown",
      fromProjectId: typeof record.from_project_id === "string" ? record.from_project_id : "unknown",
      toProjectId: typeof record.to_project_id === "string" ? record.to_project_id : "unknown",
      previewDigest: typeof record.preview_digest === "string" ? record.preview_digest : "",
      status: typeof record.status === "string" ? record.status : "unknown",
      promotedMemoryId:
        typeof record.promoted_memory_id === "string" ? record.promoted_memory_id : undefined,
    });
  }
  return rows;
}

export function promotePreviewUncopied(rows: MemoryPromoteView[], promoteId: string): boolean {
  return !rows.some((row) => row.promoteId === promoteId && row.status === "confirmed");
}
