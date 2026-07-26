# 客户端持续开发执行提示词（自主推进 · 全权委托）

> 类别：prompt（informative）｜ 日期：2026-07-26 ｜ owner：Lane-CON ｜ 授权人：项目 Owner（2026-07-26）
>
> 用法：整体粘贴到新窗口。目标：按 [clients/plan/development-plan.md](../plan/development-plan.md) 持续自动推进开发与调试测试，直到高质量完成全部计划或仅剩外部阻断项。

---

## 0. 授权与边界（Owner 已批准，无需逐次确认）

- **授权自动执行**：git add / commit / push、创建分支、发起与合并 PR（或本地合并入 `main`）。
- **合并硬条件**：本批全部验证绿灯（见 §2 第 3 步）；原子提交；提交信息 `<域>: <动作>`；禁止 force-push `main`、改写已推送历史、删除非本会话分支。
- **授权计划内自主决策**；超出计划范围的结构/产品决策先按治理登记（`CLIENTS-DEC-*` 或对应产品决策日志）再执行。
- **外部项不可自动化**（法务评估 POC-LIC、Apple/Anthropic/OpenAI 确认、真机/签名/商店账号）：登记状态后跳过，继续其它可执行项；**禁止虚构其结论**。
- 验证环境不可用（无法运行 pnpm/cargo/runner）时：**不得声明测试已执行**，登记 blocker 并停在验证步。

## 1. 首读（按序，读完再动手）

1. `AGENTS.md`、`.cursor/rules/`（02、11、16、17）
2. `docs/plan/PROGRESS.md`（全局唯一真相）
3. `clients/plan/development-plan.md`（执行计划）+ `clients/plan/milestones.md` §当前可执行工作面
4. `clients/README.md`、`clients/GOVERNANCE.md`（§4 计数指针规则、§8 文档系统地图）、`clients/READINESS.md`、`clients/governance/readiness-gates.md`
5. `clients/governance/decision-log.md` **CLIENTS-DEC-002**（PoC 代码唯一豁免：落位仓库根 `poc/`）
6. 涉 Agent Hub 时：`clients/agent-hub/docs/GOVERNANCE.md` §7 + `clients/agent-hub/plan/`

## 2. 执行主循环（重复直到 §5 停止条件）

1. **选任务**：当前 Phase 最高优先未完成项。顺序：Phase 0（T5→T6→T7）→ Phase 1（A/B/C 三线并行）+ Console gate 依赖组 1/2/7 对账留证 → gate 全绿后 Phase 2（S1→S2→S3/S4/S5 并行）→ Phase 3（AH-M2..M4）→ Phase 4。
2. **开分支** `lane/<域>-<任务简名>`；测试先行：先写失败测试与安全负例，再实现。
3. **验证（绿灯定义）**：
   - 触碰 TS：`pnpm -r build && pnpm -r test`，无 `any` 逃逸新增；
   - 触碰 Rust：`cargo test`（含所触 crate 全部测试）；
   - 触碰文档：`pnpm run check:consistency` + `git diff --check` + `clients/` 手动链接/anchor 核对（`clients/README.md` §9）；
   - PoC：真实 API/OS 行为执行，证据按 `clients/shared/docs/poc-execution-record.md` 模板留档；
   - 实现切片：附带该片验收（含"错误完成声明数为零"安全验收；UI 过 WCAG 2.2 AA 与键盘全通路）。
4. **绿灯** → commit + push + 合并（有 PR 流程用 PR，无则本地合并推送）；**红灯** → 修复循环（同一任务最多 3 轮，见 §5c）。
5. **回写**：`development-plan.md` 状态列；语义/gate/readiness 变化回写 `PROGRESS` 并写 handoff（`docs/checkpoints/`）；履行 rules/16 同批义务（README 索引、READINESS、受影响文档同批更新）。
6. **Phase 出口**：写 checkpoint 评审文档，对照出口判据逐条附证据路径，然后进入下一 Phase。

## 3. 不可违反的纪律（治理红线）

- **gate 未过禁产品实现**；PoC 代码只进仓库根 `poc/`（CLIENTS-DEC-002），不携带产品 ID、不被产品代码引用、不构成任何实现/测试/Profile 声明。
- 四类状态用语严格分开（规范已登记/实现已提供/测试已执行/Profile 已符合）；未执行写 `not-run`/`none`；禁止把静态检查/文档完成写成实现或测试证据；**禁止为解阻把 `blocked` 改写成 GO**。
- 客户端一律**非 authority**（只发请求、渲染 snapshot+watch 投影）；`OUTCOME_UNKNOWN` 一等状态不盲重试；`pause_pending` ≠ paused；断连 fail closed；手机仅 remote companion（R0/R1）；接管 L6 保持阻断、L8 永久禁止。
- canonical 唯一：不造第二事实源；全局计数只指 `PROGRESS`（GOVERNANCE §4）；产品 ID 不重编号、不重用。
- 不跨车道改 Lane-TSC/CTR/RUN 所属 package 接口；需要时登记 findings 并走所属车道流程。

## 4. 质量标准（每任务/每片出口必须同时满足）

全部适用测试绿；新功能含负例、降级与断连路径测试；TS `strict` 全开；文档/状态/证据同批一致；完成声明必须附证据路径——达不到就不合并、不声明完成。

## 5. 停止与续接

- **停止**：(a) 计划全部完成且绿灯——输出终版总结；(b) 剩余项全部为外部阻断——输出待人工清单（每项：需要谁、做什么、卡住哪个出口）；(c) 同一任务 3 轮修复仍红——登记 blocker 转下一任务，全部任务被阻则停。
- **context 将尽**：写 handoff（当前任务、分支、验证状态、下一步）→ 新窗口粘贴本提示词继续。
- **每次会话结束输出**：完成项清单、提交/合并列表（含哈希）、状态回写位置、剩余任务与阻断。
