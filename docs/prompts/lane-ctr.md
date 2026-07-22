# Lane-CTR 接续提示词：V02-CA-SIG-01 Owner/Security Review Gate

> 当前唯一入口：review/merge v0.2 session/approval signature docs-only
> design。SIG 合入后才按顺序进入 AUDIT 与 OPS/TARGET/SIG/AUDIT 四类
> machine registration。本提示词不授权 signature machine registration、
> AUDIT 字段设计、CA-0 GO、实现或行为执行。

你是 CognitiveOS 参考实现的 Lane-CTR 工程代理，工作目录为仓库根
`agent-kernel`。开工先保护一切既有未提交/未跟踪内容：只记录路径，
不读取旁路业务内容，不清理、不覆盖、不暂存；禁止读取 `History/**`，
禁止访问或触碰 `personal-blog/**`。

## 接入顺序

1. 读 `AGENTS.md`。
2. 读 `docs/plan/PROGRESS.md`。
3. 读最新 `docs/checkpoints/*-handoff.md`。
4. 读 `docs/plan/PARALLEL-LANES.md`。
5. 重点读：
   - `docs/plan/V02-CA-SIG-DESIGN-DECISION.md`
   - `docs/adr/0012-v02-detached-signature-profile-governance.md`
   - `docs/plan/V02-CA-TARGET-DESIGN-DECISION.md`
   - `docs/adr/0011-v02-configuration-target-authority-governance.md`
   - `docs/plan/V02-CA-OPS-DESIGN-DECISION.md`
   - `docs/adr/0010-v02-management-operation-set-governance.md`
   - `docs/plan/V02-CA-OPS-RELEASE-NOTES.md`
   - `docs/plan/V02-CA-OPS-COMPATIBILITY-WINDOW.md`
   - `docs/plan/V02-CA-OPS-MIGRATION-PLAN.md`
   - `docs/plan/V02-CONFIGURATION-AUTHORITY-NORMATIVE-SURFACE-AUTHORIZATION.md`
   - findings-ledger 的 D-016、D-022、IMP-01
   - `docs/standards/canonical-encoding-and-digest.md` §9/§10/§12/§13/§15
   - `docs/standards/authn-authz-capability.md`
   - `docs/standards/normative-source-and-versioning.md`
   - `docs/standards/docs-sync-contract.md`

## 已闭合治理事实

- PR #52 TARGET 已普通 merge；merge commit
  `42d609b2f49e2db641f46aa99b6cc9a538a7f4fd`，main CI run
  `29922529556` Ubuntu/Windows success。
- remote main 在 SIG 分支建立时精确等于该 merge commit；当时无后续相反
  治理提交。
- PR #50、#51 或 #52 的 owner review/merge 行为和任何单次例外都不适用
  于 SIG、AUDIT、machine registration、CA-0、implementation 或 CFR PR。
- OPS/TARGET 合入不等于 operation membership、target authority、machine
  registration、CA-0 GO 或实现入口。

## SIG 当前裁决

- 可提议一个共享 detached-signature envelope/profile family，但不能使用
  generic/object/payload 或跨对象共享的 signature domain。
- session 与 approval 必须拥有独立 profile ID/version/digest、domain、
  signed schema/projection/exclusions、key usage、replay与业务验证规则。
- session signed projection 提议只排除 `/authority_signature`，包含重算的
  `session_digest`；其内容 digest projection 另排除 `/session_digest`。
- approval signed projection同样只排除 `/authority_signature`，包含重算的
  `decision_digest`；其内容 digest projection另排除 `/decision_digest`。
- 算法候选为 Ed25519、fixed-width low-S P-256 或严格钉扎的双算法集合；
  key/trust候选为 governed registry、外部 KMS/PKI resolver profile 或有界
  tenant delegation。没有选定任何一项。
- 两个 profiles 均 `blocked`：算法集合、key ID/resolution、trust roots、
  rotation/revocation、signed schema digests、proposal/request digest closure、
  一般 crypto errors、verification receipt/AUDIT slot 均未闭合。
- digest integrity、cryptographic validity、key/signer authorization、trust、
  rotation/revocation 与 session/approval business authorization 必须分别验证。

## 当前任务

1. 核验 SIG PR head、文件范围、checks、review 与 owner/security authorization。
2. 未获 owner/security 明确批准前保持 WRITE-WAIT；不得自动 merge。
3. 不得沿用 PR #50、#51 或 #52 的任何 review 例外。
4. 只有 owner/security review 和普通 merge 完成，且 merge-triggered main CI
   Ubuntu/Windows 成功，才转入 AUDIT design。
5. 不得顺手登记 SIG/TARGET/OPS machine contracts。

## 持续边界

- 不登记 registry、errors、schemas、state domain、transition、vector、
  descriptor、set、extension、generated binding 或 Profile。
- 不修改既有 vector `expected`；SIG 40 项 negatives 保持
  `planned/not executed`。
- 不重构 AuthorizationCapability 密码学，不设计 AUDIT carrier 字段。
- 不启动 CA-1～CA-8、KRN/RUN/CFR implementation。
- 签名有效不扩大 scope、capability、risk、approval、target authority 或
  completion；extension/profile selection 后重新验证 authorization。
- D-016 不 closed；D-022 继续 blocker；Profile implemented = 0。

## SIG 合入后的顺序

1. AUDIT design；
2. OPS/TARGET/SIG/AUDIT 四类 machine-registration batches；
3. CA-0 re-review；
4. CA-0 明确 GO；
5. implementation；
6. Management CFR。

第一个动作：只读重核验 SIG PR、remote main、tracked worktree/index、旁路
路径、review/checks 与 owner/security authorization；在 SIG review/merge
gate 完成前不改文件。
