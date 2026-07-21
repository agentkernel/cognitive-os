# 20260721 Lane-RUN Handoff（M5 批 2b：Harness Loop / Shell / intent / recovery 6–7）

## 1. 本次会话完成

按 `docs/prompts/milestone-m5.md` / `lane-run.md` 与 KRN M5 handoff §7 冻结端口，交付 M5 RUN 批 2b（分支 `lane/run`，基线含 CTR 绑定与 KRN/RUN 换装 merge `1a8e0b8`）。零新第三方依赖。向量零改写、零执行（仍 not-run，归 Lane-CFR）。

- **`cognitive-runtime::harness_loop`**：`BoundedHarness` 消费 `LoopDriver`（start / begin_iteration / record_progress / stagnation）；`decide_stagnation` 在 ceiling 上 Stop 或 Escalate——禁止无界空转；硬预算错误以 `EffectError` 上抛。
- **`cognitive-runtime::shell`**：`ShellService` 实现 proposal / preview / submit（receipt-level）/ attach / detach / cancel。**detach 永不取消**；cancel → `CANCEL_PENDING`（Effect reconcile 由调用方）；终态 effect → `CANCEL_TOO_LATE`。客户端面永不写 authority。
- **`cognitive-runtime::intent_flow`**：`admit_and_mint_contract` = record → candidate → admit → mint；`correct_and_supersede` 包装 kernel `supersede_task_contract`（epoch fencing）。
- **`cognitive-runtime::recovery_flow`**：`plan_recovery_continuations` 消费 `reauthorization_obligations` + `reauthorization_satisfied`；`pre_crash_binding_is_stale` 消费 `context_rebinding`（步骤 6/7）。
- **`apps/kernel-server`**：`POST /shell/{detach,cancel,attach}` 非 authority JSON 面；e2e `m5_http_sse::shell_detach_and_cancel_routes_are_non_authority`。
- **文档联动**：matrix 回填 RUN-004/005/007/008、REC-001、SHELL-ATTACH/CONTROL/CORRECTION/DETACH/PREVIEW；PROGRESS；本 handoff。

## 2. 未完成 / 进行中

- Shell HTTP 面为 `--once` 参考路由（进程内无完整 session 状态机）；完整 Effect cancel reconcile / channel isolation / watch 恢复端到端 = CFR 向量 + TSC 真集成。
- D-018 仍 partially-implemented（组装器 + shell/watch 车道面已有；闭合仍待 CFR 行为证据 + 治理对象端口）。
- F-011 **不闭合**（三负例向量仍 not-run）。
- Lane-TSC 真 HTTP/SSE 对接与 Lane-CFR M5 向量执行 = 下一批（M5 出口）。

## 3. 测试与证据状态

- 新增车道行为测试：**runtime 单元 4**（shell detach/cancel、stagnation、recovery 6/7）+ 既有 event envelope 1；**kernel-server e2e +1** shell 路由。
- 验证矩阵：以本批 push 后本地 `cargo test --workspace` / clippy / fmt / pnpm / check:consistency / gen-matrix --check 为准；CI 结论以 PR 页为准。
- **向量：零改动**——84 / 46 pass / 38 not-run。
- 状态用语＝「实现已提供 + 车道测试已执行」；**不构成 Profile 覆盖声明**。

## 4. 未决风险与漂移

- 无新漂移登记。
- **给 Lane-TSC**：可对接 `POST /management/*`、`GET /task/watch`、`POST /shell/{attach,detach,cancel}`；客户端永不做 authority。
- **给 Lane-CFR**：优先 shell-cancel-semantics-005、shell-detach-attach-004、loop-contract/gate、intent-supersede-002、management-deterministic-fallback、F-011 三负例。
- **给 Lane-KRN**：治理对象持久化/解析端口（D-018 ②）仍开放。

## 5. 下一步入口

- 建议提示词：`docs/prompts/lane-tsc.md` → `docs/prompts/lane-cfr.md` → M5 milestone review。
- 工作分支：`lane/run`（合入后切 TSC/CFR）。
- 第一个动作：TSC 对接真 kernel-server HTTP/SSE（proposal/preview/submit/attach/detach/cancel/watch）。

## 6. 快照

- PROGRESS 已更新：是。
- 本次提交：实现批（runtime harness/shell/intent/recovery + kernel-server shell 路由）→ docs 批（本文件所在提交）。
