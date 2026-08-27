# Personal Account Hub

- 状态：基于现有 Provider 基础的 Personal 2.0 已采纳目标
- 规范语言：[英文原文](account-hub.md)
- 当前权威基础：[Provider Control Plane](provider-control-plane.md)
- 凭据边界：
  [ADR-0055](../../../docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md)

## 1. 必须分离的事实

Account Hub 不得混淆 consumer subscription、Provider account/auth、API
billing/quota、model catalog、binding、Personal budget 与 actual usage/cost。
订阅不证明 API entitlement；Credential 存在不证明 reachability；quota 不能从 usage
推断；unknown usage/cost 不能显示为 0。

## 2. Reality ledger

| 边界 | 事实 |
|---|---|
| **Current implementation (Now)** | 已有命名 OpenAI/Anthropic/custom OpenAI-compatible account、API key SecretStore handoff、model discovery/manual model、fixed Agent binding、usage/cost、advisory budget/alert、audit 与 Provider UI。 |
| **Adopted Personal 2.0 target** | Settings Account Hub 分开 account/subscription/billing；支持 global/Project/employee/Task binding、Project/member/Task budget、DSH/Pi daemon proxy 与诚实 quota/usage。 |
| **Requires-backend** | 更多 adapter、OAuth/subscription observation、ADR-0055 import reader、binding hierarchy、budget enforcement、quota 与 runtime rebind/restart。 |

## 3. 有效绑定

```text
global default
  -> Project default
  -> digital employee override
  -> Task temporary override
```

最窄的已准入绑定生效。Role Blueprint 只声明能力要求，不保存具体 Provider。
变更必须说明对新/当前 Task 的影响和 DSH/Pi 是否需要重启。禁止 silent fallback、
ambient load balancing、caller credential 或 arbitrary auth header。

## 4. Secret 与 proxy 边界

API key、未来 OAuth/subscription token、ADR-0055 per-source import 与 custom endpoint
都必须进入 approved SecretStore 并由 daemon proxy 使用。Raw secret 不得进入 UI
storage/URL、Agent/employee config、DSH、Pi、MCP、Vault、Conversation archive、
ordinary config、SQLite、argv、environment、log、evidence 或 chat。

Import 成功只表示 SecretStore 已接收；Provider、model、quota 仍分别检查。

## 5. Budget、quota 与 usage

- Project budget：项目总 envelope；
- member budget：员工在项目内的分配；
- Task budget：单个 Task 的临时上限；
- Provider quota：Provider 给出的 allowance/reset/source；
- actual usage：按 Task/member/Project/account/model 归集并标 metering source。

Budget enforcement 在边界停止新 dispatch，并将调整请求送入 Inbox。它不能抹去
unknown Effect 或自行选择更便宜 route。现有 budget 在 enforcement 完成前必须继续
标为 advisory。

## 6. Setup、状态与恢复

流程为选择 Provider/method → non-logging secret path → 检查 redacted endpoint/
account/model/quota/trust → 选择 scope → 分开执行 reachability/credential/model
probe → 保存并返回来源对象。

失败后保留非 secret 输入。locked SecretStore、expired credential、unreachable
endpoint、model missing、quota unknown、stale catalog、budget exceeded 与
rebind-required 是不同状态。

Account Hub 覆盖 empty、loading、partial、stale、permission、error、unknown、
offline、budget-warning/stopped、success 与 archived-account。所有 reading 都带
source/period；percentage 必须带 denominator。

Expanded Account Hub、binding hierarchy、budget enforcement、DSH/Pi proxy composition
和 quota integration 均为 **Requires-backend**。本文不构成 Provider quality、
entitlement、cost accuracy、support、Gate、release、Profile 或业务结果声明。
