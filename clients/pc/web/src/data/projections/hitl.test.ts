import { describe, expect, it } from "vitest";
import {
  hitlCanvasPath,
  pendingPreviewsPath,
  previewDetailPath,
  previewIsConfirmable,
  projectPendingPreviews,
  projectPreviewDetail,
} from "./hitl";

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

  it("deep-links Today into the Projects canvas, never an Inbox or #/hitl space", () => {
    expect(hitlCanvasPath("prev-1")).toBe("/projects?preview=prev-1");
    expect(hitlCanvasPath("prev-1", "proj-1")).toBe("/projects/proj-1?preview=prev-1");
    expect(hitlCanvasPath("prev-1", "proj-1")).not.toMatch(/hitl|inbox|team/i);
  });
});

describe("preview-detail projection (P12-T06)", () => {
  it("maps digest from preview-detail and never invents one", () => {
    expect(
      projectPreviewDetail({
        preview_id: "prev-1",
        subject_kind: "activation",
        preview_digest: "digest-1",
        status: "pending",
      }),
    ).toEqual([
      {
        previewId: "prev-1",
        subjectKind: "activation",
        previewDigest: "digest-1",
        status: "pending",
        receiptRef: "",
        supersededBy: "",
        baseStateDigest: "",
      },
    ]);
    expect(projectPreviewDetail({ status: "ok" })).toEqual([]);
    expect(previewDetailPath("prev-1")).toBe(
      "/management/project/v1/preview-detail?preview_id=prev-1",
    );
  });

  it("treats missing digest or non-pending as not confirmable", () => {
    expect(
      previewIsConfirmable({
        previewId: "prev-1",
        subjectKind: "activation",
        previewDigest: "",
        status: "pending",
        receiptRef: "",
        supersededBy: "",
        baseStateDigest: "",
      }),
    ).toBe(false);
    expect(
      previewIsConfirmable({
        previewId: "prev-1",
        subjectKind: "activation",
        previewDigest: "digest-1",
        status: "stale",
        receiptRef: "",
        supersededBy: "",
        baseStateDigest: "",
      }),
    ).toBe(false);
    expect(
      previewIsConfirmable({
        previewId: "prev-1",
        subjectKind: "activation",
        previewDigest: "digest-1",
        status: "pending",
        receiptRef: "",
        supersededBy: "",
        baseStateDigest: "",
      }),
    ).toBe(true);
    expect(previewIsConfirmable(undefined)).toBe(false);
  });
});
