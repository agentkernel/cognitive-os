# Personal P1-T08 Linux Bootstrap Handoff

**Date:** 2026-07-28  
**Base:** `main@3e205450b5ecbee74a1dbaf9f48231180d6d3228`  
**Branch:** `lane/personal-p1-t08-linux-bootstrap`  
**Status:** P1-T08 remains **in-progress**  
**Development track:** `experimental-local-only`

## Delivered boundary

`deploy/linux/install.sh` is an inspectable POSIX release template. Unrendered
source placeholders fail before curl. A rendered release script owns the fixed
version, HTTPS object directory, one allowed redirect host, verifier SHA-256,
public keyring, and Pi pin. It downloads through a private `mktemp -d`
directory using partial paths and cleanup traps, validates the verifier digest
before executing it, then passes the local bundle directory to the narrow
`linux-bundle-verifier` adapter.

The adapter has no network, systemd, secret, authority, deployment, staging,
or activation operation. It only constructs script-bound `TrustedKeyring` and
`ExpectedPiCompatibility` values and delegates to `verify_linux_bundle`.
There is no unconditional health callback and no invocation of
`install_linux_bundle` in this slice.

## Failure-first evidence

Before implementation, the new focused test attempted real `sh` execution and
failed because `deploy/linux/install.sh` did not exist (exit 101). This red
state was not committed. The implemented focused target then passed **6/6**:

```text
windows_wsl2_linux_guest
CARGO_TARGET_DIR=/tmp/cognitiveos-p1-t08-linux-bootstrap \
  cargo test -p cognitive-runtime --test linux_installer_bootstrap --locked
# 6 passed; 0 failed
```

Coverage executes a rendered production path with signed test-only fixture
bytes, fake bounded curl transport, and the real Rust adapter. It proves:

- unrendered templates reject before network access;
- private partial downloads become final files only after success;
- the adapter runs exactly once on success;
- artifact download failure and adapter digest mismatch never execute it;
- verifier rejection leaves external active/user data untouched;
- temporary bootstrap downloads are cleaned after success and failure; and
- the template forbids curl-pipe-shell, eval, sudo, systemctl, activation,
  test signing material, and Pi/Node installation.

Existing verifier and lifecycle suites retain the complete attestation,
keyring/revocation, Pi pin, unsafe file, lease, staging, interruption, and
activation semantics. This bootstrap does not duplicate them in shell.

## Trust, redirect, and temporary-directory decisions

ADR-0029 records the release-rendered policy, bounded HTTPS curl calls,
one-hop explicit redirect restriction, script-bound bootstrap executable
digest, and private cleanup boundary. ADR-0028 remains the cryptographic
bundle truth; its provenance URL is not permission to fetch a bundle URL.

## Evidence separation and remaining work

The focused result above is `windows_wsl2_linux_guest`, not Linux-native
evidence. Supported Ubuntu and Windows/MSVC CI are pending PR execution.
No Linux-native campaign was run. ShellCheck is `not-run` unless present on
the host; shell syntax is checked with `sh -n`.

This batch does not provide a production key/trust root, real release or
GitHub Release, SBOM/provenance, systemd user unit, service health/rollback,
uninstall/data retention, Linux-native campaign, B01, Gate, Profile,
containment, RC, or release claim. The next P1-T08 entry is systemd user unit
plus bounded service-health, health-gated activation, and service rollback.
