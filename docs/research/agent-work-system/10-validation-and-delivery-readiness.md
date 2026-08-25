# CognitiveOS Agent Work System 验证与 Delivery Readiness

Date: 2026-08-25
Status: **candidate / owner-confirmed discovery / non-canonical / no implementation authorization**

本文件汇总 `02`–`09` 的 candidate acceptance、negative tests、validation route 与
pre-development readiness。**本 documentation task 未运行任何 product test、build、
benchmark、usability test、threat campaign 或 Gate。**

## 1. Decision trace

| Owner decision | Requirement documents |
|---|---|
| AI Workforce OS；Personal literal domain | `03` §1；`04` §1 |
| office worker/programmer/researcher, one Personal product | `03` §1/§18；`04` §18 |
| activation + continuity P0 | `03` §2/§13–18；`05` §6/§17 |
| native desktop + existing Web | `04` §2；`05` §2–5 |
| Profile/Instance split | `03` §5；`09` §3 |
| Provider taxonomy/cost honesty | `03` §5；`09` §4 |
| card-led + operational hybrid | `04` §4；`07` §1 |
| Task-first P1 | `03` §2；`09` §5 |
| shell framework comparison | `05` §5 |
| execution assurance Enterprise | `06` §1–6；`08` §1–9 |
| Continuation Package | `06` §6；`08` §8；`09` §6 |
| qualified guarantee + terminal fallback | `06` §6.4；`08` §9 |
| central + node sole writer | `08` §1–3 |
| federated SoR/registry | `06` §6.1；`08` §3–5 |
| pluggable policy | `08` §6 |
| minimized evidence | `08` §7 |
| managed Knowledge index | `06` §6.6；`08` §11 |
| Desktop primary + Web fallback | `07` §2；`08` §2 |
| local-canonical context + opt-in sync | `08` §10 |
| Provider same first release/separate acceptance | `06` §5.2；`08` §12 |
| Desktop priorities: Provider/Binding/Knowledge/Memory/Skill/Tool/usage/Context/Conversation | `02` §16；`03` |
| OSS-first, authority non-replacement | `05` §16；`09` §18；`12` |
| protected baseline / D13 conflict | `13` |

## 2. Claim policy

- `designed` 不等于 `implemented`；
- `implemented` 不等于 `HTTP-accessible`；
- `HTTP-accessible` 不等于 `tested`；
- `tested` 不等于 `Gate-proven`；
- ordinary CI/native/local 不等于 release/Profile/B01；
- estimate/accrual 不等于 invoice；
- Agent/session/process success 不等于 Task acceptance；
- this package is documentation evidence only。

## 3. Personal Desktop 1.0 candidate acceptance matrix

| Acceptance | Failure-first negative | Evidence needed |
|---|---|---|
| user-triggered discovery | background/silent trust creation rejected | adapter + UI tests |
| review before import | unreviewed facts cannot register | service/component |
| per-fact provenance | source-less fact shown unknown/rejected | projection test |
| manual fallback | discovery denied still completes registration | journey |
| Profile/Instance/Installation split | ID conflation does not create eligible binding | domain/UI |
| approved Provider access | cookie/password/token scrape unavailable | security |
| one qualified API path | unsupported path cannot claim ready | integration |
| explicit account/model binding | stale revision/wrong instance rejected | CAS tests |
| readiness receipt | unavailable/partial not rendered ready | projection/UI |
| source/freshness cost | estimate never invoice；unknown preserved | API/UI |
| restart/resume | wizard non-secret progress survives；secret does not | crash journey |
| capability honesty | unknown 200 stub does not enable action | network negative |
| accessibility | keyboard/AT activation complete | a11y scenario |
| one local Conversation | restart returns same revision/turns | storage/journey |
| one authorized source/input | unauthorized source never enters Context | auth/retrieval |
| Context inspection | included/omitted/changed/loss/freshness visible | projection/UI |
| explicit Memory retention | no implicit retain；reject candidate persists no Memory | domain/journey |
| Memory forget/tombstone | deleted Memory cannot reappear from index/cache | purge negative |
| Skill visibility | installed/enabled/bound/source separated | projection/UI |
| Tool visibility | descriptor/permission/availability/Effect class separated | projection/UI |
| source-typed tokens | unknown/partial source never shown complete | importer/API/UI |
| Conversation privacy | body excluded from telemetry/diagnostic | security |
| export/delete | omission and retained-authority receipt complete | journey |
| Continuation Checkpoint | drift reviewed；unknown Effect blocks unsafe resume | recovery |

P0 exit：one registered Agent + one supported Provider access + verified explicit Binding + one
restartable Conversation + one authorized source/input + inspectable Context + one explicit Memory
retain/reject decision + visible Skill/Tool readiness + source-typed token/cost status。

## 4. Personal threat and recovery tests

| Threat/fault | Expected |
|---|---|
| renderer tries secret IPC | denied；no value/log |
| malicious origin loopback | auth/origin denied |
| duplicate register/bind | idempotent or conflict receipt |
| Provider timeout | partial/unavailable；progress preserved |
| auth expires mid-wizard | metadata preserved；reauth required |
| model catalog revision changes | re-review binding |
| stale Agent health | eligibility fail closed |
| local usage only | cost labeled estimate/unavailable |
| daemon crash after request | persisted/no-persist fact reconciled |
| update incompatibility | activation blocked with supported route |
| unknown worktree | P1 mutation fail closed |
| imported history contains active tool call | stored inert；no dispatch/Task/Memory transition |
| Knowledge ACL revoked | query/body denied before retrieval；index purge scheduled |
| stale Knowledge ACL/source version | Context excludes or marks unavailable；no silent reuse |
| prompt injection asks for permission | treated as content；cannot change Tool/Task/policy |
| Memory extractor overclaims | candidate preview；no direct write |
| Memory tombstoned but vector hit remains | result denied；purge verification fails |
| Skill package revision drifts | binding/readiness blocked pending re-review |
| Tool descriptor changes | permission/Effect path revalidated；no implicit enable |
| Context token truncation | explicit loss/omission；required source fail closed |
| Conversation delete | unrelated Task/Evidence/retained Memory not silently deleted |
| desktop shell invokes generic fs/exec | IPC denied and audited |
| updater downgrade/tamper | install blocked；previous version preserved |

## 5. Personal UX scenarios

1. first-run office worker reaches first Conversation within candidate 10 minutes；
2. exits after Review，resumes without re-discovery；
3. denies discovery permission，uses manual registration；
4. expired auth recovers without duplicate account；
5. unknown cost remains honest and does not prevent readiness unless policy says；
6. lost readiness notification deep-links exact object；
7. keyboard-only complete wizard and binding；
8. 960 px window preserves location/action；
9. reduced-motion flow keeps orientation；
10. screen reader announces state/source/freshness/next action；
11. programmer reviews Agent/Binding/Tool/worktree drift before governed Work；
12. researcher explains Knowledge/Memory/Context differences and source loss；
13. office worker exports/deletes Conversation with complete retention receipt；
14. offline return keeps local archive and names unavailable Provider/source operations；
15. Continuation Checkpoint blocks unknown Effect and preserves next action。

## 6. Enterprise first-release acceptance tracks

### 6.1 Track A — execution assurance and continuation

| Acceptance | Negative |
|---|---|
| federated registry overlay maps source/sponsor/workload | ambiguous identity blocks assignment |
| node sole-writer | central request cannot write DB/state directly |
| signed/versioned request | forged/replayed/unsupported version denied |
| owner-confirmed cross-tool transfer | no confirmation → no import |
| same-binding bounded retry | bound exhaustion → terminal disposition |
| Continuation Package typed | hidden/private/secret/unauthorized fields excluded |
| source Effects reconciled | unknown irreversible Effect blocks replay |
| target import idempotent | duplicate package no duplicate Effect |
| independent acceptance | Agent/native session cannot self-complete |
| qualified guarantee | precondition loss withdraws guarantee |
| terminal accountability | blocked/failed has evidence/owner/next action |
| partition policy | high-risk transfer/enrollment fails closed |

### 6.2 Track B — Provider/subscription, same release

Track B ships in same release but has independent acceptance：

- supported Provider/account/auth/entitlement/allocation/model/binding；
- separate plan/account/auth/SecretRef/entitlement/budget/usage/cost/invoice；
- delegated mutation only after capability proof；
- source/freshness and advisory/hard enforcement class；
- estimate not invoice；
- unsupported consumer mutation absent；
- track failure blocks Provider track acceptance and release claim according to release policy，不得
  伪装 execution track capability。

### 6.3 Track C — managed Knowledge

- source opt-in；
- pre-index authorization；
- approved content/classification only；
- tenant partition + encrypted storage；
- provenance to source/version/chunk/embedding；
- ACL freshness SLO；
- authorization before search and body；
- revocation immediate deny；
- deletion + verified purge denominator；
- prompt injection/DLP/poisoning controls；
- source remains SoR。

## 7. Enterprise failure-first negatives

| Negative | Required outcome |
|---|---|
| hidden CoT requested | explicit not portable |
| credential in transcript | redacted/rejected |
| unauthorized source ref | package build/import denied |
| target adapter incompatible | transfer blocked before assignment |
| same package ID/different digest | conflict |
| source Effect unknown | no replay |
| authority revoked during retry | stop/terminal receipt |
| budget/deadline exhausted | qualified guarantee withdrawn |
| unsatisfiable verifier | admission deny or terminal blocked |
| stale revocation watermark | high-risk fail closed |
| cross-tenant projection/query | deny + security event |
| stale Knowledge ACL | search/body deny |
| purge omits embedding/cache | purge fail, no success receipt |
| prompt injection requests policy action | treated as data, denied |
| Provider estimate labeled invoice | test fail |
| central direct SQLite attempt | impossible/denied by topology |

## 8. Performance and reliability candidate bounds

These are proposed targets, not measurements：

| Area | Candidate bound |
|---|---|
| Personal cold shell usable | p95 ≤3 s on qualified Windows device |
| Personal route switch | p95 ≤200 ms with cached projection |
| readiness refresh | visible progress ≤500 ms；bounded timeout/recovery |
| Enterprise queue query | p95 ≤2 s for declared denominator |
| node projection lag | SLO named per risk class；stale explicit |
| revocation propagation | high-risk SLO prerequisite before release |
| Continuation package build | bounded by package size；progress/cancel |
| Knowledge revocation deny | immediate logical deny before physical purge |
| purge verification | deadline and exact denominator |
| central RPO/RTO | owner/security disposition required |

No performance claim without fixed workload、hardware、denominator、warm/cold、source latency。

## 9. Desktop framework spike acceptance

Compare Tauri and hardened Electron under same matrix。Tauri is conditionally preferred；Electron
is rejected unless equivalent evidence reverses current assessment：

1. existing SPA route/network compatibility；
2. Windows package/install/uninstall/signing/update/rollback；
3. accessibility and keyboard/AT；
4. WebView/runtime provenance + SBOM；
5. process isolation/sandbox；
6. narrow IPC allowlist negatives；
7. SecretStore/browser/device auth without renderer secret；
8. crash/restart/deep-link/tray/notification；
9. cold/warm startup、RSS、package size；
10. future macOS/Linux risk；
11. maintenance/security update ownership；
12. no authority writer in shell。

Tauri acceptance additionally requires exact version/digest、license/NOTICE、SBOM/provenance、CSP、
no remote content、WebView origin-confusion regression、signed updater/rollback and dependency
removal test。Winner requires ADR after evidence；spike does not itself authorize implementation。

## 10. Supported validation environments

| Validation | Supported environment | Claim ceiling |
|---|---|---|
| Markdown/link/static | local Windows | docs consistency |
| TS unit/network/component/a11y/build | local Node/pnpm, CI | implementation test only |
| Rust fmt | local Windows allowed | formatting |
| Rust build/test/Clippy | CI Ubuntu/Windows MSVC or exact pushed Linux | test only |
| Personal daemon/browser | exact pushed revision on qualified Linux/Windows route | journey only |
| Windows desktop packaging | qualified clean Windows environment | candidate packaging |
| macOS/Linux desktop | separately qualified environment | platform-specific only |
| Enterprise partition/HA/tenant | preregistered isolated campaign | hypothesis |
| Knowledge purge/security | preregistered representative corpus/tenant | no production claim |

`DEV-WIN-GNU-01` 不运行 Rust linking/compiling。`B01-Desktop-Linux-002` 不用于普通开发。
No test executed for this documentation task。

## 11. Candidate implementation waves

No formal `P*-T*` IDs are registered。

### 11.1 Personal

1. **Ownership/canonical scope**：resolve P7-T05/D13、clean branch/lease、accepted release-scope ADR；
2. **Desktop spike**：fixed matrix, no winner before evidence；
3. **P0 projection foundation**：capability/stub/error/source/freshness；
4. **Activation wizard + Agent discovery/manual registration**；
5. **Provider link/readiness/binding/source-typed usage receipt**；
6. **Conversation archive + authorized input + Context inspection**；
7. **Memory retain/forget + Skill/Tool readiness + Knowledge source minimum**；
8. **native shell packaging/deep links/tray**；
9. **P0 three-persona journey/a11y/privacy/security acceptance**；
10. **next vertical slice**：owner Assignment→preview/admit→governed Task→evidence。

### 11.2 Enterprise same-release staged tracks

1. federation/system-of-record prototype；
2. node protocol + minimized projection；
3. registry overlay + Desktop/Web route parity；
4. governed Work and qualified guarantee；
5. Continuation Package + owner-confirmed transfer；
6. Knowledge opt-in index + purge；
7. Provider/subscription independent track；
8. policy contract/plugin harness；
9. partition/revocation/HA/DR；
10. integrated release acceptance。

Provider track can develop in parallel but same-release exit requires its independent acceptance。

## 12. Development Readiness Gates

| Gate | Current result | Required to pass |
|---|---|---|
| owner requirements | PASS | Rounds 1–5 + scope expansion confirmed |
| candidate document set | PASS after this package validation | `03`–`13` consistent |
| implementation authorization | FAIL / absent | formal owner task instruction + lease |
| canonical product scope | FAIL now | accepted ADR reconciling Linux `1.0.0` and Desktop candidate |
| repository ownership | FAIL now | active P7-T05/D13 closure/transfer |
| Personal API gap disposition | PARTIAL | private/Lane-CTR decision |
| desktop framework | OPEN | fixed spike + ADR |
| Personal user validation | OPEN | interviews/diary/commitment |
| Enterprise design partners | OPEN | ≥3 org validation + topology |
| Continuation protocol | OPEN | schema/threat/conformance |
| Knowledge security | OPEN | auth/tenant/purge/injection campaigns |
| Enterprise deployment | OPEN | tenant/HA/DR/SLO/security ownership |

## 13. Active collision and ownership blocker

**FACT at 2026-08-25**：active `lease/personal/P7-T05/control-plane-foundation`,
P7-T05/D13 owns implementation/governance paths and works on Control Plane Work inventory +
governed Task creation。Its external client branch/worktree ownership overlaps future
shell/Home/Providers/Work/Conversation/UI shaping。

Before implementation：

1. inspect current snapshot and lease；
2. close or explicitly transfer P7-T05/D13；
3. accept/reject product-semantic ADR and formal-plan rebaseline；
4. resolve dirty/untracked ownership；
5. create one formal task branch/Draft PR/lease；
6. never reuse this discovery lease as implementation authorization。

## 14. Blockers, dependencies, owners

| Item | Owner candidate | Next action |
|---|---|---|
| P7 collision | active task owner/repo owner | close/transfer D13 and clean ownership |
| canonical 1.0 conflict | product owner/ADR owner | reconcile Linux `1.0.0` vs Desktop candidate |
| shell framework | Personal architecture/security | run equivalent spike |
| contract gaps | Lane-CTR owner | disposition after private prototype need |
| Personal research | product owner | recruit target operators |
| Enterprise partner/topology | product/enterprise sponsor | obtain real topology |
| Knowledge index | security/Knowledge/data owner | threat + retention + purge design |
| policy | architecture/security | representative corpus + contract |
| Provider controls | connector/FinOps | capability matrix and separate acceptance |

## 15. Stop/go criteria

### GO to formal shaping only when

- owner explicitly authorizes implementation；
- collision and worktree ownership resolved；
- exact first slice and writable paths leased；
- validation environments available；
- public/private contract disposition recorded；
- no axiom/security contradiction。

### STOP

- unknown concurrent changes；
- requirement contradicts A1–A8；
- secret/credential exposure；
- destructive/irreversible operation without confirmation；
- unsupported Provider mutation required for acceptance；
- Knowledge authorization/purge cannot be proven；
- qualified guarantee preconditions cannot be made testable。

## 16. Open-source PoC gates

No OSS candidate may enter product paths before all common gates pass。Exact project/version/license
evidence is in [open-source reuse assessment](./12-open-source-reuse-assessment.md)：

| Gate | Required evidence | Failure disposition |
|---|---|---|
| identity | exact project/tag/commit/package/image digest | reject unknown/mutable |
| license | root/file/tree license + NOTICE + trademark/assets | reject incompatible/ambiguous |
| security | advisory history, threat model, sandbox, secret canary | reject high uncontrolled risk |
| provenance | source→builder→recipe→artifact attestation | not releaseable |
| SBOM | SPDX 2.3 + CycloneDX 1.6 incl. transitive/native/container | not packageable |
| authority | upstream cannot write Task/Binding/Memory/Tool/Context authority | reject |
| secret | no env/argv/config/log/trace/upstream DB secret | reject |
| failure | timeout/crash/duplicate/stale epoch/unknown Effect fail closed | reject |
| data | import provenance/loss/export/delete/rebuild | reject state lock-in |
| removal | dependency removal preserves CognitiveOS authority | reject |
| upgrade | pinned upgrade/rollback/security owner/SLO | defer |
| accessibility | keyboard/AT/high-contrast/reduced-motion for user-facing deps | defer |

Candidate-specific gates：

- **Tauri**：fixed shell matrix、CSP/no remote content、narrow IPC、signed update/rollback、
  WebView origin test、Windows AT。
- **ccusage**：sanitized explicit input、no ambient scan、unknown≠0、estimate≠invoice、
  idempotent delete/reimport。
- **OpenHands**：isolated workspace、no Docker socket/credential/home、worktree A8、
  independent verifier。
- **LiteLLM**：fixed route/model、routing/fallback/virtual keys/budget/logging disabled、
  daemon ledger canonical。
- **RAGFlow**：pre-query authorization、rebuildable index、source/version provenance、
  complete purge。
- **Mem0**：candidate-only、explicit admission、version/tombstone、no resurrection、
  portable export。
- **OpenLLMetry**：content capture disabled、allowlisted local spans、telemetry cannot complete Task。
- **MCP SDK**：post-1.0 formal activation、exact spec/package pin、descriptor/result remain candidate。

## 17. Domain-specific acceptance packs

### 17.1 Provider/account/usage truth

- account/plan/auth/entitlement/model/Binding/usage/budget/cost/invoice remain separate；
- duplicate/ambiguous account identity cannot silently merge；
- local usage cannot infer consumer-plan remaining allowance；
- `0`、`unknown`、`unavailable`、`not reported` distinct；
- no supported mutation means Open Provider/manual/unavailable, not fake disabled lifecycle；
- invoice requires Provider/finance source；
- binding change uses revision/CAS and invalidates relevant continuation readiness。

### 17.2 Knowledge and Context

- authorization before index search/body；
- source/ACL/version/purpose/freshness recorded；
- revoked source denied immediately；
- physical purge covers chunk/embedding/cache/queue denominator；
- prompt injection cannot grant Tool/Task/policy authority；
- Context omission/truncation/conflict explicit；
- required source loss fails closed；
- index loss rebuilds without changing source authority。

### 17.3 Memory, Skill and Tool

- Memory candidate cannot write directly；
- retain/reject/forget produce durable receipt；
- tombstone prevents extractor/index resurrection；
- Skill revision/package/source/publisher validated；
- Tool descriptor/permission/availability/Effect class distinct；
- Skill/Tool drift forces re-review, not silent continue；
- no generic Resource mutation bypasses family lifecycle。

### 17.4 Conversation and privacy

- local archive restart/resume deterministic；
- import active content inert；
- export lists included/omitted data and digest；
- delete distinguishes transcript/index/Memory/Task evidence；
- telemetry/diagnostic excludes body and secret；
- offline behavior honest；
- Continuation Package excludes hidden CoT、secret、private session、unauthorized content；
- unknown Effect cannot replay。

### 17.5 Desktop shell

- renderer/shell never authority writer；
- IPC allowlist rejects generic fs/net/exec/secret；
- loopback auth/origin/channel binding proven；
- signed package/update/downgrade/tamper/rollback；
- crash/restart/deep-link/tray idempotency；
- no transcript/secret in crash dumps/logs；
- Windows keyboard、screen reader、200% zoom、high contrast、reduced motion/transparency；
- narrow-window recovery without hidden action。

## 18. Documentation validation report

Executed on 2026-08-25：

| Check | Result |
|---|---|
| heading hierarchy across 10 owned package files | PASS |
| fenced code/Mermaid balance | PASS |
| duplicate heading/anchor scan | PASS, zero duplicates |
| relative local links | PASS, zero broken |
| trailing whitespace | PASS |
| stale D12/client-blocker wording | PASS, zero matches |
| `CognitiveOS` terminology/spelling scan | PASS, zero known misspellings |
| ReadLints on edited/new files + lease ledger | PASS, no diagnostics |
| `git diff --check` limited to owned paths | PASS |
| source-map inspection | PASS: no `docs/agent-work-system/**` mapping |
| protected `docs/design/**` | intentionally untouched; only read/indexed |
| product tests/build/benchmark/usability/Gate | **not-run** |

These results validate documentation structure only。No product test、capability、release or Gate
claim is implied。

## 19. Explicit non-claims

- no code、service、dependency、schema、contract、ADR、formal task、branch、PR；
- no Personal/Enterprise implementation；
- no test/Gate/release/Profile/market validation；
- no hidden context portability or unconditional completion；
- no consumer subscription/invoice truth；
- no source-map/handbook mutation；
- no resolution of active P7 implementation collision；
- no code/state migration or dependency install from any OSS project；
- no supersession of Linux `1.0.0` by the Desktop candidate。

Canonical conflict and D13 sequencing are defined in
[baseline delta map](./13-control-plane-baseline-to-personal-desktop-1.0-delta.md).
