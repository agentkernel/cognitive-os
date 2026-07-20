import type { MDXComponents } from "mdx/types";
import type { ComponentPropsWithoutRef } from "react";
import { CodeBlock } from "@/components/content/code-block";
import { ArticleSnapshot, SampleNotice } from "@/components/content/content-callouts";
import { GovernedFlowThread } from "@/components/content/governed-flow-thread";
import {
  AuthorityBoundaryDiagram,
  ContextPipelineDiagram,
  GovernedFlowDiagram,
  LifecycleDomainsDiagram,
  OverallArchitectureDiagram,
} from "@/components/diagrams/governance-diagrams";

const sharedComponents: MDXComponents = {
  ArticleSnapshot,
  AuthorityBoundaryDiagram,
  ContextPipelineDiagram,
  GovernedFlowDiagram,
  GovernedFlowThread,
  LifecycleDomainsDiagram,
  OverallArchitectureDiagram,
  SampleNotice,
  table: (props: ComponentPropsWithoutRef<"table">) => (
    <div
      className="table-scroll"
      role="region"
      aria-label="Scrollable data table / 可滚动数据表"
      tabIndex={0}
    >
      <table {...props} />
    </div>
  ),
  pre: CodeBlock,
};

export function useMDXComponents(components: MDXComponents): MDXComponents {
  return {
    ...sharedComponents,
    ...components,
  };
}
