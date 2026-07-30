# Personal P1-T08 MVP Single-Service Installer Handoff

**Date:** 2026-07-29
**Base:** `main@4a807e6`
**Branch:** `lane/personal-p1-t08-mvp-single-service`
**Development track:** `experimental-local-only`
**Task status:** P1-T08 remains `in-progress`

## Delivered boundary

ADR-0034 and the Personal roadmap now make one canonical user service the
first production path:

```text
cognitiveos-personal.service
127.0.0.1:48181
```

The candidate unit and port 48182 remain valid historical fixture evidence and
an optional future upgrade optimization. They no longer block P1-T08, P1-T09,
or the public Linux MVP convergence task `P7-T08 / GMVP-LINUX`.

The inspected `deploy/linux/install.sh` still owns only bounded HTTPS download,
private temporary paths, cleanup, and a release-rendered executable digest. It
now downloads and invokes `cognitiveos-linux-bundle-installer` rather than
stopping after the offline verifier. The Rust adapter constructs the explicitly
rendered public keyring and Pi compatibility pin, derives the product XDG data
root, and delegates installation to `cognitive-runtime`.

The generic and service-aware installers now share one transaction prefix:

```text
offline verification
-> private deployment-parent creation
-> OS-backed per-root lifecycle lease
-> deployment open
-> validated previous-active read
-> safe private extraction and staging
```

The single-service suffix is fixed:

```text
immutable version publication
-> canonical unit publication
-> systemctl --user daemon-reload
-> canonical service restart
-> bounded exact /personal/health confirmation on 48181
-> atomic active-pointer publication
-> pointer and service reconfirmation
-> non-secret receipt
```

The production controller derives the user unit directory from absolute
`XDG_CONFIG_HOME` or `HOME`, invokes `/usr/bin/systemctl` with fixed arguments,
and accepts no unit name, command, health URL, executable, deployment root, or
runtime argument from bundle metadata. The active unit uses the daemon's real
user XDG roots; the deferred candidate fixture alone retains its private
runtime-root argument.

## Compensation semantics

Upgrade failure after any possible canonical-unit mutation performs:

```text
stop canonical service
-> restore previous active pointer
-> republish previous canonical unit
-> restart previous service
-> confirm previous health
```

First-install failure performs:

```text
stop canonical service when it may have changed
-> clear any active pointer
-> remove the canonical unit
-> systemctl --user daemon-reload
```

User data and immutable version bytes are retained for inspection. A receipt is
issued only after pointer and service reconfirmation. Any incomplete
compensation returns `LinuxBundleServiceError::RollbackIncomplete`.

## Failure-first and fixture coverage

The new `linux_bundle_single_service` integration test first referenced the
absent narrow controller and transaction API. It now covers:

- successful upgrade confirmation before and after pointer publication;
- pre-pointer health failure and previous release restoration;
- post-pointer confirmation failure and pointer restoration;
- failed first install with no active pointer or canonical unit;
- incomplete rollback returning `RollbackIncomplete` without a receipt.

Existing installation/lifecycle/bootstrap tests were retained and updated for
the first-install XDG contract. The real controller fixture confirms that Rust
publishes only `cognitiveos-personal.service`, reloads user systemd, and invokes
a fixed canonical restart without publishing the candidate unit.

## Executed verification

Executed in `windows_wsl2_linux_guest`:

```text
CARGO_TARGET_DIR=/mnt/d/agent-kernel/target/wsl-single-service \
  cargo test -p cognitive-runtime \
  --test linux_bundle_single_service \
  --test linux_bundle_service_lifecycle \
  --test linux_bundle_installation \
  --test linux_bundle_installation_lifecycle \
  --test linux_installer_bootstrap \
  --test linux_bundle_installer_adapter
# 50 passed; 0 failed; 1 ignored child-process entrypoint

CARGO_TARGET_DIR=/mnt/d/agent-kernel/target/wsl-single-service \
  cargo clippy -p cognitive-runtime --all-targets -- -D warnings
# passed

cargo fmt --all -- --check
# passed

pnpm run check:consistency
# OK: 273 requirements, 55 error codes, 63 schemas, 85 vectors

git diff --check
# passed
```

Follow-up adapter hardening extracted fixed bootstrap-fact parsing,
release-version verification, and transaction orchestration into the runtime
library. The production executable remains a redacted process wrapper that
derives the product deployment root and constructs only the fixed production
user-systemd controller. The focused adapter test executes a fully signed,
positive transaction through an isolated controller boundary; it proves receipt,
immutable publication, pointer activation, and fixed controller action order
without adding a production manager/path/environment override.

The local Windows GNU toolchain still fails in dependency build-script linking
with the known unsupported MinGW linker exit. It did not reach these tests and
is not represented as a product failure or successful Windows evidence.

## Explicit non-claims and remaining work

This batch provides implementation and local fixture evidence only. It does
not provide or imply:

- a rendered production release or production signing material;
- Linux-native user-systemd execution;
- clean-VM install-to-first-conversation evidence;
- uninstall, backup, restore, or update-channel evidence;
- B01, `GMVP-LINUX`, any other product Gate, or Profile conformance;
- containment, Task, Memory, Multi-Agent, UI, or Windows installer parity.

P1-T08 remains `in-progress`. Its next safe batch is native Linux user-systemd
validation of the rendered release layout and deterministic failure cases,
followed by remaining uninstall/release integration. P1-T09 remains
`not-started`; B01 must not be inferred from WSL, fake-systemctl, or CI tests.
