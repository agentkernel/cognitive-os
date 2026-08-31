/**
 * Personal-private Project work projections (P12-T03/T04 Dual Track).
 * Sources are GET /management/project/v1/{detail,axis,roster,employee.catalog}.
 * Writes are POST roster.register / employee.seat.request / employee.seat.confirm.
 * These files never invent a Project, Employee, or completion.
 * HITL Confirm is P12-T06.
 */

import { asList, asRecord } from "../projections";

export const PROJECT_DETAIL_PATH = "/management/project/v1/detail";
export const PROJECT_AXIS_PATH = "/management/project/v1/axis";
export const PROJECT_ROSTER_PATH = "/management/project/v1/roster";
export const PROJECT_CATALOG_PATH = "/management/project/v1/employee.catalog";
export const ROSTER_REGISTER_PATH = "/management/project/v1/roster.register";
export const SEAT_REQUEST_PATH = "/management/project/v1/employee.seat.request";
export const SEAT_CONFIRM_PATH = "/management/project/v1/employee.seat.confirm";

export function projectDetailPath(projectId: string): string {
  return `${PROJECT_DETAIL_PATH}?project_id=${encodeURIComponent(projectId)}`;
}

export function projectAxisPath(projectId: string): string {
  return `${PROJECT_AXIS_PATH}?project_id=${encodeURIComponent(projectId)}`;
}

export function projectRosterPath(projectId: string): string {
  return `${PROJECT_ROSTER_PATH}?project_id=${encodeURIComponent(projectId)}`;
}

export function projectCatalogPath(projectId: string, employeeId: string): string {
  return `${PROJECT_CATALOG_PATH}?project_id=${encodeURIComponent(projectId)}&employee_id=${encodeURIComponent(employeeId)}`;
}

export function projectDetailKey(projectId: string): string {
  return `opc:project-detail:${projectId}`;
}

export function projectAxisKey(projectId: string): string {
  return `opc:project-axis:${projectId}`;
}

export function projectRosterKey(projectId: string): string {
  return `opc:project-roster:${projectId}`;
}

export function projectCatalogKey(projectId: string, employeeId: string): string {
  return `opc:project-catalog:${projectId}:${employeeId}`;
}

export interface ProjectDetailRow {
  projectId: string;
  state: string;
  createdAt: string;
  activatedAt: string;
  acceptedAt: string;
  charterStatus: string;
  charterDigest: string;
  planRevisionId: string;
  pendingPreviewCount: string;
  cost: string;
}

export interface ProjectAxisStageRow {
  stageId: string;
  position: string;
  title: string;
  objective: string;
  confirmStatus: string;
  ready: string;
  seated: string;
  outputDigest: string;
  deliverableType: string;
  saveFormat: string;
  openWith: string;
  gapCount: string;
  responsibleSlot: string;
}

export interface ProjectRosterRow {
  employeeId: string;
  state: string;
  modelBound: string;
  isCurrentManager: string;
  runtimeBindingRef: string;
  authorityNote: string;
  responsibleStageIds: string;
}

export interface ProjectCatalogRow {
  capabilityRef: string;
  authorityNote: string;
}

function stated(value: unknown, fallback = "unknown"): string {
  return typeof value === "string" && value.length > 0 ? value : fallback;
}

export function projectProjectDetail(body: unknown): ProjectDetailRow[] {
  const record = asRecord(body);
  const project = asRecord(record.project);
  const projectId = stated(project.project_id, "");
  if (projectId.length === 0) {
    return [];
  }
  const charter = asRecord(record.charter);
  const plan = asRecord(record.plan);
  return [
    {
      projectId,
      state: stated(project.state),
      createdAt: stated(project.created_at),
      activatedAt: stated(project.activated_at),
      acceptedAt: stated(project.accepted_at),
      charterStatus: stated(charter.status),
      charterDigest: stated(charter.content_digest),
      planRevisionId: stated(plan.plan_revision_id),
      pendingPreviewCount:
        typeof record.pending_preview_count === "number"
          ? String(record.pending_preview_count)
          : "unknown",
      cost: stated(record.cost),
    },
  ];
}

export function projectProjectAxis(body: unknown): ProjectAxisStageRow[] {
  const rows: ProjectAxisStageRow[] = [];
  for (const item of asList(body, ["stages"])) {
    const record = asRecord(item);
    if (typeof record.stage_id !== "string" || record.stage_id.length === 0) {
      continue;
    }
    const output = asRecord(record.output_contract);
    const gaps = Array.isArray(record.gaps) ? record.gaps : [];
    rows.push({
      stageId: record.stage_id,
      position: typeof record.position === "number" ? String(record.position) : stated(record.position),
      title: stated(record.title),
      objective: stated(record.objective),
      confirmStatus: stated(record.confirm_status),
      ready: typeof record.ready === "boolean" ? String(record.ready) : stated(record.ready),
      seated: typeof record.seated === "boolean" ? String(record.seated) : stated(record.seated),
      outputDigest: stated(output.digest),
      deliverableType: stated(output.deliverable_type),
      saveFormat: stated(output.save_format),
      openWith: stated(output.open_with),
      gapCount: String(gaps.length),
      responsibleSlot: stated(record.responsible_slot),
    });
  }
  return rows;
}

export function projectProjectRoster(body: unknown): ProjectRosterRow[] {
  const record = asRecord(body);
  const authorityNote = stated(record.authority_note, "employee");
  const rows: ProjectRosterRow[] = [];
  for (const item of asList(body, ["roster"])) {
    const row = asRecord(item);
    if (typeof row.employee_id !== "string" || row.employee_id.length === 0) {
      continue;
    }
    rows.push({
      employeeId: row.employee_id,
      state: stated(row.state),
      modelBound: typeof row.model_bound === "boolean" ? String(row.model_bound) : stated(row.model_bound),
      isCurrentManager:
        typeof row.is_current_manager === "boolean"
          ? String(row.is_current_manager)
          : stated(row.is_current_manager),
      runtimeBindingRef: stated(row.runtime_binding_ref),
      authorityNote,
      responsibleStageIds: Array.isArray(row.responsible_stage_ids)
        ? row.responsible_stage_ids.filter((id): id is string => typeof id === "string").join(", ")
        : stated(row.responsible_stage_ids),
    });
  }
  return rows;
}

export function projectEmployeeCatalog(body: unknown): ProjectCatalogRow[] {
  const record = asRecord(body);
  const authorityNote = stated(record.authority_note, "grant");
  const rows: ProjectCatalogRow[] = [];
  const catalog = Array.isArray(record.catalog) ? record.catalog : [];
  for (const item of catalog) {
    if (typeof item !== "string" || item.length === 0) {
      continue;
    }
    rows.push({ capabilityRef: item, authorityNote });
  }
  return rows;
}

export function uniqueResponsibleSlots(stages: ProjectAxisStageRow[]): string[] {
  const seen = new Set<string>();
  const slots: string[] = [];
  for (const stage of stages) {
    if (stage.responsibleSlot.length === 0 || stage.responsibleSlot === "unknown") {
      continue;
    }
    if (!seen.has(stage.responsibleSlot)) {
      seen.add(stage.responsibleSlot);
      slots.push(stage.responsibleSlot);
    }
  }
  return slots;
}
