/**
 * Paired P/O runner dry-run (P9-T08/D03). Emits a fairness record. Not B0.
 */

import { readFileSync } from "node:fs";
import path from "node:path";
import { checkFairness } from "./fairness-checker.mjs";
import {
  COMMAND_MANIFEST,
  INSTRUMENT_ROOT,
  PI_SRI,
  PI_VERSION,
  RETRY,
  TIMEOUT_MS,
  buildFreezeLedger,
} from "./freeze.mjs";
import { WORKSPACE_TOOL_SCHEMAS } from "./workspace-fixture-adapter.mjs";
import { redactPairedEvidence } from "./redactor.mjs";

export const FORBIDDEN_SHARED_PROMPT_PLACEHOLDER = "frozen-c1-c2-prompt-v1";

export function frozenSystemTaskPromptBytes() {
  return Buffer.byteLength(
    readFileSync(path.join(INSTRUMENT_ROOT, "frozen-system-task-prompt.txt")),
  );
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
  return {
    kind: "c1-c2-paired-dry-run",
    retry: RETRY,
    counted_sample: false,
    b0: false,
    freeze_file_count: Object.keys(ledger.files).length,
    fairness: record,
  };
}
