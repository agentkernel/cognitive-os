/**
 * P13-T11 Dual Track: daemon-generated reflection + versioned Member Runtime.
 * Confirm stays on the HITL canvas. No Admit / Approve / fake apply.
 */

export const REFLECTION_GENERATE_PATH = "/management/project/v1/reflection.generate";
export const REFLECTION_LIST_PATH = "/management/project/v1/reflection.list";
export const REFLECTION_IMPROVE_PROPOSE_PATH =
  "/management/project/v1/reflection.improve.propose";
export const REFLECTION_IMPROVE_ROLLBACK_PATH =
  "/management/project/v1/reflection.improve.rollback";
export const REFLECTION_ROLE_TEMPLATE_PROPOSE_PATH =
  "/management/project/v1/reflection.role-template.propose";

export function reflectionListPath(projectId: string, employeeId: string): string {
  return `${REFLECTION_LIST_PATH}?project_id=${encodeURIComponent(projectId)}&employee_id=${encodeURIComponent(employeeId)}`;
}

export function generateBody(projectId: string): Record<string, unknown> {
  return { project_id: projectId };
}

export function proposeImprovementBody(input: {
  candidateId: string;
  proposedPrompt: string;
  proposedTools: string[];
  newBlueprintRevisionId?: string;
}): Record<string, unknown> {
  const body: Record<string, unknown> = {
    candidate_id: input.candidateId,
    proposed_prompt: input.proposedPrompt,
    proposed_tools: input.proposedTools,
  };
  if (input.newBlueprintRevisionId && input.newBlueprintRevisionId.length > 0) {
    body.new_blueprint_revision_id = input.newBlueprintRevisionId;
  }
  return body;
}

export function rollbackBody(improvementId: string): Record<string, unknown> {
  return { improvement_id: improvementId };
}

export function roleTemplateBody(employeeId: string): Record<string, unknown> {
  return { employee_id: employeeId };
}
