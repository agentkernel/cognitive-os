# Adapter Dossier — OpenHands

> 类别：informative research ｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：product-only / not-implemented / evidence none
>
> 事实来源见 [../../sources/paseo-and-comparables-ledger.md](../../sources/paseo-and-comparables-ledger.md)（OpenHands 节，查询日 2026-07-20）。

## 身份

- 官方仓库：控制面 `OpenHands/agent-canvas`；执行/Agent Server `OpenHands/software-agent-sdk`；旧主仓库拆分中。
- 适用基线：Agent Canvas commit `44f2cc22feb3c94a06d1cc80c5e299deb757e818`，release v1.5.1（2026-07-19）；SDK commit `4fe565663af2b4f1130a6e0dac7566b002bfe9b4`，release v1.36.1。
- 维护状态：活跃。
- 许可证：Agent Canvas 与 SDK 为 MIT；旧主仓库 enterprise 目录为 PolyForm Free Trial（每年最多 30 天、禁止分发），其余 MIT。**复用前须过法务 gate。**

## 官方控制接口

- Agent Server：长驻 HTTP/WebSocket daemon，拥有 OpenHands conversation 及其 workspace。
- ACP：由 Agent Server spawn ACP 子进程，走 JSON-RPC over stdio。
- 不 attach 外部 PID/PTY；客户端重连 conversation ID 是**平台自有会话恢复**。
- 稳定性：Agent Server/ACP 为文档化能力；通用 conversation list 的公开 API 文档不完整（未决）。

## 接管层级适用性

| Level | 适用 | 条件/限制 |
|---|---|---|
| L1 官方控制 | 是 | Agent Server HTTP/WS + ACP |
| L2 Host-launched | 是 | Host 启动 Agent Server / ACP 子进程 |
| L3 官方 session 采用 | 条件 | 恢复的是平台自有 conversation，不是第三方 native session |
| L4 受管终端 | 条件 | 一般不需要；如需仅 Host-owned |
| L5 只读文件 | 待核验 | 未发现扫描/导入供应商 native session |
| L7 observe-only | 是 | 外部进程只观察 |
| L6 / L8 | 阻断 / 禁止 | |

## session / 文件事实

- OpenHands 恢复自身 conversation（platform-owned），不代表接管供应商 native 会话。
- 支持创建/get/count、send_message、真正的 `pause()`、再 `run()` resume。
- ACP existing conversation 必须以 ACPAgent 重新连接。
- 未发现扫描/导入 provider native session 文件或修改其配置。

## 账号与凭据

- ACP 可自动使用本机 Codex/Claude/Gemini 登录文件/Keychain，或把 secret registry 值注入子进程环境并脱敏。
- `OH_SECRET_KEY` 加密服务端静态 secret。
- 风险：ACP 文档称 permission 请求会**自动批准**——高风险默认，本产品必须能关闭并改为显式确认（未决：不同部署模式下可否关闭）。

## 平台

- 远程通过 Agent Server、VM、Docker、Modal 或 Cloud；API key + TLS/VPN，无应用层 E2EE Relay。
- 原生 Windows 支持待核验（多数部署经容器/服务端）。

## 条款 / 许可影响

- MIT 部分可参考；enterprise 目录 PolyForm 禁止分发；复用前法务 gate。

## 未决与 Open PoC

- 通用 conversation list 公开 API 完整性；ACP 自动批准可否按部署关闭；原生 Windows 路径。
- Open PoC：POC-SESS-001（跨版本漂移）、POC-SEC-003（凭据注入零落盘）——状态 not-run。

## 产品映射

- 真 pause / Agent Server 分层 / secret-at-rest 可借鉴；自动批准与默认无 API auth 的本地 Agent Server 不可直接采用。
