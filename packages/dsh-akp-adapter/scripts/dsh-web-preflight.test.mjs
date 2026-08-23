import test from "node:test";
import assert from "node:assert/strict";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  DEFAULT_WEB_HOST,
  DEFAULT_WEB_PORT,
  assertFrontendDist,
  assertLoopbackHost,
  assertWebPort,
  frontendDistIndex,
  listenUrl,
} from "./dsh-web-preflight.mjs";

test("loopback hosts are accepted and 0.0.0.0 is refused", () => {
  assert.equal(assertLoopbackHost("127.0.0.1"), "127.0.0.1");
  assert.equal(assertLoopbackHost("127.0.0.8"), "127.0.0.8");
  assert.equal(assertLoopbackHost("localhost"), DEFAULT_WEB_HOST);
  assert.equal(assertLoopbackHost("::1"), "::1");
  assert.throws(() => assertLoopbackHost("0.0.0.0"), /loopback|refused/);
  assert.throws(() => assertLoopbackHost("::"), /loopback|refused/);
  assert.throws(() => assertLoopbackHost("192.168.1.2"), /loopback/);
  assert.throws(() => assertLoopbackHost(""), /loopback/);
});

test("web port rejects zero and non-integers", () => {
  assert.equal(assertWebPort(DEFAULT_WEB_PORT), 3080);
  assert.equal(assertWebPort("3080"), 3080);
  assert.throws(() => assertWebPort(0), /1\.\.65535/);
  assert.throws(() => assertWebPort("nope"), /1\.\.65535/);
});

test("missing frontend dist fails closed", () => {
  const root = join(tmpdir(), `p8t15-dsh-web-preflight-${process.pid}`);
  rmSync(root, { recursive: true, force: true });
  mkdirSync(root, { recursive: true });
  assert.throws(() => assertFrontendDist(root), /frontend dist is missing/);
  const index = frontendDistIndex(root);
  mkdirSync(join(root, "apps/web/dist"), { recursive: true });
  writeFileSync(index, "<!doctype html><title>DeepSeek Harness</title>\n");
  assert.equal(assertFrontendDist(root), index);
  assert.equal(listenUrl("127.0.0.1", 3080), "http://127.0.0.1:3080");
  rmSync(root, { recursive: true, force: true });
});
