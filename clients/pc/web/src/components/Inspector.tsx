import type { ReactNode } from "react";

/**
 * Inspector — the 5-minute layer beside a master list. Facts and class-B
 * actions; never inline edits (mutations live in governed flows).
 */
export function Inspector({
  title,
  children,
  label,
}: {
  title: string;
  children: ReactNode;
  label?: string;
}) {
  return (
    <aside className="cp-inspector" aria-label={label ?? `${title} inspector`}>
      <h3>{title}</h3>
      {children}
    </aside>
  );
}
