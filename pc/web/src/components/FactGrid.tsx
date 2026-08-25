import type { ReactNode } from "react";

export interface Fact {
  label: string;
  value: ReactNode;
}

/**
 * FactGrid — definition-list fact display. Labels quiet/small, values mono.
 * Every fact should carry its source's semantics; unknown renders as
 * "unknown", never blank or zero.
 */
export function FactGrid({ facts }: { facts: Fact[] }) {
  return (
    <dl className="cp-factgrid">
      {facts.map((fact) => (
        <div key={fact.label} style={{ display: "contents" }}>
          <dt>{fact.label}</dt>
          <dd>{fact.value ?? "unknown"}</dd>
        </div>
      ))}
    </dl>
  );
}
