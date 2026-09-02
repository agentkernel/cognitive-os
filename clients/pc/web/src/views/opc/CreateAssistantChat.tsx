import { useEffect, useState, type FormEvent } from "react";
import { Link } from "react-router-dom";
import { readJson } from "../../api";
import {
  ASSISTANT_OBJECT_KINDS,
  ASSISTANT_STATUS_PATH,
  ASSISTANT_TURN_KINDS,
  ASSISTANT_TURN_PATH,
  assistantTurnReady,
  isProviderUnbound,
  projectAssistantStatus,
  projectAssistantTurn,
  type AssistantObjectKind,
  type AssistantStatusRow,
  type AssistantTurnKind,
  type AssistantTurnRow,
} from "../../data/projections/assistant";
import { hitlCanvasPath } from "../../data/projections/hitl";
import { httpErrorMessage } from "./httpError";

const DRAFT_CREATE_PATH = "/management/project/v1/draft.create";

export interface CreateAssistantChatProps {
  /** Wizard step id; picks the default object kind the assistant proposes. */
  step: string;
  /** Current wizard title, used to label the research draft the candidates land on. */
  title: string;
}

interface ChatEntry {
  kind: AssistantTurnKind;
  text: string;
  turn: AssistantTurnRow;
}

export function defaultObjectKind(step: string): AssistantObjectKind {
  switch (step) {
    case "create-init":
      return "business-brief";
    case "create-process":
      return "axis";
    case "create-members":
      return "roster";
    case "create-test":
      return "recipe";
    default:
      return "charter";
  }
}

function field(body: unknown, key: string): string | undefined {
  if (!body || typeof body !== "object") {
    return undefined;
  }
  const value = (body as Record<string, unknown>)[key];
  return typeof value === "string" && value.length > 0 ? value : undefined;
}

/**
 * Create-page assistant chat (P13-T03). The daemon runs exact Pi through its
 * Provider proxy and returns a candidate object chain with typed provenance;
 * this surface renders the reply and the chain as candidates only. When no
 * Provider is bound it renders a Settings pointer instead of a chat box: no
 * input, no key prompt, no silent bind. Nothing here Approves or writes.
 */
export function CreateAssistantChat({ step, title }: CreateAssistantChatProps) {
  const [status, setStatus] = useState<AssistantStatusRow | undefined>();
  const [statusError, setStatusError] = useState<string | undefined>();
  const [kind, setKind] = useState<AssistantTurnKind>("explain");
  const [objectKind, setObjectKind] = useState<AssistantObjectKind>(defaultObjectKind(step));
  const [text, setText] = useState("");
  const [researchTargets, setResearchTargets] = useState("");
  const [draftId, setDraftId] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | undefined>();
  const [entries, setEntries] = useState<ChatEntry[]>([]);

  useEffect(() => {
    setObjectKind(defaultObjectKind(step));
  }, [step]);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const response = await readJson(ASSISTANT_STATUS_PATH, "management");
        if (cancelled) {
          return;
        }
        if (!response.ok) {
          setStatusError(httpErrorMessage(response.status, response.body));
          return;
        }
        setStatus(projectAssistantStatus(response.body));
      } catch {
        if (!cancelled) {
          setStatusError("assistant.status is unavailable.");
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  async function ensureDraft(): Promise<string | undefined> {
    if (draftId) {
      return draftId;
    }
    const created = await readJson(DRAFT_CREATE_PATH, "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        payload: `assistant-research:${title.trim() || "untitled"}`,
        charter: `assistant research draft for: ${title.trim() || "(untitled)"}\n(candidates only; the Project draft is minted on the last step)`,
      }),
    });
    if (!created.ok) {
      setError(`${httpErrorMessage(created.status, created.body)} No research draft; nothing was sent to the assistant.`);
      return undefined;
    }
    const next = field(created.body, "draft_id");
    if (!next) {
      setError("draft.create returned no draft_id. Nothing was sent to the assistant.");
      return undefined;
    }
    setDraftId(next);
    return next;
  }

  async function send(event: FormEvent) {
    event.preventDefault();
    setError(undefined);
    const ready = assistantTurnReady(text);
    if (!ready.ok) {
      setError(ready.reason);
      return;
    }
    if (!status?.chatInput) {
      setError("Assistant input is closed until the daemon reports a bound Provider and configured Pi.");
      return;
    }
    setBusy(true);
    try {
      const draft = await ensureDraft();
      if (!draft) {
        return;
      }
      const targets = researchTargets
        .split(/\s+/)
        .map((item) => item.trim())
        .filter((item) => item.length > 0)
        .slice(0, 4);
      const body: Record<string, unknown> = {
        kind,
        draft_id: draft,
        object_kind: objectKind,
        payload: { text: text.trim() },
        provenance: { kind: "owner-stated" },
      };
      if (kind === "research") {
        body.tools = ["HttpFetchReadOnly"];
        body.research_targets = targets;
      }
      const response = await readJson(ASSISTANT_TURN_PATH, "management", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(body),
      });
      if (isProviderUnbound(response.status, response.body)) {
        setStatus(projectAssistantStatus({ status: "provider_unbound", chat_input: false }));
        return;
      }
      if (!response.ok) {
        setError(`${httpErrorMessage(response.status, response.body)} No candidate was registered. Chat cannot Approve.`);
        return;
      }
      const turn = projectAssistantTurn(response.body);
      if (!turn) {
        setError("assistant.turn returned no candidate_digest. Treat the reply as absent.");
        return;
      }
      setEntries((previous) => [...previous, { kind, text: text.trim(), turn }]);
      setText("");
    } finally {
      setBusy(false);
    }
  }

  if (statusError) {
    return (
      <section data-region="opc-create-assistant" data-assistant-state="unavailable">
        <h3 className="cp-section-title">Assistant</h3>
        <p className="cp-quiet" data-region="opc-create-assistant-unavailable">
          Assistant status is unavailable ({statusError}). No chat box is shown; nothing is
          inferred locally.
        </p>
      </section>
    );
  }

  if (!status) {
    return (
      <section data-region="opc-create-assistant" data-assistant-state="loading">
        <h3 className="cp-section-title">Assistant</h3>
        <p className="cp-quiet">Checking the daemon's assistant binding…</p>
      </section>
    );
  }

  if (status.status === "provider_unbound") {
    return (
      <section data-region="opc-create-assistant" data-assistant-state="provider-unbound">
        <h3 className="cp-section-title">Assistant</h3>
        <p data-region="opc-create-assistant-unbound">
          No model is connected to the assistant yet.{" "}
          <Link to={status.settingsRoute} className="cp-button">
            Open Settings to connect a Provider
          </Link>
        </p>
        <p className="cp-quiet">
          The chat does not accept keys and does not bind a model silently. Once a
          Provider is bound to the assistant in Settings, this page shows the chat.
        </p>
      </section>
    );
  }

  if (status.status !== "ready" || !status.chatInput) {
    return (
      <section data-region="opc-create-assistant" data-assistant-state="pi-unavailable">
        <h3 className="cp-section-title">Assistant</h3>
        <p className="cp-quiet" data-region="opc-create-assistant-pi-unavailable">
          A Provider is bound{status.modelId ? ` (${status.modelId})` : ""}, but the exact Pi
          runtime is not available on this daemon{status.piDetail ? `: ${status.piDetail}` : ""}.
          No chat box is shown; nothing is inferred locally.
        </p>
      </section>
    );
  }

  return (
    <section data-region="opc-create-assistant" data-assistant-state="ready">
      <h3 className="cp-section-title">Assistant</h3>
      <p className="cp-quiet">
        Bound model <code className="cp-mono">{status.modelId || "unknown"}</code>. Every reply is
        a daemon-registered candidate with typed provenance; nothing is written until you confirm
        on the canvas. Chat cannot Approve.
        {draftId ? (
          <>
            {" "}
            Candidates land on research draft <code className="cp-mono">{draftId}</code>.
          </>
        ) : null}
      </p>
      <ol data-region="opc-create-assistant-log" className="cp-quiet">
        {entries.map((entry, index) => (
          <li key={`${index}-${entry.turn.candidateDigest}`} data-turn-kind={entry.kind}>
            <p>
              <strong>You ({entry.kind})</strong>: {entry.text}
            </p>
            <p data-region="opc-create-assistant-reply">
              <strong>Assistant</strong>: {entry.turn.reply || "(no reply text)"}{" "}
              <span className="cp-quiet">
                · candidate <code className="cp-mono">{entry.turn.candidateDigest.slice(0, 12)}</code>
                {entry.turn.modelId ? ` · ${entry.turn.modelId}` : ""}
                {` · ${entry.turn.providerRoundTrips} Provider round trip${entry.turn.providerRoundTrips === 1 ? "" : "s"}`}
              </span>
            </p>
            {entry.turn.chain.length > 0 ? (
              <ul data-region="opc-create-assistant-chain">
                {entry.turn.chain.map((object) => (
                  <li key={object.objectKind} data-object-kind={object.objectKind}>
                    <strong>{object.objectKind}</strong>
                    {object.summary ? ` — ${object.summary}` : ""}
                    <ul>
                      {object.fields.map((chainField) => (
                        <li key={chainField.name} data-provenance={chainField.provenanceKind}>
                          {chainField.name}: {chainField.value}{" "}
                          <span className="cp-quiet">
                            [{chainField.provenanceKind}
                            {chainField.sourceUris.length > 0 ? `: ${chainField.sourceUris.join(", ")}` : ""}]
                          </span>
                        </li>
                      ))}
                    </ul>
                  </li>
                ))}
              </ul>
            ) : null}
            {entry.turn.refusedSources.length > 0 ? (
              <p className="cp-quiet" data-region="opc-create-assistant-refused">
                Not fetched (outside pinned read-only origins): {entry.turn.refusedSources.join(", ")}
              </p>
            ) : null}
            {entry.turn.previewId ? (
              <p className="cp-quiet">
                Preview {entry.turn.previewId} is announce-only.{" "}
                <Link to={hitlCanvasPath(entry.turn.previewId)}>Open on Projects canvas</Link>
              </p>
            ) : null}
          </li>
        ))}
      </ol>
      <form onSubmit={send} data-region="opc-create-assistant-form">
        <label className="cp-field">
          <span>Turn</span>
          <select
            name="assistant_kind"
            value={kind}
            onChange={(event) => setKind(event.target.value as AssistantTurnKind)}
          >
            {ASSISTANT_TURN_KINDS.map((item) => (
              <option key={item} value={item}>
                {item}
              </option>
            ))}
          </select>
        </label>
        <label className="cp-field">
          <span>Object</span>
          <select
            name="assistant_object_kind"
            value={objectKind}
            onChange={(event) => setObjectKind(event.target.value as AssistantObjectKind)}
          >
            {ASSISTANT_OBJECT_KINDS.map((item) => (
              <option key={item} value={item}>
                {item}
              </option>
            ))}
          </select>
        </label>
        <label className="cp-field">
          <span>Ask the assistant</span>
          <textarea
            name="assistant_text"
            value={text}
            onChange={(event) => setText(event.target.value)}
            rows={3}
          />
        </label>
        {kind === "research" ? (
          <label className="cp-field">
            <span>Research targets (HTTPS, pinned read-only origins only, up to 4)</span>
            <input
              name="assistant_research_targets"
              value={researchTargets}
              onChange={(event) => setResearchTargets(event.target.value)}
              autoComplete="off"
            />
          </label>
        ) : null}
        <p>
          <button type="submit" className="cp-button" disabled={busy}>
            Send to assistant
          </button>
        </p>
      </form>
      {error ? (
        <p className="cp-error" role="alert" data-region="opc-create-assistant-error">
          {error}
        </p>
      ) : null}
    </section>
  );
}
