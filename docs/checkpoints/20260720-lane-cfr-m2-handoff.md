# 20260720 Lane-CFR Handoff（M2 行为执行批）

## 1. 本次会话完成

按协调者任务书（M2 行为向量执行批 + M2 出口评审）交付（分支 `lane/cfr`，基线 `3b8461b` = CTR 缺口批 merge，含 KRN M2 内核批 `72fdb6a`）：

- **任务 1 — runner 行为执行模式**（`c74c345`；REQ-STATE-003/REQ-EFF-STATE-001/REQ-GW-002/REQ-INTENT-ACCEPT-001/REQ-REC-003 证据）：新增 `exec/behavior` 模块，被测实现 = `cognitive_kernel::TransitionEngine` + `cognitive_store::SqliteAuthorityStore`（真实权威路径；conformance 为叶子 crate，依赖方向合法）。每向量一个 throwaway SQLite WAL 库 + 确定性 harness（固定时钟/顺序 UUIDv7）。KRN 候选清单逐项清点结果：
  - `STATE-CAS-002` → **行为 pass**（cas-behavior）：12 次真实过门提交到权威 v13（ACTIVE↔BLOCKED ping-pong，guards/evidence 齐备），陈旧 expected_version=12 写被拒 STATE_CONFLICT，reload 断言状态/版本/事件日志零变化；audit_required 为合同常量（依据落 evidence）。
  - `EFFECT-STATE-CLOSURE-008` → **行为 pass**（effect-closure-behavior）：seeded OUTCOME_UNKNOWN Effect 拒绝 COMMITTED 出口（EFFECT_OUTCOME_UNKNOWN，rejection 自带 allowed_exits=[RECONCILED]）；still-unknown 双出口（COMPENSATING/QUARANTINED）以真实提交演示 + 注册表完备性交叉核对（不一致会 Environment fail，防表/实现漂移静默）。
  - `GW-REMOTE-COMPLETE-001` → **脱 not-run，行为 pass**（task-acceptance-behavior）：ACTIVE 强推 COMPLETED 被拒 STATE_CONFLICT、任务保持 ACTIVE、事件日志零增长（acceptance_committed=false 实测）；CANDIDATE_COMPLETE 缺 completion_claim/fixed_post_state 证据被拒、带证据可提交（探针对象）；`transition_to_completed_requires` 为 prose 记录不比对（注册表两跳路径落 evidence）。
  - `STATE-STORE-DEGRADE-001` → **如实保持 not-run**，M2 只读降级子集**真实执行**落 `partial_contract_assertions.m2_behavioral_read_only_subset`（写全拒 STATE_STORE_UNAVAILABLE/读存活/零缓冲/重放 digest 跨降级不变/恢复后同写可提交）；disk-full 注入与 dispatch/stop/revoke 期望 deferred M4/M5（清单落报告）。字段 `static_contract_assertions` → `partial_contract_assertions`（语义如实：M1 静态 + M2 行为子集）。
- **任务 2 — 钉扎与自检同批调整**（同 `c74c345`）：五态 81/**31**/0/0/0/**50**（ci.yml + runner_execution.rs 7 测试同批红→绿）；行为自检 = gate-bypassing 直写 store 的错误实现（schema-shaped 事件/记录、无表查询/无 CAS/无 guard——"绕过集中迁移入口"反模式）对三份行为向量全部翻 fail，corrupted 语料 11→**12**，CI self-check 断言 ≥12；静态 CAS/迁移表比较器门退役（被行为执行取代）；`tools/static_check.py` 无需改（schema/向量计数不变）。
- **任务 3 — M2 出口评审**（本 handoff 同批）：`docs/checkpoints/20260720-m2-milestone-review.md`——DEVELOPMENT-PLAN M2 五判据 + 预算范围项逐条对照（证据 = KRN PR #4 逐判据测试名 + 本批行为向量执行记录），全过 → **M2 done，GO → M3**；判据 5 如实注记 M2 行为侧完成、disk-full 模式归 M4。
- **文档联动批**（本 handoff 所在提交）：PROGRESS（M2 done、五态 31/50、行为执行口径、车道表、handoff 列表）、findings-ledger（F-005 → verified-by-vector（M2 行为侧）；F-008 M2 行为子集落档；F-012 task 状态机行为证据；F-001 证据推进）、M2 review、本 handoff。

## 2. 未完成 / 进行中

- 50 份 not-run 行为向量消化路径：M3（context/治理链：security-negative 5、context-semantic 4 等）→ M4（effect-recovery 7、幂等、state-store-degradation 完整行为侧）→ M5+（shell-intent 8、management 8、memory/discovery/catalog/semantic 11、agent-installation 4、harness-loop 2、cim 1）。
- 行为执行模式当前只接 M2 内核公开 API；M3 治理链落地后 security-negative/context 类向量的行为执行需要 context/capability 运行时（届时按同纪律扩展 behavior 模块）。
- matrix 未在本批回填（REQ-GW-002/REQ-INTENT-ACCEPT-001 的 impl 归 M5 运行时；行为向量证据由 M2 review 承载）。

## 3. 测试与证据状态

- Rust：workspace 全绿（cargo build/test/clippy -D warnings/fmt --check；conformance 3 单元 + 7 集成，全 workspace 113 tests：106 KRN 基线 + 7 CFR 集成）。
- runner（本地实测）：81 枚举 / **31 pass**（28 静态 + 3 行为）/ 0 fail / **50 not-run**；行为执行记录含 `implementation` = "cognitive-kernel TransitionEngine + cognitive-store SqliteAuthorityStore (real authority path)" + grounding + reload 后置断言；self-check **12/12** 翻 fail。报告与 self-check digest 由 runner 打印（artifacts gitignore）。
- TS/工具：pnpm -r build/test 绿（110 测试）；check-consistency OK（273/55/60/81）；gen-matrix --check 无 drift；static_check.py ALL CHECKS PASSED；validate-manifest OK；codegen diff 空；golden byte-identical（本地对称比较）。
- CI：push + PR 后 Windows+Linux 矩阵全绿方可合并（合并事实见 PR）。

## 4. 未决风险与漂移

- 无新漂移登记；schema/向量表面零变化（60/81 钉扎不动），向量文件零改写。
- **执行边界纪律（M3+ 提醒）**：行为门只准以真实 kernel/store 公开 API 为被测路径；`audit_required`/`remote_completed_treated_as` 类合同常量字段的依据必须落 evidence（本批先例）；prose 字段记录不比对。禁止为向量造"迎合 expected 的运行时 stub"。
- 行为执行引入平台差异面（SQLite 文件系统行为）：Windows 上只读重开/WAL 均已本地验证；CI 双 OS 为最终门。
- 本机 Windows 工具链（不影响 CI）：rusqlite(bundled) 需 `CC` 指向 llvm-mingw 的 `x86_64-w64-mingw32-gcc.exe` **绝对路径**（winget MartinStorsjo.LLVM-MinGW.UCRT 包内），`AR` 指 `llvm-ar.exe`，其 bin 目录**不得**前置 PATH；dlltool-shim 仍前置（getrandom raw-dylib）。
- 教训复述（M1 批）：`git reset --hard` 前必须确认无未提交文档工作；handoff 引用哈希 → 先提交代码批再写 handoff。

## 5. 下一步入口

- **Lane-KRN M3**（入口 gate 的 M2 分量已达成）：`docs/prompts/milestone-m3.md` / `lane-krn.md`；第一动作 = F-007 行为侧测试计划（先写「capability 交集只缩不扩」与「撤销后缓存复用被拒」失败测试，KRN handoff §5）。
- **Lane-CTR 下一批**（KRN handoff §4 缺口清单待处置）：state-transition-request/record 生成绑定评估、状态投影合同/digest 域注册决策、事件 envelope 升格路径（M3 后）；错误码映射口径已落档无需动作。
- **Lane-TSC**：换生成绑定批可继续并行（与本车道无代码交集）。
- M3 治理链落地后：CFR 行为执行扩展批（security-negative/context 向量脱 not-run + F-007 行为复验）。
- 工作分支：`lane/cfr`；第一动作：`git fetch origin; git merge origin/main`，读 PROGRESS 车道表触碰通告。

## 6. 快照

- PROGRESS 已更新：是（M2 done、五态 31/50、行为执行口径、车道表、handoff 列表）。
- 本次提交：`c74c345`（行为执行 + 自检 + 钉扎 + README）→ 本 handoff 批（PROGRESS + ledger + M2 review + handoff，哈希见 git log）。基线 `3b8461b`。
