import type { ContentFrontmatter } from "@/lib/content/schema";

export function isPublishableFrontmatter(
  frontmatter: ContentFrontmatter,
): boolean {
  return (
    frontmatter.status === "published" &&
    frontmatter.placeholder === false
  );
}

export function isLabFrontmatter(frontmatter: ContentFrontmatter): boolean {
  return !isPublishableFrontmatter(frontmatter);
}
