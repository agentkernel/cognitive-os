# P0-T01/D02 — closure (local Rust toolchain repair on `DEV-WIN-GNU-01`)

- Task: `P0-T01` 固定可复现基线与支持工具链 (Phase 0) — status remains `done` at task level; Delivery Slice `P0-T01/D02` `done` with this delivery
- Change class: **corrective** (environment registration + local-only rustup override) + **implementation-only** (`scripts/v01-auto-run.*` Option A re-pin 89/62/27 + pin-guard test); tracked `rust-toolchain.toml` untouched; normative surface unchanged
- Lease: `lease/personal/P0-T01/toolchain-repair` → closed in this merge-closure commit on `main` (PARALLEL-LANES §3.1)
- Branch: `personal/P0-T01-D02-toolchain`; content/merge head `2cbc0975`; GitHub merge `main@e9826f70`; local + remote branch deleted, worktree removed
- PR: [#314](https://github.com/agentkernel/cognitive-os/pull/314) — Draft → ready → **merged at `main@e9826f70`** (2026-09-03T08:45:35Z)
- Required CI: [33733732726](https://github.com/agentkernel/cognitive-os/actions/runs/33733732726) **SUCCESS** at `2cbc0975` (resolve 5s; verify ubuntu-latest 4m24s; verify windows-latest 13m41s; required-ci 2s). Earlier PR-attached SUCCESS: [33704289512](https://github.com/agentkernel/cognitive-os/actions/runs/33704289512) at `e2ed3ddb`; Option A head [33724823585](https://github.com/agentkernel/cognitive-os/actions/runs/33724823585) **SUCCESS** at `329b1b94`
- Running report: [2026-09-03-personal-p0-t01-d02-toolchain-report.md](2026-09-03-personal-p0-t01-d02-toolchain-report.md)

## Acceptance mapping (`P0-T01/D02` slice row + plan.md 关闭门)

| Acceptance item | Implementation | Negative / evidence |
|---|---|---|
| Owner decision (a): local-only override; do not change tracked `rust-toolchain.toml` | `rustup override set 1.97.1-x86_64-pc-windows-msvc` for `D:\agent-kernel` and `D:\agent-kernel-wt-p0-t01` (rustup settings only). `.cargo/config.toml` is not gitignored, so it was not used. `git diff origin/main -- rust-toolchain.toml` empty at merge | report §2 S1–S4; CI still resolves the tracked pin (C3) |
| `cargo build --workspace --locked` on this host | pass at `27a9da0e` under the MSVC override (`host: x86_64-pc-windows-msvc`, `link.exe` 14.44.35228.0) | report U1 |
| `cargo test --workspace --locked -- --test-threads=1` | 1356 pass / 4 fail / 3 ignored over 147 binaries. The 4 failures are `kernel-server` `tool_executor` symlink-fixture setup panics (OS 1314, non-elevated shell, Developer Mode off). Tests not weakened; those four remain `not-run (host privilege)` here and pass on elevated `CI-WINDOWS-MSVC-01` | report U2 / U2a |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | pass | report U3 |
| `cargo fmt --all -- --check` | pass | report U4 |
| Environments registry §3 rewrite; `RUST-LINK-DEV-WIN-GNU-01` retained as GNU-host history; MSVC override is the current local allowlist; capability ceiling unchanged | `PERSONAL-TEST-ENVIRONMENTS.md` §1.1/§2/§3; `AGENTS.md` §5/§6; Operating Model §3.0; baseline README; rules 10/15; bilingual handbook + fingerprints | report §4 G1–G7; 6c guard fragments kept |
| `verify:local` Option A: re-pin to CI counts 89/62/27 | `scripts/v01-auto-run.ps1` / `.sh`; pin-guard test in `tools/test/check.test.mjs`; handbook/AGENTS wording | report §6–§7 A1–A3; full orchestrator run `not-run` (A4, disk/time) |
| required CI still green; no `rust-toolchain.toml` change | PR #314 required-ci SUCCESS at `2cbc0975` | this closure |

## Non-claims

Claim ceiling `hypothesis`. Local development evidence only. Capability ceiling of `DEV-WIN-GNU-01` is **unchanged**: not a supported product Windows environment, not `DEV-WINDOWS-NATIVE-OPC-01`, not B01-W. Local Rust results never promote Gate / release / Profile / Windows-support claims. `not-run` is never pass. Four symlink-fixture tests remain host-privilege `not-run` here. Full `pnpm run verify:local` was not run (A4). This close does not claim `P13-T07`.

## Next unique action

Closed. Continue `P13-T06` on Draft PR [#316](https://github.com/agentkernel/cognitive-os/pull/316) (`personal/P13-T06-group-chat`). After T06 closes, claim ready Phase 13 cards per the owner 2026-09-03 instruction. Do not auto-claim `P11-T15`.
