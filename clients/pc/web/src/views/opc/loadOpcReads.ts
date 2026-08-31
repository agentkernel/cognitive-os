import { fetchProjection } from "../../data/fetchProjection";
import {
  HITL_KEY,
  pendingPreviewsPath,
  previewDetailKey,
  previewDetailPath,
  projectPendingPreviews,
  projectPreviewDetail,
  type PreviewDetailRow,
} from "../../data/projections/hitl";
import {
  firstLiveProjectId,
  firstReadyProjectId,
  PROJECTS_KEY,
  PROJECT_LIST_PATH,
  projectProjectList,
  type ProjectListRow,
} from "../../data/projections/projects";
import {
  projectAxisKey,
  projectAxisPath,
  projectCatalogKey,
  projectCatalogPath,
  projectDetailKey,
  projectDetailPath,
  projectEmployeeCatalog,
  projectProjectAxis,
  projectProjectDetail,
  projectProjectRoster,
  projectRosterKey,
  projectRosterPath,
  type ProjectAxisStageRow,
  type ProjectCatalogRow,
  type ProjectDetailRow,
  type ProjectRosterRow,
} from "../../data/projections/projectWork";
import { appProjections } from "../../data/store";
import type { Projection } from "../../data/store";

export async function loadProjectList(): Promise<Projection<ProjectListRow[]>> {
  return fetchProjection(
    appProjections,
    PROJECTS_KEY,
    PROJECT_LIST_PATH,
    "management",
    projectProjectList,
  );
}

export function readyProjectId(list: Projection<ProjectListRow[]>): string | undefined {
  if (list.status !== "ready") {
    return undefined;
  }
  return firstReadyProjectId(list.data);
}

/** Live (accepted) Project only. Creating drafts are not daily-packet subjects. */
export function liveProjectId(list: Projection<ProjectListRow[]>): string | undefined {
  if (list.status !== "ready") {
    return undefined;
  }
  return firstLiveProjectId(list.data);
}

/** N8: no Project id ⇒ no pending-previews call. */
export async function loadPendingPreviewsForReadyProject(
  list: Projection<ProjectListRow[]>,
): Promise<void> {
  const id = readyProjectId(list);
  if (!id) {
    return;
  }
  await fetchProjection(
    appProjections,
    HITL_KEY,
    pendingPreviewsPath(id),
    "management",
    projectPendingPreviews,
  );
}

/** N8 + P12-T05: creating-only lists must not fetch daily packets. */
export async function loadPendingPreviewsForLiveProject(
  list: Projection<ProjectListRow[]>,
): Promise<void> {
  const id = liveProjectId(list);
  if (!id) {
    return;
  }
  await fetchProjection(
    appProjections,
    HITL_KEY,
    pendingPreviewsPath(id),
    "management",
    projectPendingPreviews,
  );
}

export async function loadProjectDetail(
  projectId: string,
): Promise<Projection<ProjectDetailRow[]>> {
  return fetchProjection(
    appProjections,
    projectDetailKey(projectId),
    projectDetailPath(projectId),
    "management",
    projectProjectDetail,
  );
}

export async function loadProjectAxis(
  projectId: string,
): Promise<Projection<ProjectAxisStageRow[]>> {
  return fetchProjection(
    appProjections,
    projectAxisKey(projectId),
    projectAxisPath(projectId),
    "management",
    projectProjectAxis,
  );
}

export async function loadProjectRoster(
  projectId: string,
): Promise<Projection<ProjectRosterRow[]>> {
  return fetchProjection(
    appProjections,
    projectRosterKey(projectId),
    projectRosterPath(projectId),
    "management",
    projectProjectRoster,
  );
}

export async function loadEmployeeCatalog(
  projectId: string,
  employeeId: string,
): Promise<Projection<ProjectCatalogRow[]>> {
  return fetchProjection(
    appProjections,
    projectCatalogKey(projectId, employeeId),
    projectCatalogPath(projectId, employeeId),
    "management",
    projectEmployeeCatalog,
  );
}

/** N8: missing Project id ⇒ no pending-previews call. */
export async function loadPendingPreviewsForProject(projectId: string): Promise<void> {
  if (projectId.length === 0) {
    return;
  }
  await fetchProjection(
    appProjections,
    `${HITL_KEY}:${projectId}`,
    pendingPreviewsPath(projectId),
    "management",
    projectPendingPreviews,
  );
}

/** Digest lives on preview-detail, never on the pending list. */
export async function loadPreviewDetail(
  previewId: string,
): Promise<Projection<PreviewDetailRow[]>> {
  return fetchProjection(
    appProjections,
    previewDetailKey(previewId),
    previewDetailPath(previewId),
    "management",
    projectPreviewDetail,
  );
}
