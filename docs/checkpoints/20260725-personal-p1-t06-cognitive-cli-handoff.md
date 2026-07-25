# 20260725 Personal P1-T06 Cognitive CLI Handoff

## 1. Task Snapshot

- Task: `P1-T06` — `cognitive init/doctor/status/daemon`
- Date: 2026-07-25
- Branch: `lane/personal-p1-t06-cognitive-cli`
- Base commit: `deae801` (`origin/main` after P1-T05 merge)
- Lane: Personal product CLI in `apps/admin-cli` (does **not** take Lane-RUN ownership of `cognitive-management` / `cognitive-runtime`)
- Status: **in-progress** — implementation landed; executable evidence depends on CI Ubuntu/Windows-MSVC. Local Windows GNU linker exit 121 remains the P0-T01 non-supported host. Not a G0/B01-B12/Profile claim.

## 2. Completed in this atomic batch

- Dual bins in `apps/admin-cli`: `admin-cli` (emergency management) + `cognitive` (Personal product entry)
- Library module `apps/admin-cli/src/personal_cli/`:
  - `init` — XDG layout, `prepare_personal_databases`, optional Provider configure via SecretStore, idempotent re-init
  - `status` / `doctor` — management-channel HTTP clients for daemon projections
  - `daemon start|status|stop` — spawn/stop `kernel-server --personal`
- URL normalization: strip trailing `/`, reject `http://` and credentials
- Secret capture: `--api-key-file` / stdin; Unix echo-off interactive; Windows fails closed to file input
- ADR-0024 freezes ownership and non-claims
- Integration tests: `apps/admin-cli/tests/p1_t06_cognitive_cli.rs`
- Formal ledger + PROGRESS updated; trailing orphan P1-T03 row on the formal ledger removed as hygiene

## 3. Not completed / out of scope

- Live Secret Service / live Provider network probe during init
- Pi package checks (P1-T07)
- Installer / user service (P1-T08)
- B01 clean-run Gate (P1-T09)
- G0 / B01-B12 / Profile claims
- Registry / schema / vector / transition changes

## 4. Tests and evidence

| Check | Status | Result |
|---|---|---|
| Local Windows GNU `cargo test -p admin-cli` | not-supported host | linker exit 121 (P0-T01) |
| Local MSVC | not-supported host | `link.exe` not found |
| `pnpm run check:consistency` | executed before push | see commit notes |
| `git diff --check` | executed before push | see commit notes |
| CI PR run | pending | create PR after push |
| Personal Gates / B01-B12 / Profile | not-run | No claim |

## 5. Design and safety boundaries

- CLI is non-authority: no Task/Effect/Verification authority writes
- Provider API keys never enter config JSON, SQLite, env, argv, logs, or reports
- `--allow-ephemeral-secret-backend` is tests-only
- Status/doctor require management bearer via the Personal daemon
- Reports always include `profile_claim` / `gate_claim` = `not-claimed`

## 6. Next entry

1. Push branch and open PR; wait for Ubuntu + Windows-MSVC CI.
2. On CI green, mark P1-T06 `done` with run IDs and merge.
3. Owner decision still required for **P0-T03** before P0-T06 / installer.
4. Suggested prompt: after CI green, close P1-T06 ledger.

## 7. Snapshot

- PROGRESS updated: yes (P1-T06 in-progress; no Profile claim)
- Formal Personal ledger updated: yes (`in-progress`)
- PR: pending
- CI: pending
- Commits: pending
