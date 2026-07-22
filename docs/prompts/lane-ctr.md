# Lane-CTR 接续提示词：V02-CA-AUDIT-01 Review/Merge Gate

> 当前唯一入口：review/merge v0.2 authoritative-audit docs-only design。
> AUDIT 合入后才按顺序进入 OPS/TARGET/SIG/AUDIT 四类独立 machine
> registration。本提示词不授权任何 machine registration、CA-0 GO、实现、
> 行为执行或 Profile claim。

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
   - `docs/plan/V02-CA-AUDIT-DESIGN-DECISION.md`
   - `docs/adr/0013-v02-authoritative-audit-governance.md`
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
   - `docs/standards/event-audit-watch.md`
   - `docs/standards/state-and-transition-contract.md`
   - `docs/standards/intent-effect-idempotency.md`
   - `docs/standards/authn-authz-capability.md`
   - `docs/standards/canonical-encoding-and-digest.md` §9/§10/§12/§13/§15
   - `docs/standards/normative-source-and-versioning.md`
   - `docs/standards/docs-sync-contract.md`
   - `docs/standards/conformance-evidence.md`

## 已闭合入口事实

- SIG PR #53 已 merge；head
  `c27787941e5d260a116ffe39fc76bb8c21d152b3`，merge commit
  `0a30ac70769f0501f7928d96f55f17636eaa9888`。
- Merge-triggered main CI run `29930557168` 的 Ubuntu/Windows 均 success。
- PR #53 reviews、review decision、requested reviewers 为空；SIG independent
  security review 不得声称完成。
- PR #50～#53 的 merge/review 行为或任何单次例外都不适用于 AUDIT、
  machine registration、CA-0、implementation 或 CFR。
- OPS/TARGET/SIG merge 不等于 operation membership、target authority、
  signature machine profile、CA-0 GO 或实现入口。

## AUDIT owner-confirmed docs-only selections

- Event 是跨边界 outer envelope；只有未来 closed
  `AuthoritativeAuditRecord` payload/profile 才是 authoritative carrier。
- platform stream 与 tenant stream 分离；tenant stream 按 tenant、management
  domain、audit-profile digest 分区，禁止 mixed global tenant stream。
- 每 stream 单一 current fenced sequence authority；连续 logical sequence、
  previous-record digest chain、CAS high-watermark、duplicate/gap/regression/
  fork 检测；wall-clock、UUID、数据库自增不构成 ordering proof。
- digest-pinned finite checkpoint policy；signed periodic checkpoints 使用
  独立 `audit-checkpoint-signing` key/profile。初始 profile 不要求 Merkle 或
  external WORM；二者若加入须新版本/迁移。
- retention floor 来自 exact digest-pinned policy，不虚构统一年限；有效 floor
  取全部适用义务最大值。Legal hold 只能由不同 principal/ActorChain 的同级
  或更高 compliance authority 解除，被审计主体/写入者/hold setter 均不能解除。
- record 在创建时最小化；secret、protected body、cross-tenant content、raw/
  replayable signature material 禁止写入。Redaction 只能产生 digest-pinned
  deterministic derived view，不能原地修改 authoritative bytes。
- export 为 ordered RFC 8785 NDJSON + canonical manifest；固定 authorization、
  filter/scope、redaction、record digests、checkpoint/high-watermark，并使用独立
  `audit-export-signing` key/profile 签名；export 本身被审计。
- denial 允许且要求一个安全 audit commit，但业务 commit/dispatch/Effect/
  mutation/success receipt 均为零；successful governed commit 必须原子闭合
  state/transition/Event/SIG receipt handoff/audit/outbox/result visibility。
- external Effect 使用 pre-dispatch → attempt → outcome/unknown → reconcile →
  Verification → commit/abort/quarantine → closure 的可恢复链。

全部选择仍是 docs-only design。精确 schemas、digests、policy values、checkpoint
thresholds、keys/usages、errors/category、critical extension、persistence port、
vectors、bindings 与 implementation 均未登记或提供。

## 当前任务

1. 核验 AUDIT PR head、文件范围、checks、reviews 与请求 reviewer 状态。
2. 等待独立 owner/security/audit/compliance review 和普通 merge；不得自动 merge。
3. 不得沿用 PR #50～#53 的任何 review 例外。
4. 只有 AUDIT merge 且 merge-triggered main CI Ubuntu/Windows success，才进入
   四类独立 machine-registration batches。
5. 只有用户明确要求并指定 reviewer 时才创建 GitHub reviewer request。

## 持续边界

- 不登记 registry、errors、schemas、state domain、transition、vector、
  descriptor、set、extension、generated binding、evidence 或 Profile。
- 不修改既有 vector `expected`；AUDIT negatives 保持 planned/not executed。
- Event、transition record、receipt、`audit_ref`、SQLite row、outbox、DTO、
  boolean、log、trace、telemetry 或 vector fact 都不是 authoritative audit。
- audit receipt/checkpoint/export 不替代 authorization、Effect closure、
  Verification、authority commit、acceptance 或 completion。
- 不启动 CA-1～CA-8、KRN/RUN/CFR implementation。
- D-016 不 closed；D-022 继续 blocker；Profile implemented = 0。

## AUDIT 合入后的顺序

1. OPS/TARGET/SIG/AUDIT 四类独立 machine-registration batches；
2. CA-0 re-review；
3. CA-0 明确 GO；
4. implementation；
5. Management CFR。

第一个动作：只读重核验 AUDIT PR、remote main、tracked worktree/index、旁路
路径、review/checks 与独立 reviewer 状态；review/merge gate 完成前不改文件。
