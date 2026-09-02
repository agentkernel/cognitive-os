#!/usr/bin/env node
/**
 * Hidden hosted DSH Attempt child (P13-T02).
 *
 * Spawned only by the Personal daemon's stdio broker (`cognitive-runtime`
 * `hosted_dsh_broker`). Reads exactly one `request` frame from stdin, runs the
 * pinned headless dsh checkout with the bounded Context as its task, and
 * writes newline-delimited JSON frames to stdout:
 *
 *   {"frame":"observation", ...}   redacted progress / dsh stdout lines
 *   {"frame":"heartbeat"}          liveness only — never authority
 *   {"frame":"candidate", "operation":"DeliverableDraft", "payload":{...}}
 *   {"frame":"response", "status":"done"|"failed", ...}
 *
 * Nothing here is Task completion: the daemon records every frame as an
 * observation and the independent verifier decides later (P13-T04). Argv
 * carries paths and the pin only (`--dsh-root --adapter-root --revision
 * --provider-path b`). The daemon management bearer is minted from the
 * bootstrap file *path* named in the request frame, written to a 0600
 * credentials file inside a private disposable DSH_HOME, and removed on exit.
 * Provider traffic goes to the daemon proxy (`/provider/v1/dsh`) only; a
 * direct Provider path, an API key file, or a native MCP flag is refused
 * before anything is spawned.
 *
 * Exit classes: 0 dsh exited 0 · 2 protocol/argv refusal · 3 pin mismatch ·
 * 4 dsh CLI unavailable · 5 daemon unavailable · 6 dsh failed / timed out.
 */
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { chmodSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { httpJson, issueToken } from "./daemon-task.mjs";
import {
  PROBE_COMPLETION_BUDGET_TOKENS,
  llmDeepseekPatchLines,
  pathBWebCatalogModels,
  pathBWebChildExtras,
  pathBWebCredentialsYaml,
} from "./dsh-web-preflight.mjs";

export const HOSTED_FRAME_PROTOCOL = "cognitiveos.personal.hosted-dsh-stdio/0.1";
export const HOSTED_DSH_REVISION_PIN = "528c682e061696f5a160f363f236ecbf53cbd006";
const REVISION_FILE_NAME = ".cognitiveos-dsh-revision";
const MAX_OBSERVATION_LINES = 400;
const MAX_FRAME_TEXT_CHARS = 1024;
const MAX_CANDIDATE_TEXT_CHARS = 8192;
const HEARTBEAT_MS = 5000;
const DEFAULT_TIMEOUT_MS = 120000;
const REFUSED_FLAGS = ["--api-key-file", "--direct-base-url", "--mcp", "--native-mcp", "--base-tool", "--hmr", "--home-patch"];

function emit(frame) {
  process.stdout.write(`${JSON.stringify(frame)}\n`);
}

export function redact(text) {
  return String(text ?? "")
    .replace(/Bearer\s+\S+/gi, "Bearer [redacted]")
    .replace(/sk-[A-Za-z0-9._-]+/g, "sk-[redacted]")
    .replace(/sess-[A-Za-z0-9._-]+/g, "sess-[redacted]")
    .replace(/boot-[A-Za-z0-9._-]+/g, "boot-[redacted]")
    .replace(/ssv1:\S+/g, "ssv1:[redacted]");
}

function bounded(text, max) {
  const redacted = redact(text);
  return redacted.length > max ? redacted.slice(0, max) : redacted;
}

function arg(name) {
  const index = process.argv.indexOf(name);
  if (index >= 0 && process.argv[index + 1] !== undefined) return process.argv[index + 1];
  return undefined;
}

function fail(status, reason, exitCode, extra = {}) {
  emit({ frame: "response", status, reason, ...extra, completion_claimed: false });
  process.exit(exitCode);
}

function readRequestLine() {
  return new Promise((resolve) => {
    let data = "";
    process.stdin.setEncoding("utf8");
    process.stdin.on("data", (chunk) => {
      data += chunk;
      if (data.includes("\n")) {
        resolve(data.split("\n")[0]);
      }
    });
    process.stdin.on("end", () => resolve(data.split("\n")[0]));
    process.stdin.on("error", () => resolve(""));
  });
}

export function parseRequest(line) {
  let request;
  try {
    request = JSON.parse(line);
  } catch {
    return { error: "request frame is not JSON" };
  }
  if (!request || request.frame !== "request") return { error: "first frame must be request" };
  if (request.protocol !== HOSTED_FRAME_PROTOCOL) return { error: "unknown frame protocol" };
  if (typeof request.context !== "string" || !request.context.trim()) return { error: "context required" };
  if (typeof request.attempt_id !== "string" || !request.attempt_id) return { error: "attempt_id required" };
  const digest = createHash("sha256").update(request.context, "utf8").digest("hex");
  if (request.context_digest && request.context_digest !== digest) return { error: "context digest mismatch" };
  if (request.provider_direct === true) return { error: "direct Provider is refused" };
  return { request, digest };
}

export function isLoopbackOrigin(origin) {
  try {
    const url = new URL(origin);
    const host = url.hostname.replace(/^\[|\]$/g, "");
    return (url.protocol === "http:" || url.protocol === "https:") && (host === "127.0.0.1" || host === "localhost" || host === "::1");
  } catch {
    return false;
  }
}

function dshCliInvocation(root) {
  const compiled = join(root, "apps/cli/lib/bin.js");
  const compiledHostGraph = join(root, "packages/api/gateway/lib/index.js");
  if (existsSync(compiled) && existsSync(compiledHostGraph)) {
    return { args: [compiled], mode: "compiled-lib" };
  }
  const source = join(root, "apps/cli/src/bin.ts");
  if (existsSync(source)) {
    return { args: ["--import", "tsx/esm", source], mode: "tsx-source" };
  }
  return null;
}

function childEnvironment(dshRoot, dshHome, extra) {
  const allow = ["PATH", "HOME", "USER", "LOGNAME", "SHELL", "LANG", "LC_ALL", "TZ", "TMPDIR", "TMP", "TEMP", "PNPM_HOME", "SystemRoot", "WINDIR", "ComSpec", "PATHEXT", "USERPROFILE", "APPDATA", "LOCALAPPDATA"];
  const env = {};
  for (const key of allow) {
    const value = process.env[key];
    if (value) env[key] = value;
  }
  env.DSH_HOME = dshHome;
  env.DSH_TELEMETRY_MODE = "DISABLED";
  env.DSH_PERMISSION_MODE = "read-only";
  const compileCache = join(dshRoot, ".cognitiveos-node-compile-cache");
  try {
    mkdirSync(compileCache, { mode: 0o700, recursive: true });
    env.NODE_COMPILE_CACHE = compileCache;
  } catch {
    // A read-only checkout simply runs without the compile cache.
  }
  for (const [key, value] of Object.entries(extra)) {
    if (/API_KEY|SECRET|TOKEN|PASSWORD|BEARER/i.test(key)) {
      throw new Error(`child environment refuses secret-shaped key ${key}`);
    }
    env[key] = value;
  }
  return env;
}

async function boundModelOverlay(origin, managementToken) {
  try {
    const bindings = await httpJson(origin, "GET", "/management/agent-bindings", managementToken);
    const rows = Array.isArray(bindings.json?.bindings) ? bindings.json.bindings : [];
    const dsh = rows.find((row) => row && row.agent === "agent://personal/dsh" && row.status === "active");
    if (!dsh?.account_id) return { bound: false, model: "", catalog: [] };
    const listed = await httpJson(
      origin,
      "GET",
      `/management/providers/models?account_id=${encodeURIComponent(dsh.account_id)}`,
      managementToken,
    );
    const model = String(dsh.model_id ?? "").trim();
    const catalog = Array.isArray(listed.json?.models) ? listed.json.models : [];
    return { bound: true, model, catalog: pathBWebCatalogModels(catalog, model) };
  } catch {
    return { bound: false, model: "", catalog: [] };
  }
}

async function main() {
  for (const flag of REFUSED_FLAGS) {
    if (process.argv.includes(flag)) fail("failed", `refused flag ${flag}`, 2);
  }
  const dshRoot = arg("--dsh-root");
  const adapterRoot = arg("--adapter-root");
  const revision = arg("--revision");
  const providerPath = arg("--provider-path") ?? "b";
  if (providerPath !== "b") fail("failed", "only Path B (daemon Provider proxy) is allowed", 2);
  if (!dshRoot || !adapterRoot) fail("failed", "--dsh-root and --adapter-root are required", 2);
  if (revision !== HOSTED_DSH_REVISION_PIN) fail("failed", "pin-mismatch: --revision is not the product pin", 3);

  const line = await readRequestLine();
  const parsed = parseRequest(line);
  if (parsed.error) fail("failed", `protocol: ${parsed.error}`, 2);
  const { request, digest } = parsed;
  const timeoutMs = Number.isSafeInteger(request.timeout_ms) && request.timeout_ms > 0 ? request.timeout_ms : DEFAULT_TIMEOUT_MS;
  emit({
    frame: "observation",
    text: "child.started",
    attempt_id: request.attempt_id,
    context_digest: digest,
    context_bytes: Buffer.byteLength(request.context, "utf8"),
    provider_path: "b",
    completion_authority: "daemon",
  });

  let pinned = "";
  try {
    pinned = readFileSync(join(dshRoot, REVISION_FILE_NAME), "utf8").trim();
  } catch {
    fail("failed", "pin-mismatch: dsh revision pin file is absent", 3);
  }
  if (pinned !== HOSTED_DSH_REVISION_PIN) fail("failed", "pin-mismatch: dsh revision pin file is not the product pin", 3);

  const cli = dshCliInvocation(dshRoot);
  if (!cli) {
    emit({ frame: "observation", text: "dsh.cli.missing", dsh_root_present: existsSync(dshRoot) });
    fail("failed", "dsh-cli-missing", 4);
  }
  emit({ frame: "observation", text: `dsh.cli.${cli.mode}` });

  const origin = request.daemon_origin;
  const bootstrapPath = request.bootstrap_file;
  if (!origin || !isLoopbackOrigin(origin)) fail("failed", "daemon-unavailable: daemon_origin must be loopback", 5);
  if (!bootstrapPath || !existsSync(bootstrapPath)) fail("failed", "daemon-unavailable: bootstrap file is absent", 5);

  const work = join(tmpdir(), `p13t02-hosted-attempt-${process.pid}`);
  mkdirSync(work, { mode: 0o700, recursive: true });
  const dshHome = join(work, "dsh-home");
  mkdirSync(dshHome, { mode: 0o700, recursive: true });
  const patchFile = join(work, "hosted-attempt.yml");
  const cleanup = () => {
    rmSync(work, { recursive: true, force: true });
  };

  let managementToken;
  try {
    const bootstrap = readFileSync(bootstrapPath, "utf8").trim();
    managementToken = await issueToken(origin, bootstrap, "management");
  } catch {
    cleanup();
    fail("failed", "daemon-unavailable: management session could not be minted", 5);
  }
  writeFileSync(join(dshHome, ".credentials.yaml"), pathBWebCredentialsYaml(managementToken), {
    encoding: "utf8",
    mode: 0o600,
  });
  chmodSync(join(dshHome, ".credentials.yaml"), 0o600);
  const providerBase = `${origin.replace(/\/$/, "")}/provider/v1/dsh`;
  const overlay = await boundModelOverlay(origin, managementToken);
  emit({ frame: "observation", text: "provider.proxy.bound", bound: overlay.bound, model: overlay.model || null });
  writeFileSync(
    patchFile,
    llmDeepseekPatchLines(providerBase, "DAEMON_BEARER", overlay.model, overlay.catalog, PROBE_COMPLETION_BUDGET_TOKENS),
    { encoding: "utf8", mode: 0o600 },
  );

  const started = Date.now();
  const child = spawn(process.execPath, [...cli.args, "--profile", "headless", "--patch", patchFile, request.context], {
    cwd: dshRoot,
    env: childEnvironment(dshRoot, dshHome, pathBWebChildExtras(providerBase)),
    stdio: ["ignore", "pipe", "pipe"],
  });
  emit({ frame: "observation", text: "dsh.spawned", dsh_pid: child.pid ?? null });

  let stdoutBuffer = "";
  const assistantLines = [];
  let observationLines = 0;
  let stderrTail = "";
  let timedOut = false;
  const killDsh = () => {
    if (child.exitCode === null && child.signalCode === null) {
      child.kill("SIGTERM");
      setTimeout(() => {
        if (child.exitCode === null && child.signalCode === null) child.kill("SIGKILL");
      }, 2000).unref();
    }
  };
  const heartbeat = setInterval(() => emit({ frame: "heartbeat", elapsed_ms: Date.now() - started }), HEARTBEAT_MS);
  const budgetMs = Math.max(1000, timeoutMs - 3000);
  const deadline = setTimeout(() => {
    timedOut = true;
    killDsh();
  }, budgetMs);
  process.on("SIGTERM", killDsh);
  process.on("SIGINT", killDsh);

  child.stdout.setEncoding("utf8");
  child.stderr.setEncoding("utf8");
  child.stdout.on("data", (chunk) => {
    stdoutBuffer += chunk;
    let index = stdoutBuffer.indexOf("\n");
    while (index >= 0) {
      const raw = stdoutBuffer.slice(0, index);
      stdoutBuffer = stdoutBuffer.slice(index + 1);
      const text = raw.trimEnd();
      if (text && !text.startsWith("$")) assistantLines.push(text);
      if (text && observationLines < MAX_OBSERVATION_LINES) {
        observationLines += 1;
        emit({ frame: "observation", text: bounded(text, MAX_FRAME_TEXT_CHARS) });
      }
      index = stdoutBuffer.indexOf("\n");
    }
  });
  child.stderr.on("data", (chunk) => {
    stderrTail = `${stderrTail}${chunk}`.slice(-4096);
  });
  const exitCode = await new Promise((resolve) => {
    child.on("error", () => resolve(4));
    child.on("close", (code) => resolve(code ?? 6));
  });
  clearInterval(heartbeat);
  clearTimeout(deadline);
  if (stdoutBuffer.trim() && !stdoutBuffer.trim().startsWith("$")) assistantLines.push(stdoutBuffer.trim());
  cleanup();

  const assistant = assistantLines.length ? assistantLines[assistantLines.length - 1] : "";
  const elapsedMs = Date.now() - started;
  if (assistant) {
    emit({
      frame: "candidate",
      operation: "DeliverableDraft",
      payload: {
        text: bounded(assistant, MAX_CANDIDATE_TEXT_CHARS),
        attempt_id: request.attempt_id,
        task_ref: request.task_ref,
        context_digest: digest,
        dsh_exit: exitCode,
      },
      text: bounded(assistant, MAX_FRAME_TEXT_CHARS),
    });
  }
  const status = !timedOut && exitCode === 0 && assistant ? "done" : "failed";
  emit({
    frame: "response",
    status,
    reason: timedOut ? "timed-out" : exitCode === 0 ? (assistant ? "dsh-exited-0" : "dsh-produced-no-text") : `dsh-exit-${exitCode}`,
    dsh_exit: exitCode,
    elapsed_ms: elapsedMs,
    stderr_tail: bounded(stderrTail.split(/\r?\n/).slice(-20).join("\n"), 2048),
    completion_claimed: false,
  });
  process.exit(timedOut ? 6 : exitCode === 0 ? 0 : 6);
}

const invokedDirectly = process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (invokedDirectly) {
  main().catch((error) => {
    emit({ frame: "response", status: "failed", reason: `child-error: ${bounded(error?.message ?? error, 256)}`, completion_claimed: false });
    process.exit(6);
  });
}
