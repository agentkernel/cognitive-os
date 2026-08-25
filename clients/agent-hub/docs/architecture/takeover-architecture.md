# Agent Hub — Takeover Host 架构

> 类别：informative architecture ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 状态：architecture design / implementation not-implemented / evidence none。本文是产品架构推论，机器安全义务须由 normative contract 与实现证据闭合。

## 1. 推荐拓扑

```text
PC/Mobile Client
  → authenticated local IPC 或 E2E Relay
  → Takeover Host（per-user，非提权）
  → Provider Adapter（L1 官方 / L3 session / L4 terminal / L5 file / L7 observe）
  → Agent process / session / files
```

首发 Host 身份：当前交互用户下的 per-user、非提权 Host（[CONSOLE-AGENTHUB-V1-DEC-013](../decisions/decision-log.md)）。不启用 `SeDebugPrivilege`/`SeBackupPrivilege`/`SeRestorePrivilege`/`SeTakeOwnershipPrivilege`；不支持管理员 Agent 的自动 UAC 提权。Session-0 service + per-user broker 仅作未来企业选项，需独立安全评审。

## 2. Host 内部职责分离

| 组件 | 职责 | 禁止 |
|---|---|---|
| Control API | 客户端身份、target binding、request ID、sequence、anti-replay、expiry、rate limit、capability、device revoke | 把 loopback 可达性当身份 |
| Process Supervisor | spawn、process group、Job Object/cgroup/launchd、signal、terminate、reap、crash recovery、ownership generation | 任意 PID 注入/劫持 |
| Terminal Broker | ConPTY/PTY/tmux、stdin/stdout、ANSI/OSC 清洗、capture/send/resize/detach | 抢占任意既有 console |
| Session Adopter | 官方 list/import/resume/fork、provider-native handle、session ownership、history hydration、release/handoff | 无 exclusive-lease 证明即写接管 |
| File Observer | 只读 snapshot、parser、lock、digest、watch、rotation、gap、敏感字段裁剪 | 写 provider auth/internal DB |
| Workspace Manager | cwd、repo、branch、worktree、文件范围、端口、dev server、artifact、conflict | 用 cwd 推导 ownership |
| Credential Broker | opaque handle、短期注入、scope、revoke、rotation、audit metadata | 导出 provider secret |
| Local Event Ledger | 记录 Host 接受的请求/信号/进程观察/文件快照/Adapter 事件/用户决定/verifier 结果 | 冒充 CognitiveOS audit |
| Verifier | 运行固定 build/test/lint/diff/file digest/remote query/artifact validation | 仅接受 Agent 自述 |

## 3. 接管层级安全条件

层级枚举见 [../product/deployment-modes-and-guarantees.md](../product/deployment-modes-and-guarantees.md)。逐层最低条件：

- **L2 Host Launched**：Host spawn 前验证 executable identity（handle+PID+creation time+canonical path+publisher/signature+version），suspended→Job/cgroup assign→resume，固定 cwd/env/account/预算，观察退出并管理 crash/restart/reap。进程/终端/身份细节见 [process-and-terminal.md](./process-and-terminal.md)。
- **L3 Official Session Adopted**：候选发现与最终 adopt 分开；写接管前证明旧 runtime inactive 或取得供应商 exclusive lease/fencing，否则退回 read-only。详见 [session-and-file-adoption.md](./session-and-file-adoption.md)。
- **L4 Managed Terminal Attached**：仅 Host 创建的 ConPTY（Windows v1）；未来 tmux/screen 需独立私有 socket、Host-owned、记录 server identity 与 pane ID；终端文本仍不可信。
- **L5 Read-only File Observed**：opt-in、documented root、版本化 parser、一致 snapshot、digest、敏感字段裁剪；不升级为可信提交或完成。
- **L7 Observe-only Process**：只观察 executable canonical path/publisher/signature/version/owner/cwd/parent/child tree/start time/CPU/memory/health；无受支持 IPC/PTY/session handle 时不得发送输入、抢占终端、修改文件或宣称接管成功。
- **L6 / L8**：L6 v1 阻断（仅未来条件能力）；L8 永久禁止。

## 4. ownership generation 与 single controller lease

- 每个可写目标只能存在一个 active controller lease。
- 以下操作都推进 ownership generation：takeover、handoff、release、用户抢回、Host 重启、session resume、Adapter reload。
- lease 字段至少含：`target_id`（进程用 `handle+PID+creation time+canonical image`；终端用 `socket/server identity + pane ID`；文件用 `root identity + dev+ino + parser/schema version`）、`ownership_generation`（单调）、`lease_id`、`controller_id`、`scope`、`expiry`、`state`。
- 每个 mutation 在同一串行事务内检查：target identity 一致、lease active 未过期、controller 匹配、expected generation 匹配、request ID 未执行。
- generation/ledger 持久化提交先于 terminal write 或 signal；terminal keystroke 不可安全重试，超时标 `unknown`，先 capture/reconcile。
- 旧客户端输入必须被拒绝；进程从 Host 启动时立即登记 ownership；启动超时、提前退出、Host 崩溃和关机必须可清理或恢复。
- 每个 Agent 使用独立 process group、Job Object/cgroup、worktree、环境、credential handle、端口范围；停止受管 Agent 不得误杀用户其他进程。

## 5. 本机控制面安全

- 本地控制面不得只依赖 `127.0.0.1` 可达性。
- Windows：优先 named pipe，随机高熵名称、`nMaxInstances=1`、`FILE_FLAG_FIRST_PIPE_INSTANCE`、`PIPE_REJECT_REMOTE_CLIENTS`、不可继承 handle、`SE_DACL_PROTECTED` DACL 仅授当前 logon SID 最小 rights；连接后取 client PID/session、`ImpersonateNamedPipeClient` 后校验 token（失败即拒绝，revert 失败即终止）。
- macOS/Linux：Unix socket owner/mode（私有目录 `0700`、socket `0600`）、`SO_PEERCRED`/`SO_PEERPIDFD`/`LOCAL_PEERCRED`/XPC peer requirement；abstract socket 不进入默认控制面。
- 确需 TCP 时：独立认证、TLS、Host/origin 检查、DNS rebinding 防护、限流、anti-replay；不在 URL 保存长期密码。
- PC、手机、CLI、Web、自动化客户端分别授予最小能力；不得把所有配对设备视为等权 operator。
- 已知限制：同一 OS 用户内，logon SID/owner 只隔离其他用户，不隔离同用户恶意进程；强隔离需独立 OS principal 或容器，产品 UI 不得声称已替代。

## 6. 进程接管最低条件

必须：用户确认目标 Agent、PID/session；显示账号、cwd、workspace、接管层级、可执行动作、不保证事项。同时校验：OS owner、executable canonical path、publisher/signature、version、process start time、parent/child tree、PID reuse、当前 terminal/session ownership。默认优先新建受控子进程；不得默认抢占用户正在交互的普通终端。

分别建模：send、Ctrl+C、SIGINT、SIGTERM、TerminateProcess、kill tree、detach、release。发送信号不等于 Agent 已停止。任何 attach 不得跳过 Agent permission、账号限制、OS 权限、供应商 safety gate。

## 7. 文件接管最低条件

必须区分：provider-owned source of truth、Host mirror、Host ledger、read-only index、用户产物、worktree、credential/auth 文件。只读 parser 版本化；未知字段/半写/truncate/rotation/WAL 未同步/schema drift/lock 冲突/损坏/parser error 进入 partial/unknown。打开文件前检查 real path、owner、mode/ACL、symlink、reparse point、hardlink、大小、类型、设备文件、跨用户边界、路径穿越。默认禁止直接修改 credential/auth/entitlement/policy/approval/history/provider 内部数据库/provider completion 状态。Host ledger 与 provider 文件分离；Host 不得在 provider 数据库伪造“受管”标记。文件变化只表示“观察到磁盘变化”，不证明 Agent 完成、Effect 闭合、结果正确或用户接受。

## 8. Host 崩溃与恢复

- 首发 fail-closed：Job/cgroup 边界随 Host 关闭；Windows `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`；ledger write-ahead。
- Host 重启后扫描 `running` 记录：PID 不存在/creation time 不同 → 记录旧 generation 消失；同 identity 仍存活 → `orphan-suspected`，不按 PID 直接杀，需重新证明身份与归属；不可检查 → 保留待下次对账。
- L2 raw PTY：Host master FD 丢失后不能找回同一 master，进程即使存活也只能 `channel-lost/observe-only` 或按预设策略终止。
- L4 tmux/screen：server 持有 master，可在验证原 server/socket/session identity 后恢复 client。
- 任何 crash 窗口内已发送未确认的输入/信号/外部动作均为 `unknown`，不得自动重放。durable detach 与 fail-closed crash recovery 的取舍需在实现前冻结，不得静默关闭 kill-on-close。

## 9. 与 Governed 的关系

Governed 模式下 Takeover Host 语义被 authority 取代：进程是 Runtime projection，任何控制仍经 Task/AgentExecution/Effect gate；第三方 Agent 作为受治理 Adapter（`REQ-AGENT-*`），不绕过确定性入口。Direct Host ledger 不是 authority；迁移见 [relay-pairing-and-migration.md](./relay-pairing-and-migration.md)。
