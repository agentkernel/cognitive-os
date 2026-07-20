# 20260720 M3 Milestone Review

## 1. 范围回顾

M3 = 治理链与 Context（`docs/plan/DEVELOPMENT-PLAN.md` M3 节：TenantContext/Principal/Membership/ActorChain 决策快照、capability 交集 + 单调衰减 + 撤销、九阶段确定性 Context Resolution、缓存键治理绑定、确定性渲染与前缀稳定（IMP-02）；标准 `authn-authz-capability.md`、`context-resolution-and-cache.md`）。交付分两批：

- **Lane-KRN M3 批**（PR #9，merge `3d9187c`）：`cognitive_domain::capability`（衰减/交集算术）+ `cognitive_kernel::{authz, context, context_cache}`（六步判定序、拒绝与 not-found 字节同形、九阶段管线、7+1 维治理绑定缓存、确定性渲染、停滞界）+ store 层 F-007 双竞态测试；workspace 131 Rust 测试（M3 新增 26，全部含负例）。
- **Lane-CFR M3 行为执行扩展批**（本评审所在 PR）：runner 九个治理/context 行为门——8 份向量脱 not-run + CTX-TRUST-004 静态→行为升级，被测 = 真实 `authz`/`context`/`context_cache`/`capability` 公开 API；治理类错误实现自检扩展（corrupted 语料 12→20，全部翻 fail）。

## 2. 验收判据逐条对照

判据 1–7 = DEVELOPMENT-PLAN M3 节验收判据原文（全部为安全负例或含负例）；范围项（capability 算术、停滞、F-007 竞态）一并列证。

| # | 判据 | 结果 | 证据 |
|---|---|---|---|
| 1 | 同租户横向越权被拒且响应与 not-found 同形（`tenant-lateral-read-denial`） | **通过** | KRN：`governance_gate::criterion_1_same_tenant_lateral_read_denied_isomorphic_with_not_found`。CFR 行为向量：GOBJ-TENANT-LATERAL-001 → **pass（lateral-read-behavior）**——`protected_read` 拒绝 CONTEXT_AUTH_DENIED，「存在但无权」与「不存在」序列化字节相同（实测比对），denial/audit 不含正文标记与 owner 名，DenialAudit 服务端产出（audit_required 实测），own-scope 正例对照通过 |
| 2 | 管理员身份不含正文授权时读取正文被拒（管理≠读内容） | **通过** | KRN：`governance_gate::criterion_2_admin_governance_capability_does_not_read_body`（govern 允许、read_body 拒，同一 capability；无专属向量——判据原文即无向量点名，内核行为测试为证据） |
| 3 | 撤销后缓存复用被拒（`context-revocation-cache-reuse`，epoch 键失配） | **通过** | KRN：`context_pipeline::criterion_3_revocation_invalidates_cached_views_by_key_mismatch`。CFR 行为向量：CTX-REVOKE-CACHE-001 → **pass（revocation-cache-behavior）**——当前绑定结构性 miss、声明陈旧绑定拒 CONTEXT_AUTH_DENIED、四类派生缓存（kv/prompt/embedding/summary）随条目失效（InvalidationReport 实测）、二次声明无可购回、陈旧 epoch 链新解析零加载 |
| 4 | 检索前过滤：ranker 输入集不含未过滤对象（`context-rank-before-auth`） | **通过** | KRN：`context_pipeline::criterion_4_unauthorized_bodies_never_reach_the_ranker`。CFR 行为向量：CTX-RANK-AUTH-001 → **pass（rank-before-auth-behavior）**——RecordingRanker 证明未授权正文（标记字符串）从未进入 ranking，未授权项落 rejected（CONTEXT_AUTH_DENIED），HostileRanker 无法购回被拒项或注入外部 ref |
| 5 | 跨 Conversation 污染被拒；注入内容不得提升为控制（`prompt-injection-isolation`） | **通过** | KRN：`context_pipeline::criterion_5_cross_conversation_candidates_are_rejected_before_ranking` + `untrusted_input_cannot_reach_the_control_plane`。CFR 行为向量：CTX-TRUST-004 → **pass（trust-plane-behavior，M1 静态门升级）**——真实管线保持注入项 untrusted_input 角色、`effective_control_plane` 只认 authority 声明的 control 项、`admit_control_mutation` 拒绝 untrusted 源的控制变更、零 capability 铸造 |
| 6 | required 超预算 fail-closed（`context-required-over-budget`） | **通过** | KRN：`context_pipeline::criterion_6_required_over_hard_budget_fails_closed`（向量原数值）。CFR 行为向量：CTX-REQ-007 → **pass（required-budget-behavior）**——向量本身的字节/token 数（4096/512 vs 3300+1700/430+240）驱动真实管线 → CONTEXT_BUDGET_EXCEEDED、无 view 发射、missing_items 恰为超额 required 项 |
| 7 | 渲染字节稳定 + 前缀稳定断言（`context-render-stability`） | **通过** | KRN：`context_pipeline::criterion_8_render_is_byte_stable_and_prefix_stable`。CFR 行为向量：CTX-RENDER-001 → **pass（render-stability-behavior）**——两次解析 render digest/bytes 相同；新增无关对象后旧段字节不变、旧流为新流严格前缀、新内容 append-only suffix（全部从真实 RenderedView 读回比对） |
| （范围项）capability 交集只缩不扩 + 单调衰减 | **通过** | KRN：domain `intersection_only_narrows_and_is_commutative` + `governance_gate::criterion_7` + `widening_any_dimension_is_rejected`（向量原数值）。CFR 行为向量：CAP-ATTEN-004 → **pass（attenuation-behavior）**——`attenuation_violations` 对向量的 parent/derived 实际数值（5000→7500）返回恰 `["parameter_binding.max_amount_minor"]`，拒绝 AUTH_CAPABILITY_ATTENUATION_VIOLATION |
| （范围项）有界停滞 + 候选收窄 | **通过** | CFR 行为向量：DISC-STAGNATION-004 → **pass（stagnation-behavior，`ResolutionSession` 有界无增益重试 → CONTEXT_RESOLUTION_STAGNATED）**；DISC-ADMISSION-002 → **pass（candidate-admission-behavior，越权/跨租户候选在预过滤+授权阶段收窄，ranker 只见收窄集）**；KRN：`repeated_no_gain_resolution_stagnates_with_the_registered_code` |
| （范围项）F-007 双竞态 | **通过** | KRN：`m3_revocation_races::{revocation_after_resolution_blocks_dispatch, revocation_after_dispatch_blocks_commit}`（真 engine+SQLite，未撤销对照组可提交）；台账 F-007 → verified-by-vector（M3 行为侧） |

**反虚报证据（治理类自检）**：`--self-check` 新增八种治理错误实现（真实驱动：membership 即读、先 rank 后授权、忽略 epoch 维度的缓存命中、静默截断 required、无界重试、重排渲染、内容声称即控制面、接受放大衍生）→ 对应行为向量全部翻 **fail**；合计 **20/20** corrupted 向量翻 fail（CI 步骤断言 ≥20）。

## 3. 安全负例清单（M3 执行）

- 同租户横向越权拒 + 拒绝/不存在同形（GOBJ-TENANT-LATERAL-001 行为执行 + KRN criterion 1）；
- 管理权≠读内容（KRN criterion 2）；
- 撤销后缓存复用拒 + 派生缓存连带失效（CTX-REVOKE-CACHE-001 行为执行 + KRN criterion 3 + F-007 双竞态）；
- 未授权正文不达 ranker + 排名分不可购回（CTX-RANK-AUTH-001 行为执行 + KRN criterion 4 + HostileRanker 负例）；
- 跨 Conversation/跨租户候选预过滤拒（KRN criterion 5 + DISC-ADMISSION-002 行为执行）；
- 注入内容不得提升为控制/铸造 capability（CTX-TRUST-004 行为执行 + KRN 附加测试）；
- required 超预算 fail-closed（CTX-REQ-007 行为执行 + KRN criterion 6）；
- capability 放大拒（CAP-ATTEN-004 行为执行 + KRN 全维度负例）；
- 无增益重试有界拒（DISC-STAGNATION-004 行为执行）；
- 八种治理错误实现被 runner 判 fail（自检）。

## 4. 五态分布变化（本批）

81 向量：**pass 31 → 39**（8 份治理/context 向量脱 not-run）、**not-run 42**；CTX-TRUST-004 执行模式静态→行为升级；DISC-DELTA-SCOPE-003 如实保持 not-run（delta 消费 = M5 运行时路径，无内核 API 可执行，理由入报告）。fail / not-applicable / documented-degradation 均 0。行为执行向量累计 **12**（M2 3 + M3 9）。

## 5. 漂移与规范变更

无新漂移登记；schema/向量/REQ/错误码表面零变化（60/81 钉扎不动），向量文件零改写。findings-ledger 升级：F-007 → verified-by-vector（M3 行为侧）；F-018/F-021 → verified-by-vector（M3 行为执行）；F-004 补 M3 管线结构侧注记（完整运行时 admission 决策明确挂 M5）；F-001/F-015 证据推进。F-006/F-010 本批未执行（其向量 = effect-recovery/恢复行为，M4 自身验收判据 1~5 的交付物），台账维持 M4 挂载不变。

## 6. 指标快照

- 向量五态（实测）：**pass 39 / fail 0 / not-applicable 0 / documented-degradation 0 / not-run 42**；行为执行 12；自检 20/20。
- Rust 测试：workspace 全绿（KRN 131 基线 + CFR 本批集成测试 8 项）；clippy -D warnings / fmt 绿。
- 开放 P0：0（合同缺口类）；F-001 证据缺口持续消解。开放 P1：F-011（M5）、F-014（M4）、F-023（M4）、F-017（M6）、F-015（持续）。开放漂移：0（D-016/D-017 deferred-to-v0.2 为决策落档）。
- 报告与 self-check digest 由 runner 打印（本地值见 handoff；CI 产物为准）。

## 7. 结论与 M4 入口 gate 判定

**GO → M4（M3 done）**。M4 入口 gate 按 DEVELOPMENT-PLAN = **F-002~F-010 类 P0 全闭合（= M1 出口 + M2/M3 行为验收；findings-ledger 为准）+ F-014/F-023 排入 M4**，逐条核验：

- F-002 closed-by-1.0.1 ✓；F-003 closed-by-M1（runner 负例执行）✓；F-004 closed-by-1.0.1（M1 静态复验 + M3 管线结构侧；运行时 admission 决策按 KRN 口径归 M5 运行时组装器，非合同缺口）✓；F-005 verified-by-vector（M2 行为侧）✓；F-006 closed-by-1.0.1（行为验收 = M4 判据 2 交付物）✓；F-007 **verified-by-vector（M3 行为侧）**✓；F-008 closed-by-1.0.1（M2 行为子集在案；disk-full = M4 故障注入框架交付物）✓；F-009 closed-by-1.0.1 ✓；F-010 closed-by-1.0.1（行为验收 = M4 判据 1/5 交付物）✓。
- **无开放 P0**；F-006/F-008(disk-full)/F-010 的行为项本身就是 M4 验收判据的交付物，不构成入口循环依赖。
- F-014（sink fencing 矩阵）与 F-023（准入拒绝矩阵）已在 M4 范围内排入（DEVELOPMENT-PLAN M4 范围原文）。

**判定：tracer bullet 入口 gate 开启，Lane-KRN M4 可启动。** 遗留条件（不阻断）：

1. DISC-DELTA-SCOPE-003 行为执行挂 M5（delta 消费路径落地后）。
2. F-004 完整运行时 admission 决策挂 M5；F-011 R1 合同登记仍为 M5 出口阻断。
3. 渲染 digest 域（`cognitiveos.impl.context-render/0.1`）待 CTR 并入 D-017 或出具同款判定；membership 生成绑定评估、D-018 事件 envelope 升格评估同批（KRN M3 handoff §4）。
