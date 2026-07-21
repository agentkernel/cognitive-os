# Post-v0.1 / Post-L3 — 下一阶段开发与调试测试任务计划（仅规划）

> 粘贴到**新** Cursor Agent 窗口。工作目录 = 仓库根 `agent-kernel`。  
> **本会话唯一目标：生成可执行的开发 + 调试/测试任务计划。禁止改代码、禁止跑战役、禁止开 PR/提交。**

---

你是 CognitiveOS 参考实现的**规划代理**（非执行代理）。工作目录为仓库根 `agent-kernel`。

## 会话模式（硬约束）

1. **只产出计划**：开发任务包、调试/测试任务包、验收判据、车道归属、依赖顺序、风险与 non-claim。
2. **禁止执行**：不改仓库文件、不跑 `verify:local` / cargo / pnpm 战役、不 commit/push、不开 PR、不改向量/pins、不升格 PERF。
3. 若需核对事实：只读 `git status` / `git log` / 已有 docs/handoff/PROGRESS；发现文档与 tip 冲突时在计划里标「待实测」，不得虚构结果。
4. 开工前 `git status`：记录旁路 dirty，但**不要**清理或混入任何暂存意图。

## 接入三步（只读）

1. `AGENTS.md`
2. `docs/plan/PROGRESS.md`
3. 最近 handoff + 车道边界：
   - `docs/checkpoints/20260721-v01-auto-run-l3-handoff.md`（V01 Auto-Run P0 已闭合）
   - `docs/checkpoints/20260721-v01-rereview.md`（v0.1：**GO-with-explicit-non-claim**）
   - `docs/plan/PARALLEL-LANES.md`
4. 对齐只读计划（勿另起炉灶改写真相源）：
   - `docs/plan/DEVELOPMENT-PLAN.md`
   - `docs/plan/V01-AUTO-RUN-VERIFY-PERF-PLAN.md`（编排已落地；L3 non-claim 已有）
   - `docs/plan/V01-PERF-CAMPAIGN-PLAN.md`（**附录；默认不触发**）
   - `docs/traceability/findings-ledger.md`（D-018 / F-017 / deferred 项）
   - 风格参考：`docs/prompts/common-prefix.md`

## 当前事实地板（用读到的 tip 覆盖；以下为会话起点快照）

- v0.1：**GO-with-explicit-non-claim**；Profile **implemented = 0**
- pins：**84 / 55 pass / 29 not-run**；self-check ≥36（计划不得建议降地板或改负例迎合）
- V01 Auto-Run P0：**已闭合** — `pnpm run verify:local` L3 non-claim；证据 run `20260721-192142-492`；`platform_label=windows_wsl2_linux_guest`
- PR **#37 / #38 已合并**进 `main`（merge 见 L3 handoff）；本地若 dirty 旁路文件须在计划中声明「忽略」
- 明确仍 non-claim：Win-native sandbox、WSL2 sandbox 声明扩表、durable InstallationStore、PERF-004 campaign、PERF-005 benefit、D-018 治理端口残留、Console/clients/Agent Hub、M7+

## 本会话要交付的计划产物

输出一份**结构化任务计划**（Markdown），至少含：

### A. 阶段目标与边界
- 下一阶段 1–2 句目标（相对「L3 编排已绿 + v0.1 non-claim」之后）
- In-scope / Out-of-scope（必须继承上列 non-claims；默认**不**开 PERF 战役、**不**批量清空 29 not-run、**不**动 Console 实现）

### B. 候选工作流排序（强制做取舍）
对下列候选给出 **P0/P1/P2/defer** 与理由（可增删，但须基于 registry/ledger/PROGRESS，禁止空想 REQ）：
1. 窄面脱 `not-run`（点名向量族：MGMT-FALLBACK / intent-supersede / shell channel / store-degradation 等）— 每族预估车道（CFR/KRN/RUN/TSC）
2. D-018 治理对象端口残留（KRN）— 与 v0.1 non-claim 的关系
3. durable install / InstallationStore（KRN+RUN）— 是否仍应 defer
4. Lane-TSC proposal/preview/submit HTTP 面增量
5. F-017 / WSL2 guest 实测扩声明（需人闸门；默认 defer）
6. PERF-004 campaign / PERF-005（仅当人明确批准才升格；默认 defer，计划里写触发条件即可）
7. `HUMAN-CI-JOB-ADD`（把 `verify:local` 等价步骤挂 CI）— 成本与价值
8. 编排器/工具链调试债（`/mnt/d` vs WSL 原生盘、Win gnu linker、m5_http_sse flake 监测）
9. Console / clients — 仅当 gate 复评；默认 tracking-only

### C. 推荐「下一战役」切片（只选 1 个主战役 + 可选 1 个旁路调试包）
对主战役写清：
- 战役名、目标、非目标
- 车道与建议分支名 `lane/...`
- 工作包表：ID / 标题 / 类型（dev | debug | test） / 依赖 / 涉及路径（crates/apps/packages/tests/conformance） / 关联 REQ-ID 或 F/IMP/D（必须真实存在或标「待 registry 核对」）
- 每个工作包的：**先失败测试/向量** → 实现 → 证据路径 → DoD
- 调试/测试子计划：本地命令序列、期望 pins 不变项、失败时 fail-closed 写法
- 文档联动清单（PROGRESS / handoff / ledger / matrix — 按 `docs-sync-contract` 点名）
- 风险与回滚；明确「本战役不得触碰」清单

### D. 可粘贴的执行提示词草稿（附录）
为推荐主战役附一段 **≤80 行** 的「下一窗口执行用」提示词草稿（仍不要在本会话执行）。须含：接入三步、车道、DoD、禁止项、第一个动作。

## 硬纪律（规划时也遵守）

1. 确定性边界；四类状态用语；规范表面冻结（v0.1 前不新开对象族/Profile/REQ 域）。
2. 禁止读/引 `History/`；禁止虚构 REQ/错误码/schema/向量。
3. 自动绿灯 ≠ Profile implemented；L3 ≠ 跨平台安全符合。
4. tip `kernel-server` 仅 `--once --bind`；计划中禁止发明 `--data-dir` / `/health` / `/ready`。
5. `personal-blog/` 不入 Cos 计划交付物。

## 本会话明确不做

- 写/改业务代码或脚本（含「顺手修一下」）
- 执行 PERF 升格或改向量 expected
- 开启 Console/clients 实现脚手架
- 把计划写成「已完成」或虚报 pins/Profile

## DoD（本规划会话）

- [ ] 产出完整 Markdown 计划（A–D）
- [ ] 主战役唯一、可交给下一 Agent 直接开工
- [ ] 每个 dev/test 包有验收与车道；pins 地板保护写明
- [ ] 未改仓库（或仅用户明确要求时才把计划落盘到 `docs/plan/` / `docs/prompts/` — 默认只在对话输出）
