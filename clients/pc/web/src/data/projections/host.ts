/**
 * Windows host close-background projection (P12-T08). Source is
 * GET /management/host/v1/status?home_id=. Close is POST close.request
 * with choice background|pause. Tray does not write authority. Native
 * install/tray E2E is not-run.
 */

import { asRecord } from "../projections";

export const HOST_STATUS_KEY = "opc:host-status";
export const HOST_STATUS_PATH = "/management/host/v1/status";
export const HOST_CLOSE_PATH = "/management/host/v1/close.request";

export interface HostStatusRow {
  homeId: string;
  daemonId: string;
  daemonState: string;
  canHonorBackground: string;
  closeDisposition: string;
  trayProvesWork: string;
}

export function hostStatusPath(homeId: string): string {
  return `${HOST_STATUS_PATH}?home_id=${encodeURIComponent(homeId)}`;
}

export function projectHostStatus(body: unknown): HostStatusRow[] {
  const record = asRecord(body);
  if (typeof record.home_id !== "string" || record.home_id.length === 0) {
    return [];
  }
  return [
    {
      homeId: record.home_id,
      daemonId: typeof record.daemon_id === "string" ? record.daemon_id : "unknown",
      daemonState:
        typeof record.daemon_state === "string" ? record.daemon_state : "unknown",
      canHonorBackground:
        record.can_honor_background === true
          ? "true"
          : record.can_honor_background === false
            ? "false"
            : "unknown",
      closeDisposition:
        typeof record.close_disposition === "string" && record.close_disposition.length > 0
          ? record.close_disposition
          : "unknown",
      trayProvesWork:
        record.tray_proves_work === true
          ? "true"
          : record.tray_proves_work === false
            ? "false"
            : "unknown",
    },
  ];
}

export function hostCanHonorBackground(row: HostStatusRow | undefined): boolean {
  return row?.canHonorBackground === "true";
}
