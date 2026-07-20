# 客户端文档/结构局部进度

> 类别：plan（局部）｜ owner：Lane-CON ｜ 最后更新：2026-07-20（B8 终审）
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
| readiness review + PROGRESS + handoff（B8） | done（本提交） |
| structure-ready | **yes**（逐项证据见 [READINESS](../READINESS.md)） |
| implementation-ready | **no / blocked**（九项 blocker，见 [READINESS](../READINESS.md)） |

客户端 implementation 均 `not-implemented`；平台/PoC evidence `none`；向量 76 全 `not-run`；Profile `not implemented`。结构就绪不构成实现授权。
