# 20260720 M2 Milestone Review

## 1. 范围回顾

M2 = 对象/状态/事件内核（`docs/plan/DEVELOPMENT-PLAN.md` M2 节：GovernedObject 仓储（SQLite WAL，ADR-0002）、五状态机执行器（消费 `specs/transitions/`）、CAS、append-only 事件日志 + outbox、预算计量原语、状态+事件原子提交）。交付分两批：

- **Lane-KRN M2 内核批**（PR #4，merge `72fdb6a`）：`cognitive-domain`（五迁移表编译期嵌入 + digest 钉扎 + newtype 域层）、`cognitive-kernel`（集中 transition 入口：表钉扎→CAS→表行/reason→guards→evidence→硬预算→原子提交；注册错误码单点映射；重放投影）、`cognitive-store`（SQLite WAL 五绑定规则、append-only 触发器、同事务原子提交、outbox、只读降级卷）。工作区 106 Rust 测试（新增 51，含 M2 验收套件 8 项真 SQLite 行为测试）。
- **Lane-CFR M2 行为执行批**（本评审所在 PR）：runner 行为执行模式——以 `cognitive-kernel::TransitionEngine` + `cognitive-store::SqliteAuthorityStore` 为被测实现真实驱动向量场景；3 份向量行为执行（STATE-CAS-002 / EFFECT-STATE-CLOSURE-008 / GW-REMOTE-COMPLETE-001）、state-store-degradation 的 M2 只读降级子集真实执行并落档；行为侧错误实现自检（gate-bypassing 直写 store 的实现被判 fail）。

## 2. 验收判据逐条对照

判据 1–5 = DEVELOPMENT-PLAN M2 节验收判据原文顺序；预算计量为 M2 范围项一并列证。

| # | 判据 | 结果 | 证据 |
|---|---|---|---|
| 1 | 并发 CAS：N 个并发写仅 1 个成功，其余 `STATE_CONFLICT` 且无副作用 | **通过** | KRN：`crates/cognitive-store/tests/m2_acceptance.rs::criterion_1_concurrent_cas_exactly_one_winner_others_state_conflict`（8 线程 barrier 真并发，1 胜 7 败全 STATE_CONFLICT，事件/记录恰 +1）。CFR 行为向量：STATE-CAS-002 → **pass（cas-behavior）**——真实提交 12 次合法迁移到权威版本 13 后陈旧写（expected_version 12）被真实内核门拒绝 STATE_CONFLICT，reload 断言状态/版本/事件日志零变化（报告执行记录含 grounding/evidence；报告 digest 见 §6） |
| 2 | 非法迁移全拒（逐表穷举非法 from→to），状态不变、错误码与 registry 一致 | **通过** | KRN：`criterion_2_every_unregistered_pair_rejected_with_registry_codes_and_state_unchanged`（五表全体非法有序对穷举 >400 例）+ `every_registered_edge_commits_through_the_gate`（91 条合法边全部过门提交，防"全拒"假阳性）。CFR 行为向量：EFFECT-STATE-CLOSURE-008 → **pass（effect-closure-behavior）**——OUTCOME_UNKNOWN→COMMITTED 被真实门拒 EFFECT_OUTCOME_UNKNOWN，rejection 携带 allowed_exits=[RECONCILED]，still-unknown 两出口（COMPENSATING/QUARANTINED）以真实提交演示 + 注册表完备性交叉核对；GW-REMOTE-COMPLETE-001 → **pass（task-acceptance-behavior）**——ACTIVE 强推 COMPLETED 被拒 STATE_CONFLICT、任务保持 ACTIVE、CANDIDATE_COMPLETE 缺证据被拒/带证据可提交 |
| 3 | 投影重放 digest 稳定（事件重放两次 → canonical digest 相同） | **通过** | KRN：`criterion_3_replaying_committed_history_yields_byte_identical_projection_digests`（同库两次重放 byte-identical；重开句柄同 digest；固定 clock/ids 两独立库收敛）。CFR：降级子集探针中降级前后重放 digest 相同（`replay_digest_stable_across_degradation: true` 落报告） |
| 4 | 事件不可原地修改（UPDATE 事件行必须失败/被拒） | **通过** | KRN：`criterion_4_committed_events_and_records_reject_update_and_delete`（裸连接 UPDATE/DELETE/时间戳改写 6+1 式全拒——存储层 BEFORE UPDATE/DELETE 触发器 RAISE(ABORT) 对任意连接生效，历史 digest 不变） |
| 5 | 提交路径故障注入 `STATE_STORE_UNAVAILABLE` fail-closed（`state-store-degradation.json` 行为侧） | **通过（M2 行为侧；disk-full 模式归 M4 故障注入框架）** | KRN：`criterion_6_mid_transaction_failure_leaves_no_partial_commit`（对象 CAS 与预算扣减已执行后注入事件 append 冲突 → 整体回滚四表零残留 STATE_STORE_UNAVAILABLE；故障移除后同命令可提交）+ `criterion_6_read_only_store_fails_closed_and_keeps_reads_available`。CFR：向量 STATE-STORE-DEGRADE-001 的 **M2 只读降级子集真实执行**并作 `partial_contract_assertions.m2_behavioral_read_only_subset` 落报告（写全拒 STATE_STORE_UNAVAILABLE/读路径存活/零缓冲/历史 digest 不丢/恢复后同写可提交，全部 true）；向量本体如实保持 not-run（disk-full 注入、dispatch/stop/revoke 期望 = M4/M5 行为，逐项 deferred 清单落报告） |
| （范围项）硬预算 fail-closed + 同事务扣减 | **通过** | KRN：`criterion_5_over_budget_rejected_fail_closed_and_debit_commits_atomically`（超额拒绝零写、扣减与状态同 commit、耗尽后再拒）+ kernel 层 `hard_budget_admission_is_deterministic_and_rides_the_commit` |

**反虚报证据（runner 行为自检）**：`--self-check` 以 gate-bypassing 直写 store 的错误实现（无表查询、无 CAS 尊重、无 guard/evidence——正是"绕过集中迁移入口写 authority 状态"反模式）执行三份行为向量 → 全部翻 **fail**；合计 **12/12** corrupted 向量翻 fail（静态 9 + 行为 3；self-check 报告 digest 见 §6；CI 步骤断言 ≥12）。

## 3. 安全负例清单（M2 执行）

- 陈旧 CAS 写拒绝（STATE-CAS-002，行为执行 + KRN 真并发）；
- 非法迁移穷举全拒（五表 >400 负例，KRN）+ OUTCOME_UNKNOWN 非法出口（EFFECT-STATE-CLOSURE-008 行为执行）；
- 远端 completed 强推本地 COMPLETED 拒绝（GW-REMOTE-COMPLETE-001 行为执行：remote report 只是观察证据，authority 状态只经门）；
- 事件/记录 UPDATE/DELETE 全拒（存储触发器，KRN）；
- 超预算拒绝（fail-closed 零部分扣减，KRN）；
- 提交路径故障 fail-closed（事务中断注入 + 只读降级，KRN + CFR 向量子集）；
- 错误实现（gate-bypass 直写）被 runner 判 fail（3 份行为向量 + 9 份静态）。

## 4. 五态分布变化（本批）

81 向量：**pass 30 → 31**（GW-REMOTE-COMPLETE-001 脱 not-run）、**not-run 51 → 50**；STATE-CAS-002 与 EFFECT-STATE-CLOSURE-008 执行模式由 M1 静态门升级为 **行为执行**（cas-behavior / effect-closure-behavior，被测 = 真实 kernel/store 路径）；STATE-STORE-DEGRADE-001 保持 not-run 但携带真实执行的 M2 行为子集断言。fail / not-applicable / documented-degradation 均 0。

## 5. 漂移与规范变更

无新漂移登记；无 schema/向量/REQ/错误码表面变化（钉扎计数 60 schema / 81 向量不变）。向量文件零改写（口径红线遵守）。findings-ledger 更新：F-005 → verified-by-vector（M2 行为侧）；F-008 补 M2 行为子集证据；F-012 补 task 状态机行为证据；F-001 证据推进注记。

## 6. 指标快照

- 向量五态（实测）：**pass 31 / fail 0 / not-applicable 0 / documented-degradation 0 / not-run 50**；行为执行 3 + 行为子集落档 1；自检 12/12。
- 本批参考运行报告（本地）：`artifacts/evidence/conformance/conformance-report.json`（digest 由 runner 打印，合并 PR 的 CI 产物为准）；self-check-report.json 同。
- Rust 测试：工作区全绿（KRN 106 + CFR 本批行为集成测试 7 项）；clippy -D warnings / fmt 绿。
- 开放 P0：0（合同缺口类）；F-001 证据缺口持续消解（内核行为测试 51 项 + 行为向量 3 份在案）。

## 7. 结论

**GO → M3（M2 done）**。M3 入口 gate（DEVELOPMENT-PLAN：M2 出口 + F-007 行为侧测试计划评审）：M2 出口分量本评审达成；F-007 行为侧测试计划评审 = M3 启动会话第一动作（KRN handoff §5 已定义入口：先写「capability 交集只缩不扩」与「撤销后缓存复用被拒」失败测试，正是 F-007 行为侧计划的落地）。遗留条件（不阻断 M3）：

1. STATE-STORE-DEGRADE-001 完整行为执行（disk-full 故障注入 + dispatch/stop/revoke 期望）挂 M4/M5，deferred 清单已落报告。
2. 事件 envelope 升格（GovernedObjectHeader 全治理字段）与请求幂等归 M3/M4（KRN handoff §2 既定）。
3. 51 → 50 份 not-run 行为向量的后续消化路径不变：M3（context/治理链）→ M4（恢复/fencing/幂等）→ M5+（shell/management/memory/discovery/catalog/semantic）。
