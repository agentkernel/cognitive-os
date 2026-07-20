import { readFile, stat } from "node:fs/promises";
import { resolve } from "node:path";
import { authorProfile, sampleTimeline } from "@content/data/profile";
import { bodyHeadingIds, loadDiskContent, splitFrontmatter } from "@scripts/lib/content-files";
import { cognitiveOsSnapshot } from "@/data/cognitiveos";

function invariant(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

const entries = await loadDiskContent();
const articles = entries.filter((entry) => entry.frontmatter.kind !== "project");
const projects = entries.filter((entry) => entry.frontmatter.kind === "project");

invariant(articles.length === 8, `Expected 8 bilingual article files, found ${articles.length}`);
invariant(projects.length === 8, `Expected 8 bilingual project files, found ${projects.length}`);
invariant(
  articles.filter((entry) => entry.frontmatter.placeholder).length === 6,
  "Expected exactly three bilingual sample articles.",
);
invariant(
  articles.filter((entry) => entry.frontmatter.kind === "cognitiveos").length === 2,
  "Expected exactly one bilingual CognitiveOS flagship.",
);
for (const entry of articles.filter(
  (candidate) => candidate.frontmatter.kind === "cognitiveos",
)) {
  invariant(
    entry.frontmatter.hero?.src === "/images/ai/governed-trace-hero.avif",
    `${entry.filePath} must use the approved local generated hero.`,
  );
}
invariant(
  cognitiveOsSnapshot.commit === "b626e88be3b985399051e6e7624223b9cb38e7c6",
  "CognitiveOS snapshot must use the current parent commit.",
);
invariant(cognitiveOsSnapshot.conformanceVectors === 76, "Expected 76 vectors.");
invariant(cognitiveOsSnapshot.vectorsNotRun === 76, "All 76 vectors must remain not-run.");
invariant(
  cognitiveOsSnapshot.schemaIdsEqualFileNames === 56 &&
    cognitiveOsSnapshot.absoluteSchemaIds === 0,
  "All 56 schema IDs must equal file names with zero absolute IDs.",
);

const groups = new Map<string, typeof entries>();
for (const entry of entries) {
  const key = entry.frontmatter.translationKey;
  groups.set(key, [...(groups.get(key) || []), entry]);
}
for (const [translationKey, pair] of groups) {
  invariant(pair.length === 2, `${translationKey} must have exactly two locales.`);
  invariant(
    new Set(pair.map((entry) => entry.frontmatter.locale)).size === 2,
    `${translationKey} must contain zh-CN and en.`,
  );
  const [first, second] = pair;
  invariant(first.frontmatter.kind === second.frontmatter.kind, `${translationKey} kind drift.`);
  invariant(
    first.frontmatter.publishedAt === second.frontmatter.publishedAt,
    `${translationKey} publication date drift.`,
  );
  invariant(
    first.frontmatter.placeholder === second.frontmatter.placeholder,
    `${translationKey} placeholder drift.`,
  );
  invariant(
    first.frontmatter.pairingSnapshot === second.frontmatter.pairingSnapshot,
    `${translationKey} pairing snapshot drift.`,
  );
  invariant(
    JSON.stringify(first.frontmatter.anchors) === JSON.stringify(second.frontmatter.anchors),
    `${translationKey} anchor drift.`,
  );
}

for (const entry of entries) {
  const ids = bodyHeadingIds(entry.source);
  invariant(
    JSON.stringify(ids) === JSON.stringify(entry.frontmatter.anchors),
    `${entry.filePath} heading IDs must exactly match frontmatter anchors.`,
  );

  if (entry.frontmatter.kind === "project") {
    invariant(entry.frontmatter.placeholder, `${entry.filePath} project must remain placeholder.`);
    invariant(
      JSON.stringify(ids) === JSON.stringify(entry.frontmatter.structure),
      `${entry.filePath} project body must use Problem/Constraints/Approach/Outcome/Reflection.`,
    );
  }

  const body = splitFrontmatter(entry.source).body;
  const claimPatterns = [
    /CognitiveOS (?:has passed|is production-ready|is implemented)/i,
    /CognitiveOS 已(?:经)?(?:通过|实现|生产就绪)/,
    /\b(?:revenue|users served|client was|award-winning)\b/i,
    /(?:营收|服务用户|真实客户|获奖项目)/,
  ];
  for (const line of body.split(/\r?\n/)) {
    const hasClaim = claimPatterns.some((pattern) => pattern.test(line));
    const hasNegation =
      /\b(?:not|no |never|cannot|unsafe|without|does not|do not)\b/i.test(line) ||
      /(?:不|没有|禁止|不可|未|无证据)/.test(line);
    invariant(
      !hasClaim || hasNegation,
      `Unsupported public claim in ${entry.filePath}: ${line.trim()}`,
    );
  }
}

invariant(authorProfile.placeholder, "Author profile must remain placeholder.");
invariant(
  sampleTimeline.every((entry) => entry.placeholder),
  "Every timeline entry must remain placeholder.",
);

for (const fileName of ["cognitiveos-sourcebook.zh-CN.md", "cognitiveos-sourcebook.en.md"]) {
  const path = resolve(process.cwd(), "content", "research", fileName);
  const source = await readFile(path, "utf8");
  for (let index = 1; index <= 18; index += 1) {
    invariant(
      source.includes(`FACT-COS-${String(index).padStart(3, "0")}`),
      `${fileName} is missing FACT-COS-${String(index).padStart(3, "0")}.`,
    );
  }
  for (const term of [
    "OperationDescriptor",
    "AuthorizationCapability",
    "ContextView",
    "OUTCOME_UNKNOWN",
    "CANDIDATE_COMPLETE",
    "GOBJ-LEGACY-METADATA-001",
    "GOBJ-LEGACY-STRONGREF-001",
    "Verification",
    "Acceptance",
    "research snapshot",
  ]) {
    invariant(source.toLowerCase().includes(term.toLowerCase()), `${fileName} missing ${term}.`);
  }
  invariant(
    source.includes("b626e88be3b985399051e6e7624223b9cb38e7c6"),
    `${fileName} must identify the current parent commit.`,
  );
  invariant(!source.includes("4a1c6c"), `${fileName} contains the stale snapshot commit.`);
  invariant(!source.includes("74 / 74"), `${fileName} contains the stale vector count.`);
  invariant(!source.includes("13 absolute"), `${fileName} contains the closed absolute-ID finding.`);
}

for (const baseName of ["governed-trace-hero", "orthogonal-lifecycles-hero"]) {
  for (const extension of ["avif", "webp"]) {
    const asset = resolve(
      process.cwd(),
      "public",
      "images",
      "ai",
      `${baseName}.${extension}`,
    );
    invariant((await stat(asset)).size > 10_000, `${asset} is missing or unexpectedly small.`);
  }
}
invariant(
  (
    await stat(
      resolve(process.cwd(), "public", "images", "og", "system-notebook.png"),
    )
  ).size > 1_000,
  "The local Open Graph PNG is missing or unexpectedly small.",
);
const assetProvenance = await readFile(
  resolve(process.cwd(), "ASSET_PROVENANCE.md"),
  "utf8",
);
for (const requiredAsset of [
  "governed-trace-hero",
  "orthogonal-lifecycles-hero",
]) {
  invariant(
    assetProvenance.includes(requiredAsset),
    `ASSET_PROVENANCE.md is missing ${requiredAsset}.`,
  );
}

console.log(
  `Content checks passed: ${articles.length / 2} paired articles, ${projects.length / 2} paired projects, 18 traced facts per sourcebook.`,
);
