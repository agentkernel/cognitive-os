# 20260720 M1 Milestone Review

## 1. 范围回顾

M1 = 合同收敛与符合性 Runner（`docs/plan/DEVELOPMENT-PLAN.md` M1 节）。交付分两批：

- **Lane-CTR 契约批**（PR #1，merge `b626e88`）：F-003 收尾（负例向量 + 双语言合同测试 + legacy `$defs` 保留决策）、D-001/D-006 `$id` 统一、ADR-0006 codegen 管线（IMP-08 A.1 14 对象双语言绑定 + CI regenerate-diff 门）、注册式 bundle digest（§13，D-011）、golden §14 全覆盖。
- **Lane-CFR runner 批**（本评审所在 PR）：runner 静态合同执行能力（五态输出）、错误实现自检、F-003 关闭 gate、M1 复验 8 项逐条落档、D-004/D-012 闭合、CI 断言演进。

## 2. 验收判据逐条对照

判据 1–6 = DEVELOPMENT-PLAN M1 验收；判据 7 = `docs/prompts/milestone-m1.md` 出口评审产出义务。

| # | 判据 | 结果 | 证据 |
|---|---|---|---|
| 1 | F-003 关闭：全仓无 legacy metadata/strongRef 双轨引用；56 schema 全过 2020-12 元校验与 `$ref` 解析 | **通过** | 双轨引用禁令：`python tools/static_check.py` 检查 5 绿（零引用）+ `crates/cognitive-contracts/tests/schema_contract.rs::legacy_defs_stay_deprecated_and_unreferenced`；元校验与 `$ref`：`node tools/src/check-consistency.mjs` 绿（56 schema 编译、`$id`==文件名红灯生效）；**关闭 gate**：runner 真实执行 GOBJ-LEGACY-METADATA-001 / GOBJ-LEGACY-STRONGREF-001 → **pass**（schema-gate 拒绝双轨形态，SCHEMA_MISMATCH 对 registry，执行记录含逐条校验错误），钉扎测试 `runner_execution.rs::f003_legacy_negatives_are_executed_and_pass`；台账 F-003 → closed-by-M1 |
| 2 | runner 对全部向量输出五态结果；**故意错误实现被判 fail** | **通过** | 参考实现运行：76 向量全部落入五态（pass 25 / fail 0 / not-applicable 0 / documented-degradation 0 / not-run 51），机器报告 `artifacts/evidence/conformance/conformance-report.json`（本批产出 sha256:5eb3150b0388280819b8bc53c26a94c7f994b61f63c07619b38e640fd3ffabf0，runner 每次打印实际 digest）+ 人读摘要；自检：`conformance-runner --self-check` 以 schema-valid、行为错误的实现（桥接 legacy 形态/接受陈旧 CAS/放行 OUTCOME_UNKNOWN→COMMITTED/接受不完整收益声明/提升 untrusted 至控制面）执行 → **6/6 corrupted 向量全部翻 fail**（self-check-report.json sha256:b9f0c1bea38c6a779fd87dbc472993fce4fc693f5c5100ae91d41f34cb2d5b26；钉扎测试 `runner_execution.rs::wrong_implementation_is_failed_by_the_runner`；CI 专设步骤断言） |
| 3 | 未实现层保持 not-run，无一虚报 | **通过** | 51 份需内核/运行时行为的向量全部 not-run 且逐条携带 `not_run_reason`（含行为归属里程碑）；CI 诚实性门断言：pass/fail 必附执行记录（grounding + compared_fields≥1）、not-run 必附理由、五态计数钉扎（76/25/0/0/0/51）且求和=总数；注入演练证明伪造 pass 被拒（见 §4） |
| 4 | M1 复验项负例向量全部执行（含 `effect-state-closure-008`、`prompt-injection-isolation`、`state-store-degradation` 的静态合同侧断言） | **通过（按 DEVELOPMENT-PLAN 括注口径，逐条落档）** | 点名三向量：effect-state-closure-008 → **pass**（transition-gate，注册迁移表判定）；prompt-injection-isolation → **pass**（trust-plane-gate，schema probe + 确定性控制面规则）；state-store-degradation → 静态合同侧断言随报告落档（错误码 fail-closed 描述 + `intent_durably_persisted` guard 在表），向量本体如实 not-run（故障注入行为=M2/M4）。8 项逐条：F-005/F-018 → verified-by-vector（静态合同侧）；F-004/F-012 → 无专属负例向量（如实记录），traceability 执行 pass + static_check 正负 fixtures 为静态证据；F-006/F-007/F-010 → 不可静态判定，not-run 理由入报告，行为验收维持 M2/M3/M4 挂载（台账 M0 起即如此规划）。台账 §四 复验口径节 + 逐条目更新 |
| 5 | codegen 再生成 diff 为空（CI 钉住） | **通过** | CI `Regenerate schema bindings and diff` 步骤（Lane-CTR 交付）在本 PR 上必须绿方可合并；本地 `cargo run -p cognitive-contracts --bin contracts-codegen` + `git diff --exit-code` 验证为空 |
| 6 | registry↔schema↔vector 双向无孤儿保持绿 | **通过** | `pnpm run check:consistency` 绿（273 REQ / 55 错误码 / 56 schema / 76 向量，双向闭合、链接、matrix、台账覆盖全查）；`gen-matrix --check` 无 drift |
| 7 | 出口评审文档产出 | **通过** | 本文档 |

## 3. 安全负例清单（M1 执行）

真实执行且 pass 的安全/拒绝类负例（全部为静态合同执行，非行为证明）：

- GOBJ-LEGACY-METADATA-001 / GOBJ-LEGACY-STRONGREF-001（双轨治理对象形态必须被拒，F-003）；
- STATE-CAS-002（陈旧 CAS 写必须被拒，STATE_CONFLICT）；
- EFFECT-STATE-CLOSURE-008（OUTCOME_UNKNOWN→COMMITTED 非法迁移必须被拒，EFFECT_OUTCOME_UNKNOWN，F-005）；
- CTX-TRUST-004（prompt 注入内容不得进入控制面/铸造 capability，F-018）；
- PERF-REPORT-CONTRACT-001 负例分支（缺 native/governance-only 臂与预注册的收益声明必须被拒，PERFORMANCE_REPORT_INCOMPLETE）。

错误实现自检 = 以上负例的反向证明：桥接/放行这些拒绝路径的实现被 runner 判 fail（6/6）。

## 4. 注入演练记录（docs-sync-contract §5，检查器被修改的义务）

本批修改了 `tools/src/validate-manifest.mjs`（移除 `$id` 剥离兼容层）与 CI 断言脚本，注入演练三组（红→还原→绿）：

1. **孤儿 REQ 引用**：向 `docs/plan/PROGRESS.md` 注入 registry 不存在的 REQ 编号（DRILL 域 999 号）→ `check-consistency: 1 violation(s)`，逐条指出文件与原因 → 还原后绿。
2. **manifest 违约**：删除样例 manifest 的 `spec` 必填节 + 写入非法 profile 值 `totally-conformant` → `validate-manifest: INVALID`（两条 ajv 错误：required 'spec'、enum 违例）退出码 1 → 删除注入副本后绿。
3. **报告伪造**：①把 not-run 向量改写为 pass 并加计数 → 钉扎计数断言红（`summary.pass = 26, pinned 25`）；②保持计数守恒的换牌伪造（真 pass 改 not-run、not-run 伪造成 pass）→ 逐向量证据断言红（`STATE-STORE-DEGRADE-001 reported pass without execution evidence`）→ 重跑 runner 再生成报告后绿。

## 5. 漂移与规范变更

- **D-004 闭合**（文档化跨切片映射）：层 7/8 不补专属 slug；钉扎映射入 `conformance/README.md` 与 runner `CROSS_SLICE_HOSTED`，报告在层 7/8 显示 hosted 明细。修正型变更。
- **D-012 登记并闭合**：4 份 traceability 向量 `input.owner_spec` 滞留 informative 白皮书锚点（F-002 关闭时未同步），改指 registry 机器真相；`expected` 未动、无负例删除（同 schema-meta-001 先例）。修正型变更。
- 向量/REQ/错误码/schema 表面零新增（IMP-01 冻结遵守）；`conformance/README.md` Running 节与层映射节更新为 runner 事实。

## 6. 指标快照

- REQ：273 specified / 0 REQ 级实现声明 / 行为已执行 0（**静态合同层 25/76 向量 pass**，逐条 grounding+evidence）/ Profile 13 全 planned。
- 向量五态（实测）：**pass 25 / fail 0 / not-applicable 0 / documented-degradation 0 / not-run 51**；自检 6/6 fail。
- 开放 P0：0（合同缺口类）；F-001 为证据缺口性质，随 M2~M6 消解。开放 P1：F-011/F-014/F-023/F-017（+F-015 持续）。开放漂移：0。
- CI：本 PR 合并即证明 Windows+Linux 矩阵全绿（合并前置条件）。

## 7. 结论

**GO → M2**（M1 done）。**tracer bullet 入口 gate（F-002~F-010 类合同收敛）开启**——注意 M4 入口按 DEVELOPMENT-PLAN 另需 M2/M3 行为验收 + F-014/F-023 排入。遗留条件（不阻断 M2）：

1. F-004/F-012 无专属负例向量——如实落档；若 M2/M3 行为验收需要，可按 docs-sync-contract 增补（修正型路径）。
2. 行为向量 51 份 not-run 的消化路径：M2（state/effect 内核）→ M3（context/治理链）→ M4（恢复/fencing）→ M5+（shell/management/memory/discovery/catalog/semantic）。
3. matrix impl/impl_tests 仅回填 REQ-CONF-001/002/003；合同层 REQ（GOBJ/PROTO 等）的 impl 字段待 Lane-CTR 按其证据口径回填。
