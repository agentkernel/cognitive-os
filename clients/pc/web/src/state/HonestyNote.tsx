import type { ReactNode } from "react";

/**
 * HonestyNote — capability-honesty furniture. Owner chrome leads with the
 * decision surface; developer sourcing notes use placement="secondary".
 */
export function HonestyNote({
  children,
  placement = "primary",
}: {
  children: ReactNode;
  placement?: "primary" | "secondary";
}) {
  if (placement === "secondary") {
    return (
      <details data-honesty="secondary" className="cp-honesty-secondary">
        <summary className="cp-quiet">How this page is sourced</summary>
        <p className="cp-honesty" role="note">
          {children}
        </p>
      </details>
    );
  }
  return (
    <p className="cp-honesty" role="note">
      {children}
    </p>
  );
}
