import { describe, expect, it } from "vitest";
import { interpretCandidate, uuidV7, workspaceSearchDraft } from "./taskDraft";

describe("uuid v7", () => {
  it("emits lowercase hyphenated ids with version 7", () => {
    const id = uuidV7();
    expect(id).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/,
    );
  });
});

describe("workspace search draft", () => {
  it("does not include bash/edit/write or secret fields", () => {
    const draft = workspaceSearchDraft("search the workspace for needle");
    expect(draft.allowed_tools).toEqual(["native.workspace.search"]);
    expect(draft.scope.out_of_scope).toEqual(["bash", "edit", "write"]);
    expect(JSON.stringify(draft)).not.toMatch(/api_key|sk-|ss:\/\//);
    expect(draft.task_ref.startsWith("task://personal/web-ui/")).toBe(true);
  });
});

describe("interpret candidate", () => {
  it("copies the objective and invents no Provider override", () => {
    const candidate = interpretCandidate("search the workspace for needle");
    expect(candidate.objectives).toEqual(["search the workspace for needle"]);
    expect(JSON.stringify(candidate)).not.toMatch(/fallback|api_key/);
  });
});
