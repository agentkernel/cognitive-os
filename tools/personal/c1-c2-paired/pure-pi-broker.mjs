/**
 * Campaign-only pure-Pi credential broker (execution plan §2.2 option 2).
 *
 * Not an authority writer. No Context, Tool-as-authority, Memory, Task, retry,
 * cache, or verification. Provider material is never placed in argv, env,
 * logs, or the Pi-facing token.
 */

import http from "node:http";
import { Buffer } from "node:buffer";

export const PI_PLACEHOLDER_TOKEN = "campaign-broker-nonsecret-token";
export const LOOPBACK_HOST = "127.0.0.1";
export const MAX_REQUEST_BYTES = 1024 * 1024;
export const MAX_RESPONSE_BYTES = 4 * 1024 * 1024;
export const UPSTREAM_TIMEOUT_MS = 120_000;

const SECRET_SHAPED =
  /sk-[A-Za-z0-9]{10,}|BEGIN [A-Z ]+PRIVATE KEY|-----BEGIN/;
const SECRET_ENV_NAMES = /^(?:PROVIDER|OPENAI|DEEPSEEK|ANTHROPIC|API|LLM).*KEY$/i;

const FORBIDDEN_CAPABILITIES = [
  "context",
  "memory",
  "skill",
  "task",
  "retry",
  "cache",
  "verify",
  "verification",
  "accept",
  "acceptance",
];

const THREAT_REVIEW_ITEMS = Object.freeze([
  "key_from_linux_secret_service_once",
  "inject_upstream_auth_in_memory_only",
  "loopback_only_single_user",
  "no_request_response_header_logs",
  "pi_sees_only_placeholder_token",
  "per_request_count_duration_byte_bounds",
  "cleanup_drops_process_not_owner_key",
  "no_context_tool_memory_task_retry_cache_verify",
]);

export function isSecretShaped(value) {
  return typeof value === "string" && SECRET_SHAPED.test(value);
}

export function assertSecretFreeProcess({ env = process.env, argv = process.argv } = {}) {
  for (const [name, value] of Object.entries(env)) {
    if (value === undefined || value === "") {
      continue;
    }
    if (SECRET_ENV_NAMES.test(name) || isSecretShaped(String(value))) {
      throw new Error(`secret-shaped process input refused: env ${name}`);
    }
  }
  for (const argument of argv) {
    if (isSecretShaped(String(argument))) {
      throw new Error("secret-shaped process input refused: argv");
    }
  }
}

export function assertLoopbackBind(host) {
  if (host !== LOOPBACK_HOST) {
    throw new Error(`broker bind host must be ${LOOPBACK_HOST}`);
  }
}

export function brokerThreatReview() {
  return Object.freeze({
    kind: "pure-pi-credential-broker",
    plan_section: "2.2 option 2",
    items: THREAT_REVIEW_ITEMS,
    retry: 0,
  });
}

function readLimited(request, maxBytes) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    let size = 0;
    request.on("data", (chunk) => {
      size += chunk.length;
      if (size > maxBytes) {
        reject(new Error("request exceeds byte bound"));
        request.destroy();
        return;
      }
      chunks.push(chunk);
    });
    request.on("end", () => resolve(Buffer.concat(chunks)));
    request.on("error", reject);
  });
}

export function createPurePiBroker({
  host = LOOPBACK_HOST,
  port,
  getSecret,
  upstreamOrigin,
} = {}) {
  assertLoopbackBind(host);
  if (!Number.isInteger(port) || port < 0 || port > 65535) {
    throw new Error("broker port must be a TCP port integer");
  }
  if (typeof getSecret !== "function") {
    throw new Error("broker requires getSecret()");
  }
  if (upstreamOrigin !== undefined) {
    let parsed;
    try {
      parsed = new URL(upstreamOrigin);
    } catch {
      throw new Error("upstream origin is invalid");
    }
    if (parsed.hostname !== LOOPBACK_HOST && parsed.protocol !== "https:") {
      throw new Error("upstream origin must be loopback http or https");
    }
  }

  let material = null;
  let server = null;
  let forwards = 0;
  let lastLocalLatencyMs = null;
  let boundPort = port;

  const loadSecretOnce = async () => {
    if (material !== null) {
      return material;
    }
    const resolved = await Promise.resolve(getSecret());
    if (typeof resolved !== "string" || resolved.length === 0) {
      throw new Error("broker secret resolution failed");
    }
    material = resolved;
    return material;
  };

  const dropMaterial = () => {
    material = null;
  };

  const bindFacts = () =>
    Object.freeze({
      bind: `${host}:${boundPort}`,
      pi_token: PI_PLACEHOLDER_TOKEN,
      secret_material_written: false,
      retry: 0,
    });

  const handler = async (request, response) => {
    const started = Date.now();
    const fail = (status, message) => {
      if (!response.headersSent) {
        response.statusCode = status;
        response.setHeader("content-type", "application/json");
        response.end(JSON.stringify({ error: message, retry: 0 }));
      }
    };
    try {
      if (request.method === "GET" && request.url === "/health") {
        response.setHeader("content-type", "application/json");
        response.end(
          JSON.stringify({
            ok: true,
            forwards,
            retry: 0,
            last_local_latency_ms: lastLocalLatencyMs,
          }),
        );
        return;
      }
      const authorization = request.headers.authorization ?? "";
      if (authorization !== `Bearer ${PI_PLACEHOLDER_TOKEN}`) {
        fail(401, "placeholder token required");
        return;
      }
      if (upstreamOrigin === undefined) {
        fail(503, "upstream origin is not configured");
        return;
      }
      if (material === null) {
        fail(503, "broker secret is not loaded");
        return;
      }
      const body = await readLimited(request, MAX_REQUEST_BYTES);
      const target = new URL(request.url ?? "/", upstreamOrigin);
      const upstreamHeaders = { authorization: `Bearer ${material}` };
      if (request.headers["content-type"]) {
        upstreamHeaders["content-type"] = request.headers["content-type"];
      }
      const upstream = await fetch(target, {
        method: request.method,
        headers: upstreamHeaders,
        body: request.method === "GET" || request.method === "HEAD" ? undefined : body,
        signal: AbortSignal.timeout(UPSTREAM_TIMEOUT_MS),
        redirect: "error",
      });
      const upstreamBytes = Buffer.from(await upstream.arrayBuffer());
      if (upstreamBytes.length > MAX_RESPONSE_BYTES) {
        fail(502, "response exceeds byte bound");
        return;
      }
      forwards += 1;
      lastLocalLatencyMs = Date.now() - started;
      response.statusCode = upstream.status;
      response.setHeader("content-type", upstream.headers.get("content-type") ?? "application/json");
      response.end(upstreamBytes);
    } catch (error) {
      fail(502, "upstream forward failed");
      void error;
    }
  };

  const broker = {
    kind: "pure-pi-credential-broker",
    host,
    port,
    piPlaceholderToken: PI_PLACEHOLDER_TOKEN,
    threatReview: brokerThreatReview(),
    async bind({ env = process.env, argv = process.argv } = {}) {
      assertSecretFreeProcess({ env, argv });
      await loadSecretOnce();
      return bindFacts();
    },
    async listen({ env = process.env, argv = process.argv } = {}) {
      assertSecretFreeProcess({ env, argv });
      await loadSecretOnce();
      if (server !== null) {
        throw new Error("broker already listening");
      }
      server = http.createServer((request, response) => {
        void handler(request, response);
      });
      await new Promise((resolve, reject) => {
        server.once("error", reject);
        server.listen(port, host, () => {
          const address = server.address();
          boundPort = typeof address === "object" && address !== null ? address.port : port;
          resolve();
        });
      });
      return bindFacts();
    },
    async close() {
      dropMaterial();
      if (server === null) {
        return;
      }
      const closing = server;
      server = null;
      await new Promise((resolve, reject) => {
        closing.close((error) => (error ? reject(error) : resolve()));
      });
    },
    stats() {
      return Object.freeze({
        forwards,
        retry: 0,
        last_local_latency_ms: lastLocalLatencyMs,
      });
    },
    hasRetainedSecretMaterial() {
      return false;
    },
  };

  for (const name of FORBIDDEN_CAPABILITIES) {
    if (name in broker) {
      throw new Error(`broker must not expose ${name}`);
    }
  }

  return Object.freeze(broker);
}
