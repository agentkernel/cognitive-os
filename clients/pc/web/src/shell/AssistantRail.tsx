import { Link } from "react-router-dom";
import { hitlCanvasPath, HITL_KEY, type PendingPreviewRow } from "../data/projections/hitl";
import { useProjection } from "../data/useProjection";
import { HonestyNote } from "../state/HonestyNote";
import { RailCanvasWrite } from "./RailCanvasWrite";

/**
 * Assistant rail — Personal 2.0 chrome. HITL announce-only plus P12-T09
 * edit → review → write canvas. Candidate-only; no Approve control.
 */
export function AssistantRail() {
  const hitl = useProjection<PendingPreviewRow[]>(HITL_KEY);
  const pending = hitl.status === "ready" ? (hitl.data ?? []) : [];
  const first = pending[0];

  return (
    <aside data-rail="assistant" className="cp-rail" aria-label="Assistant">
      <h2>Assistant</h2>
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
    </aside>
  );
}
