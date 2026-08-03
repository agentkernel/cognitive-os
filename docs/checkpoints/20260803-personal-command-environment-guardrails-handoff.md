# Personal command and environment guardrails handoff

- Date: 2026-08-03
- Task: P0-T01 corrective environment-baseline maintenance
- Lease: `lease/personal/P0-T01/command-environment-guardrails`
- Branch: `main`
- Change class: corrective repository governance
- Product/normative surface: unchanged
- Task/Gate/release/Profile status: unchanged

## Outcome

Two previously scattered environment facts are now fail-fast development
rules with stable identifiers:

1. `COMMAND-SHELL-PS51`: the current local Cursor Shell is Windows PowerShell
   5.1. Local commands must not use `&&` or `||`. Independent commands use
   separate parallel Shell calls; dependent commands use separate calls or
   `if ($LASTEXITCODE -eq 0) { <next-command> }`. A parser rejection before
   process start is `not-run`, not a test failure.
2. `RUST-LINK-DEV-WIN-GNU-01`: the current local
   `x86_64-pc-windows-gnu` Rust host is an unsupported linking environment
   with the already recorded linker exit 121. Feature Slices must not repeat
   workspace build/test/Clippy/run/bench or the exhausted LLVM-MinGW,
   shim, PATH and toolchain-pin workarounds. Only an explicitly approved and
   leased P0-T01 toolchain-repair Slice may reassess this boundary.

The local GNU allowlist is now limited to non-linking work: Rust formatting,
Node/TypeScript, documentation/static consistency and diff checks. Required
Rust build/test/Clippy validation must be selected before implementation and
routed to `CI-UBUNTU-01`, `CI-WINDOWS-MSVC-01`, or an exact-revision
disposable `DEV-LINUX-NATIVE-01` worktree. If the selected route is
unavailable, validation remains `blocked`/`not-run`; the known GNU failure is
not reproduced first.

These rules are synchronized in `AGENTS.md`, the Development Operating Model,
the Personal environment registry and the P0-T01 baseline. The consistency
checker requires both stable guard IDs and their routing facts. Failure
injection removes each guard and proves the checker rejects the drift.

## Verification

All executed commands were inside the registered `DEV-WIN-GNU-01` non-linking
allowlist:

| Check | Result |
|---|---|
| `pnpm run check:consistency` | pass |
| `pnpm --filter @cognitiveos/repo-tools test` | pass; 5/5 including command/environment guard removal injection |
| `node --check tools/src/check-consistency.mjs` | pass |
| `node --check tools/test/check.test.mjs` | pass |
| `git diff --check` | pass |
| local Rust build/test/Clippy/run/bench | deliberately not-run; prohibited by `RUST-LINK-DEV-WIN-GNU-01` for this corrective Slice |

## Non-claims and next action

This correction does not make local Windows GNU supported, does not create
Windows/MSVC or native Linux evidence, and does not change any task acceptance,
Gate, release or Profile result. Future Slices must read the environment
registry and select a supported validation route before writing or running
Rust tests.
