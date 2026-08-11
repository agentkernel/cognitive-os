# ADR-0050: Personal B10 Dynamic Tool Ecosystem MVP Validation Policy

- Status: Accepted (owner session standing continuous-delivery direction
  2026-08-11: ADR-0040/ADR-0046/ADR-0047/ADR-0048-class fixed denominator for
  P5-T04/D04 B10 MVP)
- Date: 2026-08-11
- Decision owner: CognitiveOS Personal product owner
- Classification: product-semantic documentation decision
- Related: P5-T04, B10, P5-T03, ADR-0040, ADR-0046, ADR-0047, ADR-0048,
  GMVP-LINUX
- Supersedes: a live Provider/MCP marketplace statistical campaign for
  **P5-T04 MVP B10 disposition only**

## Context

P5-T03 already delivers transport-only MCP adapter qualification with
drift/timeout/direct-bypass negatives. P5-T04 extends that post-1.0 train with
dynamic discovery/package/exposure/enable/disable/quarantine/reconcile,
TaskContract-scoped exposure, composite child Intent/Effect retention, and
pure-read cache telemetry. A live marketplace statistical campaign would add
Provider/credential ceremony without strengthening the MVP authority-path
signal already covered by those focused daemon tests.

Owner standing direction for Gate/campaign slices: prefer ADR-0040-class fixed
denominators (authority-path / fixture / non-claim report) unless formal
acceptance explicitly forbids the MVP path. P5-T04 acceptance requires an
independent B10 campaign and does not require a live Provider statistical
suite for the B10 MVP disposition itself. B10 remains non-blocking for Linux
1.0 / `GMVP-LINUX`.

## Decision

For the P5-T04 MVP disposition of B10, the fixed validation denominator is the
complete authority-path matrix below, executed at one exact reviewed revision
on `DEV-LINUX-NATIVE-01`, plus required Ubuntu/Windows CI and a non-claim
report:

| Required observation | Fixed evidence |
|---|---|
| `dynamic_package_identity_bound` | `cognitive-runtime` test `binds_package_and_discovers_disabled_candidate` |
| `discovery_disabled_no_auto_enable` | `cognitive-runtime` test `rejects_identity_schema_auto_enable_and_authority_writer` |
| `task_contract_scoped_exposure` | `cognitive-runtime` test `enable_disable_quarantine_and_task_contract_exposure` |
| `enable_requires_requalification` | same focused enable negative |
| `disable_removes_exposure` | same focused disable/exposure path |
| `quarantine_blocks_enable` | same focused quarantine negative |
| `package_manifest_drift_fail_closed` | `cognitive-runtime` test `reject_manifest_drift_composite_cache_reconcile_and_bypass` |
| `reconcile_unknown_outcome_original_key` | same focused reconcile negative |
| `composite_retains_child_intent_effect` | same focused composite path |
| `pure_read_cache_only` | same focused cache mutation negative |
| `sandbox_bypass_rejected` | same focused direct-bypass negative |
| Non-claim suite harness | `tools` Node tests for `b10-dynamic-tool-gate` (incomplete observation and authority-shaped claim negatives) |

MVP pass conditions for B10 are all of the following:

1. every row in the matrix passes at one exact reviewed revision;
2. focused Rust checks run on qualified native Linux and pass Clippy with
   warnings denied for the exercised packages;
3. required Ubuntu and Windows CI pass for the review revision;
4. a non-claim B10 suite report is generated (`claim_scope: non-claim`;
   evaluator cannot set Gate state); and
5. an affirmative or rejecting disposition for B10 is recorded against that
   bounded evidence. Under standing continuous-delivery authorization, the
   agent may record that disposition for this ADR-0050-class MVP path (and
   equivalent ADR-0040/0046/0047/0048-class fixed-denominator Gate MVPs) when
   items 1–4 are complete; the product owner may override. Unresolved Gate
   thresholds, live statistical campaigns, release/Profile promotion, and
   other Operating Model §2.4 boundaries still require explicit owner
   confirmation.

Live Provider/MCP marketplace statistical campaigns remain available for later
promotion work when additional signal is needed. They are not a P5-T04 MVP
completion mutex. Dynamic discovery must never auto-enable unqualified tools.

## Consequences

- P5-T04/D04 can close B10 after the fixed matrix, native Linux/Clippy,
  required CI, non-claim report, recorded B10 disposition, and docs sync.
- The B10 evaluator remains non-authoritative: reports cannot mutate Gate
  state; the documented product decision owns Gate status.
- Discovery≠enable, TaskContract-scoped exposure, quarantine fail-closed,
  composite child evidence retention, pure-read cache only, and original-key
  reconcile stay mandatory observations inside the fixed matrix.
- This decision does not transfer to GMVP-LINUX, release, or Profile and does
  not claim a public Tool marketplace.

## Non-goals and non-claims

This ADR does not claim automatic marketplace discovery enablement, public Tool
schema authority, live MCP statistical benefit, GMVP-LINUX, release, or
Profile.
