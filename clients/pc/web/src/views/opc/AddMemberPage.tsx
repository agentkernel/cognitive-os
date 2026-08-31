import { useCallback, useEffect, useMemo, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { readJson } from "../../api";
import { PageHeader } from "../../components/PageHeader";
import { EmptyState } from "../../components/states";
import {
  PROJECT_AXIS_PATH,
  PROJECT_DETAIL_PATH,
  PROJECT_ROSTER_PATH,
  ROSTER_REGISTER_PATH,
  SEAT_CONFIRM_PATH,
  SEAT_REQUEST_PATH,
  projectAxisKey,
  projectDetailKey,
  projectRosterKey,
  uniqueResponsibleSlots,
  type ProjectAxisStageRow,
  type ProjectDetailRow,
  type ProjectRosterRow,
} from "../../data/projections/projectWork";
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { DaemonReadPanel } from "./DaemonReadPanel";
import { httpErrorMessage, jsonStringList } from "./httpError";
import { loadProjectAxis, loadProjectDetail, loadProjectRoster } from "./loadOpcReads";
import { ProjectWorkNav } from "./ProjectWorkNav";

/**
 * Add member (P12-T04). Current Project roster only. Duty/slot first.
 * Write join = roster.register then seat.request then seat.confirm.
 * Refuse = no register, or seat.confirm accept=false. No Install store.
 */
export function AddMemberPage() {
  const { projectId = "" } = useParams();
  const navigate = useNavigate();
  const roster = useProjection<ProjectRosterRow[]>(projectRosterKey(projectId));
  const detail = useProjection<ProjectDetailRow[]>(projectDetailKey(projectId));
  const axis = useProjection<ProjectAxisStageRow[]>(projectAxisKey(projectId));
  const [specialization, setSpecialization] = useState<"member" | "project-manager">("member");
  const [prompt, setPrompt] = useState("");
  const [model, setModel] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | undefined>();
  const [joinedIds, setJoinedIds] = useState<string[]>([]);
  const refresh = useCallback(async () => {
    if (projectId.length === 0) {
      return;
    }
    await Promise.all([loadProjectRoster(projectId), loadProjectDetail(projectId), loadProjectAxis(projectId)]);
  }, [projectId]);
  useEffect(() => {
    void refresh();
    setJoinedIds([]);
    setError(undefined);
  }, [refresh]);

  const planRevisionId = detail.data?.[0]?.planRevisionId;
  const slots = useMemo(() => uniqueResponsibleSlots(axis.data ?? []), [axis.data]);

  async function writeJoin() {
    if (!planRevisionId || planRevisionId === "unknown" || slots.length === 0) {
      setError("Write join needs a PlanRevision with responsible slots. Nothing was registered.");
      return;
    }
    setBusy(true);
    setError(undefined);
    setJoinedIds([]);
    try {
      const registered = await readJson(ROSTER_REGISTER_PATH, "management", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          project_id: projectId,
          plan_revision_id: planRevisionId,
          proposals: slots.map((slot) => ({
            slot,
            specialization,
            prompt: prompt.trim(),
            tools_declared: [],
          })),
        }),
      });
      if (!registered.ok) {
        setError(httpErrorMessage(registered.status, registered.body));
        return;
      }
      const employeeIds = jsonStringList(registered.body, "employee_ids");
      if (employeeIds.length === 0) {
        setError("roster.register returned no employee_ids. No member was invented locally.");
        return;
      }
      for (const employeeId of employeeIds) {
        const requested = await readJson(SEAT_REQUEST_PATH, "management", {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ employee_id: employeeId }),
        });
        if (!requested.ok) {
          setError(httpErrorMessage(requested.status, requested.body));
          return;
        }
        const seated = await readJson(SEAT_CONFIRM_PATH, "management", {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({
            employee_id: employeeId,
            accept: true,
            ...(model.trim() === "" ? {} : { model_binding: model.trim() }),
          }),
        });
        if (!seated.ok) {
          setError(httpErrorMessage(seated.status, seated.body));
          return;
        }
      }
      setJoinedIds(employeeIds);
      await loadProjectRoster(projectId);
    } finally {
      setBusy(false);
    }
  }

  async function refuseJoin() {
    setBusy(true);
    setError(undefined);
    try {
      if (joinedIds.length === 0) {
        setPrompt("");
        setModel("");
        return;
      }
      for (const employeeId of joinedIds) {
        const refused = await readJson(SEAT_CONFIRM_PATH, "management", {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ employee_id: employeeId, accept: false }),
        });
        if (!refused.ok) {
          setError(httpErrorMessage(refused.status, refused.body));
          return;
        }
      }
      setJoinedIds([]);
      await loadProjectRoster(projectId);
    } finally {
      setBusy(false);
    }
  }

  return (
    <section data-page="opc-add-member">
      <PageHeader
        title="Add member"
        lede="Duty and slot first. Write join is management Intent. Refuse leaves the post unjoined."
      />
      <HonestyNote>
        Product origin is daemon-served hash /ui/. GET {PROJECT_ROSTER_PATH} is the
        current Project roster — not a global sample. Write join posts{" "}
        {ROSTER_REGISTER_PATH} then {SEAT_REQUEST_PATH} then {SEAT_CONFIRM_PATH}.
        Missing model becomes pending. Role is not merged into Agent. No Install
        store. Member-level budget is not chrome.
      </HonestyNote>
      <p className="cp-quiet">
        <Link to={`/projects/${encodeURIComponent(projectId)}/members`}>Members</Link>
        {projectId ? (
          <>
            {" "}
            · <code className="cp-mono">{projectId}</code>
          </>
        ) : null}
      </p>
      {projectId ? <ProjectWorkNav projectId={projectId} /> : null}
      <DaemonReadPanel
        projection={roster}
        surface="Add member roster"
        emptyTitle="Add member: empty roster"
        emptyBody="authority_note empty-roster. This post does not exist yet. Write join still requires PlanRevision slots from GET axis."
        region="opc-add-member-roster"
      >
        <table className="cp-table">
          <caption className="cp-quiet">GET {PROJECT_ROSTER_PATH}</caption>
          <thead>
            <tr>
              <th>Employee</th>
              <th>State</th>
            </tr>
          </thead>
          <tbody>
            {(roster.data ?? []).map((row) => (
              <tr key={row.employeeId} data-row-key={row.employeeId}>
                <td>
                  <code className="cp-mono">{row.employeeId}</code>
                </td>
                <td>{row.state}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </DaemonReadPanel>
      <p className="cp-quiet">
        GET {PROJECT_DETAIL_PATH} plan {planRevisionId ?? "unknown"}. GET {PROJECT_AXIS_PATH}{" "}
        slots: {slots.length === 0 ? "none — register will fail closed" : slots.join(", ")}.
      </p>
      <label className="cp-field">
        Specialization
        <select
          name="specialization"
          value={specialization}
          onChange={(event) =>
            setSpecialization(event.target.value === "project-manager" ? "project-manager" : "member")
          }
        >
          <option value="member">member</option>
          <option value="project-manager">project-manager</option>
        </select>
      </label>
      <label className="cp-field">
        Duty
        <textarea
          name="duty"
          value={prompt}
          onChange={(event) => setPrompt(event.target.value)}
          rows={4}
        />
      </label>
      <label className="cp-field">
        Model binding (empty = pending)
        <input
          name="model"
          value={model}
          onChange={(event) => setModel(event.target.value)}
        />
      </label>
      <p className="cp-quiet">
        <button type="button" className="cp-button cp-button--primary" onClick={() => void writeJoin()} disabled={busy}>
          Write join
        </button>{" "}
        <button type="button" className="cp-button" onClick={() => void refuseJoin()} disabled={busy}>
          Refuse join
        </button>
      </p>
      {error ? (
        <p className="cp-stateview" role="alert" data-join-error="true">
          {error} No member was invented locally.
        </p>
      ) : null}
      {joinedIds.length > 0 ? (
        <p className="cp-quiet" data-region="opc-join-written">
          Daemon returned {joinedIds.join(", ")}.{" "}
          <button
            type="button"
            className="cp-button"
            onClick={() => navigate(`/projects/${encodeURIComponent(projectId)}/members/${encodeURIComponent(joinedIds[0])}`)}
          >
            Open configuration
          </button>
        </p>
      ) : (
        <EmptyState title="Add member: not joined">
          Refuse join does not register a seat. Chat cannot Approve this.
        </EmptyState>
      )}
    </section>
  );
}
