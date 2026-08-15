# CognitiveOS Personal C1/C2 governed-task benchmark addendum

- Campaign: `PERSONAL-PERF-EVAL-004`
- Status: **preregistered; B0 blocked on target-environment access**
- Full OS-only scope amendment: [personal-performance-benchmark-full-os-only-addendum.md](personal-performance-benchmark-full-os-only-addendum.md)
- Parent execution contract: [personal-performance-benchmark-execution-plan.md](personal-performance-benchmark-execution-plan.md) v1.1
- Scope: this document defines C1/C2 execution. The owner-authorized full
  OS-only amendment additionally governs the O2-O14, Tool, fault, concurrency,
  soak, and journey-register dispositions without creating a second campaign.
- Source revision to freeze: `93dde21da1635329bd11949b265f205ead46186b`
- Target: `B01-DESKTOP-002` / `B01-Desktop-Linux-002`
- Claim ceiling: `hypothesis` / non-claim; no Gate, release, Profile, B01,
  or Agent-benefit promotion.

## 1. Scope and arms

The campaign does not rewrite the closed EVAL-002 report. It uses the current
`main` product revision and records new denominators independently.

| Class | Required capability | Pure Pi reference | Governed OS arm |
|---|---|---|---|
| C1 | WorkspaceRead and WorkspaceSearch against a fixed read-only workspace | Pi plus an equivalent campaign fixture tool adapter | admitted Task -> Context -> Pi candidate -> daemon Tool executor |
| C2a | WorkspaceWrite and WorkspacePatch with expected preimage and atomic publish | Pi plus the same fixture workspace oracle | governed mutation carrier -> Effect -> reconcile -> verifier |
| C2b | Memory/Skill session-2 reuse | Pi receives the frozen procedure bytes as the reference condition | daemon-authorized Memory/Skill consumption with exact pin and no restatement |
| C2c | crash/unknown-outcome recovery | fixture mutation reference with original-key query | persisted Effect, reconcile, independent verification and acceptance |
| C2d | verified completion | external mechanical oracle | CAS-backed verifier report and acceptance authority |

The pure-Pi reference never writes CognitiveOS authority state and never uses
the daemon, Extension, Task, Context, Memory, Skill, retry, cache, or verifier.
If equivalent fixture tooling cannot be qualified, the affected pair remains
`not-run` rather than being replaced by a daemon proxy.

## 2. Frozen corpus and sample plan

The campaign corpus is a fixture-only, secret-free repository snapshot with
immutable input digest, allowlisted paths, expected preimages, hidden tests,
independent oracle, reset procedure, and cleanup digest per seed.

- B0: one qualification seed per class, three warmups per arm, secret scan,
  tool-equivalence check, timeout and cleanup check; no claim samples.
- B1: five pilot seeds per class, two runs per arm, used only to validate the
  runner and classify instrumentation failures.
- B2: 30 held-out paired seeds per class, three runs per arm where the Provider
  does not support deterministic replay; every started run is retained.
- B3: ten seeds per applicable arm for stale epoch, descriptor drift,
  preimage mismatch, duplicate dispatch, process kill, and unknown outcome.
- B4: C1/C2 mixed local workload at concurrency 1/8/16 with 100 local
  observations per profile; no live external mutation outside the fixture.

Primary endpoints are oracle completion, paired wall time, Tool dispatch count,
duplicate side effects, reconcile time, output bytes, CPU/RSS/FD, and safety
counters. Token/cost is `not_available` unless complete request-bound usage and
pricing facts are exposed for both arms.

## 3. Execution gates

1. Owner authorization is recorded in the preregistration before any target
   mutation or credential entry.
2. Exact source, corpus, runner, oracle, broker and Extension digests are
   frozen before B0.
3. Target access, Provider credential path, graphical/SecretStore boundary,
   and isolated campaign root must be available. Snapshot restore or changes
   to residual P9-T04 state require a separate owner decision.
4. B0 must pass before B1/B2. A missing target, broker, runner, credential,
   fixture, or oracle records `blocked`/`not-run`; it is never silently
   replaced.
5. No product code, contract, negative, test, or generated handbook source is
   modified for this campaign. Campaign instruments live only in ignored
   artifact roots or the target guest campaign root.

## 4. Capability and claim boundaries

- A pure-Pi completion is not OS Task completion.
- A Tool receipt, Effect closure, Pi `agent_end`, or verifier report alone is
  not Task completion; only the daemon acceptance authority may close a Task.
- C1/C2 results are descriptive/hypothesis evidence until independently
  reviewed.
- Any missing denominator, mixed workspace state, unqualified fixture, or
  cleanup failure makes the affected cell `not-run` or `partial` and prevents
  promotion.
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
