# Post-v0.1 / Post-L3 下一阶段开发与调试测试任务计划

- 状态：active（2026-07-22）；`V02-CA-GOV-00` 已合入，`V02-CA-OPS-01` 已 [materialize 为 owner-review 设计包](V02-CA-OPS-DESIGN-DECISION.md)（[ADR-0010](../adr/0010-v02-management-operation-set-governance.md)）；八项 operation candidates 全部 blocked，set digest unresolved/not computed，四类 machine contracts 未登记，D-022 与 CA-1～CA-8 blocker 不变；pins **59/25**；self-check **40**；类别 plan（informative）
- 承接：[20260721-v01-rereview.md](../checkpoints/20260721-v01-rereview.md)（GO-with-explicit-non-claim）+ [20260721-v01-auto-run-l3-handoff.md](../checkpoints/20260721-v01-auto-run-l3-handoff.md)（L3 non-claim）
- 对齐：[DEVELOPMENT-PLAN.md](DEVELOPMENT-PLAN.md)、[V01-AUTO-RUN-VERIFY-PERF-PLAN.md](V01-AUTO-RUN-VERIFY-PERF-PLAN.md)、[V01-PERF-CAMPAIGN-PLAN.md](V01-PERF-CAMPAIGN-PLAN.md)（附录；默认不触发）、[findings-ledger.md](../traceability/findings-ledger.md)
- RUN handoff：[20260722-lane-run-shell-target-ambiguity-handoff.md](../checkpoints/20260722-lane-run-shell-target-ambiguity-handoff.md)
- CFR handoff：[20260722-lane-cfr-shell-target-ambiguity-handoff.md](../checkpoints/20260722-lane-cfr-shell-target-ambiguity-handoff.md)
- 下一唯一门禁：owner review/merge 独立 docs-only OPS PR；合入后进入 TARGET design，承接三个 configure candidates 的 target authority/consumer/readback/receipt closure；SIG/AUDIT 仍须独立设计；勿启动 CA 实现、勿批量清 not-run、勿开 PERF；**隔离 PR #36（M7 plan）**
- 更新责任：主战役合入或候选优先级变更时同批更新本文件与 [PROGRESS.md](PROGRESS.md)

## A. 阶段目标与边界

### 阶段目标

在 `origin/main@41fce4d`（PR #50 merge）与 pins **59/25** 之后，OPS 设计包已落盘等待 owner review；下一设计入口仅在 OPS 合入后转为 TARGET。本文件历史 P0 Intent、shell-channel 与 shell-target 批均已交付，不得再规划为 P0；Configuration Authority 尚未登记机器合同或解除实现门禁。

本阶段的成功定义是「实现已提供 + 测试已执行 + 相关向量 pass 的窄面证据」，不是 Profile implemented、跨平台安全符合、完整 M5/M6 或 v0.2 发布。

### 当前事实地板

- tip：以 `origin/main` 实测为准（规划会话起点曾为 `63617a0`；RUN merge `8e57e6d`；CFR 合入后继续前进）。
- conformance：84 vectors，**59 pass**，**25 not-run**，fail/not-applicable/documented-degradation 为 0。
- self-check：**40** 个 corrupted vectors 必须翻 fail；不得降低地板。
- Profile implemented：0；自动绿灯不改变该状态。
- V01 canonical run：`20260721-192142-492`，`level=L3`，`release=non_claim_preserved`，`platform_label=windows_wsl2_linux_guest`。
- 工作树旁路 dirty（skills / 职校选型 / `artifacts/_local/**` 等）——执行窗口必须保护、忽略、不暂存、不回退。

### In-scope

- Lane-CFR：增加针对已存在确定性实现的窄面行为执行和错误实现自检。
- 真实 vector：`INTENT-SUPERSEDE-002`、`INTENT-ACCEPTANCE-007`；分别绑定 `REQ-INTENT-SUPERSEDE-001` / `REQ-INTENT-ACCEPT-001`，并保留既有附属 REQ 映射。
- 必要的 runner 测试、证据输出、traceability matrix 回填和 handoff；如行为发现真实规范漂移，先登记 findings-ledger，再按 docs-sync-contract 最小修正。
- 工具链调试记录：仅处理影响该主战役的 WSL 原生盘、Windows GNU linker、`m5_http_sse` 并发 flake；不顺手扩大编排器范围。

### Out-of-scope / 继承 non-claims

- Windows-native sandbox unsupported；WSL2 guest sandbox 仍为 not_tested。本战役不能扩大 F-017 声明集。
- durable InstallationStore；当前 installation ledger 仍为 in-process，不得称 durable install authority。
- D-018 governance object ports；当前仅 partially-implemented，事件发布组装器已有但治理对象持久化/解析端口仍缺。
- PERF-004 full HW campaign、PERF-005 benefit；默认不触发 `HUMAN-PERF004-CAMPAIGN` / `HUMAN-PERF005-CLAIM`。
- 不批量清空 25 not-run，不改 vector expected、不删除安全负例、不降低 84/59/25 或 self-check 40。
- 不发明 `kernel-server` 的 `--data-dir`、`/health`、`/ready`；tip 仍只允许 `--once --bind` 参考面。
- Console、clients、Agent Hub、M7+；Lane-CON 继续 tracking-only，不启动实现脚手架。
- 不修改 `personal-blog/`，不把它纳入 Cos 交付。

## B. 候选工作流排序

### P0：主战役

**P0-1：Lane-CFR M5 Intent Authority 窄面行为批**

选择：执行 `INTENT-SUPERSEDE-002` 与 `INTENT-ACCEPTANCE-007`，仅针对已存在的真实 KRN/store 行为和确定性 acceptance gate 接入 runner。

理由：

- `INTENT-SUPERSEDE-002` 的实现已在 `crates/cognitive-kernel/src/intent_chain.rs`、`effects.rs`、`crates/cognitive-store/src/sqlite.rs`；已有 `m5_intent_chain.rs` 证明新 epoch、pending effect 的 `reconcile_before_continue`、旧 epoch dispatch 的 `INTENT_VERSION_SUPERSEDED` 和零 sink dispatch。
- `INTENT-ACCEPTANCE-007` 已有真实 task transition/acceptance gate 承载；`crates/cognitive-conformance/src/exec/behavior.rs` 已行为执行其关联的 `GW-REMOTE-COMPLETE-001`，但 vector 自身仍 not-run，应先做 vector-specific adapter/比较，不把 generic 行为证据冒充该 vector。
- 两项均属于同一条「意图修正/验收 authority」语义链，主实现车道保持 CFR，依赖已合入的 KRN/RUN，不需要新增对象族、Profile、REQ 域或错误码。
- 预计变更车道：CFR 主体；KRN/RUN 仅在失败证据证明已有实现缺口时另开依赖任务，禁止在 CFR 批中跨车道修业务 crate。

### P1：下一批或协作批

**P1-1：窄面脱 not-run：shell channel isolation**

- vector：`SHELL-CHANNEL-ISOLATION-003`。
- 真实已有 TS client channel-binding 测试和 `SHELL_CHANNEL_BINDING_MISMATCH`；但 runner 是 Rust leaf，跨语言执行需要明确 CFR↔TSC 证据接口，不能只复制 SDK 单测。
- 车道：TSC 实现/测试 + CFR runner 协作；先确认是否能以现有 HTTP/AKP 面取得 authority denial，再决定是否纳入同一批。

**P1-2：窄面脱 not-run：shell target ambiguity**

- vector：`SHELL-TARGET-AMBIGUITY-001`，REQ `REQ-SHELL-TARGET-001` / `REQ-SHELL-AMBIGUITY-001`。
- KRN intent chain 已有 material ambiguity → `clarification_required`、`INTENT_CLARIFICATION_REQUIRED` 的确定性测试，但该 vector 期待 shell target selector `stop it` → `SHELL_TARGET_AMBIGUOUS` 且 dispatch=false，仍需确认 target-resolution API 与错误码路径是否真实存在。
- 车道：TSC/RUN/KRN 先确定承载，CFR 后接入；不得用 intent ambiguity 单测替代 shell target vector。

**P1-3：HUMAN-CI-JOB-ADD**

- 价值：让 `verify:local` 的等价步骤在 CI 可重复运行，减少本地/CI 编排漂移。
- 成本：当前 CI 已有 verify matrix、runner pins/self-check/evidence 上传；新增独立 job 会增加运行时、并发和证据重复，且 V01 仍要求平台标签和 non-claim 人闸门。
- 判定：P1 governance/tooling，不能成为本主战役的依赖；只有明确 CI 资源/权限和重复步骤收敛方案后才立项。默认保留 HUMAN gate，不写成已提供。

**P1-4：编排器/工具链调试债**

- `/mnt/d` 9p 慢且 CONNECT 易抖；优先 WSL 原生盘副本。
- Windows GNU linker 可能缺 `libgcc`；按既有 handoff 的环境准备执行，不把环境 workaround 写成产品能力。
- `m5_http_sse` 已使用 `--test-threads=1` 降低偶发 `ConnectionReset`；若主战役复现 flake，只做证据分类/重试边界审计，不以无界重试掩盖失败。
- 判定：P1 旁路 debug package，可随主战役执行，但不得改 pins 或扩大 V01。

### P2：有条件推进

**P2-1：D-018 governance object ports（Lane-KRN）**

- 当前状态是 partially-implemented，不是开放 P0；已交付 `cognitive-runtime::event_envelope::assemble_event`，仍缺治理对象持久化/解析端口以提供 owner/authority/resource_scope/policy_refs 等 strong refs。
- 与 v0.1 non-claim 的关系：闭合可减少 D-018 exchange-surface non-claim，但不会自动把 Profile 变为 implemented，也不解除 F-017/Windows/durable/PERF non-claims。
- 判定：P2。需先给出 KRN 端口契约和失败测试，且不得在 CFR 主战役跨车道实现；若实现反馈要求新 wire/schema/REQ，应按语义/结构型 docs-sync 重新评审。

**P2-2：durable InstallationStore（KRN+RUN）**

- 当前 `crates/cognitive-runtime/src/installer.rs` 是 `InstallationLedger` in-process，且 D-020 明确不新增 installation transition table；需要 durable authority store、崩溃恢复、事务可见性与跨进程证据。
- 判定：P2/defer。工作量和风险显著大于本阶段窄面；其完成也不应被写作「Profile implemented」。触发条件是产品明确要求 durable install authority，并先完成 KRN/RUN 接口设计、失败注入和持久化恢复计划。

**P2-3：Lane-TSC proposal/preview/submit HTTP 增量**

- 现有 RUN `ShellService` 已有 proposal/preview/submit（submit 是 receipt-level、authority=false），`kernel-server` 已有 detach/cancel/attach/watch 的 `--once` 参考路由；完整 proposal/preview/submit HTTP 语义仍依赖 RUN 扩展。
- 判定：P2，除非明确需要真实客户端联通验收。若启动，顺序必须是 RUN HTTP contract/negative tests → TSC client → CFR live/evidence；客户端永远不写 authority。

**P2-4：F-017 / WSL2 guest 实测扩声明**

- 当前 F-017 仅 closed-for-release-claim-set；WSL2 guest run 只证明 Linux guest 编排，不是 WSL2 sandbox 证据。
- 判定：defer，需人闸门、平台矩阵新增 digest、重开 F-017 后才能做；默认不触发。

### defer

**defer-1：PERF-004 campaign / PERF-005 benefit**

- [V01-PERF-CAMPAIGN-PLAN.md](V01-PERF-CAMPAIGN-PLAN.md) 已明确为 informative appendix，默认不执行。
- PERF-004 触发需 HUMAN-PERF004-CAMPAIGN、L2 green、硬件拓扑/并发预注册和 campaign digest；PERF-005 还需 M7+ 四臂 harness、BenchmarkManifest、独立 verifier。当前不得把 sample/builder 写成 campaign 或 benefit。

**defer-2：MGMT-FALLBACK-008 全族（OPS design materialized；machine registration / implementation blocked）**

- vector `MGMT-FALLBACK-008` 对应 `REQ-MGMT-FALLBACK-001`，要求 session.create_restricted、status.inspect、capability.revoke、execution.stop、effect.reconcile、gateway.configure、diagnostics.configure 七个 operations 全可达。
- `V02-CA-OPS-01` 已把五个 intended-core 与三个 intended critical-extension candidates 的设计落盘，但八项全部 blocked，没有 design-approved 或 machine-registered member；下一门禁是 owner review/merge，之后由 TARGET 承接三个 configure candidates。四类合同合入且 CA-0 re-review GO 前不得实现或执行该 vector，不能在 CFR 中硬编码「全可达」。

**defer-3：store-degradation disk-full**

- vector `STATE-STORE-DEGRADE-001` / `REQ-REC-003` 仍因 portable disk-full 注入和 management stop/revoke/audit expectations 缺口保持 not-run；已有 read-only degradation 与 fencing subset 是 partial evidence，不是 vector pass。
- 判定 defer，除非出现可移植 fault injector 和完整 management authority path。

**defer-4：SHELL-EXECUTION-MIGRATION-009 / DISC-DELTA-SCOPE-003**

- migration vector 涉及 distributed runtime migration、stable execution identity、source fencing、watch continuity；不属于当前单节点 R0/R1 窄战役。
- delta vector 对应 M7/discovery/CRB 路径，当前 runner 已明确无 kernel API，保持 not-run。

**tracking-only：Console / clients / Agent Hub**

- 继续 Lane-CON informative tracking；implementation-ready 仍 blocked 于 PoC、技术栈 ADR、依赖组 1/2/7、法务 gate 等条件。不改实现、不造 mock、不把目录存在写成实现已提供。

## C. 推荐下一战役

### 主战役

**战役名：`CFR-M5-INTENT-AUTHORITY-SLICE`**

**目标：**把 `INTENT-SUPERSEDE-002` 与 `INTENT-ACCEPTANCE-007` 从 not-run 接入真实确定性 KRN/store 行为执行，生成可审计 conformance evidence，并让 deliberately-wrong implementation 对两者翻 fail。

**非目标：**不闭合整个 M5；不执行 MGMT fallback 全动词、shell channel/target、migration、store-degradation、PERF、F-017 扩表、D-018、InstallationStore、Console；不新增/修改 vector expected、REQ、错误码、schema、transition table。

**车道与分支：**Lane-CFR；建议 `lane/cfr-m5-intent-authority-slice`。依赖的 KRN/RUN 代码只读消费；若发现业务实现缺陷，停止并另开所属车道分支，不在 CFR 分支越权修复。

**执行提示词：**[cfr-m5-intent-authority-slice.md](../prompts/cfr-m5-intent-authority-slice.md)

### 工作包

| ID | 标题 | 类型 | 依赖 | 涉及路径 | 关联资产 |
|---|---|---|---|---|---|
| IA-0 | 接入与资产基线 | debug/test | 无 | `AGENTS.md`、`docs/plan/PROGRESS.md`、最近 handoff、`docs/plan/PARALLEL-LANES.md`、`git status` | REQ/向量事实以 registry 核对 |
| IA-1 | Supersede vector 行为执行 | dev + test | IA-0 | `crates/cognitive-conformance/src/exec.rs`、新增/扩展 `exec/behavior_m5.rs`、`crates/cognitive-kernel/**`、`crates/cognitive-store/**`（只读消费） | `INTENT-SUPERSEDE-002`; `REQ-INTENT-SUPERSEDE-001`, `REQ-SHELL-CORRECTION-001`, `REQ-AKP-INTENT-001`; error `INTENT_VERSION_SUPERSEDED` |
| IA-2 | Acceptance vector 行为执行 | dev + test | IA-0；可复用既有 task acceptance behavior | `crates/cognitive-conformance/src/exec/behavior.rs` 或 M5 behavior 模块、`crates/cognitive-kernel/**`、`crates/cognitive-store/**` | `INTENT-ACCEPTANCE-007`; `REQ-INTENT-ACCEPT-001`, `REQ-SHELL-STATUS-001`; 真实验收/verification assets 需 registry 复核 |
| IA-3 | 错误实现自检扩展 | test | IA-1、IA-2 | `crates/cognitive-conformance/src/exec.rs`、`crates/cognitive-conformance/tests/runner_execution.rs`、CI honesty assertions | self-check ≥36；新增两个 corrupted modes 必须翻 fail |
| IA-4 | 证据与 traceability 回填 | docs | IA-1~IA-3 全绿 | `artifacts/evidence/conformance/**`（gitignored）、`docs/traceability/matrix.yaml`、`docs/plan/PROGRESS.md`、`docs/checkpoints/*`、必要时 `findings-ledger.md` | 关联 REQ/F/IMP/D 真实核对；无漂移则明确「无」 |
| IA-5 | 工具链/编排器旁路调试包（可选） | debug | IA-0 | `scripts/v01-auto-run*`、相关 CONNECT 测试、CI 仅在确有范围时 | 文档计划条目；不得变更 pins/claim set |

### 每个工作包的先失败测试 → 实现 → 证据 → DoD

**IA-0**

1. 先失败测试/检查：执行窗口首先确认当前 branch/status；确认目标 vector 在 registry、matrix 和 `conformance/vectors/` 中存在且仍 not-run；当前仓库地板已前进为 84/59/25、self-check 40（本 IA 历史工作包已交付，不得重跑为新任务）。
2. 实现：无业务实现。
3. 证据：执行日志与最终 handoff 的基线记录。
4. DoD：旁路 dirty 未被暂存；未读取/引用 `History/`；主战役分支只归 Lane-CFR。

**IA-1**

1. 先失败测试：在 runner execution test 中先声明 vector 仍 not-run/尚未有对应 execution mode，或先写 reference / deliberately-wrong 期望，使测试红；同时以现有 `m5_intent_chain.rs` 的真实语义为 oracle，不改 vector。
2. 实现：增加最小 execution mode/classification/dispatch；reference 使用 SQLite WAL authority store、`correct_and_supersede`/`supersede_task_contract`、task binding 和真实 effect dispatch fence；比较 vector 要求 `old_epoch_new_dispatch_rejected=true`、`pending_effect_action=reconcile_before_continue`、错误 `INTENT_VERSION_SUPERSEDED`/`intent`、sink dispatch 为零。deliberately-wrong 路径必须模拟旧 epoch 放行，确保自检翻 fail。
3. 证据：`artifacts/evidence/conformance/conformance-report.json` 中该 vector 为 `behavior-*` pass；execution implementation/grounding 指向真实 KRN/store 文件和 registered error；自检报告记录翻 fail。
4. DoD：不改 vector expected；旧记录不重写；authority 状态只由确定性 kernel transition 入口推进；模型/receipt 不参与完成证明。

**IA-2**

1. 先失败测试：以 `INTENT-ACCEPTANCE-007` 的输入构造 runner case，先断言当前没有 vector-specific execution；明确不要直接复用 `GW-REMOTE-COMPLETE-001` 的结果。
2. 实现：使用真实 task acceptance transition gate，remote/agent completed 只能保持 `CANDIDATE_COMPLETE`，没有 verification/acceptance 时 `display_completed=false`，next gate 为 `verification_or_acceptance`；wrong implementation 直接把 agent report 变成 completed，必须翻 fail。
3. 证据：该 vector 进入行为执行并逐字段比较；grounding 指向 `crates/cognitive-conformance/src/exec/behavior.rs`、kernel engine/transition 实现和已有 acceptance test；报告仍明确不构成 Profile 覆盖。
4. DoD：不接受远端 completed、receipt 或模型自述为完成证据；不得新增状态机或 acceptance carrier；真实 registry error/transition 名称全部先核对。

**IA-3**

1. 先失败测试：runner self-check 预期 corrupted corpus 增长，且两新增 vector 均 reference pass / wrong fail。
2. 实现：只增与两个 execution mode 对应的 corrupted mode，保持其他 not-run 原因不变。
3. 证据：`self-check-report.json` 的 `corrupted_but_still_passing=[]`，`must_flip` 不低于原地板且包含新增 case。
4. DoD：CI pinned counts 按「脱两个 vector」更新前必须由任务验收批准；计划默认不把 pins 写死成新结果，执行时以实测为准并在同批更新所有 pin 消费者。任何实现失败都保持 not-run/NO-GO，不降级为 documented-degradation 以求绿。

**IA-4**

1. 先失败测试：`check:consistency` / `gen-matrix --check` 在文档和 matrix 未同步时应能暴露 drift。
2. 实现：只回填真实 impl、impl_tests、evidence、docs；如果发现语义漂移，先写 ledger 漂移项并停止升格。
3. 证据：matrix、PROGRESS、handoff、runner JSON digest；必要时 PR 描述中的 REQ/error/schema/vector 影响扫描。
4. DoD：docs-sync-contract §2/§3 全部适用项完成或明确「无影响」；PROGRESS 使用「实现已提供/测试已执行」，不得写「Profile 已符合」。

### 调试/测试子计划

建议执行顺序：

1. `git status --short --branch`；确认旁路 dirty 只记录不处理。
2. 只读核对 `AGENTS.md`、`docs/plan/PROGRESS.md`、最新 handoff、`PARALLEL-LANES.md`、registry/vector/matrix；确认目标向量和错误码真实存在。
3. 在 Lane-CFR 分支先运行目标 crate 的现有窄测试，先观察 supersede/acceptance oracle；执行环境优先 WSL 原生盘，Windows GNU 场景按既有 linker workaround 处理。
4. 先跑 runner execution 的定向测试，再跑 `cargo test -p cognitive-conformance`。
5. 运行 `cargo run --locked -p cognitive-conformance --bin conformance-runner`，检查目标 vector 的 execution mode、grounding、digest 和 84-vector summary。
6. 运行 `cargo run --locked -q -p cognitive-conformance --bin conformance-runner -- --self-check`，确认错误实现全部翻 fail。
7. 运行 `pnpm run check:consistency` 与 `gen-matrix --check`；执行相关 workspace Rust/TS tests，再按 lane DoD 进入 CI 两 OS。
8. 只有全部绿且证据齐全，才更新 pins、PROGRESS、handoff；失败时保持目标 vector `not-run`，输出 `auto_fail`/NO-GO，禁止把单元测试、partial assertion 或 retry 当作 pass。

期待的不变量（pins 地板保护）：

- 任何失败不得改变已通过的 59 个向量、其余 not-run 族的诚实原因、self-check 40 和 Profile implemented=0。
- 主战役最多改变两个目标 vector 的状态；不批量改变 not-run。
- 当前 GOV/OPS 规范治理入口不得改变 pins；保持 84/59/25。未来只有 runner 对真实实现执行并留证据的独立 CFR 批，才可按实测同批更新 CI honesty pin 与 runner tests。
- 不产生 `fail`、`not-applicable` 或 documented-degradation 伪绿。
- 旧 epoch 新 dispatch 必须零 sink call；acceptance 必须由 verification/acceptance authority 证明。
- 测试并发导致 `m5_http_sse` flake 时，只允许受控、可记录的单线程重跑；重复失败即 NO-GO，不能无限重试。

### 文档联动清单

- `docs/plan/PROGRESS.md`：实测 vector/pass/not-run/self-check、主战役状态、车道分支、证据入口。
- `docs/checkpoints/YYYYMMDD-lane-cfr-m5-intent-authority-handoff.md`：完成/未完成 REQ、提交哈希、测试命令与证据 digest、风险/non-claim、下一步入口。
- `docs/traceability/matrix.yaml`：运行 `gen-matrix` 后回填真实 `impl`/`impl_tests`/`evidence`/`docs`；不得把 generic traceability 当 behavior pass。
- `docs/traceability/findings-ledger.md`：只有发现 F/IMP/D 或规范资产漂移才登记；D-018、F-017、F-015 状态不能被顺手改写。
- `docs/standards/docs-sync-contract.md`：按修正型/语义型/结构型分类；本战役预期是实现+测试+追溯修正，若触碰语义资产则必须同批联动 companion、registry、schema/generated、vector、实现、matrix、Console 漂移标注、ADR/迁移说明（适用时）。
- `docs/plan/PARALLEL-LANES.md`：如启动 lane 分支或所有权状态变化，更新车道表；不得让 CFR 修改 KRN/RUN 业务 crate。
- `docs/prompts/common-prefix.md`：下一执行提示词应保留接入三步和硬纪律；公共前缀本身不因本战役改动。

### 风险与回滚

- 若 vector expected 与实现行为冲突：以 registry/schema/transition/vector normative 资产为准，先登记漂移，停止实现；绝不改 expected 迎合实现。
- 若 supersede 只在 unit test 成功、runner 无法构造完整 vector 输入：保持 not-run，记录「runner adapter 缺口」，另开设计任务。
- 若 acceptance 需要未登记的 shell-status carrier：不得新增 carrier；保持 not-run，并将需求列为 P2/RUN-TSC 依赖。
- 若 CFR 需要跨车道修改 KRN/RUN：回滚本工作包的未提交 CFR changes（仅回滚自己新增内容），拆分所属车道任务；不触碰用户 dirty。
- 若 pins/CI honesty 与新结果不一致：CI 必须红，修正测试/证据或放弃升格；不降低地板。
- 失败交付不得 commit/push；成功执行按 ADR-0008 逐路径暂存、push/PR。

### 本战役不得触碰

- `specs/registry/**`、`specs/schemas/**`、`specs/transitions/**`、`conformance/vectors/**` 的语义内容和 expected；只有在发现真实漂移后按流程另案处理。
- `crates/cognitive-kernel/**`、`crates/cognitive-store/**`、`crates/cognitive-runtime/**` 的业务实现；CFR 只消费其公开 API。发现缺口即停、转所属车道。
- `apps/kernel-server` 的启动面；不加 flags/health endpoints。
- F-017 matrix/digests、PERF campaign、InstallationStore、D-018 端口、Console/clients/personal-blog。
- 用户旁路 dirty 文件、`History/`。

## D. 下一窗口执行提示词

完整可粘贴正文见 [docs/prompts/cfr-m5-intent-authority-slice.md](../prompts/cfr-m5-intent-authority-slice.md)。
