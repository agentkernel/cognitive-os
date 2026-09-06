import { useCallback, useEffect } from "react";
import { Link, useParams, useSearchParams } from "react-router-dom";
import { FactGrid } from "../../components/FactGrid";
import { PageHeader } from "../../components/PageHeader";
import {
  HITL_KEY,
  type PendingPreviewRow,
} from "../../data/projections/hitl";
import { PROJECTS_KEY, type ProjectListRow } from "../../data/projections/projects";
import {
  projectAxisKey,
  PROJECT_AXIS_PATH,
  PROJECT_DETAIL_PATH,
  projectDetailKey,
  type ProjectAxisStageRow,
  type ProjectDetailRow,
} from "../../data/projections/projectWork";
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { DaemonReadPanel } from "./DaemonReadPanel";
import { HitlCanvasTable } from "./HitlCanvasTable";
import {
  loadPendingPreviewsForProject,
  loadProjectAxis,
  loadProjectDetail,
} from "./loadOpcReads";
import { ProjectLifecyclePanel } from "./ProjectLifecyclePanel";
import { ownerProjectStatus, ProjectWorkNav } from "./ProjectWorkNav";

/**
 * Project detail — daemon GET detail + read-only PlanRevision axis.
 * Owner chrome vs frozen v9. L2 goes to 成员/运行/产出. HITL Confirm stays
 * digest-bound on this canvas. Missing axis is honest empty.
 */
export function ProjectDetailPage() {
  const { projectId = "" } = useParams();
  const [params] = useSearchParams();
  const focusPreviewId = params.get("preview");
  const detail = useProjection<ProjectDetailRow[]>(projectDetailKey(projectId));
  const axis = useProjection<ProjectAxisStageRow[]>(projectAxisKey(projectId));
  const list = useProjection<ProjectListRow[]>(PROJECTS_KEY);
  const hitl = useProjection<PendingPreviewRow[]>(`${HITL_KEY}:${projectId}`);
  const refresh = useCallback(async () => {
    if (projectId.length === 0) {
      return;
    }
    await loadProjectDetail(projectId);
    await loadProjectAxis(projectId);
    await loadPendingPreviewsForProject(projectId);
  }, [projectId]);
  useEffect(() => {
    void refresh();
  }, [refresh]);
  const row = detail.data?.[0];
  const listRow = (list.data ?? []).find((item) => item.projectId === projectId);
  const name =
    listRow?.titleSummary && listRow.titleSummary !== "unknown"
      ? listRow.titleSummary
      : (row?.projectId ?? projectId);
  const stageCount = axis.data?.length ?? 0;

  return (
    <section data-page="opc-project-detail">
      <PageHeader
        title="项目详情"
        lede="只读章程与流程轴。改章程走预览确认，不当表单页。去成员 / 运行 / 产出。"
      />
      <HonestyNote placement="secondary">
        Product origin is daemon-served hash /ui/. GET {PROJECT_DETAIL_PATH} is
        the header. The process axis is read-only. Confirm-before-activate stays
        on management HTTP. Chat cannot Approve.
      </HonestyNote>
      <p className="cp-quiet">
        <Link to="/projects">项目列表</Link>
        {projectId ? (
          <>
            {" "}
            · <code className="cp-mono">{projectId}</code>
          </>
        ) : null}
      </p>
      {projectId ? <ProjectWorkNav projectId={projectId} /> : null}
      <DaemonReadPanel
        projection={detail}
        surface="Project detail"
        emptyTitle="Project detail: no Project"
        emptyBody="This hash is not a daemon Project. It is not a Task ref renamed as a Project."
        region="opc-project-detail"
      >
        {row ? (
          <section className="cp-region">
            <h3>
              {name} · {ownerProjectStatus(row.state)}
            </h3>
            <FactGrid
              facts={[
                { label: "名称", value: name },
                { label: "章程", value: row.charterStatus },
                { label: "状态", value: `${ownerProjectStatus(row.state)} (${row.state})` },
                { label: "费用", value: row.cost },
                {
                  label: "流程环节",
                  value:
                    stageCount > 0
                      ? `${stageCount} 环 · 执行进度在运行管理`
                      : "无轴 · 执行进度在运行管理",
                },
                { label: "成员", value: "去成员管理查看。不跨项目共享。" },
              ]}
            />
            <p className="cp-toolbar">
              <Link className="cp-button" to={`/projects/${encodeURIComponent(projectId)}/members`}>
                成员管理
              </Link>{" "}
              <Link className="cp-button" to={`/projects/${encodeURIComponent(projectId)}/runs`}>
                运行管理
              </Link>{" "}
              <Link className="cp-button" to={`/projects/${encodeURIComponent(projectId)}/outputs`}>
                产出管理
              </Link>
            </p>
            <p className="cp-quiet">GET {PROJECT_DETAIL_PATH}</p>
          </section>
        ) : null}
      </DaemonReadPanel>
      <DaemonReadPanel
        projection={axis}
        surface="Project process axis"
        emptyTitle="没有流程轴 · no PlanRevision axis"
        emptyBody="无轴仍诚实 empty。不发明示范环节。Missing plan is empty, not a fake wizard. Runs and outputs stay empty until the daemon states stages."
        region="opc-project-axis"
      >
        <p className="cp-quiet">流程轴 · 只读章程。点运行管理看当前步骤。GET {PROJECT_AXIS_PATH}</p>
        <table className="cp-table">
          <caption className="cp-quiet">GET {PROJECT_AXIS_PATH} — read-only</caption>
          <thead>
            <tr>
              <th>Stage</th>
              <th>Title</th>
              <th>Confirm</th>
              <th>Ready</th>
            </tr>
          </thead>
          <tbody>
            {(axis.data ?? []).map((stage) => (
              <tr key={stage.stageId} data-row-key={stage.stageId}>
                <td>
                  <code className="cp-mono">{stage.stageId}</code>
                </td>
                <td>{stage.title}</td>
                <td>{stage.confirmStatus}</td>
                <td>{stage.ready}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </DaemonReadPanel>
      {projectId && detail.status === "ready" && row ? (
        <ProjectLifecyclePanel
          projectId={projectId}
          onChanged={() => {
            void refresh();
          }}
        />
      ) : null}
      {projectId ? (
        <DaemonReadPanel
          projection={hitl}
          surface="Project HITL canvas"
          emptyTitle="Project: no pending ApprovalPreview"
          emptyBody="No pending ApprovalPreview. Chat cannot Approve. Confirm stays on this canvas when preview-detail supplies a digest."
          region="opc-hitl"
        >
          <HitlCanvasTable
            projectId={projectId}
            rows={hitl.data ?? []}
            focusPreviewId={focusPreviewId}
            deepLink={false}
            onWritten={() => {
              void refresh();
            }}
          />
        </DaemonReadPanel>
      ) : null}
    </section>
  );
}
