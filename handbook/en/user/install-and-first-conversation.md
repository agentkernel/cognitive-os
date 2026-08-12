---
doc_id: user.install-first-conversation
locale: en
kind: guide
audience: [user]
status: partial
generated: false
sources:
  - path: deploy/linux/install.sh
  - path: crates/cognitive-runtime/src/linux_bundle_service.rs
    symbols: ["install_linux_bundle_single_service", "cognitiveos-personal.service"]
  - path: crates/cognitive-runtime/src/bin/linux_bundle_installer.rs
  - path: apps/admin-cli/src/personal_cli/init.rs
    symbols: ["run_init"]
tests:
  - crates/cognitive-runtime/tests/linux_bundle_single_service.rs
  - crates/cognitive-runtime/tests/linux_installer_bootstrap.rs
  - apps/admin-cli/tests/p1_t06_cognitive_cli.rs
fingerprint: "sha256:f543ba606e59b310562b3f241c4b8b140dfb8ff82d9da65d257964ec6da8f5fe"
non_claims:
  - No public GitHub Release or production signing ceremony exists yet; installable artifacts so far are experimentally signed campaign builds. Install-route correctness evidence (B01) is owned by the formal plan and not restated here.
---

# Install and reach the first conversation

`partial`: the whole route below is implemented and exercised end-to-end on clean
Linux machines, but there is **no public production release artifact yet** — bundles
so far come from the experimental campaign builder with a non-production signing
key. Platform: Linux x86_64 with user systemd; desktop needs a Secret Service
keyring (GNOME Keyring).

## 1. Run the inspected bootstrap installer

A release-shaped bundle ships a rendered `install.sh` (from the inspected template
[`deploy/linux/install.sh`](../../../deploy/linux/install.sh)). It is deliberately
boring: fail-closed shell settings, HTTPS-only bounded downloads, one pinned
redirect host, SHA-256 verification of the installer binary before executing it, and
no `curl | sh`, no `sudo`, no embedded keys.

The Rust installer then verifies the Ed25519-signed bundle attestation (product,
platform, version, Pi pin, safe archive layout), stages immutable bytes under your
XDG data directory, installs the single user service
`cognitiveos-personal.service` (loopback `127.0.0.1:48181`,
`NoNewPrivileges=true`), confirms health plus process identity, and only then
switches the atomic `active-version` pointer. Any failure compensates: the previous
version, unit, and pointer are restored, and no success receipt is issued.

## 2. Initialize configuration and secrets

```text
cognitive init --provider <id> --base-url <https-url> --api-key-file -
```

`cognitive init` prepares the databases (with pre-migration backups), stores your
Provider key into the OS secret store (Linux Secret Service here; on Windows
hosts the same command selects the Credential Manager backend) via stdin/hidden
input — never argv or files — probes the Provider, and persists two non-secret
files: `provider.json` (with an opaque `SecretRef`) and `selected-model.json`. If
no production secret backend is usable the command fails closed — there is no
plaintext fallback.

## 3. Start and check the daemon

```text
cognitive daemon start          # binds 127.0.0.1:48181 by default
cognitive status                # component projection
cognitive doctor                # redacted diagnostics
```

`first_conversation_ready` in the status output additionally requires Pi
configuration; overall readiness does not.

## 4. Configure and launch Pi

```text
cognitive pi configure --executable <abs-path-to-pi> --extension-entry <abs-path-to-dist/index.js>
cognitive pi launch
```

Launch is fail-closed: it requires all doctor components ready and the exact pinned
Pi version, passes only `--extension`, and never hands Pi a Provider key. Your first
message flows Pi → daemon Provider proxy → Provider; see [The Pi shell](./pi-shell.md).

## Failure exits worth knowing

Bad signature/pin → nothing is installed; health failure → previous service
restored; locked or missing keyring → `init` refuses; stale `daemon.lock` →
`cognitive daemon stop` cleans it only after proving the process is gone.
