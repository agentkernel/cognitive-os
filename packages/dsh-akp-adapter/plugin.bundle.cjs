"use strict";
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name2 in all)
    __defProp(target, name2, { get: all[name2], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// src/plugin.ts
var plugin_exports = {};
__export(plugin_exports, {
  apply: () => apply,
  applyDshAkpCordisPlugin: () => applyDshAkpCordisPlugin,
  inject: () => inject,
  name: () => name
});
module.exports = __toCommonJS(plugin_exports);
var import_node_fs = require("node:fs");

// src/index.ts
var import_node_perf_hooks = require("node:perf_hooks");
var BRIDGE_PROTOCOL = "cognitiveos.dsh-akp/0.1";
var ADAPTER_ID = "deepseek.dsh.akp";
var PINNED_DSH_REVISION = "528c682e061696f5a160f363f236ecbf53cbd006";
var PINNED_AKP_SCHEMA_DIGEST = "sha256:feeaeb0942ce2796d0155b4b9c316a87cca94eccbf7b0fd7b031a2135dd7ee7b";
var DEFAULT_MAX_FRAME_BYTES = 1048576;
var DEFAULT_TIMEOUT_MS = 1e4;
var DEFAULT_FENCING_EPOCH = 1;
var SECRET_FIELD_NAMES = ["api_key", "apikey", "authorization", "password", "secret", "token"];
var AUTHORITY_FIELD_NAMES = [
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
  "capability"
];
function encodeRequest(request, op = "event") {
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
    ...request.taskRef ? { task_ref: request.taskRef } : {},
    event: {
      kind: request.event.kind,
      operation: request.event.operation,
      payload: request.event.payload,
      authority_claim: request.event.authorityClaim === true,
      secret_shaped: request.event.secretShaped === true
    }
  });
}
function decodeResponse(parsed) {
  if (parsed === null || typeof parsed !== "object") {
    throw new DshAdapterError("RESPONSE_INVALID", "dsh AKP response must be an object");
  }
  const row = parsed;
  const candidateOnly = row["candidate_only"] ?? row["candidateOnly"];
  if (typeof row["accepted"] !== "boolean" || typeof row["sequence"] !== "number" || candidateOnly !== true) {
    throw new DshAdapterError(
      "RESPONSE_INVALID",
      "daemon response is not a candidate-only response"
    );
  }
  return {
    accepted: row["accepted"],
    sequence: row["sequence"],
    candidateOnly: true,
    ...row["result"] !== void 0 ? { result: toJsonValue(row["result"]) } : {},
    ...typeof row["error"] === "string" ? { error: row["error"] } : {}
  };
}
var HttpAkpTransport = class {
  endpoint;
  bearer;
  maxFrameBytes;
  fetchImpl;
  constructor(options) {
    if (!options.endpoint.trim() || !options.bearer.trim()) {
      throw new DshAdapterError("INVALID_EVENT", "dsh HTTP transport requires an endpoint and harness bearer");
    }
    this.endpoint = options.endpoint;
    this.bearer = options.bearer;
    this.maxFrameBytes = options.maxFrameBytes ?? DEFAULT_MAX_FRAME_BYTES;
    this.fetchImpl = options.fetchImpl ?? fetch;
  }
  activate(session, signal) {
    return this.post(
      {
        op: "activate",
        dsh_version: session.dshVersion,
        session_id: session.sessionId,
        fencing_epoch: session.fencingEpoch,
        ...session.taskRef ? { task_ref: session.taskRef } : {}
      },
      signal
    );
  }
  send(request, signal) {
    return this.post(JSON.parse(encodeRequest(request)), signal);
  }
  async post(body, signal) {
    const frame = JSON.stringify(body);
    if (Buffer.byteLength(frame, "utf8") > this.maxFrameBytes) {
      throw new DshAdapterError("FRAME_TOO_LARGE", "dsh AKP HTTP frame exceeds the configured byte limit");
    }
    let response;
    try {
      response = await this.fetchImpl(this.endpoint, {
        method: "POST",
        headers: {
          "content-type": "application/json",
          authorization: `Bearer ${this.bearer}`
        },
        body: frame,
        ...signal ? { signal } : {}
      });
    } catch (error) {
      throw new DshAdapterError("TRANSPORT_ERROR", error instanceof Error ? error.message : "dsh AKP HTTP failed");
    }
    let parsed;
    try {
      parsed = await response.json();
    } catch {
      throw new DshAdapterError("RESPONSE_INVALID", "dsh AKP HTTP response is not JSON");
    }
    return decodeResponse(parsed);
  }
};
var DshAdapterError = class extends Error {
  constructor(code, message) {
    super(message);
    this.code = code;
    this.name = "DshAdapterError";
  }
  code;
};
var DshAkpAdapter = class {
  constructor(options) {
    this.options = options;
    if (!options.dshVersion.trim() || !options.schemaDigest.trim() || !options.sessionId.trim() || !options.pluginId.trim()) {
      throw new DshAdapterError("INVALID_EVENT", "dsh version, schema digest, session, and plugin identity are required");
    }
    this.maxFrameBytes = options.maxFrameBytes ?? DEFAULT_MAX_FRAME_BYTES;
    this.timeoutMs = options.timeoutMs ?? DEFAULT_TIMEOUT_MS;
    this.fencingEpoch = options.fencingEpoch ?? DEFAULT_FENCING_EPOCH;
    this.now = options.now ?? (() => import_node_perf_hooks.performance.now());
  }
  options;
  maxFrameBytes;
  timeoutMs;
  fencingEpoch;
  now;
  pending = Promise.resolve();
  sequence = 0;
  active = false;
  get lastSequence() {
    return this.sequence;
  }
  activate() {
    this.active = true;
    this.sequence = 0;
    const activate = this.options.transport.activate;
    if (activate) {
      const operation = this.pending.then(
        () => activate.call(this.options.transport, {
          dshVersion: this.options.dshVersion,
          sessionId: this.options.sessionId,
          fencingEpoch: this.fencingEpoch,
          ...this.options.taskRef ? { taskRef: this.options.taskRef } : {}
        })
      );
      this.pending = operation.then(
        () => void 0,
        (error) => {
          this.active = false;
          throw error;
        }
      );
    }
  }
  stop() {
    this.active = false;
  }
  submit(event, signal) {
    const operation = this.pending.then(() => this.submitOne(event, signal));
    this.pending = operation.then(() => void 0, () => void 0);
    return operation;
  }
  async submitOne(event, signal) {
    if (!this.active) throw new DshAdapterError("TRANSPORT_ERROR", "dsh AKP adapter is inactive");
    validateEvent(event);
    const sequence = this.sequence + 1;
    const request = {
      bridgeProtocol: BRIDGE_PROTOCOL,
      dshVersion: this.options.dshVersion,
      schemaDigest: this.options.schemaDigest,
      sessionId: this.options.sessionId,
      fencingEpoch: this.fencingEpoch,
      sequence,
      pluginId: this.options.pluginId,
      correlationId: `${this.options.sessionId}:${sequence}`,
      deadline: new Date(Date.now() + this.timeoutMs).toISOString(),
      ...this.options.taskRef ? { taskRef: this.options.taskRef } : {},
      event
    };
    const started = this.now();
    const frame = encodeRequest(request);
    const serializedBytes = Buffer.byteLength(frame, "utf8");
    if (serializedBytes > this.maxFrameBytes) {
      throw new DshAdapterError("FRAME_TOO_LARGE", "dsh AKP frame exceeds the configured byte limit");
    }
    const serializedAt = this.now();
    let response;
    const timeoutController = new AbortController();
    const timeout = setTimeout(() => timeoutController.abort(), this.timeoutMs);
    const forwardAbort = () => timeoutController.abort();
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
        serializationNanos: Math.max(0, Math.round((serializedAt - started) * 1e6)),
        transportNanos: Math.max(0, Math.round((finished - serializedAt) * 1e6)),
        totalNanos: Math.max(0, Math.round((finished - started) * 1e6))
      }
    };
  }
};
function validateEvent(event) {
  if (!event.operation.trim() || event.payload === void 0) {
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
function payloadRejection(value) {
  if (typeof value === "string") {
    if (value.startsWith("sk-") || value.includes("Bearer ")) {
      return new DshAdapterError("SECRET_SHAPED_PAYLOAD", "dsh event contains secret-shaped material");
    }
    return void 0;
  }
  if (Array.isArray(value)) {
    for (const child of value) {
      const rejection = payloadRejection(child);
      if (rejection) return rejection;
    }
    return void 0;
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
  return void 0;
}
function validateResponse(response, sequence) {
  if (response.sequence !== sequence || response.candidateOnly !== true || typeof response.accepted !== "boolean") {
    throw new DshAdapterError("RESPONSE_INVALID", "daemon response is not a candidate-only response for this sequence");
  }
}
function attachDshCordisPlugin(host, options) {
  const adapter = new DshAkpAdapter(options);
  adapter.activate();
  host.on(options.eventName ?? "cognitiveos:candidate", (payload) => {
    void adapter.submit(normalizeHostEvent(payload)).then(options.onResult).catch(options.onError);
  });
  return adapter;
}
function normalizeHostEvent(payload) {
  if (payload === null || typeof payload !== "object") {
    throw new DshAdapterError("INVALID_EVENT", "dsh host event must be an object");
  }
  const candidate = payload;
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
    ...candidate["authorityClaim"] === true ? { authorityClaim: true } : {},
    ...candidate["secretShaped"] === true ? { secretShaped: true } : {}
  };
}
function toJsonValue(value) {
  if (value === null || typeof value === "string" || typeof value === "number" || typeof value === "boolean") {
    return value;
  }
  if (Array.isArray(value)) return value.map(toJsonValue);
  if (typeof value === "object") {
    return Object.fromEntries(Object.entries(value).map(([key, child]) => [key, toJsonValue(child)]));
  }
  throw new DshAdapterError("INVALID_EVENT", "dsh payload is not JSON-compatible");
}

// src/plugin.ts
var name = "cognitiveos-akp";
var inject = [];
function apply(ctx, config) {
  return applyDshAkpCordisPlugin(ctx, config);
}
function applyDshAkpCordisPlugin(ctx, config, deps = {}) {
  if (!config.endpoint.trim() || !config.bearerFile.trim()) {
    throw new DshAdapterError(
      "INVALID_EVENT",
      "dsh AKP plugin requires an HTTP endpoint and a bearer file path"
    );
  }
  const transport = deps.transport ?? new HttpAkpTransport({
    endpoint: config.endpoint,
    bearer: readBearerMaterial(config.bearerFile, deps.readBearer)
  });
  const adapter = attachDshCordisPlugin(ctx, {
    dshVersion: PINNED_DSH_REVISION,
    schemaDigest: PINNED_AKP_SCHEMA_DIGEST,
    sessionId: config.sessionId ?? "dsh-cordis",
    pluginId: config.pluginId ?? ADAPTER_ID,
    transport,
    ...config.fencingEpoch !== void 0 ? { fencingEpoch: config.fencingEpoch } : {},
    ...config.taskRef !== void 0 ? { taskRef: config.taskRef } : {},
    ...config.timeoutMs !== void 0 ? { timeoutMs: config.timeoutMs } : {},
    ...deps.onResult ? { onResult: deps.onResult } : {},
    ...deps.onError ? { onError: deps.onError } : {}
  });
  for (const event of config.startupEvents ?? []) {
    void adapter.submit(event).then(deps.onResult).catch(deps.onError);
  }
  return adapter;
}
function readBearerMaterial(path, readBearer) {
  const bearer = (readBearer ?? defaultReadBearer)(path);
  if (!bearer) {
    throw new DshAdapterError("INVALID_EVENT", "dsh AKP bearer file is empty");
  }
  return bearer;
}
function defaultReadBearer(path) {
  return (0, import_node_fs.readFileSync)(path, "utf8").trim();
}
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  apply,
  applyDshAkpCordisPlugin,
  inject,
  name
});
