import { describe, expect, it } from "vitest";
import { projectVaultIndex, vaultIndexPath } from "./vault";

describe("vault index projection (P11-T13)", () => {
  it("maps daemon entries and ignores is_authority", () => {
    const rows = projectVaultIndex({
      status: "ok",
      is_authority: true,
      entries: [
        {
          entry_id: "ent-1",
          document_id: "doc-1",
          layer: "sourced excerpts",
          excerpt: "note",
        },
      ],
    });
    expect(rows).toEqual([
      {
        entryId: "ent-1",
        documentId: "doc-1",
        layer: "sourced excerpts",
        excerpt: "note",
      },
    ]);
    expect(JSON.stringify(rows)).not.toContain("is_authority");
  });

  it("does not invent a Vault file from an empty or malformed body", () => {
    expect(projectVaultIndex({ status: "ok", entries: [] })).toEqual([]);
    expect(projectVaultIndex({ status: "ok" })).toEqual([]);
    expect(projectVaultIndex(null)).toEqual([]);
    expect(projectVaultIndex({ entries: [{ excerpt: "x" }] })).toEqual([]);
  });

  it("keeps caller_project_id equal to project_id", () => {
    expect(vaultIndexPath("proj-1")).toBe(
      "/management/project/v1/vault.index?project_id=proj-1&caller_project_id=proj-1",
    );
  });
});
