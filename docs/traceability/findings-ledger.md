# Findings 台账：F-001~F-030 与 IMP-01~18 逐条现状

- 状态：M0 首次逐条核验（2026-07-20，基于 `CognitiveOS-Review-Conclusions.md` v2.0 §6.4 处置记录 + 对 v1.0.1 机器资产的抽查复验）
- 用途：**M1 入口 gate 的依据**。开放 P0 未闭合前，对应子系统不得进入实现里程碑（AGENTS.md 硬纪律 6）。
- 状态取值：`closed-by-1.0.1`（1.0.1 静态反例复验关闭）/ `partially-closed`（部分收敛，余项列明）/ `open`（未闭合，阻断标注的里程碑）/ `framework`（判定框架级/非可修缺陷，以证据推进消解，不阻断单项合同）。
- 独立审查针对白皮书 v1.0.0（commit `f2f826a`）；1.0.1（commit `4e02bbf`）为其后的修复轮。里程碑编号以 `docs/plan/DEVELOPMENT-PLAN.md` 为准。
- 更新义务：触碰任一条目的 PR 必须同步本台账（`.cursor/rules/02-workflow-docs-sync.mdc`）。

## 一、独立审查 F-001~F-030

| 编号 | 级 | 主题 | 状态 | 证据 / 备注 | 阻断 |
|---|---|---|---|---|---|
| F-001 | P0 | 无实现、无 runner、无已执行测试证据 | open（性质：证据缺口，随实现消解） | M0 已建 runner 骨架（全 `not-run`，不构成证据）；执行能力 M1 | 关闭路径贯穿 M1~M6；不阻断 M1 启动（M1 正是其闭合手段） |
| F-002 | P0 | 17 条 REQ 的 owner_spec 指向 informative 白皮书 | closed-by-1.0.1 | 抽查 `specs/registry/requirements.yaml`：owner_spec 现指向 schema/companion（如 REQ-STATE-003 → `specs/core/README.md`，REQ-CTX-001 → `context-request.schema.json`）；REQ-PERF-002/004 归属 `conformance/README.md`、REQ-PERF-005 归属 `docs/evaluation/agent-benefit-benchmark.md`（文内显式 owner 声明） | — |
| F-003 | P0 | 治理对象双轨：legacy metadata/strongRef 与 GovernedObjectHeader 并存 | **partially-closed（合同层复验已完成）** | 迁移落地（2026-07-20，`c190d7b`：36 份 schema + registry owner_spec + 2 向量改指，legacy 定义零引用）。M1 Lane-CTR 收尾（2026-07-20）：①迁移批次两份向量调整核验**合法**——schema-meta-001 跟随 registry owner_spec 同批改指；effect-state-closure-008 对齐 `effect.transitions.json` v0.2 机器真相（OUTCOME_UNKNOWN 直接出口仅 RECONCILED；COMPENSATING/QUARANTINED 属 reconcile 后 still_unknown 路径），deny 期望与错误码未动、无负例删除；②新增负例向量 GOBJ-LEGACY-METADATA-001 / GOBJ-LEGACY-STRONGREF-001（双轨形态必须被拒；registry 已映射；状态 not-run）；③合同层可执行复验：`crates/cognitive-contracts/tests/schema_contract.rs` 与 `packages/contracts-ts/src/schema-contract.test.ts` 双侧证明 56 schema 可编译、负例实例被拒、迁移正例通过；④legacy `$defs` 决策 = **保留**（deprecated、零引用，仅作 §6 legacy adapter 的版本钉扎映射源，REQ-GOBJ-REF-003/REQ-GOBJ-MIG-001 的被引用物；引用禁令由 static_check 检查 5 + 双侧合同测试守护）。M1 出口仍需：runner 真实执行负例向量（Lane-CFR）+ codegen 对齐 | M1 出口复验后关闭（tracer bullet 入口 gate 组成部分） |
| F-004 | P0 | ActivityContext↔ContextView 创建环；ContextRequest 缺租户/ActorChain 治理字段 | closed-by-1.0.1 | `context-request.schema.json` 现含治理字段；启动环由 admission（`context-request-admission.schema.json`）拆解；M1 复验负例向量 | M1 复验项 |
| F-005 | P0 | Effect 三处真相重复；`COMMITTED+unknown+pending` 反例 schema-valid | closed-by-1.0.1 | `effect.schema.json` 加约束 + `effect.transitions.json` 收口；向量 `effect-state-closure-008.json`（REQ-EFF-STATE-001）为负例 | M1 复验项 |
| F-006 | P0 | ABORTED 终态掩盖外部残留副作用 | closed-by-1.0.1 | effect 迁移表含 reconciliation 语义（`metadata.reconciliation_result`）；`effect-unknown-outcome.json` 覆盖 | M1 复验项 |
| F-007 | P0 | Context 解析后撤销 capability 的竞态未闭合 | closed-by-1.0.1（合同层） | dispatch/commit 重验 guard 已入 effect 迁移表；`context-revocation-cache-reuse.json`（REQ-CAP-005）为负例；**行为证明在 M3/M4** | 行为验收挂 M3/M4 |
| F-008 | P0 | 持久化失败无唯一安全语义 | closed-by-1.0.1 | `STATE_STORE_UNAVAILABLE` 已登记（fail-before-effect 语义入描述）；向量 `state-store-degradation.json`（REQ-REC-003） | 行为验收挂 M2/M4 |
| F-009 | P0 | canonical digest/signature 缺自引用投影规则 | closed-by-1.0.1 | `canonical-encoding-and-digest.md` §10（digest_excluded 显式声明、无声明即 fail） | — |
| F-010 | P0 | Checkpoint 缺恢复稳定事实（high-watermark/epoch/版本钉扎） | closed-by-1.0.1 | `loop-checkpoint.schema.json` 已含恢复字段；M4 行为验收 | 行为验收挂 M4 |
| F-011 | P0→P1（v2.0 重定级） | 高风险审批安全基线仍 informative | **open** | §12.12 文本已写（IMP-05），机器合同（approval schema 硬化 + 向量）未登记；v2.0 定为"发布前 P1 义务" | **阻断 M5 出口**（R1 聊天内结构化确认最低集）；R2/R3 完整审批不阻塞 v0.1 |
| F-012 | P1 | TaskContract 旧四值 state enum、允许无验收条件 | closed-by-1.0.1 | `task-contract.schema.json` 对齐 `task.transitions.json`（9 态）；验收条件必填 | M1 复验项 |
| F-013 | P1 | 同键异参幂等冲突错误码未登记 | closed-by-1.0.1 | `EFFECT_IDEMPOTENCY_CONFLICT` 已登记；向量 `effect-idempotency-conflict.json`（REQ-EFF-002） | — |
| F-014 | P1 | fencing 未证明覆盖所有提交端（sink 矩阵缺失） | **open** | 需在 M4 产出全 sink fencing 清单 + 逐端测试 | **阻断 M4 出口** |
| F-015 | P1 | 向量数量≠行为覆盖（大量 REQ 仅挂 generic 追溯向量） | partially-closed | 1.0.1 补部分负例向量；`spec-contract-coverage.json` 仍托管 170+ REQ 的 generic 追溯；M1 runner 分层执行 + 覆盖缺口清单 | 覆盖提升贯穿 M1~M6 |
| F-016 | P1 | implemented 声明可被 degradation 稀释 | closed-by-1.0.1 | `conformance/README.md` 状态语言收紧（degradation 强制缩范围或降 experimental；安全负例不可豁免） | — |
| F-017 | P1 | sandbox 不可绕过证据过粗（缺平台/通道矩阵） | **open** | 需按宿主平台分别声明与测试（Linux 参考平台；Windows 经 WSL2/CI） | **阻断 M6 出口** |
| F-018 | P1 | ContextView 允许 `untrusted+control` 信任矛盾 | closed-by-1.0.1 | `context-view.schema.json` 约束收紧；`prompt-injection-isolation.json` 负例 | M1 复验项 |
| F-019 | P1 | read-your-write 隔离无法机器表达 | closed-by-1.0.1 | REQ-MEM-ADMIT-002 + 向量 `memory-read-your-write.json`；行为实现挂 M7 | 行为验收挂 M7 |
| F-020 | P1 | 性能报告 schema 不支撑 A/B、消融、非劣效 | closed-by-1.0.1 | `performance-report.schema.json` 扩展 + `agent-benefit-benchmark.md`（REQ-PERF-005 四臂+预注册） | — |
| F-021 | P1 | 相对顺序不变 ≠ 字节前缀稳定 | closed-by-1.0.1 | REQ-CTX-012 + `context-render-stability.json`；行为验收挂 M3（IMP-02） | 行为验收挂 M3 |
| F-022 | P1 | SLO 只有容器无阈值 | closed-by-1.0.1 | performance-report `slo_profile` 结构化；阈值预注册机制见 benchmark 文档 | M6 实测 |
| F-023 | P1 | 不可查询/不可幂等执行器缺准入拒绝矩阵 | **open** | 需 M4 落 OperationDescriptor 准入矩阵（查询性/幂等性能力自描述 + 拒绝路径） | **阻断 M4 出口** |
| F-024 | P1 | §4.7 四视图映射遗漏 owner | closed-by-1.0.1 | 责任矩阵补全（含 Approval Service 归属） | — |
| F-025 | P1 | OS 身份缺不可绕过参考监视器实现证据 | framework | 随 M6 sandbox 拦截证据 + M4 门禁行为证据积累；宣称口径持续用"Agent control plane + durable governed runtime" | 定性宣称约束，全程有效 |
| F-026 | P1 | Agent 收益 E0 假设、无实测 | framework | REQ-PERF-005 合同已立；实测四臂对照挂 M7+（优化落地后）；此前禁止任何收益宣称 | 宣称约束，全程有效 |
| F-027 | P2 | 规范表面积远大于最小 Core | framework | 对冲=冻结条款（IMP-01）+ M1 最小核心对象集优先（IMP-08 A.1 14 对象） | — |
| F-028 | P2 | 外部证据分级混类、版本过期 | partially-closed | 1.0.1 修订附录 C 部分条目；剩余为文档级修订，随 Lane-DOC 顺带 | 不阻断 |
| F-029 | P2 | 无受版本控制的里程碑/ADR 入口 | closed-by-M0 | 本次交付 `docs/plan/`、`docs/adr/0001~0006`、`docs/checkpoints/`、本台账 | — |
| F-030 | P2 | 高级 Profile 应继续延后 | closed-by-plan | DEVELOPMENT-PLAN：distributed/多 Agent/具身/学习列 M9~M11，不阻塞 v0.1；具身无独立安全证据前仅 experimental | — |

**开放 P0 汇总**：F-003（阻断 M1 出口/tracer bullet 入口；2026-07-20 Lane-CTR 后合同层义务全部完成，唯一剩余 gate = Lane-CFR runner 真实执行负例向量）。F-001 为证据缺口性质（随 M1~M6 消解，不单独阻断）。
**开放 P1 汇总**：F-011（M5）、F-014（M4）、F-023（M4）、F-017（M6）、F-015（持续）。
**tracer bullet 入口 gate**（审查 §1.2"F-002~F-010 类合同缺口收敛"）：F-002/004/005/006/007/008/009/010 已由 1.0.1 关闭且待 M1 负例复验；F-003 为唯一残余 → **M1 完成 = tracer bullet 入口开启**。

## 二、评审结论 IMP-01~18

| 编号 | 主题 | 现状 | 落点 |
|---|---|---|---|
| IMP-01 | 实现优先、冻结规范表面扩张 | 已应用（§21/§1.2；`.cursor/rules/01` 第 4 条） | 全程纪律 |
| IMP-02 | 渲染前缀稳定性显式要求 | 已登记（REQ-CTX-012、`context-render-stability.json`） | M3 验收 |
| IMP-03 | 记忆准入异步化 + read-your-write | 已登记（REQ-MEM-ADMIT-002、`memory-read-your-write.json`） | M7 实现 |
| IMP-04 | 治理开销性能契约 | 已登记（REQ-PERF-004 + schema 扩展） | M6 基线报告；M1 runner 证据格式 |
| IMP-05 | 审批子系统（分级确认/防疲劳） | 文本已应用（§12.12）；机器合同未登记（=F-011） | M5（R1 最低集）；R2/R3 后置 |
| IMP-06 | R0 薄路径合法降级形态 | 已应用（§20.5 降级映射+不可降级边界） | M6 readiness case |
| IMP-07 | Effect/恢复/fencing 形式化验证承诺 | 承诺挂 §21 Phase 4；v2.0 义务化 | M4 交付七性质模型 |
| IMP-08 | 对象本体分层（最小核心 14 对象） | 已应用（附录 A.1–A.3）；**M1 生成器已交付**（2026-07-20 Lane-CTR：`contracts-codegen` 按 A.1 优先序生成 14 对象 ↔ 17 schema + `$ref` 闭包 2 份，Rust/TS 双侧入库、CI regenerate-and-diff 门生效，映射表见 ADR-0006 Delivery record） | 生成器优先序已落地；其余对象族随消费里程碑 |
| IMP-09 | 验证带宽=部署一级约束 | 已应用（§2.2、§19.7） | M5/M6 验收口径 |
| IMP-10 | 参考文献补全 | 已应用（附录 C.5） | — |
| IMP-11 | 带外修改对账路径 | 已登记（REQ-AGENT-OOB-001、`agent-out-of-band-reconciliation.json`） | M6 |
| IMP-12 | 批量 tool proxy 合法形态 | 已应用澄清 | M6 |
| IMP-13 | CIM 细节下沉 companion | 已应用（§14 减半） | — |
| IMP-14 | 意图澄清义务按 profile 裁剪 | 已应用（§6.9；R0 放宽为预览义务） | M5 验收（`task-loop-verification.md` §2） |
| IMP-15 | C2 生态差距承认与激励 | 已应用（§5.2/§21） | — |
| IMP-16 | 主文档瘦身 | 长期编辑纪律 | Lane-DOC 持续 |
| IMP-17 | 计数必须每轮实测 | v2.0 revised 采纳 | PROGRESS 一律实测数（本轮：273 REQ/55 码/56 schema/74 向量） |
| IMP-18 | 收益声明合同（REQ-PERF-005） | 已登记（四臂+预注册） | M7+ 实测；此前只可声明 hypothesis/non_inferiority |

## 三、漂移登记（M0 盘点新发现）

| # | 漂移 | 处置 | 状态 |
|---|---|---|---|
| D-001 | `specs/schemas/profile-manifest.schema.json` 与 `effect.schema.json` 缺顶层 `$id`（其余 54 份风格亦不统一：三种 `$id` 风格并存） | **M1 Lane-CTR 已闭合（2026-07-20）**：56 份 schema 全量统一为 `$id` = 文件名（43 份改写：13 绝对 URL 替换、29 补齐、1 非常规值替换；13 份原本已合规）；`check-consistency.mjs` 新增 `$id` 策略红灯并移除剥离 `$id` 兼容层；策略写入 `conformance/README.md` 与 `.cursor/rules/12`；双侧合同测试断言策略 | closed-by-M1 |
| D-002 | `conformance/README.md` 执行结果四态（pass/fail/not-applicable/documented-degradation）与开发提示词五态（+not-run）措辞差 | 非漂移：`not-run` 定义为**报告级**状态，收口于 `docs/standards/conformance-evidence.md` §2；向量与 README 不改 | closed（术语收口） |
| D-003 | 计数漂移：审查文档写"约 270 REQ"，实测 273；白皮书/提示词写"约 56 schema"，实测 56 | 采纳 IMP-17：PROGRESS 只用实测数；历史文档不回改 | closed（口径固定） |
| D-004 | conformance README 15 层中第 7 层（知识编译）与第 8 层（性能）无专属向量 `layer` slug：知识类向量落在 `harness-loop`/`context-semantic`/`security-negative`，性能合同向量落在 `wire-schema` | 登记；runner 骨架如实呈现（layer 7/8 vectors=0 + 跨切片计数）；M1 runner 交付时与 Lane-CFR 决定是否补 slug（属修正型变更） | open → M1 |
| D-005 | `state-transition-table.schema.json` 钉 `version: const "0.1"`，但 1.0.1 修订后 `effect.transitions.json` 已升 `version: "0.2"`（关闭 F-005/F-006 时未同步表 schema）→ effect 迁移表对表 schema 校验失败 | **M0 最小修正已闭合**：const 放宽为 `enum ["0.1","0.2"]`（修正型变更，不改语义；提交说明注明）；M1 合同收敛时随 D-001 统一版本策略 | closed-by-M0 |
| D-006 | schema `$id` 三种风格并存：13 份治理对象 schema 用绝对 URL `$id`（`https://schemas.cognitiveos.dev/...`），`state-transition-table` 用相对文件名 `$id`，`profile-manifest`/`effect` 等缺 `$id`；绝对 `$id` 与 conformance README"相对 `$ref` 从所在文件解析"的规则冲突（会劫持 base URI） | 并入 D-001 处置；**M1 Lane-CTR 已闭合（2026-07-20）**：统一策略落地 = 全量相对文件名 `$id`（`$id` == 文件名），绝对 URL 全部移除，`static_check.py` 同批删除 `schemas.cognitiveos.dev` 别名注册 | closed-by-M1 |
| D-007 | Console `PRODUCT-DESIGN.md` 与 `requirements-traceability.md` 在 D-005 关闭后仍称 transition table schema 只接受 `0.1` | 2026-07-20 对齐机器事实：登记 schema 已接受 `0.1/0.2`，保留 D-005 作为已关闭历史，不再列为 Console blocker | closed-by-doc-repair |
| D-008 | Console 产品追踪在 F-003 单轨迁移落地后仍称 legacy metadata/strongRef 双轨尚未迁移 | 2026-07-20 对齐 findings-ledger：迁移已落地但 F-003 仍待 M1 runner 负例、codegen 与 legacy `$defs` 决策复验 | closed-by-doc-repair |
| D-009 | Lane-CON 与 DEVELOPMENT-PLAN 的实现 gate 指向已不存在的 `PRODUCT-DESIGN §12.6 POC-01~12` | 2026-07-20 改指 `docs/platforms/README.md#console-实现-gate` 及各平台可定位的 Open PoC/GA gate；仍要求真实 API，禁止 mock 冒充 | closed-by-doc-repair |
| D-010 | F-003 结构变更后，PRODUCT-DESIGN 缺少 docs-sync-contract §2.8 要求的文首漂移登记节/标注 | 2026-07-20 恢复漂移登记节并记录 F-003、D-005/D-007、D-009 影响；不改变 normative 资产 | closed-by-doc-repair |
| D-011 | `canonical-encoding-and-digest.md` §13 要求 bundle manifest 每资产携带 SemVer，但已登记规范资产（schema/registry/transition）均未声明**每资产** SemVer——标准与资产之间的登记缺口 | M1 Lane-CTR 登记并最小修正：注册式 bundle 程序对每资产统一施加套件机器版本 `0.1.0-draft.1`（`cognitive_contracts::bundle::SPEC_SUITE_VERSION`，双语言常量一致），依据与口径写入 `conformance-evidence.md` §6；待任一资产真实分岔版本时再登记每资产版本（属修正型变更）。原编号 D-007 与 Lane-CON 同日条目撞号，后合并方（CTR）改号为 D-011 | closed-by-M1（决策落档） |

## 四、复验方法备注

M0 核验方式 = 静态证据抽查（schema 字段、registry 条目、向量存在性与 expected 结构），未执行任何向量（无 runner 执行能力，符合四类状态用语）。标注"M1 复验项"的条目：M1 runner 分层执行负例向量后，将本台账状态从 closed-by-1.0.1 升级为 verified-by-vector 或降级重开。
