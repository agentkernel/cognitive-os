/**
 * Bounded loopback client for the Personal daemon front door (ADR-0022).
 *
 * This is the Extension's only source of CognitiveOS facts. It speaks exactly
 * the protocol `apps/kernel-server/src/personal/server.rs` implements:
 *
 *   - `POST /local/session` with `{channel, principal_id, bootstrap_secret}`
 *     mints a channel-scoped bearer;
 *   - `GET /personal/status` requires that bearer on the `management` channel;
 *   - cookies are forbidden, and the `Host` header must be loopback.
 *
 * The daemon holds sessions in memory, so a daemon restart invalidates every
 * token. The client therefore re-mints once on a `401` and fails explicitly if
 * the second attempt is still refused — it never falls back to a cached or
 * synthesized projection. There is no "assume ready" path anywhere in this file.
 *
 * This client is read-only: it performs no state transition, requests no
 * capability and carries no authority. `authority_side_effects` is asserted to
 * be `false` on every projection it accepts.
 */

import {
  nodeFileReader,
  readBootstrapSecret,
  readDaemonEndpoint,
  resolvePersonalDaemonPaths,
  type EnvironmentSlice,
  type FileReader,
  type PersonalDaemonPaths,
} from "./daemon-discovery.js";
import { DaemonClientError } from "./errors.js";

/** Principal the Personal CLI uses for the local owner session. */
export const LOCAL_OWNER_PRINCIPAL = "principal://local/owner";

/** The Extension only ever asks for the read-only management channel. */
export const EXTENSION_CHANNEL = "management";

/** Default per-request deadline. The daemon's own header budget is 10s. */
export const DEFAULT_REQUEST_TIMEOUT_MS = 5_000;

export type OverallReadiness = "blocked" | "degraded" | "ready";

export interface ReadinessComponent {
  readonly component: string;
  readonly status: string;
  readonly required: boolean;
  readonly errorClass: string | undefined;
}

/** The `/personal/status` projection, narrowed to what the Extension displays. */
export interface ReadinessProjection {
  readonly schemaVersion: number;
  readonly surface: string;
  readonly overall: OverallReadiness;
  readonly firstConversationReady: boolean;
  readonly components: readonly ReadinessComponent[];
  readonly staticCheckIsNotRuntimeReady: boolean;
  readonly profileClaim: string;
  readonly gateClaim: string;
  readonly authoritySideEffects: boolean;
}

/** Read-only daemon model identity used to register one Pi model. */
export interface SelectedModelProjection {
  readonly schemaVersion: number;
  readonly surface: string;
  readonly selectedModel: string;
  readonly selectedSnapshotDigest: string;
  readonly chatCapable: true;
  readonly authoritySideEffects: false;
}

/** The only completion result accepted by the one-shot bridge. */
export interface BoundedCompletion {
  readonly content: string;
  readonly finishReason: "stop";
}

export type FetchLike = (input: string, init: RequestInit) => Promise<Response>;

export interface PersonalDaemonClientOptions {
  readonly environment?: EnvironmentSlice;
  readonly files?: FileReader;
  readonly fetchImpl?: FetchLike;
  readonly requestTimeoutMs?: number;
}

export class PersonalDaemonClient {
  private readonly environment: EnvironmentSlice;
  private readonly files: FileReader;
  private readonly fetchImpl: FetchLike;
  private readonly requestTimeoutMs: number;
  private sessionToken: string | undefined;

  constructor(options: PersonalDaemonClientOptions = {}) {
    this.environment = options.environment ?? process.env;
    this.files = options.files ?? nodeFileReader;
    this.fetchImpl = options.fetchImpl ?? defaultFetch;
    this.requestTimeoutMs = options.requestTimeoutMs ?? DEFAULT_REQUEST_TIMEOUT_MS;
  }

  /**
   * Fetch the readiness projection. Throws `DaemonClientError` on every
   * failure path; there is no degraded-but-silent return.
   */
  async fetchReadiness(): Promise<ReadinessProjection> {
    const paths = resolvePersonalDaemonPaths(this.environment);
    const endpoint = readDaemonEndpoint(paths, this.files);

    let token = this.sessionToken ?? (await this.issueSession(endpoint, paths));
    let response = await this.getStatus(endpoint, token);
    if (response.status === 401) {
      // The daemon keeps sessions in memory; a restart invalidates the bearer.
      // Re-mint exactly once, then fail explicitly.
      this.sessionToken = undefined;
      token = await this.issueSession(endpoint, paths);
      response = await this.getStatus(endpoint, token);
    }

    const bodyText = await readBodyText(response);
    if (response.status !== 200) {
      throw authOrProtocolError(response.status, bodyText, "GET /personal/status");
    }
    this.sessionToken = token;
    return parseReadinessProjection(bodyText);
  }

  /** Fetch the daemon-owned selected model, reminting only before dispatch. */
  async fetchSelectedModel(signal?: AbortSignal): Promise<SelectedModelProjection> {
    const paths = resolvePersonalDaemonPaths(this.environment);
    const endpoint = readDaemonEndpoint(paths, this.files);
    if (signal?.aborted) throw abortedRequestError();

    let token = this.sessionToken ?? (await this.issueSession(endpoint, paths));
    let response = await this.getSelectedModel(endpoint, token, signal);
    if (response.status === 401) {
      this.sessionToken = undefined;
      token = await this.issueSession(endpoint, paths);
      response = await this.getSelectedModel(endpoint, token, signal);
    }
    const bodyText = await readBodyText(response);
    if (response.status !== 200) {
      this.sessionToken = undefined;
      throw authOrProtocolError(response.status, bodyText, "GET /provider/v1/selected-model");
    }
    this.sessionToken = token;
    return parseSelectedModelProjection(bodyText);
  }

  /**
   * Dispatch one bounded non-streaming completion. This method never retries:
   * replaying an accepted completion could duplicate Provider billing/output.
   */
  async completeChat(
    model: string,
    messages: readonly { readonly role: "system" | "user" | "assistant"; readonly content: string }[],
    signal?: AbortSignal,
  ): Promise<BoundedCompletion> {
    if (signal?.aborted) throw abortedRequestError();
    const paths = resolvePersonalDaemonPaths(this.environment);
    const endpoint = readDaemonEndpoint(paths, this.files);
    const token = this.sessionToken ?? (await this.issueSession(endpoint, paths));
    const response = await this.send(endpoint, "/provider/v1/chat/completions", {
      method: "POST",
      headers: { authorization: `Bearer ${token}`, "content-type": "application/json" },
      body: JSON.stringify({ model, messages, stream: false }),
      ...(signal === undefined ? {} : { signal }),
    });
    const bodyText = await readBodyText(response);
    if (response.status !== 200) {
      if (response.status === 401) this.sessionToken = undefined;
      throw authOrProtocolError(response.status, bodyText, "POST /provider/v1/chat/completions");
    }
    this.sessionToken = token;
    return parseBoundedCompletion(bodyText);
  }

  /** Drop the cached bearer, e.g. after the operator restarts the daemon. */
  forgetSession(): void {
    this.sessionToken = undefined;
  }

  private async issueSession(endpoint: string, paths: PersonalDaemonPaths): Promise<string> {
    const bootstrapSecret = readBootstrapSecret(paths, this.files);
    const body = JSON.stringify({
      channel: EXTENSION_CHANNEL,
      principal_id: LOCAL_OWNER_PRINCIPAL,
      bootstrap_secret: bootstrapSecret,
    });

    const response = await this.send(endpoint, "/local/session", {
      method: "POST",
      headers: {
        // Content-Length matters: the daemon treats a missing length as a
        // zero-length body. fetch derives it from this string body.
        "content-type": "application/json",
      },
      body,
    });

    const bodyText = await readBodyText(response);
    if (response.status !== 200) {
      throw authOrProtocolError(response.status, bodyText, "POST /local/session");
    }

    const token = extractStringField(bodyText, "token");
    if (token === undefined || token.length === 0) {
      throw new DaemonClientError(
        "PI_EXTENSION_DAEMON_PROTOCOL_ERROR",
        "daemon session response carried no token",
        { httpStatus: response.status },
      );
    }
    const channel = extractStringField(bodyText, "channel");
    if (channel !== EXTENSION_CHANNEL) {
      throw new DaemonClientError(
        "PI_EXTENSION_DAEMON_PROTOCOL_ERROR",
        `daemon issued a session on channel ${channel ?? "<absent>"}, expected ${EXTENSION_CHANNEL}`,
        { httpStatus: response.status },
      );
    }
    this.sessionToken = token;
    return token;
  }

  private async getStatus(endpoint: string, token: string): Promise<Response> {
    return this.send(endpoint, "/personal/status", {
      method: "GET",
      headers: { authorization: `Bearer ${token}` },
    });
  }

  private async getSelectedModel(
    endpoint: string,
    token: string,
    signal?: AbortSignal,
  ): Promise<Response> {
    return this.send(endpoint, "/provider/v1/selected-model", {
      method: "GET",
      headers: { authorization: `Bearer ${token}` },
      ...(signal === undefined ? {} : { signal }),
    });
  }

  private async send(endpoint: string, route: string, init: RequestInit): Promise<Response> {
    const url = `http://${endpoint}${route}`;
    try {
      return await this.fetchImpl(url, {
        ...init,
        // Cookies are forbidden by the daemon; never attach ambient credentials.
        credentials: "omit",
        redirect: "error",
        signal: init.signal ?? AbortSignal.timeout(this.requestTimeoutMs),
      });
    } catch (cause) {
      throw new DaemonClientError(
        "PI_EXTENSION_DAEMON_UNREACHABLE",
        `cannot reach the CognitiveOS daemon at ${endpoint} (${route}); start it with \`cognitive daemon start\``,
        {},
      );
    }
  }
}

function abortedRequestError(): DaemonClientError {
  return new DaemonClientError(
    "PI_EXTENSION_DAEMON_UNREACHABLE",
    "the CognitiveOS completion request was cancelled",
  );
}

export function parseSelectedModelProjection(bodyText: string): SelectedModelProjection {
  const record = parseJsonRecord(bodyText, "selected-model response");
  const schemaVersion = record["schema_version"];
  const surface = record["surface"];
  const selectedModel = record["selected_model"];
  const selectedSnapshotDigest = record["selected_snapshot_digest"];
  if (
    schemaVersion !== 1 ||
    typeof surface !== "string" ||
    typeof selectedModel !== "string" || selectedModel.length === 0 ||
    typeof selectedSnapshotDigest !== "string" || selectedSnapshotDigest.length === 0 ||
    record["chat_capable"] !== true ||
    record["authority_side_effects"] !== false
  ) {
    throw new DaemonClientError("PI_EXTENSION_DAEMON_PROTOCOL_ERROR", "the CognitiveOS daemon returned an invalid selected-model projection");
  }
  return { schemaVersion, surface, selectedModel, selectedSnapshotDigest, chatCapable: true, authoritySideEffects: false };
}

export function parseBoundedCompletion(bodyText: string): BoundedCompletion {
  const record = parseJsonRecord(bodyText, "completion response");
  const choices = record["choices"];
  if (!Array.isArray(choices) || choices.length !== 1) throw completionProtocolError();
  const choice = choices[0];
  if (typeof choice !== "object" || choice === null) throw completionProtocolError();
  const choiceRecord = choice as Record<string, unknown>;
  const message = choiceRecord["message"];
  if (typeof message !== "object" || message === null) throw completionProtocolError();
  const messageRecord = message as Record<string, unknown>;
  if (typeof messageRecord["content"] !== "string" || "tool_calls" in messageRecord || "function_call" in messageRecord) {
    throw completionProtocolError();
  }
  if (choiceRecord["finish_reason"] !== "stop") throw completionProtocolError();
  return { content: messageRecord["content"], finishReason: "stop" };
}

function parseJsonRecord(bodyText: string, label: string): Record<string, unknown> {
  try {
    const parsed: unknown = JSON.parse(bodyText);
    if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) throw new Error();
    return parsed as Record<string, unknown>;
  } catch {
    throw new DaemonClientError("PI_EXTENSION_DAEMON_PROTOCOL_ERROR", `the CognitiveOS daemon ${label} was not a valid JSON object`);
  }
}

function completionProtocolError(): DaemonClientError {
  return new DaemonClientError("PI_EXTENSION_DAEMON_PROTOCOL_ERROR", "the CognitiveOS daemon completion response was unsupported");
}

function defaultFetch(input: string, init: RequestInit): Promise<Response> {
  return globalThis.fetch(input, init);
}

async function readBodyText(response: Response): Promise<string> {
  try {
    return await response.text();
  } catch {
    throw new DaemonClientError(
      "PI_EXTENSION_DAEMON_UNREACHABLE",
      "the CognitiveOS daemon closed the connection before the response body was read",
      { httpStatus: response.status },
    );
  }
}

function authOrProtocolError(
  httpStatus: number,
  bodyText: string,
  route: string,
): DaemonClientError {
  const daemonErrorCode = extractDaemonErrorCode(bodyText);
  const suffix = daemonErrorCode === undefined ? "" : ` (${daemonErrorCode})`;
  if (httpStatus === 401 || httpStatus === 403) {
    return new DaemonClientError(
      "PI_EXTENSION_DAEMON_AUTH_REFUSED",
      `the CognitiveOS daemon refused ${route} with HTTP ${httpStatus}${suffix}`,
      daemonErrorCode === undefined ? { httpStatus } : { httpStatus, daemonErrorCode },
    );
  }
  return new DaemonClientError(
    "PI_EXTENSION_DAEMON_PROTOCOL_ERROR",
    `the CognitiveOS daemon answered ${route} with HTTP ${httpStatus}${suffix}`,
    daemonErrorCode === undefined ? { httpStatus } : { httpStatus, daemonErrorCode },
  );
}

function extractDaemonErrorCode(bodyText: string): string | undefined {
  try {
    const parsed: unknown = JSON.parse(bodyText);
    if (typeof parsed !== "object" || parsed === null) {
      return undefined;
    }
    const error = (parsed as Record<string, unknown>)["error"];
    if (typeof error !== "object" || error === null) {
      return undefined;
    }
    const code = (error as Record<string, unknown>)["code"];
    return typeof code === "string" ? code : undefined;
  } catch {
    return undefined;
  }
}

function extractStringField(bodyText: string, field: string): string | undefined {
  try {
    const parsed: unknown = JSON.parse(bodyText);
    if (typeof parsed !== "object" || parsed === null) {
      return undefined;
    }
    const value = (parsed as Record<string, unknown>)[field];
    return typeof value === "string" ? value : undefined;
  } catch {
    return undefined;
  }
}

/**
 * Parse and validate the readiness projection.
 *
 * Anything unexpected is a protocol error rather than a default: an Extension
 * that guessed `ready` on a malformed projection would be exactly the "synthetic
 * ready" failure B01 forbids.
 */
export function parseReadinessProjection(bodyText: string): ReadinessProjection {
  let parsed: unknown;
  try {
    parsed = JSON.parse(bodyText);
  } catch {
    throw new DaemonClientError(
      "PI_EXTENSION_DAEMON_PROTOCOL_ERROR",
      "the CognitiveOS daemon status response was not valid JSON",
    );
  }
  if (typeof parsed !== "object" || parsed === null) {
    throw new DaemonClientError(
      "PI_EXTENSION_DAEMON_PROTOCOL_ERROR",
      "the CognitiveOS daemon status response was not a JSON object",
    );
  }
  const record = parsed as Record<string, unknown>;

  const schemaVersion = record["schema_version"];
  if (typeof schemaVersion !== "number") {
    throw protocolFieldError("schema_version");
  }
  const surface = record["surface"];
  if (typeof surface !== "string") {
    throw protocolFieldError("surface");
  }
  const overall = record["overall"];
  if (overall !== "blocked" && overall !== "degraded" && overall !== "ready") {
    throw protocolFieldError("overall");
  }
  const firstConversationReady = record["first_conversation_ready"];
  if (typeof firstConversationReady !== "boolean") {
    throw protocolFieldError("first_conversation_ready");
  }
  const staticCheckIsNotRuntimeReady = record["static_check_is_not_runtime_ready"];
  if (typeof staticCheckIsNotRuntimeReady !== "boolean") {
    throw protocolFieldError("static_check_is_not_runtime_ready");
  }
  const profileClaim = record["profile_claim"];
  if (typeof profileClaim !== "string") {
    throw protocolFieldError("profile_claim");
  }
  const gateClaim = record["gate_claim"];
  if (typeof gateClaim !== "string") {
    throw protocolFieldError("gate_claim");
  }
  const authoritySideEffects = record["authority_side_effects"];
  if (authoritySideEffects !== false) {
    // The read-only projection must never report an authority side effect. If
    // it ever does, this client refuses the response rather than display it.
    throw new DaemonClientError(
      "PI_EXTENSION_DAEMON_PROTOCOL_ERROR",
      "the CognitiveOS daemon status response reported an authority side effect on a read-only projection",
    );
  }

  const rawComponents = record["components"];
  if (!Array.isArray(rawComponents)) {
    throw protocolFieldError("components");
  }

  const components: ReadinessComponent[] = [];
  for (const entry of rawComponents) {
    if (typeof entry !== "object" || entry === null) {
      throw protocolFieldError("components[]");
    }
    const componentRecord = entry as Record<string, unknown>;
    const component = componentRecord["component"];
    const status = componentRecord["status"];
    const required = componentRecord["required"];
    if (typeof component !== "string" || typeof status !== "string" || typeof required !== "boolean") {
      throw protocolFieldError("components[]");
    }
    const errorClass = componentRecord["error_class"];
    components.push({
      component,
      status,
      required,
      errorClass: typeof errorClass === "string" ? errorClass : undefined,
    });
  }

  return {
    schemaVersion,
    surface,
    overall,
    firstConversationReady,
    components,
    staticCheckIsNotRuntimeReady,
    profileClaim,
    gateClaim,
    authoritySideEffects: false,
  };
}

function protocolFieldError(field: string): DaemonClientError {
  return new DaemonClientError(
    "PI_EXTENSION_DAEMON_PROTOCOL_ERROR",
    `the CognitiveOS daemon status response has no usable ${field} field`,
  );
}
