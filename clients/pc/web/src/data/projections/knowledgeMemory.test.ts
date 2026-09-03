import { describe, expect, it } from "vitest";
import {
  CHAT_AUTO_ADMIT_REQUIRES_BACKEND,
  MEMORY_AUTO_ADMIT_PATH,
  MEMORY_PROMOTE_CONFIRM_PATH,
  MEMORY_PROMOTE_REQUEST_PATH,
  memoryPromotesPath,
  projectMemoryPromotes,
  promotePreviewUncopied,
} from "./knowledgeMemory";

describe("P13-T07 Knowledge Memory projections", () => {
  it("maps promote rows and treats pending as not copied", () => {
    const rows = projectMemoryPromotes({
      status: "ok",
      promotes: [
        {
          promote_id: "prm-1",
          memory_id: "mem-1",
          from_project_id: "proj-a",
          to_project_id: "proj-b",
          preview_digest: "abc",
          status: "pending",
        },
      ],
    });
    expect(rows).toEqual([
      {
        promoteId: "prm-1",
        memoryId: "mem-1",
        fromProjectId: "proj-a",
        toProjectId: "proj-b",
        previewDigest: "abc",
        status: "pending",
        promotedMemoryId: undefined,
      },
    ]);
    expect(promotePreviewUncopied(rows, "prm-1")).toBe(true);
    expect(projectMemoryPromotes({ status: "ok", promotes: [] })).toEqual([]);
    expect(projectMemoryPromotes(null)).toEqual([]);
  });

  it("keeps promote and auto-admit on management HTTP", () => {
    expect(memoryPromotesPath("proj-1")).toBe(
      "/management/resource/v1/memory/promotes?project_id=proj-1",
    );
    expect(MEMORY_PROMOTE_REQUEST_PATH).toBe("/management/resource/v1/memory/promote.request");
    expect(MEMORY_PROMOTE_CONFIRM_PATH).toBe("/management/resource/v1/memory/promote.confirm");
    expect(MEMORY_AUTO_ADMIT_PATH).toBe("/management/resource/v1/memory/auto-admit.chat");
    expect(CHAT_AUTO_ADMIT_REQUIRES_BACKEND).toMatch(/Requires-backend/);
    expect(CHAT_AUTO_ADMIT_REQUIRES_BACKEND).not.toMatch(/\bAdmit\b/);
    expect(CHAT_AUTO_ADMIT_REQUIRES_BACKEND).toMatch(/No admission control/);
  });
});
