/**
 * P13-T10 Dual Track: Skill/MCP acquire writes. Install is not a grant.
 * Confirm stays on the HITL canvas. Chat cannot Approve.
 */

export const CAPABILITY_DISCOVER_PATH = "/management/project/v1/capability.discover";
export const CAPABILITY_ACQUIRE_PATH = "/management/project/v1/capability.acquire";
export const CAPABILITY_COMPAT_PATH = "/management/project/v1/capability.compat-test";
export const CAPABILITY_ROLLBACK_PATH = "/management/project/v1/capability.rollback";

export type AcquirePhase = "install" | "grant";

export interface SecurityReviewDraft {
  source: string;
  license: string;
  hiddenInstruction: string;
  promptInjection: string;
  fileIntent: string;
  networkIntent: string;
  commandIntent: string;
  dependencies: string;
  executableCode: string;
  secretAccess: string;
  toolPermissions: string;
  supplyChain: string;
  sources: string;
}

export const EMPTY_REVIEW: SecurityReviewDraft = {
  source: "",
  license: "",
  hiddenInstruction: "none",
  promptInjection: "none",
  fileIntent: "none",
  networkIntent: "none",
  commandIntent: "none",
  dependencies: "none",
  executableCode: "none",
  secretAccess: "none",
  toolPermissions: "",
  supplyChain: "",
  sources: "",
};

export function reviewIsComplete(kind: string, draft: SecurityReviewDraft): boolean {
  if (
    draft.source.trim().length === 0 ||
    draft.license.trim().length === 0 ||
    draft.hiddenInstruction.trim().length === 0 ||
    draft.promptInjection.trim().length === 0 ||
    draft.sources.trim().length === 0
  ) {
    return false;
  }
  if (kind === "mcp") {
    return (
      draft.dependencies.trim().length > 0 &&
      draft.executableCode.trim().length > 0 &&
      draft.secretAccess.trim().length > 0 &&
      draft.toolPermissions.trim().length > 0 &&
      draft.supplyChain.trim().length > 0
    );
  }
  return true;
}

export function acquireBody(input: {
  projectId: string;
  employeeId: string;
  capabilityRef: string;
  versionPin: string;
  kind: string;
  scope: string;
  phase: AcquirePhase;
  draft: SecurityReviewDraft;
}): Record<string, unknown> {
  const sources = input.draft.sources
    .split(/\s+/)
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
  return {
    project_id: input.projectId,
    employee_id: input.employeeId,
    capability_ref: input.capabilityRef,
    version_pin: input.versionPin,
    kind: input.kind,
    scope: input.scope,
    phase: input.phase,
    review: {
      source: input.draft.source.trim(),
      license: input.draft.license.trim(),
      hidden_instruction: input.draft.hiddenInstruction.trim(),
      prompt_injection: input.draft.promptInjection.trim(),
      file_intent: input.draft.fileIntent.trim(),
      network_intent: input.draft.networkIntent.trim(),
      command_intent: input.draft.commandIntent.trim(),
      dependencies: input.draft.dependencies.trim(),
      executable_code: input.draft.executableCode.trim(),
      secret_access: input.draft.secretAccess.trim(),
      tool_permissions: input.draft.toolPermissions.trim(),
      supply_chain: input.draft.supplyChain.trim(),
      sources,
    },
  };
}
