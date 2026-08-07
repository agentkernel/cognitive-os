# CognitiveOS Development Operating Model

- Status: active repository governance
- Effective date: 2026-07-30
- Scope: implementation work, evidence recording, product Gates, release and
  Profile claims in this repository

This document is the tool-neutral source for day-to-day development workflow.
Editor-specific rules may summarize or link to it, but cannot silently impose a
stricter task-status, evidence, or workflow interpretation.

Repository identity and the only active product project are defined by
[PROJECT-IDENTITY.md](PROJECT-IDENTITY.md). At present all implementation work
defaults to `cognitiveos-personal`; CognitiveOS specifications and reusable
kernel assets are its architecture and contract foundation, not a second
parallel product backlog.

## 0. Sources of truth

Each fact has exactly one canonical owner:

| Fact | Canonical source | Other documents may do |
|---|---|---|
| repository/project identity | `docs/governance/PROJECT-IDENTITY.md` | link or summarize |
| workflow and evidence semantics | this document | link or summarize |
| Personal tasks, acceptance, and Gates | `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md` | reference task IDs |
| current task/Gate/claim snapshot | `docs/plan/PROGRESS.md` `Current snapshot` | preserve dated history |
| delivery-slice definitions and exits | `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md` | reference slice IDs; do not create a second definition |
| current delivery-slice queue and status | `docs/plan/PROGRESS.md` `Current snapshot` | preserve dated history; do not copy into the formal plan |
| active writable ownership | `docs/plan/PARALLEL-LANES.md` active lease table | reference a lease ID |
| stable Personal product intent and UX | `docs/product/personal/` | link tasks/Gates without copying status |
| Personal architecture composition | `docs/architecture/personal/` | explain registered contracts and accepted decisions without creating them |
| detailed research and task cards | root `plan.md` | provide non-current detail |
| operational continuity | latest matching handoff | record `status_at_handoff` only |

When two documents disagree, update the non-canonical copy or label it as a
dated historical fact. Do not create another current-status table.

## 1. Independent progress dimensions

Every product task is tracked independently across these dimensions:

| Dimension | Allowed values | Meaning |
|---|---|---|
| `task_status` | `not-started`, `in-progress`, `blocked`, `done`, `cancelled` | Whether task work has begun and whether its complete acceptance is satisfied |
| `development_track` | `production-path`, `experimental-local-only` | Where incomplete work may execute |
| `implementation_evidence` | `none`, `provided`, `tested-local`, `tested-supported-ci` | Strongest implementation-level evidence actually obtained |
| `gate_status` | `not-run`, `running`, `pass`, `fail`, `blocked` | Result of a predeclared product campaign |
| `claim_scope` | `non-claim`, `product-gate`, `release`, `profile` | Maximum statement supported by current evidence |

Rules:

1. `not-started` means no task-specific implementation or test slice has begun.
   Pure research, discussion, or an uncommitted planning draft does not change
   task status. Starting the first real task-specific implementation or test
   slice, including a failure-first test, changes the task to `in-progress`;
   the formal plan must be reconciled in the same atomic delivery.
2. `done` requires the task's complete acceptance criteria. It does not imply a
   product Gate, release, or Profile result.
3. Local, fixture, WSL, or ordinary CI evidence may advance
   `implementation_evidence`; it cannot advance a formal `gate_status` unless
   it is part of that Gate's predeclared environment and campaign.
4. A Gate may remain `not-run` while its enabling task is `in-progress` or
   `done`.
5. Status summaries must report these dimensions separately. A shadow status
   must not replace the canonical task status.

## 2. Typed dependencies

Dependencies use three meanings:

- `implementation_requires`: code or contracts required before isolated work
  can begin;
- `acceptance_requires`: evidence required before the task can become `done`;
- `promotion_requires`: Gates required before release or claim expansion.

An acceptance or promotion dependency is not an implementation mutex. Work may
proceed on `experimental-local-only` when its `implementation_requires` are
satisfied and all secret, authority, contract, and non-claim boundaries remain
intact.

## 2.1 Delivery slices and continuous forward progress

A formal `P*-T*` task is the acceptance boundary; a delivery slice is the
smallest internally checkable increment inside that task. The default delivery,
branch, PR, lease, reporting and completion boundary is the **whole formal
task**, not a slice. Slice IDs use the
form `<task-id>/DNN` (for example, `P2-T03/D04`). The formal plan owns each
slice's purpose, dependencies, exit criteria and required validation. The
`PROGRESS.md` Current snapshot owns its current status. A slice status is not a
shadow task status and cannot promote a Gate, release or Profile claim.

**`TASK-ATOMIC-DELIVERY-01`:** after claiming a formal task such as `P2-T04`,
the owner continues through every required slice, real integration, focused
negative, supported validation, acceptance assessment and documentation closure
in one task branch, one Draft PR and one task-scoped lease. A slice completion,
checkpoint commit, push, CI submission, intermediate validation result or
recoverable environment failure is not a stopping or reporting boundary. Work
stops only when the task is honestly `done`, the owner explicitly pauses or
changes scope, unexpected concurrent changes appear, or a concrete external
blocker cannot be removed autonomously.

Allowed slice statuses are `ready`, `in-progress`, `blocked`, `done`, and
`cancelled`. A slice may be `done` only when all of the following are true:

1. it produces a user-visible path, a durable authority fact, a real
   integration boundary, or a closed correctness property;
2. its focused failure-first or negative test and the required supported
   validation have actually run and passed, or the formal slice definition
   explicitly names a non-executable documentation-only exit;
3. its evidence, non-claims, immutable revision and next dependency are
   recorded in the same delivery;
4. it does not silently leave a required consumer, writer, reconciliation
   path or rollback path as an unowned helper-only TODO.

An implementation may exist while its slice is `blocked` when required
validation has not run. `not-run` is evidence, not completion. Unsupported
local toolchains do not automatically block an isolated implementation, but a
slice whose exit requires Rust/runtime validation must transfer that validation
to the predeclared supported Linux/CI environment before it is closed.

The following anti-fragmentation rules apply:

- One formal task has at most one `in-progress` delivery slice at a time.
- A task uses one branch, one evolving Draft PR and one active task lease.
  Slices do not receive separate branches, PRs or routine handoffs. Existing
  legacy multi-branch work migrates at its next safe continuation boundary and
  must not create another fragmented branch.
- After one enabling/foundation slice, the next slice must connect it to a
  real caller or durable outcome. A second consecutive helper-only slice
  requires a bounded blocker record naming the missing implementation
  prerequisite and its owner.
- When a task has enough primitives for an end-to-end path, the next slice is
  vertical-closure-first: wire the real call chain before adding another
  horizontal helper, parser, or boundary.
- A completed slice is not a reason to open a parallel slice over the same
  path. The task's single lease, branch and Draft PR remain the coordination
  boundary through complete acceptance.
- When a slice exit passes, immediately proceed to the next unmet task
  acceptance item. Do not defer a separate `acceptance-assessment` branch or
  leave task closure as an unowned follow-up. The final slice owns the complete
  acceptance mapping and task-status reconciliation.
- **Continuous autonomous delivery:** when an owner requests continued
  development, the agent must autonomously select and implement the next
  smallest vertical slice within the active task and lease. It continues until
  task completion, an explicit non-autonomous blocker, or an owner-requested
  pause/scope change. Checkpoint commits, CI submission, intermediate
  validation results, or recoverable environment failures are workflow events,
  not stop conditions. The agent records them while proceeding to the next
  implementation, repair, or registered validation action. It must not stop
  merely to provide a progress summary unless a decision is needed, unknown
  concurrent changes appear, or the owner asks for one.

### 2.1.1 MVP-first implementation and authorization

The first runnable product path uses the least complex authorization model that
can satisfy the current task acceptance and the repository's immutable safety
boundaries. The default MVP is owner-local, single-principal, task-scoped and
daemon-issued. Requests outside that bounded path fail closed. Full RBAC,
approval chains, generic capability administration, multi-tenant policy
languages and speculative extension frameworks are deferred unless the formal
task explicitly requires them or the minimal path cannot preserve a registered
threat boundary.

MVP-first does not weaken authority separation. The daemon remains the only
writer; secrets remain in an approved Secret Store; external mutation remains
persist-before-dispatch Intent/Effect work; budget, fencing, idempotency,
reconciliation, audit and independent verification remain mandatory where the
operation uses them. Prefer the existing management session, task bearer and
private application services. A new public contract or generic policy subsystem
requires evidence that the narrow private composition cannot safely satisfy the
real caller and current acceptance.

The progress view must report three independent layers: formal task status,
delivery-slice status, and Gate/campaign status. A task can remain
`in-progress` while several slices are done; conversely, a slice can be
`blocked` without blocking unrelated tasks whose `implementation_requires` are
satisfied.

## 2.2 Git delivery states and recoverable checkpoints

**`CHECKPOINT-DELIVERY-01`: Checkpoint persistence is not task closure.** Git
delivery state is a fourth operational dimension and does not replace task,
Slice, evidence or Gate status:

| Git delivery state | Purpose | What it does not mean |
|---|---|---|
| coherent worktree | locally reviewable change owned by one task lease | durable or remotely recoverable |
| checkpoint commit | immutable, scoped and secret-free recovery point needed by remote validation or recovery | Slice/task `done`, a report boundary or CI pass |
| pushed checkpoint | exact revision available to CI/Linux and another window | ready to merge |
| Draft PR | review/CI container for one active formal task | approval to merge or close the lease |
| ready PR | complete task acceptance submitted for final review | automatic Gate/release/Profile promotion |
| merged closure | accepted task delivery with lease closed and branch reconciled | product Gate pass unless separately proven |

An active task may accumulate coherent checkpoint commits on its dedicated task
branch only when remote CI, exact-revision Linux validation or abnormal recovery
needs an immutable revision. Checkpoints are background persistence events, not
reasons to pause, report, create a handoff or split the task. The repository
owner grants standing delivery authorization: after locally eligible safeguards
pass, the agent automatically commits, pushes and creates or updates the same
Draft PR without asking again in each window, unless the user explicitly pauses
delivery. Required remote CI is a precondition for ready/merge and task closure,
not for persisting a checkpoint. A Draft PR remains Draft until the complete
task acceptance is satisfied.

One task PR may graduate from Draft to ready only after all task acceptance and
all required slice exits are present. When focused negative tests, supported
validation, required CI, final acceptance mapping, evidence synchronization and
review requirements all pass, standing authorization also allows the agent to
mark the PR ready and merge it without another routine confirmation. The
authorization never permits merging an incomplete task,
bypassing a failed/pending check, resolving a product/normative decision or
force pushing. Direct feature work on `main` and merging an explicitly
incomplete checkpoint are prohibited. Multiple formal tasks must not share one
branch or PR; if legacy work already does, split future continuation at the next
safe boundary without rewriting or discarding existing history.

### 2.3 Standing operator authorization

The repository owner grants the agent standing authority to decide and execute
operations needed by an active leased Personal task without per-operation
confirmation. This includes disposable remote validation, approved Secret Store
access, least-privilege elevation, and scoped user-service or system
configuration changes. The authority is operational, not evidentiary: every
remote build still uses a pushed exact revision; every action remains within the
task lease; and Gate, release, Profile, task-completion, and campaign claims
remain governed by their separately registered evidence requirements.

The authorization does not relax secret handling: secret material must never be
placed in arguments, ordinary configuration, SQLite, logs, CI output, tests, or
evidence. Operations must be narrowly scoped, auditable, and use an available
rollback or cleanup path. It does not authorize force push, destructive or
irreversible repository commands, modification of isolated formal campaign
guests outside their preregistered procedure, or changes beyond the active
task's declared system boundary.

A coherent checkpoint must satisfy all locally eligible safeguards: declared
lease scope, reviewed staged paths, no secret material, no unexpected syntax or
behavior failure, and explicit `not-run`/blocker records for validation that
requires another environment. An intentionally failing failure-first checkpoint
is allowed only when its failure is isolated, expected and named in the task's
Draft PR; it cannot be ready or merged.

Native Linux and remote CI consume pushed immutable revisions. Copying an
uncommitted tree, testing a stale remote snapshot or reporting a local diff as
the tested revision is invalid. If a task PR is accidentally merged before task
acceptance is complete, preserve the honest task and Slice statuses, close the
merged branch lease, record the premature merge, and require one task-scoped
continuation branch and lease. Do not rewrite history or retroactively claim
closure.

## 3. Validation stages

### 3.0 Command shell and environment routing

Command syntax and validation environment are preconditions, not discoveries
to repeat inside every delivery:

1. **`COMMAND-SHELL-PS51`:** the repository's current local Cursor Shell on
   Windows is parsed by Windows PowerShell 5.1. Do not use `&&` or `||` in
   local commands. Independent commands use separate parallel tool calls;
   dependent commands use separate calls or
   `if ($LASTEXITCODE -eq 0) { <next-command> }`. A parser rejection before a
   process starts is `not-run`, not a failed build or test.
2. **`RUST-LINK-DEV-WIN-GNU-01`:** the current local
   `x86_64-pc-windows-gnu` host is a registered unsupported Rust linking
   environment. Workspace build, test, Clippy, run and bench commands are
   known to stop at linker exit 121. Do not repeat them, and do not retry the
   exhausted LLVM-MinGW/shim/PATH/toolchain-pin workarounds, unless an explicit
   P0-T01 toolchain-repair Delivery Slice has been approved and leased.
3. On `DEV-WIN-GNU-01`, only non-linking work is eligible: Rust formatting,
   documentation/static consistency, Node/TypeScript checks and diff checks.
   Rust build/test/Clippy validation must be routed before implementation to
   `CI-UBUNTU-01`, `CI-WINDOWS-MSVC-01`, or an exact-revision disposable
   worktree on `DEV-LINUX-NATIVE-01`, according to the Slice's evidence need.
4. If the selected supported environment is unavailable, try the other
   predeclared supported route and continue the same task. If no supported
   route can run, record the affected validation as `blocked` or `not-run` and
   stop only with a bounded external blocker. Do not switch tasks merely to
   avoid closure, reproduce the known GNU linker failure, or close a Slice from
   formatting/consistency alone.

The canonical capability registry is
[`PERSONAL-TEST-ENVIRONMENTS.md`](../plan/PERSONAL-TEST-ENVIRONMENTS.md).
Handoffs may record an execution result but cannot redefine these routing
rules.

### Before checkpoint commit

- observe the intended failure-first test fail for behavior changes;
- run affected package tests and focused negative/regression tests available on
  the selected environment;
- run affected lint and formatting checks;
- run secret/diff checks applicable to the changed paths;
- have no unexpected known failure in behavior affected by the commit;
- inspect and stage only paths owned by the active lease.

### Before checkpoint push or Draft PR update

- run broader relevant regressions and repository consistency checks where the
  supported local toolchain is available;
- record every required check that was not run and why;
- inspect staged paths and the complete branch push surface.
- record the full commit hash and confirm that remote validation will consume
  that exact revision;
- keep the PR Draft while any task acceptance or formal Slice exit item remains
  incomplete.

### Before ready, merge or task completion

- all required protected-branch CI checks are green;
- no required failure is unresolved;
- every formal task acceptance item, Slice exit and supported validation is
  satisfied and mapped to exact evidence;
- canonical task status, current progress snapshot and final task handoff are
  reconciled;
- the branch contains only the task's owned changes and no unresolved or
  unowned worktree state;
- the PR is explicitly changed from Draft to ready only after these facts are
  true; auto-merge must not bypass this transition.

Before delivery-slice closure, the author must also reconcile the slice ID,
its exact exit checklist, the strongest actual evidence level, and the next
executable slice. If a required environment is unavailable, close the
task lease only when the task itself is complete; otherwise retain it or close
it with a bounded `blocked` task record and a single recovery action. Do not
label the slice or task `done` merely because formatting or consistency checks
passed.

A checkpoint commit and Draft PR may exist while remote CI is pending.
Unsupported local environments are recorded as `not-run`; they are neither pass
evidence nor an automatic blocker for an isolated checkpoint. A required red
check must never be merged or used for a completion claim.

## 4. Documentation closure

Implementation and closure documentation belong to the same whole-task
delivery and PR, not necessarily the same Git commit. Intermediate checkpoints
do not each require closure documentation. One final closure documentation
commit records the accepted immutable hash, complete test evidence, non-claims
and remote visibility.

Create or update a handoff only when the complete task closes, the owner
explicitly pauses, ownership transfers, unknown changes appear, or a concrete
external blocker prevents further autonomous work. Do not create handoffs for
ordinary Slice exits, commits, pushes, CI rounds or recoverable failures.
Handoffs carry operational continuity but never override the formal task plan
or Gate ledger. A normal handoff records:

- Slice ID and honest status;
- branch, full HEAD hash and upstream branch;
- PR URL and Draft/ready/merged state;
- clean or dirty worktree state;
- implemented and remaining exit items;
- each required validation as `pass`, `fail` or `not-run` with environment;
- evidence boundary, non-claims and one next executable action.

Ending a session with coherent task changes only in the worktree is not the
normal path. A `dirty handoff` is allowed only for a non-coherent intermediate
state, unexpected pre-existing changes, an ownership/safety conflict, or an
explicit user pause. It must list every affected path, the reason it cannot be
checkpointed, checks already run, owner and one recovery action. Absent a user
pause, the standing delivery authorization requires automatic checkpoint
commit, push and Draft PR update instead of forcing the next window to
rediscover the worktree.

### Deterministic task closure

The final step of a task is part of implementation, not a later administrative
task. The owner must complete this sequence without opening a separate
acceptance-assessment branch:

1. map every formal acceptance item to implementation, focused negatives and
   evidence at the exact candidate HEAD;
2. fix and rerun every missing task-owned item, then complete supported
   validation and required CI;
3. reconcile the formal plan, `PROGRESS.md` Current snapshot, affected trace and
   one final handoff while preserving independent Gate/non-claim status;
4. verify the task branch and PR contain only declared task paths, change Draft
   to ready and merge normally without force push or history rewriting;
5. close the task lease, verify the PR is merged, delete the remote task branch
   when safe, switch the local worktree to `main`, fast-forward to the merge and
   verify clean status, expected HEAD/upstream and no active lease for the
   completed task.

If any step cannot complete, the task remains `in-progress` or `blocked` with
one explicit recovery action. “Implementation complete but acceptance pending”,
an unmerged ready branch, a merged PR with an active lease, or a local worktree
left on the completed task branch are not valid closure states.

### Fast resume protocol

A new window first verifies only the recorded recovery tuple: current branch,
worktree clean/dirty state, local HEAD, upstream HEAD, PR state/checks, active
lease and latest matching handoff. When that tuple is consistent, continue from
the handoff's next action without repeating a broad repository Git audit. Expand
inspection only if the tuple differs, the tree is dirty, the lease is missing,
the PR was merged/closed unexpectedly, or new unowned changes appear.

## 5. Change classification

- `implementation-only`: realizes or corrects an unchanged normative/product
  contract; requires implementation, focused tests, affected docs, and an
  explicit statement that the normative surface is unchanged;
- `corrective`: non-semantic drift, typo, count, or link repair;
- `product-semantic`: changes a Personal product version, supported platform,
  release scope, formal task acceptance, product Gate threshold, default Agent
  or adapter inclusion without changing a CognitiveOS public machine/behavior
  contract;
- `normative-semantic`: changes public behavior, DTO/schema, registered error,
  transition, vector expectation, or acceptance semantics;
- `structural`: adds/removes an object family, Profile, subsystem, or migration
  track.

Product-semantic changes require owner decision, a Personal ADR when release or
platform scope changes, and synchronized formal-plan/trace/support/campaign
updates. They do not require registry/schema/vector changes unless the public
CognitiveOS contract also changes. Only normative-semantic and structural
contract changes require Lane-CTR contract coordination. An
implementation-only or product-semantic change must not create a parallel DTO,
schema, error, transition or vector.

## 6. Ownership leases

Writable ownership is a time-bounded lease over declared paths, not permanent
ownership by a historical branch name. Each active lease records task, branch,
primary lane, owned paths, owner/session, claim time, and last heartbeat.

- Active leases must not overlap writable paths.
- One formal task has one active task-scoped lease, one branch and one Draft PR.
  A lease may list all narrow runtime, client, test and documentation paths
  needed for full task acceptance; Slice-specific leases are not created during
  ordinary continuation.
- One branch or PR must not carry multiple formal tasks. Existing legacy shared
  branches are preserved but must split future task continuation at the next
  safe boundary.
- Leases must name exact files or narrow feature directories. Broad
  `docs/plan/**`, `docs/standards/**`, `docs/adr/**`, `specs/**` or equivalent
  protected-tree ownership is invalid.
- `PARALLEL-LANES.md` itself uses a narrow coordination update: a session may
  add/heartbeat/close only its own row while preserving every unrelated row.
  The ledger cannot be exclusively owned through a parent-directory glob.
- One cohesive task may declare secondary paths across runtime, CLI, tests, and
  docs in one PR.
- Normative contract assets remain Lane-CTR-owned.
- Merged, abandoned, or stale leases become history and cannot block new work.
- Unexpected uncommitted changes remain protected until their owner resolves or
  explicitly releases them.

`PARALLEL-LANES.md` is the only active lease ledger. Every lease has a stable
`lease_id`, one of `active`, `closed`, `abandoned`, or `stale`, and timestamps
for claim and heartbeat. `PROGRESS.md` may only reference an active `lease_id`
or `none`; it must not maintain a second lease status table. Closed leases move
out of the active table and cannot block future work.

A Draft PR does not close its lease. A ready task PR that merges must close its lease
in the same closure delivery. If any merged branch is later found still active,
the first non-overlapping governance session may move that row to closed while
preserving the merged work and all unrelated lease rows. If the underlying task
is incomplete, it remains `in-progress` or `blocked` and needs one task-scoped
continuation branch/lease; the old merged lease must not be reused.

## 7. Forward-progress protocol

After onboarding, a session claims one formal task with complete acceptance and
a clear vertical path. It continuously executes the smallest internal slices
until it produces one of:

1. a fully accepted, validated and closed formal task;
2. a fully accepted corrective governance/documentation delivery; or
3. a bounded blocker record with `blocked_paths`, `blocked_task_ids`,
   `blocked_gate_ids`, owner, evidence, and the next executable action.

The selected task must be registered before implementation begins, must have
one task lease/branch/Draft PR, and its internal slices must name their vertical
consumer or durable exit. If one slice is blocked by validation infrastructure,
the session tries the other predeclared supported route and continues the same
task. Only when all supported routes or an external dependency are unavailable
may it stop with a bounded blocker. It must not create another same-task helper
slice, switch tasks to avoid closure, or treat an intermediate report as a
deliverable.

Re-reading plans, broad auditing, or creating another plan is not a deliverable
when the task, dependency, and safe path are already known. Acceptance and
promotion dependencies do not prevent implementation work. Conversely, an
actual missing `implementation_requires`, secret boundary, authority boundary,
overlapping lease, or unknown worktree change must fail closed only for the
affected paths and claims, not freeze unrelated Personal work.

## 8. Invariants that this model does not relax

1. Provider and user secrets remain in approved Secret Stores and never enter
   argv, ordinary config, SQLite, logs, CI, or evidence.
2. Pi, CLI, SDK, and UI remain clients; deterministic server code owns
   authorization, CAS, state transitions, budgets, idempotency, fencing,
   Effect commit/reconciliation, and final acceptance.
3. External mutations remain persist-before-dispatch Intent/Effect operations.
4. Task completion requires independent verification; process exit, Pi
   `agent_end`, Provider response, or external receipt alone is insufficient.
5. Contract changes use Lane-CTR and negative vectors cannot be weakened to fit
   an implementation.
6. WSL, fixture, fake-systemd, local smoke, and ordinary CI evidence cannot be
   promoted to native campaign, Gate, release, containment, or Profile claims.
7. Unknown worktree changes are never overwritten, reverted, staged, or mixed
   into another delivery.
