# Personal P1-T08 Offline Attestation Verifier Handoff

**Date:** 2026-07-28
**Branch:** `lane/personal-p1-t08-attestation-verifier`
**Base:** `origin/main@434d411`
**Status:** P1-T08 remains **in-progress** (`experimental-local-only`).

## Delivered failure-first slice

This batch turns the earlier structural `attestation_reference` check into a
strictly local trust boundary documented by ADR-0028:

- detached **Ed25519** signature verification via `ed25519-dalek`;
- a closed, RFC 8785 JCS canonical attestation statement whose exact bytes are
  signed;
- exact manifest/statement binding for product, platform, version, artifact
  filename/digest, Pi version/integrity, and provenance reference;
- explicitly supplied, product-owned versioned keyrings with canonical
  URL-safe Base64 public keys, bounded key IDs, multiple active keys, and
  fail-closed unknown/revoked keys;
- closed signature envelopes: schema/version/algorithm/key ID/signature only;
- strict HTTPS provenance URLs: absolute host required, no user information or
  control characters;
- safe, distinct local artifact/statement/signature paths that cannot replace
  `manifest.json`, bounded metadata reads, and regular-file-only bundle inputs
  (no symlinks, directories, or special files); and
- streamed artifact hashing and staged copying from one already opened handle.
  Staging re-hashes immediately before its destination is created, so an
  artifact modified after verification cannot create a staged candidate.

`VerifiedLinuxBundle` has private construction and is the only accepted input
to the staging API. Verification has no downloader, network, subprocess,
environment, configuration, secret, capability, Effect, Task, daemon, or
active-pointer mutation path.

## Test evidence

Executed locally in the WSL guest (`windows_wsl2_linux_guest`), not as
Linux-native, Gate, Profile, containment, or release evidence:

```text
CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t08-attestation-failure \
  /root/.cargo/bin/cargo test -p cognitive-runtime \
  --test linux_bundle_attestation --locked
# 14 passed; 0 failed

CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t08-attestation-failure \
  /root/.cargo/bin/cargo test -p cognitive-runtime --locked
# cognitive-runtime unit tests: 49 passed; 0 failed
# linux_bundle_attestation: 14 passed; 0 failed
# m5_event_envelope: 2 passed; 0 failed
# pi_linux_launcher: 5 passed; 0 failed
# doc tests: 0 passed; 0 failed

CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t08-attestation-failure \
  /root/.cargo/bin/cargo clippy -p cognitive-runtime --all-targets \
  --locked -- -D warnings
# passed

cargo fmt --all -- --check
# passed

pnpm run check:consistency
# OK: 273 requirements, 55 error codes, 63 schemas, 85 vectors

git diff --check
# passed
```

The 14 integration tests cover valid verification; wrong, unknown, revoked,
and bundle-selected keys; artifact/statement/signature tampering; unknown and
duplicate statement fields; non-canonical JCS JSON/Base64; every signed field
binding; malformed/unsafe/colliding URLs and file paths; post-verification
artifact mutation; Unix symlinks; information disclosure; and staged
activation/interruption/health-failure data retention.

`pnpm run check:consistency` initially could not run because this isolated
worktree lacked `node_modules` and therefore `ajv`; `pnpm install
--frozen-lockfile` restored only lockfile-pinned dependencies, after which the
check passed. No package manifest or lockfile change resulted from that
installation.

## Documentation and scope

- Added ADR-0028, `docs/plan/PROGRESS.md`, and the P1-T08 task row in
  `PERSONAL-DEVELOPMENT-PLAN.md`.
- This is an implementation and local-test checkpoint only. The repository
  contains no production private key, public trust root, release bundle,
  generated attestation, signing ceremony, SBOM, GitHub Release, downloader,
  inspected installer, systemd user unit, uninstall workflow, cross-process
  installer lease, Linux-native campaign, B01, Gate, Profile, containment, or
  release claim.
- Production key approval/ceremony and real release evidence remain P7-T01.

## Current worktree state

Changes are intentionally **uncommitted** in this handoff:

```text
M  Cargo.lock
M  Cargo.toml
M  crates/cognitive-runtime/Cargo.toml
M  crates/cognitive-runtime/src/lib.rs
M  crates/cognitive-runtime/src/linux_bundle.rs
M  docs/plan/PERSONAL-DEVELOPMENT-PLAN.md
M  docs/plan/PROGRESS.md
?? crates/cognitive-runtime/tests/linux_bundle_attestation.rs
?? docs/adr/0028-personal-offline-linux-bundle-attestation.md
?? docs/checkpoints/20260728-personal-p1-t08-offline-attestation-verifier-handoff.md
```

Do not add production key material, signing keys, release artifacts, or
`personal-blog/`. The next P1-T08 slice must integrate this verifier into an
inspected installer/service workflow and separately design remaining
cross-process lease, interruption, rollback, uninstall, and Linux-native
campaign evidence.
