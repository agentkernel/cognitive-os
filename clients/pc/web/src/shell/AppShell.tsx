import type { ReactNode } from "react";
import { useCallback, useEffect, useState } from "react";
import { useLocation } from "react-router-dom";
import { CommandPalette } from "../components/CommandPalette";
import { liveProjectRows, PROJECTS_KEY, type ProjectListRow } from "../data/projections/projects";
import { useProjection } from "../data/useProjection";
import type { Projection } from "../data/store";
import { AssistantRail } from "./AssistantRail";
import { PrimaryNav } from "./PrimaryNav";
import { StatusStrip } from "./StatusStrip";
import { useShellKeyboard } from "./useShellKeyboard";

function hideAssistantRail(pathname: string, projects: Projection<ProjectListRow[]>): boolean {
  if (pathname.startsWith("/projects/new")) {
    return true;
  }
  if (pathname !== "/") {
    return false;
  }
  if (projects.status === "empty") {
    return true;
  }
  if (projects.status === "ready" && (projects.data?.length ?? 0) === 0) {
    return true;
  }
  if (projects.status === "ready" && liveProjectRows(projects.data).length === 0) {
    return true;
  }
  if (projects.status === "loading") {
    return true;
  }
  return false;
}

/**
 * App shell — Personal 2.0: strip + L1 + main + assistant rail.
 * ⌘K is chrome, not a space. The rail never Approves. Empty home and the
 * create wizard hide the rail (P12-T02). Creating-only Today also hides it
 * (P12-T05 today-incomplete).
 */
export function AppShell({ children }: { children: ReactNode }) {
  const location = useLocation();
  const projects = useProjection<ProjectListRow[]>(PROJECTS_KEY);
  const hideRail = hideAssistantRail(location.pathname, projects);
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
      {hideRail ? null : <AssistantRail />}
      <CommandPalette open={paletteOpen} onClose={() => setPaletteOpen(false)} />
    </div>
  );
}
