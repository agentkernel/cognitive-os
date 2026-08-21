#!/usr/bin/env node
/**
 * Drive pinned dsh as a real process (P8-T09 Path B LLM + Cordis plugin).
 *
 * dsh --profile headless --patch <generated yaml> loads ./plugin.js, activates
 * POST /task/akp/dsh, and points llm-deepseek at POST /provider/v1/chat/completions.
 * The daemon management token is read from a 0600 file. The Provider key stays
 * in SecretStore on the daemon host. A dsh response is never Task completion.
 *
 * Argv only (no CognitiveOS env-var literals): --port --bootstrap-file
 * --revision --dsh-root --adapter-root --task
 */
import { spawn } from "node:child_process";
import { chmodSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

function arg(name, fallback) {
  const index = process.argv.indexOf(name);
  if (index >= 0 && process.argv[index + 1]) return process.argv[index + 1];
  return fallback;
}

const port = Number(arg("--port", "48509"));
const bootstrapPath = arg("--bootstrap-file");
const revisionPin = arg("--revision");
const dshRoot = arg("--dsh-root");
const adapterRoot = arg("--adapter-root");
const origin = arg("--origin", `http://127.0.0.1:${port}`);
const task = arg("--task", "Reply with the single word pong and nothing else.");
if (!Number.isInteger(port) || port < 1 || !bootstrapPath || !dshRoot || !adapterRoot) {
  throw new Error("--port --bootstrap-file --dsh-root --adapter-root are required");
}
const bootstrap = readFileSync(bootstrapPath, "utf8").trim();
const pluginHref = pathToFileURL(join(adapterRoot, "dist", "plugin.js")).href;
const work = join(tmpdir(), `p8t09-dsh-real-${process.pid}`);
mkdirSync(work, { mode: 0o700, recursive: true });
const bearerFile = join(work, "daemon.bearer");
const patchFile = join(work, "headless-akp.yml");
const dshHome = join(work, "dsh-home");
mkdirSync(dshHome, { mode: 0o700 });

function redact(error) {
  return String(error).replace(/Bearer\s+\S+/gi, "Bearer [redacted]").replace(/sk-[A-Za-z0-9]+/g, "sk-[redacted]");
}

async function httpJson(method, path, token, body) {
  const headers = {};
  if (token) headers.authorization = `Bearer ${token}`;
  const init = { method, headers };
  if (body !== undefined) {
    headers["content-type"] = "application/json";
    init.body = JSON.stringify(body);
  }
  const response = await fetch(`${origin}${path}`, init);
  const text = await response.text();
  let json;
  try {
    json = JSON.parse(text);
  } catch {
    json = { parse_error: true, http_status: response.status };
  }
  return { status: response.status, json };
}

async function issueToken(channel) {
  const { json } = await httpJson("POST", "/local/session", "", {
    channel,
    principal_id: "principal://local/owner",
    bootstrap_secret: bootstrap,
  });
  const token = json.token;
  if (typeof token !== "string" || !token) {
    throw new Error(`session token missing for ${channel}`);
  }
  return token;
}

const taskToken = await issueToken("task");
const managementToken = await issueToken("management");

writeFileSync(bearerFile, `${taskToken}\n`, { encoding: "utf8", mode: 0o600 });
chmodSync(bearerFile, 0o600);
writeFileSync(
  join(dshHome, ".credentials.yaml"),
  `version: 1\n\nrefs:\n  DAEMON_BEARER: ${managementToken}\n`,
  { encoding: "utf8", mode: 0o600 },
);
chmodSync(join(dshHome, ".credentials.yaml"), 0o600);

writeFileSync(
  patchFile,
  [
    "- insert:",
    "  - id: cognitiveos-akp",
    `    name: "${pluginHref}"`,
    "    config:",
    `      endpoint: ${origin}/task/akp/dsh`,
    `      bearerFile: "${bearerFile}"`,
    "      sessionId: dsh-real-process",
    "      timeoutMs: 20000",
    "      startupEvents:",
    "        - kind: lifecycle",
    "          operation: adapter.ready",
    "          payload:",
    "            ok: true",
    "- id: llm-deepseek",
    "  config:",
    `    baseURL: ${origin}/provider/v1`,
    "    apiKeyEnv: DAEMON_BEARER",
    "",
  ].join("\n"),
  { encoding: "utf8", mode: 0o600 },
);

function childEnvironment() {
  const allow = [
    "PATH",
    "HOME",
    "USER",
    "LOGNAME",
    "SHELL",
    "LANG",
    "LC_ALL",
    "TZ",
    "TMPDIR",
    "TMP",
    "TEMP",
    "PNPM_HOME",
    "http_proxy",
    "https_proxy",
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "no_proxy",
    "NO_PROXY",
  ];
  const env = {};
  for (const key of allow) {
    if (process.env[key]) env[key] = process.env[key];
  }
  env.DSH_HOME = dshHome;
  env.DSH_TELEMETRY_MODE = "DISABLED";
  env.DSH_PERMISSION_MODE = "read-only";
  return env;
}

const selected = await httpJson("GET", "/provider/v1/selected-model", managementToken);
const started = Date.now();
const child = spawn(
  "pnpm",
  ["dsh", "--profile", "headless", "--patch", patchFile, task],
  {
    cwd: dshRoot,
    env: childEnvironment(),
    stdio: ["ignore", "pipe", "pipe"],
  },
);

let stdout = "";
let stderr = "";
child.stdout.setEncoding("utf8");
child.stderr.setEncoding("utf8");
child.stdout.on("data", (chunk) => {
  stdout += chunk;
});
child.stderr.on("data", (chunk) => {
  stderr += chunk;
});

const exitCode = await new Promise((resolve) => {
  child.on("close", resolve);
});
const elapsedMs = Date.now() - started;
const assistant = stdout.trim().split(/\r?\n/).filter((line) => line && !line.startsWith("$")).at(-1) ?? "";

rmSync(work, { recursive: true, force: true });

const summary = {
  revision_pin: revisionPin ?? null,
  guest_port: port,
  adapter: "dsh --patch cognitiveos-akp + llm-deepseek via daemon provider proxy",
  candidate_only: true,
  dsh_response_is_not_task_completion: true,
  selected_model: selected.json?.model ?? selected.json?.selected_model ?? null,
  dsh_exit: exitCode,
  elapsed_ms: elapsedMs,
  assistant_preview_bytes: Buffer.byteLength(assistant, "utf8"),
  assistant_is_pong: /^pong\.?$/i.test(assistant),
  stderr_redacted_bytes: Buffer.byteLength(redact(stderr), "utf8"),
  non_claims: ["Gate", "release", "Profile", "B01", "Agent-benefit"],
};
process.stdout.write(`${JSON.stringify(summary)}\n`);
process.exit(exitCode === 0 && summary.assistant_is_pong ? 0 : 1);
