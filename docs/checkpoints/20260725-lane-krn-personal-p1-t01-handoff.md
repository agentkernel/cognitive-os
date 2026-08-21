# 20260725 Lane-KRN Personal P1-T01 Handoff

## 1. Task Snapshot

- Task: `P1-T01` — 版本化数据库迁移与 XDG 布局
- Date: 2026-07-25
- Branch: `lane/krn-personal-p1-t01-xdg-migrations`
- Base commit: `43a1d85` (`main` after P0-T07 evidence)
- Lane: Lane-KRN (`cognitive-store` only; no registry/schema/vector/transition)
- Status: **done** (implementation + CI Ubuntu/Windows-MSVC evidence)

## 2. Completed in this atomic batch

- Added `crates/cognitive-store/src/layout.rs`:
  - `PersonalDataLayout` resolves XDG config/data/state/cache/runtime roots
  - Product dir `cognitiveos/` under each root
  - Paths for `authority.sqlite`, `installations.sqlite`, `backups/`,
    `migration/` scratch, and exclusive `migration.lock`
  - `ensure_directories` creates private dirs (Unix mode `0700`)
  - `XDG_RUNTIME_DIR` required (fail-closed; no shared-temp fallback)
- Added `crates/cognitive-store/src/personal_db.rs`:
  - Production plans v1 share `AUTHORITY_SCHEMA_V1` / `INSTALLATION_SCHEMA_V1`
    with the existing open paths (no schema drift)
  - `prepare_personal_databases` creates empty SQLite files if missing,
    acquires exclusive migration lock, applies each DB independently with
    non-overwriting backups under state, sets Unix DB mode `0600`
  - Explicit non-claim of cross-database atomicity; installation failure after
    authority apply surfaces authority backup path for recovery
- Added focused tests `tests/p1_t01_layout_migrations.rs`:
  - empty → latest (both DBs)
  - previous fixture → latest (v1 then v2 additive)
  - reapply / replay-safe prepare
  - digest mismatch fails closed (later version not applied)
  - disk/copy failure on illegal backup destination leaves source unmigrated
  - exclusive `migration.lock` blocks concurrent prepare
  - Unix permission assertions for 0700/0600
- Updated ADR-0017 validation/data-layout sections for P1-T01 realization
- Updated formal Personal ledger, `docs/plan/plan.md` task card, and `PROGRESS.md`

## 3. Not completed / out of scope

- Long-term backup retention policy automation (keep-N) not implemented
- Full daemon single-instance lifecycle lock remains P1-T04
- Coordinated two-database atomic upgrade not claimed
- No admin-cli / kernel-server wiring of prepare yet (P1-T04/T06)
- P0-T03 license/platform/distribution owner decision still open
- P0-T06 Pi PoC still blocked on P0-T03
- G0 / B01-B12 / Profile claims are **not** made

## 4. Tests and evidence

| Check | Status | Result |
|---|---|---|
| Implementation unit tests (`p1_t01_layout_migrations`) | not-run locally | Windows GNU linker exit 121 (P0-T01 non-supported baseline) |
| `cargo test -p cognitive-store --test p1_t01_layout_migrations --locked` | not-supported host | linker exit 121 before compile complete |
| `pnpm run check:consistency` | pass (local) | 273 REQ / 55 codes / 63 schemas / 85 vectors |
| `git diff --check` | pass (local) | — |
| CI `cargo test --workspace --locked` | pass | run [30155053950](https://github.com/agentkernel/cognitive-os/actions/runs/30155053950) Ubuntu + Windows/MSVC; includes `p1_t01_layout_migrations` (7 pass) |
| CI clippy / rustfmt | pass | same run |
| Personal Gates / B01-B12 / Profile | not-run | No claim |

## 5. Design and safety boundaries

- Clients/Pi/CLI remain non-authority; layout/prepare are store adapters only
- No secrets, env credentials, or provider keys enter paths or SQLite
- Authority transition semantics unchanged; only schema history + paths
- Migration lock is prepare-only; not a substitute for daemon lease (P1-T04)

## 6. Next entry

1. PR #92 CI green (run 30155053950); merge when ready.
2. Dependency-satisfied next Personal tasks:
   - **P1-T02** after P1-T01 done (SecretStore formal backend)
   - **P1-T04** after P1-T01 done + P0-T07 (bounded daemon)
   - **P0-T03** still needs owner license/platform/distribution GO/NO-GO
3. Suggested prompt: `Continue Personal plan. Read AGENTS.md, PROGRESS,
   20260725-lane-krn-personal-p1-t01-handoff.md, PARALLEL-LANES,
   PERSONAL-DEVELOPMENT-PLAN. Prefer next dependency-satisfied task (P1-T02
   or P1-T04) without claiming G0/Profile. If selecting P0-T03, stop and ask
   owner for license/platform/distribution.`

## 7. Snapshot

- PROGRESS updated: yes (P1-T01 done; CI 30155053950)
- Formal Personal ledger updated: yes (`done`)
- Commits: `6e92d24` (feat), `c17c6b7` (fmt); docs evidence commit follows
- PR: [#92](https://github.com/agentkernel/cognitive-os/pull/92)
- CI: [30155053950](https://github.com/agentkernel/cognitive-os/actions/runs/30155053950) success
