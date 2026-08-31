/**
 * Personal-private Project list projection (P11-T13 Dual Track).
 * Source is GET /management/project/v1/list. Title/cost stay daemon-stated;
 * this file never invents a Project or a completion.
 */

export const PROJECTS_KEY = "opc:projects";
export const PROJECT_LIST_PATH = "/management/project/v1/list";

export interface ProjectListRow {
  projectId: string;
  state: string;
  titleSummary: string;
  cost: string;
}

export function firstReadyProjectId(rows: ProjectListRow[] | undefined): string | undefined {
  const id = rows?.[0]?.projectId;
  return typeof id === "string" && id.length > 0 ? id : undefined;
}

export function projectProjectList(body: unknown): ProjectListRow[] {
  if (!body || typeof body !== "object") {
    return [];
  }
  const projects = (body as { projects?: unknown }).projects;
  if (!Array.isArray(projects)) {
    return [];
  }
  const rows: ProjectListRow[] = [];
  for (const row of projects) {
    if (!row || typeof row !== "object") {
      continue;
    }
    const record = row as Record<string, unknown>;
    if (typeof record.project_id !== "string" || record.project_id.length === 0) {
      continue;
    }
    rows.push({
      projectId: record.project_id,
      state: typeof record.state === "string" ? record.state : "unknown",
      titleSummary: typeof record.title_summary === "string" ? record.title_summary : "unknown",
      cost: typeof record.cost === "string" ? record.cost : "unknown",
    });
  }
  return rows;
}
