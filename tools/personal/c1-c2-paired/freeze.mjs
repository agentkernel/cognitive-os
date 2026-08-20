/**
 * Frozen C1/C2 paired assets (P9-T08/D03). Secret-free. Not a sample.
 */

import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import { readdir, readFile, stat } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { WORKSPACE_TOOL_SCHEMAS } from "./workspace-fixture-adapter.mjs";
import { PI_PLACEHOLDER_TOKEN } from "./pure-pi-broker.mjs";
import { FAIRNESS_AXES } from "./fairness-checker.mjs";

export const INSTRUMENT_ROOT = path.dirname(fileURLToPath(import.meta.url));

export const RETRY = 0;
export const TIMEOUT_MS = 120_000;
export const MAX_AGENT_TURN = 8;
export const ARM_ORDER_SEED = "p9-t08-c1-c2-arm-order-v1";
export const PI_VERSION = "0.81.1";
export const PI_SRI =
  "sha512-r6ovAsZOgAqbC/aU6s+/dPnv/sGZBuWyZNvi3pXjpbuX5wvp3XvGkQI7/VLvX2o9XpmpFaPUxKNym1WfkN/P8A==";

const CLASSES = Object.freeze(["C1", "C2a", "C2b", "C2c", "C2d"]);

function sha256Hex(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function seedId(stratum, classId, index) {
  return `sha256:${sha256Hex(`${stratum}|${classId}|${index}`).slice(0, 16)}`;
}

export function frozenSeeds() {
  const b0 = {};
  const b1 = {};
  const b2 = {};
  for (const classId of CLASSES) {
    b0[classId] = [seedId("c1-c2-b0-qualification-v1", classId, 0)];
    b1[classId] = Array.from({ length: 5 }, (_, index) =>
      seedId("c1-c2-b1-pilot-v1", classId, index),
    );
    b2[classId] = Array.from({ length: 30 }, (_, index) =>
      seedId("c1-c2-b2-heldout-v1", classId, index),
    );
  }
  return Object.freeze({ retry: RETRY, b0, b1, b2 });
}

export function assertDisjointSeeds(seeds = frozenSeeds()) {
  const seen = new Set();
  for (const stratum of ["b0", "b1", "b2"]) {
    for (const classId of CLASSES) {
      for (const seed of seeds[stratum][classId]) {
        if (seen.has(seed)) {
          throw new Error(`overlapping seed ${seed}`);
        }
        seen.add(seed);
      }
    }
  }
  return seen.size;
}

export const CELL_OVERLAY = Object.freeze({
  C1: {
    comparable: true,
    p: "fixture WorkspaceRead/Search",
    o: "daemon WorkspaceRead/Search",
  },
  C2a: {
    comparable: true,
    p: "fixture WorkspaceWrite/Patch",
    o: "daemon WorkspaceWrite/Patch plus Effect/verifier/acceptance",
  },
  C2b: {
    comparable: false,
    class: "capability-gap / split scores",
    p: "frozen procedure bytes; not daemon Memory/Skill",
    o: "daemon-authorized Memory/Skill session-2",
  },
  C2c: {
    comparable: false,
    class: "split scores",
    p: "fixture mutation reference",
    o: "governed original-key reconcile",
  },
  C2d: {
    comparable: false,
    class: "split scores",
    p: "external mechanical oracle",
    o: "daemon acceptance; pure-Pi completion is not OS Task completion",
  },
});

export const COMMAND_MANIFEST = Object.freeze({
  retry: RETRY,
  timeout_ms: TIMEOUT_MS,
  max_agent_turn: MAX_AGENT_TURN,
  arm_order_seed: ARM_ORDER_SEED,
  pi_version: PI_VERSION,
  pi_sri: PI_SRI,
  broker_bind: "127.0.0.1:48400",
  pi_placeholder_token: PI_PLACEHOLDER_TOKEN,
  daemon_bind: "127.0.0.1:48300",
  tool_schemas: WORKSPACE_TOOL_SCHEMAS,
  fairness_axes: FAIRNESS_AXES,
  append_system_prompt: "frozen-system-task-prompt.txt",
});

export const CORPUS_RELATIVE_PATHS = Object.freeze([
  "fixtures/c1/workspace/note.txt",
  "fixtures/c2a/workspace/src/repair.ts",
  "fixtures/c2a/workspace/tests/repair.test.ts",
  "fixtures/c2a/oracle.json",
  "fixtures/c2b/procedure.txt",
  "fixtures/c2c/original-key.txt",
  "fixtures/c2d/oracle.json",
]);

export async function digestFile(relativePath) {
  const absolute = path.join(INSTRUMENT_ROOT, relativePath);
  const hash = createHash("sha256");
  await new Promise((resolve, reject) => {
    const stream = createReadStream(absolute);
    stream.on("data", (chunk) => hash.update(chunk));
    stream.on("error", reject);
    stream.on("end", resolve);
  });
  return `sha256:${hash.digest("hex")}`;
}

export async function buildFreezeLedger() {
  assertDisjointSeeds();
  const files = {};
  for (const relativePath of CORPUS_RELATIVE_PATHS) {
    files[relativePath] = await digestFile(relativePath);
  }
  const instruments = [
    "pure-pi-broker.mjs",
    "workspace-fixture-adapter.mjs",
    "linux-secret-service.mjs",
    "linux-secret-get-helper.py",
    "fairness-checker.mjs",
    "redactor.mjs",
    "freeze.mjs",
    "paired-runner.mjs",
    "prove-linux-secret-get.mjs",
    "cells.json",
    "frozen-system-task-prompt.txt",
  ];
  for (const relativePath of instruments) {
    files[relativePath] = await digestFile(relativePath);
  }
  return Object.freeze({
    kind: "c1-c2-paired-freeze-ledger",
    retry: RETRY,
    timeout_ms: TIMEOUT_MS,
    arm_order_seed: ARM_ORDER_SEED,
    pi_version: PI_VERSION,
    pi_sri: PI_SRI,
    seeds: frozenSeeds(),
    cell_overlay: CELL_OVERLAY,
    command_manifest: COMMAND_MANIFEST,
    files,
    counted_sample: false,
    b0: false,
  });
}

export async function listCorpusBytes() {
  const bytes = {};
  for (const relativePath of CORPUS_RELATIVE_PATHS) {
    const absolute = path.join(INSTRUMENT_ROOT, relativePath);
    const info = await stat(absolute);
    if (!info.isFile()) {
      throw new Error(`corpus path is not a file: ${relativePath}`);
    }
    const text = await readFile(absolute, "utf8");
    if (text.includes("sk-") || text.includes("BEGIN ")) {
      throw new Error(`secret-shaped corpus bytes: ${relativePath}`);
    }
    bytes[relativePath] = info.size;
  }
  return bytes;
}

export async function assertNoExtraCorpusFiles() {
  const listed = new Set(CORPUS_RELATIVE_PATHS.map((entry) => path.normalize(entry)));
  async function walk(relative) {
    const absolute = path.join(INSTRUMENT_ROOT, relative);
    const entries = await readdir(absolute, { withFileTypes: true });
    for (const entry of entries) {
      const child = path.join(relative, entry.name);
      if (entry.isDirectory()) {
        await walk(child);
      } else if (entry.isFile() && !listed.has(path.normalize(child))) {
        throw new Error(`unfrozen corpus file: ${child}`);
      }
    }
  }
  await walk("fixtures");
}
