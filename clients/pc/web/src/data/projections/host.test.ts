import { describe, expect, it } from "vitest";
import {
  hostCanHonorBackground,
  hostStatusPath,
  projectHostStatus,
  HOST_CLOSE_PATH,
  HOST_STATUS_PATH,
} from "./host";

describe("host status projection (P12-T08)", () => {
  it("maps daemon status without inventing a home or proving tray work", () => {
    const rows = projectHostStatus({
      status: "ok",
      home_id: "home-1",
      daemon_id: "daemon-1",
      daemon_state: "bound",
      can_honor_background: true,
      close_disposition: "background-honored",
      tray_proves_work: false,
    });
    expect(rows).toEqual([
      {
        homeId: "home-1",
        daemonId: "daemon-1",
        daemonState: "bound",
        canHonorBackground: "true",
        closeDisposition: "background-honored",
        trayProvesWork: "false",
      },
    ]);
    expect(hostCanHonorBackground(rows[0])).toBe(true);
  });

  it("keeps unknown honor and missing home as empty, never as a fake background", () => {
    expect(projectHostStatus({ status: "ok" })).toEqual([]);
    expect(projectHostStatus(null)).toEqual([]);
    const unknownHonor = projectHostStatus({ home_id: "home-2" });
    expect(unknownHonor[0]?.canHonorBackground).toBe("unknown");
    expect(hostCanHonorBackground(unknownHonor[0])).toBe(false);
    expect(projectHostStatus({ home_id: "home-3", can_honor_background: false })[0]?.canHonorBackground).toBe(
      "false",
    );
    expect(hostCanHonorBackground(projectHostStatus({ home_id: "home-3", can_honor_background: false })[0])).toBe(
      false,
    );
  });

  it("binds status to home_id and keeps close on management HTTP", () => {
    expect(hostStatusPath("home-1")).toBe(`${HOST_STATUS_PATH}?home_id=home-1`);
    expect(HOST_CLOSE_PATH).toBe("/management/host/v1/close.request");
  });
});
