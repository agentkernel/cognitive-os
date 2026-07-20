# 20260720 Lane-CON Agent Hub 直连接管设计 Handoff

> 写给没有本次对话历史的接续代理。本次为 informative 文档 + 计划 + 提示词工作，**未启动任何实现**。

## 1. 本次会话完成

- 建立 Agent Hub canonical 文档体系（`apps/cognitiveos-console/docs/agent-hub/`）：
  - 治理：`README.md`、`GOVERNANCE.md`、`progress.md`、`planning/README.md`；
  - 产品：`product/{product-design,deployment-modes-and-guarantees,journeys-and-screens,states-content-and-accessibility}.md`；
  - 架构：`architecture/{takeover-architecture,process-and-terminal,session-and-file-adoption,relay-pairing-and-migration}.md`；
  - 安全：`security/{threat-model,security-and-credentials,computer-control,licensing-and-terms}.md`（20 项 `CONSOLE-AGENTHUB-V1-TM-*`）；
  - 协作/平台/决策/追踪：`collaboration/lead-workers.md`、`platforms/product-scope.md`、`decisions/decision-log.md`（26 项 `CONSOLE-AGENTHUB-V1-DEC-*`）、`traceability/{product-requirements,evidence-index}.md`（28 项 PRD、27+ Open PoC 全 not-run）；
  - Adapter：`adapters/{README,capability-matrix,interface-layering,other-tiers}.md` + `adapters/tier1/` 六份 dossier；
  - 来源：`sources/{README,provider-interfaces-ledger,terms-and-licenses-ledger,platform-security-ledger,paseo-and-comparables-ledger}.md`；
  - 模板：`templates/{adapter-dossier,source-record,threat-record,open-poc,development-task}.md`。
- 开发编排：`docs/plan/agent-hub-development-plan.md`（Master）+ `docs/plan/agent-hub/`（README/progress/milestones/dependency-dag/risk-register/evidence-index + 12 宏车道 + 6 Adapter 子车道）+ `docs/prompts/agent-hub/`（README + 12 宏车道提示词 + 6 Adapter 提示词，全部 `blocked`）。
- 最小仓库联动（保留既有 ID/anchor）：`apps/cognitiveos-console/README.md`、`PRODUCT-DESIGN.md`（§17/§20.3 anchor 未动）、8 份 Console docs 薄入口；`docs/README.md`、`docs/plan/DEVELOPMENT-PLAN.md`、`docs/plan/PARALLEL-LANES.md`、`docs/platforms/README.md` + desktop/mobile parity + 新建 `docs/platforms/agent-hub-platform-parity.md`；入口提示词标记为已执行。
- 计数漂移修正（findings-ledger D-012）：`AGENTS.md`、`docs/plan/DEVELOPMENT-PLAN.md`、findings-ledger IMP-17 三处 74→76，对齐 `check:consistency` 实测。
- 关联条目：`CONSOLE-AGENTHUB-V1-*`（产品 informative，不进 registry）；D-012（findings-ledger 漂移）；IMP-01（规范表面冻结遵守）。

## 2. 未完成 / 进行中

- 全部 Agent Hub 实现任务 `blocked`，未启动（设计即交付物）。
- Hermes、OpenClaw 官方接口无一手核验，dossier 标 `待核验`，对应 Adapter 车道硬前置 `blocked`。
- Codex/OpenCode/Claude Agent SDK 官方一手接口仍需核验（当前多来自竞品间接观察）。

## 3. 测试与证据状态

- `pnpm run check:consistency`：**通过**——`273 requirements, 55 error codes, 56 schemas, 76 vectors, markdown links and traceability verified`。
- `git diff --check`：仅 CRLF→LF 警告（Windows 换行），无空白错误。
- ReadLints：新建/编辑文件无 linter 错误。
- 向量：76，全部 `not-run`（未改变）。
- Agent Hub 证据：implementation `not-implemented`；Open PoC 全 `not-run`；evidence `none`；Profile `not implemented`。
- 五路只读终审：Paseo/许可、进程/终端/session/file、凭据/Relay/远控、UX/无障碍、仓库边界/链接/追溯——已并发执行；高置信问题已按结果最小修正。

## 4. 未决风险与漂移

- 漂移：D-012（74→76 向量计数）已登记并闭合（closed-by-doc-repair）。
- 风险（见 `docs/plan/agent-hub/risk-register.md`）：接口频繁变更（AH-R2）、双 writer（AH-R3）、same-UID 绕过（AH-R4）、AGPL 复用义务（AH-R5）、平台差异虚假安全声明（AH-R6）、Hermes/OpenClaw 接口缺失（AH-R10）。
- 全局 gate 未过：Console 后端组 1/2/7 + M5、平台 PoC/GA、技术栈 ADR、接口一手核验、Paseo/AGPL 法务、Governed 契约。

## 5. 下一步入口

- 建议提示词：`docs/prompts/agent-hub/README.md`（按车道选具体提示词，全部 `blocked`，只可做接口核验/文档，不得编码或 mock 解阻）。
- 优先可推进（不违反 gate 的 informative 工作）：CTR 车道 `AH-CTR-02` 六 Adapter 接口一手核验 → 补齐 `sources/provider-interfaces-ledger.md` 与各 dossier 的 `待核验`。
- 工作分支：随 Lane-CON PR。
- 第一个动作：读 `apps/cognitiveos-console/docs/agent-hub/README.md` → `GOVERNANCE.md` → `decisions/decision-log.md`。

## 6. 快照

- PROGRESS 已更新：是（Agent Hub 独立 hunk；里程碑/车道/handoff 列表）。
- 本次提交列表：batch1 `03250e1b3e3249d2cc600346f2861886a351a079`（Agent Hub canonical 文档 + Master plan + 车道计划 + 提示词）；batch2 `8e1de44150c3ded0a7561fd69a7cbf1f7df89ad4`（仓库联动 + D-012 计数修正）；batch3 本提交（handoff + 目录索引会话提示词）。
