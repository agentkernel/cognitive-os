declare module "*.mdx" {
  import type { ComponentType } from "react";

  const MDXComponent: ComponentType;
  export const frontmatter: unknown;
  export default MDXComponent;
}

declare module "*.md" {
  import type { ComponentType } from "react";

  const MDXComponent: ComponentType;
  export default MDXComponent;
}
