import type { ReactNode } from "react";
import { useCallback, useEffect, useState } from "react";
import { CommandPalette } from "../components/CommandPalette";
import { PrimaryNav } from "./PrimaryNav";
import { StatusStrip } from "./StatusStrip";
import { useShellKeyboard } from "./useShellKeyboard";

/**
 * App shell — docs/design/12. Status strip (top, full width) + primary nav
 * (left) + main region. Three content layouts are sanctioned (MI / MID / CS);
 * the shell provides the frame only. ⌘K is chrome, not a space.
 */
export function AppShell({ children }: { children: ReactNode }) {
  const [paletteOpen, setPaletteOpen] = useState(false);
  const openPalette = useCallback(() => setPaletteOpen(true), []);
  useShellKeyboard({ paletteOpen, onOpenPalette: openPalette });

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setPaletteOpen((open) => !open);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

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
      <PrimaryNav onOpenPalette={() => setPaletteOpen(true)} paletteOpen={paletteOpen} />
      <main id="main" className="cp-main" tabIndex={-1}>
        {children}
      </main>
      <CommandPalette open={paletteOpen} onClose={() => setPaletteOpen(false)} />
    </div>
  );
}
