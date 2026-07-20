# Agent Hub 里程碑

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON
>
> 里程碑均以真实证据为出口 gate；未满足前对应车道 `blocked`。当前处于 AH-M0（文档/计划），后续里程碑未启动。

## AH-M0 — 文档与计划（本轮）

- 目标：canonical 文档、Master 计划、12 宏车道 + 6 Adapter 计划、提示词、契约核验清单。
- 出口：文档一致（anchor/ID/状态用语）、gate 明确、计数无漂移。
- 状态：进行中（文档）；无实现。

## AH-M1 — 接口核验 + 威胁 + PoC

- 目标：六 Adapter 官方接口一手核验；威胁模型逐项 oracle 设计；全部 Open PoC 在真实环境执行并留证；法务 gate。
- 依赖：接口 gate、法务 gate。
- 出口：接口核验完成、PoC pass、Paseo/AGPL 评估完成。
- blocked_by：接口未核验、法务未过、无真实环境。

## AH-M2 — Host/Control/Ledger + Process/Terminal（Windows）

- 目标：Takeover Host、认证控制面、ownership generation、single controller lease、ledger；Windows spawn/身份/Job containment/停止分层/ConPTY。
- 依赖：AH-M1、ADR gate、Console 后端 gate。
- 出口：HOST/PROC 安全负例 pass（squatting、PID reuse、breakaway、stdin 抢占拒绝、kill-on-close）。

## AH-M3 — Session/File + Credential/Workspace/Verifier

- 目标：官方 session 采用（条件写）、只读文件观察（JSONL/SQLite）、opaque credential handle、worktree、checks。
- 依赖：AH-M2。
- 出口：SESS/CRED 安全负例 pass（双 writer 拒绝、symlink/hardlink 逃逸拒绝、零 secret 落盘、一致 snapshot）。

## AH-M4 — Desktop Direct v1

- 目标：Windows 桌面客户端单 Agent 全旅程（发现→接管预览→监督→纠偏→停止→完成双轴→释放）。
- 依赖：AH-M2、AH-M3。
- 出口：桌面无障碍（键盘/Narrator/High Contrast/缩放/reduced motion）+ 安全 + 恢复证据。

## AH-M5 — Relay/Pairing + iPhone

- 目标：E2EE Relay、配对、设备身份、扩权 PC-local 确认、恢复；iPhone companion。
- 依赖：AH-M4。
- 出口：Relay 安全负例 pass（MITM、replay、revoke）+ iOS 无障碍。

## AH-M6 — Multi-Agent + Android + Governed 迁移

- 目标：单层 Lead+Workers 调度器；Android companion；Direct→Governed evidence-only 迁移。
- 依赖：AH-M5、Governed 契约 gate（M6）。
- 出口：全 PoC pass、发布 gate、迁移不追认 authority。
