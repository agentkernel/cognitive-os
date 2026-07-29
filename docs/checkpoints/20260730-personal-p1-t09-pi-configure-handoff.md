# Personal P1-T09 Non-secret Pi Configuration Handoff

**Date:** 2026-07-30
**Branch:** `lane/personal-p1-t08-mvp-single-service`
**Task state:** install-to-first-conversation route `in-progress`; `P1-T09 / B01`
remains `not-started`

## Completed atomic slice

- Confirmed the reviewed pinned Pi `0.81.1` Extension loading syntax from the
  trusted upstream source commit
  `20be4b18d4c57487f8993d2762bace129f0cf7c6`:
  `--extension <path>` / `-e <path>`. The authoritative parser is
  `packages/coding-agent/src/cli/args.ts` at that commit; its `printHelp`
  output describes the same option.
- Added `cognitive pi configure --executable <absolute-path>
  --extension-entry <absolute-path>` with an optional hermetic
  `--runtime-root`.
- The command atomically writes the existing non-secret `pi.json` surface
  only: `schema_version`, `surface`, `executable_path`, and
  `extension_entry_path`. It rejects relative paths and rejects every flag
  outside this narrow configuration set, including `--api-key-file`.
- This slice does not start Pi or run the Extension. It does not read Provider
  configuration, `SecretRef`, SecretStore material, `selected-model.json`,
  SQLite, or authority state. Daemon-side Pi file/version observation remains
  the owner of readiness classification.

## Failure-first and verification

The first focused test failed before implementation because the configuration
writer returned its deliberate `not implemented` error:

```text
cargo test -p admin-cli --lib \
  personal_cli::pi::tests::configuration_rejects_relative_paths_before_writing_any_file --locked
# failed as expected before implementation (exit 101)
```

After implementation, the following ran in `windows_wsl2_linux_guest` with
`CARGO_TARGET_DIR=/tmp/cognitiveos-p1t09-pi-route`:

```text
cargo test -p admin-cli --lib personal_cli --locked
# 9 passed

cargo test -p kernel-server --test p1_t07_pi_readiness --locked
cargo test -p kernel-server --test p1_t05_personal_readiness --locked
cargo test -p kernel-server --test p1_t07_provider_proxy --locked
cargo test -p admin-cli --test p1_t06_cognitive_cli --locked
# passed

cargo clippy -p kernel-server -p admin-cli -p pi-agent-adapter \
  -p cognitive-provider-transport --all-targets -- -D warnings
# initially rejected test-only expect calls; after adding the local test-module
# lint allowance, the same strict command passed

cargo fmt --all -- --check
pnpm run check:consistency
git diff --check
# passed
```

## Contract and claim boundary

No public DTO, SSE shape, registry entry, schema, transition, conformance
vector, or error-code asset changed. No Pi process was launched, and no
Provider request was made. Pi remains a client: this command cannot make a
Task completion, Effect dispatch, or authority state transition.

This is implementation and local-test evidence only. It is not Pi launch or
Extension-load evidence, a binary deterministic Provider fixture, a real first
conversation, native Secret Service evidence, B01, a Gate, release, or Profile
claim.

## Remaining work and next safe entry

1. Add the smallest fail-closed `cognitive pi` launch command using the now
   confirmed `--extension` / `-e` syntax. Before spawning, it must require the
   daemon endpoint and all existing daemon-side readiness prerequisites,
   validate the Pi pin/config, clear Provider-like inherited environment, and
   never put Provider material in Pi argv, environment, configuration, logs,
   SQLite, test output, or evidence.
2. Add binary-level composition-root coverage only after the launch/session
   boundary is designed; retain Rustls-only production Provider transport and
   keep any fixture behind a test-only internal seam.
3. Keep the unrelated uncommitted
   `apps/kernel-server/src/personal/server.rs` modification untouched and out
   of all staging and commits.

**Commit:** `c34f59e` (`P1-T09: add non-secret Pi configuration`).
**Push status:** successful. `git push -u origin HEAD` completed successfully;
local tracking is `origin/lane/personal-p1-t08-mvp-single-service`, and
`git ls-remote` confirmed
`c34f59e083db91dbb405a9cf4cab7167275a4938` at that remote branch.
