import { describe, expect, it } from "vitest";
import {
  REFLECTION_GENERATE_PATH,
  REFLECTION_IMPROVE_PROPOSE_PATH,
  generateBody,
  proposeImprovementBody,
  reflectionListPath,
  roleTemplateBody,
  rollbackBody,
} from "./memberReflection";

describe("P13-T11 member reflection Dual Track", () => {
  it("builds generate / propose / rollback bodies without claiming apply", () => {
    expect(REFLECTION_GENERATE_PATH).toBe("/management/project/v1/reflection.generate");
    expect(REFLECTION_IMPROVE_PROPOSE_PATH).toBe(
      "/management/project/v1/reflection.improve.propose",
    );
    expect(generateBody("proj-1")).toEqual({ project_id: "proj-1" });
    expect(
      proposeImprovementBody({
        candidateId: "cand-1",
        proposedPrompt: "tighten",
        proposedTools: ["workspace-write"],
      }),
    ).toEqual({
      candidate_id: "cand-1",
      proposed_prompt: "tighten",
      proposed_tools: ["workspace-write"],
    });
    expect(rollbackBody("improve-1")).toEqual({ improvement_id: "improve-1" });
    expect(roleTemplateBody("emp-1")).toEqual({ employee_id: "emp-1" });
    expect(reflectionListPath("proj-1", "emp-1")).toContain("employee_id=emp-1");
  });

  it("forwards an implicit Blueprint field so the daemon can refuse it", () => {
    expect(
      proposeImprovementBody({
        candidateId: "cand-1",
        proposedPrompt: "tighten",
        proposedTools: [],
        newBlueprintRevisionId: "role-blueprint-rev:forged",
      }),
    ).toMatchObject({
      new_blueprint_revision_id: "role-blueprint-rev:forged",
    });
  });
});
