# Personal P1-T07 Completion Bridge Handoff

**Date:** 2026-07-27
**Branch:** `lane/personal-p1-t07-provider-proxy`
**Status:** implementation and local verification complete; supported CI remains required.

## Delivered

- Added a separate daemon-owned, non-secret `selected-model.json` carrier.
- Cleared selected-model state on provider lifecycle invalidation and failed or
  non-chat-capable discovery probes.
- Added management-only `GET /provider/v1/selected-model`; it reads the
  projection without initializing a secret backend.
- Enforced the daemon-selected model before proxy secret resolution and added
  stable unavailable/mismatch errors for chat completions.
- Added the Pi complete-provider bridge with exactly one projected model and a
  bounded, one-shot `stream:false` completion path through the local daemon.
- Corrected the Pi package test glob so every compiled `*.test.js` test runs.

## Local Evidence

All commands completed successfully in the WSL guest unless noted otherwise:

- `cargo test -p cognitive-secret --test p1_t02_provider_secret --locked`
- `cargo test -p cognitive-secret --test p1_t03_provider_discovery --locked`
- `cargo test -p kernel-server --test p1_t07_provider_proxy --locked`
- `cargo test -p kernel-server --test p1_t07_pi_readiness --locked`
- `cargo test -p kernel-server --locked`
- `cargo fmt --all -- --check`
- `cargo clippy -p kernel-server --all-targets --locked -- -D warnings`
- `pnpm --filter @cognitiveos/pi-cognitiveos build`
- `pnpm --filter @cognitiveos/pi-cognitiveos test`
- `pnpm run check:consistency`
- `git diff --check`

This is local test evidence only. It is not a supported-matrix CI result, Gate,
Profile, containment, or release claim.

## Follow-up

1. Push the branch and obtain/merge a PR after supported CI is green.
2. Keep P1-T07 `in-progress` until all milestone evidence is accepted.
3. Do not stage the recovered pre-existing checkpoint,
   `.cursor/`, `.vscode/`, or `personal-blog/`.
