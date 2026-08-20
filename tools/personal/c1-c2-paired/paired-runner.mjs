/**
 * Paired P/O runner (P9-T08 dry-run + P9-T12 live cells).
 * Measurement-only. Not a second authority writer. Not B0.
 */

import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { checkFairness } from "./fairness-checker.mjs";
import {
  ARM_ORDER_SEED,
  COMMAND_MANIFEST,
  INSTRUMENT_ROOT,
  PI_SRI,
  PI_VERSION,
  RETRY,
  TIMEOUT_MS,
  buildFreezeLedger,
  frozenSeeds,
} from "./freeze.mjs";
import { isSecretShaped, assertSecretFreeProcess } from "./pure-pi-broker.mjs";
import { WORKSPACE_TOOL_SCHEMAS } from "./workspace-fixture-adapter.mjs";
import { redactPairedEvidence } from "./redactor.mjs";

export const FORBIDDEN_SHARED_PROMPT_PLACEHOLDER = "frozen-c1-c2-prompt-v1";

const COUNTED_STRATA = new Set(["b1", "b2"]);
const KNOWN_STRATA = new Set(["b0", "b1", "b2"]);

export function frozenSystemTaskPromptPath() {
  return path.join(INSTRUMENT_ROOT, "frozen-system-task-prompt.txt");
}

export function frozenSystemTaskPromptBytes() {
  return Buffer.byteLength(readFileSync(frozenSystemTaskPromptPath()));
}

export function liveArmAppendSystemPromptArgs() {
  return Object.freeze(["--append-system-prompt", frozenSystemTaskPromptPath()]);
}

export function liveArmCommandManifest() {
  const promptArgs = liveArmAppendSystemPromptArgs();
  return Object.freeze({
    p: Object.freeze(["pi", "--print", ...promptArgs]),
    o: Object.freeze(["cognitive", "pi", "launch", "--print", ...promptArgs]),
    system_task_prompt_bytes: frozenSystemTaskPromptBytes(),
  });
}

export function assertLiveLaunchArgv(argv) {
  if (!Array.isArray(argv) || argv.length === 0) {
    throw new Error("live launch argv is required");
  }
  const promptPath = frozenSystemTaskPromptPath();
  const flagIndex = argv.indexOf("--append-system-prompt");
  const file = flagIndex >= 0 ? argv[flagIndex + 1] : undefined;
  if (flagIndex < 0 || file !== promptPath || !path.isAbsolute(String(file))) {
    throw new Error(
      "live launch requires --append-system-prompt <frozen-system-task-prompt.txt absolute path>",
    );
  }
  return true;
}

export function assertLiveCountedLabel({
  counted_sample,
  stratum,
  dry_run = false,
} = {}) {
  if (dry_run && counted_sample) {
    throw new Error("dry-run cannot be labeled counted B1/B2");
  }
  if (counted_sample && stratum === "b0") {
    throw new Error("b0 is non-counted qualification");
  }
  if (counted_sample && !COUNTED_STRATA.has(stratum)) {
    throw new Error("counted_sample is only allowed for frozen b1/b2 cells");
  }
  return true;
}

export function armOrderForSeed(seed) {
  if (typeof seed !== "string" || seed.length === 0) {
    throw new Error("arm order requires a frozen seed");
  }
  const digest = createHash("sha256").update(`${ARM_ORDER_SEED}|${seed}`).digest();
  return digest[0] % 2 === 0 ? Object.freeze(["p", "o"]) : Object.freeze(["o", "p"]);
}

export function equalArmSnapshot(overrides = {}) {
  return {
    pi_package_version_sri: `${PI_VERSION}/${PI_SRI}`,
    node_version: process.version,
    provider_base_url_model: "https://api.deepseek.com|deepseek-v4-flash",
    system_task_prompt_bytes: frozenSystemTaskPromptBytes(),
    task_input_digest: "sha256:fixture",
    sampling_parameters: { temperature: 0, top_p: 1, seed: 1, max_output_tokens: 256 },
    timeout_retry0_max_turn: {
      timeout_ms: TIMEOUT_MS,
      retry: RETRY,
      max_agent_turn: COMMAND_MANIFEST.max_agent_turn,
    },
    visible_tool_set_schema: WORKSPACE_TOOL_SCHEMAS,
    workspace_snapshot: "sha256:fixture-workspace",
    network_policy: "loopback-broker-or-daemon-proxy",
    cpu_memory_cwd_fs: "guest-applied-at-eval",
    oracle_version: "c1-c2-mechanical-v1",
    warm_cold_stratum: "warm",
    ...overrides,
  };
}

export async function dryRunFairness({ mutateAxis } = {}) {
  const ledger = await buildFreezeLedger();
  const p = equalArmSnapshot();
  const o = equalArmSnapshot(mutateAxis ? { [mutateAxis]: "mutated" } : {});
  const record = checkFairness({ p, o });
  redactPairedEvidence(record);
  const result = {
    kind: "c1-c2-paired-dry-run",
    retry: RETRY,
    counted_sample: false,
    b0: false,
    freeze_file_count: Object.keys(ledger.files).length,
    fairness: record,
  };
  assertLiveCountedLabel({
    counted_sample: result.counted_sample,
    stratum: "b0",
    dry_run: true,
  });
  return result;
}

function frozenSeedForCell(stratum, classId, seedIndex) {
  if (!KNOWN_STRATA.has(stratum)) {
    throw new Error(`unknown stratum ${stratum}`);
  }
  const seed = frozenSeeds()[stratum]?.[classId]?.[seedIndex];
  if (typeof seed !== "string") {
    throw new Error(`unknown frozen seed ${stratum}/${classId}/${seedIndex}`);
  }
  return seed;
}

function summarizeArm(result, argv) {
  assertLiveLaunchArgv(argv);
  if (result?.secret_shaped || isSecretShaped(JSON.stringify(result ?? {}))) {
    throw new Error("arm result secret-shaped");
  }
  redactPairedEvidence(result ?? {});
  return Object.freeze({
    exit_code: Number(result?.exit_code ?? 1),
    timed_out: Boolean(result?.timed_out),
    append_system_prompt: true,
    retry: RETRY,
  });
}

function bothArmsOk(arms) {
  return ["p", "o"].every((arm) => {
    const row = arms[arm];
    return row?.exit_code === 0 && row?.timed_out === false && row?.append_system_prompt === true;
  });
}

/**
 * Live paired cell. Campaigns must inject `executeArm`; tests must stub it.
 * The default spawn helper is exported separately and is never called here.
 */
export async function runLivePairedCell({
  stratum,
  classId,
  seedIndex,
  executeArm,
  env = process.env,
  argv = process.argv,
  mutateAxis,
} = {}) {
  assertSecretFreeProcess({ env, argv });
  if (typeof executeArm !== "function") {
    throw new Error("executeArm required; live cells do not spawn by accident");
  }
  const seed = frozenSeedForCell(stratum, classId, seedIndex);
  const live = liveArmCommandManifest();
  assertLiveLaunchArgv(live.p);
  assertLiveLaunchArgv(live.o);

  const fairness = checkFairness({
    p: equalArmSnapshot(),
    o: equalArmSnapshot(mutateAxis ? { [mutateAxis]: "mutated" } : {}),
  });

  const order = armOrderForSeed(seed);
  const arms = {};
  if (fairness.result === "pass") {
    for (const arm of order) {
      const armArgv = arm === "p" ? [...live.p] : [...live.o];
      assertLiveLaunchArgv(armArgv);
      const result = await executeArm({
        arm,
        argv: armArgv,
        seed,
        timeout_ms: TIMEOUT_MS,
        retry: RETRY,
      });
      arms[arm] = summarizeArm(result, armArgv);
    }
  }

  const counted_sample =
    COUNTED_STRATA.has(stratum) && fairness.result === "pass" && bothArmsOk(arms);
  assertLiveCountedLabel({ counted_sample, stratum, dry_run: false });

  const record = {
    kind: "c1-c2-paired-live-cell",
    stratum,
    class_id: classId,
    seed,
    seed_index: seedIndex,
    retry: RETRY,
    timeout_ms: TIMEOUT_MS,
    arm_order: [...order],
    append_system_prompt: true,
    counted_sample,
    b0: false,
    fairness,
    arms,
  };
  redactPairedEvidence(record);
  return record;
}

export function spawnLiveArm({ argv, timeout_ms = TIMEOUT_MS } = {}) {
  assertSecretFreeProcess({ env: process.env, argv });
  assertLiveLaunchArgv(argv);
  const [command, ...args] = argv;
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      env: process.env,
      stdio: ["ignore", "pipe", "pipe"],
      windowsHide: true,
    });
    let stdout = "";
    let stderr = "";
    let timedOut = false;
    const timer = setTimeout(() => {
      timedOut = true;
      child.kill("SIGKILL");
    }, timeout_ms);
    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    child.on("error", (error) => {
      clearTimeout(timer);
      reject(error);
    });
    child.on("close", (code) => {
      clearTimeout(timer);
      if (isSecretShaped(`${stdout}${stderr}`)) {
        reject(new Error("arm output secret-shaped"));
        return;
      }
      resolve({
        exit_code: code ?? 1,
        timed_out: timedOut,
        append_system_prompt: true,
        retry: RETRY,
      });
    });
  });
}
