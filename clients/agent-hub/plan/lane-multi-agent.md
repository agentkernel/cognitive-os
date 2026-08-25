# 车道计划 — Multi-Agent（MULTI）

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：blocked
>
> 目标：单 Host、一层 Lead+Workers 协作与确定性调度器。设计见 [collaboration/lead-workers.md](../docs/collaboration/lead-workers.md)。

## 范围与路径

- 允许（激活后）：Multi-Agent 调度器模块。
- 禁止：他人车道代码；多层递归/跨 Host/自治 A2A；Lead 直接改 authority 或 Host 控制状态。
- 依赖：HOST、DESK、CRED（worktree）。gate：AH-B1。

## 任务

### AH-MULTI-01 确定性调度器
- owner/lane：Lane-CON / MULTI｜depends_on：AH-HOST-03,AH-CRED-03｜blocked_by：AH-B1
- 交付物：DAG/预算/并发上限/worktree 分配/controller lease/handoff 记录；单 Host 一层
- 安全负例：Lead 只产 proposal，不直接派发/停止/提交
- oracle：POC-COLLAB-001 pass｜evidence：not-run

### AH-MULTI-02 Worker 隔离与冲突
- owner/lane：Lane-CON / MULTI｜depends_on：AH-MULTI-01｜blocked_by：—
- 交付物：每 Worker 独立 worktree/process group/Job/cgroup/env/credential handle/端口；写冲突进 conflict
- 安全负例：停止一个 Worker 不误杀其他进程
- oracle：POC-COLLAB-002 pass｜evidence：not-run

### AH-MULTI-03 handoff 与群组完成
- owner/lane：Lane-CON / MULTI｜depends_on：AH-MULTI-01｜blocked_by：—
- 交付物：handoff 显示 scope/预算/验收；child done 不完成 parent；群组完成语言分开
- 安全负例：POC-COLLAB-003 handoff 无越权
- oracle：parent 完成需自身双轴｜evidence：not-run
