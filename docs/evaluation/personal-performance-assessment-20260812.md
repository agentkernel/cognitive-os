# CognitiveOS Personal performance and capability assessment — 2026-08-12

- Campaign: `PERSONAL-PERF-EVAL-002` (owner-directed evaluation campaign,
  [Operating Model §2.5](../governance/DEVELOPMENT-OPERATING-MODEL.md))
- Execution contract: [personal-performance-benchmark-execution-plan.md](personal-performance-benchmark-execution-plan.md) v1.1
- Freeze and per-cell execution record: [20260812-personal-perf-eval-002-preregistration.md](../checkpoints/20260812-personal-perf-eval-002-preregistration.md)
- Source revision: `4cbec8470bc7a19f23f978e8754ed20133122eb1` (pushed, clean)
- Environment: `B01-DESKTOP-002`, guest `B01-Desktop-Linux-002`
- Claim level: **`hypothesis` / non-claim**
- Independent verifier disposition: **`not_reviewed`**
- Agent benefit claimed: **no**
- Document status: final campaign report

## 1. What this report says, and what it cannot say

It reports measured latencies, counts, outcome classes, resource behaviour and
authority outcomes for CognitiveOS Personal, on a named guest, at one exact
pushed revision, with every started sample retained.

It does **not** establish a Gate, a release, a Profile, a B01 or B01-W outcome,
a governance non-inferiority result, or any Agent benefit. Most importantly:

> **This campaign contains no Agent comparison at all.** The pure-Pi arm never
> ran, so nothing here is a measurement of "CognitiveOS versus plain Pi". Any
> number below that is read as an OS overhead or an OS benefit is being read
> wrongly.

Two independent reasons, both decided before execution and neither negotiable
mid-campaign:

1. **No paired instrument exists.** Plan §2.2 requires an approved pure-Pi
   credential path and plan §2.5 requires a preregistered, digest-frozen paired
   runner. Neither exists in the repository. Operating Model §2.5 states that a
   missing capability, runner, or credential path is recorded `not-run` and
   **never implemented mid-campaign**, so building them was not available to
   this campaign.
2. **No Provider credential exists on the target guest.** Determined by
   measurement, not assumption — see §2.

The consequence is that the plan's entire confirmatory composition — 270 paired
task-seeds across `G1/G2/G3/G4/G6/G9-C0`, `A1-C0`, `A4-C0`, `A5-C0`, up to 1620
runs — is `not-run`. Non-inferiority thresholds (plan §7.2) and benefit criteria
(plan §7.3) were **not evaluated**: neither met nor failed.

What the campaign did instead is measure the product's own surfaces exhaustively
on the executable side, and it found four concrete defects and a clear
performance profile. That is the value here.

## 2. Headline findings

**1. The product reports `ready` on a Provider path that cannot work.**
All 80 Provider requests failed closed with `PERSONAL_PROVIDER_SECRET_UNAVAILABLE`,
while `cognitive status` and `cognitive doctor` reported `secret: ready`,
`provider: ready`, `overall: ready`. A Secret Service search (object paths only,
no value ever requested) showed the referenced key item is **absent**: the
P9-T04 cleanup removed its SecretStore entry while leaving the `provider.json`
that references it. The `provider` readiness component's source is
`filesystem:provider-config` and it asserts only `secret_ref_present: true`; the
`secret` component probes backend availability, not the reference. A dangling
reference therefore reads as ready and fails on every real request.

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

**4. Everything else about the local daemon is fast, clean and honest.**
In-daemon reads answer in under 1 ms; 832 concurrent requests through 33
connections produced zero errors; ten stop/start cycles left zero orphans, zero
stale locks and zero stale endpoint files with flat RSS, FD and thread counts;
all six authority negative controls failed closed with precise registered codes;
and the Tool projection honestly declares 2 of 6 families execution-ready.

## 3. Cell disposition

| Cell | Instrument | Started / retained | Disposition |
|---|---|---:|---|
| `B0` staging and qualification | build + digest verify + daemon bring-up | 66 / 66 files | **pass** |
| `D1` Provider proxy marker | frozen route runner | 30 / 30 | partial (fail-closed only) |
| `D2` warm repeated route | frozen route runner | 50 / 50 | partial (fail-closed only) |
| `UJ4` / `O1` Task admission | frozen T1 runner | 30 / 30 | **pass** |
| `UJ3` daily operations | public surface | 730 / 730 | **pass** |
| `UJ3b` readiness attribution | public surface | 180 / 180 | **pass** |
| `T-GOV` Tool projection + lifecycle | public surface | 1 dump + 4 probes | partial |
| `MS-AUTH` Memory/Skill authority | public surface | 46 / 46 | partial |
| `B3` faults, restart, cleanup | frozen runner + public surface | 40 / 40 | partial |
| `B4` concurrency and overload | public surface | 832 / 832 | **pass** |
| `B5` 1 h soak | public surface | see §12 | see §12 |
| `O` Pi route, `B1`, `B2`, `P` arm | — | 0 | **not-run** (no credential, no broker, no paired runner) |
| `A3`/`A6`/`A7`, `G*-C1/C2`, `A1-C1` | — | 0 | **not-run** (no OS product path) |
| `S4`/`S8`, `T4`–`T9`, `O4`–`O6` | — | 0 | **not-run** (no governed consumer / production caller) |
| `O2`/`O3` Context, `O14` backup/restore | — | 0 | **`not_available`** (no public observation surface) |
| `B5` 8 h / 24 h, `B6` replay | — | 0 | **not-run** (gated on prior exits) |

Total retained samples: 1954 across the executed cells, plus the soak in §12.

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

## 5. General Agent result (plan §11.2)

**`not-run` in full.** `G1` multi-document research, `G2` tabular analysis,
`G3` constrained planning and replanning, `G4` procurement comparison, `G6`
policy-constrained communication and `G9` security/privacy review — none
executed, in either arm. No correctness, grounding, planning, robustness, time,
token or cost figure exists for any general Agent task.

## 6. Software and operations result (plan §11.3)

**`not-run` in full.** `A1` root-cause and impact analysis, `A4` operations
incident diagnosis and `A5` ambiguity clarification did not execute; `A3`
controlled repair, `A6` cross-session Memory/Skill reuse and `A7` external
mutation with unknown outcome remain registered as unreachable on the OS arm.

The one adjacent fact this campaign does contribute is that `A5`'s OS-only half
— intent record, interpret, preview, admit — is fully functional and fast
(§8), even though its conversational half never ran.

## 7. Skill and Memory result (plan §11.4)

Installation, selection and benefit must stay three separate denominators, and
here two of the three are empty.

| Layer | Result |
|---|---|
| Authority negatives | **6 / 6 fail closed** with precise registered codes: `RESOURCE_SKILL_CONFLICT`, `RESOURCE_SKILL_ID_INVALID`, `RESOURCE_OBJECT_ID_INVALID`, `SHELL_CHANNEL_BINDING_MISMATCH`, `LOCAL_SESSION_UNAUTHORIZED`, `RESOURCE_MEMORY_REASON_REQUIRED` |
| Positive lifecycle writes | **0 / 40 admitted** — every Memory and Skill write returned a generic 409 |
| `skill/binding/revoke` | **unreachable** (route shadowing, §2 finding 2) |
| Forget non-resurrection / revoked reuse | **`not_applicable`** — nothing was admitted, so nothing could resurrect or be reused. This is explicitly *not* `observed_zero` |
| `S4` Agent Skill invocation, `S8` cross-task reuse | **`not-run`** — no governed Agent consumer |
| Per-operation latency | 0.78–0.91 ms p50 across all ten operations |

**Honest limit.** 0 / 40 does not prove the authority layer is broken. The deep
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
| Fail-closed Provider path | 80 / 80 refused, no plaintext fallback |
| Client deadline 120 ms | 10 / 10 retained; max 125.35 ms, slightly past the deadline |
| Client deadline 20 ms | **`not-run`** — the frozen runner applies the deadline to its own preflight and aborted before sampling |
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

Agent throughput, Provider dispatch counts under concurrency and mixed
Pi workload are **`not-run`**.

## 12. Long-run behaviour (plan §11.11)

See [§12 of this section](#12-long-run-behaviour-plan-1111) — filled in from the
`B5` 1 h soak below.

<!-- SOAK-RESULT -->

`B5` 8 h and 24 h are **`not-run`**: the plan promotes to 8 h only after a clean
1 h exit, and 24 h is conditional on an unresolved 8 h slope plus owner budget.

## 13. Safety hard conditions (plan §11.12, §6.8)

Every counter carries its evidence disposition. A structure default or a
hard-coded zero never produces `observed_zero`.

| Counter | Result | Disposition | Basis |
|---|---:|---|---|
| Provider secret exposure | 0 | `not_applicable` for the Provider key (no such item existed on the guest at any point) + `observed_zero` for the local bootstrap secret | the campaign entered, read, copied and hashed no credential; the bootstrap secret was held in memory only and never printed, logged or passed in argv; Secret Service was queried by object path only |
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
| Prompt-contained Agent task, OS arm | `not-run` | no Provider credential |
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

The guest was **not** in its pristine qualified baseline when the campaign
started — it carried P9-T04 residue including an idle second daemon — so no
clean-install or B01-class claim is available from this campaign, and all
resource figures are stated against that background load.

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
vocabulary.** Evidence: 40/40 writes returned one generic 409 per family,
identical for rich and minimal payloads, while validation errors returned six
distinct precise codes. Impact: an operator cannot self-diagnose a rejected
Memory or Skill write, which is exactly the situation this campaign hit and
could not resolve from outside.

**Priority 5 — close the `scheduler → Tool → verifier` production chain.**
Evidence: 30/30 admitted with 0 executed and 0 verified; 4 of 6 Tool families
`registered_only`. Impact: this is the gap that makes every `C1`/`C2` capability
unreachable and keeps the entire Agent-benefit question unanswerable. It is
ranked below the four correctness items because it is a large programme, not a
defect, and the plan already registers it.

**Priority 6 — build the paired evaluation instruments, as an owner decision.**
Evidence: the whole of §5 and §6 is `not-run`. Nothing about CognitiveOS's Agent
value can be measured until an approved pure-Pi credential path, a frozen paired
runner, and a Provider credential on the target guest all exist. These are
prerequisites for evaluation, not product features, and creating them is outside
a measurement-only campaign.

**Priority 7 — the daemon's single thread.** Evidence: constant `Threads: 1`,
flat 1.1–1.4 k rps, latency scaling linearly with concurrency, zero errors at
33 connections. Deliberately ranked last: nothing observed here shows a user
hitting this ceiling, degradation is graceful, and recovery is instant.
Optimising it now would be architectural intuition rather than evidence.

**Measurement-side, not product:** the frozen route runner classifies
`PERSONAL_PROVIDER_SECRET_UNAVAILABLE` as `outcome_unknown` because that code is
absent from its `DENIED_BEFORE_DISPATCH_CODES` set, and it applies the client
deadline to its own preflight so that short-deadline cells abort before
sampling. Both would distort future denominators. Neither was changed during
this campaign.

## 17. Non-claims

This campaign does not pass or contribute to B01, B01-W, B02, B03, B04, B05,
B06, B07, B08, B09, B10, B11, B12, or `GMVP-LINUX`. It authorizes no release, no
Profile, no Windows claim, and no generalized Agent-benefit statement. It makes
no comparison against pure Pi. It does not promote local, fixture or ordinary CI
evidence. The claim level is `hypothesis`, the independent verifier disposition
is `not_reviewed`, and `not-run` remains `not-run`.
