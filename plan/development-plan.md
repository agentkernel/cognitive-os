# 客户端开发计划（operative）

> 类别：plan ｜ owner：Lane-CON ｜ 日期：2026-07-26 ｜ 来源：[2026-07-26 评审版开发计划](../review/2026-07-26-clients-development-plan.md)（分析与依据见该文与 [design review](../review/2026-07-26-clients-design-review.md)）
>
> 本文是计划域的执行版：只登记任务、状态与出口判据。排期不构成实现授权；所有实现项受 [readiness-gates](../governance/readiness-gates.md) 与 [Agent Hub gate](../agent-hub/docs/GOVERNANCE.md#7-实现-gate不可跳过) 约束。PoC 代码唯一豁免见 [CLIENTS-DEC-002](../governance/decision-log.md#clients-dec-002-poc-证据采集代码豁免与落位)。

## Phase 0 — 文档修复与治理收敛（进行中）

| ID | 任务 | 状态（2026-07-26） |
|---|---|---|
| P0-T1 | PoC 死锁解除决策（CLIENTS-DEC-002 登记；rules/17、pc/app/README、agent-hub prompts README、agent-hub decision-log 同批修订） | **done** |
| P0-T2 | 规则执行层修复（`.cursor/rules/11、16、17` 恢复至活动目录；READINESS 证据行修正） | **done** |
| P0-T3 | 单一事实源收敛（全树硬编码计数改 PROGRESS 指针；READINESS blocked-by 刷新） | **done** |
| P0-T4 | 计划层重建导航（milestones 增"当前可执行工作面"；本文件落位并入索引） | **done** |
| P0-T5 | `clients/` 纳入 `check:consistency` 自动扫描 | `planned`（owner Lane-CFR，跨车道，见 READINESS §3） |
| P0-T6 | 小修批：MIGRATION-MAP B8/I1 哈希回填（需 git）、Android PoC ID 前缀统一、iOS `result-unknown` 术语统一 | `planned` |
| P0-T7 | 元文本瘦身：agent-hub gate 清单收敛至 GOVERNANCE §7 一处，其余改指针 | `planned` |

出口：T1–T7 全部完成 + `check:consistency`（含 clients/）绿灯。

## Phase 1 — 解阻三线（可并行启动）

| 线 | 内容 | 出口 |
|---|---|---|
| A（主线，Lane-CON） | 按 [windows-poc-runbook](../pc/docs/platforms/windows/windows-poc-runbook.md) 执行 WIN-RG-01..10，代码落位 `poc/windows/`，证据按 [poc-execution-record](../shared/docs/poc-execution-record.md) 留档；留证后 2 周内基于 [tech-stack-comparison](../pc/docs/architecture/tech-stack-comparison.md) 收敛 PC 技术栈 ADR；同批产出 design-system token 对比矩阵（浅/深/High Contrast + 中英文样张） | PoC 证据留档、ADR 批准、token 矩阵过 WCAG 2.2 AA |
| B（外部依赖，尽早发起） | POC-LIC-001..003（Paseo/AGPL）法务评估执行留证；Anthropic/OpenAI/Apple 等外部确认发起并跟踪 | 法务 gate 结论（通过/不通过均为出口） |
| C（Lane-CON + 治理） | Tier 1 首发 6→2 决策（建议 Claude Agent SDK + Codex/OpenCode 按 B 线结论二选一，其余降 Tier 2）；首发 adapter 旅程补字段级页面规格 | 决策登记 + capability-matrix 标注 + 页面规格评审通过 |

## Phase 2 — Windows v1 MVP（CL-M1；前置：Console 实现 gate 全绿）

垂直切片 S1 信任与身份 → S2 创建与监督（S2 后 S3 暂停与托盘 / S4 Agent 生命周期 / S5 不确定性与降级可并行）；切片与 JRN/PAGE 映射、安全验收（"错误完成声明数为零"）与北极星分母建立见评审版 §3。

## Phase 3 — Agent Hub Direct Desktop v1（AH-M2..M4；前置：Phase 1B/1C 出口 + AH gate）

顺序照 [Master plan](../agent-hub/plan/agent-hub-development-plan.md)：HOST/PROC 骨架（安全负例先行）→ SESS/CRED → Desktop Direct v1 全旅程（含首发 2 adapter）。L6 保持阻断、L8 永久禁止。

## Phase 4 — Mobile companion 与桌面 parity

iOS 先行（IOS-POC-01..18 → iOS ADR → CL-M3），Android 随后（CL-M4）；macOS/Linux parity（CL-M2）按各自 §13 gate；Relay/Pairing（AH-M5）为 mobile 远程路径前置；手机侧保持 remote companion 边界（仅 R0/R1）。

## 里程碑对照与验收

对照表、风险与度量沿用评审版 §6–§8，不在此复写。状态推进唯一回写点：本表状态列 + [PROGRESS](https://github.com/agentkernel/cognitive-os/blob/main/docs/plan/PROGRESS.md)（gate/readiness 结论变化时）。
