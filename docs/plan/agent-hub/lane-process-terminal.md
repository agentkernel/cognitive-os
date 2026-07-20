# 车道计划 — Process + Terminal（PROC）

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：blocked
>
> 目标：进程发现/身份/containment/停止分层、Host-owned 终端（ConPTY/pty）。设计见 [process-and-terminal.md](../../../apps/cognitiveos-console/docs/agent-hub/architecture/process-and-terminal.md)。

## 范围与路径

- 允许（激活后）：Process Supervisor / Terminal Broker 模块。
- 禁止：他人车道代码；任意 stdin 抢占、TIOCSTI、ptrace、内存注入、二进制 patch（L8）。
- 依赖：HOST。gate：AH-B1、AH-B2（平台 PoC）、AH-B3。

## 任务

### AH-PROC-01 进程身份锚（防 PID reuse）
- owner/lane：Lane-CON / PROC｜depends_on：AH-HOST-03｜blocked_by：AH-B2
- 交付物：Windows handle+PID+creation time；Linux pidfd+starttime+exe(dev,ino)；macOS PID+start+unique/version+code requirement；动作前重验
- 失败测试先行：PID 复用后旧 token 不命中新进程
- 安全负例：TM-003 零误杀
- oracle：POC-PROC-004 pass｜evidence：not-run

### AH-PROC-02 受控 spawn 与 containment
- owner/lane：Lane-CON / PROC｜depends_on：AH-PROC-01｜blocked_by：AH-B2
- 交付物：Windows suspended→Job→resume + KILL_ON_JOB_CLOSE；Linux setsid+cgroup；env 白名单/独立端口/预算
- 失败测试先行：assignment 前无运行窗口
- 安全负例：Job breakaway/WMI 逃逸被阻止或检测（TM-015）
- oracle：POC-PROC-001/003 pass｜evidence：not-run

### AH-PROC-03 停止语义分层
- owner/lane：Lane-CON / PROC｜depends_on：AH-PROC-02｜blocked_by：—
- 交付物：send/interrupt/cancel/graceful/force kill/kill tree/detach/release 各自建模 + timeout + 状态机
- 失败测试先行：信号后仍存活标 unknown 不显示 stopped
- 安全负例：忽略 Ctrl+C 的 Agent 不被误报停止（TM-017）
- oracle：状态机 requested→…→reaped 完整｜evidence：none

### AH-PROC-04 Host-owned ConPTY（Windows）
- owner/lane：Lane-CON / PROC｜depends_on：AH-PROC-02｜blocked_by：AH-B2
- 交付物：CreatePseudoConsole 自持 master；capture/send/resize/detach 经 Host API 过 lease；OSC/ANSI 清洗；Win10/Win11 teardown 状态机
- 失败测试先行：普通既有 console 不进入 terminal-attached
- 安全负例：TM-004 注入样本清洗；TM-005 外部 PID send 被拒
- oracle：POC-TERM-001/002 pass｜evidence：not-run

### AH-PROC-05 reaping 与退出观察
- owner/lane：Lane-CON / PROC｜depends_on：AH-PROC-02｜blocked_by：—
- 交付物：Windows Job completion/wait；Unix waitpid/pidfd/subreaper；exit code 只入 process-observed
- 安全负例：exit code 不自动映射任务成功
- oracle：压力测试无 Host-owned zombie｜evidence：none
