# Personal C1/C2 Paired Benchmark Readiness Delivery Plan

- Status: active navigation document; no evaluation campaign is active
- Purpose: preserve the dependency-ordered delivery route to a **new**, not yet
  activated, C1/C2 paired benchmark readiness assessment
- Current product task: `P2-T37` public WorkspaceWrite/Patch path on
  `personal/P2-T37-c2a-public-mutation-path` / Draft PR [#246](https://github.com/agentkernel/cognitive-os/pull/246), based on
  `main@08819a82688c78f56af3cbe8b202b787986feefb`
- Current C1 implementation checkpoint: P2-T36 closed at
  `main@3efd7011b605a32ac0c9ec114321831995f32d90`
- Last reconciled: 2026-08-20

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
- An owner-directed campaign begins only when the `Owner-directed campaign`
  row in `PROGRESS.md` names a newly activated EVAL ID. Closed EVAL-004 through
  EVAL-010 are never resumed, amended, or used as a denominator.

At every new window or context recovery, read those canonical sources first,
then use this document to select the next dependency-safe readiness item. If a
canonical source conflicts with this document, update this document in the
active task delivery and follow the canonical source.

## 2. Continuation progress board

This is a compact recovery board, not a second source of task or campaign
truth. Update it together with the P2-T36 running report after each completed
validation unit. On recovery, use the `Next recovery action` column only after
reconciling the authoritative sources in section 1.

| Package | Status | Completed, supported facts | Open boundary | Next recovery action |
|---|---|---|---|---|
| Plan persistence | complete | This navigation document is linked from `PROGRESS.md`; the P2-T36 lease is closed and the narrow closure-ledger lease owns these fact corrections. | Keep revision and row status current during reconciliation. | Read this board after `PROGRESS.md`, lease, and branch reconciliation. |
| P2-T36/D01: Pi candidate surface | complete | WorkspaceRead is I/O-free and daemon-governed; native Pi filesystem/shell tools remain denied. Exact Linux adapter protocol test passed **21/21**. | Unit protocol evidence is not a public C1 route. | Preserve existing negatives while testing the real public route. |
| P2-T36/D01: exact source and base tools | complete | A cleanable, non-B01 native Linux Git worktree was refreshed from the pushed branch and detached at exact current task head `33a95017a4fd300a622c0d3a79485f4dda1f09a5`, clean. Rust adapter/admin CLI, Extension, and pinned Pi `0.81.1` build/version checks passed. The daemon, admin CLI, and adapter were rebuilt, and public Pi configuration was rewritten without secret access. | The public daemon must establish its own readiness projection from the rebuilt exact revision. | Continue with the public candidate-to-authority bridge gap. |
| P2-T36/D02: disposable Pi configuration | complete | Public `cognitive pi configure` created a cleanable non-B01 runtime configuration referencing the pinned Pi, built Extension, and candidate adapter. No Provider material was configured or inspected. | Configuring Pi neither starts the daemon nor proves readiness or candidate execution. | Start the public daemon in that disposable runtime and capture only redacted status/doctor facts. |
| P2-T36/D02: public C1 WorkspaceRead/Search | complete | Fresh non-B01 Linux runtimes independently completed public Task-bound WorkspaceRead and WorkspaceSearch through authenticated daemon candidate admission, scheduler lease, native executor, passed independent verification, and daemon acceptance; both Tasks reached `COMPLETED`. PR [#244](https://github.com/agentkernel/cognitive-os/pull/244) is merged at `main@3efd7011`. | C1 completion does not establish C2, benchmark, or promotion claims. | Reconfirm C2a WorkspaceWrite/Patch on exact merged `main`. |
| P2-T36/D02: provider/readiness boundary | complete | No secret was read, printed, searched, logged, or placed in command arguments. The updated owner-designated file was streamed only through non-PTY public CLI stdin; redacted doctor reports Provider SecretRef resolution, selected-model digest match, and `first_conversation_ready: true`. The independent P2 runtime did not reuse EVAL-011 state. | Pi launch is permitted only after daemon-owned readiness admits it. | Repair the public Pi conversation caller/runtime integration; do not inspect material or reuse any closed-EVAL item. |
| P2-T36/D03: supported validation and CI | complete | Final documentation-head workflow `32245868452` passed Ubuntu, Windows, and required-ci. | C1-supported validation remains scoped to the merged product revision. | Preserve the report as C1 evidence only. |
| P2-T36/D03: task closure | complete | PR #244 merged at `3efd7011`; the lease is archived; local/remote task branches are deleted; local `main` equals `origin/main`. | No P2-T36 closure work remains. | Select C2a per section 7. |
| C2a-C2d public O-arm | P2-T37/D02 in progress | Existing P2-T21 through P2-T24 provide product evidence for candidate parameters, mutation, consumption, recovery, and observations; P2-T36 is closed. Public launcher and Extension expose daemon-governed WorkspaceWrite/Patch. Public Write Task completed the full authority chain. Draft PR [#246](https://github.com/agentkernel/cognitive-os/pull/246) is open. | Public WorkspacePatch still needs its own fresh Task. | Run a separate public WorkspacePatch lifecycle; then reconfirm C2b-d. |
| Pure-Pi P arm and frozen paired assets | not started | No new runner, corpus, broker, oracle, or campaign asset is in use. | Product O-arm readiness is a prerequisite; new EVAL activation remains owner-only. | Do not begin until the formal preparation boundary permits it. |
| New C1/C2 campaign | not started / inactive | EVAL-004 through EVAL-010 remain closed and isolated. | No new EVAL ID, preregistration, root, port, SecretStore item, denominator, or sample is authorized by this plan. | Only a future owner activation may request a new preregistration and B0. |

### 2.1 Delivery-stage progress table

Use this table as the compact checkpoint ledger for a resumed session. A row is
`complete` only when its stated evidence exists on the recorded exact revision;
`in progress` means that a single next action is already identified; and
`blocked` means the blocker and recovery route must be recorded in the running
report before selecting unrelated work. `Not started` is not a failure.

| Stage | Status | Exact evidence or durable fact | Exit criterion | Single next action |
|---|---|---|---|---|
| 0. Navigation and ownership | complete | P2-T37 is leased on `personal/P2-T37-c2a-public-mutation-path` / Draft PR [#246](https://github.com/agentkernel/cognitive-os/pull/246). Public Write lifecycle evidence is in the P2-T37 report. | Current revision, active lease, report, and next action reconcile. | Complete the separate public WorkspacePatch Task. |
| 1. P2-T36 failure-first surface proof | complete | The missing `WorkspaceRead` Extension registration was observed as the expected failure before implementation. | A focused regression proves the new surface and native Pi deny policy. | Preserve the focused negative while changing D02. |
| 2. WorkspaceRead candidate implementation | complete | Pi Extension registration, adapter event extraction, and protocol negatives are implemented; exact native adapter protocol test is **21/21 pass**. | WorkspaceRead and WorkspaceSearch can each produce one untrusted, schema-bound candidate while built-ins fail closed. | Exercise this implementation only through the real public composition. |
| 3. Exact native source and toolchain | complete | Cleanable non-B01 Git worktree refreshed directly from GitHub and detached at `33a95017a4fd300a622c0d3a79485f4dda1f09a5`; its status is clean. Rust 1.97.1, Node 22.19.0, and Pi 0.81.1 were verified. Exact-revision daemon/admin/adapter binaries were rebuilt and the non-secret Pi configuration was refreshed. | Public readiness is observed from the rebuilt exact revision. | Continue with the public candidate-to-authority bridge gap. |
| 4. Public Pi configuration | complete | `cognitive pi configure` created a non-secret, cleanable non-B01 `pi.json` using the pinned Pi, Extension, and adapter. | Daemon-owned readiness can inspect the configuration without direct Provider or secret handling by Pi. | Continue with a bounded public conversation after caller recovery. |
| 5. Daemon-owned readiness | complete | Exact-revision public daemon at `127.0.0.1:48436` reports system/database/secret/provider/daemon/Pi ready, SecretRef resolution and selected-model digest match, Pi `0.81.1`, and `first_conversation_ready: true`; Provider material was delivered only through non-PTY public CLI stdin and was never exposed to argv/environment/config/logs/evidence. | Public doctor admits Pi without exposing Provider material. | Drive one bounded public Pi conversation and capture lifecycle evidence. |
| 6. C1 WorkspaceRead public route | complete | A fresh non-B01 Linux Task completed public Pi WorkspaceRead through candidate admission, lease, executor, passed independent verification, and daemon acceptance. | The evidence is retained in the P2-T36 report. | Reconfirm C2a without borrowing C1 evidence. |
| 7. C1 WorkspaceSearch public route | complete | A separate fresh non-B01 Linux Task independently completed public Pi WorkspaceSearch through the same authority chain. | The evidence is retained in the P2-T36 report. | Reconfirm C2a without borrowing C1 evidence. |
| 8. Required CI and supported regressions | complete | Final documentation-head required CI `32245868452` passed Ubuntu, Windows, and required-ci. | C1 CI is complete at the merged revision. | Use exact merged `main` for C2a reconfirmation. |
| 9. P2-T36 closure | complete | PR #244 merged at `3efd7011`; lease/branch/main reconciliation is complete. | No P2-T36 work remains. | Select C2a under a new narrow lease only if the real public route exposes a gap. |
| 10. C2 and paired-benchmark readiness | in progress | C1 is complete. P2-T37 public WorkspaceWrite completed the authority chain; public WorkspacePatch remains. C2b-d, P arm, frozen assets, fairness B0, and a new campaign assessment remain future dependencies. | Every section 4 package has supported evidence and no unresolved non-pass result. | Complete the separate public WorkspacePatch Task on exact native Linux. |

### 2.2 Resume checklist

Before executing the `Single next action` above, record the following facts in
the running report if they changed since the prior session:

- task branch, local `HEAD`, upstream `HEAD`, Draft PR URL/status, and current
  CI run;
- active lease and its exact writable paths;
- remote host, disposable runtime root, and exact Git worktree revision;
- completed/remaining validation units as `pass`, `fail`, `partial`, or
  `not-run`; and
- the one next action, including any concrete blocker recovery route.

## 3. Fixed boundaries

1. The daemon remains the only authority writer; Pi, runners, fixture adapters,
   brokers, CLI, and test code produce observations or candidates only.
2. Provider material stays exclusively in an approved SecretStore and approved
   non-logging input paths. Never use `secret-tool search` or
   `secret-tool lookup`; never expose secret material in argv, environment,
   configuration, logs, evidence, Git, or chat.
3. `B01-Desktop-Linux-002` is not a development environment. Use it only after
   a new owner-activated evaluation lease and preregistered procedure. Never
   access or operate `B01-Clean-Linux-001`.
4. Product work must use a formal `P*-T*` task, one task branch, one Draft PR,
   one narrow lease, focused negatives, exact supported validation, and full
   merge/lease/branch closure.
5. Windows GNU may run TypeScript, documentation, consistency, diff, and Rust
   formatting only. Rust build/test/Clippy/runtime validation consumes a pushed
   exact revision on `DEV-LINUX-NATIVE-01` or supported CI.
6. Runner, broker, corpus, oracle, redactor, analysis, and cleanup assets are
   measurement-only. They must not become a second authority writer or add
   benchmark-only product authority.
7. No readiness work itself makes a performance, Gate, release, Profile, B01,
   or Agent-benefit claim. A readiness assessment may conclude only that a
   newly preregistered B0 is eligible to be requested.

## 4. Readiness definition and dependency order

The programme is complete only when every row below has supported evidence and
no unexplained `partial`, `not-run`, product gap, asset gap, broker gap, or
public-observation gap remains.

| Order | Readiness package | Required outcome before advancing | Current navigation state |
|---:|---|---|---|
| 1 | C1 public O-arm | WorkspaceRead and WorkspaceSearch each traverse public admit -> Context -> real Pi candidate -> scheduler lease -> daemon Tool executor -> independent verifier -> daemon acceptance. | **Complete:** P2-T36 merged as PR [#244](https://github.com/agentkernel/cognitive-os/pull/244) at `main@3efd7011`; independent fresh non-B01 Linux WorkspaceRead and WorkspaceSearch Tasks completed the full public chain. Final required CI `32245868452` passed Ubuntu, Windows, and required-ci. |
| 2 | C2a mutation O-arm | WorkspaceWrite/Patch carry schema-bound input and expected preimage through public authority; Intent/Effect, original-key reconcile, independent verification, and acceptance close the Task. | P2-T37/D02: public Write Task completed the full chain; public Patch Task remains. |
| 3 | C2b governed session-2 | A real user path proves daemon-authorized Memory/Skill consumption and resume without forged governance state or private helper use. | Reconfirm P2-T23 public consumption path with a real caller; register only an uncovered product gap. |
| 4 | C2c recovery | Controlled fixture crash/OUTCOME_UNKNOWN cases query by original key, reconcile, independently verify, and then accept or honestly remain unresolved. | Reconfirm P2-T24 plus production public caller; register only an uncovered product gap. |
| 5 | C2d public closure | Public observations distinguish admission, receipt, Effect closure, verification, and daemon acceptance, and demonstrate that acceptance closes the Task. | Reconfirm P2-T14/P2-T21 terminal evidence through each C1/C2 route; register only an uncovered observation/product gap. |
| 6 | Pure-Pi P arm | A same-fixture adapter works without daemon, Extension, Task, Context, Memory, Skill, retry, cache, or verifier; its credential route is approved and secret-safe. | No campaign asset work begins until product O-arm routes are supported; then create a permitted preparation task/lease. |
| 7 | Frozen paired assets | Runner, fixture corpus, oracle, redactor, analysis, reset, cleanup, command manifests, seeds, retry=0, timeout, arm order, and all digests are frozen. | No campaign samples. Assets require an explicit preparation boundary and must be independently reproducible. |
| 8 | B0 fairness readiness | A fresh B0 can prove P/O equality of tool set, input bytes, workspace, oracle, Provider/model, timeout, retry=0, environment, and cleanup. | Requires packages 1-7. A new EVAL ID is still not authorized by this plan. |
| 9 | New campaign readiness assessment | A non-claim assessment references all supported evidence, allocation strategy for a new root/port/SecretStore item, Provider budget, B1-B5 freezes, and cleanup. | Must state only that a new B0 may be requested; it cannot activate or execute a campaign. |

## 5. Completed task route: P2-T36

### Objective

Closed the smallest missing C1 product prerequisite: real Pi can select only
daemon-governed WorkspaceRead/Search on the public `cognitive daemon start`
composition, and the selected candidate reaches the existing daemon authority,
executor, verifier, and acceptance chain.

### Done only when

- [x] Pi native filesystem and shell tools remain default-deny; the Extension's
  `WorkspaceRead` handler remains I/O-free and candidate-only.
- [x] Adapter event extraction accepts exactly one valid WorkspaceRead/Search call,
  refuses Pi built-ins, unknown tools, duplicate Workspace calls, and mixed
  JSON/tool candidates.
- [x] A supported exact revision proves the public production caller rather than a
  SQLite injection, private transport injection, mock authority, or test-only
  caller.
- [x] The supported evidence records separate facts for candidate validation,
  scheduler lease, dispatch, Effect closure, verification, and acceptance.
- [x] The task's required CI, handbook synchronization, acceptance mapping, PR,
  merge, lease closure, and branch/main reconciliation are complete.

### Closure record

1. PR [#244](https://github.com/agentkernel/cognitive-os/pull/244) merged at
   `main@3efd7011b605a32ac0c9ec114321831995f32d90`.
2. Final documentation-head workflow `32245868452` completed successfully on
   Ubuntu and Windows, including `required-ci`.
3. The task lease is closed and archived; local `main` equals `origin/main` and
   the local and remote task branch are deleted.
4. Product evidence remains bounded to C1 readiness: it does not promote C2,
   EVAL, Gate, release, Profile, B01, or Agent-benefit conclusions.

## 6. Environment configuration checklist

This checklist configures supported development evidence, not B01 or a
benchmark campaign:

- [ ] Branch revision is pushed to GitHub and the remote worktree checks out the
  exact commit.
- [ ] A disposable remote root exists outside B01 campaign roots and contains a
  Git worktree, not a copied local source tree.
- [ ] Linux native checks record Rust, Node, pnpm, Pi, adapter, and Extension
  versions/digests appropriate to the task.
- [ ] Any Provider-dependent product path uses only the approved SecretStore;
  no secret is inspected, logged, or placed in a command argument.
- [ ] Product test fixtures are controlled and cleanable; no B01 guest state,
  closed campaign root, port, SecretStore item, runner, corpus, oracle, or
  evidence denominator is reused.
- [ ] Cleanup removes task-created runtime/process state and records only
  redacted facts.

## 7. Future task-selection rules

After P2-T36 closes, select the first unresolved package in section 3 that has
all `implementation_requires` satisfied. First run an existing supported
public path where it exists. Register a new formal task only when that run
proves a real product, public-observation, or environment-qualification gap.

Do not create implementation tasks for a campaign-only runner, broker, corpus,
or report until product paths are supported and the formal preparation boundary
permits it. Do not start Provider benchmark samples during any product task.

## 8. Completion gate

Before reporting readiness, perform one evidence-indexed assessment against
the final checklist in the owner instruction: new-EVAL isolation strategy;
merged supported product revision; C1 and C2a-d public O-arm paths; pure-Pi
P-arm and credential route; frozen assets/digests; B0 fairness; B1-B5 freezes;
and no unresolved non-pass disposition. The assessment must explicitly say:

> A newly preregistered B0 may be requested. No paired benchmark, performance,
> Agent-benefit, Gate, release, Profile, or B01 conclusion has been produced.
