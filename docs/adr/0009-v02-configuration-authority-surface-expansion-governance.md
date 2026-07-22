# ADR-0009: v0.2 Configuration Authority Surface Expansion Governance

- Status: Accepted（owner-approved；design pending）
- Date: 2026-07-22
- Decision owners: 仓库所有者 + CognitiveOS 参考实现维护者
- Classification: 版本边界与规范治理流程决策；不是字段级机器合同
- Baseline: `origin/main@251c69c9249a350f54853e13d632a37076b9b88d`

## Context

CA-0 的 [Configuration Authority Contract Sufficiency Decision](../plan/CONFIGURATION-AUTHORITY-CONTRACT-DECISION.md) 与后续 [Spec-Correction Feasibility Decision](../plan/CONFIGURATION-AUTHORITY-SPEC-CORRECTION-DECISION.md) 已分别裁决 NO-GO：Management operation set/configure target、session/approval signature 和 authoritative audit carrier 存在新的 normative/wire surface，不能全部解释成 IMP-01 允许的 v0.1 修正型收敛。

该历史理由保持不变。以开放字符串、vector reachability、URI、Event 开放 payload、outbox、SQLite 私有行、`audit_ref` 或字符串长度校验补合同，会扩大既有规范解释或把实现私有形状当 authority。继续坚持 v0.1 PATCH 路径会违反冻结纪律；直接启动实现则没有合法冻结输入。

仓库所有者已逐项批准 [V02-CA-GOV-00](../plan/V02-CONFIGURATION-AUTHORITY-NORMATIVE-SURFACE-AUTHORIZATION.md) 所列八项治理裁决，需要为其建立独立、可审计的版本边界和交付门禁。

## Decision

1. Configuration Authority 所需扩张按 **v0.2 breaking Draft** 管理，不作为 v0.1 PATCH 或修正型变更。
2. v0.2 创建新的 specification set 和 digest，发布 release notes、有限 compatibility window 与 migration plan。已发布或 digest-pinned 的旧资产不得原地改字节或复用旧 SemVer/digest 身份。
3. v0.2 变化不得进入既有 v0.1 negotiation epoch；必须建立新 epoch，钉扎新的 specification/schema/requirement/operation set 身份并重新验证 authorization 与 continuation。
4. 后续独立设计 PR 可提议新增/调整 object/wire contract、在既有 REQ 域内新增 REQ、新增精确错误码或一般 authority-managed state domain。继续禁止新增 REQ 域、直接新增/提升 Profile、第六 execution lifecycle，或把一般 state domain 解释成 execution lifecycle。
5. Management operation set 采用“v0.2 封闭核心集合 + digest-pinned、显式协商的版本化扩展集合”。未知或未协商 operation fail closed；operation 名称不扩大 scope/capability/authorization；每项必须绑定 request/result、channel、risk/permission、target/readback 和 error mapping。
6. Configuration target authority 优先复用 governed-object authority。只有逐字段设计证明不能闭合时，才可通过独立结构型 ADR 提议新对象族/state domain，并提供迁移、兼容边界与独立评审。URI、开放 JSON、实现私有 row 或调用方值不是 authority。
7. Signature 设计采用可复用 detached-signature profile 的方向，但算法、允许集合和密钥体系由 SIG PR 决定。最终合同必须钉扎 algorithm set、key resolution、domain、signed projection/exclusions、encoding、trust root/rotation/revocation 与精确失败映射；验签失败在任何副作用前 fail closed。
8. `Event` 可作 audit 跨边界 envelope，但不能由开放 payload 自行升级为 authoritative audit。AUDIT PR 必须登记专用 record/payload profile 和 persistence port，覆盖完整性、顺序、保留、敏感度、导出与 authority commit 原子失败语义。
9. 交付拆分为 OPS、TARGET、SIG、AUDIT 四个独立规范设计批及其机器合同/生成绑定批。四类机器合同全部合入后独立执行 CA-0 re-review；只有 CA-0 明确 GO，才允许 CA-1～CA-8、KRN/RUN 与 Management CFR 启动。
10. 本 ADR 不登记 operation 成员、payload/schema 字段、configuration 对象族、signature 算法/密钥体系、audit record 字段或 persistence 结构，也不改变 vector、runner pin、Profile 或 v0.1 claim。

## Consequences

- v0.2 normative surface expansion 现在是 **owner-approved / design pending**；不是 specified/implemented/tested/conformant。
- D-016 可记录为 v0.2 authorized/design pending，但不得 closed；D-022 继续是 CA-1～CA-8 blocker。
- 新 specification set、digest、release notes、compatibility window、migration plan 和新 negotiation epoch 成为后续规范批的强制交付。
- TARGET 必须承接 OPS 对 configure operation 的裁决；SIG/AUDIT 可在 GOV 合入后并行设计。
- 四类机器合同冻结前，任何实现私有 DTO、row、JSON 或通用成功响应都不能形成 Configuration Authority 证据。
- 相关向量在真实 runner 执行前保持 not-run；Profile implemented 保持 0。

## Deferred technical decisions

- `V02-CA-OPS-01`：核心 operation 的准确成员、扩展 descriptor、digest 与 negotiation binding。
- TARGET PR：三个 configure target 的对象复用/必要新结构、payload、consumer、readback/verifier、receipt 与 error mapping。
- SIG PR：具体算法、允许算法集合、key/trust-root/rotation/revocation 体系和签名 wire profile。
- AUDIT PR：carrier 字段、完整性与 sequence 机制、retention/sensitivity/export 合同、persistence port 和原子故障路径。

## Alternatives considered

### 保持 v0.1 修正型路径

拒绝。两个既有 CA NO-GO 裁决已证明缺口包含新的业务、wire、authority 与失败语义；按 PATCH 或修正型落地会违反 IMP-01 和版本标准。

### 在既有 v0.1 negotiation epoch 中协商可选扩展

拒绝。authority、signature、audit 与 operation-set digest 的变化可能改变关键治理语义，不能在旧 epoch 内静默生效或依靠非关键扩展降级。

### 先实现私有合同，再反向登记

拒绝。会让 URI/开放 JSON/私有 row/开放 Event payload 成为事实 authority，并绕过 CA-0 与规范优先纪律。

## Rollback strategy

若任何后续设计不能满足版本义务、fail-closed、迁移/兼容、原子 audit 或 lifecycle 边界，则该设计批 NO-GO：不合入机器资产或实现，保留 v0.1 资产和 negotiation epoch 不变，D-022 与 CA-1～CA-8 blocker 继续有效。若已发布 v0.2 Draft 资产需要修正，发布新的 SemVer/digest 与 migration note；不得原地改写或回退旧身份。
