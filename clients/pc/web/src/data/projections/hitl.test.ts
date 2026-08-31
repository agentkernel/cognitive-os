import { describe, expect, it } from "vitest";
import { pendingPreviewsPath, projectPendingPreviews } from "./hitl";

describe("pending-previews projection (P11-T13)", () => {
  it("maps announcement rows and never copies preview_digest", () => {
    const rows = projectPendingPreviews({
      status: "ok",
      previews: [
        {
          preview_id: "prev-1",
          subject_kind: "activation",
          subject_ref: "proj-1",
          status: "pending",
          preview_digest: "must-not-land",
        },
      ],
    });
    expect(rows).toEqual([
      {
        previewId: "prev-1",
        subjectKind: "activation",
        subjectRef: "proj-1",
        status: "pending",
      },
    ]);
    expect(JSON.stringify(rows)).not.toContain("must-not-land");
    expect(JSON.stringify(rows)).not.toContain("preview_digest");
  });

  it("does not invent a preview from an empty or malformed body", () => {
    expect(projectPendingPreviews({ status: "ok", previews: [] })).toEqual([]);
    expect(projectPendingPreviews({ status: "ok" })).toEqual([]);
    expect(projectPendingPreviews(null)).toEqual([]);
    expect(projectPendingPreviews({ previews: [{ status: "pending" }] })).toEqual([]);
  });

  it("encodes subject_ref on the management read path", () => {
    expect(pendingPreviewsPath("proj-1")).toBe(
      "/management/project/v1/pending-previews?subject_ref=proj-1",
    );
  });
});
