---
doc_id: dev.dev-environments
locale: en
kind: guide
audience: [developer, ai]
status: implemented
generated: false
sources:
  - path: docs/plan/PERSONAL-TEST-ENVIRONMENTS.md
    symbols: ["CI-UBUNTU-01", "DEV-LINUX-NATIVE-01", "RUST-LINK-DEV-WIN-GNU-01"]
  - path: rust-toolchain.toml
  - path: .gitattributes
fingerprint: "sha256:3e3ce206ee0b9041acbc3ebd62a42b928074cfb9518d52a6323294afdcc1d455"
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
| `DEV-LINUX-NATIVE-01` (native Linux host) | exact-revision native validation, experimental service/Pi work; consumes **pushed commits only** into a cleanable worktree | uncommitted code, production claims |
| Local Windows GNU host | pnpm builds/tests, `cargo fmt`, Node checkers, docs work | any workspace `cargo build/test/clippy/run` — registered linker exit 121 |
| WSL2 | historical engineering evidence | product-path claims (product target is native Linux) |
| `B01-Desktop-Linux-002` | dedicated Gate-campaign guest under preregistered procedures | ordinary development, deployment, or testing |

Toolchain pins: Rust 1.97.1 (`rust-toolchain.toml`), pnpm 10.33.2 + Node ≥22
(`package.json`), workspace-wide `unsafe_code = "forbid"` and pedantic clippy
(`Cargo.toml`). Line endings are forced LF for text (`.gitattributes`), which is
also what keeps handbook fingerprints platform-stable.

Shell discipline on this Windows host: PowerShell 5.1 — no `&&`/`||`; sequence
with separate invocations or `if ($LASTEXITCODE -eq 0) { … }`.

Command cheat sheet: see [AI validation commands](../ai/validation-commands.md) —
identical content, maintained once.
