# CognitiveOS 客户端开发计划（2026-07-26 修订版）

> 类别：informative plan proposal（非 canonical；不改写 `clients/plan/`、`agent-hub/plan/` 既有 canonical 计划，被采纳的条目应按治理流程回写对应文件）
>
> 依据：[同日设计评估报告](2026-07-26-clients-design-review.md)；全局状态基准：`docs/plan/PROGRESS.md`（M0–M5 done，M6 出口 GO-with-explicit-non-claim，85 向量 60 pass / 25 not-run）
>
> 原则：不违背既有 gate 语义——本计划不把任何 `blocked` 改写为 GO，而是给出**解阻路径**与解阻后的实施顺序。文档完成度不是实现证据；PoC/ADR/法务证据留档要求全部保留。

## 0. 计划总览

```text
Phase 0  文档修复与治理收敛      （≈1–2 周，无外部依赖，立即可启动）
Phase 1  解阻三线：PoC+ADR / 法务 / Adapter 子集 （≈2–4 周，部分依赖外部账号与法务）
Phase 2  Windows v1 MVP（CL-M1）  （≈6–10 周，依赖 Phase 1 出口）
Phase 3  Agent Hub Direct Desktop v1（AH-M2..M4）（依赖 Phase 1/2 部分出口）
Phase 4  Mobile companion（iOS 先行）+ macOS/Linux parity（CL-M2..M4）
```

关键判断：当前唯一真正卡死全局的是 **PoC 授权死锁**（评审 P0-1）与**治理层事实失真**（P0-2/P0-3）。二者均可在无任何外部依赖的情况下 1–2 周内解决；解开后 Windows PoC → 技术栈 ADR → CL-M1 是一条清晰主线。

## 1. Phase 0 — 文档修复与治理收敛（部分重构）

目标：让文档系统重新可信、让计划重新可导航。全部为文档/工具工作，不触碰实现 gate。

| ID | 任务 | 产出 / 出口判据 | 对应问题 |
|---|---|---|---|
| P0-T1 | **PoC 死锁解除决策**：新增一条 decision（clients/governance/decision-log.md + agent-hub decision-log 同步）：PoC harness 代码属"证据采集工具"，豁免"gate 前禁实现"；落位仓库根 `poc/`（或 `tools/poc/`），明确不得进入 `clients/*/app/`、不得被引用为产品代码；owner 指认 Lane-CON | 决策登记生效；`pc/app/README.md`、`agent-hub/prompts/README.md` 补豁免措辞 | P0-1 |
| P0-T2 | **规则执行层修复**：将 `History/.cursor/rules/16、17`（及 11）恢复至活动 `.cursor/rules/`，或改由 CI 校验替代；同步修正 `clients/READINESS.md:23` 证据行、`clients/README.md` §9、risk-register CLR-5 | 活动规则文件存在或替代校验落地；READINESS 证据行与事实一致 | P0-2 |
| P0-T3 | **单一事实源收敛**：全树删除复写的向量/REQ 计数数值（≥7 处），一律改为指向 `docs/plan/PROGRESS.md` 的指针句；刷新 `clients/READINESS.md` blocked-by（移除"M5 未完成"，保留仍真实的 PoC/ADR/法务/依赖组剩余项） | `clients/**` 内 grep 不到硬编码计数；READINESS 与 PROGRESS 无矛盾 | P0-3 |
| P0-T4 | **计划层重建导航**：`clients/plan/milestones.md` 增补"当前可执行工作面"章节（列 Phase 0/1 任务与 owner），保留原 blocked 表 | 读 milestones 一页即知"现在能做什么" | P1-1 |
| P0-T5 | **clients/ 纳入自动扫描**：落实 READINESS §3 已登记的 Lane-CFR 任务——`tools/src/lib.mjs` SCAN_ROOTS 与 `check-consistency.mjs` LIVING_SCOPES 加入 `clients/`，含链接可达、必填字段、唯一 canonical 校验 | `pnpm run check:consistency` 覆盖 clients/ 并绿灯 | P0-2 |
| P0-T6 | **小修批**：修断链（contracts-sdk README）、回填 MIGRATION-MAP B8/I1 哈希、补 clients/README 索引缺行、修 agent-hub plan/README 链接文字、统一 Android PoC ID 前缀与 `result-unknown` 术语 | 逐项修复并过 P0-T5 扫描 | P2-1..5 |
| P0-T7 | **元文本瘦身**：agent-hub gate 清单收敛至 GOVERNANCE §7 一处，master plan / planning README / risk-register 改指针；状态免责样板压缩为单行引用 | 重复 gate 定义仅剩 1 处 canonical | P2-6 |

出口：全部 T1–T7 完成 + `check:consistency`（含 clients/）绿灯。**Phase 0 完成前不启动任何 PoC 之外的新设计文档。**

## 2. Phase 1 — 解阻三线（可并行）

### 2.1 A 线：Windows PoC + 技术栈 ADR（主线，Lane-CON）

1. 按 `pc/docs/platforms/windows/windows-poc-runbook.md` 执行 WIN-RG-01..10（真实 Windows Service/IPC、WebView2 隔离、UAC、托盘/lease、Narrator、资源预算），代码落位 P0-T1 决定的 `poc/`；证据按 `shared/docs/poc-execution-record.md` 模板留档。
2. PoC 全部留证后 **2 周内**基于 `tech-stack-comparison.md` 收敛技术栈 ADR（候选 Tauri 2 + React/TS vs 备选），终结"不收敛无期限"状态（评审 P1-3）。
3. 同批产出 design-system 验证第一批：浅/深/High Contrast token 对比矩阵 + 中英文排版样张（design-system §10 第 1–2 项，评审 P1-5）。

出口：WIN-RG-01..10 证据留档；PC 技术栈 ADR 批准；token 对比矩阵过 WCAG 2.2 AA。

### 2.2 B 线：法务与外部确认（依赖外部，尽早启动）

- POC-LIC-001/002/003（Paseo/AGPL）法务评估执行并留证；
- Anthropic 订阅自动化、OpenAI 包装意见、Apple PLA 等外部阻断项（见各 runbook 与 agent-hub risk-register）发起并跟踪；
- 出口：法务 gate 结论（通过/不通过均为出口，不通过则相关 adapter 降级或移除）。

### 2.3 C 线：Adapter 首发子集决策（Lane-CON + 治理）

- 决策 Tier 1 首发从 6 缩至 **2**：建议 Claude Agent SDK（接口一手核验最完整）+ Codex 与 OpenCode 中据 B 线结论二选一；Hermes/OpenClaw/OpenHands 降 Tier 2（接口核验补齐后再升）；
- agent-hub 对首发 2 个 adapter 涉及的旅程补字段级页面规格（对齐 pc journeys-and-screens 模板，评审 P1-4），并明确复用 Console design-system 的 canonical 指针与差异清单；
- 出口：决策登记 + capability-matrix 标注 Tier 调整 + 首发旅程页面规格评审通过。

## 3. Phase 2 — Windows v1 MVP（CL-M1）

前置：Console 实现 gate 全绿（依赖组 1/2/7 剩余项 + Phase 1A 出口）。按垂直切片交付，每片含实现 + 状态矩阵 + 无障碍 + 自动化验收（journeys-and-screens §6 场景转为 Playwright/原生辅助技术测试）：

| 切片 | 内容 | 对应旅程/页面 |
|---|---|---|
| S1 信任与身份 | Service 安装/验证、endpoint key、Owner 领取、登录、identity-change 阻断 | JRN-001/002；PAGE-001/002/003/018/019 |
| S2 创建与监督 | Shell 对话、Intent 固定、Preview/R1、任务详情五轨、Flow Thread | JRN-003/008；PAGE-004/005/006/007 |
| S3 暂停与托盘 | pause_pending 语义、托盘监督、退出请求暂停、lease 到期恢复 | JRN-004/005；PAGE-014 + tray |
| S4 Agent 生命周期 | 受控获取（SSRF/broker）、包检查、安装/升级/回滚/卸载、兼容性报告 | JRN-006/007；PAGE-009/010/011/012/013 |
| S5 不确定性与降级 | OUTCOME_UNKNOWN 对账、store/audit/watch 降级、系统概览、对象记录 | JRN-009/010；PAGE-008/015/016/017 |

规则：S1→S2 严格顺序；S3/S4/S5 可在 S2 后并行。每片出口必须含"错误完成声明数为零"类安全验收（product-brief §7.3：安全失败不可被产品指标抵消）。北极星指标采集（受验证任务收敛数）在 S2 落地，Alpha 期只建分母不设目标值。

## 4. Phase 3 — Agent Hub Direct Desktop v1（AH-M2..M4）

前置：Phase 1B/1C 出口 + AH 六类 gate 中接口/法务/ADR 项落实。顺序照 agent-hub master plan：HOST/PROC 骨架（AH-M2，安全负例先行）→ SESS/CRED（AH-M3）→ Desktop Direct v1 单 Agent 全旅程（AH-M4，含首发 2 adapter）。复用 Phase 2 的 design-system 实测产物；L6 保持阻断、L8 永久禁止不变。

## 5. Phase 4 — Mobile companion 与桌面 parity

- iOS 先行（与 agent-hub 首发顺序一致）：IOS-POC-01..18 → iOS ADR → CL-M3；Android 随后（POC 18 项 → ADR → CL-M4）；
- macOS/Linux parity（CL-M2）：MAC-POC-01..12 / LNX-POC-01..12 → 各自产品设计 §13 gate；
- Relay/Pairing（AH-M5）为 mobile 远程路径前置，手机侧继续遵守 remote companion 边界（仅 R0/R1，R2/R3 只解释阻断）。

## 6. 里程碑对照

| 本计划 | 对应 canonical 里程碑 | 解阻条件（不变） |
|---|---|---|
| Phase 0 | —（文档/工具工作，无 gate） | 无 |
| Phase 1A/1B/1C | Console 实现 gate 的 PoC/ADR/法务分量；AH-M1 部分 | 真实执行留证 |
| Phase 2 | CL-M1 | Console 实现 gate 全绿 |
| Phase 3 | AH-M2..M4 | AH 六类 gate |
| Phase 4 | CL-M2/M3/M4、AH-M5..M6 | 各平台 PoC/GA gate + ADR |

## 7. 风险

| 风险 | 影响 | 缓解 |
|---|---|---|
| P0-T1 决策被治理层否决 | PoC 死锁延续，全线停摆 | 备选：PoC 移入独立仓库，仅证据回填本仓 |
| 依赖组 1/2/7 剩余项交付晚于 Phase 1A | Phase 2 起点后移 | S1 切片先按 PoC 环境开发，对真实 server 的集成测试后置到依赖交付 |
| 法务 gate 不通过（AGPL/ToS） | adapter 子集再缩水 | C 线决策已按"逐 adapter 独立 gate"设计，单点失败不阻塞其余 |
| 设计系统实测推翻候选 token | 视觉返工 | Phase 1A 即做对比矩阵，先于任何高保真投入 |
| 治理瘦身引入链接回归 | 文档可信度受损 | P0-T5 自动扫描先行合入，瘦身批次在扫描保护下进行 |

## 8. 验收与度量

- Phase 0：`check:consistency`（含 clients/）绿灯；READINESS 与 PROGRESS 零矛盾；PoC 决策生效。
- Phase 1：PoC 证据留档率 100%；ADR 批准；token 矩阵 AA 达标；adapter 决策登记。
- Phase 2：北极星（每周活跃操作者受验证任务收敛数）分母建立；`pause_requested→paused` 时延、R1 理解正确率、错误完成声明数（必须为 0）按 product-brief §7.2 采集。
- 全程：四类状态用语（规范已登记/实现已提供/测试已执行/Profile 已符合）不混用；任何阶段完成声明附证据路径。
