# 客户端持续开发执行提示词（独立仓库版 · 自主推进）

> 类别：prompt（informative）｜ 日期：2026-07-26（拆分后重写）｜ owner：Lane-CON
>
> 用法：整体粘贴到新窗口，工作目录 `D:\agent-kernel-clients`。目标：按 [plan/development-plan.md](../plan/development-plan.md) 持续推进开发与调试测试，直到高质量完成或仅剩外部阻断项。
>
> 本文替代 2026-07-26 拆分前的同名提示词。旧版所有 `clients/**` 路径与 `pnpm`/`cargo` 验证命令在本仓库**均不成立**，见 §1、§2。

---

## 0. 环境

| 项 | 值 |
|---|---|
| 本仓库（客户端，你的工作区） | `D:\agent-kernel-clients`（WSL `/mnt/d/agent-kernel-clients`）→ `github.com/agentkernel/cognitiveos-clients`，默认分支 `main` |
| 主仓库（内核/规范，只读参考） | `D:\agent-kernel`（WSL `/mnt/d/agent-kernel`）→ `github.com/agentkernel/cognitive-os` |
| 路径换算 | 本仓库根 == 原 monorepo 的 `clients/`。旧文档里的 `clients/X` 现在是 `X` |
| 推送凭据 | WSL 内无 credential helper。用 `gh auth token`（Windows `gh.exe` 已登录 `agentkernel`）或 Windows `git.exe` |

主仓库若不在本地，跨仓引用一律走 URL，不要为了读一个文件去 clone 整个 monorepo。

## 1. 先接受这条事实：本仓库现在没有任何可运行的东西

实测（2026-07-26）：161 个 tracked 文件 = **159 个 `.md` + `.gitattributes` + `.gitignore`**。没有 `package.json`、没有 pnpm workspace、没有 Cargo、没有 CI、没有 `.cursor/rules/`。`pc/app/`、`mobile/android/app/`、`mobile/ios/app/` 三个目录各只有一个 README 占位。

由此：

- **现在没有任何测试可以跑**，也没有任何东西可以启动或调试。你在本仓库跑不出 `pnpm -r test` 或 `cargo test`——那些命令属于主仓库。
- 任何"测试已执行 / 实现已提供"的声明在写下第一行代码前都是假的。四类状态用语（规范已登记 / 实现已提供 / 测试已执行 / Profile 已符合）严格分开，未执行写 `not-run`／`none`。
- "运行调试测试"在本仓库的真实含义是 §4 的三个阶段，**阶段 A 之外的东西现在都不存在**，需要你先造出来。

## 2. 拆分遗留的未决项（先登记决策，不得默认，不得猜）

仓库拆分（2026-07-26）使下列既有治理条款失去唯一解释。动手前逐条在 `governance/decision-log.md` 登记 `CLIENTS-DEC-003+`，或明确向 Owner 提问：

1. **PoC 代码落位**：[CLIENTS-DEC-002](../governance/decision-log.md) 规定 PoC harness 落"**仓库根 `poc/`**"且"不得进入 `clients/**`"。拆分后"仓库根"指本仓库还是主仓库？原措辞的排除项 `clients/**` 现在正是本仓库全部内容。**这是阶段 B 的前置，必须先定。**
2. **PROGRESS 跨仓回写**：`PROGRESS` 仍是全局计数与 gate 结论的唯一真相，但它在主仓库。gate/readiness 变化时的回写要落成两个仓库两次提交，机制需写明。
3. **P0-T5 失效**：原任务"把 `clients/` 纳入 `check:consistency`"不再可行——主仓库 checker（`tools/src/check-consistency.mjs`）只遍历 `docs/ crates/ packages/ specs/`，从未覆盖 `clients/`，现在还跨了仓库。应重定义为"本仓库自带一致性检查"（见 §4 阶段 A）。
4. **规则强制层缺失**：`.cursor/rules/` 11/16/17 只在主仓库生效，本仓库没有。rules/16 的"同批更新 README 索引 / READINESS / 受影响文档"义务目前无强制层，靠人工纪律或需在本仓库补一份。
5. **自动 push 授权范围**：Owner 2026-07-26 的自动 commit/push/merge 授权（主仓库 ADR-0008）是在拆分前给的，是否覆盖向 `cognitiveos-clients` 的推送，请确认后再自动推送；未确认前本地提交、暂不 push。

## 3. 首读（按序，读完再动手）

本仓库：

1. `README.md`（项目地图 + §9 持续维护与手动 gate）、`GOVERNANCE.md`（§4 计数指针规则、§8 文档系统地图）
2. `plan/development-plan.md`（执行计划）+ `plan/milestones.md` §当前可执行工作面
3. `READINESS.md`、`governance/readiness-gates.md`（Console 实现 gate canonical 定义）
4. `governance/decision-log.md`（尤其 CLIENTS-DEC-002）
5. 涉 Agent Hub 时：`agent-hub/docs/GOVERNANCE.md` §7 + `agent-hub/plan/`
6. 背景分析：`review/2026-07-26-clients-design-review.md`、`review/2026-07-26-clients-development-plan.md`

主仓库（URL 直读即可）：`AGENTS.md`、`.cursor/rules/`（11/16/17）、`docs/plan/PROGRESS.md`、`docs/plan/PARALLEL-LANES.md`。

## 4. "运行调试测试"的三个阶段

**阶段 A — 现在就能做，不碰任何 gate：本仓库文档工具链 bootstrap**

这是唯一现在可执行的工程工作，也是 §2.3 的落地。最小可用集：

- Markdown 链接与 anchor 解析器：检出仓内死链、失效 anchor；**并断言零条 `../` 逃逸链接**（迁移时已清零为 0，不要再引入；跨仓引用一律写绝对 URL `https://github.com/agentkernel/cognitive-os/blob/main/...`）。
- 状态用语线性检查：`not-run`/`none`/`planned`/`blocked` 的使用与 gate 状态是否自洽；硬编码全局计数是否仍指向 `PROGRESS`（GOVERNANCE §4）。
- `git diff --check`。
- 选型自便（Node 单文件脚本足够），但**要为 checker 本身写测试**——它是本仓库第一个真正可跑的测试目标。这一步完成后 `pnpm test`（或等价命令）才第一次有意义。

**阶段 B — 前置 §2.1 落位澄清：Windows PoC harness**

`CLIENTS-DEC-002` 豁免下**唯一**允许写的非工具代码。执行 [windows-poc-runbook](../pc/docs/platforms/windows/windows-poc-runbook.md) `WIN-RG-01..10`：真实分进程、真实 OS 行为、**禁 mock**。证据按 [poc-execution-record](../shared/docs/poc-execution-record.md) 模板留档。PoC 代码不得携带产品 ID、不得被产品代码引用，其存在不构成任何实现/测试/Profile 声明。

**阶段 C — 产品实现：当前全部阻断**

Console 实现 gate 五条（[readiness-gates §1](../governance/readiness-gates.md)）无一满足；所有平台 Open PoC/GA gate 全 `not-run`；技术栈 ADR 未批准；`POC-LIC-001..003` 法务 gate 未出结论。**gate 未过禁产品实现，且禁止为解阻把 `blocked` 改写成 GO。**

同期可推进的非代码项（不违反 gate）：POC-LIC 法务评估发起、Tier 1 首发 6→2 adapter 决策准备、Console gate 依赖组 1/2/7 对账留证、Phase 0 剩余 P0-T6/T7。

## 5. 执行主循环

1. **选任务**：`plan/development-plan.md` 当前 Phase 最高优先未完成项。顺序 Phase 0（T5→T6→T7）→ Phase 1（A/B/C 并行）→ gate 全绿后 Phase 2 → Phase 3 → Phase 4。
2. **开分支** `lane/<域>-<任务简名>`；测试先行：先写失败测试与安全负例，再实现。
3. **验证（本仓库的绿灯定义）**：
   - 触碰文档 → 本仓库 checker（阶段 A 产物）+ `git diff --check` + `README.md` §9 手动链接/anchor 核对；
   - 触碰 `poc/**` → 该 PoC 自带测试 + 真实执行记录（非 mock）；
   - 新增跨仓链接 → 必须是绝对 URL，checker 断言逃逸链接为 0；
   - **工具不可用就不许声称跑过**：登记 blocker 停在验证步。
4. **绿灯** → 原子提交，信息 `<域>: <动作>`；push 前先确认 §2.5 授权。**红灯** → 修复循环，同一任务最多 3 轮。
5. **回写**：`plan/development-plan.md` 状态列；gate/readiness 结论变化时按 §2.2 机制同步主仓 `PROGRESS`；履行 rules/16 同批义务（README 索引、READINESS、受影响文档同批更新）。
6. **Phase 出口**：写评审 checkpoint，对照出口判据逐条附证据路径。

## 6. 不可违反的红线

- gate 未过禁产品实现；PoC 代码边界见 §4 阶段 B。
- 四类状态用语严格分开；禁止把静态检查或文档完成写成实现/测试证据；禁止虚构外部结论（法务、Apple/Anthropic/OpenAI、真机签名）。
- 客户端一律**非 authority**（只发请求、渲染 snapshot+watch 投影）；`OUTCOME_UNKNOWN` 一等状态不盲重试；`pause_pending` ≠ paused；断连 fail closed；手机仅 remote companion（R0/R1）；接管 L6 保持阻断、L8 永久禁止。
- canonical 唯一：不造第二事实源；全局计数只指主仓 `PROGRESS`；产品 ID 不重编号、不重用。
- 不跨车道改 Lane-TSC/CTR/RUN 所属 package 接口；需要时登记 findings 走所属车道流程。
- 禁止 force-push `main`、改写已推送历史、删除非本会话分支。

## 7. 停止与续接

- **停止**：(a) 计划全部完成且绿灯 → 输出终版总结；(b) 剩余项全为外部阻断 → 输出待人工清单（每项：需要谁、做什么、卡住哪个出口）；(c) 同一任务 3 轮仍红 → 登记 blocker 转下一任务，全阻则停。
- **context 将尽**：写 handoff（当前任务、分支、验证状态、下一步）→ 新窗口粘贴本提示词继续。
- **每次会话结束输出**：完成项清单、提交列表（含哈希）、状态回写位置、剩余任务与阻断。

## 8. 已知未修问题（迁移时发现，非本次迁移造成）

- `agent-hub/docs/traceability/poc-prep-checklist.md` 指向 `../legal/licensing-and-terms.md`，实际文件在 `../security/licensing-and-terms.md`。本仓库当前唯一的内部死链，阶段 A checker 应能抓到。
