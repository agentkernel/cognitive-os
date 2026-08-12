---
doc_id: user.security-boundaries
locale: en
kind: concept
audience: [user]
status: implemented
generated: false
sources:
  - path: apps/kernel-server/src/personal/auth.rs
    symbols: ["LocalSessionAuthority"]
  - path: apps/kernel-server/src/personal/bounds.rs
  - path: packages/pi-cognitiveos/src/tool-policy.ts
  - path: crates/cognitive-runtime/src/pi_launcher.rs
    symbols: ["admit_pi_launch"]
  - path: docs/governance/AXIOMS.md
tests:
  - apps/kernel-server/tests/p1_t04_personal_daemon.rs
  - packages/pi-cognitiveos/src/safety.test.ts
  - crates/cognitive-runtime/tests/pi_linux_launcher.rs
fingerprint: "sha256:dc3a553d159650e6ff1ac848e59207cd74679d7ef52f750fb6b42087c9dccee4"
non_claims:
  - Local bearers use a non-cryptographic RNG and Windows file ACL hardening is absent — this page states the boundary as it is, not as a hardening guarantee.
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

Honest limits: bearer generation uses a non-cryptographic hash source; whoever can
read the bootstrap file can name any principal; per-OS-user isolation relies on
file permissions (no Windows ACL hardening).

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

## Request bounds (DoS hygiene)

1 MiB bodies, 16 KiB header block, 64 headers, 10 s/30 s read timeouts, 32
connections (16 in-flight) — all fail-closed with registered error codes.

## What protects your data at rest

Authority databases are 0600 WAL SQLite files owned by the daemon; secrets live
exclusively in the Secret Service (see
[Provider and secrets](./provider-and-secrets.md)); backups exclude secret material
by construction. The append-only audit/event history cannot be rewritten through
any daemon surface — updates and deletes are rejected by database triggers.
