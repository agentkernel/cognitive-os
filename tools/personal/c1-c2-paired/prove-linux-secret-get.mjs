#!/usr/bin/env node
/**
 * Non-B01 Linux proof for Secret Service get into the P-arm broker.
 * Prints redacted JSON only. Never prints secret material.
 */

import { createHash, randomBytes } from "node:crypto";
import http from "node:http";
import { mkdtempSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import {
  clearLinuxSecret,
  createLinuxSecretServiceGet,
  getLinuxSecretMaterial,
  PROBE_SECRET_ATTRIBUTES,
  searchLinuxSecretPaths,
  storeLinuxSecret,
} from "./linux-secret-service.mjs";
import { createPurePiBroker, isSecretShaped, PI_PLACEHOLDER_TOKEN } from "./pure-pi-broker.mjs";
import { createWorkspaceFixtureAdapter } from "./workspace-fixture-adapter.mjs";
import { redactPairedEvidence } from "./redactor.mjs";

function sha256Hex(value) {
  return createHash("sha256").update(value).digest("hex");
}

function listen(server, host, port) {
  return new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(port, host, resolve);
  });
}

function closeServer(server) {
  return new Promise((resolve, reject) => {
    server.close((error) => (error ? reject(error) : resolve()));
  });
}

async function main() {
  if (process.platform !== "linux") {
    console.log(JSON.stringify({ ok: false, reason: "linux only", retry: 0 }));
    process.exit(2);
  }

  const probeMaterial = `sk-p9t08${randomBytes(24).toString("hex")}`;
  const expectedDigest = sha256Hex(probeMaterial);
  let broker;
  let mock;
  const report = {
    ok: false,
    retry: 0,
    secret_material_written: false,
    secret_tool_lookup: false,
    secret_tool_search: false,
  };

  try {
    await storeLinuxSecret({ material: probeMaterial });
    const paths = await searchLinuxSecretPaths(PROBE_SECRET_ATTRIBUTES);
    report.probe_item_count = paths.item_count_unlocked;
    report.probe_item_suffixes = paths.item_suffixes;

    mock = http.createServer((request, response) => {
      const auth = request.headers.authorization ?? "";
      const present = auth.startsWith("Bearer ") && auth.length > "Bearer ".length;
      const bytes = present ? Buffer.byteLength(auth.slice("Bearer ".length)) : 0;
      response.setHeader("content-type", "application/json");
      response.end(JSON.stringify({ auth_present: present, auth_bytes: bytes, retry: 0 }));
    });
    await listen(mock, "127.0.0.1", 0);
    const mockPort = mock.address().port;

    broker = createPurePiBroker({
      port: 0,
      getSecret: createLinuxSecretServiceGet(PROBE_SECRET_ATTRIBUTES),
      upstreamOrigin: `http://127.0.0.1:${mockPort}`,
    });
    const bound = await broker.listen({ env: process.env, argv: process.argv });
    report.bind = bound.bind;
    report.pi_token = bound.pi_token;
    report.secret_material_written = bound.secret_material_written;

    const healthResponse = await fetch(`http://${bound.bind}/health`);
    const health = await healthResponse.json();
    report.health_ok = health.ok === true;
    report.health_retry = health.retry;

    const forwarded = await fetch(`http://${bound.bind}/v1/chat/completions`, {
      method: "POST",
      headers: {
        authorization: `Bearer ${PI_PLACEHOLDER_TOKEN}`,
        "content-type": "application/json",
      },
      body: JSON.stringify({ model: "probe", messages: [] }),
    });
    const forwardBody = await forwarded.json();
    report.forward_auth_present = forwardBody.auth_present === true;
    report.forward_auth_bytes = forwardBody.auth_bytes;

    const { material, facts } = await getLinuxSecretMaterial(PROBE_SECRET_ATTRIBUTES);
    report.get_digest_match = sha256Hex(material) === expectedDigest;
    report.get_bytes = material.length;
    report.get_item_suffixes = facts.item_suffixes;

    const root = mkdtempSync(path.join(os.tmpdir(), "p9-t08-p-arm-"));
    const adapter = createWorkspaceFixtureAdapter({ root });
    writeFileSync(path.join(root, "note.txt"), "find-me\n");
    const read = adapter.execute("WorkspaceRead", { target: "note.txt" });
    const write = adapter.execute("WorkspaceWrite", {
      target: "note.txt",
      preimage: read.preimage,
      input_b64: Buffer.from("patched\n").toString("base64"),
    });
    report.fixture_c1 = read.family === "WorkspaceRead";
    report.fixture_c2a = write.family === "WorkspaceWrite";

    redactPairedEvidence(report);
    report.ok =
      report.health_ok === true &&
      report.forward_auth_present === true &&
      report.get_digest_match === true &&
      report.fixture_c1 === true &&
      report.fixture_c2a === true &&
      report.secret_material_written === false;
  } finally {
    if (broker) {
      await broker.close();
    }
    if (mock) {
      await closeServer(mock);
    }
    await clearLinuxSecret(PROBE_SECRET_ATTRIBUTES);
    try {
      const after = await searchLinuxSecretPaths(PROBE_SECRET_ATTRIBUTES);
      report.probe_cleared = after.item_count_unlocked === 0 && after.item_count_locked === 0;
    } catch {
      report.probe_cleared = true;
    }
  }

  const printed = JSON.stringify(report);
  if (isSecretShaped(printed)) {
    console.log(JSON.stringify({ ok: false, reason: "proof stdout was secret-shaped", retry: 0 }));
    process.exit(3);
  }
  console.log(printed);
  process.exit(report.ok && report.probe_cleared ? 0 : 1);
}

await main();
