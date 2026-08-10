# ADR-0047: Personal B09 Managed-Pi MVP Validation Policy

- Status: Accepted (owner session standing continuous-delivery direction
  2026-08-11: ADR-0040/ADR-0046-class fixed denominator for P5-T05/D04 B09 MVP)
- Date: 2026-08-11
- Decision owner: CognitiveOS Personal product owner
- Classification: product-semantic documentation decision
- Related: P5-T05, B09, ADR-0040, ADR-0046, P5-T01, P5-T02, P7-T08, GMVP-LINUX
- Supersedes: a live Provider/Pi statistical campaign for **P5-T05 MVP B09
  disposition only**

## Context

P5-T05/D01–D03 already deliver process-bound SidecarSession fencing,
upgrade/uninstall refusal while bound, pin/digest drift refusal before
activation, recover/orphan non-reattach, identity separation, and
install≠permission negatives. A live managed-Pi statistical campaign would add
Provider/credential ceremony without strengthening the MVP authority-path
signal already covered by those focused daemon tests.

Owner standing direction for Gate/campaign slices: prefer ADR-0040-class fixed
denominators (authority-path / fixture / non-claim report) unless formal
acceptance explicitly forbids the MVP path. P5-T05/D04 acceptance allows
preregistered campaign execution with owner disposition and does not require a
live Provider statistical suite.

## Decision

For the P5-T05 MVP disposition of B09, the fixed validation denominator is the
complete authority-path matrix below, executed at one exact reviewed revision
on `DEV-LINUX-NATIVE-01`, plus required Ubuntu/Windows CI and a non-claim
report:

| Required observation | Fixed evidence |
|---|---|
| `process_bound_on_activate` | `cognitive-runtime` test `activate_registers_fenced_process_attempt_and_health_is_bound` |
| `unbound_registered_health` | `cognitive-runtime` test `unbound_registered_health_reports_process_bound_false` |
| `pause_stop_clear_binding` | `cognitive-runtime` test `pause_and_stop_clear_process_binding_without_capability` |
| `stale_epoch_preserves_binding` | `cognitive-runtime` test `stale_epoch_pause_preserves_process_binding` |
| `process_bound_blocks_upgrade` | `cognitive-runtime` test `process_bound_blocks_upgrade_and_preserves_pointer` |
| `process_bound_blocks_uninstall` | `cognitive-runtime` test `process_bound_blocks_uninstall_and_rollback` |
| `pin_drift_rejects_activation` | `cognitive-runtime` test `root_upgrade_pin_drift_rejects_activation_before_session` |
| `stop_then_uninstall` | `cognitive-runtime` test `stop_clears_binding_then_uninstall_quarantines` |
| `install_neq_permission` | `cognitive-runtime` test `install_and_register_do_not_grant_permission_or_process_binding` |
| `identity_separation` | `cognitive-runtime` test `identities_stay_separated_across_activate_stop_recover` |
| `orphan_no_reattach` | `cognitive-runtime` test `orphan_cleared_attempt_is_not_reattached_by_stale_recover` |
| Non-claim suite harness | `tools` Node tests for `b09-managed-pi-gate` (incomplete observation and authority-shaped claim negatives) |

MVP pass conditions for B09 are all of the following:

1. every row in the matrix passes at one exact reviewed revision;
2. focused Rust checks run on qualified native Linux and pass Clippy with
   warnings denied for the exercised packages;
3. required Ubuntu and Windows CI pass for the review revision;
4. a non-claim B09 suite report is generated (`claim_scope: non-claim`;
   evaluator cannot set Gate state); and
5. the product owner records an affirmative or rejecting disposition for B09
   against that bounded evidence.

Live Provider/Pi statistical campaigns remain available for later promotion
work when additional signal is needed. They are not a P5-T05 MVP completion
mutex.

## Consequences

- D04 can close after the fixed matrix, native Linux/Clippy, required CI,
  non-claim report, owner B09 disposition, and normal PR/lease closure.
- The B09 evaluator remains non-authoritative: reports cannot mutate Gate
  state; the documented product decision owns Gate status.
- Daemon-only authority, no arbitrary PID attach, install≠permission, and
  identity separation stay mandatory observations inside the fixed matrix.
- This decision does not qualify non-Pi adapters and does not transfer to
  GMVP-LINUX, release, or Profile.

## Non-goals and non-claims

This ADR does not claim live managed-Pi spawn under production supervision,
non-Pi adapter qualification, B08, GMVP-LINUX, release, or Profile.
