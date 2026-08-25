import type { ReactNode } from "react";

/**
 * HonestyNote — the product's capability-honesty furniture. Used for
 * unavailable/not-backed/partial facts with their named dependency (BD-n)
 * or CLI path. This is information, not an apology.
 */
export function HonestyNote({ children }: { children: ReactNode }) {
  return (
    <p className="cp-honesty" role="note">
      {children}
    </p>
  );
}
