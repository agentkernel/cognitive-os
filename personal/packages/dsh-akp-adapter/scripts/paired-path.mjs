#!/usr/bin/env node
/**
 * Same-host Path A vs Path B timing (P8-T10). Path A is dsh → DeepSeek Flash
 * direct. Path B is dsh → AKP adapter → daemon → Flash. This script is
 * measurement-only. It never claims lossless, Gate, release, Profile, B01, or
 * Agent-benefit. Keys are read from --api-key-file and never logged.
 *
 * Path A does not use the daemon Provider proxy. Path B requires a live
 * Personal daemon and SecretStore-bound Flash.
 */
import { spawn } from "node:child_process";
import { chmodSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { stdin as stdinStream } from "node:process";

function arg(name, fallback) {
  const index = process.argv.indexOf(name);
  if (index >= 0 && process.argv[index + 1]) return process.argv[index + 1];
  return fallback;
}

function percentile(values, p) {
  if (values.length === 0) return null;
  const sorted = [...values].sort((a, b) => a - b);
  const index = Math.min(sorted.length - 1, Math.max(0, Math.ceil((p / 100) * sorted.length) - 1));
  return sorted[index];
}

const helper = join(dirname(fileURLToPath(import.meta.url)), "dsh-real-process.mjs");
const rawProbe = join(dirname(fileURLToPath(import.meta.url)), "provider-raw-probe.mjs");
const n = Number(arg("--n", "5"));
const dshRoot = arg("--dsh-root");
const adapterRoot = arg("--adapter-root");
const revision = arg("--revision");
const apiKeyFile = arg("--api-key-file");
const port = arg("--port");
const bootstrapFile = arg("--bootstrap-file");
const DEFAULT_LLM_TASK =
  "Reply with one sentence that summarizes this text and nothing else: CognitiveOS Personal is a local-first OS for governed agent work.";
const task = arg("--task", DEFAULT_LLM_TASK);
const runRaw = process.argv.includes("--raw");
if (!Number.isInteger(n) || n < 1 || !dshRoot || !adapterRoot || !revision || !apiKeyFile) {
  throw new Error("--n --dsh-root --adapter-root --revision --api-key-file are required");
}
if (!port || !bootstrapFile) {
  throw new Error("Path B requires --port and --bootstrap-file");
}

async function materializeKeyFile(source) {
  if (source !== "-") {
    return { path: source, cleanup: () => {} };
  }
  const chunks = [];
  for await (const chunk of stdinStream) {
    chunks.push(chunk);
  }
  const key = Buffer.concat(chunks).toString("utf8").trim();
  if (!key) {
    throw new Error("stdin key was empty");
  }
  const dir = mkdtempSync(join(tmpdir(), "p8t10-key-"));
  chmodSync(dir, 0o700);
  const path = join(dir, "provider.key");
  writeFileSync(path, `${key}\n`, { encoding: "utf8", mode: 0o600 });
  chmodSync(path, 0o600);
  return {
    path,
    cleanup: () => {
      try {
        writeFileSync(path, "0".repeat(64));
      } catch {
        /* best-effort overwrite */
      }
      rmSync(dir, { recursive: true, force: true });
    },
  };
}

function runSample(providerPath, extraArgs) {
  return new Promise((resolve, reject) => {
    const child = spawn(
      process.execPath,
      [
        helper,
        "--provider-path",
        providerPath,
        "--dsh-root",
        dshRoot,
        "--adapter-root",
        adapterRoot,
        "--revision",
        revision,
        "--task",
        task,
        ...extraArgs,
      ],
      { stdio: ["ignore", "pipe", "pipe"] },
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
    child.on("close", (code) => {
      const line = stdout.trim().split(/\r?\n/).filter(Boolean).at(-1) ?? "";
      let parsed = null;
      try {
        parsed = JSON.parse(line);
      } catch {
        parsed = null;
      }
      resolve({
        path: providerPath,
        exit: code,
        ok: code === 0 && (parsed?.assistant_ok === true || parsed?.assistant_is_pong === true),
        elapsed_ms: parsed?.elapsed_ms ?? null,
        ttft_ms: parsed?.ttft_ms ?? null,
        workspace: parsed?.workspace ?? null,
        parse_error: parsed ? false : true,
        stderr_redacted_bytes: parsed?.stderr_redacted_bytes ?? Buffer.byteLength(stderr, "utf8"),
      });
    });
    child.on("error", reject);
  });
}

const keyMaterial = await materializeKeyFile(apiKeyFile);
const resolvedKeyFile = keyMaterial.path;
try {
  let raw = null;
  if (runRaw) {
    raw = await new Promise((resolve, reject) => {
      const child = spawn(
        process.execPath,
        [rawProbe, "--api-key-file", resolvedKeyFile, "--prompt", task],
        { stdio: ["ignore", "pipe", "pipe"] },
      );
      let stdout = "";
      child.stdout.setEncoding("utf8");
      child.stdout.on("data", (chunk) => {
        stdout += chunk;
      });
      child.on("close", (code) => {
        const line = stdout.trim().split(/\r?\n/).filter(Boolean).at(-1) ?? "";
        let parsed = null;
        try {
          parsed = JSON.parse(line);
        } catch {
          parsed = null;
        }
        resolve({
          exit: code,
          elapsed_ms: parsed?.elapsed_ms ?? null,
          ttfb_ms: parsed?.ttfb_ms ?? null,
          status: parsed?.status ?? null,
          stream: parsed?.stream ?? null,
          body_bytes: parsed?.body_bytes ?? null,
        });
      });
      child.on("error", reject);
    });
  }
  const retained = { a: [], b: [] };
  const discarded = { a: 0, b: 0 };
  for (const path of ["a", "b"]) {
    for (let i = 0; i < n; i += 1) {
      const extra =
        path === "a"
          ? ["--api-key-file", resolvedKeyFile]
          : ["--port", port, "--bootstrap-file", bootstrapFile];
      const sample = await runSample(path, extra);
      sample.stratum = i === 0 ? "cold" : "warm";
      sample.repeat = i;
      if (sample.ok && Number.isFinite(sample.elapsed_ms)) {
        retained[path].push(sample);
      } else {
        discarded[path] += 1;
      }
    }
  }

  function summarize(path) {
    const samples = retained[path];
    const elapsed = samples.map((sample) => sample.elapsed_ms).filter((value) => Number.isFinite(value));
    const ttft = samples.map((sample) => sample.ttft_ms).filter((value) => Number.isFinite(value));
    return {
      started: n,
      retained: samples.length,
      discarded: discarded[path],
      elapsed_ms: {
        n: elapsed.length,
        min: elapsed.length ? Math.min(...elapsed) : null,
        p50: percentile(elapsed, 50),
        p95: elapsed.length >= 2 ? percentile(elapsed, 95) : null,
        max: elapsed.length ? Math.max(...elapsed) : null,
      },
      ttft_ms: {
        n: ttft.length,
        min: ttft.length ? Math.min(...ttft) : null,
        p50: percentile(ttft, 50),
        p95: ttft.length >= 2 ? percentile(ttft, 95) : null,
        max: ttft.length ? Math.max(...ttft) : null,
      },
      cold_elapsed_ms: samples.find((sample) => sample.stratum === "cold")?.elapsed_ms ?? null,
      warm_elapsed_ms: samples.filter((sample) => sample.stratum === "warm").map((sample) => sample.elapsed_ms),
    };
  }

  const pathA = summarize("a");
  const pathB = summarize("b");
  const overhead =
    pathA.elapsed_ms.p50 != null && pathB.elapsed_ms.p50 != null
      ? pathB.elapsed_ms.p50 - pathA.elapsed_ms.p50
      : null;

  const report = {
    kind: "p8-t11-paired-path-observation",
    same_host: true,
    n_requested: n,
    raw_provider: raw,
    path_a: pathA,
    path_b: pathB,
    overhead_b_minus_a_p50_ms: overhead,
    lossless_preset: false,
    limitations: [
      "Provider network still dominates wall time when the model is slow; n is small",
      "Path B includes Workspace* admits plus daemon SSE proxy; Path A is LLM-only direct Flash",
      "TTFT is first stdout byte of the dsh process, not a streaming token timestamp",
      "NODE_COMPILE_CACHE is set for the dsh Node/tsx child; first sample may still be cold",
    ],
    non_claims: ["Gate", "release", "Profile", "B01", "Agent-benefit", "lossless"],
  };
  process.stdout.write(`${JSON.stringify(report)}\n`);
  process.exitCode = pathA.retained === n && pathB.retained === n ? 0 : 1;
} finally {
  keyMaterial.cleanup();
}
