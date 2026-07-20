import type { MDXComponents } from "mdx/types";
import { useId, type ComponentPropsWithoutRef } from "react";
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
import type { Locale } from "@/i18n/config";

type DiagramProps = { locale: Locale };

function LocalizedTable(props: ComponentPropsWithoutRef<"table">) {
  const labelId = useId();
  return (
    <div
      className="table-scroll"
      role="region"
      aria-labelledby={labelId}
      tabIndex={0}
    >
      <span id={labelId} className="sr-only">
        <span className="locale-copy locale-copy--zh">可横向滚动的数据表</span>
        <span className="locale-copy locale-copy--en">Scrollable data table</span>
      </span>
      <table {...props} />
    </div>
  );
}

const sharedComponents: MDXComponents = {
  ArticleSnapshot,
  AuthorityBoundaryDiagram: (props: DiagramProps) => (
    <AuthorityBoundaryDiagram {...props} mode="summary" />
  ),
  ContextPipelineDiagram: (props: DiagramProps) => (
    <ContextPipelineDiagram {...props} mode="summary" />
  ),
  GovernedFlowDiagram: (props: DiagramProps) => (
    <GovernedFlowDiagram {...props} mode="summary" />
  ),
  GovernedFlowThread,
  LifecycleDomainsDiagram: (props: DiagramProps) => (
    <LifecycleDomainsDiagram {...props} mode="summary" />
  ),
  OverallArchitectureDiagram: (props: DiagramProps) => (
    <OverallArchitectureDiagram {...props} mode="summary" />
  ),
  SampleNotice,
  table: LocalizedTable,
  pre: CodeBlock,
};

export function useMDXComponents(components: MDXComponents): MDXComponents {
  return {
    ...sharedComponents,
    ...components,
  };
}
