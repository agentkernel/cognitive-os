# CognitiveOS Personal Test and Development Environments

- Status: active environment registry
- Last reconciled: 2026-09-05
- Product/task status source: [PROGRESS.md](PROGRESS.md) `Current snapshot`
- Platform claim source: [PERSONAL-SUPPORT-MATRIX.md](PERSONAL-SUPPORT-MATRIX.md)

This registry explains where existing development and test evidence was
obtained and the maximum claim that each environment can support. It is not a
Gate ledger and does not turn an environment name into evidence.

## 1. Common pins and rules

| Item | Current pin or policy |
|---|---|
| Rust | `1.97.1` from `rust-toolchain.toml`, with rustfmt and Clippy |
| pnpm | `10.33.2` from the workspace `packageManager` |
| Workspace Node policy | `>=22`; Pi-qualified execution requires `>=22.19.0` |
| Pi | `@earendil-works/pi-coding-agent@0.81.1` |
| Pi source commit | `20be4b18d4c57487f8993d2762bace129f0cf7c6` |
| Pi npm SRI | `sha512-r6ovAsZOgAqbC/aU6s+/dPnv/sGZBuWyZNvi3pXjpbuX5wvp3XvGkQI7/VLvX2o9XpmpFaPUxKNym1WfkN/P8A==` |
| Linux payload ABI target | `x86_64-unknown-linux-gnu.2.35` for the experimental builder |
| Product service/endpoint | `cognitiveos-personal.service`, `127.0.0.1:48181` |
| Local Cursor command shell | Windows PowerShell 5.1; `COMMAND-SHELL-PS51` applies |

Every native/remote slice must re-record exact versions, artifact/adapter
digests, secret preconditions, reset and cleanup. Secret material, raw Provider
traffic and sensitive SQLite content are forbidden from command arguments,
terminal captures and committed evidence.

### 1.1 Fail-fast command and validation routing

These are persistent environment facts. A normal implementation Slice must
consume them before selecting commands; it must not rediscover them by running
known-invalid syntax or a known-unsupported linker:

| Need | Required route | Forbidden repeat |
|---|---|---|
| Multiple independent local commands | separate parallel Shell calls | joining them with `&&` or `||` under Windows PowerShell 5.1 |
| Dependent local commands | separate calls, or `if ($LASTEXITCODE -eq 0) { <next-command> }` | bash command chaining unless the command explicitly starts in bash |
| Local documentation, consistency, TypeScript or diff verification | `DEV-WIN-GNU-01` is eligible | treating a PowerShell parser rejection as a test failure |
| Rust formatting | `DEV-WIN-GNU-01` may run `cargo fmt --all -- --check` | using formatting as build/test evidence |
| Local Rust build/test/Clippy iteration | `DEV-WIN-GNU-01` **only in a directory carrying the registered local MSVC `rustup override`** (§3; `rustc -vV` must report `host: x86_64-pc-windows-msvc`); results are local development evidence and are recorded in the Slice's running report | running them in a directory without the override (that is still the GNU host: linker exit 121); treating a local pass as supported-CI, Gate, release, Profile or Windows-support evidence |
| Rust build/test/Clippy/run/bench as **supported validation** | `CI-UBUNTU-01`, `CI-WINDOWS-MSVC-01`, or exact-revision `DEV-LINUX-NATIVE-01` according to the required evidence | substituting the local MSVC override result for the supported route; invoking anything Rust-linking on the GNU host |
| Local toolchain change | a separately approved and leased P0-T01 Delivery Slice with explicit acceptance (`P0-T01/D02` registered the MSVC override) | ad hoc LLVM-MinGW, shim, PATH, Rust pin, `rust-toolchain.toml` or source workaround inside a feature Slice |

`COMMAND-SHELL-PS51` means a command rejected by the local PowerShell parser
did not execute and is recorded as `not-run`. `RUST-LINK-DEV-WIN-GNU-01`
is the registered **GNU-host** fact: with the default `x86_64-pc-windows-gnu`
host (which `rust-toolchain.toml` still resolves to in any directory without
the local override) workspace linking stops at linker exit 121; that fact is
an environment boundary, not a regression to reproduce. Since `P0-T01/D02`
(2026-09-03) the two registered local directories carry a rustup directory
override to `1.97.1-x86_64-pc-windows-msvc`, which makes local Rust iteration
possible there (§3) without changing the host's capability ceiling. If the
required supported route is unavailable, the validation and affected Delivery
Slice remain `blocked`/`not-run`; an unrelated `ready` Slice may proceed.

## 2. Environment summary

| ID | Environment | Kind | Maximum current evidence scope |
|---|---|---|---|
| `DEV-WIN-GNU-01` | local Windows host (GNU default host; registered directories carry a local MSVC `rustup override` since 2026-09-03) | local development | TypeScript/static checks everywhere; local Rust build/test/Clippy iteration only inside the override directories — development evidence, never supported-CI/Gate/release/Profile |
| `CI-UBUNTU-01` | GitHub `ubuntu-latest` | ordinary supported CI | `tested-supported-ci` implementation evidence |
| `CI-WINDOWS-MSVC-01` | GitHub `windows-latest` | ordinary supported CI | `tested-supported-ci` implementation evidence |
| `DEV-WINDOWS-NATIVE-OPC-01` | same physical host as `DEV-WIN-GNU-01` (owner 2026-09-05: project runtime testing on this machine; OS version is not a provision gate; recorded Windows 10 Pro 10.0.19045) | **D01 qualified** 2026-09-05 (`P13-T13`); D02 hung E2E cells accounted | unsigned bootstrap fail-closed + live cargo-built daemon admit + `/ui/`; tray/OS-sleep/OS-DACL/sandbox/signed-install remain `not-run`; not Gate/release/Profile/B01-W |
| `CLOUD-AGENT-LINUX-01` | Cursor Cloud Agent Linux pod | ephemeral remote container | strong local/container implementation evidence |
| `DEV-WSL2-01` | Windows WSL2 Linux guest | local Linux guest | strong local/fixture implementation evidence |
| `DEV-LINUX-NATIVE-01` | `personal-linux-native-01` | experimental native Linux | `tested-local` native evidence |
| `BUILD-LINUX-EXPERIMENTAL-01` | protected experimental campaign builder | reviewed CI build/sign | experimental artifact evidence |
| `B01-DESKTOP-002` | Ubuntu Desktop KVM campaign host | **sole active B01 environment** | individual B01 attempt evidence |
| `B01-W-DESKTOP-001` | required clean Windows B01-W campaign VM | **not provisioned** | none; nothing may cite it until it is provisioned and qualified |
| `FIXTURE-SYSTEMD-01` | fake-systemd/installer fixtures | deterministic fixture | lifecycle implementation evidence |
| `FIXTURE-PROVIDER-HTTPS-01` | loopback HTTPS Provider fixture | deterministic fixture | Provider transport implementation evidence |
| `CONTRACT-RUNNERS-01` | golden/conformance/consistency runners | contract/tooling | scoped contract and tooling evidence |

## 3. `DEV-WIN-GNU-01` — local Windows host (GNU default, local MSVC override)

The environment ID is retained for continuity with every earlier evidence
record; it names the owner's local Windows development machine, whose rustup
**default host** is still `x86_64-pc-windows-gnu`. Re-registered 2026-09-03 by
`P0-T01/D02` (owner decision: local-only override, tracked
`rust-toolchain.toml` unchanged).

- **Recorded platform:** Windows 10 Pro 10.0.19045, x86_64. rustup 1.29.0
  (`RUSTUP_HOME=D:\DevEnv\Rustup`, `CARGO_HOME=D:\DevEnv\Cargo`,
  `CARGO_TARGET_DIR=D:\DevEnv\CargoTarget`); default host
  `x86_64-pc-windows-gnu`; installed toolchains `stable-x86_64-pc-windows-gnu`,
  `1.97.1-x86_64-pc-windows-gnu`, `1.97.1-x86_64-pc-windows-gnullvm`,
  `1.97.1-x86_64-pc-windows-msvc` (with `rustfmt` + `clippy` added by
  `P0-T01/D02`).
- **Local MSVC override (the registered local Rust link path):**
  `rustup override set 1.97.1-x86_64-pc-windows-msvc` is set for exactly
  `D:\agent-kernel` (the `D:\agent-kernel-wt-p0-t01` worktree was removed in
  2026-09-05 disk hygiene); it lives in rustup's own
  settings, not in the repository, and takes precedence over
  `rust-toolchain.toml` only in those directories (`.cargo/config.toml` is not
  gitignored here and is therefore **not** used). In an override directory
  `rustc -vV` reports `host: x86_64-pc-windows-msvc` for the same pinned
  `1.97.1` release/commit as CI. Any new local worktree must set its own
  override; `rustup override unset --path <dir>` reverts. No PATH,
  environment-variable, `vcvars`, or machine-wide change is part of the
  registration: rustc locates `link.exe` through the Visual Studio setup
  configuration on its own.
- **Linker:** Visual Studio Build Tools 17.14.37 (installationVersion
  `17.14.37516.0`) at `D:\VSBuildTools`; MSVC toolset 14.44.35207;
  `D:\VSBuildTools\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64\link.exe`
  version 14.44.35228.0 (not on the Cursor Shell PATH — it does not need to
  be); Windows SDK 10.0.26100.0.
- **Recorded tools:** Rust 1.97.1 (`rustc 8bab26f4f 2026-07-14`, LLVM 22.1.6);
  pnpm 10.33.2; Node 24.15.0 for the historical TypeScript baseline;
  PowerShell 7.6.5 (`pwsh`) installed alongside the Windows PowerShell 5.1
  Cursor Shell; Git `core.autocrlf=true` at system scope, overridden for every
  tracked text path by `.gitattributes` `* text=auto eol=lf` (no local Git
  change required).
- **Command shell:** local Cursor commands use Windows PowerShell 5.1
  (`COMMAND-SHELL-PS51`). Never use `&&` or `||`; use parallel calls,
  separate dependent calls, or `if ($LASTEXITCODE -eq 0) { <next-command> }`.
  If `cargo`/`rustup` are not on PATH: `$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"`.
- **Observed allowlist:** frozen pnpm install, workspace TS build/test, Node
  tooling, documentation/static consistency, diff checks, Rust formatting,
  and — **inside an override directory only** — `cargo build --workspace --locked`,
  `cargo test --workspace --locked -- --test-threads=1`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo fmt --all -- --check` (executed results and exact revision in the
  `P0-T01/D02` running report). Local Rust iteration is development evidence
  that helps a Slice reach a pushable checkpoint sooner; supported validation
  is still the CI/native route below.
- **Known local limitations of the override path (recorded 2026-09-03;
  disk remeasured 2026-09-05 after Pchat-backup and cargo-target hygiene):**
  (1) disk — `CARGO_TARGET_DIR` is `D:\DevEnv\CargoTarget`; after 2026-09-05
  hygiene `C:` had ~14 GB free and `D:` ~25 GB free. The full workspace test
  build still uses `CARGO_PROFILE_DEV_DEBUG=0` in the shell session (no
  persistent change; debuginfo level does not alter what is compiled or
  asserted). Linker temp should use `TEMP`/`TMP` on `D:\tmp\rust-link`, not
  the default `C:\Users\...\Temp`. (2) privilege — the Cursor Shell is not elevated,
  `SeCreateSymbolicLinkPrivilege` is not held and Windows Developer Mode is
  off, so the four `kernel-server` `tool_executor` unit tests whose fixture
  creates a symlink/reparse point fail at fixture setup with OS error 1314
  (`ERROR_PRIVILEGE_NOT_HELD`); they pass on the elevated hosted
  `windows-latest` runner. Treat those four as `not-run (host privilege)`
  locally — do not weaken or skip them in code; enabling Developer Mode or an
  elevated shell is an owner-level machine setting, not a Slice action.
- **Known limitation (`RUST-LINK-DEV-WIN-GNU-01`, GNU-host history):** with
  the default `x86_64-pc-windows-gnu` host — i.e. in any directory **without**
  the override, or if the override is unset — workspace Rust
  build/test/Clippy/run/bench fails during linking with exit 121, including the
  already exhausted LLVM-MinGW/shim retry (2026-07-25 baseline). The override
  does not change that host fact; it routes around it.
- **No-repeat rule:** do not run Rust compiling/linking commands on the GNU
  host merely to reconfirm the known result, and do not "fix" a linker exit 121
  by ad hoc PATH/toolchain/`rust-toolchain.toml` edits inside a feature Slice
  — check `rustc -vV` reports `host: x86_64-pc-windows-msvc` first. Only an
  explicitly approved P0-T01 toolchain-repair Slice may change
  linker/PATH/toolchain settings or this registration.
- **Required transfer:** route Rust **validation** (the evidence a Slice exit
  or acceptance cites) to supported Ubuntu CI (`CI-UBUNTU-01`), Windows/MSVC CI
  (`CI-WINDOWS-MSVC-01`), or an exact-revision native Linux worktree
  (`DEV-LINUX-NATIVE-01`) before the Slice starts; if unavailable, record
  `blocked`/`not-run` rather than substituting this host's local result.
- **Maximum evidence:** local TS/development checks actually executed,
  including local MSVC-override Rust build/test/Clippy results labelled as
  such. Capability ceiling **unchanged** by `P0-T01/D02`.
- **Cannot claim:** supported Windows Rust, `tested-supported-ci`, Windows
  product install, citing cargo as native product-runtime E2E (this machine
  is designated `DEV-WINDOWS-NATIVE-OPC-01`; cargo is not that
  qualification), B01-W, sandbox/containment, release or Profile.
- **Evidence:** [`personal/tests/baseline/README.md`](../../personal/tests/baseline/README.md)
  (2026-07-25 GNU baseline);
  [`P0-T01/D02` running report](../checkpoints/2026-09-03-personal-p0-t01-d02-toolchain-report.md)
  (2026-09-03 override fact probe and local cargo results).

## 4. `CI-UBUNTU-01` — supported Ubuntu ordinary CI

- **Runner:** GitHub `ubuntu-latest`; the exact image is floating and must be
  captured separately for a release campaign.
- **Tools:** Rust 1.97.1, Node major 22 selected by workflow, pnpm 10.33.2.
- **Workloads:** TS build/test; Rust build/test/Clippy/fmt; codegen drift;
  consistency/trace checks; conformance; wrong-implementation self-check;
  cross-language golden digests.
- **Maximum evidence:** `tested-supported-ci` for affected implementation and
  contract tooling.
- **Cannot claim:** native user-systemd, Secret Service, B01/B09,
  `GMVP-LINUX`, containment, release or Profile unless a campaign explicitly
  preregisters the exact environment.
- **Definition:** [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml).

## 5. `CI-WINDOWS-MSVC-01` — supported Windows/MSVC ordinary CI

- **Runner:** GitHub `windows-latest`; hosted MSVC image/tool details float.
- **Tools/workloads:** same repository pins and broad workspace checks as the
  Ubuntu job.
- **Maximum evidence:** supported Windows compile/test implementation evidence.
- **Cannot claim:** Windows installer/service, native credential backend,
  B01-W, install parity, release or Profile.
- **Evidence:** [baseline](../../personal/tests/baseline/README.md) and CI workflow.

### 5.1 Phase 11 Windows OPC validation route

ADR-0059 / Personal 2.0.0 chrome makes Windows the product target.
`DEV-WINDOWS-NATIVE-OPC-01` is **D01-qualified** (2026-09-05, `P13-T13`) as the
unsigned development-runtime host on this machine. That is not a Gate / release /
Profile / B01-W product environment. Do not invent environment IDs.
`B01-DESKTOP-002` is **not** the 2.0 daily development default.

| Phase 11 work | Required development evidence | Explicit non-substitute |
|---|---|---|
| P11-T02 host/tray/background | `CI-WINDOWS-MSVC-01` compile/test plus `DEV-WINDOWS-NATIVE-OPC-01` unsigned install (D01 **pass** fail-closed) / SecretStore Credential Manager (D02 **pass** synthetic) ; tray / OS sleep / service remain `not-run` | citing cargo as install/tray/sleep E2E; WSL, Linux, Canvas, `B01-DESKTOP-002` as daily default |
| P11-T03/T04 Project/Employee | daily authority tests: `CI-UBUNTU-01` and `CI-WINDOWS-MSVC-01`; native daemon/store when needed: pushed exact-revision `DEV-LINUX-NATIVE-01` | docs/fixtures alone; GNU Rust link; treating `B01-DESKTOP-002` as daily default |
| P11-T05/T06 Conversation/Pi Assistant | projection negatives on required CI; Windows archive/index/Pi native route remains unqualified (`not-run`) | Linux Pi qualification transfer |
| P11-T07 hidden hosted DSH | required CI plus qualified Windows artifact/sandbox/process/stdio/Provider/update/rollback E2E on `DEV-WINDOWS-NATIVE-OPC-01` (`not-run` until qualified) | existing Linux dsh Path B or research HEAD as Windows product; DSH `apps/web` as `/ui/` |
| P11-T08 Routine | required CI plus qualified Windows clock/sleep/offline/restart E2E when available | process exit or synthetic timer only |
| P11-T09 HITL canvas (not Inbox) | required CI plus qualified Windows UI/runtime E2E when available; HITL is canvas + Today deep link | first-level Inbox queue; chat Approve |
| P11-T10/T11 Knowledge/Memory | projection/retrieval negatives on CI; qualified Windows filesystem/index/privacy/rebuild E2E when available | proprietary Obsidian app availability |
| P11-T12 Provider/honest usage | required CI plus qualified Windows SecretStore/daemon-proxy/usage route; unknown cost ≠ 0; member budget not current chrome | raw env/plaintext credential; member-level budget stop as 2.0.0 chrome |
| P11-T13 OPC UI | client tests/contract mock/empty states plus daemon-served `/ui/`; NVDA/200%/host-theme contrast hung `not-run`; Dual Track after T03 HTTP stable | Vite preview as product origin; claiming full IA before Project authority |
| P11-T14 X connector | `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` plus exact-revision `DEV-LINUX-NATIVE-01` store/HTTP. Live X/Twitter API/account/CAPTCHA/platform qualification = `Requires-environment` / `not-run` | CAPTCHA/fingerprint/anti-abuse evasion; Linux CI as platform qualification |
| P11-T15 fixed acceptance | **done** on qualified `DEV-WINDOWS-NATIVE-OPC-01` at exact `main@4ca9b046`; N=15 frozen 2026-09-05; cell 1 **partial**, cell 2 Dual Track empty Home **pass**, cells 3–15 **not-run**; Draft PR [#325](https://github.com/agentkernel/cognitive-os/pull/325); validated `e55adb82` required CI [33963162039](https://github.com/agentkernel/cognitive-os/actions/runs/33963162039) **SUCCESS** | ordinary CI, local GNU, Linux, WSL, Canvas; Linux cells standing in for Windows scenarios |

### 5.2 Phase 13 completion validation route

Phase 13 closes the gap between the P11 walking skeletons / P12 walkable
scenes and the frozen prototype + design goals. Linux native evidence closes
"the implementation exists"; every Windows-native cell stays `not-run` until
`P13-T13` qualifies the host and backfills it. Do not invent environment IDs.

| Phase 13 work | Required development evidence | Explicit non-substitute |
|---|---|---|
| P13-T02 hosted DSH real Attempt loop | `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` plus pushed exact-revision `DEV-LINUX-NATIVE-01` real child spawn, stdio broker, terminal observation; `DEV-WIN-GNU-01` fmt/docs/TS only (`HOSTED_DSH_WIN_GNU_FENCE`) | Linux Path B or GNU fence as Windows sandbox/ACL/supply-chain qualification (that is `P13-T13`) |
| P13-T03 hidden Pi real inference | required CI plus pushed exact-revision `DEV-LINUX-NATIVE-01` with the pinned Pi and daemon Provider proxy | Linux Pi qualification as a Windows Pi route; echoing client payload as "inference" |
| P13-T04 / T05 / T06 / T09 / T10 / T11 authority + `/ui/` | required CI plus `DEV-LINUX-NATIVE-01` for authority/store/HTTP; Dual Track TS on the `DEV-WIN-GNU-01` allowed surface for `/ui/`; product origin daemon `/ui/` | file open / clock-sleep-restart / FS / supply-chain host cells before `P13-T13` |
| P13-T07 / T08 surfaces | Dual Track TS + required CI; `DEV-LINUX-NATIVE-01` for SecretStore route | Windows SecretStore / privacy host cells before `P13-T13`; Vite as product origin |
| P13-T12 visual spec + a11y qualification | D01 documentation-only; D02 rendered browser / NVDA / 200% / host-theme review from a registered host's local browser against pushed exact-revision guest daemon `/ui/` (`DEV-LINUX-NATIVE-01` over the documented SSH tunnel) = implementation evidence | canvas screenshots; skipping a cell as pass; rendered review as Windows native chrome qualification |
| P13-T13 Windows native qualification | owner-designated local host (same machine as `DEV-WIN-GNU-01`; OS version is not a provision gate; recorded Windows 10 Pro 10.0.19045) as `DEV-WINDOWS-NATIVE-OPC-01`; D01 unsigned path **qualified** 2026-09-05; D02 hung cells accounted pass/fail/`not-run` on this host | CI / WSL / Linux / `B01-DESKTOP-002`; B01-W VM as a daily development host; unsigned dev path as release; citing cargo as native install/tray/sleep E2E |

`DEV-WINDOWS-NATIVE-OPC-01` was designated (2026-09-05, `DOC-LOCAL-RUNTIME-HOST`)
and **D01-qualified** (2026-09-05, `P13-T13`) on this local machine. Qualification
is the unsigned development path actually running here, not CI/GNU/WSL/Linux.
Hung native cells that lack a capability stay honest `not-run`. This is not Gate,
release, Profile, B01-W, or T15.

### 5.3 `DEV-WINDOWS-NATIVE-OPC-01` — D01-qualified local project-runtime host

Owner instruction 2026-09-05: run project runtime tests on this machine; do
not require Windows 11 as a provision gate. No new environment ID.

| Item | Fact |
|---|---|
| Physical host | Same machine as `DEV-WIN-GNU-01` (`D:\agent-kernel`) |
| Recorded OS | Windows 10 专业版 (Pro) `10.0.19045`, x86_64 (fact, not a gate) |
| Designation | 2026-09-05 via `DOC-LOCAL-RUNTIME-HOST` |
| D01 qualification | 2026-09-05 `P13-T13/D01`: unrendered `personal/deploy/windows/install.ps1` via system PowerShell → exit **64** (`release policy is not rendered`, no TEMP leftover); live cargo-built `kernel-server --personal` admits a disposable Windows path ending in `Personal Home`, rejects GNU/Linux roots (422) and the task channel (403), `daemon.bind`, `GET /ui/` is 503 `LOCAL_UI_BUNDLE_UNAVAILABLE` without a bundle and 200 with `data/cognitiveos/ui/index.html`. Test: `kernel-server` `--test p13_t13_windows_native_host` **2/2**. |
| rustc | `1.97.1` (`8bab26f4f 2026-07-14`), `host: x86_64-pc-windows-msvc`, LLVM 22.1.6; directory override on `D:\agent-kernel` |
| Node / pnpm | Node `24.15.0`; pnpm `10.33.2` |
| D02 hung cells (2026-09-05) | T02 install **pass** (fail-closed); T02 SecretStore **pass** (`cognitive-secret` `--test p7_t07_windows_credential_store` **7/7** synthetic Credential Manager); T02 tray **not-run** (`tray_proves_work=false`); T02/T05 OS sleep/restart **not-run** (daily machine not slept); P13-T02 sandbox **not-run** (Windows native channels remain `Unsupported`; `sandbox::tests::matrix_keeps_windows_native_unsupported_without_evidence` **pass**); P13-T02 ACL **not-run** (SQLite policy only); P13-T02 supply chain **not-run** (no rendered/signed Windows bundle); P13-T08 Settings `connection.connect` **not-run** (live discovery not invoked); P13-T04 host file-open **not-run** (`host_file_open_e2e` stays `not-run`); UI native chrome **not-run** (fixture `/ui/` 200 is not product chrome); live X **not-run**. Install-surface template tests **10/10**. |
| Cargo / Dual Track TS on this host | development evidence only; not native product-runtime E2E; not Gate/release/Profile |
| B01-W / signing / release / Profile | unchanged and separate |

`CI-WINDOWS-MSVC-01` may prove compile/test behavior only. It cannot prove
Windows install, tray/background, SecretStore, process/ACL containment, sleep/
missed recovery, DSH supply chain/runtime, connector behavior, B01-W, support,
release, or Profile.

## 6. `DEV-WSL2-01` — WSL2 Linux guest

- **Recorded kernel:** `6.18.33.2-microsoft-standard-WSL2`.
- **Recorded tools:** Rust 1.97.1, Node 22.14.0, pnpm 10.33.2. Node 22.14.0 is
  below Pi's `>=22.19.0` requirement, so the environment is not Pi-qualified.
- **Observed use:** broad Rust/TS regressions and fake-systemd/single-service
  installer fixtures on a Linux filesystem target directory.
- **Maximum evidence:** local Linux-guest implementation/fixture evidence.
- **Cannot claim:** Linux-native product behavior, native Secret Service,
  B01/B09, containment, release or Profile. WSL2 is not a 1.0 product runtime.
- **Evidence examples:**
  [toolchain handoff](../checkpoints/20260726-toolchain-recovery-and-worktree-landing-handoff.md)
  and [P1-T08 handoff](../checkpoints/20260729-personal-p1-t08-mvp-single-service-handoff.md).

## 7. `DEV-LINUX-NATIVE-01` — `personal-linux-native-01`

- **Access identity:** `wuz@192.168.1.2`, non-interactive SSH. The standing
  operator authorization permits required approved Secret Store access,
  least-privilege elevation, and scoped service/system configuration changes
  without per-operation confirmation; secret values remain prohibited from SSH
  arguments/output, ordinary configuration, SQLite, logs, CI, tests, and
  evidence.
- **Registered B01 route (verified 2026-08-15):** connect to the libvirt host
  as `wuz@192.168.1.2` (host identity `hal9000`), then use SSH ProxyJump to
  `hal9001@192.168.123.160` (guest identity `hal9001-Standard-PC-Q35-ICH9-2009`).
  The guest is `B01-Desktop-Linux-002`; host-side libvirt operations must use
  `virsh -c qemu:///system`.
- **Recorded platform:** Linux x86_64, non-WSL, native user-systemd/user D-Bus,
  glibc 2.35.
- **Recorded tools:** Rust 1.97.1, Node 22.19.0; exact Pi 0.81.1 must be
  rechecked for every slice.
- **Git remote:** GitHub HTTPS access is available. Before native validation,
  fetch or clone the already pushed candidate revision directly from
  `https://github.com/agentkernel/cognitive-os.git`; do not substitute a
  no-Git source snapshot or a copied local tree. This was rechecked on
  2026-08-06 with a non-interactive `git ls-remote` probe.
- **Observed use:** real Extension invocation, native Secret Service/Provider
  prerequisite checks, independently verified experimental deployment and
  focused P2 tests.
- **Reset/cleanup:** disposable remote roots; no credentials, raw responses or
  authority paths in SSH arguments/output; remove installed test state as the
  slice declares.
- **Maximum evidence:** `experimental-local-only` / `tested-local` native
  implementation evidence.
- **Cannot claim:** B01/B09, product Gate, release, containment or Profile.

### Execution routing and KVM isolation

Linux daemon, Pi/sidecar, installer, user-service, native integration and
experimental deployment slices use this SSH host as their primary execution
environment. Before a remote build, test or experimental deployment, the
operator must create a disposable Git worktree from an already pushed/reviewed
revision, record `git rev-parse HEAD`, and verify it equals the local candidate
commit. An old source tree, a no-Git snapshot, or copied uncommitted local
files are invalid test inputs.

Windows may perform formatting, static, documentation and platform-independent
checks, but does not substitute for this native Linux validation. If the host,
toolchain, exact revision or disposable root is unavailable, record the Linux
check as `not-run` or blocked; do not silently fall back to Windows and label
the result Linux-native.

The host's `B01-Desktop-Linux-002` libvirt guest is the **sole active formal
B01 campaign environment** and, per the 2026-08-27 owner instruction, is also
registered for **owner-authorized Personal 2.0 development validation**. The
B01 boundary is unchanged and takes precedence: only a preregistered B01
campaign lease may change the guest baseline, snapshots, product installation
baseline, or credential state, and standing operator authorization does not
expand that boundary. Owner-authorized 2.0 development validation on this
guest is limited to exact-revision disposable Git worktrees and task-declared
cleanable directories/data roots; it must not alter the guest baseline,
snapshot set, or credential state, must clean up as each slice declares, and
ends each debug/validation session with the SSH port-forward owner review
path in the "Owner viewing" subsection below. During an active preregistered
B01 campaign, development use of the guest is frozen. The retired `B01-Clean-Linux-001` guest remains historical
qualification evidence only and must not be restored as a B01 or ordinary
development target.

### Owner viewing from `DEV-WIN-GNU-01` (default after guest deploy)

When an agent deploys or validates Control Plane / dsh on **linux-002**, the
default owner review path is **local Windows browser via SSH port forward**,
not guest-desktop Firefox alone.

**Route:** `wuz@192.168.1.2` (libvirt host `hal9000`) → ProxyJump →
`hal9001@192.168.123.160` (`B01-Desktop-Linux-002`).

**Typical owner-ops loopback ports** (confirm on guest before forwarding):

| Surface | Guest loopback | Product path |
|---|---|---|
| Personal daemon + Control Plane | `127.0.0.1:48681` | `http://127.0.0.1:48681/ui/` |
| Native dsh harness panel | `127.0.0.1:3080` | `http://127.0.0.1:3080/` |

Default product daemon bind is `127.0.0.1:48181`; long-running owner-ops
runtimes on linux-002 often use **`48681`** — always forward the port the
active `cognitive daemon status` reports.

**PowerShell on the owner Windows machine (`COMMAND-SHELL-PS51`):**

```powershell
ssh -J wuz@192.168.1.2 -L 48681:127.0.0.1:48681 -L 3080:127.0.0.1:3080 hal9001@192.168.123.160
```

Keep the session open. Open locally:

- `http://127.0.0.1:48681/ui/` — Control Plane (management-session gate:
  bootstrap secret from guest runtime, not Provider API key)
- `http://127.0.0.1:3080/` — native dsh panel (Path B via daemon proxy)

After guest **daemon restart** or kernel-server replace, agents must restart
`cognitive dsh web` on that runtime before owner dsh review. The new daemon
projects dsh as `INACTIVE`, so `cognitive dsh apply` cannot recover the stale
Path B management session; reserve `apply` for supported overlay synchronization
while the runtime is already `ACTIVE`. This is an operator recovery step, not
a Control Plane code defect.

Agents should record the forwarded ports and exact Git revision deployed when
handing off UI review. **After each guest debug session, remind the owner** with
the SSH forward command and local URLs — do not omit this handoff. Vite preview
or a separate clients dev server is not the product Control Plane origin.

## 8. `BUILD-LINUX-EXPERIMENTAL-01` — protected experimental builder

- **Trigger/runner:** manual workflow dispatch on `ubuntu-latest` with GitHub
  Environment `personal-linux-experimental-campaign`.
- **Tools:** Zig 0.14.0, cargo-zigbuild 0.23.0, glibc 2.35 target.
- **Trust:** protected experimental signing seed/keyring; artifact retention is
  seven days; no GitHub Release and no runtime execution.
- **Maximum evidence:** reviewed, non-production build/signing and ABI facts.
- **Cannot claim:** P7 production trust, B01/B09 by itself, `GMVP-LINUX`,
  release or Profile.
- **Definition:**
  [personal experimental workflow](../../.github/workflows/personal-experimental-linux-campaign.yml).

## 9. `B01-CLEAN-SERVER-001` — Ubuntu Server KVM candidate

- **Platform:** Ubuntu Server 24.04 LTS, x86_64, 2 vCPU, 4 GiB RAM, independent
  16 GiB QCOW2 overlay, native user-systemd.
- **Reset:** snapshot `b01-pre-install-baseline`.
- **Result:** transient Secret Service worked, but a product-compatible
  persistent default/login collection could not satisfy the headless reset
  procedure. The attempt start gate was never crossed.
- **Maximum evidence:** environment qualification and failed prerequisite.
- **Cannot claim:** a B01 attempt or failure denominator entry, Provider/Pi
  execution, release or Profile.
- **Evidence:** [clean-VM handoff](../checkpoints/20260731-personal-p1-t09-b01-clean-vm-handoff.md).

## 10. `B01-DESKTOP-002` — Ubuntu Desktop campaign environment

- **Platform:** Ubuntu Desktop 24.04.4 LTS, x86_64, non-WSL, PID 1 systemd,
  native user-systemd and encrypted login keyring.
- **Image:** `ubuntu-24.04.4-desktop-amd64.iso`, SHA-256
  `3a4c9877b483ab46d7c3fbe165a0db275e1ae3cfe56a5657e5a47c2f99a99d1e`.
- **Reset:** `b01-platform-qualified-baseline`.
- **Attempt 1 artifact:** `0.0.0-campaign.20260801.1` from `main@0a5524b`,
  artifact SHA-256
  `80e6a4d0d633b34e949fce92afb8b8fcfc4ae6dca6c4fd244888540a777a3394`,
  experimental signer/keyring and exact Pi 0.81.1/SRI.
- **Result:** attempt 1 passed executed phases, produced the expected bounded
  response in 6295 ms with `authority_side_effects:false`, and completed secret
  cleanup.
- **Maximum evidence:** the completed ADR-0039 successor campaign `002` on this
  guest (fixed N=6, 5 successes / 1 failure, zero critical safety failures,
  complete aggregate statistics, and affirmative independent verifier
  disposition). Retained campaign `001` remains a historical fail record.
- **Cannot claim:** G1, GMVP-LINUX, release, Profile, or P7 production trust from
  the experimental signer; B01 pass does not transfer those claims.
- **Evidence:** [attempt ledger](../checkpoints/20260801-personal-p1-t09-b01-attempt-ledger.md),
  [ADR-0039](../adr/0039-personal-b01-six-attempt-campaign-policy.md), and the
  PROGRESS.md Current snapshot B01 row.
- **Registered dual use (2026-08-27 owner instruction):** also the primary
  real-machine validation host for Personal 2.0 development slices under the
  §7 isolation paragraph — exact-revision disposable worktrees and
  task-declared cleanable roots only; no guest-baseline, snapshot, or
  credential change outside a preregistered B01 campaign lease; development
  use is frozen while such a campaign is active.

## 11. `B01-W-DESKTOP-001` — required Windows B01-W campaign environment (not provisioned)

- **Status:** requirement registration only. No VM exists; no evidence may
  cite this ID until the environment is provisioned, snapshotted, and
  qualified under the §15 template.
- **Required platform:** dedicated clean Windows 11 x86_64 (or Windows 10
  22H2+) VM with a PID-controlled reset snapshot, native per-user Credential
  Manager, a graphical interactive session capable of hidden input, system
  Windows PowerShell 5.1 and System32 `curl.exe`, and no developer toolchain
  or preexisting CognitiveOS state.
- **Purpose:** sole future execution environment for the preregistered
  `B01-W-clean-windows-first-install-first-conversation-001` campaign
  ([preregistration](../checkpoints/20260812-personal-p7-t07-b01-w-preregistration.md),
  [ADR-0052 §3](../adr/0052-personal-windows-install-surface.md)).
- **Isolation:** ordinary development must not target this ID once it exists;
  only the preregistered B01-W campaign procedure and lease may change its
  state. The Linux `B01-DESKTOP-002` guest and its host are not B01-W targets.
- **Maximum evidence:** none until provisioned and qualified.
- **Cannot claim:** anything; `CI-WINDOWS-MSVC-01`, fixtures, WSL, or local
  Windows development hosts cannot substitute for it.

## 12. `FIXTURE-SYSTEMD-01` — installer/fake-systemd fixtures

- **Form:** temporary roots, synthetic signed archives, recording controllers,
  fake `systemctl --user` scripts and loopback health fixtures.
- **Proves:** fixed unit/port/action ordering, safe extraction, staging/active
  pointer semantics, compensation and no receipt on incomplete rollback.
- **Maximum evidence:** deterministic runtime/installer implementation tests.
- **Cannot claim:** real systemd, native persistence, Secret Service, Provider,
  Pi, clean VM, B01, release, containment or Profile.
- **Tests:** `personal/crates/cognitive-runtime/tests/linux_bundle_*.rs` and
  `linux_installer_bootstrap.rs`.

## 13. `FIXTURE-PROVIDER-HTTPS-01` — loopback HTTPS Provider fixture

- **Form:** local TLS Provider with a test-only additional root that preserves
  production Rustls policy.
- **Covers:** discovery serialization, malformed/unauthorized responses,
  capability mismatch, timeout, response cap, redirect refusal, selected-model
  persistence, request counts and secret redaction.
- **Maximum evidence:** Provider transport implementation behavior in supported
  ordinary CI.
- **Cannot claim:** live Provider, native Secret Service, Pi conversation,
  B01 or release.
- **Test:**
  [`p1_t09_deterministic_provider_fixture.rs`](../../personal/crates/cognitive-provider-transport/tests/p1_t09_deterministic_provider_fixture.rs).

## 14. `CONTRACT-RUNNERS-01` — golden, conformance and consistency

- **Golden:** Rust and TypeScript canonical JSON/digest consumers share
  `core/tests/golden/` fixtures.
- **Conformance:** declarative vectors and executable runners distinguish pass,
  fail, not-run and not-applicable; static validity is not behavior pass.
- **Consistency:** registry/schema/vector/link/project/lease and Personal-plan
  invariants are checked in ordinary CI.
- **Maximum evidence:** exact contract/tooling checks that were executed.
- **Cannot claim:** a Personal product Gate, Agent benefit, release or Profile
  without the separately required applicable-MUST evidence.

## 15. `CLOUD-AGENT-LINUX-01` — Cursor Cloud Agent Linux pod

- **Kind:** ephemeral `x86_64` Linux container that checks out one pushed
  revision of `agentkernel/cognitive-os` and is discarded with the run.
- **Bootstrap:** [`.cursor/environment.json`](../../.cursor/environment.json)
  runs [`scripts/setup-dev-env.sh`](../../scripts/setup-dev-env.sh), which
  installs the pnpm workspace, materializes the pinned Rust toolchain, and
  registers the docs-sync hooks. Without that bootstrap a pod starts with no
  `node_modules` and no hook registration.
- **Command routing:** `COMMAND-SHELL-PS51` does **not** apply — the shell is
  bash, so `&&`/`||` are valid. `RUST-LINK-DEV-WIN-GNU-01` does **not** apply
  either: this is a native GNU/Linux link host, so `cargo build/test/clippy`
  run here instead of being routed away.
- **Recorded capability (2026-08-24, `main@46397764`):** `pnpm -r build` and
  `pnpm -r test` pass; `cargo fmt --all -- --check` clean; `cargo build
  --workspace` 48.9 s; `cargo clippy --workspace --all-targets` 19.6 s clean;
  `cargo test --workspace` 1210 passed / 0 failed; `conformance-runner`
  completes. Every figure is wall-clock on shared cloud hardware.
- **Known interaction:** once `target/debug/kernel-server` exists, the
  optional `packages/sdk-ts` live tests stop being skipped and
  `live: task watch stream…` fails reproducibly. This is a pre-existing
  condition that ordinary CI never reaches, because it runs `pnpm -r test`
  before the Rust steps. See the
  [environment diagnosis](../checkpoints/20260824-cloud-agent-dev-environment-and-push-diagnosis.md)
  §4.1.
- **Git identity:** commits are authored by the Cursor GitHub App
  (`cursor[bot]`). Its installation token is scoped to the repositories in the
  run's environment, so pushes to any repository outside that list fail with
  HTTP 403 regardless of that repository's visibility.
- **Maximum evidence:** container-class implementation evidence, equivalent in
  standing to `DEV-WSL2-01` — useful for fast iteration and pre-CI triage.
- **Cannot claim:** native user-systemd, Secret Service, Pi-qualified
  execution, timing baselines, B01/B09, `GMVP-LINUX`, containment, release or
  Profile. Required CI on the pushed revision remains the merge gate, and
  `DEV-LINUX-NATIVE-01` remains the native-Linux evidence route.

## 16. Formal campaign qualification template

Before an environment contributes to B01, P2/P3/P4 Gates, B09 or `GMVP-LINUX`, its
preregistration must bind:

1. campaign and Gate ID plus formal-plan revision/digest;
2. exact OS image, architecture, native/virtual classification and reset;
3. source revision, product artifact/signature/SBOM/attestation;
4. Node and Agent package/version/SRI/digest plus sidecar package, protocol,
   adapter, instance and process-identity pins where applicable;
5. local Skill package/revision digest and binding set;
6. Memory/Context schema and migration versions plus source/index rebuild policy;
7. native Tool descriptor/catalog version and digest, including the exact
   enabled operation set;
8. selected SecretStore backend and operator credential opt-in boundaries;
   desktop campaigns pin Secret Service behavior, while headless campaigns pin
   encrypted-vault version, locked start, SSH TTY unlock and any systemd
   encrypted-credential unlock path without retaining unlock or Provider/user
   secret material;
9. workload, attempt denominator, thresholds and failure accounting; when
   applicable bind the exact
   [UCR-01 workload](../evaluation/personal-unified-cognitive-resource-workload.md)
   revision/digest and the six-resource same-Task trace;
10. evidence collector version, redaction and cleanup;
11. operator and independent verifier identities;
12. allowed claim scope and explicit non-claims, including that one UCR-01 run
    cannot automatically pass multiple Gates.
