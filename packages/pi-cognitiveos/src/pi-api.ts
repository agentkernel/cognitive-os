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
 * Only the five surfaces the CognitiveOS Extension actually uses are declared:
 * `on("project_trust")`, `on("tool_call")`, `on("session_start")` and
 * `registerCommand`, plus `setModel` for the daemon-selected model. If Pi
 * changes any of these shapes, the compatibility pin
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

/**
 * The exact model shape consumed by the pinned complete-provider surface.
 * It intentionally carries no Provider endpoint, headers, or credentials.
 */
export interface PiModel {
  readonly id: string;
  readonly name: string;
  readonly provider: string;
  readonly api: string;
  readonly reasoning: boolean;
  readonly input: readonly ["text"];
  readonly cost: {
    readonly input: number;
    readonly output: number;
    readonly cacheRead: number;
    readonly cacheWrite: number;
  };
  readonly contextWindow: number;
  readonly maxTokens: number;
}

/** Subset of the pinned Context consumed by the bounded text bridge. */
export interface PiCompletionContext {
  readonly systemPrompt?: string;
  readonly messages: readonly unknown[];
}

/** The only Pi stream option honored by the daemon bridge is cancellation. */
export interface PiStreamOptions {
  readonly signal?: AbortSignal;
}

/** A keyless availability check; it exposes no Pi credential storage surface. */
export interface PiProviderAuth {
  readonly apiKey: {
    readonly name: string;
    check(): Promise<{ readonly type: "api_key"; readonly source: string }>;
    resolve(): Promise<{ readonly auth: Record<string, never>; readonly source: string }>;
  };
}

export interface PiAssistantMessage {
  readonly role: "assistant";
  readonly content: readonly PiTextContent[];
  readonly api: string;
  readonly provider: string;
  readonly model: string;
  readonly usage: {
    readonly input: number;
    readonly output: number;
    readonly cacheRead: number;
    readonly cacheWrite: number;
    readonly totalTokens: number;
    readonly cost: {
      readonly input: number;
      readonly output: number;
      readonly cacheRead: number;
      readonly cacheWrite: number;
      readonly total: number;
    };
  };
  readonly stopReason: "stop" | "error" | "aborted";
  readonly timestamp: number;
  readonly errorMessage?: string;
}

export interface PiTextContent {
  readonly type: "text";
  readonly text: string;
}

export type PiAssistantMessageEvent =
  | { readonly type: "start"; readonly partial: PiAssistantMessage }
  | { readonly type: "text_start"; readonly contentIndex: number; readonly partial: PiTextContent }
  | { readonly type: "text_delta"; readonly contentIndex: number; readonly delta: string }
  | { readonly type: "text_end"; readonly contentIndex: number; readonly content: PiTextContent }
  | { readonly type: "done"; readonly message: PiAssistantMessage }
  | { readonly type: "error"; readonly error: PiAssistantMessage };

export interface AssistantMessageEventStream extends AsyncIterable<PiAssistantMessageEvent> {
  push(event: PiAssistantMessageEvent): void;
  end(result?: PiAssistantMessage): void;
  result(): Promise<PiAssistantMessage>;
}

/** The complete custom-provider surface pinned from Pi 0.81.1. */
export interface Provider {
  readonly id: string;
  readonly name: string;
  readonly auth: PiProviderAuth;
  getModels(): readonly PiModel[];
  stream(model: PiModel, context: PiCompletionContext, options?: PiStreamOptions): AssistantMessageEventStream;
  streamSimple(model: PiModel, context: PiCompletionContext, options?: PiStreamOptions): AssistantMessageEventStream;
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
  registerProvider(provider: Provider): void;
  setModel(model: PiModel): Promise<boolean>;
}
