/**
 * Tool-call policy for the CognitiveOS Pi Extension.
 *
 * The Extension is a non-authority client (PERS-PR-005; ADR-0022 §9). It cannot
 * mint a capability, open an Intent, persist an Effect or advance a Task, so it
 * has nothing with which to authorize a tool. Consequently the policy is
 * **default-deny**: every Pi built-in tool call is refused, and the three known
 * mutating built-ins get a specific reason so the refusal is legible.
 *
 * ADR-0026 puts the same conclusion the other way round: tier classification is
 * a property of a catalog-bound operation, and "unknown or unclassifiable
 * operations default to Tier 2". The Extension has no catalog, therefore every
 * tool is unclassifiable here, therefore none may run ungoverned. Governed tool
 * execution arrives with the Tool Registry and the process supervisor
 * (P2-T05/P2-T06) and runs in the daemon, never in Pi.
 *
 * `READ_ONLY_TOOL_ALLOWLIST` is deliberately empty. It is the single, explicit
 * place where a future batch may admit a *Pi-native* tool to run ungoverned,
 * and it exists so that admitting one is a reviewed edit rather than an
 * accidental gap. Daemon-governed WorkspaceSearch/Write/Patch are advertised
 * separately via `registerTool`; they are not entries on this allowlist.
 */

import type { ToolCallDecision, ToolCallEvent } from "./pi-api.js";
import { isDaemonGovernedWorkspaceTool } from "./workspace-tools.js";

/** Pi built-ins that mutate the workspace or execute commands. */
export const BLOCKED_MUTATING_TOOLS: readonly string[] = ["bash", "edit", "write"];

/**
 * Tools this Extension would let Pi run directly. Intentionally empty: a
 * non-authority client authorizes nothing. Do not add entries without a
 * reviewed decision recorded in the task ledger.
 */
export const READ_ONLY_TOOL_ALLOWLIST: readonly string[] = [];

export const MUTATING_TOOL_BLOCK_REASON =
  "CognitiveOS governed mode blocks Pi's direct mutating tools; external mutating work must go through a governed Intent/Effect in the daemon";

export const UNGOVERNED_TOOL_BLOCK_REASON =
  "CognitiveOS governed mode blocks ungoverned Pi tool execution; the Extension is a non-authority client and cannot authorize a tool";

/** True when the tool is one of Pi's known mutating built-ins. */
export function isBlockedMutatingTool(toolName: string): boolean {
  return BLOCKED_MUTATING_TOOLS.includes(normalizeToolName(toolName));
}

/**
 * Decide one `tool_call`. Daemon-governed Workspace* tools return `undefined`
 * so Pi runs the Extension's I/O-free `execute`. Every other tool is refused
 * while the Pi-native allowlist stays empty.
 */
export function decideToolCall(event: ToolCallEvent): ToolCallDecision {
  const toolName = normalizeToolName(event.toolName);
  if (isDaemonGovernedWorkspaceTool(toolName)) {
    return undefined;
  }
  if (READ_ONLY_TOOL_ALLOWLIST.includes(toolName)) {
    return undefined;
  }
  return {
    block: true,
    reason: isBlockedMutatingTool(toolName)
      ? MUTATING_TOOL_BLOCK_REASON
      : UNGOVERNED_TOOL_BLOCK_REASON,
  };
}

/**
 * Fold case and trim so a renamed-by-case or padded tool name cannot slip past
 * the denylist. An unnamed tool folds to the empty string, which is not on the
 * allowlist and is therefore blocked.
 */
function normalizeToolName(toolName: string): string {
  return toolName.trim().toLowerCase();
}
