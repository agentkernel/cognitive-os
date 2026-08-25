---
doc_id: user.getting-started
locale: en
kind: guide
audience: [user]
status: partial
generated: false
sources:
  - path: deploy/linux/install.sh
  - path: crates/cognitive-runtime/src/linux_bundle_service.rs
    symbols: ["cognitiveos-personal.service"]
  - path: apps/admin-cli/src/personal_cli/init.rs
    symbols: ["run_init"]
  - path: apps/admin-cli/src/personal_cli/mod.rs
    symbols: ["COGNITIVE_USAGE"]
  - path: apps/admin-cli/src/personal_cli/daemon.rs
  - path: apps/admin-cli/src/personal_cli/pi.rs
tests:
  - crates/cognitive-runtime/tests/linux_bundle_single_service.rs
  - crates/cognitive-runtime/tests/linux_installer_bootstrap.rs
  - apps/admin-cli/tests/p1_t06_cognitive_cli.rs
  - apps/admin-cli/tests/p2_t27_backup_restore.rs
fingerprint: "sha256:197ab38453e6b0c0ea2ca6c4c6819b79a1eaaabe97fc1ccefb248c65afd2911a"
non_claims:
  - There is no public production release artifact yet; installable bundles are experimental campaign builds.
  - This guide does not claim a Gate, Profile, production readiness, or Windows installation.
  - A first conversation proves connectivity, not autonomous task completion or agent quality.
---

# Getting started

This is the shortest supported path for a Linux x86_64 user-systemd machine. It
assumes a working OS Secret Service (for example GNOME Keyring), a Provider HTTPS
endpoint, and an exact pinned Pi package. There is no plaintext-secret fallback.

## 1. Install the bundle

Use the `install.sh` included with an installable bundle. The bootstrap path is
HTTPS-only, verifies the installer and signed bundle before activation, installs
one user service named `cognitiveos-personal.service`, and binds the daemon to
`127.0.0.1:48181`. A failed health or identity check restores the previous
version instead of reporting success.

## 2. Initialize the Provider

Pass the key through stdin or a protected file descriptor, never as an argument:

```text
cognitive init --provider <id> --base-url <https-url> --api-key-file -
```

The command probes the Provider, stores the key in the approved secret store, and
writes only an opaque `SecretRef` plus selected-model metadata to ordinary files.
For a disposable runtime that should reuse an existing binding, use
`--reuse-existing-secret-binding` with the Provider and base URL instead of
capturing another key. Rotate with `cognitive init --rotate-key`.

## 3. Start and inspect the daemon

```text
cognitive daemon start
cognitive status
cognitive doctor
```

`status` is a component projection, while `doctor` adds redacted diagnostics.
Look for `first_conversation_ready`; it additionally requires Pi configuration.
`ready` means the local configuration and process checks passed. It is not a
live guarantee that the Provider key will succeed on every request.

## 4. Configure and launch Pi

```text
cognitive pi configure \
  --executable <absolute-path-to-pi> \
  --extension-entry <absolute-path-to-dist/index.js>
cognitive pi launch
```

Launch checks the complete doctor projection and the pinned Pi version. The
Extension uses the daemon Provider proxy; Pi never receives the Provider key.
Pi-native shell and file tools remain disabled. For one bounded non-interactive
prompt, use:

```text
printf '%s\n' 'Summarize the current workspace.' | cognitive pi launch --print
```

`--append-system-prompt <absolute-path>` is optional and forwards an existing,
non-empty UTF-8 file. Relative, missing, and empty files fail closed.

## 5. Observe durable facts

Use the CLI projections rather than treating chat output as authority:

```text
cognitive resource get --family memory
cognitive resource get --family task
cognitive task watch
cognitive task evidence --task-ref task://<id>
```

The exact task URI and available families are returned by the authenticated
projection. A task may remain `DRAFT` until the scheduler acquires its lease;
that is normal and does not mean the conversation failed.

## 6. Back up and recover

```text
cognitive backup --output <directory>
cognitive restore --archive <directory> --preflight
cognitive restore --archive <directory>
```

Backups are digest-bound and exclude Provider keys, bearer tokens, and authority
SQLite. Run the preflight before a restore. After recovery, re-run `cognitive
status` and `cognitive doctor`; if the secret store no longer has the key, run
`cognitive init` again.

For troubleshooting, see [Operations and recovery](./operations-and-recovery.md),
[Provider and secrets](./provider-and-secrets.md), and [Known limitations](./known-limitations.md).
