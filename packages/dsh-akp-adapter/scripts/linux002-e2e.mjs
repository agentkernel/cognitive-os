#!/usr/bin/env node
/**
 * linux-002 / exact-revision dsh AKP E2E harness (P8-T09).
 *
 * Path: attachDshCordisPlugin → HttpAkpTransport → POST /task/akp/dsh →
 * public candidate admission. A dsh response is never Task completion.
 * Prints redacted facts only: no bearer, bootstrap secret, or Provider key.
 */
import { readFileSync } from "node:fs";
import {
  DshAkpAdapter,
  HttpAkpTransport,
  PINNED_AKP_SCHEMA_DIGEST,
  PINNED_DSH_REVISION,
  attachDshCordisPlugin,
} from "../dist/index.js";

function arg(name, fallback) {
  const index = process.argv.indexOf(name);
  if (index >= 0 && process.argv[index + 1]) return process.argv[index + 1];
  return fallback;
}

const port = Number(arg("--port", "48509"));
const bootstrapPath = arg("--bootstrap-file");
const revisionPin = arg("--revision");
if (!Number.isInteger(port) || port < 1 || !bootstrapPath) {
  throw new Error("--port and --bootstrap-file are required");
}

const origin = `http://127.0.0.1:${port}`;
const secret = readFileSync(bootstrapPath, "utf8").trim();

function uuid7Like(kind) {
  const n = (Date.now() ^ process.pid ^ kind.length) >>> 0;
  const suffix = n.toString(16).padStart(12, "0").slice(-12);
  const variant = kind === "budget" ? "8" : "9";
  return `00000000-0000-7000-${variant}000-${suffix}`;
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
    bootstrap_secret: secret,
  });
  const token = json.token;
  if (typeof token !== "string" || !token) {
    throw new Error(`session token missing for ${channel}`);
  }
  return token;
}

async function admitTask(token, spec) {
  const recorded = await httpJson("POST", "/task/intent.record", token, {
    conversation_or_scope_ref: spec.conversation,
    raw_expression: spec.objective,
    schema_version: "cognitiveos.task-intent-record-request/0.1",
  });
  const interpreted = await httpJson("POST", "/task/intent.interpret", token, {
    schema_version: "cognitiveos.task-intent-interpret-request/0.1",
    user_intent_record_id: recorded.json.user_intent_record_id,
    candidate: {
      objectives: [spec.objective],
      constraints: [],
      forbidden: ["bash", "edit", "write"],
      assumptions: [],
      ambiguities: [],
      information_gaps: [],
    },
  });
  const draft = {
    allowed_state_domains: ["task", "effect"],
    allowed_tools: [spec.tool],
    budget: { semantic_calls: 4, tool_calls: 4 },
    budget_id: uuid7Like(`budget-${spec.family}`),
    conditions: [
      {
        description: "independent fixed-effect verification",
        id: "acceptance",
        kind: "acceptance",
        verifier_ref: "verifier://personal/fixed-effect",
      },
    ],
    deadline: "2027-12-31T00:00:00Z",
    loop_object_id: uuid7Like(`loop-${spec.family}`),
    max_iterations: 4,
    max_retries: 0,
    objective: spec.objective,
    scope: {
      in_scope: [`workspace ${spec.family}`],
      out_of_scope: ["bash", "edit", "write"],
    },
    task_ref: spec.taskRef,
  };
  const previewed = await httpJson("POST", "/task/preview", token, {
    schema_version: "cognitiveos.task-preview-request/0.1",
    task_contract_draft: draft,
  });
  const admitted = await httpJson("POST", "/task/admit", token, {
    schema_version: "cognitiveos.task-admit-request/0.1",
    expected_current_epoch: 0,
    preview_digest: previewed.json.preview_digest,
    task_contract_draft: draft,
    acceptance: {
      accepted_by: "principal://local/owner",
      accepted_digest: interpreted.json.interpretation_digest,
      interpretation_id: interpreted.json.interpretation_id,
    },
  });
  if (admitted.json.task_ref !== spec.taskRef) {
    throw new Error(`admit failed for ${spec.family}: ${JSON.stringify(admitted.json)}`);
  }
  return spec.taskRef;
}

async function waitLifecycle(token, taskRef) {
  const encoded = encodeURIComponent(taskRef);
  let lifecycle = "absent";
  for (let attempt = 0; attempt < 24; attempt += 1) {
    const evidence = await httpJson("GET", `/task/evidence?task_ref=${encoded}`, token);
    lifecycle = evidence.json?.lifecycle?.current_state ?? "absent";
    if (lifecycle !== "DRAFT" && lifecycle !== "absent") {
      return lifecycle;
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  return lifecycle;
}

function pluginHost() {
  const listeners = [];
  return {
    on(_event, listener) {
      listeners.push(listener);
    },
    emit(payload) {
      for (const listener of listeners) listener(payload);
    },
  };
}

async function runFamily(token, spec, event) {
  const host = pluginHost();
  const transport = new HttpAkpTransport({
    endpoint: `${origin}/task/akp/dsh`,
    bearer: token,
  });
  const pending = [];
  const adapter = attachDshCordisPlugin(host, {
    dshVersion: PINNED_DSH_REVISION,
    schemaDigest: PINNED_AKP_SCHEMA_DIGEST,
    sessionId: spec.sessionId,
    pluginId: spec.sessionId,
    taskRef: spec.taskRef,
    transport,
    timeoutMs: 15_000,
    onResult: (result) => pending.push({ ok: true, result }),
    onError: (error) => pending.push({ ok: false, error: error instanceof Error ? error.message : String(error) }),
  });
  host.emit(event);
  const started = Date.now();
  while (pending.length === 0 && Date.now() - started < 15_000) {
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  adapter.stop();
  const outcome = pending[0];
  if (!outcome?.ok) {
    return { family: spec.family, accepted: false, error: outcome?.error ?? "no adapter result" };
  }
  const lifecycle = await waitLifecycle(token, spec.taskRef);
  return {
    family: spec.family,
    accepted: outcome.result.response.accepted === true,
    candidateOnly: outcome.result.response.candidateOnly === true,
    error: outcome.result.response.error ?? null,
    admitted: outcome.result.response.result?.admission?.admitted === true,
    lifecycle,
    serializationNanos: outcome.result.timing.serializationNanos,
    transportNanos: outcome.result.timing.transportNanos,
    totalNanos: outcome.result.timing.totalNanos,
  };
}

async function negatives(token, taskRef) {
  const transport = new HttpAkpTransport({
    endpoint: `${origin}/task/akp/dsh`,
    bearer: token,
  });
  const wrongVersion = await transport.activate({
    dshVersion: "0.0.0",
    sessionId: "dsh-negative-version",
    fencingEpoch: 1,
    taskRef,
  });
  const adapter = new DshAkpAdapter({
    dshVersion: PINNED_DSH_REVISION,
    schemaDigest: PINNED_AKP_SCHEMA_DIGEST,
    sessionId: "dsh-negative-secret",
    pluginId: "plugin.core",
    taskRef,
    transport,
  });
  await transport.activate({
    dshVersion: PINNED_DSH_REVISION,
    sessionId: "dsh-negative-secret",
    fencingEpoch: 1,
    taskRef,
  });
  adapter.activate();
  let secretError = "missing";
  try {
    await adapter.submit({
      kind: "observation",
      operation: "lifecycle.observe",
      payload: { api_key: "sk-example" },
    });
  } catch (error) {
    secretError = error instanceof Error && "code" in error ? String(error.code) : "other";
  }
  return {
    wrongVersionAccepted: wrongVersion.accepted,
    wrongVersionError: wrongVersion.error ?? null,
    secretShaped: secretError,
  };
}

const token = await issueToken("task");
const readSpec = {
  family: "read",
  tool: "native.workspace.read",
  conversation: "conversation://personal/p8-t09",
  objective: "read README.md",
  taskRef: "task://personal/p8-t09-dsh-read",
  sessionId: "dsh-session-read",
};
const searchSpec = {
  family: "search",
  tool: "native.workspace.search",
  conversation: "conversation://personal/p8-t09-search",
  objective: "search the workspace for needle",
  taskRef: "task://personal/p8-t09-dsh-search",
  sessionId: "dsh-session-search",
};
const writeSpec = {
  family: "write",
  tool: "native.workspace.write",
  conversation: "conversation://personal/p8-t09-write",
  objective: "mutate workspace through daemon-governed WorkspaceWrite",
  taskRef: "task://personal/p8-t09-dsh-write",
  sessionId: "dsh-session-write",
};

await admitTask(token, readSpec);
await admitTask(token, searchSpec);
await admitTask(token, writeSpec);

const read = await runFamily(token, readSpec, {
  kind: "candidate",
  operation: "WorkspaceRead",
  payload: { target: "README.md" },
});
const search = await runFamily(token, searchSpec, {
  kind: "candidate",
  operation: "WorkspaceSearch",
  payload: { query: "needle" },
});
const write = await runFamily(token, writeSpec, {
  kind: "candidate",
  operation: "WorkspaceWrite",
  payload: {
    target: "p8-t09-write.txt",
    input_b64: Buffer.from("p8-t09 disposable write\n", "utf8").toString("base64"),
    preimage: "absent",
  },
});
const negative = await negatives(token, readSpec.taskRef);

const summary = {
  revision_pin: revisionPin ?? null,
  guest_port: port,
  adapter: "attachDshCordisPlugin+HttpAkpTransport",
  candidate_only: true,
  dsh_response_is_not_task_completion: true,
  read,
  search,
  write,
  negatives: negative,
  non_claims: ["Gate", "release", "Profile", "B01", "Agent-benefit"],
};
process.stdout.write(`${JSON.stringify(summary)}\n`);
const failed =
  !read.accepted ||
  !search.accepted ||
  !write.accepted ||
  negative.wrongVersionAccepted !== false ||
  negative.secretShaped !== "SECRET_SHAPED_PAYLOAD";
process.exit(failed ? 1 : 0);
