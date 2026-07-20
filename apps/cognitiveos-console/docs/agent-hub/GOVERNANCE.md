# Agent Hub 文档治理

> 类别：informative governance ｜ 日期：2026-07-20 ｜ owner：Lane-CON（治理文件由 Lane-DOC 协作）

本文件定义 Agent Hub 文档体系的 canonical source、状态用语、owner、版本、更新触发、deprecated/superseded 规则，以及与仓库既有 docs-sync、PROGRESS、findings-ledger、lane ownership、handoff 的双向同步义务。它不产生 CognitiveOS 规范要求。

## 1. canonical source 与 owner

- 每条产品/架构/安全事实只有一个 canonical 文档（见 [README 文档地图](./README.md#2-文档地图)）；其他文档只能引用，不得复制正文。
- canonical owner：
  - 产品/架构/安全/协作/平台/决策/追踪：Lane-CON。
  - 与全局 `docs/README.md`、`docs/plan/*`、`docs/traceability/*`、`AGENTS.md` 的联动：Lane-DOC 协作。
  - normative 机器资产（`specs/**`、`conformance/**`）：一律经 Lane-CTR 契约流程，Agent Hub 文档不得代替。
- 冲突解释顺序遵循仓库硬纪律：机器 schema/registry/transition/vector 与 normative companion > 固定版本 RFC/Core/Profile > 白皮书 > 本产品文档；冲突时取不扩大权限、数据范围、风险、预算或完成声明的解释。

## 2. 四态状态用语

所有文档引用状态时必须使用四态之一，且不得互相替代：

1. 规范已登记（specified）：REQ/schema/vector/transition 在机器资产中存在；
2. 实现已提供（implementation available）：适用代码存在且可构建；
3. 测试已执行（test executed）：runner 真实执行并留证据；
4. Profile 已符合（implemented）：全部适用 MUST 有通过或有据 not-applicable 证据。

产品层附加正交维度：

- Contract：`registered / partial / prose-only / missing / product-only`；
- Implementation：`not-implemented / partial / available`；
- Evidence：`none / not-run / pass / fail / not-applicable / documented-degradation`；
- 产品状态词：`product-only / unregistered / planned / blocked / not-implemented / none / not-run`。

Direct Takeover 专用事实来源标签（不得升级为 authority）：`host-managed`、`provider-reported`、`process-observed`、`terminal-observed`、`file-observed`、`check-observed`、`user-accepted`、`unknown`。

接管结果标签：`managed-from-start`、`officially-adopted`、`terminal-attached`、`read-only-observed`、`unmanaged-observed`、`unsupported`、`blocked-by-policy`。

## 3. ID 规则

- 产品要求：`CONSOLE-AGENTHUB-V1-PRD-*`
- 决策：`CONSOLE-AGENTHUB-V1-DEC-*`
- 旅程：`CONSOLE-AGENTHUB-V1-JRN-*`
- 页面：`CONSOLE-AGENTHUB-V1-PAGE-*`
- 接管层级：`CONSOLE-AGENTHUB-V1-LVL-*`
- 威胁：`CONSOLE-AGENTHUB-V1-TM-*`
- Open PoC：`CONSOLE-AGENTHUB-V1-POC-*`
- 开发任务：`AH-<lane>-<seq>`

规则：

- ID 一经发布不得重用；删除项标 `deprecated` 并保留映射。
- 一个 PRD 只表达一个可独立通过/失败的产品行为。
- 这些 ID 不进入 CognitiveOS normative registry；不得与真实 `REQ-*` 混称。
- 引用真实 `REQ-*` 时必须是 `specs/registry/requirements.yaml` 中存在的条目。

## 4. 版本与更新触发

- 每份 canonical 文档头部标 `日期` 与 `状态`；语义变化时更新日期并在 [decisions/decision-log.md](./decisions/decision-log.md) 或本文件登记。
- 更新触发条件：
  - 产品决策变化 → 先更 decision-log，再更相关专题；
  - 外部平台事实/条款/许可变化 → 更对应 `sources/` ledger 并标查询日；
  - 后端契约（Lane-CTR/CFR/KRN/RUN）语义变化影响设计 → 按 docs-sync-contract 联动并在受影响文档标注；
  - 计数变化 → 从全局 PROGRESS 读取实测数，不沿用旧数。

## 5. deprecated / superseded

- 被取代的决策在 decision-log 追加 `superseded` 记录，写明日期、替代原因、successor ID；旧 ID 与原文保留。
- 被取代的文档在文首加 deprecation banner 并链接 successor；不得静默删除以免断链。
- 平台/移动既有 canonical 决策（`CONSOLE-V2-*`、`CONSOLE-MAC/LNX/IOS/AND-V1-*`）不得被 Agent Hub 文档改写、重编号或覆盖。

## 6. 与仓库既有治理的双向同步

| 仓库机制 | Agent Hub 义务 |
|---|---|
| [docs/standards/docs-sync-contract.md](../../../../docs/standards/docs-sync-contract.md) | 语义/结构型变更执行同批联动；本目录属产品文档分类，至少加漂移标注 |
| [docs/plan/PROGRESS.md](../../../../docs/plan/PROGRESS.md) | 里程碑状态回写全局 PROGRESS；本目录 progress.md 只记局部文档进度 |
| [docs/traceability/findings-ledger.md](../../../../docs/traceability/findings-ledger.md) | 触碰 F/IMP 或登记漂移时同步；本轮登记的计数漂移在此闭合 |
| [docs/plan/PARALLEL-LANES.md](../../../../docs/plan/PARALLEL-LANES.md) | 所有权与 gate 以 lane 表为准；不跨车道改他人 crate/package |
| `docs/checkpoints/*-handoff.md` | 会话结束写 handoff，作为跨会话唯一记忆载体 |

## 7. 实现 gate（不可跳过）

Agent Hub 任一实现里程碑必须同时满足，未满足则相关任务保持 `blocked`：

1. `docs/plan/DEVELOPMENT-PLAN.md` Console 依赖组 1、2、7 已交付；
2. M5 出口评审通过；
3. 目标平台 [Open PoC 与 GA gates](../../../../docs/platforms/README.md#console-实现-gate) 用真实 API/真实 OS 行为留证；
4. 技术栈 ADR 已批准；
5. 适用 machine contract、implementation、executed evidence 分别达到声明门槛；
6. Paseo/AGPL 源码复用完成独立法务与第三方组件义务评估。

禁止用 mock、原型、代码存在或 Agent 自述冒充 gate 已通过或任务已完成。
