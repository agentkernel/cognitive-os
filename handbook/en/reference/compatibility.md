---
doc_id: ref.compatibility
locale: en
kind: reference
audience: [user, developer]
status: implemented
generated: false
sources:
  - path: rust-toolchain.toml
  - path: package.json
  - path: apps/admin-cli/src/personal_cli/pi.rs
    symbols: ["PINNED_PI_VERSION"]
  - path: docs/product/personal/linux-1.0-scope.md
fingerprint: "sha256:21b56142d1ca9be353dae6e81efc6b26115f3c2280c421da2e35f933625112b9"
non_claims:
  - Compilation on a platform is not product support; only the listed product target carries installation and service paths.
---

# Compatibility

## Product target

Linux x86_64 with user systemd (lingering enabled for boot start). Desktop
sessions need a Secret Service keyring (GNOME Keyring). Headless operation is
designed (encrypted vault) but not selectable yet. WSL2 and Windows-native hosts
are explicitly refused by the Pi launch admission path.

## Pinned versions

| Component | Pin | Where |
|---|---|---|
| Rust toolchain | 1.97.1 | `rust-toolchain.toml` |
| pnpm | 10.33.2 | root `package.json` `packageManager` |
| Node | ≥ 22 | root `package.json` engines; Node 22 in CI |
| Pi agent | exactly `0.81.1` (`@mariozechner/pi`, pinned sha512 integrity) | acquisition + launch admission |
| SQLite mode | WAL, `synchronous=FULL`, foreign keys on | store open assertions |
| HTTP surface | local loopback only, port 48181 by convention | daemon config |

## What compiles vs what is supported

CI builds and tests the workspace on Ubuntu and Windows MSVC; that is engineering
evidence, not Windows product support. A Windows install **surface** now exists in
the tree — the Windows Credential Manager production secret backend plus
inspectable bootstrap-installer and per-user scheduled-task templates
(ADR-0052) — but the end-to-end Windows install campaign (B01-W) has not been
executed, so no install parity is claimed and local files still carry no
ACL hardening. The registered local Windows GNU host cannot link Rust at all.
macOS has no CI lane and no backend.

## Client compatibility

The Pi extension and TypeScript SDK speak the daemon's local HTTP surface with
AKP 0.2 envelope semantics and generated contract types; consumers must treat
unknown error codes and unknown response fields as protocol failures (fail
closed), matching the Rust side.
