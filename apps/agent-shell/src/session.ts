/**
 * Shell interaction session: proposal → preview → submit → attach/watch,
 * with detach and cancel (docs/standards/task-loop-verification.md §6;
 * REQ-SHELL-DETACH-001, REQ-SHELL-ATTACH-001, REQ-SHELL-CONTROL-001,
 * REQ-SHELL-STATUS-001, REQ-SHELL-PREVIEW-001, REQ-AKP-SHELL-001/003).
 *
 * The Shell is a non-authority client:
 * - `phase` is client-local UI state; it never becomes task state;
 * - every displayed task status comes from the projection store, which is
 *   fed exclusively by authority status views arriving over watch;
 * - detach stops consuming and retains the cursor; it sends nothing, and
 *   in particular it never cancels (vector shell-detach-attach-004);
 * - cancel is a request whose closure the authority decides through Effect
 *   state; `cancel_pending` is not `cancelled` (vector
 *   shell-cancel-semantics-005), and the session never rewrites a display
 *   state on its own — remote completed reports, receipts, or local
 *   "looks done" signals change nothing (REQ-GW-002).
 *
 * The interactive CLI front end is deferred to M5; this command layer and
 * state machine are its complete non-visual core.
 */

import {
  consumeWatch,
  isShellStatusView,
  shellActionProposal,
  shellControlRequest,
  type ResultEnvelope,
  type ShellActionProposal,
  type ShellCommandPreview,
  type ShellControlRequest,
  type ShellStatusView,
  type TaskChannelClient,
  type WatchItem,
  type WatchSubscription,
} from "@cognitiveos/sdk-ts";

/** Client-local interaction phase. Never authority state. */
export type ShellPhase = "idle" | "previewed" | "submitted" | "attached" | "detached";

export type ShellFlowErrorReason =
  | "submit-without-preview"
  | "preview-mismatch"
  | "command-failed";

/** Client-side interaction-flow rejection (not an authority response). */
export class ShellFlowError extends Error {
  readonly reason: ShellFlowErrorReason;
  readonly errorCode: string | undefined;

  constructor(reason: ShellFlowErrorReason, detail: string, errorCode?: string) {
    super(`${reason}: ${detail}`);
    this.name = "ShellFlowError";
    this.reason = reason;
    this.errorCode = errorCode;
  }
}

export interface ShellSessionDeps {
  /** The Shell holds a task-channel client only (management is admin-cli/Console). */
  readonly client: TaskChannelClient;
  /** Canonical-timestamp deadline provider for outgoing envelopes. */
  readonly deadline: () => string;
}

/**
 * Control-result disposition (REQ-AKP-SHELL-003 distinctions surfaced
 * verbatim; the session maps envelope status/code, it never decides).
 */
export interface CancelDisposition {
  readonly disposition:
    | "accepted"
    | "cancel_pending"
    | "cancelled"
    | "too_late"
    | "outcome_unknown"
    | "error";
  readonly errorCode?: string;
}

export interface CancelRequest {
  readonly reason: string;
  readonly idempotencyKey: string;
}

export class ShellSession {
  #phase: ShellPhase = "idle";
  #proposal: ShellActionProposal | undefined;
  #preview: ShellCommandPreview | undefined;
  #detachRequested = false;
  #watchPosition: { snapshotVersion: number; lastAckCursor: number } | undefined;
  readonly #pendingCancels = new Set<string>();
  readonly #deps: ShellSessionDeps;

  constructor(deps: ShellSessionDeps) {
    this.#deps = deps;
  }

  get phase(): ShellPhase {
    return this.#phase;
  }

  /** Retained watch position surviving a detach (REQ-SHELL-ATTACH-001). */
  get watchPosition(): { snapshotVersion: number; lastAckCursor: number } | undefined {
    return this.#watchPosition;
  }

  /**
   * Ask the authority to preview a proposal (`shell.preview`, read). The
   * returned preview must bind to this proposal's digest; a mismatch is a
   * stale/foreign preview and fails closed.
   */
  async preview(proposal: ShellActionProposal): Promise<ShellCommandPreview> {
    const result = await this.#deps.client.call<ShellActionProposal, ShellCommandPreview>({
      operation: "shell.preview",
      kind: "read",
      schemaDigest: shellActionProposal.SCHEMA_DIGEST,
      deadline: this.#deps.deadline(),
      payload: proposal,
    });
    if (result.status !== "ok" || result.result === undefined) {
      throw new ShellFlowError(
        "command-failed",
        `shell.preview returned ${result.status}`,
        result.error?.code,
      );
    }
    const preview = result.result;
    if (preview.proposal_digest !== proposal.proposal_digest) {
      throw new ShellFlowError(
        "preview-mismatch",
        `preview binds ${preview.proposal_digest}, proposal is ${proposal.proposal_digest}`,
      );
    }
    this.#proposal = proposal;
    this.#preview = preview;
    this.#phase = "previewed";
    return preview;
  }

  /**
   * Submit the previewed action (`shell.submit`, effecting). The envelope
   * payload is the proposal itself, which fixes proposal digest, preview
   * digest, resolved targets, expected versions and the idempotency key
   * (REQ-AKP-SHELL-001). An `accepted` result is receipt-level only: it
   * changes no displayed state.
   */
  async submit(): Promise<ResultEnvelope> {
    const proposal = this.#proposal;
    const preview = this.#preview;
    if (proposal === undefined || preview === undefined) {
      throw new ShellFlowError("submit-without-preview", "submit requires a bound preview first");
    }
    const result = await this.#deps.client.call({
      operation: "shell.submit",
      kind: "effecting",
      schemaDigest: shellActionProposal.SCHEMA_DIGEST,
      deadline: this.#deps.deadline(),
      payload: proposal,
      idempotencyKey: proposal.idempotency_key,
    });
    if (result.status === "ok" || result.status === "accepted") {
      this.#phase = "submitted";
    }
    return result;
  }

  /**
   * Attach to the task's authority projection stream. Yields watch items;
   * every ShellStatusView payload is ingested into the projection store
   * (keyed by the authority target identity and versioned by
   * `target_version`). Reattaching after a detach resumes from the retained
   * cursor.
   */
  async *attach(subscription: WatchSubscription): AsyncGenerator<WatchItem, void, undefined> {
    this.#detachRequested = false;
    this.#phase = "attached";
    const items = consumeWatch(this.#deps.client, {
      subscription,
      deadline: this.#deps.deadline(),
      ...(this.#watchPosition !== undefined ? { resumeFrom: this.#watchPosition } : {}),
    });
    for await (const item of items) {
      this.#watchPosition = {
        snapshotVersion: item.snapshotVersion,
        lastAckCursor: item.cursor,
      };
      if (isShellStatusView(item.payload)) {
        this.#deps.client.projections.ingest(
          item.payload.target_ref.id,
          item.payload.target_version,
          item.payload,
        );
      }
      yield item;
      if (this.#detachRequested) {
        return;
      }
    }
  }

  /**
   * Detach: stop consuming and keep the cursor. Sends nothing — detaching
   * or exiting the Shell never cancels a task (REQ-SHELL-DETACH-001).
   */
  detach(): void {
    this.#detachRequested = true;
    this.#phase = "detached";
  }

  /**
   * Request cancellation (`shell.control`, effecting). The result is a
   * disposition, not a state change: the task shows as cancelled only when
   * the authority projection says so (REQ-SHELL-CONTROL-001). No automatic
   * retry: closure progress is observed via watch, not by resending.
   */
  async cancel(taskRef: string, request: CancelRequest): Promise<CancelDisposition> {
    const payload: ShellControlRequest = {
      schema_version: "cognitiveos.shell-control-request/0.1",
      control: "cancel",
      target_ref: taskRef,
      reason: request.reason,
    };
    const result = await this.#deps.client.call({
      operation: "shell.control",
      kind: "effecting",
      schemaDigest: shellControlRequest.SCHEMA_DIGEST,
      deadline: this.#deps.deadline(),
      payload,
      idempotencyKey: request.idempotencyKey,
      // A control request is sent once; its closure is watched, not retried.
    }, { maxAttempts: 1 });

    const disposition = mapControlDisposition(result);
    if (disposition.disposition === "cancel_pending" || disposition.disposition === "accepted") {
      this.#pendingCancels.add(taskRef);
    }
    return disposition;
  }

  hasPendingCancel(taskRef: string): boolean {
    return this.#pendingCancels.has(taskRef);
  }

  /**
   * The only read path for displayed task state: the projection store
   * (REQ-SHELL-STATUS-001). Returns the latest authority status view for
   * the target, or undefined when none has arrived.
   */
  displayStatus(targetId: string): ShellStatusView | undefined {
    return this.#deps.client.projections.get<ShellStatusView>(targetId)?.view;
  }
}

function mapControlDisposition(result: ResultEnvelope): CancelDisposition {
  const code = result.error?.code;
  switch (result.status) {
    case "accepted":
      return { disposition: "accepted", ...(code !== undefined ? { errorCode: code } : {}) };
    case "cancel_pending":
      return { disposition: "cancel_pending", ...(code !== undefined ? { errorCode: code } : {}) };
    case "outcome_unknown":
      return { disposition: "outcome_unknown", ...(code !== undefined ? { errorCode: code } : {}) };
    case "error":
      if (code === "CANCEL_TOO_LATE") {
        return { disposition: "too_late", errorCode: code };
      }
      if (code === "CANCEL_PENDING") {
        return { disposition: "cancel_pending", errorCode: code };
      }
      return { disposition: "error", ...(code !== undefined ? { errorCode: code } : {}) };
    default:
      // ok / verified / committed / partial on a control result: surface
      // verbatim as non-error; the projection remains the display source.
      return { disposition: "accepted" };
  }
}
