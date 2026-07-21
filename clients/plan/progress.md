# 客户端文档/结构局部进度

> 类别：plan（局部）｜ owner：Lane-CON ｜ 最后更新：2026-07-21（Phase 0 文档准备收口；等 M5 出口 + 外部设备/账号）
>
> **职责边界**：本文件只记录 `clients/` 文档与结构的局部准备状态。全局工程状态、里程碑、REQ/向量计数、证据声明的唯一真相是 [docs/plan/PROGRESS.md](../../docs/plan/PROGRESS.md)；两者冲突时以全局 PROGRESS 为准。

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
| structure-ready | **yes**（见 [READINESS](../READINESS.md)） |
| implementation-ready | **no / blocked**（见 [READINESS](../READINESS.md)；未改 yes） |

客户端 implementation 均 `not-implemented`；平台/PoC evidence `none`；Agent Hub Open PoC = 28 not-run + 5 planned；全局向量 84（46 `pass` / 38 `not-run`）但客户端平台证据仍为 `none`；Profile `not implemented`。Phase 0 文档准备已尽本地所能（PR #18/#19）；等待上游 M5 出口与外部设备/账号——结构就绪与手册均不构成实现授权。
