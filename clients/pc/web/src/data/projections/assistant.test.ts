import { describe, expect, it } from "vitest";
import {
  ASSISTANT_STATUS_PATH,
  ASSISTANT_TURN_PATH,
  DRAFT_APPLY_PATH,
  assistantTurnReady,
  isProviderUnbound,
  projectAssistantChain,
  projectAssistantStatus,
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
      reply: "",
      modelId: "",
      providerRoundTrips: 0,
      chain: [],
      fetchedSources: [],
      refusedSources: [],
    });
    expect(JSON.stringify(row)).not.toContain("Approve");
    expect(ASSISTANT_TURN_PATH).toBe("/management/project/v1/assistant.turn");
    expect(ASSISTANT_STATUS_PATH).toBe("/management/project/v1/assistant.status");
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

  it("maps the P13-T03 inferred chain with typed provenance and drops unprovenanced fields", () => {
    const row = projectAssistantTurn({
      status: "ok",
      candidate_digest: "d".repeat(64),
      preview_id: "prev-2",
      object_kind: "charter",
      reply: "Here is a candidate charter.",
      model_id: "deepseek-chat",
      provider_round_trips: 1,
      chain: [
        {
          object_kind: "research-run",
          fields: {
            format: {
              value: "one page",
              provenance: { kind: "sources", sources: [{ uri: "https://example.invalid/report-format" }] },
            },
          },
        },
        {
          object_kind: "charter",
          summary: "weekly",
          fields: {
            title: { value: "Weekly report", provenance: { kind: "owner-stated" } },
            cadence: { value: { every: "week" }, provenance: { kind: "assistant-assumption" } },
            forged: { value: "no provenance" },
          },
        },
      ],
      research: {
        fetched: ["https://example.invalid/report-format"],
        refused: [{ uri: "http://example.invalid/plain", reason: "https only" }],
      },
    });
    expect(row?.reply).toBe("Here is a candidate charter.");
    expect(row?.modelId).toBe("deepseek-chat");
    expect(row?.providerRoundTrips).toBe(1);
    expect(row?.chain.map((object) => object.objectKind)).toEqual(["research-run", "charter"]);
    expect(row?.chain[0]?.fields[0]).toEqual({
      name: "format",
      value: "one page",
      provenanceKind: "sources",
      sourceUris: ["https://example.invalid/report-format"],
    });
    expect(row?.chain[1]?.summary).toBe("weekly");
    expect(row?.chain[1]?.fields.map((field) => field.name)).toEqual(["title", "cadence"]);
    expect(row?.chain[1]?.fields[1]?.value).toBe('{"every":"week"}');
    expect(row?.fetchedSources).toEqual(["https://example.invalid/report-format"]);
    expect(row?.refusedSources).toEqual(["http://example.invalid/plain"]);
    expect(projectAssistantChain("not a chain")).toEqual([]);
    expect(JSON.stringify(row)).not.toMatch(/confidence/i);
  });

  it("maps assistant.status and opens the input only on an explicit daemon ready", () => {
    expect(projectAssistantStatus({ status: "ready", chat_input: true, model_id: "deepseek-chat" })).toEqual({
      status: "ready",
      chatInput: true,
      modelId: "deepseek-chat",
      guidance: "",
      settingsRoute: "/settings",
      piDetail: "",
    });
    expect(projectAssistantStatus({ status: "ready" }).chatInput).toBe(false);
    expect(projectAssistantStatus({ status: "provider_unbound", chat_input: false, guidance: "Open Settings" })).toMatchObject({
      status: "provider_unbound",
      chatInput: false,
      guidance: "Open Settings",
    });
    expect(projectAssistantStatus({ status: "pi_unavailable", pi_detail: "Pi runtime is not configured" })).toMatchObject({
      status: "pi_unavailable",
      chatInput: false,
      piDetail: "Pi runtime is not configured",
    });
    expect(projectAssistantStatus({ status: "surprise", chat_input: true }).chatInput).toBe(false);
    expect(projectAssistantStatus(null).status).toBe("unknown");
  });

  it("recognises the daemon's provider-unbound refusal and nothing else", () => {
    expect(isProviderUnbound(409, { status: "provider_unbound", code: "ASSISTANT_PROVIDER_UNBOUND" })).toBe(true);
    expect(isProviderUnbound(409, { status: "error", code: "ASSISTANT_PROVIDER_UNBOUND" })).toBe(true);
    expect(isProviderUnbound(409, { status: "error", code: "PROJECT_CONFLICT" })).toBe(false);
    expect(isProviderUnbound(200, { status: "provider_unbound" })).toBe(false);
    expect(isProviderUnbound(403, { code: "ASSISTANT_PROVIDER_UNBOUND" })).toBe(false);
  });

  it("gates a create-page turn on non-empty, bounded, secret-free text", () => {
    expect(assistantTurnReady("   ").ok).toBe(false);
    expect(assistantTurnReady("x".repeat(4001)).ok).toBe(false);
    expect(assistantTurnReady("sk-abcdefghijklmnopqrstuvwxyz").ok).toBe(false);
    expect(assistantTurnReady("propose a weekly client report")).toEqual({ ok: true });
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
