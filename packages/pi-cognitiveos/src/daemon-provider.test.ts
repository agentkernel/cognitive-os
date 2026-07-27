import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";

import { PersonalDaemonClient } from "./daemon-client.js";
import type { EnvironmentSlice, FileReader } from "./daemon-discovery.js";
import { resolvePersonalDaemonPaths } from "./daemon-discovery.js";
import { createDaemonProvider } from "./daemon-provider.js";
import { boundedCompletionBody, selectedModelProjectionBody, startFakeDaemon } from "./test-support.js";

const BOOTSTRAP_SECRET = "boot-0123456789abcdef-fedcba9876543210";
const ENVIRONMENT: EnvironmentSlice = {
  HOME: path.join(path.sep, "home", "owner"),
  XDG_RUNTIME_DIR: path.join(path.sep, "run", "user", "1000"),
};

function clientFor(endpoint: string): PersonalDaemonClient {
  const paths = resolvePersonalDaemonPaths(ENVIRONMENT);
  const contents: Record<string, string> = {
    [paths.endpointFile]: JSON.stringify({ schema_version: 1, endpoint, surface: "personal-daemon-endpoint" }),
    [paths.bootstrapSecretFile]: `${BOOTSTRAP_SECRET}\n`,
  };
  const files: FileReader = { readTextFile: (filePath) => contents[filePath] ?? (() => { throw new Error(`ENOENT: ${filePath}`); })() };
  return new PersonalDaemonClient({ environment: ENVIRONMENT, files, requestTimeoutMs: 2_000 });
}

test("complete daemon provider registers one projected model and emits bounded text events", async () => {
  const daemon = await startFakeDaemon({ bootstrapSecret: BOOTSTRAP_SECRET, statusBody: "{}" });
  try {
    const provider = await createDaemonProvider(clientFor(daemon.endpoint));
    assert.equal(provider.id, "cognitiveos");
    assert.equal(provider.getModels().length, 1);
    const model = provider.getModels()[0]!;
    const stream = provider.streamSimple(model, { messages: [{ role: "user", content: "hello" }] });
    const events = [];
    for await (const event of stream) events.push(event.type);
    assert.deepEqual(events, ["start", "text_start", "text_delta", "text_end", "done"]);
    assert.equal((await stream.result()).content[0]?.text, "daemon text");
    assert.deepEqual(
      daemon.requests.map((request) => `${request.method} ${request.url}`),
      [
        "POST /local/session",
        "GET /provider/v1/selected-model",
        "POST /provider/v1/chat/completions",
      ],
    );
    const completionRequest = daemon.requests[2]!;
    assert.match(completionRequest.body, /"stream":false/);
    assert.ok(!completionRequest.body.includes(BOOTSTRAP_SECRET));
  } finally {
    await daemon.close();
  }
});

test("unsupported tool output produces one terminal error without tool events", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: "{}",
    completionBody: JSON.stringify({ choices: [{ message: { content: "text", tool_calls: [] }, finish_reason: "stop" }] }),
  });
  try {
    const provider = await createDaemonProvider(clientFor(daemon.endpoint));
    const model = provider.getModels()[0]!;
    const stream = provider.streamSimple(model, { messages: [{ role: "user", content: "hello" }] });
    const events = [];
    for await (const event of stream) events.push(event.type);
    assert.deepEqual(events, ["start", "error"]);
    assert.equal((await stream.result()).stopReason, "error");
  } finally {
    await daemon.close();
  }
});

test("pre-dispatch abort creates no completion request", async () => {
  const daemon = await startFakeDaemon({ bootstrapSecret: BOOTSTRAP_SECRET, statusBody: "{}", completionBody: boundedCompletionBody() });
  try {
    const provider = await createDaemonProvider(clientFor(daemon.endpoint));
    const controller = new AbortController();
    controller.abort();
    const stream = provider.streamSimple(provider.getModels()[0]!, { messages: [] }, { signal: controller.signal });
    const events = [];
    for await (const event of stream) events.push(event.type);
    assert.deepEqual(events, ["error"]);
    assert.equal(daemon.requests.filter((request) => request.url === "/provider/v1/chat/completions").length, 0);
  } finally {
    await daemon.close();
  }
});

test("malformed selected-model projection refuses provider construction", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: "{}",
    selectedModelBody: selectedModelProjectionBody({ chat_capable: false }),
  });
  try {
    await assert.rejects(createDaemonProvider(clientFor(daemon.endpoint)));
    assert.equal(daemon.requests.filter((request) => request.url === "/provider/v1/chat/completions").length, 0);
  } finally {
    await daemon.close();
  }
});
