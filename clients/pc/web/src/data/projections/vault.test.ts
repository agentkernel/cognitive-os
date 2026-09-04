import { describe, expect, it } from "vitest";
import {
  projectVaultConflicts,
  projectVaultDocuments,
  projectVaultIndex,
  projectVaultInjectOrder,
  projectVaultLabeled,
  vaultConflictsPath,
  vaultDocumentsPath,
  vaultImportIsAuthority,
  vaultIndexPath,
  vaultLabeledPath,
  VAULT_DOCUMENTS_PATH,
  VAULT_IMPORT_PATH,
  VAULT_LABELED_PATH,
  VAULT_REBUILD_PATH,
} from "./vault";

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
    expect(vaultConflictsPath("proj-1")).toBe(
      "/management/project/v1/vault.conflicts?project_id=proj-1&caller_project_id=proj-1",
    );
  });
});

describe("vault Why this fragment + conflicts (P12-T07)", () => {
  it("projects inject_order from vault.index and does not invent layers", () => {
    expect(
      projectVaultInjectOrder({
        inject_order: ["task-contract", "fixed-decision", "sourced-excerpt", "summary", "older-narrative"],
      }),
    ).toEqual(["task-contract", "fixed-decision", "sourced-excerpt", "summary", "older-narrative"]);
    expect(projectVaultInjectOrder({ status: "ok", entries: [] })).toEqual([]);
    expect(projectVaultInjectOrder(null)).toEqual([]);
  });

  it("maps conflict rows without treating them as Project authority", () => {
    expect(
      projectVaultConflicts({
        conflicts: [
          {
            conflict_id: "c-1",
            relative_path: "notes/a.md",
            incumbent_document_id: "doc-old",
            incoming_document_id: "doc-new",
            resolution: "unresolved",
          },
        ],
      }),
    ).toEqual([
      {
        conflictId: "c-1",
        relativePath: "notes/a.md",
        incumbentDocumentId: "doc-old",
        incomingDocumentId: "doc-new",
        resolution: "unresolved",
      },
    ]);
    expect(projectVaultConflicts({ status: "ok", conflicts: [] })).toEqual([]);
  });

  it("treats import is_authority true as a lie the UI must not promote", () => {
    expect(vaultImportIsAuthority({ status: "ok", is_authority: false, document_id: "doc-1" })).toBe(
      false,
    );
    expect(vaultImportIsAuthority({ status: "ok", is_authority: true })).toBe(true);
    expect(VAULT_IMPORT_PATH).toBe("/management/project/v1/vault.import");
    expect(VAULT_REBUILD_PATH).toBe("/management/project/v1/vault.index.rebuild");
  });
});

describe("vault labeled index + document status (P13-T07)", () => {
  it("maps provenance/rights/freshness/exclusion and never treats files as authority", () => {
    const rows = projectVaultLabeled({
      status: "ok",
      is_authority: false,
      entries: [
        {
          entry_id: "ent-1",
          document_id: "doc-1",
          relative_path: "notes/owned.md",
          excerpt: "Owned excerpt",
          layer: "sourced-excerpt",
          provenance_source_uri: "owner-paste:owned",
          rights_class: "owner-owned",
          freshness: "current",
          exclusion: "included",
          exclusion_reason: "",
          untrusted_observation: false,
          is_authority: false,
        },
        {
          entry_id: "ent-2",
          document_id: "doc-2",
          relative_path: "notes/cite.md",
          excerpt: "Cite",
          layer: "sourced-excerpt",
          provenance_source_uri: "https://example.invalid/cite",
          rights_class: "citation-only",
          freshness: "current",
          exclusion: "excluded",
          exclusion_reason: "citation-only",
          untrusted_observation: true,
          is_authority: false,
        },
      ],
    });
    expect(rows[0]?.freshness).toBe("current");
    expect(rows[1]?.exclusion).toBe("excluded");
    expect(rows[1]?.untrustedObservation).toBe(true);
    expect(rows.every((row) => row.isAuthority === false)).toBe(true);
    expect(projectVaultLabeled({ status: "ok", entries: [] })).toEqual([]);
  });

  it("keeps not-indexed documents visible", () => {
    expect(
      projectVaultDocuments({
        documents: [
          {
            document_id: "doc-pending",
            relative_path: "notes/pending.md",
            provenance_source_uri: "owner-paste:pending",
            index_status: "not-indexed",
          },
        ],
      }),
    ).toEqual([
      {
        documentId: "doc-pending",
        relativePath: "notes/pending.md",
        provenanceSourceUri: "owner-paste:pending",
        indexStatus: "not-indexed",
      },
    ]);
    expect(vaultLabeledPath("proj-1")).toBe(
      `${VAULT_LABELED_PATH}?project_id=proj-1&caller_project_id=proj-1`,
    );
    expect(vaultDocumentsPath("proj-1")).toBe(
      `${VAULT_DOCUMENTS_PATH}?project_id=proj-1&caller_project_id=proj-1`,
    );
  });
});
