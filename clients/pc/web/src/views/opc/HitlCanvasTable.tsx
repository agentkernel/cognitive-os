import { useEffect, useState, type FormEvent } from "react";
import { Link } from "react-router-dom";
import { readJson } from "../../api";
import {
  CONFIRM_PATH,
  NARROW_PATH,
  REJECT_PATH,
  hitlCanvasPath,
  pendingPreviewsPath,
  previewDetailKey,
  previewIsConfirmable,
  type PendingPreviewRow,
  type PreviewDetailRow,
} from "../../data/projections/hitl";
import { useProjection } from "../../data/useProjection";
import { loadPreviewDetail } from "./loadOpcReads";

function errorMessage(status: number, body: unknown): string {
  if (body && typeof body === "object") {
    const record = body as Record<string, unknown>;
    const nested = record.error;
    if (nested && typeof nested === "object") {
      const error = nested as Record<string, unknown>;
      const code = typeof error.code === "string" ? error.code : "error";
      const message = typeof error.message === "string" ? error.message : "";
      return `HTTP ${status} · ${code}${message ? ` — ${message}` : ""}`;
    }
    if (typeof record.code === "string") {
      const message = typeof record.message === "string" ? record.message : "";
      return `HTTP ${status} · ${record.code}${message ? ` — ${message}` : ""}`;
    }
    if (typeof record.detail === "string" && record.detail.length > 0) {
      return `HTTP ${status} · ${record.detail}`;
    }
  }
  return `HTTP ${status}`;
}

/**
 * HITL canvas: announce table plus digest-bound Confirm / Narrow / Reject.
 * Stop is the fourth control and stays honest when the daemon has no
 * in-flight preview.stop. Chat cannot Approve. Not Inbox L1.
 */
export function HitlCanvasTable({
  projectId,
  rows,
  focusPreviewId,
  deepLink,
  onWritten,
}: {
  projectId: string;
  rows: PendingPreviewRow[];
  focusPreviewId?: string | null;
  deepLink: boolean;
  onWritten?: () => void;
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
          GET {pendingPreviewsPath(projectId)} — announce list. Confirm uses
          preview-detail digest on management HTTP. HITL lives on the Projects
          canvas, not Inbox.
        </caption>
        <thead>
          <tr>
            <th>Preview</th>
            <th>Kind</th>
            <th>Status</th>
            <th>{deepLink ? "Canvas" : "Review"}</th>
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
                <td>
                  <Link to={hitlCanvasPath(row.previewId, deepLink ? undefined : projectId)}>
                    {deepLink ? "Open on Projects canvas" : "Review on canvas"}
                  </Link>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      {focus && !focusMissing ? (
        <HitlCanvasActions previewId={focus} onWritten={onWritten} />
      ) : null}
    </>
  );
}

function HitlCanvasActions({
  previewId,
  onWritten,
}: {
  previewId: string;
  onWritten?: () => void;
}) {
  const detail = useProjection<PreviewDetailRow[]>(previewDetailKey(previewId));
  const [narrowBytes, setNarrowBytes] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | undefined>();
  const [receipt, setReceipt] = useState<string | undefined>();
  useEffect(() => {
    setError(undefined);
    setReceipt(undefined);
    setNarrowBytes("");
    void loadPreviewDetail(previewId);
  }, [previewId]);
  const row = detail.data?.[0];
  const confirmable = detail.status === "ready" && previewIsConfirmable(row);

  async function postAction(path: string, body: Record<string, string>) {
    if (!confirmable || !row) {
      setError("No digest-bound pending preview. Confirm is not offered.");
      return;
    }
    setBusy(true);
    setError(undefined);
    setReceipt(undefined);
    try {
      const written = await readJson(path, "management", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(body),
      });
      if (!written.ok) {
        setError(errorMessage(written.status, written.body));
        return;
      }
      const record = written.body && typeof written.body === "object"
        ? (written.body as Record<string, unknown>)
        : {};
      const nextReceipt =
        typeof record.receipt_ref === "string"
          ? record.receipt_ref
          : typeof record.preview_id === "string"
            ? record.preview_id
            : "ok";
      setReceipt(nextReceipt);
      await loadPreviewDetail(previewId);
      onWritten?.();
    } finally {
      setBusy(false);
    }
  }

  function confirmPreview() {
    if (!row) {
      return;
    }
    void postAction(CONFIRM_PATH, {
      preview_id: row.previewId,
      preview_digest: row.previewDigest,
    });
  }

  function rejectPreview() {
    if (!row) {
      return;
    }
    void postAction(REJECT_PATH, {
      preview_id: row.previewId,
      preview_digest: row.previewDigest,
    });
  }

  function narrowPreview(event: FormEvent) {
    event.preventDefault();
    if (!row) {
      return;
    }
    const bytes = narrowBytes.trim();
    if (bytes.length === 0) {
      setError("Narrow needs new preview bytes. The old digest is not reused as success.");
      return;
    }
    void postAction(NARROW_PATH, {
      preview_id: row.previewId,
      preview_digest: row.previewDigest,
      preview_bytes: bytes,
    });
  }

  return (
    <div data-region="opc-hitl-actions" className="cp-stack">
      {detail.status === "loading" || detail.status === "stale" ? (
        <p className="cp-quiet">Fetching preview-detail digest from the daemon.</p>
      ) : null}
      {detail.status === "denied" ? (
        <p data-hitl-blocked="denied">
          Preview-detail was denied. Confirm is not offered. Nothing is inferred
          as approved.
        </p>
      ) : null}
      {detail.status === "disconnected" ? (
        <p data-hitl-blocked="disconnected">
          Daemon unreachable. Confirm is not offered. Offline cannot approve
          an external mutation.
        </p>
      ) : null}
      {detail.status === "unknown" || detail.status === "not-run" || detail.status === "empty" ? (
        <p data-hitl-blocked="unknown">
          This preview is unknown. Nothing is inferred as approved or successful.
          Blind retry is forbidden.
        </p>
      ) : null}
      {detail.status === "ready" && row && row.status === "stale" ? (
        <p data-hitl-blocked="stale">
          This preview is stale. Confirm is not offered. Narrow must mint a new
          preview; the old digest is never confirmable.
        </p>
      ) : null}
      {detail.status === "ready" && row && row.status !== "pending" && row.status !== "stale" ? (
        <p data-hitl-blocked={row.status}>
          Preview status <code className="cp-mono">{row.status}</code> is not
          pending. Confirm is not offered.
        </p>
      ) : null}
      {detail.status === "ready" && row && row.status === "pending" && row.previewDigest.length === 0 ? (
        <p data-hitl-blocked="no-digest">
          preview-detail returned no digest. Confirm is not offered. The list
          row is never used as a digest.
        </p>
      ) : null}
      <p className="cp-quiet">
        Chat cannot Approve. Persist-before-dispatch stays on the daemon. Stop
        is for in-flight execution; this pending preview is not executing and
        the daemon has no preview.stop route.
      </p>
      <div className="cp-row">
        <button
          type="button"
          className="cp-button"
          disabled={!confirmable || busy}
          onClick={confirmPreview}
        >
          Confirm preview
        </button>
        <button
          type="button"
          className="cp-button"
          disabled={!confirmable || busy}
          onClick={rejectPreview}
        >
          Reject preview
        </button>
        <button type="button" className="cp-button" disabled>
          Stop execution
        </button>
      </div>
      <form onSubmit={narrowPreview}>
        <label>
          Narrowed preview bytes
          <textarea
            name="narrow-bytes"
            value={narrowBytes}
            onChange={(event) => setNarrowBytes(event.target.value)}
            disabled={!confirmable || busy}
          />
        </label>
        <button type="submit" className="cp-button" disabled={!confirmable || busy}>
          Narrow preview
        </button>
      </form>
      {error ? (
        <p data-hitl-error="true" className="cp-error">
          {error}
        </p>
      ) : null}
      {receipt ? (
        <p data-region="opc-hitl-written">
          Daemon receipt <code className="cp-mono">{receipt}</code>
        </p>
      ) : null}
    </div>
  );
}
