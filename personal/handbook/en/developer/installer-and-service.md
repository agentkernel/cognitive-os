---
doc_id: dev.installer-service
locale: en
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: personal/crates/cognitive-runtime/src/linux_bundle.rs
    symbols: ["verify_linux_bundle", "stage_verified_bundle", "activate_after_health_check"]
  - path: personal/crates/cognitive-runtime/src/linux_bundle_installation.rs
    symbols: ["PreparedLinuxBundleInstallation", "install_linux_bundle"]
  - path: personal/crates/cognitive-runtime/src/linux_bundle_service.rs
    symbols: ["install_linux_bundle_single_service", "render_personal_user_service_unit"]
  - path: personal/deploy/linux/install.sh
  - path: personal/crates/cognitive-runtime/src/bin/linux_bundle_campaign_builder.rs
tests:
  - personal/crates/cognitive-runtime/tests/linux_bundle_single_service.rs
  - personal/crates/cognitive-runtime/tests/linux_installer_bootstrap.rs
  - personal/crates/cognitive-runtime/tests/linux_bundle_installation.rs
fingerprint: "sha256:34782e8252bec39c742631966d1e809a9a2d554b0f3a0e3d536be7fd43f47857"
non_claims:
  - The campaign builder uses an experimental signing key; no production signing ceremony, GitHub Release, or B01 claim is made here.
---

# Installer and service

## Offline bundle verifier

`verify_linux_bundle` checks an Ed25519-signed attestation over canonical bundle
metadata: product/platform identity, version-consistent root directory, exact
SHA-256 + size per entry, path safety (no absolute/`..`/symlink escapes), and
rejection of vendored Node/Pi payloads (`node_modules`, `pi-runtime/`, …).
`stage_verified_bundle` re-verifies as it extracts into `deployments/<version>/` with
0700/0755/0644 modes; `activate_after_health_check` health-gates then atomically flips the
`active-version` text pointer.

## Leased installer transaction

`PreparedLinuxBundleInstallation::prepare` → `install_linux_bundle`: a
cross-process OS file lock (`installer.lock`) serializes installers; steps are
verify → stage → (optional systemd unit render/enable) → health probe →
activate → immutable receipt. Failures compensate in reverse (restore previous
unit and pointer, remove staged version). Receipts and failure reports are typed;
`--dry-run` performs verification only.

## Single-service production shape

`install_linux_bundle_single_service` renders `cognitiveos-personal.service`
(user systemd): `ExecStart=<versioned kernel-server> --personal --bind
127.0.0.1:48181`, `NoNewPrivileges=true`, `Restart=on-failure`, plus
`cognitive-daemon@.service` template compatibility. Health confirmation demands
both `GET /personal/health` liveness **and** MainPID identity under the expected
deployment root — a rogue same-port process fails activation.

## Bootstrap chain

The rendered `install.sh` (template under `personal/deploy/linux/`, filled by the campaign
builder with pinned URLs + digests) downloads over HTTPS with bounded sizes and a
single pinned redirect host, SHA-256-verifies the installer binary, then hands off
to the Rust installer — no `curl | sh`, no sudo, no embedded secrets. The builder
(`linux_bundle_campaign_builder`) assembles daemon+CLI+installer bundles and signs with an
**experimental** key; release-manifest verification (`release_manifest.rs`)
covers manifest identity/digests/toolchain pins as a separate P7-T01 gate.
