import type { MDXComponents } from "mdx/types";
import type { ComponentPropsWithoutRef } from "react";
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
  pre: (props: ComponentPropsWithoutRef<"pre">) => (
    <pre
      {...props}
      tabIndex={0}
      aria-label="Scrollable code block / 可滚动代码块"
    />
  ),
};

export function useMDXComponents(components: MDXComponents): MDXComponents {
  return {
    ...sharedComponents,
    ...components,
  };
}
