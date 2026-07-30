# Personal P1-T09 Fail-closed Pi Launch Handoff

## Record metadata

- record_type: historical-handoff
- project_id: cognitiveos-personal
- task_id: P1-T09 precursor slice
- lease_id: historical-unstructured-lease (closed)
- status_at_handoff: alpha-conversation in-progress; formal P1-T09 not-started
- gate_status_at_handoff: B01 not-run
- claim_scope_at_handoff: non-claim
- current_status_source: `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md` and
  `docs/plan/PROGRESS.md` Current snapshot
- supersedes: none-recorded
- superseded_by: `20260730-personal-p1-t09-provider-fixture-handoff.md`

**Date:** 2026-07-30  
**Branch:** `lane/personal-p1-t08-mvp-single-service`  
**Historical state at handoff time (not current):** alpha-conversation `in-progress`;
formal P1-T09 `not-started`; B01 `not-run`

## Completed atomic slice

- Added `cognitive pi launch [--runtime-root <dir>]`. It accepts no Provider,
  model, secret, endpoint, or arbitrary Pi passthrough flags.
- Before reading the bootstrap secret, the command validates the daemon-owned
  endpoint document as schema/surface-valid and numeric loopback. It then uses
  the authenticated daemon doctor projection and requires `overall: ready`,
  `first_conversation_ready: true`, and ready system/database/secret/provider/
  daemon/Pi components.
- It validates the exact four-field non-secret `pi.json`, absolute regular
  executable/Extension files, and the exact pinned Pi `0.81.1` version. The
  actual child receives only `--extension <absolute-path>` and an `env_clear()`
  OS-execution allowlist; no Provider configuration, selected-model material,
  SecretRef, secret, SQLite path, or authority state is read or passed.
- Pi remains a client. This command has no Task, Effect, Verification, or
  authority transition path; its success report claims only process spawn
  preparation, never conversation or Extension load.

## Failure-first and verification

The first focused launch test was intentionally run before implementation and
failed to compile because `PiLaunchOptions` and the launch-preparation boundary
did not yet exist (exit 101). After implementation, in
`windows_wsl2_linux_guest` with
`CARGO_TARGET_DIR=/tmp/cognitiveos-p1t09-pi-route`:

```text
cargo test -p admin-cli --lib personal_cli --locked
# 15 passed
cargo test -p kernel-server --test p1_t07_pi_readiness --locked
# 1 passed
cargo test -p kernel-server --test p1_t05_personal_readiness --locked
# 1 passed
cargo test -p kernel-server --test p1_t07_provider_proxy --locked
# 2 passed
cargo test -p admin-cli --test p1_t06_cognitive_cli --locked
# 5 passed
cargo clippy -p kernel-server -p admin-cli -p pi-agent-adapter \
  -p cognitive-provider-transport --all-targets -- -D warnings
# passed
```

The focused tests cover corrupt/non-loopback endpoint, unavailable readiness,
missing/corrupt/relative Pi configuration, missing files, version drift, and
argv/environment minimization. No real Pi binary, Extension load, Provider, or
native Secret Service was exercised.

## Contract and claim boundary

No DTO, SSE shape, error code, registry, schema, transition, or conformance
vector changed. The user-dirty `apps/kernel-server/src/personal/server.rs` was
not read, modified, staged, or committed.

This is implementation and local-test evidence only. It is not a real Pi
Extension-load, Provider conversation, deterministic binary Provider fixture,
native Secret Service, B01, Gate, release, or Profile claim.

## Next safe entry

Design the smallest daemon-only scoped Pi session and binary-level deterministic
Provider fixture without introducing a public contract or passing Provider
material to Pi. Retain the existing dirty `server.rs` boundary.

**Implementation commit:** `1487ebd` (`P1-T09: add fail-closed Pi launch admission`).  
**Handoff commit:** `ebdc068` (`docs: record P1-T09 Pi launch handoff`).  
**Push/remote visibility:** `git push -u origin HEAD` succeeded, and
`git ls-remote origin refs/heads/lane/personal-p1-t08-mvp-single-service`
confirmed `ebdc068901ef32ba231a74f61c17e5a86e48525c`.
