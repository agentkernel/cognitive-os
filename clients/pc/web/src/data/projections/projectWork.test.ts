import { describe, expect, it } from "vitest";
import {
  projectEmployeeCatalog,
  projectProjectAxis,
  projectProjectDetail,
  projectProjectRoster,
  uniqueResponsibleSlots,
} from "./projectWork";

describe("project work projections (P12-T03)", () => {
  it("maps daemon detail without inventing title or cost", () => {
    const rows = projectProjectDetail({
      status: "ok",
      project: {
        project_id: "proj-1",
        state: "active",
        created_at: "t0",
        activated_at: "t1",
        accepted_at: null,
      },
      charter: { status: "confirmed", content_digest: "dig-1" },
      plan: { plan_revision_id: "plan-1" },
      pending_preview_count: 0,
      cost: "unknown",
    });
    expect(rows).toEqual([
      {
        projectId: "proj-1",
        state: "active",
        createdAt: "t0",
        activatedAt: "t1",
        acceptedAt: "unknown",
        charterStatus: "confirmed",
        charterDigest: "dig-1",
        planRevisionId: "plan-1",
        pendingPreviewCount: "0",
        cost: "unknown",
      },
    ]);
  });

  it("does not invent a Project from a missing project_id", () => {
    expect(projectProjectDetail({ status: "ok", project: { state: "active" } })).toEqual([]);
    expect(projectProjectDetail(null)).toEqual([]);
  });

  it("maps PlanRevision axis stages including unknown output contracts", () => {
    const rows = projectProjectAxis({
      status: "ok",
      plan_revision_id: "plan-1",
      stages: [
        {
          stage_id: "st-1",
          position: 0,
          title: "Intake",
          confirm_status: "confirmed",
          ready: false,
          seated: true,
          output_contract: {
            digest: "out-1",
            deliverable_type: "unknown",
            save_format: "unknown",
            open_with: "unknown",
          },
          gaps: [{ gap_id: "g1" }],
        },
      ],
    });
    expect(rows[0]).toMatchObject({
      stageId: "st-1",
      position: "0",
      title: "Intake",
      confirmStatus: "confirmed",
      ready: "false",
      seated: "true",
      deliverableType: "unknown",
      gapCount: "1",
      responsibleSlot: "unknown",
    });
  });

  it("does not invent stages from an empty axis", () => {
    expect(projectProjectAxis({ status: "ok", stages: [] })).toEqual([]);
    expect(projectProjectAxis({ status: "ok" })).toEqual([]);
  });

  it("maps roster rows and keeps empty-roster as empty", () => {
    expect(
      projectProjectRoster({
        status: "ok",
        roster: [],
        authority_note: "empty-roster",
      }),
    ).toEqual([]);
    const rows = projectProjectRoster({
      status: "ok",
      authority_note: "employee",
      roster: [
        {
          employee_id: "emp-1",
          state: "seated",
          model_bound: true,
          is_current_manager: true,
          runtime_binding_ref: "run-1",
        },
      ],
    });
    expect(rows).toEqual([
      {
        employeeId: "emp-1",
        state: "seated",
        modelBound: "true",
        isCurrentManager: "true",
        runtimeBindingRef: "run-1",
        authorityNote: "employee",
        responsibleStageIds: "unknown",
      },
    ]);
  });

  it("maps grant catalog without treating recipe mentions as grants", () => {
    expect(projectEmployeeCatalog({ status: "ok", catalog: [] })).toEqual([]);
    expect(
      projectEmployeeCatalog({ status: "ok", catalog: ["tool.read", "skill.pack"] }),
    ).toEqual([
      { capabilityRef: "tool.read", authorityNote: "grant" },
      { capabilityRef: "skill.pack", authorityNote: "grant" },
    ]);
  });

  it("dedupes responsible slots without inventing missing ones", () => {
    expect(
      uniqueResponsibleSlots([
        {
          stageId: "a",
          position: "0",
          title: "A",
          objective: "unknown",
          confirmStatus: "unknown",
          ready: "false",
          seated: "false",
          outputDigest: "unknown",
          deliverableType: "unknown",
          saveFormat: "unknown",
          openWith: "unknown",
          gapCount: "0",
          responsibleSlot: "manager",
        },
        {
          stageId: "b",
          position: "1",
          title: "B",
          objective: "unknown",
          confirmStatus: "unknown",
          ready: "false",
          seated: "false",
          outputDigest: "unknown",
          deliverableType: "unknown",
          saveFormat: "unknown",
          openWith: "unknown",
          gapCount: "0",
          responsibleSlot: "manager",
        },
      ]),
    ).toEqual(["manager"]);
    expect(uniqueResponsibleSlots([])).toEqual([]);
  });

  it("exposes Dual Track process-ring slots instead of collapsed owner", () => {
    const ring = (stageId: string, slot: string, position: string) => ({
      stageId,
      position,
      title: stageId,
      objective: "unknown",
      confirmStatus: "unknown",
      ready: "false",
      seated: "false",
      outputDigest: "unknown",
      deliverableType: "unknown",
      saveFormat: "unknown",
      openWith: "unknown",
      gapCount: "0",
      responsibleSlot: slot,
    });
    expect(
      uniqueResponsibleSlots([
        ring("collect", "collect", "0"),
        ring("analyze", "analyze", "1"),
        ring("draft", "draft", "2"),
      ]),
    ).toEqual(["collect", "analyze", "draft"]);
  });
});
