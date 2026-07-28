# Personal P1-T08 Installer Lifecycle Lease Handoff

**Date:** 2026-07-28
**Branch:** `lane/personal-p1-t08-installer-lease`
**Base:** `main@afef1e73d48a11873b5ee165686e579a00f3ddac`
**Implementation commit:** pending local commit
**Pull request:** pending
**Merge commit:** pending
**Status:** P1-T08 remains **in-progress**
**Development track:** `experimental-local-only`

## 1. Delivered boundary

The official `cognitive_runtime::install_linux_bundle` entry point now fixes
this failure-closed order:

1. execute the complete `verify_linux_bundle` verifier;
2. only after verification succeeds, derive and acquire the per-deployment
   lifecycle lease;
3. only after lease acquisition, open or create the deployment root;
4. read the previous active version;
5. stage only the verifier-returned `VerifiedLinuxBundle`;
6. invoke the caller health callback exactly once;
7. atomically activate only after health succeeds;
8. re-read and confirm the active pointer equals the verified target version;
9. only then return the non-secret installation receipt.

The lease uses the Rust standard library OS-backed file-lock API. Its stable
lock path is product-owned and consists of a fixed prefix plus a SHA-256
digest of the canonical deployment root. Bundle and manifest data cannot
select the path. Ownership depends only on the live file descriptor and the
OS lock; file existence and contents have no ownership authority. There is no
process-local mutex, TTL, PID/token metadata, stale-owner takeover, or lock
file unlink on release. Keeping the path stable avoids the inode/path split
race that could occur if one process unlinked a file already opened by a
successor.

The deployment root itself may be absent when installation starts, but its
immediate parent must exist to host the stable lock outside the root. This
requirement is documented and tested. A missing parent fails after complete
verification without creating the parent, deployment root, or lock file.

`LinuxBundleError::InstallationLeaseHeld` has fixed text and carries no path,
PID, owner token, or lock state. Deterministic fault injection is available
only under the `test-fault-injection` Cargo feature. The existing public
health-check helper remains intact; the activation-only helper needed by the
orchestrator is restricted to `pub(crate)`.

## 2. Concurrency and interruption evidence

The lifecycle integration test uses signed local bundles, the complete
verifier, real filesystems, and real child processes. It covers:

- same deployment root: exactly one process holds the lease;
- same root with different target versions: still mutually exclusive;
- different deployment roots: independent progress;
- release after successful installation;
- verifier failure: no lock file, deployment root, or health call;
- missing lease parent: no parent/root/lock mutation;
- real staging I/O failure: release and successful successor;
- health failure: release while preserving inspectable staging and old state;
- activation failure: release, no receipt, and no torn active pointer;
- real child-process termination: OS release and successful successor;
- persistent empty lock file and injected stale contents: no live ownership;
- deterministic error faults after lease, deployment open, staging, health,
  and activation;
- panic unwind immediately after lease acquisition;
- activation completed but receipt not issued: complete new pointer, error
  return, and successful retry;
- error `Debug` and `Display` redaction for key material, artifact/user data,
  deployment path, lock path, and PID.

Health callback counts are asserted at every deterministic boundary. The
activation-completed-before-confirmation fault proves that no receipt is
returned even though the pointer already contains the complete target value.

## 3. Local tests executed

All Rust commands ran inside WSL2 and are classified only as
`windows_wsl2_linux_guest` evidence:

```text
CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t08-installer-lease \
  cargo test -p cognitive-runtime \
  --test linux_bundle_installation_lifecycle \
  --features test-fault-injection --locked
# 14 passed; 0 failed; 1 ignored child entrypoint

CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t08-installer-lease \
  cargo test -p cognitive-runtime --locked
# runtime unit: 49 passed
# linux_bundle_attestation: 14 passed
# linux_bundle_installation: 10 passed
# linux_bundle_installation_lifecycle: 11 passed; 1 ignored
# m5_event_envelope: 2 passed
# pi_linux_launcher: 5 passed
# 91 substantive tests passed; 0 failed; 1 ignored

CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t08-installer-lease \
  cargo clippy -p cognitive-runtime --all-targets \
  --features test-fault-injection --locked -- -D warnings
# passed

cargo fmt --all -- --check
# passed

pnpm run check:consistency
# passed: 273 requirements, 55 error codes, 63 schemas, 85 vectors

git diff --check
# passed
```

The isolated worktree initially had no `node_modules`, so the first
consistency invocation could not import `ajv` and did not execute the checker.
`pnpm install --frozen-lockfile` installed only lockfile-pinned dependencies;
the exact consistency command then passed. No package manifest or lockfile was
changed.

After the complete non-feature runtime command, one additional missing-parent
boundary test was added and passed as part of the 14-test focused feature
command. All production code and the complete lifecycle test target were then
checked by the final strict feature Clippy command.

## 4. Evidence separation and pending CI

- **WSL2 local (`windows_wsl2_linux_guest`):** focused lifecycle tests,
  complete runtime tests, strict feature Clippy, formatting, consistency, and
  diff checks passed as listed above.
- **Supported Ubuntu CI:** pending pull request.
- **Supported Windows/MSVC CI:** pending pull request; this is the required
  proof for Windows file-open sharing, `LockFileEx` contention/release,
  process termination, stale-file behavior, and active-pointer replacement.
- **Linux-native host/campaign:** not run. WSL2 evidence is not Linux-native
  evidence.

## 5. Unfinished P1-T08 scope

P1-T08 remains `in-progress`. This batch does not provide:

- an inspectable `deploy/linux/install.sh`, downloader, temporary download
  protocol, or network access;
- a systemd user unit, daemon spawning, bounded service-health protocol, or
  service rollback;
- uninstall or user-data retention workflow;
- a production signing key, production trust root, release attestation,
  signing ceremony, SBOM, GitHub Release, or release evidence;
- a Linux-native fixture campaign or P1-T09 B01 clean-run.

No authority object, capability, Intent, Effect, Task transition, registry,
schema, transition table, or conformance vector was added or changed. The
separate agent package/source admission authority in `installer.rs` remains
untouched.

## 6. Recommended next entry

First complete this slice's supported Ubuntu and Windows/MSVC PR checks and
record the implementation, PR, merge, and CI identifiers here. Then continue
P1-T08 in separate batches, in order:

1. inspectable Linux shell bootstrap/download/temporary-directory flow;
2. systemd user unit, bounded service-health protocol, and rollback;
3. uninstall and user-data retention;
4. Linux-native fixture campaign;
5. only then P1-T09 B01 clean-run.

## 7. Non-claims

This checkpoint is an implementation and local-test slice only. It is not a
P1-T08 completion, Linux-native installer, B01, Gate, Profile, containment,
system-service, RC, release-readiness, or release claim.
