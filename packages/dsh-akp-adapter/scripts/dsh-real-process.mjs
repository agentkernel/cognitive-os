#!/usr/bin/env node
/**
 * Drive pinned dsh as a real process (P8-T09 Path B LLM + Workspace* plugin events).
 *
 * dsh --profile headless --patch <generated yaml> loads ./plugin.js, activates
 * POST /task/akp/dsh, submits admitted Workspace* candidates as startupEvents,
 * and points llm-deepseek at POST /provider/v1/chat/completions.
 * The daemon management token is read from a 0600 file. The Provider key stays
 * in SecretStore on the daemon host. A dsh response is never Task completion.
 *
 * Argv only (no CognitiveOS env-var literals): --port --bootstrap-file
 * --revision --dsh-root --adapter-root --task --provider-path a|b
 * Path A also requires --api-key-file (0600 or "-") and never logs the key.
 */
import { spawn } from "node:child_process";
import { chmodSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { admitTask, httpJson, issueToken, waitLifecycle } from "./daemon-task.mjs";

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
const providerPath = arg("--provider-path", "b");
const apiKeyFile = arg("--api-key-file");
const directBaseUrl = arg("--direct-base-url", "https://api.deepseek.com");
if (providerPath !== "a" && providerPath !== "b") {
  throw new Error("--provider-path must be a (direct Flash) or b (AKP/daemon)");
}
if (!dshRoot || !adapterRoot) {
  throw new Error("--dsh-root and --adapter-root are required");
}
if (providerPath === "b" && (!Number.isInteger(port) || port < 1 || !bootstrapPath)) {
  throw new Error("Path B requires --port and --bootstrap-file");
}
if (providerPath === "a" && !apiKeyFile) {
  throw new Error("Path A requires --api-key-file <0600-path|->");
}
const pluginHref = pathToFileURL(join(adapterRoot, "dist", "plugin.js")).href;
const scriptDir = dirname(fileURLToPath(import.meta.url));
const work = join(tmpdir(), `p8t10-dsh-real-${process.pid}`);
mkdirSync(work, { mode: 0o700, recursive: true });
const bearerFile = join(work, "daemon.bearer");
const patchFile = join(work, "headless-akp.yml");
const dshHome = join(work, "dsh-home");
mkdirSync(dshHome, { mode: 0o700 });

function redact(error) {
  return String(error).replace(/Bearer\s+\S+/gi, "Bearer [redacted]").replace(/sk-[A-Za-z0-9]+/g, "sk-[redacted]");
}

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

async function runDsh(patchBody) {
  writeFileSync(patchFile, patchBody, { encoding: "utf8", mode: 0o600 });
  const started = Date.now();
  const ttftHolder = { ms: null };
  // Invoke the pinned CLI entry with Node. `pnpm dsh` is not portable on an
  // installed guest: pnpm 11's deps-status check requires git, which Personal
  // linux-002 does not ship, and copied node_modules are not a pnpm workspace
  // root. `node --import tsx/esm apps/cli/src/bin.ts` is the same script the
  // dsh package.json `dsh` entry runs.
  const child = spawn(
    process.execPath,
    ["--import", "tsx/esm", join(dshRoot, "apps/cli/src/bin.ts"), "--profile", "headless", "--patch", patchFile, task],
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
    if (ttftHolder.ms === null && String(chunk).trim()) {
      ttftHolder.ms = Date.now() - started;
    }
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
  return {
    exitCode,
    elapsedMs,
    ttftMs: ttftHolder.ms,
    assistant,
    stderrRedactedBytes: Buffer.byteLength(redact(stderr), "utf8"),
  };
}

if (providerPath === "a") {
  const keySource = apiKeyFile === "-" ? 0 : apiKeyFile;
  const apiKey = readFileSync(keySource, "utf8").trim();
  writeFileSync(
    join(dshHome, ".credentials.yaml"),
    `version: 1\n\nrefs:\n  DEEPSEEK_KEY: ${apiKey}\n`,
    { encoding: "utf8", mode: 0o600 },
  );
  chmodSync(join(dshHome, ".credentials.yaml"), 0o600);
  const outcome = await runDsh(
    ["- id: llm-deepseek", "  config:", `    baseURL: ${directBaseUrl}`, "    apiKeyEnv: DEEPSEEK_KEY", ""].join("\n"),
  );
  rmSync(work, { recursive: true, force: true });
  const summary = {
    revision_pin: revisionPin ?? null,
    provider_path: "a",
    adapter: "dsh --patch llm-deepseek direct Flash (no AKP, no daemon Provider proxy)",
    candidate_only: true,
    dsh_response_is_not_task_completion: true,
    dsh_exit: outcome.exitCode,
    elapsed_ms: outcome.elapsedMs,
    ttft_ms: outcome.ttftMs,
    assistant_preview_bytes: Buffer.byteLength(outcome.assistant, "utf8"),
    assistant_is_pong: /^pong\.?$/i.test(outcome.assistant),
    stderr_redacted_bytes: outcome.stderrRedactedBytes,
    workspace: null,
    non_claims: ["Gate", "release", "Profile", "B01", "Agent-benefit"],
  };
  process.stdout.write(`${JSON.stringify(summary)}\n`);
  process.exit(outcome.exitCode === 0 && summary.assistant_is_pong ? 0 : 1);
}

const bootstrap = readFileSync(bootstrapPath, "utf8").trim();
const taskToken = await issueToken(origin, bootstrap, "task");
const managementToken = await issueToken(origin, bootstrap, "management");
const stamp = `${process.pid}`;
const writeTarget = `p8-t10-write-${stamp}.txt`;
const readSpec = {
  family: "read",
  tool: "native.workspace.read",
  conversation: "conversation://personal/p8-t10",
  objective: "read README.md",
  taskRef: `task://personal/p8-t10-dsh-read-${stamp}`,
};
const searchSpec = {
  family: "search",
  tool: "native.workspace.search",
  conversation: "conversation://personal/p8-t10-search",
  objective: "search the workspace for needle",
  taskRef: `task://personal/p8-t10-dsh-search-${stamp}`,
};
const writeSpec = {
  family: "write",
  tool: "native.workspace.write",
  conversation: "conversation://personal/p8-t10-write",
  objective: "mutate workspace through daemon-governed WorkspaceWrite",
  taskRef: `task://personal/p8-t10-dsh-write-${stamp}`,
};
await admitTask(origin, taskToken, readSpec);
await admitTask(origin, taskToken, searchSpec);
await admitTask(origin, taskToken, writeSpec);

writeFileSync(bearerFile, `${taskToken}\n`, { encoding: "utf8", mode: 0o600 });
chmodSync(bearerFile, 0o600);
writeFileSync(
  join(dshHome, ".credentials.yaml"),
  `version: 1\n\nrefs:\n  DAEMON_BEARER: ${managementToken}\n`,
  { encoding: "utf8", mode: 0o600 },
);
chmodSync(join(dshHome, ".credentials.yaml"), 0o600);

function startSseBridge(upstream) {
  const child = spawn(
    process.execPath,
    [join(scriptDir, "provider-sse-bridge.mjs"), "--listen", "127.0.0.1:0", "--upstream", upstream],
    { stdio: ["ignore", "pipe", "pipe"] },
  );
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error("sse bridge listen timeout")), 5000);
    const onExit = (code) => {
      clearTimeout(timer);
      reject(new Error(`sse bridge exited ${code}`));
    };
    child.stdout.setEncoding("utf8");
    child.stdout.on("data", (chunk) => {
      const match = String(chunk).match(/listening ([^\s]+):(\d+)/);
      if (match) {
        clearTimeout(timer);
        child.off("exit", onExit);
        resolve({ child, origin: `http://127.0.0.1:${match[2]}` });
      }
    });
    child.once("exit", onExit);
  });
}

const sseBridge = await startSseBridge(`${origin}/provider/v1`);
const providerBase = `${sseBridge.origin}/provider/v1`;

function pluginInsert(id, sessionId, pluginId, taskRef, events) {
  const lines = [
    `  - id: ${id}`,
    `    name: "${pluginHref}"`,
    "    config:",
    `      endpoint: ${origin}/task/akp/dsh`,
    `      bearerFile: "${bearerFile}"`,
    `      sessionId: ${sessionId}`,
    `      pluginId: ${pluginId}`,
    "      timeoutMs: 20000",
  ];
  if (taskRef) {
    lines.push(`      taskRef: "${taskRef}"`);
  }
  lines.push("      startupEvents:");
  for (const event of events) {
    lines.push(`        - kind: ${event.kind}`);
    lines.push(`          operation: ${event.operation}`);
    lines.push("          payload:");
    for (const [key, value] of Object.entries(event.payload)) {
      lines.push(`            ${key}: ${JSON.stringify(value)}`);
    }
  }
  return lines;
}

const selected = await httpJson(origin, "GET", "/provider/v1/selected-model", managementToken);
const outcome = await runDsh(
  [
    "- insert:",
    ...pluginInsert("cognitiveos-akp", "dsh-real-process", "deepseek.dsh.akp", undefined, [
      { kind: "lifecycle", operation: "adapter.ready", payload: { ok: true } },
    ]),
    ...pluginInsert("cognitiveos-akp-read", `dsh-session-read-${stamp}`, "deepseek.dsh.akp.read", readSpec.taskRef, [
      { kind: "candidate", operation: "WorkspaceRead", payload: { target: "README.md" } },
    ]),
    ...pluginInsert("cognitiveos-akp-search", `dsh-session-search-${stamp}`, "deepseek.dsh.akp.search", searchSpec.taskRef, [
      { kind: "candidate", operation: "WorkspaceSearch", payload: { query: "needle" } },
    ]),
    ...pluginInsert("cognitiveos-akp-write", `dsh-session-write-${stamp}`, "deepseek.dsh.akp.write", writeSpec.taskRef, [
      {
        kind: "candidate",
        operation: "WorkspaceWrite",
        payload: {
          target: writeTarget,
          input_b64: Buffer.from("p8-t10 disposable write\n", "utf8").toString("base64"),
          preimage: "absent",
        },
      },
    ]),
    "- id: llm-deepseek",
    "  config:",
    `    baseURL: ${providerBase}`,
    "    apiKeyEnv: DAEMON_BEARER",
    "",
  ].join("\n"),
);
sseBridge.child.kill("SIGTERM");
const [readLife, searchLife, writeLife] = await Promise.all([
  waitLifecycle(origin, taskToken, readSpec.taskRef, { want: "COMPLETED" }),
  waitLifecycle(origin, taskToken, searchSpec.taskRef, { want: "COMPLETED" }),
  waitLifecycle(origin, taskToken, writeSpec.taskRef, { want: "COMPLETED" }),
]);

rmSync(work, { recursive: true, force: true });

const summary = {
  revision_pin: revisionPin ?? null,
  guest_port: port,
  provider_path: "b",
  adapter: "dsh --patch cognitiveos-akp Workspace* + llm-deepseek via SSE-to-unary daemon provider proxy",
  candidate_only: true,
  dsh_response_is_not_task_completion: true,
  selected_model: selected.json?.model ?? selected.json?.selected_model ?? null,
  dsh_exit: outcome.exitCode,
  elapsed_ms: outcome.elapsedMs,
  ttft_ms: outcome.ttftMs,
  assistant_preview_bytes: Buffer.byteLength(outcome.assistant, "utf8"),
  assistant_is_pong: /^pong\.?$/i.test(outcome.assistant),
  stderr_redacted_bytes: outcome.stderrRedactedBytes,
  workspace: {
    read: { taskRef: readSpec.taskRef, lifecycle: readLife },
    search: { taskRef: searchSpec.taskRef, lifecycle: searchLife },
    write: { taskRef: writeSpec.taskRef, target: writeTarget, lifecycle: writeLife },
  },
  non_claims: ["Gate", "release", "Profile", "B01", "Agent-benefit"],
};
process.stdout.write(`${JSON.stringify(summary)}\n`);
const workspacePass =
  readLife === "COMPLETED" && searchLife === "COMPLETED" && writeLife === "COMPLETED";
process.exit(outcome.exitCode === 0 && summary.assistant_is_pong && workspacePass ? 0 : 1);
