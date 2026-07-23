# Lane-RUN single-node serve handoff

- Date: 2026-07-23
- Base: `origin/main@043c4e4`
- Branch: `lane/run-single-node-serve`
- Scope: deployable process foundation only; no Console and no High-Assurance scope.

## Completed

- Added `kernel-server --serve --bind <loopback>`, a long-lived listener that
  accepts multiple sequential HTTP connections.
- The mode rejects non-loopback binds until authenticated deployment middleware
  exists. It does not claim MANAGEMENT_READY, USER_READY, OPERATIONAL, external
  authentication, or enterprise deployment readiness.
- Test-first evidence: the prior binary exited without listening; the new test
  proves two sequential requests are served by one process.

## Verification

- `cargo test -p kernel-server --test m5_http_sse`: 4/4 pass.
- `cargo clippy -p kernel-server --all-targets -- -D warnings`: pass.
- `cargo fmt --check` and `git diff --check`: pass.
- Linux CI initially exposed a real concurrent-child-process fixture flake
  (`ConnectionReset`); the integration tests now serialize listener-owning
  cases with a process-local mutex and pass under the default parallel test
  runner.

## Next entry

- Connect the HTTP management route to a real persisted SQLite store and
  validated PrivilegedManagementSession; remove the current transport-only
  success stub before claiming a usable deployment profile.
