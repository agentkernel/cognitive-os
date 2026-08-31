import type { ReactNode } from "react";
import { useCallback, useEffect, useState } from "react";
import { CommandPalette } from "../components/CommandPalette";
import { AssistantRail } from "./AssistantRail";
import { PrimaryNav } from "./PrimaryNav";
import { StatusStrip } from "./StatusStrip";
import { useShellKeyboard } from "./useShellKeyboard";

/**
 * App shell — Personal 2.0: strip + L1 + main + assistant rail.
 * ⌘K is chrome, not a space. The rail never Approves.
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
      <AssistantRail />
      <CommandPalette open={paletteOpen} onClose={() => setPaletteOpen(false)} />
    </div>
  );
}
