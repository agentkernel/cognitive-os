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
smallest independently closable increment inside that task. Slice IDs use the
form `<task-id>/DNN` (for example, `P2-T03/D04`). The formal plan owns each
slice's purpose, dependencies, exit criteria and required validation. The
`PROGRESS.md` Current snapshot owns its current status. A slice status is not a
shadow task status and cannot promote a Gate, release or Profile claim.

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
- After one enabling/foundation slice, the next slice must connect it to a
  real caller or durable outcome. A second consecutive helper-only slice
  requires a bounded blocker record naming the missing implementation
  prerequisite and its owner.
- When a task has enough primitives for an end-to-end path, the next slice is
  vertical-closure-first: wire the real call chain before adding another
  horizontal helper, parser, or boundary.
- A completed slice is not a reason to open a parallel slice over the same
  path. The slice's exact lease and handoff are the coordination boundary.

The progress view must report three independent layers: formal task status,
delivery-slice status, and Gate/campaign status. A task can remain
`in-progress` while several slices are done; conversely, a slice can be
`blocked` without blocking unrelated tasks whose `implementation_requires` are
satisfied.

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
4. If the selected supported environment is unavailable, record the affected
   validation as `blocked` or `not-run` and choose an unrelated ready Slice.
   Do not first reproduce the known GNU linker failure and do not close a
   Slice from formatting/consistency alone.

The canonical capability registry is
[`PERSONAL-TEST-ENVIRONMENTS.md`](../plan/PERSONAL-TEST-ENVIRONMENTS.md).
Handoffs may record an execution result but cannot redefine these routing
rules.

### Before commit

- observe the intended failure-first test fail for behavior changes;
- run affected package tests and focused negative/regression tests;
- run affected lint and formatting checks;
- run secret/diff checks applicable to the changed paths;
- have no known failure in behavior affected by the commit.

### Before push

- run broader relevant regressions and repository consistency checks where the
  supported local toolchain is available;
- record every required check that was not run and why;
- inspect staged paths and the complete branch push surface.

### Before merge or task completion

- all required protected-branch CI checks are green;
- no required failure is unresolved;
- canonical task status, current progress snapshot, and handoff are reconciled.

Before delivery-slice closure, the author must also reconcile the slice ID,
its exact exit checklist, the strongest actual evidence level, and the next
executable slice. If a required environment is unavailable, close the
implementation lease only with a `blocked` slice record; do not label the
slice `done` merely because formatting or consistency checks passed.

A commit may exist while remote CI is pending. Unsupported local environments
are recorded as `not-run`; they are neither pass evidence nor an automatic
blocker for an isolated commit. A required red check must never be merged or
used for a completion claim.

## 4. Documentation closure

Implementation and closure documentation belong to the same atomic delivery or
PR, not necessarily the same Git commit. One implementation commit may be
followed by one closure documentation commit that records its immutable hash,
test evidence, non-claims, and remote visibility.

Update a handoff when transferring or ending a task/session, not after every
mechanical commit. Handoffs carry operational continuity but never override the
formal task plan or Gate ledger.

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

A merged PR must close its lease in the same closure delivery. If a merged
branch is later found still active, the first non-overlapping governance
session may move that row to closed while preserving the merged work and all
unrelated lease rows.

## 7. Forward-progress protocol

After onboarding, a session must select one smallest deliverable slice with a
clear exit. The session should produce one of:

1. a vertical implementation slice with focused verification;
2. a failure-first regression or negative test followed by the fix;
3. a verifiable governance/documentation correction; or
4. a bounded blocker record with `blocked_paths`, `blocked_task_ids`,
   `blocked_gate_ids`, owner, evidence, and the next executable action.

The selected slice must be registered before implementation begins, must have
one primary lease, and must name its vertical consumer or durable exit. If the
selected slice is blocked by validation infrastructure, the session must do
one of two things: perform the predeclared validation on the supported
environment, or record the bounded blocker and select an unrelated ready slice
whose implementation dependencies are satisfied. It must not create another
same-task helper slice to avoid the blocker.

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
