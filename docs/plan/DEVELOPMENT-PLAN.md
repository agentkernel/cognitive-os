# CognitiveOS 参考实现开发计划

- 状态：v1.0（M0 产出，2026-07-20）；类别 plan（informative）
- 更新责任：里程碑出入口评审时必更；任何范围变更同批更新本文档与 `PROGRESS.md`
- 依据：白皮书 v1.0.1（§4.7 最小闭环、§16 恢复、§20 部署形态、§21 路线图与冻结）、独立审查 §1.2/§13/§14、Review-Conclusions v2.0、`docs/traceability/findings-ledger.md`

## 1. 首版定义：v0.1 Single-node R0/R1

**做**：

- 单节点 Core 最小闭环（白皮书 §4.7）：治理对象 + 五状态机 + 事件日志 + capability 门禁 + Context Resolution + Intent/Effect + Verification/acceptance + checkpoint/恢复。
- §20.5 R0 最小合法实现形态（薄路径降级映射 + 不可降级边界）。
- 确定性 Management API + admin CLI（无模型可 inspect/stop/revoke/reconcile）。
- 任务 Shell（proposal/preview/attach/cancel/watch，客户端非 authority）。
- C0/C1 安装适配（AgentPackageManifest 验证、安装事务、sandbox 拦截）。
- R1 聊天内结构化确认（IMP-05 最低集）。

**明确不做**（v0.1 排除项，写入每条车道禁区）：

- distributed / 多 Agent（M9）；R2/R3 完整审批矩阵；SMS/CRB（M8）；具身（M10）；CIM 异构（M10）；在线/受控学习（M11）；Console 完整产品（独立车道，见 §4）。
- Intelligent Management Shell 保持 **experimental**：不得成为确定性管理/恢复/停止路径的依赖（REQ-MGMT-FALLBACK-001）。
- 受治理记忆与认知发现（M7，独立发布实验 Profile，不阻塞 v0.1）。

## 2. 里程碑

> 工作量两档估算均为**假设**（A 档 = 单人 + AI 代理；B 档 = 3–5 人小队 + AI 代理），以周为单位，含测试与文档联动。每个里程碑出口 = `docs/checkpoints/YYYYMMDD-<里程碑>-milestone-review.md` 逐条对照验收判据，未通过项列为阻断。

### M0 工程基线与开发体系（本会话，已完成）

- **范围**：仓库骨架、Cursor 规则、AGENTS、文档系统、追溯台账、计划、防漂移、CI、runner 骨架、并行机制与接续提示词。
- **交付物 / 验收**：见 `docs/checkpoints/20260720-m0-milestone-review.md`（M0 验收清单逐项）。
- **工作量**：A 档 1 周内 / B 档 2–3 天。（实际：1 个会话）

### M1 合同收敛与符合性 Runner

- **范围**：①按 findings-ledger 闭合仍开放的机器合同缺口——F-003 治理对象双轨迁移（30+ schema 单轨化到 GovernedObjectHeader/ObjectReference，负例向量先行）、D-001/D-006 `$id` 统一、D-004 层 slug 决策、M1 复验项逐条负例复验（F-004/005/006/007/008/010/012/018）；②runner 执行能力：分层执行 M1 当时全部登记向量、五态结果输出、机器 JSON + 人读报告、profile manifest 生成（当前总数与分布只读 PROGRESS）；③schema→Rust/TS 代码生成管线（ADR-0006），最小核心对象集（IMP-08 附录 A.1 14 对象）优先；④注册式 bundle digest 程序（替换 M0 临时 digest，`conformance-evidence.md` §6）。
- **交付物**：单轨 schema 集 + 迁移说明；codegen 工具 + 生成绑定入库；可执行 runner；复验后的 findings-ledger。
- **验收判据**（安全负例必含）：
  1. F-003 关闭：不存在 legacy metadata/strongRef 双轨引用，全 schema 过元校验与 `$ref` 解析；
  2. runner 对全部向量输出五态结果；**加入一个故意错误实现（schema-valid、行为错误），runner 必须将其判 fail**（“仅 schema-valid 不能 pass”自检）；
  3. 未实现层保持 not-run，无一虚报；
  4. M1 复验项负例向量全部执行（含 `effect-state-closure-008`、`prompt-injection-isolation`、`state-store-degradation` 的静态合同侧断言）；
  5. codegen 再生成 diff 为空（CI 钉住）；
  6. registry↔schema↔vector 双向无孤儿保持绿。
- **依赖**：M0。**REQ 域**：OBJ、PROTO、GOBJ-*、CONF、ERR、EFF-STATE。
- **入口 gate**：M0 出口评审通过。**出口**：= tracer bullet 入口 gate 开启（F-002~F-010 类全收敛）。
- **工作量**：A 档 3–4 周 / B 档 1.5–2 周。（假设）

### M2 对象/状态/事件内核

- **范围**：GovernedObject 仓储（SQLite WAL，ADR-0002）、五状态机执行器（消费 `specs/transitions/`）、CAS、append-only 事件日志 + outbox、预算计量原语、状态+事件原子提交。
- **验收判据**：
  1. 并发 CAS：N 个并发写仅 1 个成功，其余 `STATE_CONFLICT` 且无副作用；
  2. 非法迁移全拒（逐表穷举非法 from→to），状态不变、错误码与 registry 一致；
  3. 投影重放 digest 稳定（事件重放两次 → canonical digest 相同）；
  4. 事件不可原地修改（负例：UPDATE 事件行必须失败/被拒）；
  5. 提交路径故障注入 `STATE_STORE_UNAVAILABLE` fail-closed（`state-store-degradation.json` 行为侧）。
- **依赖**：M1（生成合同 + runner）。**REQ 域**：STATE、EVT、OBJ、GOBJ、REC(部分)。
- **入口 gate**：M1 出口通过。
- **工作量**：A 档 3–4 周 / B 档 2 周。（假设）

### M3 治理链与 Context

- **范围**：TenantContext/Principal/Membership/ActorChain、Conversation/ResourceScope、capability 交集 + 单调衰减 + 撤销、九阶段确定性 Context Resolution、缓存键治理绑定、确定性渲染与前缀稳定（IMP-02）。标准：`authn-authz-capability.md`、`context-resolution-and-cache.md`。
- **验收判据**（全部为安全负例或含负例）：
  1. 同租户横向越权被拒且响应与 not-found 同形（`tenant-lateral-read-denial`）；
  2. 管理员身份不含正文授权时读取正文被拒（管理≠读内容）；
  3. 撤销后缓存复用被拒（`context-revocation-cache-reuse`，epoch 键失配）；
  4. 检索前过滤：ranker 输入集不含未过滤对象（`context-rank-before-auth`）；
  5. 跨 Conversation 污染被拒；注入内容不得提升为控制（`prompt-injection-isolation`）；
  6. required 超预算 fail-closed（`context-required-over-budget`）；
  7. 渲染字节稳定 + 前缀稳定断言（`context-render-stability`）。
- **依赖**：M2。**REQ 域**：CTX、CAP、AUTH、GOBJ-AUTHDEL、SCOPE、SEC。
- **入口 gate**：M2 出口 + F-007 行为侧测试计划评审。
- **工作量**：A 档 4–5 周 / B 档 2–3 周。（假设）

### M4 Intent/Effect 与恢复 + tracer bullet

- **范围**：Intent 持久化、OperationDescriptor/绑定与准入拒绝矩阵（F-023）、Effect 状态机、幂等记录、reconcile/verify、checkpoint、恢复八步（§16.6）、故障注入框架、全 sink fencing 清单（F-014）、七性质形式化模型（IMP-07）。标准：`intent-effect-idempotency.md`。
- **验收判据**：
  1. 三个 crash point 全覆盖（`eff-crash-001..003` 行为执行）；
  2. unknown outcome 不成功不换键（`effect-unknown-outcome`）；
  3. 同键异参 `EFFECT_IDEMPOTENCY_CONFLICT` 拒绝（`effect-idempotency-conflict`）；
  4. receipt/远端 completed 不完成 Task（`remote-completed-not-acceptance`）；
  5. 恢复顺序错乱注入（如先恢复 Loop 后 fence）必须被测试捕获；
  6. sink fencing 矩阵逐端负例（旧 epoch dispatch 在每个提交端被拒）；
  7. **tracer bullet 端到端竖切**：一条真实任务 UserIntent→…→acceptance 全链在单节点跑通并留证据。
- **依赖**：M3。**REQ 域**：EFF、REC、RUN(部分)、GW。
- **入口 gate**：**F-002~F-010 类 P0 全闭合**（= M1 出口 + M2/M3 行为验收；findings-ledger 为准）+ F-014/F-023 工作项已排入本里程碑。
- **工作量**：A 档 5–6 周 / B 档 3 周。（假设）

### M5 意图链与 Harness/Shell/管理面

- **范围**：UserIntentRecord→IntentInterpretation（准入）→TaskContract、有界 Loop 与进展/停滞判定、Management API + 确定性 admin CLI、任务 Shell proposal/preview/attach/cancel、snapshot+cursor watch、R1 聊天内结构化确认（IMP-05 最低集，闭合 F-011 的 R1 部分）、AKP envelope + HTTP/SSE（ADR-0003）。标准：`task-loop-verification.md`、`akp-envelope-and-http-profile.md`、`event-audit-watch.md`。
- **验收判据**：
  1. 实质歧义必须澄清（`shell-target-ambiguity-001`，`INTENT_CLARIFICATION_REQUIRED`）；
  2. 用户修正推进 epoch 并 fence 旧 dispatch（`intent-supersede-002`）；
  3. Shell 退出不取消（`shell-detach-attach-004`）；cancel 经 Effect 闭合（`shell-cancel-semantics-005`）；
  4. 无模型仍可 inspect/stop/revoke/reconcile（`management-deterministic-fallback`）；
  5. 通道隔离负例（`shell-channel-isolation-003`、`SHELL_CHANNEL_BINDING_MISMATCH`）；
  6. 管理门禁负例组全绿（`management-gate-denials`、`management-untrusted-self-authorization`、`management-independent-approval`）；
  7. watch 断线重连 + 陈旧 cursor（`shell-watch-resume-006`、`WATCH_CURSOR_STALE`）；
  8. R1 确认：高风险动作无结构化确认不执行（IMP-05 负例）。
- **依赖**：M4。**REQ 域**：INTENT、SHELL、RUN、MGMT、AKP、AUDIT、EVT。
- **入口 gate**：M4 出口（tracer bullet 证据在案）+ F-011 R1 最低集机器合同登记完成。
- **工作量**：A 档 5–7 周 / B 档 3–4 周。（假设）

### M6 安装与适配、v0.1 发布

- **范围**：AgentPackageManifest 验证、安装事务与回滚、sandbox 拦截（**Linux 为参考平台**；Windows 开发经 WSL2 或 Linux CI 覆盖负例，平台分别声明——F-017）、C0/C1 adapter、带外修改对账（IMP-11）、readiness case（MANAGEMENT_READY→USER_READY→OPERATIONAL）、profile manifest 首次真实声明、治理开销指标基线（IMP-04/REQ-PERF-004，声明 ungoverned 基线）、§20.5 R0 降级映射验证（IMP-06）。
- **验收判据**：
  1. 篡改包安装被拒（`agent-installation-verification`，`AGENT_PACKAGE_VERIFICATION_FAILED`）；
  2. adapter 绕过被拦截（`agent-adapter-bypass`，按平台矩阵分别声明）；
  3. 安装事务中断回滚干净（半安装状态不可见）；
  4. 带外修改被对账发现（`agent-out-of-band-reconciliation`）；
  5. readiness 分级：管理面先于用户面可用（故障注入验证顺序）；
  6. REQ-PERF-004 全指标族首次报告（p50/p95/p99、cache-hit preservation、每调用额外持久化写、审批延迟与橡皮图章率、开销占比）；
  7. profile manifest：core_digital 相关层按真实证据声明，其余 planned/experimental。
- **依赖**：M5。**REQ 域**：AGENT-*、PERF、CONF、PROFILE-CORE。
- **入口 gate**：M5 出口。**出口 = v0.1 发布评审**（含 F-017 平台矩阵、F-011 R1 部分闭合核验）。
- **工作量**：A 档 4–6 周 / B 档 2–3 周。（假设）

### M7+ 扩展 Profile（每个独立发布实验 Profile，不阻塞 v0.1）

| 里程碑 | 范围 | 关键闭环 | 工作量（假设 A/B） |
|---|---|---|---|
| **M7** 受治理记忆与认知发现 | MemoryCandidate/准入/生命周期、IMP-03 异步准入 + read-your-write、发现 delta/停滞 | F-019 行为侧；`memory-*`、`context-candidate-admission`、`discovery-read-separation`、跨 scope 晋升负例；REQ-PERF-005 四臂对照首次实测（IMP-18） | 4–6 周 / 2–3 周 |
| **M8** Operation Catalog 与 SMS/CRB | catalog 生命周期/match/bind、语义中介降级边界、CRB 核算 | `catalog-*`、`semantic-fallback-bounds`、`MODEL_EGRESS_DENIED` 负例 | 4–6 周 / 2–3 周 |
| **M9** 分布式与多 Agent | placement、mailbox、分区、委派单调衰减、handoff/verifier | 独立审查 No-Go 解除条件：单节点正确性 + 全 sink fencing + 分区模型；多 Agent 需单 Agent 强基线 | 8–12 周 / 4–6 周 |
| **M10** 具身与异构 | 实时安全内核隔离、CIM 校准、staleness | **无独立安全证据前只标 experimental**（F-030）；`stale-embodied-observation`、`cim-calibration-mismatch` | 8–12 周 / 4–6 周 |
| **M11** 受控学习与知识编译 | shadow/canary、直接晋升拒绝、投毒隔离、失效传播 | `learning-*`、`knowledge-*` 负例组 | 6–10 周 / 3–5 周 |

### Console 产品车道（单列，不排期实现）

依据 `apps/cognitiveos-console/PRODUCT-DESIGN.md` §17（MVP 与路线图）与 §20.3（后端依赖结论）。**M0 仅建立依赖追踪**；每组依赖 = 对应机器契约 + 后端能力交付 + gate 通过后，才启动相应 Console 里程碑。

2026-07-20 批准 Lane-CON 激活前的窄幅 informative 文档例外：可维护平台研究/产品设计、产品要求与决策、README/roadmap/index、parity matrix、相关治理说明和已登记漂移修正；不得启动 Console 实现、修改 normative 机器资产或扩大实现/测试/Profile 声明。客户端项目根见 [`clients/`](../../clients/README.md)（ADR-0007），实现 gate 见 [`clients/governance/readiness-gates.md`](../../clients/governance/readiness-gates.md#console-实现-gate)。

Agent Hub / 直连接管是 Lane-CON 下的独立产品线（Direct Takeover + Governed 两部署模式），canonical 文档见 [`clients/agent-hub/docs/`](../../clients/agent-hub/docs/README.md)，Master 计划见 [`clients/agent-hub/plan/agent-hub-development-plan.md`](../../clients/agent-hub/plan/agent-hub-development-plan.md)。同受本节 gate 阻断，另加 Paseo/AGPL 复用法务 gate；当前仅 informative 文档与计划/提示词，`implementation not-implemented / evidence none`，未激活实现车道。

| # | §20.3 依赖组 | 提供方里程碑 | 状态 |
|---|---|---|---|
| 1 | Shell/Management/Watch API 与 AKP envelope/transport | M5 | 未交付 |
| 2 | AgentExecution/Task/Verification 生命周期载体、AcceptanceDecision | M2+M4+M5 | 未交付 |
| 3 | installer/sandbox/adapter/compatibility runner 与安装迁移表 | M6 | 未交付 |
| 4 | approval queue/challenge/quorum/trusted surface/anti-fatigue | M5（R1 最低集）→ R2/R3 后置 | 未交付 |
| 5 | approval canonical projection/域分离/密钥轮换、server-side risk floor、idle 规则 | M5~M6 | 未交付 |
| 6 | Memory private working set、Knowledge Evidence/Claim schemas | M7 | 未交付 |
| 7 | AuditRecord/export、StateSnapshot/ack、reconciliation/recovery report | M4~M6 | 未交付 |
| 8 | native-app auth redirect/PKCE、offline lease、deep-link、IdP/push/config/update/health | M6+（部署基建） | 未交付 |
| 9 | conformance runner、executed vectors、profile-manifest 实例 | M1（runner）→ M6（真实 manifest） | runner 已具 M1–M5 执行能力；当前 84 vectors 中 **52 pass / 32 not-run**（以 PROGRESS / CI pins 实测为准），真实 Profile manifest 仍待 M6 |

激活规则：依赖组 1/2/7 交付并过 M5 出口评审后，且目标平台 [Open PoC 与 GA gates](../../clients/governance/readiness-gates.md#console-实现-gate) 用真实 API/真实 OS 行为出具可复现实测报告，才可启动 Console "MVP Desktop 只读监督"实现里程碑规划；不得用 mock 冒充。文档例外不改变此 gate。

## 3. IMP-01~18 与 F-001~F-030 → 里程碑映射

完整逐条状态见 `docs/traceability/findings-ledger.md`（唯一权威）。汇总：

| 里程碑 | 关闭/验收的 F | 落地的 IMP |
|---|---|---|
| M0 | F-029（计划/ADR 入口）；F-028 部分（文档修订随 Lane-DOC） | IMP-17（实测计数）；IMP-01（冻结写入规则） |
| M1 | **F-003（P0，出口阻断）**；F-015 残余；复验 F-004/005/006/008/010/012/018；D-001/004/005/006 | IMP-08（最小对象集优先）；IMP-04（证据格式） |
| M2 | F-008 行为侧 | — |
| M3 | F-007 行为侧、F-021（IMP-02）行为侧 | IMP-02 |
| M4 | **F-014（出口阻断）**、**F-023（出口阻断）**、F-010 行为侧；tracer bullet（§1.2 gate） | IMP-07 |
| M5 | **F-011 R1 最低集（出口阻断）** | IMP-05（R1 部分）、IMP-09、IMP-14 |
| M6 | **F-017（出口阻断）**、F-022 实测 | IMP-04、IMP-06、IMP-11、IMP-12 |
| M7 | F-019 行为侧、F-026 首次实测路径 | IMP-03、IMP-18 |
| M9~M11 | F-030 解除条件逐项 | IMP-13（M10） |
| 全程 | F-001（证据积累消解）、F-025/F-026（宣称口径约束）、F-027（冻结对冲） | IMP-01、IMP-16 |

## 4. 风险清单与对冲

| # | 风险 | 对冲 |
|---|---|---|
| R1 | 规范继续膨胀，实现永远追不上 | §21 冻结条款 + `.cursor/rules/01`（v0.1 前禁新增对象族/Profile/REQ 域）；变更只准修正型 |
| R2 | schema-valid 冒充 behavior-pass，符合性空转 | M1 runner 自检（故意错误实现必须 fail）+ 五态口径（`conformance-evidence.md`）+ 状态用语红线 |
| R3 | 治理开销驱赶实践者 | IMP-02 前缀稳定（M3 验收）+ IMP-04 开销指标族自 M6 持续报告；超预算 = 发布阻断讨论项 |
| R4 | 并行车道冲突、接口漂移 | PARALLEL-LANES 所有权表 + 接口变更只经 Lane-CTR + 禁止同 crate 双改 + 合并顺序 CTR→{KRN,CFR,TSC}→RUN |
| R5 | Windows/Linux sandbox 差异导致虚假安全声明 | F-017 处置：Linux 参考平台，按平台矩阵分别声明与测试；Windows 负例经 WSL2/Linux CI；不做跨平台合并声明 |
| R6 | 后端-Console 期望错位（产品文档先行于契约） | §2 Console 依赖台账（九组逐项 gate）+ PRODUCT-DESIGN 漂移标注义务（docs-sync-contract）|

## 5. 指标基线（逐里程碑生效）

- **每里程碑**：向量分层通过率（15 层 + 跨切片）；REQ 覆盖计数（specified/实现中/已测试）；安全负例计数（新增/累计执行）；开放 P0/P1 finding 数。
- **自 M6**：REQ-PERF-004 治理开销指标族（授权/Context/Effect 各阶段延迟 p50/p95/p99、cache-hit preservation、每受治理调用额外持久化写、审批延迟与橡皮图章率、开销占端到端比例），**声明 ungoverned 基线**。
- **Agent 收益声明**（M7+ 才可能出现）：遵循 REQ-PERF-005 与 `docs/evaluation/agent-benefit-benchmark.md`——四臂对照（native / governance-only / optimized / ablation）+ 预注册门槛；非劣化不得报告为性能提升；此前一律只可声明 hypothesis。

## 6. 待决项（不阻塞开发）

| # | 事项 | 现状 |
|---|---|---|
| P-1 | 开源许可证选择 | 未定（Cargo/package.json 均未声明 license）；候选 Apache-2.0 / MIT+Apache 双许可；需权利人决定 |
| P-2 | 发布渠道（crates.io/npm 是否发布、命名空间） | 未定；当前全部 `private`/`publish=false` |
| P-3 | CI 远端（GitHub 仓库/Actions 配额） | ci.yml 已备；仓库当前无 remote，推送后生效 |
| P-4 | pnpm 大版本升级（10→11 提示） | 暂钉 10.33.2（packageManager 字段）；升级属修正型变更 |
