# Personal P1-T08 Service Health Handoff

**Date:** 2026-07-28
**Base:** `main@7eae31be8e7125620db7bf67488d5bdf0c430f01`
**Branch:** `lane/personal-p1-t08-service-health`
**Implementation commit:** `26bbf12`
**Pull request:** [#112](https://github.com/agentkernel/cognitive-os/pull/112)
**Merge commit:** `3fc6faff44ec6a739852cb74c4ca3c55dac87fd7`

## Delivered boundary

This failure-first slice adds a separate service-aware installation transaction.
It performs complete offline verification before any deployment, unit, or
service mutation; holds the existing OS-backed per-root lease through staging,
candidate controller actions, activation, confirmation, and compensation; and
returns a non-secret receipt only when pointer and active service confirmation
both succeed.

The narrow controller interface receives a checked version and candidate path,
not a manifest, keyring, artifact bytes, secret, user data, or free-form
command. A failure after candidate start stops the candidate and restores the
previous pointer/service; a fail-closed start preflight does not stop the
canonical active unit. First-install failure clears the pointer. A failed stop,
pointer restore, restart, or confirmation is reported as distinct
`rollback incomplete`; it never yields a receipt. Staging and user data remain
inspectable.

`deploy/linux/cognitiveos-personal.service` is a source-controlled,
unrendered user-unit template. The production `SystemdUserServiceController`
accepts loopback health addresses only and fixed `systemctl --user`
arguments, drains child output with an output cap, and kills/reaps a timed-out
child. It rejects unresolved units and missing safe extracted executable
layout before a systemd action. Since the current verified artifact remains an
opaque archive, the controller is intentionally fail-closed: this batch does
not claim a runnable service.

`GET /personal/health` now exposes only the strict liveness payload required
by the bounded loopback probe. It is not `/personal/status`, readiness,
doctor, Provider/Pi/SecretStore proof, or product readiness.

## Failure-first and local verification

The first new focused target was intentionally compiled before the service API
existed. Windows GNU stopped in the known linker exit-121 baseline before Rust
test compilation; the WSL target then reached missing service symbols and was
not committed. Final WSL results (`windows_wsl2_linux_guest`) were:

```text
CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t08-service-health \
  cargo test -p cognitive-runtime --test linux_bundle_service_lifecycle --locked --offline
# 6 passed; 0 failed

cargo test -p kernel-server --test p1_t04_personal_daemon --locked --offline
# 4 passed; 0 failed

cargo test -p cognitive-runtime --test linux_bundle_attestation \
  --test linux_bundle_installation \
  --test linux_bundle_installation_lifecycle --locked --offline
# 14 passed; 10 passed; 12 passed, 1 ignored child entrypoint

cargo test -p cognitive-runtime --test linux_installer_bootstrap --locked --offline
# 6 passed; 0 failed

cargo clippy -p cognitive-runtime --all-targets \
  --features test-fault-injection --locked --offline -- -D warnings
# passed

cargo fmt --all -- --check
pnpm run check:consistency
# passed
```

All local Linux commands above ran in a Windows WSL2 guest. They are not
Linux-native systemd evidence. PR #112 merged after both push and pull-request
CI workflows passed their supported Ubuntu and Windows/MSVC matrices:
`30359906515` and `30359909728`. That supported-matrix build/test evidence
does not create Linux-native systemd, B01, Gate, Profile, containment, or
release evidence. The follow-up merge-evidence commit `8b51018` also passed
the post-merge Ubuntu and Windows/MSVC matrix in
[run 30360532366](https://github.com/agentkernel/cognitive-os/actions/runs/30360532366).

## Remaining gaps and next safe atomic action

No production key, release archive, release rendering, safe archive extraction,
runnable `kernel-server` layout, installed unit, real systemd campaign,
uninstall, SBOM/provenance, GitHub Release, B01, Gate, Profile, containment,
RC, or release claim is provided.

The next safe P1-T08 action is an independently reviewed safe archive
extraction and fixed runnable layout slice, with archive traversal/link/device
rejection and bounded expanded bytes/counts. Only after that boundary exists
can the currently fail-closed controller gain a real candidate unit/action and
Linux-native systemd evidence campaign.
