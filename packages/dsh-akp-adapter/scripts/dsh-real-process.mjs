#!/usr/bin/env node
/**
 * Drive pinned dsh as a real process (P8-T09 Path B LLM + Workspace* plugin events).
 *
 * dsh --profile headless --patch <generated yaml> loads plugin.bundle.cjs
 * (CommonJS; Node 22.23 rejects require(esm) of dist/plugin.js), activates
 * POST /task/akp/dsh, submits admitted Workspace* candidates as startupEvents,
 * and points llm-deepseek at POST /provider/v1/dsh/chat/completions.
 * The daemon management token is read from a 0600 file. The Provider key stays
 * in SecretStore on the daemon host. A dsh response is never Task completion.
 *
 * Argv only (no CognitiveOS env-var literals): --port --bootstrap-file
 * --revision --dsh-root --adapter-root --task --provider-path a|b
 * --mode headless|web --web-host --web-port --dsh-home
 * Path A also requires --api-key-file (0600 or "-") and never logs the key.
 * Web mode is `--profile web --no-open` (native panel, not Personal `/ui/`).
 */
import { spawn } from "node:child_process";
import { chmodSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { pathToFileURL } from "node:url";
import { admitTask, httpJson, issueToken, waitLifecycle } from "./daemon-task.mjs";
import {
  assertFrontendDist,
  assertLoopbackHost,
  assertWebPort,
  listenUrl,
  pathBWebChildExtras,
  pathBWebCredentialsYaml,
  pathBWebSettingsYaml,
} from "./dsh-web-preflight.mjs";

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
const DEFAULT_LLM_TASK =
  "Reply with one sentence that summarizes this text and nothing else: CognitiveOS Personal is a local-first OS for governed agent work.";
const task = arg("--task", DEFAULT_LLM_TASK);
const providerPath = arg("--provider-path", "b");
const apiKeyFile = arg("--api-key-file");
const directBaseUrl = arg("--direct-base-url", "https://api.deepseek.com");
const mode = arg("--mode", "headless");
const webHost = arg("--web-host", "127.0.0.1");
const webPort = arg("--web-port", "3080");
const dshHomeOverride = arg("--dsh-home");
if (providerPath !== "a" && providerPath !== "b") {
  throw new Error("--provider-path must be a (direct Flash) or b (AKP/daemon)");
}
if (mode !== "headless" && mode !== "web") {
  throw new Error("--mode must be headless or web");
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
const work = join(tmpdir(), `p8t11-dsh-real-${process.pid}`);
mkdirSync(work, { mode: 0o700, recursive: true });
const bearerFile = join(work, "daemon.bearer");
const patchFile = join(work, "headless-akp.yml");
const dshHome = dshHomeOverride || join(work, "dsh-home");
mkdirSync(dshHome, { mode: 0o700, recursive: true });

function redact(error) {
  return String(error).replace(/Bearer\s+\S+/gi, "Bearer [redacted]").replace(/sk-[A-Za-z0-9]+/g, "sk-[redacted]");
}

function childEnvironment(extra = {}) {
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
  const compileCache = join(dshRoot, ".cognitiveos-node-compile-cache");
  mkdirSync(compileCache, { mode: 0o700, recursive: true });
  env.NODE_COMPILE_CACHE = compileCache;
  for (const [key, value] of Object.entries(extra)) {
    if (/API_KEY|SECRET|TOKEN|PASSWORD|BEARER/i.test(key)) {
      throw new Error(`child environment refuses secret-shaped key ${key}`);
    }
    env[key] = value;
  }
  return env;
}

async function bindRuntime(origin, token, processId) {
  return httpJson(origin, "POST", "/personal/dsh/runtime", token, {
    schema_version: 1,
    surface: "personal-dsh-runtime",
    op: "bind",
    process_id: processId,
  });
}

async function clearRuntime(origin, token) {
  return httpJson(origin, "POST", "/personal/dsh/runtime", token, {
    schema_version: 1,
    surface: "personal-dsh-runtime",
    op: "clear",
  });
}

function dshCliInvocation(root) {
  const compiled = join(root, "apps/cli/lib/bin.js");
  const compiledHostGraph = join(root, "packages/api/gateway/lib/index.js");
  if (existsSync(compiled) && existsSync(compiledHostGraph)) {
    return { args: [compiled], mode: "compiled-lib" };
  }
  return {
    args: ["--import", "tsx/esm", join(root, "apps/cli/src/bin.ts")],
    mode: "tsx-source",
  };
}

function llmDeepseekPatch(baseURL, apiKeyEnv) {
  return [
    "- id: llm-deepseek",
    "  config:",
    `    baseURL: ${baseURL}`,
    `    apiKeyEnv: ${apiKeyEnv}`,
    "    thinking: disabled",
    "    reasoningEffort: off",
    "    maxTokens: 256",
    "",
  ].join("\n");
}

function assistantLooksComplete(text) {
  const trimmed = String(text || "").trim();
  if (!trimmed) return false;
  if (/^pong\.?$/i.test(trimmed)) return true;
  return trimmed.split(/\s+/).length >= 4;
}

function runtimeFacts(payload) {
  const json = payload?.json ?? payload ?? {};
  return {
    state: json.state ?? null,
    session_count: json.session_count ?? null,
    process_id_bound: Number.isFinite(json.process_id),
    process_alive: json.process_alive ?? null,
    fencing_epochs: Array.isArray(json.sessions)
      ? json.sessions.map((session) => session.fencing_epoch)
      : [],
  };
}

async function runWebPathB() {
  const boundHost = assertLoopbackHost(webHost);
  const boundPort = assertWebPort(webPort);
  const distIndex = assertFrontendDist(dshRoot);
  const cli = dshCliInvocation(dshRoot);
  const pluginPath = join(adapterRoot, "plugin.bundle.cjs");
  if (!existsSync(pluginPath)) {
    throw new Error(`plugin.bundle.cjs is missing at ${pluginPath}`);
  }
  const bootstrap = readFileSync(bootstrapPath, "utf8").trim();
  const taskToken = await issueToken(origin, bootstrap, "task");
  const managementToken = await issueToken(origin, bootstrap, "management");
  writeFileSync(bearerFile, `${taskToken}\n`, { encoding: "utf8", mode: 0o600 });
  chmodSync(bearerFile, 0o600);
  const selected = await httpJson(
    origin,
    "GET",
    "/provider/v1/dsh/selected-model",
    managementToken,
  );
  const providerBase = `${origin}/provider/v1/dsh`;
  const settingsPath = join(dshHome, "settings.yaml");
  const existingSettings = existsSync(settingsPath) ? readFileSync(settingsPath, "utf8") : "";
  writeFileSync(settingsPath, pathBWebSettingsYaml(providerBase, existingSettings), {
    encoding: "utf8",
  });
  writeFileSync(join(dshHome, ".credentials.yaml"), pathBWebCredentialsYaml(managementToken), {
    encoding: "utf8",
    mode: 0o600,
  });
  chmodSync(join(dshHome, ".credentials.yaml"), 0o600);
  const patchBody = [
    "- insert:",
    ...pluginInsert("cognitiveos-akp", "dsh-web-process", "deepseek.dsh.akp", undefined, [
      { kind: "lifecycle", operation: "adapter.ready", payload: { ok: true, mode: "web" } },
    ]),
    llmDeepseekPatch(providerBase, "DAEMON_BEARER"),
  ].join("\n");
  writeFileSync(patchFile, patchBody, { encoding: "utf8", mode: 0o600 });
  const child = spawn(
    process.execPath,
    [
      ...cli.args,
      "--profile",
      "web",
      "--patch",
      patchFile,
      "--no-open",
      "--host",
      boundHost,
      "--port",
      String(boundPort),
    ],
    {
      cwd: dshRoot,
      env: childEnvironment(pathBWebChildExtras(providerBase)),
      stdio: ["ignore", "inherit", "inherit"],
    },
  );
  if (!child.pid) {
    throw new Error("dsh web spawn produced no pid");
  }
  await bindRuntime(origin, managementToken, child.pid);
  process.stderr.write(
    `cognitive dsh web listening at ${listenUrl(boundHost, boundPort)} dist=${distIndex} selected_model=${selected.json?.model ?? selected.json?.selected_model ?? "unset"}\n`,
  );
  const exitCode = await new Promise((resolve, reject) => {
    child.once("error", reject);
    child.once("close", resolve);
  });
  await clearRuntime(origin, managementToken);
  rmSync(work, { recursive: true, force: true });
  if (exitCode !== 0) {
    throw new Error(`dsh web exited ${exitCode}`);
  }
}

async function runDsh(patchBody, runtime) {
  writeFileSync(patchFile, patchBody, { encoding: "utf8", mode: 0o600 });
  const started = Date.now();
  const ttftHolder = { ms: null };
  // Prefer the compiled host CLI (`apps/cli/lib/bin.js`) when `build:lib`
  // outputs exist. `pnpm dsh` is not portable on an installed guest (pnpm 11
  // deps-status wants git). tsx-from-source on a 2 vCPU guest was ~11 s of
  // harness bootstrap before any Provider byte; compiled-lib is the product
  // launch path. Fallback remains `node --import tsx/esm apps/cli/src/bin.ts`.
  const cli = dshCliInvocation(dshRoot);
  const child = spawn(
    process.execPath,
    [...cli.args, "--profile", "headless", "--patch", patchFile, task],
    {
      cwd: dshRoot,
      env: childEnvironment(),
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  let runtimeAfterBind = null;
  if (runtime) {
    runtimeAfterBind = runtimeFacts(await bindRuntime(runtime.origin, runtime.token, child.pid));
  }
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
  const stderrRedacted = redact(stderr);
  return {
    exitCode,
    elapsedMs,
    ttftMs: ttftHolder.ms,
    assistant,
    stderrRedactedBytes: Buffer.byteLength(stderrRedacted, "utf8"),
    stderrPreviewRedacted: stderrRedacted.split(/\r?\n/).slice(-40).join("\n").slice(0, 2048),
    runtimeAfterBind,
    cliMode: cli.mode,
  };
}

if (mode === "web" && providerPath === "a") {
  throw new Error(
    "cognitive dsh web Path A is measurement-only; use Path B (AKP/daemon Provider). Do not put API keys in dsh .env",
  );
}

if (mode === "web") {
  await runWebPathB();
  process.exit(0);
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
  const outcome = await runDsh(llmDeepseekPatch(directBaseUrl, "DEEPSEEK_KEY"));
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
    cli_mode: outcome.cliMode,
    assistant_preview_bytes: Buffer.byteLength(outcome.assistant, "utf8"),
    assistant_is_pong: /^pong\.?$/i.test(outcome.assistant),
    assistant_ok: assistantLooksComplete(outcome.assistant),
    stderr_redacted_bytes: outcome.stderrRedactedBytes,
    stderr_preview_redacted: outcome.stderrPreviewRedacted,
    workspace: null,
    non_claims: ["Gate", "release", "Profile", "B01", "Agent-benefit"],
  };
  process.stdout.write(`${JSON.stringify(summary)}\n`);
  process.exit(outcome.exitCode === 0 && summary.assistant_ok ? 0 : 1);
}

function seedDisposableWorkspace() {
  let workspace = null;
  if (process.env.XDG_DATA_HOME) {
    workspace = join(process.env.XDG_DATA_HOME, "cognitiveos", "workspace");
  } else if (bootstrapPath) {
    const runtimeRoot = dirname(dirname(bootstrapPath));
    workspace = join(runtimeRoot, "data", "cognitiveos", "workspace");
  }
  if (!workspace) {
    throw new Error("Path B cannot infer the disposable workspace root");
  }
  mkdirSync(workspace, { recursive: true, mode: 0o700 });
  writeFileSync(
    join(workspace, "README.md"),
    "CognitiveOS Personal disposable workspace.\nneedle\n",
    { encoding: "utf8", mode: 0o600 },
  );
  return workspace;
}

const workspaceRoot = seedDisposableWorkspace();
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

function pluginHrefFor(_entryName) {
  return pathToFileURL(join(adapterRoot, "plugin.bundle.cjs")).href;
}

function pluginInsert(id, sessionId, pluginId, taskRef, events) {
  const lines = [
    `  - id: ${id}`,
    `    name: "${pluginHrefFor(id)}"`,
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

const selected = await httpJson(origin, "GET", "/provider/v1/dsh/selected-model", managementToken);
const providerBase = `${origin}/provider/v1/dsh`;
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
          input_b64: Buffer.from("p8-t11 disposable write\n", "utf8").toString("base64"),
          preimage: "absent",
        },
      },
    ]),
    llmDeepseekPatch(providerBase, "DAEMON_BEARER"),
  ].join("\n"),
  { origin, token: managementToken },
);
const runtimeAfterRun = runtimeFacts(await httpJson(origin, "GET", "/personal/dsh/runtime", managementToken));
const runtimeAfterClear = runtimeFacts(await clearRuntime(origin, managementToken));
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
  adapter: "dsh --patch cognitiveos-akp Workspace* + llm-deepseek via daemon Provider SSE proxy",
  candidate_only: true,
  dsh_response_is_not_task_completion: true,
  selected_model: selected.json?.model ?? selected.json?.selected_model ?? null,
  dsh_exit: outcome.exitCode,
  elapsed_ms: outcome.elapsedMs,
  ttft_ms: outcome.ttftMs,
  cli_mode: outcome.cliMode,
  assistant_preview_bytes: Buffer.byteLength(outcome.assistant, "utf8"),
  assistant_is_pong: /^pong\.?$/i.test(outcome.assistant),
  assistant_ok: assistantLooksComplete(outcome.assistant),
  stderr_redacted_bytes: outcome.stderrRedactedBytes,
  stderr_preview_redacted: outcome.stderrPreviewRedacted,
  runtime: {
    after_bind: outcome.runtimeAfterBind,
    after_run: runtimeAfterRun,
    after_clear: runtimeAfterClear,
  },
  workspace: {
    seeded_readme: true,
    workspace_root: workspaceRoot,
    read: { taskRef: readSpec.taskRef, lifecycle: readLife },
    search: { taskRef: searchSpec.taskRef, lifecycle: searchLife },
    write: { taskRef: writeSpec.taskRef, target: writeTarget, lifecycle: writeLife },
  },
  non_claims: ["Gate", "release", "Profile", "B01", "Agent-benefit"],
};
process.stdout.write(`${JSON.stringify(summary)}\n`);
const workspacePass =
  readLife === "COMPLETED" && searchLife === "COMPLETED" && writeLife === "COMPLETED";
process.exit(outcome.exitCode === 0 && summary.assistant_ok && workspacePass ? 0 : 1);
