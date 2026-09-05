import { describe, expect, it } from "vitest";
import { isKnownRoute } from "../normalize";
import {
  ATTEMPT_RUN_PATH,
  attemptListPath,
  ledgerDecisionRows,
  projectAttemptHistory,
  projectRoutineRuns,
  routineRunsPath,
} from "./routineRuns";

const RUNS_BODY = {
  status: "ok",
  projection_id: "personal-private.routine-arming/0.1",
  project_id: "proj-1",
  host: { available: true, reason: null },
  scheduler: "daemon-tick-only",
  armings: [
    {
      arming_id: "arm-1",
      project_id: "proj-1",
      routine_id: "rt-1",
      revision_id: "rev-1",
      stage_id: "s1",
      employee_id: "emp-1",
      cadence_kind: "interval",
      interval_ms: 5000,
      state: "armed",
      apply_mode: "continue",
      armed_after: "G2",
      next_due_at: 1000,
      last_fired_at: null,
    },
    { arming_id: "", routine_id: "ghost" },
  ],
  occurrences: [
    {
      occurrence_id: "occ-1",
      routine_id: "rt-1",
      revision_id: "rev-1",
      trigger_kind: "schedule",
      trigger_source: "daemon-tick",
      requested_at: 900,
      disposition: "active",
      dispatch_state: "running",
      coalesced_by: null,
      miss_reason: null,
      attempt_id: "att-1",
      attempt_outcome: null,
      completion_claimed: false,
      verification_status: "not-run",
    },
    {
      occurrence_id: "occ-2",
      routine_id: "rt-1",
      revision_id: "rev-1",
      trigger_kind: "manual",
      trigger_source: "owner-run",
      requested_at: 950,
      disposition: "coalesced",
      dispatch_state: "coalesced",
      coalesced_by: "occ-3",
      miss_reason: null,
      completion_claimed: false,
      verification_status: "not-run",
    },
    {
      occurrence_id: "occ-3",
      routine_id: "rt-1",
      revision_id: "rev-1",
      trigger_kind: "manual",
      trigger_source: "owner-run",
      requested_at: 960,
      disposition: "queued",
      dispatch_state: "queued",
      completion_claimed: false,
      verification_status: "not-run",
    },
    {
      occurrence_id: "occ-4",
      routine_id: "rt-1",
      revision_id: "rev-1",
      trigger_kind: "schedule",
      trigger_source: "daemon-tick",
      requested_at: 500,
      disposition: "missed",
      dispatch_state: "missed",
      miss_reason: "host-unavailable:close-paused",
      completion_claimed: false,
      verification_status: "not-run",
    },
    { occurrence_id: "", disposition: "active" },
  ],
  summary: {
    active: 1,
    running: 1,
    queued: 1,
    missed: 1,
    coalesced: 1,
    attempted: 0,
    done: 0,
    failed: 0,
    unknown: 0,
  },
  attempt_history_path: "/management/project/v1/dsh.hosted.attempt.list?project_id=proj-1",
  manual_trigger_path: "/management/project/v1/routine.trigger",
  receipt_is_not_completion: true,
  verification_status: "not-run",
  clock_sleep_restart_host_e2e: "not-run",
};

describe("P13-T05/D02 routine runs projection", () => {
  it("builds paths without inventing a routine_id", () => {
    expect(routineRunsPath("proj 1")).toBe("/management/project/v1/routine.runs?project_id=proj%201");
    expect(attemptListPath("proj-1")).toBe(
      "/management/project/v1/dsh.hosted.attempt.list?project_id=proj-1",
    );
  });

  it("projects the ledger with queue / miss / coalesce facts and drops id-less rows", () => {
    const [view] = projectRoutineRuns(RUNS_BODY);
    expect(view.projectId).toBe("proj-1");
    expect(view.scheduler).toBe("daemon-tick-only");
    expect(view.hostAvailable).toBe("true");
    expect(view.armings).toHaveLength(1);
    expect(view.armings[0].armedAfter).toBe("G2");
    expect(view.armings[0].intervalMs).toBe("5000");
    expect(view.occurrences.map((row) => row.occurrenceId)).toEqual([
      "occ-1",
      "occ-2",
      "occ-3",
      "occ-4",
    ]);
    expect(view.occurrences[0].dispatchState).toBe("running");
    expect(view.occurrences[0].attemptId).toBe("att-1");
    expect(view.occurrences[1].coalescedBy).toBe("occ-3");
    expect(view.occurrences[3].missReason).toBe("host-unavailable:close-paused");
    expect(view.occurrences.every((row) => row.completionClaimed === "false")).toBe(true);
    expect(view.occurrences.every((row) => row.verificationStatus === "not-run")).toBe(true);
    expect(view.receiptIsNotCompletion).toBe("true");
    expect(view.clockSleepRestartHostE2e).toBe("not-run");
    expect(ledgerDecisionRows(view).map((row) => row.occurrenceId)).toEqual([
      "occ-2",
      "occ-3",
      "occ-4",
    ]);
  });

  it("keeps summary counts daemon-stated and never turns a missing count into 0", () => {
    const [view] = projectRoutineRuns(RUNS_BODY);
    expect(view.summary.missed).toBe("1");
    expect(view.summary.coalesced).toBe("1");
    expect(view.summary.done).toBe("0");
    const [partial] = projectRoutineRuns({ ...RUNS_BODY, summary: { active: "1" } });
    expect(partial.summary.active).toBe("unknown");
    expect(partial.summary.missed).toBe("unknown");
  });

  it("returns no view row without a project_id and still reads an empty ledger as a view", () => {
    expect(projectRoutineRuns({ status: "ok" })).toEqual([]);
    expect(projectRoutineRuns(null)).toEqual([]);
    const [empty] = projectRoutineRuns({
      status: "ok",
      project_id: "proj-1",
      armings: [],
      occurrences: [],
      summary: {},
    });
    expect(empty.occurrences).toEqual([]);
    expect(empty.armings).toEqual([]);
    expect(empty.summary.active).toBe("unknown");
    expect(empty.scheduler).toBe("unknown");
  });

  it("projects Attempt history as stated facts, never as completion", () => {
    const rows = projectAttemptHistory({
      projection: "personal-private.hosted-attempt/0.1",
      attempts: [
        {
          attempt_id: "att-1",
          employee_id: "emp-1",
          task_ref: "task://personal/routine/occ-1",
          state: "terminal",
          terminal_kind: "exited",
          exit_code: 0,
          response_status: "done",
          completion_claimed: false,
          verification_status: "not-run",
          elapsed_ms: 1234,
          created_at: 1,
          terminal_at: 2,
        },
        { attempt_id: "", state: "terminal" },
        "junk",
      ],
      receipt_is_not_completion: true,
    });
    expect(rows).toHaveLength(1);
    expect(rows[0].responseStatus).toBe("done");
    expect(rows[0].completionClaimed).toBe("false");
    expect(rows[0].verificationStatus).toBe("not-run");
    expect(rows[0].exitCode).toBe("0");
    expect(projectAttemptHistory({ attempts: "nope" })).toEqual([]);
  });

  it("names the management Write Attempt route without whitelisting the task alias", () => {
    expect(ATTEMPT_RUN_PATH).toBe("/management/project/v1/dsh.hosted.attempt.run");
    expect(isKnownRoute("POST", ATTEMPT_RUN_PATH)).toBe(true);
    expect(isKnownRoute("POST", "/task/project/v1/dsh.hosted.attempt.run")).toBe(false);
  });
});
