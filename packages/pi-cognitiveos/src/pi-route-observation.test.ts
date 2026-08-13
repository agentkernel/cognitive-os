/**
 * P9-T05 failure-first proof.
 *
 * `PERSONAL-PERF-EVAL-002` measured a +1828.5 ms median OS-arm overhead whose
 * stage is still only inferred. This test states the missing capability as an
 * executable requirement: one real Pi run must produce a single joined,
 * monotonic, non-overlapping per-stage observation covering both the Pi side
 * and the daemon side, and must surface the Provider usage the assistant
 * message already carries so a campaign runner can read it.
 *
 * It resolves the entry point dynamically so it compiles against today's
 * package surface and fails at run time on the capability itself, rather than
 * failing as a type error that hides which requirement is unmet.
 */

import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";

import { PersonalDaemonClient } from "./daemon-client.js";
import type { EnvironmentSlice, FileReader } from "./daemon-discovery.js";
import { resolvePersonalDaemonPaths } from "./daemon-discovery.js";
import { createDaemonProvider } from "./daemon-provider.js";
import type { ProviderConfig } from "./pi-api.js";
import { startFakeDaemon } from "./test-support.js";

const BOOTSTRAP_SECRET = "boot-0123456789abcdef-fedcba9876543210";
const ENVIRONMENT: EnvironmentSlice = {
  HOME: path.join(path.sep, "home", "owner"),
  XDG_RUNTIME_DIR: path.join(path.sep, "run", "user", "1000"),
};

/** Every stage the campaign runner needs to attribute one Pi request. */
const REQUIRED_STAGES = [
  "pi_request_preparation",
  "extension_dispatch",
  "loopback_wait",
  "daemon_preflight",
  "provider_network",
  "response_parse",
  "pi_event_delivery",
] as const;

function clientFor(endpoint: string): PersonalDaemonClient {
  const paths = resolvePersonalDaemonPaths(ENVIRONMENT);
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
  return new PersonalDaemonClient({ environment: ENVIRONMENT, files, requestTimeoutMs: 2_000 });
}

test("one Pi run yields joined per-stage timings and runner-readable Provider usage", async () => {
  const daemon = await startFakeDaemon({
    bootstrapSecret: BOOTSTRAP_SECRET,
    statusBody: "{}",
    completionBody: JSON.stringify({
      choices: [{ message: { content: "daemon text" }, finish_reason: "stop" }],
      usage: { prompt_tokens: 7, completion_tokens: 3, total_tokens: 10 },
    }),
    providerNetworkElapsedNanos: "4000000",
    daemonPreflightElapsedNanos: "1000000",
    echoCorrelationId: true,
  });
  try {
    const packageSurface = (await import("./index.js")) as Record<string, unknown>;
    const openObservationSession = packageSurface["openPiRouteObservationSession"];
    assert.equal(
      typeof openObservationSession,
      "function",
      "the package exposes no campaign-runner entry point for Pi route observations",
    );
    const session = (
      openObservationSession as (authorization: Record<string, string>) => {
        readonly observations: readonly Record<string, unknown>[];
        close(): void;
      }
    )({
      COGNITIVEOS_PI_ROUTE_OBSERVATION: "enabled",
      COGNITIVEOS_PI_ROUTE_OBSERVATION_CAMPAIGN: "PERSONAL-PERF-EVAL-003",
    });

    // The observation session is not yet part of the provider signature; the
    // cast keeps this probe compiling so the assertions below report which
    // requirement is unmet rather than collapsing into a type error.
    const createInstrumentedProvider = createDaemonProvider as unknown as (
      client: PersonalDaemonClient,
      options: { readonly session: unknown },
    ) => Promise<ProviderConfig>;

    try {
      const provider = await createInstrumentedProvider(clientFor(daemon.endpoint), { session });
      const stream = provider.streamSimple(provider.models[0]!, {
        messages: [{ role: "user", content: "hello" }],
      });
      for await (const _event of stream) {
        // Drain so Pi event delivery is genuinely part of the measured run.
      }
      await stream.result();

      assert.equal(session.observations.length, 1, "one run must publish exactly one observation");
      const observation = session.observations[0]!;
      const stages = observation["stages"] as readonly Record<string, unknown>[];
      assert.deepEqual(
        stages.map((stage) => stage["stage"]),
        [...REQUIRED_STAGES],
        "the observation must cover every stage of the Pi route in order",
      );
      for (const stage of stages) {
        const elapsedNanos = stage["elapsedNanos"];
        assert.ok(
          typeof elapsedNanos === "number" && Number.isSafeInteger(elapsedNanos) && elapsedNanos > 0,
          `stage ${String(stage["stage"])} must carry a positive monotonic duration`,
        );
      }
      assert.match(
        String(observation["correlationId"]),
        /^campaign-[0-9a-f]{32}$/,
        "the Pi and daemon observations must be joined by one opaque correlation id",
      );
      assert.equal(
        daemon.requests.at(-1)?.headers["x-cognitiveos-correlation-id"],
        observation["correlationId"],
        "the daemon must have observed the same correlation id the runner reads",
      );
      assert.deepEqual(observation["providerUsage"], {
        availability: "measured",
        promptTokens: 7,
        completionTokens: 3,
        totalTokens: 10,
      });
    } finally {
      session.close();
    }
  } finally {
    await daemon.close();
  }
});
