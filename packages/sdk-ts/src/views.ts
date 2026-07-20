/**
 * Shell-family payload bindings consumed by the task-channel client:
 * ShellActionProposal, ShellCommandPreview, ShellStatusView,
 * WatchSubscription, UserIntentRecord (specs/schemas/shell-*.json,
 * watch-subscription.schema.json, user-intent-record.schema.json).
 *
 * INTERIM, registered contract gap (20260720 lane-tsc handoff): these five
 * schemas are outside the IMP-08 codegen set, so contracts-ts has no
 * generated binding yet. The shapes below are hand-modeled strictly against
 * the schema files and drift-gated by `views.test.ts`, which validates
 * typed fixtures against the real schemas with ajv and re-derives
 * {@link SHELL_SCHEMA_DIGESTS} from `specs/schemas/` at test time. When
 * Lane-CTR extends the codegen set, this module switches to the generated
 * bindings and these interfaces disappear. Every `$ref`-shared definition
 * already generated (Budget, Digest, UriRef, StrongReference,
 * GovernedObjectHeader) is consumed from `@cognitiveos/contracts-ts`, not
 * re-modeled.
 */

import {
  canonicalize,
  digest,
  type commonDefs,
  type governedObjectHeader,
  type objectReference,
} from "@cognitiveos/contracts-ts";

/**
 * Canonical content digests of the shell-family schema files (canonical
 * bytes, domain `schema-bundle/0.1` — same recipe as the generated module
 * headers). Used as the envelope `schema_digest` pin for these payloads;
 * drift-gated by `views.test.ts`.
 */
export const SHELL_SCHEMA_DIGESTS = {
  "shell-action-proposal.schema.json":
    "sha256:f0057d1ad298fff34c452487567517f6ca352bacfb474dcefb111497cdbefaed",
  "shell-command-preview.schema.json":
    "sha256:7e538f23d999783931d057939bf2bb0a52f6f7a42d8ae60597b7a7506a5fee43",
  "shell-status-view.schema.json":
    "sha256:d4cf8856c906db93052aaebc1f3cfa59a49fa7a242b31bbea3a3d7205fcb12ab",
  "watch-subscription.schema.json":
    "sha256:1e2b2d6439b306cf558fdb22ad6c45bfcaee41b146940d97d377cd71fa321e3a",
  "user-intent-record.schema.json":
    "sha256:b3064740e47c0e67bd5646f20a87db33662858e913e90ff631fe182ef51383b5",
} as const;

export type RiskClass = "R0" | "R1" | "R2" | "R3";

export type EffectClass = "pure" | "local_ephemeral" | "governed_external" | "emergency_safety";

export type TargetResolutionStatus = "unique" | "not_found" | "ambiguous" | "stale";

export interface TargetResolution {
  readonly selector_text: string;
  readonly resolution_status: TargetResolutionStatus;
  readonly candidate_count: number;
  readonly resolved_targets: ReadonlyArray<objectReference.StrongReference>;
}

/** `shell-action-proposal.schema.json` (cognitiveos.shell-action-proposal/0.1). */
export interface ShellActionProposal {
  readonly schema_version: "cognitiveos.shell-action-proposal/0.1";
  readonly proposal_id: string;
  readonly channel: "task" | "management";
  readonly intent_ref: commonDefs.UriRef;
  readonly task_contract_ref: commonDefs.UriRef;
  readonly management_session_ref?: commonDefs.UriRef;
  readonly target_resolution: TargetResolution;
  readonly action: string;
  readonly parameters_digest: commonDefs.Digest;
  readonly expected_versions: Readonly<Record<string, number>>;
  readonly effect_class: EffectClass;
  readonly risk_class: RiskClass;
  readonly budget: commonDefs.Budget;
  readonly deadline: string;
  readonly egress?: ReadonlyArray<commonDefs.UriRef>;
  readonly idempotency_key: string;
  readonly actor_chain_digest: commonDefs.Digest;
  readonly activity_context_ref: commonDefs.UriRef;
  readonly preview_digest: commonDefs.Digest;
  readonly approval_refs?: ReadonlyArray<commonDefs.UriRef>;
  readonly proposal_digest: commonDefs.Digest;
  readonly confirmation_required: boolean;
  readonly confirmation_ref?: commonDefs.UriRef;
  readonly independent_approval_required: boolean;
}

/** `shell-command-preview.schema.json` (cognitiveos.shell-command-preview/0.1). */
export interface ShellCommandPreview {
  readonly schema_version: "cognitiveos.shell-command-preview/0.1";
  readonly proposal_ref: commonDefs.UriRef;
  readonly proposal_digest: commonDefs.Digest;
  readonly target_refs: ReadonlyArray<objectReference.StrongReference>;
  readonly changes: ReadonlyArray<string>;
  readonly assumptions: ReadonlyArray<string>;
  readonly ambiguities: ReadonlyArray<string>;
  readonly risk_class: RiskClass;
  readonly cost_bound: commonDefs.Budget;
  readonly authorization_requirements: ReadonlyArray<string>;
  readonly verification: string;
  readonly cancellation: string;
  readonly compensation: string;
  readonly expires_at: string;
  readonly preview_digest: commonDefs.Digest;
}

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
] as const;
export type ShellStatusValue = (typeof SHELL_STATUS_VALUES)[number];

export type SafeExitState = "available" | "pending" | "unavailable" | "unknown";

/** `shell-status-view.schema.json` (cognitiveos.shell-status-view/0.1). */
export interface ShellStatusView {
  readonly schema_version: "cognitiveos.shell-status-view/0.1";
  readonly view_id: string;
  readonly target_ref: objectReference.StrongReference;
  readonly target_version: number;
  readonly derived_from_refs: ReadonlyArray<objectReference.StrongReference>;
  readonly status: ShellStatusValue;
  readonly reason_code: string;
  readonly waiting_on: ReadonlyArray<commonDefs.UriRef>;
  readonly next_gate: string;
  readonly remaining_budget: commonDefs.Budget;
  readonly deadline: string;
  readonly safe_exit_state: SafeExitState;
  readonly available_actions: ReadonlyArray<string>;
  readonly as_of: string;
  readonly view_digest: commonDefs.Digest;
}

export type BackpressureMode =
  | "bounded_block"
  | "disconnect_resume"
  | "spill"
  | "coalesce_non_authoritative";

/** `watch-subscription.schema.json` (cognitiveos.watch-subscription/0.1). */
export interface WatchSubscription {
  readonly schema_version: "cognitiveos.watch-subscription/0.1";
  readonly subscription_id: string;
  readonly actor_chain_digest: commonDefs.Digest;
  readonly resource_scope_ref: commonDefs.UriRef;
  readonly purpose: string;
  readonly selector: string;
  readonly visible_fields: ReadonlyArray<string>;
  readonly event_types: ReadonlyArray<string>;
  readonly snapshot_version: number;
  readonly cursor: number;
  readonly high_watermark: number;
  readonly dedupe_window: number;
  readonly budget: commonDefs.Budget;
  readonly expires_at: string;
  readonly backpressure: BackpressureMode;
}

/** `user-intent-record.schema.json`; the header is the generated binding. */
export interface UserIntentRecord {
  readonly header: governedObjectHeader.GovernedObjectHeader;
  readonly actor_chain_digest: commonDefs.Digest;
  readonly conversation_or_scope_ref: commonDefs.UriRef;
  readonly input_refs: ReadonlyArray<commonDefs.UriRef>;
  readonly raw_expression: string;
  readonly recorded_at: string;
  readonly intent_digest: commonDefs.Digest;
}

/**
 * `shell.control` cancel payload. PROVISIONAL: AKP §13 names the
 * `shell.control` operation family and REQ-AKP-SHELL-003 fixes its result
 * distinctions, but no payload schema is registered yet (Lane-CTR gap,
 * 20260720 lane-tsc handoff). The shape is confined here and the envelope
 * pin below is explicitly provisional.
 */
export interface CancelControl {
  readonly control: "cancel";
  readonly target_ref: commonDefs.UriRef;
  readonly reason: string;
}

/**
 * PROVISIONAL envelope pin for `shell.control` payloads: the canonical
 * digest of an inline descriptor, NOT a registered schema digest. Derived
 * at module load (never hand-copied) so it cannot rot silently; replaced by
 * the registered schema digest when Lane-CTR lands the control schema. A
 * real kernel-server will reject this pin — that is intended fail-closed
 * behavior, not a defect.
 */
export const SHELL_CONTROL_PROVISIONAL_PIN: string = digest(
  canonicalize(
    JSON.stringify({
      $comment:
        "PROVISIONAL shell.control payload descriptor; no registered schema (Lane-CTR gap, 20260720-lane-tsc-handoff)",
      fields: ["control", "target_ref", "reason"],
    }),
  ),
  "akp-payload/0.2",
);

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
