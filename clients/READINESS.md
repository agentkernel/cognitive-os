# clients/ 开发就绪判定（READINESS）

> 类别：informative readiness review ｜ 日期：2026-07-20（B8 终审）｜ owner：Lane-CON
>
> 判定口径：structure readiness 与 implementation readiness 分开记录；任何真实 gate 未满足时只能得到 `implementation-ready: no (blocked)`。不得为满足任务目标把 `blocked` 改写成 GO。

## 1. Structure readiness

```text
structure-ready: yes
```

逐项证据（B8 实测）：

| # | 判据 | 结果 | 证据 |
|---|---|---|---|
| 1 | 目标目录和薄 README 完整 | 满足 | clients/ 树实测 146 个文件全部为 Markdown、零非 md 文件、零 `.gitkeep`；§5 目标树全部保留目录（pc/app、mobile/{ios,android}/app、plan/prompts/governance/shared 各域、pc/docs 缺内容子目录 architecture/accessibility/quality/release）均有薄 README；内容子目录（如 pc/docs/product、agent-hub/docs/adapters）由所属树根 README 索引 |
| 2 | old→new migration map 覆盖率 100% | 满足 | [MIGRATION-MAP §2](MIGRATION-MAP.md#2-old--new-逐文件映射) 全部行 `done`：B1 索引 1 + B2 PC 13 + B3 mobile 4 + B5 Agent Hub 86 + B6 gate 正文迁移 + B4 新建 2；§1 批次哈希已回填 |
| 3 | 无重复 canonical | 满足 | 4 个兼容 stub（docs/clients、apps console README、PRODUCT-DESIGN、docs/platforms/README）全部带 deprecated + successor 且无正文复制；唯一项目地图=clients/README.md；gate canonical 唯一=governance/readiness-gates.md；各域清单见 [canonical-sources](governance/canonical-sources.md) |
| 4 | 产品 ID、anchor、相对链接可达 | 满足 | `check:consistency` 每批 OK（含活文档断链检查）；clients 专项临时脚本终审 156 文件 0 断链/断 anchor；PRODUCT-DESIGN 23 个显式 anchor 原样、`implementation-gate`/`console-实现-gate` 双别名在新 canonical 生效；ID 计数实测总表见 [traceability](governance/traceability.md) |
| 5 | owner 与 gate 非空 | 满足 | [clients/README.md](README.md) 各域表 platform/role/status/owner/canonical/gate/README 列全部非空；各薄 README 带 owner/状态/gate |
| 6 | 必要文档系统齐全 | 满足 | 治理 7 件（GOVERNANCE + governance/ 六件）、计划 5 件（plan/ 四件 + 各域 plan README）、shared 六域 + test-strategy + telemetry 政策、PC/mobile 产品树、Agent Hub docs/plan/prompts 三子树 |
| 7 | rules 已生效 | 满足 | `.cursor/rules/16` 已改指 clients/README.md 并覆盖 `clients/**` 触发；`.cursor/rules/17-client-project-boundaries.mdc` 新增（B7，`5902a25`） |
| 8 | docs-sync/PROGRESS/handoff 已联动 | 满足 | 每批提交说明关联 CLIENTS-DEC-001/ADR-0007 并注明无 REQ 影响；PROGRESS 与 handoff 随 B8 同批更新（见 [MIGRATION-MAP §1](MIGRATION-MAP.md#1-批次与提交哈希)） |
| 9 | consistency 与 whitespace 检查通过 | 满足 | B1–B8 原分支每批检查通过；集成到远端 M5 gate 基线后复验以 273/55/61/84 为准，`git diff --check` 零空白错误 |

结构就绪不构成任何实现授权；下述 implementation gate 全部未满足。

## 2. Implementation readiness

```text
structure-ready: yes
implementation-ready: no
status: blocked
blocked-by:
  - Console 后端依赖组 1/2/7 尚未完整交付（M1–M4 已完成且 F-011 R1 合同已登记；M5 runtime/management/AKP/Shell 集成与组7剩余项待实现）
  - M5 入口 gate 已达成，但 M5 实现与出口评审尚未完成
  - 五平台 Open PoC / GA gate 全部 not-run、evidence none：macOS MAC-POC-01..12、Linux LNX-POC-01..12、iPhone IOS-POC-01..18、Android POC-001..018 均未执行；Windows 无独立编号 PoC 表（gate 依赖 windows-v1-scope §10 release gate 与依赖组交付）
  - PC/iOS/Android 技术栈 ADR 不存在（Tauri 2 + React/TS 仅为候选，非批准 ADR）
  - Agent Hub Paseo/AGPL 法务 gate 未过（POC-LIC-001/002/003 全部 not-run）
  - Agent Hub Tier 1 provider 一手接口核验未完成（Hermes/OpenClaw 无一手证据，Codex/OpenCode/Claude Agent SDK 待核验；AH-CTR-02 未执行）
  - 当前 84 份向量中 46 pass / 38 not-run；已执行的 M1–M4 证据不覆盖客户端平台行为，Profile 已符合计数仍为 0
  - 规则明令 gate 前禁实现（PARALLEL-LANES §2.1、.cursor/rules/16、.cursor/rules/17、readiness-gates canonical）
next-unblock:
  - 启动 M5 Lane-RUN/KRN/TSC/CFR 并完成依赖组 1/2/7 与出口评审
  - Lane-CFR 领取 clients 扫描自动防漂移任务
  - 不违反 gate 的 informative 工作：AH-CTR-02 六 Adapter 接口一手核验（clients/agent-hub/prompts/ 对应提示词）
  - 技术栈 ADR 需在平台 PoC 留证后提交批准
```

## 3. 自动化缺口登记

- `tools/src/lib.mjs` `SCAN_ROOTS` 与 `tools/src/check-consistency.mjs` `LIVING_SCOPES` 均不含 `clients/`：`pnpm run check:consistency` 不校验 `clients/**` 的链接、REQ 引用、结构与覆盖率。
- 自动化任务：owner **Lane-CFR**，状态 `planned`；范围=把 `clients/` 纳入扫描根 + 目录索引"真实路径、必填字段、唯一 canonical、覆盖率"校验；按 [docs-sync-contract §5](../docs/standards/docs-sync-contract.md) 修改检查器必须附注入演练输出。
- 交付前手动 gate 生效：每个触碰 `clients/**` 的 PR 按 [clients/README.md §9](README.md#9-持续维护与手动-gate) 手动核对路径/链接/anchor/必填字段，并运行 `check:consistency` + `git diff --check`。
- 静态检查只能证明目录/链接/追踪一致，不能写成客户端实现、平台 PoC、向量执行或 Profile 证据。
