# CognitiveOS 文档地图

本页登记 `docs/` 与根目录全部文档的**类别**与**更新责任**。类别口径遵循
[docs/standards/normative-source-and-versioning.md](standards/normative-source-and-versioning.md)
的六类 primary class（normative-machine / normative-behavior / normative-test /
informative / implementation-private / historical），并为仓库工作文档扩展四个
本地类别：**plan / adr / checkpoint / prompt**（均属 informative 派生类，不产生
规范要求）。

四类状态用语（规范已登记 / 实现已提供 / 测试已执行 / Profile 已符合）定义见
[conformance/README.md](../conformance/README.md) 与根 [README.md](../README.md)；
所有文档引用状态时必须使用这四个用语之一。

## 根目录

| 文档 | 类别 | 更新责任 |
|---|---|---|
| [CognitiveOS-Architecture.md](../CognitiveOS-Architecture.md) | informative（白皮书 v1.0.1；语义真相在 registry/schema/companion，白皮书随后对齐） | 语义/结构型变更时按 docs-sync-contract 联动 |
| [RFC-0001-...md](../RFC-0001-cognitiveos-governance-context-access.md) | normative-behavior（v0.2 Draft companion） | 契约变更经 Lane-CTR 流程 |
| [CognitiveOS-Review-Conclusions.md](../CognitiveOS-Review-Conclusions.md) | informative（评审处置记录 v2.0） | 冻结；后续处置写 findings-ledger |
| [CognitiveOS-Architecture-Independent-Review.md](../CognitiveOS-Architecture-Independent-Review.md) | informative（独立审查 F-001~F-030） | 冻结；现状核验写 findings-ledger |
| [README.md](../README.md) | informative（四区导航） | 结构变化时同批更新 |
| [AGENTS.md](../AGENTS.md) | informative（代理入口/DoD/会话协议） | 流程变更时更新 |

## specs/ 与 conformance/

| 资产 | 类别 | 更新责任 |
|---|---|---|
| `specs/registry/*.yaml` | normative-machine | 仅修正型变更（冻结期）；同批过 tools 检查 |
| `specs/schemas/*.json` | normative-machine | 同上；遵循 `.cursor/rules/12-schemas-protocol.mdc` |
| `specs/transitions/*.json` | normative-machine（可执行迁移表） | 同上 |
| `specs/*/README.md`（11 份 companion） | normative-behavior | 契约变更联动 |
| `conformance/README.md` | normative-behavior（测试层与状态用语） | Lane-CFR |
| `conformance/vectors/*.json` | normative-test | 禁止迎合实现改写；漂移走台账 |

## docs/standards/（normative-behavior，机器可判定口径）

既有 4 份 + M0 新增 8 份 + 防漂移契约 1 份：

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

## docs/adr/（adr 类；0001/0002/0003/0006 为参考实现决策，非规范要求）

| ADR | 决策 |
|---|---|
| [0001-rust-typescript.md](adr/0001-rust-typescript.md) | Rust stable + Tokio 内核；TypeScript + pnpm 客户端 |
| [0002-sqlite-wal.md](adr/0002-sqlite-wal.md) | SQLite（WAL）为首个事务型对象/事件/Effect 存储 |
| [0003-json-http-sse.md](adr/0003-json-http-sse.md) | 单节点外部 API = HTTP JSON + SSE watch |
| [0004-canonical-json.md](adr/0004-canonical-json.md) | canonical JSON 编码 profile（规范基线） |
| [0005-id-and-clock.md](adr/0005-id-and-clock.md) | UUIDv7 与三时钟域（规范基线） |
| [0006-code-generation-policy.md](adr/0006-code-generation-policy.md) | schema → Rust/TS 代码生成策略（生成物入库、禁手改） |

## docs/plan/（plan 类；每次合并更新 PROGRESS）

- [plan/DEVELOPMENT-PLAN.md](plan/DEVELOPMENT-PLAN.md)：v0.1 定义、M0~M11 里程碑、映射表、风险清单。
- [plan/PROGRESS.md](plan/PROGRESS.md)：单页进度仪表（合并必更）。
- [plan/PARALLEL-LANES.md](plan/PARALLEL-LANES.md)：车道划分、并行规则、所有权表。

## docs/platforms/（informative 产品设计）

- [platforms/README.md](platforms/README.md)：Console 桌面平台入口、激活前文档例外与实现 gate。
- [platforms/macos-product-design.md](platforms/macos-product-design.md)：macOS v1 范围、信任边界、生命周期、要求、PoC 与官方来源。
- [platforms/linux-product-design.md](platforms/linux-product-design.md)：受限 Linux v1 范围、A/B 更新、要求、PoC 与官方来源。
- [platforms/desktop-parity-matrix.md](platforms/desktop-parity-matrix.md)：Windows/macOS/Linux 直接复用、适配、替换和阻断矩阵。
- [platforms/platform-decision-log.md](platforms/platform-decision-log.md)：`CONSOLE-MAC-V1-DEC-*` / `CONSOLE-LNX-V1-DEC-*` 产品决策。

本目录不产生 normative machine requirements，也不表示 Console implementation 已启动。维护受 PARALLEL-LANES 的 Lane-CON informative 文档例外约束。

## docs/traceability/（plan 类，机器可读）

- [traceability/matrix.yaml](traceability/matrix.yaml)：REQ-ID ↔ 实现模块 ↔ 测试 ↔ 证据 ↔ 文档章节矩阵（由 `tools/` 派生骨架 + 人工充实；CI 校验路径真实）。
- [traceability/findings-ledger.md](traceability/findings-ledger.md)：F-001~F-030 与 IMP-01~18 逐条现状台账 + 漂移登记（M1 入口 gate 依据）。

## docs/checkpoints/（checkpoint 类）

命名 `YYYYMMDD-<车道或里程碑>-handoff.md` / `YYYYMMDD-<里程碑>-milestone-review.md`；
模板见 [checkpoints/TEMPLATE.md](checkpoints/TEMPLATE.md)。交接文档是跨会话唯一记忆载体。

## docs/prompts/（prompt 类）

- [prompts/00-bootstrap-dev-system.md](prompts/00-bootstrap-dev-system.md)：M0 引导提示词（已执行）。
- [prompts/01-architecture-audit-and-refactor.md](prompts/01-architecture-audit-and-refactor.md)：**复用资产**——未来任何架构审查/重构场景，粘贴到新会话即可开展反方优先审查。
- [prompts/console-mobile-platform-product-design.md](prompts/console-mobile-platform-product-design.md)：生成 iOS/Android 独立移动产品设计的分阶段研究、决策与交付提示词。
- [prompts/common-prefix.md](prompts/common-prefix.md)：接续提示词公共前缀（硬纪律 + 会话协议）。
- `prompts/lane-*.md`（7 份）与 `prompts/milestone-m1..m6.md`（6 份）：各车道/里程碑自包含接续提示词（任务 F 产出）。

## docs/evaluation/（normative-behavior）

- [evaluation/agent-benefit-benchmark.md](evaluation/agent-benefit-benchmark.md)：REQ-PERF-005 归属文档（四臂对照 + 预注册门槛）。

## 其他

- `apps/cognitiveos-console/PRODUCT-DESIGN.md`：informative 产品设计（状态 planned）；后端依赖台账登记于 DEVELOPMENT-PLAN Console 节。
- `History/`：historical——禁止读取、引用、参与构建与符合性声明。
