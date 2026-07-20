# clients/ 开发就绪判定（READINESS）

> 类别：informative readiness review ｜ 日期：2026-07-20 ｜ owner：Lane-CON
>
> 判定口径：structure readiness 与 implementation readiness 分开记录；任何真实 gate 未满足时只能得到 `implementation-ready: no (blocked)`。不得为满足任务目标把 `blocked` 改写成 GO。

## 1. Structure readiness

```text
structure-ready: no
status: migration-in-progress（B1 骨架批；B2–B8 未完成）
```

判据清单（B8 终审逐项给证据，全部满足才写 yes）：

| # | 判据 | 当前状态 |
|---|---|---|
| 1 | 目标目录和薄 README 完整（§5 目标树逐目录有 README，无 `.gitkeep` 冒充） | B1 已建骨架；待 B2–B5 迁移后复核 |
| 2 | old→new migration map 完整（覆盖率 100%） | [MIGRATION-MAP.md](MIGRATION-MAP.md) 已列全量映射；迁移执行中 |
| 3 | 无重复 canonical（无两个自称 canonical 的同职责文件） | B1 起 docs/clients 已 stub 化；B6 复核其余 3 个 stub |
| 4 | 产品 ID、anchor、相对链接可达 | 待 B2–B6 链接修复完成后终审 |
| 5 | owner 与 gate 非空（索引各行、各 README） | B1 已填；随批次维护 |
| 6 | 必要文档系统齐全（治理/平台/共享/Agent Hub 四域） | B1 治理件已建；B4 补测试策略与遥测政策；B5 迁 Agent Hub |
| 7 | rules 已生效（16 更新 + 17 新增） | pending-B7 |
| 8 | docs-sync/PROGRESS/handoff 已联动 | pending-B8 |
| 9 | consistency 与 whitespace 检查通过 | 每批执行；B1 批通过后记录 |

## 2. Implementation readiness

```text
structure-ready: no（见上）
implementation-ready: no
status: blocked
blocked-by:
  - Console 后端依赖组 1/2/7 未交付（docs/plan/DEVELOPMENT-PLAN.md Console 节：组1 Shell/Management/Watch API 与 AKP envelope→M5；组2 AgentExecution/Task/Verification 生命周期载体→M2+M4+M5；组7 AuditRecord/StateSnapshot/reconciliation→M4~M6；当前 M1 in-progress）
  - M5 出口评审不存在（M5 not-started）
  - 五平台 Open PoC / GA gate 全部 not-run、evidence none：macOS MAC-POC-01..12、Linux LNX-POC-01..12、iPhone IOS-POC-01..18、Android POC-001..018 均未执行；Windows 无独立编号 PoC 表（gate 依赖 windows-v1-scope §10 release gate 与依赖组交付）
  - PC/iOS/Android 技术栈 ADR 不存在（Tauri 2 + React/TS 仅为候选，非批准 ADR）
  - Agent Hub Paseo/AGPL 法务 gate 未过（POC-LIC-001/002/003 全部 not-run）
  - Agent Hub Tier 1 provider 一手接口核验未完成（Hermes/OpenClaw 无一手证据，Codex/OpenCode/Claude Agent SDK 待核验；AH-CTR-02 未执行）
  - P0 F-003 剩 Lane-CFR runner 真实执行负例向量的 gate（findings-ledger）
  - 76 份向量全部 not-run；REQ 级实现已提供 / 测试已执行 / Profile 已符合计数均为 0（PROGRESS 实测）
  - 规则明令 gate 前禁实现（PARALLEL-LANES §2.1、.cursor/rules/16、平台 README 激活前例外条款）
next-unblock:
  - 推进 M1（Lane-CFR runner 执行能力）→ M2/M4/M5 依赖组交付
  - 不违反 gate 的 informative 工作：AH-CTR-02 六 Adapter 接口一手核验（docs/prompts/agent-hub/ 对应提示词）
  - 技术栈 ADR 需在平台 PoC 留证后提交批准
```

## 3. 自动化缺口登记

- `tools/src/lib.mjs` `SCAN_ROOTS` 与 `tools/src/check-consistency.mjs` `LIVING_SCOPES` 均不含 `clients/`：`pnpm run check:consistency` 不校验 `clients/**` 的链接、REQ 引用、结构与覆盖率。
- 自动化任务：owner **Lane-CFR**，状态 `planned`；范围=把 `clients/` 纳入扫描根 + 目录索引"真实路径、必填字段、唯一 canonical、覆盖率"校验；按 [docs-sync-contract §5](../docs/standards/docs-sync-contract.md) 修改检查器必须附注入演练输出。
- 交付前手动 gate 生效：每个触碰 `clients/**` 的 PR 按 [clients/README.md §9](README.md#9-持续维护与手动-gate) 手动核对路径/链接/anchor/必填字段，并运行 `check:consistency` + `git diff --check`。
- 静态检查只能证明目录/链接/追踪一致，不能写成客户端实现、平台 PoC、向量执行或 Profile 证据。
