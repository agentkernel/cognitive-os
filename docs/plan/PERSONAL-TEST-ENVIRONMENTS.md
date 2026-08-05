# CognitiveOS Personal Test and Development Environments

- Status: active environment registry
- Last reconciled: 2026-08-03
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
| Local documentation, consistency, TypeScript or diff verification | `DEV-WIN-GNU-01` is eligible when the command does not trigger Rust compile/link | treating a PowerShell parser rejection as a test failure |
| Rust formatting | `DEV-WIN-GNU-01` may run `cargo fmt --all -- --check` | using formatting as build/test evidence |
| Rust build/test/Clippy/run/bench | `CI-UBUNTU-01`, `CI-WINDOWS-MSVC-01`, or exact-revision `DEV-LINUX-NATIVE-01` according to the required evidence | invoking these first on `DEV-WIN-GNU-01`, whose registered result is linker exit 121 |
| Windows GNU toolchain repair | a separately approved and leased P0-T01 Delivery Slice with explicit acceptance | ad hoc LLVM-MinGW, shim, PATH, Rust pin or source workaround inside a feature Slice |

`COMMAND-SHELL-PS51` means a command rejected by the local PowerShell parser
did not execute and is recorded as `not-run`. `RUST-LINK-DEV-WIN-GNU-01`
means the current GNU host's known linker exit 121 is an environment capability
boundary, not a regression to reproduce for each Rust change. If the required
supported route is unavailable, the validation and affected Delivery Slice
remain `blocked`/`not-run`; an unrelated `ready` Slice may proceed.

## 2. Environment summary

| ID | Environment | Kind | Maximum current evidence scope |
|---|---|---|---|
| `DEV-WIN-GNU-01` | local Windows GNU/MinGW host | local development | TypeScript and non-linking checks only |
| `CI-UBUNTU-01` | GitHub `ubuntu-latest` | ordinary supported CI | `tested-supported-ci` implementation evidence |
| `CI-WINDOWS-MSVC-01` | GitHub `windows-latest` | ordinary supported CI | `tested-supported-ci` implementation evidence |
| `DEV-WSL2-01` | Windows WSL2 Linux guest | local Linux guest | strong local/fixture implementation evidence |
| `DEV-LINUX-NATIVE-01` | `personal-linux-native-01` | experimental native Linux | `tested-local` native evidence |
| `BUILD-LINUX-EXPERIMENTAL-01` | protected experimental campaign builder | reviewed CI build/sign | experimental artifact evidence |
| `B01-DESKTOP-002` | Ubuntu Desktop KVM campaign host | **sole active B01 environment** | individual B01 attempt evidence |
| `FIXTURE-SYSTEMD-01` | fake-systemd/installer fixtures | deterministic fixture | lifecycle implementation evidence |
| `FIXTURE-PROVIDER-HTTPS-01` | loopback HTTPS Provider fixture | deterministic fixture | Provider transport implementation evidence |
| `CONTRACT-RUNNERS-01` | golden/conformance/consistency runners | contract/tooling | scoped contract and tooling evidence |

## 3. `DEV-WIN-GNU-01` — local Windows GNU/MinGW

- **Recorded platform:** `x86_64-pc-windows-gnu`.
- **Recorded tools:** Rust 1.97.1; local TypeScript baseline used Node 24.15.0
  and pnpm 10.33.2.
- **Command shell:** local Cursor commands use Windows PowerShell 5.1. Never
  use `&&` or `||`; use parallel calls, separate dependent calls, or
  `if ($LASTEXITCODE -eq 0) { <next-command> }`.
- **Observed allowlist:** frozen pnpm install, workspace TS build/test, Node
  tooling, documentation/static consistency, diff checks and Rust formatting.
- **Known limitation (`RUST-LINK-DEV-WIN-GNU-01`):** workspace Rust
  build/test/Clippy/run/bench is unsupported and fails during linking with exit
  121, including the already exhausted LLVM-MinGW/shim retry.
- **No-repeat rule:** do not run Rust compiling/linking commands on this host
  merely to reconfirm the known result. Only an explicitly approved P0-T01
  toolchain-repair Slice may retest or change linker/PATH/toolchain settings.
- **Required transfer:** route Rust validation to supported Ubuntu CI,
  Windows/MSVC CI, or an exact-revision native Linux worktree before the Slice
  starts; if unavailable, record `blocked`/`not-run` rather than substituting
  this host.
- **Maximum evidence:** local TS/development checks actually executed.
- **Cannot claim:** supported Windows Rust, Windows product install, B01-W,
  sandbox/containment, release or Profile.
- **Evidence:** [`tests/baseline/README.md`](../../tests/baseline/README.md).

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
- **Evidence:** [baseline](../../tests/baseline/README.md) and CI workflow.

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
- **Recorded platform:** Linux x86_64, non-WSL, native user-systemd/user D-Bus,
  glibc 2.35.
- **Recorded tools:** Rust 1.97.1, Node 22.19.0; exact Pi 0.81.1 must be
  rechecked for every slice.
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
B01 campaign environment**. Ordinary development must not start, stop,

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
- **Maximum evidence:** one successful formal B01 attempt.
- **Cannot claim:** campaign-level B01 pass until at least 20 attempts, at least
  90% success, zero critical failures, complete statistics and independent
  verifier closure; the experimental signer cannot satisfy P7 production trust.
- **Evidence:** [attempt ledger](../checkpoints/20260801-personal-p1-t09-b01-attempt-ledger.md).

## 11. `FIXTURE-SYSTEMD-01` — installer/fake-systemd fixtures

- **Form:** temporary roots, synthetic signed archives, recording controllers,
  fake `systemctl --user` scripts and loopback health fixtures.
- **Proves:** fixed unit/port/action ordering, safe extraction, staging/active
  pointer semantics, compensation and no receipt on incomplete rollback.
- **Maximum evidence:** deterministic runtime/installer implementation tests.
- **Cannot claim:** real systemd, native persistence, Secret Service, Provider,
  Pi, clean VM, B01, release, containment or Profile.
- **Tests:** `crates/cognitive-runtime/tests/linux_bundle_*.rs` and
  `linux_installer_bootstrap.rs`.

## 12. `FIXTURE-PROVIDER-HTTPS-01` — loopback HTTPS Provider fixture

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
  [`p1_t09_deterministic_provider_fixture.rs`](../../crates/cognitive-provider-transport/tests/p1_t09_deterministic_provider_fixture.rs).

## 13. `CONTRACT-RUNNERS-01` — golden, conformance and consistency

- **Golden:** Rust and TypeScript canonical JSON/digest consumers share
  `tests/golden/` fixtures.
- **Conformance:** declarative vectors and executable runners distinguish pass,
  fail, not-run and not-applicable; static validity is not behavior pass.
- **Consistency:** registry/schema/vector/link/project/lease and Personal-plan
  invariants are checked in ordinary CI.
- **Maximum evidence:** exact contract/tooling checks that were executed.
- **Cannot claim:** a Personal product Gate, Agent benefit, release or Profile
  without the separately required applicable-MUST evidence.

## 14. Formal campaign qualification template

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
