import { Link } from "react-router-dom";
import {
  hitlCanvasPath,
  pendingPreviewsPath,
  type PendingPreviewRow,
} from "../../data/projections/hitl";

/**
 * HITL announce table. Today deep-links into the Projects canvas. The canvas
 * itself never mints Confirm/Approve; digest-bound confirm stays on
 * management HTTP (T09). Not Inbox L1.
 */
export function HitlCanvasTable({
  projectId,
  rows,
  focusPreviewId,
  deepLink,
}: {
  projectId: string;
  rows: PendingPreviewRow[];
  focusPreviewId?: string | null;
  deepLink: boolean;
}) {
  const focus = focusPreviewId && focusPreviewId.length > 0 ? focusPreviewId : undefined;
  const focusMissing = Boolean(focus) && !rows.some((row) => row.previewId === focus);
  return (
    <>
      {focusMissing ? (
        <p className="cp-quiet">
          Preview <code className="cp-mono">{focus}</code> is not on this pending
          list. Nothing is inferred as approved.
        </p>
      ) : null}
      <table className="cp-table">
        <caption className="cp-quiet">
          GET {pendingPreviewsPath(projectId)} — announce only; no Confirm. HITL
          lives on the Projects canvas, not Inbox.
        </caption>
        <thead>
          <tr>
            <th>Preview</th>
            <th>Kind</th>
            <th>Status</th>
            {deepLink ? <th>Canvas</th> : null}
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => {
            const selected = focus === row.previewId;
            return (
              <tr
                key={row.previewId}
                data-row-key={row.previewId}
                data-canvas-focus={selected ? "true" : undefined}
                aria-selected={selected ? true : undefined}
              >
                <td>
                  <code className="cp-mono">{row.previewId}</code>
                </td>
                <td>{row.subjectKind}</td>
                <td>{row.status}</td>
                {deepLink ? (
                  <td>
                    <Link to={hitlCanvasPath(row.previewId)}>Open on Projects canvas</Link>
                  </td>
                ) : null}
              </tr>
            );
          })}
        </tbody>
      </table>
    </>
  );
}
