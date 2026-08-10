<!--
Task: P2-T08
Slice: D04
Gates: B02, B04, B05, B12
Campaign: runtime-spine-gates/1
Classification: campaign-preregistration
Status: registered under ADR-0046 MVP fixed denominator
-->

# P2-T08 Runtime Spine Gate campaign preregistration

## Claim boundary

This campaign uses ADR-0046's fixed authority-path denominator for MVP
disposition of B02/B04/B05/B12. The Runtime Spine suite report remains
`claim_scope: non-claim` and cannot set Gate state. Product-owner disposition
owns each Gate status after evidence review.

Live Provider/Pi statistical campaigns and ≥30-run denominators are deferred
and are not an MVP mutex.

## Fixed campaign identity

| Field | Value | Status |
|---|---|---|
| Task / slice | `P2-T08/D04` | registered |
| Target Gates | `B02`, `B04`, `B05`, `B12` | registered; disposition pending owner review |
| Campaign ID | `runtime-spine-gates/1` | matches `tools/src/runtime-spine-gate.mjs` |
| Policy | ADR-0046 | accepted for MVP disposition |
| Branch / Draft PR | `personal/P2-T08-runtime-spine-gates` / #182 | active |
| Lease | `lease/personal/P2-T08/runtime-spine-gates` | active |
| Operator | Cursor agent | standing operator authorization |
| Independent verifier | user/owner | disposition review after evidence |
| Registration authority | user/owner session direction (fixed-denominator path) | granted |
| Execution authorization | granted for `DEV-LINUX-NATIVE-01` exact-revision rerun | granted |

## Qualified environment and reset

| Field | Required value |
|---|---|
| Environment | `DEV-LINUX-NATIVE-01` (`personal-linux-native-01`, `wuz@192.168.1.2`) |
| Source checkout | clean disposable Git worktree at the pushed immutable implementation revision |
| Network / secrets | no Provider credential, no Secret Store mutation, no live Pi Provider traffic required for MVP matrix |
| Cleanup | stop processes, remove disposable checkout, redact evidence before retention |

## Fixed denominator matrix

| Observation | Command / test |
|---|---|
| `six_family_projection_isolated` | `cargo test -p kernel-server --test p2_t02_resource_projection resource_projection_is_private_versioned_and_management_channel_bound` |
| `task_management_channel_isolated` | `cargo test -p admin-cli --test p2_t02_cli_parity cognitive_uses_isolated_task_and_resource_daemon_channels` |
| `default_path_confirmation_recorded` | `cargo test -p cognitive-runtime --test p2_t01_task_application_service preview_digest_mismatch_is_refused_before_any_kernel_mutation` |
| `tier2_purge_requires_explicit_confirmation` | `cargo test -p cognitive-management --test m5_session_approval runtime_spine_tier2_purge_requires_explicit_confirmation` |
| `shell_close_preserved_authority` | `cargo test -p cognitive-runtime --lib runtime_spine_shell_close_preserves_authority_without_cancelling_task` |
| `daemon_close_recoverable` | `cargo test -p kernel-server --lib runtime_spine_daemon_close_recoverable_without_duplicate_dispatch` |
| `outcome_unknown_reconciled_by_original_key` / `no_blind_retry_without_key_change` | `cargo test -p kernel-server --lib runtime_spine_outcome_unknown_reconciles_original_key_and_rejects_blind_retry` |
| `no_false_completion` | `cargo test -p kernel-server --lib runtime_spine_false_completion_self_check_rejects_passed_report_as_task_completion` |
| `adr0018_local_native_exception_absent_or_replaced` | `cargo test -p pi-agent-adapter --bin pi-agent-adapter expired_local_native_provider_exception_is_rejected` |
| Non-claim harness | `pnpm --filter @cognitiveos/repo-tools test` (includes `runtime-spine-gate` negatives) |

Supporting checks: focused Clippy `-D warnings` for exercised packages; local
`pnpm run check:consistency` and `git diff --check` where eligible; required
Ubuntu/Windows CI for the review revision.

## Accounting

- Complete denominator: every matrix row is `pass`, `fail`, or `not-run`.
- No averaging across Gates: owner dispositions B02/B04/B05/B12 separately.
- Evidence is redacted; no secrets, Provider traffic, or raw credential material.
- Suite report must keep `claim_scope: non-claim` and must not contain
  authority-shaped claim keys.

## Owner disposition ask (after evidence)

After execution evidence is recorded, owner only needs to affirm or reject each
Gate against the fixed matrix:

1. B02 — affirm / reject
2. B04 — affirm / reject
3. B05 — affirm / reject
4. B12 — affirm / reject

This preregistration document does not itself set Gate state.
