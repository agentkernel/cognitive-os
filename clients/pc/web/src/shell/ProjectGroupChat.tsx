import { useCallback, useEffect, useMemo, useState, type FormEvent, type KeyboardEvent } from "react";
import { Link } from "react-router-dom";
import { readJson } from "../api";
import { hitlCanvasPath } from "../data/projections/hitl";
import {
  CHAT_POST_PATH,
  CHAT_SETTINGS_ROUTE,
  chatDraftReady,
  chatThreadPath,
  insertMention,
  isChatSecretRefused,
  parseLeadingMention,
  parsePlanStageLines,
  planRevisionProposal,
  projectChatPost,
  projectChatThread,
  type ChatPostView,
  type ChatThreadView,
} from "../data/projections/projectChat";
import { HonestyNote } from "../state/HonestyNote";
import { httpErrorMessage } from "../views/opc/httpError";

export interface ProjectGroupChatProps {
  projectId: string;
  /** Kept mounted while hidden so the unsent draft survives a layer switch. */
  hidden?: boolean;
}

type ThreadState =
  | { status: "loading" }
  | { status: "error"; message: string }
  | { status: "ready"; thread: ChatThreadView };

function authorLabel(author: string, kind: string, handle: string | undefined): string {
  if (author === "owner") {
    return "Owner";
  }
  if (author === "manager") {
    return `manager${handle ? ` (${handle})` : ""} · ${kind}`;
  }
  return `member${handle ? ` @${handle}` : ""} · ${kind}`;
}

/**
 * Project group chat (P13-T06): Owner / manager / Members in the right rail.
 * The client posts the Owner's message with its parsed mention; the daemon
 * routes it and enforces speech rules by record kind. `@` chips only edit the
 * unsent draft. Previews are announced here and confirmed on the Projects
 * canvas — there is no Approve control in this component.
 */
export function ProjectGroupChat({ projectId, hidden = false }: ProjectGroupChatProps) {
  const [state, setState] = useState<ThreadState>({ status: "loading" });
  const [draft, setDraft] = useState("");
  const [planLines, setPlanLines] = useState("");
  const [attachPlan, setAttachPlan] = useState(false);
  const [taskStageId, setTaskStageId] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | undefined>();
  const [settingsPointer, setSettingsPointer] = useState(false);
  const [lastPost, setLastPost] = useState<ChatPostView | undefined>();

  const load = useCallback(async () => {
    try {
      const response = await readJson(chatThreadPath(projectId), "management");
      if (!response.ok) {
        setState({ status: "error", message: httpErrorMessage(response.status, response.body) });
        return;
      }
      setState({ status: "ready", thread: projectChatThread(response.body) });
    } catch {
      setState({ status: "error", message: "chat.thread is unavailable." });
    }
  }, [projectId]);

  useEffect(() => {
    setState({ status: "loading" });
    setDraft("");
    setPlanLines("");
    setAttachPlan(false);
    setTaskStageId("");
    setError(undefined);
    setSettingsPointer(false);
    setLastPost(undefined);
    void load();
  }, [load]);

  const participants = state.status === "ready" ? state.thread.participants : [];
  const parsed = useMemo(() => parseLeadingMention(draft, participants), [draft, participants]);
  const targetMember = participants.find(
    (p) => p.role === "member" && p.employeeId === parsed.targetEmployeeId,
  );
  const hasManager = participants.some((p) => p.role === "manager");

  function routeHint(): string {
    if (parsed.mention === "manager") {
      return attachPlan
        ? "Routes to the manager as a PlanRevision candidate → canvas preview. Chat cannot approve."
        : "Routes to the manager for an observed-facts briefing (manager-default speech).";
    }
    if (parsed.mention === "member") {
      return `Routes only to @${parsed.handle}'s own Task as a task-revision candidate → canvas preview.`;
    }
    if (!parsed.known) {
      return `@${parsed.handle} is not a participant here; the message stays conversational.`;
    }
    return hasManager
      ? "Un-addressed: the manager speaks by default from observed facts."
      : "No manager is seated yet; the message is recorded as conversation.";
  }

  function onMention(handle: string) {
    setDraft((current) => insertMention(current, handle));
  }

  async function send(event?: FormEvent) {
    event?.preventDefault();
    setError(undefined);
    setSettingsPointer(false);
    const ready = chatDraftReady(draft);
    if (!ready.ok) {
      setError(ready.reason);
      setSettingsPointer(ready.settings === true);
      return;
    }
    const body: Record<string, unknown> = {
      project_id: projectId,
      body: draft.trim(),
      mention: parsed.mention,
    };
    if (parsed.mention !== "none" && parsed.targetEmployeeId.length > 0) {
      body.target_employee_id = parsed.targetEmployeeId;
    }
    if (parsed.mention === "manager" && attachPlan) {
      const lines = parsePlanStageLines(planLines);
      if (!lines.ok) {
        setError(lines.reason);
        return;
      }
      body.proposal = planRevisionProposal(lines.stages);
    }
    if (parsed.mention === "member" && targetMember && targetMember.stageIds.length > 1) {
      if (taskStageId.length === 0) {
        setError(`@${targetMember.handle} owns several stages; pick the stage this redirect is bounded to.`);
        return;
      }
      body.proposal = { kind: "task-revision", stage_id: taskStageId, objective: draft.trim() };
    }
    setBusy(true);
    try {
      const response = await readJson(CHAT_POST_PATH, "management", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(body),
      });
      if (isChatSecretRefused(response.status, response.body)) {
        setSettingsPointer(true);
        setError("The daemon refused secret-shaped material; nothing was posted or archived.");
        return;
      }
      if (!response.ok) {
        setError(`${httpErrorMessage(response.status, response.body)} Nothing was routed. Chat cannot approve.`);
        return;
      }
      const posted = projectChatPost(response.body);
      if (!posted) {
        setError("chat.post returned no turn_id; treat the message as not posted.");
        return;
      }
      setLastPost(posted);
      setDraft("");
      setPlanLines("");
      setAttachPlan(false);
      setTaskStageId("");
      await load();
    } finally {
      setBusy(false);
    }
  }

  function onDraftKey(event: KeyboardEvent<HTMLTextAreaElement>) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      void send();
    }
  }

  return (
    <section
      data-region="opc-group-chat"
      data-project-id={projectId}
      hidden={hidden}
      aria-label="Project group chat"
    >
      <h3 className="cp-section-title">Project group · Owner / manager / Members</h3>
      <HonestyNote>
        Group conversation for this Project. The daemon routes @manager and @member;
        manager speaks by default, Members only when mentioned, delivering, handing off,
        blocked, or requesting a decision. Candidates are announced here and confirmed on
        the Projects canvas. Chat has no Approve and never carries a preview digest.
      </HonestyNote>
      {state.status === "loading" ? (
        <p className="cp-quiet" data-region="opc-group-chat-loading">
          Loading the group thread…
        </p>
      ) : null}
      {state.status === "error" ? (
        <p className="cp-error" data-region="opc-group-chat-error">
          Group thread unavailable: {state.message}
        </p>
      ) : null}
      {state.status === "ready" ? (
        <>
          <div role="group" aria-label="Participants" data-region="opc-group-chat-participants">
            {participants.map((p) => (
              <span key={`${p.role}:${p.handle}`} data-participant-role={p.role} className="cp-quiet">
                {p.role === "owner" ? "Owner" : `@${p.handle}`}
                {p.role !== "owner" ? ` (${p.state})` : ""}{" "}
              </span>
            ))}
          </div>
          <ol data-region="opc-group-chat-thread" className="cp-quiet">
            {state.thread.rows.length === 0 ? (
              <li data-region="opc-group-chat-empty">
                No delivered messages yet. Filtered Member chatter never appears here.
              </li>
            ) : null}
            {state.thread.rows.map((row) => {
              const speaker = participants.find(
                (p) => p.employeeId.length > 0 && p.employeeId === row.employeeId,
              );
              return (
                <li key={row.rowId} data-author={row.author} data-kind={row.kind}>
                  <p>
                    <strong>{authorLabel(row.author, row.kind, speaker?.handle)}</strong>: {row.body}
                  </p>
                  {row.routing.length > 0 && row.routing !== "conversational" ? (
                    <p className="cp-quiet" data-region="opc-group-chat-routing">
                      routed: {row.routing}
                      {row.targetStageId ? ` · stage ${row.targetStageId}` : ""}
                      {row.candidateDigest
                        ? ` · ${row.candidateKind} candidate ${row.candidateDigest.slice(0, 12)}`
                        : ""}
                    </p>
                  ) : null}
                  {row.previewId ? (
                    <p className="cp-quiet" data-region="opc-group-chat-preview">
                      Preview {row.previewId} is announce-only.{" "}
                      <Link to={hitlCanvasPath(row.previewId, projectId)}>Open on Projects canvas</Link>
                    </p>
                  ) : null}
                  {row.receiptRef ? (
                    <p className="cp-quiet" data-region="opc-group-chat-receipt">
                      Receipt {row.receiptRef}
                      {row.appliedRef ? ` → applied ${row.appliedRef}` : ""}. Applied on the canvas,
                      not in chat.
                    </p>
                  ) : null}
                </li>
              );
            })}
          </ol>
          {state.thread.truncated ? (
            <p className="cp-quiet">Older rows are not loaded (bounded thread page).</p>
          ) : null}
          <form onSubmit={send} data-region="opc-group-chat-form">
            <div role="group" aria-label="Insert a mention into the unsent draft" data-region="opc-group-chat-mentions">
              {participants
                .filter((p) => p.role !== "owner")
                .map((p) => (
                  <button
                    key={`mention:${p.handle}`}
                    type="button"
                    className="cp-button"
                    data-mention={p.handle}
                    onClick={() => onMention(p.handle)}
                  >
                    @{p.handle}
                  </button>
                ))}
            </div>
            <label className="cp-field">
              <span>Message to the Project group</span>
              <textarea
                name="group_chat_draft"
                value={draft}
                onChange={(event) => setDraft(event.target.value)}
                onKeyDown={onDraftKey}
                rows={3}
                placeholder="Ask the manager, or redirect a Member's work with @…"
              />
            </label>
            <p className="cp-quiet" data-region="opc-group-chat-route-hint">
              {routeHint()}
            </p>
            {parsed.mention === "manager" ? (
              <details
                data-region="opc-group-chat-plan"
                open={attachPlan}
                onToggle={(event) => setAttachPlan((event.target as HTMLDetailsElement).open)}
              >
                <summary>Attach a plan revision (candidate → canvas preview)</summary>
                <label className="cp-field">
                  <span>One stage per line: stage_id | title | responsible_slot | objective</span>
                  <textarea
                    name="plan_stage_lines"
                    value={planLines}
                    onChange={(event) => setPlanLines(event.target.value)}
                    rows={4}
                  />
                </label>
              </details>
            ) : null}
            {parsed.mention === "member" && targetMember && targetMember.stageIds.length > 1 ? (
              <label className="cp-field">
                <span>Bound to which of @{targetMember.handle}'s stages</span>
                <select
                  name="task_stage_id"
                  value={taskStageId}
                  onChange={(event) => setTaskStageId(event.target.value)}
                >
                  <option value="">choose a stage</option>
                  {targetMember.stageIds.map((stageId) => (
                    <option key={stageId} value={stageId}>
                      {stageId}
                    </option>
                  ))}
                </select>
              </label>
            ) : null}
            <p>
              <button type="submit" className="cp-button" disabled={busy}>
                Send to group
              </button>
            </p>
          </form>
          {lastPost ? (
            <p className="cp-quiet" data-region="opc-group-chat-posted" data-routing={lastPost.routing}>
              Posted · routed {lastPost.routing}
              {lastPost.candidateDigest
                ? ` · ${lastPost.candidateKind} candidate ${lastPost.candidateDigest.slice(0, 12)}`
                : ""}
              {lastPost.previewId ? (
                <>
                  {" "}
                  · preview {lastPost.previewId} announce-only ·{" "}
                  <Link to={hitlCanvasPath(lastPost.previewId, projectId)}>Open on Projects canvas</Link>
                </>
              ) : null}
              {lastPost.reply ? (
                <span data-region="opc-group-chat-reply">
                  {" "}
                  · {lastPost.reply.role} spoke ({lastPost.reply.reason}): {lastPost.reply.body}
                </span>
              ) : (
                <span> · reply: {lastPost.replyReason}</span>
              )}
            </p>
          ) : null}
        </>
      ) : null}
      {settingsPointer ? (
        <p data-region="opc-group-chat-settings-pointer">
          Keys are not accepted in chat.{" "}
          <Link to={CHAT_SETTINGS_ROUTE} className="cp-button">
            Open Settings to connect a Provider
          </Link>{" "}
          <span className="cp-quiet">SecretStore takeover: the chat never stores or forwards a key.</span>
        </p>
      ) : null}
      {error ? (
        <p className="cp-error" role="alert" data-region="opc-group-chat-error-message">
          {error}
        </p>
      ) : null}
    </section>
  );
}
