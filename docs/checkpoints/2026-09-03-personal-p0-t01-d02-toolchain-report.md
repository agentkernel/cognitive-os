# P0-T01/D02 — running report (local Rust toolchain repair on `DEV-WIN-GNU-01`)

- Task / slice: formal task `P0-T01` (Phase 0; `done` → `in-progress` while this Slice is open), Delivery Slice `P0-T01/D02`
- Lease: `lease/personal/P0-T01/toolchain-repair` (Lane-DOC + narrow Lane-CFR guard fragments)
- Branch: `personal/P0-T01-D02-toolchain` (worktree `D:\agent-kernel-wt-p0-t01`, created from `origin/main@27a9da0e`)
- Owner decision already given (2026-09-03): **(a) local-only override**. Tracked `rust-toolchain.toml` is not modified.
  `git check-ignore -v .cargo/config.toml` → exit 1 (not ignored), therefore no `.cargo/config.toml` is created; only
  `rustup override set` directory overrides are used.
- Still-open owner sub-decision: `pnpm run verify:local` + `scripts/v01-auto-run.*` — re-pin to CI counts (89/62/27) or
  deprecate/remove (§6). Not chosen here.
- Environment for every local unit: `DEV-WIN-GNU-01` (Windows 10 Pro 10.0.19045; Cursor Shell = Windows PowerShell 5.1.19041.6456)
- Claim ceiling: `hypothesis`. Local development evidence only — the capability ceiling of this host is **unchanged**:
  not a supported product Windows environment, not `DEV-WINDOWS-NATIVE-OPC-01`, not B01-W; local Rust results never
  promote Gate/release/Profile/Windows-support claims. `not-run` is never pass.
- Reporting rule: `TEST-REPORT-INCREMENTAL-01` — append on completion; append-only.

## 1. Fact probe (read-only; before any change)

| # | Fact | Instrument | Result |
|---|---|---|---|
| P1 | Active Rust toolchain in the worktree | `rustc -vV` | `rustc 1.97.1 (8bab26f4f 2026-07-14)`, **`host: x86_64-pc-windows-gnu`**, LLVM 22.1.6 — active because `rust-toolchain.toml` (`channel = "1.97.1"`) resolves against the GNU default host |
| P2 | Cargo | `cargo -vV` | `cargo 1.97.1 (c980f4866 2026-06-30)`, host `x86_64-pc-windows-gnu`, `os: Windows 10.0.19045 (Windows 10 Pro) [64-bit]` |
| P3 | rustup | `rustup --version`; `rustup show` | rustup 1.29.0; `RUSTUP_HOME=D:\DevEnv\Rustup`, `CARGO_HOME=D:\DevEnv\Cargo`, `CARGO_TARGET_DIR=D:\DevEnv\CargoTarget` (user environment); default host `x86_64-pc-windows-gnu`; installed toolchains `stable-x86_64-pc-windows-gnu` (default), `1.97.1-x86_64-pc-windows-gnu`, `1.97.1-x86_64-pc-windows-gnullvm`, `1.97.1-x86_64-pc-windows-msvc`; `rustup override list` → `no overrides` |
| P4 | MSVC toolchain contents | `rustup target/component list --installed --toolchain 1.97.1-x86_64-pc-windows-msvc` | target `x86_64-pc-windows-msvc`; components `cargo`, `rust-std`, `rustc` only — **`rustfmt` and `clippy` absent** (must be added for the Slice's clippy/fmt commands) |
| P5 | `rustc +1.97.1-x86_64-pc-windows-msvc -vV` | rustup toolchain selector | same commit `8bab26f4f`, **`host: x86_64-pc-windows-msvc`** |
| P6 | Visual Studio Build Tools | `vswhere -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64` | installationPath `D:\VSBuildTools`; product display version `17.14.37 (July 2026)`; installationVersion `17.14.37516.0` |
| P7 | `link.exe` | `Get-ChildItem D:\VSBuildTools -Recurse -Filter link.exe` | `D:\VSBuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64\link.exe`, file/product version **14.44.35228.0** (plus Hostx64\x86, Hostx86\x64, Hostx86\x86 copies of the same version); `where.exe link.exe` → not on the Cursor Shell PATH (exit 1) |
| P8 | Windows SDK | `${env:ProgramFiles(x86)}\Windows Kits\10\Lib` | `10.0.26100.0` |
| P9 | PowerShell 7 | `pwsh --version` | `PowerShell 7.6.5` (present; `#Requires -Version 7.0` of `scripts/v01-auto-run.ps1` is satisfiable) |
| P10 | Git line endings | `git config --show-origin core.autocrlf`; `.gitattributes` | `core.autocrlf=true` from `C:/Program Files/Git/etc/gitconfig` (system scope); tracked `.gitattributes` has `* text=auto eol=lf` (+ binary image rules), which overrides autocrlf for every tracked text path — no local Git configuration change is needed |
| P11 | Disk | `Get-PSDrive` | `D:` 5.5 GB free (target dir lives here), `C:` 3.4 GB free, `E:` 0 GB free; existing `D:\DevEnv\CargoTarget\debug` = 0.33 GB of GNU-era artifacts |
| P12 | Workspace size | `Cargo.lock`, `Cargo.toml` | 299 locked packages; 13 workspace members |

## 2. Toolchain switch (owner option (a), local-only)

| # | Unit | Instrument | Result | Notes |
|---|---|---|---|---|
| S1 | Directory override, task worktree | `rustup override set 1.97.1-x86_64-pc-windows-msvc` in `D:\agent-kernel-wt-p0-t01` | **pass** | `info: override toolchain for D:\agent-kernel-wt-p0-t01 set to 1.97.1-x86_64-pc-windows-msvc` |
| S2 | Directory override, main checkout | `rustup override set --path D:\agent-kernel 1.97.1-x86_64-pc-windows-msvc` | **pass** | overrides are stored in rustup's own settings (`D:\DevEnv\Rustup\settings.toml`), not in the repository; `git status -- .cargo rust-toolchain.toml` stays empty |
| S3 | Missing components on the MSVC toolchain | `rustup component add rustfmt clippy --toolchain 1.97.1-x86_64-pc-windows-msvc` | **pass** | downloaded `rustfmt` + `clippy`; `cargo fmt --version` → `rustfmt 1.9.0-stable (8bab26f4f6 2026-07-14)`; `cargo clippy --version` → `clippy 0.1.97 (8bab26f4f6 2026-07-14)` |
| S4 | Host triple after the switch | `rustc -vV`; `rustup show active-toolchain` | **pass** | **`host: x86_64-pc-windows-msvc`**, same rustc commit `8bab26f4f` / release `1.97.1` as the pinned channel; `1.97.1-x86_64-pc-windows-msvc (directory override for 'D:\agent-kernel-wt-p0-t01')`; `rustup override list` → both directories listed |

Mechanism recorded for the registry: rustup precedence is `RUSTUP_TOOLCHAIN` env → **directory override** → `rust-toolchain.toml` → default. The override therefore wins over the tracked `rust-toolchain.toml` in exactly these two directories on this machine and nowhere else; CI (`actions-rust-lang/setup-rust-toolchain`) and every other clone still resolve `rust-toolchain.toml` unchanged. Any new local worktree needs its own `rustup override set` (per path). Reverting is `rustup override unset --path <dir>`.

## 3. Local Rust validation units (the only cargo runs authorized on this host, by this Slice)

All units: `DEV-WIN-GNU-01`, worktree `D:\agent-kernel-wt-p0-t01` at exact revision `27a9da0e` (= `origin/main` at claim
time; the Slice's own commits are documentation-only and do not change Rust sources), toolchain
`1.97.1-x86_64-pc-windows-msvc` via directory override (`rustc -vV` → `host: x86_64-pc-windows-msvc`, commit
`8bab26f4f`), linker `D:\VSBuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64\link.exe` 14.44.35228.0 located by rustc
itself (no `vcvars`, no PATH edit), `CARGO_TARGET_DIR=D:\DevEnv\CargoTarget` (user environment, pre-existing).

| # | Unit | Instrument | Result | Notes |
|---|---|---|---|---|
| U1 | Workspace build, default `dev` profile | `cargo build --workspace --locked` | **pass** | `Finished dev profile [unoptimized + debuginfo] target(s) in 3m 44s`, exit 0. Three `warning: linker stdout: 正在创建库 … .lib 和对象 … .exp` (`#[warn(linker_messages)]`) for the `cognitive`, `kernel-server` and `p1_t09_provider_fixture` binaries — `link.exe` on this machine prints its "Creating library …" notice in the Chinese locale; this is the same warning class the hosted Windows CI job sees in English and it is not an error. Disk after the build: `D:` 2.88 GB free (target `debug/` ≈ 3.1 GB, of which `deps/*.pdb` = 706 MB). |
| U1a | Disk feasibility check for the test build (recorded, not a test) | `Get-ChildItem … tests\*.rs` count; drive free space | **fail-closed decision** | 111 integration-test source files ⇒ >110 test executables plus unit-test binaries. With full MSVC debuginfo the measured dependency PDBs alone are 706 MB and each test executable would carry its own PDB, so the default-profile test build cannot fit in the remaining 2.88 GB (`C:` 3.4 GB, `E:` 0 GB — no alternative target drive). Deliberately filling the owner's `D:` drive to zero is not an acceptable recovery route. Recovery chosen (reversible, session-only, no tracked or persistent change): `cargo clean` of the shared target dir (removes the stale 2026-07 GNU-era artifacts as well) and `$env:CARGO_PROFILE_DEV_DEBUG = "0"` for the remaining units in this shell session. Debuginfo level does not change what is compiled, linked, or asserted — only the absence of PDB/line tables — so pass/fail semantics of `cargo test` / `clippy` / `fmt` are unaffected; U2–U4 are labelled accordingly. |
| U1b | `cargo clean` of `D:\DevEnv\CargoTarget` | `cargo clean` | **pass** | `Removed 7929 files, 4.2GiB total` (MSVC build from U1 + stale GNU/gnullvm artifacts); `D:` 5.94 GB free afterwards |
| U2 | Workspace tests, serial, `CARGO_PROFILE_DEV_DEBUG=0` (session env) | `cargo test --workspace --locked -- --test-threads=1` | **fail (environment)** — exit 101 after 18m42s | Build of all test targets fit the disk (`D:` never below 2.9 GB free). **107 test binaries executed: 106 `ok`, 1 `FAILED`** — `kernel-server` bin unit tests: `376 passed; 4 failed`. The four failures are all fixture-setup panics in `personal/apps/kernel-server/src/personal/tool_executor/tests.rs:107/117` (`create_test_file_link` / `create_test_directory_link` → `std::os::windows::fs::symlink_file/symlink_dir`): `Os { code: 1314, … "客户端没有所需的特权。" }` = `ERROR_PRIVILEGE_NOT_HELD`. Affected tests: `durable_executor_state_creation_never_follows_a_link_or_reparse_point`, `workspace_mutation_refuses_a_symlinked_target_and_an_escaping_parent`, `workspace_search_rejects_active_directory_swap_to_a_link_or_reparse_point`, `workspace_search_rejects_active_file_swap_to_a_link_or_reparse_point`. Cause on this host (probed read-only): the Cursor Shell process is **not elevated** (`IsInRole(Administrator)` = False), `whoami /priv` does **not** list `SeCreateSymbolicLinkPrivilege`, and Windows Developer Mode is **off** (`HKLM\…\AppModelUnlock` key absent). Creating symlinks on Windows needs one of those; hosted `windows-latest` CI runs elevated, which is why the same tests pass on `CI-WINDOWS-MSVC-01`. This is a host-capability fact, not a product or test defect — the tests are **not** weakened. Because cargo stops at the first failing target, the `kernel-server` integration-test binaries and anything after them did not run in this invocation → U2a. |
| U2a | Full workspace test denominator, serial, `CARGO_PROFILE_DEV_DEBUG=0`, `--no-fail-fast` | `cargo test --workspace --locked --no-fail-fast -- --test-threads=1` | **fail (environment) — otherwise green**: exit 101 after 18m19s (no recompilation) | **147 test binaries: 146 `ok`, 1 `FAILED`; 1356 tests passed, 4 failed, 3 ignored** (12 doc-test targets included). The single failing target is again `-p kernel-server --bin kernel-server` with exactly the same four `tool_executor` symlink/reparse-point fixture tests (OS error 1314, host privilege — see U2). Every `kernel-server` integration-test binary (loopback daemon spawns, `node` children, Windows Credential Manager round-trip in `cognitive-secret` `windows_native`, hosted-DSH broker, installer/service fixtures) passed on this host. Honest local reading: workspace tests **pass except 4 privilege-bound fixture tests that are `not-run (host privilege)` here**; the supported evidence for those four remains `CI-WINDOWS-MSVC-01` (elevated runner), where they pass at the same revision lineage. Disk after the run: `D:` ≈ 2.9 GB free. |
| U3 | Clippy, deny warnings, `CARGO_PROFILE_DEV_DEBUG=0` | `cargo clippy --workspace --all-targets --locked -- -D warnings` | **pass** | `Finished dev profile [unoptimized] target(s) in 1m 49s`, exit 0, zero warnings/errors emitted (check mode; the U1 `linker_messages` notice does not arise because nothing is linked) |
| U4 | Formatting | `cargo fmt --all -- --check` | **pass** | exit 0 in 11s (`rustfmt 1.9.0-stable` from the MSVC toolchain) |

Summary of §3 at `27a9da0e` on the local MSVC override: **build pass; clippy pass; fmt pass; tests 1356 pass / 4 fail (host
privilege, `not-run` here) / 3 ignored across 147 binaries.** The Slice's acceptance sentence "the three cargo commands pass
on this host" is therefore met for build and clippy, and met for test **except** the four symlink-fixture tests that this
non-elevated session cannot set up; that residual is a registered host limitation, not a code defect, and is written into
the environments registry §3. CI is unaffected by any of this: the override is invisible to `rust-toolchain.toml`
consumers and no tracked Rust/CI file changed.

Observation recorded, not decided: the workspace test suite exercises the **real Windows Credential Manager** on this host
(`cognitive-secret` `windows_native::real_credential_manager_roundtrip_rotate_and_delete … ok`) exactly as it does on
`CI-WINDOWS-MSVC-01`; the test creates and deletes its own entry. Anyone running local tests should know that side
effect exists.

## 4. Connected documentation and static gates

Rewritten in this delivery (all bilingual where the handbook is involved; `RUST-LINK-DEV-WIN-GNU-01` retained everywhere
as the GNU-host history fact, the MSVC override added as the current local allowlist, capability ceiling unchanged):

- `docs/plan/PERSONAL-TEST-ENVIRONMENTS.md` §1.1 routing rows + paragraph, §2 summary row, §3 full re-registration
  (platform, override mechanism, linker, tools, allowlist, local limitations, GNU history, no-repeat rule, transfer,
  ceiling, evidence links);
- `AGENTS.md` §5 Rust row and §6 (`RUST-LINK-DEV-WIN-GNU-01` bullet, PATH bullet, `verify:local` bullet, local-command table);
- `docs/governance/DEVELOPMENT-OPERATING-MODEL.md` §3.0 items 2–3;
- `personal/tests/baseline/README.md` (new "Local MSVC Override" section; command-routing paragraph);
- `.cursor/rules/10-autonomous-personal-development.mdc` (routing bullet, forbidden-stop bullet) and
  `.cursor/rules/15-owner-directed-evaluation-campaign.mdc` (still-applies bullet); rules 00/20 do not restate the rule;
  30/40 are untracked and untouched;
- handbook `developer/development-environments`, `ai/validation-commands`, `ai/safe-editing`, `reference/compatibility`,
  `developer/conformance-and-testing` (en + zh-CN) + `fill-handbook-fingerprints` (also refreshed `ai/source-of-truth`
  and `developer/contributing-workflow` fingerprints, whose sources `AGENTS.md`/`package.json`-routed pages changed);
- `docs/plan/PROGRESS.md` (snapshot header, unique-next paragraph, Active task lease row, new P0-T01/D02 row, Layer 1
  167/133/2/1/15/34, Layer 2 slice row), `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md` (status line, Phase 0 + total
  summary rows, P0-T01 row `in-progress`, `P0-T01/D02` slice row, Phase 13 配套维护 row), `docs/plan/plan.md`
  (`P0-T01/D02` card status), `docs/plan/PARALLEL-LANES.md` (own lease row only).
- `tools/src/check-consistency.mjs` 6c guard fragments were **not** changed: the rewritten documents keep every required
  fragment (`RUST-LINK-DEV-WIN-GNU-01`, `linker exit 121`, `Do not repeat them`, `No-repeat rule`, `must not rerun`,
  `linker reported exit code \`121\``) as the GNU-history statement.

| # | Unit | Instrument | Result | Notes |
|---|---|---|---|---|
| G1 | fingerprints | `node tools/src/fill-handbook-fingerprints.mjs` | **pass** | 10 pages updated (5 × 2 locales) |
| G2 | consistency (first run) | `pnpm run check:consistency` | **fail → fixed** | (a) 9 × `broken relative link … (exists locally but is not tracked by Git)` for this very report — the P0-T09 tracked-only check doing its job on a not-yet-staged file; resolved by `git add` of the report; (b) `AGENTS.md: command/environment guard is missing required fragment: linker exit 121` — my rewrite had split the fragment across a line break; wording adjusted |
| G3 | consistency (rerun) | `pnpm run check:consistency` | **pass** | `OK (… tracked-only links … leases, and Phase 13 build-order edge set verified)` |
| G4 | handbook + generator | `pnpm run check:handbook`; `node tools/src/generate-handbook.mjs --check` | **pass** | `OK (58 documents x 2 locales, 9 generated)`; `18 pages byte-identical` |
| G5 | agent rules | `pnpm run check:rules` | **pass** | `OK (4 rules, 0 commands, 88 path references, 5 local-only warning(s), path existence = git-tracked)` |
| G6 | whitespace | `git diff --check` | **pass** | clean |
| G7 | final static rerun after the §3 limitation paragraphs were added | same as G3–G6 | see §5 | recorded there with the commit |

## 5. Checkpoint, Draft PR, required CI

_Appended below as each unit completes._

## 6. Owner sub-decision left open: `pnpm run verify:local` / `scripts/v01-auto-run.*`

Facts (read-only inspection at `27a9da0e`, no orchestrator run — it is not among the cargo commands this Slice authorizes):

- `package.json` `verify:local` → `scripts/v01-auto-run-entry.mjs` → `pwsh -NoProfile -File scripts/v01-auto-run.ps1` on
  Windows (falls back to `powershell.exe`), `bash scripts/v01-auto-run.sh` elsewhere. pwsh 7.6.5 is present, so the
  `#Requires -Version 7.0` header is no longer a blocker.
- Both scripts pin `total_vectors 85 / pass 60 / not-run 25` and `self_check_min 41`; `ci.yml` pins **89 / 62 / 27** and
  `must_flip ≥ 40`. Every run therefore ends `VERIFY-PINS = auto_fail`, `stopped = true`, exit 1 — the orchestrator is red by
  construction today, independent of the toolchain.
- Its BOOT stage runs `cargo build --workspace --locked`; before the override that was the forbidden GNU link. In an
  override directory that step is now allowed (U1), but the orchestrator also runs
  `cargo run … conformance-runner`, `cargo test -p admin-cli --test m5_deterministic_fallback`,
  `cargo test -p cognitive-management --test m5_fallback_verbs`, `cargo test -p kernel-server --test m5_http_sse`,
  `cargo test -p cognitive-runtime --lib sandbox::tests` and the `perf::tests::overhead_report_…` unit — all of these
  targets still exist in the tree (checked), so nothing but the pins is stale.
- Other references that would move with the decision: `AGENTS.md` §6 (wording only), handbook
  `ai/validation-commands` + `user/known-limitations` (bilingual "known stale entry"), `tools/test/check.test.mjs`
  ("POSIX and Windows verify orchestrators share evidence safeguards" reads both scripts and asserts their safeguard
  strings), `docs/plan/templates/v01-auto-run-summary.schema.json`, `docs/prompts/v01-auto-*.md`, and the archived
  `docs/plan/archive/V01-*.md` plans (history; untouched either way).

| Option | What changes | Pros | Cons |
|---|---|---|---|
| **A. Re-pin to CI counts** | `scripts/v01-auto-run.ps1` + `.sh`: `total_vectors 89`, `pass 62`, `not-run 27` (and keep `self_check_min 41` ≥ CI's 40); a one-line "re-pinned 2026-09-03 to `ci.yml`" comment; handbook "known stale entry" → "usable locally inside an MSVC-override directory"; `AGENTS.md` §6 wording | Keeps a one-shot local L0–L3 evidence orchestrator (summary.json + sha256 manifest) that now *can* execute end-to-end on this machine; smallest diff; the existing repo-tools safeguard test keeps guarding both scripts; CI pins and script pins agree again | The pins are a second copy of `ci.yml`'s numbers and will drift again at the next vector change unless the orchestrator learns to read them from one place (a small follow-up); verifying the re-pin honestly means running the whole orchestrator locally (≈ full build + conformance runner + focused tests, ~15–25 min, extra target-dir disk) which this Slice's cargo authorization does not yet cover; the orchestrator's V01-era non-claims list and `docs/plan/archive` plan are historical framing that would also deserve a refresh |
| **B. Deprecate / remove** | Delete `scripts/v01-auto-run.{ps1,sh,-entry.mjs}` and the `verify:local` script from `package.json`; delete the safeguard test in `tools/test/check.test.mjs` and `docs/plan/templates/v01-auto-run-summary.schema.json`; handbook "known stale entry" removed (bilingual) + `AGENTS.md` §6 bullet removed; `docs/prompts/v01-auto-*.md` left as history with a one-line deprecation note | Removes a red-by-construction entry point and the duplicated pin numbers for good; nothing else in the tree depends on the summary schema; the individual §5 commands + required CI already cover every check the orchestrator ran | Loses the only local one-shot evidence bundle (digest manifest, platform label, human-gate defaults); a larger, cross-lane diff (Lane-CFR test + `package.json` + templates) that needs its own required-CI round; if a future Slice wants a local orchestrator it starts from scratch |

**Recommendation:** **Option A (re-pin)** — just re-pin the two scripts and add the comment; making the pin block read
its numbers from `ci.yml` is a possible later follow-up, not part of this Slice. Reasoning: with the
MSVC override the orchestrator is finally executable on this machine, the stale numbers are its only defect, and the
repo-tools safeguard test already protects its evidence semantics; removal throws away working tooling to save two
constants. If the owner picks A, the follow-up inside this Slice is: re-pin, run `pnpm run verify:local` once in the
override directory (recorded here as the authorizing Slice), update the handbook/AGENTS wording, and close P0-T01 back
to `done`. If the owner picks B, the follow-up is the deletion set above plus a required-CI round.

**Not decided here.** This is an owner boundary (Operating Model §2.4 item 3, tooling/support semantics); the Slice
stays `in-progress` and this choice is the unique next action in `PROGRESS.md`.
