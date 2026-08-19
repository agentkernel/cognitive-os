/**
 * Linux Secret Service get for the P-arm broker.
 *
 * Get uses D-Bus SearchItems + GetSecret only. secret-tool lookup/search are
 * forbidden. Probe store/clear may use secret-tool store (stdin) and clear
 * (attributes only). Material never enters argv, env, logs, or helper stdout.
 */

import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { isSecretShaped } from "./pure-pi-broker.mjs";

export const PROBE_SECRET_ATTRIBUTES = Object.freeze({
  application: "cognitiveos-personal-p9-t08",
  provider: "probe",
  purpose: "p-arm-broker-get-probe",
});

export const PROBE_SECRET_LABEL = "cognitiveos-p9-t08-p-arm-broker-probe";

export const PRODUCT_PROVIDER_ATTRIBUTES = Object.freeze({
  application: "cognitiveos-personal",
  provider: "deepseek",
  purpose: "provider-api-key",
});

const HELPER = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  "linux-secret-get-helper.py",
);

export function assertNonSecretAttributes(attributes) {
  if (attributes == null || typeof attributes !== "object") {
    throw new Error("secret attributes are required");
  }
  for (const [key, value] of Object.entries(attributes)) {
    if (typeof key !== "string" || typeof value !== "string") {
      throw new Error("attribute keys and values must be strings");
    }
    if (isSecretShaped(key) || isSecretShaped(value)) {
      throw new Error("secret-shaped attribute refused");
    }
  }
}

function requireLinux() {
  if (process.platform !== "linux") {
    throw new Error("linux secret service is not available");
  }
}

function collectStream(stream) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    stream.on("data", (chunk) => chunks.push(chunk));
    stream.on("error", reject);
    stream.on("end", () => resolve(Buffer.concat(chunks)));
  });
}

function parseFacts(stdout) {
  const text = stdout.toString("utf8").trim();
  if (text === "") {
    throw new Error("secret service helper returned no facts");
  }
  if (isSecretShaped(text)) {
    throw new Error("secret-shaped helper stdout refused");
  }
  let facts;
  try {
    facts = JSON.parse(text);
  } catch {
    throw new Error("secret service helper facts were not json");
  }
  if (isSecretShaped(JSON.stringify(facts))) {
    throw new Error("secret-shaped helper facts refused");
  }
  return facts;
}

function spawnHelper({ attributes, pathsOnly }) {
  requireLinux();
  assertNonSecretAttributes(attributes);
  const args = [HELPER];
  if (pathsOnly) {
    args.push("--paths-only");
  } else {
    args.push("--material-fd", "3");
  }
  const child = spawn("python3", args, {
    stdio: ["pipe", "pipe", "pipe", "pipe"],
    env: sanitizedEnv(),
  });
  const stderrPromise = collectStream(child.stderr);
  const stdoutPromise = collectStream(child.stdout);
  const materialPromise = pathsOnly ? Promise.resolve(Buffer.alloc(0)) : collectStream(child.stdio[3]);
  child.stdin.write(JSON.stringify(attributes));
  child.stdin.end();
  return { child, stdoutPromise, stderrPromise, materialPromise };
}

export async function searchLinuxSecretPaths(attributes = PROBE_SECRET_ATTRIBUTES) {
  const { child, stdoutPromise, stderrPromise, materialPromise } = spawnHelper({
    attributes,
    pathsOnly: true,
  });
  const [stdout, stderr, material, status] = await Promise.all([
    stdoutPromise,
    stderrPromise,
    materialPromise,
    new Promise((resolve, reject) => {
      child.on("error", reject);
      child.on("close", (code, signal) => resolve({ code, signal }));
    }),
  ]);
  if (material.length !== 0) {
    throw new Error("paths-only helper wrote material");
  }
  if (isSecretShaped(stderr.toString("utf8"))) {
    throw new Error("secret-shaped helper stderr refused");
  }
  const facts = parseFacts(stdout);
  if (status.code !== 0 || facts.ok !== true) {
    throw new Error(`secret service path search failed: ${facts.reason ?? "closed"}`);
  }
  return facts;
}

export async function getLinuxSecretMaterial(attributes = PROBE_SECRET_ATTRIBUTES) {
  const { child, stdoutPromise, stderrPromise, materialPromise } = spawnHelper({
    attributes,
    pathsOnly: false,
  });
  const [stdout, stderr, material, status] = await Promise.all([
    stdoutPromise,
    stderrPromise,
    materialPromise,
    new Promise((resolve, reject) => {
      child.on("error", reject);
      child.on("close", (code, signal) => resolve({ code, signal }));
    }),
  ]);
  if (isSecretShaped(stderr.toString("utf8"))) {
    throw new Error("secret-shaped helper stderr refused");
  }
  const facts = parseFacts(stdout);
  if (status.code !== 0 || facts.ok !== true) {
    throw new Error(`secret service get failed: ${facts.reason ?? "closed"}`);
  }
  if (material.length === 0) {
    throw new Error("secret service get returned empty material");
  }
  return { material: material.toString("utf8"), facts };
}

export function createLinuxSecretServiceGet(attributes = PROBE_SECRET_ATTRIBUTES) {
  assertNonSecretAttributes(attributes);
  return async () => {
    const { material } = await getLinuxSecretMaterial(attributes);
    return material;
  };
}

const SECRET_ENV_NAMES = /^(?:PROVIDER|OPENAI|DEEPSEEK|ANTHROPIC|API|LLM).*KEY$/i;

function sanitizedEnv() {
  const env = {};
  for (const [name, value] of Object.entries(process.env)) {
    if (value === undefined) {
      continue;
    }
    if (SECRET_ENV_NAMES.test(name) || isSecretShaped(String(value))) {
      continue;
    }
    env[name] = value;
  }
  return env;
}

function attributeArgv(attributes) {
  assertNonSecretAttributes(attributes);
  const argv = [];
  for (const [key, value] of Object.entries(attributes)) {
    argv.push(key, value);
  }
  return argv;
}

export async function storeLinuxSecret({
  label = PROBE_SECRET_LABEL,
  attributes = PROBE_SECRET_ATTRIBUTES,
  material,
} = {}) {
  requireLinux();
  if (typeof material !== "string" || material.length === 0) {
    throw new Error("probe material is required");
  }
  if (typeof label !== "string" || isSecretShaped(label)) {
    throw new Error("secret-shaped label refused");
  }
  const child = spawn(
    "secret-tool",
    ["store", "--label", label, ...attributeArgv(attributes)],
    { stdio: ["pipe", "pipe", "pipe"], env: sanitizedEnv() },
  );
  const stdoutPromise = collectStream(child.stdout);
  const stderrPromise = collectStream(child.stderr);
  child.stdin.write(material);
  child.stdin.end();
  const [stdout, stderr, status] = await Promise.all([
    stdoutPromise,
    stderrPromise,
    new Promise((resolve, reject) => {
      child.on("error", reject);
      child.on("close", (code, signal) => resolve({ code, signal }));
    }),
  ]);
  const combined = `${stdout}${stderr}`;
  if (isSecretShaped(combined.toString("utf8"))) {
    throw new Error("secret-shaped secret-tool store output refused");
  }
  if (status.code !== 0) {
    throw new Error("secret-tool store failed");
  }
}

export async function clearLinuxSecret(attributes = PROBE_SECRET_ATTRIBUTES) {
  requireLinux();
  const child = spawn("secret-tool", ["clear", ...attributeArgv(attributes)], {
    stdio: ["ignore", "pipe", "pipe"],
    env: sanitizedEnv(),
  });
  const stdoutPromise = collectStream(child.stdout);
  const stderrPromise = collectStream(child.stderr);
  const [stdout, stderr, status] = await Promise.all([
    stdoutPromise,
    stderrPromise,
    new Promise((resolve, reject) => {
      child.on("error", reject);
      child.on("close", (code, signal) => resolve({ code, signal }));
    }),
  ]);
  const combined = `${stdout}${stderr}`;
  if (isSecretShaped(combined.toString("utf8"))) {
    throw new Error("secret-shaped secret-tool clear output refused");
  }
  if (status.code !== 0 && status.code !== 1) {
    throw new Error("secret-tool clear failed");
  }
}
