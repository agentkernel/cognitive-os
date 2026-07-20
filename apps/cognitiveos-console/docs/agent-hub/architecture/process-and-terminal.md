# Agent Hub — 进程与终端管理

> 类别：informative architecture ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 平台事实来源与查询日见 [../sources/platform-security-ledger.md](../sources/platform-security-ledger.md)（Microsoft Learn / man pages / Apple 文档，查询日 2026-07-19/20）。所有能力 `not-implemented / evidence none`。

## 1. 进程身份（防 PID reuse）

- PID 在所有桌面平台都会复用；产品对象禁止只锚定 PID。
- 目标身份元组：
  - Windows：open handle + PID + `ProcessStartTime`（handle 存活期身份稳定）；
  - Linux：`pidfd_open` + `/proc/<pid>` 元数据交叉验证；
  - macOS：PID + `proc_pidinfo` 启动时间 + code signature（audit token 仅适用于 IPC 对端）。
- 附加校验：executable canonical path、publisher/signature、version、OS owner、cwd、parent/child tree。
- 校验完成到动作执行之间存在固有 TOCTOU 窗口：Windows 凭 handle 消除；Linux 凭 pidfd 消除；仅有 PID 的路径必须在动作前重验 creation time，且 UI 标注该限制。

## 2. spawn 与进程组管理

### 2.1 Windows

- `CreateProcess` + `CREATE_SUSPENDED` → `AssignProcessToJobObject` → resume，消除 assignment 前逃逸窗口。
- Job Object 设 `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`；Host 崩溃即随 handle 关闭终止整棵树（fail-closed）。
- Windows 8+ 支持嵌套 Job；子进程加入新 Job 失败的情形要登记 `containment-degraded`，不得静默。
- `TerminateProcess`/`TerminateJobObject` 是强制终止，不给优雅清理机会；先走协作停止序列。

### 2.2 macOS / Linux

- spawn 后 `setsid()` 新会话/进程组；组信号 `kill(-pgid)`。
- Linux 加 cgroup v2 作 containment 与资源账；`cgroup.kill`（Linux 5.14+）整组终止；PR_SET_PDEATHSIG 只保护直接子进程，不可作为整树保证。
- macOS 无 cgroup；依赖 process group + launchd（未来服务化）；双 fork 逃逸只能靠 ledger 对账补偿。
- 逃逸窗口差异必须在平台 parity 中如实呈现：Windows Job ≈ 强 containment；Linux cgroup ≈ 强；纯 POSIX 进程组 = 弱（双 fork 可逃逸）。

### 2.3 环境与账号

- spawn 固定：cwd、env 白名单（不继承 Host 全部环境）、credential handle 短期注入、独立端口范围、资源预算。
- 不跨 OS 用户 spawn；不默认以管理员运行；UAC 提权目标不支持自动接管。

## 3. 停止语义（分别建模）

| 动作 | 语义 | 平台机制 | 结果保证 |
|---|---|---|---|
| send input | 向受管终端/官方通道发输入 | ConPTY write / SDK interrupt | 无停止保证 |
| interrupt | 请求当前 turn 中断 | Ctrl+C（ConPTY 序列）/ SIGINT | Agent 可拒绝/忽略 |
| cancel turn | 官方 API 取消当前 turn | provider API | provider-reported |
| stop session | 官方 API 结束 session | provider API | provider-reported |
| graceful stop | 协作退出 | SIGTERM（Unix）；Windows 无通用等价，console 组可尝试 CTRL_BREAK，GUI 用 WM_CLOSE | 超时后升级 |
| force kill | 强制终止单进程 | `TerminateProcess` / SIGKILL | 不可拒绝；无清理 |
| kill tree | 终止整个进程组/Job | `TerminateJobObject` / `kill(-pgid)` / `cgroup.kill` | 树内全部终止 |
| detach | 断开控制不终止 | 关闭 PTY client / 释放 lease | 进程继续运行 |
| release | 释放 ownership | ledger 记录 + generation 推进 | 后续输入被拒 |

规则：发出信号 ≠ 已停止；每个动作都有超时与升级路径；超时后状态 `unknown` 并进入对账。Ctrl+C 在 ConPTY 中是向 PTY 写入中断序列，与 Unix SIGINT 语义不同，文案不得混称。

## 4. reaping 与退出观察

- Windows：Job completion port / `WaitForSingleObject` 收 exit code；handle 保持期间 PID 不复用。
- Unix：`waitpid`/`pidfd` 收割 zombie；Host 是 subreaper（Linux `PR_SET_CHILD_SUBREAPER`）时才能观察孙进程退出，否则孙进程 reparent 到 init，仅 ledger 级观察。
- exit code 只进入 `process-observed`；不自动映射为任务成功/失败。

## 5. 终端架构

### 5.1 Windows v1：Host-owned ConPTY

- Host 用 `CreatePseudoConsole` 创建 ConPTY，自持 master 端；client 只经 Host API 读写。
- capture、send、resize（`ResizePseudoConsole`）、detach、interrupt 都是 Host API 上的显式动作，逐动作过 lease/generation 检查。
- ConPTY 输出是 VT 序列渲染流，不是逐字节原始 stdout；OSC/ANSI 清洗后再入 ledger 与 UI；终端内容永远标注不可信。
- Windows 10 22H2 的 conhost/ConPTY 行为与 Windows 11 Terminal 栈存在差异；22H2 仅 ESU + Experimental，行为差异必须逐版本记录，不外推。

### 5.2 普通既有 console 不可安全抢占

- Windows 没有安全的“接管别人 console”的公开机制；`FreeConsole`/`AttachConsole` 注入方案属 L8 禁止面。
- 既有普通进程只能：L7 观察、以 L2 重新启动、或走 L3 官方 resume。

### 5.3 未来 tmux/screen（macOS/Linux）

- 仅 Host 创建的独立私有 socket（`tmux -S`，目录 `0700`）；记录 server PID/identity、session/pane ID。
- attach 前重验 socket owner/mode 与 server identity；不 attach 用户默认 socket 的既有会话。
- screen 同理；无法验证 identity 时只提供只读 capture（如可行）或拒绝。

## 6. 崩溃与恢复（进程/终端专述)

- raw PTY master FD 丢失不可找回：Host 崩溃后 L2 终端进入 `channel-lost`，进程按预设策略（fail-closed kill-on-close 或 documented orphan）处理。
- tmux/screen：server 存活时 Host 重启后可重新 attach，须重验 server/socket/session identity 并推进 generation。
- 崩溃窗口内未确认的 keystroke/信号一律 `unknown`；不自动重放；恢复顺序：resnapshot → 对账 → 用户决定。

## 7. Open PoC（进程/终端）

以下断言在实现前必须用真实 OS 行为验证，当前全部 `not-run / evidence none`（PoC 清单 canonical 于 [../traceability/evidence-index.md](../traceability/evidence-index.md)）：

- `POC-PROC-001` Windows suspended→Job→resume 无逃逸窗口实测；
- `POC-PROC-002` kill-on-job-close 在 Host 崩溃（非正常退出）路径确实触发；
- `POC-PROC-003` Linux cgroup.kill 对双 fork 树的整树终止；
- `POC-PROC-004` pidfd/handle 在高频 PID 复用压力下的身份稳定性；
- `POC-TERM-001` ConPTY capture/send/resize/detach 全动作在 Win11 24H2/25H2 与 Win10 22H2 ESU 的行为差异表；
- `POC-TERM-002` OSC/ANSI 注入清洗对已知 escape-injection 样本的覆盖。
