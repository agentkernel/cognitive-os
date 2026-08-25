# CognitiveOS Enterprise 交互与视觉候选规范

Date: 2026-08-25
Status: **candidate / owner-confirmed discovery / non-canonical / no implementation authorization**

本规范建立在 [Enterprise 产品设计](06-enterprise-product-design.md)上。所有 dimensions、
tokens 与 interaction pattern 均是 design candidate。Enterprise 未实现；Desktop Fleet 与
Web fallback 只描述目标关系。

## 1. Visual relationship

- shared restrained CognitiveOS brand、typography roles、semantic states、evidence grammar；
- Personal：spacious consumer cards + hybrid operations；
- Enterprise：同品牌，但 queue/fleet/Activity/evidence 提高 density；
- 不使用“数字员工”头像、org-theater、neon control-room、terminal-only aesthetic；
- authority、observation、risk、freshness 通过结构与文字区分。

## 2. Surface relationship

| Surface | Role | Parity requirement |
|---|---|---|
| Desktop Fleet | primary operator experience | full queue/fleet/work/continuation workflows |
| Web fallback | supported fallback/deep-admin | same backend、permissions、durable routes、semantic states |

Desktop-only OS integration（tray、native notification、window restore）在 Web 中用 in-app
attention 替代。任何 mutation、approval、evidence interpretation 必须 semantic parity；
不能形成 desktop 与 Web 两套 authority behavior。

## 3. Desktop shell candidate

### 3.1 Layout

```text
┌ tenant/scope + global status + search/command + user/session ─────────────┐
├ global nav 220 ┬ queue/master 360–440 ┬ detail flexible ┬ inspector 360 ┤
│ Home           │                      │                 │               │
│ Queue          │                      │                 │               │
│ Registry       │                      │                 │               │
│ Work           │                      │                 │               │
│ Continuations  │                      │                 │               │
│ Providers ...  │                      │                 │               │
└────────────────┴──────────────────────┴─────────────────┴───────────────┘
```

Candidate：

- minimum 1180 × 720；target 1440–1920；
- sidebar 220 px；collapsed 64 px；
- master 360–440 px；inspector 360–480 px；
- toolbar 56 px；dense row 36–44 px；
- tenant/scope 永久可见；scope switch 清楚 announce 并清空不兼容 selection。

<1180 px：隐藏 inspector 为 route/sheet；<960 px Web fallback 只支持 bounded review/decision，
不声称 fleet administration parity。

### 3.2 Navigation

Home、Queue、Registry、Work、Continuations、Providers、Knowledge、Policy、Fleet、Evidence、
Integrations、System。Nav group 可折叠但 active item、scope、count reason 可见。count 只表示
actionable items，不表示 raw events。

## 4. Shared design candidates

引用 Personal visual tokens，但 Enterprise 增加 density aliases：

| Token | Personal | Enterprise |
|---|---|---|
| page gutter | 24 | 20–24 |
| card padding | 20–24 | 16–20 |
| operational row | 40–48 | 36–44 |
| detail section gap | 24 | 20 |
| inspector label | 13/18 | 12–13/18 |

Typography、semantic color、focus ring、radius 与 material shared。Enterprise 不通过更小于
12 px 文本换取 density；density 来自 structure、columns、progressive disclosure。

## 5. Core patterns

### 5.1 Governance queue

- columns：priority reason、risk、object、requester、sponsor、scope、age/expiry、next action；
- stable sort、saved view candidate、filter chips、bulk-safe selection；
- detail：request digest、policy result、alternatives、reversibility、evidence；
- approve/deny/narrow 互斥且 consequence preview；
- stale request disables mutation with reason。

### 5.2 Fleet table and split pane

- columns：node、realm/tenant、online/attested、policy version、revocation watermark、
  evidence backlog、last seen；
- grouping by scope/location only if source authoritative；
- selected node detail shows authority facts vs central observations；
- partitioned 不显示 offline=failed；显示 last authoritative local fact and central freshness。

### 5.3 Evidence timeline

three layers：

1. card receipt；
2. readable sequence；
3. audit timeline/export。

timeline event anatomy：kind、authority/observation、actor/principal、object、time、source、
digest/ref、freshness、limitations。policy diff、Effect、verifier、owner decision 各有 icon +
label，不用 rainbow。

### 5.4 Continuation preview

Two-column source→target：

- package version/digest/signature；
- included sections；
- excluded/redacted/private/unsupported facts；
- source/target Agent/tool/model compatibility；
- Effects already occurred；
- budget/deadline/acceptance；
- authority requested；
- owner confirm。

Hidden CoT、credentials、Provider-private state 明示 “not portable”，不是 missing-error。

### 5.5 Knowledge enrollment

Stepper：

1. source + owner；
2. connector authority；
3. approved content/classification；
4. tenant/residency/encryption；
5. retention/purge；
6. ACL freshness/revocation；
7. review；
8. ingest progress/receipt。

search result 在 authorization 前不得显示 title/count/snippet。revoked source 立即 deny，
purge progress 与 verified receipt 分离。

### 5.6 Provider allocation and cost

- hierarchical allocation view：tenant/account→pool→scope→Agent/Task；
- plan/account/auth/entitlement/allocation/budget/usage/cost/invoice columns separate；
- source/freshness sticky；
- bulk allocation mutation requires diff、conflict strategy、partial receipt；
- invoice view 是 external ref/read-only unless delegated capability proven。

## 6. Component anatomy

| Component | Anatomy |
|---|---|
| TenantScopeSelector | tenant、realm、source、permission、switch consequence |
| QueueItem | reason、risk、object、owner、expiry、next action |
| FleetNodeRow | node identity、attestation、policy/watermark、freshness |
| PolicyDecision | Permit/Deny/RequireApproval、reason、obligations、version、digest、expiry |
| ContinuationPackageCard | source/target、version、coverage、redactions、signature、status |
| GuaranteeBadge | qualified/not-qualified/withdrawn + precondition reason |
| KnowledgeEnrollment | source、classification、residency、retention、ACL freshness |
| EvidenceReceipt | disposition、verifier、Effects、source refs、missing facts |
| CostSource | estimate/accrual/invoice/unavailable + source/time |
| PartitionBanner | affected scope、last contact、allowed/denied actions、recovery |

## 7. Bulk action safety

- bulk refresh/export 可直接；
- bulk assign/allocate/suspend/revoke/purge 必须 preview exact denominator；
- mixed permission returns itemized allowed/denied，不能 all-green；
- destructive action 输入 consequence phrase 仅高风险使用；
- partial success persists per-object receipt；
- retry uses idempotency key and excludes already-completed items；
- undo only when backend semantics truly reversible。

## 8. Query, filter, command palette

Global search groups：objects、routes、commands。敏感 Knowledge metadata 和 objects 在 search
前 authorization。query language candidate supports `state:`, `owner:`, `source:`,
`stale:`, `risk:`, `tenant:`；invalid query inline explains。

Command palette：

- context actions first；
- disabled reason；
- selected tenant/scope；
- confirmation for cross-scope mutation；
- focus return；
- no hidden generic “run command”。

## 9. Approval and destructive interactions

Approval card：

- exact request/digest；
- why stopped；
- affected resources；
- risk/reversibility；
- cost state；
- recommendation + alternatives；
- expiry/staleness；
- separation-of-duty status。

Approval only permits reevaluation。Revoke/suspend/purge 显示 propagation state、in-flight work
handling、rollback limits。Unknown external Effect requires incident/reconcile，不能显示 undo。

## 10. State matrix

| State | Required presentation |
|---|---|
| Empty | source/setup explanation + concrete connector/work action |
| Loading | prior signed projection remains with stale marker |
| Partial | missing nodes/sources and claim impact |
| Stale | timestamp/version/watermark and blocked actions |
| Offline | local vs central facts separated |
| Partition | policy mode、expiry、allowed/denied classes |
| Permission | exact scope/reason/request route |
| Incident | severity、containment、owner、timeline、next action |
| Purge pending | retrieval denied now；deletion progress separately |
| Conflict | revisions/diff；preserve user intent |
| Terminal | accepted or durable blocked/failed receipt |

## 11. Role-based information priority

| Role | First viewport | Secondary |
|---|---|---|
| Operator | queue、fleet、continuation、work failures | provider/knowledge status |
| Sponsor | outcome、guarantee、budget、acceptance | technical evidence |
| Security | policy/revocation/incidents/attestation | work summaries |
| Knowledge owner | enrollment/ACL freshness/purge | retrieval evidence |
| FinOps | allocation/usage/cost/source | work detail without content |
| Auditor | receipts/timeline/export/coverage | no mutation |

Role view is presentation preference after backend authorization，不是 ACL。

## 12. Notifications and escalation

| Severity | Channel candidate | Rule |
|---|---|---|
| Critical | Desktop + in-app + configured enterprise channel | revocation failure、cross-tenant leak、purge failure、uncontained Effect |
| Action | Desktop/in-app | approval、transfer、guarantee precondition loss、Task failure |
| Summary | in-app digest | stale fleet、cost anomaly、connector lag |
| Informational | Activity only | routine success/refresh |

dedupe by durable object/revision；ack does not resolve underlying state；deep-link restores tenant/
scope/object。Web fallback shows same queue even if OS notification unavailable。

## 13. Motion, accessibility, localization

- transitions 100–180 ms；large pane 180–220 ms；no ornamental fleet pulse；
- progress uses durable counters/stages；partition/revocation no fake animation；
- reduced motion preserves location；
- keyboard：`Ctrl/Cmd+K` command、`G then Q/W/F` candidate nav、`J/K` queue、`Shift+Space`
  row select、`Esc` close；
- table/grid semantics and visible focus；
- timeline textual alternative；
- policy diff added/removed words；
- tenant/scope announced；
- contrast WCAG 2.2 AA candidate；
- Chinese/English 1.5× expansion；timezone/currency/unit source shown；
- evidence export/print has header、scope、generated time、redaction and claim ceiling。

## 14. Web fallback behavior

- same durable routes/IDs；deep links map between surfaces；
- same permission and error envelopes；
- same approval/continuation preview；
- no OS-only feature assumed；
- unsupported browser operation shows desktop path，not fake button；
- responsive replacement prioritizes queue/detail，not full fleet grid；
- logout/session/tenant switch semantics identical。

## 15. Usability scenarios

| Scenario | Candidate acceptance |
|---|---|
| Queue triage | operator finds highest-risk actionable item ≤60 s |
| Transfer | owner identifies included/excluded package data and confirms correct target |
| Guarantee loss | sponsor sees which precondition failed and terminal fallback |
| Partition | operator distinguishes local authority fact from stale central projection |
| Revocation | security confirms deny before purge completes |
| Knowledge purge | owner verifies no search/body exposure and receives purge receipt |
| Cost dispute | FinOps distinguishes estimate/accrual/invoice source |
| Keyboard | queue→detail→approve/deny→receipt without trap |
| Web fallback | same object/permission/state accessible by durable route |
| Auditor export | read-only redacted evidence retains ordering/digests/coverage |

## 16. Non-claims

No final tokens、prototype、Desktop Fleet、Web UI、multitenancy、policy engine、Knowledge index、
usability test、contrast result、implementation、release 或 Gate evidence exists。Pixel values are
candidate only；visual design cannot override daemon authority or capability honesty。
