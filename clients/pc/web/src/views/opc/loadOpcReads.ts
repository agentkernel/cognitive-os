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
import {
  attemptHistoryKey,
  attemptListPath,
  projectAttemptHistory,
  projectRoutineRuns,
  routineRunsKey,
  routineRunsPath,
  type AttemptHistoryRow,
  type RoutineRunsView,
} from "../../data/projections/routineRuns";
import {
  projectTodayOverview,
  TODAY_OVERVIEW_KEY,
  todayOverviewPath,
  type TodayOverviewView,
  type TodayPeriod,
} from "../../data/projections/todayOverview";
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

/** P13-T05/D02: occurrence ledger + armings for one Project. No routine_id is invented. */
export async function loadRoutineRuns(
  projectId: string,
): Promise<Projection<RoutineRunsView[]>> {
  return fetchProjection(
    appProjections,
    routineRunsKey(projectId),
    routineRunsPath(projectId),
    "management",
    projectRoutineRuns,
  );
}

/** P13-T05/D02: real Attempt history (P13-T02 `dsh.hosted.attempt.list`). */
export async function loadAttemptHistory(
  projectId: string,
): Promise<Projection<AttemptHistoryRow[]>> {
  return fetchProjection(
    appProjections,
    attemptHistoryKey(projectId),
    attemptListPath(projectId),
    "management",
    projectAttemptHistory,
  );
}

/**
 * P13-T05/D02: Today run overview for one period. Only fetched when the list
 * has a live Project (creating-only and empty homes stay only-create).
 */
export async function loadTodayOverview(
  period: TodayPeriod,
): Promise<Projection<TodayOverviewView[]>> {
  return fetchProjection(
    appProjections,
    TODAY_OVERVIEW_KEY,
    todayOverviewPath(period),
    "management",
    projectTodayOverview,
  );
}

export async function loadTodayOverviewForLiveProject(
  list: Projection<ProjectListRow[]>,
  period: TodayPeriod,
): Promise<void> {
  if (!liveProjectId(list)) {
    return;
  }
  await loadTodayOverview(period);
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
