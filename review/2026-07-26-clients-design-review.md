# CognitiveOS 客户端项目（clients/）设计评估报告

> 类别：informative design review（非 canonical；不改变任何状态口径、gate 或 readiness 结论）
>
> 日期：2026-07-26 ｜ 评审对象：`clients/` 全树（pc / mobile / shared / agent-hub / governance / plan / prompts，约 130+ 份 Markdown，无实现代码）
>
> 方法：精读 pc 四份核心设计（product-brief、design-system、information-architecture、journeys-and-screens）与治理/计划核心文件；三路并行子评审覆盖 agent-hub、mobile+shared、governance+plan+pc 平台文档；关键事实（rules 文件存在性、全局 PROGRESS 计数、断链、哈希占位）逐项二次核验。

## 1. 总体结论

**设计本体质量为高档，不需要整体重构；治理与计划层需要部分重构；设计层需要增量修补。**

| 维度 | 评分（/10） | 一句话结论 |
|---|---:|---|
| 设计合理性 | 9.0 | 治理语义（authority projection、fail closed、OUTCOME_UNKNOWN 一级状态）贯穿全树，两模式/L1–L8/companion 边界高度自洽 |
| 完备性 | 7.5 | 产品→架构→安全→计划→追踪链条闭合且计数对账；但状态快照集体过期、存在断链与虚假证据引用、UI 视觉规格缺失 |
| 实用性 | 5.5 | 全部里程碑 blocked、PoC 授权死锁、6 Adapter 同批首发不现实、技术栈比较不收敛无期限——计划层当前失去导航价值 |
| 交互友好性 | 8.5 | 15 通用页面状态矩阵、旅程规格含失败出口/恢复入口/可访问性，达到可直接开工线框的水平 |
| UI 美观度 | 7.0 | 有明确品牌命题与完整 token/动效/反模式体系，但全部停留在"候选"，零视觉稿/原型验证 |

## 2. 五维详评

### 2.1 设计合理性（9.0）

强项：

- **铁律一致**：三条核心不变量在全树无一处违背——① 客户端只显示 authority projection，不凭本地状态宣称提交/批准/完成；② 不确定性是一级状态（`OUTCOME_UNKNOWN`/`pause_pending`/stale 均有独立结构，禁止盲重试）；③ 失联收窄能力（fail closed）。证据：`pc/docs/product/product-brief.md` §5、`pc/docs/ux/journeys-and-screens.md` JRN-009/010、`agent-hub/docs/product/deployment-modes-and-guarantees.md` §1.3。
- **边界划分干净**：Direct Takeover 与 Governed 两模式互斥且"连接状态不是第三模式"；接管 L1–L8 逐层最低安全条件冻结于 decision-log（DEC-006~011）；手机 remote companion 定位（不承载 runtime/authority/Vault，仅 R0/R1）在 5 处口径完全一致。
- **安全模型成熟**：System Card 与 Agent 内容不共享 renderer/IPC capability、并诚实放弃"视觉不可仿冒"承诺（`design-system.md` CMP-003）；acquisition 由无 ambient credential 的低权限 broker 执行并防 SSRF/UNC（JRN-006）；R1 确认初始焦点不在批准按钮、Enter 不默认批准（JRN-008）。

弱项：Agent Hub 对 Console 设计系统的复用只有一句话（`agent-hub/plan/lane-desktop.md:9`），未指明 canonical 文档与复用边界。

### 2.2 完备性（7.5）

强项：PRD↔DEC↔TM↔PoC↔blocker 追踪链闭合（agent-hub：PRD-001~028 / DEC-001~026 / TM-001~021 / 33 项 PoC 对账一致）；mobile 双平台设计各 21 章、16 页面 + 18/19 状态；MIGRATION-MAP 逐文件映射全 `done`。

弱项（详见 §3 问题清单）：

1. 状态快照过期：`clients/` 全树仍以"84 向量 46 pass / 38 not-run、M5 未完成"为基准，而全局 `docs/plan/PROGRESS.md` 实测已是 **85 向量 60 pass / 25 not-run，M5 done（2026-07-21）、M6 出口 GO-with-explicit-non-claim**。该计数在 ≥7 处复写、集体失真。
2. 证据链有洞：`clients/READINESS.md` 第 23 行以 `.cursor/rules/16 + 17-client-project-boundaries.mdc` 为 structure-ready 证据，但活动目录 `D:\agent-kernel\.cursor\rules\` 现仅存 `qingtian-mcp.mdc`，16/17/11 号规则实际已被归档至 `History/.cursor/rules/`——**clients 治理所依赖的规则执行层已整体失效而文档仍称"已生效"**。
3. 断链：`shared/docs/contracts-sdk/README.md:7` 指向不存在的 `11-typescript-clients.mdc`；`agent-hub/plan/README.md:45` 链接文字与实链不一致。
4. UI 视觉规格缺口：全树无线框/视觉稿；design-system 自认全部 token 为候选、未过 WCAG/High Contrast 实测。

### 2.3 实用性（5.5）——本次评审的主要扣分点

- **PoC 授权死锁**：`pc/docs/platforms/windows/windows-poc-runbook.md`（WIN-RG-01..10）要求真实分进程、禁 mock——必须写代码；而 `pc/app/README.md` NO-GO 规定"PoC 留证 + ADR 批准前禁止任何实现文件"，ADR 又依赖 PoC 留证；`agent-hub/prompts/README.md` 同样规定 gate 前"不得启动编码"且未豁免 PoC harness。**PoC 代码的落位目录与豁免授权无人定义，不解开即永久阻塞。**
- **计划失去导航价值**：`clients/plan/milestones.md` 五行全部 `blocked`，无任何可执行起点；真正可操作项散落在 READINESS `next-unblock` 与 progress.md 中。
- **6 Adapter 同批首发不现实**：Hermes/OpenClaw 接口事实缺失（`agent-hub/docs/adapters/capability-matrix.md` 自认待核验）、订阅路径 ToS 未决（risk-register AH-EXT-01/02），缺"最小首发子集"决策。
- **技术栈不收敛**：`pc/docs/architecture/tech-stack-comparison.md` 十维定性、明确不收敛，且未定义收敛机制与期限。
- 正面：`windows-v1-scope.md` §10 + PoC runbook 内容级已可启动，阻在授权而非设计。

### 2.4 交互友好性（8.5）

强项：

- **15 个通用页面状态**（initial-loading / refreshing-last-good / partial / redacted / stale-offline / result-unknown / conflict / privacy-locked / reauth-required 等）配呈现/动作/可访问性三列规则（`information-architecture.md` §9、`journeys-and-screens.md` §4），远超行业常见"有数据/错误"二态设计。
- 旅程规格完整：10 条旅程均含触发、页面序列、用户决定、失败出口、恢复入口、验收（含并发 Owner claim、lease 到期、崩溃恢复等边角）。
- 收件箱模型区分 `unread/acknowledged/handled/expired`，且 `acknowledged` 不能使安全事项从待办消失。
- 无障碍达 AA 级具体度：Narrator+键盘关键旅程、live region 聚合、reduced motion 语义等价、40/44px 目标尺寸；mobile 侧 48dp/TalkBack 细则。

弱项：agent-hub `journeys-and-screens.md` §3 的 13 个 PAGE 各只有一行主任务（仅 PAGE-003 有字段级规格），达不到 pc 侧同等深度；iOS `outcome-unknown` 与全局 `result-unknown` 术语分裂。

### 2.5 UI 美观度（7.0）

强项：品牌命题清晰（"会说人话的控制界面"；签名元素 Governed Flow Thread 以任务线而非渐变发光做记忆点）；完整浅/深色 token、字体（Sora + Segoe UI Variable + Cascadia Mono）、4/8px 网格、动效 token（90/160/240/360ms）与 easing；§9"禁止的通用 AI UI 模式"清单（禁紫色渐变、发光球体、持续脉冲等）品位明确。

弱项：一切停留在文字候选——零视觉稿、零原型、零对比度实测；Risk 色/品牌色仅列值未验证 WCAG；agent-hub 与 mobile 未定义如何继承这套 token（design-system/README 缺口登记自认）。**当前"美观度"只能评设计意图，不能评落地效果。**

## 3. 问题清单

| # | 级别 | 问题 | 位置 | 建议动作 |
|---|---|---|---|---|
| P0-1 | P0 | PoC 授权死锁：PoC 需写代码 vs gate 前禁写代码 vs ADR 依赖 PoC | `pc/app/README.md`、`pc/docs/platforms/windows/windows-poc-runbook.md`、`agent-hub/prompts/README.md` | 出一条决策：PoC harness 代码豁免 + 落位 `poc/`（不入产品目录）+ owner 指认 |
| P0-2 | P0 | 规则执行层失效但文档称"已生效"（虚假证据）| `clients/READINESS.md:23`、`clients/README.md` §9、`clients/plan/risk-register.md`（CLR-5）；实际 `.cursor/rules/` 仅存 qingtian-mcp.mdc | 恢复 16/17 号规则或改用 CI 校验替代，并同步修正 READINESS 证据行 |
| P0-3 | P0 | 状态快照集体过期（84/46/38 与 M5 未完成 vs 实测 85/60/25 与 M5 done、M6 GO）| `clients/README.md:21`、`clients/READINESS.md` §2、`governance/evidence-index.md`、`pc/docs/product/requirements-traceability.md` §1、`desktop-parity-matrix.md` §10、`pc/docs/quality/README.md`、`plan/progress.md` | 收敛单一事实源：计数只留指针指向全局 PROGRESS，全树删除复写数值 |
| P1-1 | P1 | 计划层无可执行起点，可操作项散落 | `clients/plan/milestones.md`、`clients/READINESS.md` | 按本次开发计划补"当前可执行工作面"章节 |
| P1-2 | P1 | 6 Adapter 同批首发不现实、缺最小子集决策 | `agent-hub/docs/adapters/capability-matrix.md`、`agent-hub/plan/risk-register.md` | 决策首发子集（建议 Claude Agent SDK + Codex/OpenCode 二选一），其余降 Tier 2 |
| P1-3 | P1 | 技术栈比较不收敛且无期限 | `pc/docs/architecture/tech-stack-comparison.md` | 绑定收敛机制：PoC 完成后 2 周内出 ADR |
| P1-4 | P1 | agent-hub 页面规格深度不足以开高保真原型 | `agent-hub/docs/product/journeys-and-screens.md` §3 | 对首发旅程补字段级页面规格（对齐 pc 侧模板） |
| P1-5 | P1 | 全树零视觉验证（token 候选未实测 WCAG/High Contrast） | `pc/docs/ux/design-system.md` §10 | Phase 1 启动 token 对比矩阵 + 中英文排版样张 |
| P2-1 | P2 | 断链：contracts-sdk README → 不存在的 11 号规则 | `shared/docs/contracts-sdk/README.md:7` | 改指 History 归档或恢复后的新路径 |
| P2-2 | P2 | Android PoC ID 命名漂移（POC-001..018 vs CONSOLE-AND-V1-POC-001..018）；iOS 术语 outcome-unknown vs result-unknown | `governance/traceability.md` vs `governance/evidence-index.md`；`mobile/ios/docs/ios-product-design.md` | 统一 ID 前缀与状态术语表 |
| P2-3 | P2 | MIGRATION-MAP B8/I1 提交哈希未回填（"本提交"占位） | `clients/MIGRATION-MAP.md` §1 | 回填哈希 |
| P2-4 | P2 | clients/README 索引缺 mobile/ios/plan、mobile/android/plan、shared/plan 行 | `clients/README.md` §3/§4 | 补索引行 |
| P2-5 | P2 | agent-hub 链接文字与实链不一致（迁移残留） | `agent-hub/plan/README.md:5,45` | 修正链接文字 |
| P2-6 | P2 | 元/治理文本占比过高（约 1/3~1/2），gate 清单四处重复、状态免责样板每文件复读 | agent-hub GOVERNANCE §7、master plan §4、`docs/planning/README.md` §2、risk-register | gate 清单收敛到 GOVERNANCE 一处，其余改指针 |

## 4. 重构判定

- **不重构**：pc 四份核心设计、trust-safety-ux、mobile 双平台设计、agent-hub 部署模式/接管架构/威胁模型/追踪链——这些是全树资产，重写只有损失。
- **部分重构（治理与计划层）**：单一事实源收敛（P0-3）、规则执行层修复（P0-2）、计划层重建可执行工作面（P1-1）、重复元文本瘦身（P2-6）。
- **增量修补（设计层）**：PoC 死锁决策（P0-1）、adapter 首发子集（P1-2）、agent-hub 页面规格加深（P1-4）、视觉验证启动（P1-5）、其余 P2 项。

配套开发计划见同目录 [2026-07-26-clients-development-plan.md](2026-07-26-clients-development-plan.md)。
