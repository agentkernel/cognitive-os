# Personal P1-T08 Closeout and XDG Endpoint Handoff

**Date:** 2026-07-29
**Base:** `main@4a807e6`
**Branch:** `lane/personal-p1-t08-mvp-single-service`
**Task status:** P1-T08 `done`; install-to-first-conversation route `in-progress`

## Completed and committed

Commit `7d18c95` completes P1-T08's release-shaped, non-production native
installer campaign and records the formal task closeout. The campaign uses the
inspected shell, fixed production Rust adapter, and fixed
`/usr/bin/systemctl --user` boundary on `personal-linux-native-01`. It covers
clean install `.3`, healthy upgrade `.4`, pre-pointer failure `.5`, and
post-pointer confirmation failure `.6`. Both failed paths restored the
canonical unit, active service, 48181 liveness, and non-secret
`active-version=.4`; immutable campaign versions remained retained.

The same commit begins the next first-conversation route prerequisite:

- installed `kernel-server --personal` uses real user XDG roots when no
  hermetic `--runtime-root` is supplied;
- after successful loopback bind the daemon atomically publishes the actual
  endpoint to `state/cognitiveos/daemon-endpoint.json` and removes it on
  orderly shutdown;
- `cognitive daemon start` no longer writes an endpoint before bind and waits
  for daemon-owned lock/bootstrap/endpoint state before reporting success.

`PERSONAL-DEVELOPMENT-PLAN.md` now reports P1-T08 as `done`: Phase 1 is 8
done / 0 in-progress / 1 not-started; total is 15 done / 0 in-progress / 38
not-started. These are task-accounting statements only, not B01, Gate,
Profile, production release, signing, containment, uninstall, or
first-conversation claims.

## Verification executed

In `windows_wsl2_linux_guest`:

```text
cargo test -p cognitive-runtime --test linux_bundle_campaign_builder \
  --test linux_bundle_service_lifecycle --test linux_bundle_single_service \
  --test linux_bundle_installer_adapter
# 20 passed

cargo test -p kernel-server --test p1_t04_personal_daemon
# 5 passed

cargo build -p kernel-server
cargo test -p admin-cli --test p1_t06_cognitive_cli
# 5 passed

cargo clippy -p cognitive-runtime --all-targets -- -D warnings
cargo clippy -p kernel-server --all-targets -- -D warnings
cargo clippy -p admin-cli --all-targets -- -D warnings
cargo fmt --all -- --check
pnpm run check:consistency
git diff --check
# passed
```

The initial full admin-cli run in an otherwise fresh target directory failed
only because its integration test requires the sibling `kernel-server` binary.
After building that binary in the same target directory, all five tests passed.

## Push state

`git push -u origin HEAD` was attempted twice after the commit. Both attempts
failed locally before upload with:

```text
schannel: failed to receive handshake, SSL/TLS connection failed
```

The branch and working tree are clean locally. Do not state that the commit is
remote-visible until a later successful push.

## Next safe batch

Continue the install-to-first-conversation route with Provider discovery and
selected-model persistence. `ProviderDiscoveryService::discover_probe_and_persist`
already clears stale selected-model state, executes catalog/capability probes,
and atomically writes `selected-model.json` only for a chat-capable selection.

Do not make `admin-cli` depend on `kernel-server`, and do not add HTTP/TLS to
`cognitive-secret`. Extract the existing bounded HTTPS Rustls transport from
`apps/kernel-server/src/personal/provider_proxy.rs` into a small shared adapter
crate consumed by both approved composition roots. Then add an admin-cli-private
composition seam with a deterministic injected Provider transport test:

1. successful exact-catalog discovery persists the provider digest and
   `selected-model.json` without leaking key material;
2. subsequent failed discovery clears stale selected-model state;
3. `--model-id` maps to exact-catalog selection, not silent manual fallback;
4. production `cognitive init` uses the native SecretStore, shared HTTPS
   adapter, and redacted result summary only.

This needs no Lane-CTR contract change. It remains local implementation/test
evidence; Pi launch, real first conversation and B01 stay pending.
