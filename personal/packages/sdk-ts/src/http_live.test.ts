/**
 * Optional live integration against Lane-RUN `kernel-server --once`.
 *
 * Skipped unless `KERNEL_SERVER_BIN` points at a built binary (or the
 * workspace `target/debug/kernel-server(.exe)` exists after a local cargo
 * build). Clients never become authority: we only assert transport routes
 * and non-authority shell markers.
 */

import assert from "node:assert/strict";
import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { createServer } from "node:net";
import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import { akpRequestEnvelope } from "@cognitiveos/contracts-ts";
import { buildRequestEnvelope, serializeEnvelope } from "./envelope.js";
import { HttpSseTransport } from "./transport.js";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const DEFAULT_BIN = process.platform === "win32"
  ? path.join(ROOT, "target", "debug", "kernel-server.exe")
  : path.join(ROOT, "target", "debug", "kernel-server");
const BIN = process.env.KERNEL_SERVER_BIN ?? DEFAULT_BIN;
const LIVE = existsSync(BIN);

function freePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (address === null || typeof address === "string") {
        server.close();
        reject(new Error("no port"));
        return;
      }
      const port = address.port;
      server.close((err) => (err ? reject(err) : resolve(port)));
    });
  });
}

async function withServer<T>(fn: (baseUrl: string) => Promise<T>): Promise<T> {
  const port = await freePort();
  const child: ChildProcessWithoutNullStreams = spawn(BIN, ["--once", "--bind", `127.0.0.1:${port}`], {
    stdio: "pipe",
  });
  const baseUrl = `http://127.0.0.1:${port}`;
  // Give the listener a moment; retries happen inside fetch loops below.
  await new Promise((r) => setTimeout(r, 80));
  try {
    return await fn(baseUrl);
  } finally {
    child.kill();
  }
}

test(
  "live: management inspect posts AKP envelope to /management/inspect",
  { skip: !LIVE },
  async () => {
    await withServer(async (baseUrl) => {
      const transport = new HttpSseTransport({
        baseUrl,
        channel: "management",
        bearer: "mgmt-live",
      });
      const envelope = buildRequestEnvelope({
        operation: "management.inspect",
        kind: "read",
        sender: "principal://a",
        audience: "service://kernel/management",
        correlationId: "c-live",
        deadline: "2026-07-21T01:00:00Z",
        schemaDigest: akpRequestEnvelope.SCHEMA_DIGEST,
        payload: { target: "agent-execution://1" },
        messageId: "m-live",
      });
      let reply = { transportStatus: 0, body: "" };
      for (let i = 0; i < 40; i += 1) {
        try {
          reply = await transport.request(serializeEnvelope(envelope));
          break;
        } catch {
          await new Promise((r) => setTimeout(r, 25));
        }
      }
      assert.equal(reply.transportStatus, 200);
      assert.match(reply.body, /"status":"ok"/);
      assert.match(reply.body, /management_ready/);
    });
  },
);

test(
  "live: task shell detach is non-authority and does not claim cancel",
  { skip: !LIVE },
  async () => {
    await withServer(async (baseUrl) => {
      const transport = new HttpSseTransport({
        baseUrl,
        channel: "task",
        bearer: "task-live",
      });
      const envelope = buildRequestEnvelope({
        operation: "shell.detach",
        kind: "read",
        sender: "principal://a",
        audience: "kernel://node",
        correlationId: "c-detach",
        deadline: "2026-07-21T01:00:00Z",
        schemaDigest: akpRequestEnvelope.SCHEMA_DIGEST,
        payload: { task_ref: "task://t1" },
        messageId: "m-detach",
      });
      let reply = { transportStatus: 0, body: "" };
      for (let i = 0; i < 40; i += 1) {
        try {
          reply = await transport.request(serializeEnvelope(envelope));
          break;
        } catch {
          await new Promise((r) => setTimeout(r, 25));
        }
      }
      assert.equal(reply.transportStatus, 200);
      assert.match(reply.body, /"cancelled":false/);
      assert.match(reply.body, /"authority":false/);
    });
  },
);

test(
  "live: task watch stream yields snapshot then delta frames",
  { skip: !LIVE },
  async () => {
    await withServer(async (baseUrl) => {
      const transport = new HttpSseTransport({
        baseUrl,
        channel: "task",
        bearer: "task-live",
      });
      const open = buildRequestEnvelope({
        operation: "watch.open",
        kind: "read",
        sender: "principal://a",
        audience: "kernel://node",
        correlationId: "c-watch",
        deadline: "2026-07-21T01:00:00Z",
        schemaDigest: akpRequestEnvelope.SCHEMA_DIGEST,
        payload: { cursor: null },
        messageId: "m-watch",
      });
      const frames: string[] = [];
      for (let attempt = 0; attempt < 40 && frames.length === 0; attempt += 1) {
        try {
          for await (const frame of transport.openStream(serializeEnvelope(open))) {
            frames.push(frame);
          }
        } catch {
          await new Promise((r) => setTimeout(r, 25));
        }
      }
      assert.ok(frames.some((f) => f.includes('"kind":"snapshot"')));
      assert.ok(frames.some((f) => f.includes('"kind":"delta"')));
    });
  },
);
