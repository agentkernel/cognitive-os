/**
 * Schema-valid sample builders for the shell-family payloads. Test support
 * shared by sdk-ts and agent-shell suites (and reusable by the M5 harness);
 * `views.test.ts` validates each builder's output against the real schemas
 * under `specs/schemas/` with ajv, so a drifting fixture is a red build,
 * not a silently wrong test double.
 */

import type { governedObjectHeader, objectReference } from "@cognitiveos/contracts-ts";

import { payloadDigest } from "./envelope.js";
import type {
  ShellActionProposal,
  ShellCommandPreview,
  ShellStatusView,
  UserIntentRecord,
  WatchSubscription,
} from "./views.js";

/** Deterministic sample digest derived through the contracts layer. */
function sampleDigest(marker: string): string {
  return payloadDigest({ marker });
}

const UUID_TASK = "01890a5d-ac96-774b-bcce-b302099a8057";
const UUID_SCOPE = "01890a5d-ac96-774b-bcce-b302099a8058";
const UUID_OWNER = "01890a5d-ac96-774b-bcce-b302099a8059";
const UUID_AUTHORITY = "01890a5d-ac96-774b-bcce-b302099a805a";
const UUID_TENANT = "01890a5d-ac96-774b-bcce-b302099a805b";
const UUID_INTENT = "01890a5d-ac96-774b-bcce-b302099a805c";

export function sampleStrongRef(
  overrides: Partial<objectReference.StrongReference> = {},
): objectReference.StrongReference {
  return {
    kind: "strong",
    id: UUID_TASK,
    object_version: 3,
    content_digest: sampleDigest("target"),
    ...overrides,
  };
}

export function sampleStatusView(overrides: Partial<ShellStatusView> = {}): ShellStatusView {
  return {
    schema_version: "cognitiveos.shell-status-view/0.1",
    view_id: "ssv_task-1-view-0001",
    target_ref: sampleStrongRef(),
    target_version: 3,
    derived_from_refs: [sampleStrongRef({ id: UUID_SCOPE })],
    status: "runnable",
    reason_code: "LOOP_STEP_ACTIVE",
    waiting_on: [],
    next_gate: "verification",
    remaining_budget: { tool_calls: 5, wall_time_ms: 60000 },
    deadline: "2026-07-20T13:00:00Z",
    safe_exit_state: "available",
    available_actions: ["watch", "cancel"],
    as_of: "2026-07-20T11:00:00Z",
    view_digest: sampleDigest("view"),
    ...overrides,
  };
}

export function sampleWatchSubscription(
  overrides: Partial<WatchSubscription> = {},
): WatchSubscription {
  return {
    schema_version: "cognitiveos.watch-subscription/0.1",
    subscription_id: "wsub_task-1-sub-0001",
    actor_chain_digest: sampleDigest("actor-chain"),
    resource_scope_ref: "scope://tenant-a/tasks",
    purpose: "shell task status watch",
    selector: "task://tenant-a/task-1",
    visible_fields: ["status", "target_version", "as_of"],
    event_types: ["shell.status-view"],
    snapshot_version: 0,
    cursor: 0,
    high_watermark: 0,
    dedupe_window: 64,
    budget: { wall_time_ms: 600000 },
    expires_at: "2026-07-21T00:00:00Z",
    backpressure: "disconnect_resume",
    ...overrides,
  };
}

export function sampleProposal(overrides: Partial<ShellActionProposal> = {}): ShellActionProposal {
  return {
    schema_version: "cognitiveos.shell-action-proposal/0.1",
    proposal_id: "sap_task-1-prop-0001",
    channel: "task",
    intent_ref: "intent://tenant-a/intent-1",
    task_contract_ref: "task-contract://tenant-a/tc-1",
    target_resolution: {
      selector_text: "the nightly export job",
      resolution_status: "unique",
      candidate_count: 1,
      resolved_targets: [sampleStrongRef()],
    },
    action: "job.restart",
    parameters_digest: sampleDigest("parameters"),
    expected_versions: { "task://tenant-a/task-1": 3 },
    effect_class: "governed_external",
    risk_class: "R1",
    budget: { tool_calls: 1 },
    deadline: "2026-07-20T13:00:00Z",
    idempotency_key: "idem-sap-0001",
    actor_chain_digest: sampleDigest("actor-chain"),
    activity_context_ref: "activity://tenant-a/act-1",
    preview_digest: sampleDigest("preview"),
    proposal_digest: sampleDigest("proposal"),
    confirmation_required: false,
    independent_approval_required: false,
    ...overrides,
  };
}

export function samplePreview(overrides: Partial<ShellCommandPreview> = {}): ShellCommandPreview {
  return {
    schema_version: "cognitiveos.shell-command-preview/0.1",
    proposal_ref: "proposal://tenant-a/sap-task-1-prop-0001",
    proposal_digest: sampleDigest("proposal"),
    target_refs: [sampleStrongRef()],
    changes: ["restart job job-7 on worker pool A"],
    assumptions: ["job-7 is idempotent to restart"],
    ambiguities: [],
    risk_class: "R1",
    cost_bound: { tool_calls: 1 },
    authorization_requirements: ["capability://tenant-a/job-restart"],
    verification: "job reports healthy within 120s",
    cancellation: "cancellable until dispatch",
    compensation: "job can be restarted again",
    expires_at: "2026-07-20T12:30:00Z",
    preview_digest: sampleDigest("preview"),
    ...overrides,
  };
}

export function sampleHeader(): governedObjectHeader.GovernedObjectHeader {
  return {
    id: UUID_INTENT,
    type: "UserIntentRecord",
    schema_version: "cognitiveos.user-intent-record/0.1",
    object_version: 1,
    scope_domain: "tenant",
    tenant_id: UUID_TENANT,
    resource_scope_ref: sampleStrongRef({ id: UUID_SCOPE }),
    owner_ref: sampleStrongRef({ id: UUID_OWNER }),
    authority_ref: sampleStrongRef({ id: UUID_AUTHORITY }),
    policy_refs: [],
    purpose_constraints: ["task-execution"],
    sensitivity: "internal",
    compartments: [],
    retention: { policy: "standard-90d", expires_at: null, legal_hold: false },
    provenance: { created_by: "principal://tenant-a/user-1", source_refs: [] },
    lineage: { parents: [], transform: "recorded" },
    content_digest: sampleDigest("intent-content"),
    created_at: "2026-07-20T10:59:00Z",
    valid_time: { from: "2026-07-20T10:59:00Z", until: null },
  };
}

export function sampleIntentRecord(overrides: Partial<UserIntentRecord> = {}): UserIntentRecord {
  return {
    header: sampleHeader(),
    actor_chain_digest: sampleDigest("actor-chain"),
    conversation_or_scope_ref: "conversation://tenant-a/conv-1",
    input_refs: [],
    raw_expression: "restart the nightly export job",
    recorded_at: "2026-07-20T10:59:00Z",
    intent_digest: sampleDigest("intent-content"),
    ...overrides,
  };
}
