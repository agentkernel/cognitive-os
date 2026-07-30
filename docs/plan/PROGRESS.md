# PROGRESS — 单页进度仪表

## Current snapshot (2026-07-30)

This section is the authoritative current view. Entries below `Historical
evidence journal` preserve execution-time facts and cannot override it.

| Area | Current status | Evidence boundary | Next actionable step |
|---|---|---|---|
| P1-T09 route implementation | `in-progress` | `experimental-local-only`; prior `tested-local` launch/readiness evidence; deterministic binary Provider fixture implementation staged but focused execution is `not-run` in this unsupported local linker environment | execute the fixture/discovery focused suite on supported Linux CI, then add the real pinned Pi Extension load |
| B01 first-install/first-conversation Gate | `not-run` | no product-Gate, release, or Profile claim | pre-register qualified Linux campaign environment and runner |
| GMVP-LINUX | `not-run` | no release claim | waits for B01 plus P2 and P7 acceptance evidence |
| Profile conformance | `implemented: 0` | non-claim | independent applicable-MUST evidence only |
| Active task lease | P1-T09 deterministic binary Provider fixture follow-up; active Lane-RUN lease on `lane/personal-p1-t09-provider-fixture` | compile-fix paths are limited to the existing transport composition roots and closure docs; normative assets remain Lane-CTR-owned | push the compile fix and inspect the replacement CI result |

The P1-T09 implementation evidence is real but incomplete. The current status is
therefore intentionally `in-progress`, not `done`; B01 remains `not-run`.

The current atomic slice adds a loopback-only HTTPS Provider fixture process and
an additional-root test seam that preserves the production Rustls policy. Its
failure-first integration suite covers real discovery serialization, malformed
and unauthorized responses, non-chat capability, timeout, oversized response,
redirect refusal, selected-model persistence, deterministic request counts, and
secret redaction. The suite has not yet executed locally because the supported
Linux toolchain is unavailable and the Windows GNU linker exits 121 before the
tests start; this is not pass evidence.

## Historical evidence journal

> **Fail-closed Pi launch preparation slice (2026-07-30):** The
> install-to-first-conversation route remains `in-progress`; `P1-T09 / B01`
> remains `not-run` while task P1-T09 is `in-progress`. `cognitive pi launch` now admits only a daemon-owned,
> numeric loopback endpoint document and an authenticated ready Personal doctor
> projection before it reads the fixed non-secret `pi.json`. It requires all
> first-conversation components (including native SecretStore, Provider and
> digest-matched selected model) to be ready; corrupt/missing endpoint or
> configuration, relative/missing Pi paths, readiness failures, and exact Pi
> `0.81.1` version drift reject launch. The spawned client receives only the
> confirmed `--extension <absolute-path>` argument and a cleared,
> OS-execution allowlist environment; it receives no Provider or secret
> material. Focused `windows_wsl2_linux_guest` admin-cli Personal units
> **15/15**, Pi/readiness **1/1**, Personal readiness **1/1**, Provider-proxy
> **2/2**, and cognitive CLI **5/5** passed; strict changed-package Clippy
> passed. This is implementation and local-test evidence only: it does not
> demonstrate a real Pi Extension load, Provider conversation, native Secret
> Service, B01, Gate, release, or Profile claim.

> **Non-secret Pi configuration slice (2026-07-30):** The
> install-to-first-conversation route remains `in-progress`; `P1-T09 / B01`
> remains `not-run` while task P1-T09 is `in-progress`. Trusted public upstream source at the reviewed Pi
> `0.81.1` commit confirms `--extension` / `-e <path>` as the exact Extension
> loading option. The Personal CLI now offers `cognitive pi configure`, which
> atomically writes only the existing non-secret `pi.json` fields after
> rejecting relative paths and all non-configuration flags (including Provider
> secret inputs). It does not start Pi, access Provider configuration,
> SecretRefs, SecretStore, SQLite, or authority state; the daemon still owns
> Pi file/version readiness observation. Focused `windows_wsl2_linux_guest`
> admin-cli Personal units **9/9** passed after a failure-first relative-path
> test. This is implementation and local-test evidence only: it does not
> provide a Pi launch, Pi Extension load, deterministic binary Provider
> fixture, first conversation, native Secret Service, B01, Gate, release, or
> Profile claim.

> **Readiness truth and installed XDG launch slice (2026-07-29):** The
> install-to-first-conversation route remains `in-progress`; `P1-T09 / B01`
> remains `not-run` while task P1-T09 is `in-progress`. A Provider component now becomes `ready` only when
> its non-secret `provider.json` snapshot digest has a matching valid,
> chat-capable `selected-model.json`; missing, malformed, or digest-mismatched
> selected-model state blocks both aggregate and first-conversation readiness
> with redacted local error classes. `cognitive daemon start` now leaves
> `--runtime-root` absent for installed XDG launches, so `kernel-server`,
> `cognitive init`, and the Pi extension share the real user layout; an
> explicit hermetic root is still forwarded only for tests. The CLI default is
> aligned with the canonical loopback service at `127.0.0.1:48181`. Focused
> `windows_wsl2_linux_guest` evidence: kernel-server unit suite **32/32**,
> Pi/readiness integration **1/1** each, Provider-proxy regression **2/2**,
> admin-cli Personal units **6/6**, and cognitive CLI regression **5/5**
> passed. This is implementation and local-test evidence only: it does not
> provide a Pi configuration/launch command, actual Pi Extension loading, a
> deterministic binary Provider fixture, a real Provider conversation, native
> Secret Service evidence, B01, a Gate, release, or Profile claim.

> **Provider discovery and selected-model prerequisite (2026-07-29):** The
> install-to-first-conversation route remains `in-progress`; `P1-T09 / B01`
> remains `not-run` while task P1-T09 is `in-progress`. A new shared `cognitive-provider-transport` adapter
> now owns the bounded Rustls-only Provider egress boundary used by both the
> daemon proxy (through its compatibility re-export) and `cognitive init`.
> The adapter preserves HTTPS-only URLs, no redirects, URL-user-info and
> header-injection rejection, timeout/cancellation behavior, and the 1 MiB
> response limit. When Provider flags are supplied, `cognitive init` now
> configures the SecretStore binding and runs `ProviderDiscoveryService`; a
> supplied `--model-id` is `ExactCatalog`, never a manual fallback. Only a
> chat-capable probe persists `selected-model.json` and the non-secret snapshot
> digest; a failed or missing catalog model clears stale selection and reports
> a redacted actionable error. Focused `windows_wsl2_linux_guest` evidence:
> private init discovery tests **2/2 passed**, shared transport tests **2/2
> passed**, daemon proxy regression **2/2 passed**, and hermetic cognitive CLI
> regression **5/5 passed**. Strict Clippy for the changed packages and
> formatter passed. This is implementation and local-test evidence only: no
> real Provider, Pi launch, first conversation, native campaign, B01, Gate,
> release, or Profile claim is made.

> **Install-to-first-conversation XDG/endpoint foundation (2026-07-29):** The
> current first-conversation work item is `in-progress`. `kernel-server
> --personal` now resolves the real user XDG layout when its explicit
> hermetic-only `--runtime-root` is absent; it no longer creates a PID-scoped
> temporary layout for the installed user service. After a successful loopback
> bind, the daemon atomically publishes its actual bound endpoint to the shared
> non-secret `state/cognitiveos/daemon-endpoint.json` document and removes it
> during orderly shutdown. `cognitive daemon start` no longer pre-publishes an
> endpoint: it waits for the lock, bootstrap secret, and daemon-owned endpoint
> before reporting success. Focused `windows_wsl2_linux_guest` evidence:
> kernel-server Personal daemon integration **5/5 passed**, CLI daemon lifecycle
> integration **1/1 passed**, and strict Clippy passed for both crates. This is
> implementation and local-test evidence only, not Provider discovery,
> selected-model persistence, Pi launch, first conversation, B01, a product
> Gate, or Profile conformance.

> **P1-T08 Linux-native closeout (2026-07-29):** P1-T08 is `done`.
> The experimental release-shaped campaign executed the inspected shell through
> the fixed production Rust adapter and `/usr/bin/systemctl --user` on the
> designated independent Linux-native host `personal-linux-native-01`.
> Evidence covers a clean install of `0.0.0-campaign.20260729.3`, a healthy
> upgrade to `.4`, a pre-pointer failure for `.5`, and a post-pointer final
> confirmation failure for `.6`. Both failure cases returned the stable
> installer failure boundary; after the runs, the canonical unit and service
> were active, the exact 48181 liveness endpoint was healthy, and the bounded
> non-secret `active-version` pointer was restored to `.4`. Immutable `.2`
> through `.6` campaign versions remained retained. Focused WSL implementation
> tests passed **19/19** (`linux_bundle_campaign_builder`,
> `linux_bundle_service_lifecycle`, and `linux_bundle_single_service`) and
> strict runtime Clippy passed. This provides Linux-native experimental test
> evidence for the P1-T08 installer transaction only; it is not a production
> release/signing, B01, product Gate, Profile, containment, uninstall, or
> first-conversation claim. P1-T09 remains `not-started`; the next active
> work item is the install-to-first-conversation route.

> **P1-T08 MVP single-service installer transaction (2026-07-29):** P1-T08
> was `in-progress` with `development_track: experimental-local-only` before
> the Linux-native closeout recorded above.
> The inspected shell now verifies a release-bound Rust installer digest and
> hands the complete downloaded bundle to `linux-bundle-installer`. The Rust
> path shares one offline verify → OS lease → private staging prefix, publishes
> immutable bytes, atomically publishes the fixed canonical user unit, runs
> fixed `systemctl --user` daemon-reload/restart actions, checks the exact
> 48181 liveness contract before and after active-pointer publication, and
> deterministically restores the previous pointer/unit/service or removes the
> first-install unit without issuing a receipt. The adapter runner now owns
> fixed bootstrap-fact parsing, release-version verification, and controller
> injection while the production binary still creates only the fixed
> `/usr/bin/systemctl` controller; its positive transaction test uses an
> isolated controller boundary. Focused tests executed in
> `windows_wsl2_linux_guest`: **50 passed, 0 failed, 1 ignored child
> entrypoint**; runtime strict Clippy, formatting, repository consistency, and
> diff whitespace checks passed. This is implementation and fixture evidence
> only. Linux-native user-systemd, release artifact/signing, B01, Gate,
> Profile, containment, uninstall and first-conversation evidence remain
> `not-run` or not provided.

> **Personal MVP-first route decision (2026-07-29):** ADR-0034 records the
> owner-approved first production path: one canonical user service,
> `cognitiveos-personal.service`, on `127.0.0.1:48181`, with bounded downtime
> during explicit Alpha upgrades. ADR-0032/0033 dual-service fixtures remain
> valid implementation-fixture evidence and an optional future upgrade design,
> but no longer block P1-T08/P1-T09. Existing task IDs remain stable;
> `P7-T08 / GMVP-LINUX` is added as a product-only convergence Gate after B01,
> P2 and P7-T01..T03. P1-T08 remains `in-progress`, P1-T09 and P7-T08 remain
> `not-started`, all Personal product Gates remain `not-run`, and Profile
> `implemented` remains 0. This planning decision provides no single-service,
> Linux-native, B01, release, containment or Profile evidence.

> **P1-T08 fake-systemctl controller fixture (2026-07-28):** P1-T08 remains
> `in-progress` with `development_track: experimental-local-only`. ADR-0033
> specifies a private/injected unit-root controller boundary and fixed
> daemon-reload, candidate start/stop, and canonical active restart actions.
> The controller renders and atomically publishes the candidate unit before a
> fixed-argument daemon-reload and candidate start; a focused Unix fake harness
> records the exact action order and confirms candidate isolation from the
> canonical unit. Focused lifecycle tests executed in
> `windows_wsl2_linux_guest`: **10/10 passed**. This is
> implementation-fixture evidence only, not Linux-native systemd, B01, Gate,
> Profile, containment, or release evidence. PR
> [#115](https://github.com/agentkernel/cognitive-os/pull/115) merged as
> `main@aa09f6c`; supported Ubuntu/Windows-MSVC push and pull-request matrices
> passed in runs
> [30382894322](https://github.com/agentkernel/cognitive-os/actions/runs/30382894322)
> and
> [30382932475](https://github.com/agentkernel/cognitive-os/actions/runs/30382932475).
> Pointer/unit/service compensation fault injection, full redaction coverage,
> and a Linux-native campaign remain separate work.

> **P1-T08 rendered user-service foundation (2026-07-28):** P1-T08 remains
> `in-progress` with `development_track: experimental-local-only`. ADR-0032
> fixes two product-owned user-unit identities, disjoint loopback liveness
> ports, staged-versus-active executable paths, and the candidate-stop before
> canonical-active-start ordering. `cognitive-runtime` now renders only fixed
> candidate/active unit content, atomically publishes a fixture unit through a
> private temporary file, and rejects unsafe version/path input. The existing
> service transaction stops a healthy candidate before activation and starts
> then confirms the canonical active service after the pointer changes; failed
> flows retain deterministic compensation and never issue a receipt. Focused
> service lifecycle tests executed in `windows_wsl2_linux_guest`: **9/9
> passed**. PR [#114](https://github.com/agentkernel/cognitive-os/pull/114)
> merged as `main@b151b54` after the initial Windows path-separator failure was
> corrected in `0a90033`; its supported Ubuntu/Windows-MSVC push and
> pull-request CI matrices passed in runs
> [30379506413](https://github.com/agentkernel/cognitive-os/actions/runs/30379506413)
> and
> [30379508772](https://github.com/agentkernel/cognitive-os/actions/runs/30379508772).
> This is implementation-fixture and supported-matrix evidence only, not
> Linux-native systemd, B01, Gate, Profile, containment, or release evidence.
> A production user-systemd installation path, daemon-reload fixture, and
> Linux-native systemd campaign remain separate work.

> **P1-T08 safe-extraction slice (2026-07-28):** P1-T08 remains
> `in-progress` with `development_track: experimental-local-only`. ADR-0031
> specifies a bounded, fixed-layout `tar.gz` extraction boundary. The
> implementation verifies the existing signed artifact before any lease or
> deployment mutation, then re-hashes it under the existing per-root OS lease
> and extracts only into private staging. It rejects unsafe paths, links,
> special entries, non-executable or privileged entry modes, and layouts other
> than `bin/kernel-server`; only a fully validated candidate is atomically
> published as `staged/<version>`. Extraction failure leaves the active pointer
> unchanged and creates no receipt. Focused local tests executed in
> `windows_wsl2_linux_guest`: installation **12/12**, lifecycle **12/12** with
> one ignored child entrypoint, and service lifecycle **6/6**; strict feature
> Clippy, formatting, and consistency also passed. The successful fixture
> layout satisfies static controller preflight only: the checked-in user unit
> remains unrendered and the controller still makes no systemd action. This is
> neither Linux-native systemd, B01, Gate, Profile, containment, nor release
> evidence. PR [#113](https://github.com/agentkernel/cognitive-os/pull/113)
> merged as `main@d57efc1` after both push and pull-request CI matrices passed
> on Ubuntu and Windows/MSVC. That supported-matrix evidence remains distinct
> from Linux-native systemd, B01, Gate, Profile, containment, and release
> evidence. The merge-evidence documentation commit `main@6ee68a2` also
> passed post-merge Ubuntu and Windows/MSVC CI run
> [30367954074](https://github.com/agentkernel/cognitive-os/actions/runs/30367954074).

> **P1-T08 service-lifecycle slice (2026-07-28):** P1-T08 remains
> `in-progress` with `development_track: experimental-local-only`.
> Implementation commit `26bbf12` adds a separate service-aware transaction
> that retains the existing per-root OS lifecycle
> lease across verified staging, candidate controller calls, bounded liveness,
> pointer activation/final confirmation, and deterministic compensation. The
> checked-in systemd user-unit is intentionally unrendered; the production
> controller rejects that template and the absent safe extracted daemon layout
> before any systemd action. `/personal/health` is now a small stable liveness
> response and is explicitly not readiness. Focused fake-controller/loopback
> tests passed **6/6** locally in `windows_wsl2_linux_guest`; this is neither
> real Linux-native systemd evidence nor B01, Gate, Profile, containment, or
> release evidence. PR [#112](https://github.com/agentkernel/cognitive-os/pull/112)
> merged as `main@3fc6faf` after both push and pull-request Ubuntu and
> Windows/MSVC CI matrices passed. Its follow-up merge-evidence commit
> `main@8b51018` also passed post-merge CI run
> [30360532366](https://github.com/agentkernel/cognitive-os/actions/runs/30360532366)
> on Ubuntu and Windows/MSVC. Safe archive extraction/runnable layout, real unit
> rendering, production service campaign, uninstall, signing/release material,
> and all release claims remain absent.

> **P1-T08 inspectable bootstrap/download slice (2026-07-28):** P1-T08 remains
> `in-progress` with `development_track: experimental-local-only`. The new
> `deploy/linux/install.sh` is an inspectable, unrendered source template that
> fails before network access until release rendering binds its fixed version,
> HTTPS object directory, redirect host, verifier SHA-256, public keyring and
> Pi pin. Its bounded `curl --disable` download path uses private temporary
> directories, partial files, one restricted HTTPS redirect, and cleanup traps.
> A digest-authenticated `linux-bundle-verifier` adapter delegates to the
> existing offline Rust verifier only; it does not stage, activate, invoke a
> health callback, start systemd, or create authority state. Focused shell
> behavior tests passed locally in `windows_wsl2_linux_guest`; supported
> Ubuntu and Windows/MSVC push/pull-request CI passed for PR
> [#111](https://github.com/agentkernel/cognitive-os/pull/111), merged as
> `main@35115d3`, and post-merge CI run
> [30350642356](https://github.com/agentkernel/cognitive-os/actions/runs/30350642356)
> also passed. This is not Linux-native evidence. Production keys/releases,
> service health/rollback, uninstall, campaign, B01, Gate, Profile,
> containment, and release claims remain absent.

> **P1-T08 installer lifecycle lease slice (2026-07-28):** P1-T08 remains
> `in-progress` on `lane/personal-p1-t08-installer-lease` with
> `development_track: experimental-local-only`. The official
> `install_linux_bundle` entry point now completes the full offline verifier
> before creating any lease or deployment state, then acquires a stable,
> product-owned OS file lock for the canonical deployment root before opening
> that root. Lock ownership depends only on the live descriptor and OS lock:
> there is no process-local mutex, TTL, owner metadata, or stale-file
> takeover. The fixed lifecycle remains verify -> lease -> deployment open ->
> previous-version read -> verified staging -> exactly one health callback ->
> atomic activation -> active-pointer re-read and confirmation -> non-secret
> receipt. Cross-process and deterministic interruption tests cover same-root
> and cross-version exclusion, different-root independence, normal/error/panic
> and child-termination release, verifier zero mutation, staging/health/
> activation failures, every exposed fault boundary, stale lock contents,
> untorn pointers, activation-completed-without-receipt, and lease-error
> redaction. Local WSL feature tests passed **14/14** with one child entrypoint
> ignored; the complete non-feature runtime surface passed **91/91** with one
> child entrypoint ignored. Strict feature Clippy, formatting, and consistency
> checks passed. PR [#110](https://github.com/agentkernel/cognitive-os/pull/110)
> merged as `main@8aa0031` after push and pull-request workflows passed on
> both supported Ubuntu and Windows/MSVC runners. Local test results remain
> `windows_wsl2_linux_guest` evidence and are not Linux-native evidence. No
> downloader,
> inspected shell installer, systemd service, uninstall, production signing
> key/trust root, release bundle, Linux-native campaign, B01, Gate, Profile,
> containment, or release claim is added.

> **P1-T08 offline attestation verifier merge (2026-07-28):** PR
> [#108](https://github.com/agentkernel/cognitive-os/pull/108) merged as
> `main@afa1d5d` after both push and pull-request Ubuntu/Windows-MSVC CI
> matrices passed. P1-T08 remains `in-progress`. ADR-0028 now
> fixes an offline Ed25519 detached-signature mechanism over an RFC 8785 JCS
> canonical, closed attestation statement. `cognitive-runtime::linux_bundle`
> accepts only an explicitly supplied product-owned versioned keyring; unknown,
> revoked, malformed, duplicate, or bundle-selected trust roots fail closed.
> The signed statement binds product, platform, version, artifact filename and
> digest, the caller-fixed Pi version/integrity, and a strict HTTPS provenance
> reference. Metadata reads are bounded; unsafe, colliding, non-regular, and
> symlink bundle files are rejected; staging re-hashes artifact bytes to reject
> post-verification tampering before candidate creation. Focused WSL tests
> passed **14/14**, the complete `cognitive-runtime` test surface passed, and
> strict runtime Clippy plus formatting passed before the supported CI
> matrices succeeded. No
> production signing key, release attestation, downloader, inspected installer,
> systemd user service, uninstall path, Linux-native campaign, B01, Gate,
> Profile, containment, or release claim exists.

> **P1-T08 first implementation slice (2026-07-27):** P1-T08 is now
> `in-progress` on `lane/personal-p1-t08-bundle-foundation`. The first
> failure-first foundation is a local, non-downloading Linux bundle manifest
> validator plus staged filesystem activation model. It rejects tampered
> artifacts, missing/unsupported attestation references, incorrect Pi pins,
> and vendored Node/Pi payloads; interrupted staging and failed health checks
> retain the prior active version and user data, while a successful check
> atomically replaces the version pointer and retains the prior version. WSL
> focused tests passed. This is local implementation evidence only: no release
> bundle, downloader, systemd user service, trusted attestation verifier,
> Linux-native Gate, B01, Profile, containment, or release claim exists yet.

> **P1-T07 closeout (2026-07-27):** PR
> [#105](https://github.com/agentkernel/cognitive-os/pull/105) merged as
> `main@9d4c3d9` after its Ubuntu and Windows/MSVC CI checks succeeded. The
> task is now **done**: the Pi extension registers exactly one daemon-projected
> model and sends a bounded one-shot `stream:false` completion only through the
> management-authenticated daemon proxy. The extension neither receives
> Provider configuration nor secret material; the daemon proxy remains
> HTTPS-only, redirect-free, bounded, and non-streaming. P1-T07 completion is
> implementation and test evidence only. It is **not** a G0/B01-B12, Profile,
> containment, Linux-native Gate, or release claim. P1-T08 is the next planned
> task; no installer or service claim has been made.

> **每次合并必须更新本页**（`.cursor/rules/02-workflow-docs-sync.mdc`）。计数一律实测（IMP-17），禁止沿用文档旧数。
> 最后更新：2026-07-27（Personal P0-T06 已完成：在 `wuz@192.168.1.2` 上实际执行 `extension-load` probe，证据记录已脱敏并核对为 `extension_command_registered=true`、`session_start_hook_observed=true`、`status_command_observed=true`、`status=executed`、`raw_output_included=false`、`output_redacted=true`、`authority_committed=false`、`effects_created=false`、`task_transitions=0`、`capabilities_granted=0`；仍是 PoC / non-claim evidence，不构成 containment、Profile 或 release claim。P1-T07 已交付 Pi runtime observation 和 daemon-owned non-streaming Provider proxy：`POST /provider/v1/chat/completions` 只接受 management bearer，Provider material 只在 daemon 内解析并仅送至 outbound request；production transport 是 daemon composition root 的 `reqwest` + Rustls，HTTPS-only、no redirects、1 MiB response bound，且 `stream:true` 稳定拒绝。ADR-0027 记录不采用 subprocess 的原因。focused provider test 目前在 Windows GNU linker exit 121 前未能执行；本 WSL instance 无 `cargo`，必须由 CI 验证。当前 pinned Pi API mirror 尚无已验证 completion-provider hook，故 Pi 未接线至 proxy，P1-T07 仍 in-progress。Owner 已批准 ADR-0018 的**默认关闭、本机 Linux、P2 到期**开发例外：adapter 仅在精确显式开关和独立 Provider config 目录存在时，从 native Secret Store 解析已配置的 DeepSeek key 后传给初始 Pi 子进程；不读取 parent env，Windows/CI/无 native backend 一律 fail-closed。该例外不构成 Pi containment、G0/B01-B12/C0/C1/Profile 或 release claim。此前完整 Windows-native 基线验证保持通过；Pi 外部 Agent 的候选执行边界已交付：Pi 0.81.1 + DeepSeek 实际 5/5 无工具 smoke，观测模型 `deepseek-v4-flash`，p50/p95/p99 = 6081/6451/6451 ms；固定 **authority=0 / Effect=0 / uncontained_candidate_only**。Lane-KRN durable InstallationStore 已合入 `main`：SQLite WAL 暂存/提交、显式崩溃恢复和跨句柄原子可见性测试已提供。Lane-RUN 现通过 in-process `DurableInstallationManager` 消费该 store；验证先于 stage/commit，recovery 仅限 manager，会话不授予 capability。`admin-cli install` 现以已认证 management session 的 `principal://` 为唯一 Custom 确认操作者，显示固定风险提示、构建确定性 `file://` bundle、拒绝无 lockfile/浮动依赖、并仅执行 `npm ci --ignore-scripts --offline`；它记录并输出 bundle/lockfile/adapter/sandbox/compatibility digests 后再 durable commit。来源/确认的**耐久查询记录**尚无 KRN store carrier，因此本批不将该 CLI 输出冒充 release evidence。该确认不是上游签名、C0/C1、Profile 或 sandbox 声明；官方供应链 verifier、Linux-native OS sandbox、lifecycle/I/O adapter 与跨进程 lifecycle lease 仍待完成。见 [PI-AGENT-INTEGRATION-PLAN.md](PI-AGENT-INTEGRATION-PLAN.md)。P0-T01 已完成：`01ceb93` 的跨平台 CI 成功，且本机 Windows GNU linker failure 已如实记录为非支持基线；不构成 Personal 产品、G0、B01-B12 或 Profile 声明。）
> **P1-T07 verification correction (2026-07-27):** the preceding status text's
> statement that this WSL instance has no Cargo is obsolete. Cargo is available
> at `/root/.cargo/bin/cargo`; the WSL/Linux focused provider-proxy process test
> and `kernel-server` strict Clippy check passed. This is focused local Linux
> evidence only, not a supported-matrix, Gate, Profile, containment, or release
> claim. Windows GNU linker exit 121 remains a non-supported local limitation,
> and supported CI (including Windows/MSVC) remains required. The unresolved Pi
> completion-provider hook keeps P1-T07 `in-progress`.

> **P1-T07 post-merge update (2026-07-27):** PR [#104](https://github.com/agentkernel/cognitive-os/pull/104)
> merged after Ubuntu and Windows/MSVC CI both passed. In addition to the
> earlier focused WSL checks, `cargo test -p kernel-server --locked` also
> passed in the WSL guest. This does not upgrade the batch into any Gate,
> Profile, containment, or release claim; it only closes the provider-proxy
> implementation checkpoint. P1-T07 still remains `in-progress` because the
> pinned Pi completion/provider integration surface is not yet verified.

> **P1-T07 completion-bridge update (2026-07-27):** the daemon now persists a
> separate non-secret selected-model projection only after a minimally-ready
> discovery probe, clears it on lifecycle invalidation and unavailable probes,
> and exposes it through management-only `GET /provider/v1/selected-model`.
> The Pi adapter registers exactly one daemon-projected model and forwards one
> bounded `stream:false` completion through the authenticated daemon proxy;
> it never receives Provider configuration or secret material. Local focused
> Rust provider/projection and Pi bridge tests, complete `kernel-server` tests,
> strict Clippy, formatting, TypeScript build/tests, and static consistency
> checks passed. This is local implementation and test evidence only: it is not
> a Gate, Profile, containment, or release claim. P1-T07 remains `in-progress`
> pending supported CI and any remaining milestone evidence.

> **P1-T07 integration-surface investigation (2026-07-27):** the exact pinned
> Pi `0.81.1` source commit documents `ExtensionAPI.registerProvider(...)` and
> a complete custom-provider streaming API. This is a supported extension
> surface, not an approved direct-provider bypass. It cannot be wired safely by
> the current batch: the checked-in structural mirror intentionally omits that
> API, the daemon proxy deterministically rejects Pi's required `stream:true`
> requests, and no authenticated, non-secret daemon model projection exists.
> The legacy provider/interception hooks are insufficient because they could
> reintroduce Pi credential/config resolution. P1-T07 remains `in-progress`;
> no Pi-side Provider config, upstream credential, environment-key fallback,
> SQLite write, or direct Provider route was added. P1-T08 installer work is
> dependency-blocked until this completion-path boundary is safely closed.

> 2026-07-24 carrier 批：KRN 已为 existing installation staging/commit record 加入 Custom source acknowledgement evidence，并以同一 SQLite 事务持久化；RUN 的 manager-only query 与 CLI 输出均读取 committed evidence。该批仍须两平台 CI，且不改变官方 provenance、Linux sandbox、Pi adapter、恢复战役、PERF 或 Profile non-claim。
> 2026-07-24 Pi P4 pre-launch admission 批（`lane/run-pi-batch1`）：新增显式 Windows-native/WSL2 拒绝，且 Linux-host admission 必须精确绑定有效 policy、sandbox adapter、compatibility digest、healthy registered adapter 及 HTTPS DeepSeek egress proxy；permit 不携带 authority/capability/Effect/Task completion，仓库未提供可启动 Pi 的 permissive adapter。WSL2 guest 诊断 runtime tests **52/52** + runtime clippy passed；Windows 本机 MinGW linker error 121，未形成 Windows test pass。没有 Linux-native evidence、F-017 扩大声明、Profile claim 或 release GO。

> 2026-07-26 V01 cross-platform evidence repair：POSIX 与 Windows 编排器均尊重
> `CARGO_TARGET_DIR`，使用完整 Rust test path 执行 PERF-004（避免 `--exact` 命中 0
> 个测试仍 exit 0），并复用 conformance runner 生成的完整 schema-shaped builder
> report；两者都验证 release-candidate manifest 的本地 evidence graph，再复制
> `performance-report-v01-sample.json`。repo-tools 新增脚本对齐测试。WSL focused
> evidence：`pi-agent-adapter` **20 substantive tests passed**，PERF-004 exact
> unit **1 passed**，
> conformance runner **85 vectors: 60 pass / 25 not-run**；均为 local/builder
> evidence，不是 Profile、release 或 measured campaign。随后受支持的 WSL/POSIX
> `verify:local` 全流程以独立 Cargo target 完成：**exit 0 / L3 /
> stopped=false / release=non_claim_preserved**；manifest、pins、self-check、F-017
> freeze 与 PERF-004 均 `auto_pass`，`profile_implemented=0`，平台标签保持
> `windows_wsl2_linux_guest`。编排器还解析 PERF exact 日志确认真实
> `1 passed; 0 failed`；该结果仍不构成 Windows-native/Linux-native sandbox、
> Profile、release 或 campaign evidence。
> Pi real-load 预检随后确认当前 guest 为 WSL2，不能使用 Linux-native secret
> exception；adapter 已在选择/probe Secret Service 前显式拒绝 WSL、Windows 与
> enabled CI，更新后的 WSL suite **20 substantive tests passed**。当前 guest 无独立
> `pi` executable，因而未解析 credential、未启动 Pi、未生成 Extension load evidence。

> 2026-07-26 P2 卡扩写批：`plan.md` 的 P2-T01..P2-T08 压缩卡已按 §11.1 扩写为
> 完整强制字段集（范围/依赖/验收语义与任务状态零变更，仅补字段、仓库锚点与
> ADR-0026/0018 等既有决策引用）。该批原停留在工作树且 §15.2 全部 not-run
> （当时会话沙箱无 shell）；已于本日随下述工具链恢复批一并落盘并真实执行验证。
> Owner 待办一次性清单见
> [20260726-personal-p2-cards-expansion-handoff.md](../checkpoints/20260726-personal-p2-cards-expansion-handoff.md)
> §5（沙箱磁盘项已随环境恢复消解；既有：`wuz@192.168.1.2` SSH、Linux-native DeepSeek
> key、干净 Linux VM）。本批非 Gate/Profile/release 声明。

> 2026-07-26 本机 Linux 工具链恢复与工作树落盘批：此前多个窗口把"无 shell /
> Windows GNU linker exit 121"当作本机不可测试的既定条件。本窗口在 WSL2 guest
> 内安装了与 `rust-toolchain.toml` 一致的 **Linux-native Rust 1.97.1**、Node
> 22.14.0 与 pnpm 10.33.2，本机因此首次可以完整执行受支持的测试面。实测结果
> （均为 `windows_wsl2_linux_guest` 本机执行，非 CI、非 Linux-native Gate 证据）：
> `cargo test --workspace --locked` **358 passed / 0 failed（67 个 suite）**、
> `cargo clippy --workspace --all-targets --locked -- -D warnings` 通过、
> `cargo fmt --all -- --check` 通过、`pnpm -r build` 通过、`pnpm -r test` 通过、
> `pnpm run check:consistency` OK（273 REQ / 55 码 / 63 schema / 85 向量）。
> 该批只改变"本机能否执行测试"这一环境事实，**不改变** 任何 Gate、Profile、
> release、G0/B01-B12 结论，也不把 WSL2 结果升级为 Linux-native evidence。

> 2026-07-26 客户端文档域仓库拆分（owner 执行）：`clients/` 整体迁出至独立仓库
> [agentkernel/cognitiveos-clients](https://github.com/agentkernel/cognitiveos-clients)
> （保留 subtree 历史，外仓根对应原 `clients/` 目录）。本仓已删除 155 个
> `clients/**` 文件，所有跨仓引用改为 `blob/main/<path>` URL，并修复由此产生的
> 9 条断链；`docs/clients/`、`docs/platforms/`、`apps/cognitiveos-console/` 兼容
> stub 保留。ADR-0007、CLIENTS-DEC-001 与 2026-07-20 Lane-CON 例外作为历史记录
> 注记保留而非删除。本批不改变任何 Gate、readiness、Profile 或 release 结论。

## 里程碑状态

> 2026-07-26 evidence foundation batch：POSIX V01 pins 已与实测 `85/60/25` 和
> self-check `41` 对齐；conformance runner 现在写出 schema-valid builder sample，
> manifest validator 验证本地 result/report 引用及 digest。该批仍是 builder/sample
> evidence，不是 measured campaign；`verify:local` 现在将 manifest/evidence graph
> 校验作为 L2 必需项；Personal 后续阶段可在本机或隔离环境以
> `experimental-local-only` 开发，不改变产品 Gate、Profile 或 release 状态。

| 里程碑 | 状态 | 出口评审 | 备注 |
|---|---|---|---|
| M0 工程基线与开发体系 | **done** | [20260720-m0-milestone-review.md](../checkpoints/20260720-m0-milestone-review.md) | — |
| M1 合同收敛与 Runner | **done** | [20260720-m1-milestone-review.md](../checkpoints/20260720-m1-milestone-review.md) | CTR 契约批（F-003 收尾、$id 统一、codegen、bundle digest、golden §14）+ CFR runner 批（静态合同执行 25 pass、错误实现自检 fail、F-003 关闭、D-004/D-012 闭合）。**M2 入口 gate 开启；tracer bullet 入口 gate（F-002~F-010 类合同收敛）开启**（M4 入口另需 M2/M3 行为验收） |
| M2 对象/状态/事件内核 | **done** | [20260720-m2-milestone-review.md](../checkpoints/20260720-m2-milestone-review.md) | KRN 内核批（三 crate 实现 + 六判据行为测试，PR #4）+ CFR 行为执行批（runner 行为模式：3 向量对真实 kernel/store 行为执行 pass、只读降级子集落档、gate-bypass 错误实现自检 12/12 fail）。**M3 入口 gate 的 M2 出口分量达成** |
| M3 治理链与 Context | **done** | [20260720-m3-milestone-review.md](../checkpoints/20260720-m3-milestone-review.md) | KRN M3 批（六步授权门、capability 算术、九阶段管线、治理缓存键、确定性渲染、F-007 双竞态，PR #9）+ CFR 行为执行扩展批（8 向量脱 not-run + CTX-TRUST-004 静态→行为升级、治理类自检 20/20 fail）。**M4 入口 gate（tracer bullet；F-002~F-010 类全收敛）逐条核验通过 → 开启**（评审 §7） |
| M4 Intent/Effect 与恢复 + tracer bullet | **done** | [20260720-m4-milestone-review.md](../checkpoints/20260720-m4-milestone-review.md) | KRN M4 批（Intent/幂等/准入矩阵/Effect 协议/sink fencing/恢复八步/faults 框架/tracer bullet，PR #12）+ CFR 行为执行批（7 向量脱 not-run 全经故障注入驱动、fencing 子集落档、反模式自检 27/27 fail、tracer bullet 复现确认）。**F-014/F-023 闭合；F-023 拒绝码 NO_AUTHORIZED_OPERATION_CANDIDATE 确认**。M5 入口 = M4 分量达成 + **F-011 R1 合同登记（剩余项，归 Lane-CTR）** |
| M5 意图链/Harness/Shell/管理面 | **done** | [20260721-m5-milestone-review.md](../checkpoints/20260721-m5-milestone-review.md) | KRN+CTR+RUN 1–2b+TSC+CFR 已合入。行为向量当时 **52 pass / 32 not-run**；F-011 三负例行为闭合；D-018 仍 partially-implemented。**GO M6**（附带条件见评审 §7） |
| M6 安装与适配、v0.1 发布 | **实现已提供；测试已执行（局部）；出口 GO-with-explicit-non-claim** | [20260721-v01-rereview.md](../checkpoints/20260721-v01-rereview.md)（初评 [NO-GO](../checkpoints/20260721-m6-milestone-review.md)） | RUN/CFR M6 交付 + EXIT 声明集/F-017 digests；当前 runner pins **60/25**（85 vectors；self-check 41/41）；RC ≤ experimental；**implemented = 0**；durable install / PERF 战役 / D-018 / Win-native / WSL2 = explicit non-claim；计划：[M6-EXIT-PLAN.md](M6-EXIT-PLAN.md) |
| M7~M11 扩展 Profile | not-started | — | 不阻塞 v0.1 |
| Console 产品车道 | **tracking-only（informative 文档例外）** | — | 客户端项目根迁移完成（ADR-0007）；Phase 0 文档收口；M5 出口已 GO，但 implementation-ready 仍 **no (blocked)**：缺五平台 PoC / 技术栈 ADR / 依赖组 1/2/7 完整交付与法务 gate；与 M6 核心可并行 tracking-only，不混入主线 PR；handoff：`docs/checkpoints/20260721-lane-con-m5-unblock-review-handoff.md` |

## 隔离产品子工程

| 子工程 | 状态 | 测试证据 | 与 Profile 的关系 |
|---|---|---|---|
| `personal-blog/` CognitiveOS Research | **实现已提供；本地测试已执行**（嵌套独立仓；**不入** Cos `origin/main`） | Next.js 静态/SSG；Vitest / Playwright / axe 证据以 **blog 仓** 为准 | 仅研究发布与展示层；不改变 REQ/向量/Profile。**唯一路径** `personal-blog/`；远程 [`agentkernel/blog`](https://github.com/agentkernel/blog)；纪律见 `.cursor/rules/19-personal-blog-boundary.mdc` |
| Personal 产品化计划 | **P1-T08 in-progress；P0-T01..T07 / P1-T01..T07 done；无产品 Gate/Profile 声明** | P1-T07 已交付 daemon-owned Provider proxy 与 Pi completion bridge；P1-T08 已有 verifier、lease、安全解包和 dual-service controller fixture，但 production single-service installer、native systemd、完整 XDG/Provider/Pi 首聊和 B01 均未提供或未执行。ADR-0034 将 single service/48181 定为首个生产路径，新增 P7-T08/GMVP-LINUX；所有 local/WSL/fake/CI 证据继续按其原始范围记账。Personal B01-B12/GMVP-LINUX 仍 `not-run`，Profile implemented 仍为 0。 | 正式台账：[PERSONAL-DEVELOPMENT-PLAN.md](PERSONAL-DEVELOPMENT-PLAN.md)；[PERS-PR trace](personal-trace.yaml) 独立于 registry matrix。Personal task `done` 不代表 product Gate 或 Profile 已符合。 |

## REQ 覆盖计数（实测：`node tools/src/check-consistency.mjs` / `gen-matrix`）

| 口径 | 计数 |
|---|---|
| 规范已登记（specified） | **273**（40 域；errors 55 码；schema **63**；迁移表 5） |
| 实现已提供（构建通过且有实现代码的 REQ） | **70**（matrix 实测非空 impl；shell channel + target resolution 两批各回填 2 条后的当前值） |
| 测试已执行（行为层，runner 真实执行并留证据） | **行为执行 33 向量**（既有 32 + **ORDINARY-CORE-AUDIT-INSPECT-001**）+ workspace Rust 项 + tracer bullet；静态执行 27 向量；**均不构成 Profile 覆盖声明**；TS **85** 项（sdk-ts 72 / agent-shell 13） |
| Profile 已符合（implemented） | 0（样例 manifest 全 `planned`；RC manifest ≤ `experimental`） |

## 向量分层计数（15 层 + 跨切片；实测：conformance runner，2026-07-23 Ordinary Core AUDIT 行为批）

| 状态 | 计数 |
|---|---|
| 向量总数 | **85** |
| **pass** | **60** = 静态 27 + **行为 33**（既有 32 + **ORDINARY-CORE-AUDIT-INSPECT-001**） |
| fail / not-applicable / documented-degradation | 0 / 0 / 0 |
| **not-run** | **25**（含 MGMT-FALLBACK 其余未执行范围、shell migration、delta-scope、store-degradation disk-full 等） |
| 错误实现自检 | **41/41 corrupted 向量全部翻 fail**（新增 audit-before-release / receipt mismatch anti-pattern）；CI 地板 ≥41 |

分层明细见 `artifacts/evidence/conformance/conformance-report.json`（本地再生成：`cargo run -p cognitive-conformance --bin conformance-runner`；报告 sha256 由 runner 打印）。层 7/8 无专属 slug = D-004 已按文档化跨切片映射闭合（conformance/README + runner `CROSS_SLICE_HOSTED`）。

## 开放 finding 计数（权威：[findings-ledger](../traceability/findings-ledger.md)）

| 级别 | 开放 | 条目 |
|---|---|---|
| P0 | 0（+1 证据性质） | F-001（证据缺口，随里程碑消解，不阻断） |
| P1 | **0**（+持续） | F-017 **closed-for-release-claim-set**；F-015 持续。**F-011 已于 CFR M5 行为批闭合**；F-014/F-023 已于 M4 闭合 |
| 漂移 | **0 open**（+3 deferred/design-materialized，+1 decided/partial） | **D-022 v0.2 design/registration blocker**（AUDIT owner-authorized security/audit/compliance review 分量完成但 provenance 受限；SIG independent review、四类 machine registration、OPS member closure 与 CA-0 GO pending；继续阻断 CA-1～CA-8）；**D-017 deferred-to-v0.2**；**D-018 partially-implemented**（组装器 + watch/shell 行为证据已有；治理对象端口仍缺）；**D-016 registration eligibility NO-GO**（八项 blocked；machine contracts 未登记）；D-019 已闭合 |

## 车道当前分工（权威：[PARALLEL-LANES](PARALLEL-LANES.md)）

| 车道 | 状态 | 分支 | 当前任务 |
|---|---|---|---|
| Lane-CTR 契约与生成 | **Ordinary Core AUDIT vector mapping registered in joint batch** | `lane/cfr-ctr-ordinary-core-audit-inspect` | `REQ-AUDIT-001` / `002` both map to `ORDINARY-CORE-AUDIT-INSPECT-001`; matrix is fresh; no schema/candidate semantics changed |
| Lane-CFR 符合性与工具 | **Ordinary Core AUDIT vector test executed** | `lane/cfr-ctr-ordinary-core-audit-inspect` | `ORDINARY-CORE-AUDIT-INSPECT-001` pass via audited public consumer; pins **60/25**; self-check **41/41**; non-Profile claim |
| Lane-KRN 内核主线 | **durable InstallationStore 原子批已合入**（PR #78） | `main` @ `7324227` | SQLite WAL staging→commit、显式 interrupted-staging recovery、跨句柄可见性及不可覆盖负例已提供；不新增 installation transition table（D-020）。Lane-RUN local authority consumption has passed targeted verification; cross-process lifecycle lease remains undecided. |
| Lane-KRN Personal P1-T01 | **XDG layout + dual-DB prepare done（PR #92 CI green）** | `lane/krn-personal-p1-t01-xdg-migrations` | CI run 30155053950 Ubuntu/Windows-MSVC success；不改 registry/schema/vector；非 G0/Profile claim |
| Personal P1-T02 | **Provider config + SecretStore binding done（PR #93 CI green）** | `lane/personal-p1-t02-secret-provider-config` | CI run 30156079691 Ubuntu/Windows-MSVC success；ADR-0020；非 G0/Profile claim |
| Personal P1-T03 | **Provider discovery + capability snapshot done（PR #94 CI green）** | `lane/personal-p1-t03-provider-discovery-probe` | PR #94 / `main@118d20a`；CI runs 30157577277 + 30157576277 Ubuntu/Windows-MSVC success；ADR-0021；非 G0/Profile claim |
| Personal P1-T04 | **bounded daemon + timeout/concurrency done（PR #96 CI green）** | `lane/personal-p1-t04-timeout-concurrency` | PR #96 CI runs 30162481713 + 30162477963 Ubuntu/Windows-MSVC success；ADR-0022；auth/size/timeout/concurrency/restart covered；非 G0/Profile claim |
| Personal P1-T05 | **readiness/status/doctor done（PR #97 CI green）** | `lane/personal-p1-t05-readiness-doctor` | CI runs 30164114878 + 30164113787 Ubuntu/Windows-MSVC success；ADR-0023；blocked/degraded/ready + auth；非 G0/Profile claim |
| Personal P1-T06 | **cognitive CLI done（PR #98 CI green）** | `main` @ `adbb0e5` | CI run 30167503487 Ubuntu/Windows-MSVC success；ADR-0024；非 G0/Profile claim |
| Personal P0-T03 | **License/platform/distribution decision done（PR #99 CI green）** | `main` @ `fd6ff6b` | CI runs 30180002937 + 30179991223 Ubuntu/Windows-MSVC success；ADR-0025；非 G0/Profile claim |
| Personal P0-T06 | **Pi Extension fixture + real local `extension-load` evidence done** | `main` @ `a6c99d6` | The pinned fixture rejects project trust and mutating tools, and its `extension-load` mode drove a real Pi RPC session. On 2026-07-27 the designated Linux-native local experimental host produced a redacted record with registered command/session hook/status command and no authority commit, Effects, Task transitions or capabilities. It remains PoC/non-claim evidence, not a Gate, Profile, containment or release claim. ADR-0018's exact-opt-in, Linux-only local-development secret exception still expires at P2; WSL2/Windows/CI fail closed. |
| Personal P1-T07 | **Pi Extension + readiness observation + daemon Provider proxy batch delivered (in-progress)** | `lane/personal-p1-t07-provider-proxy` | The extension remains default-deny/non-authority; `pi` readiness uses real non-secret runtime observation without changing ADR-0023 aggregation. The daemon route `POST /provider/v1/chat/completions` is management-channel authenticated and resolves Provider material only inside the daemon; the production transport is `reqwest` + Rustls, HTTPS-only, redirect-free, bounded and non-streaming. ADR-0027 records why a subprocess was rejected and why `stream:true` is refused. A synthetic focused service test verifies the credential reaches only daemon-to-transport traffic; route failures are covered. Windows GNU testing is blocked by known linker exit 121 and this WSL instance has no cargo; CI remains required. Pi lacks a verified completion-provider hook, so it is not wired to the proxy and P1-T07 remains in progress. No G0/B01-B12/Profile/containment/release claim. |
| Lane-TSC TS 客户端 | **M5 HTTP/SSE 已交付**（PR #28） | `lane/tsc` | proposal/preview/submit 完整 HTTP 面增量（计划标 P2）；channel isolation 已由 RUN+CFR 补 authority 证据 |
| Lane-RUN 运行时与管理面 | **Pi P4 fail-closed pre-launch admission merged (PR #83)** | `main` @ `937e727` | Custom CLI/durable evidence baseline remains; Pi P4 additionally refuses Windows-native/WSL2 and requires Linux host + valid exact policy/adapter/compatibility digests + healthy registered adapter + HTTPS DeepSeek proxy. No Pi process/authority/Effect/Task completion path exists. WSL2 guest tests 52/52 + clippy pass; Windows local linker blocked; Linux-native evidence, official provenance, lifecycle/I/O adapter and cross-process lease remain pending. |
| Lane-DOC 文档维护 | **ADR-0015 complexity boundary accepted** | `lane/doc-product-complexity-boundary` | Ordinary Core remains the default product range; strict independent AUDIT/SIG/TARGET work is High-Assurance deferred/tracking. This changes priority only, never factual D-016/D-022 or Profile gates. |
| Lane-CON Console | tracking-only（文档域已迁出本仓） | — | 客户端文档域 2026-07-26 迁至独立仓库 [cognitiveos-clients](https://github.com/agentkernel/cognitiveos-clients)；本仓只余 `apps/cognitiveos-console/`、`docs/platforms/`、`docs/clients/` 兼容 stub。M5 GO 后可复评 gate；仍缺 PoC/ADR；implementation-ready blocked |

## 最近 handoff / 评审（最多列 3 条，新的在上）

1. [20260726-personal-p1-t07-pi-extension-package-handoff.md](../checkpoints/20260726-personal-p1-t07-pi-extension-package-handoff.md)（Personal：P1-T07 第一个原子部分 Pi Extension 包；45 TS tests passed；任务仍 in-progress；非 Gate/Profile）
2. [20260726-toolchain-recovery-and-worktree-landing-handoff.md](../checkpoints/20260726-toolchain-recovery-and-worktree-landing-handoff.md)（本机 Linux 工具链恢复、工作树两批落盘、clients 仓库拆分收口；实测 358 Rust tests passed；非 Gate/Profile）
3. [20260726-personal-p2-cards-expansion-handoff.md](../checkpoints/20260726-personal-p2-cards-expansion-handoff.md)（Personal：P2 卡 §11.1 扩写 docs-only 批；已随本日批落盘；含 owner 待办清单；非 Gate/Profile）

## 客户端目录治理交付

> **2026-07-26 仓库拆分：** 客户端文档域已整体迁出至独立仓库
> [agentkernel/cognitiveos-clients](https://github.com/agentkernel/cognitiveos-clients)；
> 本仓不再包含 `clients/` 目录，也不得重建。下表是**迁出前**的交付与结论记录，
> 其后续维护责任归外仓；本仓只保留兼容 stub 与跨仓指针。readiness 结论本身未变。

| 交付 | 状态 | 证据与入口 |
|---|---|---|
| 客户端项目根与 canonical 索引 | **done（informative 文档；结构迁移完成）** | canonical 项目地图迁至 [clients/README.md](https://github.com/agentkernel/cognitiveos-clients/blob/main/README.md)（ADR-0007、CLIENTS-DEC-001）；PC 13 + mobile 4 + Agent Hub 86 + 索引 1 共 104 文件 `git mv`；4 个旧路径兼容 stub（docs/clients、apps console README/PRODUCT-DESIGN、docs/platforms/README）；Console 实现 gate canonical 迁至 [readiness-gates](https://github.com/agentkernel/cognitiveos-clients/blob/main/governance/readiness-gates.md)；未启动任何客户端实现 |
| readiness 结论 | **structure-ready: yes；implementation-ready: no (blocked)** | [clients/READINESS.md](https://github.com/agentkernel/cognitiveos-clients/blob/main/READINESS.md)：PoC runbook/模板与技术栈比较草案已提供（非执行/非 ADR）；M5 出口已 GO，仍 blocked 于依赖组 1/2/7 完整交付、五平台 PoC 执行、技术栈 ADR、AGPL 法务评估（POC-LIC not-run）、Tier 1 runtime PoC |
| 持续维护规则 | **done** | `.cursor/rules/16-client-directory-index.mdc`（canonical 改指 clients/README.md）+ 新增 `.cursor/rules/17-client-project-boundaries.mdc`；专用 consistency 自动校验保持 `planned`（Lane-CFR，checker 不扫 `clients/`），交付前执行 [clients/README.md §9](https://github.com/agentkernel/cognitiveos-clients/blob/main/README.md#9-持续维护与手动-gate) 手动 gate |
| 本轮静态验证 | **pass（非实现/PoC 证据）** | 迁移集成后 `check:consistency` 以 273 REQ / 55 码 / 61 schema / 84 向量为准；clients 专项链接检查仍为手动 gate；[handoff](../checkpoints/20260720-lane-con-clients-root-migration-handoff.md) |
