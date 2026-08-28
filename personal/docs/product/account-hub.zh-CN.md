# Personal Model Connections

- 状态：基于现有 Provider 基础的 Personal 2.0 已采纳目标
- 规范语言：[英文原文](account-hub.md)
- 当前权威基础：[Provider Control Plane](provider-control-plane.md)
- 需求基线：
  [Personal 2.0 OPC 需求分析](personal-2.0-opc-requirements-analysis.md)
- 交互基线：
  [**Owner-approved interaction baseline (2026-08-28)**](../../../clients/docs/design/opc-2.0/personal-20-ai-ceo-e2e-optimized-v2.canvas.tsx)
- 基线身份：同一份 V2 文件（不是 v3）。Owner 于 2026-08-28 接受本轮有竞争对照的
  覆盖：可见 CEO 闭环、Today 决策包加四条例外泳道、仅画布 HITL，以及 daemon 授权
  路径。这不是覆盖前的 overlay 对话 / 拆栏 V2。
- 凭据边界：
  [ADR-0055](../../../docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md)

## 1. 产品边界

Model Connections 必须分开 connection endpoint/compatibility/account、
SecretStore custody、model catalog、Member 明确选择、Provider quota 与
actual/estimated/unknown cost。Personal 2.0 不提供 consumer subscription、invoice、
plan 或产品 billing 管理。Credential 存在不证明 reachability；quota 不能从 usage
推断；unknown usage/cost 不能显示为 0。

## 2. Reality ledger

| 边界 | Connection 事实 |
|---|---|
| **Current implementation (Now)** | 已有命名 OpenAI/Anthropic/custom OpenAI-compatible account、API key SecretStore handoff、model discovery/manual model、fixed Agent binding、usage/cost、advisory budget/alert、audit 与 Provider UI。 |
| **Adopted Personal 2.0 target** | Settings > Model Connections 提供主流 Provider 快速模板、advanced custom connection、每个 Member 的明确 Provider/model 选择、DSH/Pi daemon proxy 与诚实 quota/usage/cost。 |
| **Requires-backend** | 更多 adapter、custom compatibility mode、SecretStore reader、Member selection/revision、更广 quota、cost composition 与 runtime rebind/restart。 |

现有 fixed Agent binding 与 advisory budget 是事实基础，不定义 2.0 产品组织或自动停止
策略。

## 3. Connection 创建

```text
选择主流 Provider template
  -> key 单向进入 SecretStore
  -> discover/select model
  -> 检查 redacted endpoint/account/model
  -> 保存 connection receipt
```

Advanced setup 明确输入 custom URL、compatibility mode、key 与 model。Endpoint trust、
compatibility、credential、reachability 与 model availability 分开检查，不进入
subscription/invoice 流程。

## 4. Member 明确选择

创建每个项目成员时，Owner 都必须明确选择 Provider/model。Assistant 可解释和推荐，
不能静默绑定。Role Runtime Template 只声明模型能力，不携带 connection 或 credential。
后续变更形成带版本的 Member/Task revision，说明受影响工作及是否需要重启。现有工作
不得静默切换。禁止 ambient load balancing、hidden fallback、caller credential 或
arbitrary auth header。

## 5. Secret 与 proxy 边界

API key、ADR-0055 per-source import 与 custom endpoint 都必须进入 approved
SecretStore 并由 daemon proxy 使用。Raw secret 不得进入 UI storage/DOM/URL、
Agent/Member config、DSH、Pi、MCP、Vault、Conversation archive、
ordinary config、SQLite、argv、environment、log、evidence 或 chat。

Import 成功只表示 SecretStore 已接收；Provider、model、quota 仍分别检查。

## 6. Cost、quota 与 usage

- Provider quota：Provider 给出的 allowance/reset/source；
- actual cost：Provider 报告或直接计量，带 source/period；
- estimated cost：带 model/pricing/method/version/scope；
- unknown cost：无法得出结论，绝不能显示为 0；
- cost warning：供 Owner/manager 处理的可见阈值或偏差信号。

Personal 2.0 不会因为产品预算阈值自动停止工作。告警不能抹去 unknown Effect 或自行
选择更便宜 route。Provider quota、credential failure 或 Provider unavailable 仍可导致
外部失败。现有 advisory budget 继续标为 current implementation fact，不成为 2.0
目标策略。

## 7. Model Connection setup、状态与恢复

流程为选择主流模板或 custom connection → non-logging secret path → 检查 redacted
endpoint/account/model/quota/trust → 分开执行 reachability/credential/model probe →
保存 → 返回成员创建/Settings → 为成员明确选择 connection/model。

失败后保留非 secret 输入。locked SecretStore、expired credential、unreachable
endpoint、model missing、quota unknown、stale catalog、cost warning 与
rebind-required 是不同状态。

Model Connections 覆盖 empty、loading、partial、stale、permission、error、unknown、
offline、cost-warning、quota-unavailable、success 与 archived。所有 reading 都带
source/period；percentage 必须带 denominator。

Expanded Model Connections、Member selection/revision、DSH/Pi proxy composition
和 quota/cost integration 均为 **Requires-backend**。本文不构成 Provider quality、
entitlement、cost accuracy、support、Gate、release、Profile 或业务结果声明。
