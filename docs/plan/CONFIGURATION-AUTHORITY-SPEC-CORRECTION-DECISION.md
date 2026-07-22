# Configuration Authority Spec-Correction Feasibility Decision

- 战役：`CONFIG-AUTHORITY-FOUNDATION-THEN-MGMT-CONFIG-CFR`
- 原子批：D-016 / D-022 规范修正可行性裁决（Lane-CTR）
- 基线：2026-07-22，`origin/main@d1a4d6a`（PR #48 merge）
- 裁决：**NO-GO**
- 变更分类：docs-only 裁决；未修改 normative machine assets
- 状态纪律：`273 REQ / 55 errors / 61 schemas / 84 vectors / 59 pass / 25 not-run / self-check 40 / matrix impl 70 / Profile implemented 0`

## 1. 裁决

D-016 / D-022 的三个缺口**不能**全部解释为“既有 normative 语义的纠错型收敛”。现有合同固定了 management authority 的安全不变量与流程骨架，但没有唯一固定下列机器合同：

1. Management operation set 的 wire 名称集合、operation-set digest、逐 operation payload schema 与 target/readback profile；
2. `PrivilegedManagementSession` / `ManagementApprovalDecision` 的签名算法、key resolution、签名域、signed projection/exclusions、编码和 trust-root binding；
3. 覆盖完整字段、顺序完整性、保留、敏感度、导出和 authority commit 原子性的 authoritative audit carrier。

因此至少需要新增或收紧 normative/wire surface，并补充当前不存在的失败映射。它不是 typo、断链或“文本已经逐字段定名但漏交机器 schema”的修正型变更。按 IMP-01，v0.1 冻结期不得实施这些变更；D-016 保持 `deferred-to-v0.2`，D-022 改为 `NO-GO / deferred-to-v0.2`，CA-1～CA-8 继续阻断。

本批没有修改 registry、schemas、errors、transitions 或 vectors；既有 vector `expected` 未改，Management 目标向量未执行，状态保持 84/59/25、self-check 40、Profile implemented 0。

## 2. 裁决口径

本裁决按仓库优先级核对 digest-pinned machine schema/registry/vector 与 normative companion，再核对实现是否提供可消费事实。下列证据均不单独证明合同充分：

- schema 中的开放字符串或开放 JSON；
- vector input 中的 operation 字符串或布尔 `authority_signature_valid`；
- 字符串长度、digest 存在、fixture 构造器或 schema-valid；
- Event 开放 `payload`、outbox destination、telemetry/log/trace、`audited=true`；
- SQLite 私有列或事务实现；
- AKP result 的可选 `audit_ref`；
- HTTP 2xx 或通用 `ok`。

## 3. A — Management operation set

### 3.1 operation 逐项审计

| operation | normative 业务语义来源 | exact machine 固定 | 分类 | 登记为固定 wire operation 的结论 |
|---|---|---|---|---|
| `session.create_restricted` | Core REQ-MGMT-FALLBACK-001 要求建立受限 session；AKP §10.1 描述建立 session 消息的绑定事实；RFC-0001 §7.5 定义 session authority | 仅 `MGMT-FALLBACK-008` 的字符串；AKP request `operation` 与 session scope `actions` 均为开放字符串 | 已有宽泛业务语义；名称仅由 vector 固定；无 payload/wire binding | **不能直接登记**。建立消息未命名，未固定 issuance request/result、signature verification 与错误闭包；登记会新增 wire surface |
| `status.inspect` | Core `ReadState` 与 REQ-MGMT-FALLBACK-001 的“检查状态” | 仅 `MGMT-FALLBACK-008` 字符串 | 已有语义；名称仅由 vector 固定 | **不能直接登记**。缺 selector/result/read authority profile；vector 只要求 reachable |
| `capability.revoke` | Core capability 撤销不变量与 REQ-MGMT-FALLBACK-001 的“撤销 capability” | 仅 `MGMT-FALLBACK-008` 字符串 | 已有语义；名称仅由 vector 固定 | **不能直接登记**。目标 capability、expected revocation version、receipt 与失败闭包未绑定；Core 文本类别 `CAPABILITY_REVOKED` 未在 55-code registry 登记 |
| `execution.stop` | Core `Cancel`、§18.3 stop 语义、REQ-MGMT-FALLBACK-001 | `MGMT-FALLBACK-008`、`MGMT-IDEM-006` 与 session fixtures 固定字符串 | 已有语义；机器场景多处使用；无独立 operation descriptor/payload schema | **不能直接登记完整 wire contract**。名称较充分，但 stop target/reason/deadline/result 与 management envelope 未形成闭合绑定 |
| `effect.reconcile` | Core `ReconcileEffect`、Effect 状态机、REQ-MGMT-RECOVERY-001/REQ-MGMT-FALLBACK-001 | `MGMT-FALLBACK-008` 固定字符串 | 已有完整流程语义；名称仅由 management vector 固定 | **不能直接登记完整 wire contract**。缺 management payload/result schema 与 session/proposal/audit binding |
| `gateway.configure` | REQ-MGMT-FALLBACK-001 只要求“配置 gateway” | 仅 `MGMT-FALLBACK-008` 字符串和通道分类实现 | 只有目标类别和 reachability 期望 | **需要新增业务语义和 wire surface**：无 target authority、配置参数、consumer、readback/verifier 或专属安全失败映射 |
| `diagnostics.configure` | REQ-MGMT-FALLBACK-001 只要求“配置 diagnostics” | 仅 `MGMT-FALLBACK-008` 字符串和通道分类实现 | 只有目标类别和 reachability 期望 | **需要新增业务语义和 wire surface**：无 target authority、配置参数、consumer、readback/verifier 或专属安全失败映射 |
| `system.configure` | Core REQ-MGMT-IDEM-001 只列举通用 `configure`；RFC-0001 固定 proposal/gate 流程 | `MGMT-CONFIG-001` 与 `SHELL-CHANNEL-ISOLATION-003` 固定字符串；schema `action` 仍开放 | vector 场景字符串；`system` 目标语义未定义 | **需要新增业务语义和 wire surface**：不存在 system configuration authority、payload/target/readback profile 或 consumer |

### 3.2 operation-set 结论

`MGMT-FALLBACK-008` 使七个 fallback 名称成为该 conformance 场景不可任意改写的机器字符串；`MGMT-CONFIG-001` 与 shell channel vector 同样固定 `system.configure` 的场景拼写。它们没有同时定义：

- 这是完整、封闭还是可扩展的 Management operation set；
- operation-set digest 与协商规则（AKP REQ-AKP-CONF-001 要求声明 implemented 时固定）；
- operation 到 request payload schema/result schema 的映射；
- operation 到 target authority、risk、readback/verifier 与 error mapping 的映射；
- operation 名本身是否扩大 session scope 或 authorization capability。

所以“把 8 个字符串登记成固定 operation set”虽不必新增 REQ 域，却会新增 normative wire surface，并会把 reachability vector 扩张解释成完整业务/权限合同。该解释不满足“不扩大权限或范围”的纠错型条件。

## 4. B — Configuration payload 与 target profile

### 4.1 必需事实追踪矩阵

| 必需事实 | normative 来源 | 现有 machine 落点 | 已登记错误映射 | 唯一可导出？ | 缺口/裁决 |
|---|---|---|---|---|---|
| target authority ref | Core §3.2 `authority_ref`、§4/§5.1 单一写 authority、REQ-AUTH-001 | proposal `target_refs`；Intent `target`；GovernedObjectHeader 可含 authority | `STATE_CONFLICT`、`STATE_STORE_UNAVAILABLE` | **否** | URI target 不等于 authority；三个 configure target 没有已登记对象/authority mapping |
| parameters schema/digest | Core OperationDescriptor/REQ-OP-001、Intent §9.2；REQ-MGMT-EFFECT-001 | proposal `parameters` 为开放 object、`parameters_digest`；Intent `parameters_digest`；AKP envelope `schema_digest`/`payload_digest` | `SCHEMA_MISMATCH`、`PROTOCOL_SCHEMA_DIGEST_MISMATCH`、`DIGEST_MISMATCH` | digest **是**；schema **否** | 无 per-operation parameters schema、digest domain 或 schema pin mapping |
| expected version | Core REQ-STATE-003、REQ-MGMT-EFFECT-001 | proposal `expected_versions`；Intent `expected_state_version`；AKP request `expected_versions` | `STATE_CONFLICT` | **是，前提是 target domain 已固定** | target authority/domain 未固定使 version 的比较对象仍不唯一 |
| idempotency binding | Core REQ-EFF-002、REQ-MGMT-IDEM-001 | proposal/Intent/AKP request `idempotency_key`；Effect 同时固定 key + parameters digest | `EFFECT_IDEMPOTENCY_CONFLICT` | **是** | 可复用；不补足 target/payload profile |
| writer epoch | Core §5.4、REQ-MGMT-RECOVERY-001 | Effect 可选 `fencing_token`；kernel port 可选 `fencing_epoch`；recovery vector 有 `original_writer_epoch` | 当前实现映射 `STATE_CONFLICT` | **否** | 不在 proposal/AKP payload 中强制，且 configuration authority 的 epoch source/receipt 未固定 |
| proposal/session/approval digests | RFC-0001 §7.5/§7.6、REQ-MGMT-EFFECT-001 | proposal `proposal_digest`、session `session_digest`、decision `decision_digest`；AKP 只固定 proposal/approval refs | `DIGEST_MISMATCH` 可覆盖 digest mismatch；签名失败见 §5 | **部分** | 三个对象各有 digest，但没有一个 request profile 强制同时绑定 session/proposal/approval digests；ref 不能替代 digest contract |
| Intent/Effect/Verification refs | Core §9、REQ-MGMT-EFFECT-001/RECOVERY-001 | Effect→Intent strong ref；Effect verification `report_ref`；VerificationReport→subject/fixed post-state | `EFFECT_OUTCOME_UNKNOWN`、`EFFECT_RECOVERY_QUARANTINED`；验证文本类别并非全已登记 | **流程关系部分可导出** | proposal/AKP/config receipt 未强制闭合三者；缺 configuration target-specific binding |
| readback/postcondition verifier | Core REQ-EFF-003、Intent postconditions、VerificationReport | Intent `postconditions[].verifier_ref` 可选；VerificationReport 固定 verifier/version/post-state | `STATE_STALE_OBSERVATION`；无完整 target-specific verifier error closure | **否** | gateway/diagnostics/system 无 read API、postcondition、verifier identity/version 或 evidence profile |
| authority version receipt | Core §5.3/§12 `CompareAndSet`/`CommitEffect` | kernel `CommitReceipt` 仅 event sequence；AKP result 可选 `observed_versions`/result ref | `STATE_CONFLICT`、`STATE_STORE_UNAVAILABLE` | **否** | 无 configuration receipt schema 强制 target/new version/epoch/Effect/Event refs |
| audit sequence/high-watermark | Core StateSnapshot/high-watermark、AppendEvent；REQ-AUDIT-002 | SQLite event sequence；AKP result 仅可选 `audit_ref`；Event schema无 sequence；transition record无 audit watermark | persistence failure 可用 `STATE_STORE_UNAVAILABLE` | **否** | 内部序列未成为 audit carrier contract；无 audit receipt/high-watermark 字段或 gap rule |

### 4.2 对象族、状态域与复用裁决

- **新的 configuration 对象族**：现有 machine assets 没有 system/gateway/diagnostics configuration 对象或 target profile。若不先指定某个既有 governed object 为 authority target，就必须新增对象形合同或 operation-specific profile；两者均超出修正型 docs/schema 漏登。
- **第六 lifecycle domain**：不需要也不允许新增第六执行 lifecycle。`state-and-transition-contract.md` 明确五个 execution lifecycle 正交，同时一般 authority-managed state domain 是开放集。但为 configuration 新登记状态域/表仍是新的 normative semantic surface，不能借“开放字符串”自动推导。
- **可复用对象**：`ManagementActionProposal`、Intent、Effect、VerificationReport、Event 可以复用为 proposal→effect→verify→commit 的流程骨架；它们不能替代 configuration target state、operation-specific payload/readback profile 或 authoritative audit record。
- **gateway/diagnostics consumer**：全仓只有 fallback vector、通道 operation 分类和交接文档提及；没有 backend adapter、authority store target、readback API、verifier 或 behavior evidence。二者不是当前可定义为真实 consumer 的既有目标语义。

## 5. C — Signature profile

### 5.1 canonical §12 对照

| signature contract 必需项 | Session schema | Approval schema | 现有来源 | 裁决 |
|---|---|---|---|---|
| signature algorithm | 无 | 无 | canonical §12 只要求具体 contract 识别 algorithm | **缺失** |
| key ID / key resolution | 无 | 无 | 无 key/signature schema；实现只有 canonical preimage helper | **缺失** |
| signature domain | 无 | 无 | 未登记 session/approval signature domain | **缺失** |
| signed schema digest | 无 | 无 | 对象 schema 有 bundle digest，但 signature 字段不绑定它 | **缺失** |
| signed projection | 无 | 无 | `session_digest` / `decision_digest` 不是 signed projection 定义 | **缺失** |
| digest/signature excluded paths | 无 | 无 | canonical §10/§12 禁止按惯例猜测排除项 | **缺失** |
| signature encoding | `string(minLength=16)` | `string(minLength=16)` | 无 base64/base64url/DER/raw 或长度 profile | **缺失** |
| trust-root binding | `session_authority` 仅 URI | `deciding_authority` 仅 URI | RFC-0001 REQ-MGMT-AUTHZ-001 只给 trust-root 语义；无 key resolution/rotation/revocation binding | **缺失** |
| invalid signature 登记错误 | 无 | 无 | `MANAGEMENT_SELF_AUTHORIZATION_DENIED` 仅覆盖自签/自授权；`DIGEST_MISMATCH` 只覆盖 digest mismatch | **缺失** |

`MGMT-CONFIG-001` 的 `authority_signature_valid: true` 是 behavior input 预设，不定义验证算法；`MGMT-APPROVAL-005` 的 `independent_signed_decision_required` 是期望性质，不定义签名 profile。现有实现的字符串长度校验、fixture 和 schema-valid 均不构成密码学验证。

把 `authority_signature: string` 改成结构体，或在 session/approval 顶层增加 algorithm/key/domain/schema/projection/encoding 字段，会改变两份已登记 wire schema、生成绑定、digest projection、兼容性和失败行为。仓库也没有既有 detached-signature schema 可 `$ref` 复用。故该工作是新的 normative/wire surface，不是无语义变化的纠错型收敛。

## 6. D — Authoritative audit carrier

### 6.1 必需事实到现有合同

| audit 必需事实 | normative 来源 | 当前 machine/实现事实 | 完整？ |
|---|---|---|---|
| actor chain；initiating/effective/workload/device principal；purpose | RFC-0001 §14；REQ-MGMT-TRUST-001；event/audit standard §4 | Event header 有通用 provenance/governance字段但无 audit actor-chain profile；transition record 仅 `actor_ref`；denial DTO 为局部实现 | **否** |
| proposal/approval/session refs | RFC-0001 §7.6；AKP §10.1 | AKP request 可带 refs；Event/transition/audit carrier 未强制 | **否** |
| decision/stage/reason/error | REQ-AUDIT-001/002；error-contract；event/audit standard §4 | transition record 有 reason；Event payload 开放；无统一 audit schema 强制 registered error/stage/decision | **否** |
| target/version | Core §5.3/§14 | transition record有 subject/before/after version；Event storage row有 object/version；非全部 management stages | **部分** |
| Effect state/result refs | Core REQ-AUDIT-001；RFC-0001 §14 | Effect 有 event refs/result-related refs；无逐阶段 audit record binding | **部分** |
| sequence/high-watermark | Core REQ-AUDIT-002、event log ordering | SQLite events/outbox/records 有内部 sequence；Event schema与 `audit_ref` 不暴露 audit sequence/high-watermark | **否** |
| tamper resistance / append-only | REQ-AUDIT-002；event/audit standard §2 | SQLite event/transition triggers阻止 update/delete；无 audit carrier integrity/signature/hash-chain contract | append-only **部分**；tamper resistance **否** |
| retention / sensitivity control | REQ-AUDIT-002；RFC-0001 §14 | GovernedObjectHeader 可表达 retention/sensitivity，但 Event audit profile未绑定；SQLite row无相应列/策略合同 | **否** |
| export audit | REQ-AUDIT-002；RFC-0001 §14 要求正文导出独立授权 | D-018 允许 Event envelope作为审计导出形态，但无 audit payload schema、export authorization/result contract | **否** |
| 与 state/Effect commit 原子性 | Core §5.3、REQ-REC-003；event/audit standard §2；state-transition standard | SQLite transition commit 原子包含 state/event/transition/outbox；没有 authoritative audit record/batch 端口 | state/Event **是**；audit **否** |

### 6.2 carrier 合法性裁决

| 候选 carrier | 可复用事实 | 不能承担完整 authoritative audit 的原因 | 裁决 |
|---|---|---|---|
| `Event` | 已登记 envelope；治理 header；不可变 payload/ref；标准要求 state transition 与 Event 同事务 | `payload` 开放且无 audit payload profile；schema 无 sequence/high-watermark、逐管理阶段字段或 export contract；D-018 明确内部事件行不是完整 Event envelope | **当前不合法**。可作为未来 audit record 的外层 envelope，但必须新增 audit payload/profile 与原子绑定 |
| state transition record | 固定 actor/authority/reason/causation/evidence/before-after version，可 append-only | 只覆盖已提交 lifecycle transition；缺 principals/purpose/session/proposal/approval/decision-stage-error/retention/sensitivity/export；开放 `metadata` 不能补合同 | **当前不合法**。只能是 audit chain 的一个证据节点 |
| outbox | 与 commit 同事务保存 event pointer；有 delivery sequence | 只有 event ID、destination、dispatch bookkeeping；destination 不是 audit；派发可重试且不代表持久审计 | **不合法** |
| 内部 SQLite event/transition row | 原子事务、内部序列、append-only triggers | 实现私有形状不是 normative carrier；event `canonical_json` 当前是内部事件值；无完整 audit 字段、retention/sensitivity/export/tamper profile | **不合法**。未来可作为 carrier 的 storage adapter，但不能反向定义合同 |
| AKP result `audit_ref` | 可引用某个 audit 资源 | 可选 URI 不定义被引用对象、sequence、完整性、retention、export 或 atomic commit | **不合法** |

telemetry、日志、trace、`audited=true` 与开放 payload 均不在候选范围内。`docs/traceability/matrix.yaml` 对 REQ-AUDIT-001/002 的 `impl`、`impl_tests`、`evidence` 仍为空；静态 `SPEC-CONTRACT-COVERAGE-001` 只证明要求存在，不证明 audit behavior。

## 7. GO 条件逐项结果

| GO 条件 | 结果 | 说明 |
|---|---|---|
| 不新增对象族、REQ 域、第六状态机或未登记权限语义 | **fail** | 可不新增 REQ 域/第六 lifecycle，但 operation/target/signature/audit 至少需要新的对象形 profile或 wire/权限语义 |
| 所有新增机器字段都有既有 normative 来源 | **fail** | algorithm、key resolution、signature domains/projections、target profiles、audit sequence/export/atomic carrier 均不能唯一导出 |
| 所有失败路径都有已登记错误映射 | **fail** | 一般 invalid signature/key/trust-root failure 无精确 registered code；若新增 target/profile，也无闭合 per-operation error mapping |
| operation、payload、signature、audit 可形成机器可验证合同 | **fail** | 四项均至少一处缺机器闭包 |
| 不需要修改 vector expected | **pass（保持）** | 本裁决不改 expected；未来应新增负例而非放宽既有 expected |
| 不扩大 Profile 或 v0.1 release claim | **pass（保持）** | Profile implemented 仍为 0；v0.1 claims 不变 |
| 可证明属于修正型收敛 | **fail** | 需要语义型/结构型 normative surface；按 docs-sync-contract 不确定时按更重分类 |

任一 fail 即 NO-GO；本批有多项 fail。

## 8. 最小解除条件与建议版本边界

建议版本边界：**v0.2 normative surface expansion**，不要在 v0.1 冻结期用“修正型”名义实施。解除 D-016/D-022 前，独立 Lane-CTR 规范批至少必须完成：

1. 明确 Management operation set 是封闭集合还是版本化可扩展集合，登记 operation-set digest，并逐项绑定 request/result payload schema、channel、risk/permission 与 error mapping；不得从 vector reachability 自动扩大授权。
2. 为 `system/gateway/diagnostics.configure` 指定已登记 authority target/object profile、parameters schema/domain、expected-version/CAS 域、writer epoch、真实 consumer、readback/postcondition verifier 与 authority receipt。若新增对象族或 state domain，走结构型变更/ADR/迁移流程；不新增第六 execution lifecycle。
3. 登记可复用的 signature profile（或等价明确合同），固定 algorithm、key ID/resolution、domain、signed schema/projection/exclusions、encoding、trust-root/rotation/revocation，并登记一般 invalid signature/key resolution 错误。
4. 登记 authoritative audit carrier/profile 与 persistence port，覆盖本裁决 §6 的全部字段、sequence/high-watermark、tamper resistance、append-only、retention、sensitivity、authorized export，以及与 state/Effect/Event commit 的原子失败语义。
5. 添加缺字段、错 domain/schema、未知/撤销 key、坏签名、audit insert failure/rollback、sequence gap、retention/sensitivity/export denial 等红测；保留现有 vector expected，不把 not-run 改成 pass，直到 runner 对真实实现执行并留证据。
6. 按 docs-sync-contract 将 companion、registry/errors、schema/generated bindings、vectors、matrix、台账、ADR/迁移说明与受影响产品文档同批联动；独立评审后才允许重新做 CA-0/进入 CA-1。

在这些解除条件满足前，唯一安全入口仍是 Lane-CTR 的 v0.2 规范设计与评审；不得启动 KRN、RUN 或 CFR configuration authority 实现。

## 9. 本批验证与状态边界

本批仅修改 living docs 与 handoff。要求执行：

- `pnpm run check:consistency`
- `node tools/src/gen-matrix.mjs --check`
- `git diff --check`
- `pnpm -r build`
- `pnpm -r test`

未修改 machine assets，因此不声称执行了新的行为向量，也不提升“规范已登记 / 实现已提供 / 测试已执行 / Profile 已符合”中的任何状态。
