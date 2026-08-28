# MCP capability acquisition 与治理

- 状态：已采纳的 Personal 2.0 项目 capability 路径；广义 family console 不在范围
- 规范语言：[英文原文](mcp-resource-family.md)
- 决策：
  [ADR-0057](../../../docs/adr/0057-personal-2-0-mcp-resource-family.md)、
  [ADR-0058](../../../docs/adr/0058-personal-2-0-mcp-conversation-private-projection.md) 与
  [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- 需求基线：
  [Personal 2.0 OPC 需求分析](personal-2.0-opc-requirements-analysis.md)
- 当前交互原型：
  [**personal-20-opc-e2e（旅程减法后）**](../../../clients/docs/design/opc-2.0/personal-20-opc-e2e.canvas.tsx)
- 已归档历史 V2（不是当前 chrome）：
  [pre-subtraction history](../../../clients/docs/design/opc-2.0/history/2026-08-28-pre-subtraction/README.md)
- 原型身份：当前 chrome 是旅程减法后的画布，不是 V2 的 CEO 轨 / X 英雄圈。

## 1. Scope

当项目 setup 或运行识别出能力缺口时，Personal 助手可发现 MCP capability。
安全审查后的 acquisition、exact-version pin 和独立 Project/Member grant 属于
Personal 2.0 目标，不再整体 deferred。Skill 路径不同：通过同类 source/prompt-injection
审查后可自动安装。MCP 更严：首次安装或任何扩权仍需 Owner 确认 exact version 与
permissions。

保留的底层模型继续区分 server、package、connection、capability、binding、health 与
quarantine。MCP 不是 Tool alias、Agent、Project object、Provider route 或 host-session
controller。广义 marketplace/family-management console 不在 2.0 范围。

## 2. Current truth 与 private envelope

Linux Personal 1.0 仍是 six-family。P5 MCP Tool transport/dynamic Tool 仍属于 Tool
family，不是 MCP family manager。ADR-0058 保留
`cognitiveos.personal.mcp-family/0.1` Personal-private envelope；1.0 projection
继续拒绝 `mcp`。本文不新增 Core schema、generic `Resource` 或 older-client coercion。

仍把 MCP 描述为完全 deferred 的 2026-08-27 architecture/formal-plan 文案处于
**pending architecture/plan reconciliation**。本文不改变已接受的 private-envelope
兼容性决策。

## 3. Discovery 与安全审查

助手可进行广泛联网发现，不必为普通读取逐次提问。每个 candidate 都是不可信输入。
Acquisition 前记录 source/exact version/digest/license/maintainer、hidden instruction、
prompt injection、dependency/executable code/supply chain、filesystem/network/command/
Secret/model/Tool permission、destination、compatibility、remove/update/rollback/
quarantine 行为。外部文本不能执行、安装或扩权；研究不得包含 raw credential 或 Owner
无权披露的第三方数据。

## 4. Acquisition、grant 与 admission

首次安装和每次扩权都需要 Owner 确认 exact version 与 permission set。Acquisition
只产生可全局复用的 pinned artifact；每个 Project/Member 仍需独立、最小权限 grant。
Install/connect 不隐式授予 Tool、Context、workspace、network、command、model、
secret、Memory 或 host-session authority。

MCP advertisement 仍是 candidate：

- operation -> Tool candidate；
- resource/data -> Context candidate；
- prompt/instruction -> Skill candidate；
- returned content 不自动进入 Memory；
- process/connection state 不形成 Runtime/Task authority。

每项必须进入其家族 admission。默认 2.0 managed DSH path 禁用 DSH native MCP 与 base
tools；不得借启用 DSH config 间接“资格化”MCP。

## 5. Version、client projection、Secret 与冲突

Client projection 只配置 exact admitted server/binding，不控制 Agent live session。
Write-back 必须是 persist-before-dispatch Intent/Effect，并有 version check、reconcile
和 receipt。版本必须 pin；update 重做安全审查和 compatibility test，显示 permission/
destination 差异并保留 rollback。新 client、扩权、endpoint 改变、filesystem/network
扩大或 conflict 都需 fresh preview；冲突 fail closed，不能靠 timestamp/model judgment
单独解决。

Credential 只在 approved SecretStore 与 daemon proxy。Raw secret 不得进入 MCP
metadata、ordinary config、DSH/Pi env、Agent message、Context、Memory、evidence 或 log。

## 6. 状态与 non-claims

目标产品区分 discovered/reviewing、confirmation-required、installing/connecting、
grant-required、partial projection、unhealthy、permission denied、drifted、stale、
conflict、quarantined、update available、compatibility failed、rollback available、
outcome unknown 与 removed-with-history；计数必须带 denominator。

Discovery、review、acquisition、grant、runtime 与 client projection 均为
**Requires-backend**；外部执行还需相应环境资格化。本文不实现或资格化 MCP server、
Tool、Agent、DSH path、client、marketplace、support、Gate、release、Profile 或
ecosystem claim。
