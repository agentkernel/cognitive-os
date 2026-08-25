import { useCallback, useState, type FormEvent } from "react";
import { Link, useNavigate } from "react-router-dom";
import { readJson } from "../../api";
import { PageHeader } from "../../components/PageHeader";
import {
  admitFailure,
  candidateFacts,
  chainErrorMessage,
  noteObservedTask,
  noteSessionChain,
  projectAdmission,
  projectInterpretation,
  projectIntentRecord,
  projectPreview,
  type AdmissionView,
  type AmbiguityDraft,
  type InterpretationView,
  type PreviewView,
} from "../../data/projections/work";
import { appProjections } from "../../data/store";
import { sessionPrincipal } from "../../session";
import { interpretCandidate, uuidV7, workspaceSearchDraft } from "../../taskDraft";
import type { WorkspaceSearchDraft } from "../../taskDraft";
import { HonestyNote } from "../../state/HonestyNote";
import { AdmissionReceipt } from "./AdmissionReceipt";
import { InterpretationReview } from "./InterpretationReview";
import { TaskPreview } from "./TaskPreview";

const CONVERSATION_REF = "conversation://personal/web-ui";

/**
 * New task — the governed creation flow (docs/design/14 §5).
 *
 * The chain is the daemon's, in its exact order:
 *   objective
 *     → POST /task/intent.record      (fixes the raw expression first)
 *     → POST /task/intent.interpret   (candidate facts; daemon derives status)
 *     → interpretation review         (ambiguities and gaps are the content)
 *     → POST /task/preview            (contract draft → preview digest)
 *     → digest-bound review
 *     → POST /task/admit              (same digest, interpretation, principal)
 *     → admission receipt
 *
 * The browser mints no authority at any step: it builds a draft with the
 * shared `taskDraft` helper and the daemon decides everything that matters.
 */
export function NewTaskPage() {
  const navigate = useNavigate();

  const [objective, setObjective] = useState("search the workspace for needle");
  const [constraints, setConstraints] = useState("");
  const [forbidden, setForbidden] = useState("");
  const [gaps, setGaps] = useState("");
  const [ambiguities, setAmbiguities] = useState<AmbiguityDraft[]>([]);

  const [recordId, setRecordId] = useState<string | undefined>();
  const [recordedAt, setRecordedAt] = useState<string | undefined>();
  const [interpretation, setInterpretation] = useState<InterpretationView | undefined>();
  const [supersedes, setSupersedes] = useState<string | undefined>();
  const [superseded, setSuperseded] = useState<string[]>([]);
  const [preview, setPreview] = useState<PreviewView | undefined>();
  const [draft, setDraft] = useState<WorkspaceSearchDraft | undefined>();
  const [admission, setAdmission] = useState<AdmissionView | undefined>();

  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | undefined>();
  const [needsReconfirm, setNeedsReconfirm] = useState(false);

  const lines = (value: string) =>
    value
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line !== "");

  const answeredAssumptions = ambiguities
    .filter((item) => item.answer.trim() !== "")
    .map((item) => `${item.question} → ${item.answer.trim()}`);

  const interpret = useCallback(
    async (userIntentRecordId: string, facts: AmbiguityDraft[], supersedesId?: string) => {
      const candidate = {
        ...interpretCandidate(objective),
        ...candidateFacts({
          objective,
          constraints: lines(constraints),
          forbidden: lines(forbidden),
          assumptions: [],
          ambiguities: facts,
          informationGaps: lines(gaps),
        }),
        ...(supersedesId ? { supersedes_interpretation_id: supersedesId } : {}),
      };
      const result = await readJson("/task/intent.interpret", "task", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          schema_version: "cognitiveos.task-intent-interpret-request/0.1",
          user_intent_record_id: userIntentRecordId,
          candidate,
        }),
      });
      if (!result.ok) {
        setError(chainErrorMessage("intent.interpret", result.status, result.body));
        return undefined;
      }
      const view = projectInterpretation(result.body);
      setInterpretation(view);
      setSupersedes(view.interpretationId);
      if (supersedesId != null && supersedesId !== "") {
        setSuperseded((ids) => (ids.includes(supersedesId) ? ids : [...ids, supersedesId]));
      }
      // A new interpretation voids any contract preview taken against the old one.
      setPreview(undefined);
      setNeedsReconfirm(false);
      return view;
    },
    [objective, constraints, forbidden, gaps],
  );

  async function describe(event: FormEvent) {
    event.preventDefault();
    setBusy(true);
    setError(undefined);
    setAdmission(undefined);
    try {
      const recorded = await readJson("/task/intent.record", "task", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          schema_version: "cognitiveos.task-intent-record-request/0.1",
          conversation_or_scope_ref: CONVERSATION_REF,
          raw_expression: objective,
        }),
      });
      if (!recorded.ok) {
        setError(chainErrorMessage("intent.record", recorded.status, recorded.body));
        return;
      }
      const record = projectIntentRecord(recorded.body);
      setRecordId(record.userIntentRecordId);
      setRecordedAt(record.recordedAt);
      await interpret(record.userIntentRecordId, ambiguities, undefined);
    } finally {
      setBusy(false);
    }
  }

  async function reinterpret() {
    if (!recordId) {
      return;
    }
    setBusy(true);
    setError(undefined);
    try {
      await interpret(recordId, ambiguities, supersedes);
    } finally {
      setBusy(false);
    }
  }

  /*
   * `preserveError` matters after a 409: the fresh preview is part of the
   * recovery, so it must not erase the reason the admission was refused.
   */
  async function makePreview(reconfirm: boolean, preserveError = false) {
    setBusy(true);
    if (!preserveError) {
      setError(undefined);
    }
    try {
      const nextDraft = workspaceSearchDraft(objective);
      setDraft(nextDraft);
      const result = await readJson("/task/preview", "task", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          schema_version: "cognitiveos.task-preview-request/0.1",
          task_contract_draft: nextDraft,
        }),
      });
      if (!result.ok) {
        setPreview(undefined);
        setError(chainErrorMessage("preview", result.status, result.body));
        setNeedsReconfirm(false);
        return;
      }
      setPreview(projectPreview(result.body));
      setNeedsReconfirm(reconfirm);
    } finally {
      setBusy(false);
    }
  }

  async function admit() {
    if (!draft || !preview || !interpretation) {
      return;
    }
    setBusy(true);
    setError(undefined);
    try {
      const principal = sessionPrincipal();
      const result = await readJson("/task/admit", "task", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          schema_version: "cognitiveos.task-admit-request/0.1",
          expected_current_epoch: 0,
          preview_digest: preview.previewDigest,
          task_contract_draft: draft,
          acceptance: {
            accepted_by: principal,
            accepted_digest: interpretation.interpretationDigest,
            interpretation_id: interpretation.interpretationId,
          },
        }),
      });
      if (!result.ok) {
        const failure = admitFailure(result.status, result.body);
        setError(`${failure.code} — ${failure.message}`);
        if (failure.requiresFreshPreview) {
          // Never an automatic retry: void this digest and take a fresh one,
          // which the operator must confirm again. The refusal reason stays on
          // screen through the recovery.
          setPreview(undefined);
          await makePreview(true, true);
        }
        return;
      }
      const view = projectAdmission(result.body);
      setAdmission(view);
      /*
       * The daemon has no per-principal task index, so a ref the client just
       * minted would otherwise be invisible until it appears in the envelope
       * list. Remembering it here is what makes Home and Work able to show it
       * at all — and it is session-local memory, labelled as such.
       */
      const admittedAtMs = Date.now();
      noteObservedTask(appProjections, {
        taskRef: view.taskRef,
        objective,
        observedAtMs: admittedAtMs,
        origin: "task/admit",
      });
      /*
       * Keep the chain this session just ran. The daemon exposes no route that
       * returns an intent record, an interpretation or a preview after the
       * fact, so this in-memory record is the only place the reviewed preview
       * digest survives — and it survives only for this tab. The Work detail
       * view reads it; nothing persists it.
       */
      noteSessionChain(appProjections, {
        taskRef: view.taskRef,
        admittedAtMs,
        intent: {
          userIntentRecordId: recordId ?? "",
          rawExpression: objective,
          recordedAt,
        },
        interpretation: {
          interpretationId: interpretation.interpretationId,
          interpretationDigest: interpretation.interpretationDigest,
          status: interpretation.status,
          materialAmbiguityCount: interpretation.materialAmbiguityCount,
          openAmbiguities: ambiguities
            .filter((item) => item.answer.trim() === "")
            .map((item) => ({
              id: item.id,
              question: item.question,
              material: item.material,
            })),
          recordedDecisions: answeredAssumptions,
          informationGaps: lines(gaps),
          supersededInterpretationIds: superseded,
        },
        preview: {
          previewDigest: preview.previewDigest,
          objective: preview.objective || draft.objective,
          conditionCount: preview.conditionCount,
          ephemeral: true,
        },
        admission: { ...view, acceptedBy: principal },
      });
    } finally {
      setBusy(false);
    }
  }

  if (admission) {
    return (
      <>
        <PageHeader title="New task" lede="The governed chain completed. Here is the receipt." />
        <AdmissionReceipt admission={admission} />
        <p className="cp-next">
          <button
            type="button"
            className="cp-button"
            onClick={() => navigate(`/work?task=${encodeURIComponent(admission.taskRef)}`)}
          >
            Back to Work
          </button>
        </p>
      </>
    );
  }

  return (
    <>
      <PageHeader
        title="New task"
        lede="Objective, interpretation, contract preview, admission — the daemon's own chain, in order. This browser mints no authority at any step."
      />
      <p className="cp-next">
        <Link to="/work">Back to Work</Link>
      </p>

      <form onSubmit={(event) => void describe(event)}>
        <h3 className="cp-section-title">Describe the work</h3>
        <label className="cp-field">
          Objective
          <input
            name="objective"
            value={objective}
            onChange={(event) => setObjective(event.target.value)}
          />
        </label>
        <label className="cp-field">
          Constraints (one per line)
          <textarea
            name="constraints"
            value={constraints}
            onChange={(event) => setConstraints(event.target.value)}
          />
        </label>
        <label className="cp-field">
          Forbidden (one per line)
          <textarea
            name="forbidden"
            value={forbidden}
            onChange={(event) => setForbidden(event.target.value)}
          />
        </label>
        <label className="cp-field">
          Information gaps — URI references, one per line
          <textarea name="gaps" value={gaps} onChange={(event) => setGaps(event.target.value)} />
        </label>
        <fieldset className="cp-fieldset">
          <legend className="cp-quiet">Ambiguities you already know about</legend>
          {ambiguities.map((item, index) => (
            <div key={item.id} className="cp-field">
              <input
                name={`ambiguity_question_${index}`}
                value={item.question}
                placeholder="What is unclear?"
                onChange={(event) =>
                  setAmbiguities((rows) =>
                    rows.map((row) =>
                      row.id === item.id ? { ...row, question: event.target.value } : row,
                    ),
                  )
                }
              />
              <label className="cp-field">
                <input
                  type="checkbox"
                  name={`ambiguity_material_${index}`}
                  checked={item.material}
                  onChange={(event) =>
                    setAmbiguities((rows) =>
                      rows.map((row) =>
                        row.id === item.id ? { ...row, material: event.target.checked } : row,
                      ),
                    )
                  }
                />{" "}
                Material — the work cannot be admitted until this is decided
              </label>
            </div>
          ))}
          <button
            type="button"
            className="cp-button"
            onClick={() =>
              setAmbiguities((rows) => [
                ...rows,
                { id: `amb-${uuidV7().slice(0, 8)}`, question: "", material: true, answer: "" },
              ])
            }
          >
            Add an ambiguity
          </button>
        </fieldset>
        <button type="submit" className="cp-button cp-button--primary" disabled={busy}>
          Record and interpret
        </button>
      </form>

      <HonestyNote>
        The daemon fixes the raw expression as a UserIntentRecord before anything interprets it,
        then derives the candidate&apos;s status from the material-ambiguity facts declared here —
        it never picks a status and never resolves an ambiguity by choosing the most likely
        reading. Declaring an ambiguity as material is therefore a real gate:{" "}
        <code>/task/admit</code> would refuse with <code>INTENT_CLARIFICATION_REQUIRED</code> while
        one stands.
      </HonestyNote>

      {error ? (
        <p className="cp-reason" role="alert">
          {error}
        </p>
      ) : null}

      {interpretation && !preview ? (
        <InterpretationReview
          interpretation={interpretation}
          ambiguities={ambiguities}
          informationGaps={lines(gaps)}
          assumptions={answeredAssumptions}
          onAnswer={(id, answer) =>
            setAmbiguities((rows) =>
              rows.map((row) => (row.id === id ? { ...row, answer } : row)),
            )
          }
          onReinterpret={() => void reinterpret()}
          onPreview={() => void makePreview(false)}
          busy={busy}
        />
      ) : null}

      {preview && draft && interpretation ? (
        <TaskPreview
          preview={preview}
          draft={draft}
          interpretationId={interpretation.interpretationId}
          principal={sessionPrincipal()}
          onAdmit={() => void admit()}
          onCancel={() => {
            setPreview(undefined);
            setNeedsReconfirm(false);
          }}
          busy={busy}
          needsReconfirm={needsReconfirm}
        />
      ) : null}
    </>
  );
}
