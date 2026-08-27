# MCP 资源家族

- 状态：已采纳的 advanced family；不在 Personal 2.0 OPC P0 路径
- 规范语言：[英文原文](mcp-resource-family.md)
- 决策：
  [ADR-0057](../../../docs/adr/0057-personal-2-0-mcp-resource-family.md)、
  [ADR-0058](../../../docs/adr/0058-personal-2-0-mcp-conversation-private-projection.md) 与
  [ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)

## 1. Scope

MCP 保持 Personal 第七 product family 的已采纳语义，server、package、connection、
capability、binding、health 与 quarantine 身份相互独立。MCP 不是 Tool alias、Agent、
Project object、Provider route 或 host-session controller。

在 Windows OPC rebaseline 中，MCP 是 **advanced/deferred capability**，不阻塞 Project
setup、Personal Assistant、DSH 员工、Conversation/Memory、Routine、Inbox、Knowledge、
Provider routing、UI 或首个 X/Twitter acceptance scenario。

## 2. Current truth 与 private envelope

Linux Personal 1.0 仍是 six-family。P5 MCP Tool transport/dynamic Tool 仍属于 Tool
family，不是 MCP family manager。ADR-0058 保留
`cognitiveos.personal.mcp-family/0.1` Personal-private envelope；1.0 projection
继续拒绝 `mcp`。本文不新增 Core schema、generic `Resource` 或 older-client coercion。

## 3. Family 行为

未来 surface 可显示 exact source/version/digest/license、secret-free connection、
capability digest/drift、health 与 permission/Task eligibility 的区别、binding/client
projection、update/rollback/quarantine/requalification/removal、conflict 与 receipt。
Install/connect 不授予 Tool、Context、workspace、network、model、secret 或 host-session
authority。

## 4. Candidate admission

- operation -> Tool candidate；
- resource/data -> Context candidate；
- prompt/instruction -> Skill candidate；
- returned content 不自动进入 Memory；
- process/connection state 不形成 Runtime/Task authority。

每项必须进入其家族 admission。默认 2.0 managed DSH path 禁用 DSH native MCP 与 base
tools；不得借启用 DSH config 间接“资格化”MCP。

## 5. Client configuration、Secret 与冲突

Client projection 只配置 exact admitted server/binding，不控制 Agent live session。
Write-back 必须是 persist-before-dispatch Intent/Effect，并有 version check、reconcile
和 receipt。新 client、扩权、endpoint 改变、filesystem/network 扩大或 conflict 都需
fresh preview；冲突 fail closed，不能靠 timestamp/model judgment 单独解决。

Credential 只在 approved SecretStore 与 daemon proxy。Raw secret 不得进入 MCP
metadata、ordinary config、DSH/Pi env、Agent message、Context、Memory、evidence 或 log。

## 6. 状态与 non-claims

未来产品区分 empty、installing/connecting、partial projection、unhealthy、
permission denied、drifted、stale、conflict、quarantined、update/rollback available、
outcome unknown 与 removed-with-history；计数必须带 denominator。

所有 runtime 行为均为 **Requires-backend** 且 deferred。本文不实现或资格化 MCP
server、Tool、Agent、DSH path、client、marketplace、support、Gate、release、Profile
或 ecosystem claim。
