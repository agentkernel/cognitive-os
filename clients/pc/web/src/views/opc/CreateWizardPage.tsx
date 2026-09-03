import { useState, type FormEvent } from "react";
import { Link, useNavigate } from "react-router-dom";
import { readJson } from "../../api";
import { PageHeader } from "../../components/PageHeader";
import { HonestyNote } from "../../state/HonestyNote";
import { CreateAssistantChat } from "./CreateAssistantChat";

const STEPS = [
  { id: "create-init", title: "Charter" },
  { id: "create-process", title: "Process" },
  { id: "create-members", title: "Members" },
  { id: "create-test", title: "Verification" },
  { id: "create-joint", title: "Preview and write" },
] as const;

type StepId = (typeof STEPS)[number]["id"];

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
  }
  return `HTTP ${status}`;
}

function field(body: unknown, key: string): string | undefined {
  if (!body || typeof body !== "object") {
    return undefined;
  }
  const value = (body as Record<string, unknown>)[key];
  return typeof value === "string" && value.length > 0 ? value : undefined;
}

/**
 * Five-step create wizard (P12-T02). Local notes are not Project authority.
 * Minting requires management POST draft.create → preview.request → confirm.
 * Labels avoid fake Create project / Activate / Confirm chrome.
 */
export function CreateWizardPage() {
  const navigate = useNavigate();
  const [step, setStep] = useState(0);
  const [title, setTitle] = useState("");
  const [charter, setCharter] = useState("");
  const [process, setProcess] = useState("");
  const [members, setMembers] = useState("");
  const [verification, setVerification] = useState("");
  const [draftId, setDraftId] = useState<string | undefined>();
  const [previewId, setPreviewId] = useState<string | undefined>();
  const [previewDigest, setPreviewDigest] = useState<string | undefined>();
  const [projectId, setProjectId] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | undefined>();
  const current: StepId = STEPS[step].id;

  function goNext(event: FormEvent) {
    event.preventDefault();
    setError(undefined);
    if (step === 0 && (title.trim() === "" || charter.trim() === "")) {
      setError("Charter title and body are required before leaving this step. Nothing is written yet.");
      return;
    }
    setStep((value) => Math.min(value + 1, STEPS.length - 1));
  }

  async function requestPreview() {
    setBusy(true);
    setError(undefined);
    setPreviewId(undefined);
    setPreviewDigest(undefined);
    try {
      const created = await readJson("/management/project/v1/draft.create", "management", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          payload: title.trim(),
          charter: [
            `title: ${title.trim()}`,
            `process:\n${process.trim() || "(none)"}`,
            `members:\n${members.trim() || "(none — seating is a later card)"}`,
            `verification:\n${verification.trim() || "(none — independent verification is not this page)"}`,
            `charter:\n${charter.trim()}`,
          ].join("\n\n"),
        }),
      });
      if (!created.ok) {
        setError(errorMessage(created.status, created.body));
        return;
      }
      const nextDraft = field(created.body, "draft_id");
      if (!nextDraft) {
        setError("draft.create returned no draft_id. No Project was minted.");
        return;
      }
      setDraftId(nextDraft);
      const previewed = await readJson("/management/project/v1/preview.request", "management", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          subject_kind: "activation",
          subject_ref: nextDraft,
        }),
      });
      if (!previewed.ok) {
        setError(errorMessage(previewed.status, previewed.body));
        return;
      }
      const nextPreview = field(previewed.body, "preview_id");
      const nextDigest = field(previewed.body, "preview_digest");
      if (!nextPreview || !nextDigest) {
        setError("preview.request returned no digest-bound preview. No Project was minted.");
        return;
      }
      setPreviewId(nextPreview);
      setPreviewDigest(nextDigest);
    } finally {
      setBusy(false);
    }
  }

  async function writeProject() {
    if (!previewId || !previewDigest) {
      setError("Request a preview first. This page does not mint a Project without a digest.");
      return;
    }
    setBusy(true);
    setError(undefined);
    try {
      const written = await readJson("/management/project/v1/confirm", "management", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          preview_id: previewId,
          preview_digest: previewDigest,
        }),
      });
      if (!written.ok) {
        setError(errorMessage(written.status, written.body));
        return;
      }
      const nextProject = field(written.body, "new_ref");
      if (!nextProject) {
        setError("confirm returned no new_ref. Treat the Project as not minted.");
        return;
      }
      setProjectId(nextProject);
      navigate("/");
    } finally {
      setBusy(false);
    }
  }

  return (
    <section data-page="opc-create-wizard" data-step={current}>
      <PageHeader
        title="Create Project"
        lede="Five-step wizard. Local notes are not authority. Activation is digest-bound management HTTP."
      />
      <HonestyNote>
        Product origin is daemon-served hash /ui/. Vite is not the product origin.
        Process, members, and verification notes stay on this page until later
        cards. This wizard does not Activate, Approve, or write a Project until
        preview.request then confirm succeed.
      </HonestyNote>
      <ol className="cp-quiet" aria-label="Create steps">
        {STEPS.map((item, index) => (
          <li key={item.id} data-step-item={item.id} aria-current={index === step ? "step" : undefined}>
            {index + 1}. {item.title}
          </li>
        ))}
      </ol>
      {current === "create-init" ? (
        <form onSubmit={goNext}>
          <label className="cp-field">
            Title
            <input
              name="title"
              value={title}
              onChange={(event) => setTitle(event.target.value)}
            />
          </label>
          <label className="cp-field">
            Charter
            <textarea
              name="charter"
              value={charter}
              onChange={(event) => setCharter(event.target.value)}
              rows={8}
            />
          </label>
          <p className="cp-quiet">A draft exists only after preview on the last step.</p>
          <button type="submit" className="cp-button cp-button--primary">
            Continue
          </button>
        </form>
      ) : null}
      {current === "create-process" ? (
        <form onSubmit={goNext}>
          <label className="cp-field">
            Process notes
            <textarea
              name="process"
              value={process}
              onChange={(event) => setProcess(event.target.value)}
              rows={6}
            />
          </label>
          <p className="cp-quiet">Not a PlanRevision HTTP write. Notes fold into the charter blob.</p>
          <button type="button" className="cp-button" onClick={() => setStep(0)}>
            Back
          </button>{" "}
          <button type="submit" className="cp-button cp-button--primary">
            Continue
          </button>
        </form>
      ) : null}
      {current === "create-members" ? (
        <form onSubmit={goNext}>
          <label className="cp-field">
            Intended members
            <textarea
              name="members"
              value={members}
              onChange={(event) => setMembers(event.target.value)}
              rows={6}
            />
          </label>
          <p className="cp-quiet">Not roster.register or seating. Those remain a later card.</p>
          <button type="button" className="cp-button" onClick={() => setStep(1)}>
            Back
          </button>{" "}
          <button type="submit" className="cp-button cp-button--primary">
            Continue
          </button>
        </form>
      ) : null}
      {current === "create-test" ? (
        <form onSubmit={goNext}>
          <label className="cp-field">
            Verification notes
            <textarea
              name="verification"
              value={verification}
              onChange={(event) => setVerification(event.target.value)}
              rows={6}
            />
          </label>
          <p className="cp-quiet">
            This page cannot pass unknown verification. Independent verification
            is not this wizard.
          </p>
          <button type="button" className="cp-button" onClick={() => setStep(2)}>
            Back
          </button>{" "}
          <button type="submit" className="cp-button cp-button--primary">
            Continue
          </button>
        </form>
      ) : null}
      {current === "create-joint" ? (
        <div>
          <p className="cp-quiet">
            Title: {title.trim() || "(missing)"}. Charter length: {charter.trim().length}.
            {draftId ? ` Draft ${draftId}.` : " No draft yet."}
            {previewId ? ` Preview ${previewId}.` : ""}
          </p>
          <p className="cp-quiet">
            Request preview mints a digest-bound ApprovalPreview. Write Project
            posts that digest on management confirm. Chat cannot do this.
          </p>
          <button type="button" className="cp-button" onClick={() => setStep(3)} disabled={busy}>
            Back
          </button>{" "}
          <button
            type="button"
            className="cp-button"
            onClick={() => void requestPreview()}
            disabled={busy}
          >
            Request preview
          </button>{" "}
          <button
            type="button"
            className="cp-button cp-button--primary"
            onClick={() => void writeProject()}
            disabled={busy || !previewId || !previewDigest}
          >
            Write Project
          </button>
        </div>
      ) : null}
      {error ? (
        <p className="cp-stateview" role="alert" data-wizard-error="true">
          {error} No Project was invented locally.
        </p>
      ) : null}
      {projectId ? (
        <p className="cp-quiet">
          Daemon returned <code className="cp-mono">{projectId}</code>.
        </p>
      ) : null}
      <CreateAssistantChat step={current} title={title} />
      <p>
        <Link to="/">Back to Today</Link>
      </p>
    </section>
  );
}
