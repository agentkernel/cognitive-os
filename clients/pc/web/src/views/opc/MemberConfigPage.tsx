import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { PageHeader } from "../../components/PageHeader";
import { EmptyState } from "../../components/states";
import {
  PROJECT_AXIS_PATH,
  PROJECT_CATALOG_PATH,
  PROJECT_ROSTER_PATH,
  projectAxisKey,
  projectCatalogKey,
  projectRosterKey,
  type ProjectAxisStageRow,
  type ProjectCatalogRow,
  type ProjectRosterRow,
} from "../../data/projections/projectWork";
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { DaemonReadPanel } from "./DaemonReadPanel";
import {
  loadEmployeeCatalog,
  loadProjectAxis,
  loadProjectRoster,
} from "./loadOpcReads";
import { MEMBER_CONFIG_TABS, type MemberConfigTabId } from "./memberTabs";
import { ProjectWorkNav } from "./ProjectWorkNav";

function assignedStages(member: ProjectRosterRow, stages: ProjectAxisStageRow[]): ProjectAxisStageRow[] {
  const ids = member.responsibleStageIds
    .split(",")
    .map((item) => item.trim())
    .filter((item) => item.length > 0 && item !== "unknown");
  return stages.filter(
    (stage) => ids.includes(stage.stageId) || stage.responsibleSlot === member.employeeId,
  );
}

/**
 * Select-then-configure (P12-T04). Eight tabs. No member budget, no engine store.
 */
export function MemberConfigPage() {
  const { projectId = "", memberId = "" } = useParams();
  const roster = useProjection<ProjectRosterRow[]>(projectRosterKey(projectId));
  const axis = useProjection<ProjectAxisStageRow[]>(projectAxisKey(projectId));
  const catalog = useProjection<ProjectCatalogRow[]>(projectCatalogKey(projectId, memberId));
  const [tab, setTab] = useState<MemberConfigTabId>("duty");
  const refresh = useCallback(async () => {
    if (projectId.length === 0 || memberId.length === 0) {
      return;
    }
    await Promise.all([
      loadProjectRoster(projectId),
      loadProjectAxis(projectId),
      loadEmployeeCatalog(projectId, memberId),
    ]);
  }, [projectId, memberId]);
  useEffect(() => {
    void refresh();
    setTab("duty");
  }, [refresh]);

  const member = (roster.data ?? []).find((row) => row.employeeId === memberId);
  const stages = axis.data ?? [];
  const grants = catalog.data ?? [];
  const duty = member ? assignedStages(member, stages) : [];
  const skillRows = grants.filter((row) => row.capabilityRef.startsWith("skill"));
  const toolRows = grants.filter((row) => row.capabilityRef.startsWith("tool"));

  return (
    <section data-page="opc-member-config">
      <PageHeader
        title="Member configuration"
        lede="Select the member, then configure duty, contracts, grants, brief, loop, and perms."
      />
      <HonestyNote>
        Product origin is daemon-served hash /ui/. GET {PROJECT_ROSTER_PATH} is the
        current Project. GET {PROJECT_CATALOG_PATH} is the grant catalog, not a
        recipe. Role is not merged into Agent. Member-level budget is not chrome.
        The engine store is not default chrome.
      </HonestyNote>
      <p className="cp-quiet">
        <Link to={`/projects/${encodeURIComponent(projectId)}/members`}>Members</Link>
        {memberId ? (
          <>
            {" "}
            · <code className="cp-mono">{memberId}</code>
          </>
        ) : null}
      </p>
      {projectId ? <ProjectWorkNav projectId={projectId} /> : null}
      {!member ? (
        <EmptyState title="Member: not on this roster">
          GET {PROJECT_ROSTER_PATH} did not include {memberId || "this employee"}. The
          configuration is not invented.
        </EmptyState>
      ) : (
        <>
          <p className="cp-quiet">
            State {member.state} · manager {member.isCurrentManager} · model{" "}
            {member.modelBound}
          </p>
          <p className="cp-quiet" role="tablist" aria-label="Member configuration">
            {MEMBER_CONFIG_TABS.map((item) => (
              <button
                key={item.id}
                type="button"
                role="tab"
                aria-selected={tab === item.id}
                className={tab === item.id ? "cp-button cp-button--primary" : "cp-button"}
                onClick={() => setTab(item.id)}
              >
                {item.label}
              </button>
            ))}
          </p>
          {tab === "duty" ? (
            duty.length === 0 ? (
              <EmptyState title="Duty: no stage assigned">
                GET {PROJECT_AXIS_PATH} did not bind this employee. The post stays empty.
              </EmptyState>
            ) : (
              <table className="cp-table" data-region="opc-member-duty">
                <caption className="cp-quiet">
                  GET {PROJECT_AXIS_PATH} · {member.responsibleStageIds}
                </caption>
                <thead>
                  <tr>
                    <th>Stage</th>
                    <th>Title</th>
                    <th>Slot</th>
                  </tr>
                </thead>
                <tbody>
                  {duty.map((stage) => (
                    <tr key={stage.stageId}>
                      <td>
                        <code className="cp-mono">{stage.stageId}</code>
                      </td>
                      <td>{stage.title}</td>
                      <td>{stage.responsibleSlot}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )
          ) : null}
          {tab === "input" || tab === "output" ? (
            <>
              <EmptyState title={tab === "input" ? "Input: no contract" : "Output: no contract"}>
                GET {PROJECT_AXIS_PATH} output_contract is unknown until the daemon names a digest.
                This tab does not invent a file.
              </EmptyState>
              <table className="cp-table">
                <caption className="cp-quiet">PlanRevision stages</caption>
                <thead>
                  <tr>
                    <th>Stage</th>
                    <th>Deliverable</th>
                    <th>Save</th>
                    <th>Open with</th>
                  </tr>
                </thead>
                <tbody>
                  {stages.map((stage) => (
                    <tr key={stage.stageId}>
                      <td>
                        <code className="cp-mono">{stage.stageId}</code>
                      </td>
                      <td>{stage.deliverableType}</td>
                      <td>{stage.saveFormat}</td>
                      <td>{stage.openWith}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </>
          ) : null}
          {tab === "skills" ? (
            <DaemonReadPanel
              projection={catalog}
              surface="Skills"
              emptyTitle="Skills: no grant"
              emptyBody={`GET ${PROJECT_CATALOG_PATH} is a grant catalog. Recipe mentions are not grants.`}
              region="opc-member-skills"
            >
              {skillRows.length === 0 ? (
                <EmptyState title="Skills: no grant">
                  GET {PROJECT_CATALOG_PATH} listed no skill grant. A recipe mention is not a grant.
                </EmptyState>
              ) : (
                <ul>
                  {skillRows.map((row) => (
                    <li key={row.capabilityRef}>
                      <code className="cp-mono">{row.capabilityRef}</code>
                    </li>
                  ))}
                </ul>
              )}
            </DaemonReadPanel>
          ) : null}
          {tab === "tools" ? (
            <DaemonReadPanel
              projection={catalog}
              surface="Tools"
              emptyTitle="Tools: no grant"
              emptyBody={`GET ${PROJECT_CATALOG_PATH} is a grant catalog. Recipe mentions are not grants.`}
              region="opc-member-tools"
            >
              {toolRows.length === 0 ? (
                <EmptyState title="Tools: no grant">
                  GET {PROJECT_CATALOG_PATH} listed no tool grant. A recipe mention is not a grant.
                </EmptyState>
              ) : (
                <ul>
                  {toolRows.map((row) => (
                    <li key={row.capabilityRef}>
                      <code className="cp-mono">{row.capabilityRef}</code>
                    </li>
                  ))}
                </ul>
              )}
            </DaemonReadPanel>
          ) : null}
          {tab === "prompt" ? (
            <EmptyState title="Brief: no rewrite">
              Duty text is the prompt written at join. This tab does not rewrite it locally.
              Member {member.employeeId} state {member.state}.
            </EmptyState>
          ) : null}
          {tab === "loop" ? (
            <EmptyState title="Loop: no engine">
              Runtime binding {member.runtimeBindingRef}. The engine store is not 2.0.0 default chrome.
            </EmptyState>
          ) : null}
          {tab === "perms" ? (
            <DaemonReadPanel
              projection={catalog}
              surface="Perms"
              emptyTitle="Perms: no grant"
              emptyBody={`GET ${PROJECT_CATALOG_PATH} empty. A recipe mention is not a grant.`}
              region="opc-member-perms"
            >
              <ul>
                {grants.map((row) => (
                  <li key={row.capabilityRef}>
                    <code className="cp-mono">{row.capabilityRef}</code>
                  </li>
                ))}
              </ul>
            </DaemonReadPanel>
          ) : null}
        </>
      )}
    </section>
  );
}
