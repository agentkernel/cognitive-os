/**
 * HITL announcement projection (P11-T13 Dual Track).
 * Source is GET /management/project/v1/pending-previews?subject_ref=.
 * pending-previews omits preview_digest (T09). This file never copies a
 * digest into the row and never invents Confirm/Approve.
 */

import { asList, asRecord } from "../projections";

export const HITL_KEY = "opc:hitl-previews";
export const PENDING_PREVIEWS_PATH = "/management/project/v1/pending-previews";

export function pendingPreviewsPath(subjectRef: string): string {
  return `${PENDING_PREVIEWS_PATH}?subject_ref=${encodeURIComponent(subjectRef)}`;
}

/**
 * Project-center HITL canvas (T09). Today deep-links here. Not Inbox L1
 * and not an independent `#/hitl/:id` space.
 */
export function hitlCanvasPath(previewId: string): string {
  return `/projects?preview=${encodeURIComponent(previewId)}`;
}

export interface PendingPreviewRow {
  previewId: string;
  subjectKind: string;
  subjectRef: string;
  status: string;
}

export function projectPendingPreviews(body: unknown): PendingPreviewRow[] {
  const rows: PendingPreviewRow[] = [];
  for (const item of asList(body, ["previews"])) {
    const record = asRecord(item);
    if (typeof record.preview_id !== "string" || record.preview_id.length === 0) {
      continue;
    }
    rows.push({
      previewId: record.preview_id,
      subjectKind: typeof record.subject_kind === "string" ? record.subject_kind : "unknown",
      subjectRef: typeof record.subject_ref === "string" ? record.subject_ref : "unknown",
      status: typeof record.status === "string" ? record.status : "unknown",
    });
  }
  return rows;
}
