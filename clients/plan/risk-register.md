# 客户端迁移与结构风险登记

> 类别：plan risk register ｜ owner：Lane-CON ｜ 日期：2026-07-20
>
> 只登记 clients/ 迁移与结构风险；产品/技术风险归各产品文档与 [agent-hub risk-register](../agent-hub/plan/risk-register.md)。

| # | 风险 | 影响 | 对冲 | 状态 |
|---|---|---|---|---|
| CLR-1 | consistency checker 不扫 `clients/`（SCAN_ROOTS/LIVING_SCOPES 盲区） | 断链/孤儿引用/虚构路径可长期潜伏 | [clients/README.md §9](../README.md#9-持续维护与手动-gate) 手动 gate + 每批临时脚本链接检查；自动化任务 Lane-CFR `planned` | open |
| CLR-2 | main 长期分叉导致客户端迁移与远端 M1–M4/F-011 状态冲突 | 迁移批可能覆盖工程真相 | 本地历史已推送远端备份；I1 从最新 origin/main squash 集成、排除独立博客并闭合 D-019 | closed |
| CLR-3 | 多代理并发工作区曾存在未登记 Lane-KRN 改动 | 共享文件与里程碑状态漂移 | Lane-KRN 已以 `4c372ae`、M4 handoff 和 PR #12 完成交付；I1 保留远端状态 | closed |
| CLR-4 | heading 锚点脆弱：跨文件 `#heading` 链接依赖 heading 文字不变 | heading 改字即断链且 checker 只查文件存在性不查 anchor | 关键 anchor 用显式 `<a id>` 保全（[MIGRATION-MAP §3](../MIGRATION-MAP.md#3-anchor--id-保全方案)）；手动 anchor 检查纳入每批验证 | open |
| CLR-5 | 迁移窗口内新会话仍按旧路径写文档 | 旧树复活、双 canonical | 4 个兼容 stub 首行 deprecated + rules 16/17 更新（B7）+ handoff 明示新入口 | open |
