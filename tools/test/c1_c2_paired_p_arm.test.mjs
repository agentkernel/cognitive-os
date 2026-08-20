import assert from "node:assert/strict";
import http from "node:http";
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  PI_PLACEHOLDER_TOKEN,
  assertSecretFreeProcess,
  brokerThreatReview,
  createPurePiBroker,
} from "../personal/c1-c2-paired/pure-pi-broker.mjs";
import {
  WORKSPACE_PATCH_PAYLOAD,
  WORKSPACE_TOOL_SCHEMAS,
  applyUnifiedPatch,
  createWorkspaceFixtureAdapter,
} from "../personal/c1-c2-paired/workspace-fixture-adapter.mjs";
import {
  assertNonSecretAttributes,
  createLinuxSecretServiceGet,
  getLinuxSecretMaterial,
} from "../personal/c1-c2-paired/linux-secret-service.mjs";
import { checkFairness } from "../personal/c1-c2-paired/fairness-checker.mjs";
import { redactPairedEvidence } from "../personal/c1-c2-paired/redactor.mjs";
import {
  INSTRUMENT_ROOT,
  assertDisjointSeeds,
  assertNoExtraCorpusFiles,
  buildFreezeLedger,
  frozenSeeds,
  listCorpusBytes,
} from "../personal/c1-c2-paired/freeze.mjs";
import { dryRunFairness, equalArmSnapshot, FORBIDDEN_SHARED_PROMPT_PLACEHOLDER, frozenSystemTaskPromptBytes, frozenSystemTaskPromptPath, liveArmCommandManifest } from "../personal/c1-c2-paired/paired-runner.mjs";

const FIXTURE_SCHEMAS = [
  { name: "WorkspaceRead", parameters: ["target"] },
  { name: "WorkspaceSearch", parameters: ["query", "target"] },
  { name: "WorkspaceWrite", parameters: ["input_b64", "preimage", "target"] },
  { name: "WorkspacePatch", parameters: ["input_b64", "preimage", "target"] },
];

test("broker refuses a non-loopback bind host", () => {
  assert.throws(
    () => createPurePiBroker({ host: "0.0.0.0", port: 48400, getSecret: () => "k" }),
    /127\.0\.0\.1/,
  );
});

test("broker refuses secret-shaped argv", () => {
  assert.throws(
    () => assertSecretFreeProcess({ env: {}, argv: ["node", "broker", "sk-abcdefghijklmnop"] }),
    /argv/,
  );
});

test("broker refuses secret-shaped environment values", () => {
  assert.throws(
    () =>
      assertSecretFreeProcess({
        env: { PATH: "/usr/bin", PROVIDER_API_KEY: "sk-abcdefghijklmnop" },
        argv: ["node"],
      }),
    /env PROVIDER_API_KEY/,
  );
});

test("broker bind does not retain or return secret material", async () => {
  const material = "sk-fixture-material-not-for-logs";
  const broker = createPurePiBroker({
    port: 48400,
    getSecret: () => material,
  });
  const bound = await broker.bind({ env: { PATH: "/usr/bin" }, argv: ["node"] });
  assert.equal(bound.pi_token, PI_PLACEHOLDER_TOKEN);
  assert.equal(bound.secret_material_written, false);
  assert.equal(bound.retry, 0);
  assert.equal(broker.hasRetainedSecretMaterial(), false);
  assert.doesNotMatch(JSON.stringify(broker), /sk-fixture/);
  assert.doesNotMatch(JSON.stringify(bound), /sk-fixture/);
  assert.equal("context" in broker, false);
  assert.equal("memory" in broker, false);
  assert.equal("task" in broker, false);
  assert.equal("retry" in broker, false);
  assert.equal("verify" in broker, false);
  assert.equal(brokerThreatReview().items.length, 8);
  assert.equal(brokerThreatReview().retry, 0);
});

test("P-arm fixture schemas match O-arm Workspace* names and parameter keys", () => {
  assert.deepEqual(WORKSPACE_TOOL_SCHEMAS, FIXTURE_SCHEMAS);
});

test("C1 WorkspaceRead refuses fixture-root escape", () => {
  const root = mkdtempSync(path.join(os.tmpdir(), "c1-c2-p-arm-"));
  const adapter = createWorkspaceFixtureAdapter({ root });
  assert.throws(() => adapter.execute("WorkspaceRead", { target: "../secret" }), /escaped/);
});

test("C1 WorkspaceRead and C2a WorkspaceWrite complete against a fixture root", () => {
  const root = mkdtempSync(path.join(os.tmpdir(), "c1-c2-p-arm-"));
  const adapter = createWorkspaceFixtureAdapter({ root });
  writeFileSync(path.join(root, "note.txt"), "find-me\n");

  const read = adapter.execute("WorkspaceRead", { target: "note.txt" });
  assert.equal(read.family, "WorkspaceRead");
  assert.equal(read.bytes.toString("utf8"), "find-me\n");

  const search = adapter.execute("WorkspaceSearch", { target: "note.txt", query: "find-me" });
  assert.equal(search.hits.length, 1);

  const write = adapter.execute("WorkspaceWrite", {
    target: "note.txt",
    preimage: read.preimage,
    input_b64: Buffer.from("patched\n").toString("base64"),
  });
  assert.equal(write.family, "WorkspaceWrite");
  const after = adapter.execute("WorkspaceRead", { target: "note.txt" });
  assert.equal(after.bytes.toString("utf8"), "patched\n");
});

test("C2a WorkspacePatch refuses replacement bytes and applies a unified diff", () => {
  const root = mkdtempSync(path.join(os.tmpdir(), "c1-c2-p-arm-"));
  const adapter = createWorkspaceFixtureAdapter({ root });
  writeFileSync(path.join(root, "note.txt"), "patched\n");
  const before = adapter.execute("WorkspaceRead", { target: "note.txt" });
  assert.throws(
    () =>
      adapter.execute("WorkspacePatch", {
        target: "note.txt",
        preimage: before.preimage,
        input_b64: Buffer.from("reseeded\n").toString("base64"),
      }),
    /unexpected line outside a hunk/,
  );
  assert.equal(readFileSync(path.join(root, "note.txt"), "utf8"), "patched\n");

  const patch = adapter.execute("WorkspacePatch", {
    target: "note.txt",
    preimage: before.preimage,
    input_b64: Buffer.from("@@ -1 +1 @@\n-patched\n+reseeded\n").toString("base64"),
  });
  assert.equal(patch.family, "WorkspacePatch");
  const patched = adapter.execute("WorkspaceRead", { target: "note.txt" });
  assert.equal(patched.bytes.toString("utf8"), "reseeded\n");
});

test("C2a frozen unified diff repairs the C2a corpus to the oracle", () => {
  const root = mkdtempSync(path.join(os.tmpdir(), "c1-c2-c2a-"));
  const adapter = createWorkspaceFixtureAdapter({ root });
  const source = readFileSync(path.join(INSTRUMENT_ROOT, "fixtures/c2a/workspace/src/repair.ts"));
  const diff = readFileSync(path.join(INSTRUMENT_ROOT, "fixtures/c2a/workspace-patch.unified.diff"));
  const oracle = JSON.parse(
    readFileSync(path.join(INSTRUMENT_ROOT, "fixtures/c2a/oracle.json"), "utf8"),
  );
  mkdirSync(path.join(root, "src"), { recursive: true });
  writeFileSync(path.join(root, "src/repair.ts"), source);
  const before = adapter.execute("WorkspaceRead", { target: "src/repair.ts" });
  adapter.execute("WorkspacePatch", {
    target: "src/repair.ts",
    preimage: before.preimage,
    input_b64: diff.toString("base64"),
  });
  const after = adapter.execute("WorkspaceRead", { target: "src/repair.ts" });
  assert.equal(after.bytes.toString("utf8"), oracle.expected_source);
  assert.equal(WORKSPACE_PATCH_PAYLOAD, "unified-diff");
});

test("applyUnifiedPatch matches daemon no-newline marker cases and fails closed", () => {
  assert.equal(
    applyUnifiedPatch("final line", "@@ -1 +1 @@\n-final line\n\\ No newline at end of file\n+final line\n"),
    "final line\n",
  );
  assert.equal(
    applyUnifiedPatch("final line\n", "@@ -1 +1 @@\n-final line\n+final line\n\\ No newline at end of file\n"),
    "final line",
  );
  assert.throws(
    () =>
      applyUnifiedPatch(
        "final line\n",
        "@@ -1 +1 @@\n-final line\n\\ No newline at end of file\n+changed\n",
      ),
    /old-side no-newline marker contradicts the preimage/,
  );
  assert.throws(() => applyUnifiedPatch("a\n", ""), /patch contains no hunk/);
});

test("C2a WorkspaceWrite fails closed on preimage mismatch", () => {
  const root = mkdtempSync(path.join(os.tmpdir(), "c1-c2-p-arm-"));
  const adapter = createWorkspaceFixtureAdapter({ root });
  writeFileSync(path.join(root, "note.txt"), "v1\n");
  assert.throws(
    () =>
      adapter.execute("WorkspaceWrite", {
        target: "note.txt",
        preimage: "digest:sha256:" + "0".repeat(64),
        input_b64: Buffer.from("v2\n").toString("base64"),
      }),
    /preimage mismatch/,
  );
});

test("Linux Secret Service get fails closed off Linux and on secret-shaped attributes", async () => {
  assert.throws(
    () => assertNonSecretAttributes({ purpose: "sk-abcdefghijklmnop" }),
    /secret-shaped attribute/,
  );
  if (process.platform !== "linux") {
    await assert.rejects(getLinuxSecretMaterial(), /linux secret service is not available/);
    const getter = createLinuxSecretServiceGet();
    await assert.rejects(getter(), /linux secret service is not available/);
  }
});

test("broker injects upstream auth in memory without logging material", async () => {
  const material = "sk-fixture-material-not-for-logs";
  const mock = http.createServer((request, response) => {
    const auth = request.headers.authorization ?? "";
    const present = auth.startsWith("Bearer ") && auth.length > "Bearer ".length;
    const bytes = present ? Buffer.byteLength(auth.slice("Bearer ".length)) : 0;
    response.setHeader("content-type", "application/json");
    response.end(JSON.stringify({ auth_present: present, auth_bytes: bytes, retry: 0 }));
  });
  await new Promise((resolve, reject) => {
    mock.once("error", reject);
    mock.listen(0, "127.0.0.1", resolve);
  });
  const mockPort = mock.address().port;
  const broker = createPurePiBroker({
    port: 0,
    getSecret: () => material,
    upstreamOrigin: `http://127.0.0.1:${mockPort}`,
  });
  try {
    const bound = await broker.listen({ env: { PATH: "/usr/bin" }, argv: ["node"] });
    const health = await (await fetch(`http://${bound.bind}/health`)).json();
    assert.equal(health.ok, true);
    assert.equal(health.retry, 0);
    const unauthorized = await fetch(`http://${bound.bind}/v1/chat/completions`, {
      method: "POST",
      headers: { authorization: "Bearer wrong", "content-type": "application/json" },
      body: "{}",
    });
    assert.equal(unauthorized.status, 401);
    const forwarded = await fetch(`http://${bound.bind}/v1/chat/completions`, {
      method: "POST",
      headers: {
        authorization: `Bearer ${PI_PLACEHOLDER_TOKEN}`,
        "content-type": "application/json",
      },
      body: "{}",
    });
    const body = await forwarded.json();
    assert.equal(body.auth_present, true);
    assert.equal(body.auth_bytes, Buffer.byteLength(material));
    assert.equal(broker.stats().forwards, 1);
    assert.equal(broker.stats().retry, 0);
    assert.doesNotMatch(JSON.stringify(health), /sk-fixture/);
    assert.doesNotMatch(JSON.stringify(body), /sk-fixture/);
    assert.doesNotMatch(JSON.stringify(bound), /sk-fixture/);
  } finally {
    await broker.close();
    await new Promise((resolve, reject) => mock.close((error) => (error ? reject(error) : resolve())));
  }
});

test("system_task_prompt_bytes is the frozen prompt length, not a shared placeholder", () => {
  const snapshot = equalArmSnapshot();
  assert.notEqual(snapshot.system_task_prompt_bytes, FORBIDDEN_SHARED_PROMPT_PLACEHOLDER);
  assert.equal(typeof snapshot.system_task_prompt_bytes, "number");
  assert.ok(snapshot.system_task_prompt_bytes > 0);
  const p = equalArmSnapshot();
  const o = equalArmSnapshot();
  assert.equal(p.system_task_prompt_bytes, o.system_task_prompt_bytes);
});

test("live P and O command manifests share --append-system-prompt and the freeze file", () => {
  const promptPath = frozenSystemTaskPromptPath();
  const manifest = liveArmCommandManifest();
  assert.equal(manifest.system_task_prompt_bytes, frozenSystemTaskPromptBytes());
  assert.deepEqual(manifest.p, ["pi", "--print", "--append-system-prompt", promptPath]);
  assert.deepEqual(manifest.o, [
    "cognitive",
    "pi",
    "launch",
    "--print",
    "--append-system-prompt",
    promptPath,
  ]);
  assert.equal(manifest.p[manifest.p.length - 1], manifest.o[manifest.o.length - 1]);
});

test("fairness checker passes equal arms and fails a missing or mutated axis", () => {
  const snapshot = equalArmSnapshot();
  const pass = checkFairness({ p: snapshot, o: snapshot });
  assert.equal(pass.result, "pass");
  assert.equal(pass.b0, false);
  assert.equal(pass.retry, 0);
  const fail = checkFairness({
    p: snapshot,
    o: equalArmSnapshot({ visible_tool_set_schema: [] }),
  });
  assert.equal(fail.result, "fail");
  assert.equal(
    fail.axes.find((row) => row.axis === "visible_tool_set_schema").status,
    "fail_mismatch",
  );
  const missing = checkFairness({ p: snapshot, o: {} });
  assert.equal(missing.result, "fail");
  assert.equal(missing.failed_axes, 13);
});

test("redactor rejects unredacted secret-shaped evidence", () => {
  assert.throws(() => redactPairedEvidence("Authorization: Bearer sk-abcdefghijklmnop"), /unredacted/);
  const ok = redactPairedEvidence({ pi_token: PI_PLACEHOLDER_TOKEN, retry: 0 });
  assert.equal(ok.redacted, true);
});

test("freeze ledger has disjoint seeds, retry=0, and secret-free corpus", async () => {
  const seeds = frozenSeeds();
  assert.equal(seeds.retry, 0);
  assert.equal(seeds.b0.C1.length, 1);
  assert.equal(seeds.b1.C2a.length, 5);
  assert.equal(seeds.b2.C2d.length, 30);
  assert.equal(assertDisjointSeeds(seeds), 1 * 5 + 5 * 5 + 30 * 5);
  await assertNoExtraCorpusFiles();
  const bytes = await listCorpusBytes();
  assert.equal(Object.keys(bytes).length, 8);
  const ledger = await buildFreezeLedger();
  assert.equal(ledger.retry, 0);
  assert.equal(ledger.b0, false);
  assert.equal(ledger.command_manifest.retry, 0);
  assert.equal(ledger.command_manifest.append_system_prompt, "frozen-system-task-prompt.txt");
  assert.equal(ledger.command_manifest.workspace_patch_payload, "unified-diff");
  assert.match(ledger.files["pure-pi-broker.mjs"], /^sha256:[a-f0-9]{64}$/);
});

test("paired runner dry-run emits a non-B0 fairness record", async () => {
  const pass = await dryRunFairness();
  assert.equal(pass.fairness.result, "pass");
  assert.equal(pass.b0, false);
  assert.equal(pass.retry, 0);
  const fail = await dryRunFairness({ mutateAxis: "oracle_version" });
  assert.equal(fail.fairness.result, "fail");
});
