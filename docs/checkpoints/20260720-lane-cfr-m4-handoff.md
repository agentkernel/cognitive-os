# 20260720 Lane-CFR Handoff（M4 故障注入向量执行批）

## 1. 本次会话完成

按协调者任务书（M4 故障注入向量执行批 + M4 出口评审）交付（分支 `lane/cfr`，基线 `bae94b4` = KRN M4 批 merge，含 CTR 评估批 `03ccb51`；KRN 对 behavior.rs 的 3 处 `fencing_epoch: None` 机械补丁已核对无行为影响）：

- **任务 1 — effect/recovery 向量行为执行**（`0990ecd`；REQ-EFF-002/004/006、REQ-RUN-006、REQ-AGENT-RECOVERY-001 证据）：新增 `exec/behavior_m4` 模块七个行为门，被测 = `cognitive_kernel::{effects, recovery}` + `SqliteAuthorityStore`，故障注入复用**公开** `cognitive_store::faults`（CrashHarness drop-and-reopen + ScriptedExecutor 台账）。KRN 候选清单逐项清点结果（全部可行，全部脱 not-run）：
  - EFF-CRASH-001 → **pass**（crash 后恢复处置 = ReadyToRedispatchOriginalKey，原键单次重发，台账证明无重复副作用；恢复八步序钉扎）；
  - EFF-CRASH-002 → **pass**（EXECUTING→OUTCOME_UNKNOWN→RECONCILED 通道从已提交事件链读回；原键 query 对账；无盲重试）；
  - EFF-CRASH-003 → **pass**（从证据 commit、外部动作零重执行；固定后态漂移负例孪生 → 拒绝文本点名 `verification_still_current` → verification_still_current_check_performed 实测）；
  - RECOVERY-CRASH-006 → **pass**（三场景聚合，各自 fresh 库，audit 链闭合实测）；
  - EFF-UNK-003 → **pass**（opaque 幂等 sink ExecuteThenTimeout → query Indeterminate → QUARANTINED + EFFECT_OUTCome_UNKNOWN 实测——见 §4 拼写勘误，恰一次外呼）；
  - EFF-IDEM-CONFLICT-001 → **pass**（同键异 canonical 参数 → EFFECT_IDEMPOTENCY_CONFLICT；耐久行 digest 未改写、既有 effect 状态零变化 reload 实测；input 占位 digest 如实注记）；
  - AGENT-RECOVERY-003 → **pass**（在途 Effect 对账/隔离先于 checkpoint 校验；error_code 映射依据 = 事件链观察到 OUTCOME_UNKNOWN 通道 + 注册码语义，终局处置码 EFFECT_RECOVERY_QUARANTINED 落 evidence）。
  - `state-store-degradation`：新增 **m4_behavioral_fencing_subset 真实执行**（陈旧 epoch 写在 store 提交 sink 事务内被拒、当前 epoch 可提交）；disk-full 分量如实维持 deferred（F-008 口径不变），not-run 理由改为专属文本（三层子集落档说明）。
- **任务 2 — 钉扎与自检**（同 `0990ecd`）：五态 81/**46**/0/0/0/**35**；反模式错误实现五种（换键重铸并双发、unknown 盲重发、commit 恢复期重执行、冲突当去重、未对账即恢复 loop——可行处真实驱动 store/executor）→ corrupted 语料 20→**27** 全部翻 fail（含修一处自检盲点：crash-recovery 聚合向量的 wrong 分支最初未真发原键导致无重复副作用、比对面恰好不覆盖——改为「处置照发 + 换键再发」后聚合向量正确翻 fail）；ci.yml + runner_execution.rs 9 测试同批红→绿；static_check 无需改（面不变）。
- **任务 3 — 台账升级**（本 handoff 同批）：F-006/F-010 → **verified-by-vector（M4 行为侧）**；F-014/F-023 → **closed-by-M4**（评审判据 6/8 确认；**F-023 拒绝码 `NO_AUTHORIZED_OPERATION_CANDIDATE` 正式确认**，依据落台账）；F-008 补 M4 fencing 子集（disk-full deferred 维持 + 复议条件）；F-001 证据推进（行为执行 19 向量 / 内核行为测试 93 / tracer bullet 链）。
- **任务 4 — M4 出口评审**（本 handoff 同批）：`docs/checkpoints/20260720-m4-milestone-review.md`——八判据全过；**tracer bullet 复现确认**（`cargo test -p cognitive-store --test m4_tracer_bullet` 本地再执行 + 证据工件再生成，sha256 落评审）；**M5 入口 gate 如实判定：M4 分量达成，F-011 R1 审批合同登记为唯一剩余项（归 Lane-CTR）**。
- **文档联动批**：PROGRESS（M4 done、五态 46/35、P1 开放降至 2、车道表、handoff 列表）、findings-ledger、M4 review、本 handoff。

## 2. 未完成 / 进行中

- 35 份 not-run 主体挂 M5+：shell-intent 8、management 8、harness-loop 2、DISC-DELTA-SCOPE-003、F-004 运行时 admission、state-store-degradation（disk-full deferred + 管理面分量）、agent-installation 3、memory 4、discovery 2、catalog 2、semantic 1、knowledge/embodied/CIM 4。
- M5 落地后 CFR 下一批：shell/management/harness 向量行为执行 + F-011 R1 负例向量执行 + crash-recovery 复跑覆盖跨 activity 重授权分支（KRN M4 handoff §4.3）+ M5 出口评审。
- matrix 本批未动（M4 REQ impl 已由 KRN 回填；向量行为证据由 M4 review 承载）。

## 3. 测试与证据状态

- Rust：workspace 全绿（conformance 3 单元 + 9 集成；KRN 147 基线不动）；clippy -D warnings / fmt 绿。
- runner（本地实测）：81 枚举 / **46 pass**（静态 27 + 行为 19）/ 0 fail / **35 not-run**；self-check **27/27**；tracer bullet 复现 + 证据工件 sha256:b09431a7…（评审 §2 判据 7）。报告/self-check digest 由 runner 打印。
- TS/工具：pnpm -r build/test 绿；check-consistency OK（273/55/60/81）；gen-matrix --check 无 drift；static_check.py ALL CHECKS PASSED；validate-manifest OK；codegen diff 空；golden byte-identical（验证记录见 PR）。
- CI：push + PR 后 Windows+Linux 矩阵全绿方可合并（合并事实见 PR）。

## 4. 未决风险与漂移

- 无新漂移登记；schema/向量面零变化（60/81 钉扎不动），向量文件零改写。
- **AGENT-RECOVERY-003 的 error_code 映射口径（接续者知悉）**：向量期望 `EFFECT_OUTCOME_UNKNOWN`（迫使「对账先于恢复」的条件码），恢复的终局处置码为 `EFFECT_RECOVERY_QUARANTINED`——本批以事件链实测的 OUTCOME_UNKNOWN 通道 + 注册码语义发射期望码，终局码落 evidence（两码均注册；映射依据入报告 `error_code_mapping`）。若 M5+ 认为应改判为向量修正（期望改为终局码），走 ledger 修正流程，勿静默改。
- 本 handoff §1 中 "EFFECT_OUTCome_UNKNOWN" 为本文档一处笔误（正确 = EFFECT_OUTCOME_UNKNOWN），报告与代码无此拼写。
- 自检盲点教训（M4 新记）：聚合类向量的 wrong 分支必须确认其**比对面**（scenario_results 只含 action/duplicate_effect）真被扰动——先跑 self-check 看 flip 清单再定稿。
- 磁盘运维照旧：本批开工时 D 盘仅 0.81GB，清 ctr/krn target 后 5.68GB；`TMP=D:\tmp`；llvm-mingw CC/AR 绝对路径 + dlltool-shim。

## 5. 下一步入口

- **Lane-CTR F-011 登记批（M5 入口唯一剩余项，可立即启动）**：R1 聊天内结构化确认最低集——approval-request/approval-decision 机器 schema 硬化 + 负例向量（缺确认强执行、自批、疲劳批准防护）+ registry 映射 + matrix；范围建议见 M4 review §7 与 F-011/IMP-05 台账。同批候选：无（D-016 维持 defer；membership/D-018 实施等 M5 消费方）。
- **Lane-KRN/RUN M5**（F-011 登记后）：意图链/Harness/Shell/管理面（KRN M4 handoff §5 kernel 剩余面 + RUN 的 envelope 组装器/HTTP+SSE）。
- 工作分支：`lane/cfr`；第一动作：`git fetch origin; git merge origin/main`，读 PROGRESS 车道表触碰通告。

## 6. 快照

- PROGRESS 已更新：是（M4 done、五态 46/35、P1=2、车道表、handoff 列表）。
- 本次提交：`0990ecd`（M4 行为门 + fencing 子集 + 自检 + 钉扎 + README）→ 本 handoff 批（ledger + PROGRESS + M4 review + handoff，哈希见 git log）。基线 `bae94b4`。
