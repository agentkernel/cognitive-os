# P9-T01 async event decision gate

- Status: `in-progress` / D01
- Task: `P9-T01`
- Change class: `implementation-only` decision evidence; no runtime semantic change
- Branch: `personal/P9-T01-async-decision-gate`
- Lease: `lease/personal/P9-T01/async-decision-gate`
- Starting revision: `870e44c1fff9760af799d78b456f7470372d6ad1`

## Decision purpose

P9-T01 decides whether the governed HTTP/watch/sidecar event paths need a
bounded async-runtime migration. It does not authorize an async rewrite by
itself. The authority SQLite path remains single-writer, and no async design
may create a second Task/Effect writer or bypass persist-before-dispatch,
fencing, budget, secret, or independent-verifier boundaries.

## Preregistered evidence method

The evidence input is the existing P7-T04/D02 `GovernedPathStageCollector`,
re-run on the exact pushed P9-T01 revision in a disposable native Linux
worktree. The run will retain only a redacted hypothesis report containing
raw stage durations and environment/revision digests. No Provider traffic,
secret, authority database content, or raw user data is collected.

The four stages are `authorization`, `context_resolution`, `cache_reuse`, and
`effect_persistence`. The collector's effect stage intentionally includes the
daemon-owned SQLite open, admission, Intent persistence, and reload boundary;
that stage is therefore treated as a governance-path observation, not as a
claim about a production request distribution.

## Deterministic decision rule

1. Compute p50/p95/p99 separately for each stage and cache mode, without
   combining warm and cold observations.
2. Select the dominant stage by cold p95 share of the total cold p95. A stage
   is dominant only when it contributes at least 50% of that total and is
   reproducible in both native runs.
3. Select **stream-only async migration** only if a separately measured
   HTTP/watch/sidecar transport stage dominates p95 and the result is
   reproduced in both runs. The aggregate `effect_persistence` stage includes
   authority-store open, admission, Intent persistence, and reload, so it can
   request component profiling but cannot authorize a stream migration.
4. Otherwise select the conservative outcome: no async migration is justified
   by this evidence; retain the synchronous stream path and record follow-up
   profiling as non-claim work.

The decision is hypothesis-only. It cannot promote a Gate, release, Profile,
or generalized performance/Agent-benefit claim.

## Validation state

| Check | Status | Evidence |
|---|---|---|
| task claim, exact lease, and plan reconciliation | pass | current `PROGRESS.md`, formal plan, and active lease row |
| native Linux exact-revision collector | pass | exact `826745c` bundle worktree; focused `perf::tests` 5/5 |
| local diff/consistency checks | pass | `git diff --check`; linter diagnostics absent |
| required CI | not-run | required after the immutable checkpoint revision exists |

## Exact native Linux result

At `826745c868b26a5aab71e0abeedb038e364267e4`, a disposable native worktree
created from the transferred Git bundle on `DEV-LINUX-NATIVE-01` passed
`cargo test -p cognitive-runtime --lib perf::tests -- --nocapture` (5/5).
The P9-T01 runner then collected two cold and two warm five-sample observations.
`effect_persistence` dominated cold p95 (run 1: 310.790 ms of 311.863 ms;
run 2: 1250.922 ms of 1251.939 ms). This is explicitly **not** stream
transport evidence: the stage includes authority-store open, admission, Intent
persistence, and reload.

The executable rule therefore returned `conservative-no-migration`. No async
runtime migration is authorized. Any later P9-T01 reconsideration needs a new
bounded measurement that separates HTTP/watch/sidecar transport from the
single-writer authority path.

## Remaining validation

The native result is `tested-local` implementation evidence only. Required CI
for the exact final task revision remains pending; no Gate, release, Profile,
or generalized performance claim is made.
