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
  DAEMON_WORKSPACE_READ,
  DAEMON_WORKSPACE_SEARCH,
  daemonGovernedWorkspaceTools,
} from "./workspace-tools.js";

const PUBLIC_DAEMON_GOVERNED_TOOL_NAMES = [
  DAEMON_WORKSPACE_READ,
  DAEMON_WORKSPACE_SEARCH,
] as const;

/** Deny project trust unconditionally; governed mode authorizes nothing via Pi. */
export const PROJECT_TRUST_DECISION = { trusted: "no" } as const;

export interface CognitiveOsExtensionOptions {
  /** Injected in tests; production constructs a default client. */
  readonly client?: PersonalDaemonClient;
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
  let daemonSelectedModel: PiModel | undefined;

  pi.on("session_start", async (_event, context) => {
    // Pi exposes setActiveTools after it binds the Extension runtime. This
    // explicit allowlist keeps all native and mutating tools inactive.
    pi.setActiveTools(PUBLIC_DAEMON_GOVERNED_TOOL_NAMES);
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

  for (const tool of daemonGovernedWorkspaceTools()) {
    pi.registerTool(tool);
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
