/*
 * Legacy pages — moved VERBATIM from the pre-refactor App.tsx
 * (origin/main @ 74eea38) so current real capability is preserved until
 * each space's redesign wave lands. Providers/Bindings migrated to
 * views/providers in W2; Tasks remains here until W4/W5.
 * This page intentionally keeps its original styling hooks (styles.css)
 * and local helpers. Do not extend this file; new work goes to views/.
 */

import { useEffect, useMemo, useState } from "react";
import { readJson } from "../../api";
import { asList, asRecord } from "../../data/projections";
import {
  escapeUntrustedText,
  inferCompletionFromObservation,
  redactSecrets,
  unavailableLabel,
} from "../../policy";
import { sessionPrincipal } from "../../session";
import { interpretCandidate, workspaceSearchDraft } from "../../taskDraft";
import { createWatchController } from "../../watch";
import { isWatchResumeStale, latestSequence, parseSse } from "../../watchSse";

/* ---------- legacy local helpers (verbatim) ---------- */

type LoadState = {
  status: "loading" | "ready" | "empty" | "denied" | "disconnected" | "unknown" | "not-run";
  ms?: number;
  body?: unknown;
  message?: string;
};

async function load(path: string, channel: "management" | "task"): Promise<LoadState> {
  try {
    const result = await readJson(path, channel);
    if (result.status === 401 || result.status === 403) {
      return { status: "denied", ms: result.ms, body: result.body, message: `HTTP ${result.status}` };
    }
    if (!result.ok) {
      return { status: "unknown", ms: result.ms, body: result.body, message: `HTTP ${result.status}` };
    }
    const list = asList(result.body, ["items", "accounts", "bindings", "events", "alerts", "models"]);
    if (list.length === 0 && JSON.stringify(result.body).includes("[]")) {
      return { status: "empty", ms: result.ms, body: result.body };
    }
    return { status: "ready", ms: result.ms, body: result.body };
  } catch (error) {
    return {
      status: "disconnected",
      message: error instanceof Error ? error.message : "disconnected",
    };
  }
}

const STATE_LABELS: Record<LoadState["status"], string> = {
  loading: "Loading…",
  ready: "Ready",
  empty: "Empty (authoritative)",
  denied: "Denied — issue a session below or on the Session page",
  disconnected: "Daemon unreachable",
  unknown: "Unexpected response",
  "not-run": "Not run",
};

function StateNote({ state }: { state: LoadState }) {
  return (
    <p className={`state-note state-${state.status}`} role="status">
      <span className="state-dot" aria-hidden="true" />
      {STATE_LABELS[state.status]}
      {state.ms != null ? ` · ${state.ms} ms` : ""}
      {state.message ? ` · ${state.message}` : ""}
    </p>
  );
}

function JsonPanel({ title, value }: { title: string; value: unknown }) {
  return (
    <section className="panel">
      <h3>{title}</h3>
      <pre>{JSON.stringify(redactSecrets(value ?? {}), null, 2)}</pre>
    </section>
  );
}

function PageHeader({ title, description }: { title: string; description?: string }) {
  return (
    <header className="page-head">
      <h2>{title}</h2>
      {description ? <p className="lede">{description}</p> : null}
    </header>
  );
}

/* ---------- Tasks (verbatim) ---------- */

export function TasksPage() {
  const [effects, setEffects] = useState<LoadState>({ status: "loading" });
  const [observation, setObservation] = useState<LoadState>({ status: "loading" });
  const [evidence, setEvidence] = useState<LoadState>({ status: "empty" });
  const watch = useMemo(() => createWatchController(), []);
  const [watchState, setWatchState] = useState(watch.state);
  const [taskRef, setTaskRef] = useState("");
  const [objective, setObjective] = useState("search the workspace for needle");
  const [previewDigest, setPreviewDigest] = useState("");
  const [interpretationId, setInterpretationId] = useState("");
  const [acceptedDigest, setAcceptedDigest] = useState("");
  const [draft, setDraft] = useState<ReturnType<typeof workspaceSearchDraft> | null>(null);
  const [runMessage, setRunMessage] = useState("Admit uses the typed Task channel only.");
  const [resumeFrom, setResumeFrom] = useState<number | undefined>(undefined);

  async function refresh(ref: string) {
    if (!ref) {
      return;
    }
    const encoded = encodeURIComponent(ref);
    setEffects(await load(`/task/effects?task_ref=${encoded}`, "task"));
    setObservation(await load(`/task/observation?family=o13&task_ref=${encoded}`, "task"));
    const nextEvidence = await load(`/task/evidence?task_ref=${encoded}`, "task");
    setEvidence(nextEvidence);
    const inferred = inferCompletionFromObservation({
      processExit: 0,
      providerResponse: observation.body,
      httpReceipt: nextEvidence.body,
      streamClosed: true,
    });
    if (inferred !== "unknown") {
      watch.noteGap();
    }
    setWatchState(watch.state);
  }

  async function pollWatch() {
    const path =
      resumeFrom == null ? "/task/watch" : `/task/watch?resume_from=${encodeURIComponent(String(resumeFrom))}`;
    const result = await readJson(path, "task");
    if (isWatchResumeStale(result.status, result.body)) {
      watch.noteGap();
      setWatchState(watch.state);
      setResumeFrom(undefined);
      setRunMessage("Watch cursor gap: snapshot reload required. Completion stays unknown.");
      return;
    }
    const text =
      typeof result.body === "string"
        ? result.body
        : typeof asRecord(result.body).raw === "string"
          ? String(asRecord(result.body).raw)
          : JSON.stringify(result.body ?? {});
    const frames = parseSse(text);
    for (const frame of frames) {
      const id = frame.id ?? JSON.stringify(frame.data);
      watch.accept({
        id,
        cursor: frame.id ?? String(latestSequence(frames) ?? ""),
        kind: frame.event,
      });
    }
    const latest = latestSequence(frames);
    if (latest != null) {
      setResumeFrom(latest);
    }
    setWatchState(watch.state);
  }

  async function startTask(event: React.FormEvent) {
    event.preventDefault();
    const principal = sessionPrincipal();
    const recorded = await readJson("/task/intent.record", "task", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        schema_version: "cognitiveos.task-intent-record-request/0.1",
        conversation_or_scope_ref: "conversation://personal/web-ui",
        raw_expression: objective,
      }),
    });
    if (!recorded.ok) {
      setRunMessage(`intent.record HTTP ${recorded.status}`);
      return;
    }
    const userIntentRecordId = String(asRecord(recorded.body).user_intent_record_id ?? "");
    const interpreted = await readJson("/task/intent.interpret", "task", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        schema_version: "cognitiveos.task-intent-interpret-request/0.1",
        user_intent_record_id: userIntentRecordId,
        candidate: interpretCandidate(objective),
      }),
    });
    if (!interpreted.ok) {
      setRunMessage(`intent.interpret HTTP ${interpreted.status}`);
      return;
    }
    const nextDraft = workspaceSearchDraft(objective);
    setDraft(nextDraft);
    const previewed = await readJson("/task/preview", "task", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        schema_version: "cognitiveos.task-preview-request/0.1",
        task_contract_draft: nextDraft,
      }),
    });
    if (!previewed.ok) {
      setRunMessage(`preview HTTP ${previewed.status}`);
      return;
    }
    const digest = String(asRecord(previewed.body).preview_digest ?? "");
    setPreviewDigest(digest);
    setInterpretationId(String(asRecord(interpreted.body).interpretation_id ?? ""));
    setAcceptedDigest(String(asRecord(interpreted.body).interpretation_digest ?? ""));
    setRunMessage(
      `Preview ready for ${principal}. Digest bound. Confirm admit; HTTP 200 is not Task completion.`,
    );
  }

  async function admitTask() {
    if (!draft || !previewDigest || !interpretationId) {
      setRunMessage("Preview first.");
      return;
    }
    const principal = sessionPrincipal();
    const admitted = await readJson("/task/admit", "task", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        schema_version: "cognitiveos.task-admit-request/0.1",
        expected_current_epoch: 0,
        preview_digest: previewDigest,
        task_contract_draft: draft,
        acceptance: {
          accepted_by: principal,
          accepted_digest: acceptedDigest,
          interpretation_id: interpretationId,
        },
      }),
    });
    const ref = String(asRecord(admitted.body).task_ref ?? draft.task_ref);
    setTaskRef(ref);
    setRunMessage(
      admitted.ok
        ? `Admitted ${ref}. Watch and projections are observations, not completion.`
        : `admit HTTP ${admitted.status} ${String(asRecord(admitted.body).code ?? "")}`,
    );
    if (admitted.ok) {
      await refresh(ref);
      await pollWatch();
    }
  }

  useEffect(() => {
    if (taskRef) {
      void refresh(taskRef);
    }
  }, [taskRef]);

  return (
    <>
      <PageHeader title="Tasks, Effects, Evidence" />
      <p className="muted">
        Cancel is {unavailableLabel("task-cancel")}. Detach does not cancel a Task or stop an Agent.
        Process/Provider/Pi/HTTP receipt is not Task completion.
      </p>
      <form onSubmit={(event) => void startTask(event)}>
        <h3>Start a governed Task</h3>
        <label>
          Objective
          <input
            name="objective"
            value={objective}
            onChange={(event) => setObjective(event.target.value)}
          />
        </label>
        <button type="submit">Record, interpret, and preview</button>
        <button type="button" onClick={() => void admitTask()}>
          Confirm admit
        </button>
      </form>
      <p className="muted">
        Preview digest: {previewDigest || "none"}. Interpretation: {interpretationId || "none"}.
      </p>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          const next = String(new FormData(event.currentTarget).get("task_ref") ?? "");
          setTaskRef(next);
        }}
      >
        <label>
          Task ref
          <input name="task_ref" defaultValue={taskRef} key={taskRef} />
        </label>
        <button type="submit">Load projections</button>
        <button
          type="button"
          onClick={() => {
            void pollWatch();
          }}
        >
          Watch poll
        </button>
        <button
          type="button"
          onClick={() => {
            watch.reconnect();
            setResumeFrom(undefined);
            setWatchState(watch.state);
          }}
        >
          Reconnect snapshot
        </button>
        <button
          type="button"
          onClick={() => {
            watch.noteGap();
            setWatchState(watch.state);
          }}
        >
          Simulate cursor gap
        </button>
        <button
          type="button"
          onClick={() => {
            watch.detach();
            setWatchState(watch.state);
          }}
        >
          Detach observation
        </button>
      </form>
      <p className="live" role="status" aria-live="polite">
        Watch {watchState}. Completion from observation remains unknown. {runMessage}
      </p>
      <StateNote state={effects} />
      <JsonPanel title="Effects" value={effects.body} />
      <JsonPanel title="Evidence" value={evidence.body} />
      <JsonPanel
        title="Observation (escaped)"
        value={escapeUntrustedText(JSON.stringify(observation.body ?? {}, null, 2))}
      />
    </>
  );
}
