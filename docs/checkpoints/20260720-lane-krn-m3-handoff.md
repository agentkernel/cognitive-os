# 20260720 Lane-KRN Handoff（M3 治理链与 Context 批）

## 1. 本次会话完成

按 `docs/prompts/milestone-m3.md` / `docs/prompts/lane-krn.md` 交付 M3（分支 `lane/krn`，基线 `2dc3e02` = CFR M2 行为执行批 merge）。零新 Cargo 依赖、零 schema/runner 触碰；提交哈希见 §6。

- **`cognitive-domain::capability`（纯算术层）**：`CapabilityConstraints`（subject/audience/resource/purpose/actions/parameter_bounds/lease/depth/issued_epoch）；`attenuation_violations`（单调衰减违规清单，字段路径拼写对齐 `capability-attenuation.json` 的 `parameter_binding.<name>`；数值上界只降、集合只缩、lease 只窄、depth 严格递减、耗尽 depth fail-closed）；`intersect_chain`（链交集只缩不扩：actions ∩、参数取最紧、lease 取重叠、resource 取嵌套较窄者、purpose 必须一致、最旧 issued_epoch 治理链新鲜度；空链 = default deny）；`LeaseWindow`（半开区间 contains/intersect）。`ids::WallTimestamp::instant_key()`：canonical 时间戳的正确瞬时比较（裸字典序在变长小数位下不正确——`..00.5Z` vs `..00Z`——文档化并测试钉扎）。
- **`cognitive-kernel::authz`（六步判定序）**：认证/链解析 → 租户+membership（同租户身份本身零授权）→ capability 链交集 + 撤销 epoch 时效（stalest link < 当前 epoch = 陈旧材料拒绝）→ 显式 deny 压倒 allow（default deny）→ lease（`AUTH_CAPABILITY_EXPIRED`，过期不延展）→ scope/purpose/action 绑定（`CONTEXT_AUTH_DENIED`）。**拒绝与 not-found 同形**：`protected_read` 对「存在但无权」与「不存在」产出字节相同的公开拒绝（固定 detail 文本不含对象名/owner；`DenialAudit` 服务端留存，REQ-SEC-001）。**F-007 原语**：`revalidate_grant`（epoch/capability 集版本/lease 三因子）+ `capability_and_revocation_current`（effect 表该 guard 的唯一 sanctioned attestation 派生路径）。
- **`cognitive-kernel::context`（九阶段确定性管线，REQ-CTX-007 逐阶段 reason code）**：admission → 治理预过滤（跨租户 fail-closed 审计、跨 Conversation 结构性拒绝，先于一切内容接触）→ 候选集固定 → **逐对象正文授权重验（先于 ranker/renderer）** → ranking（`ProposalRanker` 端口 = 唯一概率槽位：只见授权幸存者，输出仅可重排/缩减——集合外/重复提案被弃并记录，`rank_score_cannot_restore_denied_item`）→ 预算装配（required 先行：缺失/无权 → `CONTEXT_INCOMPLETE`，超硬预算 → `CONTEXT_BUDGET_EXCEEDED`，均 fail-closed 不发 view；显式 partial → `complete:false` + missing 清单）→ loss declaration（optional 降级必须显式，禁静默）→ **确定性渲染**（header 段只依赖稳定绑定；per-item 段字节只依赖该 item；分区序 control→authoritative_state→evidence→working→untrusted_input，分区内保持装配序 = 新增无关对象 append-only suffix，前缀字节稳定，IMP-02/REQ-CTX-012）→ view 发射（loaded/rejected/missing/loss/pinned_versions/stage_records/ranker_input_refs/render/binding）。控制面：`effective_control_plane`（只认 authority 声明 control 角色 + control/authoritative 信任）与 `admit_control_mutation`（untrusted 来源的控制变更拒绝，REQ-CTX-008/REQ-SEC-002）。`ResolutionSession`（REQ-DISC-STAGNATION-001：有界无增益重试 → `CONTEXT_RESOLUTION_STAGNATED`）。
- **`cognitive-kernel::context_cache`（治理绑定缓存）**：`GovernanceBinding` 键 = tenant / actor_chain_digest / capability_set_version / revocation_epoch / purpose / schema_digest / encoding_profile **+ conversation**（标准七维为下限）；`lookup_current` 只按当前绑定精确命中（陈旧键结构性不可达）；`serve_declared`（声明陈旧绑定 → `CONTEXT_AUTH_DENIED` + 条目按键清除 + 派生缓存 kv/prompt/embedding/summary 随条目失效并出具 `InvalidationReport`）；`evict_stale_epochs` 为 housekeeping（正确性不依赖扫描）。
- **`cognitive-kernel::error` 扩展**：新增注册码常量 `CONTEXT_AUTH_DENIED`/`CONTEXT_INCOMPLETE`/`CONTEXT_BUDGET_EXCEEDED`/`AUTH_CAPABILITY_ATTENUATION_VIOLATION`/`AUTH_CAPABILITY_EXPIRED`/`CONTEXT_RESOLUTION_STAGNATED`（三元组逐条钉扎 `errors.yaml` 的测试同批扩展至 11 码）。
- **文档联动**：matrix 回填 16 REQ 的 impl/impl_tests/evidence/notes（清单见 §3）；findings-ledger F-007/F-018/F-021 行为证据状态推进；PROGRESS（M3 行、计数 8→24、车道表、handoff 列表）；本 handoff。

## 2. 未完成 / 进行中

- **M3 出口评审未做（有意）**：等 CFR 行为执行扩展批（security-negative/context 向量对内核公开 API 真实执行）后由协调者安排；本批口径 =「实现已提供 + Rust 行为测试已执行」，向量计数保持 81/31/50 未动（本地复跑 runner 确认，报告 sha256:950b...43e1，self-check 12/12）。
- 治理对象（Principal/Membership/Conversation/ResourceScope）以**决策快照**形态进入门（`AuthzSnapshot`/`ObjectGovernance`），持久化编排归 M5 运行时（治理对象无注册迁移表，不走五表 transition 机制；快照组装器 = 运行时职责）。
- ContextView 发射为内核投影类型（`ResolvedContextView`），完整 `context-view.schema.json` envelope（GovernedObjectHeader + ActivityContext 两阶段 CAS 绑定，REQ-GOBJ-BIND-004）挂 M5 运行时（同 M2 事件 envelope 升格路径）。
- F-004 行为侧（运行时 admission 拒绝）：管线 admission 阶段现只做结构校验，`context-request-admission.schema.json` 的完整运行时 admission 决策挂 M5。
- 渲染 target 目前实现 `structured` profile；llm/human profile 渲染器挂 M5+（渲染器版本已入缓存键，扩展不破坏既有键）。
- **消费 CTR 缺口批（PR #8，本批 rebase 时并入）**：state-transition-request/record 已入 codegen 生成集（30 模块）——`engine.rs` 手写记录组装替换为生成绑定的重构排 M4 首批（行为零变化的机械替换，替换时删除 M2 handoff §4.1 缺口注记）。

## 3. 测试与证据状态

- **Rust（本地全绿 + 待 CI 复核）**：`cargo build/test/clippy -D warnings/fmt --check` 全绿；workspace 131 测试，其中 M3 新增 **26**：domain +8（capability 7 + instant_key 1）、kernel error +0（既有测试扩展）、`tests/governance_gate.rs` 9、`tests/context_pipeline.rs` 7、store `tests/m3_revocation_races.rs` 2。
- **验收判据 ↔ 测试对照**（判据编号按协调者任务书；全部含负例）：
  1. 同租户横向越权拒 → `governance_gate::criterion_1_same_tenant_lateral_read_denied_isomorphic_with_not_found`（CONTEXT_AUTH_DENIED + 拒绝/不存在字节同形 + audit 无泄露 + owner 自读正例对照）；
  2. 管理员正文读取拒 → `governance_gate::criterion_2_admin_governance_capability_does_not_read_body`（govern 允许、read_body 拒，同一 capability）；
  3. 撤销后缓存复用拒 → `context_pipeline::criterion_3_revocation_invalidates_cached_views_by_key_mismatch`（当前键结构性 miss + 声明陈旧键 CONTEXT_AUTH_DENIED + 派生缓存四类随条目失效 + 二次声明无可购回 + 陈旧链新解析零加载）；
  4. 检索前过滤 → `context_pipeline::criterion_4_unauthorized_bodies_never_reach_the_ranker`（RecordingRanker 证明输入集不含未授权 ref/body + HostileRanker 无法购回被拒项或注入外部 ref）；
  5. 跨 Conversation 污染拒 → `context_pipeline::criterion_5_cross_conversation_candidates_are_rejected_before_ranking`（预过滤阶段拒绝、ranker 未见、跨租户候选同阶段 fail-closed）；
  6. required 超预算 fail-closed → `context_pipeline::criterion_6_required_over_hard_budget_fails_closed`（`context-required-over-budget.json` 原数值：4096B/512tok vs 3300+1700 → CONTEXT_BUDGET_EXCEEDED + missing_items + 无 view；required 缺失 → CONTEXT_INCOMPLETE；显式 partial → complete:false；optional 超额 → 显式 loss declaration）；
  7. capability 交集只缩不扩 → domain `capability::tests::intersection_only_narrows_and_is_commutative` + `governance_gate::criterion_7_capability_intersection_only_narrows_at_the_gate`（加链只减权、参数取最紧）+ 衰减违规全维度负例（`widening_any_dimension_is_rejected`，CAP-ATTEN-004 原数值）；
  8. 确定性 + 前缀稳定 → `context_pipeline::criterion_8_render_is_byte_stable_and_prefix_stable`（同输入 byte-identical、九阶段记录序断言、无关新增后旧段字节不变/旧流为严格前缀/新内容 append-only suffix）。
  附加：注入隔离 `untrusted_input_cannot_reach_the_control_plane`（渲染角色保留 + 控制面不可变 + 零 capability 铸造）；停滞 `repeated_no_gain_resolution_stagnates_with_the_registered_code`；F-007 双竞态 `m3_revocation_races::{revocation_after_resolution_blocks_dispatch, revocation_after_dispatch_blocks_commit}`（真 engine+SQLite，未撤销对照组可提交）。
- **matrix impl 回填（16 REQ）**：REQ-CAP-001/002/003/005、REQ-CTX-002/004/005/006/007/008/011/012、REQ-SEC-001/002、REQ-DISC-STAGNATION-001、REQ-PROFILE-CVM-001（evidence = 本 handoff；notes 逐条限定行为测试范围）。
- **向量**：0 变化（81 枚举 / 31 pass / 50 not-run 保持；runner 与 CI 钉扎未触碰；`pnpm -r build/test`、`check:consistency`（273/55/60/81）、`gen-matrix --check`、self-check 12/12 全绿）。
- CI：push + PR 后 Windows+Linux 矩阵全绿方可合并（合并事实见 PR）。

## 4. 未决风险与漂移（含交 Lane-CTR 契约缺口）

无规范资产间漂移登记。缺口/口径（M2 清单基础上新增）：

1. **渲染 digest 域未注册**：`cognitiveos.impl.context-render/0.1`（实现域，`context.rs` 常量注释声明），与 M2 投影域 `cognitiveos.impl.execution-status-projection/0.1` 同一处置路径——CTR 已对投影域判 **D-017 deferred-to-v0.2**（`cognitiveos.impl.` 域维持），建议渲染域并入 D-017 范围或由 CTR 出具同款判定。
2. **membership.schema.json 无生成绑定**（codegen 0.2.0 的 28 模块不含 membership；principal/actor-chain/resource-scope/conversation-binding 均有）——M3 以 `MembershipFacts` 快照类型消费，M5 组装器若需 schema 级 Membership 对象请 CTR 评估纳入 codegen。
3. **拒绝同形 vs SHELL_TARGET_NOT_FOUND 口径**（提请 CTR/CFR 知悉，非缺口）：M3 受保护读路径按 error-contract §5 以 `CONTEXT_AUTH_DENIED` 同形覆盖「不存在」；shell 层 `SHELL_TARGET_NOT_FOUND` 语义（存在性不受保护的目标解析）归 M5，两者边界在 shell 任务书中应显式（避免 M5 误用 not-found 泄露存在性）。
4. **guard 语义分层口径**（延续 M2 口径 4）：effect 表其余 guards（`fencing_epoch_current`/`idempotency_binding_valid`/`intent_durably_persisted`）在 F-007 竞态测试中为 fixture 事实，其 sanctioned 派生器归 M4（幂等/fencing/Intent 持久化落地时逐一收口，与 `capability_and_revocation_current` 同模式）。
5. 本机 Windows 工具链坑照旧（M2 handoff §4.5：llvm-mingw CC/AR 绝对路径 + dlltool-shim 前置；本批无新增依赖故无新坑）。

## 5. 下一步入口

- **CFR 行为执行扩展批（候选脱 not-run 清单，均已有内核行为孪生）**：
  - security-negative：`tenant-lateral-read-denial`（GOBJ-TENANT-LATERAL-001）、`context-rank-before-auth`（CTX-RANK-AUTH-001）、`context-revocation-cache-reuse`（CTX-REVOKE-CACHE-001）、`capability-attenuation`（CAP-ATTEN-004）、`prompt-injection-isolation`（CTX-TRUST-004，M1 静态 pass 可升级行为执行）；
  - context-semantic：`context-required-over-budget.json`、`context-render-stability.json`（CTX-RENDER-001）、`context-resolution-stagnation.json`（DISC-STAGNATION-004）、`context-candidate-admission.json`（DISC-ADMISSION-002，narrowing 语义 = 预过滤+授权阶段行为）、`context-delta-scope.json`（DISC-DELTA-SCOPE-003，delta 不扩 scope——注意：delta 消费实现挂 M5，CFR 自行判定是否保持 not-run）；
  - 被测路径 = `cognitive_kernel::{authz, context, context_cache}` 公开 API（纯确定性，无 store 依赖；F-007 竞态另有 store 集成测试可作 grounding）。
- **Lane-KRN M4**：Intent/Effect/幂等/reconcile/checkpoint/恢复八步 + 故障注入框架 + F-014 sink fencing 清单 + F-023 准入矩阵 + tracer bullet（`docs/prompts/lane-krn.md` M4 节）；入口 gate = M3 出口 + F-002~F-010 类全闭合。
- **Lane-CTR**：M2 缺口清单 + 本批 §4.1/4.2 两项。
- 工作分支：`lane/krn`；第一动作（M4 会话）：读 `docs/standards/intent-effect-idempotency.md` 全文，先写「同键异参 EFFECT_IDEMPOTENCY_CONFLICT 拒绝」与「eff-crash-001 dispatch 前崩溃恢复」失败测试。

## 6. 快照

- PROGRESS 已更新：是（M3 行、实现计数 8→24、车道表、handoff 列表）。
- 本次提交（按序，哈希见 git log）：domain capability 批 → kernel authz/context/cache/error 批 → store F-007 竞态测试批 → docs 联动批（matrix + ledger + PROGRESS + 本 handoff）。
