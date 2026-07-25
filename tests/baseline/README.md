# Personal P0-T01 Baseline Evidence

This document records the reproducible baseline evidence collected for
`P0-T01`. It is a planning and toolchain record, not implementation, conformance,
or Profile evidence.

## Scope and status

- Task: `P0-T01 - Fixed reproducible baseline and supported toolchain`
- Date: 2026-07-25
- Source commit: `f32efd1d188e4da352304228ae73a66a96686d16`
- Status: blocked pending the owner's Windows support decision.
- Secrets: none were read, created, or recorded.

## Pinned CI baseline

The repository CI workflow pins Rust through `rust-toolchain.toml` and runs the
same locked workspace checks on `ubuntu-latest` and `windows-latest`.

| Evidence | Result | Notes |
|---|---|---|
| GitHub Actions run [`30135181631`](https://github.com/agentkernel/cognitive-os/actions/runs/30135181631) | pass | Commit `f32efd1`; CI completed 2026-07-25. |
| `verify (ubuntu-latest)` | pass | Locked install, TypeScript build/tests, Rust build/tests/clippy/fmt, codegen drift, consistency, matrix, conformance, and golden digest steps passed. |
| `verify (windows-latest)` | pass | The same CI steps passed using the hosted Windows toolchain. |

CI is the clean Linux runner evidence for this task. The hosted Windows runner
proves that the pinned repository checks are compatible with its supported MSVC
toolchain; it does not prove support for a local GNU host toolchain.

## Local Windows GNU probe

The local host selected the repository-pinned Rust `1.97.1` GNU toolchain:

```text
host: x86_64-pc-windows-gnu
rustc: 1.97.1
node: v24.15.0
pnpm: 10.33.2
```

| Command | Result | Observation |
|---|---|---|
| `cargo fmt --all -- --check` | pass | Formatting baseline passed. |
| `pnpm install --frozen-lockfile` | pass | Lockfile was current. |
| `pnpm -r build` | pass | All TypeScript workspace packages built. |
| `pnpm -r test` | pass | All TypeScript workspace tests passed. |
| `pnpm run check:consistency` | pass | 273 requirements, 55 errors, 63 schemas, and 85 vectors verified. |
| `node tools/src/gen-matrix.mjs --check` | pass | Traceability matrix was current. |
| `cargo build --workspace --locked` | fail (exit 101) | GNU linker invocation failed while resolving the runtime libraries, including `-lgcc_eh`; linker returned exit 121. |
| `cargo test --workspace --locked` | not-run | Not run because the prerequisite GNU workspace build failed. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | not-run | Not run because the prerequisite GNU workspace build failed. |

No local MSVC Rust target, Visual C++ compiler, or MSVC linker was available for
an equivalent local run. No toolchain was installed or changed during this probe.

## Required owner decision

`P0-T01` cannot be marked `done` until the supported Windows toolchain is an
explicit product/support decision. The available evidence supports three
mutually exclusive choices:

1. Support Windows CI/MSVC and treat local GNU as unsupported for Personal.
2. Support Windows GNU and fund the linker/runtime repair plus a clean GNU
   verification environment.
3. Do not support Windows for Personal; keep Linux x86_64 as the only supported
   product platform and retain Windows CI solely as repository engineering
   coverage.

After the decision, rerun the applicable clean toolchain baseline and update
this record, the Personal plan, PROGRESS, and the P0-T01 handoff before marking
the task `done`.
