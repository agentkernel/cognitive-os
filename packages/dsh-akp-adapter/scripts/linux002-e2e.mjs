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
import { admitTask, httpJson, issueToken, waitLifecycle } from "./daemon-task.mjs";

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
  const lifecycle = await waitLifecycle(origin, token, spec.taskRef, { want: "COMPLETED" });
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
  const digestSession = "dsh-negative-digest";
  await transport.activate({
    dshVersion: PINNED_DSH_REVISION,
    sessionId: digestSession,
    fencingEpoch: 1,
    taskRef,
  });
  const wrongDigest = await httpJson(origin, "POST", "/task/akp/dsh", token, {
    op: "event",
    bridge_protocol: "cognitiveos.dsh-akp/0.1",
    dsh_version: PINNED_DSH_REVISION,
    schema_digest: "sha256:0000000000000000000000000000000000000000000000000000000000000000",
    session_id: digestSession,
    fencing_epoch: 1,
    sequence: 1,
    plugin_id: "plugin.core",
    correlation_id: `${digestSession}:1`,
    deadline: "2030-01-01T00:00:00.000Z",
    event: { kind: "lifecycle", operation: "adapter.ready", payload: { ok: true }, authority_claim: false, secret_shaped: false },
  });
  const protocolSession = "dsh-negative-protocol";
  await transport.activate({
    dshVersion: PINNED_DSH_REVISION,
    sessionId: protocolSession,
    fencingEpoch: 1,
    taskRef,
  });
  const wrongProtocol = await httpJson(origin, "POST", "/task/akp/dsh", token, {
    op: "event",
    bridge_protocol: "cognitiveos.dsh-akp/9.9",
    dsh_version: PINNED_DSH_REVISION,
    schema_digest: PINNED_AKP_SCHEMA_DIGEST,
    session_id: protocolSession,
    fencing_epoch: 1,
    sequence: 1,
    plugin_id: "plugin.core",
    correlation_id: `${protocolSession}:1`,
    deadline: "2030-01-01T00:00:00.000Z",
    event: { kind: "lifecycle", operation: "adapter.ready", payload: { ok: true }, authority_claim: false, secret_shaped: false },
  });
  const epochSession = "dsh-negative-epoch";
  await transport.activate({
    dshVersion: PINNED_DSH_REVISION,
    sessionId: epochSession,
    fencingEpoch: 1,
    taskRef,
  });
  const staleEpoch = await httpJson(origin, "POST", "/task/akp/dsh", token, {
    op: "event",
    bridge_protocol: "cognitiveos.dsh-akp/0.1",
    dsh_version: PINNED_DSH_REVISION,
    schema_digest: PINNED_AKP_SCHEMA_DIGEST,
    session_id: epochSession,
    fencing_epoch: 9,
    sequence: 1,
    plugin_id: "plugin.core",
    correlation_id: `${epochSession}:1`,
    deadline: "2030-01-01T00:00:00.000Z",
    event: { kind: "lifecycle", operation: "adapter.ready", payload: { ok: true }, authority_claim: false, secret_shaped: false },
  });
  const dupSession = "dsh-negative-dup";
  await transport.activate({
    dshVersion: PINNED_DSH_REVISION,
    sessionId: dupSession,
    fencingEpoch: 1,
    taskRef,
  });
  const firstDup = {
    op: "event",
    bridge_protocol: "cognitiveos.dsh-akp/0.1",
    dsh_version: PINNED_DSH_REVISION,
    schema_digest: PINNED_AKP_SCHEMA_DIGEST,
    session_id: dupSession,
    fencing_epoch: 1,
    sequence: 1,
    plugin_id: "plugin.core",
    correlation_id: `${dupSession}:1`,
    deadline: "2030-01-01T00:00:00.000Z",
    event: { kind: "lifecycle", operation: "adapter.ready", payload: { ok: true }, authority_claim: false, secret_shaped: false },
  };
  await httpJson(origin, "POST", "/task/akp/dsh", token, firstDup);
  const duplicateSequence = await httpJson(origin, "POST", "/task/akp/dsh", token, firstDup);
  const malformed = await fetch(`${origin}/task/akp/dsh`, {
    method: "POST",
    headers: { "content-type": "application/json", authorization: `Bearer ${token}` },
    body: "{not-json",
  });
  const malformedJson = await malformed.json().catch(() => ({ parse_error: true }));
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
    wrongDigestAccepted: wrongDigest.json?.accepted === true,
    wrongDigestError: wrongDigest.json?.error ?? null,
    wrongProtocolAccepted: wrongProtocol.json?.accepted === true,
    wrongProtocolError: wrongProtocol.json?.error ?? null,
    staleEpochAccepted: staleEpoch.json?.accepted === true,
    staleEpochError: staleEpoch.json?.error ?? null,
    duplicateSequenceAccepted: duplicateSequence.json?.accepted === true,
    duplicateSequenceError: duplicateSequence.json?.error ?? null,
    malformedAccepted: malformedJson?.accepted === true,
    malformedError: malformedJson?.error ?? malformedJson?.code ?? null,
    secretShaped: secretError,
  };
}

const token = await issueToken(origin, secret, "task");
const stamp = `${process.pid}`;
const readSpec = {
  family: "read",
  tool: "native.workspace.read",
  conversation: "conversation://personal/p8-t09",
  objective: "read README.md",
  taskRef: `task://personal/p8-t09-dsh-read-${stamp}`,
  sessionId: `dsh-session-read-${stamp}`,
};
const searchSpec = {
  family: "search",
  tool: "native.workspace.search",
  conversation: "conversation://personal/p8-t09-search",
  objective: "search the workspace for needle",
  taskRef: `task://personal/p8-t09-dsh-search-${stamp}`,
  sessionId: `dsh-session-search-${stamp}`,
};
const writeSpec = {
  family: "write",
  tool: "native.workspace.write",
  conversation: "conversation://personal/p8-t09-write",
  objective: "mutate workspace through daemon-governed WorkspaceWrite",
  taskRef: `task://personal/p8-t09-dsh-write-${stamp}`,
  sessionId: `dsh-session-write-${stamp}`,
};

await admitTask(origin, token, readSpec);
await admitTask(origin, token, searchSpec);
await admitTask(origin, token, writeSpec);

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
    target: `p8-t09-write-${stamp}.txt`,
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
  read.lifecycle !== "COMPLETED" ||
  !search.accepted ||
  search.lifecycle !== "COMPLETED" ||
  !write.accepted ||
  write.lifecycle !== "COMPLETED" ||
  negative.wrongVersionAccepted !== false ||
  negative.wrongDigestAccepted !== false ||
  negative.wrongProtocolAccepted !== false ||
  negative.staleEpochAccepted !== false ||
  negative.duplicateSequenceAccepted !== false ||
  negative.malformedAccepted !== false ||
  negative.secretShaped !== "SECRET_SHAPED_PAYLOAD";
process.exit(failed ? 1 : 0);
