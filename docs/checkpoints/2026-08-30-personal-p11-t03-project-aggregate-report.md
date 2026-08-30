# P11-T03 Project aggregate walking skeleton — running report

- Task: `P11-T03` / slice `P11-T03/D01` (14 §10 five knives are **internal** steps, not separate branches/PRs/leases)
- Change class: `implementation-only` (Personal-private tables + private projection; no `core/specs`, no Lane-CTR)
- Product: CognitiveOS Personal 2.0.0 (v9 is a canvas filename only)
- Lease: closed (`lease/personal/P11-T03/project-aggregate` → PARALLEL-LANES §3.1)
- Branch: `personal/P11-T03-project-aggregate` (merged; delete after this commit)
- PR: [#281](https://github.com/agentkernel/cognitive-os/pull/281) **merged** as `main@46407380`
- HEAD: `main@464073809ffadf1f2c08e7391bbac5b4b2c0ed8b` (implementation `7d9f13e4`; tempfile `8374d560`; CI-unblock `aef5574e`; merge PR 281)
- Claim ceiling: `hypothesis` (A7: local/CI is not Gate/release/Profile)
- Evaluation routing: **OFF** (`PERSONAL-PERF-EVAL-015` closed)

## Recovery (2026-08-30 restart)

| Fact | Value |
|---|---|
| Git at claim | `main@6092f31abb43e20ac245197d5884e3f118fad4e2` tracking `origin/main`; no prior T03 branch/PR |
| Worktree | Dirty DOC-owned docs/handbook/skills — **protected, never staged** |
| Lease at claim | Only DOC `dev-prep` + `OPC-REFRAME`; no P11 implementation lease |
| Running report | This file created at claim (prior interruption wrote nothing durable) |
| Guidance present | 13, 14, 15, 16, 17, 18, 21, 22 |
| Guidance missing | 19, 20, 23, 24, 25, 26; `docs/plan/p11-plan-review-and-optimization.md` |

## Written analysis (required before product code)

### Applicable guidance

| Source | How T03 uses it |
|---|---|
| 14 §3 objects / G1 / G2 | Authoritative HOW: new table, not `family=task`; G1 mints Project + confirmed CharterRevision in `creating`; G2 writes AcceptanceFact → `active` |
| 14 §5 ④⑤ oracle | StageAcceptanceSpec storage; StageTestPassed is a **derived** fact; reuse P2-T14 `acceptance_decision` bytes; LLM-as-judge out of scope |
| 14 §7.8 | Routes `/management/project/v1/*`; Personal-private; do not touch P7-T05 frozen inventory |
| 14 §8 | Failure-first negatives N1–N16 (`p11_t03_*`); 22 号 cites these as T03-N1..N16 without renaming |
| 14 §9 | Rejected shortcuts — deviation requires a written reason **in this report first** |
| 14 §10 | D01 write order (knives 1–5 + projections + cadence declarations) |
| 15 号 | Cited only: §13 A5 attempt payload excludes verifier params (field isolation hook). Do not rewrite 15 |
| 16 号 | Preview-chain consumer only; no T06 assistant body |
| 17 号 | Employee seam: seated predicate fail-closed; roster projection empty + `employee-authority-not-implemented`. No T04 body |
| 18 号 | Conversation announcement seam: pending-previews list omits `preview_digest`; digest only on canvas `preview-detail`. No T05 body |
| 21 号 | Column-level tables, migration drop point, route fields, per-knife close sentences |
| 22 号 §2 | T03 close sentences per knife; N1–N16 remain 14 §8 text |

### 14 §3 objects this slice MUST land

`Project`, `CharterRevision`, `PlanRevision`, `Stage`, `Gap`
plus T03-required supporting rows from 14 §6.5 / 21 §2: `Draft`, `Candidate`, `ApprovalPreview` (subject_kind ∈ activation \| plan-change \| acceptance), `StageTestFact`, `AcceptanceFact`.

### 14 §3 objects this slice MUST NOT land

| Object | Owner |
|---|---|
| RoleBlueprint / Employee / EmployeeRevision / Grant body | T04 |
| Conversation new private version | T05 |
| Hidden Pi assistant | T06 |
| Hosted DSH engine | T07 |
| Routine arming / scheduler | T08 (cadence stored as declaration only) |
| Canvas HITL full text / time-box / narrow / stop | T09 |
| Vault / Memory admission | T10 / T11 |
| Full `/ui/` IA | T13 |
| Honest usage body (beyond unknown≠0 projection hook) | T12 |

### What this slice is NOT

Not a Today page. Not a six-family rename. Not an assistant skin. Not `/work` retrofit. Not Task-row impersonation. Not Vite product origin. Not chat Approve.

### 14 §8 negatives — write first / not this task

| ID | Test name | This task? |
|---|---|---|
| N1 | `p11_t03_project_is_not_a_task_row` | yes (knife 1) |
| N2 | `p11_t03_unconfirmed_activate_rejected` | yes (knife 1) |
| N3 | `p11_t03_stale_total_preview_rejected` | yes (knife 1 store + knife 2 route) |
| N4 | `p11_t03_cross_project_write_rejected` | yes (knife 1) |
| N5 | `p11_t03_gap_stage_cannot_confirm_or_test` | yes (knife 1 half + knife 3 remainder) |
| N6 | `p11_t03_completion_requires_current_verification` | yes (knife 4) |
| N7 | `p11_t03_missing_openable_artifact_blocks_pass` | yes (knife 4) |
| N8 | `p11_t03_unseated_stage_cannot_start_test` | yes (knife 4; empty seat table fail-closed) |
| N9 | `p11_t03_joint_acceptance_requires_all_stage_facts` | yes (knife 5) |
| N10 | `p11_t03_superseded_revision_confirm_rejected` | yes (knife 3) |
| N11 | `p11_t03_secret_shape_rejected_at_registration` | yes (knife 5) |
| N12 | `p11_t03_non_owner_principal_cannot_confirm` | yes (knife 2) |
| N13 | `p11_t03_draft_apply_wrong_base_seq_rejected` | yes (knife 2) |
| N14 | `p11_t03_unknown_cost_never_zero` | yes (knife 5 projection hook; honest usage body = T12) |
| N15 | `p11_t03_copy_excludes_secrets_and_inflight` | yes (knife 5) |
| N16 | `p11_t03_preview_survives_process_death` | store-reopen half in knife 5 if min preview table lands; process-death/Pi/DSH half → T09 |
| 15 A5 | `p11_t03_attempt_payload_excludes_verifier_params` | hook only (15 §13) |

Explicitly **not** done here: T04–T13 A/B/C/D catalogs in 22 号 §3–§12.

### 14 §9 shortcuts — none taken

No `/work` rename, no Task-row Project, no heartbeat authority, no chat Approve, no 0.1 reinterpretation, no Vite shell, no fake buttons.

### Recorded deviations from 21 (not from 14 §9)

1. **Circular FKs omitted** (`p11_project.current_charter_revision_id` ↔ `p11_charter_revision.project_id`). SQLite cannot insert both rows if both FKs are live. Integrity is enforced in the daemon writer. 14 semantics unchanged.
2. **`inactive` added to Project `state` CHECK.** 14 §3.3 and N15 require copy to land inactive; 21 §2 enum omitted it. Follow 14.
3. **N6/N7 oracle tests arrange seating facts as a function argument.** Production load of seating is always the empty table (N8). This is not a stub pass: production cannot mint `StageTestFact` without T04 seated rows.

### Path collisions (DOC)

| Path | Owner | T03 action |
|---|---|---|
| `docs/plan/PROGRESS.md`, `plan.md`, `PERSONAL-DEVELOPMENT-PLAN.md`, `personal-trace.yaml` | `DOC-PERSONAL-2.0.0/dev-prep` | **blocked_paths** — do not write; status stays DOC-owned until they adopt the lease row |
| `personal/handbook/` | `DOC-PERSONAL-2.0.0/dev-prep` tree lease | docs-sync-contract §2 required mapped v26/route pages in this changeset (not `DOCS_IMPACT_NONE`). Product/architecture handbook bodies that only fingerprint-drifted because of DOC dirty sources are **not** taken. `PROGRESS.md` still blocked. |
| `clients/docs/design/opc-2.0/`, `personal/docs/product/` | `DOC-PERSONAL-2.0-OPC-REFRAME` | do not write |
| `personal/docs/architecture/personal-2.0.0-dev-prep-index.md` | DOC dev-prep | do not write |

### Handbook delta (proposed; DOC must apply)

`personal/handbook/{en,zh-CN}/developer/store-and-migrations.md`: authority map **v1–v25 → v1–v26**; add row “v26 = Personal-private Project aggregate (`p11_project` / charter / plan / stage / gap / draft / candidate / approval_preview / stage_test_fact / acceptance_fact)”; nuance sentence “v18–v25” → “v18–v26”. Other store-mapped pages (memory-skill, operations-recovery, provider-control-plane) need **review, no fact change**.

`daemon-http` pages only if `server.rs` is wired; new file `project_aggregate.rs` is not in that source-map list.

## Recovery (2026-08-30, third session)

Previous subagents left knives 1–5 **implemented and staged, never committed**. This session recovers from disk/git only.

| Fact | Value |
|---|---|
| Branch | `personal/P11-T03-project-aggregate` (no upstream at recovery) |
| HEAD at recovery | `6092f31abb43e20ac245197d5884e3f118fad4e2` = `origin/main` |
| Draft PR | none at recovery |
| Lease | `lease/personal/P11-T03/project-aggregate` already in §3 |
| Evaluation routing | **OFF** |
| Protected dirty (not staged) | DOC product/architecture/PROGRESS; `.cursor/skills`; untracked skills/commands |

### Recorded deviations (additions)

4. **G2 `acceptance_decision_ref` is a daemon-authored JSON digest** (`schema_version` + `decision=granted` + project/plan ids) stored as `cas:{sha256}`. Walking skeleton does not open the filesystem ArtifactStore inside the SQLite aggregate (tests are layout-only). P2-T14 body shape is reused; full ArtifactStore `put` remains a T03 follow-up if CI proves the digest-only ref insufficient. Not a 14 §9 shortcut.
5. **Confirm chain is the Personal-private `/management/project/v1/{draft.apply,preview.request,confirm}` triple**, not `/task/intent.record`. 21 §4 forbids reusing P7-T05 inventory and Task intent as Project identity. Discipline (persist candidate → digest preview → admit) is the same; A3 Intent/Effect persist-before-dispatch is not used because G1/G2 are internal authority writes (14 §3).
6. **Mapped handbook pages + `tools/src/generate-handbook.mjs` are in this changeset** because `cognitive-store/**` and `server.rs` hit source-map (`dev.store-migrations`, `daemon-http`). This is docs-sync-contract §2, not a product-doc rewrite. `PROGRESS.md` / plan cards remain **blocked_paths** (DOC `dev-prep`). Did not fake `DOCS_IMPACT_NONE`.

## Recovery (2026-08-30, parent window after subagent disconnect)

Subagents died (resume-no-progress, connection-failed, then Claude unpaid invoice). This parent window recovered from disk/git only. No product-code rewrite.

| Fact | Value |
|---|---|
| Branch / upstream | `personal/P11-T03-project-aggregate` tracks origin at `8374d560` |
| Draft PR | [#281](https://github.com/agentkernel/cognitive-os/pull/281) still Draft |
| Evaluation routing | **OFF** |
| Knives 1–5 + `/management/project/v1/*` | already committed (`7d9f13e4` + tempfile lock `8374d560`) |
| Protected dirty | DOC product/architecture/PROGRESS; `.cursor/skills`; untracked 13–22 — **not staged** |

### Required CI on `8374d560` (run [33281002286](https://github.com/agentkernel/cognitive-os/actions/runs/33281002286))

**fail** `verify (ubuntu-latest)` / `verify (windows-latest)` / `required-ci` in ~2 min (TypeScript `check-consistency`; Rust workspace tests **not-run** on this job). Five consistency violations:

| # | File | Cause | Owner |
|---|---|---|---|
| 1 | `personal/docs/architecture/personal-2.0.0-dev-prep-index.md` | broken link to `opc-2.0/13-…md` (file untracked, not in git) | DOC `dev-prep` + OPC-REFRAME |
| 2 | `personal/docs/product/README.md` | committed absolute `C:\Users\wuron\.cursor\…v9.canvas.tsx` | OPC-REFRAME (already on `origin/main`) |
| 3 | `personal/docs/product/web-ui-design.md` | same absolute canvas path | OPC-REFRAME (already on `origin/main`) |
| 4 | `docs/plan/PROGRESS.md` | `active lease is not referenced: lease/personal/P11-T03/project-aggregate` | DOC `dev-prep` (**blocked_paths**) |
| 5 | `docs/plan/PROGRESS.md` | `CURRENT_SNAPSHOT_LEASE_MISMATCH`: `P11-T03/D01` is `ready` not `in-progress` | DOC `dev-prep` (**blocked_paths**) |

`origin/main` CI is also **failure** for several DOC commits (same class of product/architecture link defects). T03 cannot uniquely turn required CI green without DOC landing those files **and** the Current snapshot lease/slice rows. This window will not steal `PROGRESS.md`, `PERSONAL-DEVELOPMENT-PLAN.md`, `opc-2.0/`, or `personal/docs/product/`.

### DOC patch needed (T03 status only; do not rewrite 13–26)

Once DOC owns the write, Current snapshot must:

1. Active task lease row also list `` `lease/personal/P11-T03/project-aggregate` `` (keep the two DOC leases).
2. Layer 2 `` `P11-T03/D01` `` → `in-progress`, evidence = this report + PR #281 + Linux proof at `7d9f13e4`.
3. Formal plan `P11-T03` `not-started` → `in-progress`, Phase 11 summary `15/1/0/0/14` → `15/1/1/0/13`, 合计 `144/109/0/1/18` → `144/109/1/1/17`, Layer 1 `144/109/0/1/18/35` → `144/109/1/1/17/35` (Remaining = Total − Done, unchanged).
4. Land `13-…md` (or drop the index link) and replace absolute canvas paths with repo-relative links.

## Owner supersede (2026-08-30, this window)

Owner ordered continue-until-complete and unblocked T03 writing the minimum status/link paths. This window restores HEAD copies of DOC-dirty plan/product files, then applies T03-only patches: Current snapshot lease + Layer 1/2, formal-plan `P11-T03` `in-progress`, repo-relative canvas links, and the already-written `13-…md` (absolute canvas / untracked Window C links neutralized so CI can resolve). Do not author 14–26. Do not claim T02/T04.

## Unique next action

`P11-T03` is closed: required CI SUCCESS on `aef5574e`, PR [#281](https://github.com/agentkernel/cognitive-os/pull/281) merged as `main@46407380`, lease moved to §3.1. Do not auto-claim `P11-T04`.

---

## Incremental validation log (TEST-REPORT-INCREMENTAL-01)

Units are appended **immediately** after each finishes. `not-run` is never pass.

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| (claim) | written analysis | recorded | docs-only | uncommitted on `personal/P11-T03-project-aggregate` | no product code yet |
| 2026-08-30 | knife 1–5 store + `/management/project/v1/*` handler | **partial** (implemented; tests not executed) | `DEV-WIN-GNU-01` | uncommitted on `personal/P11-T03-project-aggregate` | `cargo fmt -p cognitive-store -p kernel-server` pass. `cargo test`/`clippy`/`build` **not-run** (`RUST-LINK-DEV-WIN-GNU-01`). Waiting on commit+push for `CI-UBUNTU-01` / `DEV-LINUX-NATIVE-01`. |
| 2026-08-30 | N1 `p11_t03_project_is_not_a_task_row` | not-run | `CI-UBUNTU-01` | uncommitted | store test written |
| 2026-08-30 | N2 `p11_t03_unconfirmed_activate_rejected` | not-run | `CI-UBUNTU-01` | uncommitted | store test written |
| 2026-08-30 | N3 `p11_t03_stale_total_preview_rejected` | not-run | `CI-UBUNTU-01` | uncommitted | store test written |
| 2026-08-30 | N4 `p11_t03_cross_project_write_rejected` | not-run | `CI-UBUNTU-01` | uncommitted | store test written |
| 2026-08-30 | N5 `p11_t03_gap_stage_cannot_confirm_or_test` | not-run | `CI-UBUNTU-01` | uncommitted | store test written |
| 2026-08-30 | N6 `p11_t03_completion_requires_current_verification` | not-run | `CI-UBUNTU-01` | uncommitted | store test written |
| 2026-08-30 | N7 `p11_t03_missing_openable_artifact_blocks_pass` | not-run | `CI-UBUNTU-01` | uncommitted | store test written |
| 2026-08-30 | N8 `p11_t03_unseated_stage_cannot_start_test` | not-run | `CI-UBUNTU-01` | uncommitted | store test written; production seating = empty table |
| 2026-08-30 | N9 `p11_t03_joint_acceptance_requires_all_stage_facts` | not-run | `CI-UBUNTU-01` | uncommitted | store test written; G2 positive also written |
| 2026-08-30 | N10 `p11_t03_superseded_revision_confirm_rejected` | not-run | `CI-UBUNTU-01` | uncommitted | keep/rollback asserted in test |
| 2026-08-30 | N11 `p11_t03_secret_shape_rejected_at_registration` | not-run | `CI-UBUNTU-01` | uncommitted | store test written |
| 2026-08-30 | N12 `p11_t03_non_owner_principal_cannot_confirm` | not-run | `CI-UBUNTU-01` | uncommitted | store + HTTP task-channel 403 tests written |
| 2026-08-30 | N13 `p11_t03_draft_apply_wrong_base_seq_rejected` | not-run | `CI-UBUNTU-01` | uncommitted | store test written |
| 2026-08-30 | N14 `p11_t03_unknown_cost_never_zero` | not-run | `CI-UBUNTU-01` | uncommitted | projection hook only; honest usage body = T12 |
| 2026-08-30 | N15 `p11_t03_copy_excludes_secrets_and_inflight` | not-run | `CI-UBUNTU-01` | uncommitted | copy lands `inactive` |
| 2026-08-30 | N16 `p11_t03_preview_survives_process_death` | partial (store-reopen half written) / not-run (Pi/DSH death) | store: `CI-UBUNTU-01`; Pi/DSH: T09 | uncommitted | store reopen test written; process-death/Pi/DSH half **handed to T09** |
| 2026-08-30 | docs-sync / handbook v26 | **blocked** | `DEV-WIN-GNU-01` | uncommitted | `personal/crates/cognitive-store/**` hits `dev.store-migrations`; `server.rs` hits `daemon-http`. `personal/handbook/` is owned by `DOC-PERSONAL-2.0.0/dev-prep`. Will not fake `DOCS_IMPACT_NONE`. |
| 2026-08-30 | PROGRESS Active task lease row | **blocked** | docs-only | uncommitted | `check-consistency` requires every active lease in the Current snapshot; `PROGRESS.md` is DOC-owned |
| 2026-08-30 | docs-sync / handbook v26 (committed) | **pass** | `DEV-WIN-GNU-01` | `7d9f13e4cfca76525672fdabf4f624ca1fe98aee` | Mapped handbook + generator in same changeset. pre-commit and pre-push docs-sync-gate OK. |
| 2026-08-30 | Draft PR [#281](https://github.com/agentkernel/cognitive-os/pull/281) | recorded | github | `7d9f13e4cfca76525672fdabf4f624ca1fe98aee` | Draft. Do not merge until CI green and PROGRESS lists the lease. |
| 2026-08-30 | store N1–N15 + G2 positive + cadence + pending-digest | **pass** (19/19) | `DEV-LINUX-NATIVE-01` worktree `/home/wuz/cognitiveos-personal-worktrees/p11-t03-7d9f13e4` | `7d9f13e4cfca76525672fdabf4f624ca1fe98aee` | `cargo test -p cognitive-store --test p11_t03_project_aggregate` |
| 2026-08-30 | `p1_t01_layout_migrations` including v26 `p11_project` | **pass** (8/8) | `DEV-LINUX-NATIVE-01` same worktree | `7d9f13e4cfca76525672fdabf4f624ca1fe98aee` | empty→latest includes 26 |
| 2026-08-30 | N16 store-reopen half | **pass** | `DEV-LINUX-NATIVE-01` | `7d9f13e4cfca76525672fdabf4f624ca1fe98aee` | Pi/DSH process-death half remains **not-run** / T09 |
| 2026-08-30 | HTTP G1/list/roster/N12 task 403/pending-digest | **pass** (6/6) | `DEV-LINUX-NATIVE-01` | `7d9f13e4cfca76525672fdabf4f624ca1fe98aee` | `cargo test -p kernel-server --bin kernel-server --` named filters |
| 2026-08-30 | `Cargo.lock` kernel-server tempfile | recorded | `DEV-LINUX-NATIVE-01` | `8374d560cd55e5a4cb322cbf6588218309565ccc` | `--locked` on `7d9f13e4` wanted one tempfile line; follow-up commit carries it |
| 2026-08-30 | required CI run [33281002286](https://github.com/agentkernel/cognitive-os/actions/runs/33281002286) | **fail** (consistency) | `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | `8374d560cd55e5a4cb322cbf6588218309565ccc` | 5 check-consistency violations; Rust tests **not-run** on this job. 2 are T03 lease/snapshot (DOC `PROGRESS.md`). 3 inherited from `origin/main` DOC product/architecture links. |
| 2026-08-30 | parent recovery after subagent death | recorded | `DEV-WIN-GNU-01` | `8374d560cd55e5a4cb322cbf6588218309565ccc` | No product rewrite. Unique next action = DOC snapshot + link repair. |
| 2026-08-30 | owner supersede: T03 writes PROGRESS/plan + inherited link correctives | recorded | `DEV-WIN-GNU-01` | uncommitted on `personal/P11-T03-project-aggregate` | Keep DOC leases. Add existing `13-…md` only. Absolute canvas paths → repo-relative. |
| 2026-08-30 | `node tools/src/check-consistency.mjs` | **pass** | `DEV-WIN-GNU-01` | uncommitted on `personal/P11-T03-project-aggregate` | 0 violations after T03 snapshot + 13-assessment + portable canvas links |
| 2026-08-30 | `check-handbook` + `generate-handbook --check` | **pass** | `DEV-WIN-GNU-01` | uncommitted (13-assessment staged) | 58×2 handbook; 18 generated pages byte-identical |
| 2026-08-30 | required CI run [33288037382](https://github.com/agentkernel/cognitive-os/actions/runs/33288037382) | **SUCCESS** | `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | `aef5574e3a4c76a5b5e0e19fe4ed4ab0b0872e88` | resolve 3s; Ubuntu 3m32s (Rust workspace + consistency + handbook); Windows 11m56s; `required-ci` |
| 2026-08-30 | PR [#281](https://github.com/agentkernel/cognitive-os/pull/281) ready + merge | recorded | github | `main@464073809ffadf1f2c08e7391bbac5b4b2c0ed8b` | merge commit; task branch delete follows this status/closure commit |

## Non-claims

Not Gate, release, Profile, B01, Windows OPC qualification, Agent-benefit, or live `/ui/` IA. Live `/ui/` remains Linux 1.0 six-family.
