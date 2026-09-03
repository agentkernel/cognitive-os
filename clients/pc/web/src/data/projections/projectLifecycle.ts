/**
 * Project lifecycle projections (P13-T09). Copy is an inactive 副本.
 * Archive stops triggers. Delete is impact preview + second confirmation
 * and is never a physical drop. Restore points are same-disk and not a
 * disaster backup. Export is not authority and never includes secrets.
 * These paths are posted through `readJson` (T08 owns the fetchProjection
 * allowlist).
 */

import { asList, asRecord } from "../projections";

export const PROJECT_COPY_PATH = "/management/project/v1/copy";
export const PROJECT_ARCHIVE_PATH = "/management/project/v1/archive";
export const PROJECT_DELETE_PREVIEW_PATH = "/management/project/v1/delete.preview";
export const PROJECT_DELETE_CONFIRM_PATH = "/management/project/v1/delete.confirm";
export const PROJECT_RESTORE_POINT_PATH = "/management/project/v1/restore-point";
export const PROJECT_EXPORT_PATH = "/management/project/v1/export";
export const PROJECT_LIFECYCLE_PATH = "/management/project/v1/lifecycle";

export function projectLifecyclePath(projectId: string): string {
  return `${PROJECT_LIFECYCLE_PATH}?project_id=${encodeURIComponent(projectId)}`;
}

export function projectLifecycleKey(projectId: string): string {
  return `opc:project-lifecycle:${projectId}`;
}

export interface LifecycleEventRow {
  eventId: string;
  kind: string;
  createdAt: string;
  copyProjectId: string;
  dataDir: string;
  isDisasterBackup: false;
}

export interface RestorePointRow {
  eventId: string;
  versionName: string;
  createdAt: string;
  isDisasterBackup: false;
}

export interface DeletePreviewRow {
  previewId: string;
  confirmationDigest: string;
  triggersStopped: boolean;
  status: string;
}

export interface ProjectLifecycleRow {
  projectId: string;
  state: string;
  dataDir: string;
  logicallyDeleted: boolean;
  isDisasterBackup: false;
  events: LifecycleEventRow[];
  restorePoints: RestorePointRow[];
  pendingDeletePreview: DeletePreviewRow | undefined;
}

function asText(value: unknown): string {
  if (typeof value === "string" && value.length > 0) {
    return value;
  }
  if (typeof value === "number" && Number.isFinite(value)) {
    return String(value);
  }
  return "unknown";
}

export function projectLifecycle(body: unknown): ProjectLifecycleRow | undefined {
  const record = asRecord(body);
  const projectId = asText(record.project_id);
  if (projectId === "unknown") {
    return undefined;
  }
  const pending = asRecord(record.pending_delete_preview);
  return {
    projectId,
    state: asText(record.state),
    dataDir: asText(record.data_dir),
    logicallyDeleted: record.logically_deleted === true,
    isDisasterBackup: false,
    events: asList(record, ["events"]).map((row) => {
      const item = asRecord(row);
      return {
        eventId: asText(item.event_id),
        kind: asText(item.kind),
        createdAt: asText(item.created_at),
        copyProjectId: asText(item.copy_project_id),
        dataDir: asText(item.data_dir),
        isDisasterBackup: false as const,
      };
    }),
    restorePoints: asList(record, ["restore_points"]).map((row) => {
      const item = asRecord(row);
      return {
        eventId: asText(item.event_id),
        versionName: asText(item.version_name),
        createdAt: asText(item.created_at),
        isDisasterBackup: false as const,
      };
    }),
    pendingDeletePreview:
      asText(pending.preview_id) === "unknown"
        ? undefined
        : {
            previewId: asText(pending.preview_id),
            confirmationDigest: asText(pending.confirmation_digest),
            triggersStopped: pending.triggers_stopped === true,
            status: asText(pending.status),
          },
  };
}

export function copyReply(body: unknown): { copyProjectId: string; state: string } | undefined {
  const record = asRecord(body);
  const copyProjectId = asText(record.copy_project_id);
  if (copyProjectId === "unknown") {
    return undefined;
  }
  return { copyProjectId, state: asText(record.state) };
}

export function refusesSecretExport(includeSecrets: boolean): boolean {
  return includeSecrets;
}

export function refusesBackupClaim(claimedAsBackup: boolean): boolean {
  return claimedAsBackup;
}
