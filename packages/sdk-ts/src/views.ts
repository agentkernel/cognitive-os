/**
 * Shell-family payload bindings consumed by the task-channel client — all
 * GENERATED (`@cognitiveos/contracts-ts` codegen 0.2.0; D-013/D-014 closure,
 * 20260720 lane-ctr gaps handoff). The former hand-modeled interfaces,
 * `SHELL_SCHEMA_DIGESTS` table, `CancelControl` shape and
 * `SHELL_CONTROL_PROVISIONAL_PIN` are deleted; every schema module now
 * exports `SCHEMA_ID`/`SCHEMA_DIGEST`, and `SCHEMA_DIGESTS` aggregates the
 * envelope `schema_digest` pins.
 *
 * This module only adds: ergonomic type aliases, the runtime status list
 * (compile-time-checked against the generated union), and the
 * `isShellStatusView` ingestion guard.
 */

import {
  SCHEMA_DIGESTS,
  shellActionProposal,
  shellCommandPreview,
  shellControlRequest,
  shellStatusView,
  userIntentRecord,
  watchSubscription,
} from "@cognitiveos/contracts-ts";

// Generated namespaces re-exported for clients of this SDK (agent-shell
// imports through here; each namespace carries SCHEMA_ID/SCHEMA_DIGEST).
export {
  SCHEMA_DIGESTS,
  shellActionProposal,
  shellCommandPreview,
  shellControlRequest,
  shellStatusView,
  userIntentRecord,
  watchSubscription,
};

/** Root-type aliases over the generated bindings. */
export type ShellActionProposal = shellActionProposal.ShellActionProposal;
export type TargetResolution = shellActionProposal.ShellActionProposalTargetResolution;
export type ShellCommandPreview = shellCommandPreview.ShellCommandPreview;
export type ShellControlRequest = shellControlRequest.ShellControlRequest;
export type ShellStatusView = shellStatusView.ShellStatusView;
export type ShellStatusValue = shellStatusView.ShellStatusViewStatus;
export type UserIntentRecord = userIntentRecord.UserIntentRecord;
export type WatchSubscription = watchSubscription.WatchSubscription;

/**
 * Runtime companion of the generated `ShellStatusViewStatus` union (types
 * erase; the ingestion guard needs a value list). `satisfies` pins every
 * entry to the union and the `_exhaustive` witness fails to compile if the
 * generated union ever gains a member missing here.
 */
export const SHELL_STATUS_VALUES = [
  "queued",
  "runnable",
  "waiting",
  "blocked",
  "cancel_pending",
  "outcome_unknown",
  "quarantined",
  "candidate_complete",
  "completed",
  "failed",
  "cancelled",
  "escalated",
] as const satisfies readonly ShellStatusValue[];

type StatusListCoversUnion = [ShellStatusValue] extends [(typeof SHELL_STATUS_VALUES)[number]]
  ? true
  : never;
const _exhaustive: StatusListCoversUnion = true;
void _exhaustive;

/**
 * Shape guard for projection ingestion: is this delta payload a
 * ShellStatusView? (`schema_version` const plus the fields the Shell reads.)
 * Schema validation proper stays with the authority and the test suite.
 */
export function isShellStatusView(payload: unknown): payload is ShellStatusView {
  if (payload === null || typeof payload !== "object" || Array.isArray(payload)) {
    return false;
  }
  const view = payload as Record<string, unknown>;
  return (
    view["schema_version"] === "cognitiveos.shell-status-view/0.1" &&
    typeof view["view_id"] === "string" &&
    typeof view["target_version"] === "number" &&
    typeof view["status"] === "string" &&
    (SHELL_STATUS_VALUES as ReadonlyArray<string>).includes(view["status"] as string) &&
    view["target_ref"] !== null &&
    typeof view["target_ref"] === "object" &&
    typeof (view["target_ref"] as Record<string, unknown>)["id"] === "string"
  );
}
