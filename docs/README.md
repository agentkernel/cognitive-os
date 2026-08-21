# CognitiveOS 文档地图

本页登记 `docs/` 与根目录全部文档的**类别**与**更新责任**。类别口径遵循
[docs/standards/normative-source-and-versioning.md](standards/normative-source-and-versioning.md)
的六类 primary class（normative-machine / normative-behavior / normative-test /
informative / implementation-private / historical），并为仓库工作文档扩展四个
本地类别：**plan / adr / checkpoint / prompt**（均属 informative 派生类，不产生
规范要求）。

规范资产状态用语（规范已登记 / 实现已提供 / 测试已执行 / Profile 已符合）定义见
[conformance/README.md](../conformance/README.md) 与根 [README.md](../README.md)。
仓库身份与唯一活动项目由
[PROJECT-IDENTITY.md](governance/PROJECT-IDENTITY.md) 定义：CognitiveOS 是架构和合同
参考层，`cognitiveos-personal` 是唯一活动实现项目。
Personal 任务的 task/evidence/Gate/claim 正交状态以
[Development Operating Model](governance/DEVELOPMENT-OPERATING-MODEL.md) 为准；
文档不得用单一状态覆盖这些维度。

## 根目录

根目录 Markdown **必须留在仓库根**，不得迁入 `docs/` 或 `History/`：

- [PROJECT-IDENTITY.md](governance/PROJECT-IDENTITY.md) §2.1 把白皮书、RFC、评审结论登记为架构层入口；
- `docs/governance/project-scope.yaml` 机器钉住白皮书与 RFC 的根路径；
- `tools/src/check-consistency.mjs` 将白皮书、RFC 与两份评审列为根相对 `FROZEN_DOCS`（活文档链接核验跳过其内部历史锚点，但路径本身不可改）；
- RFC 路径是 `docs/traceability/matrix.yaml` 与 conformance 向量的 `owner_spec`；
- `AGENTS.md`、Operating Model 与 handbook 事实源将根 [`plan.md`](../plan.md) 定为研究/任务卡细节源（绝非当前状态）。

过程分析、战役笔记和参赛稿不放根目录。Personal owner 分析归档在 [docs/research/](research/README.md)。`History/` 是禁止引用的冻结归档，不能当作这些根文件的迁入目标。

| 文档 | 类别 | 更新责任 |
|---|---|---|
| [CognitiveOS-Architecture.md](../CognitiveOS-Architecture.md) | informative（白皮书 v1.0.2；语义真相在 registry/schema/companion，白皮书随后对齐） | 语义/结构型变更时按 docs-sync-contract 联动 |
| [RFC-0001-...md](../RFC-0001-cognitiveos-governance-context-access.md) | normative-behavior（v0.2 Draft companion） | 契约变更经 Lane-CTR 流程 |
| [CognitiveOS-Review-Conclusions.md](../CognitiveOS-Review-Conclusions.md) | informative（评审处置记录 v2.0） | 冻结；后续处置写 findings-ledger |
| [CognitiveOS-Architecture-Independent-Review.md](../CognitiveOS-Architecture-Independent-Review.md) | informative（独立审查 F-001~F-030） | 冻结；现状核验写 findings-ledger |
| [README.md](../README.md) | informative（四区导航） | 结构变化时同批更新 |
| [AGENTS.md](../AGENTS.md) | informative（精简代理入口；不复制治理正文） | 项目入口或流程链接变化时更新 |
| [plan.md](../plan.md) | informative（研究/任务卡细节；非当前状态源） | 正式计划结构变化时与 PERSONAL-DEVELOPMENT-PLAN 同批对齐 |
| [llms.txt](../llms.txt) | informative（AI 入口指针） | handbook AI 入口变化时更新 |
| [PROJECT-IDENTITY.md](governance/PROJECT-IDENTITY.md) | governance（仓库架构层与唯一活动项目边界） | 项目身份或默认工作范围变化时更新 |
| [project-scope.yaml](governance/project-scope.yaml) | governance-machine（项目身份机器镜像） | 与 PROJECT-IDENTITY 同批更新 |
| [DEVELOPMENT-OPERATING-MODEL.md](governance/DEVELOPMENT-OPERATING-MODEL.md) | governance（工具无关开发/evidence/Gate/lease 规则） | 治理变更时更新 |

## Personal 产品与组合架构（informative canonical design）

这两组文档拥有稳定设计，但不拥有 task、current status、Gate 或 evidence。

| 入口 | 责任 |
|---|---|
| [product/personal/README.md](product/personal/README.md) | 产品愿景、用户模型、认知资源、Linux 1.0 范围和用户旅程 |
| [architecture/personal/README.md](architecture/personal/README.md) | Personal 分层架构、Pi 双角色、Agent lifecycle、authority/data/recovery |

## specs/ 与 conformance/

| 资产 | 类别 | 更新责任 |
|---|---|---|
| `specs/registry/*.yaml` | normative-machine | 仅修正型变更（冻结期）；同批过 tools 检查 |
| `specs/schemas/*.json` | normative-machine | 同上；遵循 tracked normative standards；`.cursor/rules/` 如存在仅为编辑器适配层 |
| `specs/transitions/*.json` | normative-machine（可执行迁移表） | 同上 |
| `specs/*/README.md`（11 份 companion） | normative-behavior | 契约变更联动 |
| `conformance/README.md` | normative-behavior（测试层与状态用语） | Lane-CFR |
| `conformance/vectors/*.json` | normative-test | 禁止迎合实现改写；漂移走台账 |

## docs/standards/（normative-behavior，机器可判定口径）

本目录登记当前 normative-behavior 与防漂移标准；具体数量由目录和一致性检查决定：

| 标准 | 主题 |
|---|---|
| [canonical-encoding-and-digest.md](standards/canonical-encoding-and-digest.md) | canonical JSON 与 digest/签名前缀 |
| [governed-object-contract.md](standards/governed-object-contract.md) | 受治理对象家族合同 |
| [normative-source-and-versioning.md](standards/normative-source-and-versioning.md) | 规范源分类与版本化 |
| [state-and-transition-contract.md](standards/state-and-transition-contract.md) | 状态域与迁移处理规则 |
| [error-contract.md](standards/error-contract.md) | 错误码使用、重试性与失败语义（M0 新增） |
| [authn-authz-capability.md](standards/authn-authz-capability.md) | 认证/授权/capability 判定顺序（M0 新增） |
| [context-resolution-and-cache.md](standards/context-resolution-and-cache.md) | 九阶段 Context 解析与缓存键治理绑定（M0 新增） |
| [intent-effect-idempotency.md](standards/intent-effect-idempotency.md) | Intent/Effect/幂等/恢复口径（M0 新增） |
| [event-audit-watch.md](standards/event-audit-watch.md) | 事件、审计与 watch 恢复语义（M0 新增） |
| [task-loop-verification.md](standards/task-loop-verification.md) | 任务/Loop/验证与验收判定（M0 新增） |
| [akp-envelope-and-http-profile.md](standards/akp-envelope-and-http-profile.md) | AKP envelope 与 HTTP/SSE 传输 profile（M0 新增） |
| [conformance-evidence.md](standards/conformance-evidence.md) | 符合性结果五态与证据 digest 规则（M0 新增） |
| [docs-sync-contract.md](standards/docs-sync-contract.md) | 文档联动与防漂移契约（M0 新增，任务 D） |

## docs/adr/（adr 类；0001/0002/0003/0006/0007 为参考实现决策，非规范要求）

| ADR | 决策 |
|---|---|
| [0001-rust-typescript.md](adr/0001-rust-typescript.md) | Rust stable + Tokio 内核；TypeScript + pnpm 客户端 |
| [0002-sqlite-wal.md](adr/0002-sqlite-wal.md) | SQLite（WAL）为首个事务型对象/事件/Effect 存储 |
| [0003-json-http-sse.md](adr/0003-json-http-sse.md) | 单节点外部 API = HTTP JSON + SSE watch |
| [0004-canonical-json.md](adr/0004-canonical-json.md) | canonical JSON 编码 profile（规范基线） |
| [0005-id-and-clock.md](adr/0005-id-and-clock.md) | UUIDv7 与三时钟域（规范基线） |
| [0006-code-generation-policy.md](adr/0006-code-generation-policy.md) | schema → Rust/TS 代码生成策略（生成物入库、禁手改） |
| [0007-clients-project-root-and-doc-migration.md](adr/0007-clients-project-root-and-doc-migration.md) | `clients/` 客户端项目根与文档迁移（canonical 地图、4 stub、不移代码 package）；该树已于 2026-07-26 迁出至独立仓库 [cognitiveos-clients](https://github.com/agentkernel/cognitiveos-clients)，本仓不再含该目录 |
| [0017-personal-sqlite-migration-and-recovery.md](adr/0017-personal-sqlite-migration-and-recovery.md) | Personal SQLite 迁移/备份/恢复边界（P0-T04；非 Profile） |
| [0018-personal-secret-store-boundary.md](adr/0018-personal-secret-store-boundary.md) | Personal SecretStore 端口与 fail-closed 后端（P0-T05；非 Profile） |
| [0019-personal-daemon-transport-auth-threat-model.md](adr/0019-personal-daemon-transport-auth-threat-model.md) | Personal daemon UDS/loopback transport、本地认证与威胁模型（P0-T07；非 Profile） |
| [0024-personal-cognitive-cli-product-entry.md](adr/0024-personal-cognitive-cli-product-entry.md) | Personal `cognitive` CLI 产品入口（P1-T06；非 Profile） |
| [0025-personal-license-platform-distribution.md](adr/0025-personal-license-platform-distribution.md) | Personal License / 首发平台 / 分发决策（P0-T03；非 Profile） |
| [0022-personal-bounded-daemon-local-auth.md](adr/0022-personal-bounded-daemon-local-auth.md) | Personal bounded daemon + local auth（P1-T04；非 Profile） |
| [0035-personal-pi-shell-and-managed-agent-role-separation.md](adr/0035-personal-pi-shell-and-managed-agent-role-separation.md) | Pi-hosted Agent Shell 与 managed Pi Agent 角色/身份分离 |
| [0036-personal-linux-1-0-and-official-pi-acquisition.md](adr/0036-personal-linux-1-0-and-official-pi-acquisition.md) | Linux 1.0 范围与固定官方 npm Pi acquisition |

## docs/plan/（plan 类；每次合并更新 PROGRESS）

只有 `PERSONAL-DEVELOPMENT-PLAN.md` 能生成当前产品任务和 Gate。其余 M0-M11、M6、
v0.1 和 Post-v0.1 计划是 CognitiveOS 架构形成过程或验证参考，除非被 Personal 正式
计划明确引用，否则不得作为并行 backlog 领取。

- [plan/DEVELOPMENT-PLAN.md](plan/DEVELOPMENT-PLAN.md)：v0.1 定义、M0~M11 里程碑、映射表、风险清单。
- [plan/M6-PLAN.md](plan/M6-PLAN.md)：M6 安装与适配 / v0.1 发布开发与验收计划。
- [plan/M6-EXIT-PLAN.md](plan/M6-EXIT-PLAN.md)：M6 出口闭合 / v0.1 重评审计划（出口 WP canonical）。
- [plan/V01-AUTO-RUN-VERIFY-PERF-PLAN.md](plan/V01-AUTO-RUN-VERIFY-PERF-PLAN.md)：v0.1 无人值守 Boot→Connect→Verify→Perf 一键编排（L0–L3；默认 non-claim）。
- [plan/V01-PERF-CAMPAIGN-PLAN.md](plan/V01-PERF-CAMPAIGN-PLAN.md)：PERF 战役/收益升格附录（默认不执行；人闸门）。
- [plan/POST-V01-NEXT-PHASE-PLAN.md](plan/POST-V01-NEXT-PHASE-PLAN.md)：Post-v0.1 / Post-L3 下一阶段开发与调试测试任务计划（推荐主战役 `CFR-M5-INTENT-AUTHORITY-SLICE`）。
- [plan/PROGRESS.md](plan/PROGRESS.md)：单页进度仪表（合并必更）。
- [plan/PARALLEL-LANES.md](plan/PARALLEL-LANES.md)：并行车道所有权与合入序。
- [plan/PERSONAL-DEVELOPMENT-PLAN.md](plan/PERSONAL-DEVELOPMENT-PLAN.md)：CognitiveOS Personal 的正式开发任务、状态和阶段 Gate；其可机读 [PERS-PR trace](plan/personal-trace.yaml) 不属于 registry matrix，也不构成 REQ 或执行证据。
- [plan/PI-AGENT-INTEGRATION-PLAN.md](plan/PI-AGENT-INTEGRATION-PLAN.md)：Pi-hosted Shell 与 managed Pi 的双轨实现映射；非 backlog/状态源。
- [Agent Hub Master Development Plan](https://github.com/agentkernel/cognitiveos-clients/blob/main/agent-hub/plan/agent-hub-development-plan.md)：受 gate 阻断，未激活实现车道；已迁至独立仓库 cognitiveos-clients 的 `agent-hub/plan/`（子计划与提示词同树）。
- [plan/PERSONAL-SUPPORT-MATRIX.md](plan/PERSONAL-SUPPORT-MATRIX.md)：Personal 首发支持矩阵（P0-T03 / ADR-0025；非 G0/B01 证据）。
- [plan/PERSONAL-TEST-ENVIRONMENTS.md](plan/PERSONAL-TEST-ENVIRONMENTS.md)：已知开发、fixture、CI 与 campaign 环境的 pins、用途和 claim limits。
- [legal/THIRD-PARTY-NOTICES.md](legal/THIRD-PARTY-NOTICES.md)：第三方 notices 与再分发清单（P0-T03；SBOM 归 P7-T01）。

## docs/clients/（兼容入口）

- [docs/clients/README.md](clients/README.md)：**deprecated 兼容 stub**。唯一 canonical 客户端项目地图/目录索引已迁至独立仓库 [cognitiveos-clients 根 README](https://github.com/agentkernel/cognitiveos-clients/blob/main/README.md)（`CLIENTS-DEC-001`、[ADR-0007](adr/0007-clients-project-root-and-doc-migration.md)；2026-07-26 拆分后本仓只保留本 stub）。
- 持续维护由 [cognitiveos-clients README §9](https://github.com/agentkernel/cognitiveos-clients/blob/main/README.md#9-持续维护与手动-gate) 执行，并入 [docs-sync-contract](standards/docs-sync-contract.md)。

## docs/platforms/（informative 产品设计）

- [platforms/README.md](platforms/README.md)：兼容 stub——平台产品文档已迁至 [clients/](https://github.com/agentkernel/cognitiveos-clients/blob/main/README.md)，Console 实现 gate canonical 已迁至 [clients/governance/readiness-gates.md](https://github.com/agentkernel/cognitiveos-clients/blob/main/governance/readiness-gates.md#console-实现-gate)。
- [platforms/macos-product-design.md](https://github.com/agentkernel/cognitiveos-clients/blob/main/pc/docs/platforms/macos/macos-product-design.md)：macOS v1 范围、信任边界、生命周期、要求、PoC 与官方来源。
- [platforms/linux-product-design.md](https://github.com/agentkernel/cognitiveos-clients/blob/main/pc/docs/platforms/linux/linux-product-design.md)：受限 Linux v1 范围、A/B 更新、要求、PoC 与官方来源。
- [platforms/ios-product-design.md](https://github.com/agentkernel/cognitiveos-clients/blob/main/mobile/ios/docs/ios-product-design.md)：iPhone-only v1 范围、APNs/设备绑定/R1、要求、PoC 与 Apple 来源。
- [platforms/android-product-design.md](https://github.com/agentkernel/cognitiveos-clients/blob/main/mobile/android/docs/android-product-design.md)：列名 Android phone v1 范围、FCM/Keystore/Play、要求、PoC 与 Google 来源。
- [platforms/desktop-parity-matrix.md](https://github.com/agentkernel/cognitiveos-clients/blob/main/pc/docs/platforms/desktop-parity-matrix.md)：Windows/macOS/Linux 直接复用、适配、替换和阻断矩阵。
- [platforms/mobile-parity-matrix.md](https://github.com/agentkernel/cognitiveos-clients/blob/main/mobile/shared/docs/mobile-parity-matrix.md)：Windows/macOS/Linux 到 iPhone/Android 的复用、适配、替换、不提供和阻断矩阵。
- [platforms/platform-decision-log.md](https://github.com/agentkernel/cognitiveos-clients/blob/main/pc/docs/platforms/platform-decision-log.md)：`CONSOLE-MAC-V1-DEC-*` / `CONSOLE-LNX-V1-DEC-*` 产品决策。
- [platforms/mobile-platform-decision-log.md](https://github.com/agentkernel/cognitiveos-clients/blob/main/mobile/shared/docs/mobile-platform-decision-log.md)：`CONSOLE-IOS-V1-DEC-*` / `CONSOLE-AND-V1-DEC-*` canonical 产品决策。
- [Agent Hub 平台 parity](https://github.com/agentkernel/cognitiveos-clients/blob/main/agent-hub/docs/platforms/agent-hub-platform-parity.md)：Agent Hub Direct Takeover 接管能力的跨平台差异（已迁至独立仓库 cognitiveos-clients 的 `agent-hub/docs/platforms/`；canonical 在同仓 `agent-hub/docs/`）。

本目录不产生 normative machine requirements，也不表示 Console implementation 已启动。维护受 PARALLEL-LANES 的 Lane-CON informative 文档例外约束。

## docs/traceability/（plan 类，机器可读）

- [traceability/matrix.yaml](traceability/matrix.yaml)：REQ-ID ↔ 实现模块 ↔ 测试 ↔ 证据 ↔ 文档章节矩阵（由 `tools/` 派生骨架 + 人工充实；CI 校验路径真实）。
- [traceability/findings-ledger.md](traceability/findings-ledger.md)：F-001~F-030 与 IMP-01~18 逐条现状台账 + 漂移登记（M1 入口 gate 依据）。

## docs/checkpoints/（checkpoint 类）

命名 `YYYYMMDD-<车道或里程碑>-handoff.md` / `YYYYMMDD-<里程碑>-milestone-review.md`；
模板见 [checkpoints/TEMPLATE.md](checkpoints/TEMPLATE.md)。handoff 承载跨会话操作连续性，但正式任务台账、PROGRESS current snapshot 与 Gate ledger 优先，历史 handoff 不得覆盖当前状态。

## docs/prompts/（prompt 类）

- [prompts/00-bootstrap-dev-system.md](prompts/00-bootstrap-dev-system.md)：M0 引导提示词（已执行）。
- [prompts/01-architecture-audit-and-refactor.md](prompts/01-architecture-audit-and-refactor.md)：**复用资产**——未来任何架构审查/重构场景，粘贴到新会话即可开展反方优先审查。
- [prompts/console-mobile-platform-product-design.md](prompts/console-mobile-platform-product-design.md)：生成 iOS/Android 独立移动产品设计的分阶段研究、决策与交付提示词。
- [prompts/console-agent-hub-direct-mode-product-design.md](prompts/console-agent-hub-direct-mode-product-design.md)：生成无 CognitiveOS 安全接管与完整治理两种部署下，PC/手机统一管理第三方 Agent 的多代理产品设计和开发任务编排提示词（**已执行**，canonical 见独立仓库 [cognitiveos-clients `agent-hub/docs/`](https://github.com/agentkernel/cognitiveos-clients/tree/main/agent-hub/docs)）。
- Agent Hub 接续提示词（12 宏车道 + 6 Adapter，全部 `blocked`）已迁至 [clients/agent-hub/prompts/](https://github.com/agentkernel/cognitiveos-clients/blob/main/agent-hub/prompts/README.md)。
- [prompts/common-prefix.md](prompts/common-prefix.md)：dated non-executable reference；当前 Personal 会话必须从 `AGENTS.md` 和正式计划启动。
- `prompts/lane-*.md`、`prompts/milestone-m1..m6.md` 与 `prompts/v01-auto-*.md`：历史/复用输入；不得生成当前 Personal task、branch、lease、Gate 或状态。
- [prompts/cfr-m5-intent-authority-slice.md](prompts/cfr-m5-intent-authority-slice.md)：Post-v0.1 推荐主战役执行提示词（Lane-CFR；计划见 [POST-V01-NEXT-PHASE-PLAN.md](plan/POST-V01-NEXT-PHASE-PLAN.md)）。
- [prompts/post-v01-next-phase-planning.md](prompts/post-v01-next-phase-planning.md)：生成下一阶段计划的规划会话入口提示词（已执行）。

## docs/evaluation/（normative-behavior）

- [evaluation/agent-benefit-benchmark.md](evaluation/agent-benefit-benchmark.md)：REQ-PERF-005 归属文档（四臂对照 + 预注册门槛）。

## docs/research/（informative；非计划源）

研究评审与 owner 分析。不创建任务、REQ、Gate、lease 或当前状态。

- [research/20260726-frontier-review-and-environment-perception.md](research/20260726-frontier-review-and-environment-perception.md)：前沿对照与运行环境感知评审
- [research/20260812-personal-next-batch-development-analysis.md](research/20260812-personal-next-batch-development-analysis.md)：Personal 下一批次能力分析
- [research/20260812-personal-optimization-proposal.md](research/20260812-personal-optimization-proposal.md)：Personal 架构优化决策方案

## 其他

- `apps/cognitiveos-console/PRODUCT-DESIGN.md`：informative 产品设计（状态 planned）；后端依赖台账登记于 DEVELOPMENT-PLAN Console 节。
- 独立仓库 [cognitiveos-clients `agent-hub/docs/`](https://github.com/agentkernel/cognitiveos-clients/tree/main/agent-hub/docs)：Agent Hub / 直连接管 canonical 文档（informative，`not-implemented`）；两部署模式、接管层级、Adapter 研究、威胁模型、决策与追踪。
- `History/`：historical——禁止读取、引用、参与构建与符合性声明。
