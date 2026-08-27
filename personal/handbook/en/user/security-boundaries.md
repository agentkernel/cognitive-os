---
doc_id: user.security-boundaries
locale: en
kind: concept
audience: [user]
status: implemented
generated: false
sources:
  - path: personal/apps/kernel-server/src/personal/auth.rs
    symbols: ["LocalSessionAuthority"]
  - path: personal/apps/kernel-server/src/personal/bounds.rs
  - path: personal/packages/pi-cognitiveos/src/tool-policy.ts
  - path: personal/crates/cognitive-runtime/src/pi_launcher.rs
    symbols: ["admit_pi_launch"]
  - path: docs/governance/AXIOMS.md
  - path: docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md
  - path: docs/adr/0057-personal-2-0-mcp-resource-family.md
tests:
  - personal/apps/kernel-server/tests/p1_t04_personal_daemon.rs
  - personal/apps/kernel-server/tests/p2_t18_local_token_csprng.rs
  - personal/packages/pi-cognitiveos/src/safety.test.ts
  - personal/crates/cognitive-runtime/tests/pi_linux_launcher.rs
fingerprint: "sha256:09d42f2e888c412ccde64d84dcaca513bac1675d17f8920887161727e42597ce"
non_claims:
  - Windows file ACL hardening for local runtime files is absent — OS-CSPRNG token generation does not make an ACL claim.
  - ADR-0055 adopts a credential-import boundary but no concrete import mechanism; Account Hub import remains Requires-backend.
---

# Security boundaries

## Network

The daemon binds loopback only and rejects non-loopback bind addresses lexically
before listening. There is no TLS on the local listener (localhost-only by design),
no cookies (any `Cookie` header is refused), and optional `Host` validation.
Provider egress is HTTPS-only with redirects disabled.

## Identity and channels

`POST /local/session` exchanges the per-boot `local-bootstrap.secret` (file mode
0600 in your XDG runtime dir) for **channel-bound** bearers: management tokens can
never call task routes and vice versa. Sessions expire (12 h absolute / 30 min
idle) and die with the daemon process.

Bootstrap and session opaque tokens each carry 256 bits produced by the operating
system CSPRNG. If OS entropy is unavailable, short, zero, or repeats its independent
probe block, initialization/session issuance fails before creating a file, session,
or token; there is no PID/time/hash fallback. A persisted bootstrap with the legacy
predictable shape or any malformed non-empty shape is not grandfathered: startup
fails closed. With the daemon stopped, remove only that runtime credential to let
the next start mint a CSPRNG replacement. Whoever can read the bootstrap file can
still name any principal, and per-OS-user isolation relies on file permissions (no
Windows ACL hardening).

## Agent containment

- The Pi shell extension denies `project_trust` and every built-in Pi tool; source
  scans assert the extension itself has no filesystem/subprocess/SQLite/key access.
- The daemon-launched Pi **candidate** process runs with tools, skills, sessions,
  and extension discovery disabled, a cleared environment allowlist, byte-capped
  frames, and hard deadlines; its only network path is a one-shot private socket
  back to the daemon.
- `admit_pi_launch` fail-closes on Windows-native/WSL2 hosts, missing sandbox
  adapter, digest mismatches, and any model egress other than the registered HTTPS
  proxy endpoint.

## User-directed credential import (adopted target)

ADR-0055 extends approved non-logging input paths without weakening secret
isolation. Every future import must satisfy all of these conditions:

- the user initiates it and consents to the exact named source and target
  SecretStore before the source is read;
- only the Rust daemon reads the source and writes the approved SecretStore;
- source material is transient in daemon memory and never reaches the UI,
  Agents, sidecars, argv, environment, ordinary CognitiveOS configuration,
  SQLite, logs, CI/test output, evidence, support output, or chat;
- audit records contain redacted metadata only;
- source retention is the default; secure deletion is an explicit per-import
  user choice.

Browser profile/cookie decryption, third-party Agent credential-file parsing,
subscription-token import, and OAuth capture are `Requires-backend`. The
accepted boundary is not proof that any of them exists.

The adopted MCP seventh-family target is also `Requires-backend`: connection
credentials stay in an approved SecretStore, MCP clients/servers/packages/
adapters remain candidate or observation producers, and an advertised tool,
resource, or prompt grants no capability. Raw connection material never
reaches the Control Plane, Agent, sidecar, package metadata, ordinary config,
SQLite, Context, logs, evidence, or chat.

## Request bounds (DoS hygiene)

1 MiB bodies, 16 KiB header block, 64 headers, 10 s/30 s read timeouts, 32
connections (16 in-flight) — all fail-closed with registered error codes.

## What protects your data at rest

Authority databases are 0600 WAL SQLite files owned by the daemon; secrets live
exclusively in the Secret Service (see
[Provider and secrets](provider-and-secrets.md)). Named Provider Control Plane
accounts persist only an opaque `secret_ref` in SQLite; API keys never appear in
authority rows, CLI output, audit payloads, or agent-readable files. Operator
usage of the CLI (including `--allow-private-network` / `--allow-insecure-http`
and `--reconfirm`) is in
[Provider Control Plane](provider-control-plane.md). Backups
exclude secret material
by construction. The append-only audit/event history cannot be rewritten through
any daemon surface — updates and deletes are rejected by database triggers.
