import type { ComponentType } from "react";
import type { ContentId } from "@/lib/content/manifest";

type MdxModule = {
  default: ComponentType;
  frontmatter: unknown;
};

const contentLoaders: Record<ContentId, () => Promise<MdxModule>> = {
  "cognitiveos:zh-CN:verifiable-agent-actions": () =>
    import("@content/cognitiveos/verifiable-agent-actions/zh-CN.mdx"),
  "cognitiveos:en:verifiable-agent-actions": () =>
    import("@content/cognitiveos/verifiable-agent-actions/en.mdx"),
  "article:zh-CN:context-budget-notes": () =>
    import("@content/articles/context-budget-notes/zh-CN.mdx"),
  "article:en:context-budget-notes": () =>
    import("@content/articles/context-budget-notes/en.mdx"),
  "article:zh-CN:designing-failure-semantics": () =>
    import("@content/articles/designing-failure-semantics/zh-CN.mdx"),
  "article:en:designing-failure-semantics": () =>
    import("@content/articles/designing-failure-semantics/en.mdx"),
  "article:zh-CN:testing-bilingual-content": () =>
    import("@content/articles/testing-bilingual-content/zh-CN.mdx"),
  "article:en:testing-bilingual-content": () =>
    import("@content/articles/testing-bilingual-content/en.mdx"),
  "project:zh-CN:accessible-docs-migration": () =>
    import("@content/projects/accessible-docs-migration/zh-CN.mdx"),
  "project:en:accessible-docs-migration": () =>
    import("@content/projects/accessible-docs-migration/en.mdx"),
  "project:zh-CN:evidence-first-cli": () =>
    import("@content/projects/evidence-first-cli/zh-CN.mdx"),
  "project:en:evidence-first-cli": () =>
    import("@content/projects/evidence-first-cli/en.mdx"),
  "project:zh-CN:governed-context-prototype": () =>
    import("@content/projects/governed-context-prototype/zh-CN.mdx"),
  "project:en:governed-context-prototype": () =>
    import("@content/projects/governed-context-prototype/en.mdx"),
  "project:zh-CN:incident-replay-notebook": () =>
    import("@content/projects/incident-replay-notebook/zh-CN.mdx"),
  "project:en:incident-replay-notebook": () =>
    import("@content/projects/incident-replay-notebook/en.mdx"),
};

export async function loadContentComponent(
  id: ContentId,
): Promise<ComponentType> {
  return (await contentLoaders[id]()).default;
}
