import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  PI_PLACEHOLDER_TOKEN,
  assertSecretFreeProcess,
  createPurePiBroker,
} from "../personal/c1-c2-paired/pure-pi-broker.mjs";
import {
  WORKSPACE_TOOL_SCHEMAS,
  createWorkspaceFixtureAdapter,
} from "../personal/c1-c2-paired/workspace-fixture-adapter.mjs";

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

test("broker bind does not retain or return secret material", () => {
  const material = "sk-fixture-material-not-for-logs";
  const broker = createPurePiBroker({
    port: 48400,
    getSecret: () => material,
  });
  const bound = broker.bind({ env: { PATH: "/usr/bin" }, argv: ["node"] });
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

  const patch = adapter.execute("WorkspacePatch", {
    target: "note.txt",
    preimage: after.preimage,
    input_b64: Buffer.from("reseeded\n").toString("base64"),
  });
  assert.equal(patch.family, "WorkspacePatch");
  const patched = adapter.execute("WorkspaceRead", { target: "note.txt" });
  assert.equal(patched.bytes.toString("utf8"), "reseeded\n");
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
