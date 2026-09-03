/**
 * Project group chat projections (P13-T06). The right rail inside a Project is
 * the group (Owner / manager / Members) layered against the Personal Assistant
 * conversation. The client only posts the Owner's message with its parsed
 * mention; the daemon routes it (`@manager` → PlanRevision candidate →
 * `plan-revision` preview; `@member` → task-revision candidate bounded to that
 * Member's Task; un-addressed → manager-default briefing) and enforces the
 * speech rules by record kind. A preview id is an announcement: chat has no
 * Approve, and Confirm stays on the Projects canvas. `@` chips only edit the
 * unsent draft. Secret-shaped drafts never leave the browser (Settings
 * pointer); the daemon refuses them a second time with `CHAT_SECRET_SHAPED_REFUSED`.
 */

import { asList, asRecord } from "../projections";
import { containsSecretMaterial } from "../../policy";

export const CHAT_POST_PATH = "/management/project/v1/chat.post";
export const CHAT_THREAD_PATH = "/management/project/v1/chat.thread";
export const CHAT_THREAD_LIMIT = 32;
export const CHAT_BODY_LIMIT = 4000;
export const CHAT_SECRET_REFUSED_CODE = "CHAT_SECRET_SHAPED_REFUSED";
export const CHAT_APPROVE_FORBIDDEN_CODE = "CHAT_APPROVE_FORBIDDEN";
/** Mirrors the daemon's `settings_route` for the SecretStore takeover pointer. */
export const CHAT_SETTINGS_ROUTE = "/settings";

export type ChatMention = "none" | "manager" | "member";

export function chatThreadPath(projectId: string, limit = CHAT_THREAD_LIMIT): string {
  return `${CHAT_THREAD_PATH}?project_id=${encodeURIComponent(projectId)}&limit=${limit}`;
}

export function chatThreadKey(projectId: string): string {
  return `opc:project-chat:${projectId}`;
}

export interface ChatParticipantRow {
  role: "owner" | "manager" | "member" | "unknown";
  employeeId: string;
  handle: string;
  state: string;
  stageIds: string[];
}

export interface ChatThreadRowView {
  rowId: string;
  author: "owner" | "manager" | "member" | "unknown";
  employeeId: string;
  kind: string;
  body: string;
  createdAt: string;
  turnId: string;
  mention: string;
  routing: string;
  targetEmployeeId: string;
  targetStageId: string;
  candidateKind: string;
  candidateDigest: string;
  previewId: string;
  replyReason: string;
  receiptRef: string;
  appliedRef: string;
}

export interface ChatThreadView {
  projectId: string;
  rows: ChatThreadRowView[];
  participants: ChatParticipantRow[];
  truncated: boolean;
}

export interface ChatPostReplyView {
  recordId: string;
  employeeId: string;
  role: string;
  kind: string;
  body: string;
  reason: string;
}

export interface ChatPostView {
  turnId: string;
  routing: string;
  candidateKind: string;
  candidateDigest: string;
  previewId: string;
  targetEmployeeId: string;
  targetStageId: string;
  reply: ChatPostReplyView | undefined;
  replyReason: string;
}

function text(record: Record<string, unknown>, key: string): string {
  const value = record[key];
  if (typeof value === "string") {
    return value;
  }
  if (typeof value === "number") {
    return String(value);
  }
  return "";
}

function role(value: unknown): ChatParticipantRow["role"] {
  return value === "owner" || value === "manager" || value === "member" ? value : "unknown";
}

export function projectChatThread(body: unknown): ChatThreadView {
  const record = asRecord(body);
  const participants: ChatParticipantRow[] = [];
  for (const item of asList(body, ["participants"])) {
    const row = asRecord(item);
    const handle = text(row, "handle");
    if (handle.length === 0) {
      continue;
    }
    participants.push({
      role: role(row.role),
      employeeId: text(row, "employee_id"),
      handle,
      state: text(row, "state") || "unknown",
      stageIds: Array.isArray(row.stage_ids)
        ? row.stage_ids.filter((id): id is string => typeof id === "string")
        : [],
    });
  }
  const rows: ChatThreadRowView[] = [];
  for (const item of asList(body, ["rows"])) {
    const row = asRecord(item);
    const rowId = text(row, "row_id");
    if (rowId.length === 0) {
      continue;
    }
    rows.push({
      rowId,
      author: role(row.author),
      employeeId: text(row, "employee_id"),
      kind: text(row, "kind") || "unknown",
      body: text(row, "body"),
      createdAt: text(row, "created_at") || "unknown",
      turnId: text(row, "turn_id"),
      mention: text(row, "mention"),
      routing: text(row, "routing"),
      targetEmployeeId: text(row, "target_employee_id"),
      targetStageId: text(row, "target_stage_id"),
      candidateKind: text(row, "candidate_kind"),
      candidateDigest: text(row, "candidate_digest"),
      previewId: text(row, "preview_id"),
      replyReason: text(row, "reply_reason"),
      receiptRef: text(row, "receipt_ref"),
      appliedRef: text(row, "applied_ref"),
    });
  }
  return {
    projectId: text(record, "project_id"),
    rows,
    participants,
    truncated: record.truncated === true,
  };
}

export function projectChatPost(body: unknown): ChatPostView | undefined {
  const record = asRecord(body);
  const turnId = text(record, "turn_id");
  if (turnId.length === 0) {
    return undefined;
  }
  const replyRecord = asRecord(record.reply);
  const reply =
    text(replyRecord, "record_id").length > 0
      ? {
          recordId: text(replyRecord, "record_id"),
          employeeId: text(replyRecord, "employee_id"),
          role: text(replyRecord, "role"),
          kind: text(replyRecord, "kind"),
          body: text(replyRecord, "body"),
          reason: text(replyRecord, "reason"),
        }
      : undefined;
  return {
    turnId,
    routing: text(record, "routing") || "unknown",
    candidateKind: text(record, "candidate_kind"),
    candidateDigest: text(record, "candidate_digest"),
    previewId: text(record, "preview_id"),
    targetEmployeeId: text(record, "target_employee_id"),
    targetStageId: text(record, "target_stage_id"),
    reply,
    replyReason: text(record, "reply_reason"),
  };
}

/** True when the daemon refused the message as secret-shaped (SecretStore takeover pointer). */
export function isChatSecretRefused(status: number, body: unknown): boolean {
  return status === 422 && asRecord(body).code === CHAT_SECRET_REFUSED_CODE;
}

/**
 * `@` inserts only into the unsent draft. Pure text edit: no network, no
 * state outside the draft. Adds one trailing space so the message continues.
 */
export function insertMention(draft: string, handle: string): string {
  const mention = `@${handle}`;
  if (draft.trim().length === 0) {
    return `${mention} `;
  }
  if (draft.startsWith(mention)) {
    return draft;
  }
  const separator = /\s$/.test(draft) ? "" : " ";
  return `${draft}${separator}${mention} `;
}

export interface ParsedMention {
  mention: ChatMention;
  handle: string;
  targetEmployeeId: string;
  /** Message with the leading mention token stripped when a Member is addressed. */
  known: boolean;
}

/**
 * Parse the leading `@handle` of a draft against the participant chips.
 * `@manager` routes to the manager; a Member's slot handle routes to that
 * Member; an unknown handle stays conversational and is reported so the
 * composer can say so instead of guessing.
 */
export function parseLeadingMention(draft: string, participants: ChatParticipantRow[]): ParsedMention {
  const match = /^@([A-Za-z0-9_-]+)/.exec(draft.trimStart());
  if (!match) {
    return { mention: "none", handle: "", targetEmployeeId: "", known: true };
  }
  const handle = match[1] ?? "";
  if (handle.toLowerCase() === "manager") {
    const manager = participants.find((p) => p.role === "manager");
    return {
      mention: "manager",
      handle: "manager",
      targetEmployeeId: manager?.employeeId ?? "",
      known: true,
    };
  }
  const member = participants.find(
    (p) => p.role === "member" && p.handle.toLowerCase() === handle.toLowerCase(),
  );
  if (member) {
    return { mention: "member", handle: member.handle, targetEmployeeId: member.employeeId, known: true };
  }
  return { mention: "none", handle, targetEmployeeId: "", known: false };
}

/** Owner draft gate: non-empty, bounded, secret-free. Never posts a key. */
export function chatDraftReady(draft: string): { ok: true } | { ok: false; reason: string; settings?: boolean } {
  const trimmed = draft.trim();
  if (trimmed.length === 0) {
    return { ok: false, reason: "type a message before sending." };
  }
  if (trimmed.length > CHAT_BODY_LIMIT) {
    return { ok: false, reason: `message is too long for one turn (${CHAT_BODY_LIMIT} characters).` };
  }
  if (containsSecretMaterial(trimmed)) {
    return {
      ok: false,
      reason:
        "secret-shaped material is not posted to the group chat. Keys belong in Settings (SecretStore takeover); nothing was sent.",
      settings: true,
    };
  }
  return { ok: true };
}

export interface PlanStageLine {
  stageId: string;
  title: string;
  responsibleSlot: string;
  objective: string;
}

/**
 * One stage per line: `stage_id | title | responsible_slot | objective`.
 * Lines that do not carry all four cells are reported, never guessed.
 */
export function parsePlanStageLines(
  source: string,
): { ok: true; stages: PlanStageLine[] } | { ok: false; reason: string } {
  const stages: PlanStageLine[] = [];
  const lines = source
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
  if (lines.length === 0) {
    return { ok: false, reason: "a plan revision needs at least one stage line." };
  }
  for (const line of lines) {
    const cells = line.split("|").map((cell) => cell.trim());
    if (cells.length !== 4 || cells.some((cell) => cell.length === 0)) {
      return {
        ok: false,
        reason: `stage line "${line}" must read: stage_id | title | responsible_slot | objective`,
      };
    }
    const [stageId, title, responsibleSlot, objective] = cells as [string, string, string, string];
    if (stages.some((stage) => stage.stageId === stageId)) {
      return { ok: false, reason: `stage id ${stageId} is repeated.` };
    }
    stages.push({ stageId, title, responsibleSlot, objective });
  }
  return { ok: true, stages };
}

export function planRevisionProposal(stages: PlanStageLine[]): Record<string, unknown> {
  return {
    kind: "plan-revision",
    stages: stages.map((stage) => ({
      stage_id: stage.stageId,
      title: stage.title,
      objective: stage.objective,
      responsible_slot: stage.responsibleSlot,
    })),
  };
}
