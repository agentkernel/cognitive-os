# Configuration Authority Contract Sufficiency Decision

- 战役：`CONFIG-AUTHORITY-FOUNDATION-THEN-MGMT-CONFIG-CFR`
- 工作包：CA-0（Lane-CTR）
- 基线：2026-07-22，`origin/main@dfb3091`
- 裁决：**NO-GO**
- 状态纪律：`84 total / 59 pass / 25 not-run / self-check 40 / Profile implemented 0`

## 1. 结论

现有合同足以约束 configuration authority 的一部分内部语义：proposal 固定目标、参数摘要、expected versions 与幂等键；Core/standards 固定 CAS、fencing、Intent/Effect、reconcile、Verification、commit 与 fail-closed 错误语义。它们不足以在不扩大或修正规范表面的情况下完成本战役，原因有三项：

1. 管理 wire operation set 仍未登记（D-016）；`system.configure`、`gateway.configure`、`diagnostics.configure` 只有 prose/vector/受保护字符串，没有固定的 payload schema、target profile、真实 consumer 或 readback/postcondition 合同。
2. `PrivilegedManagementSession.authority_signature` 只有字符串形状。现有合同没有满足 `canonical-encoding-and-digest.md` §12 的 signature algorithm、key ID、signature domain、signed schema/projection 与 excluded paths，也没有完整的无效 authority signature 错误映射；不能把长度校验或任意 JSON 解析当成 issuance authority。
3. REQ-AUDIT-001/002 与 `event-audit-watch.md` §4 只给出 audit 性质和最低字段；仓库没有可机器固定顺序、完整性、保留、敏感度、导出及与 authority commit 原子关联的 AuditRecord carrier/port。Event 的开放 `payload` 与 outbox destination 不能自行补成该合同。

因此 CA-1 的端口红测没有合法的冻结输入，CA-2～CA-8 全部停止。8 个 Management not-run 向量保持不变；禁止通过实现私有 JSON、通用 HTTP `ok` 或修改 vector expected 绕过。

## 2. 入口与失败追踪检查

| 必需事实 | 既有 normative / machine 落点 | 已登记错误映射 | 检查 | 裁决说明 |
|---|---|---|---|---|
| proposal 固定 action/target/parameters/expected version/idempotency | RFC-0001 §7.6；`management-action-proposal.schema.json` | `STATE_CONFLICT`、`EFFECT_IDEMPOTENCY_CONFLICT`、`DIGEST_MISMATCH` | pass | 可作为内部 authority request 的输入引用，不是 configuration state record |
| configuration authority target identity/version | Core §5.3、§12.1；proposal `target_refs`/`expected_versions` | `STATE_CONFLICT`、`STATE_STORE_UNAVAILABLE` | **fail** | 无 configuration record/target profile；不能从任意 URI 或 JSON 推导 authority domain |
| target-specific parameters 与 readback/postcondition | Core §9、§12.1；Effect/Verification schema | `SCHEMA_MISMATCH`、`STATE_STALE_OBSERVATION` | **fail** | 三个 configure 名称无固定 payload schema、consumer、readback 或 verifier binding |
| session 字段、scope、expiry、版本 | RFC-0001 §7.5；`privileged-management-session.schema.json` | `MANAGEMENT_SESSION_EXPIRED`、`MANAGEMENT_SESSION_REVOKED`、`MANAGEMENT_SCOPE_MISMATCH`、`MANAGEMENT_STEP_UP_REQUIRED` | pass | 仅证明 schema shape 与 gate 输入存在 |
| session issuance 与 authority signature verification | RFC-0001 REQ-MGMT-SESSION-001/003、REQ-MGMT-AUTHZ-001；canonical standard §12 | self-sign 可用 `MANAGEMENT_SELF_AUTHORIZATION_DENIED`；一般无效 authority signature 无精确已登记映射 | **fail** | signature profile、key resolution、signed projection 均未登记；现有 `from_json_value`/内存 archive 不构成 authority |
| CAS 与唯一 winner | Core REQ-STATE-003、REQ-MGMT-EFFECT-001；state standard §3 | `STATE_CONFLICT` | pass | 可复用既有确定性语义 |
| idempotency binding | Core REQ-EFF-002、REQ-MGMT-IDEM-001；intent/effect standard §3 | `EFFECT_IDEMPOTENCY_CONFLICT` | pass | 同 key 同参等价、异参拒绝已有合同 |
| writer fencing | Core §5.4、REQ-MGMT-RECOVERY-001；现有 store sink 语义 | `STATE_CONFLICT` | pass | 可复用当前 epoch 的事务内校验；不新造 `FENCING_REJECTED` |
| Effect/reconcile/Verification/final commit | Core §9、§12.1；Effect schema/transition table；intent/effect standard | `EFFECT_OUTCOME_UNKNOWN`、`EFFECT_RECOVERY_QUARANTINED`、`STATE_STORE_UNAVAILABLE` | pass | receipt/HTTP/model completion 均不能替代 authority commit |
| per-operation authoritative audit | RFC-0001 REQ-MGMT-TRUST-001；Core REQ-AUDIT-001/002；event/audit standard §4 | audit persistence failure 可映射 `STATE_STORE_UNAVAILABLE` | **fail** | 有性质要求，无足以冻结 CA-1 atomic port 的机器 carrier/顺序/完整性合同 |
| Management API/CLI operation set | AKP §10.1；Core REQ-MGMT-FALLBACK-001；D-016 | envelope 错误合同已存在 | **fail** | AKP 明确不定义业务 operation；登记操作名集会扩大当前冻结面 |
| gateway/diagnostics real consumer | Core REQ-MGMT-FALLBACK-001；`MGMT-FALLBACK-008` | 无专属新码获准 | **fail** | 当前仅 `channel_binding.rs` 字符串分类，无 backend/readback；四 fallback verbs 不含 configure |

故障敏感性检查：移除 expected version、parameters digest、idempotency key、writer epoch、Verification ref、audit 原子写入或 authority signature profile 中任一项，均必须保持/转为 NO-GO；不得降级为 warning。

## 3. 允许冻结的内部接口语义（不授权实现）

若 §4 的规范 gate 日后解除，CA-1 可定义实现内部端口，但不得将其暴露为新 wire object：

- request 必须同时携带唯一 target authority ref、expected version、proposal/session/approval digests、idempotency key 与 parameters digest、writer epoch、Intent/Effect/Verification refs，以及完整 audit batch；缺一即拒绝。
- 原子单元必须在同一 authority transaction 内完成：校验 current epoch → 幂等绑定 → configuration CAS → Effect/Verification 引用闭合 → append-only audit/event → commit。
- receipt 只能在事务 commit 后返回，并证明 CAS winner、new authority version、current epoch、committed Effect/Event refs 与 audit sequence/high-watermark。
- audit insert/commit 失败必须回滚 configuration/session 可见状态；若外部 apply 已发生，则 Effect 保持 `OUTCOME_UNKNOWN` 或进入 reconcile/quarantine，不能报告成功。
- session issue/renew/revoke 必须由已登记 signature profile 验证并持久化单调版本；任意 schema-valid JSON 不能直接获得 authority。

这些语义来自现有 CAS/Effect/audit prose，可供后续规范修正评审使用；本裁决不声称端口、store 或 target 已实现。

## 4. NO-GO 解除条件

必须由独立规范修正批完成并评审，不能在 KRN/RUN/CFR 实现批顺手补齐：

1. 处理 D-016：登记 Management operation set，并明确 `session.create_restricted`、`system/gateway/diagnostics.configure` 的通道与 envelope/payload 绑定。
2. 为每个 configure target 固定 target authority、参数 schema、version/CAS 域、实际 consumer、readback/postcondition verifier 与 safe error mapping；不得新增第六 lifecycle domain。
3. 为 PrivilegedManagementSession/approval 登记满足 canonical standard §12 的 signature profile、key resolution、signed projection/exclusions 和无效签名错误映射。
4. 登记或明确复用一个机器可验证的 authoritative audit carrier，覆盖顺序完整性、不可篡改、保留、敏感度、导出，以及与 state/Effect commit 的原子关联。
5. 证明上述修正属于既有 normative 语义的纠错型收敛；若属于新对象族、REQ 域、错误码、transition 或 wire surface，则保持 deferred-to-v0.2，并另案解除 IMP-01 freeze。

解除前，`MGMT-CONFIG-001`、`MGMT-FALLBACK-008` 以及其余 6 个 Management 向量保持 not-run，kernel-server 的通用 Management `ok` 仅是 reference stub，不得作为 authority 证据。

## 5. 实测与状态

- `origin/main@dfb3091` 的 GitHub CI：success（run `29853440295`）。
- WSL Linux guest 全量 runner：84 / 59 pass / 25 not-run；报告 sha256 `31d524d5c8a3bd194fac8eabb0c9c65c6887667298034057b1e552d5408e86f1`。
- WSL deliberately-wrong self-check：40/40 flipped，`corrupted_but_still_passing=[]`；报告 sha256 `29631a657af610ff31f59c1a9e820a317ab75623aaac646bbb83b29e93b4da7c`。
- matrix 非空 `impl` 条目按 `requirements[*].impl.length > 0` 重算为 70；`gen-matrix --check` 与 consistency 均通过，D-023 已闭合并将 PROGRESS 的旧 68 修正为 70。
- Windows-native runner 未执行：GNU linker 缺 `libgcc` / `libgcc_eh`，在构建阶段阻断；WSL 结果不构成 Windows-native 产品能力声明。
- 本批未修改 registry/schema/transition/vector/error/runner，未生成 Profile implemented 声明。
