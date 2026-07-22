# v0.2 Configuration Authority Normative Surface Expansion Authorization

- 决议 ID：`V02-CA-GOV-00`
- 日期：2026-07-22
- 基线：`origin/main@251c69c9249a350f54853e13d632a37076b9b88d`（PR #49 merge）
- 来源：仓库所有者逐项确认的八项裁决
- 所有者授权状态：**approved**
- 工作状态：**v0.2 normative surface expansion = owner-approved / design pending**
- 分类：docs-only 治理授权；后续规范变化按 **v0.2 breaking Draft** 管理
- 承接：[Configuration Authority Contract Sufficiency Decision](CONFIGURATION-AUTHORITY-CONTRACT-DECISION.md) 与 [Spec-Correction Feasibility Decision](CONFIGURATION-AUTHORITY-SPEC-CORRECTION-DECISION.md)
- ADR：[ADR-0009](../adr/0009-v02-configuration-authority-surface-expansion-governance.md)

## 1. 授权结论与边界

仓库所有者批准将下列四类工作纳入 **v0.2 Configuration Authority normative surface expansion**：

1. Management operation set 与 operation-set digest；
2. `system.configure`、`gateway.configure`、`diagnostics.configure` 的 target authority、payload、consumer、readback/verifier 与 receipt contract；
3. session/approval signature profile；
4. authoritative audit carrier/profile/persistence port。

该授权解除的是“这些主题只能作为 v0.1 修正型收敛”的版本边界限制。它**不**表示任何具体字段、operation 成员、算法、对象族、错误码、schema、vector、transition、生成绑定或实现已经获批或登记。四类机器合同冻结并全部合入后，仍须独立执行 CA-0 re-review；只有 CA-0 明确 GO，CA-1～CA-8、KRN/RUN 实现与 Management CFR 才可启动。

本决议不改写两个既有 NO-GO 裁决的历史理由。v0.1 修正型路径仍为 NO-GO；本决议以新的 v0.2 版本边界承接其解除条件。

## 2. v0.2 breaking Draft 与版本义务

本扩张按 `normative-source-and-versioning.md` §5 的 **v0.2 breaking Draft** 管理。后续规范交付必须同时满足：

- 创建新的 specification set、完整 SemVer 和 digest；
- 发布明确的 release notes；
- 声明有限、封闭的 compatibility window；
- 提供 migration plan，并标明旧资产、旧引用、旧 operation 与旧 negotiation epoch 的处置；
- 将 v0.2 set、schema bundle、requirement set、operation set 与适用 suite 的身份按标准钉扎；
- 禁止把变化注入既有 v0.1 negotiation epoch；v0.2 必须建立新的 negotiation epoch，并重新验证 authorization、continuation 与 critical extensions；
- 已发布、已协商、已签名、已引用证据或 digest-pinned 的资产不得原地改字节，不得复用旧 SemVer 或 digest 身份。

同一资产 ID/SemVer 下字节或 digest 不一致，以及使用 `latest`、可变分支或未钉扎位置代替 specification set 身份，均为 NO-GO。

## 3. 允许和禁止的 surface

### 3.1 后续独立设计与评审允许提议

- 新增或调整 object/wire contract；
- 在既有 REQ 域内新增 REQ；
- 新增精确错误码；
- 新增一般 authority-managed state domain；
- 为本决议四条设计线增加必要的安全负例和机器绑定。

每项仅是“允许进入设计与评审”，不是预先批准。具体变化须在所属独立 PR 中按 docs-sync-contract 分类、审计、迁移和评审。

### 3.2 继续禁止

- 新增 REQ 域；
- 新增或直接提升 conformance/release Profile；
- 新增第六 execution lifecycle，或把一般 authority-managed state domain 解释成 execution lifecycle；
- 修改既有 vector `expected`、放宽安全负例或用实现反推规范；
- 在 runner 对真实实现执行并保留证据前改变相关向量的 `not-run` 状态；
- 在 CA-0 re-review GO 前启动 CA-1～CA-8 或 KRN/RUN/CFR 实现。

## 4. Management operation-set 授权模型

采用：**v0.2 封闭核心集合 + digest-pinned、显式协商的版本化扩展集合**。

后续 `V02-CA-OPS-01` 必须逐项裁决核心集合成员；本批不预先登记任何成员。OPS 设计必须保证：

- 未知或未协商 operation fail closed；
- operation 名称不能自动扩大 session scope、capability 或 authorization；
- reachability vector 不能自动解释为完整业务、wire 或权限合同；
- 每个 operation 显式绑定 request/result、channel、risk/permission、target/readback 和 error mapping；
- 核心和扩展集合均有确定性 digest、版本与 negotiation binding；扩展只有在显式协商并通过 criticality/authorization 检查后可用。

## 5. Configuration target-authority 原则

- 优先复用既有 governed-object authority 模型。
- 只有逐字段设计证明既有模型不能闭合 target identity/version、CAS、writer epoch、payload domain、consumer、readback/verifier 与 receipt 时，才允许通过独立结构型 ADR 提议新的 configuration 对象族或一般 authority-managed state domain。
- 新对象族或 state domain 必须有迁移说明、兼容边界和独立评审；不得新增 execution lifecycle。
- URI、开放 JSON、实现私有 row 或调用方注入值均不得直接充当 authority。

TARGET 设计必须承接 OPS 对 `system.configure`、`gateway.configure`、`diagnostics.configure` 的 operation 裁决，不得先行预定义对象族或字段。

## 6. Signature 设计边界

SIG 独立设计可以提议可复用 detached-signature profile，但必须完整裁决并机器钉扎：

- v0.2 允许算法集合；
- key ID 与确定性 key resolution；
- 独立 signature domain；
- signed schema/projection/excluded paths；
- 固定 encoding；
- trust root、rotation 与 revocation；
- unknown key、revoked key、坏签名等精确错误映射。

验签或 canonicalization 失败必须在 authorization、Effect、dispatch、状态变更和 commit 前 fail closed。具体算法、密钥体系和允许算法集合留给 SIG PR；本授权不预选、不登记。

## 7. Authoritative audit 设计边界

- `Event` 可以作为跨边界外层 envelope；必须新增专用、机器可验证的 audit record/payload profile 和 persistence port。
- Event 开放 `payload`、transition record、outbox、SQLite 私有行和 AKP `audit_ref` 均不能单独承担 authoritative audit carrier。
- audit 合同必须覆盖完整字段、sequence/high-watermark、tamper resistance、append-only、retention、sensitivity 与 authorized export。
- audit persistence 必须与 state/Effect/Event authority commit 具有闭合的原子失败语义；audit 写入或 commit 失败不得报告成功。
- 若外部 apply 已发生，只能进入 `OUTCOME_UNKNOWN`、reconcile 或 quarantine 路径，不得伪造回滚或成功。

具体 carrier 字段、完整性机制和 persistence 结构留给 AUDIT PR；本授权不预定义。

## 8. 交付拆分与执行门禁

后续按下列治理顺序交付：

1. `V02-CA-OPS-01`：operation-set 规范设计与评审；
2. TARGET：承接 OPS 的 configure operation 裁决，完成 target authority/consumer/readback/receipt 设计；
3. SIG：session/approval signature profile 独立设计；
4. AUDIT：authoritative audit carrier/profile/persistence port 独立设计；
5. 分别登记四类机器合同及生成绑定；每批独立审计并合入；
6. 四类机器合同全部合入后，独立执行 CA-0 re-review；
7. 只有 CA-0 明确 GO，才可启动 CA-1～CA-8、KRN/RUN 实现和 Management CFR。

SIG 与 AUDIT 可在本 GOV 批合入后并行设计。TARGET 必须等待并承接 OPS 对 configure operation 的裁决。任何设计批发现需要超出 §3.1 的 surface，均须停止并取得新的所有者裁决。

## 9. 先失败 GO/NO-GO checklist

### 9.1 本 GOV 批 GO 条件

- [x] 所有者八项授权范围可核实，状态为 `approved`。
- [x] 版本分类为 v0.2 breaking Draft，不是 v0.1 PATCH 或修正型变更。
- [x] 新 specification set/digest、release notes、compatibility window、migration plan 和新 negotiation epoch 义务明确。
- [x] allowed/forbidden surface、四条设计线、CA-0 re-review 与 CA-1～CA-8 blocker 明确。
- [x] 本批只修改允许的治理文档，不登记机器合同、不改实现或证据状态。

### 9.2 任一命中即 NO-GO

- 无法核实所有者授权范围，或把技术选项写成已经批准的机器合同；
- 把变化描述为 v0.1 PATCH/修正型，或缺少新 specification set、release notes、compatibility window、migration plan；
- 允许进入既有 v0.1 negotiation epoch，或原地改写/复用既有 SemVer/digest 身份；
- 允许未知/未协商 operation 继续，或允许 operation 名称自动扩大 scope、capability、authorization；
- 允许 reachability vector 替代逐 operation 的业务、wire、权限与错误合同；
- 允许 URI、开放 JSON、实现私有 row 或调用方注入值替代 target authority；
- 允许 Event 开放 payload、transition record、outbox、SQLite 私有行或 `audit_ref` 替代 authoritative audit contract；
- 允许新增第六 lifecycle，或把一般 authority-managed state domain 当作新的 execution lifecycle；
- 允许机器合同冻结和 CA-0 re-review GO 前启动实现；
- 自动改变 vector 状态、runner pins、Profile 或 v0.1 release claim；
- 修改既有 vector `expected` 或删除/放宽安全负例。

安全出口：上述任一项命中时，停止对应批次，不提交机器或实现变化；保持 D-022 blocker、CA-1～CA-8 blocked、相关向量 not-run，并将新增冲突登记到 findings-ledger 后请求独立所有者裁决。

## 10. 当前状态与非声明

本授权落档后，下列事实保持不变：

- v0.1 release claim 不变；
- 273 REQ / 55 errors / 61 schemas / 84 vectors；
- 59 pass / 25 not-run；self-check 40；matrix 非空 impl 70；
- Profile implemented = 0；
- D-016 = v0.2 authorized / design pending，不得写 closed；
- D-022 = blocker，继续阻断 CA-1～CA-8，不得写 closed；
- 四类 machine contracts **尚未登记**；
- Configuration Authority 实现 **尚未提供**；
- 新行为测试 **未执行**。

本授权仅证明版本与治理入口已获所有者批准，不等于“规范已登记”，也不等于实现、测试或 Profile 符合。
