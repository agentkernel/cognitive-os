import { useState } from "react";
import { Link, useLocation } from "react-router-dom";
import { hitlCanvasPath, HITL_KEY, type PendingPreviewRow } from "../data/projections/hitl";
import { useProjection } from "../data/useProjection";
import { HonestyNote } from "../state/HonestyNote";
import { ProjectGroupChat } from "./ProjectGroupChat";
import { RailCanvasWrite } from "./RailCanvasWrite";

export type RailLayer = "group" | "assistant";

/** Live Project id when the route is inside one Project (never the create wizard). */
export function railProjectId(pathname: string): string | undefined {
  const match = /^\/projects\/([^/]+)/.exec(pathname);
  if (!match) {
    return undefined;
  }
  const raw = match[1] ?? "";
  if (raw.length === 0 || raw === "new") {
    return undefined;
  }
  try {
    return decodeURIComponent(raw);
  } catch {
    return raw;
  }
}

/**
 * Assistant rail — Personal 2.0 chrome. Outside a Project the rail is the
 * Personal Assistant (HITL announce-only plus P12-T09 edit → review → write
 * canvas). Inside a Project the group conversation (Owner / manager /
 * Members, P13-T06) is layered on top and switchable; both layers stay
 * mounted so switching preserves each unsent draft and never merges, clears,
 * or sends. Candidate-only; no Approve control anywhere in the rail.
 */
export function AssistantRail() {
  const location = useLocation();
  const projectId = railProjectId(location.pathname);
  const [layer, setLayer] = useState<RailLayer>("group");
  const hitl = useProjection<PendingPreviewRow[]>(HITL_KEY);
  const pending = hitl.status === "ready" ? (hitl.data ?? []) : [];
  const first = pending[0];
  const showGroup = projectId !== undefined && layer === "group";

  return (
    <aside data-rail="assistant" className="cp-rail" aria-label="Assistant">
      <h2>{projectId ? "Conversation" : "Assistant"}</h2>
      {projectId ? (
        <div role="group" aria-label="Conversation layer" data-region="opc-rail-layers">
          <button
            type="button"
            className="cp-button"
            aria-pressed={layer === "group"}
            data-layer="group"
            onClick={() => setLayer("group")}
          >
            Project group
          </button>{" "}
          <button
            type="button"
            className="cp-button"
            aria-pressed={layer === "assistant"}
            data-layer="assistant"
            onClick={() => setLayer("assistant")}
          >
            Personal Assistant
          </button>
          <p className="cp-quiet">
            Switching layers keeps each unsent draft; it never merges, clears, or sends.
          </p>
        </div>
      ) : null}
      {projectId ? <ProjectGroupChat projectId={projectId} hidden={!showGroup} /> : null}
      <div data-region="opc-rail-assistant-layer" hidden={showGroup}>
        <HonestyNote>
          Candidate-only. This rail does not Approve, persist a Project, or write
          SecretStore or archive. Chat Approve is not a Control Plane control.
          HITL Confirm stays on management HTTP.
        </HonestyNote>
        {first ? (
          <p className="cp-quiet" data-region="opc-rail-hitl">
            {pending.length} pending ApprovalPreview
            {pending.length === 1 ? "" : "s"} loaded in this tab. Announce only.
            Confirm stays on the Projects canvas via management HTTP.{" "}
            <Link to={hitlCanvasPath(first.previewId)}>Open on Projects canvas</Link>
          </p>
        ) : (
          <p className="cp-quiet">
            Pi drafts stay on the daemon preview path. Nothing here is an
            authority writer.
          </p>
        )}
        <RailCanvasWrite />
      </div>
    </aside>
  );
}
