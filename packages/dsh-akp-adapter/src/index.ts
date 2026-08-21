import { performance } from "node:perf_hooks";
import { createInterface } from "node:readline";
import type { Readable, Writable } from "node:stream";

export const BRIDGE_PROTOCOL = "cognitiveos.dsh-akp/0.1" as const;
export const ADAPTER_ID = "deepseek.dsh.akp" as const;
export const PINNED_DSH_REVISION = "528c682e061696f5a160f363f236ecbf53cbd006";
export const PINNED_AKP_SCHEMA_DIGEST =
  "sha256:feeaeb0942ce2796d0155b4b9c316a87cca94eccbf7b0fd7b031a2135dd7ee7b";
export const DEFAULT_MAX_FRAME_BYTES = 1_048_576;
export const DEFAULT_TIMEOUT_MS = 10_000;
export const DEFAULT_FENCING_EPOCH = 1;

const SECRET_FIELD_NAMES = ["api_key", "apikey", "authorization", "password", "secret", "token"];
const AUTHORITY_FIELD_NAMES = [
  "task_ref",
  "authorization_id",
  "effect",
  "acceptance",
  "budget",
  "lease",
  "wia",
  "worker_authorization",
  "complete",
  "completed",
  "capability",
];

export type PluginEventKind = "candidate" | "observation" | "lifecycle";

export interface DshPluginEvent {
  readonly kind: PluginEventKind;
  readonly operation: string;
  readonly payload: JsonValue;
  readonly authorityClaim?: boolean;
  readonly secretShaped?: boolean;
}

export interface DshAdapterRequest {
  readonly bridgeProtocol: typeof BRIDGE_PROTOCOL;
  readonly dshVersion: string;
  readonly schemaDigest: string;
  readonly sessionId: string;
  readonly fencingEpoch: number;
  readonly sequence: number;
  readonly pluginId: string;
  readonly correlationId: string;
  readonly deadline: string;
  readonly taskRef?: string;
  readonly event: DshPluginEvent;
}

export interface DshAdapterResponse {
  readonly accepted: boolean;
  readonly sequence: number;
  readonly candidateOnly: true;
  readonly result?: JsonValue;
  readonly error?: string;
}

export interface DshAkpTiming {
  readonly serializationNanos: number;
  readonly transportNanos: number;
  readonly totalNanos: number;
}

export interface DshAkpResult {
  readonly response: DshAdapterResponse;
  readonly timing: DshAkpTiming;
}

export type JsonPrimitive = string | number | boolean | null;
export type JsonValue = JsonPrimitive | { readonly [key: string]: JsonValue } | readonly JsonValue[];

export interface AkpTransport {
  send(request: DshAdapterRequest, signal?: AbortSignal): Promise<DshAdapterResponse>;
  activate?(
    session: {
      readonly dshVersion: string;
      readonly sessionId: string;
      readonly fencingEpoch: number;
      readonly taskRef?: string;
    },
    signal?: AbortSignal,
  ): Promise<DshAdapterResponse>;
}

export function encodeRequest(request: DshAdapterRequest, op = "event"): string {
  return JSON.stringify({
    op,
    bridge_protocol: request.bridgeProtocol,
    dsh_version: request.dshVersion,
    schema_digest: request.schemaDigest,
    session_id: request.sessionId,
    fencing_epoch: request.fencingEpoch,
    sequence: request.sequence,
    plugin_id: request.pluginId,
    correlation_id: request.correlationId,
    deadline: request.deadline,
    ...(request.taskRef ? { task_ref: request.taskRef } : {}),
    event: {
      kind: request.event.kind,
      operation: request.event.operation,
      payload: request.event.payload,
      authority_claim: request.event.authorityClaim === true,
      secret_shaped: request.event.secretShaped === true,
    },
  });
}

export function decodeResponse(parsed: unknown): DshAdapterResponse {
  if (parsed === null || typeof parsed !== "object") {
    throw new DshAdapterError("RESPONSE_INVALID", "dsh AKP response must be an object");
  }
  const row = parsed as Record<string, unknown>;
  const candidateOnly = row["candidate_only"] ?? row["candidateOnly"];
  if (
    typeof row["accepted"] !== "boolean" ||
    typeof row["sequence"] !== "number" ||
    candidateOnly !== true
  ) {
    throw new DshAdapterError(
      "RESPONSE_INVALID",
      "daemon response is not a candidate-only response",
    );
  }
  return {
    accepted: row["accepted"],
    sequence: row["sequence"],
    candidateOnly: true,
    ...(row["result"] !== undefined ? { result: toJsonValue(row["result"]) } : {}),
    ...(typeof row["error"] === "string" ? { error: row["error"] } : {}),
  };
}

/**
 * Length-bounded JSONL transport for a long-lived dsh child process. One
 * request is in flight at a time; this preserves the adapter sequence and
 * prevents stdout responses from being attributed to the wrong event.
 */
export class JsonlAkpTransport implements AkpTransport {
  private pending: Promise<unknown> = Promise.resolve();
  private readonly lines;

  public constructor(
    private readonly input: Readable,
    private readonly output: Writable,
    private readonly maxFrameBytes = DEFAULT_MAX_FRAME_BYTES,
  ) {
    this.lines = createInterface({ input });
  }

  public send(request: DshAdapterRequest, signal?: AbortSignal): Promise<DshAdapterResponse> {
    const operation = this.pending.then(() => this.sendOne(request, signal));
    this.pending = operation.then(() => undefined, () => undefined);
    return operation;
  }

  public close(): void {
    this.lines.close();
  }

  private async sendOne(request: DshAdapterRequest, signal?: AbortSignal): Promise<DshAdapterResponse> {
    const frame = encodeRequest(request);
    if (Buffer.byteLength(frame, "utf8") > this.maxFrameBytes) {
      throw new DshAdapterError("FRAME_TOO_LARGE", "dsh AKP JSONL frame exceeds the configured byte limit");
    }
    if (signal?.aborted) throw new DshAdapterError("TRANSPORT_ERROR", "dsh AKP transport was aborted");
    const responsePromise = new Promise<string>((resolve, reject) => {
      const onAbort = (): void => {
        this.lines.removeListener("line", onLine);
        reject(new DshAdapterError("TRANSPORT_ERROR", "dsh AKP transport was aborted"));
      };
      const onLine = (line: string): void => {
        signal?.removeEventListener("abort", onAbort);
        resolve(line);
      };
      signal?.addEventListener("abort", onAbort, { once: true });
      this.lines.once("line", onLine);
    });
    await new Promise<void>((resolve, reject) => {
      this.output.write(`${frame}\n`, (error?: Error | null) => (error ? reject(error) : resolve()));
    });
    let line: string;
    try {
      line = await responsePromise;
    } catch (error) {
      throw new DshAdapterError("TRANSPORT_ERROR", error instanceof Error ? error.message : "dsh AKP read failed");
    }
    let parsed: unknown;
    try {
      parsed = JSON.parse(line);
    } catch {
      throw new DshAdapterError("RESPONSE_INVALID", "dsh AKP response is not JSON");
    }
    return decodeResponse(parsed);
  }
}

export interface HttpAkpTransportOptions {
  readonly endpoint: string;
  readonly bearer: string;
  readonly maxFrameBytes?: number;
  readonly fetchImpl?: typeof fetch;
}

/**
 * HTTP transport for `POST /task/akp/dsh`. The bearer is supplied by the
 * harness, never read from a plugin event, and never logged.
 */
export class HttpAkpTransport implements AkpTransport {
  private readonly endpoint: string;
  private readonly bearer: string;
  private readonly maxFrameBytes: number;
  private readonly fetchImpl: typeof fetch;

  public constructor(options: HttpAkpTransportOptions) {
    if (!options.endpoint.trim() || !options.bearer.trim()) {
      throw new DshAdapterError("INVALID_EVENT", "dsh HTTP transport requires an endpoint and harness bearer");
    }
    this.endpoint = options.endpoint;
    this.bearer = options.bearer;
    this.maxFrameBytes = options.maxFrameBytes ?? DEFAULT_MAX_FRAME_BYTES;
    this.fetchImpl = options.fetchImpl ?? fetch;
  }

  public activate(
    session: {
      readonly dshVersion: string;
      readonly sessionId: string;
      readonly fencingEpoch: number;
      readonly taskRef?: string;
    },
    signal?: AbortSignal,
  ): Promise<DshAdapterResponse> {
    return this.post(
      {
        op: "activate",
        dsh_version: session.dshVersion,
        session_id: session.sessionId,
        fencing_epoch: session.fencingEpoch,
        ...(session.taskRef ? { task_ref: session.taskRef } : {}),
      },
      signal,
    );
  }

  public send(request: DshAdapterRequest, signal?: AbortSignal): Promise<DshAdapterResponse> {
    return this.post(JSON.parse(encodeRequest(request)) as Record<string, JsonValue>, signal);
  }

  private async post(body: Record<string, JsonValue>, signal?: AbortSignal): Promise<DshAdapterResponse> {
    const frame = JSON.stringify(body);
    if (Buffer.byteLength(frame, "utf8") > this.maxFrameBytes) {
      throw new DshAdapterError("FRAME_TOO_LARGE", "dsh AKP HTTP frame exceeds the configured byte limit");
    }
    let response: Response;
    try {
      response = await this.fetchImpl(this.endpoint, {
        method: "POST",
        headers: {
          "content-type": "application/json",
          authorization: `Bearer ${this.bearer}`,
        },
        body: frame,
        ...(signal ? { signal } : {}),
      });
    } catch (error) {
      throw new DshAdapterError("TRANSPORT_ERROR", error instanceof Error ? error.message : "dsh AKP HTTP failed");
    }
    let parsed: unknown;
    try {
      parsed = await response.json();
    } catch {
      throw new DshAdapterError("RESPONSE_INVALID", "dsh AKP HTTP response is not JSON");
    }
    return decodeResponse(parsed);
  }
}

export interface DshAkpAdapterOptions {
  readonly dshVersion: string;
  readonly schemaDigest: string;
  readonly sessionId: string;
  readonly pluginId: string;
  readonly transport: AkpTransport;
  readonly fencingEpoch?: number;
  readonly taskRef?: string;
  readonly maxFrameBytes?: number;
  readonly timeoutMs?: number;
  readonly now?: () => number;
}

export class DshAdapterError extends Error {
  public constructor(
    public readonly code:
      | "INVALID_EVENT"
      | "AUTHORITY_CLAIM_FORBIDDEN"
      | "SECRET_SHAPED_PAYLOAD"
      | "FORBIDDEN_PAYLOAD_FIELD"
      | "FRAME_TOO_LARGE"
      | "SEQUENCE_NOT_MONOTONIC"
      | "TIMEOUT"
      | "TRANSPORT_ERROR"
      | "RESPONSE_INVALID",
    message: string,
  ) {
    super(message);
    this.name = "DshAdapterError";
  }
}

/**
 * Candidate-only dsh -> AKP boundary. The transport is injected so the dsh
 * shim can use a daemon-owned socket, HTTP, or a test loopback without giving
 * dsh authority or Provider credentials.
 */
export class DshAkpAdapter {
  private readonly maxFrameBytes: number;
  private readonly timeoutMs: number;
  private readonly fencingEpoch: number;
  private readonly now: () => number;
  private pending: Promise<unknown> = Promise.resolve();
  private sequence = 0;
  private active = false;

  public constructor(private readonly options: DshAkpAdapterOptions) {
    if (
      !options.dshVersion.trim() ||
      !options.schemaDigest.trim() ||
      !options.sessionId.trim() ||
      !options.pluginId.trim()
    ) {
      throw new DshAdapterError("INVALID_EVENT", "dsh version, schema digest, session, and plugin identity are required");
    }
    this.maxFrameBytes = options.maxFrameBytes ?? DEFAULT_MAX_FRAME_BYTES;
    this.timeoutMs = options.timeoutMs ?? DEFAULT_TIMEOUT_MS;
    this.fencingEpoch = options.fencingEpoch ?? DEFAULT_FENCING_EPOCH;
    this.now = options.now ?? (() => performance.now());
  }

  public get lastSequence(): number {
    return this.sequence;
  }

  public activate(): void {
    this.active = true;
    this.sequence = 0;
    const activate = this.options.transport.activate;
    if (activate) {
      const operation = this.pending.then(() =>
        activate.call(this.options.transport, {
          dshVersion: this.options.dshVersion,
          sessionId: this.options.sessionId,
          fencingEpoch: this.fencingEpoch,
          ...(this.options.taskRef ? { taskRef: this.options.taskRef } : {}),
        }),
      );
      this.pending = operation.then(
        () => undefined,
        (error: unknown) => {
          this.active = false;
          throw error;
        },
      );
    }
  }

  public stop(): void {
    this.active = false;
  }

  public submit(event: DshPluginEvent, signal?: AbortSignal): Promise<DshAkpResult> {
    const operation = this.pending.then(() => this.submitOne(event, signal));
    this.pending = operation.then(() => undefined, () => undefined);
    return operation;
  }

  private async submitOne(event: DshPluginEvent, signal?: AbortSignal): Promise<DshAkpResult> {
    if (!this.active) throw new DshAdapterError("TRANSPORT_ERROR", "dsh AKP adapter is inactive");
    validateEvent(event);
    const sequence = this.sequence + 1;
    const request: DshAdapterRequest = {
      bridgeProtocol: BRIDGE_PROTOCOL,
      dshVersion: this.options.dshVersion,
      schemaDigest: this.options.schemaDigest,
      sessionId: this.options.sessionId,
      fencingEpoch: this.fencingEpoch,
      sequence,
      pluginId: this.options.pluginId,
      correlationId: `${this.options.sessionId}:${sequence}`,
      deadline: new Date(Date.now() + this.timeoutMs).toISOString(),
      ...(this.options.taskRef ? { taskRef: this.options.taskRef } : {}),
      event,
    };
    const started = this.now();
    const frame = encodeRequest(request);
    const serializedBytes = Buffer.byteLength(frame, "utf8");
    if (serializedBytes > this.maxFrameBytes) {
      throw new DshAdapterError("FRAME_TOO_LARGE", "dsh AKP frame exceeds the configured byte limit");
    }
    const serializedAt = this.now();
    let response: DshAdapterResponse;
    const timeoutController = new AbortController();
    const timeout = setTimeout(() => timeoutController.abort(), this.timeoutMs);
    const forwardAbort = (): void => timeoutController.abort();
    signal?.addEventListener("abort", forwardAbort, { once: true });
    try {
      response = await this.options.transport.send(request, timeoutController.signal);
    } catch (error) {
      if (timeoutController.signal.aborted && !signal?.aborted) {
        throw new DshAdapterError("TIMEOUT", `dsh AKP transport exceeded ${this.timeoutMs} ms`);
      }
      throw new DshAdapterError("TRANSPORT_ERROR", error instanceof Error ? error.message : "transport failed");
    } finally {
      clearTimeout(timeout);
      signal?.removeEventListener("abort", forwardAbort);
    }
    const finished = this.now();
    validateResponse(response, sequence);
    this.sequence = sequence;
    return {
      response,
      timing: {
        serializationNanos: Math.max(0, Math.round((serializedAt - started) * 1_000_000)),
        transportNanos: Math.max(0, Math.round((finished - serializedAt) * 1_000_000)),
        totalNanos: Math.max(0, Math.round((finished - started) * 1_000_000)),
      },
    };
  }
}

function validateEvent(event: DshPluginEvent): void {
  if (!event.operation.trim() || event.payload === undefined) {
    throw new DshAdapterError("INVALID_EVENT", "dsh event operation and payload are required");
  }
  if (event.authorityClaim === true) {
    throw new DshAdapterError("AUTHORITY_CLAIM_FORBIDDEN", "dsh events cannot claim CognitiveOS authority");
  }
  if (event.secretShaped === true) {
    throw new DshAdapterError("SECRET_SHAPED_PAYLOAD", "dsh event contains secret-shaped material");
  }
  const rejection = payloadRejection(event.payload);
  if (rejection) throw rejection;
}

function payloadRejection(value: JsonValue): DshAdapterError | undefined {
  if (typeof value === "string") {
    if (value.startsWith("sk-") || value.includes("Bearer ")) {
      return new DshAdapterError("SECRET_SHAPED_PAYLOAD", "dsh event contains secret-shaped material");
    }
    return undefined;
  }
  if (Array.isArray(value)) {
    for (const child of value) {
      const rejection = payloadRejection(child);
      if (rejection) return rejection;
    }
    return undefined;
  }
  if (value !== null && typeof value === "object") {
    for (const [key, child] of Object.entries(value)) {
      const normalized = key.toLowerCase();
      if (SECRET_FIELD_NAMES.includes(normalized)) {
        return new DshAdapterError("SECRET_SHAPED_PAYLOAD", "dsh event contains secret-shaped material");
      }
      if (AUTHORITY_FIELD_NAMES.includes(normalized)) {
        return new DshAdapterError("FORBIDDEN_PAYLOAD_FIELD", "dsh event contains an authority-shaped field");
      }
      const rejection = payloadRejection(child);
      if (rejection) return rejection;
    }
  }
  return undefined;
}

function validateResponse(response: DshAdapterResponse, sequence: number): void {
  if (response.sequence !== sequence || response.candidateOnly !== true || typeof response.accepted !== "boolean") {
    throw new DshAdapterError("RESPONSE_INVALID", "daemon response is not a candidate-only response for this sequence");
  }
}

export interface DshCordisEventSource {
  on(event: string, listener: (payload: unknown) => void): void;
}

export interface DshCordisPluginOptions extends DshAkpAdapterOptions {
  readonly eventName?: string;
  readonly onResult?: (result: DshAkpResult) => void;
  readonly onError?: (error: unknown) => void;
}

/**
 * Small Cordis-compatible attach function. It intentionally depends only on
 * the host's `on` event primitive, which keeps dsh preview API churn in this
 * package instead of the Rust daemon or AKP contract.
 */
export function attachDshCordisPlugin(
  host: DshCordisEventSource,
  options: DshCordisPluginOptions,
): DshAkpAdapter {
  const adapter = new DshAkpAdapter(options);
  adapter.activate();
  host.on(options.eventName ?? "cognitiveos:candidate", (payload) => {
    void adapter.submit(normalizeHostEvent(payload)).then(options.onResult).catch(options.onError);
  });
  return adapter;
}

function normalizeHostEvent(payload: unknown): DshPluginEvent {
  if (payload === null || typeof payload !== "object") {
    throw new DshAdapterError("INVALID_EVENT", "dsh host event must be an object");
  }
  const candidate = payload as Record<string, unknown>;
  const kind = candidate["kind"];
  const operation = candidate["operation"];
  if (kind !== "candidate" && kind !== "observation" && kind !== "lifecycle") {
    throw new DshAdapterError("INVALID_EVENT", "dsh host event kind is unsupported");
  }
  if (typeof operation !== "string") {
    throw new DshAdapterError("INVALID_EVENT", "dsh host event operation is missing");
  }
  return {
    kind,
    operation,
    payload: toJsonValue(candidate["payload"]),
    ...(candidate["authorityClaim"] === true ? { authorityClaim: true } : {}),
    ...(candidate["secretShaped"] === true ? { secretShaped: true } : {}),
  };
}

function toJsonValue(value: unknown): JsonValue {
  if (value === null || typeof value === "string" || typeof value === "number" || typeof value === "boolean") {
    return value;
  }
  if (Array.isArray(value)) return value.map(toJsonValue);
  if (typeof value === "object") {
    return Object.fromEntries(Object.entries(value as Record<string, unknown>).map(([key, child]) => [key, toJsonValue(child)]));
  }
  throw new DshAdapterError("INVALID_EVENT", "dsh payload is not JSON-compatible");
}
