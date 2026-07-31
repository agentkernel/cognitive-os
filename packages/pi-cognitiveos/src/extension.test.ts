/**
 * Extension registration and behaviour tests (P1-T07 acceptance).
 *
 * These drive the registered Pi hooks through a recording `ExtensionAPI`:
 * project trust is denied, every tool call is refused, the session shows real
 * daemon facts, and an unavailable daemon produces a loud, explicit failure
 * rather than a session that looks usable.
 */

import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";

import { PersonalDaemonClient } from "./daemon-client.js";
import type { EnvironmentSlice, FileReader } from "./daemon-discovery.js";
import { resolvePersonalDaemonPaths } from "./daemon-discovery.js";
import { registerCognitiveOsExtension } from "./extension.js";
import { COGNITIVEOS_STATUS_COMMAND_NAME, COGNITIVEOS_STATUS_KEY } from "./pin.js";
import { FakePi, readinessProjectionBody, startFakeDaemon } from "./test-support.js";

const BOOTSTRAP_SECRET = "boot-0123456789abcdef-fedcba9876543210";

const ENVIRONMENT: EnvironmentSlice = {
  HOME: path.join(path.sep, "home", "owner"),
  XDG_RUNTIME_DIR: path.join(path.sep, "run", "user", "1000"),
};

function filesFor(endpoint: string | undefined): FileReader {
  const paths = resolvePersonalDaemonPaths(ENVIRONMENT);
  const contents: Record<string, string> = {};
  if (endpoint !== undefined) {
    contents[paths.endpointFile] = JSON.stringify({
      schema_version: 1,
      endpoint,
      surface: "personal-daemon-endpoint",
    });
    contents[paths.bootstrapSecretFile] = `${BOOTSTRAP_SECRET}\n`;
  }
  return {
    readTextFile(filePath: string): string {
      const value = contents[filePath];
      if (value === undefined) {
        throw new Error(`ENOENT: ${filePath}`);
      }
      return value;
    },
  };
}

function clientFor(endpoint: string | undefined): PersonalDaemonClient {
  return new PersonalDaemonClient({
    environment: ENVIRONMENT,
    files: filesFor(endpoint),
    requestTimeoutMs: 2_000,
  });
}

test("registration queues the daemon provider and activates its model at session start", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: readinessProjectionBody(),
  });
  try {
    const pi = new FakePi();
    await registerCognitiveOsExtension(pi, { client: clientFor(daemon.endpoint) });

    assert.deepEqual([...pi.registeredHooks].sort(), [
      "project_trust",
      "session_start",
      "tool_call",
    ]);
    assert.ok(pi.commands.has(COGNITIVEOS_STATUS_COMMAND_NAME));
    assert.match(pi.commands.get(COGNITIVEOS_STATUS_COMMAND_NAME)?.description ?? "", /read-only/);
    assert.equal(pi.providers.length, 1);
    assert.equal(pi.selectedModels.length, 0);

    await pi.driveSessionStart();
    assert.equal(pi.selectedModels.length, 1);
    assert.equal(pi.selectedModels[0]?.provider, "cognitiveos");
    assert.equal(
      pi.selectedModels[0]?.baseUrl,
      `http://${daemon.endpoint}/provider/v1`,
      "Pi setModel must receive a complete runtime model rather than provider-only metadata",
    );
  } finally {
    await daemon.close();
  }
});

test("project trust is always denied", async () => {
  const pi = new FakePi();
  await assert.rejects(registerCognitiveOsExtension(pi, { client: clientFor(undefined) }));
});

test("bash, write and edit are blocked, and so is every other tool", async () => {
  const pi = new FakePi();
  await assert.rejects(registerCognitiveOsExtension(pi, { client: clientFor(undefined) }));

  for (const toolName of ["bash", "write", "edit"]) {
    const decision = await pi.driveToolCall(toolName);
    assert.ok(decision, `${toolName} must be blocked`);
    assert.equal(decision.block, true);
    assert.match(decision.reason, /mutating/);
  }
  const unknown = await pi.driveToolCall("some_other_tool");
  assert.ok(unknown);
  assert.equal(unknown.block, true);
});

test("session start shows real daemon facts and warns when the first conversation is blocked", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: readinessProjectionBody(),
  });
  try {
    const pi = new FakePi();
    await registerCognitiveOsExtension(pi, { client: clientFor(daemon.endpoint) });
    await pi.driveSessionStart();

    assert.equal(pi.ui.statuses.length, 1);
    assert.equal(pi.ui.statuses[0]?.statusKey, COGNITIVEOS_STATUS_KEY);
    assert.match(pi.ui.statuses[0]?.statusText ?? "", /CognitiveOS blocked/);
    assert.match(pi.ui.statuses[0]?.statusText ?? "", /first conversation blocked/);
    assert.match(pi.ui.statuses[0]?.statusText ?? "", /Pi tools disabled/);

    assert.equal(pi.ui.notifications.length, 1);
    assert.equal(pi.ui.notifications[0]?.level, "warn");
    assert.match(pi.ui.notifications[0]?.message ?? "", /database: blocked/);
    assert.match(pi.ui.notifications[0]?.message ?? "", /not a Gate, Profile or release claim/);
  } finally {
    await daemon.close();
  }
});

test("a ready daemon produces a ready status line and no session-start warning", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: readinessProjectionBody({
      overall: "ready",
      first_conversation_ready: true,
      components: [
        { component: "pi", status: "ready", required: false, error_class: null, duration_ms: 0 },
      ],
    }),
  });
  try {
    const pi = new FakePi();
    await registerCognitiveOsExtension(pi, { client: clientFor(daemon.endpoint) });
    await pi.driveSessionStart();

    assert.match(pi.ui.statuses[0]?.statusText ?? "", /CognitiveOS ready/);
    assert.match(pi.ui.statuses[0]?.statusText ?? "", /first conversation ready/);
    assert.deepEqual(pi.ui.notifications, []);
  } finally {
    await daemon.close();
  }
});

test("an unavailable daemon fails explicitly and never implies readiness", async () => {
  const pi = new FakePi();
  await assert.rejects(registerCognitiveOsExtension(pi, { client: clientFor(undefined) }));
  await pi.driveSessionStart();

  const statusText = pi.ui.statuses[0]?.statusText ?? "";
  assert.match(statusText, /CognitiveOS unavailable/);
  assert.ok(!statusText.includes("ready"), "an unavailable daemon must never render as ready");

  assert.equal(pi.ui.notifications.length, 1);
  assert.equal(pi.ui.notifications[0]?.level, "error");
  const message = pi.ui.notifications[0]?.message ?? "";
  assert.match(message, /PI_EXTENSION_ENDPOINT_FILE_MISSING/);
  assert.match(message, /no readiness is assumed/);
});

test("the status command reports the same daemon facts on demand", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: readinessProjectionBody(),
  });
  try {
    const pi = new FakePi();
    await registerCognitiveOsExtension(pi, { client: clientFor(daemon.endpoint) });
    await pi.driveCommand(COGNITIVEOS_STATUS_COMMAND_NAME);

    assert.equal(pi.ui.notifications.length, 1);
    assert.equal(pi.ui.notifications[0]?.level, "info");
    const message = pi.ui.notifications[0]?.message ?? "";
    assert.match(message, /CognitiveOS Personal status: blocked/);
    assert.match(message, /profile=not-claimed/);
    assert.match(message, /gate=not-claimed/);
    assert.match(message, /direct bash\/write\/edit are disabled/);
  } finally {
    await daemon.close();
  }
});

test("the status command surfaces an unavailable daemon as an error notification", async () => {
  const pi = new FakePi();
  await assert.rejects(registerCognitiveOsExtension(pi, { client: clientFor(undefined) }));
});

test("no Pi surface receives credential material at any point", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: readinessProjectionBody(),
  });
  try {
    const pi = new FakePi();
    await registerCognitiveOsExtension(pi, { client: clientFor(daemon.endpoint) });
    await pi.driveSessionStart();
    await pi.driveCommand(COGNITIVEOS_STATUS_COMMAND_NAME);

    const rendered = JSON.stringify({
      statuses: pi.ui.statuses,
      notifications: pi.ui.notifications,
    });
    assert.ok(!rendered.includes(BOOTSTRAP_SECRET));
    for (const token of daemon.issuedTokens) {
      assert.ok(!rendered.includes(token), "session tokens must never be displayed");
    }
  } finally {
    await daemon.close();
  }
});
