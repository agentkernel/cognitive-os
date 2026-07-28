# Personal P1-T08 Safe Extraction Handoff

**Date:** 2026-07-28
**Base:** `main@74dac13df0bc84af1336ed550673e7e2e7077d56`
**Branch:** `lane/personal-p1-t08-safe-extraction`
**Implementation commit:** `1b5462f`
**Pull request:** [#113](https://github.com/agentkernel/cognitive-os/pull/113)
**Merge commit:** `d57efc179de1d78b59b9e08e2c72b2642db65709`

## Delivered boundary

ADR-0031 specifies an implementation-local, in-process `tar.gz` extraction
boundary for P1-T08. The existing offline verifier remains the complete trust
boundary and completes before deployment-root or lease mutation. Once the
existing per-root OS lease is held, staging re-hashes the verified artifact and
extracts it into a private staging directory. Only after bounded extraction and
fixed layout validation succeeds is the candidate atomically renamed to
`staged/<version>`.

The sole accepted release layout is a direct archive-root
`bin/kernel-server` regular executable. The implementation rejects unsafe
paths, non-UTF-8 names, links, special entries, unsafe permissions, duplicate
paths, layouts outside the fixed allowlist, missing/non-executable entries, and
bounded resource violations. On Unix it explicitly installs the entry with
`0755`; it does not trust ownership metadata or the host umask. Extraction
failure removes private partial staging when possible, never changes the active
pointer, never invokes the service controller, and never returns a receipt.

The successful fixture layout satisfies only the current controller's static
layout prerequisite. The checked-in systemd unit remains unresolved, and the
controller still rejects every systemd action. No controller input was widened
to receive artifact bytes, manifest, keyring, secret, user data, unit content,
or arbitrary command.

## Failure-first tests and local verification

The first changed installation assertion required an extracted
`bin/kernel-server` and would have failed against the former opaque-artifact
staging behavior. Focused negatives subsequently cover a verified archive with
the wrong layout, `..` traversal, symbolic link, and a non-executable entry;
each confirms the previous active pointer remains unchanged and no public staged
candidate exists.

Executed locally in `windows_wsl2_linux_guest`:

```text
CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t08-extraction \
  cargo test -p cognitive-runtime --test linux_bundle_installation \
  --test linux_bundle_installation_lifecycle \
  --test linux_bundle_service_lifecycle --locked --offline
# 12 passed; 12 passed, 1 ignored child entrypoint; 6 passed

CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t08-extraction \
  cargo test -p cognitive-runtime --test linux_bundle_installation_lifecycle \
  --features test-fault-injection --locked --offline
# 14 passed, 1 ignored child entrypoint

CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t08-extraction \
  cargo clippy -p cognitive-runtime --all-targets \
  --features test-fault-injection --locked --offline -- -D warnings
# passed

cargo fmt --all -- --check
pnpm run check:consistency
# passed
```

The local Windows GNU focused test attempt stopped at the known non-supported
GNU linker exit-121 baseline. The first WSL `--offline` attempt could not yet
resolve the new dependency cache; after a dependency fetch, every recorded
offline test/check above ran successfully. These local results are not
Linux-native systemd, B01, Gate, Profile, containment, or release evidence.

## Remaining gaps and next safe atomic action

No production archive, signing key, release rendering, rendered unit, systemd
campaign, uninstall/upgrade campaign, Linux-native evidence, B01, Gate,
Profile, containment, RC, or release claim exists. PR #113 merged after both
push and pull-request CI matrices passed on supported Ubuntu and Windows/MSVC
runners. This supported build/test evidence does not become Linux-native
systemd, B01, Gate, Profile, containment, or release evidence.

Before any future systemd campaign, independently review this extracted
runnable-layout boundary, then decide whether a separately controlled
Linux-native systemd evidence campaign is warranted. Do not render a production
unit or add a candidate systemd action merely because static layout validation
now succeeds.
