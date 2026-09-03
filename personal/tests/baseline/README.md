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

This result is registered as `RUST-LINK-DEV-WIN-GNU-01`. It is a persistent
command-routing fact for the GNU host, not a diagnostic that feature work
should reproduce. Normal Delivery Slices must not rerun local GNU `cargo
build`, `cargo test`, `cargo clippy`, `cargo run`, `cargo bench`, or exhausted
LLVM-MinGW/shim/PATH workarounds. Only a separately approved and leased P0-T01
toolchain-repair Slice may reassess this baseline.

## Local MSVC Override (P0-T01/D02, 2026-09-03)

The owner chose a **local-only** repair: `rustup override set
1.97.1-x86_64-pc-windows-msvc` for `D:\agent-kernel` and the task worktree,
using the already installed MSVC toolchain and Visual Studio Build Tools
17.14.37 at `D:\VSBuildTools` (`link.exe` 14.44.35228.0, found by rustc through
the Visual Studio setup configuration; no PATH or `vcvars` change). The
tracked `rust-toolchain.toml` is unchanged, so CI and every other clone are
unaffected. Inside an override directory `rustc -vV` reports
`host: x86_64-pc-windows-msvc` and the workspace `cargo build`, `cargo test`,
`cargo clippy` and `cargo fmt` commands run locally; the executed results,
exact revision and the disk-driven `CARGO_PROFILE_DEV_DEBUG=0` session setting
are recorded in the
[P0-T01/D02 running report](../../../docs/checkpoints/2026-09-03-personal-p0-t01-d02-toolchain-report.md).
Those results are local development evidence only; the supported
release-validation combination remains CI Linux and hosted Windows/MSVC, and
the host's capability ceiling in `PERSONAL-TEST-ENVIRONMENTS.md` §3 is
unchanged.

## TypeScript Local Measurement

After `pnpm install --frozen-lockfile` passed, three consecutive executions of
`pnpm -r build` followed by `pnpm -r test` all passed. Their combined wall
times were 29.722, 29.669, and 28.408 seconds; the observed p50 is **29.669
seconds**. These are local development measurements only, not a performance
claim or release gate.

## Command Routing

The local Cursor shell is Windows PowerShell 5.1 (`COMMAND-SHELL-PS51`). Do not
join local commands with `&&` or `||`. Run independent commands separately or
in parallel; run dependent commands as separate calls or with
`if ($LASTEXITCODE -eq 0) { <next-command> }`. A parser rejection is
`not-run`, not a build/test failure.

The following local commands are eligible on `DEV-WIN-GNU-01` because they do
not require Rust linking:

```powershell
pnpm install --frozen-lockfile
pnpm -r build
pnpm -r test
cargo fmt --all -- --check
pnpm run check:consistency
node tools/src/gen-matrix.mjs --check
```

Run Rust compiling/linking commands as **supported validation** only on a
selected supported route such as `CI-UBUNTU-01`, `CI-WINDOWS-MSVC-01`, or an
exact-revision `DEV-LINUX-NATIVE-01` worktree (locally they may additionally
be run for iteration inside a registered MSVC-override directory, see above):

```bash
cargo build --workspace --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

The authoritative cross-platform baseline is the CI workflow in
`.github/workflows/ci.yml`. If the required supported route is unavailable,
record the validation as `blocked`/`not-run`; do not substitute or first retry
the known local GNU failure, and do not convert it into a release claim.
