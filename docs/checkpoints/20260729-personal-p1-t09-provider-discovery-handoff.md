# Personal P1-T09 Provider Discovery Prerequisite Handoff

**Date:** 2026-07-29
**Branch:** `lane/personal-p1-t08-mvp-single-service`
**Task status:** install-to-first-conversation route `in-progress`; P1-T09 / B01
`not-started`

**Commit:** `702125d` (`P1-T09`). A lane-branch push was attempted after the
commit but made no progress locally and was stopped; remote visibility remains
unconfirmed.

## Completed local implementation batch

This batch provides the Provider-discovery prerequisite while preserving the
formal P1-T09 state. It adds `cognitive-provider-transport`, a shared adapter
that owns the sole concrete Rustls `ProviderTransport` implementation. The
adapter is used directly by `admin-cli` and re-exported by the daemon proxy so
`apps/kernel-server/src/personal/server.rs` needs no change.

- HTTPS-only, redirect-free, credential-free URL, header-injection, timeout,
  cancellation, and 1 MiB response protections remain fixed.
- `cognitive init` now composes the native SecretStore with the shared adapter,
  runs `ProviderDiscoveryService`, and maps `--model-id` to
  `ModelSelection::ExactCatalog`.
- A successful chat-capable probe atomically persists `selected-model.json` and
  the non-secret snapshot digest; discovery failures clear stale selection.
- Private deterministic tests cover exact-catalog persistence, stale-selection
  removal on a missing catalog model, and result/error redaction. The existing
  CLI integration remains hermetic and never performs Provider egress without
  Provider flags.

## Verification executed

In `windows_wsl2_linux_guest` with
`CARGO_TARGET_DIR=/tmp/cognitiveos-p1t09-provider-discovery`:

```text
cargo test -p admin-cli --lib personal_cli::init::tests --locked
# 2 passed

cargo test -p cognitive-provider-transport --locked
# 2 passed

cargo test -p kernel-server --test p1_t07_provider_proxy --locked
# 2 passed

cargo test -p admin-cli --test p1_t06_cognitive_cli --locked
# 5 passed

cargo clippy -p admin-cli -p kernel-server -p cognitive-provider-transport \
  --all-targets -- -D warnings
cargo fmt --all -- --check
pnpm run check:consistency
git diff --check
# passed
```

The local Windows GNU `cargo check -p admin-cli` attempt remains blocked before
crate checking by the known MinGW linker exit 121. No Windows evidence or claim
is added.

## Task accounting and claims

`P1-T09` remains `not-started`: no Pi launch, actual first conversation,
development smoke, usability campaign, clean Linux VM B01 run, product Gate,
release, or Profile evidence was executed. The only active item remains the
install-to-first-conversation implementation route.

## Next safe batch

Implement the bounded Pi launch/configuration path and kernel readiness facts,
then add a true binary-level deterministic Provider fixture test for the full
daemon/Pi readiness path. Keep Pi non-authoritative and unable to read Provider
configuration or SecretRefs. Do not start B01 until that route, native Secret
Service, pinned Pi, reset procedure, workload, redaction policy, and campaign
threshold are preregistered.
