import { describe, expect, it } from "vitest";
import {
  formatDuration,
  isTodayPeriod,
  projectTodayOverview,
  todayOverviewPath,
} from "./todayOverview";

const OVERVIEW_BODY = {
  status: "ok",
  projection_id: "personal-private.today-overview/0.1",
  period: "today",
  period_start_ms: 0,
  now_ms: 10,
  period_basis: "utc",
  counts: { created: 1, live: 2, blocked: 0 },
  rows: [
    {
      project_id: "proj-1",
      state: "active",
      status: "running",
      armed_routines: 1,
      paused_routines: 0,
      running_occurrence_id: "occ-9",
      running_since: 5,
      queued_count: 1,
      missed_count: 2,
      attempts_total: 3,
      attempts_done: 2,
      attempts_failed: 1,
      attempts_unknown: 0,
      duration_ms: 65000,
      current_stage_id: "s1",
      current_stage_title: "Draft",
      last_terminal_at: 9,
      cost: "unknown",
    },
    {
      project_id: "proj-2",
      state: "paused",
      status: "paused",
      armed_routines: 0,
      paused_routines: 1,
      running_occurrence_id: null,
      queued_count: 0,
      missed_count: 0,
      attempts_total: 0,
      attempts_done: 0,
      attempts_failed: 0,
      attempts_unknown: 0,
      duration_ms: null,
      current_stage_id: null,
      current_stage_title: null,
      last_terminal_at: null,
      cost: "unknown",
    },
    { project_id: "", status: "running" },
  ],
  kpi_wall: false,
  verification_status: "not-run",
  cost: "unknown",
};

describe("P13-T05/D02 Today overview projection", () => {
  it("builds period paths and refuses unknown periods", () => {
    expect(todayOverviewPath("today")).toBe("/management/project/v1/today.overview?period=today");
    expect(todayOverviewPath("week")).toBe("/management/project/v1/today.overview?period=week");
    expect(isTodayPeriod("month")).toBe(true);
    expect(isTodayPeriod("year")).toBe(false);
    expect(isTodayPeriod("")).toBe(false);
  });

  it("projects one row per live Project with daemon-stated counts", () => {
    const [view] = projectTodayOverview(OVERVIEW_BODY);
    expect(view.period).toBe("today");
    expect(view.created).toBe("1");
    expect(view.live).toBe("2");
    expect(view.blocked).toBe("0");
    expect(view.kpiWall).toBe("false");
    expect(view.verificationStatus).toBe("not-run");
    expect(view.rows.map((row) => row.projectId)).toEqual(["proj-1", "proj-2"]);
    expect(view.rows[0].status).toBe("running");
    expect(view.rows[0].attemptsDone).toBe("2");
    expect(view.rows[0].missedCount).toBe("2");
    expect(view.rows[0].currentStageId).toBe("s1");
    expect(view.rows[0].currentStageTitle).toBe("Draft");
    expect(view.rows[0].durationMs).toBe("65000");
    expect(view.rows[1].status).toBe("paused");
    expect(view.rows[1].durationMs).toBe("unknown");
    expect(view.rows[1].currentStageId).toBe("—");
    expect(view.rows[1].cost).toBe("unknown");
  });

  it("never turns a missing count into 0 and keeps zero live Projects as a ready view", () => {
    const [view] = projectTodayOverview({
      period: "week",
      counts: { created: 0 },
      rows: [],
    });
    expect(view.created).toBe("0");
    expect(view.live).toBe("unknown");
    expect(view.blocked).toBe("unknown");
    expect(view.rows).toEqual([]);
    expect(view.cost).toBe("unknown");
  });

  it("returns no view without a valid period so the page cannot fake an overview", () => {
    expect(projectTodayOverview({ status: "ok", counts: { live: 1 } })).toEqual([]);
    expect(projectTodayOverview({ period: "year", counts: { live: 1 } })).toEqual([]);
    expect(projectTodayOverview(null)).toEqual([]);
  });

  it("formats stated durations and leaves unknown untouched", () => {
    expect(formatDuration("unknown")).toBe("unknown");
    expect(formatDuration("500")).toBe("500 ms");
    expect(formatDuration("65000")).toBe("1 min 5 s");
    expect(formatDuration("3720000")).toBe("1 h 2 min");
  });
});
