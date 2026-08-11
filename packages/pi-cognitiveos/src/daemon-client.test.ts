/**
 * Daemon client tests, driven against a real loopback server that speaks the
 * Personal front-door protocol (ADR-0022) including its refusal paths.
 *
 * The acceptance-critical assertions are:
 *   - an unavailable daemon fails explicitly, with a stable code;
 *   - a restarted daemon (bearer invalidated) is recovered from exactly once,
 *     and a second refusal is reported rather than retried forever;
 *   - no cookie is ever sent and the bootstrap secret never leaves the session
 *     request;
 *   - a malformed projection is a protocol error, never an assumed `ready`.
 */

import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";

import {
  PersonalDaemonClient,
  parseBoundedCompletion,
  parseReadinessProjection,
} from "./daemon-client.js";
import type { EnvironmentSlice, FileReader } from "./daemon-discovery.js";
import { resolvePersonalDaemonPaths } from "./daemon-discovery.js";
import { DaemonClientError, isDaemonUnavailable } from "./errors.js";
import {
  captureRejection,
  captureThrown,
  readinessProjectionBody,
  resourceProjectionBody,
  resourceWatchSnapshotBody,
  startFakeDaemon,
  taskWatchSnapshotBody,
} from "./test-support.js";

const BOOTSTRAP_SECRET = "boot-0123456789abcdef-fedcba9876543210";

const ENVIRONMENT: EnvironmentSlice = {
  HOME: path.join(path.sep, "home", "owner"),
  XDG_RUNTIME_DIR: path.join(path.sep, "run", "user", "1000"),
};

function filesFor(endpoint: string, secret: string = BOOTSTRAP_SECRET): FileReader {
  const paths = resolvePersonalDaemonPaths(ENVIRONMENT);
  const contents: Record<string, string> = {
    [paths.endpointFile]: JSON.stringify({
      schema_version: 1,
      endpoint,
      surface: "personal-daemon-endpoint",
    }),
    [paths.bootstrapSecretFile]: `${secret}\n`,
  };
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

test("Provider usage is measured only for complete internally consistent counters", () => {
  const measured = parseBoundedCompletion(JSON.stringify({
    choices: [{ message: { content: "bounded" }, finish_reason: "stop" }],
    usage: { prompt_tokens: 7, completion_tokens: 3, total_tokens: 10 },
  }));
  assert.deepEqual(measured.providerUsage, {
    availability: "measured",
    promptTokens: 7,
    completionTokens: 3,
    totalTokens: 10,
  });

  const missingUsage = parseBoundedCompletion(JSON.stringify({
    choices: [{ message: { content: "bounded" }, finish_reason: "stop" }],
  }));
  assert.deepEqual(missingUsage.providerUsage, { availability: "not_available" });

  const inconsistentUsage = parseBoundedCompletion(JSON.stringify({
    choices: [{ message: { content: "bounded" }, finish_reason: "stop" }],
    usage: { prompt_tokens: 7, completion_tokens: 3, total_tokens: 9 },
  }));
  assert.deepEqual(inconsistentUsage.providerUsage, { availability: "not_available" });

  const zeroDuration = parseBoundedCompletion(JSON.stringify({
    choices: [{ message: { content: "bounded" }, finish_reason: "stop" }],
  }), 0);
  assert.equal(zeroDuration.loopbackHttpElapsedNanos, 1);
});

test("Provider-network timing is accepted only from a positive daemon telemetry header", async () => {
  const measuredDaemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: "{}",
    providerNetworkElapsedNanos: "123456",
  });
  try {
    const completion = await new PersonalDaemonClient({
      environment: ENVIRONMENT,
      files: filesFor(measuredDaemon.endpoint),
    }).completeChat("deepseek-v4-flash", []);
    assert.equal(completion.providerNetworkElapsedNanos, 123456);
  } finally {
    await measuredDaemon.close();
  }

  const malformedDaemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: "{}",
    providerNetworkElapsedNanos: "0",
  });
  try {
    const completion = await new PersonalDaemonClient({
      environment: ENVIRONMENT,
      files: filesFor(malformedDaemon.endpoint),
    }).completeChat("deepseek-v4-flash", []);
    assert.equal(completion.providerNetworkElapsedNanos, undefined);
  } finally {
    await malformedDaemon.close();
  }
});

test("a session is minted and the readiness projection is returned verbatim", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: readinessProjectionBody(),
  });
  try {
    const client = new PersonalDaemonClient({
      environment: ENVIRONMENT,
      files: filesFor(daemon.endpoint),
    });
    const projection = await client.fetchReadiness();

    assert.equal(projection.overall, "blocked");
    assert.equal(projection.firstConversationReady, false);
    assert.equal(projection.profileClaim, "not-claimed");
    assert.equal(projection.gateClaim, "not-claimed");
    assert.equal(projection.staticCheckIsNotRuntimeReady, true);
    assert.equal(projection.authoritySideEffects, false);
    assert.equal(projection.components.length, 3);

    const session = daemon.requests.find((request) => request.url === "/local/session");
    assert.ok(session, "the client must mint a session before reading status");
    assert.equal(session.method, "POST");
    assert.equal(JSON.parse(session.body)["channel"], "management");
    assert.equal(JSON.parse(session.body)["principal_id"], "principal://local/owner");

    const status = daemon.requests.find((request) => request.url === "/personal/status");
    assert.ok(status);
    assert.equal(status.method, "GET");
    assert.match(status.headers["authorization"] ?? "", /^Bearer sess-fake-/);
  } finally {
    await daemon.close();
  }
});

test("private resource and Task observations use isolated daemon channels", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: readinessProjectionBody(),
    resourceProjectionBody: resourceProjectionBody(),
    resourceWatchBody: resourceWatchSnapshotBody(),
    taskWatchBody: taskWatchSnapshotBody(),
  });
  try {
    const client = new PersonalDaemonClient({
      environment: ENVIRONMENT,
      files: filesFor(daemon.endpoint),
    });

    const resource = await client.fetchResourceProjection("runtime");
    const resourceWatch = await client.fetchResourceWatch("runtime", 4);
    const taskWatch = await client.fetchTaskWatch(7);

    assert.equal(resource.family, "runtime");
    assert.equal(resource.availability, "not-backed");
    assert.match(resourceWatch, /^event: snapshot\n/);
    assert.match(taskWatch, /^event: snapshot\n/);

    const sessionChannels = daemon.requests
      .filter((request) => request.url === "/local/session")
      .map((request) => JSON.parse(request.body)["channel"]);
    assert.deepEqual(sessionChannels, ["management", "task"]);

    const resourceRequest = daemon.requests.find(
      (request) => request.url === "/resource/v1/projection?family=runtime&version=1",
    );
    const resourceWatchRequest = daemon.requests.find(
      (request) => request.url === "/resource/v1/watch?family=runtime&version=1&resume_from=4",
    );
    const taskWatchRequest = daemon.requests.find(
      (request) => request.url === "/task/watch?resume_from=7",
    );
    assert.ok(resourceRequest);
    assert.ok(resourceWatchRequest);
    assert.ok(taskWatchRequest);
    assert.notEqual(
      resourceWatchRequest.headers["authorization"],
      taskWatchRequest.headers["authorization"],
      "resource and Task channels must not share a bearer",
    );
  } finally {
    await daemon.close();
  }
});

test("no cookie is ever sent and the bootstrap secret goes only to /local/session", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: readinessProjectionBody(),
  });
  try {
    const client = new PersonalDaemonClient({
      environment: ENVIRONMENT,
      files: filesFor(daemon.endpoint),
    });
    await client.fetchReadiness();

    for (const request of daemon.requests) {
      assert.equal(request.headers["cookie"], undefined, `${request.url} must not carry a cookie`);
      if (request.url !== "/local/session") {
        assert.ok(
          !request.body.includes(BOOTSTRAP_SECRET),
          `${request.url} must not carry the bootstrap secret`,
        );
        assert.ok(!JSON.stringify(request.headers).includes(BOOTSTRAP_SECRET));
      }
    }
  } finally {
    await daemon.close();
  }
});

test("a daemon restart invalidating the bearer is recovered exactly once", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: readinessProjectionBody(),
    unauthorizedStatusResponses: 1,
  });
  try {
    const client = new PersonalDaemonClient({
      environment: ENVIRONMENT,
      files: filesFor(daemon.endpoint),
    });
    const projection = await client.fetchReadiness();
    assert.equal(projection.surface, "personal-status");
    assert.equal(daemon.issuedTokens.length, 2, "the client must re-mint after a 401");
    assert.equal(
      daemon.requests.filter((request) => request.url === "/personal/status").length,
      2,
    );
  } finally {
    await daemon.close();
  }
});

test("a persistently refusing daemon fails explicitly instead of retrying forever", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: readinessProjectionBody(),
    unauthorizedStatusResponses: 10,
  });
  try {
    const client = new PersonalDaemonClient({
      environment: ENVIRONMENT,
      files: filesFor(daemon.endpoint),
    });
    const error = (await captureRejection(() => client.fetchReadiness())) as DaemonClientError;
    assert.ok(error instanceof DaemonClientError);
    assert.equal(error.code, "PI_EXTENSION_DAEMON_AUTH_REFUSED");
    assert.equal(
      daemon.requests.filter((request) => request.url === "/personal/status").length,
      2,
      "exactly one re-mint, then an explicit failure",
    );
  } finally {
    await daemon.close();
  }
});

test("a wrong bootstrap secret is an explicit auth refusal carrying the daemon code", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: readinessProjectionBody(),
  });
  try {
    const client = new PersonalDaemonClient({
      environment: ENVIRONMENT,
      files: filesFor(daemon.endpoint, "boot-wrong-secret-value"),
    });
    let captured: DaemonClientError | undefined;
    try {
      await client.fetchReadiness();
    } catch (error) {
      captured = error as DaemonClientError;
    }
    assert.ok(captured instanceof DaemonClientError);
    assert.equal(captured.code, "PI_EXTENSION_DAEMON_AUTH_REFUSED");
    assert.equal(captured.daemonErrorCode, "LOCAL_BOOTSTRAP_MISMATCH");
    assert.equal(captured.httpStatus, 401);
    assert.ok(!captured.message.includes("boot-wrong-secret-value"));
    assert.ok(!isDaemonUnavailable(captured));
  } finally {
    await daemon.close();
  }
});

test("a closed port is reported as an unreachable daemon, not as a degraded read", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: readinessProjectionBody(),
  });
  const endpoint = daemon.endpoint;
  await daemon.close();

  const client = new PersonalDaemonClient({
    environment: ENVIRONMENT,
    files: filesFor(endpoint),
    requestTimeoutMs: 1_000,
  });
  let captured: DaemonClientError | undefined;
  try {
    await client.fetchReadiness();
  } catch (error) {
    captured = error as DaemonClientError;
  }
  assert.ok(captured instanceof DaemonClientError);
  assert.equal(captured.code, "PI_EXTENSION_DAEMON_UNREACHABLE");
  assert.ok(isDaemonUnavailable(captured));
  assert.match(captured.message, /cognitive daemon start/);
});

test("malformed projections are protocol errors, never an assumed ready", () => {
  const rejected: readonly string[] = [
    "not json",
    "[]",
    readinessProjectionBody({ overall: "totally-fine" }),
    readinessProjectionBody({ first_conversation_ready: "yes" }),
    readinessProjectionBody({ schema_version: "1" }),
    readinessProjectionBody({ components: "none" }),
    readinessProjectionBody({ static_check_is_not_runtime_ready: "true" }),
    readinessProjectionBody({ profile_claim: 1 }),
    readinessProjectionBody({ gate_claim: null }),
  ];
  for (const body of rejected) {
    const error = captureThrown(() => parseReadinessProjection(body)) as DaemonClientError;
    assert.equal(error.code, "PI_EXTENSION_DAEMON_PROTOCOL_ERROR");
  }
});

test("a projection claiming an authority side effect is refused", () => {
  const error = captureThrown(() => parseReadinessProjection(readinessProjectionBody({ authority_side_effects: true }))) as DaemonClientError;
  assert.equal(error.code, "PI_EXTENSION_DAEMON_PROTOCOL_ERROR");
  assert.match(error.message, /authority side effect/);
});

test("a ready projection is reported as ready without being reinterpreted", () => {
  const projection = parseReadinessProjection(
    readinessProjectionBody({
      overall: "ready",
      first_conversation_ready: true,
      components: [
        { component: "pi", status: "ready", required: false, error_class: null, duration_ms: 0 },
      ],
    }),
  );
  assert.equal(projection.overall, "ready");
  assert.equal(projection.firstConversationReady, true);
  assert.equal(projection.components[0]?.errorClass, undefined);
});
