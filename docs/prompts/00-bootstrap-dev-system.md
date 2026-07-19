# CognitiveOS 开发体系引导与 M0 执行（Bootstrap 提示词 v2 · 基于 1.0.1 基线）

> 用法：将本文件全文粘贴到一个新的 Cursor Agent 对话窗口（工作目录为本仓库根 `agent-kernel`）。本提示词自包含，不依赖任何历史对话；M1 起的开发会话使用本次会话产出的接续提示词。

## 0. 角色、现状与本次会话目标

你是 CognitiveOS 参考实现的首席工程代理。开工前先 `git status`：保护一切已有未提交改动（如 `apps/cognitiveos-console/README.md`），不覆盖、不回退、不擅自把它们混入你的提交。

仓库现状（以你实际盘点为准，下列计数仅供快速定位）：

- `CognitiveOS-Architecture.md` v1.0.1（informative 白皮书，正式版：架构基线定稿 + 证据闭环修订）；
- `CognitiveOS-Review-Conclusions.md`（第一轮评审：设计基线 V1–V17、IMP-01~18）；
- `CognitiveOS-Architecture-Independent-Review.md`（第二轮独立审查：F-001~F-030，含 11 项 P0，总判定"需要架构重构"；v1.0.1 已关闭其中一部分证据缺口，逐条现状由你核验）；
- `RFC-0001-cognitiveos-governance-context-access.md`（v0.2 Draft normative companion）；
- `specs/`：11 份 companion 规范 + `registry/`（requirements/errors/state-domains 三份 YAML）+ `transitions/` 5 份状态迁移表 + `schemas/` 约 56 份 JSON Schema；
- `conformance/`：README（15 个测试层、状态用语）+ 约 74 份声明式向量，明确无 runner；
- `docs/`：`standards/` 4 份、`adr/` 0004/0005、`evaluation/agent-benefit-benchmark.md`（REQ-PERF-005 归属文档）、`prompts/` 既有提示词；
- `apps/cognitiveos-console/`：README + `PRODUCT-DESIGN.md`（跨平台操作台产品设计，状态 planned；§20.3 列出全部后端依赖缺口）；
- 实现代码为零；`History/` 为冻结归档。

本次会话完成里程碑 **M0：工程基线与开发体系**：

- A. 基于当前目录原地搭建实现单仓库骨架（不破坏既有规范资产）；
- B. 建立可长期持续的开发体系（Cursor 规则、AGENTS.md、文档系统、会话交接与可回溯机制）；
- C. 生成完整开发计划与进度表（含并行车道与工作量估算）；
- D. 建立文档联动与防漂移机制（覆盖未来调整/重构场景）；
- E. 建立验收与评测体系（含性能与 Agent 收益门槛）；
- F. 建立面向 Cursor Multitask 的并行开发机制，并产出后续全部会话的接续提示词。

M1 起的功能开发由后续新会话使用你生成的接续提示词继续；本次会话不实现业务逻辑。

## 1. 输入与禁止项

按顺序只读以下内容（够用即止，不必逐字全读）：

1. `CognitiveOS-Architecture.md`：§0–§4（OS 判据、双内核/三平面/七层、§4.7 最小闭环与责任矩阵）、§5–§12（安装、治理链、五状态机、状态/事件、Context、Harness、AKP、Operation/Capability/Shell、§12.12 审批分发）、§16（故障与恢复顺序）、§19–§21（评估、部署形态、路线图与规范表面冻结）；
2. `CognitiveOS-Architecture-Independent-Review.md` 全文：重点 §1 执行摘要（可否进入实现、P0 清单）与逐条 findings；
3. `CognitiveOS-Review-Conclusions.md` §2 设计基线与 IMP-01~18；
4. `RFC-0001`；`specs/core/README.md` 与 `specs/akp/README.md`；其余 companion 浏览结构即可；
5. `specs/registry/*.yaml`、`specs/transitions/*.json`、`specs/schemas/`（细读 common-defs、governed-object-header、object-reference、profile-manifest、effect、performance-report）；
6. `conformance/README.md` + 向量抽样（至少覆盖 eff-crash、management、memory、shell 各组）；
7. `docs/standards/` 4 份、`docs/adr/0004、0005`、`docs/evaluation/agent-benefit-benchmark.md`；
8. `apps/cognitiveos-console/PRODUCT-DESIGN.md` 只读 §17（MVP 与路线图）与 §20.3（后端依赖结论），用于登记 Console 车道依赖清单；本阶段不启动 Console 实现。

禁止项（全程有效）：

- **禁止读取、引用或参考 `History/` 目录下的任何内容**。该目录为冻结历史，不参与构建、schema bundle 与符合性声明。
- 禁止把白皮书 informative 文字、伪 schema、planned 向量当作已登记机器合同；禁止虚构 REQ-ID、错误码、schema 或测试向量。
- 发现白皮书、companion、registry、schema、vector 之间的漂移：先登记到台账，再最小修正闭合并在提交说明注明；不得为迎合实现改写向量或删除负例。

## 2. 全程硬纪律（写入 Cursor 规则并自我遵守）

1. **确定性边界**：概率组件（LLM、检索、排序）只能产生 candidate/proposal；授权、CAS、状态迁移、硬预算、幂等、fencing 与最终提交必须由确定性代码执行。
2. **规范优先级**：digest 固定的机器 schema/registry/transition/vector 与 normative companion > 固定版本 RFC/Core/Profile 文本 > 白皮书 > 实现建议；冲突时采用不扩大权限、数据范围、风险、预算或完成声明的解释。
3. **四类状态用语**：所有文档严格区分"规范已登记 / 实现已提供 / 测试已执行 / Profile 已符合"；`implemented` 仅指全部适用 MUST 有通过证据。
4. **测试先行**：先写失败测试再实现；schema-valid ≠ behavior-pass；完成证明只能来自 authority 状态、Effect、Verification 与 Event，不接受 mock receipt 或模型自述。
5. **规范表面冻结**：v0.1 前不新增对象族、Profile、REQ 域；只允许实现反馈驱动的修正型规范变更（IMP-01）。
6. **P0 门禁**：独立审查中仍开放的 P0 finding 未闭合前，对应子系统不得进入实现里程碑；tracer bullet 入口 = F-002~F-010 类合同缺口全部收敛（审查 §1.2）。
7. **可追溯提交**：每个提交/PR 关联 REQ-ID、F/IMP 条目或文档条目；确无关联时写明原因。

## 3. 任务 A：仓库骨架（原地转型，只增不改）

保持既有 `specs/`、`conformance/`、`docs/` 既有文件、根白皮书/审查/评审/RFC 与 `apps/cognitiveos-console/` **原位不动（只允许新增）**，新增：

```text
Cargo.toml                       # Rust workspace（虚拟清单）
crates/
  cognitive-contracts/           # schema 绑定、canonical JSON/digest、ID/引用、错误类型、golden fixtures
  cognitive-domain/              # 五个执行生命周期状态机 + 纯领域不变量（无任何 I/O 依赖）
  cognitive-store/               # SQLite 仓储、append-only 事件日志、outbox、快照（实现端口 trait）
  cognitive-kernel/              # authority、CAS、capability、预算、Context 门禁、Effect、checkpoint/恢复
  cognitive-runtime/             # Operation 执行器、sandbox/adapter 端口、Harness Loop
  cognitive-management/          # Management API、session/proposal/approval、确定性 fallback
  cognitive-akp/                 # AKP envelope 与 HTTP/SSE 传输 profile
  cognitive-conformance/         # 静态资产校验 + vector runner（执行能力 M1 交付）
apps/kernel-server/              # 单节点组合根（MANAGEMENT_READY/USER_READY/OPERATIONAL 就绪分级）
apps/admin-cli/                  # 确定性管理 CLI（不依赖模型）
apps/agent-shell/                # 任务 Shell（客户端，非 authority；与 apps/cognitiveos-console 并列共存）
packages/contracts-ts/           # TS 侧机器合同（与 Rust 共享 golden fixtures）
packages/sdk-ts/
tests/{golden,e2e,faults,security}/
tools/                           # registry/matrix/链接一致性检查脚本
artifacts/evidence/              # 运行证据目录（产物 gitignore，保留 README 说明）
.github/workflows/ci.yml         # Windows + Linux 矩阵
.gitignore / rust-toolchain.toml / pnpm-workspace.yaml 等工程配置
```

约束：

- 依赖方向固定 contracts → domain → 端口(trait) → 适配器/应用；`cognitive-domain` 与 `cognitive-kernel` 不得依赖 HTTP、SQLite 具体类型或任何模型 SDK。
- 技术基线（写入任务 B3 的 ADR，并注明是**参考实现决策而非 CognitiveOS 规范要求**）：Rust stable + Tokio；SQLite（WAL）为首个事务型对象/事件/Effect 存储；JSON Schema draft 2020-12；TypeScript + pnpm；单节点外部 API 为 HTTP JSON + SSE watch；schema → Rust/TS 代码生成（生成物入库、可复核、禁止手改）。

完成判据：空实现下 `cargo build && cargo test`、`pnpm -r build && pnpm -r test` 通过；根 `README.md` 给出"规范 / 实现 / 测试与证据 / 文档体系"四区导航并明示四类状态用语；既有资产未被移动、重命名或改写。

## 4. 任务 B：开发体系

### B1 Cursor 规则（`.cursor/rules/*.mdc`，共 9 份）

一关注点一文件，每份 ≤50 行，含 frontmatter（`alwaysApply: true` 或明确 `globs`），正文"必须做 / 禁止做 / 完成前检查"三段式，引用真实路径与 REQ-ID 前缀，不复述白皮书：

- `00-architecture-invariants.mdc`（alwaysApply）：观察/提议/授权/执行/验证五分离；五状态机不得合并；ContextView 非 authority；OperationDescriptor ≠ AuthorizationCapability；模型只产 candidate。
- `01-normative-traceability.mdc`（alwaysApply）：实现前先定位 REQ-ID 并固定 schema/vector digest；漂移先报告后闭合；禁止虚构规范资产；四类状态用语。
- `02-workflow-docs-sync.mdc`（alwaysApply）：文档联动义务（任务 D）；会话开始/结束协议（B4）；`docs/plan/PROGRESS.md` 与 findings 台账更新义务。
- `10-rust-kernel.mdc`（globs: `**/*.rs`）：依赖方向；newtype ID；库代码禁 panic/unwrap；状态迁移仅经集中 transition table（消费 `specs/transitions/`）；确定性门禁路径不得调用模型；SQLite 事务边界（状态+事件原子提交）。
- `11-typescript-clients.mdc`（globs: `**/*.{ts,tsx}`）：TS 只消费生成合同；Shell/Console 是客户端非 authority；任务/管理双通道凭据与缓存隔离；展示状态一律来自 authority 投影。
- `12-schemas-protocol.mdc`（globs: `specs/schemas/**`）：draft 2020-12；canonical 编码与 digest（ADR-0004/0005）；未知 critical extension fail-closed；变更必须同步 registry 与向量。
- `13-effect-recovery.mdc`（globs: `crates/cognitive-kernel/**,crates/cognitive-store/**,crates/cognitive-runtime/**`）：稳定幂等键且 timeout 不换键；`OUTCOME_UNKNOWN` 只能经对账分流；同键异参必须 `EFFECT_IDEMPOTENCY_CONFLICT` 拒绝；补偿独立授权；恢复严格按白皮书 §16.6 顺序；三个 crash point 必有测试。
- `14-security-testing.mdc`（globs: `crates/**,packages/**,apps/**,tests/**`）：default deny 且显式 deny 优先；逐对象正文重验先于 ranker/renderer；缓存键绑定治理维度；拒绝响应不泄露存在性；每个安全功能配负例测试。
- `15-conformance-evidence.mdc`（globs: `conformance/**,crates/cognitive-conformance/**,tools/**,tests/**`）：schema-valid ≠ pass；结果五态 pass/fail/not-applicable/degradation/not-run；证据 digest 与 profile manifest 规则；禁止改写向量迎合实现。

### B2 `AGENTS.md`（仓库根）

构建/测试/检查命令速查（Windows PowerShell 与 CI 两种写法）；目录地图；§2 硬纪律摘要；四类状态用语；Definition of Done（任务 E）；新会话接入三步：读 `AGENTS.md` → 读 `docs/plan/PROGRESS.md` → 读最近 `docs/checkpoints/*` 后领取任务。

### B3 文档系统（`docs/`）

- `docs/README.md`：文档地图，每份文档标注类别（normative-machine / normative-behavior / normative-test / informative / plan / adr / checkpoint / prompt）与更新责任，分类口径遵循 `docs/standards/normative-source-and-versioning.md`；将 `docs/prompts/01-architecture-audit-and-refactor.md` 登记为未来架构审查/重构场景的复用提示词。
- 补齐 8 份缺失标准（各 1–3 页，机器可判定口径，引用 registry 中真实 REQ）：`error-contract.md`、`authn-authz-capability.md`、`context-resolution-and-cache.md`、`intent-effect-idempotency.md`、`event-audit-watch.md`、`task-loop-verification.md`、`akp-envelope-and-http-profile.md`、`conformance-evidence.md`。
- 新增 ADR（沿用 0004/0005 格式）：`0001-rust-typescript.md`、`0002-sqlite-wal.md`、`0003-json-http-sse.md`、`0006-code-generation-policy.md`。
- `docs/plan/`：`DEVELOPMENT-PLAN.md`、`PROGRESS.md`、`PARALLEL-LANES.md`（任务 C、F 产出）。
- `docs/checkpoints/`：交接与评审文档目录 + `TEMPLATE.md`（命名 `YYYYMMDD-<车道或里程碑>-handoff.md` / `-milestone-review.md`）。
- `docs/prompts/`：`common-prefix.md`（公共前缀）+ 各里程碑/车道接续提示词（任务 F 产出）。
- `docs/traceability/matrix.yaml`：REQ-ID ↔ 实现模块路径 ↔ 测试 ↔ 证据 ↔ 文档章节的机器可读矩阵（用于定位代码片段与影响面反查）；M0 建骨架与格式说明。
- `docs/traceability/findings-ledger.md`：F-001~F-030 与 IMP-01~18 逐条现状台账（closed-by-1.0.1 / partially-closed / open，附证据链接与阻断的里程碑）——M1 入口 gate 的依据，M0 必须完成首次逐条核验。

### B4 会话接续协议（写入 `02-workflow-docs-sync.mdc` 与 AGENTS.md）

- 会话开始：读 AGENTS.md → `docs/plan/PROGRESS.md` → 最近 handoff → 确认当前车道与任务边界后再动手。
- 会话结束（或上下文接近极限时提前执行）：更新 PROGRESS → 写 handoff（已完成/未完成 REQ、提交哈希、测试与证据状态、未决风险与漂移、下一步入口与建议提示词路径）→ 提交。
- 交接文档是跨会话的唯一记忆载体；禁止依赖对话历史承载工程状态。

## 5. 任务 C：开发计划与进度表

生成 `docs/plan/DEVELOPMENT-PLAN.md`，至少包含：

1. **首版定义** `v0.1 Single-node R0/R1`：单节点 Core 最小闭环（白皮书 §4.7）+ §20.5 R0 最小合法实现形态 + 确定性 Management API/CLI + 任务 Shell + C0/C1 安装适配。**明确不做**：distributed、R2/R3、SMS/CRB、具身、CIM、在线学习、Console 完整产品；Intelligent Management Shell 保持 experimental，不得成为确定性管理/恢复/停止路径的依赖。
2. **里程碑**（每个含：范围、交付物、验收判据【必含安全负例】、依赖、REQ 域、入口 gate、出口评审、两档工作量估算【单人+AI 代理 / 3–5 人小队，标注为假设】）：
   - **M0** 工程基线与开发体系（本次会话）。
   - **M1** 合同收敛与符合性 Runner：按 findings-ledger 闭合仍开放的 P0 机器合同缺口（治理对象双轨、Effect 非法状态组合、Context 启动环与授权字段缺口、撤销竞态重验、持久层 fail-before-effect 等，以逐条核验为准，负例向量先行）；Runner 分层执行全部向量、五态结果输出、加入故意错误实现证明"仅 schema-valid 不能 pass"、未实现层保持 not-run。
   - **M2** 对象/状态/事件内核：GovernedObject、五状态机（消费 `specs/transitions/`）、CAS、事件日志/outbox、预算计量、SQLite 原子提交。验收含：并发 CAS 仅一个成功、非法迁移全拒且状态不变、投影重放 digest 稳定、事件不可原地修改。
   - **M3** 治理链与 Context：TenantContext/Principal/Membership/ActorChain、Conversation/ResourceScope、capability 交集与单调衰减/撤销、确定性九阶段 Context Resolution、缓存键治理绑定、确定性渲染与前缀稳定（IMP-02）。验收含：同租户横向越权、管理员正文读取、撤销后缓存、检索前过滤、跨 Conversation 污染全被拒；required 超预算 fail-closed。
   - **M4** Intent/Effect 与恢复：Intent 持久化、OperationDescriptor/绑定、Effect 状态机、幂等记录、reconcile/verify、checkpoint、恢复八步（§16.6）与故障注入框架。验收含：三个 crash point 覆盖；unknown outcome 不成功不换键；同键异参拒绝；receipt/远端 completed 不完成 Task。**末尾 tracer bullet 端到端竖切**（入口 gate = F-002~F-010 类 P0 全闭合）。
   - **M5** 意图链与 Harness/Shell/管理面：UserIntentRecord→IntentInterpretation（准入）→TaskContract、有界 Loop 与进展/停滞判定、Management API + 确定性 admin CLI、任务 Shell proposal/preview/attach/cancel、snapshot+cursor watch、R1 聊天内结构化确认（IMP-05）。验收含：实质歧义必须澄清、用户修正推进 epoch 并 fence 旧 dispatch、Shell 退出不取消、无模型仍可 inspect/stop/revoke/reconcile。
   - **M6** 安装与适配、v0.1 发布：AgentPackageManifest 验证、安装事务与回滚、沙箱拦截（以 Linux 为参考平台；Windows 开发用 WSL2 或 Linux CI 覆盖负例）、C0/C1 adapter、readiness case、profile manifest、治理开销指标基线（IMP-04/REQ-PERF-004）。
   - **M7+** 各 Profile（独立发布实验 Profile，不阻塞 v0.1）：M7 受治理记忆与认知发现（IMP-03 异步准入 + read-your-write）→ M8 Operation Catalog 与 SMS/CRB → M9 分布式与多 Agent → M10 具身与异构（无独立安全证据前只标 experimental）→ M11 受控学习与知识编译。
   - **Console 产品车道单列**：以 `apps/cognitiveos-console/PRODUCT-DESIGN.md` §17/§20.3 为准逐项登记后端依赖；只有对应机器契约与后端能力交付并过 gate 后才启动相应 Console 里程碑；M0 仅建立依赖追踪，不排期实现。
3. **映射表**：IMP-01~18 与 F-001~F-030 到里程碑的映射（P0 项写入对应里程碑入口/验收判据）。
4. **风险清单**：规范继续膨胀（冻结条款对冲）、schema-valid 冒充 behavior-pass（runner 自检对冲）、治理开销驱赶实践者（IMP-02/04 对冲）、并行冲突（车道规则对冲）、Windows/Linux 沙箱差异（平台分别声明对冲）、后端-Console 期望错位（依赖台账对冲）。

生成 `docs/plan/PROGRESS.md`：单页进度仪表——里程碑状态表、REQ 覆盖计数（specified/实现中/已测试）、向量分层通过计数（15 层）、开放 P0/P1 finding 计数、车道当前分工、最近三次 handoff 链接；写明"每次合并必须更新本页"。

## 6. 任务 D：文档联动与防漂移机制

新增 `docs/standards/docs-sync-contract.md`，固定：

1. **变更三分类**：修正型（typo、漂移修复，不改语义）/ 语义型（改行为、状态机、错误码、schema、验收口径）/ 结构型（重构、新增或删除对象族/Profile/子系统）。
2. **联动义务**：语义型与结构型变更必须在同一 PR/提交批次内联动更新——受影响的白皮书章节与版本说明、companion 规范、`specs/registry/*`、`specs/schemas/*`、`conformance/vectors/*`、实现与测试、`docs/traceability/matrix.yaml` 与 findings-ledger、受影响的产品文档（`apps/cognitiveos-console/PRODUCT-DESIGN.md` 至少加漂移标注）；结构型变更还须新增 ADR 与迁移说明。语义真相落在 registry/schema/companion，白皮书随后对齐。
3. **影响面扫描方法**：以 REQ-ID、错误码、schema `$id`、白皮书锚点全仓 grep + matrix.yaml 反查；扫描结果写入 PR 描述。
4. **一致性由 CI 强制**：孤儿引用、断链、计数漂移直接红灯。

完成判据：在临时分支故意制造一个孤儿 REQ 引用与一处断链，CI 检查能失败并指出位置；验证后回滚该分支。

## 7. 任务 E：验收与评测体系

1. `tools/` 静态一致性检查（接入 CI）：全部 JSON/YAML 可解析；schema 通过 draft 2020-12 元校验且相对 `$ref` 全部可解析；每个 vector 的 `requirement_ids` 与错误码在 registry 中存在（registry↔schema↔vector 双向无孤儿）；`docs/` 与根文档相对链接不断；matrix/台账引用路径真实存在。
2. `crates/cognitive-conformance` runner 骨架：枚举 15 个测试层与全部向量并全部报告 `not-run`（执行与比较能力 M1 交付）；输出机器可读 JSON 报告与人读摘要；生成样例 profile manifest（全部 Profile 标 `planned`）。
3. golden fixtures：`tests/golden/` canonical JSON 正反例；Rust 与 TS 各自实现 canonicalize+digest（遵循 `docs/standards/canonical-encoding-and-digest.md` 与 ADR-0004/0005）；CI 断言两侧 digest 完全一致。
4. `.github/PULL_REQUEST_TEMPLATE.md`：关联 REQ-ID / 状态机影响 / 错误码 / schema 兼容性 / 威胁与负例 / 文档联动清单 / 证据链接。
5. **Definition of Done**（写入 AGENTS.md）：CI 全绿 + 相关向量 pass 或 not-applicable 有据 + 文档联动完成 + PROGRESS 已更新 +（会话结束时）handoff 已写。
6. **里程碑出口评审**：每个里程碑结束产出验收报告（`docs/checkpoints/YYYYMMDD-<里程碑>-milestone-review.md`），逐条对照验收判据给出证据；未通过项列为阻断，不得启动下一里程碑对应实现车道。
7. **指标基线**（写入计划，逐里程碑生效）：向量分层通过率；REQ 覆盖率；安全负例计数；治理开销指标族（REQ-PERF-004：授权/Context/Effect 各阶段延迟 p50/p95/p99、cache-hit preservation、每受治理调用额外持久化写、审批延迟与橡皮图章率、开销占端到端比例，自 M6 起报告，声明 ungoverned 基线）；Agent 收益声明遵循 REQ-PERF-005 与 `docs/evaluation/agent-benefit-benchmark.md`：四臂对照（native / governance-only / optimized / ablation）+ 预注册门槛，非劣化不得报告为性能提升。

## 8. 任务 F：并行开发机制（面向 Cursor Multitask）

生成 `docs/plan/PARALLEL-LANES.md`，固定：

1. **车道划分**（接口先行：`cognitive-contracts`/`contracts-ts` 与 trait 端口冻结后各车道方可分叉）：
   - Lane-CTR 契约与生成：contracts 双端 + golden fixtures + codegen——所有车道的地基，最先完成；
   - Lane-KRN 内核主线：domain → store → kernel（M2–M4）；
   - Lane-CFR 符合性与工具：runner、tools、CI（M1 起持续）；
   - Lane-TSC TS 客户端：sdk-ts、admin-cli、agent-shell（golden 对齐后与 KRN 并行，M5 集成）；
   - Lane-RUN 运行时与管理面：runtime、management、akp、kernel-server（M4 后启动）；
   - Lane-DOC 文档与计划维护：持续，可随各车道 PR 附带；
   - Lane-CON Console 产品：远期占位，仅维护依赖台账，后端 gate 通过后激活。
2. **并行规则**：一个车道 = 一个 git 分支（建议配 `git worktree`）= 一个 Cursor Multitask 代理会话；跨车道接口变更只能经 Lane-CTR 走契约变更流程并在 PROGRESS 通告；两个车道禁止同时修改同一 crate/package；合并顺序 CTR → {KRN, CFR, TSC} → RUN；一律经 PR + CI 门禁合并。
3. **所有权表**：crate/package → 车道 → 当前分支/会话，写入 PARALLEL-LANES.md 并保持更新。
4. **接续提示词**：为每条车道与 M1–M6 每个里程碑生成自包含提示词存入 `docs/prompts/`，内容 = 公共前缀（§2 硬纪律 + B4 会话协议摘要）+ 车道/里程碑范围与禁止越界事项 + 该阶段相关规范路径清单 + 入口 gate 与验收判据 + 指定工作分支。提示词必须可直接粘贴到新 Cursor 窗口独立使用。

## 9. 执行顺序与本次会话完成判据

执行顺序：A → B → C → D → E → F，随后运行全部检查并分批提交。若上下文或时间不足，按 A→B→C 优先完成，并按 B4 协议写 handoff 与剩余任务的接续提示词——这本身就是对接续机制的首次演练。

M0 验收清单（结束前逐项自检并在最终报告中给出结果）：

- [ ] 既有规范资产与 `apps/cognitiveos-console/` 未被移动、重命名或改写；`History/` 未被读取或引用
- [ ] 空实现下 `cargo build && cargo test`、`pnpm -r build && pnpm -r test` 通过
- [ ] CI 在 Windows + Linux 矩阵可运行，含静态一致性检查与跨语言 golden digest 对比
- [ ] `.cursor/rules/` 9 份规则、`AGENTS.md`、`docs/README.md` 就绪且 frontmatter 正确
- [ ] 8 份标准 + 4 份 ADR 补齐，格式与既有 `docs/standards/`、`docs/adr/` 一致
- [ ] `DEVELOPMENT-PLAN.md`、`PROGRESS.md`、`PARALLEL-LANES.md`、`matrix.yaml` 骨架、`findings-ledger.md`（F/IMP 逐条现状）、checkpoints 模板、`docs/prompts/` 全套接续提示词就绪
- [ ] registry↔schema↔vector 双向无孤儿（或漂移清单列明并已用最小修正闭合）
- [ ] runner 骨架对全部向量报告 not-run；样例 profile manifest 全部标 planned
- [ ] 故意注入的孤儿引用/断链能被 CI 捕获（验证后已回滚）
- [ ] 全部工作已分批提交，提交信息引用对应任务/文档条目

最终报告要求：新建/修改文件清单（按任务 A–F 分组）、M0 验收逐项结果、findings-ledger 摘要（开放 P0 及其阻断的里程碑）、发现并闭合（或登记）的规范漂移、遗留风险与待决项（如开源许可证选择）、下一步行动（先启动哪条车道、使用 `docs/prompts/` 下哪份提示词）。

不要中途请求确认；遇到无法自决的取舍（许可证、发布渠道等）记入待决项清单继续推进，不要阻塞。
