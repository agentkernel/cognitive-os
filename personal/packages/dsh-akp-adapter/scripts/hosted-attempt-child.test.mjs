/**
 * P13-T02 hosted attempt child: protocol / argv refusals, pin mismatch, dsh
 * CLI absence, daemon absence, timeout, and one full run against a fake pinned
 * dsh CLI plus a fake loopback daemon. No frame may carry the bearer or the
 * bootstrap secret; a dsh exit is never completion.
 */
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { after, before, describe, test } from "node:test";
import { HOSTED_DSH_REVISION_PIN, HOSTED_FRAME_PROTOCOL, isLoopbackOrigin, parseRequest, redact } from "./hosted-attempt-child.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const script = join(here, "hosted-attempt-child.mjs");
const adapterRoot = dirname(here);

function requestLine(overrides = {}) {
  const context = overrides.context ?? "summarize README.md in one sentence";
  return `${JSON.stringify({
    frame: "request",
    protocol: HOSTED_FRAME_PROTOCOL,
    attempt_id: "dshattempt-test",
    task_ref: "task://personal/p13-t02",
    employee_id: "employee-test",
    project_id: "project-test",
    context,
    context_digest: createHash("sha256").update(context, "utf8").digest("hex"),
    provider_proxy: "POST /provider/v1/dsh/chat/completions",
    daemon_origin: null,
    bootstrap_file: null,
    completion_authority: "daemon",
    ...overrides,
  })}\n`;
}

/**
 * Async spawn: the fake daemon lives in this process, so the child must run
 * while this event loop keeps serving (spawnSync would deadlock the pair).
 */
function runChild(args, input, timeoutMs = 30000) {
  return new Promise((resolvePromise) => {
    const child = spawn(process.execPath, [script, ...args], {
      env: { PATH: process.env.PATH, SystemRoot: process.env.SystemRoot, TEMP: process.env.TEMP, TMP: process.env.TMP, TMPDIR: process.env.TMPDIR, HOME: process.env.HOME, USERPROFILE: process.env.USERPROFILE },
      stdio: ["pipe", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";
    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });
    const timer = setTimeout(() => child.kill("SIGKILL"), timeoutMs);
    child.on("close", (status) => {
      clearTimeout(timer);
      const frames = stdout
        .split(/\r?\n/)
        .filter((line) => line.trim())
        .map((line) => JSON.parse(line));
      resolvePromise({ status, frames, stdout, stderr });
    });
    child.stdin.on("error", () => {});
    child.stdin.end(input);
  });
}

function lastResponse(frames) {
  return frames.filter((frame) => frame.frame === "response").at(-1);
}

function makeDshRoot(withCli, options = {}) {
  const root = mkdtempSync(join(tmpdir(), "p13t02-dsh-"));
  writeFileSync(join(root, ".cognitiveos-dsh-revision"), `${options.pin ?? HOSTED_DSH_REVISION_PIN}\n`);
  if (withCli) {
    mkdirSync(join(root, "apps/cli/lib"), { recursive: true });
    mkdirSync(join(root, "packages/api/gateway/lib"), { recursive: true });
    writeFileSync(join(root, "packages/api/gateway/lib/index.js"), "module.exports = {};\n");
    const sleepMs = options.sleepMs ?? 0;
    writeFileSync(
      join(root, "apps/cli/lib/bin.js"),
      [
        "const fs = require('node:fs');",
        "const path = require('node:path');",
        "const task = process.argv[process.argv.length - 1];",
        "const patchIndex = process.argv.indexOf('--patch');",
        "const patch = patchIndex >= 0 ? fs.readFileSync(process.argv[patchIndex + 1], 'utf8') : '';",
        "const credentials = fs.existsSync(path.join(process.env.DSH_HOME, '.credentials.yaml'));",
        `fs.writeFileSync(path.join(${JSON.stringify(root)}, 'fake-dsh-observed.json'), JSON.stringify({ argv: process.argv.slice(2), env_keys: Object.keys(process.env).sort(), patch, credentials, base_url: process.env.DEEPSEEK_BASE_URL || null }));`,
        "process.stdout.write('$ ' + task + '\\n');",
        `const finish = () => { process.stdout.write('Summary: ' + task + '\\n'); process.exit(${options.exitCode ?? 0}); };`,
        sleepMs > 0 ? `setTimeout(finish, ${sleepMs});` : "finish();",
        "",
      ].join("\n"),
    );
  }
  return root;
}

describe("hosted-attempt-child protocol", () => {
  test("parseRequest refuses non-request, wrong protocol, digest drift, direct provider", async () => {
    assert.equal(parseRequest("not json").error, "request frame is not JSON");
    assert.equal(parseRequest(JSON.stringify({ frame: "observation" })).error, "first frame must be request");
    assert.equal(parseRequest(JSON.stringify({ frame: "request", protocol: "x" })).error, "unknown frame protocol");
    const good = JSON.parse(requestLine());
    assert.equal(parseRequest(JSON.stringify(good)).error, undefined);
    assert.equal(parseRequest(JSON.stringify({ ...good, context_digest: "0".repeat(64) })).error, "context digest mismatch");
    assert.equal(parseRequest(JSON.stringify({ ...good, provider_direct: true })).error, "direct Provider is refused");
    assert.equal(parseRequest(JSON.stringify({ ...good, context: "  " })).error, "context required");
  });

  test("isLoopbackOrigin admits loopback only", async () => {
    assert.equal(isLoopbackOrigin("http://127.0.0.1:48181"), true);
    assert.equal(isLoopbackOrigin("http://localhost:48181"), true);
    assert.equal(isLoopbackOrigin("http://[::1]:48181"), true);
    assert.equal(isLoopbackOrigin("https://api.deepseek.com"), false);
    assert.equal(isLoopbackOrigin("http://192.168.1.2:48181"), false);
    assert.equal(isLoopbackOrigin("not a url"), false);
  });

  test("redact hides bearer, sk-, session and bootstrap shapes", async () => {
    const text = "Authorization: Bearer sess-abc.def key sk-live-123 boot-xyz ssv1:zzz tail";
    const redacted = redact(text);
    assert.equal(redacted.includes("abc.def"), false);
    assert.equal(redacted.includes("live-123"), false);
    assert.equal(redacted.includes("boot-xyz"), false);
    assert.equal(redacted.includes("zzz"), false);
    assert.equal(redacted.endsWith("tail"), true);
  });
});

describe("hosted-attempt-child refusals", () => {
  const baseArgs = (dshRoot) => ["--dsh-root", dshRoot, "--adapter-root", adapterRoot, "--revision", HOSTED_DSH_REVISION_PIN, "--provider-path", "b"];
  let dshRoot;
  before(() => {
    dshRoot = makeDshRoot(false);
  });
  after(() => {
    rmSync(dshRoot, { recursive: true, force: true });
  });

  test("Path A, API key file and native MCP flags are refused before anything runs", async () => {
    const cases = [
      ["--dsh-root", dshRoot, "--adapter-root", adapterRoot, "--revision", HOSTED_DSH_REVISION_PIN, "--provider-path", "a"],
      [...baseArgs(dshRoot), "--api-key-file", "/dev/null"],
      [...baseArgs(dshRoot), "--mcp"],
      [...baseArgs(dshRoot), "--direct-base-url", "https://api.deepseek.com"],
    ];
    for (const args of cases) {
      const run = await runChild(args, requestLine());
      assert.equal(run.status, 2, run.stdout);
      const response = lastResponse(run.frames);
      assert.equal(response.status, "failed");
      assert.equal(response.completion_claimed, false);
    }
  });

  test("a non-request first frame is a protocol refusal", async () => {
    const run = await runChild(baseArgs(dshRoot), `${JSON.stringify({ frame: "observation" })}\n`);
    assert.equal(run.status, 2, run.stdout);
    assert.match(lastResponse(run.frames).reason, /protocol/);
  });

  test("revision argument that is not the pin refuses before reading stdin", async () => {
    const args = baseArgs(dshRoot).map((value) => (value === HOSTED_DSH_REVISION_PIN ? "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef" : value));
    const run = await runChild(args, requestLine());
    assert.equal(run.status, 3, run.stdout);
    assert.match(lastResponse(run.frames).reason, /pin-mismatch/);
  });

  test("pin file drift is a pin mismatch", async () => {
    const drifted = makeDshRoot(false, { pin: "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef" });
    try {
      const run = await runChild(baseArgs(drifted), requestLine());
      assert.equal(run.status, 3, run.stdout);
      assert.match(lastResponse(run.frames).reason, /pin file/);
    } finally {
      rmSync(drifted, { recursive: true, force: true });
    }
  });

  test("missing dsh CLI is a failed response, never success", async () => {
    const run = await runChild(baseArgs(dshRoot), requestLine());
    assert.equal(run.status, 4, run.stdout);
    assert.equal(run.frames[0].frame, "observation");
    assert.equal(run.frames[0].text, "child.started");
    assert.equal(run.frames[0].context_digest, JSON.parse(requestLine()).context_digest);
    assert.ok(run.frames.some((frame) => frame.text === "dsh.cli.missing"));
    const response = lastResponse(run.frames);
    assert.equal(response.status, "failed");
    assert.equal(response.reason, "dsh-cli-missing");
    assert.equal(response.completion_claimed, false);
  });

  test("daemon unavailable fails closed after the CLI check", async () => {
    const withCli = makeDshRoot(true);
    try {
      const noOrigin = await runChild(baseArgs(withCli), requestLine({ daemon_origin: null }));
      assert.equal(noOrigin.status, 5, noOrigin.stdout);
      assert.match(lastResponse(noOrigin.frames).reason, /daemon_origin/);
      const remote = await runChild(baseArgs(withCli), requestLine({ daemon_origin: "https://api.deepseek.com", bootstrap_file: script }));
      assert.equal(remote.status, 5, remote.stdout);
      const noBootstrap = await runChild(baseArgs(withCli), requestLine({ daemon_origin: "http://127.0.0.1:1", bootstrap_file: join(withCli, "absent-bootstrap") }));
      assert.equal(noBootstrap.status, 5, noBootstrap.stdout);
      assert.match(lastResponse(noBootstrap.frames).reason, /bootstrap/);
      const closedPort = await runChild(baseArgs(withCli), requestLine({ daemon_origin: "http://127.0.0.1:1", bootstrap_file: script }));
      assert.equal(closedPort.status, 5, closedPort.stdout);
      assert.match(lastResponse(closedPort.frames).reason, /management session/);
    } finally {
      rmSync(withCli, { recursive: true, force: true });
    }
  });
});

describe("hosted-attempt-child full run against a fake pinned dsh and fake daemon", () => {
  const bootstrapSecret = "boot-test-bootstrap-secret-not-real";
  const mintedToken = "sess-test-management-token-not-real";
  let server;
  let origin;
  let bootstrapFile;
  let dshRoot;
  const seen = { authorizations: [], sessionBodies: [] };

  before(async () => {
    dshRoot = makeDshRoot(true);
    bootstrapFile = join(dshRoot, "bootstrap.secret");
    writeFileSync(bootstrapFile, `${bootstrapSecret}\n`, { mode: 0o600 });
    server = createServer((request, response) => {
      let body = "";
      request.on("data", (chunk) => {
        body += chunk;
      });
      request.on("end", () => {
        const url = new URL(request.url, "http://127.0.0.1");
        if (request.headers.authorization) seen.authorizations.push(request.headers.authorization);
        response.setHeader("content-type", "application/json");
        if (request.method === "POST" && url.pathname === "/local/session") {
          const parsed = JSON.parse(body);
          seen.sessionBodies.push(parsed);
          if (parsed.bootstrap_secret !== bootstrapSecret) {
            response.statusCode = 401;
            response.end(JSON.stringify({ status: "error", code: "LOCAL_BOOTSTRAP_MISMATCH" }));
            return;
          }
          response.end(JSON.stringify({ token: mintedToken, channel: parsed.channel, session_id: "sess-1-1" }));
          return;
        }
        if (request.headers.authorization !== `Bearer ${mintedToken}`) {
          response.statusCode = 401;
          response.end(JSON.stringify({ status: "error", code: "LOCAL_UNAUTHORIZED" }));
          return;
        }
        if (url.pathname === "/management/agent-bindings") {
          response.end(JSON.stringify({ bindings: [{ agent: "agent://personal/dsh", status: "active", account_id: "acct-test", model_id: "deepseek-chat" }] }));
          return;
        }
        if (url.pathname === "/management/providers/models") {
          response.end(JSON.stringify({ models: [{ id: "deepseek-chat", name: "deepseek-chat" }] }));
          return;
        }
        response.statusCode = 404;
        response.end(JSON.stringify({ status: "error", code: "NOT_FOUND" }));
      });
    });
    await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
    origin = `http://127.0.0.1:${server.address().port}`;
  });
  after(async () => {
    await new Promise((resolve) => server.close(resolve));
    rmSync(dshRoot, { recursive: true, force: true });
  });

  test("streams observations, one DeliverableDraft candidate, and a non-authoritative response", async () => {
    const args = ["--dsh-root", dshRoot, "--adapter-root", adapterRoot, "--revision", HOSTED_DSH_REVISION_PIN, "--provider-path", "b"];
    const run = await runChild(args, requestLine({ daemon_origin: origin, bootstrap_file: bootstrapFile, timeout_ms: 60000 }));
    assert.equal(run.status, 0, `${run.stdout}\n${run.stderr}`);
    assert.equal(run.frames[0].text, "child.started");
    assert.ok(run.frames.some((frame) => frame.text === "dsh.cli.compiled-lib"));
    const bound = run.frames.find((frame) => frame.text === "provider.proxy.bound");
    assert.equal(bound.bound, true);
    assert.equal(bound.model, "deepseek-chat");
    assert.ok(run.frames.some((frame) => frame.text === "dsh.spawned" && Number.isInteger(frame.dsh_pid)));
    const candidate = run.frames.find((frame) => frame.frame === "candidate");
    assert.equal(candidate.operation, "DeliverableDraft");
    assert.equal(candidate.payload.text, "Summary: summarize README.md in one sentence");
    assert.equal(candidate.payload.dsh_exit, 0);
    const response = lastResponse(run.frames);
    assert.equal(response.status, "done");
    assert.equal(response.dsh_exit, 0);
    assert.equal(response.completion_claimed, false);
    assert.equal(run.stdout.includes(mintedToken), false, "bearer must never reach stdout");
    assert.equal(run.stdout.includes(bootstrapSecret), false, "bootstrap must never reach stdout");
    assert.equal(run.stderr.includes(mintedToken), false);

    assert.deepEqual(seen.sessionBodies.map((body) => body.channel), ["management"]);
    assert.ok(seen.authorizations.every((value) => value === `Bearer ${mintedToken}`));

    const observed = JSON.parse(readFileSync(join(dshRoot, "fake-dsh-observed.json"), "utf8"));
    assert.equal(observed.argv.includes("--profile"), true);
    assert.equal(observed.argv.includes("headless"), true);
    assert.equal(observed.argv.at(-1), "summarize README.md in one sentence");
    assert.equal(observed.credentials, true, "credentials file exists while dsh runs");
    assert.equal(observed.base_url, `${origin}/provider/v1/dsh`);
    assert.match(observed.patch, new RegExp(`baseURL: ${origin.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}/provider/v1/dsh`));
    assert.match(observed.patch, /apiKeyEnv: DAEMON_BEARER/);
    assert.equal(observed.patch.includes(mintedToken), false);
    assert.equal(observed.patch.includes("api.deepseek.com"), false);
    for (const key of observed.env_keys) {
      assert.doesNotMatch(key, /API_KEY|SECRET|TOKEN|PASSWORD|BEARER|CARGO/i, `env key ${key}`);
    }
    assert.ok(observed.env_keys.includes("DSH_HOME"));
    assert.equal(observed.env_keys.includes("DSH_PERMISSION_MODE"), true);
  });

  test("a dsh that outlives the budget is killed and reported failed / timed-out", async () => {
    const slowRoot = makeDshRoot(true, { sleepMs: 30000 });
    writeFileSync(join(slowRoot, "bootstrap.secret"), `${bootstrapSecret}\n`, { mode: 0o600 });
    try {
      const args = ["--dsh-root", slowRoot, "--adapter-root", adapterRoot, "--revision", HOSTED_DSH_REVISION_PIN, "--provider-path", "b"];
      const started = Date.now();
      const run = await runChild(args, requestLine({ daemon_origin: origin, bootstrap_file: join(slowRoot, "bootstrap.secret"), timeout_ms: 4500 }), 25000);
      assert.equal(run.status, 6, `${run.stdout}\n${run.stderr}`);
      assert.ok(Date.now() - started < 20000);
      const response = lastResponse(run.frames);
      assert.equal(response.status, "failed");
      assert.equal(response.reason, "timed-out");
      assert.equal(response.completion_claimed, false);
      assert.equal(run.frames.some((frame) => frame.frame === "candidate" && frame.payload.text.startsWith("Summary")), false);
    } finally {
      rmSync(slowRoot, { recursive: true, force: true });
    }
  });

  test("a dsh that exits non-zero is failed, and its text is still only a candidate", async () => {
    const failingRoot = makeDshRoot(true, { exitCode: 2 });
    writeFileSync(join(failingRoot, "bootstrap.secret"), `${bootstrapSecret}\n`, { mode: 0o600 });
    try {
      const args = ["--dsh-root", failingRoot, "--adapter-root", adapterRoot, "--revision", HOSTED_DSH_REVISION_PIN, "--provider-path", "b"];
      const run = await runChild(args, requestLine({ daemon_origin: origin, bootstrap_file: join(failingRoot, "bootstrap.secret") }));
      assert.equal(run.status, 6, run.stdout);
      const response = lastResponse(run.frames);
      assert.equal(response.status, "failed");
      assert.equal(response.reason, "dsh-exit-2");
      const candidate = run.frames.find((frame) => frame.frame === "candidate");
      assert.equal(candidate.payload.dsh_exit, 2);
      assert.equal(response.completion_claimed, false);
    } finally {
      rmSync(failingRoot, { recursive: true, force: true });
    }
  });
});
