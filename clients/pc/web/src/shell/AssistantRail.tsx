import { Link } from "react-router-dom";
import { hitlCanvasPath, HITL_KEY, type PendingPreviewRow } from "../data/projections/hitl";
import { useProjection } from "../data/useProjection";
import { HonestyNote } from "../state/HonestyNote";

/**
 * Assistant rail — Personal 2.0 chrome. Announce-only for pending HITL
 * already loaded in this tab (T09). Candidate-only; no Approve control.
 */
export function AssistantRail() {
  const hitl = useProjection<PendingPreviewRow[]>(HITL_KEY);
  const pending = hitl.status === "ready" ? (hitl.data ?? []) : [];
  const first = pending[0];

  return (
    <aside data-rail="assistant" className="cp-rail" aria-label="Assistant">
      <h2>Assistant</h2>
      <HonestyNote>
        Candidate-only. This rail does not Approve, persist, or complete a
        Project. Chat Approve is not a Control Plane control.
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
    </aside>
  );
}
