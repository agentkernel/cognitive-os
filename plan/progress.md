# 客户端文档/结构局部进度

> 类别：plan（局部）｜ owner：Lane-CON ｜ 最后更新：2026-07-21（PR #21 合入后解阻复核；gate 仍 blocked；进入等待上游）
>
> **职责边界**：本文件只记录 `clients/` 文档与结构的局部准备状态。全局工程状态、里程碑、REQ/向量计数、证据声明的唯一真相是 [docs/plan/PROGRESS.md](https://github.com/agentkernel/cognitive-os/blob/main/docs/plan/PROGRESS.md)；两者冲突时以全局 PROGRESS 为准。

| 项 | 状态 |
|---|---|
| clients/ 骨架与治理件（B1） | done（`dedd082`） |
| PC 文档迁移（B2，13 文件） | done（`41609ce`） |
| mobile 文档迁移（B3，4 文件） | done（`7591fe8`） |
| shared 新文档（B4，2 文件） | done（`8afce71`） |
| Agent Hub 迁移（B5，86 文件） | done（`85331bb`） |
| stub 定稿与 gate canonical（B6） | done（`b2c1f63`） |
| rules 与治理入口联动（B7，ADR-0007） | done（`5902a25`） |
| readiness review + PROGRESS + handoff（B8） | done |
| 远端 M5 gate 基线集成与 D-019 漂移闭合（I1） | done |
| **Phase 0：AH-CTR-02 接口文档级回填** | **done（文档级；evidence not-run）** |
| **Phase 0：POC-LIC 材料整理** | **材料 done；评估 not-run** |
| **Phase 0：威胁 oracle / planned PoC 登记** | **done（设计/登记；零执行）** |
| **Phase 0：PoC 执行手册/模板骨架** | **done（informative；全部 not-run / evidence none）** |
| **Phase 0：技术栈候选比较草案** | **done（非正式 ADR；未批准栈）** |
| **Phase 0：设计系统 planned 缺口登记** | **done（最小登记；无 token 大文件）** |
| **Phase 0：文档准备收口（本地所能）** | **done（2026-07-21）** — 真实 PoC 执行 / 正式 ADR / M5 出口仍 blocked；handoff：`docs/checkpoints/20260721-lane-con-clients-phase0-status-handoff.md` |
| **上游 M5 细监控（只读）** | **done（2026-07-21）** — 见 PR #22 / `20260721-lane-con-m5-monitor-handoff.md`（批 2a 合入前快照） |
| **上游 M5 解阻复核（只读）** | **done（2026-07-21）** — main=`bb5b356`（含 PR #21/#22）；akp 已脱离骨架；runtime/kernel-server 部分脱离；无 m5-milestone-review；依赖组 1/2/7 仍未完整；五项 gate 仍不满足；handoff：`docs/checkpoints/20260721-lane-con-m5-unblock-review-handoff.md` |
| structure-ready | **yes**（见 [READINESS](../READINESS.md)） |
| implementation-ready | **no / blocked**（见 [READINESS](../READINESS.md)；未改 yes） |

客户端 implementation 均 `not-implemented`；平台/PoC evidence `none`；Agent Hub Open PoC = 28 not-run + 5 planned；全局向量计数以 [PROGRESS](https://github.com/agentkernel/cognitive-os/blob/main/docs/plan/PROGRESS.md) 实测为准（2026-07-26：85 份 60 `pass` / 25 `not-run`；M0–M6 出口评审已过，M6 GO-with-explicit-non-claim v0.1），但客户端平台证据仍为 `none`；Profile `not implemented`。Phase 0 文档准备已尽本地所能；剩余等待项：Console gate 依赖组 1/2/7 逐项对账留证 + 外部 PoC/ADR/法务。上游里程碑通过不构成客户端实现授权。
