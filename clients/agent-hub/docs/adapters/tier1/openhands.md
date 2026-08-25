# Adapter Dossier — OpenHands

> 类别：informative research ｜ 日期：2026-07-20（2026-07-21 AH-CTR-02 文档级回填）｜ owner：Lane-CON
>
> 状态用语：**接口已核验（文档级）** / product-only / not-implemented / **evidence not-run**。

## 身份

- 官方仓库：控制面 `OpenHands/agent-canvas`；执行/Agent Server `OpenHands/software-agent-sdk`；旧主仓库 `OpenHands/OpenHands`（混合许可）仍活跃。
- 适用基线：**Agent Canvas v1.5.2**（2026-07-20）；**SDK v1.36.1**（2026-07-15；含 `agent-server-*-windows-x86_64.exe`）。
- 维护状态：活跃。
- 许可证：Canvas 与 SDK 为 **MIT**；旧主仓库 `enterprise/` 为 **PolyForm Free Trial**（每年最多 30 天、**禁分发**），其余 MIT。**复用前须过法务 gate；enterprise 目录不复用。**

## 官方控制接口（一手）

- **Agent Server**：长驻 HTTP/WebSocket；默认无 API 认证（可配 `X-Session-API-Key`）；`POST .../pause`、`/interrupt`（即时取消）、`/goal/resume`；**`GET /api/conversations/search`**（列表/搜索 API **已有公开生成文档**）。
- **SDK**：`Conversation` / `send_message` / `run` / `pause`；Local 与 Remote/Docker workspace。
- **ACP 双向**：`openhands acp`（IDE；flags 含 `--always-approve` / `--llm-approve` / `--resume`）；SDK `ACPAgent` spawn ACP 子进程——官方明示 permission **自动批准**（须可关并改显式确认）；`acp_env` 已在 1.29.0 移除，凭据经 conversation secrets。
- 文档：https://docs.openhands.dev/sdk/arch/agent-server ；api-reference/conversations/search-conversations ；sdk/guides/agent-acp 。
- 不 attach 外部 PID/PTY；重连 conversation ID = **平台自有会话恢复**。

## 接管层级适用性

| Level | 适用 | 条件/限制 |
|---|---|---|
| L1 官方控制 | 是 | Agent Server HTTP/WS + ACP（文档级） |
| L2 Host-launched | 是 | Host 启动 Agent Server / ACP 子进程 |
| L3 官方 session 采用 | 条件 | 恢复平台自有 conversation，不是第三方 native session |
| L4 受管终端 | 条件 | 一般不需要；如需仅 Host-owned |
| L5 只读文件 | 待 PoC | 磁盘布局无公开合同；未发现导入他 provider native session |
| L7 observe-only | 是 | |
| L6 / L8 | 阻断 / 禁止 | |

## session / 文件事实

- 恢复对象是平台自有 conversation（非供应商 native takeover）——与产品语义一致。
- 支持创建/get/search/count、send_message、真正的 `pause()`、再 `run()` resume；`/interrupt` 与 pause 分层。
- ACP existing conversation 必须以 ACPAgent 重新连接。
- conversation 事件持久化磁盘布局 → PoC。

## 账号与凭据

- ACP 可使用本机 Codex/Claude/Gemini 登录或 secrets registry；`OH_SECRET_KEY` 加密服务端静态 secret。
- 风险：ACP 自动批准为高风险默认——本产品必须能关闭并改为显式确认（IDE flags 部分覆盖；ACPAgent 方向仍需 PoC）。
- LLM：BYO key 或 OpenHands Cloud。

## 平台

- 远程：Agent Server / VM / Docker / Cloud；SDK release 含 **Windows 官方二进制**（行为仍需 PoC）。
- 无应用层 E2EE Relay（与我方 Relay 设计正交）。

## 条款 / 许可影响

- MIT 部分可参考；enterprise PolyForm **禁止分发**；复用前法务 gate（POC-LIC）。

## 未决与 Open PoC

- conversation 磁盘布局；默认无认证实测；ACP 自动批准在各部署可否关闭；Windows 二进制可用性。
- **已闭合（文档级）**：通用 conversation list/search API 文档完整性（现有生成式 OpenAPI 参考）。
- Open PoC：POC-SESS-001、POC-SEC-003——状态 **not-run**。

## 产品映射

- 真 pause/interrupt、Agent Server 分层、secret-at-rest 可借鉴；自动批准与默认无 API auth 不可直接采用；证据全 not-run。
