import type { ReactNode } from "react";

/**
 * ReceiptLine — post-action receipt. What changed, with its id/digest when
 * one exists. Receipts persist (inline), they are not toasts: authority acts
 * leave records, not ephemera.
 */
export function ReceiptLine({ children }: { children: ReactNode }) {
  return (
    <p className="cp-receipt" role="status">
      {children}
    </p>
  );
}
