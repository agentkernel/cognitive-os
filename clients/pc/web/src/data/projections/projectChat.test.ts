import { describe, expect, it } from "vitest";
import { isKnownRoute } from "../normalize";
import {
  CHAT_BODY_LIMIT,
  chatDraftReady,
  chatThreadPath,
  insertMention,
  isChatSecretRefused,
  parseLeadingMention,
  parsePlanStageLines,
  planRevisionProposal,
  projectChatPost,
  projectChatThread,
  type ChatParticipantRow,
} from "./projectChat";

const PARTICIPANTS: ChatParticipantRow[] = [
  { role: "owner", employeeId: "", handle: "owner", state: "owner", stageIds: [] },
  { role: "manager", employeeId: "emp-1", handle: "manager", state: "seated", stageIds: ["s1"] },
  { role: "member", employeeId: "emp-2", handle: "researcher", state: "seated", stageIds: ["s2"] },
];

describe("P13-T06 project chat projections", () => {
  it("whitelists the management chat routes and never a task-channel alias", () => {
    expect(isKnownRoute("POST", "/management/project/v1/chat.post")).toBe(true);
    expect(isKnownRoute("GET", "/management/project/v1/chat.thread")).toBe(true);
    expect(isKnownRoute("POST", "/task/project/v1/chat.post")).toBe(false);
    expect(isKnownRoute("GET", "/task/project/v1/chat.thread")).toBe(false);
    expect(isKnownRoute("POST", "/management/project/v1/chat.approve")).toBe(false);
    expect(chatThreadPath("proj/1")).toBe(
      "/management/project/v1/chat.thread?project_id=proj%2F1&limit=32",
    );
  });

  it("projects a thread without inventing rows, digests, or roles", () => {
    const thread = projectChatThread({
      status: "ok",
      project_id: "proj-1",
      truncated: true,
      participants: [
        { role: "owner", handle: "owner", state: "owner", stage_ids: [] },
        { role: "manager", employee_id: "emp-1", handle: "manager", state: "seated", stage_ids: ["s1"] },
        { role: "weird", employee_id: "emp-9", handle: "ghost", state: "seated", stage_ids: [7] },
        { role: "member", employee_id: "emp-3", handle: "", state: "seated" },
      ],
      rows: [
        {
          row_id: "turn-1",
          author: "owner",
          kind: "owner-message",
          body: "@manager add a review ring",
          created_at: 100,
          turn_id: "turn-1",
          mention: "manager",
          routing: "manager-plan-revision",
          candidate_kind: "plan-revision",
          candidate_digest: "c".repeat(64),
          preview_id: "prev-1",
          reply_reason: "manager-default",
        },
        { row_id: "conv-1", author: "manager", employee_id: "emp-1", kind: "announce", body: "Observed now", created_at: 100 },
        { author: "member", kind: "deliverable", body: "no row id" },
      ],
    });
    expect(thread.projectId).toBe("proj-1");
    expect(thread.truncated).toBe(true);
    expect(thread.participants.map((p) => p.handle)).toEqual(["owner", "manager", "ghost"]);
    expect(thread.participants[2]?.role).toBe("unknown");
    expect(thread.participants[2]?.stageIds).toEqual([]);
    expect(thread.rows).toHaveLength(2);
    expect(thread.rows[0]).toMatchObject({
      author: "owner",
      routing: "manager-plan-revision",
      candidateKind: "plan-revision",
      previewId: "prev-1",
      createdAt: "100",
    });
    expect(thread.rows[1]?.author).toBe("manager");
    expect(JSON.stringify(thread)).not.toContain("preview_digest");
  });

  it("projects a post outcome only when the daemon returned a turn id", () => {
    expect(projectChatPost({ status: "ok" })).toBeUndefined();
    const posted = projectChatPost({
      status: "ok",
      turn_id: "turn-2",
      routing: "member-task-revision",
      candidate_kind: "task-revision",
      candidate_digest: "d".repeat(64),
      preview_id: "prev-2",
      target_employee_id: "emp-2",
      target_stage_id: "s2",
      reply: null,
      reply_reason: "member-mentioned",
    });
    expect(posted).toMatchObject({
      turnId: "turn-2",
      routing: "member-task-revision",
      targetStageId: "s2",
      replyReason: "member-mentioned",
    });
    expect(posted?.reply).toBeUndefined();
    const spoken = projectChatPost({
      turn_id: "turn-3",
      routing: "manager-briefing",
      reply: { record_id: "conv-2", employee_id: "emp-1", role: "manager", kind: "announce", body: "Observed now", reason: "manager-default" },
    });
    expect(spoken?.reply?.kind).toBe("announce");
  });

  it("recognises the daemon's secret refusal only by code and status", () => {
    expect(isChatSecretRefused(422, { status: "error", code: "CHAT_SECRET_SHAPED_REFUSED" })).toBe(true);
    expect(isChatSecretRefused(422, { status: "error", code: "PROJECT_INVALID" })).toBe(false);
    expect(isChatSecretRefused(200, { code: "CHAT_SECRET_SHAPED_REFUSED" })).toBe(false);
  });

  it("@ inserts only into the unsent draft and never twice at the head", () => {
    expect(insertMention("", "manager")).toBe("@manager ");
    expect(insertMention("   ", "researcher")).toBe("@researcher ");
    expect(insertMention("@manager status?", "manager")).toBe("@manager status?");
    expect(insertMention("please", "researcher")).toBe("please @researcher ");
    expect(insertMention("please ", "researcher")).toBe("please @researcher ");
  });

  it("parses the leading mention against the participant chips", () => {
    expect(parseLeadingMention("where are we?", PARTICIPANTS)).toEqual({
      mention: "none",
      handle: "",
      targetEmployeeId: "",
      known: true,
    });
    expect(parseLeadingMention("  @Manager status", PARTICIPANTS)).toEqual({
      mention: "manager",
      handle: "manager",
      targetEmployeeId: "emp-1",
      known: true,
    });
    expect(parseLeadingMention("@researcher focus", PARTICIPANTS)).toEqual({
      mention: "member",
      handle: "researcher",
      targetEmployeeId: "emp-2",
      known: true,
    });
    expect(parseLeadingMention("@owner hi", PARTICIPANTS)).toEqual({
      mention: "none",
      handle: "owner",
      targetEmployeeId: "",
      known: false,
    });
    expect(parseLeadingMention("@nobody hi", PARTICIPANTS).known).toBe(false);
    // Without a seated manager the manager mention still routes; the daemon
    // reports `no-current-manager` rather than the client guessing.
    expect(parseLeadingMention("@manager plan", [PARTICIPANTS[0]!]).targetEmployeeId).toBe("");
  });

  it("gates the draft: empty, oversize, and secret-shaped never post", () => {
    expect(chatDraftReady("   ")).toMatchObject({ ok: false });
    expect(chatDraftReady("x".repeat(CHAT_BODY_LIMIT + 1))).toMatchObject({ ok: false });
    const secret = chatDraftReady("use sk-abcdefghijklmnopqrstuvwxyz please");
    expect(secret).toMatchObject({ ok: false, settings: true });
    expect(chatDraftReady("api_key=whatever")).toMatchObject({ ok: false, settings: true });
    expect(chatDraftReady("@manager where are we on the task-revision?")).toEqual({ ok: true });
  });

  it("parses stage lines strictly and builds a closed plan-revision proposal", () => {
    expect(parsePlanStageLines("")).toMatchObject({ ok: false });
    expect(parsePlanStageLines("s1 | Manage | manager")).toMatchObject({ ok: false });
    expect(parsePlanStageLines("s1 | Manage | manager | coordinate\ns1 | Again | member | x")).toMatchObject({
      ok: false,
    });
    const parsed = parsePlanStageLines(
      "s1 | Manage | manager | coordinate the weekly report\n\ns2 | Research | researcher | collect sources",
    );
    expect(parsed.ok).toBe(true);
    if (parsed.ok) {
      expect(planRevisionProposal(parsed.stages)).toEqual({
        kind: "plan-revision",
        stages: [
          { stage_id: "s1", title: "Manage", objective: "coordinate the weekly report", responsible_slot: "manager" },
          { stage_id: "s2", title: "Research", objective: "collect sources", responsible_slot: "researcher" },
        ],
      });
    }
  });
});
