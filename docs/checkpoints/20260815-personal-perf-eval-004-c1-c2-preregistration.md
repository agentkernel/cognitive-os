# PERSONAL-PERF-EVAL-004 C1/C2 preregistration

- Campaign: `PERSONAL-PERF-EVAL-004`
- Scope: C1 read/search, C2 mutation/recovery, Memory/Skill reuse, and
  independently verified completion
- Execution contract: [C1/C2 benchmark addendum](../evaluation/personal-c1-c2-benchmark-execution-plan.md)
- Parent contract: [personal-performance-benchmark-execution-plan.md](../evaluation/personal-performance-benchmark-execution-plan.md) v1.1
- Owner authorization: explicitly granted in the user instruction on
  2026-08-15 to re-execute C1/C2 benchmark cells.
- Source revision: `93dde21da1635329bd11949b265f205ead46186b`
- Target: `B01-DESKTOP-002` / `B01-Desktop-Linux-002`
- Claim ceiling: `hypothesis` / non-claim
- Independent reviewer: `not_reviewed`
- Product code changes: none permitted

## Freeze disposition

The prior `PERSONAL-PERF-EVAL-002` report remains historical evidence at its
old frozen revision. This campaign is a new denominator and may not rewrite
its `not-run` cells. The current product revision contains production C1/C2
callers, verifier, acceptance authority, and governed Memory/Skill consumer;
qualification must still prove that the campaign runner reaches those callers
on the target guest.

## B0 precondition status

| Gate | Required fact | Status |
|---|---|---|
| Source | exact pushed `main` revision | pass |
| Product chain | C1/C2 production chain present on source | pass by repository evidence |
| Target guest control | access to `B01-Desktop-Linux-002` and isolated campaign root | **blocked: no `virsh`, SSH, SCP, WSL, or equivalent guest control is available in the current session** |
| Provider credential | owner-approved SecretStore path | not-run until target access is restored |
| Pure-Pi broker | digest-frozen, loopback-only, no CognitiveOS authority | not-run |
| Paired runner/corpus/oracle | digest-frozen and fixture-qualified | not-run |
| B0 qualification | target execution and cleanup | not-run |

No B1/B2/B3/B4 sample has started. No sample denominator, performance result,
capability result, or safety claim is created by this preregistration.

## Required recovery action

Provide a registered target control route for `B01-Desktop-Linux-002` (or an
approved equivalent with the same environment identity), including the
predeclared SSH/guest control path and owner-approved credential-entry route.
After that route is available, freeze the runner/corpus/oracle digests, run B0,
and append every outcome here before starting B1.
