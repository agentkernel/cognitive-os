# clients/ 开发就绪判定（READINESS）

> 类别：informative readiness review ｜ 日期：2026-07-20（B8 终审；**2026-07-21 Phase 0 文档进展注记**）｜ owner：Lane-CON
>
> 判定口径：structure readiness 与 implementation readiness 分开记录；任何真实 gate 未满足时只能得到 `implementation-ready: no (blocked)`。不得为满足任务目标把 `blocked` 改写成 GO。**implementation-ready 仍为 no。**

## 1. Structure readiness

```text
structure-ready: yes
```

逐项证据（B8 实测；Phase 0 未改变结构结论）：

| # | 判据 | 结果 | 证据 |
|---|---|---|---|
| 1 | 目标目录和薄 README 完整 | 满足 | clients/ 树以 Markdown 文档为主、零实现源码/manifest；保留目录均有薄 README |
| 2 | old→new migration map 覆盖率 100% | 满足 | [MIGRATION-MAP §2](MIGRATION-MAP.md#2-old--new-逐文件映射) 全部行 `done` |
| 3 | 无重复 canonical | 满足 | 兼容 stub 全部带 deprecated + successor；唯一项目地图=clients/README.md |
| 4 | 产品 ID、anchor、相对链接可达 | 满足 | 交付前按 [README §9](README.md#9-持续维护与手动-gate) 手动 gate + `check:consistency` |
| 5 | owner 与 gate 非空 | 满足 | [clients/README.md](README.md) 各域表字段非空 |
| 6 | 必要文档系统齐全 | 满足 | 治理/计划/shared/PC/mobile/Agent Hub docs-plan-prompts |
| 7 | rules 已生效 | 满足 | `.cursor/rules/16` + `17-client-project-boundaries.mdc` |
| 8 | docs-sync/PROGRESS/handoff 已联动 | 满足 | Phase 0 handoff 见 `docs/checkpoints/20260721-lane-con-clients-phase0-audits-handoff.md` |
| 9 | consistency 与 whitespace 检查通过 | 满足 | 每 PR：`pnpm run check:consistency` + `git diff --check` |

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
  - Agent Hub Paseo/AGPL 法务 gate 未过（POC-LIC-001/002/003 全部 not-run；评估材料已整理，法务评估未执行）
  - Agent Hub Tier 1 runtime/PoC 未闭合（AH-CTR-02 文档级六 provider 一手已回填；Hermes 指认 decided；Adapter 实现仍 blocked，待 PoC + 条款 + 后端/ADR）
  - 当前 84 份向量中 46 pass / 38 not-run；已执行的 M1–M4 证据不覆盖客户端平台行为，Profile 已符合计数仍为 0
  - 规则明令 gate 前禁实现（PARALLEL-LANES §2.1、.cursor/rules/16、.cursor/rules/17、readiness-gates canonical）
  - 21 项威胁已规范登记，oracle/evidence 全 not-run（非「威胁项实测」）；新 planned PoC：POC-HOST-001/TERM-003/PROC-005/RELAY-005/GOV-001
next-unblock:
  - 启动/推进 M5 Lane-RUN/KRN/TSC/CFR 并完成依赖组 1/2/7 与出口评审
  - Lane-CFR 领取 clients 扫描自动防漂移任务
  - 已完成（informative）：AH-CTR-02 六 Adapter 接口一手核验（文档级）——ledger + dossiers 已回填
  - 下一步 informative：POC-LIC 法务评估执行并留证；Tier 1 / 平台 Open PoC 真实执行；技术栈 ADR（PoC 留证后）
  - 外部阻断：Anthropic 订阅自动化确认、OpenAI ChatGPT 包装意见、Apple PLA、非 AGPL 复用 Paseo 双许可（见 agent-hub risk-register）
```

## 3. 自动化缺口登记

- `tools/src/lib.mjs` `SCAN_ROOTS` 与 `tools/src/check-consistency.mjs` `LIVING_SCOPES` 均不含 `clients/`：`pnpm run check:consistency` 不校验 `clients/**` 的链接、REQ 引用、结构与覆盖率。
- 自动化任务：owner **Lane-CFR**，状态 `planned`；范围=把 `clients/` 纳入扫描根 + 目录索引"真实路径、必填字段、唯一 canonical、覆盖率"校验；按 [docs-sync-contract §5](../docs/standards/docs-sync-contract.md) 修改检查器必须附注入演练输出。
- 交付前手动 gate 生效：每个触碰 `clients/**` 的 PR 按 [clients/README.md §9](README.md#9-持续维护与手动-gate) 手动核对路径/链接/anchor/必填字段，并运行 `check:consistency` + `git diff --check`。
- 静态检查只能证明目录/链接/追踪一致，不能写成客户端实现、平台 PoC、向量执行或 Profile 证据。
