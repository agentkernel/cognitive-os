# 20260720 Lane-CFR Handoff（M1 runner 批）

## 1. 本次会话完成

按 `docs/prompts/lane-cfr.md` M1 任务与协调者任务书全数交付（分支 `lane/cfr`，基线 `b626e88` = CTR 契约批 merge）：

- **D-012 登记并闭合**（`ceb43c8`）：4 份 traceability 向量 `input.owner_spec` 滞留 informative 白皮书锚点（F-002 关闭时 registry 已改指 companion，向量 input 未同步；runner traceability-gate 执行时暴露）→ 改指 registry 机器真相；`expected` 未动、无负例删除（同 CTR 核验合法的 schema-meta-001 先例）。
- **任务 1 — runner 执行能力**（`c8cecad`；REQ-CONF-001/002/003、F-003 关闭 gate）：`exec` 模块七个确定性参考门，**只接注册机器资产**（schema 文件 / registry / 迁移表），拒绝对着向量 expected 造门：schema-gate（2020-12 校验 `input.object` 对 `validate_against`）、traceability-gate（18 份：status/owner_spec/双向 test 映射逐项闭合 + owner schema 编译与 required 清单入证据）、coverage-gate（170 REQ 成对校验）、cas-gate（REQ-STATE-003/STATE_CONFLICT）、transition-gate（effect 迁移表判定）、perf-contract-gate（性能报告合同 + 负例分支 PERFORMANCE_REPORT_INCOMPLETE）、trust-plane-gate（context-view 信任/角色约束 probe + 确定性控制面规则，DEVELOPMENT-PLAN M1 验收 4 点名的 prompt-injection 静态合同侧）。五态输出：**pass 25 / fail 0 / n-a 0 / d-d 0 / not-run 51**，pass 逐条 grounding+compared_fields+evidence，prose 字段（rejection_reasons/reason）记录不比对（口径入报告），not-run 逐条理由 + 行为归属里程碑；state-store-degradation 落 `static_contract_assertions`（F-008 静态合同侧）。
- **任务 2 — 错误实现自检**（同 `c8cecad`）：`--self-check` 以 schema-valid、行为错误实现（桥接 legacy/接受陈旧 CAS/放行 OUTCOME_UNKNOWN→COMMITTED/接受不完整收益声明/提升 untrusted 至控制面）执行 → **6/6 corrupted 向量翻 fail**，否则退出码 1；traceability/coverage 门的懒实现不可经 expected 比对观察（expected 为纪律常量），该缺口由报告不变量 + CI 断言收口并在 self-check 报告 `unobservable_gates` 落档。集成测试 5 项钉扎（分布/F-003 gate/点名向量/自检翻转/manifest 全 planned）。
- **任务 3 — F-003 关闭 gate**：GOBJ-LEGACY-METADATA-001 / GOBJ-LEGACY-STRONGREF-001 runner 真实执行 → pass；台账 F-003 → **closed-by-M1**；开放 P0（合同缺口类）= 0。
- **任务 4 — M1 复验 8 项逐条落档**：F-005/F-018 → verified-by-vector（静态合同侧）；F-004/F-012 → 无专属负例向量，如实记录（traceability 执行 + static_check fixtures 为静态证据）；F-006/F-007/F-010 → 不可静态判定，not-run 理由入报告，行为验收维持 M2/M3/M4；F-008 → 静态合同侧断言随报告落档。口径写入台账 §四。
- **任务 5 — D-004 闭合**（`7651147`）：文档化跨切片映射（不补 slug；README 钉扎映射 + runner `CROSS_SLICE_HOSTED` 常量，报告在层 7/8 呈现 hosted 明细）。
- **任务 6 — CI 演进 + tools 残留清理**（`76ae5e5`）：M0 全 not-run 断言 → 诚实性门（五态钉扎计数 76/25/0/0/0/51 + 求和守恒 + pass 必附执行证据 + not-run 必附理由）+ self-check 专设步骤；`validate-manifest.mjs` 移除 `$id` 剥离注册残留层；注入演练三组红→绿（孤儿 REQ / manifest 违约 / 报告伪造两式），全文见 M1 review §4 与 PR 描述。
- **任务 7 — M1 出口评审**（本 handoff 同批）：`docs/checkpoints/20260720-m1-milestone-review.md` 七判据全过 → **M1 done，GO → M2**；PROGRESS 里程碑表更新，tracer bullet 入口 gate 开启（M4 入口另需 M2/M3 行为验收）。
- **文档联动批**（本 handoff 所在提交）：findings-ledger（F-001/003/004/005/006/007/008/010/012/015/018、D-004/D-012、开放汇总、§四口径）、PROGRESS（里程碑/计数/车道/handoff 列表）、matrix（REQ-CONF-001/002/003 impl/impl_tests/evidence 回填 + 再生成）、M1 review、本 handoff。

## 2. 未完成 / 进行中

- 51 份行为向量 not-run 的消化：M2（state/effect 内核）→ M3（context/治理链）→ M4（恢复/fencing）→ M5+（shell/management/memory/discovery/catalog/semantic）；runner 侧已就绪（新增向量默认 not-run，防未审执行路径）。
- matrix impl 字段仅回填 REQ-CONF-001/002/003；合同层 REQ（GOBJ/PROTO/EFF-STATE 等）待 Lane-CTR 按其证据口径回填。
- F-004/F-012 无专属负例向量（已如实落档）；若 M2/M3 行为验收需要可走修正型增补。

## 3. 测试与证据状态

- Rust：49 测试通过（conformance 3 单元 + 5 集成新增；contracts 全绿）；clippy --workspace -D warnings 绿；fmt 绿。
- TS/工具：pnpm -r build/test 绿（tools 链接检查在本 handoff 文件落地后复绿）；check-consistency OK（273/55/56/76）；gen-matrix --check 无 drift；static_check.py ALL CHECKS PASSED。
- runner（本地实测）：pass 25 / fail 0 / not-run 51；报告 sha256:5eb3150b0388280819b8bc53c26a94c7f994b61f63c07619b38e640fd3ffabf0；self-check 6/6 翻 fail，报告 sha256:b9f0c1bea38c6a779fd87dbc472993fce4fc693f5c5100ae91d41f34cb2d5b26（`artifacts/evidence/conformance/`，gitignore，runner 打印 digest）。
- CI：推送 + PR 后须 Windows+Linux 矩阵全绿方可合并（合并事实见 PR）。

## 4. 未决风险与漂移

- **执行边界纪律**：M1 静态门刻意收窄——只执行可从注册机器资产判定者；诱惑是给行为向量写"迎合 expected 的迷你门"（tautological pass），已用"grounding 必须指向注册资产"+ 分类白名单（singleton 向量 id 钉扎）挡住；后续里程碑加行为执行时保持该纪律。
- 钉扎计数（CI 76/25/0/0/0/51 与集成测试）在向量增补/能力扩展时须同批调整——这是有意的红灯设计（IMP-17）。
- 本机 Windows 工具链坑复述（不影响 CI）：dlltool shim 前置 PATH（jsonschema→getrandom raw-dylib）；PowerShell 双引号内 `$id` 会被变量展开（本批一处提交信息因此重写——单引号规避）；禁 `>` 重定向写 JSON。
- serde_yaml（archived）保留决策：registry 只读面小且稳定，M1 复审通过；YAML 面扩大时再换。

## 5. 下一步入口

- **Lane-KRN M2 入口 gate 已开**：`docs/prompts/milestone-m2.md` / `docs/prompts/lane-krn.md`；工作分支 `lane/krn`（从合并后 main 新建 worktree）。
- M2 行为验收（CAS 并发、非法迁移穷举、重放 digest、事件不可变、STATE_STORE_UNAVAILABLE fail-closed）落地后：runner 加行为执行模式，对应向量脱 not-run；F-005/F-012 行为侧复核随 M2。
- 第一个动作（KRN）：`git fetch origin; git worktree add ../agent-kernel-krn -b lane/krn origin/main`，读 PROGRESS 车道表触碰通告（conformance crate 归 CFR，内核行为接口经 Lane-CTR 契约流程）。

## 6. 快照

- PROGRESS 已更新：是（M1 done、五态实测、P0=0、D-004/D-012 闭合、车道表、handoff 列表）。
- 本次提交：`ceb43c8`（D-012）→ `c8cecad`（runner + 自检 + 测试）→ `76ae5e5`（CI/tools）→ `7651147`（D-004/README）→ 本 handoff 批（ledger + PROGRESS + matrix + M1 review + handoff，哈希见 git log）。
