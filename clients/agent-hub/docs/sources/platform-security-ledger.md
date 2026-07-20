# Source Ledger — 平台安全原语

> 类别：informative research source ledger ｜ 查询日：2026-07-19/20 ｜ owner：Lane-CON
>
> 事实来自官方一手资料（Microsoft Learn / Sysinternals、Linux man-pages / kernel.org、Apple 开发者与 XNU 源码、SQLite 官方）。推论明确标注。产品下界见 [../architecture/process-and-terminal.md](../architecture/process-and-terminal.md)、[../security/security-and-credentials.md](../security/security-and-credentials.md)。全部能力 implementation=not-implemented、evidence=none。

## 1. Windows（查询日 2026-07-20，Microsoft Learn / Sysinternals）

### 1.1 IPC 身份

- Named pipe 默认 SD 向 LocalSystem/Administrators/creator 授完全控制、Everyone/Anonymous 读；应用 logon SID 限制。[Named Pipe Security and Access Rights]
- `PIPE_REJECT_REMOTE_CLIENTS` 拒远程；`FILE_FLAG_FIRST_PIPE_INSTANCE` 防同名后续实例。[CreateNamedPipe]
- 可取 client PID/session/server PID；`ImpersonateNamedPipeClient` 用最后读取消息的安全上下文，失败不得执行请求；`RevertToSelf` 失败须关闭进程。[GetNamedPipeClientProcessId / …SessionId / …ServerProcessId / ImpersonateNamedPipeClient / RevertToSelf]
- SQOS：`CreateFile` 可选 `SECURITY_IDENTIFICATION/IMPERSONATION`、tracking、effective-only。[CreateFile / GetTokenInformation]

产品影响：控制管道禁默认 SD、禁授 `FILE_GENERIC_WRITE`；先读定长 hello 再 impersonate；per-user 只接受 identification level。

### 1.2 进程身份与 Job

- PID 仅存活期唯一；进程 handle 存活即身份稳定；`GetProcessTimes` 取创建时间；Job completion 的 PID 可能已回收。[Process Handles and Identifiers / GetProcessTimes / JOBOBJECT_ASSOCIATE_COMPLETION_PORT]
- `QueryFullProcessImageName` 取路径；`GetFinalPathNameByHandle` 取规范路径；`WinVerifyTrust` 精确返回 0 才可信。[QueryFullProcessImageNameW / GetFinalPathNameByHandleW / WinVerifyTrust]
- `CREATE_SUSPENDED`→`AssignProcessToJobObject`→`ResumeThread` 消除逃逸窗口；`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` 关闭即杀树；Windows 8+ 嵌套 Job；WMI 创建的子进程不自动入 Job。[Process Creation Flags / AssignProcessToJobObject / Job Objects / Nested Jobs]

产品影响：进程身份=`(handle, PID, creation time, canonical image, signer, version)`；containment 用 Job + kill-on-close；WMI/service/计划任务默认禁用。

### 1.3 ConPTY

- pseudoconsole 须在创建目标进程前创建，Host 持 input write/output read 端并持续 drain。[Creating a Pseudoconsole session / CreatePseudoConsole]
- `ReleasePseudoConsole`（Win11 24H2 build 26100+）只放弃 ownership，仍须 `ClosePseudoConsole`；24H2 起 Close 立即返回，早期可能无限等待。[ReleasePseudoConsole / ClosePseudoConsole]
- `AttachConsole` 只让调用进程成为既有 console 的 client，不提供原始 PTY ownership。[AttachConsole]

产品影响：仅 Host-owned ConPTY 支持 L4；普通既有 console 不可安全抢占（归 L7 或重启）；Win10/Win11 teardown 状态机不同。

### 1.4 信号/文件/平台基线

- `CTRL_C_EVENT` 不可靠定向；`CTRL_BREAK_EVENT` 面向同 console 组；`TerminateProcess` 异步、不清理、不杀子；`TerminateJobObject` 杀 Job 及嵌套。[GenerateConsoleCtrlEvent / TerminateProcess / Terminating a Process / TerminateJobObject]
- `FILE_FLAG_OPEN_REPARSE_POINT` 打开 reparse 本体；硬链接共享 SD；`FILE_ID_INFO` 判同一文件但 file ID 可跨时间复用；`PathCchCanonicalizeEx` 仅字符串规范化不证明 sub-path identity。[Reparse Points and File Operations / CreateFile / FILE_ID_INFO / PathCchCanonicalizeEx]
- 平台：Win11 25H2 build 26200（滚动补丁）；24H2 build 26100（Home/Pro 2026-10-13 EOL）；Win10 22H2 build 19045 常规支持 2025-10-14 结束，仅 ESU。`RtlGetVersion` 取真实 build。[Supported versions of Windows client / Windows 11 release information / Windows 10 release information / Extended Security Updates / RtlGetVersion]

产品影响：停止分层建模；文件检查围绕同一 handle；GA 基线 25H2、按 edition/build/补丁动态 gate。

## 2. Linux（查询日 2026-07-20，man-pages 6.18 / kernel.org / torvalds tree）

- Unix socket：pathname socket 受父目录+inode mode 控制，新 socket mode `0777 & ~umask`；abstract socket 无文件权限保护；`SO_PEERCRED` 连接时缓存 PID/EUID/EGID；`SO_PEERPIDFD`（6.5+）直接取 peer pidfd 避免 PID reuse 窗口。[unix(7) / SO_PEERPIDFD commit / sock.c]
- 进程：`pidfd_open`（5.3+）稳定引用；`pidfd_send_signal`（5.1+，进程组 flag 6.9+）；`CLONE_PIDFD` 消除 spawn 竞态；`/proc/PID/stat` 字段 22 start time；`/proc/PID/exe` 可 `(deleted)`；`pidfd_getfd`（5.6+）只复制到调用者不改目标；`wait` 才 reap。[pidfd_open / pidfd_send_signal / proc_pid_stat / proc_pid_exe / pidfd_getfd / wait]
- 进程组/cgroup：`setsid`/`setpgid`；cgroup v2 `cgroup.events populated` 判 subtree 存活，`cgroup.kill`（5.14+）整组杀；systemd scope 管外部进程组，`KillMode=control-group` 默认。[setsid / setpgid / cgroup-v2 / systemd.scope / systemd.kill]
- PTY：写 master 才是 slave input；`grantpt` 设 slave 为调用者 UID mode 0620；`TIOCSTI`（6.2+ 通常需 CAP_SYS_ADMIN）禁用；`login_tty` 只能自身设 controlling terminal。[pty(7) / grantpt / TIOCSTI / login_tty]
- 文件：`openat2`（5.6+）`RESOLVE_BENEATH/IN_ROOT/NO_SYMLINKS/NO_MAGICLINKS/NO_XDEV`；`O_NOFOLLOW` 仅保护最后组件；硬链接=同 inode 另一名；`inotify` 可合并/丢失/`IN_Q_OVERFLOW`；`flock` advisory。[openat2 / inode(7) / fifo(7) / inotify(7) / flock / write]
- 基线：推荐 GA kernel 6.5+，6.9+ 用 pidfd 进程组信号；stable 7.1.4 / LTS 6.18.39（查询日）。

产品影响：L2 用 pidfd + 独立 cgroup/scope + 进程组；L5 文件用 openat2 受约束 open；任意 stdin 抢占/TIOCSTI 永久禁止。

## 3. macOS（查询日 2026-07-20，Apple 开发者 / XNU 12377.121.6 / launchd 842.92.1）

- Unix socket：pathname socket 用文件权限，close 后路径残留须清理；`LOCAL_PEERCRED`/`getpeereid` 连接时缓存 UID/GID；`LOCAL_PEERTOKEN` 由保存的 PID `proc_find` 再取 audit token（PID reuse 风险）；XPC per-message peer team/code-signing requirement。[unix(4) / un.h / getpeereid / xpc_connection_set_peer_team_identity_requirement / uipc_usrreq.c]
- 进程：XNU 有 `p_uniqueid`、PID version、parent unique ID、executable UUID、start time；audit token 含 PID version；`SecCodeCheckValidity` 动态验签可带 requirement，`SecCodeCopySigningInformation` 单独不验签；Endpoint Security 需 entitlement。[proc_info_private.h / libproc.h / SecCodeCheckValidity / SecCodeCopySigningInformation / es_process_t]
- launchd：可拥有 listening socket 按需启动 Host；job 不应自 daemonize/setsid；历史 plist：job 死亡默认杀同 PGID，`AbandonProcessGroup` 关闭；macOS 13+ 用 `SMAppService`。[Creating Launch Daemons and Agents / launchd.plist(5) / SMAppService]
- 文件：`openat` 支持 `O_NOFOLLOW_ANY/O_RESOLVE_BENEATH/O_UNIQUE`；`realpath` 仅返回调用时字符串。[open/openat(2) / realpath]
- 提示：多 process-information 接口位于 private header（未来可变）；新 open flags 的最低 SDK/版本须真机 feature-test。

产品影响：L2 组合 `(pid, start time, unique/version, executable UUID, code requirement)`；高权限面优先 XPC per-message requirement；crash 策略须在实现前冻结（terminate-on-host-loss 或 channel-lost 存活）。

## 4. SQLite（查询日 2026-07-20，sqlite.org）

- WAL reader 在 read transaction 开始固定 end mark 得一致 snapshot；活跃 reader 阻止 checkpoint，故观察事务须短。[Write-Ahead Logging]
- `-wal` 是持久状态一部分；单独复制主 DB 可能丢事务或损坏；只读打开需 `-wal/-shm` 可读或 `immutable`。[wal.html / open.html]
- `immutable=1` 跳过 locking/change detection，对变化文件可返回错误或 `SQLITE_CORRUPT`；`PRAGMA query_only` 非真只读；`SQLITE_OPEN_READONLY`+`mode=ro` 才是只读下界；`SQLITE_OPEN_NOFOLLOW`（3.39+）拒路径 symlink；WAL-reset bug 3.7.0–3.51.2，3.51.3+ 修复；Backup API 生成一致副本；最新 3.53.3。[lang_transaction / backup_finish / releaselog 3_39_0 / 3_53_3]

产品影响：L5 SQLite 用 3.53.3、`READONLY|URI|NOFOLLOW`、`mode=ro&cache=private`、短显式读事务；禁 live DB `immutable`/checkpoint/主库单文件复制；不满足即 `unavailable`。

## 5. 无障碍（查询日 2026-07-20）

- 依据：Apple HIG、Android/Material 与平台无障碍、Microsoft/Windows 无障碍、WCAG 2.2。具体验收见 [../product/states-content-and-accessibility.md](../product/states-content-and-accessibility.md)。证据不跨平台外推。
