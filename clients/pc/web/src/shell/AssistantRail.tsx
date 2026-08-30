import { HonestyNote } from "../state/HonestyNote";

/**
 * Assistant rail — Personal 2.0 chrome. Candidate-only; no Approve control.
 * Completing a Project is not a model reply (P11-T06).
 */
export function AssistantRail() {
  return (
    <aside data-rail="assistant" className="cp-rail" aria-label="Assistant">
      <h2>Assistant</h2>
      <HonestyNote>
        Candidate-only. This rail does not Approve, persist, or complete a
        Project. Chat Approve is not a Control Plane control.
      </HonestyNote>
      <p className="cp-quiet">
        Pi drafts stay on the daemon preview path. Nothing here is an
        authority writer.
      </p>
    </aside>
  );
}
