/**
 * HITL announcement + canvas preview-detail (P12-T06 Dual Track).
 * GET pending-previews omits preview_digest (T09). Digest is only read from
 * GET preview-detail. This file never invents Confirm from the list row.
 */

import { asList, asRecord } from "../projections";

export const HITL_KEY = "opc:hitl-previews";
export const PENDING_PREVIEWS_PATH = "/management/project/v1/pending-previews";
export const PREVIEW_DETAIL_PATH = "/management/project/v1/preview-detail";
export const CONFIRM_PATH = "/management/project/v1/confirm";
export const REJECT_PATH = "/management/project/v1/preview.reject";
export const NARROW_PATH = "/management/project/v1/preview.narrow";

export function pendingPreviewsPath(subjectRef: string): string {
  return `${PENDING_PREVIEWS_PATH}?subject_ref=${encodeURIComponent(subjectRef)}`;
}

export function previewDetailPath(previewId: string): string {
  return `${PREVIEW_DETAIL_PATH}?preview_id=${encodeURIComponent(previewId)}`;
}

export function previewDetailKey(previewId: string): string {
  return `opc:preview-detail:${previewId}`;
}

/**
 * Project-center HITL canvas (T09). Today deep-links here. Not Inbox L1
 * and not an independent `#/hitl/:id` space.
 */
export function hitlCanvasPath(previewId: string, projectId?: string): string {
  const preview = `preview=${encodeURIComponent(previewId)}`;
  if (typeof projectId === "string" && projectId.length > 0) {
    return `/projects/${encodeURIComponent(projectId)}?${preview}`;
  }
  return `/projects?${preview}`;
}

export interface PendingPreviewRow {
  previewId: string;
  subjectKind: string;
  subjectRef: string;
  status: string;
}

export interface PreviewDetailRow {
  previewId: string;
  subjectKind: string;
  previewDigest: string;
  status: string;
  receiptRef: string;
  supersededBy: string;
  baseStateDigest: string;
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

export function projectPreviewDetail(body: unknown): PreviewDetailRow[] {
  const record = asRecord(body);
  if (typeof record.preview_id !== "string" || record.preview_id.length === 0) {
    return [];
  }
  return [
    {
      previewId: record.preview_id,
      subjectKind: typeof record.subject_kind === "string" ? record.subject_kind : "unknown",
      previewDigest: typeof record.preview_digest === "string" ? record.preview_digest : "",
      status: typeof record.status === "string" ? record.status : "unknown",
      receiptRef: typeof record.receipt_ref === "string" ? record.receipt_ref : "",
      supersededBy: typeof record.superseded_by === "string" ? record.superseded_by : "",
      baseStateDigest: typeof record.base_state_digest === "string" ? record.base_state_digest : "",
    },
  ];
}

/** Confirm/narrow/reject only when daemon says pending and supplies a digest. */
export function previewIsConfirmable(row: PreviewDetailRow | undefined): boolean {
  return Boolean(row && row.status === "pending" && row.previewDigest.length > 0);
}
