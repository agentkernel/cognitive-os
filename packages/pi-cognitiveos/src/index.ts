/**
 * `@cognitiveos/pi-cognitiveos` — the CognitiveOS Pi Extension (Personal P1-T07).
 *
 * Pi is a terminal UI and a non-authority client. Nothing exported here writes
 * authority state, mints a capability, creates an Effect or advances a Task.
 */

export {
  registerCognitiveOsExtension,
  PROJECT_TRUST_DECISION,
  type CognitiveOsExtensionOptions,
} from "./extension.js";
export { default } from "./extension.js";

export {
  PersonalDaemonClient,
  parseReadinessProjection,
  DEFAULT_REQUEST_TIMEOUT_MS,
  EXTENSION_CHANNEL,
  LOCAL_OWNER_PRINCIPAL,
  type FetchLike,
  type OverallReadiness,
  type PersonalDaemonClientOptions,
  type ReadinessComponent,
  type ReadinessProjection,
} from "./daemon-client.js";

export {
  BOOTSTRAP_SECRET_FILE_NAME,
  ENDPOINT_FILE_NAME,
  ENDPOINT_SCHEMA_VERSION,
  ENDPOINT_SURFACE,
  PERSONAL_PRODUCT_DIR_NAME,
  isLoopbackEndpoint,
  nodeFileReader,
  readBootstrapSecret,
  readDaemonEndpoint,
  resolvePersonalDaemonPaths,
  type EnvironmentSlice,
  type FileReader,
  type PersonalDaemonPaths,
} from "./daemon-discovery.js";

export {
  DaemonClientError,
  isDaemonUnavailable,
  type DaemonClientErrorCode,
} from "./errors.js";

export {
  COGNITIVEOS_STATUS_COMMAND,
  COGNITIVEOS_STATUS_COMMAND_NAME,
  COGNITIVEOS_STATUS_KEY,
  PI_COMPATIBILITY_PIN,
  type PiCompatibilityPin,
} from "./pin.js";

export {
  statusDetailFromFailure,
  statusDetailFromProjection,
  statusLineFromFailure,
  statusLineFromProjection,
} from "./status.js";

export {
  BLOCKED_MUTATING_TOOLS,
  MUTATING_TOOL_BLOCK_REASON,
  READ_ONLY_TOOL_ALLOWLIST,
  UNGOVERNED_TOOL_BLOCK_REASON,
  decideToolCall,
  isBlockedMutatingTool,
} from "./tool-policy.js";

export type {
  ExtensionAPI,
  ExtensionCommandSpec,
  ExtensionContext,
  ExtensionUi,
  ProjectTrustDecision,
  ToolCallBlock,
  ToolCallDecision,
  ToolCallEvent,
} from "./pi-api.js";
