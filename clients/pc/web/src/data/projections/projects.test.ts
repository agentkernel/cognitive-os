import { describe, expect, it } from "vitest";
import { firstReadyProjectId, projectProjectList } from "./projects";

describe("project list projection (P11-T13)", () => {
  it("maps daemon rows and keeps title/cost as stated", () => {
    const rows = projectProjectList({
      status: "ok",
      projects: [
        {
          project_id: "proj-1",
          state: "active",
          title_summary: "unknown",
          cost: "unknown",
        },
      ],
    });
    expect(rows).toEqual([
      {
        projectId: "proj-1",
        state: "active",
        titleSummary: "unknown",
        cost: "unknown",
      },
    ]);
  });

  it("does not invent a Project from an empty or malformed body", () => {
    expect(projectProjectList({ status: "ok", projects: [] })).toEqual([]);
    expect(projectProjectList({ status: "ok" })).toEqual([]);
    expect(projectProjectList(null)).toEqual([]);
    expect(projectProjectList({ projects: [{ state: "active" }] })).toEqual([]);
  });

  it("does not invent a first Project id", () => {
    expect(firstReadyProjectId(undefined)).toBeUndefined();
    expect(firstReadyProjectId([])).toBeUndefined();
    expect(firstReadyProjectId([{ projectId: "", state: "active", titleSummary: "unknown", cost: "unknown" }])).toBeUndefined();
    expect(
      firstReadyProjectId([{ projectId: "proj-1", state: "active", titleSummary: "unknown", cost: "unknown" }]),
    ).toBe("proj-1");
  });
});
