/**
 * P9-T07 required negatives for the Pi route observation surface.
 *
 * Each test states one way the instrumentation could produce a measurement
 * that looks conclusive but is not, or could exceed the boundary a
 * non-authority observer is allowed to occupy. They are grouped by the failure
 * they prevent rather than by the function they call.
 */

import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";

import { PersonalDaemonClient } from "./daemon-client.js";
import type { ProviderUsage } from "./daemon-client.js";
import type { EnvironmentSlice, FileReader } from "./daemon-discovery.js";
import { resolvePersonalDaemonPaths } from "./daemon-discovery.js";
import { createDaemonProvider } from "./daemon-provider.js";
import {
  PI_ROUTE_OBSERVATION_MAX_RECORD_BYTES,
  PI_ROUTE_OBSERVATION_SCHEMA,
  PiRouteObservationError,
  PiRouteStageRecorder,
  assemblePiRouteObservation,
  createPiRouteCorrelationId,
  openPiRouteObservationSession,
  parseDaemonReportedNanos,
  parsePiRouteCorrelationId,
  personalAuthorityRoots,
  resolvePiRouteObservationAuthorization,
  validatePiRouteObservation,
  validateSinkPath,
  type ObservationSinkWriter,
  type PiRouteObservation,
  type PiRouteObservationSession,
  type PiRouteStageTiming,
} from "./pi-route-observation.js";
import { captureThrown, startFakeDaemon } from "./test-support.js";

const BOOTSTRAP_SECRET = "boot-0123456789abcdef-fedcba9876543210";
const PROMPT_SENTINEL = "sentinel-prompt-text-never-observed";
const RESPONSE_SENTINEL = "sentinel-response-text-never-observed";
const CAMPAIGN_ID = "PERSONAL-PERF-EVAL-003";
const AUTHORIZED_ENVIRONMENT: EnvironmentSlice = {
  HOME: path.join(path.sep, "home", "owner"),
  XDG_RUNTIME_DIR: path.join(path.sep, "run", "user", "1000"),
  COGNITIVEOS_PI_ROUTE_OBSERVATION: "enabled",
  COGNITIVEOS_PI_ROUTE_OBSERVATION_CAMPAIGN: CAMPAIGN_ID,
};
const UNAUTHORIZED_ENVIRONMENT: EnvironmentSlice = {
  HOME: path.join(path.sep, "home", "owner"),
  XDG_RUNTIME_DIR: path.join(path.sep, "run", "user", "1000"),
};

function clientFor(endpoint: string, environment: EnvironmentSlice): PersonalDaemonClient {
  const paths = resolvePersonalDaemonPaths(environment);
  const contents: Record<string, string> = {
    [paths.endpointFile]: JSON.stringify({
      schema_version: 1,
      endpoint,
      surface: "personal-daemon-endpoint",
    }),
    [paths.bootstrapSecretFile]: `${BOOTSTRAP_SECRET}\n`,
  };
  const files: FileReader = {
    readTextFile: (filePath) =>
      contents[filePath] ??
      (() => {
        throw new Error(`ENOENT: ${filePath}`);
      })(),
  };
  return new PersonalDaemonClient({ environment, files, requestTimeoutMs: 2_000 });
}

/** A complete, valid set of Pi-domain stages with a generous loopback wait. */
function piStages(loopbackWaitNanos = 10_000_000): readonly PiRouteStageTiming[] {
  return [
    { stage: "pi_request_preparation", domain: "pi", elapsedNanos: 100_000 },
    { stage: "extension_dispatch", domain: "pi", elapsedNanos: 200_000 },
    { stage: "loopback_wait", domain: "pi", elapsedNanos: loopbackWaitNanos },
    { stage: "response_parse", domain: "pi", elapsedNanos: 50_000 },
    { stage: "pi_event_delivery", domain: "pi", elapsedNanos: 25_000 },
  ];
}

function observationWith(
  overrides: Partial<PiRouteObservation> = {},
  usage: ProviderUsage = { availability: "not_available" },
): PiRouteObservation {
  return {
    schema: PI_ROUTE_OBSERVATION_SCHEMA,
    campaignId: CAMPAIGN_ID,
    correlationId: createPiRouteCorrelationId(),
    requestMode: "non_streaming",
    outcome: "completed",
    failureClass: "none",
    terminalStage: "pi_event_delivery",
    stages: piStages(),
    daemonStages: "not_available",
    daemonStagesUnavailableReason: "not_reported",
    providerUsage: usage,
    ...overrides,
  };
}

function refusalCode(action: () => unknown): string {
  const thrown = captureThrown(action);
  assert.ok(thrown instanceof PiRouteObservationError, `expected a refusal, got ${String(thrown)}`);
  return thrown.code;
}

// ---------------------------------------------------------------------------
// Negative 1 — malformed or duplicate correlation id
// ---------------------------------------------------------------------------

test("a malformed correlation id is refused before it can join two observations", () => {
  for (const malformed of [
    "",
    "campaign-",
    "campaign-0123456789abcdef",
    "campaign-0123456789ABCDEF0123456789ABCDEF",
    "campaign-0123456789abcdef0123456789abcdefff",
    "campaign-0123456789abcdef0123456789abcdeg",
    "Bearer sess-fake-1",
    "sk-0123456789abcdef0123456789abcdef",
    42,
    undefined,
  ]) {
    assert.equal(
      refusalCode(() => parsePiRouteCorrelationId(malformed)),
      "PI_ROUTE_OBSERVATION_CORRELATION_ID_INVALID",
    );
  }
  assert.match(createPiRouteCorrelationId(), /^campaign-[0-9a-f]{32}$/);
});

test("one correlation id cannot be published twice in a session", () => {
  const session = openPiRouteObservationSession(AUTHORIZED_ENVIRONMENT);
  assert.ok(session !== undefined);
  const observation = observationWith();
  session.publish(observation);
  assert.equal(
    refusalCode(() => session.publish(observation)),
    "PI_ROUTE_OBSERVATION_CORRELATION_ID_DUPLICATE",
  );
  assert.equal(session.observations.length, 1);
});

test("a daemon echo that names a different request never joins the two sides", () => {
  const observation = assemblePiRouteObservation({
    campaignId: CAMPAIGN_ID,
    correlationId: createPiRouteCorrelationId(),
    piStages: piStages(),
    daemonReported: {
      echoedCorrelationId: createPiRouteCorrelationId(),
      preflightElapsedNanos: 1_000,
      providerNetworkElapsedNanos: 4_000,
    },
    providerUsage: { availability: "not_available" },
  });
  assert.equal(observation.daemonStages, "not_available");
  assert.equal(observation.daemonStagesUnavailableReason, "correlation_mismatch");
  assert.ok(!observation.stages.some((stage) => stage.domain === "daemon"));
});

// ---------------------------------------------------------------------------
// Negative 2 — missing or overlapping stages
// ---------------------------------------------------------------------------

test("a Pi-domain stage cannot overlap another, and cannot be measured twice", () => {
  const recorder = new PiRouteStageRecorder();
  recorder.begin("pi_request_preparation");
  assert.equal(
    refusalCode(() => recorder.begin("extension_dispatch")),
    "PI_ROUTE_OBSERVATION_STAGE_OVERLAP",
  );
  assert.equal(
    refusalCode(() => recorder.complete("extension_dispatch")),
    "PI_ROUTE_OBSERVATION_STAGE_NOT_OPEN",
  );
  recorder.complete("pi_request_preparation");
  assert.equal(
    refusalCode(() => recorder.begin("pi_request_preparation")),
    "PI_ROUTE_OBSERVATION_STAGE_DUPLICATE",
  );
});

test("an observation missing a Pi-measured stage is refused rather than partially reported", () => {
  for (const omitted of [
    "pi_request_preparation",
    "extension_dispatch",
    "loopback_wait",
    "response_parse",
    "pi_event_delivery",
  ] as const) {
    const stages = piStages().filter((stage) => stage.stage !== omitted);
    assert.equal(
      refusalCode(() => validatePiRouteObservation(observationWith({ stages }))),
      "PI_ROUTE_OBSERVATION_STAGE_MISSING",
    );
  }
});

test("nested daemon stages that outlast their loopback wait are dropped, never trimmed", () => {
  const observation = assemblePiRouteObservation({
    campaignId: CAMPAIGN_ID,
    correlationId: "campaign-0123456789abcdef0123456789abcdef",
    piStages: piStages(5_000),
    daemonReported: {
      echoedCorrelationId: "campaign-0123456789abcdef0123456789abcdef",
      preflightElapsedNanos: 4_000,
      providerNetworkElapsedNanos: 4_000,
    },
    providerUsage: { availability: "not_available" },
  });
  assert.equal(observation.daemonStages, "not_available");
  assert.equal(observation.daemonStagesUnavailableReason, "exceeds_loopback_wait");

  const narrowWait = piStages(5_000);
  const fabricated = observationWith({
    stages: [
      ...narrowWait.slice(0, 3),
      { stage: "daemon_preflight", domain: "daemon", elapsedNanos: 4_000 },
      { stage: "provider_network", domain: "daemon", elapsedNanos: 4_000 },
      ...narrowWait.slice(3),
    ],
    daemonStages: "joined",
    daemonStagesUnavailableReason: null,
  });
  assert.equal(
    refusalCode(() => validatePiRouteObservation(fabricated)),
    "PI_ROUTE_OBSERVATION_NESTED_STAGES_EXCEED_PARENT",
  );
});

test("only one of the two daemon stages is not a joined daemon domain", () => {
  const observation = assemblePiRouteObservation({
    campaignId: CAMPAIGN_ID,
    correlationId: "campaign-0123456789abcdef0123456789abcdef",
    piStages: piStages(),
    daemonReported: {
      echoedCorrelationId: "campaign-0123456789abcdef0123456789abcdef",
      preflightElapsedNanos: undefined,
      providerNetworkElapsedNanos: 4_000,
    },
    providerUsage: { availability: "not_available" },
  });
  assert.equal(observation.daemonStagesUnavailableReason, "incomplete_stage_group");
});

test("an unknown daemon-stage availability label cannot masquerade as joined", () => {
  const forged = observationWith({
    stages: [
      ...piStages().slice(0, 3),
      { stage: "daemon_preflight", domain: "daemon", elapsedNanos: 1_000 },
      { stage: "provider_network", domain: "daemon", elapsedNanos: 4_000 },
      ...piStages().slice(3),
    ],
    daemonStages: "estimated" as never,
    daemonStagesUnavailableReason: null,
  });
  assert.equal(
    refusalCode(() => validatePiRouteObservation(forged)),
    "PI_ROUTE_OBSERVATION_DAEMON_AVAILABILITY_INCOHERENT",
  );
});

test("an unknown daemon-stage unavailability reason is refused", () => {
  const forged = observationWith({
    daemonStagesUnavailableReason: "estimated" as never,
  });
  assert.equal(
    refusalCode(() => validatePiRouteObservation(forged)),
    "PI_ROUTE_OBSERVATION_DAEMON_AVAILABILITY_INCOHERENT",
  );
});

test("a duplicated, unknown, misattributed or out-of-order stage is refused", () => {
  const duplicated = observationWith({ stages: [...piStages(), ...piStages()] });
  assert.equal(
    refusalCode(() => validatePiRouteObservation(duplicated)),
    "PI_ROUTE_OBSERVATION_STAGE_DUPLICATE",
  );

  const unknown = observationWith({
    stages: [
      ...piStages(),
      { stage: "provider_billing" as never, domain: "daemon", elapsedNanos: 1_000 },
    ],
  });
  assert.equal(
    refusalCode(() => validatePiRouteObservation(unknown)),
    "PI_ROUTE_OBSERVATION_STAGE_UNKNOWN",
  );

  const misattributed = observationWith({
    stages: piStages().map((stage) =>
      stage.stage === "loopback_wait" ? { ...stage, domain: "daemon" as const } : stage,
    ),
  });
  assert.equal(
    refusalCode(() => validatePiRouteObservation(misattributed)),
    "PI_ROUTE_OBSERVATION_STAGE_WRONG_DOMAIN",
  );

  const outOfOrder = observationWith({ stages: [...piStages()].reverse() });
  assert.equal(
    refusalCode(() => validatePiRouteObservation(outOfOrder)),
    "PI_ROUTE_OBSERVATION_STAGE_OUT_OF_ORDER",
  );
});

// ---------------------------------------------------------------------------
// Negative 3 — zero or negative duration
// ---------------------------------------------------------------------------

test("a zero, negative or non-integer stage duration is refused", () => {
  for (const elapsedNanos of [0, -1, -0.5, 1.5, Number.NaN, Number.MAX_SAFE_INTEGER + 2]) {
    const stages = piStages().map((stage) =>
      stage.stage === "response_parse" ? { ...stage, elapsedNanos } : stage,
    );
    assert.equal(
      refusalCode(() => validatePiRouteObservation(observationWith({ stages }))),
      "PI_ROUTE_OBSERVATION_DURATION_INVALID",
    );
  }
});

test("a daemon-reported duration that is not a positive integer is not read as a stage", () => {
  for (const header of ["0", "-1", "1.5", "", " 12", "12 ", "abc", "0x10", null, undefined]) {
    assert.equal(parseDaemonReportedNanos(header), undefined);
  }
  assert.equal(parseDaemonReportedNanos("1"), 1);
  assert.equal(parseDaemonReportedNanos("4000"), 4_000);
  // Beyond the safe-integer range the value is not a number we may compare.
  assert.equal(parseDaemonReportedNanos("9007199254740993"), undefined);
});

test("a clock that runs backwards is refused instead of yielding a negative stage", () => {
  const readings = [100, 40];
  const recorder = new PiRouteStageRecorder({ clock: () => readings.shift() ?? 0 });
  recorder.begin("pi_request_preparation");
  assert.equal(
    refusalCode(() => recorder.complete("pi_request_preparation")),
    "PI_ROUTE_OBSERVATION_CLOCK_NOT_MONOTONIC",
  );
});

test("a stage shorter than the clock resolution is floored at one nanosecond, never zero", () => {
  const recorder = new PiRouteStageRecorder({ clock: () => 7 });
  recorder.begin("pi_request_preparation");
  recorder.complete("pi_request_preparation");
  assert.equal(recorder.readStage("pi_request_preparation"), 1);
});

// ---------------------------------------------------------------------------
// Negative 4 — internally inconsistent Provider usage counters
// ---------------------------------------------------------------------------

test("usage counters that do not add up are refused rather than reported as measured", () => {
  for (const usage of [
    { availability: "measured", promptTokens: 2, completionTokens: 3, totalTokens: 4 },
    { availability: "measured", promptTokens: -1, completionTokens: 1, totalTokens: 0 },
    { availability: "measured", promptTokens: 1.5, completionTokens: 1.5, totalTokens: 3 },
  ] as ProviderUsage[]) {
    assert.equal(
      refusalCode(() => validatePiRouteObservation(observationWith({}, usage))),
      "PI_ROUTE_OBSERVATION_PROVIDER_USAGE_INCONSISTENT",
    );
  }
});

test("an unknown usage availability label cannot bypass measured-usage provenance", () => {
  const forgedUsage = {
    availability: "estimated",
    promptTokens: 7,
    completionTokens: 3,
    totalTokens: 10,
  } as unknown as ProviderUsage;
  const session = openPiRouteObservationSession(AUTHORIZED_ENVIRONMENT);
  assert.ok(session !== undefined);
  assert.equal(
    refusalCode(() => session.publish(observationWith({}, forgedUsage))),
    "PI_ROUTE_OBSERVATION_PROVIDER_USAGE_INCONSISTENT",
  );
  assert.equal(session.observations.length, 0);
});

test("self-asserted measured usage cannot be assembled or published as Provider evidence", () => {
  const forgedUsage: ProviderUsage = {
    availability: "measured",
    promptTokens: 7,
    completionTokens: 3,
    totalTokens: 10,
  };
  assert.equal(
    refusalCode(() =>
      assemblePiRouteObservation({
        campaignId: CAMPAIGN_ID,
        correlationId: "campaign-0123456789abcdef0123456789abcdef",
        piStages: piStages(),
        daemonReported: {
          echoedCorrelationId: undefined,
          preflightElapsedNanos: undefined,
          providerNetworkElapsedNanos: undefined,
        },
        providerUsage: forgedUsage,
      }),
    ),
    "PI_ROUTE_OBSERVATION_PROVIDER_USAGE_UNVERIFIED",
  );

  const session = openPiRouteObservationSession(AUTHORIZED_ENVIRONMENT);
  assert.ok(session !== undefined);
  assert.equal(
    refusalCode(() => session.publish(observationWith({}, forgedUsage))),
    "PI_ROUTE_OBSERVATION_PROVIDER_USAGE_UNVERIFIED",
  );
  assert.equal(session.observations.length, 0);
});

test("published measured usage and stage timings are immutable to the campaign runner", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: "{}",
    completionBody: JSON.stringify({
      choices: [{ message: { content: "daemon text" }, finish_reason: "stop" }],
      usage: { prompt_tokens: 7, completion_tokens: 3, total_tokens: 10 },
    }),
  });
  try {
    const session = openPiRouteObservationSession(AUTHORIZED_ENVIRONMENT);
    assert.ok(session !== undefined);
    await runOneCompletion(daemon.endpoint, session);
    const observation = session.observations[0]!;
    const measuredUsage = observation.providerUsage as {
      promptTokens: number;
      completionTokens: number;
      totalTokens: number;
    };
    const firstStage = observation.stages[0] as { elapsedNanos: number };

    assert.throws(() => {
      measuredUsage.totalTokens = 999;
    }, TypeError);
    assert.throws(() => {
      firstStage.elapsedNanos = 999;
    }, TypeError);
    assert.deepEqual(observation.providerUsage, {
      availability: "measured",
      promptTokens: 7,
      completionTokens: 3,
      totalTokens: 10,
    });
  } finally {
    await daemon.close();
  }
});

test("measured usage cannot be replayed under another request or campaign session", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: "{}",
    completionBody: JSON.stringify({
      choices: [{ message: { content: "daemon text" }, finish_reason: "stop" }],
      usage: { prompt_tokens: 7, completion_tokens: 3, total_tokens: 10 },
    }),
  });
  try {
    const session = openPiRouteObservationSession(AUTHORIZED_ENVIRONMENT);
    assert.ok(session !== undefined);
    await runOneCompletion(daemon.endpoint, session);
    const measuredUsage = session.observations[0]!.providerUsage;

    assert.equal(
      refusalCode(() =>
        assemblePiRouteObservation({
          campaignId: CAMPAIGN_ID,
          correlationId: createPiRouteCorrelationId(),
          piStages: piStages(),
          daemonReported: {
            echoedCorrelationId: undefined,
            preflightElapsedNanos: undefined,
            providerNetworkElapsedNanos: undefined,
          },
          providerUsage: measuredUsage,
        }),
      ),
      "PI_ROUTE_OBSERVATION_PROVIDER_USAGE_UNVERIFIED",
    );

    const secondSession = openPiRouteObservationSession(AUTHORIZED_ENVIRONMENT);
    assert.ok(secondSession !== undefined);
    assert.equal(
      refusalCode(() => secondSession.publish(session.observations[0]!)),
      "PI_ROUTE_OBSERVATION_PROVIDER_USAGE_UNVERIFIED",
    );
    assert.equal(secondSession.observations.length, 0);
  } finally {
    await daemon.close();
  }
});

test("a partial or absent Provider usage object stays not_available across the whole route", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: "{}",
    completionBody: JSON.stringify({
      choices: [{ message: { content: "daemon text" }, finish_reason: "stop" }],
      usage: { prompt_tokens: 7, total_tokens: 10 },
    }),
  });
  try {
    const session = openPiRouteObservationSession(AUTHORIZED_ENVIRONMENT);
    assert.ok(session !== undefined);
    await runOneCompletion(daemon.endpoint, session);
    assert.deepEqual(session.observations[0]?.providerUsage, { availability: "not_available" });
  } finally {
    await daemon.close();
  }
});

// ---------------------------------------------------------------------------
// Negative 5 — secret-shaped observation
// ---------------------------------------------------------------------------

test("a secret-shaped campaign id is refused, and never authorizes a session", () => {
  for (const campaignId of [
    "SK-0123456789ABCDEF0123456789ABCDEF",
    "CAMPAIGN-SECRET-001",
    "PROVIDER-TOKEN-001",
    "BEARER-EVAL-001",
    "0123456789ABCDEF0123456789ABCDEF01234567",
  ]) {
    const authorization = resolvePiRouteObservationAuthorization({
      COGNITIVEOS_PI_ROUTE_OBSERVATION: "enabled",
      COGNITIVEOS_PI_ROUTE_OBSERVATION_CAMPAIGN: campaignId,
    });
    assert.equal(authorization.authorized, false);
    assert.ok(
      authorization.authorized === false &&
        (authorization.reason === "campaign_secret_shaped" ||
          authorization.reason === "campaign_invalid"),
    );
  }
});

test("a refusal never echoes the value it refused", () => {
  const leaked = "SK-0123456789ABCDEF0123456789ABCDEF";
  const thrown = captureThrown(() =>
    assemblePiRouteObservation({
      campaignId: leaked,
      correlationId: createPiRouteCorrelationId(),
      piStages: piStages(),
      daemonReported: {
        echoedCorrelationId: undefined,
        preflightElapsedNanos: undefined,
        providerNetworkElapsedNanos: undefined,
      },
      providerUsage: { availability: "not_available" },
    }),
  );
  assert.ok(thrown instanceof PiRouteObservationError);
  assert.ok(!thrown.message.includes(leaked));
  assert.ok(!String(thrown.stack ?? "").includes(leaked));
});

// ---------------------------------------------------------------------------
// Negative 6 — raw body or header capture
// ---------------------------------------------------------------------------

test("a published observation carries no prompt, response, header or bearer material", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: "{}",
    completionBody: JSON.stringify({
      choices: [{ message: { content: RESPONSE_SENTINEL }, finish_reason: "stop" }],
      usage: { prompt_tokens: 7, completion_tokens: 3, total_tokens: 10 },
    }),
    providerNetworkElapsedNanos: "4000",
    daemonPreflightElapsedNanos: "1000",
    echoCorrelationId: true,
  });
  try {
    const session = openPiRouteObservationSession(AUTHORIZED_ENVIRONMENT);
    assert.ok(session !== undefined);
    await runOneCompletion(daemon.endpoint, session, PROMPT_SENTINEL);

    const observation = session.observations[0];
    assert.ok(observation !== undefined);
    const serialized = JSON.stringify(observation);
    for (const forbidden of [
      PROMPT_SENTINEL,
      RESPONSE_SENTINEL,
      BOOTSTRAP_SECRET,
      "Bearer",
      "authorization",
      "sess-fake",
      "127.0.0.1",
      "/provider/v1/chat/completions",
    ]) {
      assert.ok(
        !serialized.includes(forbidden),
        `an observation must not carry ${forbidden}: it is content-free by construction`,
      );
    }
    assert.deepEqual(Object.keys(observation).sort(), [
      "campaignId",
      "correlationId",
      "daemonStages",
      "daemonStagesUnavailableReason",
      "failureClass",
      "outcome",
      "providerUsage",
      "requestMode",
      "schema",
      "stages",
      "terminalStage",
    ]);
    for (const stage of observation.stages) {
      assert.deepEqual(Object.keys(stage).sort(), ["domain", "elapsedNanos", "stage"]);
    }
    assert.ok(Buffer.byteLength(serialized, "utf8") <= PI_ROUTE_OBSERVATION_MAX_RECORD_BYTES);
  } finally {
    await daemon.close();
  }
});

test("a runner cannot smuggle an extra prompt or credential field into the sink schema", () => {
  const writes: string[] = [];
  const sinkPath = path.join(path.sep, "tmp", "campaign", "observations.ndjson");
  const session = openPiRouteObservationSession(
    {
      ...AUTHORIZED_ENVIRONMENT,
      COGNITIVEOS_PI_ROUTE_OBSERVATION_SINK: sinkPath,
    },
    {
      sinkWriter: {
        appendLine: (_target, line) => writes.push(line),
      },
    },
  );
  assert.ok(session !== undefined);
  const smuggled = {
    ...observationWith(),
    prompt: PROMPT_SENTINEL,
    authorization: "Bearer should-never-enter-an-observation",
  } as PiRouteObservation;

  assert.equal(
    refusalCode(() => session.publish(smuggled)),
    "PI_ROUTE_OBSERVATION_SCHEMA_INVALID",
  );
  assert.deepEqual(writes, []);
  assert.equal(session.observations.length, 0);
});

test("a record that outgrew the content-free ceiling is refused rather than retained", () => {
  const session = openPiRouteObservationSession(AUTHORIZED_ENVIRONMENT);
  assert.ok(session !== undefined);
  const oversized = observationWith({
    stages: piStages().map((stage) => ({
      ...stage,
      // A field long enough to breach the ceiling can only come from content
      // that does not belong in an observation.
      stage: `${stage.stage}${"x".repeat(PI_ROUTE_OBSERVATION_MAX_RECORD_BYTES)}` as never,
    })),
  });
  assert.ok(
    ["PI_ROUTE_OBSERVATION_STAGE_UNKNOWN", "PI_ROUTE_OBSERVATION_RECORD_TOO_LARGE"].includes(
      refusalCode(() => session.publish(oversized)),
    ),
  );
  assert.equal(session.observations.length, 0);
});

// ---------------------------------------------------------------------------
// Negative 7 — instrumentation enabled without authorization
// ---------------------------------------------------------------------------

test("instrumentation is denied by default and by every partial authorization", () => {
  const denials: readonly [EnvironmentSlice, string][] = [
    [{}, "not_requested"],
    [{ COGNITIVEOS_PI_ROUTE_OBSERVATION: "" }, "not_requested"],
    [{ COGNITIVEOS_PI_ROUTE_OBSERVATION: "1" }, "not_enabled"],
    [{ COGNITIVEOS_PI_ROUTE_OBSERVATION: "true" }, "not_enabled"],
    [{ COGNITIVEOS_PI_ROUTE_OBSERVATION: "ENABLED" }, "not_enabled"],
    [{ COGNITIVEOS_PI_ROUTE_OBSERVATION: "enabled" }, "campaign_missing"],
    [
      { COGNITIVEOS_PI_ROUTE_OBSERVATION: "enabled", COGNITIVEOS_PI_ROUTE_OBSERVATION_CAMPAIGN: "eval" },
      "campaign_invalid",
    ],
  ];
  for (const [environment, reason] of denials) {
    const authorization = resolvePiRouteObservationAuthorization(environment);
    assert.equal(authorization.authorized, false);
    assert.equal(authorization.authorized === false ? authorization.reason : "", reason);
    assert.equal(openPiRouteObservationSession(environment), undefined);
  }
});

test("an unauthorized Pi run measures nothing and behaves exactly as before", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: "{}",
    providerNetworkElapsedNanos: "4000",
    daemonPreflightElapsedNanos: "1000",
    echoCorrelationId: true,
  });
  try {
    const client = clientFor(daemon.endpoint, UNAUTHORIZED_ENVIRONMENT);
    assert.equal(client.openCampaignObservationSession(), undefined);

    const provider = await createDaemonProvider(client);
    const stream = provider.streamSimple(provider.models[0]!, {
      messages: [{ role: "user", content: PROMPT_SENTINEL }],
    });
    const events: string[] = [];
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
  } finally {
    await daemon.close();
  }
});

test("a closed session accepts no further records", () => {
  const session = openPiRouteObservationSession(AUTHORIZED_ENVIRONMENT);
  assert.ok(session !== undefined);
  session.close();
  assert.equal(
    refusalCode(() => session.publish(observationWith())),
    "PI_ROUTE_OBSERVATION_NOT_AUTHORIZED",
  );
});

test("an observation from another campaign cannot enter this session", () => {
  const session = openPiRouteObservationSession(AUTHORIZED_ENVIRONMENT);
  assert.ok(session !== undefined);
  assert.equal(
    refusalCode(() => session.publish(observationWith({ campaignId: "PERSONAL-PERF-EVAL-999" }))),
    "PI_ROUTE_OBSERVATION_CAMPAIGN_ID_INVALID",
  );
});

// ---------------------------------------------------------------------------
// Negative 8 — instrumentation writing authority state
// ---------------------------------------------------------------------------

test("a sink inside any Personal state, runtime or config root is refused", () => {
  const roots = personalAuthorityRoots(AUTHORIZED_ENVIRONMENT);
  assert.equal(roots.length, 3);
  for (const root of roots) {
    assert.equal(
      refusalCode(() => validateSinkPath(path.join(root, "observations.ndjson"), roots)),
      "PI_ROUTE_OBSERVATION_SINK_TARGETS_AUTHORITY_ROOT",
    );
    assert.equal(
      refusalCode(() => validateSinkPath(path.join(root, "nested", "run.ndjson"), roots)),
      "PI_ROUTE_OBSERVATION_SINK_TARGETS_AUTHORITY_ROOT",
    );
  }
  const outside = path.join(path.sep, "tmp", "campaign", "observations.ndjson");
  assert.equal(validateSinkPath(outside, roots), path.resolve(outside));
});

test("a relative or non-NDJSON sink target is refused", () => {
  for (const sinkPath of [
    "observations.ndjson",
    path.join(path.sep, "tmp", "observations.json"),
    path.join(path.sep, "tmp", "observations"),
  ]) {
    assert.equal(
      refusalCode(() => validateSinkPath(sinkPath, [])),
      "PI_ROUTE_OBSERVATION_SINK_PATH_INVALID",
    );
  }
});

test("a session with no injected writer writes nothing, even with a sink named", () => {
  const writes: string[] = [];
  const recordingWriter: ObservationSinkWriter = {
    appendLine: (sinkPath, line) => writes.push(`${sinkPath}:${line}`),
  };
  const sinkPath = path.join(path.sep, "tmp", "campaign", "observations.ndjson");
  const environmentWithSink: EnvironmentSlice = {
    ...AUTHORIZED_ENVIRONMENT,
    COGNITIVEOS_PI_ROUTE_OBSERVATION_SINK: sinkPath,
  };

  const withoutWriter = openPiRouteObservationSession(environmentWithSink);
  assert.ok(withoutWriter !== undefined);
  assert.equal(withoutWriter.sinkPath, path.resolve(sinkPath));
  withoutWriter.publish(observationWith());
  assert.equal(writes.length, 0, "the package itself must never open the sink");

  const withWriter = openPiRouteObservationSession(environmentWithSink, {
    sinkWriter: recordingWriter,
  });
  assert.ok(withWriter !== undefined);
  withWriter.publish(observationWith());
  assert.equal(writes.length, 1, "an embedding harness supplies the only writer");
});

test("an instrumented run issues no request beyond the completion the operator asked for", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: "{}",
    providerNetworkElapsedNanos: "4000",
    daemonPreflightElapsedNanos: "1000",
    echoCorrelationId: true,
  });
  try {
    const session = openPiRouteObservationSession(AUTHORIZED_ENVIRONMENT);
    assert.ok(session !== undefined);
    await runOneCompletion(daemon.endpoint, session);

    assert.deepEqual(
      daemon.requests.map((request) => `${request.method} ${request.url}`),
      [
        "POST /local/session",
        "GET /provider/v1/selected-model",
        "POST /provider/v1/chat/completions",
      ],
      "measurement must add no request, and certainly no mutation",
    );
    assert.equal(session.observations.length, 1);
    assert.equal(session.observations[0]?.daemonStages, "joined");
  } finally {
    await daemon.close();
  }
});

async function runOneCompletion(
  endpoint: string,
  session: PiRouteObservationSession,
  prompt = "hello",
): Promise<void> {
  const provider = await createDaemonProvider(clientFor(endpoint, AUTHORIZED_ENVIRONMENT), {
    session,
  });
  const stream = provider.streamSimple(provider.models[0]!, {
    messages: [{ role: "user", content: prompt }],
  });
  for await (const _event of stream) {
    // Drain the stream so Pi event delivery is part of the measured run.
  }
  await stream.result();
}
