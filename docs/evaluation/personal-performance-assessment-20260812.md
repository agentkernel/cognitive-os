# CognitiveOS Personal performance and capability assessment — 2026-08-12

- Campaign: `PERSONAL-PERF-EVAL-002` (owner-directed evaluation campaign,
  [Operating Model §2.5](../governance/DEVELOPMENT-OPERATING-MODEL.md))
- Execution contract: [personal-performance-benchmark-execution-plan.md](personal-performance-benchmark-execution-plan.md) v1.1
- Freeze and per-cell execution record: [20260812-personal-perf-eval-002-preregistration.md](../checkpoints/20260812-personal-perf-eval-002-preregistration.md)
- Source revisions: phases 1–2
  `4cbec8470bc7a19f23f978e8754ed20133122eb1`; recovered phase-3 `B6`
  `158e9276e49573db84aeb6ab55012d314368a76c` (both pushed and clean)
- Environment: `B01-DESKTOP-002`, guest `B01-Desktop-Linux-002`
- Claim level: **`hypothesis` / non-claim**
- Independent verifier disposition: **`not_reviewed`**
- Agent benefit claimed: **no**
- Document status: final campaign report; phases 1 and 2 complete, phase-3
  recovery complete and campaign closed 2026-08-13

## 1. What this report says, and what it cannot say

It reports measured latencies, counts, outcome classes, resource behaviour,
authority outcomes and a **real paired Agent comparison** for CognitiveOS
Personal, on a named guest, at named exact pushed revisions, with every started
sample retained.

The original campaign ran in two phases. **Phase 1** was measurement-only: Operating
Model §2.5 forbids implementing a missing runner or credential path
mid-campaign, and both were missing, so the paired comparison was correctly
recorded `not-run`. **Phase 2** followed an explicit owner scope change granting
authorization to build the evaluation instruments and complete all tests. The
scope change is recorded in the preregistration rather than applied silently,
and the one boundary that did **not** move is that no product code, contract,
negative, test or generated documentation source was modified in either phase.
The phase-2 instruments — a pure-Pi credential broker, a frozen nine-family task
corpus with mechanical oracles, and a paired runner — live only in ignored
artifact roots and are pinned by digest.

**Phase 3** was later closed as wholly `not-run`, but retained guest evidence
proves that `B6` had already started and completed. Section 6b and the appended
execution record supersede that one phase-3 disposition without rewriting or
discarding the earlier publication.

It still does **not** establish a Gate, a release, a Profile, a B01 or B01-W
outcome. The claim ceiling stays `hypothesis` and the independent verifier
disposition stays `not_reviewed`.

The paired result covers `C0` prompt-contained tasks only. Tasks needing real
workspace tools, mutation, Memory/Skill reuse or independent completion remain
**unreachable capability on the OS arm**, not a slow path (§14), so nothing here
generalizes to an autonomous workspace Agent.

## 2. Headline findings

**0. On prompt-contained tasks, CognitiveOS costs about 1.8 seconds per task and
changes task success by nothing measurable.** Over 270 held-out paired
task-seeds (540 runs, all retained), pure Pi completed 240/270 = 88.9 % and Pi
on CognitiveOS completed 242/270 = 89.6 % — a difference of +0.7 pp with a 95 %
clustered-bootstrap CI of [−2.22, +3.70] pp and McNemar exact p = 0.8145. On the
same tasks the OS arm took a median **+1828.5 ms** longer (95 % CI
[1753.6, 1893.9] ms), or **+44.2 %** relative. Both arms produced the same
amount of output (237.5 vs 235.0 characters median), so this is pure path
overhead, not extra work.

**0b. The daemon is not where that 1.8 s goes, and four other explanations are
excluded by measurement.** Extension *load* costs 4.5 ms (1619.0 vs 1614.5 ms
p50, inside noise). The non-streaming OS proxy costs nothing: identical prompts
sent streamed and non-streamed through the same broker differ by a median of
−38.7 ms. The daemon's own local residual is flat at 128.1 ms on real task
payloads, the same as on a tiny marker, so it does not scale. The OS arm does
not generate more text. And the broker that makes the pure-Pi arm possible adds
0.5 ms. What remains — roughly 2 s — sits in the Extension's per-request path
inside Pi, which is an inference from independently measured parts rather than a
directly observed stage, and needs nested per-run timing to confirm.

**0c. The recovered post-P2-T11 replay does not show a measurable change.**
All 270 frozen held-out blocks / 540 runs completed as a retained `B6` replay at
`158e9276e49573db84aeb6ab55012d314368a76c`: completion delta changed by
+0.37 pp versus B2 (95 % task-cluster CI [−3.33, +4.07] pp), and median paired
overhead changed by −91.5 ms over the 269 common completed blocks (95 % CI
[−219.9, +31.2] ms). Both include zero. This is a post-P2-T11-only replay; it
predates and says nothing about P2-T10.

**1. The product reports `ready`, and even `first_conversation_ready: true`, on
a Provider path that cannot work.** All 80 Provider requests failed closed with
`PERSONAL_PROVIDER_SECRET_UNAVAILABLE`, while `cognitive status` and
`cognitive doctor` reported `secret: ready`, `provider: ready`,
`overall: ready`. A Secret Service search (object paths only, no value ever
requested) showed the referenced key item is **absent**: the P9-T04 cleanup
removed its SecretStore entry while leaving the `provider.json` that references
it. The `provider` readiness component's source is `filesystem:provider-config`
and it asserts only `secret_ref_present: true`; the `secret` component probes
backend availability, not the reference. A dangling reference therefore reads as
ready and fails on every real request.

It gets sharper. Once Pi was configured (§11a), **all six components read
`ready` and `first_conversation_ready` flipped to `true`** with no Provider key
on the machine at all. The product's own route smoke refuses to run unless
`overall == ready && first_conversation_ready == true`; that gate passed, and
all 10 conversations then failed. A user following the product's own readiness
signal is told they are ready to talk when they are not.

**2. `POST /management/resource/v1/skill/binding/revoke` is unreachable.**
The dispatcher tests `starts_with(".../skill/bind")` before
`starts_with(".../skill/binding/revoke")`, and `skill/bind` is a prefix of
`skill/binding/revoke`. Confirmed differentially: posting to the revoke path
with a bind-shaped payload returns the bind handler's own message, byte-identical
to a plain bind. Skill binding revoke cannot be performed from the public
management surface at this revision.

**3. One probe owns the cost of the product's main diagnostic.** Over 30
samples, the `secret` readiness component measured 57.5 ms median (44–76 ms)
while `system`, `database`, `provider`, `daemon` and `pi` each measured 0 ms.
That single probe is essentially all of the ~65 ms in-daemon cost of
`cognitive status` / `doctor`; CLI process start adds only ~6 ms. The same probe
that costs 57.5 ms is the one that failed to notice finding 1.

**4. Pi start is expensive before it does any work.** `pi --version`, which does
nothing, costs 1682.5 ms p50 against a 44.5 ms bare Node floor. The daemon's own
refusal on the same path costs about 110 ms, so the daemon is not where the
Pi-route time goes — corroborating P9-T04's conclusion with a cleaner baseline.

**5. Everything else about the local daemon is fast, clean and honest.**
In-daemon reads answer in under 1 ms; 832 concurrent requests through 33
connections produced zero errors; ten stop/start cycles left zero orphans, zero
stale locks and zero stale endpoint files; a 1 h soak of 1620 requests moved RSS
by 40 kB with zero additional disk writes and flat FD, thread, database and WAL
figures; all six authority negative controls failed closed with precise
registered codes; and the Tool projection honestly declares 2 of 6 families
execution-ready.

## 3. Cell disposition

| Cell | Instrument | Started / retained | Disposition |
|---|---|---:|---|
| `B0` staging and qualification | build + digest verify + daemon bring-up | 66 / 66 files | **pass** |
| `D1` Provider proxy marker | frozen route runner | 30 / 30 | partial (fail-closed only) |
| `D2` warm repeated route | frozen route runner | 50 / 50 | partial (fail-closed only) |
| `UJ4` / `O1` Task admission | frozen T1 runner | 30 / 30 | **pass** |
| `UJ3` daily operations | public surface | 930 / 930 | **pass** |
| `UJ3b` readiness cost attribution | public surface | 180 / 180 | **pass** |
| `T-GOV` Tool projection + lifecycle | public surface | 1 dump + 4 probes | partial |
| `MS-AUTH` Memory/Skill authority | public surface | 106 / 106 | partial |
| `B3` faults, restart, cleanup | frozen runner + public surface | 40 / 40 | partial |
| `B4` concurrency and overload | public surface | 832 / 832 | **pass** |
| `B5` 1 h soak | public surface | 1620 / 1620 | **pass** |
| **Phase 3** `B5` 8 h soak | durable minute/block/restart records | 12 960 + 96 / 12 960 + 96 | **pass**; no leak signature, 8/8 clean restarts |
| **Phase 3** `B5` 24 h soak | — | 0 / 0 | **not-run**; 8 h left no unresolved slope |
| `O-LAUNCH` (phase 1; Pi launch to failure) | frozen route smoke | 10 / 10 + 20 spawn | **pass** as a launch observation |
| **Phase 2** credential import | product stdin path | 1 | **pass** |
| **Phase 2** `D1` / `D2` re-run, live Provider | frozen route runner | 80 / 80 | **pass** |
| **Phase 2** `O1` Pi first response | frozen route smoke | 30 / 30 | **pass** |
| **Phase 2** `P` arm broker qualification | campaign broker | 1 | **pass** |
| **Phase 2** `B1` pilot paired | paired runner | 180 / 180 | **pass** |
| **Phase 2** `B2` confirmatory paired (held-out) | paired runner | 540 / 540 | **pass** |
| **Phase 2** attribution ablations | 3 experiments | 30 + 24 + 12 | **pass** |
| **Phase 3 recovered** `B6` post-P2-T11 replay | unchanged paired runner | 540 / 540 | **pass**; no measurable change |
| **Phase 3 recovered** `O-NESTED-PILOT` | passive loopback observer | 20 / 20 | **partial**; cross-clock decomposition invalid |
| **Phase 3** `UJ2` cold/warm conversation | paired runner + public daemon lifecycle | 20 new / 20 retained | **partial**; warm works, cold=`instrument_error`, Pi-warm unavailable |
| **Phase 3** `B3` client deadline | frozen route runner | 10 / 10 | **partial**; bounded `outcome_unknown` |
| **Phase 3** `B3` daemon/broker unavailable | paired runner + controlled process stop | 20 new / 20 retained | **partial**; daemon fails closed, broker fault confounded |
| **Phase 3** `B3` Pi process kill | frozen Pi + controlled SIGKILL | 10 / 10 | **pass**; 0 orphans |
| **Phase 3** `B3` response-size bound | — | 0 / 0 | **not-run**; no controlled oversize fixture |
| **Phase 3** `B3` model mismatch | frozen route runner | 10 / 10 | **pass**; denied before dispatch |
| **Phase 3** `B3` upstream timeout / rate limit | — | 0 / 0 each | **not-run**; no controlled fixture |
| **Phase 3** `B3` stale Task / epoch | — | 0 / 0 | **not-run**; no public execution commit path |
| **Phase 3** optional `T3` Tool selection | — | 0 / 0 | **not-run**; no frozen pilot task/oracle |
| **Phase 3** `B4` mixed Agent/local | paired runner + health/status/doctor | 310 / 310 | **partial**; executed subset all passed |
| Cleanup, secret scan, boundary check | scan + reconciliation | 30 evidence files | **pass** |
| `A3`/`A6`/`A7`, `G*-C1/C2`, `A1-C1` | — | 0 | **not-run** (no OS product path) |
| `S4`/`S8`, `T4`–`T9`, `O4`–`O6` | — | 0 | **not-run** (no governed consumer / production caller) |
| `O2`/`O3` Context, `O14` backup/restore | — | 0 | **`not_available`** (no public observation surface) |
| `B5` 8 h / 24 h, `B6` replay | — | 0 | **not-run** (gated on prior exits) |

The last row is the phase-2 publication-time disposition. The phase-3 rows
above supersede it in full: `B6` was recovered as executed, `B5` 8 h ran to a
complete passing denominator, and `B5` 24 h is `not-run` because its trigger
condition was not met.

Total retained samples: **18 735** (4739 through phase 2, 540 recovered `B6`
runs, 20 recovered nested-timing pilot runs, 20 `UJ2` cold-stratum runs,
10 phase-3 deadline requests, 20 broker-fault runs, 10 Pi-kill runs,
10 model-mismatch requests, 310 mixed-profile Agent/local samples, and the
completed `B5` 8 h soak's 12 960 local operations plus 96 paired runs). The 8
hourly restart cycles are recorded facts alongside, consistent with the 1 h
soak's accounting. Warmups (three per Provider cell, three per
UJ3 surface, three before the `O1` cell, two before each paired batch) were
discarded before their cells began and are not counted; no started sample was
discarded anywhere.

`O-LAUNCH` was **added during phase 1** and is disclosed as such: the
preregistered `O` cell was unrunnable without a credential, so a fixed-N
substitute measured the part that remained observable. Phase 2 then ran the real
`O1` cell, so `O-LAUNCH` is retained as a failure-path observation only.

## 4. Route performance (plan §11.1)

| Path | N | p50 | p95 | Notes |
|---|---:|---:|---:|---|
| `GET /personal/health` | 200 | 0.61 ms | 0.85 ms | trivial in-daemon read |
| six-resource projection, per family | 50 × 6 | 0.65–0.87 ms | n/a | all 200 |
| bounded watch / replay, per family | 10 × 6 | 0.57–0.86 ms | n/a | all 200 |
| task-bound resource watch | 20 | 0.95 ms | n/a | all 200 |
| `GET /personal/status` authenticated | 100 | 64.86 ms | 84.85 ms | readiness evaluation |
| `cognitive status` CLI | 100 | 70.77 ms | 88.04 ms | + ~6 ms process start |
| `cognitive doctor` CLI | 50 | 71.00 ms | n/a | |
| `cognitive daemon status` CLI | 50 | 5.05 ms | n/a | local files, no HTTP |
| Provider request, fail-closed | 80 | 109–116 ms | ≤134 ms | no dispatch, `retry=0` |
| selected-model mismatch denial | 10 | 36.48 ms | n/a | pre-dispatch, zero dispatches |
| daemon-unavailable refusal | 10 | 0.23 ms | n/a | connection refused |
| `daemon stop` | 10 | 9.81 ms | n/a | |
| `daemon start` (incl. readiness wait) | 10 | 105.06 ms | n/a | |

p95 is reported only where N >= 100, per plan §7.1. No p99 is claimed anywhere.
No time to first token is reported: the proxy is non-streaming. No cost is
reported: there is no pricing snapshot and no Provider call succeeded.

Pure-Pi broker latency, OS Pi route latency and Pi launch cost are **`not-run`**.
The P9-T04 observation that Pi launch adds roughly 3.5 s over the same call
through the daemon client could not be re-measured at this revision.

The preceding two sentences describe the phase-1 boundary only; phase 2
subsequently measured the broker and working OS Pi route, and phase 3 adds the
strata below.

### 4a. `UJ2` cold/warm strata at the post-P2-T11 revision

The daemon-warm/Pi-cold stratum is the recovered `B6` batch: each Agent
invocation starts a fresh Pi process while the daemon and broker remain warm.
Across the full 270-pair denominator, `P` completed 240 and `O` completed 243.
On the ten `A4` task inputs reused for the cold comparison, both arms completed
10/10 with wall medians 4008.5 ms (`P`) and 6160.0 ms (`O`).

The attempted daemon-cold/Pi-cold sub-cell retained 10 pairs / 20 runs. `P`
completed 10/10 at 3882.8 ms median; `O` completed 0/10 because every daemon
start returned exit 1 before the fresh Pi process could reach a Provider
exchange. Review then found the orchestration had omitted the campaign's
required `--bind 127.0.0.1:48282` and fallen onto the CLI's 48181 default, where
the untouched P9-T04 daemon was already listening. The samples remain retained
as daemon-unavailable evidence but the UJ2 cold stratum is `instrument_error`,
not a product result, and no cold/warm latency comparison is made.
Daemon-warm/Pi-warm is `not_available`: neither the product path nor the frozen
runner reuses a persistent Pi process.

## 5. General Agent result (plan §11.2)

Six general families ran paired on held-out seeds, 30 pairs each, every task
judged by a mechanical oracle (exact number, exact text, sorted id set, or a
schedule validated as a dependency-respecting permutation). No model judged
anything.

| Family | `P` pure Pi | `O` OS Pi | delta |
|---|---:|---:|---:|
| `G1` multi-document research with conflicts | 30/30 | 30/30 | 0.0 pp |
| `G2` tabular analysis | 30/30 | 30/30 | 0.0 pp |
| `G3` constrained scheduling | 30/30 | 30/30 | 0.0 pp |
| `G4` procurement under hard filters | 30/30 | 30/30 | 0.0 pp |
| `G6` policy-constrained handling | 20/30 | 21/30 | +3.3 pp |
| `G9` security/privacy review | 16/30 | 14/30 | −6.7 pp |

Four families saturate at 100 % in both arms. `G6` and `G9` are the
discriminating ones, and they discriminate equally in both arms: the OS path
neither rescues nor damages the hard cases. Per-family deltas are secondary
endpoints; none would survive Holm correction across nine families, so they are
descriptive only.

Correctness, grounding and planning are captured by the oracles above.
Robustness is built into the corpus rather than measured separately: every
family cycles `basic`, `interleaved` and `adversarial` difficulty, and the
adversarial layer plants conflicting sources, unsourced claims and social
pressure to concede. Token and cost deltas are **`not_available`** for the `O`
arm, because the Extension does not surface per-request usage to the runner;
they are not estimated.

## 6. Software and operations result (plan §11.3)

| Family | `P` pure Pi | `O` OS Pi | delta |
|---|---:|---:|---:|
| `A1` failing-test root cause | 30/30 | 30/30 | 0.0 pp |
| `A4` operations incident diagnosis | 30/30 | 30/30 | 0.0 pp |
| `A5` ambiguity clarification | 24/30 | 27/30 | +10.0 pp |

`A5` is the largest per-family delta in the campaign and it favours the OS arm,
but with 30 pairs and no correction it is an observation, not a finding.

`A3` controlled repair, `A6` cross-session Memory/Skill reuse and `A7` external
mutation with unknown outcome remain **`not-run`**: they need workspace write,
bounded test execution, governed Memory/Skill consumption and an independent
verifier, none of which has a production caller (§14). `A5`'s OS-only authority
half — intent record, interpret, preview, admit — is separately measured and
fully functional (§9).

## 6a. Paired efficiency, and how it reads against the plan's thresholds

Completion first, efficiency second — reporting efficiency on completed pairs
alone would introduce survivorship bias, so both denominators are stated.

| Endpoint | `P` pure Pi | `O` OS Pi |
|---|---:|---:|
| oracle completion (headline) | 240/270 = 88.9 % | 242/270 = 89.6 % |
| matched pairs where both completed | 270/270 | 270/270 |
| wall median | 4367.2 ms | 6204.6 ms |
| wall MAD | 682.8 ms | 804.2 ms |
| wall p95 (N >= 100) | 23 190.1 ms | 17 408.4 ms |
| wall max | 123 482.0 ms | 133 013.5 ms |

Paired wall delta `O − P`: median **+1828.5 ms**, 95 % clustered bootstrap CI
**[1753.6, 1893.9] ms**, relative median **+44.2 %**.

A detail worth not smoothing over: the OS arm's **p95 is lower** than pure Pi's
(17.4 s against 23.2 s) even though its median is 1.8 s higher. Pure Pi carried
the heavier tail in this batch, driven by a small number of very slow Provider
generations that are upstream variance rather than an arm property. Outliers
were retained, not deleted.

**Against plan §7.2, which is record-only here** — those thresholds are blocking
only if the owner preregisters them as blocking, which did not happen:

| Threshold | Result |
|---|---|
| completion drop <= 2 pp | point estimate **+0.7 pp** (OS better); CI lower bound −2.22 pp sits just outside the 2 pp margin, so formal non-inferiority is **not demonstrated** at N = 270 and would need a larger N |
| C0 total latency overhead p50 <= 10 % | **not met** — observed +44.2 % |
| C0 total latency overhead p95 <= 20 % | not applicable in the usual direction: the OS arm's p95 is lower than `P`'s |
| token/cost overhead <= 5 % | **`not_available`** — no `O`-arm usage surface |
| safety failures not higher than `P`, OS hard counters zero | met (§13) |

**Against plan §7.3 benefit criteria:** no scenario-limited benefit is claimed.
The completion CI does not support an improvement, the relative improvement
threshold (>=10 % completion or >=20 % time/token reduction) is not met in the
OS arm's favour, and `not_available` token data alone would disqualify the
claim. The honest summary is parity in task success with a measurable time cost.

## 6b. Recovered post-P2-T11 `B6` replay

The guest retained **270 / 270 paired blocks, 540 / 540 started runs**, at exact
pushed revision `158e9276e49573db84aeb6ab55012d314368a76c`; both arms retained
the same `G3` block as a 180 s timeout, so 269 blocks completed in both arms.
The corpus and runner digests are byte-identical to B2, and all 270 task IDs,
families, difficulty strata, replicas, seed digests, prompt digests, and arm
orders match B2 line-for-line.

| Endpoint | B2 | B6 |
|---|---:|---:|
| `P` completion | 240/270 = 88.9 % | 240/270 = 88.9 % |
| `O` completion | 242/270 = 89.6 % | 243/270 = 90.0 % |
| completion delta `O − P` | +0.7 pp, CI [−2.22, +3.70] | +1.1 pp, CI [−1.48, +4.07] |
| paired wall delta | +1828.5 ms, CI [1753.6, 1893.9] | +1731.6 ms, CI [1626.1, 1836.9] |
| relative paired wall delta | +44.2 % | +42.7 % |

The existing analyzer's paired before/after task-cluster bootstrap gives
**+0.37 pp [−3.33, +4.07]** for the change in completion delta and
**−91.5 ms [−219.9, +31.2]** for the change in median overhead. Neither is
distinguishable from zero. P2-T11 corrected Provider readiness and a Skill
revoke route; it did not target this C0 request path. The result is therefore
replay stability evidence, not an optimization result. It predates P2-T10 and
must never be cited as P2-T10 evidence.

## 6c. Recovered nested-timing pilot

An added 20-run passive-observer pilot is retained as **partial**: 20 / 20
started runs remain, with 9 exact-marker successes and 11 failures. Eight runs
made one Provider request; twelve made four. The retained status sequences were
eight `(200)`, one `(503, 503, 503, 200)`, and eleven
`(503, 503, 503, 503)`. Total process wall time was 18 036.5 ms median
(MAD 709.8 ms, 4214.4–18 965.2 ms); the nine successful Provider responses
spent 1041.9 ms median inside the passive daemon-plus-Provider relay.

The intended pre-request / in-daemon / post-response decomposition is invalid:
the wrapper used epoch wall-clock nanoseconds while the observer used monotonic
`perf_counter_ns`, so cross-process subtraction yielded impossible values.
Pre-request and post-response timing are therefore `not_available`; the
instrument is not repaired or rerun in this measurement-only campaign. No raw
body, header value, key, or model output was retained.

## 7. Skill result (plan §11.4)

Installation, selection and benefit must stay three separate denominators, and
here two of the three are empty.

| Layer | Result |
|---|---|
| Authority negatives | **6 / 6 fail closed** with precise registered codes: `RESOURCE_SKILL_CONFLICT`, `RESOURCE_SKILL_ID_INVALID`, `RESOURCE_OBJECT_ID_INVALID`, `SHELL_CHANNEL_BINDING_MISMATCH`, `LOCAL_SESSION_UNAUTHORIZED`, `RESOURCE_MEMORY_REASON_REQUIRED` |
| Positive lifecycle writes | **0 / 60 admitted** — 50 returned a generic 409 conflict, 10 returned 400 from the shadowed revoke route |
| Lifecycle reads after those writes | 40 / 40 returned 404, consistent with nothing having been admitted |
| `skill/binding/revoke` | **unreachable** (route shadowing, §2 finding 2) |
| Forget non-resurrection / revoked reuse | **`not_applicable`** — nothing was admitted, so nothing could resurrect or be reused. This is explicitly *not* `observed_zero` |
| `S4` Agent Skill invocation, `S8` cross-task reuse | **`not-run`** — no governed Agent consumer |
| Per-operation latency | 0.78–0.91 ms p50 across all ten operations |

**Honest limit.** 0 / 60 does not prove the authority layer is broken. The deep
per-operation matrix is owned by the B08 Gate and merged CI regressions, which
this campaign does not re-run or override. What is established is narrower: from
the product's own public management surface, using payloads derived from that
surface's route contract, no Memory or Skill lifecycle write was admitted, and
the surface returns nothing that would let an operator discover why.

## 8. Tool result (plan §11.5)

Catalog and projection are **honest and correct**:

| Tool family | availability | execution readiness | risk |
|---|---|---|---|
| `workspace_read` | enabled | **execution_ready** | read_only |
| `process_check` | enabled | **execution_ready** | process_execution |
| `workspace_search` | enabled | registered_only | read_only |
| `workspace_write` | enabled | registered_only | workspace_mutation |
| `workspace_patch` | enabled | registered_only | workspace_mutation |
| `http_fetch_read_only` | enabled | registered_only | network_read |

6 registered, 2 execution-ready, 4 registered-only, every descriptor digest
present. This matches the plan's §1.1 capability table exactly and is the
P2-T09 readiness separation working on the real surface. It is also the direct
counter-example to finding 1: the product already knows how to distinguish
"registered" from "actually callable" — the Provider readiness component simply
does not do it.

Dynamic lifecycle (`enable`, `disable`, `quarantine`, `discover`) is
**`not_available`** on the public surface: all four fall through to the generic
authority handler with `RESOURCE_OBJECT_ID_REQUIRED`. Real governed invocation
(`T4`–`T9`) is **`not-run`**: no production caller. Tool selection precision,
argument validity, unnecessary-call rate and Tool-assisted task delta are all
**`not-run`** because no Agent task ran.

## 9. Authority truth: admission is not completion (plan §11.8)

| Fact | Value |
|---|---:|
| Task admission runs started / retained | 30 / 30 |
| **Admitted** | **30 / 30** |
| Executed mutations | 0 |
| Independent acceptance | **0 / 30** |
| **Verified completions** | **0** |

| Stage | p50 | p95 (descriptive) | MAD |
|---|---:|---:|---:|
| session mint | 3.33 ms | 7.85 ms | 0.30 ms |
| `intent.record` | 14.07 ms | 25.20 ms | 1.96 ms |
| `intent.interpret` | 16.29 ms | 21.89 ms | 2.11 ms |
| `task.preview` | 8.32 ms | 13.66 ms | 0.48 ms |
| `task.admit` | 25.92 ms | 30.35 ms | 1.56 ms |
| **admission total** | **68.78 ms** | 86.07 ms | 3.78 ms |

The front half of the Task contract is complete, fast and stable. The back half
does not exist as a product path: no scheduler bootstrap, no Context resolution,
no Tool dispatch, no Effect, no independent verifier. A 100 % admission rate
must never be reported as a 100 % completion rate.

Against P9-T04 (10 runs, 8 admitted, 68.17 ms p50) the p50 is unchanged and the
two rejections are gone, consistent with P9-T04's note that they came from a
Task draft that did not match the generated `Budget` binding rather than from
the product.

## 10. Reliability (plan §11.9)

| Property | Result |
|---|---|
| Orphan process after `daemon stop` | **0 / 10 cycles** |
| Stale `daemon.lock` after stop | **0 / 10** |
| Stale `daemon-endpoint.json` after stop | **0 / 10** |
| Health while daemon down | connection refused, 10 / 10, 0.23 ms p50 |
| Restart to serving | `stop` 9.81 ms, `start` 105.06 ms, residual wait 1.73 ms |
| RSS across 10 restart cycles | 8828–9084 kB, no trend |
| FD / threads across 10 cycles | constant 9 / constant 1 |
| Selected-model mismatch | 10 / 10 denied before dispatch, **zero Provider dispatches** |
| Phase-3 selected-model mismatch | 10 / 10 denied before dispatch at 30.72 ms median, zero Provider timing/usage |
| Fail-closed Provider path | 80 / 80 refused, no plaintext fallback |
| Client deadline 120 ms | 10 / 10 retained; max 125.35 ms, slightly past the deadline |
| Phase-3 client deadline 120 ms | 10 / 10 retained; `PI_EXTENSION_DAEMON_UNREACHABLE` / `outcome_unknown`; 122.27 ms median, 123.38 ms max |
| Client deadline 20 ms | **`not-run`** — the frozen runner applies the deadline to its own preflight and aborted before sampling |
| Phase-3 daemon unavailable | 10 / 10 `O` runs failed closed with process exit 1 in 1747.0 ms median; retained from the invalidly bound UJ2 attempt, not double-counted or treated as cold-start evidence |
| Phase-3 broker unavailable | 10 / 10 `P` and 10 / 10 `O` runs timed out at 180 s; broker-only attribution unavailable |
| Phase-3 Pi process kill | 10 / 10 alive at 2 s, SIGKILL exit 137, 0 orphan Pi processes |
| Pi kill, upstream timeout, rate limit, size bound, stale epoch, unknown outcome | **`not-run`** — need a credential or a reachable mutation path |

Restart and cleanup behaviour is the strongest reliability result in this
campaign: ten full cycles with no residue of any kind.

## 11. Capacity (plan §11.10)

832 requests across nine profiles, **zero errors, zero refusals, zero dropped
connections**, including 33 concurrent connections.

| Concurrency | health p50 | health p95 | throughput |
|---:|---:|---:|---:|
| 1 | 0.70 ms | 0.91 ms | 1116.6 rps |
| 8 | 6.28 ms | 10.85 ms | 1146.4 rps |
| 16 | 7.61 ms | 17.51 ms | 1293.0 rps |
| 17 (overload) | 7.40 ms | 19.70 ms | 1390.3 rps |
| 33 (overload) | 6.36 ms | 18.68 ms | 1289.9 rps |
| 1 (post-overload) | 0.51 ms | 0.64 ms | 1657.0 rps |

The daemon runs on **one thread**, read directly from `/proc` and constant
throughout. Throughput is flat near 1.1–1.4 k local reads per second regardless
of offered concurrency, while p50 grows almost linearly with it. That is
queueing behind a single-threaded server, not resource exhaustion. Degradation
is graceful and recovery is immediate: health returned to sub-1.5 ms within
0.74 ms of the first post-overload probe, and the following serial profile was
faster than the pre-overload baseline. RSS moved ~600 kB across the whole
sequence with FD constant at 9.

Agent throughput, Provider dispatch counts under concurrency and mixed Pi
workload were **`not-run` in phase 1**. Phase 3 executed the bounded subset that
the retained instruments support:

| Mixed profile | Agent result | health median | status median | doctor median |
|---|---|---:|---:|---:|
| c1 | 1/1 pair, both arms passed; P/O 5076.6 / 7363.8 ms | 0.51 ms | 1815.0 ms | 1824.5 ms |
| c4 | 4/4 pairs, all arms passed; P/O medians 8613.3 / 14 995.8 ms | 2.68 ms | 1960.5 ms | 1812.5 ms |
| post-load | no Agent | 0.65 ms | 1778.0 ms | 1790.0 ms |

All 300 local operations succeeded, and all 10 Agent runs completed and passed
their mechanical oracle. Health shows bounded queueing at c4 and returns to
baseline afterwards. Status/doctor are already ~1.8 s after load is gone: that
is the post-P2-T11 cost of resolving the Provider secret during readiness, not
an Agent-concurrency effect. N=1/4 supports no tail, throughput, or
non-inferiority claim. Six-resource get/watch and Task-watch were not run in the
mixed profile because no retained driver composes them with live Agent load.

## 11a. The Pi route, as far as it can be observed (added cell `O-LAUNCH`)

The preregistered `O` first-response cell is `not-run`. This added cell, with a
denominator of 10 fixed before it started, measures the part that remains
observable without a credential: Pi launching, loading the Extension, reaching
the daemon, and surfacing the refusal.

Pi provenance was verified as `@earendil-works/pi-coding-agent` `0.81.1` with
integrity `sha512-r6ovAsZO…kN/P8A==`, byte-identical to the pin P9-T04 recorded.

| Measurement | N | p50 | range |
|---|---:|---:|---|
| bare `node -e ''` (runtime floor) | 10 | 44.5 ms | 39–80 ms |
| `pi --version` (Node + Pi init; no Extension, daemon or Provider) | 10 | **1682.5 ms** | 1518–1814 ms |
| Pi route to refusal (Extension + daemon + failing Provider) | 10 | **18 067 ms** | 17 941–18 500 ms |
| daemon client to the same refusal (§4) | 80 | ~110 ms | — |

Outcome was `pi_nonzero_exit` in 10 / 10 with the expected marker never observed
and non-empty output every time.

Two things are worth stating carefully.

**Pi start is expensive before anything happens.** `pi --version` does no work
at all and still costs 1.68 s — 37× the bare Node floor. That cost is paid on
every invocation of the current launch model.

**The remaining ~16.4 s is unattributed, and stays that way.** There is no
nested timing inside a single run, so plan §5.2 forbids naming the interval
between 1.68 s and 18.07 s as spawn, Extension load or governance cost.
Candidate contributors are Extension load, daemon discovery, and whatever Pi
itself does when a provider call fails. What *can* be said is that the daemon is
not the cause: its own refusal costs about 110 ms, well under 1 % of the total.

**This is time-to-failure on a broken Provider, not first-response latency.** It
is not comparable to P9-T04's 4625 ms first response, which was measured against
a working Provider, and it must not be quoted as a regression against it.

## 12. Long-run behaviour (plan §11.11)

One hour, 60 one-minute blocks, **1620 started, 1620 retained, zero non-200**.

| Slope fact | First minute | Last minute | Delta over 1 h |
|---|---:|---:|---:|
| daemon RSS | 9820 kB | 9860 kB | **+40 kB** |
| FD count | 9 | 9 | 0 |
| threads | 1 | 1 | 0 |
| `authority.sqlite` | 1 044 480 B | 1 044 480 B | **0** |
| `authority.sqlite-wal` | 0 B | 0 B | 0 |
| process `write_bytes` | 1 347 584 | 1 347 584 | **0** |
| per-minute p50 | 0.677 ms | 1.288 ms | +0.61 ms |
| worst single sample | — | — | 13.16 ms |

The closing hourly cold restart was clean: no orphan process, no stale lock, no
stale endpoint file, daemon ready again.

**No leak signature of any kind.** Memory, descriptors, threads, database size
and WAL are flat, and the daemon issued literally zero additional disk writes
across an hour of read traffic. The sub-millisecond p50 drift is non-monotonic
across the run and is reported as observed variation, not a trend.

Paired Provider soak blocks are **`not-run`** — the plan schedules a paired C0
task block every 5 minutes and no Provider arm exists.

`B5` 8 h and 24 h are **`not-run`**: the plan promotes to 8 h only after a clean
1 h exit, and 24 h is conditional on an unresolved 8 h slope plus owner budget.

That sentence is the phase-2 publication-time disposition. Phase 3 then ran the
eligible 8 h cell to its complete denominator at
`158e9276e49573db84aeb6ab55012d314368a76c` — **pass**:

| 8 h fact | Result |
|---|---|
| one-minute blocks | 480 / 480 retained |
| local operations (health, projections, watch, readiness) | **12 960 / 12 960, 0 non-OK** |
| per-minute p50 | 2.913 ms median (4.281 first, 3.07 last, 5.705 worst) |
| worst single sample | 2338.6 ms — the known ~1.8 s post-P2-T11 readiness cost under a concurrent paired block, not a new anomaly |
| hourly cold restarts | **8 / 8 clean** (0 orphans, no stale lock/endpoint, fallback never used, 159 ms median cycle) |
| RSS across 8 daemon segments | fresh ~9.2–10.5 MB → hour-end 10.9–14.7 MB; within-hour fill saturates ~14 MB, **no cross-segment trend** |
| `authority.sqlite` / WAL / per-segment write bytes | flat / 0 / flat |
| paired Provider blocks every 10 min | 48 / 48 blocks, **96 / 96 runs**, held-out seeds 30–35, zero transport/timeout/process failures |
| paired oracle completion | `P` 43/48 (89.6 %) vs `O` 44/48 (91.7 %); all 9 failures wrong-answer in `A5`/`G6`/`G9` |
| paired wall medians (descriptive; per-block invocation fixed arm order `O→P`) | `P` 4424.3 ms, `O` 6477.9 ms, delta +2217.7 ms |

`B5` 24 h stays **`not-run`**: its trigger is an unresolved 8 h slope, and the
only observed growth is a within-hour RSS fill that saturates and resets at
each hourly restart with flat database, WAL, write, FD, and thread envelopes.

## 13. Safety hard conditions (plan §11.12, §6.8)

Every counter carries its evidence disposition. A structure default or a
hard-coded zero never produces `observed_zero`.

| Counter | Result | Disposition | Basis |
|---|---:|---|---|
| Provider secret exposure | 0 | `not_applicable` for the Provider key (no such item existed on the guest at any point) + `observed_zero` for the local bootstrap secret | the campaign entered, read, copied and hashed no credential; the bootstrap secret was held in memory only, kept at mode `600`, and never printed, logged or passed in argv; Secret Service was queried by object path only; a key-shaped scan of the whole campaign root found **0 hits in `evidence/` or `runtime/`**, and the 10 repository-wide hits were triaged by character class to pure-alphabetic identifiers containing no digits (vendored OpenTelemetry constants and a runner filename) |
| unauthorized / stale Context exposure | — | **`not_available`** | no public Context observation surface |
| duplicate external Effect | — | `not_applicable` | no mutation path was reachable |
| false completion | — | **`not_available`** for the internal fact; publicly, 0 completion events were observed in 30 admission runs | plan §5.4 forbids inferring internal state from silence |
| stale-epoch commit | — | `not_applicable` | no epoch-bearing commit attempted |
| unreconciled Effect after bounded window | — | `not_applicable` | no Effect created |
| completion without independent acceptance | 0 | `observed_zero` over 30 runs | no run completed at all, and none was accepted; both facts recorded |
| scenario boundary violation | 0 | `observed_zero` | see §15 |

No counter is non-zero, so no abort condition fired. This is not a safety pass
for the product: most counters are `not_applicable` precisely because the paths
they guard were never reachable.

## 14. Capability truth (plan §11.7)

The register the plan requires, retained in full so the campaign cannot be read
as covering only what happened to work.

| Capability | Current disposition | Evidence in this campaign |
|---|---|---|
| Prompt-contained Agent task, OS arm | **working** | 242/270 held-out tasks completed |
| Prompt-contained Agent task, pure-Pi arm | **working** | 240/270, via the campaign broker |
| Paired `O vs P` comparison | **executed** | 270 held-out pairs, 540 runs, all retained |
| Pi launch, Extension load, daemon discovery | **working** | 30/30 first responses at 4625 ms p50 |
| Pi first response on a working Provider | **working** | 4625 ms p50 / 5006 ms p95 |
| Prompt-contained Agent task, pure-Pi arm | `not-run` | no approved broker; building one is out of bounds |
| Paired `O vs P` comparison | `not-run` | no paired runner |
| Workspace read/search by an Agent | **unreachable** | `workspace_search` is `registered_only` |
| Workspace write/patch/check | **unreachable** | `registered_only`; no production caller |
| Task scheduler bootstrap after admission | **unreachable** | 30/30 admitted, 0 executed |
| Independent verification / acceptance | **unreachable** | 0/30 accepted |
| Effect persist–dispatch–reconcile | **unreachable** | no production caller |
| Memory remember / review / forget | public writes rejected | 0/40 admitted, generic 409 |
| Skill import / bind / supersede | public writes rejected | 0/40 admitted, generic 409 |
| Skill binding revoke | **unreachable** | route shadowed by `skill/bind` |
| Tool enable / disable / quarantine | `not_available` | `RESOURCE_OBJECT_ID_REQUIRED` |
| Context authorization and cache correctness | `not_available` | no public observation surface |
| Backup / restore | `not_available` | no user CLI or archive wiring |
| Web UI, multi-Agent | unavailable / deferred | unchanged |
| Six-resource projection and bounded replay | **working** | 380 calls, all 200, sub-millisecond |
| Task admission chain | **working** | 30/30, 68.78 ms p50 |
| Channel isolation | **working** | 401 / 403 / 200 as designed |
| Daemon lifecycle and cleanup | **working** | 10/10 clean cycles |

## 15. Scenario boundary compliance

The preregistered guest allowlist was honoured exactly. No libvirt snapshot was
created, reverted or deleted; `b01-platform-qualified-baseline` is untouched. No
guest power-state change occurred. The residual P9-T04 root and its running
daemon were read but never modified or stopped. No `apt`, systemd, user or
network change was made. No credential was entered, read, copied, hashed or
relocated. All campaign state is confined to `~/perfeval002`.

Reconciled at cleanup: the residual P9-T04 daemon still runs unchanged as
pid 11176, its config mtimes are unchanged at 12:51 and 13:37 — hours before
this campaign began — and Secret Service still holds no
`application=cognitiveos-personal` entry, because the campaign created none.
After cleanup there were zero campaign daemon processes, zero campaign Pi
processes, no stale lock, no stale endpoint file and no listener on 48282.

The guest was **not** in its pristine qualified baseline when the campaign
started — it carried P9-T04 residue including an idle second daemon — so no
clean-install or B01-class claim is available from this campaign, and all
resource figures are stated against that background load.

**Phase-2 reconciliation.** The credential was imported through the product's
own stdin path and the campaign-created SecretStore entry was cleared at
cleanup (plan §8.2): a post-cleanup Secret Service search for
`application=cognitiveos-personal` returns empty. The broker was stopped, which
drops the only in-memory copy of the key, and no listener remains on 48383. A
key-shaped scan of the evidence directory, the runtime root, both arm homes and
all three instrument files returned **0 hits each**, and the `P` arm's
`models.json` contains no `sk-` string — it holds only the non-secret
placeholder token. The owner's source file is byte- and mtime-unchanged. The
P9-T04 daemon still runs as pid 11176 with unchanged config mtimes, no snapshot
was touched, and no guest power-state change occurred. **Scenario boundary
violations: 0.**

**Phase-3 reconciliation (2026-08-13).** Phase-3 cells ran under the same
allowlist: campaign-owned daemon, broker, observer, and soak driver only, with
no snapshot, power, system, user, or network change in any session. At closure
the campaign daemon was stopped through the product path (0 processes, no stale
lock or endpoint, no listener on 48282/48383/48484), the campaign-created
SecretStore entry at `…/collection/login/8` was cleared and `SearchItems` now
returns `[]`, and a digit-bearing key-shaped scan of the entire campaign root
found **0 hits across 18 972 files**. The P9-T04 residue still runs unchanged
as pid 11176 on 48181 with unmodified config mtimes. The one phase-3
orchestration mistake is retained, not hidden: ten `UJ2` cold-start attempts
omitted the required `--bind` and are reclassified `instrument_error`
(§4a). **Scenario boundary violations: 0.**

**Evidence retention.** Raw payloads are retained rather than deleted, since a
digest without a retrievable payload cannot support later review (plan §8.3).
Locator `b01guest:~/perfeval002/evidence/` on `B01-Desktop-Linux-002` — now
**159 files, ~1.1 MiB** including all phase-3 and B5 8 h records — with
per-file SHA-256 recorded in the preregistration and the retained
`digests.sha256` / `b5-soak-8h-digests.sha256` manifests, held until the
independent verifier disposition changes from `not_reviewed`.

## 16. Optimization priority, ranked by evidence (plan §11.13, §12)

Ranked by strength of evidence produced here multiplied by user impact, not by
architectural intuition.

**Priority 1 — make Provider readiness reflect resolvability.**
Evidence: 80/80 requests failed with `PERSONAL_PROVIDER_SECRET_UNAVAILABLE`
while `status`, `doctor` and `overall` all said `ready`; the referenced key item
was proven absent. Impact: the product's primary self-diagnostic actively
misleads a user whose Provider is broken, and this is the single most likely
first-run failure. The fix pattern already exists in-product: the Tool
projection distinguishes `registered_only` from `execution_ready`. Provider
readiness should resolve or at least existence-check its `secret_ref`, or
expose a distinct `configured_but_unresolvable` state.

**Priority 2 — fix the `skill/binding/revoke` route shadowing.**
Evidence: differential probe proving the revoke path executes the bind handler.
Impact: an authority revocation verb is unreachable from the public surface.
Revocation failing silently into a different handler is a governance-shaped bug,
not a cosmetic routing issue. Order the dispatcher longest-prefix-first or match
exact paths.

**Priority 3 — take the Secret Service probe off the readiness hot path.**
Evidence: 57.5 ms median of a ~65 ms evaluation, N = 30, every other component
0 ms; the same probe also costs ~57 ms on every fail-closed Provider refusal.
Impact: `cognitive status` and `doctor` could plausibly be ~7 ms instead of
~70 ms, a tenfold improvement in the most frequently run commands. Cache with a
short TTL, run components concurrently, or bound the probe — but note this
interacts with Priority 1: the probe should become *more* informative, not
merely faster.

**Priority 4 — give the authority transition boundary a discriminating error
vocabulary.** Evidence: 50 of the 60 attempted writes returned one generic 409
per family, identical for rich and minimal payloads, while validation errors
returned six distinct precise codes. Impact: an operator cannot self-diagnose a rejected
Memory or Skill write, which is exactly the situation this campaign hit and
could not resolve from outside.

**Priority 1b — find and remove the ~1.8 s the OS path adds per task.** Evidence:
270 held-out paired seeds, median +1828.5 ms with a 95 % CI of only ±70 ms, at
identical output size and statistically indistinguishable completion. This is
the single largest user-visible cost CognitiveOS currently imposes, and it is
now well localized: the daemon accounts for 128 ms of it, Extension load for
4.5 ms, streaming mode for nothing, and the broker for 0.5 ms. The next step is
**not** an optimization, it is one measurement — nested per-stage timing inside a
single Pi run through the Extension provider. Everything else is guessing, and
this campaign has already eliminated the four most plausible guesses.

**Priority 5 — the Pi launch model is expensive before it does anything.**
Evidence: `pi --version`, which performs no work, costs 1682.5 ms p50 against a
44.5 ms bare Node floor (N = 10 each, tight ranges). Every invocation of the
current launch model pays that. This corroborates P9-T04's finding that Pi
launch, not governance or the Provider, dominates what a user experiences —
with a cleaner baseline than P9-T04 had. The further ~16.4 s observed on the
route-to-failure path stays **unattributed** and must not be used to size a
fix; what would size it is nested per-stage timing inside a single Pi run,
which no current instrument produces. A persistent or reusable Pi process is
the obvious direction, but it needs that timing first.

**Priority 6 — close the `scheduler → Tool → verifier` production chain.**
Evidence: 30/30 admitted with 0 executed and 0 verified; 4 of 6 Tool families
`registered_only`. Impact: this is the gap that makes every `C1`/`C2` capability
unreachable and keeps the entire Agent-benefit question unanswerable. It is
ranked below the four correctness items because it is a large programme, not a
defect, and the plan already registers it.

**Priority 7 — keep the paired evaluation instruments, and extend them.** The
broker, corpus and paired runner now exist and are digest-pinned, so the
`O vs P` question is answerable on demand and `B6` replay is available for any
future optimization. Two extensions would pay for themselves: per-request usage
exposure on the `O` arm, which would turn token and cost from `not_available`
into a measured endpoint, and nested per-stage timing, which Priority 1b needs.
Note also that the owner's Provider key currently sits in plaintext at
`~/下载/deepseek.txt` on the guest — the campaign never modified it, but a key in
a Downloads folder is a wider exposure than anything the product does with it.

**Priority 8 — the daemon's single thread.** Evidence: constant `Threads: 1`,
flat 1.1–1.4 k rps, latency scaling linearly with concurrency, zero errors at
33 connections, and a 1 h soak with a +40 kB RSS delta and zero extra writes.
Deliberately ranked last: nothing observed here shows a user hitting this
ceiling, degradation is graceful, and recovery is instant. Optimising it now
would be architectural intuition rather than evidence.

**Measurement-side, not product:** the frozen route runner classifies
`PERSONAL_PROVIDER_SECRET_UNAVAILABLE` as `outcome_unknown` because that code is
absent from its `DENIED_BEFORE_DISPATCH_CODES` set, and it applies the client
deadline to its own preflight so that short-deadline cells abort before
sampling. Both would distort future denominators. Neither was changed during
this campaign.

## 17. Non-claims

This campaign does not pass or contribute to B01, B01-W, B02, B03, B04, B05,
B06, B07, B08, B09, B10, B11, B12, or `GMVP-LINUX`. It authorizes no release, no
Profile and no Windows claim. It does not promote local, fixture or ordinary CI
evidence. The claim level is `hypothesis`, the independent verifier disposition
is `not_reviewed`, and `not-run` remains `not-run` except where the append-only
phase-3 recovery explicitly supersedes the earlier `B6` disposition with
retained evidence.

It **does** now contain a real paired comparison against pure Pi, but that
comparison is bounded in four ways that must travel with any quotation of it:
it covers `C0` prompt-contained tasks only; it uses one Provider and one model
snapshot on one 2-vCPU guest on one day; its oracles are mechanical and
therefore measure task correctness rather than open-ended answer quality; and no
Agent **benefit** is claimed in either direction — the result is parity in task
success with a measured time cost. Tasks requiring workspace tools, mutation,
Memory/Skill reuse or independent verified completion remain unreachable on the
OS arm and were not measured at all.
