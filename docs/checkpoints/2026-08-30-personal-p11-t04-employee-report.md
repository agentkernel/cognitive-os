# P11-T04 Role Blueprint / Assignment / Employee — running report

- Task: `P11-T04` / slice `P11-T04/D01`
- Change class: `implementation-only` (Personal-private tables + private projection; no `core/specs`, no Lane-CTR)
- Product: CognitiveOS Personal 2.0.0 (v9 is a canvas filename only)
- Lease: `lease/personal/P11-T04/employee`
- Branch: `personal/P11-T04-employee`
- PR: [#282](https://github.com/agentkernel/cognitive-os/pull/282) Draft
- HEAD: `9cb1b6dcf15964b3107f4e4baf4aca17decfa3c4` (Clippy `HandoffSpec`); Linux store/HTTP proof at `3c7a4190`
- Claim ceiling: `hypothesis` (A7: local/CI is not Gate/release/Profile)
- Evaluation routing: **OFF** (`PERSONAL-PERF-EVAL-015` closed)

## Recovery (2026-08-30)

| Fact | Value |
|---|---|
| Git at claim | `main@2e6a2ad7289c2f8cc5e2e421275d75ba73296508` tracking `origin/main` |
| Worktree | Dirty DOC-owned docs/handbook/skills and untracked opc-2.0 14–22 — **protected, never `git add -A`** |
| Evaluation campaign | OFF |
| P11-T03 | **done** — merged PR [#281](https://github.com/agentkernel/cognitive-os/pull/281) at `main@46407380`; lease-close commit `2e6a2ad7`; required CI [33288037382](https://github.com/agentkernel/cognitive-os/actions/runs/33288037382) **SUCCESS** at `aef5574e` |
| Other P11 implementation lease | none (only DOC `dev-prep` + `OPC-REFRAME`) |
| Guidance present | 13, 14, 15, 16, 17, 18, 21, 22 |
| Guidance missing | 19, 20, 23, 24, 25, 26; `docs/plan/p11-plan-review-and-optimization.md` — recorded, not invented |

## Written analysis (required before product code)

### Applicable guidance

| Source | How T04 uses it |
|---|---|
| 14 §4.1–4.4 | Object shape: RoleBlueprint / Employee / EmployeeRevision / Grant; sequential seating; roster from axis slots; eight-tab authority (input is PlanRevision seam, not a member field); recipe ≠ grant |
| 14 §3.5 | Employee stores `responsible_stage_ids` only; input tab is a read-only seam reference. Do not copy contract bytes onto Employee |
| 14 §5.3 item 1 | Seating predicate: `seated` + model bound. T03 N8 empty-table fail-closed remains; T04 supplies real seated facts |
| 14 §7.6 | InstallFact vs Grant split; T04 lands both rows + catalog. grant-expansion HITL time-box stays T09 |
| 14 §8 | N8 variant (progress reads committed facts) is T04-N5. Other N1–N16 stay T03 |
| 14 §9 | Rejected shortcuts: Role=Agent merge, heartbeat authority, chat Approve, install-as-grant, progress-from-generation-stream |
| 15 §1 I2/I3 | Employee is the long-lived authority object; Attempt/process is not. Cited only |
| 15 §5 / A7 / A11 | Grant catalog + speech whitelist negatives. Cited only; do not rewrite 15 |
| 16 号 | Preview-chain consumer only; no T06 assistant body |
| **17 号 full text** | Employee specialty: consume, do not rewrite. First knives: tables + C1/C12, roster + seating gate, C2 recipe, speech skeleton C5, grant C8 |
| 18 号 | Conversation archive landing is T05; T04 speech filter may leave an audit row without a new projection version |
| 21 号 | T03 roster shape `{ roster, authority_note }`; T04 fills employees. Axis `seated` no longer hardcoded false |
| 22 §3 | T04-N1..N11 are the failure-first catalog. T07/T05/T09 entries are not this task |

### Objects this slice MUST land

`RoleBlueprint` (+ revision; **no Provider binding column**), `Assignment` (slot → Employee, derived from confirmed PlanRevision `responsible_slot`), `Employee` (authoritative id; chrome word Member Runtime), `EmployeeRevision` (six-slot recipe digest), `Grant` + `InstallFact` (recipe ≠ authorization), replaceable `runtime_binding_ref` (existing adapter identity string, not a third id), seating state machine (`proposed/seating/seated/pending/refused/suspended/removed`), one current manager per Project, speech-router skeleton + handoff row **without** an authority-transfer field.

T03 Project aggregate, G1/G2, fail-closed empty seating, SessionGate, and Agent adapter identity already exist and are reused.

### Objects this slice MUST NOT land

| Object | Owner |
|---|---|
| Project / Charter / Plan / Stage / Gap / G1 / G2 body | T03 (already on `main`) |
| Conversation new private version / archive | T05 |
| Hidden Pi assistant | T06 |
| Hosted DSH / Attempt process engine / W4 payload / subagent | T07 |
| Routine arming | T08 |
| Canvas HITL time-box / grant-expansion preview chrome | T09 |
| Vault / Knowledge | T10 |
| Member-private Memory admission | T11 |
| Honest usage body | T12 |
| `/ui/` IA | T13 |
| X connector / Windows OPC Gate | T14 / T15 parked |

### What this slice is NOT

Not Today. Not the six-family Agents rename. Not an assistant skin. Not a Project aggregate redo. Not Vite product origin. Not heartbeat authority. Not Role=Agent. Not a third identity id besides Employee and existing Agent/adapter identity.

### 14 §8 / 22 号 T04 negatives — write first vs not-run/handoff

| ID | Test name | This task? |
|---|---|---|
| T04-N1 (=17C1) | `p11_t04_employee_survives_process_death` | yes (store: process-exit observer must not mutate Employee). Real DSH child kill = T07 **not-run** |
| T04-N2 (=17C2) | `p11_t04_recipe_mention_grants_nothing` | yes |
| T04-N3 | `p11_t04_roster_must_cover_all_slots` | yes |
| T04-N4 | `p11_t04_sequential_seating_enforced` | yes |
| T04-N5 | `p11_t04_progress_reads_committed_facts_only` | yes (14 §8 N8 variant) |
| T04-N6 (=17C12) | `p11_t04_blueprint_upgrade_is_opt_in` | yes |
| T04-N7 | `p11_t04_employee_not_shared_across_projects` | yes |
| T04-N8 | `p11_t04_one_current_manager` | yes |
| T04-N9 (=17C5) | `p11_t04_speech_whitelist_enforced` | yes (filter + audit row). Conversation archive landing = T05 **handoff** |
| T04-N10 (=17C8/15A11) | `p11_t04_mcp_grant_is_per_scope` | yes |
| T04-N11 | `p11_t04_chat_cannot_transfer_authority` | yes |
| Card | `p11_t04_role_is_not_agent` | yes |
| Card | `p11_t04_blueprint_has_no_provider_binding` | yes |
| Card | `p11_t04_removed_manager_keeps_history` | yes |
| 17C6 | member message revises work | **not this task** (T05/T09) |
| 17C7 | manager exceeds envelope | T04 half = manager cannot mint Owner-scoped grant/team change without owner confirm; T09 canvas chrome **handoff** |
| 17C3/C4/C9–C11 | Attempt payload / deliver / DSH secrets / broker / subagent | **T07 / T03** — not-run here |
| Host E2E | lifecycle UI | `Requires-environment` / **not-run** |

### Recorded deviations (none from 14 §9)

1. **Seating preview is a dedicated Employee writer**, not a `p11_approval_preview` CHECK expansion. T03 CHECK is `activation\|plan-change\|acceptance`; SQLite cannot ALTER that CHECK. Recreating the T03 table would be a Project-aggregate redo. T09 still owns ApprovalPreview kind expansion. Seating still requires `ConfirmCaller::OwnerManagement` (SessionGate / N12 discipline).
2. **`runtime_binding_ref` is an opaque adapter-identity string** on the Employee row. This crate does not take a `cognitive-runtime` dependency; T07 binds a real DSH process later. Replacing the ref does not change `employee_id`.
3. **Empty roster `authority_note`** changes from T03's `employee-authority-not-implemented` to `empty-roster` once the Employee tables exist. This is T04 filling the seam 21 号 reserved, not a T03 body redo.

### Path collisions (DOC)

| Path | Owner | T04 action |
|---|---|---|
| `docs/plan/PROGRESS.md`, `plan.md`, `PERSONAL-DEVELOPMENT-PLAN.md`, `personal-trace.yaml` | `DOC-PERSONAL-2.0.0/dev-prep` | **blocked_paths** on the lease. Owner authorized T04-only status/evidence rows for check-consistency (same pattern as T03 unblock) — write those rows only, do not rewrite other cards or 13–26 |
| `personal/handbook/` | DOC `dev-prep` tree | not on T04 lease (overlap forbidden). docs-sync-contract §2 still requires mapped v27/route pages **in the same git changeset** (T03 precedent). Do not fake `DOCS_IMPACT_NONE` |
| `clients/docs/design/opc-2.0/`, `personal/docs/product/` | `DOC-PERSONAL-2.0-OPC-REFRAME` | do not write |
| `personal/docs/architecture/personal-2.0.0-dev-prep-index.md` | DOC `dev-prep` | do not write |

## Incremental validation log (TEST-REPORT-INCREMENTAL-01)

Units are appended **immediately** after each finishes. `not-run` is never pass.

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-08-30 | written analysis | recorded | docs-only | uncommitted | no product code yet |
| 2026-08-30 | `cargo test -p cognitive-store --test p11_t04_employee` | **not-run** | `DEV-WIN-GNU-01` | uncommitted | `RUST-LINK-DEV-WIN-GNU-01`; route to CI/Linux |
| 2026-08-30 | `cargo fmt --all -- --check` | **pass** | `DEV-WIN-GNU-01` | uncommitted | allowed GNU surface |
| 2026-08-30 | `node tools/src/check-consistency.mjs` | **pass** | `DEV-WIN-GNU-01` | uncommitted | T04-only status rows + lease |
| 2026-08-30 | `node tools/src/check-handbook.mjs` | **pass** | `DEV-WIN-GNU-01` | uncommitted | 58 docs × 2 locales |
| 2026-08-30 | `node tools/src/generate-handbook.mjs --check` | **pass** | `DEV-WIN-GNU-01` | uncommitted | 18 pages byte-identical |
| 2026-08-30 | `node tools/src/docs-sync-gate.mjs --staged` | **pass** | `DEV-WIN-GNU-01` | `3c7a4190` | store + handbook changeset; no `DOCS_IMPACT_NONE` |
| 2026-08-30 | `cargo test -p cognitive-store --test p11_t04_employee` | **pass** (16/16) | `DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t04-3c7a4190` | `3c7a419080f8e5702a99f5b8746b8f7415a47aa0` | T04-N1..N11 + Role≠Agent + Blueprint no Provider + history + seated predicate + task-channel 403 |
| 2026-08-30 | `cargo test -p cognitive-store --test p1_t01_layout_migrations` | **pass** (8/8) | `DEV-LINUX-NATIVE-01` | `3c7a419080f8e5702a99f5b8746b8f7415a47aa0` | v27 additive; `p11_employee` present |
| 2026-08-30 | `cargo test -p kernel-server --bin kernel-server -- project_aggregate` | **pass** (7/7) | `DEV-LINUX-NATIVE-01` | `3c7a419080f8e5702a99f5b8746b8f7415a47aa0` | empty-roster, register+seat HTTP, T03 G1/N12 retained |
| 2026-08-30 | `cargo clippy -p cognitive-store -p kernel-server --all-targets --locked -- -D warnings` | **fail** | `DEV-LINUX-NATIVE-01` | `3c7a419080f8e5702a99f5b8746b8f7415a47aa0` | `clippy::too_many_arguments` on `record_handoff` (8/7). Superseded by `9cb1b6dc` `HandoffSpec`. |
| 2026-08-30 | Draft PR [#282](https://github.com/agentkernel/cognitive-os/pull/282) | recorded | github | `3c7a419080f8e5702a99f5b8746b8f7415a47aa0` | Draft. Do not merge until Clippy/CI green. |
| 2026-08-30 | required CI [33290989251](https://github.com/agentkernel/cognitive-os/actions/runs/33290989251) | **fail** (Clippy) | `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | `3c7a419080f8e5702a99f5b8746b8f7415a47aa0` | `clippy::too_many_arguments` on `EmployeeStore::record_handoff` (8/7). Tests **not-run** (clippy failed first). |
| 2026-08-30 | parent recovery after subagent death | recorded | `DEV-WIN-GNU-01` | `3c7a419080f8e5702a99f5b8746b8f7415a47aa0` | Unique next action = `HandoffSpec` Clippy fix → Linux/CI → close. DOC dirty / untracked 14–22 protected. Never `git add -A`. |
| 2026-08-30 | `HandoffSpec` Clippy fix | recorded | `DEV-WIN-GNU-01` | uncommitted | `record_handoff` now takes `HandoffSpec`; bilingual store-and-migrations + generated http-api fingerprints. |
| 2026-08-30 | `cargo fmt --all -- --check` | **pass** | `DEV-WIN-GNU-01` | uncommitted | allowed GNU surface |
| 2026-08-30 | `node tools/src/check-consistency.mjs` | **pass** | `DEV-WIN-GNU-01` | uncommitted | 275 requirements; lease date format restored |
| 2026-08-30 | `node tools/src/check-handbook.mjs` | **pass** | `DEV-WIN-GNU-01` | uncommitted | 58 docs × 2 locales |
| 2026-08-30 | `node tools/src/generate-handbook.mjs --check` | **pass** | `DEV-WIN-GNU-01` | uncommitted | 18 pages byte-identical |
