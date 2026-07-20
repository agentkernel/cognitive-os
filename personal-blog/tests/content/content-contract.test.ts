import { describe, expect, it } from "vitest";
import { bodyHeadingIds, loadDiskContent } from "@scripts/lib/content-files";
import { cognitiveOsSnapshot } from "@/data/cognitiveos";

describe("bilingual content contract", () => {
  it("pins the current M1 repository snapshot without promoting evidence status", () => {
    expect(cognitiveOsSnapshot).toMatchObject({
      commit: "b626e88be3b985399051e6e7624223b9cb38e7c6",
      milestone: "M1 in-progress",
      contractLayerBatch: "delivered",
      requirementsSpecified: 273,
      registeredErrorCodes: 55,
      schemaFiles: 56,
      schemaIdsEqualFileNames: 56,
      absoluteSchemaIds: 0,
      conformanceVectors: 76,
      implementationProvidedRequirements: 0,
      behaviorExecuted: 0,
      conformantProfiles: 0,
      vectorsNotRun: 76,
      f003LegacyNegativeVectors: 2,
    });
  });

  it("validates every frontmatter document and exact content count", async () => {
    const entries = await loadDiskContent();
    const articles = entries.filter((entry) => entry.frontmatter.kind !== "project");
    const projects = entries.filter((entry) => entry.frontmatter.kind === "project");

    expect(articles).toHaveLength(8);
    expect(projects).toHaveLength(8);
    expect(articles.filter((entry) => entry.frontmatter.placeholder)).toHaveLength(6);
    expect(projects.every((entry) => entry.frontmatter.placeholder)).toBe(true);
  });

  it("pairs zh-CN and en entries without anchor drift", async () => {
    const entries = await loadDiskContent();
    const grouped = new Map<string, typeof entries>();
    for (const entry of entries) {
      const key = entry.frontmatter.translationKey;
      grouped.set(key, [...(grouped.get(key) || []), entry]);
    }

    for (const [key, pair] of grouped) {
      expect(pair, key).toHaveLength(2);
      expect(new Set(pair.map((entry) => entry.frontmatter.locale)), key).toEqual(
        new Set(["zh-CN", "en"]),
      );
      expect(pair[0].frontmatter.kind, key).toBe(pair[1].frontmatter.kind);
      expect(pair[0].frontmatter.status, key).toBe(pair[1].frontmatter.status);
      expect(pair[0].frontmatter.placeholder, key).toBe(
        pair[1].frontmatter.placeholder,
      );
      if (
        pair[0].frontmatter.kind !== "project" &&
        pair[1].frontmatter.kind !== "project"
      ) {
        expect(pair[0].frontmatter.featured, key).toBe(
          pair[1].frontmatter.featured,
        );
      }
      expect(pair[0].frontmatter.updatedAt, key).toBe(
        pair[1].frontmatter.updatedAt,
      );
      expect(pair[0].frontmatter.pairingSnapshot, key).toBe(
        pair[1].frontmatter.pairingSnapshot,
      );
      expect(pair[0].frontmatter.anchors, key).toEqual(pair[1].frontmatter.anchors);
    }
  });

  it("keeps localized route identities unique", async () => {
    const entries = await loadDiskContent();
    const identities = entries.map(
      (entry) =>
        `${entry.frontmatter.kind}:${entry.frontmatter.locale}:${entry.frontmatter.slug}`,
    );

    expect(new Set(identities).size).toBe(identities.length);
  });

  it("keeps explicit body anchors synchronized with frontmatter", async () => {
    const entries = await loadDiskContent();
    for (const entry of entries) {
      expect(bodyHeadingIds(entry.source), entry.filePath).toEqual(
        entry.frontmatter.anchors,
      );
    }
  });

  it("enforces project Problem/Constraints/Approach/Outcome/Reflection structure", async () => {
    const entries = await loadDiskContent();
    for (const entry of entries) {
      if (entry.frontmatter.kind !== "project") {
        continue;
      }
      expect(entry.frontmatter.structure).toEqual([
        "problem",
        "constraints",
        "approach",
        "outcome",
        "reflection",
      ]);
    }
  });
});
