/**
 * Structural mirror of the pinned subset of the Pi Extension API
 * (`@earendil-works/pi-coding-agent` 0.81.1).
 *
 * This package deliberately declares the shapes it uses instead of importing
 * them. ADR-0025 forbids vendoring or redistributing Pi, and the user installs
 * a compliant Pi locally; adding Pi to `pnpm-lock.yaml` would pull it into every
 * workspace install and CI job. The pinned version, integrity and source commit
 * live in `./pin.js` and are drift-checked against the Rust
 * `PiCompatibilityPin` by `pin.test.ts`.
 *
 * Only the four surfaces the CognitiveOS Extension actually uses are declared:
 * `on("project_trust")`, `on("tool_call")`, `on("session_start")` and
 * `registerCommand`. If Pi changes any of these shapes, the compatibility pin
 * must be re-reviewed before the version is moved — a wider mirror would only
 * create a second, unverified source of truth for Pi's API.
 */

/** Result of Pi's project-trust decision hook. */
export interface ProjectTrustDecision {
  readonly trusted: "yes" | "no";
}

/** The subset of Pi's `tool_call` event this Extension reads. */
export interface ToolCallEvent {
  readonly toolName: string;
}

/**
 * Returned from the `tool_call` hook. `undefined` lets Pi run the tool;
 * a block record refuses it with an operator-visible reason.
 */
export interface ToolCallBlock {
  readonly block: true;
  readonly reason: string;
}

export type ToolCallDecision = ToolCallBlock | undefined;

/** Presentation-only surface Pi hands to hooks and command handlers. */
export interface ExtensionUi {
  setStatus(statusKey: string, statusText: string): void;
  notify(message: string, level: "info" | "warn" | "error"): void;
}

/** Context Pi passes to hooks and command handlers. */
export interface ExtensionContext {
  readonly ui: ExtensionUi;
}

/** Command registration record. */
export interface ExtensionCommandSpec {
  readonly description: string;
  handler(commandArguments: unknown, context: ExtensionContext): Promise<void>;
}

/** The pinned Pi Extension registration surface. */
export interface ExtensionAPI {
  on(event: "project_trust", handler: () => Promise<ProjectTrustDecision>): void;
  on(event: "tool_call", handler: (event: ToolCallEvent) => Promise<ToolCallDecision>): void;
  on(
    event: "session_start",
    handler: (event: unknown, context: ExtensionContext) => Promise<void>,
  ): void;
  registerCommand(commandName: string, spec: ExtensionCommandSpec): void;
}
