import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { parse as parseYaml } from "yaml";
import {
  articleFrontmatterSchema,
  projectFrontmatterSchema,
  type ArticleFrontmatter,
  type ProjectFrontmatter,
} from "@/lib/content/schema";

const contentFiles = [
  ["cognitiveos:zh-CN:verifiable-agent-actions", "cognitiveos/verifiable-agent-actions/zh-CN.mdx"],
  ["cognitiveos:en:verifiable-agent-actions", "cognitiveos/verifiable-agent-actions/en.mdx"],
  ["article:zh-CN:context-budget-notes", "articles/context-budget-notes/zh-CN.mdx"],
  ["article:en:context-budget-notes", "articles/context-budget-notes/en.mdx"],
  ["article:zh-CN:designing-failure-semantics", "articles/designing-failure-semantics/zh-CN.mdx"],
  ["article:en:designing-failure-semantics", "articles/designing-failure-semantics/en.mdx"],
  ["article:zh-CN:testing-bilingual-content", "articles/testing-bilingual-content/zh-CN.mdx"],
  ["article:en:testing-bilingual-content", "articles/testing-bilingual-content/en.mdx"],
  ["project:zh-CN:accessible-docs-migration", "projects/accessible-docs-migration/zh-CN.mdx"],
  ["project:en:accessible-docs-migration", "projects/accessible-docs-migration/en.mdx"],
  ["project:zh-CN:evidence-first-cli", "projects/evidence-first-cli/zh-CN.mdx"],
  ["project:en:evidence-first-cli", "projects/evidence-first-cli/en.mdx"],
  ["project:zh-CN:governed-context-prototype", "projects/governed-context-prototype/zh-CN.mdx"],
  ["project:en:governed-context-prototype", "projects/governed-context-prototype/en.mdx"],
  ["project:zh-CN:incident-replay-notebook", "projects/incident-replay-notebook/zh-CN.mdx"],
  ["project:en:incident-replay-notebook", "projects/incident-replay-notebook/en.mdx"],
] as const;

export type ContentId = (typeof contentFiles)[number][0];

export type ArticleSummary = {
  id: ContentId;
  frontmatter: ArticleFrontmatter;
};

export type ProjectSummary = {
  id: ContentId;
  frontmatter: ProjectFrontmatter;
};

export type ContentSummary = ArticleSummary | ProjectSummary;

function parseFrontmatter(source: string): unknown {
  const match = source.match(/^---\r?\n([\s\S]*?)\r?\n---\r?\n/);
  if (!match) {
    throw new Error("Missing YAML frontmatter");
  }
  return parseYaml(match[1]);
}

function loadManifest(): ContentSummary[] {
  const contentRoot = resolve(process.cwd(), "content");
  return contentFiles.map(([id, relativePath]) => {
    const source = readFileSync(resolve(contentRoot, relativePath), "utf8");
    const data = parseFrontmatter(source);
    const kind =
      typeof data === "object" && data !== null && "kind" in data
        ? (data as { kind?: unknown }).kind
        : undefined;
    const frontmatter =
      kind === "project"
        ? projectFrontmatterSchema.parse(data)
        : articleFrontmatterSchema.parse(data);
    return { id, frontmatter } as ContentSummary;
  });
}

export const contentManifest = loadManifest();
