# clients/ 治理规则

> 类别：informative governance ｜ 日期：2026-07-20 ｜ owner：Lane-CON（治理文件由 Lane-DOC 协作）

本文件定义 `clients/` 客户端项目根的 canonical 唯一性、状态用语、owner 权威、文档联动、deprecated/superseded 与 ID 规则。它不产生 CognitiveOS 规范要求，也不替代 [AGENTS.md](../AGENTS.md)、[PARALLEL-LANES](../docs/plan/PARALLEL-LANES.md) 或 [docs-sync-contract](../docs/standards/docs-sync-contract.md)。

## 1. canonical 唯一性

- 每条客户端产品/平台/架构/安全事实只有一个 canonical 文档；各域 canonical 清单以 [governance/canonical-sources.md](governance/canonical-sources.md) 为准。
- 其他文档（含旧路径兼容 stub、`docs/README.md`、各 README）只能链接 canonical，不得复制正文形成并行事实源。
- 禁止出现两个自称 canonical 的同职责文件；发现即按 [MIGRATION-MAP.md](MIGRATION-MAP.md) 与本文件 §5 处置。
- 机器合同（registry/schema/transition/vector 与 normative companion）只在 `specs/**`、`conformance/**`；`clients/` 一律只消费、只引用。

## 2. 四类状态用语

状态口径与 [AGENTS.md](../AGENTS.md) 四类状态用语表完全一致，不得互相替代：

1. 规范已登记（specified）：REQ/schema/vector/transition 在机器资产中存在；
2. 实现已提供（implementation available）：适用代码存在且可构建；
3. 测试已执行（test executed）：runner 真实执行并留证据；
4. Profile 已符合（implemented）：全部适用 MUST 有通过或有据 not-applicable 证据。

未启动一律用 `planned` / `blocked` / `not-implemented`；未执行一律用 `none` / `not-run`。目录、README、计划或提示词的存在不表示任何一类状态成立。

## 3. owner 权威

- owner 与车道 gate 的唯一权威是 [PARALLEL-LANES §3 所有权表](../docs/plan/PARALLEL-LANES.md#3-所有权表当前)；[governance/ownership.md](governance/ownership.md) 只是指针矩阵，不复制、不另立事实。
- `clients/**` 文档域归 Lane-CON（治理由 Lane-DOC 协作）。`apps/agent-shell`、`packages/sdk-ts` 归 Lane-TSC，`packages/contracts-ts` 归 Lane-CTR，`tools/` 归 Lane-CFR；未经所属车道批准不移动、不改写。
- 接口变更只能经 Lane-CTR 契约流程；`clients/` 文档发现契约缺口时登记 findings-ledger 并通告对应车道，不代替登记。

## 4. docs-sync 并入声明

`clients/**` 全部文档并入 [docs-sync-contract](../docs/standards/docs-sync-contract.md) 的三分类联动义务：

- 修正型（typo/断链/计数）：改动本体 + 提交说明注明；
- 语义型（状态、gate、边界、canonical 职责变化）：同批更新 `clients/README.md` 索引行、受影响 README、[READINESS.md](READINESS.md)、PROGRESS 与相关 stub；
- 结构型（目录新增/更名/删除、canonical 迁移）：再加 ADR 与 [MIGRATION-MAP.md](MIGRATION-MAP.md) 登记。

`.cursor/rules/16-client-directory-index.mdc` 的同批义务与手动 gate（[clients/README.md §9](README.md#9-持续维护与手动-gate)）全程适用。`pnpm run check:consistency` 当前不扫 `clients/`（自动化缺口登记见该 §9 与 READINESS），交付前以手动链接检查代偿。

## 5. deprecated / superseded 规则

沿用 Agent Hub GOVERNANCE §5 模式（迁移后见 `clients/agent-hub/docs/GOVERNANCE.md`）：

- 被取代的文档在文首加 deprecation banner：`deprecated` 声明 + successor 链接；正文不保留、不复制，必要时保留既有显式 anchor 与 heading 作为 alias。
- 被取代的决策在对应 decision log 追加 `superseded` 记录（日期、原因、successor ID）；旧 ID 与原文保留，不重编号、不重用、不删除。
- 兼容 stub 清单与其保全的 anchor 以 [MIGRATION-MAP.md](MIGRATION-MAP.md) 为准；stub 不得反向宣称自己是 canonical。
- 静默删除会造成断链，禁止；先 stub 化，确认无 inbound 引用后方可在后续批次评估移除。

## 6. `CLIENTS-DEC-*` ID 规则

- `clients/` 结构与治理决策使用 `CLIENTS-DEC-*` 命名空间，canonical 定义点是 [governance/decision-log.md](governance/decision-log.md)；从 `CLIENTS-DEC-001` 起顺序编号，一经发布不重用。
- 该命名空间只覆盖客户端项目根的结构/治理决策；产品决策仍归四本产品决策日志（Console v2、桌面平台、移动平台、Agent Hub），不得混入。
- 既有产品 ID（`CONSOLE-V2-*`、`CONSOLE-{MAC,LNX,IOS,AND}-V1-*`、`CONSOLE-AGENTHUB-V1-*`、`AH-*`）不重编号、不重用、不删除；命名空间总表见 [governance/traceability.md](governance/traceability.md)。
- 这些 ID 不进入 CognitiveOS normative registry，不得与真实 `REQ-*` 混称。

## 7. 本地进度与全局 PROGRESS 边界

- 全局工程状态、里程碑、计数的唯一真相是 [docs/plan/PROGRESS.md](../docs/plan/PROGRESS.md)；
- [plan/progress.md](plan/progress.md) 只记录客户端文档/结构的局部准备状态，不承载里程碑推进、REQ 计数或证据声明；
- 状态变化（gate 过/不过、readiness 结论变化）必须回写全局 PROGRESS 并按会话协议写 handoff。
