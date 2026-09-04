import { describe, expect, it } from "vitest";
import {
  PROJECT_COPY_PATH,
  PROJECT_EXPORT_PATH,
  copyReply,
  projectLifecycle,
  projectLifecyclePath,
  refusesBackupClaim,
  refusesSecretExport,
} from "./projectLifecycle";

describe("P13-T09 project lifecycle projections", () => {
  it("keeps restore points from being a backup and export from carrying secrets", () => {
    expect(PROJECT_COPY_PATH).toBe("/management/project/v1/copy");
    expect(PROJECT_EXPORT_PATH).toBe("/management/project/v1/export");
    expect(projectLifecyclePath("proj/1")).toBe(
      "/management/project/v1/lifecycle?project_id=proj%2F1",
    );
    expect(refusesSecretExport(true)).toBe(true);
    expect(refusesSecretExport(false)).toBe(false);
    expect(refusesBackupClaim(true)).toBe(true);
    const view = projectLifecycle({
      project_id: "proj-1",
      state: "archived",
      data_dir: "projects/proj-1",
      logically_deleted: false,
      is_disaster_backup: true,
      events: [{ event_id: "e1", kind: "archive", created_at: 1, is_disaster_backup: true }],
      restore_points: [
        { event_id: "r1", version_name: "local-restore-v1", created_at: 2, is_disaster_backup: true },
      ],
    });
    expect(view?.isDisasterBackup).toBe(false);
    expect(view?.restorePoints[0]?.isDisasterBackup).toBe(false);
    expect(copyReply({ copy_project_id: "proj-2", state: "inactive" })).toEqual({
      copyProjectId: "proj-2",
      state: "inactive",
    });
    expect(copyReply({ state: "active" })).toBeUndefined();
  });
});
