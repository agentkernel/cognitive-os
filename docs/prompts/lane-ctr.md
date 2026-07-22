# Lane-CTR 接续提示词：OPS Registration Eligibility NO-GO Review/Merge Gate

> 当前唯一入口：review/merge v0.2 OPS registration eligibility docs-only
> NO-GO audit。本提示词不授权 machine registration、operation membership、
> TARGET/SIG/AUDIT registration、CA-0 GO、实现、行为执行或 Profile claim。

你是 CognitiveOS 参考实现的 Lane-CTR 工程代理，工作目录为仓库根
`agent-kernel`。开工先保护一切既有未提交/未跟踪内容：只记录路径，
不读取旁路业务内容，不清理、不覆盖、不暂存；禁止读取 `History/**`，
禁止访问或触碰 `personal-blog/**`。

## 接入顺序

1. `AGENTS.md`
2. `docs/plan/PROGRESS.md`
3. 最新 `docs/checkpoints/*-handoff.md`
4. `docs/plan/PARALLEL-LANES.md`
5. 重点读取：
   - `docs/plan/V02-CA-OPS-REGISTRATION-ELIGIBILITY-AUDIT.md`
   - `docs/plan/V02-CA-OPS-DESIGN-DECISION.md`
   - `docs/adr/0010-v02-management-operation-set-governance.md`
   - OPS release/compatibility/migration companions
   - TARGET/SIG/AUDIT decisions and ADR-0011～0013
   - `docs/plan/V02-CONFIGURATION-AUTHORITY-NORMATIVE-SURFACE-AUTHORIZATION.md`
   - findings-ledger D-016、D-022、IMP-01
   - normative source/versioning、canonical digest、authn/authz、Intent/Effect、
     state/transition、event/audit/watch、conformance evidence 与 docs-sync

## 已闭合入口事实

- AUDIT PR #54 merged: head
  `82fec91b5853e360de9277d9937f39a688947702`, merge commit
  `54929f1ed8fef1e09ffbb5593633f5d94d5e281e`.
- Merge-triggered main CI run `29937238562` was a `push` at that merge; Ubuntu
  and Windows completed `success`.
- PR #54 changed exactly 11 docs-only paths. Reviews, reviewDecision, and
  requested reviewers were empty.
- The owner-authorized security/audit/compliance review after merge found no
  blocking AUDIT design defect. It is not an external human, third-party, or
  GitHub review.
- SIG independent security/cryptography review remains pending.
- OPS/TARGET/SIG/AUDIT machine contracts remain unregistered.

## OPS eligibility result

`V02-CA-OPS-REG-READINESS-01` rechecked all mandatory bindings and found:

- `session.create_restricted`: blocked by issuance wire/authority and
  unregistered SIG;
- `status.inspect`: blocked by selector/result/read authority/readback/errors;
- `capability.revoke`: blocked by target/version/receipt/risk/errors;
- `execution.stop`: blocked by management request/result/authority/error/audit
  closure; shell control is a separate contract;
- `effect.reconcile`: blocked by management wire and authoritative AUDIT closure;
- `gateway.configure`, `diagnostics.configure`, `system.configure`: blocked by
  TARGET authority/consumer/readback/receipt and SIG/AUDIT bindings.

No foundation asset is eligible without owner decisions on exact IDs, complete
SemVer/publication, digest domains/projections/exclusions, empty/unpublished set
semantics, specification/requirement/schema/operation/suite freeze order,
cross-family digest-cycle break, and exact error taxonomy.

## Current task

1. Reverify the docs-only audit PR head, file scope, reviews, reviewer requests,
   and Ubuntu/Windows CI.
2. Do not auto-merge.
3. Request owner/security/protocol review in the handoff/PR text; create a GitHub
   reviewer request only when the user names the reviewer.
4. After ordinary merge, wait for merge-triggered main CI success.
5. The next technical step requires a bounded owner governance decision and
   closure of at least one OPS member or an independently useful foundation.

## Continuous boundaries

- No registry/error/schema/state-domain/transition/vector/descriptor/set/
  extension/specification/suite/generated-binding change.
- No new OPS/Management behavior execution and no old vector `expected` change.
- No TARGET/SIG/AUDIT registration or placeholder/future digest.
- Operation spelling, route, private DTO, `OperationSummary`, or reachability is
  not membership; membership is not authorization.
- Event, transition, receipt, `audit_ref`, SQLite row, boolean, log, trace,
  telemetry, or design proposal is not authoritative AUDIT.
- D-016 remains open; D-022 remains blocking; CA-1 through CA-8 remain blocked;
  Profile implemented remains 0.

## Downstream order

1. docs-only eligibility audit owner/security/protocol review and ordinary merge;
2. merge-triggered main CI;
3. bounded OPS member/foundation closure decision and follow-up OPS batch;
4. TARGET machine registration;
5. SIG independent security/cryptography review and SIG registration;
6. AUDIT machine registration;
7. remaining OPS member closure until the OPS line is genuinely registered;
8. independent CA-0 re-review after all four lines close;
9. explicit CA-0 GO;
10. implementation;
11. Management CFR.

第一个动作：只读复核 remote main、当前分支/HEAD、tracked worktree/index、
旁路路径数/哈希、PR/CI/review 状态。任何相反治理裁决、dirty tracked/index
或 CI 红灯都阻止继续。
