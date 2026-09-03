import { useCallback, useEffect, useState, type FormEvent } from "react";
import { Link, useParams } from "react-router-dom";
import { readJson } from "../../api";
import { PageHeader } from "../../components/PageHeader";
import { EmptyState } from "../../components/states";
import { hitlCanvasPath } from "../../data/projections/hitl";
import {
  ARTIFACT_STAGE_TEST_PATH,
  EXTERNAL_SEND_REQUEST_PATH,
  OUTPUT_EXPORT_PATH,
  OUTPUTS_PATH,
  RUN_ACCEPTANCE_REQUEST_PATH,
  acceptanceOfferable,
  lastRingStageId,
  outputDetailKey,
  outputOpenPath,
  outputsKey,
  publicationPacketKey,
  type OutputArtifactRow,
  type OutputDetailRow,
  type PublicationPacketRow,
} from "../../data/projections/outputs";
import {
  projectAxisKey,
  projectRosterKey,
  type ProjectAxisStageRow,
  type ProjectRosterRow,
} from "../../data/projections/projectWork";
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { DaemonReadPanel } from "./DaemonReadPanel";
import { httpErrorMessage } from "./httpError";
import {
  loadOutputDetail,
  loadProjectAxis,
  loadProjectOutputs,
  loadProjectRoster,
  loadPublicationPacket,
} from "./loadOpcReads";
import { ProjectWorkNav } from "./ProjectWorkNav";

/**
 * Outputs — select-then-view over real CAS-backed Attempt artifacts
 * (P13-T04). Open reads the bytes from the daemon CAS; export writes a copy
 * into Personal Home `data/` that is never authority. Close-out (verify state
 * + 「验收，回 Today」) is offered only on the last ring and goes through the
 * daemon ApprovalPreview on the Project canvas; chat has no Confirm. The
 * publication package is the AUTONOMY packet preview: planned ≠ published,
 * and there is no Publish button.
 */
export function ProjectOutputsPage() {
  const { projectId = "" } = useParams();
  const outputs = useProjection<OutputArtifactRow[]>(outputsKey(projectId));
  const axis = useProjection<ProjectAxisStageRow[]>(projectAxisKey(projectId));
  const roster = useProjection<ProjectRosterRow[]>(projectRosterKey(projectId));
  const [selectedId, setSelectedId] = useState<string | undefined>();
  const refresh = useCallback(async () => {
    if (projectId.length === 0) {
      return;
    }
    await loadProjectOutputs(projectId);
    await loadProjectAxis(projectId);
    await loadProjectRoster(projectId);
  }, [projectId]);
  useEffect(() => {
    void refresh();
    setSelectedId(undefined);
  }, [refresh]);
  const selected = (outputs.data ?? []).find((row) => row.artifactId === selectedId);
  const lastRing = lastRingStageId(axis.data ?? []);

  return (
    <section data-page="opc-project-outputs">
      <PageHeader
        title="Project outputs"
        lede="Real Attempt artifacts from the daemon CAS. Select one, then view. Files are not Project authority."
      />
      <HonestyNote>
        Product origin is daemon-served hash /ui/. GET {OUTPUTS_PATH} lists
        CAS-referenced artifacts with their independent-verifier state; the
        child's own “done” is never completion. Knowledge Vault files are not this
        list. Planned is not published; this page has no Publish and chat has no
        Confirm.
      </HonestyNote>
      <p className="cp-quiet">
        <Link to="/projects">Projects list</Link>
        {projectId ? (
          <>
            {" "}
            · <code className="cp-mono">{projectId}</code>
          </>
        ) : null}
      </p>
      {projectId ? <ProjectWorkNav projectId={projectId} /> : null}
      <DaemonReadPanel
        projection={outputs}
        surface="Project outputs"
        emptyTitle="Project outputs: no openable artifact yet"
        emptyBody="No terminal Attempt has produced a deliverable for this Project. Finish the current ring; this is not a Knowledge file list and not a fake publish queue."
        region="opc-project-outputs"
      >
        <table className="cp-table">
          <caption className="cp-quiet">GET {OUTPUTS_PATH} — newest first</caption>
          <thead>
            <tr>
              <th>Artifact</th>
              <th>Format</th>
              <th>Freshness</th>
              <th>Verify</th>
              <th>Stage</th>
              <th>Accepted</th>
            </tr>
          </thead>
          <tbody>
            {(outputs.data ?? []).map((artifact) => (
              <tr key={artifact.artifactId} data-row-key={artifact.artifactId}>
                <td>
                  <button
                    type="button"
                    className="cp-button"
                    aria-pressed={selectedId === artifact.artifactId}
                    onClick={() => setSelectedId(artifact.artifactId)}
                  >
                    {artifact.artifactId}
                  </button>
                </td>
                <td>{artifact.format}</td>
                <td>{artifact.freshness}</td>
                <td>{artifact.verificationStatus}</td>
                <td>{artifact.stageId}</td>
                <td>{artifact.acceptedAt}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {selected && projectId ? (
          <SelectedOutput
            projectId={projectId}
            artifact={selected}
            lastRing={lastRing}
            stages={axis.data ?? []}
            roster={roster.data ?? []}
            onWritten={() => {
              void refresh();
            }}
          />
        ) : (
          <EmptyState title="Project outputs: no output selected">
            Pick an artifact on the left. The first one is not opened by default.
            Files are not Project authority.
          </EmptyState>
        )}
      </DaemonReadPanel>
    </section>
  );
}

function SelectedOutput({
  projectId,
  artifact,
  lastRing,
  stages,
  roster,
  onWritten,
}: {
  projectId: string;
  artifact: OutputArtifactRow;
  lastRing: string | undefined;
  stages: ProjectAxisStageRow[];
  roster: ProjectRosterRow[];
  onWritten: () => void;
}) {
  const detail = useProjection<OutputDetailRow[]>(outputDetailKey(artifact.artifactId));
  const packet = useProjection<PublicationPacketRow[]>(
    publicationPacketKey(projectId, artifact.artifactId),
  );
  const [opened, setOpened] = useState<string | undefined>();
  const [openError, setOpenError] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);
  const [writeError, setWriteError] = useState<string | undefined>();
  const [exportPath, setExportPath] = useState<string | undefined>();
  const [stageTestFact, setStageTestFact] = useState<string | undefined>();
  const [acceptancePreview, setAcceptancePreview] = useState<string | undefined>();
  const [sendPreview, setSendPreview] = useState<string | undefined>();
  const [recipients, setRecipients] = useState("");
  const responsibleStages = roster
    .filter((row) => row.employeeId === artifact.employeeId)
    .flatMap((row) => row.responsibleStageIds.split(",").map((id) => id.trim()))
    .filter((id) => id.length > 0 && id !== "unknown");
  const stageChoices = responsibleStages.length > 0 ? responsibleStages : stages.map((s) => s.stageId);
  const [stageChoice, setStageChoice] = useState<string>("");
  const verified = artifact.verificationStatus === "passed";

  useEffect(() => {
    setOpened(undefined);
    setOpenError(undefined);
    setWriteError(undefined);
    setExportPath(undefined);
    setStageTestFact(undefined);
    setAcceptancePreview(undefined);
    setSendPreview(undefined);
    setRecipients("");
    setStageChoice("");
    void loadOutputDetail(artifact.artifactId);
    if (artifact.verificationStatus === "passed") {
      void loadPublicationPacket(projectId, artifact.artifactId);
    }
  }, [artifact.artifactId, artifact.verificationStatus, projectId]);

  const row = detail.data?.[0];
  const packetRow = packet.data?.[0];

  async function openBytes() {
    setOpenError(undefined);
    try {
      const read = await readJson(outputOpenPath(artifact.artifactId), "management");
      if (!read.ok) {
        setOpenError(httpErrorMessage(read.status, read.body));
        return;
      }
      const body = read.body;
      const text =
        body && typeof body === "object" && typeof (body as { raw?: unknown }).raw === "string"
          ? (body as { raw: string }).raw
          : typeof body === "string"
            ? body
            : JSON.stringify(body);
      setOpened(text);
    } catch (error) {
      setOpenError(error instanceof Error ? error.message : "daemon unreachable");
    }
  }

  async function postWrite(
    path: string,
    body: Record<string, unknown>,
  ): Promise<Record<string, unknown> | undefined> {
    setBusy(true);
    setWriteError(undefined);
    try {
      const written = await readJson(path, "management", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(body),
      });
      if (!written.ok) {
        setWriteError(httpErrorMessage(written.status, written.body));
        return undefined;
      }
      return written.body && typeof written.body === "object"
        ? (written.body as Record<string, unknown>)
        : {};
    } catch (error) {
      setWriteError(error instanceof Error ? error.message : "daemon unreachable");
      return undefined;
    } finally {
      setBusy(false);
    }
  }

  async function exportCopy() {
    const record = await postWrite(OUTPUT_EXPORT_PATH, { artifact_id: artifact.artifactId });
    const exported = record?.export;
    if (exported && typeof exported === "object") {
      const path = (exported as Record<string, unknown>).path;
      setExportPath(typeof path === "string" ? path : "unknown");
    }
    if (record) {
      void loadOutputDetail(artifact.artifactId);
    }
  }

  async function deriveStageTest(event: FormEvent) {
    event.preventDefault();
    if (stageChoice.length === 0) {
      setWriteError("Pick the stage this deliverable belongs to. No stage is assumed.");
      return;
    }
    const record = await postWrite(ARTIFACT_STAGE_TEST_PATH, {
      artifact_id: artifact.artifactId,
      stage_id: stageChoice,
    });
    if (record) {
      setStageTestFact(
        typeof record.stage_test_fact_id === "string" ? record.stage_test_fact_id : "ok",
      );
      onWritten();
    }
  }

  async function requestAcceptance() {
    if (!lastRing) {
      return;
    }
    const record = await postWrite(RUN_ACCEPTANCE_REQUEST_PATH, {
      project_id: projectId,
      stage_id: lastRing,
    });
    if (record && typeof record.preview_id === "string") {
      setAcceptancePreview(record.preview_id);
      onWritten();
    }
  }

  async function requestExternalSend(event: FormEvent) {
    event.preventDefault();
    const list = recipients
      .split(/[,\n]/)
      .map((item) => item.trim())
      .filter((item) => item.length > 0);
    if (list.length === 0) {
      setWriteError("External send needs at least one recipient. Nothing is sent by this preview.");
      return;
    }
    const record = await postWrite(EXTERNAL_SEND_REQUEST_PATH, {
      project_id: projectId,
      artifact_id: artifact.artifactId,
      recipients: list,
    });
    if (record && typeof record.preview_id === "string") {
      setSendPreview(record.preview_id);
      onWritten();
    }
  }

  const offerAcceptance = acceptanceOfferable(artifact, lastRing);
  const downloadHref =
    opened !== undefined
      ? `data:text/markdown;charset=utf-8,${encodeURIComponent(opened)}`
      : undefined;

  return (
    <div data-region="opc-output-selected" className="cp-stack">
      <table className="cp-table">
        <caption className="cp-quiet">Selected artifact — daemon statements only</caption>
        <tbody>
          <tr>
            <td>Artifact</td>
            <td>
              <code className="cp-mono">{artifact.artifactId}</code>
            </td>
          </tr>
          <tr>
            <td>CAS digest</td>
            <td>
              <code className="cp-mono">{artifact.casRef}</code>
            </td>
          </tr>
          <tr>
            <td>Format · bytes</td>
            <td>
              {artifact.format} · {artifact.byteLength}
            </td>
          </tr>
          <tr>
            <td>Source</td>
            <td>
              {artifact.source} · frame {artifact.sourceFrameSeq} · attempt{" "}
              <code className="cp-mono">{artifact.attemptId}</code>
            </td>
          </tr>
          <tr>
            <td>Freshness</td>
            <td>{artifact.freshness}</td>
          </tr>
          <tr>
            <td>Independent verification</td>
            <td data-output-verify={artifact.verificationStatus}>
              {artifact.verificationStatus}
              {row && row.evidence.length > 0 ? (
                <>
                  {" "}
                  · <code className="cp-mono">{row.evidence[0].verifierRef}</code> ·{" "}
                  {row.evidence[0].criteria.map((c) => `${c.id}=${c.result}`).join(" ")}
                </>
              ) : null}
            </td>
          </tr>
          <tr>
            <td>Stage (current StageTestPassed)</td>
            <td>{artifact.stageId}</td>
          </tr>
          <tr>
            <td>Accepted</td>
            <td>{artifact.acceptedAt}</td>
          </tr>
          <tr>
            <td>Export copy (data/)</td>
            <td>
              {row ? (
                <>
                  {row.exportExists} · <code className="cp-mono">{row.exportPath}</code> · not authority
                </>
              ) : (
                "fetching detail"
              )}
            </td>
          </tr>
        </tbody>
      </table>

      <div className="cp-row" data-region="opc-output-open">
        <button type="button" className="cp-button" onClick={() => void openBytes()}>
          Open from CAS
        </button>
        {downloadHref ? (
          <a
            className="cp-button"
            href={downloadHref}
            download={`${artifact.artifactId}.md`}
            data-region="opc-output-download"
          >
            Download
          </a>
        ) : null}
        <button type="button" className="cp-button" disabled={busy} onClick={() => void exportCopy()}>
          Export copy to data/
        </button>
      </div>
      {openError ? (
        <p className="cp-error" data-output-open-error="true">
          {openError}
        </p>
      ) : null}
      {opened !== undefined ? (
        <pre data-region="opc-output-bytes" className="cp-mono">
          {opened}
        </pre>
      ) : null}
      {exportPath ? (
        <p data-region="opc-output-exported">
          Copy written to <code className="cp-mono">{exportPath}</code>. Not authority; host
          file-open E2E is not-run.
        </p>
      ) : null}

      <section data-region="opc-output-closeout">
        <h3>Close-out</h3>
        <p className="cp-quiet">
          Close-out is openable artifact + verify state + 「验收，回 Today」, and the
          acceptance control exists only on the last ring ({lastRing ?? "unknown"}).
          Process death, exit 0, HTTP 200 and the child's “done” are not
          completion; only independent-verifier evidence can back a stage test.
        </p>
        {verified && artifact.stageId === "none" && artifact.freshness === "current" ? (
          <form onSubmit={(event) => void deriveStageTest(event)} className="cp-row">
            <label>
              Stage for this deliverable
              <select
                name="stage-id"
                value={stageChoice}
                onChange={(event) => setStageChoice(event.target.value)}
              >
                <option value="">— pick a stage —</option>
                {stageChoices.map((id) => (
                  <option key={id} value={id}>
                    {id}
                  </option>
                ))}
              </select>
            </label>
            <button type="submit" className="cp-button" disabled={busy}>
              Derive stage test from evidence
            </button>
          </form>
        ) : null}
        {!verified ? (
          <p data-output-closeout="not-verified">
            Not independently verified ({artifact.verificationStatus}); no stage test and no
            acceptance are offered.
          </p>
        ) : null}
        {stageTestFact ? (
          <p data-region="opc-output-stage-test">
            StageTestPassed derived: <code className="cp-mono">{stageTestFact}</code>. A stage
            test is not acceptance.
          </p>
        ) : null}
        {verified && artifact.stageId !== "none" && artifact.stageId !== lastRing ? (
          <p data-output-closeout="intermediate-ring">
            Stage <code className="cp-mono">{artifact.stageId}</code> is not the last ring;
            acceptance is not offered here.
          </p>
        ) : null}
        {offerAcceptance ? (
          <button
            type="button"
            className="cp-button"
            disabled={busy}
            onClick={() => void requestAcceptance()}
            data-region="opc-output-accept"
          >
            验收，回 Today (request preview)
          </button>
        ) : null}
        {acceptancePreview ? (
          <p data-region="opc-output-accept-preview">
            Acceptance preview <code className="cp-mono">{acceptancePreview}</code> is pending.{" "}
            <Link to={hitlCanvasPath(acceptancePreview, projectId)}>Review it on the Project canvas</Link>
            . Chat cannot accept.
          </p>
        ) : null}
        {artifact.acceptedAt !== "none" ? (
          <p data-output-closeout="accepted">Accepted at {artifact.acceptedAt}.</p>
        ) : null}
      </section>

      <section data-region="opc-output-publication">
        <h3>Publication package</h3>
        {!verified ? (
          <p className="cp-quiet">No package: the artifact is not independently verified.</p>
        ) : null}
        {verified && packet.status === "loading" ? (
          <p className="cp-quiet">Fetching the AUTONOMY packet from the daemon.</p>
        ) : null}
        {verified && packetRow ? (
          <div data-output-packet-published={String(packetRow.published)}>
            <p>
              <strong>{packetRow.planned ? "Planned" : "Not planned"}</strong> ·{" "}
              {packetRow.published ? "published (unexpected)" : "not published"} · connector{" "}
              <code className="cp-mono">{packetRow.connector}</code> · verified {packetRow.verified} ·
              accepted {packetRow.accepted}
              {packetRow.chatCanConfirm ? " · chat confirm claimed (unexpected)" : " · chat has no Confirm"}
            </p>
            <dl>
              {packetRow.sections.map((section) => (
                <div key={section.id} data-packet-section={section.id}>
                  <dt>{section.id}</dt>
                  {section.facts.map((fact) => (
                    <dd key={fact.key}>
                      <code className="cp-mono">{fact.key}</code>: {fact.value}
                    </dd>
                  ))}
                </div>
              ))}
            </dl>
            <form onSubmit={(event) => void requestExternalSend(event)} className="cp-row">
              <label>
                Recipients (comma separated)
                <input
                  name="recipients"
                  value={recipients}
                  onChange={(event) => setRecipients(event.target.value)}
                  disabled={busy}
                />
              </label>
              <button type="submit" className="cp-button" disabled={busy}>
                Request external-send preview
              </button>
            </form>
            {sendPreview ? (
              <p data-region="opc-output-send-preview">
                External-send preview <code className="cp-mono">{sendPreview}</code> is pending on
                the Project canvas:{" "}
                <Link to={hitlCanvasPath(sendPreview, projectId)}>review it there</Link>. Planned is
                not published; no connector is qualified.
              </p>
            ) : null}
          </div>
        ) : null}
      </section>
      {writeError ? (
        <p className="cp-error" data-output-write-error="true">
          {writeError}
        </p>
      ) : null}
    </div>
  );
}
