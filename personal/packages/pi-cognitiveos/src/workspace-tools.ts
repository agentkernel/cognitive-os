/**
 * Daemon-governed Workspace* tools advertised to Pi (P2-T29).
 *
 * These tools exist so the Agent can name Search/Write/Patch without using Pi's
 * native filesystem tools. `execute` never touches the filesystem or spawns a
 * process. In task-bound public mode it submits only untrusted operation fields
 * to the daemon Task channel; the daemon remains the sole authority writer.
 * Without a task binding, the handler retains the observation-only behavior
 * used by ordinary Pi sessions and protocol tests.
 */

import type { AgentToolResult, ExtensionToolDefinition } from "./pi-api.js";
import { Type } from "typebox";

export interface PublicCandidateSubmitter {
  submit(toolName: string, parameters: Readonly<Record<string, unknown>>): Promise<void>;
}

export const DAEMON_WORKSPACE_SEARCH = "WorkspaceSearch";
export const DAEMON_WORKSPACE_READ = "WorkspaceRead";
export const DAEMON_WORKSPACE_WRITE = "WorkspaceWrite";
export const DAEMON_WORKSPACE_PATCH = "WorkspacePatch";

export const DAEMON_GOVERNED_WORKSPACE_TOOL_NAMES: readonly string[] = [
  DAEMON_WORKSPACE_READ,
  DAEMON_WORKSPACE_SEARCH,
  DAEMON_WORKSPACE_WRITE,
  DAEMON_WORKSPACE_PATCH,
];

export const DAEMON_WORKSPACE_QUEUED_RESULT =
  "queued for daemon-governed Intent/Effect; this Extension does not mutate the workspace";

function queuedResult(): AgentToolResult {
  return { content: [{ type: "text", text: DAEMON_WORKSPACE_QUEUED_RESULT }] };
}

/** True when the name is one of the daemon-governed Workspace* tools. */
export function isDaemonGovernedWorkspaceTool(toolName: string): boolean {
  const normalized = toolName.trim().toLowerCase();
  return DAEMON_GOVERNED_WORKSPACE_TOOL_NAMES.some(
    (name) => name.toLowerCase() === normalized,
  );
}

/**
 * Registerable tool records. Public task-bound execution submits a candidate
 * through the daemon and returns only a bounded queued result.
 */
export function daemonGovernedWorkspaceTools(
  candidateSubmitter?: PublicCandidateSubmitter,
): readonly ExtensionToolDefinition[] {
  const executeCandidate = async (
    toolName: string,
    parameters: Readonly<Record<string, unknown>>,
  ): Promise<AgentToolResult> => {
    if (candidateSubmitter !== undefined) {
      await candidateSubmitter.submit(toolName, parameters);
    }
    return queuedResult();
  };
  return [
    {
      name: DAEMON_WORKSPACE_READ,
      label: "Workspace read (daemon-governed)",
      description:
        "Propose a bounded workspace read. CognitiveOS admits this as an untrusted candidate and executes it only through a daemon Intent/Effect. This tool does not read files.",
      parameters: Type.Object(
        { target: Type.String({ description: "Workspace URI target" }) },
        { additionalProperties: false },
      ),
      async execute(_toolCallId, params): Promise<AgentToolResult> {
        return executeCandidate(DAEMON_WORKSPACE_READ, params);
      },
    },
    {
      name: DAEMON_WORKSPACE_SEARCH,
      label: "Workspace search (daemon-governed)",
      description:
        "Propose a bounded workspace search. CognitiveOS admits this as an untrusted candidate and executes it only through a daemon Intent/Effect. This tool does not read files.",
      parameters: Type.Object(
        {
          query: Type.String({ description: "Bounded search query" }),
          target: Type.String({ description: "Workspace URI target" }),
        },
        { additionalProperties: false },
      ),
      async execute(_toolCallId, params): Promise<AgentToolResult> {
        return executeCandidate(DAEMON_WORKSPACE_SEARCH, params);
      },
    },
    {
      name: DAEMON_WORKSPACE_WRITE,
      label: "Workspace write (daemon-governed)",
      description:
        "Propose a bounded workspace write. CognitiveOS admits this as an untrusted candidate and executes it only through a daemon Intent/Effect. This tool does not write files.",
      parameters: Type.Object(
        {
          input_b64: Type.String({ description: "Canonical base64 payload for the write" }),
          preimage: Type.String({
            description: 'Expected preimage: "absent" or digest:sha256:<64 lowercase hex>',
          }),
          target: Type.String({ description: "Workspace URI target" }),
        },
        { additionalProperties: false },
      ),
      async execute(_toolCallId, params): Promise<AgentToolResult> {
        return executeCandidate(DAEMON_WORKSPACE_WRITE, params);
      },
    },
    {
      name: DAEMON_WORKSPACE_PATCH,
      label: "Workspace patch (daemon-governed)",
      description:
        "Propose a bounded workspace patch. CognitiveOS admits this as an untrusted candidate and executes it only through a daemon Intent/Effect. This tool does not patch files.",
      parameters: Type.Object(
        {
          input_b64: Type.String({ description: "Canonical base64 unified-diff payload" }),
          preimage: Type.String({
            description: 'Expected preimage: "absent" or digest:sha256:<64 lowercase hex>',
          }),
          target: Type.String({ description: "Workspace URI target" }),
        },
        { additionalProperties: false },
      ),
      async execute(_toolCallId, params): Promise<AgentToolResult> {
        return executeCandidate(DAEMON_WORKSPACE_PATCH, params);
      },
    },
  ];
}
