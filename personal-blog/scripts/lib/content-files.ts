import { readdir, readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { parse as parseYaml } from "yaml";
import {
  articleFrontmatterSchema,
  projectFrontmatterSchema,
  type ArticleFrontmatter,
  type ProjectFrontmatter,
} from "@/lib/content/schema";

export type DiskContentEntry =
  | {
      filePath: string;
      source: string;
      frontmatter: ArticleFrontmatter;
    }
  | {
      filePath: string;
      source: string;
      frontmatter: ProjectFrontmatter;
    };

async function walk(directory: string): Promise<string[]> {
  const entries = await readdir(directory, { withFileTypes: true });
  const nested = await Promise.all(
    entries.map(async (entry) => {
      const path = resolve(directory, entry.name);
      return entry.isDirectory() ? walk(path) : [path];
    }),
  );
  return nested.flat();
}

export function splitFrontmatter(source: string): {
  data: unknown;
  body: string;
} {
  const match = source.match(/^---\r?\n([\s\S]*?)\r?\n---\r?\n/);
  if (!match) {
    throw new Error("Missing YAML frontmatter");
  }
  return {
    data: parseYaml(match[1]),
    body: source.slice(match[0].length),
  };
}

export async function loadDiskContent(): Promise<DiskContentEntry[]> {
  const root = resolve(process.cwd(), "content");
  const files = (await walk(root)).filter((path) => path.endsWith(".mdx")).toSorted();
  return Promise.all(
    files.map(async (filePath) => {
      const source = await readFile(filePath, "utf8");
      const { data } = splitFrontmatter(source);
      const kind =
        typeof data === "object" && data !== null && "kind" in data
          ? (data as { kind?: unknown }).kind
          : undefined;
      const frontmatter =
        kind === "project"
          ? projectFrontmatterSchema.parse(data)
          : articleFrontmatterSchema.parse(data);
      return { filePath, source, frontmatter } as DiskContentEntry;
    }),
  );
}

export function bodyHeadingIds(source: string): string[] {
  const { body } = splitFrontmatter(source);
  return [...body.matchAll(/<h2 id="([a-z][a-z0-9-]*)">/g)].map(
    (match) => match[1],
  );
}
