# Personal P0-T01 Baseline Record

This record defines the reproducible toolchain baseline for Personal planning
work. It is a development and CI evidence record, not a product-release,
Profile, or Personal feature-completion claim.

## Recorded Baseline

- Recorded date: 2026-07-25
- Source revision: `01ceb93ec3189af599a0754f34ea76b76a363ff0`
- Rust: `1.97.1`, pinned by `rust-toolchain.toml`
- pnpm: `10.33.2`, pinned by `package.json`
- CI Node: `22`; local Node used for the TypeScript measurements: `24.15.0`
- CI evidence: [run 30140381194](https://github.com/agentkernel/cognitive-os/actions/runs/30140381194)

The CI run completed successfully for the exact source revision on both
supported release-validation runners:

| Runner | Result | Elapsed wall time |
|---|---|---:|
| `ubuntu-latest` | pass | 88 seconds |
| `windows-latest` | pass | 305 seconds |

The Windows CI job uses the hosted Windows/MSVC Rust host selected by the
runner and `actions-rust-lang/setup-rust-toolchain`. That is the supported
Windows validation combination for this repository.

## Local Windows GNU Result

The local `x86_64-pc-windows-gnu` host is not a supported baseline. The
following commands were executed on 2026-07-25:

1. `cargo fmt --all -- --check`: pass.
2. `cargo build --workspace --locked`: fail with Cargo exit code `101` while
   invoking `x86_64-w64-mingw32-gcc`; the linker reported exit code `121`.
3. The documented LLVM-MinGW `CC`/`AR` and `dlltool` shim workaround was
   applied for a second `cargo build --workspace --locked`; it failed with the
   same Cargo and linker exit codes.

Because the build prerequisite failed, local GNU `cargo test` and
`cargo clippy` were intentionally not claimed as executed. This does not
invalidate the successful hosted Windows/MSVC CI evidence. Do not add a
plaintext fallback, modify the pinned Rust toolchain, or promote the local GNU
host to a supported release-validation environment without a separately
reviewed toolchain decision.

## TypeScript Local Measurement

After `pnpm install --frozen-lockfile` passed, three consecutive executions of
`pnpm -r build` followed by `pnpm -r test` all passed. Their combined wall
times were 29.722, 29.669, and 28.408 seconds; the observed p50 is **29.669
seconds**. These are local development measurements only, not a performance
claim or release gate.

## Reproduction Commands

Run the following from a clean checkout at the recorded revision:

```powershell
pnpm install --frozen-lockfile
pnpm -r build
pnpm -r test
cargo fmt --all -- --check
cargo build --workspace --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
pnpm run check:consistency
node tools/src/gen-matrix.mjs --check
```

The authoritative cross-platform result is the CI workflow in
`.github/workflows/ci.yml`. Local GNU failures must be recorded honestly and
must not be converted into a release claim.
