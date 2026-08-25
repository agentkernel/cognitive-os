# CognitiveOS Enterprise 产品设计

Date: 2026-08-25
Status: **candidate / owner-confirmed discovery / non-canonical / no implementation authorization**

本文件把 [owner Rounds 1–5](./02-product-direction-decision-brief.md)转成 Enterprise
candidate product design。Enterprise 未实现、未注册正式任务、未通过 release/Profile/Gate。
node/workspace daemon 继续是 local authority 唯一 writer；central plane 不写 remote SQLite。

## 1. Product center

**OWNER DECISION**：

```text
organizational intent
→ governed work
→ portable continuity/migration
→ independently accepted evidence/accountability
```

同一个 first Enterprise release 还包含 Provider/subscription management，但必须独立
acceptance、独立 capability truth，不得阻断或伪装 execution-assurance track。

产品 promise：

> 对 qualified Task class，在声明的 authority/resource/budget/deadline preconditions 持续
> 满足时提供 qualified completion guarantee；否则提供 terminal accountability：
> independently accepted completion，或带 evidence、owner、durable next action 的
> `blocked`/`failed` 终态。

这不是无条件成功保证。

## 2. Personas and JTBD

| Persona | Primary job | Needs |
|---|---|---|
| Platform/AI operator | 让跨工具 Agent work 可运行、可恢复、可迁移 | fleet、readiness、continuation、exception queue |
| Sponsor/work owner | 把意图、预算、Agent、结果与责任连起来 | Work/Assignment、qualified guarantee、receipt |
| Security/IAM | 限制 principal/workload/Agent authority | policy、approval、revocation、audit |
| Knowledge owner | 安全 enroll content 与控制 retrieval | classification、ACL freshness、purge、usage evidence |
| FinOps/Provider admin | 管理 entitlement/allocation/usage/cost | source truth、capability honesty、invoice refs |
| Auditor/incident responder | 证明发生了什么并恢复 | signed projection、evidence、incident/partition timeline |

JTBD 均是 **PRODUCT HYPOTHESIS**，尚无三家 design partner 验证。

## 3. Product form

- **OWNER DECISION**：Desktop Fleet 是 primary operator experience。
- **OWNER DECISION**：Web UI 是 supported fallback/deep-admin surface。
- 两者使用同一 authenticated governance backend、permissions、durable routes。
- central Web governance plane/integration services 与 customer/node daemon 分离。
- node daemon reauthorizes central requests；central command 不是 direct mutation。

## 4. Enterprise IA

| Area | Primary question |
|---|---|
| Home / Command Center | 哪些 work、fleet、policy、continuation 或 cost 需要行动？ |
| Governance Queue | 哪些 decision/exception/approval 待处理？ |
| Agent Registry | 哪些 external Agents/workloads 有 sponsor、eligibility、bindings？ |
| Work / Assignments | intent→Task→Assignment→attempt→acceptance truth 是什么？ |
| Continuations / Transfers | 哪个 Continuation Package 可安全迁移，谁批准？ |
| Providers | account、auth ref、entitlement、allocation、usage、cost、invoice source？ |
| Knowledge | 哪些 source enrolled、授权、fresh、revoked、purged？ |
| Policy / Approvals | 哪个 versioned decision/obligation 生效？ |
| Fleet / Nodes | 哪些 node 在线、attested、policy-fresh、partitioned？ |
| Evidence / Audit / Incidents | authority/effect/evidence/decision sequence 与 gaps？ |
| Integrations | IAM/HRIS/Secret/SIEM/PM/registry connectors 状态？ |
| System | tenancy、sync、retention、keys、versions、DR 是否健康？ |

Candidate route family 不是 API：

```text
/home
/queue
/agents/:agentId
/work/:taskRef
/continuations/:packageId
/providers/:accountId
/knowledge/:sourceId
/policy/:policyId
/fleet/:nodeId
/evidence/:receiptId
/incidents/:incidentId
/integrations/:connectorId
/system
```

## 5. Release scope

### 5.1 Track A — execution assurance and continuity

In scope：

- federated registry overlay、Sponsor/Principal/Workload distinctions；
- governed Work/Assignment；
- bounded same-binding retry；
- owner-confirmed cross-tool transfer/re-Assignment；
- Portable Continuation Package；
- qualified guarantee preconditions 与 terminal fallback；
- minimized central evidence projections；
- fleet/partition/revocation recovery。

### 5.2 Track B — Provider/subscription in same release

In scope with separate acceptance：

- organization Provider account/tenant refs；
- supported entitlement/seat/allocation controls；
- plan/account/auth/SecretRef/entitlement/budget/usage/cost/invoice separation；
- Provider-delegated mutations only where supported；
- source/freshness/capability honesty。

Track B 不得声称通用 consumer subscription mutation、remaining quota 或 invoice authority。

### 5.3 Knowledge track

Managed central index 是 first-release architecture requirement only if acceptance gates pass：
source opt-in、pre-index auth、approved classification、residency/retention、tenant partition、
encryption、provenance、ACL freshness、revocation/deletion/verified purge、prompt-injection/DLP。

## 6. Core workflows

### 6.1 Agent discovery, federation and sponsorship

```text
external registry/workload source
→ connector proposes identity/version/capability facts
→ match or create CognitiveOS overlay
→ sponsor/owner review
→ node presence + attestation
→ policy/entitlement/binding eligibility
→ registered overlay receipt
```

conflict、duplicate identity、stale source、missing sponsor 均 fail closed for new assignment。
CognitiveOS 不成为 HRIS/IAM/CMDB canonical SoR。

### 6.2 Governed work

1. link organizational intent/external work ref；
2. create/receive TaskContract；
3. resolve Principal、Sponsor、Agent binding、Provider/model、resources；
4. preview authority、budget、acceptance、qualified-guarantee preconditions；
5. node daemon admits and persists；
6. scheduler acquires fenced lease；
7. Agent produces candidate；
8. Intent persists before Effect dispatch；
9. independent verification；
10. accepted completion or terminal accountability receipt。

### 6.3 Continuation Package

Package contains versioned：

- TaskContract/objective/acceptance；
- decisions；
- authorized transcript excerpts/summaries；
- ContextView/source refs；
- artifacts + provenance；
- Effects/evidence；
- non-secret binding/budget state；
- blocker + durable next action。

It never claims hidden chain-of-thought、credentials、Provider-private state、unsupported native
session 或 unauthorized content。official native session continuation 可作为 package reference，
但不能替代 package audit/acceptance。

Transfer flow：

```text
failure/interruption
→ bounded automatic retry on same approved binding
→ if cross-tool/re-Assignment needed: build package
→ redact/authorize/compatibility check
→ owner preview and confirm
→ target node/adapter import
→ target reauthorizes and creates new attempt epoch
→ source/target receipts linked
→ independent acceptance remains required
```

### 6.4 Completion semantics

Qualified guarantee admission checks：

- Task class is qualified；
- authority valid and revocation watermark fresh；
- resources and target binding available；
- budget/deadline sufficient for declared bound；
- acceptance is satisfiable and verifier available；
- retry/reassignment policy explicit。

任一 precondition 失效，UI 必须显示 guarantee withdrawn reason，并切换 terminal
accountability；不可继续展示“guaranteed”。

### 6.5 Provider entitlement and allocation

```text
Provider tenant/account
→ supported auth/SecretRef
→ entitlement/seat pool
→ allocation to org/team/Agent/Task
→ binding/model
→ usage observation
→ budget/advisory or hard class
→ cost estimate/accrual/invoice ref
```

Provider mutation 必须 capability-probed、idempotent、audited；unsupported action 提供
first-party deep link，不模拟成功。

### 6.6 Knowledge enrollment and retrieval

Enrollment：

1. owner opt-in source；
2. prove connector and copying authority；
3. choose content/classes、residency、retention；
4. pre-index ACL/classification authorization；
5. ingest encrypted tenant partition；
6. record source/version/chunk/embedding provenance；
7. verify search/body authorization；
8. publish enrollment receipt。

Retrieval 先 authorization 后 search；result metadata 本身也敏感。revocation triggers query
deny、index tombstone、delete、verified purge receipt。retrieved content 是 untrusted data，
不能成为 instruction/policy。

### 6.7 Revocation, incident and partition

Reconnect order：

```text
revocation watermark
→ identity/policy/ACL updates
→ queued/in-flight work reevaluation
→ Knowledge deny/purge
→ evidence/usage outbox drain
```

partition 时 high-risk mutation、new enrollment、fresh approval、cross-tool transfer fail closed。
same-binding bounded execution only if pre-issued authority、policy、budget、deadline 和
revocation TTL all permit。

## 7. Screen specifications

### 7.1 Command Center

- cards：critical incidents、decision queue、guarantee-at-risk、continuation pending；
- dense summaries：fleet health、policy freshness、Provider/cost anomalies；
- role-based order；每项 deep-link；
- no vanity productivity score。

### 7.2 Governance Queue

columns：risk/reason、object、requester、sponsor、scope、policy version、expires、next action。
detail：request、digest、alternatives、blast radius、reversibility、evidence。approval is permission
to reevaluate, not completion。

### 7.3 Registry / Fleet

- external identity + overlay + sponsor + version + node presence；
- source/freshness、match confidence basis（not generic score）、conflicts；
- bulk refresh/export allowed；bulk suspend/revoke requires preview and partial receipt；
- unavailable node 不显示 healthy。

### 7.4 Work / Continuations

- Work master/detail：objective、authority state、Assignment、attempt、blocker、guarantee state；
- dual-lane timeline：authority vs observation；
- Continuation preview：included/excluded/redacted fields、source/target compatibility、budget、
  effects already occurred、owner confirmation；
- import failure preserves package and returns one recovery action。

### 7.5 Providers / Knowledge / Policy

- Providers：entitlement/allocation/usage/cost/invoice refs separate；
- Knowledge：enrollment、classification、ACL freshness、retention countdown、revocation/purge；
- Policy：version/diff/test corpus/decision reason/obligation；engine vendor hidden behind contract；
- all screens support stale/partial/permission/partition states。

## 8. Permission and role views

| Role | Default focus | Must not see/do by default |
|---|---|---|
| Operator | fleet/work/continuation | Knowledge body、secret、policy authoring |
| Sponsor | outcome/budget/acceptance | raw node diagnostics |
| Security | policy/revocation/incidents | business content without scope |
| Knowledge owner | source/enrollment/ACL/purge | unrelated Task content |
| FinOps | entitlement/allocation/usage/cost | prompt/context body |
| Auditor | immutable projections/receipts | mutation controls |

UI filtering 不替代 backend authorization。same durable route returns permission-aware projection。

## 9. State and recovery requirements

| State | Required UX |
|---|---|
| Empty | explain connector/enrollment/work origin + concrete action |
| Loading | preserve previous signed projection + freshness |
| Partial | name missing node/source and affected claims |
| Stale | watermark/version/timestamp + fail-open/closed consequence |
| Permission | exact denied scope/reason/request path |
| Partition | local authority facts separated from central projection |
| Incident | containment state、owner、next action、evidence |
| Purging | bounded progress、deny retrieval immediately、verification pending |
| Terminal | accepted or durable blocked/failed receipt |

## 10. Notifications and evidence explanation

Notification classes：critical decision expiry、revocation/partition、guarantee precondition loss、
transfer confirmation、Task failure、Knowledge purge failure、Provider allocation breach。routine
events aggregate in Home；every notification deep-links durable state。

Evidence uses three layers：

1. card summary；
2. readable explanation；
3. audit-grade signed projections/source refs。

source、freshness、authority-vs-observation、missing facts always explicit。

## 11. Accessibility

- Desktop Fleet and Web fallback share semantics/keyboard；
- queue/table supports screen-reader headers、sort、selection、bulk summary；
- approval dialog focus and consequence text；
- timeline has ordered textual alternative；
- policy diff uses added/removed labels, not color alone；
- tenant/scope visible in title and announced on change；
- localization supports Chinese/English 1.5× expansion；
- export/print evidence remains structured and redacted。

## 12. Metrics and counter-metrics

| Candidate metric | Counter-metric |
|---|---|
| ≥90% qualified work terminally accountable | false accepted completion = 0 |
| ≥80% authorized continuation packages imported without restatement | unauthorized field transfer = 0 |
| ≥95% central receipts link node/source ref | raw secret/body in receipt = 0 |
| revocation/purge meets declared SLO | post-revocation retrieval = 0 |
| ≥95% Provider cost displays source class | estimate labeled invoice = 0 |
| operator critical item identification ≤60 s | false-critical alert ≤10% |

No threshold has been observed/tested/Gate-proven。

## 13. Entry criteria and non-claims

Before implementation：

- three target organizations validate JTBD；
- one design partner provides real IAM/HRIS/Secret/SIEM/PM/Knowledge topology；
- SoR matrix approved；
- continuation package/redaction/compatibility semantics dispositioned；
- revocation/partition/Knowledge purge SLOs candidate-frozen；
- policy contract corpus exists；
- active implementation lease collision resolved。

Explicit non-claims：

- no Enterprise implementation、SaaS、tenant、desktop app、policy engine 或 Knowledge index exists；
- no hidden CoT/session portability；
- no unqualified success guarantee；
- no central authority writer or remote SQLite mutation；
- no universal subscription/invoice truth；
- no final architecture/ADR/contract/Gate/release authorization。
