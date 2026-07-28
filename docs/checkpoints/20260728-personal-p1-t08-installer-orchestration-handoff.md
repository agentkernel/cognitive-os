# Personal P1-T08 Installer Orchestration Handoff

**Date:** 2026-07-28
**Branch:** `lane/personal-p1-t08-installer-orchestration`
**Base:** `main@63aa8fb629596284b279b3fac56550baadb79724`
**Implementation commit:** pending at handoff authoring; record the exact
commit and merge commit in the post-merge closeout update
**Status:** P1-T08 remains **in-progress**
**Development track:** `experimental-local-only`

## 1. Failure-first evidence

The new integration test was written before the production module existed.
The first WSL invocation exited `101` with the expected unresolved import for
`cognitive_runtime::linux_bundle_installation`; it also exposed two
test-authoring `Cow<str>` pattern errors, which were corrected without adding
production code. The clean failure-first rerun then exited `101` solely with:

```text
error[E0432]: unresolved import `cognitive_runtime::linux_bundle_installation`
  --> crates/cognitive-runtime/tests/linux_bundle_installation.rs:11:24
   |
11 | use cognitive_runtime::linux_bundle_installation::install_linux_bundle;
   |                        ^^^^^^^^^^^^^^^^^^^^^^^^^ could not find
   |                        `linux_bundle_installation` in `cognitive_runtime`

error: could not compile `cognitive-runtime`
       (test "linux_bundle_installation") due to 1 previous error
```

The red state was not committed.

## 2. Delivered boundary

`cognitive-runtime::linux_bundle_installation::install_linux_bundle` is the
official local orchestration entry point for this batch. Its arguments are
limited to the bundle directory, deployment root, product-fixed
`ExpectedPiCompatibility`, product-owned `TrustedKeyring`, and one
caller-bounded health-check callback.

The implementation fixes this order:

1. call the complete merged `verify_linux_bundle` implementation;
2. only after verification succeeds, open or create the deployment root;
3. read and retain the previous active version;
4. stage through the verifier-returned, private-construction
   `VerifiedLinuxBundle` value, including the pre-staging streamed re-hash;
5. invoke the health check exactly once on the staged candidate;
6. activate atomically only when health succeeds;
7. re-read the active pointer and require it to equal the verified version;
8. only then return `LinuxBundleInstallationReceipt`.

The receipt contains only installed version, optional previous active version,
resulting active version, trusted key ID, and trusted keyring version. It does
not contain manifest, artifact, statement, signature, key bytes, health-check
output, or user data. Repeating an already active version is explicitly
defined and tested as idempotent: complete verification, staging, and health
checking still occur; the existing version directory remains and no partial
staging directory remains after success.

Health-check failure retains the old active pointer, old version directory,
user data, and the staged candidate. This preserves the inspection behavior
already fixed by ADR-0028 and `LinuxBundleDeployment`; this batch does not
silently introduce cleanup or rollback semantics.

## 3. Tests executed

All local Rust commands below ran inside WSL2 and are classified only as
`windows_wsl2_linux_guest` evidence:

```text
CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t08-installer-orchestration \
  /root/.cargo/bin/cargo test -p cognitive-runtime \
  --test linux_bundle_installation --locked
# 10 passed; 0 failed

CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t08-installer-orchestration \
  /root/.cargo/bin/cargo test -p cognitive-runtime --locked
# runtime unit tests: 49 passed; 0 failed
# linux_bundle_attestation: 14 passed; 0 failed
# linux_bundle_installation: 10 passed; 0 failed
# m5_event_envelope: 2 passed; 0 failed
# pi_linux_launcher: 5 passed; 0 failed
# doc tests: 0 passed; 0 failed
# total substantive tests: 80 passed; 0 failed

CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t08-installer-orchestration \
  /root/.cargo/bin/cargo clippy -p cognitive-runtime \
  --all-targets --locked -- -D warnings
# passed

/root/.cargo/bin/cargo fmt --all -- --check
# passed

pnpm run check:consistency
# passed (exit 0)

git diff --check
# passed
```

The isolated worktree initially lacked `node_modules`, so the first exact
consistency invocation failed because `ajv` was unavailable.
`pnpm install --frozen-lockfile` installed only lockfile-pinned dependencies
without changing a package manifest or lockfile; the exact consistency command
then passed. This setup failure is not represented as a product-test failure.

The 10 focused tests cover valid orchestration, invalid detached signature,
unknown/revoked/bundle-selected keys, artifact and statement tampering, wrong
Pi pin, unsupported platform, failed health retention, successful upgrade,
same-version idempotency, information-disclosure negatives, and the existing
post-verification artifact-mutation re-hash boundary.

## 4. Evidence separation

- **WSL2 local (`windows_wsl2_linux_guest`):** focused tests, complete runtime
  tests, strict runtime Clippy, and formatting passed as listed above.
- **Supported Ubuntu and Windows/MSVC CI for this batch:** not run at handoff
  authoring; must pass on the pull request before merge.
- **Linux-native host/campaign:** not run. No result in this handoff is
  Linux-native evidence.
- **Workspace-wide Rust test/Clippy:** not run locally for this batch; the
  required scoped runtime surface was run, and supported PR CI remains
  mandatory.

## 5. Unfinished P1-T08 scope

P1-T08 remains `in-progress`. This batch does not provide:

- a cross-process installer lifecycle lease or concurrency/interruption fault
  campaign;
- an inspectable `deploy/linux/install.sh`, downloader, temporary download
  protocol, or network access;
- a systemd user unit, service spawning, bounded service-health protocol, or
  service rollback;
- uninstall or user-data retention workflow;
- a production signing key, production public trust root, release bundle,
  release attestation, signing ceremony, SBOM, GitHub Release, or release
  evidence;
- a Linux-native fixture campaign or P1-T09 B01 clean-run.

No capability, Intent, Effect, Task transition, authority completion, schema,
registry, transition, or conformance vector was added or changed. The existing
agent package/source admission authority in `installer.rs` and
`admin-cli install` was not modified or conflated with Personal product bundle
deployment.

## 6. Recommended next batches, in order

Do not combine these into this batch. Continue in this order:

1. cross-process installer lifecycle lease plus concurrency and interruption
   fault injection;
2. inspectable `deploy/linux/install.sh` bootstrap, download, and temporary
   directory flow;
3. systemd user unit, bounded service-health protocol, and rollback;
4. uninstall and user-data retention;
5. Linux-native fixture campaign;
6. only then P1-T09 B01 clean-run.

## 7. Non-claims

This checkpoint is an implementation and local-test slice only. It is not a
Linux-native, B01, Gate, Profile, containment, system service, installer
release, RC, or release claim. It does not establish a production trust root,
real signed release artifact, downloader, unattended update path, or supported
end-user installer.
