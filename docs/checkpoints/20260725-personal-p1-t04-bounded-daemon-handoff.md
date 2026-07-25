# 20260725 Personal P1-T04 Bounded Daemon Handoff

## 1. Task Snapshot

- Task: `P1-T04` — 有界 Personal daemon 与本地认证
- Date: 2026-07-25
- Branch: `lane/personal-p1-t04-bounded-daemon-auth`
- Base commit: `7ee7648` (`main` after P1-T03 evidence)
- Lane: Personal / composition root `apps/kernel-server` (does not take Lane-RUN
  ownership of `cognitive-runtime` / `cognitive-management` readiness projection)
- Status: **in-progress**. PR #95 merged at `main@6fe21b3`; CI run 30159744673
  passed on Ubuntu and Windows/MSVC for the current auth/size/host/cookie/restart
  coverage. Timeout and concurrency behavior tests remain outstanding.

## 2. Completed in this atomic batch

- `apps/kernel-server/src/personal/`:
  - `auth` — channel-scoped sessions, bootstrap secret file, redacted Debug
  - `bounds` — ADR-0019 resource ceilings
  - `lifecycle` — single-instance `daemon.lock`
  - `server` — loopback Personal HTTP front door
- `kernel-server --personal [--once] --bind 127.0.0.1:PORT --runtime-root DIR`
- Layout helpers: `daemon_lock_path`, `daemon_socket_path`,
  `local_bootstrap_secret_path`
- Integration tests `tests/p1_t04_personal_daemon.rs`
- ADR-0022 documents implementation decisions

## 3. Not completed / out of scope

- Full readiness/doctor projection (P1-T05)
- `cognitive` CLI product entry (P1-T06)
- UDS product default listener (design remains ADR-0019; loopback path for CI)
- Task scheduler / Memory / MCP
- Registry / schema / vector changes
- G0 / B01-B12 / Profile claims

## 4. Tests and evidence

| Check | Status | Result |
|---|---|---|
| Local Windows GNU `cargo check/test` | not-supported host | linker exit 121 (P0-T01) |
| CI `cargo test --workspace --locked` | executed (partial) | Run 30159744673 passed on Ubuntu + Windows/MSVC for existing `p1_t04_personal_daemon` coverage; timeout/concurrency behavior coverage is still missing |
| Personal Gates / B01-B12 / Profile | not-run | No claim |

## 5. Design and safety boundaries

- Clients remain non-authority; this front door creates no authority side effects.
- Tokens/bootstrap secret never in logs Debug as plaintext, env, or SQLite.
- M5 synthetic `--once/--serve` routes preserved without `--personal`.
- Cross-channel bearer use fails with `SHELL_CHANNEL_BINDING_MISMATCH`.

## 6. Next entry

1. Add and execute timeout/concurrency behavior tests for P1-T04; do not begin
   P1-T05 until those acceptance criteria have CI evidence.
2. **P0-T03** still needs owner license/platform/distribution GO/NO-GO.
3. Suggested prompt: continue P1-T04 timeout/concurrency coverage without
   claiming G0/Profile.

## 7. Snapshot

- PROGRESS updated: yes (no Profile claim)
- Formal Personal ledger updated: yes (`in-progress`, partial CI evidence)
- PR: #95 merged at `6fe21b3`
- CI: run 30159744673 passed on Ubuntu + Windows/MSVC for the current scope
