# CognitiveOS 开发体系引导与 M0 执行（Bootstrap 提示词）

> 用法：将本文件全文粘贴到一个新的 Cursor Agent 对话窗口（工作目录为本仓库根）。本提示词自包含，不依赖任何历史对话。

## 0. 角色、现状与本次会话目标

你是 CognitiveOS 参考实现的首席工程代理，在当前仓库（`agent-kernel`）内工作。仓库现状（以你实际盘点为准）：`CognitiveOS-Architecture.md` v0.8 Draft（informative 总体架构白皮书）、`RFC-0001-cognitiveos-governance-context-access.md`（v0.2 Draft normative companion）、`specs/` 下 11 份 companion 规范 + 约 56 份 JSON Schema + `registry/`（requirements/errors/state-domains）+ `transitions/` 5 张状态迁移表、`conformance/` 下约 65 个声明式测试向量（明确无 runner）、`docs/standards/` 4 份标准与 `docs/adr/` 2 份 ADR（0004 canonical JSON、0005 ID 与时钟）。实现代码为零。

本次会话的目标是完成里程碑 **M0：工程基线与开发体系**，具体包括：

- A. 基于当前目录原地搭建实现单仓库骨架（不破坏既有规范资产）；
- B. 建立可长期持续的开发体系（Cursor 规则、AGENTS.md、文档系统、会话交接与追溯机制）；
- C. 生成完整的开发计划与进度表；
- D. 建立文档联动与防漂移机制；
- E. 建立验收与评测体系；
- F. 建立并行开发机制，并产出后续所有会话使用的接续提示词。

M1 起的功能开发由后续新会话使用你生成的接续提示词继续；本次会话不实现业务逻辑。

## 1. 输入与禁止项

先按顺序只读以下内容（够用即止，不必逐字全读）：

1. `CognitiveOS-Architecture.md`：重点 §0–§4（OS 判据与双内核/三平面/七层）、§4.7（最小可用闭环与责任矩阵）、§6–§12（治理链、五状态机、微内核、状态/事件、Context、Harness、AKP、Operation/Capability/Shell）、§16（故障与恢复顺序）、§19–§21（评估、符合性、部署形态、路线图）；
2. `RFC-0001-cognitiveos-governance-context-access.md`（治理对象语义）；
3. `specs/core/README.md` 与 `specs/akp/README.md`；其余 companion README 浏览结构即可；
4. `specs/registry/requirements.yaml`、`specs/registry/errors.yaml`、`specs/registry/state-domains.yaml`；`specs/transitions/*.json`；
5. `specs/schemas/`（浏览清单，细读 `common-defs`、`governed-object-header`、`object-reference`、`profile-manifest` 等基础 schema）；`conformance/README.md` 与 vectors 抽样若干；
6. `docs/standards/` 现有 4 份标准与 `docs/adr/0004`、`0005`；
7. `CognitiveOS-Review-Conclusions.md`（仓库根目录，informative 评审结论）：其 §2"经验证的设计基线"不可削弱；IMP-01~18 是开发计划的输入，P0 项（IMP-01/02/03/04/05）必须落进对应里程碑验收。

禁止项（全程有效）：

- **禁止读取、引用或参考 `History/` 目录下的任何内容**。该目录为冻结历史，不参与构建、schema bundle 与符合性声明；若其中存在未跟踪文件，仅原样提交以免丢失，不得阅读内容。
- 禁止把白皮书 informative 文字、伪 schema、planned 向量当作已登记机器合同；禁止虚构 REQ-ID、错误码、schema 或测试向量。
- 发现白皮书、companion、registry、schema、vector 之间的漂移时：先记录到漂移清单，用最小修正闭合并在提交说明中注明；不得自行猜测安全语义，不得为迎合实现改写测试向量。

## 2. 全程硬纪律（须同时写入 Cursor 规则并自我遵守）

1. **确定性边界**：概率组件（LLM、检索、排序）只能产生 candidate/proposal；授权、CAS、状态迁移、硬预算、幂等与最终提交必须由确定性代码执行。
2. **规范优先级**：digest 固定的机器 schema 与 registry 条目 > 固定版本的 RFC/Core/Profile 文本 > 白皮书 > 实现建议；冲突时采用不扩大权限、数据范围、风险、预算或完成声明的解释。
3. **四类状态用语**：所有文档、README、manifest 严格区分"规范已登记 / 实现已提供 / 测试已执行 / Profile 已符合"；`implemented` 仅指全部适用 MUST 有通过证据，演示可跑、schema 可验、向量存在均不算。
4. **测试先行**：先写失败测试再实现；schema-valid ≠ behavior-pass；禁止用 mock receipt 或模型自述证明任务完成，测试必须观察 authority 状态、Effect、Verification 与 Event。
5. **规范表面冻结**：v0.1 发布前不新增对象族、Profile、REQ 域；只允许实现反馈驱动的修正型规范变更（对应评审 IMP-01）。
6. **可追溯提交**：每个提交/PR 关联 REQ-ID 或文档条目；确无关联时写明原因。

## 3. 任务 A：仓库骨架（原地转型）

保持既有 `specs/`、`conformance/`、`docs/` 既有文件、根白皮书与 RFC **原位不动（只允许新增）**，新增以下结构：

```text
Cargo.toml                       # Rust workspace（虚拟清单）
crates/
  cognitive-contracts/           # schema 绑定、canonical JSON/digest、ID/引用、错误类型、golden fixtures
  cognitive-domain/              # 五个执行生命周期状态机 + 纯领域不变量（无任何 I/O 依赖）
  cognitive-store/               # SQLite 仓储、append-only 事件日志、outbox、快照（实现 domain/kernel 定义的 trait 端口）
  cognitive-kernel/              # authority、CAS、capability、预算、Context 门禁、Effect、checkpoint/恢复
  cognitive-runtime/             # Operation 执行器、sandbox/adapter 端口、Harness Loop
  cognitive-management/          # Management API、session/proposal/approval、确定性 fallback
  cognitive-akp/                 # AKP envelope 与 HTTP/SSE 传输 profile
  cognitive-conformance/         # 静态资产校验 + vector runner（执行能力 M1 交付）
apps/kernel-server/              # 单节点组合根（MANAGEMENT_READY / USER_READY / OPERATIONAL 就绪分级）
packages/contracts-ts/           # TS 侧机器合同（与 Rust 共享 golden fixtures）
packages/sdk-ts/
apps/admin-cli/                  # 确定性管理 CLI（不依赖模型）
apps/agent-shell/                # 任务 Shell（客户端，非 authority）
tests/{golden,e2e,faults,security}/
tools/                           # registry/matrix/链接一致性检查脚本
artifacts/evidence/              # 运行证据目录（产物 gitignore，保留 README 说明）
.github/workflows/ci.yml         # Windows + Linux 矩阵
.gitignore / rust-toolchain.toml / pnpm-workspace.yaml 等工程配置
```

约束：

- 依赖方向固定为 contracts → domain → 端口(trait) → 适配器/应用；`cognitive-domain` 与 `cognitive-kernel` 不得依赖 HTTP、SQLite 具体类型或任何模型 SDK。
- 技术基线（写入任务 B3 的 ADR，并明确它们是**参考实现决策而非 CognitiveOS 规范要求**）：Rust stable + Tokio；SQLite（WAL）为首个事务型对象/事件/Effect 存储；JSON Schema draft 2020-12 为跨语言形状合同；TypeScript + pnpm；单节点外部 API 为 HTTP JSON + SSE watch；代码生成策略（schema → Rust/TS 类型，生成物入库、可复核、禁止手改）。

完成判据：空实现下 `cargo build && cargo test`、`pnpm -r build && pnpm -r test` 通过；根目录 `README.md` 给出"规范 / 实现 / 测试与证据 / 文档体系"四区导航并明示四类状态用语；既有规范资产未被移动、重命名或改写。

## 4. 任务 B：开发体系

### B1 Cursor 规则（`.cursor/rules/*.mdc`，共 9 份）

一关注点一文件，每份 ≤50 行，含 frontmatter（`alwaysApply: true` 或明确 `globs`），正文用"必须做 / 禁止做 / 完成前检查"三段式，引用仓库真实路径与 REQ-ID 前缀，不复述白皮书：

- `00-architecture-invariants.mdc`（alwaysApply）：观察/提议/授权/执行/验证五分离；AgentExecution/Task/Loop/Effect/Verification 五状态机不得合并；ContextView 非 authority；描述 ≠ 权限（OperationDescriptor ≠ AuthorizationCapability）；模型只产 candidate。
- `01-normative-traceability.mdc`（alwaysApply）：实现前先定位 REQ-ID 并固定 schema/vector digest；漂移先报告后闭合；禁止虚构规范资产；四类状态用语。
- `02-workflow-docs-sync.mdc`（alwaysApply）：文档联动义务（见任务 D）；会话开始/结束协议（见 B4）；`docs/plan/PROGRESS.md` 更新义务。
- `10-rust-kernel.mdc`（globs: `**/*.rs`）：依赖方向；newtype ID；库代码禁 panic/unwrap；状态迁移仅经集中 transition table（消费 `specs/transitions/`），禁止散落 if/else 状态判断；确定性门禁代码路径不得调用模型；SQLite 事务边界（状态+事件原子提交）。
- `11-typescript-clients.mdc`（globs: `**/*.{ts,tsx}`）：TS 只消费生成合同；Shell 是客户端非 authority；任务/管理双通道凭据与缓存隔离；展示状态一律来自 authority 投影而非模型文本。
- `12-schemas-protocol.mdc`（globs: `specs/schemas/**`）：draft 2020-12；canonical 编码与 digest（遵循 ADR-0004/0005）；未知 critical extension fail-closed；兼容性与 `scope_domain`；变更必须同步 registry 与向量。
- `13-effect-recovery.mdc`（globs: `crates/cognitive-kernel/**,crates/cognitive-store/**,crates/cognitive-runtime/**`）：稳定幂等键且 timeout 不换键；`OUTCOME_UNKNOWN` 只能经对账分流，禁止盲重试或静默成功；补偿独立授权；恢复严格按 §16.6 八步顺序；三个 crash point 必有测试。
- `14-security-testing.mdc`（globs: `crates/**,packages/**,apps/**,tests/**`）：default deny 且显式 deny 优先；逐对象正文重验（授权先于 ranker/renderer）；缓存键绑定治理维度；拒绝响应不泄露存在性；每个安全功能必须配负例测试。
- `15-conformance-evidence.mdc`（globs: `conformance/**,crates/cognitive-conformance/**,tools/**,tests/**`）：schema-valid ≠ pass；结果五态 pass/fail/not-applicable/degradation/not-run；证据 digest 与 profile manifest 规则；禁止改写向量迎合实现。

### B2 `AGENTS.md`（仓库根）

包含：构建/测试/检查命令速查（本机 Windows PowerShell 与 CI 两种写法）；目录地图；§2 硬纪律摘要；四类状态用语；Definition of Done（见任务 E）；新会话接入三步——读 `AGENTS.md` → 读 `docs/plan/PROGRESS.md` → 读最近 `docs/checkpoints/*` 后领取任务。

### B3 文档系统（`docs/`）

- `docs/README.md`：文档地图，标注每份文档类别（normative-machine / normative-behavior / normative-test / informative / plan / adr / checkpoint / prompt）与更新责任，分类口径遵循 `docs/standards/normative-source-and-versioning.md`。
- 补齐 8 份缺失标准（各 1–3 页，给出机器可判定口径，引用 registry 中真实 REQ，不复述白皮书）：`error-contract.md`、`authn-authz-capability.md`、`context-resolution-and-cache.md`、`intent-effect-idempotency.md`、`event-audit-watch.md`、`task-loop-verification.md`、`akp-envelope-and-http-profile.md`、`conformance-evidence.md`。
- 新增 ADR（沿用 0004/0005 的格式）：`0001-rust-typescript.md`、`0002-sqlite-wal.md`、`0003-json-http-sse.md`、`0006-code-generation-policy.md`。
- `docs/plan/`：`DEVELOPMENT-PLAN.md`、`PROGRESS.md`、`PARALLEL-LANES.md`（任务 C、F 产出）。
- `docs/checkpoints/`：交接文档目录 + `TEMPLATE.md`（命名 `YYYYMMDD-<车道或里程碑>-handoff.md`）。
- `docs/prompts/`：`common-prefix.md`（公共前缀）+ 各里程碑/车道接续提示词（任务 F 产出）。
- `docs/traceability/matrix.yaml`：REQ-ID ↔ 实现模块路径 ↔ 测试 ↔ 证据 ↔ 文档章节的机器可读矩阵；M0 建骨架与格式说明，随里程碑填充。

### B4 会话接续协议（写入 `02-workflow-docs-sync.mdc` 与 AGENTS.md）

- 会话开始：读 AGENTS.md → `docs/plan/PROGRESS.md` → 最近 handoff → 确认当前车道与任务边界后再动手。
- 会话结束（或上下文接近极限时提前执行）：更新 PROGRESS → 写 handoff（已完成/未完成 REQ、提交哈希、测试与证据状态、未决风险与漂移、下一步入口与建议使用的提示词路径）→ 提交。
- 交接文档是跨会话的唯一记忆载体；禁止依赖对话历史承载工程状态。

## 5. 任务 C：开发计划与进度表

生成 `docs/plan/DEVELOPMENT-PLAN.md`，至少包含：

1. **首版定义** `v0.1 Single-node R0/R1`：单节点 Core 最小闭环（白皮书 §4.7：意图固定、TaskContract、持久执行身份、状态/事件、Context 门禁、Operation/Capability、Intent/Effect、验证/验收、watch 与恢复）+ 确定性 Management API/CLI + 任务 Shell + C0/C1 安装适配。**明确不做清单**：distributed、R2/R3、SMS/CRB、具身、CIM、在线学习；自然语言 Intelligent Management Shell 保持 disabled/experimental，不得成为确定性管理、恢复或停止路径的依赖。
2. **里程碑骨架**（每个展开为：范围、交付物、验收判据含安全负例、依赖、涉及 REQ 域、两档工作量估算——"单人+AI 代理"与"3–5 人小队"，估算须标注为假设）：
   - **M0** 工程基线与开发体系（本次会话）。
   - **M1** 符合性 Runner：分层启用全部向量、五态结果输出、加入故意错误实现证明"仅 schema-valid 不能 pass"、未实现层保持 not-run。
   - **M2** 对象/状态/事件内核：GovernedObject、五状态机（消费 `specs/transitions/`）、CAS、事件日志/outbox、预算计量、SQLite 原子提交。验收含：并发 CAS 仅一个成功、非法迁移全拒且状态不变、投影重放 digest 稳定、事件不可原地修改。
   - **M3** 治理链与 Context：TenantContext/Principal/Membership/ActorChain、Conversation/ResourceScope、capability 交集与单调衰减/撤销、确定性九阶段 Context Resolution、缓存键治理绑定、**确定性渲染与前缀稳定（IMP-02）**。验收含：同租户横向越权、管理员正文读取、撤销后缓存、检索前过滤、跨 Conversation 污染全被拒；required 超预算 fail-closed。
   - **M4** Intent/Effect 与恢复：Intent 持久化、OperationDescriptor/绑定、Effect 状态机、幂等记录、reconcile/verify、checkpoint、恢复八步（§16.6）与故障注入框架。验收含：三个 crash point（Intent 持久后/调用前、执行后/receipt 前、验证后/commit 前）覆盖；unknown outcome 不成功不换键；同键异参拒绝；receipt/远端 completed 不完成 Task。**M4 末尾追加端到端竖切（tracer bullet）**：以硬编码 Operation 走通 Task→Intent→Effect→Verify→Commit→Event 全链，提前暴露跨层集成风险。
   - **M5** 意图链与 Harness/Shell/管理面：UserIntentRecord→IntentInterpretation（准入）→TaskContract、有界 Loop 与进展/停滞判定、Management API + 确定性 admin CLI、任务 Shell proposal/preview/attach/cancel、snapshot+cursor watch、**审批分级确认（IMP-05：R1 使用聊天内结构化 proposal digest 短码确认，不依赖自然语言解析）**。验收含：实质歧义必须澄清、用户修正推进 epoch 并 fence 旧 dispatch、Shell 退出不取消、无模型仍可 inspect/stop/revoke/reconcile。
   - **M6** 安装与适配、v0.1 发布：AgentPackageManifest 验证、安装事务与回滚、沙箱拦截、C0/C1 adapter（C2 视进度纳入或后移）、readiness case、profile manifest、**治理开销指标基线（IMP-04）**。注意：沙箱不可绕过声明以 Linux 为参考平台，本机 Windows 开发用 WSL2 或 Linux CI 覆盖对应负例。
   - **M7+** 后续 Profile（各自独立发布实验 Profile，不阻塞 v0.1，对应白皮书 §21 Phase 2–7）：M7 受治理记忆与认知发现（含 **IMP-03 异步准入 + read-your-write**）→ M8 Operation Catalog 与 SMS/CRB → M9 分布式与多 Agent → M10 具身与异构（独立安全项目，无独立安全证据前只标 experimental，禁止宣称 R3 implemented）→ M11 受控学习与知识编译。
3. **IMP 映射表**：IMP-01~18 与里程碑的对应关系，P0 项写入对应里程碑验收判据。
4. **风险清单**：规范继续膨胀（冻结条款对冲）、schema-valid 冒充 behavior-pass（runner 自检对冲）、治理开销驱赶实践者（IMP-02/04 对冲）、并行冲突（车道规则对冲）、Windows/Linux 沙箱差异（平台分别声明对冲）。

生成 `docs/plan/PROGRESS.md`：单页进度仪表——里程碑状态表、REQ 覆盖计数（specified/实现中/已测试）、向量分层通过计数（15 层）、车道当前分工、最近三次 handoff 链接；并写明"每次合并必须更新本页"。

## 6. 任务 D：文档联动与防漂移机制

新增 `docs/standards/docs-sync-contract.md`，固定：

1. **变更三分类**：修正型（typo、漂移修复，不改语义）/ 语义型（改行为、状态机、错误码、schema、验收口径）/ 结构型（重构、新增或删除对象族/Profile/子系统）。
2. **联动义务**：语义型与结构型变更必须在同一 PR/提交批次内联动更新——受影响的白皮书章节与版本说明、companion 规范、`specs/registry/*`、`specs/schemas/*`、`conformance/vectors/*`、实现与测试、`docs/traceability/matrix.yaml`；结构型变更还须新增 ADR 与迁移说明。白皮书（informative）与规范资产（normative）的修改责任分开陈述：语义真相落在 registry/schema/companion，白皮书随后对齐。
3. **影响面扫描方法**：以 REQ-ID、错误码、schema `$id`、白皮书锚点（如 `state-protocol`、`capability-protocol`）全仓 grep + matrix.yaml 反查；扫描结果写入 PR 描述。
4. **一致性由 CI 强制**（工具见任务 E）：孤儿引用、断链、计数漂移直接红灯。

完成判据：在临时分支故意制造一个孤儿 REQ 引用与一处断链，CI 检查能失败并指出位置；验证后回滚该分支。

## 7. 任务 E：验收与评测体系

1. `tools/` 静态一致性检查（接入 CI）：全部 JSON/YAML 可解析；schema 通过 draft 2020-12 元校验且相对 `$ref` 全部可解析；每个 vector 的 `requirement_ids` 与错误码在 registry 中存在（registry↔schema↔vector 双向无孤儿）；`docs/` 与根文档相对链接不断；matrix.yaml 中引用的路径真实存在。
2. `crates/cognitive-conformance` runner 骨架：能枚举 15 个测试层与全部向量并全部报告 `not-run`（执行与比较能力 M1 交付）；输出机器可读 JSON 报告与人读摘要；生成样例 profile manifest（全部 Profile 标 `planned`）。
3. golden fixtures：`tests/golden/` 放 canonical JSON 正反例；Rust 与 TS 各自实现 canonicalize+digest（遵循 `docs/standards/canonical-encoding-and-digest.md` 与 ADR-0004/0005）；CI 断言两侧 digest 完全一致。
4. `.github/PULL_REQUEST_TEMPLATE.md`：关联 REQ-ID / 状态机影响 / 错误码 / schema 兼容性 / 威胁与负例 / 文档联动清单 / 证据链接。
5. **Definition of Done**（写入 AGENTS.md）：CI 全绿 + 相关向量 pass 或 not-applicable 有据 + 文档联动完成 + PROGRESS 已更新 + （会话结束时）handoff 已写。
6. **指标基线**（写入计划，逐里程碑生效）：向量分层通过率；REQ 覆盖率；安全负例计数；治理开销指标族（门禁延迟 p50/p95/p99、供应商缓存保持率、每受治理调用额外持久化写次数/字节，自 M6 起报告，统计口径遵循白皮书 §19.4）。

## 8. 任务 F：并行开发机制

生成 `docs/plan/PARALLEL-LANES.md`，固定：

1. **车道划分**（接口先行：`cognitive-contracts`/`contracts-ts` 与 trait 端口冻结后各车道方可分叉）：
   - Lane-CTR 契约与生成：contracts 双端 + golden fixtures + codegen——所有车道的地基，最先完成；
   - Lane-KRN 内核主线：domain → store → kernel（M2–M4）；
   - Lane-CFR 符合性与工具：runner、tools、CI（M1 起持续）；
   - Lane-TSC TS 客户端：sdk-ts、admin-cli、agent-shell（golden 对齐后与 KRN 并行推进，M5 集成）；
   - Lane-RUN 运行时与管理面：runtime、management、akp、kernel-server（M4 后启动）；
   - Lane-DOC 文档与计划维护：持续，可随各车道 PR 附带。
2. **并行规则**：一个车道 = 一个 git 分支（建议配 `git worktree`）= 一个 Cursor 会话窗口；跨车道接口变更只能经 Lane-CTR 走契约变更流程并在 PROGRESS 通告；两个车道禁止同时修改同一 crate/package；合并顺序 CTR → {KRN, CFR, TSC} → RUN；一律经 PR + CI 门禁合并。
3. **所有权表**：crate/package → 车道 → 当前分支/会话，写入 PARALLEL-LANES.md 并保持更新。
4. **接续提示词**：为每条车道与 M1–M6 每个里程碑生成自包含提示词存入 `docs/prompts/`，内容 = 公共前缀（§2 硬纪律 + B4 会话协议摘要）+ 车道/里程碑范围与禁止越界事项 + 该阶段相关规范路径清单 + 验收判据 + 指定工作分支。提示词必须可直接粘贴到新 Cursor 窗口独立使用。

## 9. 执行顺序与本次会话完成判据

执行顺序：A → B → C → D → E → F，随后运行全部检查并分批提交。若上下文或时间不足，按 A→B→C 优先完成，并按 B4 协议写 handoff 与剩余任务的接续提示词——这本身就是对接续机制的首次演练。

M0 验收清单（结束前逐项自检并在最终报告中给出结果）：

- [ ] 既有规范资产（`specs/`、`conformance/`、`docs/` 既有文件、根白皮书与 RFC）未被移动、重命名或改写；`History/` 未被读取或引用
- [ ] 空实现下 `cargo build && cargo test`、`pnpm -r build && pnpm -r test` 通过
- [ ] CI 在 Windows + Linux 矩阵可运行，含静态一致性检查与跨语言 golden digest 对比
- [ ] `.cursor/rules/` 9 份规则、`AGENTS.md`、`docs/README.md` 就绪且 frontmatter 正确
- [ ] 8 份标准 + 4 份 ADR 补齐，格式与既有 `docs/standards/`、`docs/adr/` 一致
- [ ] `DEVELOPMENT-PLAN.md`、`PROGRESS.md`、`PARALLEL-LANES.md`、`matrix.yaml` 骨架、checkpoints 模板、`docs/prompts/` 全套接续提示词就绪
- [ ] registry↔schema↔vector 双向无孤儿（或漂移清单列明并已用最小修正闭合）
- [ ] runner 骨架对全部向量报告 not-run；样例 profile manifest 全部标 planned
- [ ] 故意注入的孤儿引用/断链能被 CI 捕获（验证后已回滚）
- [ ] 全部工作已分批提交，提交信息引用对应任务/文档条目

最终报告要求：新建/修改文件清单（按任务 A–F 分组）、M0 验收逐项结果、发现并闭合（或登记）的规范漂移、遗留风险与待决项（如开源许可证选择）、下一步行动（先启动哪条车道、使用 `docs/prompts/` 下哪份提示词）。

不要中途请求确认；遇到无法自决的取舍（许可证、发布渠道等）记入待决项清单继续推进，不要阻塞。
