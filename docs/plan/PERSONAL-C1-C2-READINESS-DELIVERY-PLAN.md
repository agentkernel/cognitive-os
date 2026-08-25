# Personal C1/C2 Paired Benchmark Readiness Delivery Plan

- Status: **packages 1–14 delivered / P9-T08 merged**; **EVAL-012 closed**;
  **EVAL-013 closed** (freeze branch; B0 C1/C2a pass, B1/B2 not-run);
  **P2-T38 done**; **P9-T09 done**; **P9-T10 done**; **P9-T11 done**;
  **P9-T12 done** (live paired executor, PR
  [#252](https://github.com/agentkernel/cognitive-os/pull/252) at
  `main@39cf8019`). No evaluation campaign is active. Next is a new EVAL
  preregistration.
- Named implementation vehicle: **P9-T12** (closed). Live paired executor is
  merged. Remaining mutex before B1/B2 is a new owner-activated EVAL.
- Product pin: `main@39cf80193a974ad765d71ebcd87e76d3538967a8` after P9-T12.
- Definition of done (this programme): after packages 1–14 execute, an owner
  can activate a **new** preregistered EVAL and immediately start **B0
  qualification on `B01-Desktop-Linux-002`** for paired C1+C2. B1/B2 can use
  the P9-T12 live paired executor. Completing packages 15–17
  is **完整真机实测**, not a substitute for packages 6–14.
- Last reconciled: 2026-08-20
- Claim ceiling: `hypothesis` / non-claim. This programme never promotes Gate,
  release, Profile, B01, or Agent-benefit.

## 1. Authority and use

This document is a durable navigation aid for the owner-authorized C1/C2
readiness programme. It does **not** own task acceptance, task status, leases,
campaign activation, benchmark denominators, or performance claims:

- Formal Personal tasks, dependencies, and Delivery Slices remain owned by
  [PERSONAL-DEVELOPMENT-PLAN.md](PERSONAL-DEVELOPMENT-PLAN.md).
- Current facts and the unique next action remain owned by the `Current
  snapshot` in [PROGRESS.md](PROGRESS.md).
- Writable ownership remains owned by the active table in
  [PARALLEL-LANES.md](PARALLEL-LANES.md).
- Parent measurement contract:
  [personal-performance-benchmark-execution-plan.md](../evaluation/personal-performance-benchmark-execution-plan.md)
  v1.1 (especially §2 arms/fairness/broker, §3.1 B0, §9 order, §10 C1/C2
  rows).
- C1/C2 class contract (closed EVAL-004 addendum, reused only as
  **constraints**, never resumed):
  [personal-c1-c2-benchmark-execution-plan.md](../evaluation/personal-c1-c2-benchmark-execution-plan.md).
- An owner-directed **measurement** campaign begins only when the
  `Owner-directed campaign` row in `PROGRESS.md` names a newly activated EVAL
  ID. Closed EVAL-002 and EVAL-004 through EVAL-011 are never resumed,
  amended, or used as a denominator.

At every new window or context recovery, read those canonical sources first,
then use this document to select the next dependency-safe readiness item. If a
canonical source conflicts with this document, update this document in the
active task delivery and follow the canonical source.

## 2. What was wrong, and what “ready” now means

The 2026-08-20 assessment
([20260820-personal-c1-c2-readiness-delivery-assessment.md](../checkpoints/20260820-personal-c1-c2-readiness-delivery-assessment.md))
closed packages 6–8 as `assessment only / not-run` and package 9 as “a newly
preregistered B0 may be requested”. That bar does **not** produce the
conditions to run complete real-machine paired C1+C2.

| Previous disposition | Why it is insufficient | Amended requirement |
|---|---|---|
| Packages 1–5 complete (O-arm product evidence) | Necessary, not sufficient | Keep as historical/done. O-arm evidence must not substitute for P-arm, freeze, fairness, or B0. |
| Package 6 assessment only | C1/C2 P-arm would be `not-run` (execution plan §2.2) | Deliver a secret-safe pure-Pi broker and equivalent Workspace* fixture adapter. |
| Package 7 assessment only | No runner/corpus/oracle/seeds/digest ledger | Freeze those assets **before any sample**. |
| Package 8 assessment only | P/O equality never observable | Encode the §2.3 fairness contract so B0 can check it; prove the checker on non-B01. |
| Package 9 “may request B0” | Leaves guest, secret bind, isolation, and confirmatory cells unspecified | Produce guest procedure, bind path, isolation allocation, rewritten C1/C2 gates, then a new EVAL preregistration. B0 is package 15 (measurement), not a request token. |

**Ready to run** means packages 1–14 are `complete` with supported evidence.
**完整真机实测** then executes packages 15–17 under that new EVAL
(measurement-only; no product edits mid-campaign).

## 3. Continuation progress board

This is a compact recovery board, not a second source of task or campaign
truth. Update it together with the P9-T08 running report after each completed
validation unit.

| Package | Class | Status | Completed, supported facts | Open boundary | Next recovery action |
|---|---|---|---|---|---|
| Plan persistence | readiness | **done** | This amended programme is the navigation source linked from `PROGRESS.md`. | Keep revision and package status current. | Retain; do not reopen packages 1–5. |
| 1. C1 public O-arm | historical product | **done** | P2-T36 PR [#244](https://github.com/agentkernel/cognitive-os/pull/244) at `main@3efd7011`. Independent non-B01 WorkspaceRead and WorkspaceSearch Tasks completed public admit → candidate → lease → executor → verifier → acceptance. CI `32245868452` passed. | Not a paired benchmark. | Preserve; do not re-open. |
| 2. C2a mutation O-arm | historical product | **done** (P2-T37 + P2-T38) | P2-T37 PR [#246](https://github.com/agentkernel/cognitive-os/pull/246); P2-T38 PR [#248](https://github.com/agentkernel/cognitive-os/pull/248) at `main@2b4c8f17`. Raw-file SHA-256 preimage and VERIFY-skip on unreconciled Effect. | P-arm replacement-bytes Patch closed by **P9-T11**. | Do not reopen P2-T37/P2-T38. |
| 3. C2b governed session-2 | historical product | **done** (reconfirmed) | P2-T23 PR [#222](https://github.com/agentkernel/cognitive-os/pull/222) at `main@795bfac8`. Public Memory/Skill consumption and session-2 resume. | P-arm cannot use daemon Memory/Skill. | Remaining work is fairness classification, not a new P2 product task. |
| 4. C2c recovery | historical product | **done** (reconfirmed) | P2-T24 PR [#223](https://github.com/agentkernel/cognitive-os/pull/223) at `main@2b803e0f`. Original-key restart reconcile. | Campaign fault profile is measurement-only. | Remaining work is freeze + B0, not a new P2 product task. |
| 5. C2d public closure | historical product | **done** (reconfirmed) | P2-T14 acceptance authority; P2-T21 `GET /task/evidence`. C1/C2a public O4/O5 observed terminal facts. | Pure-Pi completion is not OS Task completion. | Remaining work is package 7 oracles, not a new P2 product task. |
| 6. Pure-Pi P arm | readiness delivery | **done** | Local Node 14/14. Linux exact `7dc8c999729028ddb850ab858e88e2f1ba8d5bf9`: D-Bus Secret Service get into loopback broker; C1/C2a fixture calls; probe item cleared. No daemon-proxy. | Live DeepSeek/Pi Provider call is a future EVAL sample, not a package-6 mutex. | Preserve. |
| 7. Frozen paired assets | readiness delivery | **done** | 17 instrument/corpus digests on Linux; disjoint B0/B1/B2 seeds; `retry=0`. | Not a sample. | Preserve. |
| 8. B0 fairness contract | readiness delivery | **done** | Non-B01 dry-run `fairness: pass`, `b0: false`. | Full B0 on B01 is package 15. | Preserve. |
| 9. B01 guest readiness | readiness delivery | **done** | [B01 procedure](../checkpoints/20260820-personal-c1-c2-b01-guest-procedure.md). Isolation reserved, not bound. | No guest mutation. | E7 `not-run` until EVAL activation. |
| 10. Paired C1+C2 cell definitions | readiness delivery | **done** | Plan §6 plus `cells.json` `sha256:82f93a40cd9b4a8a3486f14c584096bc0fa3268e2de6e9edba65807d6f0bd37a`. | Definitions are not results. | Preserve. |
| 11. Secret / doctor bind path | readiness delivery | **done** | [Bind runbook](../checkpoints/20260820-personal-c1-c2-secret-bind-runbook.md). P-arm probe get proven. | Live doctor is EVAL start gate. | E9 `not-run` until package 15. |
| 12. Environment checklist | readiness delivery | **done** | Section 8: E1–E6 and E10–E12 `pass`; E7–E9 `not-run` (EVAL not active). | Bind at package 15. | Preserve. |
| 13. Housekeeping | readiness delivery | **done** | Remote `personal/P2-T37-c2a-public-mutation-path` deleted; `ls-remote` empty. | Not a denominator. | Preserve. |
| 14. New EVAL preregistration | readiness delivery | **done (reserved, not activated)** | [EVAL-012 scaffolding](../evaluation/personal-perf-eval-012-preregistration.md) cites `7dc8c999`. | Must not activate. | Owner activates Current snapshot row before B0. |
| 15. B0 on B01 | measurement | **closed (EVAL-012 then EVAL-013)** | EVAL-012 B0 fairness **fail** (`system_task_prompt_bytes`). EVAL-013 on `evaluation/EVAL-013-freeze` recorded C1/C2a P/O B0 fairness **pass** and left B1/B2 `not-run`. Do not resume either EVAL. | Dry-run prompt bytes **P9-T09** (**done**). Live O-arm injection **P9-T10** (**done**). Comparable C2a Patch **P9-T11** (**done**). Live paired executor **P9-T12** (**done**). | New EVAL only after a new preregistration. |
| 15a. Live paired executor | readiness delivery | **done (P9-T12)** | `runLivePairedCell` with injected `executeArm`; `--append-system-prompt`; `counted_sample` only for b1/b2 after fairness pass. Merged PR [#252](https://github.com/agentkernel/cognitive-os/pull/252) at `main@39cf8019`. | Not a counted sample. | Preserve; B1/B2 still need a new EVAL. |
| 16. B1 C1/C2 pilot | measurement | **not-run** | Requires a future EVAL whose B0 passes **and** the P9-T12 live executor. | Not opened. | Do not use EVAL-012 or EVAL-013 seeds. |
| 17. B2 C1/C2 confirmatory | measurement | **not-run** | Requires B1. | Not opened. | Do not use EVAL-012 or EVAL-013 seeds. |

## 4. Fixed boundaries

1. The daemon remains the only authority writer; Pi, runners, fixture adapters,
   brokers, CLI, and test code produce observations or candidates only.
2. Provider material stays exclusively in an approved SecretStore and approved
   non-logging input paths. Never use `secret-tool search` or
   `secret-tool lookup`; never expose secret material in argv, environment,
   configuration, logs, evidence, Git, or chat. Never copy or link
   `provider.json` keyfiles between roots.
3. `B01-Desktop-Linux-002` is not a development environment. Use it only after
   a new owner-activated evaluation lease and preregistered procedure. Never
   access or operate `B01-Clean-Linux-001`.
4. Product work uses a formal `P*-T*` task, one task branch, one Draft PR, one
   narrow lease, focused negatives, exact supported validation, and full
   merge/lease/branch closure. Campaign instruments for packages 6–8 are
   **P9-T08**, not a second product authority path.
5. Windows GNU may run TypeScript, documentation, consistency, diff, and Rust
   formatting only. Rust build/test/Clippy/runtime validation consumes a pushed
   exact revision on `DEV-LINUX-NATIVE-01` or supported CI. P-arm/broker
   qualification is Linux-native or B01, never local GNU Rust.
6. Runner, broker, corpus, oracle, redactor, analysis, and cleanup assets are
   measurement-only. They must not become a second authority writer or add
   benchmark-only product authority.
7. Closed EVAL-002 and EVAL-004 through EVAL-011 roots, ports
   `48286`–`48298` / `48386`–`48398` / `48383`, and SecretStore items `/12`–`/19`
   stay isolated. Do not reuse P2-T37 roots `p2-t37-c2a-write-20260820` /
   `p2-t37-c2a-patch-20260820`.
8. No readiness work itself makes a performance, Gate, release, Profile, B01,
   or Agent-benefit claim. Packages 15–17 remain `hypothesis` until that EVAL’s
   own evidence rules and independent review say otherwise.
9. Measurement campaigns are measurement-only (Operating Model §2.5): never
   modify product code, contracts, negatives, tests, or generated handbook
   sources to make a cell runnable. A missing capability is `not-run` /
   `not_available`.

## 5. Done vs remaining (dependency order)

The programme’s **readiness** definition of done is: every package 1–14 row
has supported evidence, and no unexplained `partial`, `not-run`, product gap,
asset gap, broker gap, or public-observation gap remains that would force
C1/C2 `not-run` at B0. Packages 15–17 are the measurement campaign that
follows.

| Order | Package | Class | Required outcome before advancing | Current navigation state |
|---:|---|---|---|---|
| 1 | C1 public O-arm | historical | WorkspaceRead and WorkspaceSearch each traverse public admit → Context → real Pi candidate → scheduler lease → daemon Tool executor → independent verifier → daemon acceptance. | **Complete:** P2-T36 / PR #244 / `main@3efd7011`. |
| 2 | C2a mutation O-arm | historical | WorkspaceWrite/Patch carry schema-bound input and expected preimage through public authority; Intent/Effect, original-key reconcile, independent verification, and acceptance close the Task. | **Complete:** P2-T37 / PR #246 / `main@286f7538`. |
| 3 | C2b governed session-2 | historical | A real user path proves daemon-authorized Memory/Skill consumption and resume without forged governance state. | **Complete (reconfirmed):** P2-T23 / PR #222. No new product gap. |
| 4 | C2c recovery | historical | Controlled fixture crash/`OUTCOME_UNKNOWN` cases query by original key, reconcile, independently verify, and then accept or honestly remain unresolved. | **Complete (reconfirmed):** P2-T24 / PR #223. No new product gap. |
| 5 | C2d public closure | historical | Public observations distinguish admission, receipt, Effect closure, verification, and daemon acceptance. | **Complete (reconfirmed):** P2-T14 + P2-T21 + C1/C2a public O4/O5. No new product gap. |
| 6 | Pure-Pi P arm | readiness | Same-fixture adapter works without daemon, Extension, Task, Context, Memory, Skill, retry, cache, or verifier; credential route is approved and secret-safe; equivalent public observation exists for C1/C2 tasks. | **Complete — P9-T08/D02.** Local 14/14; Linux D-Bus get at `7dc8c999`. No P2 gap. |
| 7 | Frozen paired assets | readiness | Runner, fixture corpus, oracle, redactor, analysis, reset, cleanup, command manifests, seeds, `retry=0`, timeout, arm order, and all digests are frozen **before any sample**. | **Complete — P9-T08/D03.** 17 Linux digests; disjoint seeds. |
| 8 | B0 fairness readiness | readiness | The execution-plan §2.3 equality checks (tool set, input bytes, workspace, oracle, Provider/model, timeout, retry=0, environment, cleanup) are encoded and a non-B01 run proves the runner can observe pass/fail. Product O-arm does not substitute. | **Complete — P9-T08/D03.** Linux dry-run pass; `b0: false`. Full B0 on B01 is package 15. |
| 9 | B01 guest readiness | readiness | Checkable procedure for `B01-Desktop-Linux-002` only: new EVAL ID, new root, new ports, new SecretStore, snapshot/baseline rules, standing-authorization limits. No guest mutation in this package. | **Complete — P9-T08/D04.** No guest mutation. |
| 10 | Paired C1+C2 cell definitions | readiness | Confirmatory rows no longer say “OS arm unreachable”. Each class lists remaining blockers and whether it is P/O performance-comparable or capability-gap. | **Complete** in section 6 and `cells.json`. |
| 11 | Secret / doctor bind path | readiness | `cognitive init --reuse-existing-secret-binding` (opaque SecretRef only; `secret_material_written: false`) then redacted doctor `secret_ref_resolves` and `first_conversation_ready: true`. No keyfile copy/link; no material on argv/stdout. | **Complete as runbook — P9-T08/D04.** Live doctor is EVAL start gate (E9). |
| 12 | Environment checklist | readiness | Section 8 is an exit table: every row `pass` or explicit `not-run` with recovery before package 14. | **Complete.** E7–E9 `not-run` only because EVAL is not active. |
| 13 | Housekeeping | readiness | Leftover remote `personal/P2-T37-c2a-public-mutation-path` deleted or recorded absent. | **Complete.** Branch deleted. |
| 14 | New EVAL preregistration | readiness | Secret-free preregistration and freeze ledger for a **new** EVAL ID; no samples; evaluation routing still off until the owner activates the Current snapshot row. | **Complete scaffolding, not activated.** `PERSONAL-PERF-EVAL-012`. |
| 15 | B0 qualification on B01 | measurement | Execution plan §3.1 + C1/C2 addendum B0: warmups, one qualification seed per class, secret scan, tool-equivalence, timeout, cleanup. Any fairness fail blocks B1. | **After** 1–14 and EVAL activation. First live measurement. |
| 16 | B1 C1/C2 pilot | measurement | C1/C2 addendum: five pilot seeds per class, two runs per arm; classify instrumentation failures; do not enter confirmatory N. | After B0 pass. Measurement-only. |
| 17 | B2 C1/C2 confirmatory | measurement | C1/C2 addendum: 30 held-out paired seeds per class, three runs per arm when the Provider lacks deterministic replay; `retry=0`; started=retained. | After B1. This is 完整真机实测 of C1+C2. |

## 6. Mapping onto execution plan §9 / §10 C1/C2 rows

Parent §9 order is unchanged: freeze → fairness/secret/denominator review →
**B0** → B1 → freeze B2 N → B2 → (B3–B5 are out of C1/C2 完整实测 unless the
owner later expands the EVAL). This programme supplies the freeze and
readiness gates so C1/C2 are no longer pre-declared `not-run` for a missing
O-arm.

| Parent §10 row | Historical expected status | O-arm product fact (packages 1–5) | Remaining blocker for a new EVAL | Comparable P/O performance? |
|---|---|---|---|---|
| B0 qualification | not-run | N/A | Packages 6–14, then package 15 on B01 | N/A (qualification, not a claim sample) |
| B2 C1 read-only workspace | expected not-run; missing product Tool caller | **Closed:** P2-T36 public Read/Search | P-arm equivalent WorkspaceRead/Search fixture (pkg 6); frozen read-only corpus/oracle (pkg 7); fairness checker (pkg 8); B0 (pkg 15) | **Yes**, if P-arm fixture tool schemas match O-arm WorkspaceRead/Search and Pi built-ins stay denied on both arms |
| B2 C2 mutation | expected not-run; missing write/test/verifier | **Closed:** P2-T37 public Write/Patch + P2-T22 repair journey | P-arm equivalent Write/Patch fixture (pkg 6); frozen repair corpus, hidden tests, preimages, mechanical oracle (pkg 7); fairness; B0 | **Yes**, if tool schemas and workspace bytes match; OS completion still requires Effect + independent acceptance (addendum §4) |
| B2 C2 Memory/Skill | expected not-run; missing user execution path | **Closed as O-arm path:** P2-T23 | P-arm reference is frozen procedure bytes, not daemon Memory/Skill (addendum §1). Package 10 must classify this row. | **Capability-gap / split scores** unless a secret-free equivalent tool set can be frozen. Do not daemon-proxy P. Still a required confirmatory cell for 完整 C1+C2 as defined by the addendum |
| C2c Effect recovery | expected not-run; no production caller | **Closed as O-arm path:** P2-T24 | Campaign-authorized default-off fault profile + original-key query on O; P-arm fixture mutation reference | Split scores: P is fixture reference; O is governed reconcile. Required confirmatory cell |
| C2d verified completion | expected not-run; verifier unwired | **Closed as O-arm path:** P2-T14 / P2-T21 | P-arm uses an external mechanical oracle; O-arm uses daemon acceptance. A pure-Pi completion is not OS Task completion | Split scores. Required confirmatory cell |
| Skill S4/S8 actual Agent consumption | expected not-run | P2-T23 is public consumption, not S4/S8 Agent-benefit | Out of **完整 C1+C2** unless the owner expands the EVAL | Do not treat S4/S8 as a C1/C2 mutex |
| Tool T4–T9 actual governed calls | expected not-run | C1/C2a public Workspace* calls exist; T6/T7 extras remain separate | Out of **完整 C1+C2** unless expanded | Do not block C1/C2 B0 on T6–T9 |

**完整 C1+C2** for this programme means the C1/C2 addendum §1 classes: **C1,
C2a, C2b, C2c, C2d**. C0, B3–B5, O2–O14 extras, and T6–T9 are not readiness
mutexes here.

Parent §2.4 “C1/C2 OS arm unreachable” is **stale product text**. This
programme does not silently edit that file during P9-T08/D01. Package 14’s
EVAL preregistration must cite this section as the readiness overlay so the
new campaign does not inherit the old expected-not-run gates.

## 7. Remaining package specifications (readiness delivery)

Each remaining readiness package below is real delivery. Fail closed rather
than recording `assessment only`.

### 7.1 Package 6 — Pure-Pi P arm (`P9-T08/D02`)

- **Purpose:** Make arm `P` reachable for the same C1/C2 tasks as arm `O`
  without CognitiveOS authority.
- **Arm definition (execution plan §2.1 / §2.2):** official Pi `0.81.1` →
  approved baseline credential broker → DeepSeek. No Extension, daemon, Task,
  Context, Memory, Skill, retry, cache, or verifier.
- **Credential route:** campaign-only loopback broker (plan §2.2 option 2):
  read once from Linux Secret Service into process memory; inject upstream
  auth only in memory; loopback-only; single-user; no request/response/header
  logs; Pi sees only a fixed non-secret local endpoint/token; per-request
  count/duration/byte bounds; cleanup deletes broker socket/process, not the
  owner key. Broker local latency is recorded separately.
- **Equivalent observation:** campaign fixture tool adapter advertising the
  **same** WorkspaceRead/Search/Write/Patch JSON schemas the O-arm Extension
  advertises. Pi built-in filesystem/shell tools stay denied. The adapter
  executes against the frozen fixture workspace only. It is not the
  CognitiveOS Extension and must not write daemon authority state.
- **Product gap rule:** no new P2 task is identified on current `main`. If D02
  proves schema-equivalent fixture tools cannot be qualified, stop and
  register a P2 task; never disguise a daemon proxy as `P`.
- **Writable paths:** `tools/personal/c1-c2-paired/` (measurement-only),
  P9-T08 running report, this plan, `PROGRESS.md`.
- **Lease:** `lease/personal/P9-T08/c1-c2-paired-readiness`
- **Validation environment:** `DEV-LINUX-NATIVE-01` (non-B01). Not
  `B01-Desktop-Linux-002`. Not local GNU Rust.
- **Acceptance:** (1) broker threat review against §2.2, item by item;
  (2) one C1-class and one C2a-class fixture call succeed through `P` with
  mechanical oracle; (3) secret scan of argv/env/config/logs/evidence is
  clean; (4) broker performs no Context/Tool-as-authority/Memory/Task/retry/
  cache/verification.
- **Fail closed:** missing SecretStore, non-loopback bind, secret-shaped
  process input, or daemon reuse.
- **Non-claims:** not B0, not paired performance, not Gate/release/Profile.

### 7.2 Package 7 — Frozen paired assets (`P9-T08/D03`)

- **Purpose:** Freeze runner, corpus, oracle, redactor, seeds, timeouts,
  arm-order, digest ledger, and `retry=0` **before any sample**.
- **Required freeze set (execution plan §2.5 + §3.1 + C1/C2 addendum §2):**
  P/O process commands; input bytes; task-seed lists (B0 / B1 / B2 held-out,
  non-overlapping); arm order RNG seed; pure-Pi broker digest; OS Extension
  digest; Pi `0.81.1` SRI; corpus version and workspace snapshot digest;
  expected preimages; hidden tests; independent/mechanical oracles; redactor;
  reset and cleanup; timeout; `retry=0`; output schema; denominator rules.
- **Placement:** secret-free sources in `tools/personal/c1-c2-paired/`;
  digest ledger in `docs/evaluation/` at package 14; raw payloads only in
  ignored `artifacts/` or the future guest campaign root.
- **Validation environment:** `DEV-LINUX-NATIVE-01` for digest computation;
  Windows may compute secret-free file digests only.
- **Acceptance:** a freeze ledger lists every instrument digest; B0/B1/B2
  seeds are disjoint; `retry=0` is in the command manifests; redactor rejects
  unredacted output.
- **Fail closed:** unfrozen instrument, overlapping seeds, or secret-shaped
  corpus bytes.
- **Non-claims:** freeze is not a sample and not B0 pass.

### 7.3 Package 8 — B0 fairness contract, checkable (`P9-T08/D03`)

- **Purpose:** Make execution plan §2.3 observable before B01 B0.
- **Equality that must be checkable for each task-seed:** same Pi
  package/version/SRI and Node; same Provider/base URL/model snapshot; same
  system/task prompt bytes and task input digest; same sampling parameters;
  same timeout, `retry=0`, max Agent turn; same visible tool set and schema;
  same workspace snapshot and network policy; same CPU/memory/cwd/fs where
  the guest will apply them; same oracle version; same warm/cold stratum
  rules.
- **Declared differences (allowed):** `P` does not pass through CognitiveOS;
  `O` uses Extension, daemon proxy, and the governed surface. Streaming vs
  non-streaming follows EVAL-002 lesson: fair endpoint is
  time-to-complete-response; TTFT stays `not_available` unless both arms
  expose it.
- **Acceptance:** a non-B01 dry-run of the paired runner emits a fairness
  record that would `pass` or `fail` on those axes without counting as B0.
  Product O-arm Tasks from P2-T36/T37 must not be filed as this evidence.
- **Fail closed:** missing checker, skipped axis, or using O-arm-only
  observation as P/O equality.
- **Non-claims:** non-B01 checker proof is not B0 and not B01 guest evidence.

### 7.4 Package 9 — B01 guest readiness (`P9-T08/D04`)

- **Purpose:** A checkable procedure so package 15 can start without
  discovering isolation or route gaps.
- **Guest:** `B01-Desktop-Linux-002` only. `B01-Clean-Linux-001` forbidden.
- **Control route:** `wuz@192.168.1.2` (libvirt host `hal9000`,
  `virsh -c qemu:///system`) → ProxyJump → `hal9001@192.168.123.160`.
- **Reserved isolation (bind only at EVAL activation, package 14/15):**
  - Campaign ID: `PERSONAL-PERF-EVAL-012` (reserved; **not active**)
  - Runtime root: `/home/hal9001/perfeval012-<activation-date>`
  - Daemon loopback: `127.0.0.1:48300`
  - P-arm broker loopback: `127.0.0.1:48400`
  - SecretStore item: campaign-unique path **not** `/12`–`/19` (planned
    `/20` unless the guest already holds that item; then choose the next
    free unused item and record it in the preregistration)
- **Snapshot/baseline:** restore or residual P9-T04/closed-EVAL state changes
  require a separate owner decision. Do not mutate the guest in this package.
- **Standing authorization:** applies to the later EVAL lease only; does not
  authorize package-9 guest mutation, force push, or `B01-Clean-Linux-001`.
- **Acceptance:** procedure document lists route, isolation table, snapshot
  rule, cleanup, and stop conditions; each item is checkable.
- **Non-claims:** writing the procedure is not B0 and not B01 Gate evidence.

### 7.5 Package 10 — Paired C1+C2 cell definitions (`P9-T08/D01` text, `D04` freeze)

- **Purpose:** Replace “OS arm unreachable” with the section 6 overlay and
  freezeable cell IDs the future EVAL will run.
- **B0 cells (package 15):** one qualification seed per class C1, C2a, C2b,
  C2c, C2d; three warmups per arm; secret scan; tool-equivalence; timeout;
  cleanup; no claim samples.
- **B1 cells (package 16):** five pilot seeds per class; two runs per arm.
- **B2 cells (package 17):** 30 held-out paired seeds per class; three runs
  per arm if the Provider lacks deterministic replay.
- **Acceptance:** cell IDs, seed strata, oracles, and skip classes are
  written so a measurement session cannot “forget” C2b–d.
- **Non-claims:** definitions are not results.

### 7.6 Package 11 — Secret / doctor / `first_conversation_ready` bind path (`P9-T08/D04`)

- **Purpose:** Repeat the P2-T37 Patch bind lesson as the **only** allowed
  Provider bind for a new isolated root.
- **Required sequence:**
  1. `cognitive init --reuse-existing-secret-binding` with an opaque
     SecretRef already in the approved store.
  2. Confirm `action: bound_existing_secret_ref`,
     `secret_material_written: false`, `secret_ref_redacted: true`.
  3. Redacted `cognitive doctor`: Provider ready, `secret_ref_resolves:
     true`, selected-model digest match, `first_conversation_ready: true`.
- **Forbidden:** file-copy or link of `provider.json`; `secret-tool search` /
  `lookup`; printing material; argv/env/config key; recapture from a
  keyfile.
- **Note:** `first_conversation_ready` is conversation-shell readiness, not
  C1/C2 Task completion.
- **Acceptance:** the runbook is the package-15 start gate; a non-B01
  disposable root may prove the commands if a SecretRef is already present.
  If no SecretRef is available, record `not-run` with recovery “owner
  graphical hidden-input import into the **new** EVAL SecretStore item”, not
  a product TODO.
- **Non-claims:** doctor ready is not B0 and not Agent-benefit.

### 7.7 Package 12 — Environment checklist as exit criterion (`P9-T08/D04`)

Section 8 is the exit table. Package 12 is complete only when every row is
`pass` or an explicit `not-run` with a recovery action that is still
compatible with starting package 15.

### 7.8 Package 13 — Housekeeping (`P9-T08/D04`)

- Delete leftover remote `personal/P2-T37-c2a-public-mutation-path` if still
  advertised, or record that GitHub no longer lists it.
- Confirm closed EVAL ports/roots/SecretStore items remain unused.
- **Non-claims:** branch deletion is not readiness of P-arm or B0.

### 7.9 Package 14 — New EVAL preregistration scaffolding (`P9-T08/D04`)

- **Purpose:** Secret-free preregistration + freeze ledger for
  `PERSONAL-PERF-EVAL-012` (or the next unused ID if 012 is taken), citing
  packages 6–13 digests.
- **Must include:** exact Git revision to freeze; isolation table; B0/B1/B2
  cell list from section 6; Provider budget ceiling; `retry=0`; claim
  ceiling `hypothesis`; reviewer `not_reviewed`; cleanup; “no product edits
  mid-campaign”.
- **Must not:** activate the `Owner-directed campaign` row; start samples;
  bind guest ports; reuse closed assets.
- **P9-T08 task acceptance** closes when package 14 exists and packages 6–13
  have supported evidence. Measurement is packages 15–17 under that EVAL.

## 8. Environment configuration checklist (exit criterion)

This table is an **exit criterion**, not leftover homework. Product evidence
on non-B01 hosts may satisfy rows marked “readiness”. Rows marked “EVAL”
become `pass` only at package 14/15.

| ID | Required fact | Scope | Current | Exit |
|---|---|---|---|---|
| E1 | Branch revision is pushed; remote worktree checks out that exact commit | readiness | **pass** `7dc8c999729028ddb850ab858e88e2f1ba8d5bf9` | `pass` before package 14 |
| E2 | Disposable non-B01 remote root exists (Git worktree, not a copied local tree); not a closed EVAL or P2-T37 root | readiness | **pass** `/home/wuz/p9-t08-c1-c2` | `pass` for packages 6–8 |
| E3 | Linux native records Rust 1.97.1, Node, pnpm, Pi `0.81.1`, adapter, Extension versions/digests | readiness | **pass** rustc 1.97.1, Node v22.19.0, pnpm 10.33.2; Pi `0.81.1` SRI frozen. Live `pi --version` **not-run** (recover at EVAL freeze) | `pass` at freeze |
| E4 | Provider-dependent paths use only approved SecretStore; no secret in argv/env/config/logs/evidence/Git/chat | both | **pass** for P-arm D-Bus probe get; product doctor remains E9 | `pass` bind path (pkg 11) before B0 |
| E5 | Product fixtures are cleanable; no reuse of closed campaign root/port/SecretStore/runner/corpus/oracle/denominator | both | **pass** new IDs only; probe suffix `9` cleared | `pass` when 6–14 record new IDs only |
| E6 | Cleanup removes task-created runtime/process state; redacted facts only | both | **pass** probe cleared; `/tmp` prove scripts removed; broker not left listening | `pass` after each Linux/B01 session |
| E7 | B01 guest identity is `B01-Desktop-Linux-002`; `B01-Clean-Linux-001` untouched | EVAL | **not-run** (EVAL not active). Recovery: bind at package 15. Procedure written. | `pass` at package 15 |
| E8 | New EVAL ID, root, ports `48300`/`48400` (or recorded substitutes), SecretStore item ≠ `/12`–`/19` | EVAL | **not-run** reserved, not bound. Recovery: bind at package 15. | `pass` at package 15 |
| E9 | `--reuse-existing-secret-binding` only; doctor `first_conversation_ready: true` without material print | EVAL | **not-run**. Recovery: bind at package 15 start gate using the runbook. | `pass` before first B0 sample |
| E10 | Fairness checker exists and has a non-B01 observability proof | readiness | **pass** Linux dry-run `fairness: pass`, `b0: false` | `pass` at package 8 |
| E11 | C1/C2 freeze ledger complete; `retry=0`; B0/B1/B2 seeds disjoint | readiness | **pass** 17 files; 180 disjoint seeds; `retry=0` | `pass` at package 7 |
| E12 | Leftover remote `personal/P2-T37-c2a-public-mutation-path` absent or deletion recorded | readiness | **pass** deleted; `ls-remote` empty | `pass` at package 13 |

Package 12 is `complete` iff E1–E6 and E10–E12 are `pass`, and E7–E9 are
either `pass` or `not-run` solely because the EVAL is not yet activated —
with the recovery action “bind at package 15 start gate”, not a product
gap.

## 9. Historical product route (packages 1–5)

### 9.1 P2-T36 (C1 O-arm) — closed

PR [#244](https://github.com/agentkernel/cognitive-os/pull/244) merged at
`main@3efd7011b605a32ac0c9ec114321831995f32d90`. Final documentation-head
workflow `32245868452` passed Ubuntu, Windows, and `required-ci`. Public
WorkspaceRead and WorkspaceSearch completed the authority chain on fresh
non-B01 Linux runtimes.

### 9.2 P2-T37 (C2a O-arm) — closed

PR [#246](https://github.com/agentkernel/cognitive-os/pull/246) merged at
`main@286f7538148ba0d22f496f1f44d1af46f0f44aa0`. Public Write
`task://personal/p2-t37-public-write` and Patch
`task://personal/p2-t37-public-patch-reseed` each reached `COMPLETED`.
Bind lesson: file-copy of `provider.json` is refused; recovery is
`cognitive init --reuse-existing-secret-binding`.

## 10. Measurement campaign (packages 15–17)

These packages are **not** P9-T08 implementation. They execute only after:

1. packages 6–14 are complete;
2. `PROGRESS.md` Current snapshot names an active
   `PERSONAL-PERF-EVAL-012` (or the ID actually preregistered);
3. an evaluation lease `lease/personal/EVAL-012/<purpose>` owns only
   `docs/evaluation/`, `docs/checkpoints/`, and `docs/plan/PROGRESS.md`.

Behavior: Operating Model §2.5 measurement-only; `TEST-REPORT-INCREMENTAL-01`
appends each finished cell immediately; Provider cells `retry=0`; every
started sample retained. Missing capability is `not-run`, never a mid-campaign
product fix.

B3/B4/B5 remain available on the parent execution plan but are **out of
scope** for 完整 C1+C2 unless the owner expands that EVAL.

## 11. Stop conditions and owner confirmation boundaries

Stop the readiness task only for:

- owner pause or scope change;
- secret exposure (rotate the Provider key; do not continue with the same
  material);
- destructive/irreversible guest or snapshot change beyond the preregistered
  allowlist;
- unknown concurrent worktree changes whose ownership cannot be resolved;
- a product gap that D02 fail-closes into a new P2 task (then register that
  task; do not invent a daemon-proxy P-arm).

Do **not** stop solely because D01 docs landed, a commit/push/CI round
finished, or package 6–8 is `not-run` without attempting delivery.

Owner confirmation is required to:

- activate the `Owner-directed campaign` row (package 15 start);
- change B01 snapshots/baselines beyond the package-9 allowlist;
- import a Provider key when `--reuse-existing-secret-binding` cannot bind
  (graphical hidden input into the **new** SecretStore item only);
- expand 完整实测 beyond C1+C2a–d into C0/B3–B5/T6–T9.

## 12. Unique next action

Packages 1–14 are delivered and P9-T08 is merged at `main@f24c282e`.
`PERSONAL-PERF-EVAL-012` stays **reserved, not active**. Unique next action:
the owner activates that EVAL ID in the Current snapshot, then package 15
(B0) may start on `B01-Desktop-Linux-002`.

## 13. Completion gate

**Readiness (packages 1–14) is complete** when every row in section 5 orders
1–14 has supported evidence and section 8 E1–E6/E10–E12 are `pass` (E7–E9
may remain `not-run` only as the EVAL start gate). Then:

> Packages 6–14 are delivered. A newly preregistered B0 on
> `B01-Desktop-Linux-002` may **start**. No paired benchmark, performance,
> Agent-benefit, Gate, release, Profile, or B01 conclusion has been produced.

**完整真机实测 (packages 15–17) is complete** when that EVAL’s B0, B1, and
B2 C1/C2 cells are each `pass`, `fail`, `partial`, or honest `not-run` with
no remaining readiness gap that this programme was supposed to close.
)
