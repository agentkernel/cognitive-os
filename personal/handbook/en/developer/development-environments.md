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
  - path: docs/bug/dsh-pathb-stale-daemon-bearer-after-daemon-restart.md
  - path: rust-toolchain.toml
  - path: .gitattributes
fingerprint: "sha256:ae40674151ddaa54c7d4e433d65b41928b66bf602ed26477e47bba46cc9295f2"
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
| Local Windows host `DEV-WIN-GNU-01` (GNU default toolchain; registered directories carry a local MSVC `rustup override` since 2026-09-03) | pnpm builds/tests, `cargo fmt`, Node checkers, docs work anywhere; workspace `cargo build/test/clippy` **inside an override directory** (`rustc -vV` → `host: x86_64-pc-windows-msvc`) as development iteration | Rust linking on the GNU default host — registered linker exit 121; citing a local MSVC result as supported validation, Gate, release, Profile or Windows support |
| `DEV-WINDOWS-NATIVE-OPC-01` | D01-qualified local project-runtime host (same machine as `DEV-WIN-GNU-01`; 2026-09-05; OS version is not a provision gate). Unsigned install fail-closed + live daemon admit. Tray/OS-sleep/sandbox/signed-install stay `not-run` | citing cargo as native install/tray/sleep E2E; Gate/release/Profile; B01-W |
| WSL2 | historical engineering evidence | Linux 1.0 or Windows OPC product-path claims |
| `B01-Desktop-Linux-002` | dedicated Gate-campaign guest under preregistered procedures; since 2026-08-27 also the owner-authorized Personal 2.0 development-validation host (exact-revision disposable worktrees and task-declared cleanable roots only; frozen while a B01 campaign is active) | guest-baseline, snapshot, or credential changes outside a preregistered B01 campaign lease |
| `B01-W-DESKTOP-001` | registered-but-not-provisioned Windows Gate guest (B01-W) | anything until provisioned per its preregistration |

Phase 11 Personal 2.0.0 routing: T03/T04 daily authority tests use
`CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` (plus exact-revision `DEV-LINUX-NATIVE-01`
when a native daemon/store is required). T02/T07 native host/DSH E2E stay
`DEV-WINDOWS-NATIVE-OPC-01` = D01-qualified 2026-09-05 (`P13-T13`); hung native
cells that lack a capability stay honest `not-run`. `P11-T15` is in-progress on this qualified host (N=15 frozen at `main@4ca9b046`). T09 is HITL on canvas, not a first-level Inbox. `B01-DESKTOP-002`
is campaign-only, not the 2.0 daily default. `P11-T15` N=15 acceptance
runs on this preregistered qualified Windows revision and is **not** the
Phase 12 prototype-completeness mutex. Phase 12 Dual Track UI work uses
`DEV-WIN-GNU-01` TS plus required CI; product-chrome native UI E2E stays `not-run`
(fixture `/ui/` 200 is not that cell).
Phase 13 routing
(`PERSONAL-TEST-ENVIRONMENTS.md` §5.2): P13-T02/T03 real child/Pi paths and
the other authority cards use `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` plus pushed
exact-revision `DEV-LINUX-NATIVE-01`; `/ui/` surfaces use Dual Track TS; the
P13-T12/D02 rendered / NVDA / 200% / host-theme review against exact-revision
guest daemon `/ui/` (SSH tunnel) is implementation evidence only; `DOC-LOCAL-RUNTIME-HOST`
(2026-09-05) designated this local host as `DEV-WINDOWS-NATIVE-OPC-01` (OS version
is not a provision gate). `P13-T13/D01` qualified the unsigned path on 2026-09-05;
D02 accounted hung cells pass/fail/`not-run`. `P11-T15` is in-progress after T13 closed. Local cargo, WSL,
Linux, ordinary CI and Canvas are explicit non-substitutes for Gate/release;
`not-run` remains `not-run`.

Toolchain pins: Rust 1.97.1 (`rust-toolchain.toml`), pnpm 10.33.2 + Node ≥22
(`package.json`), workspace-wide `unsafe_code = "forbid"` and pedantic clippy
(`Cargo.toml`). Line endings are forced LF for text (`.gitattributes`), which is
also what keeps handbook fingerprints platform-stable. On the local Windows
host `git config core.autocrlf` reports `true`; that setting is overridden by
the tracked `.gitattributes` rule `* text=auto eol=lf`, so checkouts and
commits stay LF without any local Git configuration change.

Local MSVC override on `DEV-WIN-GNU-01` (P0-T01/D02, owner decision
2026-09-03, local-only): the machine's rustup default host is
`x86_64-pc-windows-gnu`, so `rust-toolchain.toml` alone resolves to the GNU
toolchain whose linking fails (exit 121). The repair is a rustup **directory
override** — `rustup override set 1.97.1-x86_64-pc-windows-msvc` — recorded for
`D:\agent-kernel` and the task worktree; it is stored in rustup's settings, not
in the repository, so `rust-toolchain.toml`, CI and every other clone are
unchanged (`.cargo/config.toml` is not gitignored here and is not used). The
installed Visual Studio Build Tools 17.14.37 at `D:\VSBuildTools` provide
`link.exe` 14.44.35228.0, which rustc finds by itself — no PATH or `vcvars`
step. Check `rustc -vV` reports `host: x86_64-pc-windows-msvc` before running
`cargo build --workspace --locked`, `cargo test --workspace --locked --
--test-threads=1`, `cargo clippy --workspace --all-targets --locked -- -D
warnings` or `cargo fmt --all -- --check`; a new local worktree needs its own
`rustup override set`. On this disk-constrained machine the workspace test
build needs `CARGO_PROFILE_DEV_DEBUG=0` (session environment variable) to
fit, and the four `kernel-server` `tool_executor` tests whose fixture creates a
symlink/reparse point fail at setup with OS error 1314 because the shell is not
elevated and Developer Mode is off (they pass on the elevated CI runner; treat
them as `not-run (host privilege)` locally, never skip them in code). Results
are development evidence; the environments registry §3 keeps the capability
ceiling unchanged. PowerShell 7.6.5 (`pwsh`) is installed but the
Cursor Shell remains Windows PowerShell 5.1.

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
kernel-server replace, restart `cognitive dsh web` on that runtime before
expecting dsh chat; the new daemon reports dsh `INACTIVE`, so `cognitive dsh
apply` cannot recover that stale session. Reserve `apply` for supported
binding/model overlay synchronization while the runtime is already `ACTIVE`.
Vite preview is not the product origin. The environments registry owns the
full port table and isolation rules; this page only routes.

Command cheat sheet: see [AI validation commands](../ai/validation-commands.md) —
identical content, maintained once.
