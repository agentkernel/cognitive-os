# Agent Hub — 安全与凭据

> 类别：informative security design ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 本文覆盖本机控制面安全、账号/密钥分层、secret 存储与同步、多账号切换。威胁逐项见 [threat-model.md](./threat-model.md)；平台原语见 [../sources/platform-security-ledger.md](../sources/platform-security-ledger.md)。

## 1. 本机控制面安全

已冻结（[CONSOLE-AGENTHUB-V1-DEC-012](../decisions/decision-log.md)）：本机控制面不得只依赖 loopback 可达性。

- **Windows**：named pipe，随机高熵名称、`nMaxInstances=1`、`FILE_FLAG_FIRST_PIPE_INSTANCE`、`PIPE_REJECT_REMOTE_CLIENTS`、不可继承 handle、`SE_DACL_PROTECTED` DACL 仅授当前 logon SID 最小 rights（不授 `FILE_GENERIC_WRITE`/`FILE_CREATE_PIPE_INSTANCE`/`WRITE_DAC`/`WRITE_OWNER`）。连接后取 client PID/session；先读定长 hello 再 `ImpersonateNamedPipeClient`，校验 token 有效非匿名、SID/session 一致；失败即拒绝，`RevertToSelf` 失败即终止 Host。
- **macOS/Linux**：Unix socket，私有目录 `0700`、socket `0600`；accepted FD 上 `SO_PEERCRED`（Linux 6.5+ 同时 `SO_PEERPIDFD`）/`LOCAL_PEERCRED`+`LOCAL_PEERTOKEN`；abstract socket 不进入默认控制面；高权限面优先 XPC per-message peer requirement。
- **TCP 降级**（仅显式启用）：独立认证 + TLS + Host/Origin 检查 + DNS rebinding 防护 + 限流 + anti-replay；密码不入 URL/argv/env。
- 客户端反向校验 server identity（server PID + 签名 + 版本），防 squatting。

固有限制：logon SID/owner 只隔离其他用户，不隔离同用户恶意进程；强隔离需独立 OS principal 或容器。产品 UI 不得声称已抵御 same-UID 恶意进程，除非引入 package/AppContainer capability SID 或硬件绑定客户端密钥（列为 GA 安全 gate 的未决项）。

## 2. 账号与凭据分层

Direct 模式下 Credential Broker 只处理 opaque handle，不接触明文 secret 主体：

| 层 | 内容 | 存储 | 规则 |
|---|---|---|---|
| provider 原生登录 | Agent CLI/SDK 自身 OAuth/token | provider 自管（原 CLI profile） | 不复制、不迁移、不导出 |
| Host credential handle | 指向 OS secure store / enterprise broker 的 opaque reference | Windows DPAPI/Credential Manager、macOS Keychain、Linux Secret Service / enterprise broker | 短期注入子进程，不落 ledger |
| LLM API key | 用户提供的第三方 key | OS secure store | 仅注入目标 Agent 环境；不云同步 |
| Host↔client 认证 | 控制面 capability/token | 内存 + OS secure store | per-client 最小 scope；可 revoke |

规则（[CONSOLE-AGENTHUB-V1-DEC-020](../decisions/decision-log.md)）：

- secret 不云同步、不进 URL/argv/普通配置文件/AsyncStorage 明文；
- ledger/日志/Relay push 只含 metadata，零 secret；
- credential 刷新/写回必须经 Credential Broker 显式授权，不得伪装成只读 usage 查询（Paseo quota fetcher 反例）；
- quota/usage 查询默认关闭 credential probing，独立授权。

## 3. 多账号与切换

已冻结（[CONSOLE-AGENTHUB-V1-DEC-021](../decisions/decision-log.md)）：

- 支持每 provider 多账号 profile（opaque handle 选择），账号切换默认只影响**新** session。
- 不支持 provider 官方未提供的 session 内热切换账号：终止并以新 profile 恢复同一 native session ID 属 `blocked-by-policy`（上下文泄露/污染风险）。
- 安全替代：**新 session + 显式 handoff**——在新账号下新建 session，由用户显式转交上下文范围，旧 session 释放并推进 generation。
- 不从 Keychain/凭据文件抽取 token 复制到其他账号目录（Agent Deck 反例）。

## 4. secret 生命周期

- 注入：spawn 时短期注入 env 白名单；不继承 Host 全部环境。
- 使用：仅目标 Agent 进程可见；不写入 capture/ledger。
- 轮换：支持定期与应急轮换；共享密码只能全局轮换，因此优先 per-device/per-client capability。
- 撤销：单 client/单设备即时 revoke；丢失设备作废其 pending 请求。
- 销毁：会话结束清理内存副本；OS secure store 条目按用户操作删除。

## 5. 安全默认值（反例对照）

对照竞品研究（[../sources/paseo-and-comparables-ledger.md](../sources/paseo-and-comparables-ledger.md)），以下默认值被明确拒绝：

- 默认无认证 Web/控制面（amux/Opcode 反例）→ 本产品默认认证 + OS peer 校验；
- 连接即等权 `scopes:["*"]`（Paseo 反例）→ per-client 最小 capability；
- 客户端自选文件 root 到整机（Paseo 反例）→ 服务端 workspace capability 决定范围；
- `--dangerously-skip-permissions`/YOLO/AutoYes 默认（多项目反例）→ permission 默认需确认，高后果 PC-local；
- 明文 secret 落 env/URL/AsyncStorage（多项目反例）→ opaque handle + OS secure store。

## 6. Open PoC（安全/凭据）

全部 `not-run / evidence none`：

- `POC-SEC-001` named pipe DACL + impersonation 拒绝未授权同用户/跨用户 client；
- `POC-SEC-002` Unix socket peer credential + pidfd 在 PID reuse 下的稳定性；
- `POC-SEC-003` credential handle 注入全链路零 secret 落盘审计；
- `POC-SEC-004` 账号切换新 session + handoff 无上下文泄露验证。
