/**
 * Daemon-governed Workspace* tools advertised to Pi (P2-T29).
 *
 * These tools exist so the Agent can name Search/Write/Patch without using Pi's
 * native filesystem tools. `execute` must not touch the filesystem, spawn a
 * process, or HTTP the daemon: the private candidate path holds the authority
 * store while `propose()` waits, and a nested remember/search call would
 * deadlock. The adapter extracts arguments from Pi's event stream and maps
 * them onto the existing P2-T21 candidate/Intent/Effect path.
 */

import type { AgentToolResult, ExtensionToolDefinition } from "./pi-api.js";

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
  "queued for daemon-governed Intent/Effect; this Extension does not mutate the workspace or contact the daemon during private candidate generation";

function queuedResult(): AgentToolResult {
  return { content: [{ type: "text", text: DAEMON_WORKSPACE_QUEUED_RESULT }] };
}

function objectSchema(
  properties: Record<string, { readonly type: "string"; readonly description: string }>,
  required: readonly string[],
): Record<string, unknown> {
  return {
    type: "object",
    additionalProperties: false,
    properties,
    required: [...required],
  };
}

/** True when the name is one of the three daemon-governed Workspace* tools. */
export function isDaemonGovernedWorkspaceTool(toolName: string): boolean {
  const normalized = toolName.trim().toLowerCase();
  return DAEMON_GOVERNED_WORKSPACE_TOOL_NAMES.some(
    (name) => name.toLowerCase() === normalized,
  );
}

/**
 * Registerable tool records. `execute` is a no-op observation: it returns a
 * bounded queued text and never performs I/O.
 */
export function daemonGovernedWorkspaceTools(): readonly ExtensionToolDefinition[] {
  return [
    {
      name: DAEMON_WORKSPACE_READ,
      label: "Workspace read (daemon-governed)",
      description:
        "Propose a bounded workspace read. CognitiveOS admits this as an untrusted candidate and executes it only through a daemon Intent/Effect. This tool does not read files.",
      parameters: objectSchema(
        {
          target: { type: "string", description: "Workspace URI target" },
        },
        ["target"],
      ),
      async execute(): Promise<AgentToolResult> {
        return queuedResult();
      },
    },
    {
      name: DAEMON_WORKSPACE_SEARCH,
      label: "Workspace search (daemon-governed)",
      description:
        "Propose a bounded workspace search. CognitiveOS admits this as an untrusted candidate and executes it only through a daemon Intent/Effect. This tool does not read files.",
      parameters: objectSchema(
        {
          query: { type: "string", description: "Bounded search query" },
          target: { type: "string", description: "Workspace URI target" },
        },
        ["query", "target"],
      ),
      async execute(): Promise<AgentToolResult> {
        return queuedResult();
      },
    },
    {
      name: DAEMON_WORKSPACE_WRITE,
      label: "Workspace write (daemon-governed)",
      description:
        "Propose a bounded workspace write. CognitiveOS admits this as an untrusted candidate and executes it only through a daemon Intent/Effect. This tool does not write files.",
      parameters: objectSchema(
        {
          input_b64: {
            type: "string",
            description: "Canonical base64 payload for the write",
          },
          preimage: {
            type: "string",
            description: 'Expected preimage: "absent" or digest:sha256:<64 lowercase hex>',
          },
          target: { type: "string", description: "Workspace URI target" },
        },
        ["input_b64", "preimage", "target"],
      ),
      async execute(): Promise<AgentToolResult> {
        return queuedResult();
      },
    },
    {
      name: DAEMON_WORKSPACE_PATCH,
      label: "Workspace patch (daemon-governed)",
      description:
        "Propose a bounded workspace patch. CognitiveOS admits this as an untrusted candidate and executes it only through a daemon Intent/Effect. This tool does not patch files.",
      parameters: objectSchema(
        {
          input_b64: {
            type: "string",
            description: "Canonical base64 unified-diff payload",
          },
          preimage: {
            type: "string",
            description: 'Expected preimage: "absent" or digest:sha256:<64 lowercase hex>',
          },
          target: { type: "string", description: "Workspace URI target" },
        },
        ["input_b64", "preimage", "target"],
      ),
      async execute(): Promise<AgentToolResult> {
        return queuedResult();
      },
    },
  ];
}
