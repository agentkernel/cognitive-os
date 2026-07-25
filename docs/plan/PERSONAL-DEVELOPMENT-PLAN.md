# CognitiveOS Personal 产品化开发计划与进度表

> **状态：in-progress（P0-T01..T05、P0-T07、P1-T01..T06 已完成；P0-T06 与其余任务尚未开始）**
> **最后更新：2026-07-26**
> **计划追踪 ID：** `P0-T01` 至 `P7-T06` 是本计划的管理 ID，不是 `specs/registry/` 中的 REQ-ID，也不构成实现、测试或 Profile 符合性声明。
> **详细研究与任务卡草案：** 仓库根目录 `plan.md`；本文件是后续开发的**正式入口和唯一进度台账**。任务 ID 的名称、范围、依赖和阶段 Gate 以本文件为准；`plan.md` 只补充经本文件对齐的研究依据、实施细节与验收方法。
> **可机读追踪：** [personal-trace.yaml](personal-trace.yaml) 将 `PERS-PR`、本计划任务与 Gate/benchmark 对齐；它不是 registry matrix，且不构成 REQ、测试执行或 Profile 符合性声明。

## 1. 使用与更新规则

1. 开始任务前，在本表把对应任务标为 `in-progress`，填入负责人/分支、开始日期和关联 PR（如有）。
2. 一个任务只有在其验收条件满足、相关测试真实执行并留有证据后，才可标为 `done`；未执行的测试必须明确标 `not-run`，不得推断为通过。
3. 每次完成一个开发部分，同一提交必须更新该任务的状态、完成日期、证据链接或命令结果，以及阻塞项；同时按仓库纪律更新 `docs/plan/PROGRESS.md` 和 handoff。
4. 发生范围、依赖、验收或安全边界变化时，先将任务标为 `blocked`，记录原因和决策，再更新详细任务卡与依赖图；不得静默改写完成标准。
5. 允许的状态仅为：`not-started`、`in-progress`、`blocked`、`done`、`cancelled`。`done` 不等于 Profile `implemented`。
6. 如本表与 `plan.md` 的任务卡或依赖图不一致，应先按本表执行并在同一文档修正批中对齐 `plan.md`；不得仅凭详细卡片重新解释或复用既有 `P*-T*` ID。

### 进度汇总

| 阶段 | 任务数 | done | in-progress | blocked | not-started | 阶段 Gate |
|---|---:|---:|---:|---:|---:|---|
| Phase 0 - 基线与决策 | 7 | 6 | 0 | 0 | 1 | G0 |
| Phase 1 - 安装到首次对话 | 9 | 6 | 0 | 0 | 3 | G1 / B01 |
| Phase 2 - 单 Agent 任务闭环 | 8 | 0 | 0 | 0 | 8 | G2 / B02、B04、B05、B12 |
| Phase 3 - Context 与效率 | 6 | 0 | 0 | 0 | 6 | G3 / B03、B06、B07 |
| Phase 4 - Memory | 6 | 0 | 0 | 0 | 6 | G4 / B08 |
| Phase 5 - Agent 与 Tool 生态 | 5 | 0 | 0 | 0 | 5 | G5 / B09、B10 |
| Phase 6 - Multi-Agent | 4 | 0 | 0 | 0 | 4 | G6 / B11 |
| Phase 7 - 产品化与发布 | 6 | 0 | 0 | 0 | 6 | G7 / RC |
| **合计** | **51** | **12** | **0** | **0** | **39** | — |

## 2. 产品边界与不变量

- Rust daemon 是唯一 authority writer；Pi、CLI、Web UI 均为客户端，不可直接写 SQLite 或推进 Task、Effect、Verification 状态。
- Provider API Key 只保存在原生 Secret Store；不得进入配置、SQLite、Pi、环境变量、命令行、日志或证据。
- 所有外部 mutating operation 均须经 Intent/Effect、持久化后派发、幂等键、fencing 和结果 reconcile；外部工具成功不等于 Task 完成。
- Task 完成由独立 verifier/acceptance authority 推进；Pi Session 不等于 Task，Pi `agent_end` 不等于完成。
- Personal 计划不改变既有规范优先级，不得用 `PERS-*` ID 冒充 REQ-ID；合同变化必须走 Lane-CTR 流程。
- 首发产品平台为 **Linux x86_64 + Windows x86_64**（ADR-0025）；首个公开可检查安装包仍为 Linux bundle（P1-T08）。Memory、MCP、Multi-Agent、Web UI 按阶段 Gate 进入，不得提前作为主路径实现。

## 3. 阶段路线图

| 阶段 | 目标 | 入场条件 | 出场条件 | 禁止提前开发 |
|---|---|---|---|---|
| P0 | 冻结平台、架构和安全决策 | 本计划批准 | 工具链、ADR、Secret/Pi PoC、benchmark 规格完成 | 产品功能、Memory、Multi-Agent、UI |
| P1 | 从安装到受治理的首次对话 | G0 | 干净 Linux VM 的 B01 通过 | Task autonomy、Memory、MCP、多 Agent |
| P2 | 单 Agent 可恢复任务闭环 | B01 | B02/B04/B05/B12 通过 | Memory、embedding、多 Agent |
| P3 | Context、Token 与 Loop 效率 | P2 稳定 | B03/B06/B07 通过，指标可采集 | Memory consolidation、多 Agent |
| P4 | 有 provenance 的 durable Memory | P3 基线冻结 | B08 通过，Embedding 有明确 go/no-go | 自动跨工作区记忆 |
| P5 | 可审核的 Agent/Tool 生态 | P4 稳定 | B09/B10 通过 | 自动市场发现 |
| P6 | 有收益证据的 Multi-Agent | 单 Agent benchmark 稳定 | B11 显示收益且无越权/写冲突 | 默认启用多 Agent |
| P7 | 发布、升级、支持与 RC | B01-B12 功能证据齐全 | attested RC、升级/卸载、支持矩阵 | 企业 Console、五平台客户端 |

## 4. 任务进度表

> **填写约定：** “证据/备注”须写测试命令、evidence digest、PR、handoff 或阻塞原因。首次领取时只改当前任务，避免并行会话争用本文件。

### Phase 0 - 基线与决策

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P0-T01 | 固定可复现基线与支持工具链 | — | Linux runner 与支持的 Windows 工具链结论可复现 | done | 2026-07-25；Lane-DOC `lane/personal-p0-t01-baseline-2`。基线 `01ceb93`：CI run [30140381194](https://github.com/agentkernel/cognitive-os/actions/runs/30140381194) 的 Ubuntu/Windows jobs 均成功；本机 `pnpm install --frozen-lockfile`、3 次 `pnpm -r build` + `pnpm -r test`（p50 29.669s）和 `cargo fmt --all -- --check` 通过。本机 Windows GNU `cargo build --workspace --locked` 在 linker exit 121 下失败，LLVM-MinGW/shim 重试同样失败；因此支持组合明确为 CI Linux 与 Windows/MSVC，GNU host 非支持基线。详见 [tests/baseline/README.md](../../tests/baseline/README.md)。 |
| P0-T02 | 冻结 Personal 需求、追踪与架构边界 | P0-T01 | PERS-PR、任务、benchmark 无孤儿映射 | done | 2026-07-25；Lane-DOC `lane/personal-p0-t02-trace`。新增 [personal-trace.yaml](personal-trace.yaml)：20 个 PERS-PR、51 个正式任务和 21 个 Gate/benchmark 均可交叉核对；仅引用真实登记 REQ，product-only 项明确为空映射，所有证据状态为 `not-run`。验证：专用 Node 映射核对、`pnpm run check:consistency`、`git diff --check` 均通过。 |
| P0-T03 | License、首发平台与分发决策 | P0-T02 | owner GO/NO-GO 与 notices 完整 | done | 2026-07-26；PR [#99](https://github.com/agentkernel/cognitive-os/pull/99) 合入 `main@fd6ff6b`。Owner GO：Apache-2.0；首发产品平台 Linux x86_64 + Windows x86_64；GitHub Release 可检查 bundle；**不** vendor Pi/Node；crates.io/npm 仍不发布。交付：根 `LICENSE`/`NOTICE`、[ADR-0025](../adr/0025-personal-license-platform-distribution.md)、[THIRD-PARTY-NOTICES](../legal/THIRD-PARTY-NOTICES.md)、[PERSONAL-SUPPORT-MATRIX](PERSONAL-SUPPORT-MATRIX.md)；workspace `license=Apache-2.0` 且 `publish=false`/`private=true`。验证：`pnpm run check:consistency`、`git diff --check`；CI [30180002937](https://github.com/agentkernel/cognitive-os/actions/runs/30180002937) / [30179991223](https://github.com/agentkernel/cognitive-os/actions/runs/30179991223) Ubuntu/Windows-MSVC SUCCESS。非 G0/B01-B12/Profile；SBOM/attestation 归 P7-T01。handoff：[20260726-personal-p0-t03-license-platform-distribution-handoff.md](../checkpoints/20260726-personal-p0-t03-license-platform-distribution-handoff.md)。 |
| P0-T04 | 数据布局、迁移、备份与回滚设计验证 | P0-T02 | migration dry-run、重放与失败恢复评审 | done | 2026-07-25；Lane-KRN `lane/krn-personal-p0-t04-migrations`，PR #89。ADR-0017 + adapter-local `schema_migrations` dry-run/apply/replay/digest-drift/failure-recovery tests；CI run [30150183941](https://github.com/agentkernel/cognitive-os/actions/runs/30150183941) 在 Ubuntu 与 Windows/MSVC 均通过 `cargo test --workspace --locked`。本机 Windows GNU linker exit 121 是已知非支持环境，不再阻断。未修改 registry/schema/vector 或 authority transition 语义。 |
| P0-T05 | Linux Secret Service PoC | P0-T01 | set/get/rotate/delete 与泄漏负例通过 | done | 2026-07-25；分支 `lane/personal-p0-t05-secret-store-api`，PR [#90](https://github.com/agentkernel/cognitive-os/pull/90)。隔离 crate `cognitive-secret` + ADR-0018：冻结 `SecretStore::{probe,put,get,delete}` 与 opaque `SecretRef`；模拟 put/get/rotate/delete、absent/locked/prompt fail-closed、Debug/Display/env 泄漏负例；Linux native probe-only（mutating D-Bus 归 P1-T02）；无明文 fallback。CI run [30153311857](https://github.com/agentkernel/cognitive-os/actions/runs/30153311857) Ubuntu/Windows-MSVC `cargo test --workspace --locked` 通过（含 `p0_t05_secret_store`）。本机 Windows GNU linker exit 121 为非支持基线。非 G0/B01-B12/Profile 声明。handoff：[20260725-personal-p0-t05-secret-store-api-handoff.md](../checkpoints/20260725-personal-p0-t05-secret-store-api-handoff.md)。 |
| P0-T06 | Pi 版本、Extension 与 RPC 兼容性 PoC | P0-T03 | 固定版本、integrity、Extension/RPC fixture 通过 | not-started | — |
| P0-T07 | daemon transport、认证和威胁模型 | P0-T02 | transport 限制与威胁模型评审完成 | done | 2026-07-25；分支 lane/personal-p0-t07-daemon-transport-threat-model，PR #91 合入 main@ff341ef；CI run 30154100260。ADR-0019 冻结：默认 HTTP/1.1 over UDS（XDG runtime）、可选 loopback TCP、disabled-by-default listener、channel-scoped bearer bootstrap、请求/并发/会话上限；threat model 覆盖 CSRF、DNS rebinding、token theft、channel confusion、replay。不实现业务路由；非 G0/B01-B12/Profile 声明。handoff：[20260725-personal-p0-t07-daemon-transport-threat-model-handoff.md](../checkpoints/20260725-personal-p0-t07-daemon-transport-threat-model-handoff.md)。 |

### Phase 1 - 安装到首次对话

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P1-T01 | 版本化数据库迁移与 XDG 布局 | P0-T04 | upgrade/reapply/corruption/disk-failure 测试 | done | 2026-07-25；Lane-KRN `lane/krn-personal-p1-t01-xdg-migrations`，PR [#92](https://github.com/agentkernel/cognitive-os/pull/92)。`PersonalDataLayout` + `prepare_personal_databases`：XDG 五根、Unix 0700/0600、`migration.lock`、双库 v1 plan（与 open 共享 schema 常量）、state/backups 非覆盖备份。CI run [30155053950](https://github.com/agentkernel/cognitive-os/actions/runs/30155053950) Ubuntu/Windows-MSVC 全绿（含 `p1_t01_layout_migrations` 7 pass、clippy、fmt）。本机 Windows GNU linker exit 121 为非支持基线。未改 registry/schema/vector/transition；非 G0/B01-B12/Profile。handoff：[20260725-lane-krn-personal-p1-t01-handoff.md](../checkpoints/20260725-lane-krn-personal-p1-t01-handoff.md)。 |
| P1-T02 | SecretStore 正式后端与 Provider 配置 | P0-T05, P1-T01 | rotation/restart/redaction negatives 通过 | done | 2026-07-25；分支 `lane/personal-p1-t02-secret-provider-config`，PR [#93](https://github.com/agentkernel/cognitive-os/pull/93)。`cognitive-secret`：ProviderConfig/ProviderKeyService、LinuxSecretToolStore、production selection、hidden-input helper；ADR-0020。CI run [30156079691](https://github.com/agentkernel/cognitive-os/actions/runs/30156079691) Ubuntu/Windows-MSVC 全绿（含 `p1_t02_provider_secret`）。本机 Windows GNU linker exit 121 为非支持基线。非 G0/B01-B12/Profile。handoff：[20260725-personal-p1-t02-secret-provider-config-handoff.md](../checkpoints/20260725-personal-p1-t02-secret-provider-config-handoff.md)。 |
| P1-T03 | Provider、模型发现与能力快照 | P1-T02 | probe 正负例与 model snapshot 通过 | done | 2026-07-25；分支 `lane/personal-p1-t03-provider-discovery-probe`，PR [#94](https://github.com/agentkernel/cognitive-os/pull/94) 合入 `main@118d20a`。`cognitive-secret`：`ProviderTransport`、`ProviderDiscoveryService`、capability snapshot + `fnv1a64` identity digest、`persist_selected_snapshot_digest`；ADR-0021。CI runs [30157577277](https://github.com/agentkernel/cognitive-os/actions/runs/30157577277) / [30157576277](https://github.com/agentkernel/cognitive-os/actions/runs/30157576277) Ubuntu/Windows-MSVC `cargo test --workspace --locked` SUCCESS（含 `p1_t03_provider_discovery`）。本机 Windows GNU linker exit 121 为非支持基线。非 G0/B01-B12/Profile；无真实 Provider Key；无 registry/schema/vector 变更。handoff：[20260725-personal-p1-t03-provider-discovery-handoff.md](../checkpoints/20260725-personal-p1-t03-provider-discovery-handoff.md)。 |
| P1-T04 | 有界 Personal daemon 与本地认证 | P0-T07, P1-T01 | auth/size/timeout/concurrency/restart 测试 | done | 2026-07-25；PR #95（auth/size/host/cookie/restart）+ PR [#96](https://github.com/agentkernel/cognitive-os/pull/96)（timeout/concurrency）。`kernel-server --personal`：loopback、daemon lock、bootstrap secret、channel bearer、header/body 读超时 408、connection/in-flight 429、Host/Cookie fail-closed；ADR-0022。CI runs [30162481713](https://github.com/agentkernel/cognitive-os/actions/runs/30162481713) / [30162477963](https://github.com/agentkernel/cognitive-os/actions/runs/30162477963) Ubuntu/Windows-MSVC SUCCESS（含 timeout/concurrency 单元测试与既有 `p1_t04_personal_daemon`）。本机 Windows GNU linker exit 121 为非支持基线。非 G0/B01-B12/Profile；无 Task/Memory/MCP；无 registry/schema/vector 变更。handoff：[20260725-personal-p1-t04-timeout-concurrency-handoff.md](../checkpoints/20260725-personal-p1-t04-timeout-concurrency-handoff.md)。 |
| P1-T05 | Readiness、status 与 doctor 应用服务 | P1-T03, P1-T04 | blocked/degraded/ready 事实区分 | done | 2026-07-25；分支 `lane/personal-p1-t05-readiness-doctor`，PR [#97](https://github.com/agentkernel/cognitive-os/pull/97)。`kernel-server` Personal composition root：`evaluate_personal_readiness` + management-auth `GET /personal/status|readiness|doctor`；ADR-0023；blocked/degraded/ready 分离；`static_check_is_not_runtime_ready`；secret_ref/bootstrap 不入投影。CI runs [30164114878](https://github.com/agentkernel/cognitive-os/actions/runs/30164114878) / [30164113787](https://github.com/agentkernel/cognitive-os/actions/runs/30164113787) Ubuntu/Windows-MSVC SUCCESS。本机 Windows GNU linker exit 121 为非支持基线。非 G0/B01-B12/Profile。handoff：[20260725-personal-p1-t05-readiness-doctor-handoff.md](../checkpoints/20260725-personal-p1-t05-readiness-doctor-handoff.md)。 |
| P1-T06 | `cognitive init/doctor/status/daemon` | P1-T02, P1-T05 | 重复 init、hidden input、可操作错误 | done | 2026-07-25；分支 `lane/personal-p1-t06-cognitive-cli`，PR [#98](https://github.com/agentkernel/cognitive-os/pull/98) 合入 `main@adbb0e5`。`cognitive` bin + `personal_cli`（init/status/doctor/daemon）、ADR-0024、`tests/p1_t06_cognitive_cli.rs`（live daemon 路径以 Ubuntu 为权威；Windows 跑 init/usage）。CI run [30167503487](https://github.com/agentkernel/cognitive-os/actions/runs/30167503487) Ubuntu/Windows-MSVC SUCCESS。本机 Windows GNU linker exit 121 为非支持基线。非 G0/B01-B12/Profile。handoff：[20260725-personal-p1-t06-cognitive-cli-handoff.md](../checkpoints/20260725-personal-p1-t06-cognitive-cli-handoff.md)。 |
| P1-T07 | CognitiveOS Pi Package/Extension 与 proxy | P0-T06, P1-T03, P1-T04, P1-T05 | 禁用直接 mutating tool；无 key 泄漏 | not-started | — |
| P1-T08 | 可检查 Linux bundle installer 与 user service | P0-T03, P1-T01, P1-T04, P1-T06, P1-T07 | verifier、interruption、rollback 测试 | not-started | — |
| P1-T09 | B01 安装到首次对话 Gate | P1-T08 | 20 次 clean-run；redacted evidence 完整 | not-started | — |

### Phase 2 - 单 Agent 任务闭环

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P2-T01 | TaskApplicationService | P1-T09 | raw intent、preview digest、epoch fencing | not-started | — |
| P2-T02 | 真实 Task API、watch 与自然语言管理映射 | P2-T01, P1-T07 | detach/watch/cancel 的 authority 语义正确 | not-started | — |
| P2-T03 | durable scheduler、lease 与 timer | P2-T01, P1-T01 | crash/duplicate lease/clock/budget 测试 | not-started | — |
| P2-T04 | 单 Agent worker 与 BoundedHarness 接入 | P2-T02, P2-T03 | no-progress/budget/stale-lease 测试 | not-started | — |
| P2-T05 | Tool Registry 与第一个安全 operation | P2-T04 | 未注册、drift、disabled 均 dispatch=0 | not-started | — |
| P2-T06 | Process supervisor 与首个 executor | P2-T05 | dispatch 故障、orphan、redaction、idempotency | not-started | — |
| P2-T07 | Checkpoint、Evidence 与独立 Verifier | P2-T03, P2-T04, P2-T06 | criteria evidence；partial 不得 complete | not-started | — |
| P2-T08 | Phase 2 E2E Gate | P2-T07 | B02/B04/B05/B12 与 false-completion negative | not-started | — |

### Phase 3 - Context、Token 与 Loop 效率

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P3-T01 | Context source/retrieval port | P2-T08 | scope-filter 先于 ranking；revocation 测试 | not-started | — |
| P3-T02 | 最小充分 Context Builder 与预算 | P3-T01 | required fail-closed；loss 显式 | not-started | — |
| P3-T03 | Artifact Store 与 Context externalization | P3-T02 | CAS/partial-write/GC/access 测试 | not-started | — |
| P3-T04 | Loop telemetry、progress 与 strategy controls | P3-T02, P3-T03 | no-progress/repeat/strategy 控制可观测 | not-started | — |
| P3-T05 | Benchmark harness 与性能基线 | P3-T04 | raw run、CI、non-claim 与稳定基线 | not-started | — |
| P3-T06 | Phase 3 E2E Gate | P3-T05 | B03/B06/B07 通过 | not-started | — |

### Phase 4 - Memory

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P4-T01 | Memory schema、repository 与 policy | P3-T06 | scope/provenance/freshness/forgetting 语义 | not-started | — |
| P4-T02 | FTS retrieval 与 authority-first ranking | P4-T01 | unauthorized/stale/conflict 负例通过 | not-started | — |
| P4-T03 | Memory lifecycle、retention 与 privacy | P4-T01 | update/expire/forget/audit 可验证 | not-started | — |
| P4-T04 | Embedding 实验 Gate | P4-T02, P4-T03 | 与 FTS baseline 对比后 go/no-go | not-started | — |
| P4-T05 | Memory API、CLI/Pi projection 与 B08 | P4-T04 | B08、隐私和删除语义通过 | not-started | — |
| P4-T06 | Phase 4 E2E Gate | P4-T05 | memory 回归与 benchmark 证据完整 | not-started | — |

### Phase 5 - Agent 与 Tool 生态

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P5-T01 | Agent package manifest 与安装生命周期 | P4-T05 | digest/health/activate/stop/uninstall | not-started | — |
| P5-T02 | Agent registry 与 instance lifecycle | P5-T01 | capability 默认拒绝、实例隔离 | not-started | — |
| P5-T03 | MCP Tool adapter qualification | P2-T05, P4-T05 | MCP 不成为 authority；drift/timeout 测试 | not-started | — |
| P5-T04 | Tool lifecycle、sandbox 与审计 | P5-T03 | enable/disable/quarantine/reconcile | not-started | — |
| P5-T05 | B09/B10 生态 Gate | P5-T02, P5-T04 | agent/tool 安装与负例 E2E 通过 | not-started | — |

### Phase 6 - Multi-Agent

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P6-T01 | Multi-Agent policy 与 scheduler 扩展 | P5-T05 | 默认关闭、budget/lease/isolation 正确 | not-started | — |
| P6-T02 | Mailbox 与 append-only findings | P6-T01 | 消息不可作为 authority；replay 可处理 | not-started | — |
| P6-T03 | Reviewer/Verifier/Integrator 编排 | P6-T02 | child success 不可绕过 verifier | not-started | — |
| P6-T04 | B11 收益 Gate | P6-T03 | 仅在质量或速度收益达标时启用 | not-started | — |

### Phase 7 - 产品化与发布

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P7-T01 | Release pipeline、SBOM 与 attestation | P5-T05, P0-T03 | 可验证 Linux artifact；无 secret/dev path | not-started | — |
| P7-T02 | Transactional update、rollback 与 uninstall | P7-T01, P1-T01 | stage/health/rollback/uninstall 语义通过 | not-started | — |
| P7-T03 | Doctor、support bundle 与故障排查 | P7-T02 | 仅 redacted facts；stable error code | not-started | — |
| P7-T04 | 完整性能 campaign 与回归地板 | P3-T06, P4-T05, P5-T05 | 固定环境、raw evidence、CI 与阈值 | not-started | — |
| P7-T05 | 非阻塞 Web UI | P2-T08, P7-T03 | 通过 clients gate；只读 daemon projection | not-started | — |
| P7-T06 | RC、文档、支持矩阵与 B01-B12 | P7-T02, P7-T03, P7-T04, P5-T05, P6-T04 | clean VM suite 与 release claim evidence | not-started | — |

## 5. Gate 与证据要求

| Gate | 必需结果 | 最低证据 |
|---|---|---|
| G0 | 基线、决策和 PoC 已完成或明确 NO-GO | ADR、PoC report、支持矩阵、handoff |
| B01 | 安装、初始化、Provider、daemon、Pi 首次响应 | 至少 20 次 clean Linux VM redacted run |
| B02/B04/B05/B12 | 管理、任务、恢复、unknown outcome 闭环 | E2E logs、effect/verification evidence、负例 |
| B03/B06/B07 | Context 正确性与效率控制 | benchmark raw run、预算/loss/telemetry evidence |
| B08 | Memory 生命周期与隐私 | provenance/freshness/conflict/forget 测试 |
| B09/B10 | Agent/Tool/MCP 资格与隔离 | manifest、health、disable/uninstall、negative tests |
| B11 | Multi-Agent 相对单 Agent 有可复现收益 | 相同模型/预算/任务的 baseline 对比 |
| RC | 完整发布声明 | CI、SBOM、attestation、升级/卸载、支持矩阵 |

## 6. 实施时的文档联动清单

- [ ] 更新本文件的任务状态、日期和证据/阻塞说明。
- [ ] 更新 `docs/plan/PROGRESS.md` 的全局状态（不将 planning 任务误报为已实现）。
- [ ] 如有合同、schema、vector 或 generated binding 改动，遵循 Lane-CTR 与 docs-sync 流程。
- [ ] 如有开放风险或漂移，更新 `docs/traceability/findings-ledger.md`。
- [ ] 写入 `docs/checkpoints/` handoff，记录完成项、未完成项、测试、证据、风险与下一步。
- [ ] 在 PR/提交中关联 `PERS-*` 计划 ID，并在适用时关联真实 REQ-ID。
