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
  - path: personal/apps/admin-cli/src/personal_cli/pi.rs
    symbols: ["PINNED_PI_VERSION"]
  - path: personal/docs/product/linux-1.0-scope.md
  - path: docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md
  - path: docs/plan/PERSONAL-TEST-ENVIRONMENTS.md
fingerprint: "sha256:3ea9af30f8740b1c5a77a4af897133bb2fb02e442d30592f954955b873a2ca41"
non_claims:
  - Compilation on a platform is not product support; only the listed product target carries installation and service paths.
---

# Compatibility

## Current product target

Linux x86_64 with user systemd (lingering enabled for boot start). Desktop
sessions need a Secret Service keyring (GNOME Keyring). Headless operation is
designed (encrypted vault) but not selectable yet. WSL2 and Windows-native hosts
are explicitly refused by the Pi launch admission path.

Personal 2.0 is Windows-first and remains `Requires-backend` plus
`Requires-environment`. The qualified native Windows OPC development
environment and B01-W are not provisioned. The target runtime is preinstalled
managed DSH; Pi is the hidden Personal Assistant engine. Existing Linux Pi/dsh,
WSL, ordinary CI, Canvas, CLI, Provider, model, account, adapter, or Windows GNU
evidence transfers no Windows support or DSH qualification. Hermes, Codex,
Cursor and others remain future candidates. Native mobile/E2E relay remote is
Personal 2.1.

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
