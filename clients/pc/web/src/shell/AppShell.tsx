import type { ReactNode } from "react";
import { PrimaryNav } from "./PrimaryNav";
import { StatusStrip } from "./StatusStrip";

/**
 * App shell — docs/design/12. Status strip (top, full width) + primary nav
 * (left) + main region. Three content layouts are sanctioned (MI / MID / CS);
 * the shell provides the frame only.
 */
export function AppShell({ children }: { children: ReactNode }) {
  return (
    <div className="cp-app cp-shell">
      <a
        className="skip"
        href="#main"
        onClick={(event) => {
          event.preventDefault();
          document.getElementById("main")?.focus();
        }}
      >
        Skip to content
      </a>
      <StatusStrip />
      <PrimaryNav />
      <main id="main" className="cp-main" tabIndex={-1}>
        {children}
      </main>
    </div>
  );
}
