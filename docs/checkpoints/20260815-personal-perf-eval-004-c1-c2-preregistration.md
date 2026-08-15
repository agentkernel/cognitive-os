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

## Owner-authorized full OS-only scope amendment

On 2026-08-15, the owner expanded this existing campaign under the same active
lease, `lease/personal/EVAL-20260815/full-os-only-assessment`. The amended
scope and B0 capability matrix are recorded in
[personal-performance-benchmark-full-os-only-addendum.md](../evaluation/personal-performance-benchmark-full-os-only-addendum.md).
It covers the applicable C1/C2, O2-O14, Tool, fault/recovery, concurrency,
soak, and journey-register cells. It neither creates an overlapping evaluation
lease nor changes the closed EVAL-002 denominator or conclusion.

The amendment is measurement-only. A product gap pauses only its affected
cells and requires a separate product task, branch, Draft PR, lease, supported
validation, merge, and newly frozen campaign revision before re-execution.

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
| Product chain | C1/C2 production chain present on source | partial: WorkspaceRead has limited public reachability; parameter-bearing Search/Write/Patch lack a public persisted-Intent carrier |
| Target guest control | access to `B01-Desktop-Linux-002` and isolated campaign root | **pass:** `wuz@192.168.1.2` -> ProxyJump -> `hal9001@192.168.123.160`; isolated root and loopback port qualified |
| Provider credential | owner-approved SecretStore path | pass: owner completed graphical hidden input; public redacted doctor confirms resolving secret reference |
| Pure-Pi broker | digest-frozen, loopback-only, no CognitiveOS authority | paused: no compliant paired scope until product carrier gap is resolved |
| Paired runner/corpus/oracle | digest-frozen and fixture-qualified | paused: affected C1/C2 paths are product-blocked |
| B0 qualification | target execution and cleanup | partial, then blocked for affected cells by public parameter-carrier gap |

The registered route was verified non-interactively with strict host-key
checking. On `hal9000`, `virsh -c qemu:///system` identified the running target
and its guest address `192.168.123.160`; guest SSH then identified
`hal9001-Standard-PC-Q35-ICH9-2009`, Linux `7.0.0-28-generic`, and glibc
`2.39`. During route qualification, no lifecycle, snapshot, campaign root,
SecretStore, Provider, or owner-secret operation occurred; the later
owner-operated hidden-input flow is recorded below. O2, O3, and O14 are
currently `not_available`; O4 is capability `partial` but lacks a compliant
fairness observation surface.

No B1/B2/B3/B4 sample has started. No sample denominator, performance result,
capability result, or safety claim is created by this preregistration.

## B0 partial execution record: isolated runtime and asset freeze

The target route is now qualified and an isolated campaign root was created at
`/home/hal9001/perfeval004` with mode `0700`. It does not overlap the existing
`perfeval002`, `p9t04`, or `cos-current` roots. The campaign reserves
`127.0.0.1:48284`; it does not share the pre-existing daemon listener on
`127.0.0.1:48181`.

The guest lacks Git, Rust, Cargo, pnpm, Pi, and the CognitiveOS CLI. No system
package or global installation was changed. The exact source was instead
cloned and built on the authorized host at the frozen commit, then transferred
through a local temporary relay with matching SHA-256 digests. The following
campaign-only assets are retained only beneath the isolated guest root:

| Asset | Digest / version | Qualification |
|---|---|---|
| tracked source archive | `sha256:3578b4faf2c8b164d8c751dab7e5d82d85b401a964489efe924a5bd2ab1cd02c` | 13,639,680-byte archive extracted from clean `93dde...` source |
| `kernel-server` | `sha256:a38e042adc6bb733b23e98490870035ac7672e448fcacf0742d0cc7869f1b8ce` | exact-source release build; dynamic libraries resolve on guest |
| `cognitive` CLI | `sha256:257ad2218401a4dff43ddd638ee1aad429981d2859e4dd58037df43d99b8967e` | exact-source release build |
| Pi package | `0.81.1`; tarball `sha256:420113c0282160e6181656fd16cf18742f76bf9040ee3dfb9cb67e3e6ad5641c`; declared SRI `sha512-r6ovAsZOgAqbC/aU6s+/dPnv/sGZBuWyZNvi3pXjpbuX5wvp3XvGkQI7/VLvX2o9XpmpFaPUxKNym1WfkN/P8A==` | guest-local, locked runtime; no global install |
| Pi runtime lockfile | `sha256:b6e5aae1427cf029c1764fab5f0eb360e16bf3d7080845a9232b61f9494560fb` | Pi `0.81.1` executed locally |
| Pi extension | archive `sha256:2e03c53d26213ac37e594e5435d444ee294ccb9d2ac7ee2718dd0248968f9c2b`; entry `sha256:d27f97764e55b9a9b22bbf7e22e48c0ef2a017924ed13684b143b196991c1a57` | pinned Pi loaded `--extension` successfully |

Before the owner credential entry, the daemon was started by the exact public `cognitive daemon start` caller with
the campaign-owned runtime root and the reserved loopback endpoint. `cognitive
daemon status` confirms a live lock, endpoint, and mode-`0600` bootstrap file;
the bootstrap value was neither read nor recorded. `cognitive status` reports
only the expected pre-credential state: system/database/secret/daemon `ready`,
Provider `blocked` (`provider_config_missing`), Pi `not_configured`, and
`first_conversation_ready: false`. The health/status probes have no authority
side effects. No owner secret or Provider was accessed.

The owner completed the guest graphical hidden-input flow. Post-entry public
status and doctor projections are redacted and report native SecretStore
availability, a resolving secret reference, a selected Provider snapshot and
model digest, and Provider `ready`; no secret material was displayed or
recorded. `cognitive pi configure` then wrote only campaign-local executable
and extension paths (`pi.json`
`sha256:a5c3f031a2addfd0affd6f7794f732dbb34ad7b5211fb47fbb3890f91c603efd`).
The public doctor surface reports Pi package `ready`, pinned and observed
version `0.81.1`, all required runtime components `ready`, and
`first_conversation_ready: true`. This is readiness evidence only, not a
conversation, C1/C2 Task, Tool, Effect, verification, acceptance, Gate,
release, Profile, B01, or Agent-benefit claim.

Consequently B0 remains `partial`, not pass: the pure-Pi broker, equivalent
fixture corpus/tool adapter, paired runner, mechanical oracle, and redactor
are still required before any B1 sample may start.

## B0 product-gap finding and campaign pause

Frozen-source/public-contract inspection found that the admitted Task front
door and configured daemon-private candidate protocol do not carry the same
operation parameters. The public candidate carrier persists only a
`parameters_digest`; it has no schema-bound public route that persists
WorkspaceSearch `parameters.query` or WorkspaceWrite/Patch
`parameters.input_b64` and `parameters.preimage` into the governed Intent.
The production router correctly fails closed when those fields are absent.

This is a product capability gap, not a runner defect. The campaign must not
compensate through raw SQLite, private candidate transport injection, test
helpers, or campaign-only mutation of authority state. Its consequences are:

- C1 WorkspaceRead may later be qualified only as limited production-caller
  reachability, because its target does not require the missing fields;
- C1 WorkspaceSearch is `blocked/not-run` for positive execution;
- C2a/A3/T5 positive Write/Patch is `blocked/not-run`;
- real C2b/A6/S4/S8 session-2 consumption is `not-run`, because its consumer
  remains daemon-private and lacks durable public consumption evidence;
- C2c/A7/T9 reconcile/unknown-outcome is `not-run`, because no public fault,
  reconcile, or Effect-history surface exists;
- C2d/O6 verified completion is `partial/not-run`, because verifier/acceptance
  terminal evidence is not publicly queryable.

The campaign is paused before broker/corpus/runner/oracle construction and
before B1. A separate product task must establish a public, schema-bound,
authority-safe parameter carrier and the required observation surfaces; after
its supported validation and merge, EVAL-004 needs a newly frozen revision and
fresh B0 for affected cells.

## Required owner routing action

Keep this evaluation lease paused. Create a separate product task, branch,
Draft PR, and product lease for the public parameter-carrier/observation gap;
then merge it, freeze a new EVAL-004 revision, and restart B0 for affected
cells. The existing campaign assets and Provider SecretStore entry remain
isolated pending that decision.
