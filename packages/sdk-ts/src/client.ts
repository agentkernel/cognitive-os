/**
 * Channel-bound AKP clients (task / management). One instance binds exactly
 * one channel, one credential, one transport, one projection store
 * (REQ-AKP-SHELL-001..003, REQ-AKP-MGMT-001..003, REQ-SHELL-CHANNEL-001).
 *
 * The request pipeline: build envelope (contracts digests) → transport →
 * parse/gate result → contract-driven retry (error-contract §3):
 *
 * - only registry-retryable codes are ever resent, with the SAME
 *   idempotency key and a fresh message id (specs/akp/README.md §6,
 *   REQ-AKP-IDEM-001);
 * - EFFECT_OUTCOME_UNKNOWN and `outcome_unknown` results are surfaced for
 *   reconciliation, never blindly re-dispatched;
 * - STATE_CONFLICT resends only through the caller's authoritative re-read
 *   hook (stale guards are never reused);
 * - exhausted retries return the last envelope verbatim — retry is never
 *   implicit success.
 *
 * Clients make no authorization, completion, or transition decisions; they
 * send requests and render authority projections.
 */

import {
  ChannelBindingViolation,
  ProjectionStore,
  type ChannelCredential,
  type ClientChannel,
} from "./channel.js";
import {
  buildRequestEnvelope,
  parseResultEnvelope,
  serializeEnvelope,
  type ErrorEnvelope,
  type Extension,
  type OperationKind,
  type RequestSpec,
  type ResultEnvelope,
} from "./envelope.js";
import { classifyError } from "./errors.js";
import type { AkpTransport } from "./transport.js";
import type { commonDefs } from "@cognitiveos/contracts-ts";

/**
 * Task-channel operation families, exactly the ones named by
 * specs/akp/README.md §13. The task client sends nothing outside this list;
 * the management client sends nothing inside it.
 */
export const TASK_CHANNEL_OPERATIONS = [
  "intent.record",
  "intent.interpret",
  "intent.admit",
  "intent.supersede",
  "shell.preview",
  "shell.submit",
  "shell.attach",
  "shell.control",
  "watch.open",
  "watch.ack",
  "watch.resume",
  "watch.close",
] as const;
export type TaskChannelOperation = (typeof TASK_CHANNEL_OPERATIONS)[number];

/** Static identity of the requesting side, fixed per client instance. */
export interface ClientContext {
  readonly sender: string;
  readonly audience: string;
  readonly authorizationRef?: string;
}

export interface ClientDeps {
  readonly transport: AkpTransport;
  /** Fresh delivery-level message ID per attempt. Default: random UUID. */
  readonly newMessageId?: () => string;
  /** Correlation ID when the caller does not supply one. */
  readonly newCorrelationId?: () => string;
  /** Deterministic inter-attempt delay hook. Default: none. */
  readonly backoff?: (attempt: number) => Promise<void>;
}

/** Per-call request description (channel identity is added by the client). */
export interface CallSpec<P = unknown> {
  readonly operation: string;
  readonly kind: OperationKind;
  readonly schemaDigest: string;
  readonly deadline: string;
  readonly payload?: P | undefined;
  readonly payloadRef?: string | undefined;
  readonly idempotencyKey?: string | undefined;
  readonly budget?: commonDefs.Budget | undefined;
  readonly correlationId?: string | undefined;
  readonly causationId?: string | undefined;
  readonly extensions?: ReadonlyArray<Extension> | undefined;
}

export interface CallOptions<P = unknown> {
  /** Total attempts (first try included). Default 3. */
  readonly maxAttempts?: number;
  /**
   * STATE_CONFLICT hook: re-read authoritative state and return a fresh
   * spec (new guards; the caller decides the new idempotency key since the
   * parameters changed). Without it, STATE_CONFLICT is surfaced unretried.
   */
  readonly refreshOnStateConflict?: (error: ErrorEnvelope) => CallSpec<P> | undefined;
}

/**
 * Raised when delivery of an effecting request failed with an unknown
 * outcome: the effect may exist remotely. The stable idempotency key is the
 * reconciliation handle (reconcile-or-quarantine, never a new-key resend).
 */
export class OutcomeUnknownError extends Error {
  readonly code = "EFFECT_OUTCOME_UNKNOWN";
  readonly idempotencyKey: string;

  constructor(operation: string, idempotencyKey: string, cause: unknown) {
    super(`delivery outcome unknown for effecting operation ${operation}`, { cause });
    this.name = "OutcomeUnknownError";
    this.idempotencyKey = idempotencyKey;
  }
}

const DEFAULT_MAX_ATTEMPTS = 3;

abstract class AkpChannelClient<C extends ClientChannel> {
  readonly channel: C;
  readonly projections: ProjectionStore;

  protected readonly credential: ChannelCredential<C>;
  protected readonly context: ClientContext;
  private readonly transport: AkpTransport;
  private readonly newMessageId: () => string;
  private readonly newCorrelationId: () => string;
  private readonly backoff: (attempt: number) => Promise<void>;

  protected constructor(channel: C, credential: ChannelCredential<C>, context: ClientContext, deps: ClientDeps) {
    ProjectionStore.assertChannel(credential, channel);
    if (deps.transport.channel !== channel) {
      throw new ChannelBindingViolation(
        `transport is bound to ${deps.transport.channel}, client requires ${channel}`,
      );
    }
    this.channel = channel;
    this.credential = credential;
    this.context = context;
    this.transport = deps.transport;
    this.newMessageId = deps.newMessageId ?? (() => crypto.randomUUID());
    this.newCorrelationId = deps.newCorrelationId ?? (() => crypto.randomUUID());
    this.backoff = deps.backoff ?? (() => Promise.resolve());
    this.projections = new ProjectionStore(credential);
  }

  /** Channel gate for one operation name; fails closed before sending. */
  protected abstract assertOperationAllowed(operation: string): void;

  private toRequestSpec<P>(spec: CallSpec<P>, correlationId: string): RequestSpec<P> {
    return {
      operation: spec.operation,
      kind: spec.kind,
      sender: this.context.sender,
      audience: this.context.audience,
      correlationId,
      causationId: spec.causationId,
      deadline: spec.deadline,
      schemaDigest: spec.schemaDigest,
      idempotencyKey: spec.idempotencyKey,
      authorizationRef: this.context.authorizationRef,
      budget: spec.budget,
      payload: spec.payload,
      payloadRef: spec.payloadRef,
      extensions: spec.extensions,
      messageId: this.newMessageId(),
    };
  }

  /**
   * Send one request with contract-driven bounded retry. Wire errors are
   * returned as envelopes, never thrown and never rewritten; only
   * client-side violations and unknown delivery outcomes throw.
   */
  async call<P, R = unknown>(spec: CallSpec<P>, options: CallOptions<P> = {}): Promise<ResultEnvelope<R>> {
    this.assertOperationAllowed(spec.operation);
    const maxAttempts = options.maxAttempts ?? DEFAULT_MAX_ATTEMPTS;
    const correlationId = spec.correlationId ?? this.newCorrelationId();
    let currentSpec = spec;

    for (let attempt = 1; ; attempt += 1) {
      const envelope = buildRequestEnvelope(this.toRequestSpec(currentSpec, correlationId));
      let body: string;
      try {
        body = (await this.transport.request(serializeEnvelope(envelope))).body;
      } catch (cause) {
        // Delivery failure. Resending with the same idempotency key is the
        // sanctioned retry (§6); a new key would be a blind re-dispatch.
        if (attempt < maxAttempts) {
          await this.backoff(attempt);
          continue;
        }
        if (currentSpec.kind === "effecting") {
          throw new OutcomeUnknownError(
            currentSpec.operation,
            currentSpec.idempotencyKey ?? "",
            cause,
          );
        }
        throw cause;
      }

      const result = parseResultEnvelope<R>(body);
      if (result.status !== "error" || result.error === undefined) {
        return result;
      }

      const classification = classifyError(result.error.code, result.error.retryable);
      if (classification.kind === "non-retryable" || classification.kind === "reconcile") {
        return result;
      }
      if (classification.precondition === "reread-authoritative-state") {
        const refreshed = options.refreshOnStateConflict?.(result.error);
        if (refreshed === undefined || attempt >= maxAttempts) {
          return result;
        }
        currentSpec = refreshed;
        await this.backoff(attempt);
        continue;
      }
      if (attempt >= maxAttempts) {
        return result;
      }
      await this.backoff(attempt);
    }
  }

  /**
   * Open a stream (watch family). The envelope goes through the same build
   * gates; frames come back as raw texts for the watch consumer to parse.
   */
  openStream<P>(spec: CallSpec<P>): AsyncIterable<string> {
    this.assertOperationAllowed(spec.operation);
    const envelope = buildRequestEnvelope(
      this.toRequestSpec(spec, spec.correlationId ?? this.newCorrelationId()),
    );
    return this.transport.openStream(serializeEnvelope(envelope));
  }
}

/** Task-channel client: the only client the Shell is allowed to hold. */
export class TaskChannelClient extends AkpChannelClient<"task"> {
  constructor(credential: ChannelCredential<"task">, context: ClientContext, deps: ClientDeps) {
    super("task", credential, context, deps);
  }

  protected assertOperationAllowed(operation: string): void {
    if (!(TASK_CHANNEL_OPERATIONS as ReadonlyArray<string>).includes(operation)) {
      throw new ChannelBindingViolation(
        `operation ${operation} is not a task-channel operation`,
      );
    }
  }
}

/**
 * Management-channel client skeleton. Management operation names are not
 * yet registered as machine contract; until they are, this client only
 * enforces the inverse gate (no task-channel operation family may cross).
 */
export class ManagementChannelClient extends AkpChannelClient<"management"> {
  constructor(credential: ChannelCredential<"management">, context: ClientContext, deps: ClientDeps) {
    super("management", credential, context, deps);
  }

  protected assertOperationAllowed(operation: string): void {
    if ((TASK_CHANNEL_OPERATIONS as ReadonlyArray<string>).includes(operation)) {
      throw new ChannelBindingViolation(
        `operation ${operation} belongs to the task channel`,
      );
    }
  }
}
