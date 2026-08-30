# P11-T11 Memory admission, privacy, forget — running report

- Task: `P11-T11` / slice `P11-T11/D01`
- Change class: `implementation-only` (scoped episodic recall + privacy screens on the existing Memory store/HTTP; no `core/specs`, no Lane-CTR, no `/ui/` IA, no Letta/Mem0 write path)
- Product: CognitiveOS Personal 2.0.0
- Lease: `lease/personal/P11-T11/memory`
- Branch: `personal/P11-T11-memory`
- Worktree: `D:\agent-kernel-wt-P11-T05` (original `d:\agent-kernel` left untouched)
- Claim ceiling: `hypothesis` (A7: local/CI is not Gate/release/Profile; privacy/rebuild E2E is `not-run`)
- Evaluation routing: **OFF** (`PERSONAL-PERF-EVAL-015` closed)

## Unique next action

Linux focused tests on the pushed SHA: `cargo test -p cognitive-store --test p11_t11_memory` and `cargo test -p kernel-server --bin kernel-server -- p11_t11`. Do not merge T11. `DEV-WIN-GNU-01` cargo remains `not-run`. Evaluation routing OFF.

## Closed predecessor

`P11-T10` **done**: merged PR [#288](https://github.com/agentkernel/cognitive-os/pull/288) at `main@e51b616e`. Lease `lease/personal/P11-T10/vault` closed into PARALLEL-LANES §3.1. Vault files still cannot enter Memory as authority (N5 retained). Do not unpark T14/T15.

## Identifier

Acceptance: `EPISODIC_RETRIEVAL_MEMORY_PRIVACY_CORRECT_FORGET`.

Canonical episodic scope: `opc://project/{id}/employee/{id}` encoded in existing `governance_scope` / `target_scope` (no v33 migration).

Reused: `admit_memory_candidate`, P4 `POST /management/resource/v1/memory/remember` and `…/forget`, T10 `VaultStore::admit_as_memory` (kept). No second Memory store.

## Failure-first (this slice)

N1–N5 are **existing coverage** (already written on this worktree). They are not a new pass until Linux/CI runs them. This commit only thickens N6 verbs and adds the management `correct` HTTP fail-closed.

| ID | Test | Surface | Status |
|---|---|---|---|
| N1 | cross-scope episodic recall rejected | store `p11_t11_cross_scope_episodic_recall_is_rejected`; HTTP recall 403 | existing coverage (not new; cargo **not-run** on `DEV-WIN-GNU-01`) |
| N2 | secret/PII-shaped candidate denied | store `p11_t11_secret_and_pii_shaped_candidate_is_denied`; HTTP remember 422 | existing coverage (not new; cargo **not-run** on `DEV-WIN-GNU-01`) |
| N3 | Agent/self / Letta-Mem0-style direct write rejected | store `p11_t11_agent_self_and_letta_mem0_direct_write_is_rejected`; HTTP remember 422 | existing coverage (not new; cargo **not-run** on `DEV-WIN-GNU-01`) |
| N4 | forget → index/cache rebuild → no resurrection | store scoped `p11_t11_forget_then_index_rebuild_cannot_resurrect_scoped_memory`; HTTP forget + `index.rebuild` + recall empty. Existing `p4_t02` `forget_appends_a_tombstone_and_prevents_fts_resurrection` retained (workspace:// FTS poke) | existing coverage (not new; cargo **not-run** on `DEV-WIN-GNU-01`) |
| N5 | vault file still cannot enter Memory as authority | T10 `p11_t10_memory_admission_cannot_swallow_vault_files` | already-green (retained, not duplicated) |
| N6 | task-channel Memory mutation fail-closed | HTTP `POST /task/resource/v1/memory/{remember,forget,recall,correct,index.rebuild,review}` 403 `RESOURCE_MEMORY_CHANNEL_FORBIDDEN` | N6 thickened this commit (tests only; gate already at `handle_authority_or_mutation` 635–646). Cargo **not-run** on `DEV-WIN-GNU-01` |
| correct | management correct fail-closed | HTTP `POST /management/resource/v1/memory/correct` secret-shaped 422 + cross-scope 403 | new focused test `p11_t11_management_correct_is_fail_closed_for_cross_scope_and_secret`. Cargo **not-run** on `DEV-WIN-GNU-01` |

## Vertical slice

Real caller: `POST /management/resource/v1/memory/remember` (optional `project_id`+`employee_id`) → admission screens → `GET …/memory/object` view → `POST …/memory/correct` → `POST …/memory/forget` → `POST …/memory/index.rebuild`. Scoped recall: `POST …/memory/recall`. Task-channel aliases 403.

## Incremental validation log (TEST-REPORT-INCREMENTAL-01)

Units are appended **immediately** after each finishes. `not-run` is never pass.

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-08-31 | T11 claim + existing N1–N5 store/HTTP coverage | recorded | `DEV-WIN-GNU-01` | this commit | Existing coverage only; not a new pass |
| 2026-08-31 | N6 thickened: task forget/recall/correct/index.rebuild/review 403 | **not-run** | `DEV-WIN-GNU-01` | this commit | Tests added; gate already existed. `RUST-LINK-DEV-WIN-GNU-01` |
| 2026-08-31 | management `correct` HTTP fail-closed | **not-run** | `DEV-WIN-GNU-01` | this commit | `p11_t11_management_correct_is_fail_closed_for_cross_scope_and_secret` |
| 2026-08-31 | `cargo test -p cognitive-store --test p11_t11_memory` | **not-run** | `DEV-WIN-GNU-01` | this commit | `RUST-LINK-DEV-WIN-GNU-01`; route to `DEV-LINUX-NATIVE-01` / `CI-UBUNTU-01` |
| 2026-08-31 | `cargo test -p kernel-server --bin kernel-server -- p11_t11` | **not-run** | `DEV-WIN-GNU-01` | this commit | Covers thickened N6 + management correct. `RUST-LINK-DEV-WIN-GNU-01` |
| 2026-08-31 | `cargo test -p cognitive-store --test p4_t02_memory_search -- --exact forget_appends_a_tombstone_and_prevents_fts_resurrection` | **not-run** | `DEV-WIN-GNU-01` | this commit | already-green on main; not re-run locally |
| 2026-08-31 | `cargo test -p cognitive-store --test p11_t10_vault -- --exact p11_t10_memory_admission_cannot_swallow_vault_files` | **not-run** | `DEV-WIN-GNU-01` | this commit | already-green T10 N5; not duplicated |
| 2026-08-31 | `cargo build` / Clippy | **not-run** | `DEV-WIN-GNU-01` | this commit | `RUST-LINK-DEV-WIN-GNU-01` |
| 2026-08-31 | privacy/rebuild host E2E | **not-run** | unqualified | this commit | Card allows `not-run` until the host route is qualified |
| 2026-08-31 | Letta/Mem0 product write path / Agent self-admission / Vault-as-Memory / Team/Inbox L1 | **not-run** / out of scope | n/a | this commit | Do not claim |
