# 客户端迁移与结构风险登记

> 类别：plan risk register ｜ owner：Lane-CON ｜ 日期：2026-07-20
>
> 只登记 clients/ 迁移与结构风险；产品/技术风险归各产品文档与 [agent-hub risk-register](../agent-hub/plan/risk-register.md)。

| # | 风险 | 影响 | 对冲 | 状态 |
|---|---|---|---|---|
| CLR-1 | consistency checker 不扫 `clients/`（SCAN_ROOTS/LIVING_SCOPES 盲区） | 断链/孤儿引用/虚构路径可长期潜伏 | [clients/README.md §9](../README.md#9-持续维护与手动-gate) 手动 gate + 每批临时脚本链接检查；自动化任务 Lane-CFR `planned` | open |
| CLR-2 | main 分叉：本地 ahead 21+ / behind 43（快照 2026-07-20） | 远端合并时迁移批可能与他人改动冲突 | 全部提交只落本地；不 push/rebase/merge 远端；批次小步提交便于逐批解决冲突 | open |
| CLR-3 | 多代理并发工作区（lane/krn 有未提交内核改动 + 未登记 PROGRESS 的进行中工作） | 共享文件（PROGRESS/findings-ledger）后合并者需 rebase | 迁移只触碰客户端文档域；PROGRESS 更新集中在 B8 单批 | open |
| CLR-4 | heading 锚点脆弱：跨文件 `#heading` 链接依赖 heading 文字不变 | heading 改字即断链且 checker 只查文件存在性不查 anchor | 关键 anchor 用显式 `<a id>` 保全（[MIGRATION-MAP §3](../MIGRATION-MAP.md#3-anchor--id-保全方案)）；手动 anchor 检查纳入每批验证 | open |
| CLR-5 | 迁移窗口内新会话仍按旧路径写文档 | 旧树复活、双 canonical | 4 个兼容 stub 首行 deprecated + rules 16/17 更新（B7）+ handoff 明示新入口 | open |
