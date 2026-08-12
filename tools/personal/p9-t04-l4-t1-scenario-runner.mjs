/**
 * P9-T04 `L4` `T1` read-only governed-Task scenario runner.
 *
 * Drives the real daemon Task path — session, `intent.record`,
 * `intent.interpret`, `task.preview`, `task.admit` — and records stage timings
 * plus the facts the T1 oracle needs. It judges nothing itself: the outcome is
 * decided by the Rust harness rules, and this runner only reports what the
 * daemon actually did.
 *
 * It reads the local bootstrap secret to mint a task-channel session, exactly
 * as the product client does, and never prints, logs, or returns it. No prompt,
 * response, bearer, or authority-store content enters the output.
 */
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { hrtime } from "node:process";

const SCENARIO_ID = "T1-read-only-analysis";
const CONVERSATION_REF = "conversation://local/p9-t04-t1";

function requiredArgument(name, fallback) {
  const argv = process.argv.slice(2);
  const index = argv.indexOf(name);
  if (index === -1) {
    if (fallback !== undefined) return fallback;
    throw new Error(`missing required argument ${name}`);
  }
  const value = argv[index + 1];
  if (value === undefined) throw new Error(`missing value for ${name}`);
  return value;
}

const runtimeRoot = requiredArgument("--runtime-root");
const endpointHost = requiredArgument("--endpoint", "127.0.0.1:48181");
const sourceRevision = requiredArgument("--source-revision");
const startedRuns = Number.parseInt(requiredArgument("--runs", "10"), 10);
if (!/^[0-9a-f]{40}$/.test(sourceRevision)) {
  throw new Error("--source-revision must be a full hexadecimal Git revision");
}
if (!Number.isSafeInteger(startedRuns) || startedRuns < 1) {
  throw new Error("--runs must be a positive integer");
}

/** Read once, hold in memory, never emit. */
const bootstrapSecret = readFileSync(
  join(runtimeRoot, "cognitiveos", "local-bootstrap.secret"),
  "utf8",
).trim();

async function post(path, token, body) {
  const headers = { "content-type": "application/json" };
  if (token !== undefined) headers["authorization"] = `Bearer ${token}`;
  const response = await fetch(`http://${endpointHost}${path}`, {
    method: "POST",
    headers,
    body: JSON.stringify(body),
  });
  const text = await response.text();
  let parsed;
  try {
    parsed = JSON.parse(text);
  } catch {
    parsed = undefined;
  }
  return { status: response.status, body: parsed };
}

/**
 * Registered error code only; a free-form message never leaves this function.
 * The Task routes answer with a flat `code`, the front door with a nested
 * `error.code`, so both shapes are read.
 */
function errorCodeOf(response) {
  for (const candidate of [response.body?.code, response.body?.error?.code]) {
    if (typeof candidate === "string" && /^[A-Z0-9_]+$/.test(candidate)) return candidate;
  }
  return null;
}

function elapsedSince(startedAt) {
  return Number(hrtime.bigint() - startedAt);
}

function taskContractDraft(runIndex) {
  return {
    allowed_state_domains: ["Task"],
    allowed_tools: ["tool://cognitiveos/workspace.read"],
    budget: {
      input_tokens: 4096,
      output_tokens: 1024,
      semantic_calls: 2,
      tool_calls: 8,
    },
    budget_id: `00000000-0000-7000-8000-${String(runIndex).padStart(12, "0")}`,
    conditions: [
      {
        description: "candidate must cite the fixed analysis facts",
        id: "cite-fixed-facts",
        kind: "acceptance",
      },
    ],
    deadline: "2026-08-12T23:59:59Z",
    loop_object_id: `00000000-0000-7000-9000-${String(runIndex).padStart(12, "0")}`,
    max_iterations: 1,
    max_retries: 0,
    objective: "Analyse a fixed revision read-only and propose a candidate plan",
    scope: {
      in_scope: ["read-only analysis of a fixed revision"],
      out_of_scope: ["any file mutation", "any process execution"],
    },
    task_ref: `task://local/p9-t04-t1-${runIndex}`,
  };
}

const runs = [];

for (let runIndex = 0; runIndex < startedRuns; runIndex += 1) {
  const stages = {};
  const facts = {
    executed_mutations: 0,
    independent_acceptance: false,
    self_reported_complete: false,
    admitted: false,
  };
  let terminalErrorCode = null;

  const sessionStartedAt = hrtime.bigint();
  const session = await post("/local/session", undefined, {
    channel: "task",
    principal_id: "principal://local/owner",
    bootstrap_secret: bootstrapSecret,
  });
  stages.session_mint_nanos = elapsedSince(sessionStartedAt);
  const token = typeof session.body?.token === "string" ? session.body.token : undefined;

  if (token === undefined) {
    terminalErrorCode = errorCodeOf(session) ?? "SESSION_NOT_ISSUED";
  } else {
    const recordStartedAt = hrtime.bigint();
    const recorded = await post("/task/intent.record", token, {
      conversation_or_scope_ref: CONVERSATION_REF,
      raw_expression: "Analyse the fixed revision and propose a candidate fix plan without editing files",
      schema_version: "cognitiveos.task-intent-record-request/0.1",
    });
    stages.intent_record_nanos = elapsedSince(recordStartedAt);
    const recordId = recorded.body?.user_intent_record_id;

    if (typeof recordId !== "string") {
      terminalErrorCode = errorCodeOf(recorded) ?? "INTENT_RECORD_NOT_PERSISTED";
    } else {
      const interpretStartedAt = hrtime.bigint();
      const interpreted = await post("/task/intent.interpret", token, {
        candidate: {
          ambiguities: [],
          assumptions: ["the revision is fixed and read-only"],
          constraints: ["no file may be modified"],
          forbidden: ["workspace.write", "process.run"],
          information_gaps: [],
          objectives: ["identify the failing test and propose a candidate plan"],
        },
        schema_version: "cognitiveos.task-intent-interpret-request/0.1",
        user_intent_record_id: recordId,
      });
      stages.interpret_nanos = elapsedSince(interpretStartedAt);
      const interpretationId = interpreted.body?.interpretation_id;
      // Acceptance binds to the digest of the interpretation the caller
      // reviewed, not to the later preview digest.
      const interpretationDigest = interpreted.body?.interpretation_digest;

      if (typeof interpretationId !== "string" || typeof interpretationDigest !== "string") {
        terminalErrorCode = errorCodeOf(interpreted) ?? "INTERPRETATION_NOT_PERSISTED";
      } else {
        const draft = taskContractDraft(runIndex);
        const previewStartedAt = hrtime.bigint();
        const previewed = await post("/task/preview", token, {
          schema_version: "cognitiveos.task-preview-request/0.1",
          task_contract_draft: draft,
        });
        stages.preview_nanos = elapsedSince(previewStartedAt);
        const previewDigest = previewed.body?.preview_digest;

        if (typeof previewDigest !== "string") {
          terminalErrorCode = errorCodeOf(previewed) ?? "PREVIEW_NOT_ISSUED";
        } else {
          const admitStartedAt = hrtime.bigint();
          const admitted = await post("/task/admit", token, {
            acceptance: {
              accepted_by: "principal://local/owner",
              accepted_digest: interpretationDigest,
              interpretation_id: interpretationId,
            },
            expected_current_epoch: 0,
            preview_digest: previewDigest,
            schema_version: "cognitiveos.task-admit-request/0.1",
            task_contract_draft: draft,
          });
          stages.admit_nanos = elapsedSince(admitStartedAt);
          if (admitted.status === 200) {
            facts.admitted = true;
          } else {
            terminalErrorCode = errorCodeOf(admitted) ?? "ADMISSION_REFUSED";
          }
        }
      }
    }
  }

  runs.push({
    run: runIndex,
    admission_stages: stages,
    admission_total_nanos: Object.values(stages).reduce((total, value) => total + value, 0),
    facts,
    terminal_error_code: terminalErrorCode,
  });
}

const admitted = runs.filter((run) => run.facts.admitted).length;

console.log(JSON.stringify({
  report_kind: "p9-t04-l4-t1-scenario/0.1",
  claim_level: "hypothesis",
  scenario_id: SCENARIO_ID,
  source_revision: sourceRevision,
  started_runs: startedRuns,
  retained_runs: runs.length,
  admitted_runs: admitted,
  // T1 forbids mutation and requires independent acceptance before completion.
  // Neither is asserted here; the Rust harness judges the outcome.
  verified_completions_claimed: 0,
  runs,
}, null, 2));
