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
  - path: apps/admin-cli/src/personal_cli/dsh.rs
    symbols: ["configure", "launch"]
tests:
  - crates/cognitive-runtime/tests/linux_bundle_single_service.rs
  - crates/cognitive-runtime/tests/linux_installer_bootstrap.rs
  - apps/admin-cli/tests/p1_t06_cognitive_cli.rs
  - apps/admin-cli/tests/p2_t32_public_daemon_start_scheduler.rs
fingerprint: "sha256:a309ff080061039cb1128ea1a74cd512e7ab6efa503617afb098937e66e4aacf"
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
a later disposable runtime needs the same already-stored item, pass
`--reuse-existing-secret-binding` with `--provider` and `--base-url` instead of
another key capture. If no production secret backend is usable the command fails
closed — there is no plaintext fallback.

Named accounts, key rotation, fixed agent bindings, and usage queries are a
separate operator surface after the daemon is running:
[Provider Control Plane](./provider-control-plane.md). That surface is CLI talking
to the daemon only; there is no Web or desktop control panel in this phase.
`cognitive init` remains the first-conversation path (`provider.json` /
`selected-model.json`) until you set a control-plane binding.

## 3. Start and check the daemon

```text
cognitive daemon start          # binds 127.0.0.1:48181 by default
cognitive status                # component projection
cognitive doctor                # redacted diagnostics
```

`first_conversation_ready` in the status output additionally requires Pi
configuration; overall readiness does not. That flag is conversation-shell
readiness, not C1/C2 Task progress: an admitted Task can stay `DRAFT` until the
scheduler acquires a lease. CLI `cognitive daemon start` appends kernel-server
stdout/stderr to `state/cognitiveos/daemon.log` (mode `0600` under the Personal
state directory). systemd `Type=simple` still uses the journal.

## 4. Configure and launch Pi

```text
cognitive pi configure --executable <abs-path-to-pi> --extension-entry <abs-path-to-dist/index.js>
cognitive pi launch
```

Launch is fail-closed: it requires all doctor components ready and the exact pinned
Pi version, loads only the configured Extension, disables Pi-native tools that bypass
daemon authority, and never hands Pi a Provider key. Your first message flows Pi →
daemon Provider proxy → Provider; see [The Pi shell](./pi-shell.md).
For a bounded non-interactive conversation, use `cognitive pi launch --print` and
provide the prompt on stdin. The CLI remains attached until the pinned Pi process
exits; the prompt is not a Provider credential and no Provider key is placed in
the command line or environment. Optional `--append-system-prompt <absolute-path>`
forwards an existing non-empty UTF-8 file to Pi (relative, missing, and empty
files fail closed). File bytes are not printed.

## 5. Optional: configure and launch DeepSeek Harness

```text
cognitive dsh configure --dsh-root <abs-dsh-checkout> --adapter-root <abs-dsh-akp-adapter> --revision 528c682e061696f5a160f363f236ecbf53cbd006
cognitive dsh launch --print --task "Reply with one sentence that summarizes this text and nothing else: CognitiveOS Personal is a local-first OS for governed agent work."
cognitive dsh web --no-open --host 127.0.0.1 --port 3080
cognitive dsh apply
cognitive dsh status
```

This is a candidate-only agent path, not a second authority writer. Configure
writes only the pin, adapter root, and a candidate-only adapter digest.
Launch requires daemon-owned ready state (system/database/secret/provider/daemon);
Pi may stay `not_configured`. Native panel: after `pnpm run build` in the pinned
dsh root, `cognitive dsh web --no-open` serves `http://127.0.0.1:3080` (not
Personal `/ui/`). Native Models follows the current dsh-bound account catalog;
`cognitive dsh apply` (or binding set/remove) reloads that list. Workspace* candidates still complete only on the
daemon Intent/Effect/verification/acceptance path. A dsh response is never Task
completion. Direct Flash (`--path a`) is measurement-only and is refused by
`cognitive dsh launch`.

## Failure exits worth knowing

Bad signature/pin → nothing is installed; health failure → previous service
restored; locked or missing keyring → `init` refuses; stale `daemon.lock` →
`cognitive daemon stop` cleans it only after proving the process is gone.
User-data backup is a separate verb (`cognitive backup` / `restore`); archives
never contain the Provider key.
