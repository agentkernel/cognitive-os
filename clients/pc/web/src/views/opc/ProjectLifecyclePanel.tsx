import { useCallback, useEffect, useState } from "react";
import { readJson } from "../../api";
import {
  PROJECT_ARCHIVE_PATH,
  PROJECT_DELETE_CONFIRM_PATH,
  PROJECT_DELETE_PREVIEW_PATH,
  PROJECT_EXPORT_PATH,
  PROJECT_RESTORE_POINT_PATH,
  projectLifecycle,
  projectLifecyclePath,
  type ProjectLifecycleRow,
} from "../../data/projections/projectLifecycle";
import { HonestyNote } from "../../state/HonestyNote";

export function ProjectLifecyclePanel({
  projectId,
  onChanged,
}: {
  projectId: string;
  onChanged: () => void;
}) {
  const [view, setView] = useState<ProjectLifecycleRow | undefined>();
  const [message, setMessage] = useState("");
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    const result = await readJson(projectLifecyclePath(projectId), "management");
    if (result.ok) {
      setView(projectLifecycle(result.body));
    }
  }, [projectId]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  async function post(path: string, body: Record<string, unknown>) {
    setBusy(true);
    setMessage("");
    const result = await readJson(path, "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
    setBusy(false);
    if (!result.ok) {
      const record = (result.body ?? {}) as { message?: string; code?: string };
      setMessage(record.message ?? record.code ?? `HTTP ${result.status}`);
      return;
    }
    await refresh();
    onChanged();
  }

  const preview = view?.pendingDeletePreview;

  return (
    <section data-region="opc-project-lifecycle">
      <HonestyNote>
        Archive stops Routine/Trigger first. Delete is a logical impact preview
        plus a second step — the Project row stays. Restore points are
        same-disk and not a disaster backup. Export is not authority and never
        includes secrets. Chat cannot Approve.
      </HonestyNote>
      <p className="cp-quiet">
        data/ {view?.dataDir ?? "unknown"}
        {view?.logicallyDeleted ? " · logically deleted" : ""}
      </p>
      <div className="cp-row">
        <button
          type="button"
          className="cp-button"
          disabled={busy}
          onClick={() => void post(PROJECT_ARCHIVE_PATH, { project_id: projectId })}
        >
          Archive (stop triggers)
        </button>
        <button
          type="button"
          className="cp-button"
          disabled={busy}
          onClick={() => void post(PROJECT_DELETE_PREVIEW_PATH, { project_id: projectId })}
        >
          Preview delete impact
        </button>
        <button
          type="button"
          className="cp-button"
          disabled={busy || !preview}
          onClick={() =>
            void post(PROJECT_DELETE_CONFIRM_PATH, {
              project_id: projectId,
              confirmation_digest: preview?.confirmationDigest,
              second_confirm: true,
            })
          }
        >
          Apply logical delete
        </button>
        <button
          type="button"
          className="cp-button"
          disabled={busy}
          onClick={() => void post(PROJECT_RESTORE_POINT_PATH, { project_id: projectId })}
        >
          Record local restore point
        </button>
        <button
          type="button"
          className="cp-button"
          disabled={busy}
          onClick={() => void post(PROJECT_EXPORT_PATH, { project_id: projectId })}
        >
          Export without secrets
        </button>
      </div>
      {message ? <p className="cp-quiet">{message}</p> : null}
      {preview ? (
        <p className="cp-quiet">
          Delete preview {preview.previewId} · triggers stopped{" "}
          {preview.triggersStopped ? "yes" : "no"}
        </p>
      ) : null}
      {view && view.restorePoints.length > 0 ? (
        <ul>
          {view.restorePoints.map((point) => (
            <li key={point.eventId}>
              {point.versionName} · not a disaster backup
            </li>
          ))}
        </ul>
      ) : null}
    </section>
  );
}
