/**
 * The CognitiveOS Pi Extension (Personal P1-T07).
 *
 * Pi is reused as a terminal UI. It is **not** an authority: it does not write
 * SQLite, mint capabilities, create Effects or advance Task state (PERS-PR-005,
 * ADR-0022 §9). This Extension enforces that boundary from inside Pi:
 *
 *   - `project_trust` is always denied. Pi's own trust prompt would grant Pi
 *     ambient project permission; in governed mode CognitiveOS is the only thing
 *     that authorizes anything.
 *   - Pi-native `tool_call`s are refused (see `tool-policy.ts`), including
 *     bash/edit/write. The Extension advertises daemon-governed
 *     WorkspaceSearch/Write/Patch whose `execute` is I/O-free; the daemon
 *     admits those arguments as candidates on the Intent/Effect path.
 *   - `session_start` reads the daemon's readiness projection and shows it. When
 *     the daemon cannot be read, the session says so loudly and never implies
 *     readiness.
 *   - `/cognitive-status` prints the same daemon facts on demand.
 *
 * The Extension holds no Provider credential. It never reads `provider.json`,
 * resolves a `SecretRef`, or reads a Provider API key from the environment.
 * The registered daemon Provider routes model traffic through the daemon-owned
 * proxy, so Pi receives model responses without becoming secret authority.
 */

import { PersonalDaemonClient } from "./daemon-client.js";
import { createDaemonProvider } from "./daemon-provider.js";
import type { ReadinessProjection } from "./daemon-client.js";
import {
  COGNITIVEOS_STATUS_COMMAND_NAME,
  COGNITIVEOS_STATUS_KEY,
} from "./pin.js";
import type { ExtensionAPI, ExtensionContext, PiModel } from "./pi-api.js";
import {
  statusDetailFromFailure,
  statusDetailFromProjection,
  statusLineFromFailure,
  statusLineFromProjection,
} from "./status.js";
import { decideToolCall } from "./tool-policy.js";
import {
  DAEMON_WORKSPACE_PATCH,
  DAEMON_WORKSPACE_READ,
  DAEMON_WORKSPACE_SEARCH,
  DAEMON_WORKSPACE_WRITE,
  daemonGovernedWorkspaceTools,
  type PublicCandidateSubmitter,
} from "./workspace-tools.js";

const PUBLIC_DAEMON_GOVERNED_TOOL_NAMES = [
  DAEMON_WORKSPACE_READ,
  DAEMON_WORKSPACE_SEARCH,
  DAEMON_WORKSPACE_WRITE,
  DAEMON_WORKSPACE_PATCH,
] as const;

/** Deny project trust unconditionally; governed mode authorizes nothing via Pi. */
export const PROJECT_TRUST_DECISION = { trusted: "no" } as const;

export interface CognitiveOsExtensionOptions {
  /** Injected in tests; production constructs a default client. */
  readonly client?: PersonalDaemonClient;
  /** Public Task binding supplied only by the task-bound launch surface. */
  readonly taskRef?: string;
}

/**
 * Register the CognitiveOS surface with Pi.
 *
 * Pi loads this module and calls the default export with its `ExtensionAPI`.
 */
export async function registerCognitiveOsExtension(
  pi: ExtensionAPI,
  options: CognitiveOsExtensionOptions = {},
): Promise<void> {
  pi.on("project_trust", async () => PROJECT_TRUST_DECISION);

  pi.on("tool_call", async (event) => decideToolCall(event));

  const client = options.client ?? new PersonalDaemonClient();
  const taskRef = options.taskRef ?? client.readPublicTaskRef();
  const candidateSubmitter = taskRef === undefined
    ? undefined
    : createPublicCandidateSubmitter(client, taskRef);
  pi.on("before_agent_start", async () => {
    activateDaemonGovernedWorkspaceTools(pi, candidateSubmitter);
  });
  let daemonSelectedModel: PiModel | undefined;

  pi.on("session_start", async (_event, context) => {
    activateDaemonGovernedWorkspaceTools(pi, candidateSubmitter);
    if (daemonSelectedModel === undefined) {
      await showStatus(client, context, "session_start");
      return;
    }
    await activateDaemonSelectedModel(pi, daemonSelectedModel);
    await showStatus(client, context, "session_start");
  });

  pi.registerCommand(COGNITIVEOS_STATUS_COMMAND_NAME, {
    description: "Show CognitiveOS Personal daemon readiness (read-only; no authority)",
    handler: async (_commandArguments, context) => {
      await showStatus(client, context, "command");
    },
  });

  // Initial extension loading queues providers until Pi binds its session
  // context. The session_start hook activates this model after that binding.
  // Absent an explicit campaign authorization this is `undefined`, and the
  // route is registered exactly as it was before it could be measured.
  const session = client.openCampaignObservationSession();
  const daemonProvider = await createDaemonProvider(
    client,
    session === undefined ? {} : { session },
  );
  daemonSelectedModel = daemonProvider.models[0];
  if (daemonSelectedModel === undefined) {
    throw new Error("the daemon provider registered no selectable model");
  }
  pi.registerProvider(daemonSelectedModel.provider, daemonProvider);

  registerDaemonGovernedWorkspaceTools(pi, candidateSubmitter);
}

function registerDaemonGovernedWorkspaceTools(
  pi: ExtensionAPI,
  candidateSubmitter?: PublicCandidateSubmitter,
): void {
  for (const tool of daemonGovernedWorkspaceTools(candidateSubmitter)) {
    pi.registerTool(tool);
  }
}

export function createPublicCandidateSubmitter(
  client: PersonalDaemonClient,
  taskRef: string,
): PublicCandidateSubmitter {
  return {
    async submit(toolName, parameters) {
      const target = parameters["target"];
      if (typeof target !== "string" || target.length === 0) {
        throw new Error("daemon Workspace candidate target is invalid");
      }
      const query = parameters["query"];
      const isSearch = toolName === DAEMON_WORKSPACE_SEARCH;
      const isWrite = toolName === DAEMON_WORKSPACE_WRITE;
      const isPatch = toolName === DAEMON_WORKSPACE_PATCH;
      if (!isSearch && !isWrite && !isPatch && toolName !== DAEMON_WORKSPACE_READ) {
        throw new Error("daemon Workspace candidate tool is unsupported");
      }
      if (isSearch && (typeof query !== "string" || query.length === 0)) {
        throw new Error("daemon WorkspaceSearch candidate query is invalid");
      }
      const input = parameters["input_b64"];
      const preimage = parameters["preimage"];
      if ((isWrite || isPatch) &&
        (typeof input !== "string" || input.length === 0 ||
          typeof preimage !== "string" || preimage.length === 0 ||
          !isCanonicalBase64(input) || !isExpectedPreimage(preimage))) {
        throw new Error("daemon Workspace mutation candidate parameters are invalid");
      }
      const candidateParameters = isSearch
        ? { family: "WorkspaceSearch", query }
        : isWrite
          ? { family: "WorkspaceWrite", input_b64: input, preimage }
          : isPatch
            ? { family: "WorkspacePatch", input_b64: input, preimage }
            : undefined;
      await client.submitPublicCandidate({
        taskRef,
        toolRef: isSearch
          ? "native.workspace.search"
          : isWrite
            ? "native.workspace.write"
            : isPatch
              ? "native.workspace.patch"
              : "native.workspace.read",
        action: isSearch ? "search" : isWrite ? "write" : isPatch ? "patch" : "read",
        target,
        ...(candidateParameters === undefined ? {} : { parameters: candidateParameters }),
        parametersDigest: "sha256:" + "0".repeat(64),
        expectedStateVersion: 1,
        operationDescriptorId: isSearch
          ? "00000000-0000-7000-8000-000000002002"
          : isWrite
            ? "00000000-0000-7000-8000-000000002003"
            : isPatch
              ? "00000000-0000-7000-8000-000000002004"
              : "00000000-0000-7000-8000-000000002001",
      });
    },
  };
}

function isCanonicalBase64(value: string): boolean {
  if (value.length === 0 || value.length % 4 !== 0 || !/^[A-Za-z0-9+/]*={0,2}$/.test(value)) {
    return false;
  }
  try {
    return globalThis.btoa(globalThis.atob(value)) === value;
  } catch {
    return false;
  }
}

function isExpectedPreimage(value: string): boolean {
  return value === "absent" || /^digest:sha256:[0-9a-f]{64}$/.test(value);
}

function activateDaemonGovernedWorkspaceTools(
  pi: ExtensionAPI,
  candidateSubmitter?: PublicCandidateSubmitter,
): void {
  // Pi's initial extension load records tools before its runtime registry is
  // bound. Re-registering same-name tools at the pre-agent hook refreshes the
  // registry without expanding the Extension surface.
  registerDaemonGovernedWorkspaceTools(pi, candidateSubmitter);
  // This explicit allowlist keeps all native and mutating tools inactive.
  pi.setActiveTools(PUBLIC_DAEMON_GOVERNED_TOOL_NAMES);
  assertDaemonGovernedToolsAreActive(pi);
}

function assertDaemonGovernedToolsAreActive(pi: ExtensionAPI): void {
  const registeredToolNames = new Set(pi.getAllTools().map((tool) => tool.name));
  const missingRegisteredToolNames = PUBLIC_DAEMON_GOVERNED_TOOL_NAMES.filter(
    (toolName) => !registeredToolNames.has(toolName),
  );
  if (missingRegisteredToolNames.length > 0) {
    throw new Error(
      `CognitiveOS daemon-governed tools are absent from Pi's registry: ${missingRegisteredToolNames.join(", ")}`,
    );
  }

  const activeToolNames = new Set(pi.getActiveTools());
  const missingToolNames = PUBLIC_DAEMON_GOVERNED_TOOL_NAMES.filter(
    (toolName) => !activeToolNames.has(toolName),
  );
  if (missingToolNames.length > 0) {
    throw new Error(
      `CognitiveOS daemon-governed tools were not activated: ${missingToolNames.join(", ")}`,
    );
  }
}

export default registerCognitiveOsExtension;

/**
 * Read the daemon projection and render it. A failure is reported through both
 * the status bar and a notification: P1-T07 acceptance requires that an
 * unavailable daemon fails explicitly rather than leaving a session that looks
 * usable.
 */
async function showStatus(
  client: PersonalDaemonClient,
  context: ExtensionContext,
  origin: "session_start" | "command",
): Promise<void> {
  let projection: ReadinessProjection;
  try {
    projection = await client.fetchReadiness();
  } catch (error) {
    context.ui.setStatus(COGNITIVEOS_STATUS_KEY, statusLineFromFailure(error));
    context.ui.notify(statusDetailFromFailure(error), "error");
    return;
  }

  context.ui.setStatus(COGNITIVEOS_STATUS_KEY, statusLineFromProjection(projection));
  if (origin === "command") {
    context.ui.notify(statusDetailFromProjection(projection), "info");
    return;
  }
  if (!projection.firstConversationReady) {
    // Surfacing this at session start is the difference between "the operator
    // knows the first conversation is blocked" and a silently degraded session.
    context.ui.notify(statusDetailFromProjection(projection), "warn");
  }
}

async function activateDaemonSelectedModel(
  pi: ExtensionAPI,
  daemonSelectedModel: PiModel,
): Promise<void> {
  if (!(await pi.setModel(daemonSelectedModel))) {
    throw new Error("the daemon-selected CognitiveOS model could not be activated");
  }
}
