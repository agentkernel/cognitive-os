import { useCallback, useEffect, useState } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { readJson } from "../../api";
import { FactGrid } from "../../components/FactGrid";
import { PageHeader } from "../../components/PageHeader";
import { HITL_KEY, type PendingPreviewRow } from "../../data/projections/hitl";
import { PROJECT_COPY_PATH } from "../../data/projections/projectLifecycle";
import {
  creatingProjectRows,
  liveProjectRows,
  PROJECTS_KEY,
  PROJECT_LIST_PATH,
  type ProjectListRow,
} from "../../data/projections/projects";
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { DaemonReadPanel } from "./DaemonReadPanel";
import { HitlCanvasTable } from "./HitlCanvasTable";
import { httpErrorMessage } from "./httpError";
import { loadPendingPreviewsForReadyProject, loadProjectList, readyProjectId } from "./loadOpcReads";
import { ProjectAuthorityPanel } from "./ProjectAuthorityPanel";
import { ownerProjectStatus } from "./ProjectWorkNav";

/**
 * Projects — L1 inventory of daemon Project rows plus the HITL canvas.
 * Owner chrome vs frozen v9: 项目列表 / 未完成的创建 / 还没有项目.
 * Open is hash navigation to `#/projects/:id` four submenus. No create/activate
 * on this list. Creating drafts continue create; they do not open live chrome.
 */
export function ProjectsPage() {
  const [params] = useSearchParams();
  const focusPreviewId = params.get("preview");
  const projects = useProjection<ProjectListRow[]>(PROJECTS_KEY);
  const hitl = useProjection<PendingPreviewRow[]>(HITL_KEY);
  const projectId = readyProjectId(projects);
  const liveRows = liveProjectRows(projects.data);
  const creatingRows = creatingProjectRows(projects.data);
  const emptyHome =
    projects.status === "empty" ||
    (projects.status === "ready" && (projects.data?.length ?? 0) === 0);
  const incompleteOnly =
    projects.status === "ready" && liveRows.length === 0 && creatingRows.length > 0;
  const [copyMessage, setCopyMessage] = useState("");
  const refresh = useCallback(async () => {
    const list = await loadProjectList();
    await loadPendingPreviewsForReadyProject(list);
  }, []);
  useEffect(() => {
    void refresh();
  }, [refresh]);

  const title = emptyHome ? "还没有项目" : incompleteOnly ? "未完成的创建" : "项目列表";
  const lede = emptyHome
    ? "创建只从今日空首页开始。这里不再并列创建按钮。没有示范项目。"
    : incompleteOnly
      ? "项目列表现在只露出这一份草稿。继续创建，不要进详情/成员/运行/产出。"
      : "打开一个项目后，用详情 / 成员 / 运行 / 产出工作。副本不带密钥、进行中任务、对外回执、本周不再问。第二份工作请复制，不要在已上线列表另开看不见的 ①。";

  return (
    <section data-page="opc-projects">
      <PageHeader title={title} lede={lede} />
      <HonestyNote placement="secondary">
        Product origin is daemon-served hash /ui/. Rows are the daemon list.
        Confirm-before-activate stays on management HTTP. HITL on this page is
        the project-center canvas, not an Inbox.
      </HonestyNote>
      <ProjectAuthorityPanel
        projection={projects}
        surface="项目列表"
        leadHonesty={false}
        emptyBody="This daemon reports no Project. That is not an empty Task list and not an accepted OPC chrome. No action is available until a Project exists as daemon authority. 没有示范项目。"
        emptyAction={
          <Link className="cp-button" to="/">
            回今日
          </Link>
        }
      >
        {incompleteOnly ? (
          <div data-surface="projects-creating">
            {creatingRows.map((row) => (
              <section key={row.projectId} className="cp-region" data-row-key={row.projectId}>
                <h3>
                  {row.titleSummary} · {ownerProjectStatus(row.state)}
                </h3>
                <p className="cp-quiet">未上线。不能当已上线项目跑。</p>
                <p>
                  <Link className="cp-button cp-button--primary" to="/projects/new">
                    继续这份草稿
                  </Link>
                </p>
              </section>
            ))}
          </div>
        ) : (
          <>
            {creatingRows.length > 0 ? (
              <p className="cp-quiet" data-region="opc-projects-leftover-drafts">
                未完成的创建还在。继续创建，不要把它当已上线项目打开。{" "}
                <Link to="/projects/new">继续这份草稿</Link>
              </p>
            ) : null}
            {liveRows.map((row) => (
              <section key={row.projectId} className="cp-region" data-row-key={row.projectId}>
                <h3>
                  {row.titleSummary} · {ownerProjectStatus(row.state)}
                </h3>
                <FactGrid
                  facts={[
                    { label: "目标", value: row.titleSummary },
                    { label: "状态", value: `${ownerProjectStatus(row.state)} (${row.state})` },
                    { label: "费用", value: row.cost },
                  ]}
                />
                <p className="cp-toolbar">
                  <Link className="cp-button cp-button--primary" to={`/projects/${encodeURIComponent(row.projectId)}`}>
                    打开
                  </Link>{" "}
                  <Link className="cp-button" to={`/projects/${encodeURIComponent(row.projectId)}/members`}>
                    成员
                  </Link>{" "}
                  <Link className="cp-button" to={`/projects/${encodeURIComponent(row.projectId)}/runs`}>
                    运行
                  </Link>{" "}
                  <Link className="cp-button" to={`/projects/${encodeURIComponent(row.projectId)}/outputs`}>
                    产出
                  </Link>{" "}
                  <button
                    type="button"
                    className="cp-button"
                    onClick={() => {
                      void (async () => {
                        const result = await readJson(PROJECT_COPY_PATH, "management", {
                          method: "POST",
                          headers: { "content-type": "application/json" },
                          body: JSON.stringify({ project_id: row.projectId }),
                        });
                        if (!result.ok) {
                          setCopyMessage(httpErrorMessage(result.status, result.body));
                          return;
                        }
                        setCopyMessage("");
                        await refresh();
                      })();
                    }}
                  >
                    复制为草稿（副本）
                  </button>
                </p>
              </section>
            ))}
            {copyMessage ? <p className="cp-quiet">{copyMessage}</p> : null}
            {projectId ? (
              <DaemonReadPanel
                projection={hitl}
                surface="Projects HITL canvas"
                emptyTitle="Projects: no pending ApprovalPreview"
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
            <p className="cp-quiet">GET {PROJECT_LIST_PATH}</p>
          </>
        )}
      </ProjectAuthorityPanel>
    </section>
  );
}
