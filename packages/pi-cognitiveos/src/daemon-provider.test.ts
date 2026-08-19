import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";

import { PersonalDaemonClient } from "./daemon-client.js";
import type { EnvironmentSlice, FileReader } from "./daemon-discovery.js";
import { resolvePersonalDaemonPaths } from "./daemon-discovery.js";
import { createDaemonProvider } from "./daemon-provider.js";
import { openPiRouteObservationSession } from "./pi-route-observation.js";
import { boundedCompletionBody, selectedModelProjectionBody, startFakeDaemon } from "./test-support.js";

const BOOTSTRAP_SECRET = "boot-0123456789abcdef-fedcba9876543210";
const ENVIRONMENT: EnvironmentSlice = {
  HOME: path.join(path.sep, "home", "owner"),
  XDG_RUNTIME_DIR: path.join(path.sep, "run", "user", "1000"),
};
const AUTHORIZED_ENVIRONMENT: EnvironmentSlice = {
  ...ENVIRONMENT,
  COGNITIVEOS_PI_ROUTE_OBSERVATION: "enabled",
  COGNITIVEOS_PI_ROUTE_OBSERVATION_CAMPAIGN: "PERSONAL-PERF-EVAL-003",
};

function clientFor(
  endpoint: string,
  environment: EnvironmentSlice = ENVIRONMENT,
): PersonalDaemonClient {
  const paths = resolvePersonalDaemonPaths(environment);
  const contents: Record<string, string> = {
    [paths.endpointFile]: JSON.stringify({ schema_version: 1, endpoint, surface: "personal-daemon-endpoint" }),
    [paths.bootstrapSecretFile]: `${BOOTSTRAP_SECRET}\n`,
  };
  const files: FileReader = { readTextFile: (filePath) => contents[filePath] ?? (() => { throw new Error(`ENOENT: ${filePath}`); })() };
  return new PersonalDaemonClient({ environment, files, requestTimeoutMs: 2_000 });
}

test("custom Pi provider configuration registers one projected model and emits bounded text events", async () => {
  const daemon = await startFakeDaemon({ bootstrapSecret: BOOTSTRAP_SECRET, statusBody: "{}" });
  try {
    const provider = await createDaemonProvider(clientFor(daemon.endpoint));
    assert.equal(provider.api, "openai-completions");
    assert.equal(provider.models.length, 1);
    assert.equal(provider.apiKey, "cognitiveos-local-daemon");
    const model = provider.models[0]!;
    const stream = provider.streamSimple(model, { messages: [{ role: "user", content: "hello" }] });
    const events = [];
    for await (const event of stream) events.push(event.type);
    assert.deepEqual(events, ["start", "text_start", "text_delta", "text_end", "done"]);
    const resultContent = (await stream.result()).content[0];
    assert.ok(resultContent !== undefined && resultContent.type === "text");
    assert.equal(resultContent.text, "daemon text");
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

test("Pi completion exposes only measured Provider usage and leaves cost unavailable", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: "{}",
    completionBody: JSON.stringify({
      choices: [{ message: { content: "daemon text" }, finish_reason: "stop" }],
      usage: { prompt_tokens: 7, completion_tokens: 3, total_tokens: 10 },
    }),
  });
  try {
    const provider = await createDaemonProvider(clientFor(daemon.endpoint));
    const stream = provider.streamSimple(provider.models[0]!, { messages: [{ role: "user", content: "hello" }] });
    const message = await stream.result();

    assert.deepEqual(message.usage, {
      input: 7,
      output: 3,
      cacheRead: undefined,
      cacheWrite: undefined,
      totalTokens: 10,
      cost: { input: undefined, output: undefined, cacheRead: undefined, cacheWrite: undefined, total: undefined },
    });
  } finally {
    await daemon.close();
  }
});

test("structured daemon tool calls become pinned Pi tool-call events", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: "{}",
    completionBody: JSON.stringify({
      choices: [{
        message: {
          content: null,
          tool_calls: [{
            id: "call-workspace-read",
            type: "function",
            function: { name: "WorkspaceRead", arguments: JSON.stringify({ target: "README.md" }) },
          }],
        },
        finish_reason: "tool_calls",
      }],
    }),
  });
  try {
    const provider = await createDaemonProvider(clientFor(daemon.endpoint));
    const stream = provider.streamSimple(provider.models[0]!, {
      messages: [{ role: "user", content: "read README.md" }],
      tools: [{
        name: "WorkspaceRead",
        description: "daemon-governed read",
        parameters: { type: "object" },
      }],
    });
    const eventTypes: string[] = [];
    for await (const event of stream) eventTypes.push(event.type);

    assert.deepEqual(eventTypes, ["start", "toolcall_start", "toolcall_delta", "toolcall_end", "done"]);
    const resultContent = (await stream.result()).content[0];
    assert.ok(resultContent !== undefined && resultContent.type === "toolCall");
    assert.equal(resultContent.name, "WorkspaceRead");
    assert.deepEqual(resultContent.arguments, { target: "README.md" });
    const completionRequest = daemon.requests.at(-1);
    assert.match(completionRequest?.body ?? "", /"tool_choice":"auto"/);
    assert.match(completionRequest?.body ?? "", /"WorkspaceRead"/);
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
    const session = openPiRouteObservationSession(AUTHORIZED_ENVIRONMENT);
    assert.ok(session !== undefined);
    const provider = await createDaemonProvider(clientFor(daemon.endpoint, AUTHORIZED_ENVIRONMENT), {
      session,
    });
    const model = provider.models[0]!;
    const stream = provider.streamSimple(model, { messages: [{ role: "user", content: "hello" }] });
    const events = [];
    for await (const event of stream) events.push(event.type);
    assert.deepEqual(events, ["start", "error"]);
    assert.equal((await stream.result()).stopReason, "error");
    assert.equal(session.observations.length, 1);
    const observation = session.observations[0] as unknown as Record<string, unknown>;
    assert.equal(observation["outcome"], "error");
    assert.equal(observation["failureClass"], "protocol_error");
  } finally {
    await daemon.close();
  }
});

test("pre-dispatch abort creates no completion request", async () => {
  const daemon = await startFakeDaemon({ bootstrapSecret: BOOTSTRAP_SECRET, statusBody: "{}", completionBody: boundedCompletionBody() });
  try {
    const session = openPiRouteObservationSession(AUTHORIZED_ENVIRONMENT);
    assert.ok(session !== undefined);
    const provider = await createDaemonProvider(clientFor(daemon.endpoint, AUTHORIZED_ENVIRONMENT), {
      session,
    });
    const controller = new AbortController();
    controller.abort();
    const stream = provider.streamSimple(provider.models[0]!, { messages: [] }, { signal: controller.signal });
    const events = [];
    for await (const event of stream) events.push(event.type);
    assert.deepEqual(events, ["error"]);
    assert.equal(daemon.requests.filter((request) => request.url === "/provider/v1/chat/completions").length, 0);
    assert.equal(session.observations.length, 1, "a started instrumented sample must not disappear");
    const observation = session.observations[0] as unknown as Record<string, unknown>;
    assert.equal(observation["requestMode"], "non_streaming");
    assert.equal(observation["outcome"], "cancelled");
    assert.equal(observation["failureClass"], "cancelled");
    assert.equal(observation["terminalStage"], "before_request");
    assert.deepEqual(observation["stages"], []);
    assert.deepEqual(observation["providerUsage"], { availability: "not_available" });
  } finally {
    await daemon.close();
  }
});

test("a no-Provider daemon refusal retains one content-free error observation", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: "{}",
    completionStatus: 503,
    completionBody: JSON.stringify({
      status: "error",
      error: {
        code: "PERSONAL_PROVIDER_NOT_CONFIGURED",
        message: "provider proxy request was not completed",
        category: "protocol",
        retryable: false,
        stage: "personal-front-door",
      },
    }),
  });
  try {
    const session = openPiRouteObservationSession(AUTHORIZED_ENVIRONMENT);
    assert.ok(session !== undefined);
    const provider = await createDaemonProvider(clientFor(daemon.endpoint, AUTHORIZED_ENVIRONMENT), {
      session,
    });
    const stream = provider.streamSimple(provider.models[0]!, {
      messages: [{ role: "user", content: "hello" }],
    });
    const events = [];
    for await (const event of stream) events.push(event.type);
    assert.deepEqual(events, ["start", "error"]);
    assert.equal((await stream.result()).stopReason, "error");

    assert.equal(session.observations.length, 1, "error samples must remain in the denominator");
    const observation = session.observations[0] as unknown as Record<string, unknown>;
    assert.equal(observation["requestMode"], "non_streaming");
    assert.equal(observation["outcome"], "error");
    assert.equal(observation["failureClass"], "provider_unavailable");
    assert.equal(observation["terminalStage"], "response_parse");
    assert.deepEqual(observation["providerUsage"], { availability: "not_available" });
    assert.deepEqual(
      (observation["stages"] as readonly Record<string, unknown>[]).map((stage) => stage["stage"]),
      ["pi_request_preparation", "extension_dispatch", "loopback_wait", "response_parse"],
    );
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
