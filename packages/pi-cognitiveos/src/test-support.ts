/**
 * Test doubles for the Pi Extension package.
 *
 * `FakePi` records everything the Extension registers and does, so tests can
 * assert on refusals and on what was *not* done. `startFakeDaemon` speaks the
 * subset of the real Personal front-door protocol this client uses, including
 * its refusal paths, so the client is exercised over a real loopback socket
 * rather than a stubbed `fetch`.
 *
 * This module is not a `*.test.ts` file, so it is never collected as a test.
 */

import { createServer, type IncomingMessage, type Server, type ServerResponse } from "node:http";
import { AddressInfo } from "node:net";

import type {
  ExtensionAPI,
  ExtensionCommandSpec,
  ExtensionContext,
  ExtensionUi,
  PiModel,
  ProviderConfig,
  ProjectTrustDecision,
  ToolCallDecision,
  ToolCallEvent,
} from "./pi-api.js";

/**
 * Run `action` and return what it threw. `assert.throws` returns `void`, so
 * tests that want to inspect the thrown value need this instead of a cast.
 */
export function captureThrown(action: () => unknown): unknown {
  try {
    action();
  } catch (error) {
    return error;
  }
  throw new Error("expected the action to throw, but it returned normally");
}

/** Awaitable counterpart of `captureThrown`. */
export async function captureRejection(action: () => Promise<unknown>): Promise<unknown> {
  try {
    await action();
  } catch (error) {
    return error;
  }
  throw new Error("expected the action to reject, but it resolved");
}

export interface RecordedStatus {
  readonly statusKey: string;
  readonly statusText: string;
}

export interface RecordedNotification {
  readonly message: string;
  readonly level: "info" | "warn" | "error";
}

/** Recording `ExtensionUi` implementation. */
export class FakeUi implements ExtensionUi {
  readonly statuses: RecordedStatus[] = [];
  readonly notifications: RecordedNotification[] = [];

  setStatus(statusKey: string, statusText: string): void {
    this.statuses.push({ statusKey, statusText });
  }

  notify(message: string, level: "info" | "warn" | "error"): void {
    this.notifications.push({ message, level });
  }
}

/** Recording `ExtensionAPI` implementation with hook drivers. */
export class FakePi implements ExtensionAPI {
  readonly ui = new FakeUi();
  readonly context: ExtensionContext = { ui: this.ui };
  readonly commands = new Map<string, ExtensionCommandSpec>();
  readonly providers: Array<{ readonly providerName: string; readonly config: ProviderConfig }> = [];
  readonly selectedModels: PiModel[] = [];
  private projectTrustHandler: (() => Promise<ProjectTrustDecision>) | undefined;
  private toolCallHandler: ((event: ToolCallEvent) => Promise<ToolCallDecision>) | undefined;
  private sessionStartHandler:
    | ((event: unknown, context: ExtensionContext) => Promise<void>)
    | undefined;

  on(event: "project_trust", handler: () => Promise<ProjectTrustDecision>): void;
  on(event: "tool_call", handler: (event: ToolCallEvent) => Promise<ToolCallDecision>): void;
  on(
    event: "session_start",
    handler: (event: unknown, context: ExtensionContext) => Promise<void>,
  ): void;
  on(event: string, handler: unknown): void {
    if (event === "project_trust") {
      this.projectTrustHandler = handler as () => Promise<ProjectTrustDecision>;
      return;
    }
    if (event === "tool_call") {
      this.toolCallHandler = handler as (event: ToolCallEvent) => Promise<ToolCallDecision>;
      return;
    }
    if (event === "session_start") {
      this.sessionStartHandler = handler as (
        event: unknown,
        context: ExtensionContext,
      ) => Promise<void>;
      return;
    }
    throw new Error(`unexpected Pi hook registration: ${event}`);
  }

  registerCommand(commandName: string, spec: ExtensionCommandSpec): void {
    this.commands.set(commandName, spec);
  }

  registerProvider(providerName: string, config: ProviderConfig): void {
    this.providers.push({ providerName, config });
  }

  async setModel(model: PiModel): Promise<boolean> {
    this.selectedModels.push(model);
    return true;
  }

  get registeredHooks(): readonly string[] {
    const hooks: string[] = [];
    if (this.projectTrustHandler !== undefined) hooks.push("project_trust");
    if (this.toolCallHandler !== undefined) hooks.push("tool_call");
    if (this.sessionStartHandler !== undefined) hooks.push("session_start");
    return hooks;
  }

  async driveProjectTrust(): Promise<ProjectTrustDecision> {
    if (this.projectTrustHandler === undefined) {
      throw new Error("the Extension did not register a project_trust hook");
    }
    return this.projectTrustHandler();
  }

  async driveToolCall(toolName: string): Promise<ToolCallDecision> {
    if (this.toolCallHandler === undefined) {
      throw new Error("the Extension did not register a tool_call hook");
    }
    return this.toolCallHandler({ toolName });
  }

  async driveSessionStart(): Promise<void> {
    if (this.sessionStartHandler === undefined) {
      throw new Error("the Extension did not register a session_start hook");
    }
    await this.sessionStartHandler({}, this.context);
  }

  async driveCommand(commandName: string): Promise<void> {
    const spec = this.commands.get(commandName);
    if (spec === undefined) {
      throw new Error(`the Extension did not register the command ${commandName}`);
    }
    await spec.handler(undefined, this.context);
  }
}

export interface RecordedRequest {
  readonly method: string;
  readonly url: string;
  readonly headers: Readonly<Record<string, string>>;
  readonly body: string;
}

export interface FakeDaemonOptions {
  /** Bootstrap secret the fake daemon accepts. */
  readonly bootstrapSecret: string;
  /** Projection returned from `GET /personal/status`. */
  readonly statusBody: string;
  /** Private projection returned from `GET /resource/v1/projection`. */
  readonly resourceProjectionBody?: string;
  /** Snapshot-first resource stream returned from `GET /resource/v1/watch`. */
  readonly resourceWatchBody?: string;
  /** Snapshot-first Task stream returned from `GET /task/watch`. */
  readonly taskWatchBody?: string;
  /**
   * How many `GET /personal/status` requests are answered `401` before the
   * daemon starts accepting the bearer. Models a daemon restart.
   */
  readonly unauthorizedStatusResponses?: number;
  readonly selectedModelBody?: string;
  readonly completionBody?: string;
  readonly providerNetworkElapsedNanos?: string;
}

export interface FakeDaemon {
  readonly endpoint: string;
  readonly requests: readonly RecordedRequest[];
  readonly issuedTokens: readonly string[];
  close(): Promise<void>;
}

/**
 * Start a loopback server implementing the subset of the Personal front door
 * this client speaks, including the refusal paths it must handle.
 */
export async function startFakeDaemon(options: FakeDaemonOptions): Promise<FakeDaemon> {
  const requests: RecordedRequest[] = [];
  const issuedTokens: string[] = [];
  const sessionChannels = new Map<string, "management" | "task">();
  let remainingUnauthorized = options.unauthorizedStatusResponses ?? 0;
  let tokenCounter = 0;

  const server: Server = createServer((request: IncomingMessage, response: ServerResponse) => {
    const chunks: Buffer[] = [];
    request.on("data", (chunk: Buffer) => chunks.push(chunk));
    request.on("end", () => {
      const body = Buffer.concat(chunks).toString("utf8");
      const headers: Record<string, string> = {};
      for (const [name, value] of Object.entries(request.headers)) {
        headers[name.toLowerCase()] = Array.isArray(value) ? value.join(",") : (value ?? "");
      }
      requests.push({
        method: request.method ?? "",
        url: request.url ?? "",
        headers,
        body,
      });

      if (headers["cookie"] !== undefined) {
        respond(response, 403, errorBody("LOCAL_COOKIE_AUTH_FORBIDDEN"));
        return;
      }
      const host = (headers["host"] ?? "").split(":")[0] ?? "";
      if (host !== "127.0.0.1" && host !== "localhost" && host !== "::1") {
        respond(response, 400, errorBody("LOCAL_HOST_HEADER_REJECTED"));
        return;
      }

      if (request.method === "POST" && request.url === "/local/session") {
        let parsed: Record<string, unknown>;
        try {
          parsed = JSON.parse(body) as Record<string, unknown>;
        } catch {
          respond(response, 401, errorBody("LOCAL_AUTH_INVALID_REQUEST"));
          return;
        }
        if (parsed["bootstrap_secret"] !== options.bootstrapSecret) {
          respond(response, 401, errorBody("LOCAL_BOOTSTRAP_MISMATCH"));
          return;
        }
        const channel = parsed["channel"];
        if (channel !== "management" && channel !== "task") {
          respond(response, 401, errorBody("LOCAL_AUTH_INVALID_REQUEST"));
          return;
        }
        tokenCounter += 1;
        const token = `sess-fake-${tokenCounter}`;
        issuedTokens.push(token);
        sessionChannels.set(token, channel);
        respond(
          response,
          200,
          JSON.stringify({
            status: "ok",
            token,
            channel,
            session_id: `session-${tokenCounter}`,
            absolute_expiry_secs: 43_200,
            idle_expiry_secs: 1_800,
          }),
        );
        return;
      }

      if (request.method === "GET" && request.url === "/personal/status") {
        const authorization = headers["authorization"] ?? "";
        if (!authorization.startsWith("Bearer ")) {
          respond(response, 401, errorBody("LOCAL_SESSION_UNAUTHORIZED"));
          return;
        }
        if (remainingUnauthorized > 0) {
          remainingUnauthorized -= 1;
          respond(response, 401, errorBody("LOCAL_SESSION_UNAUTHORIZED"));
          return;
        }
        const presented = authorization.slice("Bearer ".length);
        if (!issuedTokens.includes(presented)) {
          respond(response, 401, errorBody("LOCAL_SESSION_UNAUTHORIZED"));
          return;
        }
        respond(response, 200, options.statusBody);
        return;
      }

      if (request.method === "GET" && request.url === "/resource/v1/projection?family=runtime&version=1") {
        if (!hasChannelBearer(headers, sessionChannels, "management")) {
          respond(response, 401, errorBody("LOCAL_SESSION_UNAUTHORIZED"));
          return;
        }
        respond(response, 200, options.resourceProjectionBody ?? resourceProjectionBody());
        return;
      }

      if (request.method === "GET" && request.url?.startsWith("/resource/v1/watch?family=runtime&version=1")) {
        if (!hasChannelBearer(headers, sessionChannels, "management")) {
          respond(response, 401, errorBody("LOCAL_SESSION_UNAUTHORIZED"));
          return;
        }
        respond(response, 200, options.resourceWatchBody ?? resourceWatchSnapshotBody());
        return;
      }

      if (request.method === "GET" && request.url?.startsWith("/task/watch")) {
        if (!hasChannelBearer(headers, sessionChannels, "task")) {
          respond(response, 401, errorBody("LOCAL_SESSION_UNAUTHORIZED"));
          return;
        }
        respond(response, 200, options.taskWatchBody ?? taskWatchSnapshotBody());
        return;
      }

      if (request.method === "GET" && request.url === "/provider/v1/selected-model") {
        const authorization = headers["authorization"] ?? "";
        if (!authorization.startsWith("Bearer ") || !issuedTokens.includes(authorization.slice("Bearer ".length))) {
          respond(response, 401, errorBody("LOCAL_SESSION_UNAUTHORIZED"));
          return;
        }
        respond(response, 200, options.selectedModelBody ?? selectedModelProjectionBody());
        return;
      }

      if (request.method === "POST" && request.url === "/provider/v1/chat/completions") {
        const authorization = headers["authorization"] ?? "";
        if (!authorization.startsWith("Bearer ") || !issuedTokens.includes(authorization.slice("Bearer ".length))) {
          respond(response, 401, errorBody("LOCAL_SESSION_UNAUTHORIZED"));
          return;
        }
        respond(
          response,
          200,
          options.completionBody ?? boundedCompletionBody(),
          options.providerNetworkElapsedNanos === undefined
            ? undefined
            : { "x-cognitiveos-provider-network-nanos": options.providerNetworkElapsedNanos },
        );
        return;
      }

      respond(response, 404, errorBody("PERSONAL_ROUTE_NOT_FOUND"));
    });
  });

  await new Promise<void>((resolve) => {
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address() as AddressInfo;

  return {
    endpoint: `127.0.0.1:${address.port}`,
    requests,
    issuedTokens,
    close(): Promise<void> {
      return new Promise<void>((resolve, reject) => {
        server.close((error) => (error ? reject(error) : resolve()));
      });
    },
  };
}

export function selectedModelProjectionBody(
  overrides: Readonly<Record<string, unknown>> = {},
): string {
  return JSON.stringify({
    schema_version: 1,
    surface: "personal-provider-selected-model",
    selected_model: "deepseek-v4-flash",
    selected_snapshot_digest: "fnv1a64:synthetic",
    chat_capable: true,
    authority_side_effects: false,
    ...overrides,
  });
}

export function boundedCompletionBody(content = "daemon text"): string {
  return JSON.stringify({ choices: [{ message: { content }, finish_reason: "stop" }] });
}

export function resourceProjectionBody(
  overrides: Readonly<Record<string, unknown>> = {},
): string {
  return JSON.stringify({
    kind: "snapshot",
    projection_version: "personal-resource-projection/1",
    family: "runtime",
    latest_sequence: 6,
    projection: {
      family: "runtime",
      availability: "not-backed",
      authority_side_effects: false,
    },
    ...overrides,
  });
}

export function taskWatchSnapshotBody(): string {
  return "event: snapshot\ndata: {\"sequence\":0,\"tasks\":[]}\n\n";
}

export function resourceWatchSnapshotBody(): string {
  return "event: snapshot\ndata: {\"kind\":\"snapshot\",\"family\":\"runtime\"}\n\n";
}

function respond(
  response: ServerResponse,
  status: number,
  body: string,
  additionalHeaders: Readonly<Record<string, string>> = {},
): void {
  response.writeHead(status, {
    "content-type": "application/json",
    "content-length": Buffer.byteLength(body),
    connection: "close",
    ...additionalHeaders,
  });
  response.end(body);
}

function hasChannelBearer(
  headers: Readonly<Record<string, string>>,
  sessionChannels: ReadonlyMap<string, "management" | "task">,
  expectedChannel: "management" | "task",
): boolean {
  const authorization = headers["authorization"] ?? "";
  if (!authorization.startsWith("Bearer ")) return false;
  return sessionChannels.get(authorization.slice("Bearer ".length)) === expectedChannel;
}

function errorBody(code: string): string {
  return JSON.stringify({
    status: "error",
    error: {
      code,
      message: "fake daemon refusal",
      category: "protocol",
      retryable: false,
      stage: "personal-front-door",
    },
  });
}

/** A readiness projection shaped exactly like the daemon's `/personal/status`. */
export function readinessProjectionBody(
  overrides: Readonly<Record<string, unknown>> = {},
): string {
  return JSON.stringify({
    schema_version: 1,
    surface: "personal-status",
    overall: "blocked",
    first_conversation_ready: false,
    evaluated_at_unix_ms: 1_769_000_000_000,
    components: [
      { component: "system", status: "ready", required: true, error_class: null, duration_ms: 1 },
      {
        component: "database",
        status: "blocked",
        required: true,
        error_class: "database_not_prepared",
        duration_ms: 1,
      },
      {
        component: "pi",
        status: "not_configured",
        required: false,
        error_class: "pi_not_configured",
        duration_ms: 0,
      },
    ],
    static_check_is_not_runtime_ready: true,
    profile_claim: "not-claimed",
    gate_claim: "not-claimed",
    authority_side_effects: false,
    ...overrides,
  });
}
