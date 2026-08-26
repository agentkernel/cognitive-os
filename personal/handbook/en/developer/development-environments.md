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
fingerprint: "sha256:661a5e80265cf94dedb761063496a1e59c98e659b8013e7f19b3354e24a8cb80"
non_claims:
  - Environment capability ceilings are owned by the environments registry; this page routes, it does not extend claims.
---

# Development environments

The environments registry
([`PERSONAL-TEST-ENVIRONMENTS.md`](../../../../docs/plan/PERSONAL-TEST-ENVIRONMENTS.md))
owns what each environment may claim. Practical routing:

| Environment | Use for | Never for |
|---|---|---|
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` (required PR checks) | full Rust + TS + conformance + drift gates | Gate/release/Profile promotion |
| `DEV-LINUX-NATIVE-01` (native Linux host) | exact-revision native validation, experimental service/Pi work; consumes **pushed commits only** into a cleanable worktree | uncommitted code, production claims |
| `CLOUD-AGENT-LINUX-01` (Cursor Cloud Agent pod) | full bash-shell Rust + TS iteration before pushing; bootstrapped by `.cursor/environment.json` | native systemd/Secret Service behavior, timing baselines, Gate/release/Profile |
| Local Windows GNU host | pnpm builds/tests, `cargo fmt`, Node checkers, docs work | any workspace `cargo build/test/clippy/run` — registered linker exit 121 |
| WSL2 | historical engineering evidence | product-path claims (product target is native Linux) |
| `B01-Desktop-Linux-002` | dedicated Gate-campaign guest under preregistered procedures | ordinary development, deployment, or testing |
| `B01-W-DESKTOP-001` | registered-but-not-provisioned Windows Gate guest (B01-W) | anything until provisioned per its preregistration |

Toolchain pins: Rust 1.97.1 (`rust-toolchain.toml`), pnpm 10.33.2 + Node ≥22
(`package.json`), workspace-wide `unsafe_code = "forbid"` and pedantic clippy
(`Cargo.toml`). Line endings are forced LF for text (`.gitattributes`), which is
also what keeps handbook fingerprints platform-stable.

Shell discipline on this Windows host: PowerShell 5.1 — no `&&`/`||`; sequence
with separate invocations or `if ($LASTEXITCODE -eq 0) { … }`. Neither that
rule nor the GNU linker ceiling applies inside `CLOUD-AGENT-LINUX-01`, which is
bash on a native GNU/Linux link host.

Cloud Agent pods and fresh Linux clones bootstrap with
`bash scripts/setup-dev-env.sh` (dependencies, pinned toolchain, docs-sync
hooks). A Cloud Agent pushes as `cursor[bot]`, whose token only covers the
repositories listed in that run's environment.

When an agent deploys Control Plane or dsh on `B01-Desktop-Linux-002`
(linux-002), the default owner review path is the **local Windows browser via
SSH port forward**, not guest-desktop Firefox alone. Confirm the daemon bind
with `cognitive daemon status` on the guest, then on `DEV-WIN-GNU-01`:

```powershell
ssh -J wuz@192.168.1.2 -L 48681:127.0.0.1:48681 -L 3080:127.0.0.1:3080 hal9001@192.168.123.160
```

Keep that session open and open `http://127.0.0.1:48681/ui/` (Control Plane;
paste the runtime management bootstrap secret, never a Provider API key) and
`http://127.0.0.1:3080/` (native dsh panel). After a guest daemon restart or
kernel-server replace, restart `cognitive dsh web` or run `cognitive dsh apply`
on that runtime before expecting dsh chat. Vite preview is not the product
origin. The environments registry owns the full port table and isolation
rules; this page only routes.

Command cheat sheet: see [AI validation commands](../ai/validation-commands.md) —
identical content, maintained once.
