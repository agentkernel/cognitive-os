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
3. Select **stream-only async migration** only if the dominant stage is an
   I/O-bound `effect_persistence` or stream transport stage and the result is
   reproduced in both runs. Authorization, Context resolution, and cache
   reuse are implementation/governance work, not permission to move the
   authority writer to an async executor.
4. Otherwise select the conservative outcome: no async migration is justified
   by this evidence; retain the synchronous stream path and record follow-up
   profiling as non-claim work.

The decision is hypothesis-only. It cannot promote a Gate, release, Profile,
or generalized performance/Agent-benefit claim.

## Validation state

| Check | Status | Evidence |
|---|---|---|
| task claim, exact lease, and plan reconciliation | pass | current `PROGRESS.md`, formal plan, and active lease row |
| native Linux exact-revision collector | not-run | must run after this checkpoint is pushed |
| local diff/consistency checks | pass | `git diff --check`; linter diagnostics absent |
| required CI | not-run | required after the immutable checkpoint revision exists |

## Next action

Push this secret-free checkpoint, create the single Draft PR, run the
collector on `DEV-LINUX-NATIVE-01` at the exact revision, then append the
redacted result and either close P9-T01 conservatively or register the bounded
stream-only D02 slice.
