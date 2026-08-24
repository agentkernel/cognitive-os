import type { ReactNode } from "react";

/**
 * One-sentence, cause-first reason in quiet type. Used under state chips and
 * in rows. Never "something went wrong" — the cause or nothing.
 */
export function ReasonLine({ children }: { children: ReactNode }) {
  return <span className="cp-reason">{children}</span>;
}
