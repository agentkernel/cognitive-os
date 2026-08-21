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
 * --revision --dsh-root --adapter-root --task
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
if (!Number.isInteger(port) || port < 1 || !bootstrapPath || !dshRoot || !adapterRoot) {
  throw new Error("--port --bootstrap-file --dsh-root --adapter-root are required");
}
const bootstrap = readFileSync(bootstrapPath, "utf8").trim();
const pluginHref = pathToFileURL(join(adapterRoot, "dist", "plugin.js")).href;
const scriptDir = dirname(fileURLToPath(import.meta.url));
const work = join(tmpdir(), `p8t09-dsh-real-${process.pid}`);
mkdirSync(work, { mode: 0o700, recursive: true });
const bearerFile = join(work, "daemon.bearer");
const patchFile = join(work, "headless-akp.yml");
const dshHome = join(work, "dsh-home");
mkdirSync(dshHome, { mode: 0o700 });

function redact(error) {
  return String(error).replace(/Bearer\s+\S+/gi, "Bearer [redacted]").replace(/sk-[A-Za-z0-9]+/g, "sk-[redacted]");
}

const taskToken = await issueToken(origin, bootstrap, "task");
const managementToken = await issueToken(origin, bootstrap, "management");
const stamp = `${process.pid}`;
const writeTarget = `p8-t09-write-${stamp}.txt`;
const readSpec = {
  family: "read",
  tool: "native.workspace.read",
  conversation: "conversation://personal/p8-t09",
  objective: "read README.md",
  taskRef: `task://personal/p8-t09-dsh-read-${stamp}`,
};
const searchSpec = {
  family: "search",
  tool: "native.workspace.search",
  conversation: "conversation://personal/p8-t09-search",
  objective: "search the workspace for needle",
  taskRef: `task://personal/p8-t09-dsh-search-${stamp}`,
};
const writeSpec = {
  family: "write",
  tool: "native.workspace.write",
  conversation: "conversation://personal/p8-t09-write",
  objective: "mutate workspace through daemon-governed WorkspaceWrite",
  taskRef: `task://personal/p8-t09-dsh-write-${stamp}`,
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

writeFileSync(
  patchFile,
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
          input_b64: Buffer.from("p8-t09 disposable write\n", "utf8").toString("base64"),
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

const selected = await httpJson(origin, "GET", "/provider/v1/selected-model", managementToken);
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
sseBridge.child.kill("SIGTERM");
const elapsedMs = Date.now() - started;
const assistant = stdout.trim().split(/\r?\n/).filter((line) => line && !line.startsWith("$")).at(-1) ?? "";
const [readLife, searchLife, writeLife] = await Promise.all([
  waitLifecycle(origin, taskToken, readSpec.taskRef, { want: "COMPLETED" }),
  waitLifecycle(origin, taskToken, searchSpec.taskRef, { want: "COMPLETED" }),
  waitLifecycle(origin, taskToken, writeSpec.taskRef, { want: "COMPLETED" }),
]);

rmSync(work, { recursive: true, force: true });

const summary = {
  revision_pin: revisionPin ?? null,
  guest_port: port,
  adapter: "dsh --patch cognitiveos-akp Workspace* + llm-deepseek via SSE-to-unary daemon provider proxy",
  candidate_only: true,
  dsh_response_is_not_task_completion: true,
  selected_model: selected.json?.model ?? selected.json?.selected_model ?? null,
  dsh_exit: exitCode,
  elapsed_ms: elapsedMs,
  assistant_preview_bytes: Buffer.byteLength(assistant, "utf8"),
  assistant_is_pong: /^pong\.?$/i.test(assistant),
  stderr_redacted_bytes: Buffer.byteLength(redact(stderr), "utf8"),
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
process.exit(exitCode === 0 && summary.assistant_is_pong && workspacePass ? 0 : 1);
