import { fetchProjection } from "../../data/fetchProjection";
import {
  firstReadyProjectId,
  PROJECTS_KEY,
  PROJECT_LIST_PATH,
  projectProjectList,
  type ProjectListRow,
} from "../../data/projections/projects";
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
