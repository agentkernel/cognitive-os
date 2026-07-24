# Lane-RUN durable installation authority handoff

- Date: 2026-07-24
- Base: `origin/main@7324227` (PR #78, durable KRN InstallationStore)
- Branch: `lane/run-installation-authority`
- Scope: REQ-AGENT-INSTALL-001/002 runtime consumption only; no KRN store or
  normative asset changes.

## 1. Completed in the working tree

- Added the Lane-RUN dependency on `cognitive-store` and a durable installation
  authority that verifies before SQLite stage then commit, mapping only existing
  registered verification, conflict, and store-unavailable error codes.
- Durable install now requires a `DurableInstallationManager` session. The
  manager retains an exclusive in-process lifecycle lock, and it is the only
  public runtime path that can commit durable installation evidence or invoke
  explicit interrupted-staging recovery.
- Added focused tests for verified durable commit with zero capability grants,
  rejected provenance/signature with no visible commit, and recovery of a real
  staged-but-uncommitted row with no committed visibility.

## 2. Remaining boundary

- The local verification blocker is resolved. The MSYS2 `ld.exe` still returns
  Windows error 121, so verification used the existing GCC driver with LLVM
  `lld` (`-Clink-arg=-fuse-ld=lld`), `clang` for native C compilation, and an
  isolated `C:\\cargo-target-cognitiveos` target directory. This is local
  environment state only; no machine path is recorded in project configuration.
- Worktree dependencies were restored with `pnpm install --offline
  --frozen-lockfile`, using the existing local store and zero downloads.
- The runtime mutex enforces manager-only recovery for one authority instance.
  A cross-process/global lifecycle lease would require a distinct Lane-KRN API
  decision; do not present the current runtime session as cross-process locking.
- The batch is ready for its lane commit, push and PR. CI, merge and all
  installation/Profile completion claims remain pending.

## 3. Verification and evidence

- `cargo test -p cognitive-runtime -j 1`: pass, 38 unit + 2 integration tests.
- `cargo clippy -p cognitive-runtime --all-targets -- -D warnings`: pass.
- `cargo fmt --check` and `git diff --check`: pass.
- `pnpm run check:consistency`: pass (273 requirements, 55 error codes, 63
  schemas, 85 vectors; markdown links and traceability verified).
- The conformance runner was not rerun. Retain the merged baseline of 85
  vectors / 60 pass / 25 not-run / 0 fail and self-check 41/41 until a real run.

## 4. Next entry

- Suggested prompt: `docs/prompts/lane-run.md`.
- Worktree: `D:\\agent-kernel\\.worktrees\\run-installation-authority`.
- First action: review the staged lane batch, push it on
  `lane/run-installation-authority`, open a PR and wait for CI. Do not claim
  cross-process locking, capability grants, Task completion, Effects or profile
  conformance.
