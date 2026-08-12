# ADR-0051: Comprehensive performance campaign registration

- Status: accepted
- Date: 2026-08-12
- Decision type: product-semantic + structural
- Owner authorization: 2026-08-12 continuous campaign instruction
- Task: `P9-T04`

## Context

`P7-T04`, `P8-T05`, `P9-T01`, and `P9-T03` provide bounded performance
observations and measurement primitives. They do not provide a single,
preregistered, redacted campaign that can measure the complete governed Task
path, real Provider route, resource usage, fault recovery, or A/B/C/D outcome.

The owner has authorized a comprehensive campaign on the sole active B01
environment, `B01-Desktop-Linux-002`. This campaign must not convert prior
local, fixture, or CI results into B01, Gate, release, Profile, or generalized
Agent-benefit evidence.

## Decision

Add `P9-T04` as a single task-atomic performance campaign. Its delivery is a
measurement-only runner and the complete execution, redacted evidence, report,
cleanup, independent-verifier disposition, and normal task closure described
in `personal-performance-benchmark-execution-plan.md`.

The campaign is not a new product Gate and does not alter the passed B01
first-install/first-conversation Gate. It creates a separately preregistered
performance campaign whose maximum claim remains the result actually supported
by the final report. In particular:

- no TTFT is reported without streaming timestamps;
- missing Provider usage is `not_available`, never measured zero;
- a failed or incomplete L5 campaign results in a complete non-claim report
  and cleanup, not a benefit claim;
- B01 use begins only after the P9-T04 preregistration and campaign lease are
  active, and every execution uses an exact pushed revision in a disposable
  Git worktree.

## Owner disposition on `L5`, 2026-08-12

The owner dropped `L5`. The repository has no runner that can execute the `A`
arm while satisfying the approved-SecretStore boundary, and no owner-designated
native Agent baseline was nominated, so the A/B/C/D benefit campaign is closed
as `not-run` rather than approximated. The campaign therefore concludes with a
complete non-claim report over `L0`-`L4`.

The direct consequence is that no governance non-inferiority or Agent-benefit
conclusion may be drawn from this campaign. The measured governance overhead
remains a single-arm observation; the execution plan's `B`-versus-`A` thresholds
are not evaluated, not met, and not failed.

## Consequences

P9-T04 may add internal instrumentation and runner surfaces only when they
preserve daemon-only authority, persist-before-dispatch, fencing, independent
verification, and SecretStore boundaries. It may not make the runner an
authority writer or permit Provider secrets outside the approved hidden-input
SecretStore path.

The formal plan, trace, Current snapshot, lease ledger, campaign
preregistration, and detailed task card are updated in the same delivery.

