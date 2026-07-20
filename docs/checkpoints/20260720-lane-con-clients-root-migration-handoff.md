# 20260720 Lane-CON clients/ 项目根建立与文档迁移 Handoff

> 写给没有本次对话历史的接续代理。本次为结构型文档迁移（ADR-0007、CLIENTS-DEC-001），**未启动任何实现**；用户已于 2026-07-20 批准文件级迁移方案（任务规范：`docs/prompts/console-client-project-foundation-and-doc-migration.md`）。
>
> 集成说明：本 handoff 的 56 schema / 76 vectors 与 ahead/behind 数字是原分叉分支快照；合入远端 M5 gate 基线时 living 文档已对齐 61 schema / 84 vectors。原分支误用 D-012 记录计数修正，编号冲突与修复统一登记为 D-019；远端 canonical D-012 保持 Lane-CFR traceability 漂移含义。

## 1. 本次会话完成

- **P0 封口上一会话工作**（三笔）：
  - `03250e1b3e3249d2cc600346f2861886a351a079` Agent Hub canonical 文档 41 + Master plan + 车道计划 24 + 提示词 19 + 平台 parity 落库；
  - `8e1de44150c3ded0a7561fd69a7cbf1f7df89ad4` Lane-CON 仓库联动 20 文件 + 旧计数修正（集成改号见 D-019）；
  - `dc55eebcec95916b6bc0971857a628aeb21f173e` agent-hub 设计 handoff（哈希回填）+ 目录索引会话提示词。
- **B1** `dedd082a3a8b3271fd0c106727c4f865d278b0e2`：建立 `clients/` 项目根（README/GOVERNANCE/READINESS/MIGRATION-MAP + 七域 36 份薄 README/治理件/计划件）；`docs/clients/README.md` git mv 为 `clients/README.md` 并降旧址为 stub。
- **B2** `41609ce712412ae45a1074743fc94d0359eef09e`：PC 13 文件 git mv 入 `clients/pc/`（console docs 九份 + docs/platforms 桌面四份），被移文件内链与全部 inbound 引用同批重算。
- **B3** `7591fe8ae837530c0c9e6d3f8ec82193946104bd`：mobile 4 文件 git mv 入 `clients/mobile/`（ios/android 产品设计 + mobile parity + 移动决策日志，32 个决策 anchor 引用改跨树路径）。
- **B4** `8afce717816a075e3639b57ac331d72f1fcc5aeb`：新建 `clients/shared/docs/test-strategy.md` 与 `telemetry-evidence/telemetry-redaction-retention-policy.md`（全部状态如实 not-run/none/not-implemented）。
- **B5** `85331bb21391abe4e6b7199f9dd717e2d06c7f92`：Agent Hub 86 文件 git mv 入 `clients/agent-hub/{docs,plan,prompts}/`（41 docs + master plan + 24 plan + 19 prompts + 平台 parity），深相对链接全量重算。
- **B6** `b2c1f63205ed209234fa8f1fa79a39f0612c9210`：Console 实现 gate canonical 正文迁入 `clients/governance/readiness-gates.md`（加 `console-实现-gate`/`implementation-gate` 双 anchor）；`docs/platforms/README.md` 压成 stub（首行 anchor、gate heading 一字不差）；apps console README/PRODUCT-DESIGN stub 定稿（23 anchor + 漂移登记表原样）；全仓 gate 链接改指新 canonical。
- **B7** `5902a252b5e4b5d80239e353e72f8e50930554e9`：rules 16 更新（canonical 改指 clients）+ 新增 rule 17（客户端项目边界）+ `docs/adr/0007-clients-project-root-and-doc-migration.md` + AGENTS/根 README/docs README/PARALLEL-LANES §2.1&§3/DEVELOPMENT-PLAN/rule 11 最小联动。
- **B8**（本提交）：READINESS 终审（structure-ready yes 逐项证据）、MIGRATION-MAP 哈希回填、PROGRESS 更新、本 handoff、`clients/pc/docs/architecture/README.md` 补齐 §5 目标树。
- 关联条目：`CLIENTS-DEC-001`、ADR-0007；全程无 REQ/错误码/schema/vector 影响（提交信息逐笔注明）。

## 2. 未完成 / 进行中

- 全部客户端实现任务保持 `blocked`（结构就绪≠实现授权）。
- Lane-CFR 自动化缺口：consistency checker 不扫 `clients/`（SCAN_ROOTS/LIVING_SCOPES），任务登记 `planned`，交付前 clients 链接/anchor 检查是手动 gate（`clients/README.md` §9）。
- AH-CTR-02 六 Adapter 一手接口核验未执行（唯一不违反 gate 的可推进 informative 工作）。
- 实测计数与既有文档口径的两处漂移未回改历史文件（见 §4）。

## 3. 测试与证据状态

- `pnpm run check:consistency`：P0 与 B1–B8 每批**通过**——`273 requirements, 55 error codes, 56 schemas, 76 vectors, markdown links and traceability verified`（计数全程不变）。
- `git diff --check`：每批仅 CRLF→LF 警告（Windows 换行），零空白错误。
- clients 专项链接/anchor 检查（临时脚本，不入库）：终审 156 文件 **0 断链/断 anchor**（checker 不扫 clients，此为手动 gate 证据）。
- ReadLints：新建/编辑文件无 linter 错误。
- 向量：76，全部 `not-run`（未改变）；REQ 级实现/测试/Profile 计数仍 0。
- 以上均为静态一致性证据，**不构成**客户端实现、平台 PoC、向量执行或 Profile 证据。

## 4. 未决风险与漂移

- 实测漂移（未回改历史，本次新文档以实测为准）：① `CONSOLE-AGENTHUB-V1-TM-*` 实测 21 项（上一 handoff 记 20）；② `CONSOLE-V2-JRN/PAGE/CMP` 实测 10/19/12（迁移方案文本记 13/25/13）。登记于 `clients/governance/traceability.md`。
- 结构风险见 `clients/plan/risk-register.md`：CLR-1 checker 盲区（Lane-CFR planned）、CLR-2 main 分叉（本地 ahead 30 / behind 43，全部提交仅落本地，未 push）、CLR-3 多代理并发（lane/krn 存在未提交 M4 内核改动与未登记 handoff `20260720-lane-krn-m4-handoff.md`，未纳入本次迁移）、CLR-4 heading 锚点脆弱、CLR-5 旧路径复活风险（由 4 stub + rules 16/17 对冲）。
- readiness 双结论：**structure-ready: yes**（九判据逐项证据见 `clients/READINESS.md` §1）；**implementation-ready: no (blocked)**（依赖组 1/2/7、M5、五平台 PoC、技术栈 ADR、Paseo/AGPL 法务、接口核验、F-003 runner gate、76 向量 not-run、规则禁令）。

## 5. 下一步入口

- 建议提示词：本任务规范 `docs/prompts/console-client-project-foundation-and-doc-migration.md`（已执行完毕，留档）；后续可推进：
  - Lane-CFR：领取 clients 扫描自动化任务（把 `clients/` 纳入 `tools/src/lib.mjs` SCAN_ROOTS + 结构校验 + 注入演练），入口 `clients/READINESS.md` §3；
  - Lane-CON/CTR：`AH-CTR-02` 接口一手核验，入口 `clients/agent-hub/prompts/README.md`。
- 工作分支：main（本地，未 push；与 origin 分叉 ahead/behind，合并策略待用户决定）。
- 第一个动作：读 `clients/README.md` → `clients/READINESS.md` → `clients/MIGRATION-MAP.md`。

## 6. 快照

- PROGRESS 已更新：是（最后更新行、Console 车道行、Lane-CON 行、handoff 列表、客户端目录治理交付表）。
- 本次提交列表：P0 `03250e1` / `8e1de44` / `dc55eeb`；B1 `dedd082`；B2 `41609ce`；B3 `7591fe8`；B4 `8afce71`；B5 `85331bb`；B6 `b2c1f63`；B7 `5902a25`；B8 = 本提交。
