# CognitiveOS Personal 自动推进提示词

> 用途：复制下方 `---` 之间全文到新窗口，作为该窗口的首条消息。文档性质：操作提示词，documentation-only，不是计划/规范/Profile 声明。

---

你在 `D:\agent-kernel`（CognitiveOS 仓库）工作。目标：按正式开发计划**持续自动**推进开发、调试与测试，直到全部 52 个任务高质量 `done` 或明确 `blocked`。不要等待逐步指示，本提示词即持续授权；一个任务完成立即进入下一个。

## 0. Git 授权与红线（owner 已批准）

- **授权**：自动创建 lane 分支（`lane/personal-*`，Lane-KRN/DOC/RUN/CTR 前缀沿用现状）、提交、推送、开 PR，并在 CI（Ubuntu + Windows/MSVC `cargo test --workspace --locked`、`pnpm -r build/test`、clippy、fmt）全绿后自动合并 main。
- **禁止**：force-push 或改写 main 历史；secret/API key 进入任何文件、argv、日志、SQLite、证据或 CI；未执行的测试写成通过（必须 `not-run`）；绕过 Lane-CTR 改 registry/schema/vector；修改已 `done` 任务的验收；用 `PERS-*` 冒充 REQ-ID。
- CI 红灯只允许修复或 revert，不得带红合并。本机 Windows GNU linker exit 121 为非支持基线，不算失败。
- 遵守仓库既有纪律（`.cursor/rules/`、`CLAUDE.md`/`AGENTS.md` 如存在）。

## 1. 每会话启动（plan.md §18.1）

1. 读 `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`——唯一进度台账，任务定义/状态以它为准。
2. 读 `docs/plan/PROGRESS.md` 与 `docs/checkpoints/` 最新 handoff，接续上个窗口。
3. 按 `plan.md` §12 依赖图选下一个可开工任务：优先 critical path（当前顺序：P0-T06 收尾 → P1-T07 → P1-T08 → P1-T09 → P2-T01…P2-T08 → P3…→ P7-T06）；被阻塞时按 §12.1 换并行任务，不空转。
4. 认领：台账标 `in-progress`（分支/日期）；`plan.md` 中 P2+ 压缩卡先扩写为完整强制字段集（§11.1），再动代码。

## 2. 每任务实现循环

1. 读 `plan.md` 同 ID 任务卡、相关 ADR（`docs/adr/0017-0026`）、涉及的 registry REQ；复用现有 kernel/store/runtime 代码，不建平行体系。
2. 实现 + 测试（含负例），真实执行并记录命令输出为证据。
3. 硬不变量（违反即 bug，不是可选项）：
   - Rust daemon 唯一 authority writer；Pi/CLI/SDK/UI 不直写 SQLite、不推进 Task/Effect/Verification 状态。
   - 外部 mutating 全走 Intent/Effect、persist-before-dispatch、幂等键、epoch fencing、reconcile；OUTCOME_UNKNOWN 不盲重试。
   - 完成由独立 verifier 判定；Pi `agent_end` ≠ 完成；partial ≠ completed；Gate suite 中 False Completion = 0。
   - **ADR-0026 trust profile**：Tier 0 静默 / Tier 1 首用记住（capability lease）/ Tier 2 显式确认；任务准入预览是唯一默认人工授权点；预算硬轨替代逐动作审批；默认路径确认 ≤1/task；治理记录全保留。
   - Secret 只在 native Secret Store；ADR-0018 本机开发例外 P2 结束到期，P2-T08 必须核查移除。
4. 验收满足且证据真实 → 台账标 `done`（日期+证据链接/命令结果），同批更新：进度汇总表、`PROGRESS.md`、`docs/checkpoints/` handoff；结构变化（新任务/验收/依赖）同步 `plan.md` 卡与依赖图、`personal-trace.yaml`（保持无孤儿）。
5. 验证：`pnpm run check:consistency`、`git diff --check`、相关 cargo/pnpm 测试。
6. commit → push → PR → CI 绿 → merge → 立即领下一任务。

## 3. Gate 与证据纪律

- `done` ≠ Profile `implemented`；G0/B01-B12/RC 只有真实执行才可声明。
- 正式 Gate 环境不可得时（干净 Linux VM、真实 Provider key、`hal9000@192.168.1.2` SSH）：实现与本地测试照常推进，记 `experimental-local-only` / `tested-local`；Gate 保持 `not-run` + blocked 原因；继续做其他可做任务，**不停摆**。
- 需要 owner 提供实物（API key、SSH 认证、VM）时：一次性列出精确清单后转做其他任务。
- B04 证据记录默认路径确认次数（≤1/task）与 Tier-2 负例；B01 证据必须 redacted、无 key。

## 4. 仅以下情形停下问 owner

1. GO/NO-GO 决策点：P4-T04 embedding、B11 multi-agent 收益、P7 发布/publish 开关。
2. 合同/schema/vector 变更（Lane-CTR 流程）。
3. 不可逆操作：删用户数据、回滚 main、对外发布。

其余一律自行决策，理由记入 handoff。

## 5. 会话收尾（§18.2）

结束前必须完成：台账/PROGRESS/handoff 更新（完成项、未完成项、已执行测试、证据、风险、下一步），保证下一窗口零上下文损失接续。上下文将满时提前收尾并写 handoff，不要写到一半中断。

---
