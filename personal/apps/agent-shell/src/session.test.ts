/**
 * Shell session tests (docs/standards/task-loop-verification.md §6;
 * REQ-SHELL-DETACH-001, REQ-SHELL-ATTACH-001, REQ-SHELL-CONTROL-001,
 * REQ-SHELL-STATUS-001, REQ-AKP-SHELL-001/003, REQ-GW-002).
 *
 * Behavior-vector analogs exercised client-side (vectors themselves stay
 * not-run; runner execution is Lane-CFR):
 * - shell-detach-attach-004: detach cancels nothing, reattach resumes from
 *   the retained cursor;
 * - shell-cancel-semantics-005: cancel resolves through Effect closure —
 *   cancel_pending never displays as cancelled;
 * - remote-completed-not-acceptance: remote completed evidence never
 *   changes the displayed task state; only an authority projection does.
 */

import assert from "node:assert/strict";
import { test } from "node:test";

import {
  buildResultEnvelope,
  frameText,
  InMemoryTransport,
  samplePreview,
  sampleProposal,
  sampleStatusView,
  sampleWatchSubscription,
  TaskChannelClient,
  taskCredential,
  type ErrorEnvelope,
  type RequestEnvelope,
  type ResultEnvelope,
  type ShellStatusValue,
  type WatchStreamFrame,
} from "@cognitiveos/sdk-ts";

import { ShellFlowError, ShellSession } from "./session.js";

const CRED = taskCredential({
  credentialId: "cred-task-1",
  principalRef: "principal://tenant-a/user-1",
  secret: "task-secret",
});

const CONTEXT = {
  sender: "principal://tenant-a/user-1",
  audience: "kernel://tenant-a/node-1",
} as const;

const TARGET_ID = sampleStatusView().target_ref.id;

type RequestScript = (envelope: RequestEnvelope) => ResultEnvelope;
type StreamScript = (envelope: RequestEnvelope) => string[];

function harness(requestScript: RequestScript, streams: StreamScript[] = []) {
  const queue = [...streams];
  const transport = new InMemoryTransport("task", requestScript, (envelope) => {
    const next = queue.shift();
    if (!next) {
      throw new Error("no scripted stream left");
    }
    return next(envelope);
  });
  const client = new TaskChannelClient(CRED, CONTEXT, { transport });
  const session = new ShellSession({
    client,
    deadline: () => "2026-07-20T12:30:00Z",
  });
  return { transport, client, session };
}

const okPreview = (envelope: RequestEnvelope): ResultEnvelope =>
  buildResultEnvelope({
    inReplyTo: envelope.message_id,
    correlationId: envelope.correlation_id,
    status: "ok",
    result: samplePreview(),
  });

const accepted = (envelope: RequestEnvelope): ResultEnvelope =>
  buildResultEnvelope({
    inReplyTo: envelope.message_id,
    correlationId: envelope.correlation_id,
    status: "accepted",
    result: { task_ref: "task://tenant-a/task-1" },
  });

const statusFrame = (
  sequence: number,
  status: ShellStatusValue,
  extra: Partial<WatchStreamFrame> = {},
  targetVersion = sequence + 1,
): string =>
  frameText({
    stream_id: "st-1",
    sequence,
    kind: sequence === 0 ? "snapshot" : "delta",
    ...(sequence === 0 ? { snapshot_version: 1 } : {}),
    payload: sampleStatusView({
      status,
      target_version: targetVersion,
      as_of: "2026-07-20T11:30:00Z",
    }),
    ...extra,
  });

test("preview then submit: the submit envelope fixes proposal and preview digests", async () => {
  const proposal = sampleProposal();
  const { transport, session } = harness((envelope) =>
    envelope.operation === "shell.preview" ? okPreview(envelope) : accepted(envelope),
  );

  const preview = await session.preview(proposal);
  assert.equal(preview.proposal_digest, proposal.proposal_digest);
  assert.equal(session.phase, "previewed");

  const result = await session.submit();
  assert.equal(result.status, "accepted");
  assert.equal(session.phase, "submitted");

  const submit = transport.requests.find((request) => request.operation === "shell.submit");
  assert.ok(submit, "shell.submit was sent");
  assert.equal(submit.idempotency_key, proposal.idempotency_key);
  const payload = submit.payload as Record<string, unknown>;
  assert.equal(payload["proposal_digest"], proposal.proposal_digest);
  assert.equal(payload["preview_digest"], preview.preview_digest);
});

test("accepted is receipt-level only: no displayed state appears from a submit result", async () => {
  const proposal = sampleProposal();
  const { session } = harness((envelope) =>
    envelope.operation === "shell.preview" ? okPreview(envelope) : accepted(envelope),
  );
  await session.preview(proposal);
  await session.submit();
  assert.equal(
    session.displayStatus(TARGET_ID),
    undefined,
    "no authority projection has arrived, so nothing may be displayed as task state",
  );
});

test("submit without a preview fails closed client-side", async () => {
  const { transport, session } = harness(accepted);
  await assert.rejects(
    session.submit(),
    (error: unknown) => error instanceof ShellFlowError && error.reason === "submit-without-preview",
  );
  assert.equal(transport.requests.length, 0);
});

test("a preview for a different proposal digest is rejected (stale preview binding)", async () => {
  const proposal = sampleProposal();
  const { session } = harness((envelope) =>
    envelope.operation === "shell.preview"
      ? buildResultEnvelope({
          inReplyTo: envelope.message_id,
          correlationId: envelope.correlation_id,
          status: "ok",
          result: samplePreview({ proposal_digest: `sha256:${"11".repeat(32)}` }),
        })
      : accepted(envelope),
  );
  await assert.rejects(
    session.preview(proposal),
    (error: unknown) => error instanceof ShellFlowError && error.reason === "preview-mismatch",
  );
});

test("attach ingests authority status views; display state comes only from projections", async () => {
  const { session } = harness(accepted, [
    () => [statusFrame(0, "queued"), statusFrame(1, "runnable"), statusFrame(2, "waiting", { final: true })],
  ]);

  assert.equal(session.displayStatus(TARGET_ID), undefined);
  for await (const item of session.attach(sampleWatchSubscription())) {
    void item;
  }
  assert.equal(session.displayStatus(TARGET_ID)?.status, "waiting");
  assert.equal(session.phase, "attached");
});

test("remote completed evidence never changes displayed state; a status view does", async () => {
  const remoteEvidence = frameText({
    stream_id: "st-1",
    sequence: 2,
    kind: "delta",
    payload: {
      event_type: "gateway.remote-report",
      protocol: "a2a",
      remote_task_state: "completed",
      artifact_ref: "artifact://remote-org/result-17",
      artifact_signature_valid: true,
    },
  });
  const { session } = harness(accepted, [
    () => [
      statusFrame(0, "runnable"),
      statusFrame(1, "runnable"),
      remoteEvidence,
      statusFrame(3, "completed", { final: true }, 9),
    ],
  ]);

  const displayedAfterEvidence: Array<string | undefined> = [];
  for await (const item of session.attach(sampleWatchSubscription())) {
    if (item.cursor === 2) {
      // Right after the remote-completed evidence: display must be unchanged.
      displayedAfterEvidence.push(session.displayStatus(TARGET_ID)?.status);
    }
  }
  assert.deepEqual(displayedAfterEvidence, ["runnable"], "remote completed is evidence, not state");
  assert.equal(
    session.displayStatus(TARGET_ID)?.status,
    "completed",
    "only the authority projection moves the displayed state",
  );
});

test("detach sends nothing — no cancel, no control — and reattach resumes from the cursor", async () => {
  const { transport, session } = harness(accepted, [
    () => [statusFrame(0, "queued"), statusFrame(1, "runnable"), statusFrame(2, "runnable")],
    // Resume delivers everything after the acknowledged cursor 1: 2, then 3.
    () => [statusFrame(2, "runnable"), statusFrame(3, "waiting", { final: true })],
  ]);

  let seen = 0;
  for await (const item of session.attach(sampleWatchSubscription())) {
    seen += 1;
    if (item.cursor === 1) {
      session.detach();
    }
  }
  assert.equal(seen, 2, "iteration stops after detach");
  assert.equal(session.phase, "detached");
  // REQ-SHELL-DETACH-001: the task was NOT cancelled — zero control requests.
  assert.deepEqual(
    transport.requests.map((request) => request.operation),
    [],
    "detach must not produce any request (especially not shell.control)",
  );

  // Reattach: watch resumes from the retained cursor.
  for await (const item of session.attach(sampleWatchSubscription())) {
    void item;
  }
  assert.equal(transport.streamOpens[1]?.operation, "watch.resume");
  const resumePayload = transport.streamOpens[1]?.payload as Record<string, unknown>;
  assert.equal(resumePayload["cursor"], 1, "watch restored from the last acknowledged cursor");
  assert.equal(session.displayStatus(TARGET_ID)?.status, "waiting");
});

test("cancel is a request: cancel_pending resolves through Effect closure, not display", async () => {
  const { transport, session } = harness((envelope) => {
    if (envelope.operation === "shell.control") {
      return buildResultEnvelope({
        inReplyTo: envelope.message_id,
        correlationId: envelope.correlation_id,
        status: "cancel_pending",
        error: {
          code: "CANCEL_PENDING",
          category: "lifecycle",
          retryable: true,
          stage: "control",
        } as ErrorEnvelope,
      });
    }
    return accepted(envelope);
  }, [
    () => [statusFrame(0, "runnable", { final: true })],
  ]);

  for await (const item of session.attach(sampleWatchSubscription())) {
    void item;
  }

  const disposition = await session.cancel("task://tenant-a/task-1", {
    reason: "user requested stop",
    idempotencyKey: "idem-cancel-01",
  });
  assert.equal(disposition.disposition, "cancel_pending");
  assert.equal(disposition.errorCode, "CANCEL_PENDING");
  assert.ok(session.hasPendingCancel("task://tenant-a/task-1"));

  // Exactly one control request: CANCEL_PENDING is closure-pending state,
  // not a resend trigger.
  assert.equal(
    transport.requests.filter((request) => request.operation === "shell.control").length,
    1,
  );
  // The displayed status is untouched by the cancel request itself.
  assert.equal(session.displayStatus(TARGET_ID)?.status, "runnable");

  const control = transport.requests.find((request) => request.operation === "shell.control");
  const payload = control?.payload as Record<string, unknown>;
  assert.equal(payload["control"], "cancel");
  assert.equal(payload["target_ref"], "task://tenant-a/task-1");
});

test("cancel too late surfaces CANCEL_TOO_LATE without retry and without display changes", async () => {
  const { transport, session } = harness((envelope) =>
    buildResultEnvelope({
      inReplyTo: envelope.message_id,
      correlationId: envelope.correlation_id,
      status: "error",
      error: {
        code: "CANCEL_TOO_LATE",
        category: "lifecycle",
        retryable: false,
        stage: "control",
      } as ErrorEnvelope,
    }),
  );
  const disposition = await session.cancel("task://tenant-a/task-1", {
    reason: "late",
    idempotencyKey: "idem-cancel-02",
  });
  assert.equal(disposition.disposition, "too_late");
  assert.equal(transport.requests.length, 1);
});

test("displayed cancellation appears only when the authority projection says cancelled", async () => {
  const { session } = harness(accepted, [
    () => [
      statusFrame(0, "runnable"),
      statusFrame(1, "cancel_pending"),
      statusFrame(2, "cancelled", { final: true }),
    ],
  ]);
  const observed: string[] = [];
  for await (const item of session.attach(sampleWatchSubscription())) {
    void item;
    const status = session.displayStatus(TARGET_ID)?.status;
    if (status) {
      observed.push(status);
    }
  }
  assert.deepEqual(observed, ["runnable", "cancel_pending", "cancelled"]);
});

test("the session state machine tracks client-local phases only", async () => {
  const { session } = harness((envelope) =>
    envelope.operation === "shell.preview" ? okPreview(envelope) : accepted(envelope),
  );
  assert.equal(session.phase, "idle");
  await session.preview(sampleProposal());
  assert.equal(session.phase, "previewed");
  await session.submit();
  assert.equal(session.phase, "submitted");
  // Phase is UI state; it is never written into the projection store.
  assert.equal(session.displayStatus(TARGET_ID), undefined);
});
