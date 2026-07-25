# 20260725 Personal P1-T06 Cognitive CLI Handoff

## 1. Task Snapshot

- Task: `P1-T06` — `cognitive init/doctor/status/daemon`
- Date: 2026-07-25
- Branch: `lane/personal-p1-t06-cognitive-cli` (merged)
- Merge: PR [#98](https://github.com/agentkernel/cognitive-os/pull/98) → `main@adbb0e5`
- Lane: Personal product CLI in `apps/admin-cli` (does **not** take Lane-RUN ownership of `cognitive-management` / `cognitive-runtime`)
- Status: **done** — CI Ubuntu/Windows-MSVC green. Local Windows GNU linker exit 121 remains the P0-T01 non-supported host. Not a G0/B01-B12/Profile claim.

## 2. Completed in this atomic batch

- Dual bins in `apps/admin-cli`: `admin-cli` (emergency management) + `cognitive` (Personal product entry)
- Library module `apps/admin-cli/src/personal_cli/`:
  - `init` — XDG layout, `prepare_personal_databases`, optional Provider configure via SecretStore, idempotent re-init
  - `status` / `doctor` — management-channel HTTP clients for daemon projections (`connect_timeout`)
  - `daemon start|status|stop` — spawn/stop `kernel-server --personal` with stale-lock cleanup after confirmed death
- URL normalization: strip trailing `/`, reject `http://` and credentials
- Secret capture: `--api-key-file` / stdin; Unix echo-off interactive; Windows fails closed to file input
- ADR-0024 freezes ownership and non-claims
- Integration tests: `apps/admin-cli/tests/p1_t06_cognitive_cli.rs`
  - Ubuntu: init + doctor/status against Child-owned daemon + full `cognitive daemon start|status|stop`
  - Windows: init/usage only (live daemon CLI process-tree tests gated to Unix after MSVC job-object hangs)

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
| CI PR run | executed | [30167503487](https://github.com/agentkernel/cognitive-os/actions/runs/30167503487) Ubuntu + Windows-MSVC SUCCESS |
| Personal Gates / B01-B12 / Profile | not-run | No claim |

## 5. Design and safety boundaries

- CLI is non-authority: no Task/Effect/Verification authority writes
- Provider API keys never enter config JSON, SQLite, env, argv, logs, or reports
- `--allow-ephemeral-secret-backend` is tests-only
- Status/doctor require management bearer via the Personal daemon
- Reports always include `profile_claim` / `gate_claim` = `not-claimed`
- `daemon stop` removes confirmed-stale lock after process death (SIGTERM skips Rust Drop)

## 6. Next entry

1. Suggested next: **P1-T07** Pi package/extension (deps: P0-T06, P1-T03, P1-T04, P1-T05) — may need owner decisions on Pi packaging.
2. Owner decision still required for **P0-T03** before P0-T06 / installer (P1-T08).
3. Suggested prompt: start P1-T07 if Pi package scope is clear; otherwise resolve P0-T03.

## 7. Snapshot

- PROGRESS updated: yes (P1-T06 done; no Profile claim)
- Formal Personal ledger updated: yes (`done`, CI 30167503487)
- PR: [#98](https://github.com/agentkernel/cognitive-os/pull/98) merged
- CI: [30167503487](https://github.com/agentkernel/cognitive-os/actions/runs/30167503487) success
- Merge commit: `adbb0e5`
