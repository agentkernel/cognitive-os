# Adapter Dossier — OpenClaw

> 类别：informative research ｜ 日期：2026-07-20（2026-07-21 AH-CTR-02 文档级回填）｜ owner：Lane-CON
>
> 状态用语：**接口已核验（文档级；Gateway WS 字段级 partial）** / product-only / not-implemented / **evidence not-run**。

## 身份

- 目标：OpenClaw（个人 AI assistant；Gateway 架构；原 Clawdbot → Moltbot → OpenClaw）。
- 官方仓库：https://github.com/openclaw/openclaw （TypeScript；查询日 2026-07-20）。
- LICENSE：https://raw.githubusercontent.com/openclaw/openclaw/main/LICENSE — **MIT**（Copyright 2026 OpenClaw Foundation；附 THIRD_PARTY_NOTICES.md）。
- 适用基线：release **v2026.7.1**（2026-07-13，CalVer）。
- 官网：https://openclaw.ai （2026-07-08 起非营利基金会等动态）。
- 维护状态：活跃。
- 许可：**MIT**（非 source-available）。

## 官方控制接口（一手）

- **Gateway（控制平面）**：常驻 daemon；默认端口 **18789**；**WebSocket** 控制面服务 Control UI / WebChat / 移动配对；「session 状态由 gateway 拥有」。
- **CLI**：`openclaw gateway status|stop`；`openclaw sessions --json`；`openclaw doctor [--session-sqlite inspect]`；`openclaw pairing approve`；`openclaw security audit`。
- **外部集成模式**（reference/rpc）：HTTP+JSON-RPC+SSE 或 stdio line-delimited JSON-RPC。
- 文档：https://docs.openclaw.ai/concepts/session 、https://docs.openclaw.ai/reference/rpc 、/gateway （**WS 协议字段级本轮 partial，未逐条核验**）。
- 风险默认：main session 工具默认宿主机直跑；渠道输入=不可信；默认绑定 loopback vs `0.0.0.0` 口径冲突 → PoC。

## 接管层级适用性

| Level | 适用 | 条件/限制 |
|---|---|---|
| L1 官方控制 | 目标（条件） | Gateway WS（字段级合同 partial；须叠加我方确认矩阵，不可继承其单用户默认） |
| L2 Host-launched | 目标 | Host 启动 gateway / agent |
| L3 session 采用 | 条件 | gateway 拥有 session；写需单 writer/lease |
| L4 受管终端 | 条件 | 仅 Host-owned |
| L5 只读文件 | 只读 | SQLite + 归档 transcript |
| L7 observe-only | 是 | |
| L6 / L8 | 阻断 / 禁止 | |

## session / 文件事实

- **运行时 SQLite**：`~/.openclaw/agents/<agentId>/agent/openclaw-agent.sqlite`。
- **归档 transcript**：`~/.openclaw/agents/<agentId>/sessions/`；遗留 `sessions.json`/JSONL 可在 gateway 启动或 `doctor --fix` 时迁入 SQLite。
- `session.maintenance` 会主动删旧行 → L5 须容忍裁剪。
- 配置：`~/.openclaw/openclaw.json`；workspace/skills 在同根下。

## 账号与凭据

- 工具无账号墙；模型 BYO；渠道测试需对应消息平台账号。
- 仅 opaque handle；再分发须随附 THIRD_PARTY_NOTICES.md。

## 平台

- 当前 README：macOS/Linux/Windows；历史版本曾强推 WSL2；Microsoft Build 2026 宣布原生 Windows 容器路径——**Windows 形态演进中，PoC 必核**。

## 未决与 Open PoC

- Gateway WS 握手/认证/方法词表；默认绑定地址实证；sqlite schema + transcript 格式；沙箱隔离强度；渠道平台 ToS。
- Open PoC：POC-SESS-001、POC-FILE-002、POC-SEC-001/003——状态 **not-run**。
- gate：Adapter 实现仍 `blocked`；文档级核验不解除实现 gate。

## 产品映射

- Gateway WS 为 L1 候选（须我方确认矩阵）；CLI/SQLite 为 L5/运维面；证据全 not-run。
