# 车道计划 — Host / Control / Ledger（HOST）

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：blocked
>
> 目标：Takeover Host 拓扑、认证控制面、ownership generation、single controller lease、Local Event Ledger。设计见 [takeover-architecture.md](../../../apps/cognitiveos-console/docs/agent-hub/architecture/takeover-architecture.md)。

## 范围与路径

- 允许（实现激活后）：Agent Hub Host crate/package（新建，命名经 PARALLEL-LANES 登记）、本计划。
- 禁止：他人车道 crate/package、`specs/**`、`conformance/**`、许可证；任意 PID 注入/劫持路径。
- 依赖：GOV、CTR。gate：AH-B1（后端）、AH-B3（ADR）、AH-B5（若复用 Paseo）。

## 任务

### AH-HOST-01 per-user 非提权 Host 骨架
- owner/lane：Lane-CON / HOST｜depends_on：AH-CTR-01｜blocked_by：AH-B1,AH-B3
- 交付物：per-user Host 进程模型；不启用 SeDebug/Backup/Restore/TakeOwnership；不自动 UAC 提权
- 失败测试先行：Host 以高完整性/其他用户目标时拒绝或只读
- 安全负例：管理员 Agent 自动接管被拒
- oracle：跨用户/提权目标不可写控制｜evidence：none

### AH-HOST-02 认证本机控制面
- owner/lane：Lane-CON / HOST｜depends_on：AH-HOST-01｜blocked_by：AH-B3
- 交付物：Windows named pipe（DACL/first-instance/reject-remote/impersonation）；Unix socket（peer cred/pidfd）；不以 loopback 为身份；client 反验 server identity
- 失败测试先行：未授权同用户/跨用户 client 全部动作被拒
- 安全负例：pipe/socket squatting 被 handshake 拒绝（TM-002）
- oracle：POC-SEC-001/002 pass｜evidence：not-run

### AH-HOST-03 ownership generation + single controller lease
- owner/lane：Lane-CON / HOST｜depends_on：AH-HOST-02｜blocked_by：—
- 交付物：lease 字段（target_id/generation/lease_id/controller/scope/expiry/state）；每 mutation 串行事务检查；generation 持久化先于 terminal write/signal
- 失败测试先行：旧 generation 客户端输入被拒
- 安全负例：双 controller 写被拒；stale 输入不重放
- oracle：Host 重启后旧 controller 全被拒｜evidence：none

### AH-HOST-04 Local Event Ledger
- owner/lane：Lane-CON / HOST｜depends_on：AH-HOST-03｜blocked_by：—
- 交付物：记录请求/信号/进程观察/文件快照/Adapter 事件/用户决定/verifier 结果；与 provider 数据分离
- 安全负例：ledger 不冒充 CognitiveOS audit；不写 provider 内部 DB 受管标记
- oracle：ledger 条目含事实来源标签且零 secret｜evidence：none

### AH-HOST-05 崩溃恢复与孤儿对账
- owner/lane：Lane-CON / HOST｜depends_on：AH-HOST-03,AH-HOST-04｜blocked_by：—
- 交付物：fail-closed（Job/cgroup 边界）；重启扫描 `running`；`orphan-suspected` 不按 PID 直杀；durable detach 取舍冻结
- 安全负例：崩溃窗口未确认动作标 unknown，不自动重放（TM-014）
- oracle：POC-PROC-002 pass；强杀 Host 后正常子进程按策略退出｜evidence：not-run
