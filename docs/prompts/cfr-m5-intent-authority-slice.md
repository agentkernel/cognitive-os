# Lane-CFR 执行提示词：CFR-M5-INTENT-AUTHORITY-SLICE

> 用法：将本文件全文粘贴到**新** Cursor Agent 会话（工作目录 = 仓库根 `agent-kernel`）。
> Canonical 计划：[docs/plan/POST-V01-NEXT-PHASE-PLAN.md](../plan/POST-V01-NEXT-PHASE-PLAN.md)
> 公共前缀内联自 `docs/prompts/common-prefix.md`（修改需同步）。

---

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

## 接入三步（动手前必做）

1. 读 `AGENTS.md`（命令速查、目录地图、DoD、红线）。
2. 读 `docs/plan/PROGRESS.md`（当前里程碑/车道状态与开放 P0）。
3. 读最近一份 `docs/checkpoints/*-handoff.md`（含 `20260721-post-v01-next-phase-planning-handoff.md`、`20260721-v01-auto-run-l3-handoff.md`、`20260721-v01-rereview.md`），并对照 `docs/plan/PARALLEL-LANES.md` 确认本车道边界与所有权后再动手。
4. 只读核对：`docs/plan/POST-V01-NEXT-PHASE-PLAN.md`、`specs/registry/requirements.yaml`、`conformance/vectors/intent-supersede-002.json`、`conformance/vectors/intent-acceptance-007.json`、`docs/traceability/matrix.yaml`。

## 硬纪律（全程有效）

1. **确定性边界**：概率组件只产 candidate/proposal；授权、CAS、状态迁移、硬预算、幂等、fencing 与最终提交必须由确定性代码执行。
2. **规范优先级**：digest 固定的机器 schema/registry/transition/vector 与 normative companion > 固定版本 RFC/Core/Profile 文本 > 白皮书 > 实现建议；冲突时采用不扩大权限、数据范围、风险、预算或完成声明的解释。
3. **四类状态用语**：规范已登记 / 实现已提供 / 测试已执行 / Profile 已符合，严格区分；`implemented` 仅指全部适用 MUST 有通过证据。
4. **测试先行**：先写失败测试再实现；schema-valid ≠ behavior-pass；完成证明只来自 authority 状态、Effect、Verification 与 Event。
5. **规范表面冻结**：v0.1 前不新增对象族、Profile、REQ 域；只允许实现反馈驱动的修正型规范变更；发现漂移先登记 `docs/traceability/findings-ledger.md` 再最小修正闭合。
6. **P0 门禁**：findings-ledger 中开放的 P0 未闭合前，对应子系统不得进入实现。
7. **可追溯提交**：每个提交/PR 关联 REQ-ID、F/IMP 条目或文档条目；确无关联时写明原因。
8. **红线**：禁止读取、引用或参考 `History/`；禁止虚构 REQ-ID/错误码/schema/向量；禁止改写向量或删除负例迎合实现。

## 本窗口唯一战役

**战役名：** `CFR-M5-INTENT-AUTHORITY-SLICE`  
**车道：** Lane-CFR  
**建议分支：** `lane/cfr-m5-intent-authority-slice`

**主目标：**把 `INTENT-SUPERSEDE-002` 与 `INTENT-ACCEPTANCE-007` 接入真实确定性 KRN/store acceptance 行为执行，生成 behavior evidence，并让 deliberately-wrong implementation 对两者翻 fail。

**关联真实资产：** `REQ-INTENT-SUPERSEDE-001`、`REQ-INTENT-ACCEPT-001`；附属 REQ/error/schema/vector 必须以 registry/matrix 实测为准，不得猜测。

**第一个动作：** `git status --short --branch`。记录并保护所有已有 dirty/untracked 文件，不清理、不回退、不 `git add -A`。然后按接入三步只读核对资产。

## 先失败测试 / oracle

必须先失败测试，再实现。复用已有真实测试作为 oracle：

- supersede：`crates/cognitive-store/tests/m5_intent_chain.rs`，检查 epoch+1、pending effect=`reconcile_before_continue`、旧 epoch dispatch=`INTENT_VERSION_SUPERSEDED`、零 sink dispatch。
- acceptance：现有 task acceptance behavior 与 `remote-completed-not-acceptance` 证据，但**不得**冒充 `INTENT-ACCEPTANCE-007` vector 已执行。

工作包顺序：IA-0 → IA-1 → IA-2 → IA-3 → IA-4（IA-5 可选）；详见 [POST-V01-NEXT-PHASE-PLAN.md](../plan/POST-V01-NEXT-PHASE-PLAN.md) §C。

## DoD

- 目标 vector 只在真实 runner 比较通过后从 not-run 变 pass；否则保持 not-run/NO-GO。
- self-check 无 `corrupted_but_still_passing`，`must_flip` 不低于原地板 36。
- 不改 vector expected、负例、REQ/schema/transition，不新增对象族/Profile/REQ 域/错误码。
- 不降低 84/55/29 地板；不批量清空 29 not-run；Profile implemented 仍为 0。成功脱 not-run 后 pins 以实测为准，并同批更新 CI honesty pin。
- 不跨车道改 KRN/RUN；不碰 D-018、InstallationStore、F-017 扩表、PERF、Console/clients、`kernel-server` 新 flag/health。
- 证据写入 gitignored `artifacts/evidence`；同步 matrix、PROGRESS、handoff，必要时 ledger；执行 `check:consistency` / `gen-matrix --check`。
- 运行定向测试、`cargo test -p cognitive-conformance`、runner、self-check；再按 CI 两 OS DoD 验证。
- 工具链问题优先 WSL 原生盘；`m5_http_sse` 只做受控单线程复现，禁止无限重试。

## 禁止项

- 禁止 `verify:local` 战役升格、PERF campaign/benefit、改 pins 迎合。
- 禁止发明 tip 不存在的 `--data-dir` / `/health` / `/ready`。
- 禁止读取 `History/`；禁止推送 `personal-blog/**`。
- 失败则停在诊断/计划结果，写清缺口与下一 lane 入口；不得把单元测试、partial assertion 或 retry 当作 vector pass。

## 会话结束协议

更新 `docs/plan/PROGRESS.md` → 按 `docs/checkpoints/TEMPLATE.md` 写 handoff（建议名 `YYYYMMDD-lane-cfr-m5-intent-authority-handoff.md`）→ 分批提交（逐路径 `git add`；push 前查 `git log --name-only origin/main..HEAD`）。
