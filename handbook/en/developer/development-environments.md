---
doc_id: dev.dev-environments
locale: en
kind: guide
audience: [developer, ai]
status: implemented
generated: false
sources:
  - path: docs/plan/PERSONAL-TEST-ENVIRONMENTS.md
    symbols: ["B01-DESKTOP-002", "CI-UBUNTU-01", "DEV-LINUX-NATIVE-01", "RUST-LINK-DEV-WIN-GNU-01"]
  - path: rust-toolchain.toml
  - path: .gitattributes
fingerprint: "sha256:37fcd64ae0f0d44f62e1abd2d7d35ea611921544dc842dcce50cba9d30d71be6"
non_claims:
  - Environment capability ceilings are owned by the environments registry; this page routes, it does not extend claims.
---

# Development environments

The environments registry
([`PERSONAL-TEST-ENVIRONMENTS.md`](../../../docs/plan/PERSONAL-TEST-ENVIRONMENTS.md))
owns what each environment may claim. Practical routing:

| Environment | Use for | Never for |
|---|---|---|
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` (required PR checks) | full Rust + TS + conformance + drift gates | Gate/release/Profile promotion |
| `linux-002` / `B01-Desktop-Linux-002` (native Linux guest, via the registered ProxyJump route) | designated primary exact-revision native development/test validation in a task-scoped, cleanable worktree once Git + Rust 1.97.1 are qualified | uncommitted code, production claims, or B01 campaign assets |
| Local Windows GNU host | pnpm builds/tests, `cargo fmt`, Node checkers, docs work | any workspace `cargo build/test/clippy/run` — registered linker exit 121 |
| WSL2 | historical engineering evidence | product-path claims (product target is native Linux) |
| `B01-Desktop-Linux-002` (B01 use) | preregistered B01 campaigns | combining ordinary development/test results with the campaign denominator, or changing its baseline, snapshot, roots, credentials, or evidence |
| `B01-W-DESKTOP-001` | registered-but-not-provisioned Windows Gate guest (B01-W) | anything until provisioned per its preregistration |

Toolchain pins: Rust 1.97.1 (`rust-toolchain.toml`), pnpm 10.33.2 + Node ≥22
(`package.json`), workspace-wide `unsafe_code = "forbid"` and pedantic clippy
(`Cargo.toml`). Line endings are forced LF for text (`.gitattributes`), which is
also what keeps handbook fingerprints platform-stable.

Current linux-002 qualification: the route, user systemd, and Node v22.23.2
are available; Git and Rust are not yet available on the guest PATH. Use the
registered CI route until those prerequisites are provisioned; do not label CI
output as linux-002 native evidence.

Shell discipline on this Windows host: PowerShell 5.1 — no `&&`/`||`; sequence
with separate invocations or `if ($LASTEXITCODE -eq 0) { … }`.

Command cheat sheet: see [AI validation commands](../ai/validation-commands.md) —
identical content, maintained once.
