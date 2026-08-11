# P9-T01 closure - Async event foundation decision gate

- Task: `P9-T01`
- Branch: `personal/P9-T01-async-decision-gate`
- Leases: `lease/personal/P9-T01/async-decision-gate`; `lease/personal/P9-T01/windows-ci-recovery`
- Decision implementation revision: `826745c868b26a5aab71e0abeedb038e364267e4`
- Closure revision: `53d9c007dced054a117ca92a731ce80f0fee51d9`
- Draft PR: https://github.com/agentkernel/cognitive-os/pull/197
- Date: 2026-08-11

## Acceptance mapping

| Formal acceptance item | Evidence |
|---|---|
| Re-use P7-T04/D02 governed-path stage timing to distinguish governance from implementation tax | `p9_t01_async_decision_gate` collects cold and warm observations through `GovernedPathStageCollector`, validates each observation, and reports p50/p95/p99 per stage as a hypothesis-only record. |
| Preserve the authoritative SQLite single-writer boundary | The decision runner changes no authority write path. Its rule expressly treats aggregate `effect_persistence` as authority-path work, not transport evidence. |
| Migrate HTTP/watch/sidecar streaming only if connection/open/lock contention dominates p95 | The collector did not measure a separable HTTP/watch/sidecar transport stage. Although aggregate cold `effect_persistence` dominated p95 in both native runs, the deterministic rule correctly selected `conservative-no-migration`. |
| Record a conservative result when migration evidence is insufficient | `docs/checkpoints/20260811-personal-p9-t01-async-decision-gate.md` retains the raw redacted observation summary: two cold and two warm five-sample runs on native Linux, with no Provider traffic, secrets, authority contents, or user data. A future reconsideration requires a new bounded transport measurement. |

## Delivery slice

| Slice | Status | Evidence |
|---|---|---|
| D01 | done | Exact native Linux `cargo test -p cognitive-runtime --lib perf::tests -- --nocapture` passed 5/5 at `826745c`; the decision runner returned `conservative-no-migration`; after a Windows integration-test stall, bounded socket reads and serial loopback-daemon execution were added and required Ubuntu/Windows CI `31516749535` passed on recovery head `195510c`, then final closure CI `31539326728` passed on `53d9c00`. |

## Validation

| Check | Result | Revision / note |
|---|---|---|
| Exact native Linux focused performance tests | **pass** 5/5 | `826745c` on `DEV-LINUX-NATIVE-01` |
| Local `cargo fmt --all -- --check` | **pass** | closure worktree |
| Local `pnpm run check:consistency` | **pass** | closure worktree |
| Required Ubuntu/Windows CI | **pass** run `31539326728` | PR #197 head `53d9c00` |

## Non-claims

- No async runtime migration is authorized.
- No Gate, release, Profile, or generalized performance/Agent-benefit claim.
- The authority SQLite path remains single-writer; clients remain candidate-only.

## Closure sequence

1. D01 acceptance mapping is complete with the conservative decision.
2. Required CI is green on the closure head.
3. PR #197 is the task closure PR; merge it normally, then remove the task branch and reconcile local `main`.
