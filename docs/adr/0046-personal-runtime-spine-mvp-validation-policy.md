# ADR-0046: Personal Runtime Spine MVP Validation Policy

- Status: Accepted (owner session direction 2026-08-11: B03/ADR-0040-class fixed
  denominator for P2-T08/D04; live Provider/Pi statistical campaigns deferred)
- Date: 2026-08-11
- Decision owner: CognitiveOS Personal product owner
- Classification: product-semantic documentation decision
- Related: P2-T08, B02, B04, B05, B12, ADR-0018, ADR-0026, ADR-0040, P7-T08,
  GMVP-LINUX
- Supersedes: the ≥30-run live Provider/Pi statistical campaign and separate
  multi-Gate live E2E ceremony for **P2-T08 MVP B02/B04/B05/B12 disposition
  only**

## Context

P2-T08/D01–D03 already deliver a non-claim Runtime Spine observation harness,
ADR-0018 local-native exception expiry, and named authority-path negatives for
shell/daemon close, OUTCOME_UNKNOWN original-key reconcile, no-blind-retry, and
false-completion floor. The research card still describes a large live E2E
statistical campaign that needs Provider credentials and ≥30 effective runs.
That ceremony adds process cost without strengthening the MVP authority-path
signal already covered by focused daemon tests.

Owner direction for D04: prefer an ADR-0040-class fixed denominator —
preregister, re-run D01–D03 authority paths and the suite on
`DEV-LINUX-NATIVE-01` at an exact revision, emit a non-claim report, then ask
only for Gate disposition — instead of defaulting to live Provider/Pi
statistics.

## Decision

For the P2-T08 MVP disposition of B02/B04/B05/B12, the fixed validation
denominator is the complete observation-to-evidence matrix below, executed at
one exact reviewed revision:

| Required observation | Fixed evidence |
|---|---|
| `six_family_projection_isolated` | `kernel-server` process test `resource_projection_is_private_versioned_and_management_channel_bound` |
| `task_management_channel_isolated` | `admin-cli` process test `cognitive_uses_isolated_task_and_resource_daemon_channels` |
| `default_path_confirmation_recorded` | `cognitive-runtime` test `preview_digest_mismatch_is_refused_before_any_kernel_mutation` plus suite report `default_path_confirmation_count <= 1` |
| `tier2_purge_requires_explicit_confirmation` | `cognitive-management` test `runtime_spine_tier2_purge_requires_explicit_confirmation` |
| `shell_close_preserved_authority` | `cognitive-runtime` test `runtime_spine_shell_close_preserves_authority_without_cancelling_task` |
| `daemon_close_recoverable` | `kernel-server` test `runtime_spine_daemon_close_recoverable_without_duplicate_dispatch` |
| `outcome_unknown_reconciled_by_original_key` | `kernel-server` test `runtime_spine_outcome_unknown_reconciles_original_key_and_rejects_blind_retry` |
| `no_blind_retry_without_key_change` | same OUTCOME_UNKNOWN original-key test |
| `no_false_completion` | `kernel-server` test `runtime_spine_false_completion_self_check_rejects_passed_report_as_task_completion` |
| `adr0018_local_native_exception_absent_or_replaced` | `pi-agent-adapter` test `expired_local_native_provider_exception_is_rejected` |
| Non-claim suite harness | `tools` Node tests for `runtime-spine-gate` (incomplete observation, authority-shaped claim, wrong gate set, confirmation-ceiling negatives) |

MVP pass conditions for each named Gate are all of the following:

1. every row in the matrix passes at one exact reviewed revision;
2. focused Rust checks run on qualified native Linux and pass Clippy with
   warnings denied for the exercised packages;
3. required Ubuntu and Windows CI pass for the review revision;
4. the disposable native validation checkout is cleaned up and evidence is
   redacted;
5. a non-claim Runtime Spine suite report is generated (`claim_scope:
   non-claim`; evaluator cannot set Gate state); and
6. the product owner records an affirmative or rejecting disposition for each
   of B02, B04, B05, and B12 against that bounded evidence.

Live Provider/Pi statistical campaigns, ≥30-run denominators, and separately
assigned independent verifier ceremonies remain available for later promotion
work when additional signal is needed. They are not a P2-T08 MVP completion
mutex.

## Consequences

- D04 can close after the fixed matrix, native Linux/Clippy, required CI,
  non-claim report, owner Gate disposition, and normal PR/lease closure.
- The Runtime Spine evaluator remains non-authoritative: reports cannot mutate
  Gate state; the documented product decision owns each Gate status.
- ADR-0018 expiry and ADR-0026 confirmation accounting stay mandatory
  observations inside the fixed matrix.
- This decision does not reduce daemon-only authority, Intent/Effect
  persist-before-dispatch, independent verification, Secret isolation, or
  false-completion floors.

## Non-goals and non-claims

This decision does not pass GMVP-LINUX, create a release, establish Profile
conformance, change public schemas, claim B01/B03/B08/B09, or transfer any one
Gate disposition to another Gate.
