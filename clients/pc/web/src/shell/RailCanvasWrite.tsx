import { useState, type FormEvent, type KeyboardEvent } from "react";
import { Link } from "react-router-dom";
import { readJson } from "../api";
import {
  ASSISTANT_TURN_PATH,
  DRAFT_APPLY_PATH,
  projectAssistantTurn,
  projectDraftApply,
  railWriteReady,
  type DraftApplyRow,
} from "../data/projections/assistant";
import { hitlCanvasPath } from "../data/projections/hitl";
import { httpErrorMessage } from "../views/opc/httpError";

function jsonField(body: unknown, key: string): string | undefined {
  if (!body || typeof body !== "object") {
    return undefined;
  }
  const value = (body as Record<string, unknown>)[key];
  return typeof value === "string" && value.length > 0 ? value : undefined;
}

/**
 * Right-rail edit → review → write canvas (P12-T09).
 * Owner posts assistant.turn then draft.apply. Chat cannot Approve.
 * Preview bypass is refused: Write to canvas exists only after review.
 */
export function RailCanvasWrite() {
  const [draftId, setDraftId] = useState("");
  const [baseSeq, setBaseSeq] = useState("0");
  const [text, setText] = useState("");
  const [step, setStep] = useState<"edit" | "review">("edit");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | undefined>();
  const [written, setWritten] = useState<DraftApplyRow | undefined>();
  const [previewId, setPreviewId] = useState<string | undefined>();

  function goReview(event?: FormEvent) {
    event?.preventDefault();
    setError(undefined);
    setWritten(undefined);
    const ready = railWriteReady({ draftId, baseSeq, text });
    if (!ready.ok) {
      setError(ready.reason);
      return;
    }
    setStep("review");
  }

  function onEditorKey(event: KeyboardEvent<HTMLTextAreaElement>) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      goReview();
    }
  }

  async function writeCanvas() {
    const ready = railWriteReady({ draftId, baseSeq, text });
    if (!ready.ok) {
      setError(ready.reason);
      return;
    }
    if (step !== "review") {
      setError("Review the write before posting. Preview bypass is refused.");
      return;
    }
    setBusy(true);
    setError(undefined);
    try {
      const proposed = await readJson(ASSISTANT_TURN_PATH, "management", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          kind: "propose",
          draft_id: draftId.trim(),
          object_kind: "charter",
          payload: { text: text.trim() },
          provenance: { kind: "owner-stated" },
        }),
      });
      if (!proposed.ok) {
        setError(
          `${httpErrorMessage(proposed.status, proposed.body)} Candidate was not applied. Chat cannot Approve.`,
        );
        return;
      }
      const turn = projectAssistantTurn(proposed.body);
      const digest = turn?.candidateDigest ?? jsonField(proposed.body, "candidate_digest");
      if (!digest) {
        setError("assistant.turn returned no candidate_digest. Canvas was not written.");
        return;
      }
      if (turn?.previewId) {
        setPreviewId(turn.previewId);
      }
      const applied = await readJson(DRAFT_APPLY_PATH, "management", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          draft_id: draftId.trim(),
          base_seq: Number(baseSeq.trim()),
          candidate_digest: digest,
        }),
      });
      if (!applied.ok) {
        setError(
          `${httpErrorMessage(applied.status, applied.body)} Canvas was not written. This rail does not Approve or confirm authority.`,
        );
        return;
      }
      const row = projectDraftApply(applied.body);
      if (!row) {
        setError("draft.apply returned no payload_digest. Canvas write is unknown, not success.");
        return;
      }
      setWritten(row);
      setStep("edit");
      setText("");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div data-region="opc-rail-write">
      <h3 className="cp-section-title">Canvas write</h3>
      <p className="cp-quiet">
        Edit → review → write the open draft canvas. Candidate-only. Chat cannot
        Approve. HITL Confirm stays on the Projects canvas. This rail does not
        write SecretStore, archive, or authority.
      </p>
      {step === "edit" ? (
        <form onSubmit={goReview}>
          <label className="cp-field">
            <span>Open draft id</span>
            <input
              name="draft_id"
              value={draftId}
              onChange={(event) => setDraftId(event.target.value)}
              autoComplete="off"
            />
          </label>
          <label className="cp-field">
            <span>base_seq</span>
            <input
              name="base_seq"
              value={baseSeq}
              onChange={(event) => setBaseSeq(event.target.value)}
              inputMode="numeric"
              autoComplete="off"
            />
          </label>
          <label className="cp-field">
            <span>Canvas edit</span>
            <textarea
              name="canvas_edit"
              value={text}
              onChange={(event) => setText(event.target.value)}
              onKeyDown={onEditorKey}
            />
          </label>
          <p>
            <button type="submit" className="cp-button">
              Review write
            </button>
          </p>
        </form>
      ) : (
        <div role="dialog" aria-label="Write canvas review" data-region="opc-rail-review">
          <p>
            Owner message (local, not archive): {text.trim()}
          </p>
          <p className="cp-quiet">
            Assistant will propose a digest-bound candidate, then owner write
            posts draft.apply. Preview announce is not Approve.
          </p>
          <p>
            <button
              type="button"
              className="cp-button"
              disabled={busy}
              onClick={() => void writeCanvas()}
            >
              Write to canvas
            </button>{" "}
            <button
              type="button"
              className="cp-button"
              disabled={busy}
              onClick={() => {
                setStep("edit");
                setError(undefined);
              }}
            >
              Discard
            </button>
          </p>
        </div>
      )}
      {previewId ? (
        <p className="cp-quiet" data-region="opc-rail-preview-announce">
          Preview {previewId} is announce-only.{" "}
          <Link to={hitlCanvasPath(previewId)}>Open on Projects canvas</Link>
        </p>
      ) : null}
      {written ? (
        <p data-region="opc-rail-written">
          Canvas written. payload_digest{" "}
          <code className="cp-mono">{written.payloadDigest}</code> · new_base_seq{" "}
          <code className="cp-mono">{written.newBaseSeq}</code>. This is not
          Project activation.
        </p>
      ) : null}
      {error ? (
        <p className="cp-error" data-region="opc-rail-write-error">
          {error}
        </p>
      ) : null}
    </div>
  );
}
