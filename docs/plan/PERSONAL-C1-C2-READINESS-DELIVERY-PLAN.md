# Personal C1/C2 Paired Benchmark Readiness Delivery Plan

- Status: active navigation document; no evaluation campaign is active
- Purpose: preserve the dependency-ordered delivery route to a **new**, not yet
  activated, C1/C2 paired benchmark readiness assessment
- Current product task: `P2-T36` on
  `personal/P2-T36-c1-public-production-path`
- Current C1 implementation checkpoint: `6dd704f5ee9dcfce59519b2f3922cc4e568e00ab`
- Last reconciled: 2026-08-18

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
| Plan persistence | complete | This navigation document is registered in the active P2-T36 lease and linked from `PROGRESS.md`. | Keep revision and row status current during the task. | Read this board after `PROGRESS.md`, lease, and branch reconciliation. |
| P2-T36/D01: Pi candidate surface | complete | WorkspaceRead is I/O-free and daemon-governed; native Pi filesystem/shell tools remain denied. Exact Linux adapter protocol test passed **21/21**. | Unit protocol evidence is not a public C1 route. | Preserve existing negatives while testing the real public route. |
| P2-T36/D01: exact source and base tools | complete | A cleanable, non-B01 native Linux Git worktree was shallow-cloned from the pushed branch. Rust adapter/admin CLI, Extension, and pinned Pi `0.81.1` build/version checks passed. | Native worktree must be refreshed to the exact current pushed head before final D02/D03 validation. | Read the branch head from `PROGRESS.md`, fetch/check it out from Git, and record exact equality before each final validation unit. |
| P2-T36/D02: disposable Pi configuration | complete | Public `cognitive pi configure` created a cleanable non-B01 runtime configuration referencing the pinned Pi, built Extension, and candidate adapter. No Provider material was configured or inspected. | Configuring Pi neither starts the daemon nor proves readiness or candidate execution. | Start the public daemon in that disposable runtime and capture only redacted status/doctor facts. |
| P2-T36/D02: public C1 WorkspaceRead/Search | in progress | The required candidate surface and supporting adapter protocol exist. Draft PR [#244](https://github.com/agentkernel/cognitive-os/pull/244) is open. | No real public daemon-start path has yet recorded both WorkspaceRead and WorkspaceSearch through candidate validation, scheduler lease, dispatch, verification, and acceptance. | Use the exact native Git worktree to run one bounded, cleanable public C1 route; append each outcome immediately to the running report. |
| P2-T36/D02: provider/readiness boundary | not started | No secret was read, printed, searched, logged, or placed in command arguments. | Pi launch is permitted only after daemon-owned readiness admits it; missing approved SecretStore readiness is a truthful `not-run`, not a workaround target. | Query public redacted readiness only after daemon start; use an approved SecretStore path only if the task route requires it. |
| P2-T36/D03: supported validation and CI | in progress | Initial Ubuntu verification passed. A previous Windows job failed on an unrelated existing reconciliation test and was retried. | Latest CI for the current pushed head is pending; no CI result may be inferred. | Observe the current CI revision; repair only task-owned failures and record unrelated failures precisely. |
| P2-T36/D03: task closure | not started | Draft PR and incremental report exist. | Full acceptance mapping, required validation, ready/merge, lease closure, branch cleanup, and `main` reconciliation remain. | Begin closure only after the public C1 route and required validation pass on the same pushed task head. |
| C2a-C2d public O-arm | blocked by P2-T36 dependency | Existing P2-T21 through P2-T24 provide product evidence for candidate parameters, mutation, consumption, recovery, and observations. | Their real-Pi/public-path readiness must not be inferred from prior unit, fixture, or campaign evidence. | After P2-T36 closes, run the first supported public path and register the smallest uncovered product gap. |
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
| 0. Navigation and ownership | complete | This plan is linked from `PROGRESS.md` and is an exact active-lease path. | Current task, lease, revision, report, and next action reconcile. | Reconcile canonical sources before changing the table. |
| 1. P2-T36 failure-first surface proof | complete | The missing `WorkspaceRead` Extension registration was observed as the expected failure before implementation. | A focused regression proves the new surface and native Pi deny policy. | Preserve the focused negative while changing D02. |
| 2. WorkspaceRead candidate implementation | complete | Pi Extension registration, adapter event extraction, and protocol negatives are implemented; exact native adapter protocol test is **21/21 pass**. | WorkspaceRead and WorkspaceSearch can each produce one untrusted, schema-bound candidate while built-ins fail closed. | Exercise this implementation only through the real public composition. |
| 3. Exact native source and toolchain | complete | Cleanable non-B01 Git worktree at `15557d18`; Rust 1.97.1, Node 22.19.0, and Pi 0.81.1 were verified for the initial D02 environment. | Before final evidence, checkout an exact pushed revision and record equality. | Refresh the cleanable worktree to the current pushed head before final validation. |
| 4. Public Pi configuration | complete | `cognitive pi configure` created a non-secret, cleanable non-B01 `pi.json` using the pinned Pi, Extension, and adapter. | Daemon-owned readiness can inspect the configuration without direct Provider or secret handling by Pi. | Start the public daemon in the disposable runtime. |
| 5. Daemon-owned readiness | in progress | No Provider or secret material has been accessed. | Public daemon start succeeds and public redacted doctor/status shows whether Pi launch is admitted. | Start `cognitive daemon start`; record only redacted status/doctor facts. |
| 6. C1 WorkspaceRead public route | not started | Candidate and configuration prerequisites are present. | Public admit -> Context -> real Pi candidate -> lease -> executor -> verifier -> acceptance is separately observable. | Run one bounded WorkspaceRead route after readiness permits it. |
| 7. C1 WorkspaceSearch public route | not started | Search candidate path existed before P2-T36 and remains covered by adapter negatives. | Same public lifecycle completes without borrowing WorkspaceRead evidence. | Run one bounded WorkspaceSearch route after the Read route is recorded. |
| 8. Required CI and supported regressions | in progress | Initial Ubuntu verification passed; Windows retry is pending for the current branch. | Required CI is green on the final task head, or an independently reproducible task-owned defect is repaired. | Monitor the current PR checks and classify any failure precisely. |
| 9. P2-T36 closure | not started | Draft PR and running report exist. | Acceptance mapping, report, handbook/docs sync, ready/merge, lease close, branch cleanup, and main reconciliation all complete. | Begin only after stages 5-8 pass on the same final pushed head. |
| 10. C2 and paired-benchmark readiness | not started | C2a-d, P arm, frozen assets, fairness B0, and a new campaign assessment remain future dependencies. | Every section 4 package has supported evidence and no unresolved non-pass result. | Select the first dependency-ready package only after P2-T36 closure. |

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
| 1 | C1 public O-arm | WorkspaceRead and WorkspaceSearch each traverse public admit -> Context -> real Pi candidate -> scheduler lease -> daemon Tool executor -> independent verifier -> daemon acceptance. | `P2-T36` in progress; WorkspaceRead Pi advertisement and adapter extraction are implemented, but public exact-revision Linux evidence is pending. |
| 2 | C2a mutation O-arm | WorkspaceWrite/Patch carry schema-bound input and expected preimage through public authority; Intent/Effect, original-key reconcile, independent verification, and acceptance close the Task. | Reconfirm existing P2-T21/P2-T22 path with a real Pi/public caller; register a narrowly scoped product task only for an uncovered gap. |
| 3 | C2b governed session-2 | A real user path proves daemon-authorized Memory/Skill consumption and resume without forged governance state or private helper use. | Reconfirm P2-T23 public consumption path with a real caller; register only an uncovered product gap. |
| 4 | C2c recovery | Controlled fixture crash/OUTCOME_UNKNOWN cases query by original key, reconcile, independently verify, and then accept or honestly remain unresolved. | Reconfirm P2-T24 plus production public caller; register only an uncovered product gap. |
| 5 | C2d public closure | Public observations distinguish admission, receipt, Effect closure, verification, and daemon acceptance, and demonstrate that acceptance closes the Task. | Reconfirm P2-T14/P2-T21 terminal evidence through each C1/C2 route; register only an uncovered observation/product gap. |
| 6 | Pure-Pi P arm | A same-fixture adapter works without daemon, Extension, Task, Context, Memory, Skill, retry, cache, or verifier; its credential route is approved and secret-safe. | No campaign asset work begins until product O-arm routes are supported; then create a permitted preparation task/lease. |
| 7 | Frozen paired assets | Runner, fixture corpus, oracle, redactor, analysis, reset, cleanup, command manifests, seeds, retry=0, timeout, arm order, and all digests are frozen. | No campaign samples. Assets require an explicit preparation boundary and must be independently reproducible. |
| 8 | B0 fairness readiness | A fresh B0 can prove P/O equality of tool set, input bytes, workspace, oracle, Provider/model, timeout, retry=0, environment, and cleanup. | Requires packages 1-7. A new EVAL ID is still not authorized by this plan. |
| 9 | New campaign readiness assessment | A non-claim assessment references all supported evidence, allocation strategy for a new root/port/SecretStore item, Provider budget, B1-B5 freezes, and cleanup. | Must state only that a new B0 may be requested; it cannot activate or execute a campaign. |

## 5. Current task route: P2-T36

### Objective

Close the smallest missing C1 product prerequisite: real Pi can select only
daemon-governed WorkspaceRead/Search on the public `cognitive daemon start`
composition, and the selected candidate reaches the existing daemon authority,
executor, verifier, and acceptance chain.

### Done only when

- Pi native filesystem and shell tools remain default-deny; the Extension's
  `WorkspaceRead` handler remains I/O-free and candidate-only.
- Adapter event extraction accepts exactly one valid WorkspaceRead/Search call,
  refuses Pi built-ins, unknown tools, duplicate Workspace calls, and mixed
  JSON/tool candidates.
- A supported exact revision proves the public production caller rather than a
  SQLite injection, private transport injection, mock authority, or test-only
  caller.
- The supported evidence records separate facts for candidate validation,
  scheduler lease, dispatch, Effect closure, verification, and acceptance.
- The task's required CI, handbook synchronization, acceptance mapping, PR,
  merge, lease closure, and branch/main reconciliation are complete.

### Current recovery sequence

1. Confirm branch/upstream and pushed exact head from `PROGRESS.md`.
2. Create or update the task Draft PR before remote validation.
3. On `DEV-LINUX-NATIVE-01`, create a disposable Git worktree from the exact
   pushed revision. Record `git rev-parse HEAD` and verify equality.
4. Run focused adapter and kernel-server tests, then the public daemon-start
   C1 integration path. Retain outcomes in the P2-T36 running report as each
   validation unit completes.
5. If a failure is product-owned, repair it inside P2-T36 when it fits the
   formal acceptance; otherwise register the smallest next formal task before
   editing unrelated paths.

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
