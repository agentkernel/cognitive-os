/**
 * Nested per-request stage observation for the Pi route (Personal P9-T05).
 *
 * `PERSONAL-PERF-EVAL-002` measured a +1828.5 ms median OS-arm overhead and
 * could not attribute it to a stage: Extension load, streaming mode, daemon
 * residual scaling and output size were each excluded by direct measurement,
 * leaving the remainder inferred. This module supplies the missing measurement
 * capability and nothing else.
 *
 * Boundaries this module holds, in order of importance:
 *
 *   - **Not an authority writer.** It records elapsed durations and usage
 *     counters. It performs no daemon request, opens no database, mints no
 *     capability and cannot advance Task, Effect or Intent state. Its optional
 *     file sink refuses any path inside a Personal state, runtime or config
 *     root precisely so it can never become a second writer of daemon-owned
 *     state.
 *   - **Denied by default.** Nothing is recorded or published unless the
 *     process environment carries an explicit campaign authorization. There is
 *     no prompt-, request- or response-driven way to switch it on.
 *   - **Content-free.** A stage carries a name, a domain and a nanosecond
 *     duration. There is no field for a prompt, a response, a header, a URL, a
 *     bearer or a credential, and the sink refuses secret-shaped strings.
 *   - **Never fabricated.** Absent or internally inconsistent Provider usage
 *     stays `not_available`. Daemon-side stages that are not reported, not
 *     joined by the correlation id, or not contained by the observed loopback
 *     wait are dropped with a reason rather than estimated.
 *   - **No cross-clock arithmetic.** Pi-side stages come from the Node
 *     monotonic clock and daemon-side stages from the daemon's own monotonic
 *     clock. Each stage keeps its domain, the two are never added or
 *     subtracted, and the only relation asserted across them is containment,
 *     which holds for elapsed durations regardless of clock offset.
 */

import { randomBytes } from "node:crypto";
import path from "node:path";

import type { ProviderUsage } from "./daemon-client.js";
import type { EnvironmentSlice } from "./daemon-discovery.js";
import { PERSONAL_PRODUCT_DIR_NAME } from "./daemon-discovery.js";

/** Schema of one published observation. */
export const PI_ROUTE_OBSERVATION_SCHEMA = "personal-pi-route-observation/1";

/** Environment variable that requests instrumentation. Absent means off. */
export const PI_ROUTE_OBSERVATION_ENABLE_VARIABLE = "COGNITIVEOS_PI_ROUTE_OBSERVATION";

/** Environment variable naming the authorized campaign. */
export const PI_ROUTE_OBSERVATION_CAMPAIGN_VARIABLE = "COGNITIVEOS_PI_ROUTE_OBSERVATION_CAMPAIGN";

/** Environment variable naming an optional NDJSON sink a runner reads. */
export const PI_ROUTE_OBSERVATION_SINK_VARIABLE = "COGNITIVEOS_PI_ROUTE_OBSERVATION_SINK";

/** The only accepted value of the enable variable. */
export const PI_ROUTE_OBSERVATION_ENABLED_VALUE = "enabled";

/** Retention ceiling. A campaign that exceeds it loses records, never memory. */
export const PI_ROUTE_OBSERVATION_MAX_RECORDS = 4_096;

/** Serialized ceiling for one record; a content-free record is far below it. */
export const PI_ROUTE_OBSERVATION_MAX_RECORD_BYTES = 2_048;

/**
 * The ordered stages of one Pi request.
 *
 * `daemon_preflight` and `provider_network` are measured by the daemon and are
 * nested inside `loopback_wait`; the remaining five are measured by Pi and are
 * sequential and disjoint.
 */
export const PI_ROUTE_STAGES = [
  "pi_request_preparation",
  "extension_dispatch",
  "loopback_wait",
  "daemon_preflight",
  "provider_network",
  "response_parse",
  "pi_event_delivery",
] as const;

export type PiRouteStage = (typeof PI_ROUTE_STAGES)[number];

/** Which monotonic clock measured a stage. Durations never cross domains. */
export type PiRouteStageDomain = "pi" | "daemon";

const PI_DOMAIN_STAGES = [
  "pi_request_preparation",
  "extension_dispatch",
  "loopback_wait",
  "response_parse",
  "pi_event_delivery",
] as const satisfies readonly PiRouteStage[];

const DAEMON_DOMAIN_STAGES = ["daemon_preflight", "provider_network"] as const satisfies
  readonly PiRouteStage[];

export type PiDomainStage = (typeof PI_DOMAIN_STAGES)[number];
export type DaemonDomainStage = (typeof DAEMON_DOMAIN_STAGES)[number];

export interface PiRouteStageTiming {
  readonly stage: PiRouteStage;
  readonly domain: PiRouteStageDomain;
  readonly elapsedNanos: number;
}

/** The product route currently makes exactly one bounded non-streaming request. */
export type PiRouteRequestMode = "non_streaming";

/** Terminal disposition of one started, authorized Pi route sample. */
export type PiRouteOutcome = "completed" | "cancelled" | "error";

/** Content-free terminal class; never copies an exception or response body. */
export type PiRouteFailureClass =
  | "none"
  | "cancelled"
  | "provider_unavailable"
  | "daemon_unavailable"
  | "protocol_error"
  | "extension_error";

/** Last Pi-domain stage reached, or a cancellation before preparation began. */
export type PiRouteTerminalStage = PiDomainStage | "before_request";

/** Why the daemon-side nested stages are absent from an observation. */
export type PiRouteDaemonStageUnavailableReason =
  | "not_reported"
  | "correlation_not_echoed"
  | "correlation_mismatch"
  | "incomplete_stage_group"
  | "invalid_duration"
  | "exceeds_loopback_wait";

export type PiRouteDaemonStageAvailability = "joined" | "not_available";

/**
 * One published observation of one Pi request. Every field is either a fixed
 * label, an opaque identifier, a nanosecond duration or a usage counter.
 */
export interface PiRouteObservation {
  readonly schema: typeof PI_ROUTE_OBSERVATION_SCHEMA;
  readonly campaignId: string;
  readonly correlationId: string;
  readonly requestMode: PiRouteRequestMode;
  readonly outcome: PiRouteOutcome;
  readonly failureClass: PiRouteFailureClass;
  readonly terminalStage: PiRouteTerminalStage;
  readonly stages: readonly PiRouteStageTiming[];
  readonly daemonStages: PiRouteDaemonStageAvailability;
  readonly daemonStagesUnavailableReason: PiRouteDaemonStageUnavailableReason | null;
  readonly providerUsage: ProviderUsage;
}

export type PiRouteObservationErrorCode =
  | "PI_ROUTE_OBSERVATION_CORRELATION_ID_INVALID"
  | "PI_ROUTE_OBSERVATION_CORRELATION_ID_DUPLICATE"
  | "PI_ROUTE_OBSERVATION_STAGE_UNKNOWN"
  | "PI_ROUTE_OBSERVATION_STAGE_DUPLICATE"
  | "PI_ROUTE_OBSERVATION_STAGE_OVERLAP"
  | "PI_ROUTE_OBSERVATION_STAGE_MISSING"
  | "PI_ROUTE_OBSERVATION_STAGE_OUT_OF_ORDER"
  | "PI_ROUTE_OBSERVATION_STAGE_NOT_OPEN"
  | "PI_ROUTE_OBSERVATION_STAGE_WRONG_DOMAIN"
  | "PI_ROUTE_OBSERVATION_DURATION_INVALID"
  | "PI_ROUTE_OBSERVATION_CLOCK_NOT_MONOTONIC"
  | "PI_ROUTE_OBSERVATION_NESTED_STAGES_EXCEED_PARENT"
  | "PI_ROUTE_OBSERVATION_DAEMON_AVAILABILITY_INCOHERENT"
  | "PI_ROUTE_OBSERVATION_PROVIDER_USAGE_INCONSISTENT"
  | "PI_ROUTE_OBSERVATION_PROVIDER_USAGE_UNVERIFIED"
  | "PI_ROUTE_OBSERVATION_OUTCOME_INCOHERENT"
  | "PI_ROUTE_OBSERVATION_SCHEMA_INVALID"
  | "PI_ROUTE_OBSERVATION_CAMPAIGN_ID_INVALID"
  | "PI_ROUTE_OBSERVATION_SECRET_SHAPED_VALUE"
  | "PI_ROUTE_OBSERVATION_RECORD_TOO_LARGE"
  | "PI_ROUTE_OBSERVATION_SINK_PATH_INVALID"
  | "PI_ROUTE_OBSERVATION_SINK_TARGETS_AUTHORITY_ROOT"
  | "PI_ROUTE_OBSERVATION_NOT_AUTHORIZED";

/**
 * A refusal from the observation surface. The message names the rule that was
 * broken and never echoes the offending value, which may be secret-shaped.
 */
export class PiRouteObservationError extends Error {
  readonly code: PiRouteObservationErrorCode;

  constructor(code: PiRouteObservationErrorCode, message: string) {
    super(message);
    this.name = "PiRouteObservationError";
    this.code = code;
  }
}

const CORRELATION_ID_PREFIX = "campaign-";
const CORRELATION_ID_PATTERN = /^campaign-[0-9a-f]{32}$/;
const CAMPAIGN_ID_PATTERN = /^[A-Z][A-Z0-9]*(?:-[A-Z0-9]+)*$/;
const CAMPAIGN_ID_MIN_LENGTH = 6;
const CAMPAIGN_ID_MAX_LENGTH = 64;

/**
 * Shapes that must never reach an observation. The correlation id is generated
 * here and the campaign id is operator-supplied, so the operator-supplied value
 * is the one that could carry a pasted credential.
 */
const SECRET_SHAPED_PATTERNS: readonly RegExp[] = [
  /SECRET|TOKEN|BEARER|PASSWD|PASSWORD|CREDENTIAL|APIKEY|PRIVATE/i,
  /(?:^|[^A-Z0-9])KEY(?:[^A-Z0-9]|$)/i,
  /^(?:sk|pk|rk|api)[-_]/i,
  /[0-9a-f]{32,}/i,
  /[A-Za-z0-9+/]{32,}={0,2}/,
];

/**
 * Runtime provenance for measured usage objects parsed from the authenticated
 * daemon response, bound to that request's correlation id. The marker is
 * deliberately not serialized: a runner can read counters, but cannot feed a
 * hand-built or replayed "measured" object back into the producer and have it
 * accepted as Provider evidence for another request.
 */
const VERIFIED_MEASURED_PROVIDER_USAGES = new WeakMap<object, string>();
const PUBLISHED_MEASURED_PROVIDER_USAGES = new WeakSet<object>();

/** Mint one opaque correlation id for a single request. */
export function createPiRouteCorrelationId(): string {
  return `${CORRELATION_ID_PREFIX}${randomBytes(16).toString("hex")}`;
}

/**
 * Accept only the fixed opaque shape. This is what keeps a bearer, a
 * `SecretRef`, a prompt fragment or a Provider key out of the join key, and it
 * matches the Rust `CampaignCorrelationId` the daemon-side envelope parses.
 */
export function parsePiRouteCorrelationId(value: unknown): string {
  if (typeof value !== "string" || !CORRELATION_ID_PATTERN.test(value)) {
    throw new PiRouteObservationError(
      "PI_ROUTE_OBSERVATION_CORRELATION_ID_INVALID",
      "a Pi route correlation id must be an opaque campaign- prefixed 32-character lowercase hexadecimal value",
    );
  }
  return value;
}

/** Reject operator-supplied values shaped like credentials before they are kept. */
export function assertNotSecretShaped(value: string, field: string): void {
  for (const pattern of SECRET_SHAPED_PATTERNS) {
    if (pattern.test(value)) {
      throw new PiRouteObservationError(
        "PI_ROUTE_OBSERVATION_SECRET_SHAPED_VALUE",
        `the ${field} is shaped like a credential and is refused before it can be recorded`,
      );
    }
  }
}

export function stageDomain(stage: PiRouteStage): PiRouteStageDomain {
  return (DAEMON_DOMAIN_STAGES as readonly PiRouteStage[]).includes(stage) ? "daemon" : "pi";
}

/**
 * Sequential recorder for the Pi-domain stages of one request.
 *
 * Overlap is structurally impossible rather than merely validated: a stage can
 * only be opened when no other stage is open, and only closed while it is the
 * open one. The recorder therefore cannot produce the double-counted stage sum
 * that would make an attribution look conclusive when it is not.
 */
export class PiRouteStageRecorder {
  private readonly correlationId: string;
  private readonly clock: () => number;
  private readonly piStages = new Map<PiDomainStage, number>();
  private openStage: { readonly stage: PiDomainStage; readonly startedAt: number } | undefined;

  constructor(options: { readonly correlationId?: string; readonly clock?: () => number } = {}) {
    this.correlationId =
      options.correlationId === undefined
        ? createPiRouteCorrelationId()
        : parsePiRouteCorrelationId(options.correlationId);
    this.clock = options.clock ?? (() => performance.now());
  }

  /** The opaque id the daemon must echo for the two sides to be joined. */
  readCorrelationId(): string {
    return this.correlationId;
  }

  /** Open one Pi-domain stage. */
  begin(stage: PiDomainStage): void {
    assertPiDomainStage(stage);
    if (this.piStages.has(stage)) {
      throw new PiRouteObservationError(
        "PI_ROUTE_OBSERVATION_STAGE_DUPLICATE",
        `stage ${stage} was already measured for this request`,
      );
    }
    if (this.openStage !== undefined) {
      throw new PiRouteObservationError(
        "PI_ROUTE_OBSERVATION_STAGE_OVERLAP",
        `stage ${stage} cannot start while ${this.openStage.stage} is still open`,
      );
    }
    this.openStage = { stage, startedAt: this.clock() };
  }

  /** Close the open Pi-domain stage and retain its elapsed duration. */
  complete(stage: PiDomainStage): void {
    const open = this.openStage;
    if (open === undefined || open.stage !== stage) {
      throw new PiRouteObservationError(
        "PI_ROUTE_OBSERVATION_STAGE_NOT_OPEN",
        `stage ${stage} cannot be completed because it is not the open stage`,
      );
    }
    const completedAt = this.clock();
    if (completedAt < open.startedAt) {
      throw new PiRouteObservationError(
        "PI_ROUTE_OBSERVATION_CLOCK_NOT_MONOTONIC",
        `stage ${stage} ended before it started; the supplied clock is not monotonic`,
      );
    }
    this.openStage = undefined;
    // A stage shorter than the clock's resolution is reported at the one
    // nanosecond floor. It is never reported as zero and never inflated.
    this.piStages.set(stage, Math.max(1, Math.round((completedAt - open.startedAt) * 1_000_000)));
  }

  /** Run `action` inside one stage, closing it even when `action` throws. */
  async measure<Result>(stage: PiDomainStage, action: () => Promise<Result>): Promise<Result> {
    this.begin(stage);
    try {
      return await action();
    } finally {
      this.complete(stage);
    }
  }

  hasStage(stage: PiDomainStage): boolean {
    return this.piStages.has(stage);
  }

  /** Elapsed nanoseconds of a completed Pi-domain stage. */
  readStage(stage: PiDomainStage): number | undefined {
    return this.piStages.get(stage);
  }

  /** The completed Pi-domain stages in canonical order. */
  readPiStages(): readonly PiRouteStageTiming[] {
    const timings: PiRouteStageTiming[] = [];
    for (const stage of PI_DOMAIN_STAGES) {
      const elapsedNanos = this.piStages.get(stage);
      if (elapsedNanos !== undefined) {
        timings.push({ stage, domain: "pi", elapsedNanos });
      }
    }
    return timings;
  }
}

/** Daemon-reported nested facts, exactly as read from the response headers. */
export interface DaemonReportedStages {
  /** Correlation id the daemon echoed, or `undefined` when it echoed none. */
  readonly echoedCorrelationId: string | undefined;
  readonly preflightElapsedNanos: number | undefined;
  readonly providerNetworkElapsedNanos: number | undefined;
}

export interface PiRouteObservationInput {
  readonly campaignId: string;
  readonly correlationId: string;
  /** Defaults to the product's only supported Provider request mode. */
  readonly requestMode?: PiRouteRequestMode;
  /** Defaults to `completed` for the success-path assembly API. */
  readonly outcome?: PiRouteOutcome;
  /** Defaults to the last measured Pi stage (event delivery for success). */
  readonly terminalStage?: PiRouteTerminalStage;
  /** Defaults from outcome: none/cancelled/extension_error. */
  readonly failureClass?: PiRouteFailureClass;
  readonly piStages: readonly PiRouteStageTiming[];
  readonly daemonReported: DaemonReportedStages;
  readonly providerUsage: ProviderUsage;
}

/**
 * Assemble one observation from what was actually measured.
 *
 * A completed request must carry every Pi-domain stage. A cancelled/error
 * request carries the exact measured prefix ending at `terminalStage`, so a
 * started sample is not lost merely because the route did not finish. The
 * daemon domain is optional and degrades to `not_available` with a reason
 * whenever it cannot be joined or contained by the observed loopback wait.
 */
export function assemblePiRouteObservation(input: PiRouteObservationInput): PiRouteObservation {
  const correlationId = parsePiRouteCorrelationId(input.correlationId);
  const campaignId = parsePiRouteCampaignId(input.campaignId);
  const requestMode = input.requestMode ?? "non_streaming";
  const outcome = input.outcome ?? "completed";
  const piStages = orderStages(input.piStages);
  const terminalStage =
    input.terminalStage ??
    (outcome === "completed"
      ? "pi_event_delivery"
      : (lastPiStage(piStages)?.stage ?? "before_request"));
  const failureClass =
    input.failureClass ??
    (outcome === "completed"
      ? "none"
      : outcome === "cancelled"
        ? "cancelled"
        : "extension_error");
  const loopbackWaitNanos = piStages.find((timing) => timing.stage === "loopback_wait")?.elapsedNanos;
  if (loopbackWaitNanos === undefined && outcome === "completed") {
    throw new PiRouteObservationError(
      "PI_ROUTE_OBSERVATION_STAGE_MISSING",
      "an observation cannot be assembled without the loopback_wait stage that contains the daemon stages",
    );
  }

  const daemonOutcome =
    loopbackWaitNanos === undefined
      ? ({ availability: "not_available", reason: "not_reported" } as const)
      : resolveDaemonStages(input.daemonReported, correlationId, loopbackWaitNanos);
  const stages =
    daemonOutcome.availability === "joined"
      ? insertDaemonStages(piStages, daemonOutcome.stages)
      : piStages;

  assertProviderUsageVerified(input.providerUsage, correlationId);
  const observation: PiRouteObservation = {
    schema: PI_ROUTE_OBSERVATION_SCHEMA,
    campaignId,
    correlationId,
    requestMode,
    outcome,
    failureClass,
    terminalStage,
    stages,
    daemonStages: daemonOutcome.availability,
    daemonStagesUnavailableReason:
      daemonOutcome.availability === "joined" ? null : daemonOutcome.reason,
    providerUsage: input.providerUsage,
  };
  validatePiRouteObservation(observation);
  return observation;
}

type DaemonStageOutcome =
  | { readonly availability: "joined"; readonly stages: readonly PiRouteStageTiming[] }
  | {
      readonly availability: "not_available";
      readonly reason: PiRouteDaemonStageUnavailableReason;
    };

function resolveDaemonStages(
  reported: DaemonReportedStages,
  correlationId: string,
  loopbackWaitNanos: number,
): DaemonStageOutcome {
  const { preflightElapsedNanos, providerNetworkElapsedNanos } = reported;
  if (preflightElapsedNanos === undefined && providerNetworkElapsedNanos === undefined) {
    return { availability: "not_available", reason: "not_reported" };
  }
  if (preflightElapsedNanos === undefined || providerNetworkElapsedNanos === undefined) {
    return { availability: "not_available", reason: "incomplete_stage_group" };
  }
  if (reported.echoedCorrelationId === undefined) {
    return { availability: "not_available", reason: "correlation_not_echoed" };
  }
  if (reported.echoedCorrelationId !== correlationId) {
    return { availability: "not_available", reason: "correlation_mismatch" };
  }
  if (
    !isPositiveDurationNanos(preflightElapsedNanos) ||
    !isPositiveDurationNanos(providerNetworkElapsedNanos)
  ) {
    return { availability: "not_available", reason: "invalid_duration" };
  }
  if (preflightElapsedNanos + providerNetworkElapsedNanos > loopbackWaitNanos) {
    // Nested work cannot outlast the wait that contains it. Rather than trim a
    // duration to fit, drop the domain: an adjusted stage would silently become
    // the strongest-looking evidence in a report.
    return { availability: "not_available", reason: "exceeds_loopback_wait" };
  }
  return {
    availability: "joined",
    stages: [
      { stage: "daemon_preflight", domain: "daemon", elapsedNanos: preflightElapsedNanos },
      { stage: "provider_network", domain: "daemon", elapsedNanos: providerNetworkElapsedNanos },
    ],
  };
}

function insertDaemonStages(
  piStages: readonly PiRouteStageTiming[],
  daemonStages: readonly PiRouteStageTiming[],
): readonly PiRouteStageTiming[] {
  return orderStages([...piStages, ...daemonStages]);
}

function orderStages(timings: readonly PiRouteStageTiming[]): readonly PiRouteStageTiming[] {
  const canonicalIndex = new Map<PiRouteStage, number>(
    PI_ROUTE_STAGES.map((stage, index) => [stage, index]),
  );
  return [...timings].sort(
    (left, right) => (canonicalIndex.get(left.stage) ?? 0) - (canonicalIndex.get(right.stage) ?? 0),
  );
}

function lastPiStage(
  timings: readonly PiRouteStageTiming[],
): PiRouteStageTiming & { readonly stage: PiDomainStage } | undefined {
  const piTimings = timings.filter(
    (timing): timing is PiRouteStageTiming & { readonly stage: PiDomainStage } =>
      timing.domain === "pi" &&
      (PI_DOMAIN_STAGES as readonly PiRouteStage[]).includes(timing.stage),
  );
  return piTimings.at(-1);
}

/**
 * Validate one observation against every rule a campaign runner depends on.
 *
 * This is the gate the sink applies before retaining a record, so a runner can
 * treat anything it reads as complete, ordered, disjoint, positively timed,
 * contained and internally consistent without re-deriving those properties.
 */
export function validatePiRouteObservation(observation: PiRouteObservation): void {
  assertExactKeys(
    observation,
    [
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
    ],
    "observation",
  );
  if (observation.schema !== PI_ROUTE_OBSERVATION_SCHEMA) {
    throw new PiRouteObservationError(
      "PI_ROUTE_OBSERVATION_SCHEMA_INVALID",
      `a Pi route observation must declare schema ${PI_ROUTE_OBSERVATION_SCHEMA}`,
    );
  }
  if (observation.requestMode !== "non_streaming") {
    throw new PiRouteObservationError(
      "PI_ROUTE_OBSERVATION_SCHEMA_INVALID",
      "the current Pi route observation schema accepts only the bounded non_streaming Provider mode",
    );
  }
  if (!["completed", "cancelled", "error"].includes(observation.outcome)) {
    throw new PiRouteObservationError(
      "PI_ROUTE_OBSERVATION_OUTCOME_INCOHERENT",
      "a Pi route observation must carry a registered terminal outcome",
    );
  }
  parsePiRouteCampaignId(observation.campaignId);
  parsePiRouteCorrelationId(observation.correlationId);

  const seenStages = new Set<PiRouteStage>();
  let previousCanonicalIndex = -1;
  for (const timing of observation.stages) {
    assertExactKeys(timing, ["domain", "elapsedNanos", "stage"], "stage timing");
    const canonicalIndex = (PI_ROUTE_STAGES as readonly string[]).indexOf(timing.stage);
    if (canonicalIndex === -1) {
      throw new PiRouteObservationError(
        "PI_ROUTE_OBSERVATION_STAGE_UNKNOWN",
        "a Pi route observation carries a stage that is not part of the registered schema",
      );
    }
    if (seenStages.has(timing.stage)) {
      throw new PiRouteObservationError(
        "PI_ROUTE_OBSERVATION_STAGE_DUPLICATE",
        `stage ${timing.stage} appears more than once in one observation`,
      );
    }
    if (canonicalIndex < previousCanonicalIndex) {
      throw new PiRouteObservationError(
        "PI_ROUTE_OBSERVATION_STAGE_OUT_OF_ORDER",
        `stage ${timing.stage} appears out of the registered route order`,
      );
    }
    if (timing.domain !== stageDomain(timing.stage)) {
      throw new PiRouteObservationError(
        "PI_ROUTE_OBSERVATION_STAGE_WRONG_DOMAIN",
        `stage ${timing.stage} is attributed to the wrong clock domain`,
      );
    }
    if (!isPositiveDurationNanos(timing.elapsedNanos)) {
      throw new PiRouteObservationError(
        "PI_ROUTE_OBSERVATION_DURATION_INVALID",
        `stage ${timing.stage} must carry a positive safe-integer nanosecond duration`,
      );
    }
    seenStages.add(timing.stage);
    previousCanonicalIndex = canonicalIndex;
  }

  validateTerminalOutcome(observation, seenStages);
  validateDaemonDomain(observation, seenStages);
  validateProviderUsage(observation.providerUsage);
}

function validateTerminalOutcome(
  observation: PiRouteObservation,
  seenStages: ReadonlySet<PiRouteStage>,
): void {
  const terminalStage = observation.terminalStage;
  const expectedPiStages =
    terminalStage === "before_request"
      ? []
      : (PI_DOMAIN_STAGES as readonly PiDomainStage[]).slice(
          0,
          (PI_DOMAIN_STAGES as readonly PiDomainStage[]).indexOf(terminalStage) + 1,
        );
  if (
    terminalStage !== "before_request" &&
    !(PI_DOMAIN_STAGES as readonly string[]).includes(terminalStage)
  ) {
    throw new PiRouteObservationError(
      "PI_ROUTE_OBSERVATION_OUTCOME_INCOHERENT",
      "the terminal stage must be before_request or one registered Pi-domain stage",
    );
  }
  const actualPiStages = PI_DOMAIN_STAGES.filter((stage) => seenStages.has(stage));
  if (
    expectedPiStages.length !== actualPiStages.length ||
    expectedPiStages.some((stage, index) => actualPiStages[index] !== stage)
  ) {
    throw new PiRouteObservationError(
      observation.outcome === "completed"
        ? "PI_ROUTE_OBSERVATION_STAGE_MISSING"
        : "PI_ROUTE_OBSERVATION_OUTCOME_INCOHERENT",
      "the measured Pi stages must be the exact route prefix ending at terminalStage",
    );
  }
  if (
    observation.outcome === "completed" &&
    terminalStage !== "pi_event_delivery"
  ) {
    throw new PiRouteObservationError(
      "PI_ROUTE_OBSERVATION_OUTCOME_INCOHERENT",
      "a completed Pi request must terminate after pi_event_delivery",
    );
  }
  if (
    (observation.outcome === "completed" && observation.failureClass !== "none") ||
    (observation.outcome === "cancelled" && observation.failureClass !== "cancelled") ||
    (observation.outcome === "error" &&
      ![
        "provider_unavailable",
        "daemon_unavailable",
        "protocol_error",
        "extension_error",
      ].includes(observation.failureClass))
  ) {
    throw new PiRouteObservationError(
      "PI_ROUTE_OBSERVATION_OUTCOME_INCOHERENT",
      "the content-free failure class must agree with the terminal outcome",
    );
  }
}

function validateDaemonDomain(
  observation: PiRouteObservation,
  seenStages: ReadonlySet<PiRouteStage>,
): void {
  const joinedDaemonStages = DAEMON_DOMAIN_STAGES.filter((stage) => seenStages.has(stage));
  if (observation.daemonStages !== "joined" && observation.daemonStages !== "not_available") {
    throw new PiRouteObservationError(
      "PI_ROUTE_OBSERVATION_DAEMON_AVAILABILITY_INCOHERENT",
      "daemon stage availability must be exactly joined or not_available",
    );
  }
  if (observation.daemonStages === "not_available") {
    if (
      joinedDaemonStages.length > 0 ||
      observation.daemonStagesUnavailableReason === null ||
      ![
        "not_reported",
        "correlation_not_echoed",
        "correlation_mismatch",
        "incomplete_stage_group",
        "invalid_duration",
        "exceeds_loopback_wait",
      ].includes(observation.daemonStagesUnavailableReason)
    ) {
      throw new PiRouteObservationError(
        "PI_ROUTE_OBSERVATION_DAEMON_AVAILABILITY_INCOHERENT",
        "an observation without joined daemon stages must carry a reason and no daemon stage",
      );
    }
    return;
  }
  if (
    joinedDaemonStages.length !== DAEMON_DOMAIN_STAGES.length ||
    observation.daemonStagesUnavailableReason !== null
  ) {
    throw new PiRouteObservationError(
      "PI_ROUTE_OBSERVATION_DAEMON_AVAILABILITY_INCOHERENT",
      "a joined observation must carry every daemon stage and no unavailability reason",
    );
  }
  const loopbackWaitNanos = elapsedNanosOf(observation, "loopback_wait");
  const nestedNanos =
    elapsedNanosOf(observation, "daemon_preflight") +
    elapsedNanosOf(observation, "provider_network");
  if (nestedNanos > loopbackWaitNanos) {
    throw new PiRouteObservationError(
      "PI_ROUTE_OBSERVATION_NESTED_STAGES_EXCEED_PARENT",
      "the daemon stages nested inside loopback_wait must not exceed the wait that contains them",
    );
  }
}

function validateProviderUsage(providerUsage: ProviderUsage): void {
  if (providerUsage.availability === "not_available") {
    assertExactKeys(providerUsage, ["availability"], "unavailable Provider usage");
    return;
  }
  if (providerUsage.availability !== "measured") {
    throw new PiRouteObservationError(
      "PI_ROUTE_OBSERVATION_PROVIDER_USAGE_INCONSISTENT",
      "Provider usage availability must be exactly not_available or measured",
    );
  }
  assertExactKeys(
    providerUsage,
    ["availability", "completionTokens", "promptTokens", "totalTokens"],
    "measured Provider usage",
  );
  const { promptTokens, completionTokens, totalTokens } = providerUsage;
  if (
    !isNonnegativeSafeInteger(promptTokens) ||
    !isNonnegativeSafeInteger(completionTokens) ||
    !isNonnegativeSafeInteger(totalTokens) ||
    promptTokens + completionTokens !== totalTokens
  ) {
    throw new PiRouteObservationError(
      "PI_ROUTE_OBSERVATION_PROVIDER_USAGE_INCONSISTENT",
      "measured Provider usage must carry complete, non-negative, internally consistent counters",
    );
  }
}

function assertExactKeys(
  value: object,
  expectedKeys: readonly string[],
  label: string,
): void {
  const actualKeys = Object.keys(value).sort();
  const expected = [...expectedKeys].sort();
  if (
    actualKeys.length !== expected.length ||
    actualKeys.some((key, index) => key !== expected[index])
  ) {
    throw new PiRouteObservationError(
      "PI_ROUTE_OBSERVATION_SCHEMA_INVALID",
      `the ${label} must carry exactly the registered content-free fields`,
    );
  }
}

/**
 * Parse Provider usage exactly once at the authenticated daemon-response
 * boundary. Missing, partial, fractional, negative or inconsistent values are
 * unavailable; the instrumentation never estimates or fills a zero.
 */
export function providerUsageFromDaemonResponse(
  value: unknown,
  correlationId: string,
): ProviderUsage {
  const parsedCorrelationId = parsePiRouteCorrelationId(correlationId);
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return { availability: "not_available" };
  }
  const usage = value as Record<string, unknown>;
  const promptTokens = usage["prompt_tokens"];
  const completionTokens = usage["completion_tokens"];
  const totalTokens = usage["total_tokens"];
  if (
    !isNonnegativeSafeInteger(promptTokens) ||
    !isNonnegativeSafeInteger(completionTokens) ||
    !isNonnegativeSafeInteger(totalTokens) ||
    promptTokens + completionTokens !== totalTokens
  ) {
    return { availability: "not_available" };
  }
  const measured: ProviderUsage = {
    availability: "measured",
    promptTokens,
    completionTokens,
    totalTokens,
  };
  VERIFIED_MEASURED_PROVIDER_USAGES.set(measured, parsedCorrelationId);
  return measured;
}

function assertProviderUsageVerified(
  providerUsage: ProviderUsage,
  correlationId: string,
): void {
  if (
    providerUsage.availability === "measured" &&
    (VERIFIED_MEASURED_PROVIDER_USAGES.get(providerUsage) !== correlationId ||
      PUBLISHED_MEASURED_PROVIDER_USAGES.has(providerUsage))
  ) {
    throw new PiRouteObservationError(
      "PI_ROUTE_OBSERVATION_PROVIDER_USAGE_UNVERIFIED",
      "measured Provider usage must originate at the authenticated daemon-response parser, remain request-bound, and be published once",
    );
  }
}

function markProviderUsagePublished(providerUsage: ProviderUsage): void {
  if (providerUsage.availability === "measured") {
    PUBLISHED_MEASURED_PROVIDER_USAGES.add(providerUsage);
  }
}

function freezePublishedObservation(observation: PiRouteObservation): PiRouteObservation {
  for (const stage of observation.stages) Object.freeze(stage);
  Object.freeze(observation.stages);
  Object.freeze(observation.providerUsage);
  return Object.freeze(observation);
}

function elapsedNanosOf(observation: PiRouteObservation, stage: PiRouteStage): number {
  return observation.stages.find((timing) => timing.stage === stage)?.elapsedNanos ?? 0;
}

/** Read a daemon-reported nanosecond duration header value. */
export function parseDaemonReportedNanos(value: string | null | undefined): number | undefined {
  if (typeof value !== "string" || !/^\d{1,19}$/.test(value)) return undefined;
  const elapsedNanos = Number(value);
  return isPositiveDurationNanos(elapsedNanos) ? elapsedNanos : undefined;
}

function isPositiveDurationNanos(value: number): boolean {
  return Number.isSafeInteger(value) && value > 0;
}

function isNonnegativeSafeInteger(value: unknown): value is number {
  return typeof value === "number" && Number.isSafeInteger(value) && value >= 0;
}

function assertPiDomainStage(stage: PiDomainStage): void {
  if (!(PI_DOMAIN_STAGES as readonly string[]).includes(stage)) {
    throw new PiRouteObservationError(
      "PI_ROUTE_OBSERVATION_STAGE_WRONG_DOMAIN",
      `stage ${String(stage)} is not measured by the Pi-side monotonic clock`,
    );
  }
}

/** Accept only a registered-shaped, non-secret campaign identifier. */
export function parsePiRouteCampaignId(value: unknown): string {
  if (
    typeof value !== "string" ||
    value.length < CAMPAIGN_ID_MIN_LENGTH ||
    value.length > CAMPAIGN_ID_MAX_LENGTH ||
    !CAMPAIGN_ID_PATTERN.test(value)
  ) {
    throw new PiRouteObservationError(
      "PI_ROUTE_OBSERVATION_CAMPAIGN_ID_INVALID",
      `an authorized campaign id must be ${CAMPAIGN_ID_MIN_LENGTH}-${CAMPAIGN_ID_MAX_LENGTH} upper-case alphanumeric characters in dash-separated groups`,
    );
  }
  assertNotSecretShaped(value, "campaign id");
  return value;
}

export type PiRouteObservationDenialReason =
  | "not_requested"
  | "not_enabled"
  | "campaign_missing"
  | "campaign_invalid"
  | "campaign_secret_shaped";

export type PiRouteObservationAuthorization =
  | { readonly authorized: false; readonly reason: PiRouteObservationDenialReason }
  | { readonly authorized: true; readonly campaignId: string; readonly sinkPath: string | undefined };

/**
 * Resolve the campaign authorization from the process environment.
 *
 * Every failure path returns a denial. There is deliberately no default
 * campaign, no implicit enable and no way for a prompt, a request body or a
 * daemon response to reach this function.
 */
export function resolvePiRouteObservationAuthorization(
  environment: EnvironmentSlice,
): PiRouteObservationAuthorization {
  const requested = environment[PI_ROUTE_OBSERVATION_ENABLE_VARIABLE];
  if (requested === undefined || requested.trim().length === 0) {
    return { authorized: false, reason: "not_requested" };
  }
  if (requested !== PI_ROUTE_OBSERVATION_ENABLED_VALUE) {
    return { authorized: false, reason: "not_enabled" };
  }
  const campaignId = environment[PI_ROUTE_OBSERVATION_CAMPAIGN_VARIABLE];
  if (campaignId === undefined || campaignId.trim().length === 0) {
    return { authorized: false, reason: "campaign_missing" };
  }
  try {
    parsePiRouteCampaignId(campaignId);
  } catch (error) {
    return {
      authorized: false,
      reason:
        error instanceof PiRouteObservationError &&
        error.code === "PI_ROUTE_OBSERVATION_SECRET_SHAPED_VALUE"
          ? "campaign_secret_shaped"
          : "campaign_invalid",
    };
  }
  const sinkPath = environment[PI_ROUTE_OBSERVATION_SINK_VARIABLE];
  return {
    authorized: true,
    campaignId,
    sinkPath: sinkPath === undefined || sinkPath.trim().length === 0 ? undefined : sinkPath,
  };
}

/**
 * Sink port for an NDJSON campaign log.
 *
 * The package ships no implementation on purpose. This package writes nothing
 * to the filesystem — a guard its own source-level safety suite enforces — so
 * any durable campaign log is owned by the harness that embeds the Extension,
 * and the only thing this module contributes is the refusal in
 * `validateSinkPath`.
 */
export interface ObservationSinkWriter {
  appendLine(sinkPath: string, line: string): void;
}

/**
 * The campaign runner's handle on one authorized measurement session.
 *
 * The session retains a bounded number of observations in memory and may
 * append them to an operator-designated NDJSON file. It holds no daemon
 * connection and writes nothing else.
 */
export interface PiRouteObservationSession {
  readonly campaignId: string;
  /** Validated sink path the embedding harness may write, if one was named. */
  readonly sinkPath: string | undefined;
  /** Observations retained so far, oldest first. */
  readonly observations: readonly PiRouteObservation[];
  /** Observations refused or dropped after the retention ceiling was reached. */
  readonly droppedObservations: number;
  /** Mint the recorder for one request. */
  openRequest(): PiRouteStageRecorder;
  publish(observation: PiRouteObservation): void;
  close(): void;
}

export interface PiRouteObservationSessionOptions {
  /**
   * Sink supplied by the embedding campaign harness. Absent — the default —
   * means the session retains observations in memory and writes nothing.
   */
  readonly sinkWriter?: ObservationSinkWriter;
  /** Roots the sink must stay out of; defaults to the Personal XDG roots. */
  readonly authorityRoots?: readonly string[];
}

/**
 * Open a measurement session, or return `undefined` when the environment does
 * not carry an explicit campaign authorization. `undefined` is the default: an
 * ordinary Pi session measures nothing and publishes nothing.
 */
export function openPiRouteObservationSession(
  environment: EnvironmentSlice,
  options: PiRouteObservationSessionOptions = {},
): PiRouteObservationSession | undefined {
  const authorization = resolvePiRouteObservationAuthorization(environment);
  if (!authorization.authorized) return undefined;
  const sinkPath =
    authorization.sinkPath === undefined
      ? undefined
      : validateSinkPath(
          authorization.sinkPath,
          options.authorityRoots ?? personalAuthorityRoots(environment),
        );
  return new BoundedObservationSession(authorization.campaignId, sinkPath, options.sinkWriter);
}

class BoundedObservationSession implements PiRouteObservationSession {
  readonly campaignId: string;
  readonly sinkPath: string | undefined;
  private readonly retained: PiRouteObservation[] = [];
  private readonly seenCorrelationIds = new Set<string>();
  private readonly sinkWriter: ObservationSinkWriter | undefined;
  private dropped = 0;
  private closed = false;

  constructor(
    campaignId: string,
    sinkPath: string | undefined,
    sinkWriter: ObservationSinkWriter | undefined,
  ) {
    this.campaignId = campaignId;
    this.sinkPath = sinkPath;
    this.sinkWriter = sinkWriter;
  }

  get observations(): readonly PiRouteObservation[] {
    return Object.freeze([...this.retained]);
  }

  get droppedObservations(): number {
    return this.dropped;
  }

  openRequest(): PiRouteStageRecorder {
    return new PiRouteStageRecorder();
  }

  publish(observation: PiRouteObservation): void {
    if (this.closed) {
      throw new PiRouteObservationError(
        "PI_ROUTE_OBSERVATION_NOT_AUTHORIZED",
        "the campaign observation session is closed and accepts no further records",
      );
    }
    if (observation.campaignId !== this.campaignId) {
      throw new PiRouteObservationError(
        "PI_ROUTE_OBSERVATION_CAMPAIGN_ID_INVALID",
        "an observation must carry the campaign id this session was authorized for",
      );
    }
    validatePiRouteObservation(observation);
    assertProviderUsageVerified(observation.providerUsage, observation.correlationId);
    if (this.seenCorrelationIds.has(observation.correlationId)) {
      throw new PiRouteObservationError(
        "PI_ROUTE_OBSERVATION_CORRELATION_ID_DUPLICATE",
        "one correlation id identifies exactly one request and cannot be published twice",
      );
    }
    const retainedObservation = freezePublishedObservation(observation);
    const line = JSON.stringify(retainedObservation);
    if (Buffer.byteLength(line, "utf8") > PI_ROUTE_OBSERVATION_MAX_RECORD_BYTES) {
      throw new PiRouteObservationError(
        "PI_ROUTE_OBSERVATION_RECORD_TOO_LARGE",
        "a content-free observation cannot exceed the record ceiling; the record was refused",
      );
    }
    markProviderUsagePublished(retainedObservation.providerUsage);
    this.seenCorrelationIds.add(observation.correlationId);
    if (this.retained.length >= PI_ROUTE_OBSERVATION_MAX_RECORDS) {
      this.dropped += 1;
      return;
    }
    this.retained.push(retainedObservation);
    if (this.sinkPath !== undefined && this.sinkWriter !== undefined) {
      this.sinkWriter.appendLine(this.sinkPath, line);
    }
  }

  close(): void {
    this.closed = true;
  }
}

/**
 * Refuse any sink that is not an absolute NDJSON path outside every Personal
 * authority root. This is the concrete guarantee that instrumentation cannot
 * become a second writer of daemon-owned state.
 */
export function validateSinkPath(sinkPath: string, authorityRoots: readonly string[]): string {
  if (!path.isAbsolute(sinkPath) || !sinkPath.endsWith(".ndjson")) {
    throw new PiRouteObservationError(
      "PI_ROUTE_OBSERVATION_SINK_PATH_INVALID",
      "an observation sink must be an absolute path to an .ndjson file",
    );
  }
  const resolvedSink = path.resolve(sinkPath);
  for (const root of authorityRoots) {
    const resolvedRoot = path.resolve(root);
    const relativePath = path.relative(resolvedRoot, resolvedSink);
    if (relativePath === "" || (!relativePath.startsWith("..") && !path.isAbsolute(relativePath))) {
      throw new PiRouteObservationError(
        "PI_ROUTE_OBSERVATION_SINK_TARGETS_AUTHORITY_ROOT",
        "an observation sink must not be written inside a Personal state, runtime or config root",
      );
    }
  }
  return resolvedSink;
}

/**
 * The Personal roots a sink must stay out of. Resolution is defensive: a
 * partially configured environment yields fewer roots, never an exception that
 * would let a sink through unchecked.
 */
export function personalAuthorityRoots(environment: EnvironmentSlice): readonly string[] {
  const roots: string[] = [];
  const home = environment["HOME"] ?? environment["USERPROFILE"];
  const runtimeRoot = environment["XDG_RUNTIME_DIR"];
  const stateRoot =
    environment["XDG_STATE_HOME"] ??
    (home === undefined ? undefined : path.join(home, ".local", "state"));
  const configRoot =
    environment["XDG_CONFIG_HOME"] ?? (home === undefined ? undefined : path.join(home, ".config"));
  for (const root of [runtimeRoot, stateRoot, configRoot]) {
    if (root !== undefined && root.trim().length > 0) {
      roots.push(path.join(root, PERSONAL_PRODUCT_DIR_NAME));
    }
  }
  return roots;
}
