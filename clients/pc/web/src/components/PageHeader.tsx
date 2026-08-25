import type { ReactNode } from "react";

/** Space-level header: one title, one short lede, nothing else. */
export function PageHeader({ title, lede }: { title: string; lede?: ReactNode }) {
  return (
    <header className="cp-page-head">
      <h2>{title}</h2>
      {lede ? <p className="cp-lede">{lede}</p> : null}
    </header>
  );
}
