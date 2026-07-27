# Personal P1-T08 Linux Bundle Foundation Handoff

**Date:** 2026-07-27
**Branch:** `lane/personal-p1-t08-linux-installer`
**Status:** P1-T08 remains **in-progress** (`experimental-local-only`).

## Delivered first slice

`cognitive-runtime::linux_bundle` adds an offline, non-downloading release
bundle boundary and filesystem activation model:

- strict schema-v1 Linux x86_64 manifest parsing;
- artifact SHA-256 comparison;
- required HTTPS attestation-reference shape (not cryptographic attestation
  verification);
- caller-controlled expected Pi version/integrity comparison;
- rejection of vendored Node and Pi payload names;
- version-specific staging that leaves the active pointer untouched;
- health-gated atomic active-version pointer replacement; and
- prior-version/user-data retention on interruption or health failure.

The module has no downloader, process spawn, `systemctl` invocation, daemon
listener, SQLite mutation, secret handling, Effect, Task transition, or
capability-grant path.

## Verification

Passed locally on `windows_wsl2_linux_guest`:

```text
cargo fmt --all -- --check
CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t08-bundle /root/.cargo/bin/cargo test -p cognitive-runtime linux_bundle --locked
CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t08-bundle /root/.cargo/bin/cargo clippy -p cognitive-runtime --all-targets --locked -- -D warnings
pnpm run check:consistency
git diff --check
```

The focused test command ran four substantive `linux_bundle` tests. Windows
GNU Rust linking remains a non-supported local baseline (exit 121); it was not
used as test evidence. Supported CI remains required for merge.

## Explicit non-claims and remaining work

This slice is not a trusted attestation verifier: it validates an attestation
reference's required form only. No real release artifact, downloader, inspected
`deploy/linux/install.sh`, systemd user unit, service-health protocol,
cross-process installation lease, uninstall path, or Linux-native B01 campaign
exists yet. Therefore it is not a P1-T08 completion, Gate, Profile,
containment, Linux-native, or release claim.

Next implementation must first choose and document the concrete trusted
attestation-verification mechanism. It must then integrate the staged model
into an inspected `curl -o install.sh; less; sh` flow without vendoring Pi or
Node, preserve user data, and add the service/interruption/rollback tests the
formal P1-T08 card requires.

## Worktree boundary

Pre-existing user-owned entries remain unstaged and must not be reverted or
committed:

- `docs/plan/AUTOPILOT-PROMPT.md`;
- `.cursor/`;
- `.vscode/`.

Do not include `personal-blog/` in this repository.
