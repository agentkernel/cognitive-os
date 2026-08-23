import test from "node:test";
import assert from "node:assert/strict";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  DEFAULT_WEB_HOST,
  DEFAULT_WEB_PORT,
  PATH_B_WEB_DAEMON_KEY_REF,
  PATH_B_WEB_OFFICIAL_KEY_REF,
  assertFrontendDist,
  assertLoopbackHost,
  assertPathBProviderBase,
  assertWebPort,
  frontendDistIndex,
  listenUrl,
  pathBWebChildExtras,
  pathBWebCredentialsYaml,
  pathBWebSettingsYaml,
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

test("Path B web settings and credentials stay on the daemon bearer", () => {
  const base = "http://127.0.0.1:48681/provider/v1/dsh";
  assert.equal(assertPathBProviderBase(`${base}/`), base);
  assert.throws(() => assertPathBProviderBase("https://api.deepseek.com"), /loopback daemon/);
  assert.throws(() => pathBWebCredentialsYaml(""), /non-empty/);
  const yaml = pathBWebCredentialsYaml("mgmt-token-fixture");
  assert.match(yaml, new RegExp(`${PATH_B_WEB_DAEMON_KEY_REF}: "mgmt-token-fixture"`));
  assert.match(yaml, new RegExp(`${PATH_B_WEB_OFFICIAL_KEY_REF}: "mgmt-token-fixture"`));
  assert.equal(yaml.includes("sk-"), false);
  const settings = pathBWebSettingsYaml(
    base,
    "ui-onboarding:\n  welcomeNoticeVersion: keep-me\n",
    "grok-4.6",
  );
  assert.match(settings, /welcomeNoticeVersion: keep-me/);
  assert.match(settings, /baseURL: http:\/\/127\.0\.0\.1:48681\/provider\/v1\/dsh/);
  assert.match(settings, new RegExp(`apiKeyEnv: ${PATH_B_WEB_DAEMON_KEY_REF}`));
  assert.match(settings, /model: grok-4\.6/);
  assert.deepEqual(pathBWebChildExtras(base), { DEEPSEEK_BASE_URL: base });
  assert.equal(Object.keys(pathBWebChildExtras(base)).some((key) => /API_KEY|SECRET|TOKEN|BEARER/i.test(key)), false);
});
