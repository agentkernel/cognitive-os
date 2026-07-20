# Agent Hub — 关键旅程与页面

> 类别：informative product design ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 约束：页面只显示事实来源标注的投影；产品 ID 不等于已登记机器合同。所有 evidence 当前为 `none`。

## 1. 旅程模板

每条旅程记录：entry / precondition / deployment mode / Agent+Adapter+version / takeover level / process·session·file target / account / user steps / source of truth / control path / failure·cancel·repeat·recovery / sensitive data / oracle / evidence。

## 2. PC 关键旅程

### `CONSOLE-AGENTHUB-V1-JRN-001` 无 CognitiveOS 首次启动与模式识别

- entry：首次启动，未发现受信 CognitiveOS。
- steps：欢迎与数据边界 → 探测 CognitiveOS（只形成建议）→ 用户显式选择 Direct/Governed → 建立并验证 Host identity → 选择扫描范围。
- source of truth：`host-managed`；未识别时显示 `connection unknown`，不创建第三模式。
- oracle：仅检测到 binary/进程不进入 Governed；无认证 endpoint/schema pin 时保持 Direct。
- evidence：none。

### `CONSOLE-AGENTHUB-V1-JRN-002` 扫描安装/进程/session/file 候选

- steps：先扫描安装，再分批扫描 Host-managed process、official native sessions、managed terminals、documented session files、external PID metadata。
- 约束：每类单独显示扫描范围与隐私影响；不默认读取 credential/auth/entitlement/cookie/内部 DB 或整个 home；native 文件仅 opt-in、documented root、敏感字段裁剪。
- 结果按七状态标签呈现，`unsupported` 与 `blocked-by-policy` 分开。
- evidence：none。

### `CONSOLE-AGENTHUB-V1-JRN-003` Host 启动并监管新 Agent（L2，默认）

- steps：选择 Agent/账号/workspace → 预览 spawn/cwd/env/process-group → Host 启动并登记 ownership generation → Work 显示 host-managed facts。
- oracle：普通 PID 无受支持通道时不可 send input；启动失败不显示运行过。
- evidence：none。

### `CONSOLE-AGENTHUB-V1-JRN-004` 官方 list/import/resume 接管 session（L3）

- steps：官方 list/import/resume → 验证旧 runtime inactive 或供应商 exclusive lease/fencing → 满足则可写接管，否则退回 read-only。
- oracle：无法证明单 writer 时不得双 writer；候选发现与最终 adopt 分开。
- evidence：none。

### `CONSOLE-AGENTHUB-V1-JRN-005` 附着受管终端（L4）

- steps：仅列出 Host 创建的 ConPTY（Windows v1）→ 预览 capture/send/detach/interrupt/resize → 附着后终端内容始终标不可信。
- oracle：普通既有 console 不进入 `terminal-attached`；只能重新以 Host-launched 启动或官方 resume。
- evidence：none。

### `CONSOLE-AGENTHUB-V1-JRN-006` 普通 PID 只读观察（L7）

- steps：显示 PID+creation time、owner、executable、签名、cwd、parent、health → 解释为何不能安全发送输入 → 提供“以 Host 管理方式重新启动”。
- oracle：任意 send/signal/write/adopt 请求返回 unsupported；仅 PC-local 精确 PID emergency containment 可发终止且显示 `result unknown`。
- evidence：none。

### `CONSOLE-AGENTHUB-V1-JRN-007` 文件候选 half-write / lock / corrupt

- steps：只读 snapshot → half-write/lock/truncate/rotation/schema drift 进入 partial/unknown → 不产生可写动作。
- oracle：SQLite 一致 read transaction；JSONL 只接受完整换行记录。
- evidence：none。

### `CONSOLE-AGENTHUB-V1-JRN-008` 拒绝进程注入 / token 抽取

- steps：内存注入、DLL 注入、调试器劫持、二进制 patch、任意 stdin 抢占、token/cookie/keychain 抽取 → 显示 `blocked-by-policy` 与安全替代。
- oracle：这些路径没有产品入口。
- evidence：none。

### `CONSOLE-AGENTHUB-V1-JRN-009` 检测现有 CLI 登录但不读取 secret；官方账号连接与多账号 profile

- steps：检测登录状态 → 调用官方登录 → 使用原 CLI profile → 保存 opaque profile handle → 选择账号。
- 约束：不复制 cookie/refresh token/keychain secret；账号切换默认只影响新 session；不支持 session 内切换时终止并新 profile 恢复同一 native session ID 属 `blocked-by-policy`，改为新 session + 显式 handoff。
- evidence：none。

### `CONSOLE-AGENTHUB-V1-JRN-010` 创建单 Agent 工作项与群组（Lead+Workers）

- steps：创建 WorkItem → 群组仅单 Host、一层 Workers；Lead 只提 proposal，确定性调度器维护 DAG/预算/并发/worktree/lease → 每次 handoff 显示 scope/预算/验收。
- oracle：child done 不完成 parent；coding 默认独立 worktree。
- evidence：none。

### `CONSOLE-AGENTHUB-V1-JRN-011` PC 转手机与手机 permission

- steps：QR/短码配对（两端 matching code + PC-local approve）→ 手机查看/请求扩权 → PC-local 批准生成新 ownership generation。
- oracle：手机不能直接扩权、发信号或扩大文件范围；push 只携 opaque hint。
- evidence：none。

### `CONSOLE-AGENTHUB-V1-JRN-012` cancel / kill / result-unknown 与 Host 崩溃恢复

- steps：分别建模 interrupt / cancel turn / stop session / SIGINT/SIGTERM/TerminateProcess / kill tree / detach / release → 请求超时显示 unknown，不显示已停止。
- oracle：Host 睡眠/锁屏/崩溃后旧 controller 输入被拒；先 resnapshot 再对账。
- evidence：none。

### `CONSOLE-AGENTHUB-V1-JRN-013` 迁移到完整 CognitiveOS

- steps：evidence-only import → Governed authority 新建对象 → 保留来源与历史。
- oracle：Host ledger 不改写为 authority Event。
- evidence：none。

### `CONSOLE-AGENTHUB-V1-JRN-014` 无障碍接管与恢复

- steps：键盘/屏幕阅读器/高对比/缩放/reduced motion 完成扫描、接管预览、permission、cancel、恢复、revoke。
- oracle：状态不只靠颜色/图标/动画；见 [states-content-and-accessibility.md](./states-content-and-accessibility.md)。
- evidence：none。

## 3. PC 页面清单

| Page ID | 页面 | 主任务 |
|---|---|---|
| `CONSOLE-AGENTHUB-V1-PAGE-001` | 首次启动与模式识别 | 选择/识别部署模式 |
| `CONSOLE-AGENTHUB-V1-PAGE-002` | Agent 扫描与候选 | 安装/进程/session/file 候选按七状态呈现 |
| `CONSOLE-AGENTHUB-V1-PAGE-003` | 接管预览与 consent | 见 §4 必显字段 |
| `CONSOLE-AGENTHUB-V1-PAGE-004` | Work 首页 / 统一工作项 | 创建、监督、纠偏、暂停、取消、恢复 |
| `CONSOLE-AGENTHUB-V1-PAGE-005` | 会话详情（终端/diff/artifact/usage） | 结构化摘要优先，raw terminal 可暂停 |
| `CONSOLE-AGENTHUB-V1-PAGE-006` | Agent Hub / Adapter | 逐能力接管等级与版本 |
| `CONSOLE-AGENTHUB-V1-PAGE-007` | Takeover Host / 主机 | health、process tree、terminals、ownership generation |
| `CONSOLE-AGENTHUB-V1-PAGE-008` | 群组工作区 | Lead+Workers、worktree、handoff、conflict |
| `CONSOLE-AGENTHUB-V1-PAGE-009` | permission Inbox | permission/clarification/unknown/local-confirm-required |
| `CONSOLE-AGENTHUB-V1-PAGE-010` | 账号与密钥 | opaque handle、多账号 profile、broker |
| `CONSOLE-AGENTHUB-V1-PAGE-011` | paired devices | 配对、scope、expiry、revoke、rotation |
| `CONSOLE-AGENTHUB-V1-PAGE-012` | 电脑控制 | selected-window、隔离浏览器、PC-local 确认 |
| `CONSOLE-AGENTHUB-V1-PAGE-013` | 来源与保证 | 当前事实来源与不保证事项 |

## 4. 接管预览必显字段（PAGE-003）

Agent；PID/session；owner；binary/signature/version；cwd；account；takeover level；可用动作；将读取的文件；将发送的信号；不保证事项；release/recovery 路径；ownership generation。初始焦点在标题/变化摘要，危险按钮不是默认 Enter。

## 5. 手机 remote companion 旅程要点

- 手机是 Takeover Host 的远程 companion，不承载本地 Agent runtime、完整 Vault、CognitiveOS node、authority 或永久 supervisor。
- 五入口：Work / Tasks / Agents（含 Team 二级）/ Inbox / More；顶部 Host switcher，所有可写页面持续显示 Host/账号/模式/freshness。
- QR/短码配对、Host identity、多 Host、任务进度、takeover source、群组消息、diff/artifact、clarification、permission、pause/cancel、current-state refresh 均按最小裁剪 metadata 呈现。
- 以下默认要求 PC 本机确认：第一次附着普通既有进程、扩大文件读取范围、observe→write、发送进程信号、启用桌面控制、访问新 credential、跨用户或提升权限。
- 手机只看到裁剪后的 process/session/file metadata；不暴露其他用户、无关路径、环境变量、credential、token、raw auth file。Push 只携 opaque hint；通知 action 不直接接管/批准/重试/杀进程/写文件。
- 恢复：手机 offline、Relay 重复/乱序、Host/Agent 重启、丢失设备、revoke、key rotation 分别处理，回前台先 reauth/resnapshot。详见 [../architecture/relay-pairing-and-migration.md](../architecture/relay-pairing-and-migration.md)。
