# Agent Hub — 威胁模型

> 类别：informative security design ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 每条威胁按统一字段记录：asset / attacker / entry / trust boundary / prevention / detection / failure mode / recovery / owner / oracle / evidence。当前所有 evidence 为 `none`；prevention/detection 描述的是设计意图，不是已实现防护。威胁记录模板见 [../templates/threat-record.md](../templates/threat-record.md)。

## 1. 信任边界总览

1. 客户端（PC/手机）↔ Takeover Host 控制面（认证 IPC / E2EE Relay）；
2. Takeover Host ↔ Agent 进程（process group / Job / cgroup；PTY；官方 API）；
3. Takeover Host ↔ provider 文件（只读 parser 边界）；
4. Takeover Host ↔ OS secure store / enterprise broker（credential 边界）；
5. 手机 ↔ Relay ↔ Host（E2EE；Relay 是不可信中继）；
6. Agent 输出 ↔ UI 渲染（不可信内容边界）；
7. Direct 产品记录 ↔ CognitiveOS authority（语义边界：永不混同）。

## 2. 威胁清单

### `CONSOLE-AGENTHUB-V1-TM-001` 恶意本机进程冒充客户端调用 Host 控制面

- asset：Host 控制 API（信号/输入/credential 动作）。
- attacker：同机低权限恶意进程。
- entry：named pipe / Unix socket / TCP loopback。
- boundary：边界 1。
- prevention：pipe DACL 限 logon SID、`FILE_FLAG_FIRST_PIPE_INSTANCE`、`PIPE_REJECT_REMOTE_CLIENTS`；Unix socket `0700/0600` + `SO_PEERCRED`/`SO_PEERPIDFD`；per-client capability；不以 loopback 可达为身份。
- detection：ledger 记录全部连接 identity 与拒绝事件；异常速率告警。
- failure：同 OS 用户内恶意进程无法被 DACL 隔离。
- recovery：revoke 该 client capability；轮换控制面名称/密钥。
- owner：Host/Control。oracle：未授权 client 全部动作被拒且留痕。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-002` pipe/socket squatting（先占名称伪装 Host）

- asset：客户端发往 Host 的命令与数据。
- attacker：同机恶意进程先创建同名 pipe/socket。
- entry：可预测名称。
- boundary：边界 1。
- prevention：随机高熵名称 + first-instance flag；客户端验证 server identity（Windows `GetNamedPipeServerProcessId` + 签名校验；Unix socket 目录 owner 检查）。
- detection：客户端 handshake 校验失败告警。
- failure：客户端把命令发给假 Host。
- recovery：终止会话、轮换名称、审计已发命令。
- owner：Host/Control。oracle：squatting 样本被 handshake 拒绝。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-003` PID reuse 导致误控/误杀

- asset：非目标进程的存活与数据。
- attacker：非恶意（OS 行为）或诱导性恶意进程。
- entry：目标退出后 PID 被复用。
- boundary：边界 2。
- prevention：handle/pidfd 锚定 + creation time 校验；动作前重验；紧急终止限精确 PID+creation time 且 PC-local 确认。
- detection：identity mismatch 事件入 ledger。
- failure：按 PID 直接 kill 误杀无辜进程。
- recovery：不可逆；只能记录与告知。
- owner：Process Supervisor。oracle：PID 复用压力测试零误杀。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-004` 终端 escape/OSC 注入

- asset：用户终端显示真实性、剪贴板、后续键入。
- attacker：Agent 输出或其转发的网页/文件内容。
- entry：PTY 字节流中的 ANSI/OSC 序列。
- boundary：边界 6。
- prevention：capture 流 OSC/危险 CSI 清洗；UI 渲染白名单序列；粘贴前 bracketed paste 展示。
- detection：清洗器记录被剥离序列。
- failure：伪造“已完成/已授权”显示、篡改标题、写剪贴板。
- recovery：以 ledger 结构化记录为准回放。
- owner：Terminal Broker。oracle：注入样本库全部被清洗且显示为字面量。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-005` 任意进程 stdin 抢占 / PTY 劫持

- asset：用户终端输入与目标进程内部状态。
- attacker：诱导用户对普通既有进程发起“接管”。
- entry：`/proc/PID/fd/0`、`pidfd_getfd`、`TIOCSTI`、AttachConsole 注入。
- boundary：边界 2。
- prevention：仅 Host 创建并持有的 PTY master 或受管 multiplexer 可写；其余归 L7；这些路径无产品入口。
- detection：目标缺少 Host launch/ConPTY ownership record 即拒绝。
- failure：抢占失败或误写他人终端。
- recovery：改为 Host-launched 重启或官方 resume。
- owner：Process Supervisor / Terminal Broker。oracle：任意外部 PID 的 send-input 请求确定性 unsupported。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-006` 无 exclusive lease 的双 writer session 采用

- asset：provider 官方 session 一致性。
- attacker：并行 runtime 或诱导性 import。
- entry：官方 resume/import，旧 writer 仍活跃。
- boundary：边界 2/3。
- prevention：写接管前证明旧 writer inactive 或取得供应商 exclusive lease/fencing，否则只读。
- detection：runtime liveness 检查与 lease 校验。
- failure：两个 writer 同时写坏 session。
- recovery：退回 read-only；以 provider 官方状态为准对账。
- owner：Session Adopter。oracle：无法证明单 writer 时写路径不可达。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-007` symlink/reparse/hardlink/TOCTOU 文件逃逸

- asset：documented root 之外的敏感文件。
- attacker：诱导性路径或恶意占位文件。
- entry：opt-in 文件观察路径解析。
- boundary：边界 3。
- prevention：逐层 `O_NOFOLLOW`/`openat2 RESOLVE_BENEATH|NO_XDEV` 或 Windows `FILE_FLAG_OPEN_REPARSE_POINT` 逐层检查；handle-bound `fstat`；拒绝 `nlink>1`、设备/FIFO、跨用户、cloud placeholder。
- detection：owner/mode/type/dev/ino/nlink 与策略不符即拒。
- failure：读到 root 外文件或触发 cloud 下载。
- recovery：`blocked-path`，要求用户显式选择真实允许路径。
- owner：File Observer。oracle：junction/symlink/hardlink/mount-swap/`..`/UNC 负例全部被拒且不阻塞。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-008` provider 文件/凭据/内部 DB 被写坏

- asset：provider auth/entitlement/history/internal DB。
- attacker：越界写或诱导性“修复”。
- entry：文件写路径。
- boundary：边界 3。
- prevention：默认禁写 credential/auth/entitlement/policy/approval/history/内部 DB；L6 v1 阻断；写只限 Host-owned 目录。
- detection：写目标不在 Host-owned protected 目录即拒。
- failure：破坏 provider 内部一致性或伪造完成状态。
- recovery：无法自动回滚；标记并告知；以 provider 官方状态为准。
- owner：File Observer / Workspace Manager。oracle：对 provider 文件的任何写请求 unsupported。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-009` credential/token/keychain 抽取

- asset：provider 长期凭据、cookie、refresh token、keychain secret。
- attacker：诱导性“统一账号”功能或恶意 Adapter。
- entry：credential 读取路径。
- boundary：边界 4。
- prevention：只保存 opaque handle；短期注入；不复制 cookie/refresh token/keychain；不落 secret 到 ledger；不云同步。
- detection：credential 访问审计只记录 metadata。
- failure：secret 泄露或跨账号移植。
- recovery：撤销并轮换该 credential；通知用户。
- owner：Credential Broker。oracle：ledger/日志/同步载荷零 secret；抽取路径无入口。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-010` Relay 中间人 / 配对劫持

- asset：远程控制通道。
- attacker：Relay 运营方或网络攻击者。
- entry：配对 offer、Relay 帧。
- boundary：边界 5。
- prevention：E2EE（Relay 只见密文）；两端 matching code 比对 + PC-local approve；device 公钥指纹绑定；offer expiry/single-use。
- detection：handshake/transcript binding 校验失败告警。
- failure：攻击者插入或读取控制流。
- recovery：撤销设备；轮换密钥。
- owner：Relay/Pairing。oracle：MITM 演练（matching code 绕过）被拒。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-011` Relay 消息重放 / 乱序

- asset：控制命令幂等性。
- attacker：记录并重放合法密文。
- entry：live session 帧。
- boundary：边界 5。
- prevention：request ID 去重、单调序号、replay window、anti-replay nonce cache。
- detection：重复/乱序序号被拒并记录。
- failure：重复信号或重复扩权。
- recovery：以 request ID 对账；拒绝已执行请求。
- owner：Relay/Pairing。oracle：重复/乱序投递不产生重复副作用。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-012` 丢失设备 / 未撤销的长期配对

- asset：Host 写控制授权。
- attacker：持有丢失手机者。
- entry：既有配对能力。
- boundary：边界 5。
- prevention：每设备独立 scope + expiry；PC 端单设备即时 revoke；key rotation 有交接窗口。
- detection：设备活动审计与异常告警。
- failure：他人以旧设备继续控制。
- recovery：revoke 设备并作废其 pending 请求。
- owner：Relay/Pairing。oracle：revoke 后该设备所有动作被拒。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-013` 手机静默越权

- asset：本机高后果动作（信号/扩权/桌面控制/新凭据）。
- attacker：被攻陷的手机会话。
- entry：Relay 请求。
- boundary：边界 5/1。
- prevention：手机只能请求；高后果动作强制 PC-local 确认并生成新 generation。
- detection：未确认请求不执行。
- failure：手机直接触发高后果动作。
- recovery：拒绝并要求 PC-local approve。
- owner：Host/Control。oracle：无 PC-local approve 的扩权/信号/桌面控制不可达。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-014` Host 崩溃 / 孤儿进程 / split-brain

- asset：单 controller 保证与外部 Effect。
- attacker：非恶意（崩溃）或诱导竞态。
- entry：Host 重启、Job handle 泄露、旧 controller 恢复。
- boundary：边界 2。
- prevention：generation 先持久化；Job `KILL_ON_JOB_CLOSE`/cgroup 边界；恢复先对账再动作。
- detection：重启后同 identity 存活标 `orphan-suspected`；旧 generation 输入被拒。
- failure：误杀或双重控制。
- recovery：重验 identity 与归属后处置；外部 Effect 对账。
- owner：Process Supervisor。oracle：强杀 Host 后正常子进程按策略退出；旧 controller 输入全被拒。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-015` Job breakaway / 外部 broker 逃逸

- asset：受管进程树完整性。
- attacker：利用 WMI/计划任务/服务/COM 逃出 Job。
- entry：外部进程创建路径。
- boundary：边界 2。
- prevention：禁用两种 breakaway；默认阻止 WMI/计划任务/服务创建能力；能力 allowlist。
- detection：出现关联但不在 Job 的进程即 `escaped-process-suspected`。
- failure：kill tree 漏杀逃逸进程。
- recovery：完整 identity 证明后单独处置；否则告知用户。
- owner：Process Supervisor。oracle：普通子孙随 Job 关闭；WMI 负例被阻止或检测为逃逸。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-016` session/JSONL/SQLite 半写、损坏与伪造完成

- asset：文件观察结论真实性。
- attacker：并发 writer、诱导性损坏样本。
- entry：只读 parser。
- boundary：边界 3。
- prevention：JSONL 只接受完整换行记录；SQLite 只读一致 read transaction，禁 `immutable`/checkpoint/主库单文件复制；版本化 parser；digest。
- detection：short read/inode change/schema drift/lock 冲突进入 partial/unknown。
- failure：半写记录被当作完成事实。
- recovery：重取稳定 snapshot；未知版本只读并标注。
- owner：File Observer。oracle：半写/rotation/truncate/损坏 DB 不产生伪记录或崩溃。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-017` 停止语义误报（信号=已停止）

- asset：完成/停止判定真实性。
- attacker：忽略信号的 Agent 或延迟 I/O。
- entry：停止动作。
- boundary：边界 2。
- prevention：interrupt→cancel→TERM→KILL/kill-tree 分层，各有 timeout；状态机 `requested→signal-delivered→exit-observed→group-empty→reaped`。
- detection：信号后仍存活/树非空/外部资源仍变化。
- failure：显示 stopped 但进程仍活或 Effect 仍在发生。
- recovery：标 `unknown`，resnapshot 与 Effect 对账。
- owner：Process Supervisor。oracle：忽略 Ctrl+C 的 Agent 不被误报停止。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-018` 跨用户 / same-UID 绕过

- asset：其他用户与同用户其他应用数据。
- attacker：同机其他 UID，或同 UID 恶意进程。
- entry：进程/文件/socket 访问。
- boundary：边界 1/2/3。
- prevention：per-user 非提权 Host；禁 root observer；OS access check；最小路径范围。
- detection：UID/home/session/codesign 不匹配即拒。
- failure：same-UID 恶意进程绕过 Host 直接操作 tmux/文件/信号。
- recovery：强隔离需独立 OS principal 或容器。
- owner：Host/Control。oracle：其他 UID 无法列出/读取 candidate；UI 不得把 same-UID 限制标为“强隔离”。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-019` Direct 记录被当作 CognitiveOS authority

- asset：完成语义与治理保证。
- attacker：产品措辞漂移或诱导性迁移。
- entry：UI 文案、迁移导入。
- boundary：边界 7。
- prevention：Direct 使用独立词汇；完成语言分开；迁移只作 evidence-only import。
- detection：文案/迁移审查禁止 authority 术语复用。
- failure：用户误以为具备 CAS/fencing/Verification/Acceptance。
- recovery：修正措辞；标注 Direct 不保证事项。
- owner：产品治理。oracle：Direct 页面不出现 “Verified/CognitiveOS completed”；迁移不改写 authority Event。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-020` 不可信内容渲染越权

- asset：Console 系统控件与用户决策。
- attacker：Agent 输出、Markdown/HTML、日志、包来源。
- entry：内容渲染面。
- boundary：边界 6。
- prevention：不可信内容在隔离低权限面渲染；系统控件由受控组件构建；二者不共享安全边界；permission 走结构化独立通道。
- detection：渲染面沙箱违规告警。
- failure：伪 permission/completion 提示误导用户。
- recovery：以结构化 ledger 记录为准。
- owner：Console 前端。oracle：注入内容不能触发真实系统动作或伪造 permission。evidence：none。

### `CONSOLE-AGENTHUB-V1-TM-021` 桌面控制误操作 / 敏感界面泄露

- asset：用户桌面其他窗口与敏感数据。
- attacker：Agent 误点击或诱导访问 secure desktop。
- entry：桌面控制动作。
- boundary：边界 2/6。
- prevention：仅 selected-window（用户显式选择的目标窗口）；不做全桌面通用控制；UAC secure desktop 与凭据界面明确不可控。
- detection：目标窗口外的动作被拒；焦点丢失即暂停。
- failure：误点其他窗口或截取敏感界面。
- recovery：暂停并要求用户重新授权目标窗口。
- owner：Computer Control。oracle：selected-window 外的输入/截屏不可达。evidence：none。详见 [computer-control.md](./computer-control.md)。

## 3. Open PoC 与 oracle 状态

所有威胁的 oracle 均为未来测试要求；对应 Open PoC 列在 [../traceability/evidence-index.md](../traceability/evidence-index.md)，当前状态一律 `not-run / evidence none`。任何 prevention/detection 描述都不得被引用为“已实现”或“已验证”。
