import { describe, expect, it } from "vitest";
import {
  ASSISTANT_TURN_PATH,
  DRAFT_APPLY_PATH,
  projectAssistantTurn,
  projectDraftApply,
  railWriteReady,
} from "./assistant";

describe("P12-T09 rail write projection", () => {
  it("maps assistant.turn candidate digest and never invents Approve", () => {
    const row = projectAssistantTurn({
      status: "ok",
      candidate_id: "cand-1",
      candidate_digest: "digest-1",
      preview_id: "prev-1",
      object_kind: "charter",
      installed_agent: false,
    });
    expect(row).toEqual({
      candidateId: "cand-1",
      candidateDigest: "digest-1",
      previewId: "prev-1",
      objectKind: "charter",
    });
    expect(JSON.stringify(row)).not.toContain("Approve");
    expect(ASSISTANT_TURN_PATH).toBe("/management/project/v1/assistant.turn");
    expect(DRAFT_APPLY_PATH).toBe("/management/project/v1/draft.apply");
  });

  it("does not invent a candidate from an empty body", () => {
    expect(projectAssistantTurn({ status: "ok" })).toBeUndefined();
    expect(projectAssistantTurn(null)).toBeUndefined();
    expect(projectDraftApply({ status: "ok" })).toBeUndefined();
  });

  it("maps draft.apply payload digest and numeric new_base_seq", () => {
    expect(
      projectDraftApply({ status: "ok", new_base_seq: 1, payload_digest: "digest-1" }),
    ).toEqual({ newBaseSeq: "1", payloadDigest: "digest-1" });
  });

  it("refuses empty identity, non-integer seq, empty text, and secret-shaped paste", () => {
    expect(railWriteReady({ draftId: "", baseSeq: "0", text: "charter" }).ok).toBe(false);
    expect(railWriteReady({ draftId: "draft-1", baseSeq: "", text: "charter" }).ok).toBe(false);
    expect(railWriteReady({ draftId: "draft-1", baseSeq: "0", text: "   " }).ok).toBe(false);
    expect(
      railWriteReady({ draftId: "draft-1", baseSeq: "0", text: "sk-secret-material-here" }).ok,
    ).toBe(false);
    expect(railWriteReady({ draftId: "draft-1", baseSeq: "0", text: "charter body" })).toEqual({
      ok: true,
    });
  });
});
