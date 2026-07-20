# 20260720 Lane-CTR Handoff（KRN M2 缺口批）

## 1. 本次会话完成

处置 Lane-KRN M2 handoff §4 登记的契约缺口清单（权威原文：`20260720-lane-krn-m2-handoff.md` §4，CFR 复述：`20260720-lane-cfr-m2-handoff.md` §5；分支 `lane/ctr`，基线 `2dc3e02` = PR #7 merge）。逐项终态：

| 项 | 终态 | 交付/理由 |
|---|---|---|
| 1 state-transition-request/record 无生成绑定 | **closed**（提交 `539ec05`） | 两份已注册 schema 入 codegen CORE_SET（26→28 输入，生成 28→30 schema 模块 ×2 语言；`$ref` 闭包零新增）；双语言绑定带 `SCHEMA_ID`/`SCHEMA_DIGEST` 常量并入 `SCHEMA_DIGESTS` 聚合；regenerate-diff 门自动覆盖；模块计数钉扎同批 28→30（`generated_types.rs`、`generated-types.test.ts`）。渲染语义零变化 → 生成器版本维持 0.2.0（依据写入 CORE_SET 注释与 ADR-0006 delivery record 第 6 条）。schema/向量计数 60/81 不变，runner/static_check/CI 钉扎零触碰 |
| 2 状态投影无注册合同/digest 域 | **deferred-to-v0.2（D-017）** | 判定依据（全文见台账）：REQ-STATE-002 是性质级要求且刻意以 projection_version 为参数；event-audit-watch §3 将投影定性 derived/disposable/never-authority-input，登记面仅限跨信任边界投影（shell-status-view/context-view/world-state）；canonical §9 域由"其 contract"登记——本投影 contract 即内核实现，digest 不跨信任边界（CFR 行为执行只做 opaque 等值比较）。注册 = 新增对象形合同，IMP-01 冻结下不做。触发条件：投影 digest 以机器合同形态跨组件/信任边界（M4 恢复证据跨组件比对，或 M5+ 暴露为钉扎合同值）→ 届时实现反馈驱动修正型注册，KRN 换域（修正型，已在其 handoff 承诺） |
| 3 事件 envelope 升格路径 | **决策落档（D-018，不实施）** | 路径固定：M3 治理链落地后由 Lane-CTR 修正型评估 KRN 的两选项——(a) 登记内核内部事件形状（预期同 D-017 逻辑 defer）或 (b) 在事件成为治理对象的边界（M5 运行时/watch 面）组装完整已登记 `event.schema.json` envelope（预期路线）；合同裁决归 CTR、`engine.rs` 事件组装替换归 KRN；触发 = M3 出口评审后协调者排批。当前实现事件值维持现状，digest 稳定性证据不受影响 |
| 4 错误码映射口径 | **核对无误，无动作** | 复核：`cognitive-kernel/src/error.rs` 常量三元组有测试逐条钉扎 `errors.yaml`；`10-rust-kernel.mdc` 点名 `STATE_CONFLICT` 为非法迁移码（guard/evidence 缺失同收，state 域唯一拒绝码，口径已在 KRN handoff §4.4 落档）；`effect@OUTCOME_UNKNOWN` 特判被 `effect-state-closure-008.json` 期望钉扎且 CFR 行为执行已验证。更细粒度码属 v0.1 后注册面议题，维持不动 |

交付批次：提交 `539ec05`（项 1 codegen 扩集 + 生成物 + 计数钉扎）→ 本 handoff 批（ADR-0006 第 6 条、台账 D-017/D-018、PROGRESS、本 handoff）。

## 2. 未完成 / 进行中

- **kernel 侧换生成绑定（归 Lane-KRN，本车道不碰）**——可替换点：`crates/cognitive-kernel/src/engine.rs` 手写 serde 组装 `state-transition-record.schema.json` / `state-transition-request.schema.json` 形状处，可换 `cognitive_contracts::generated::{state_transition_request::StateTransitionRequest, state_transition_record::CommittedStateTransitionRecord}`（根类型名按 schema title；成员按字母序；`metadata` = `Option<serde_json::Map>`；digest 钉扎可消费模块 `SCHEMA_DIGEST` 常量）。建议随 KRN M3 批顺带（非阻塞）。
- D-017 触发条件监视（M4/M5 批注意）；D-018 评估批（M3 出口评审后，协调者排入 CTR）。
- TSC 侧无新替换点（transition 对不属客户端面；`SCHEMA_DIGESTS` 聚合 28→30 为 additive，sdk-ts 无计数钉扎，零影响）。

## 3. 测试与证据状态

- Rust：contracts 全绿（15 单元 + 9 schema-contract + 6 generated-types 含 30-模块计数钉扎 + 6 projection + 2 golden）；workspace build/test/clippy -D warnings/fmt --check 绿（含 kernel/store/conformance 行为套件，rusqlite 以 llvm-mingw CC/AR 绝对路径本机编译）。
- TS：pnpm -r build/test 全绿 = 110 测试（contracts-ts 33 / tools 2 / sdk-ts 63 / agent-shell 12），sdk-ts/agent-shell 零触碰。
- 静态门：check-consistency OK（273/55/**60**/**81** 不变）；gen-matrix --check 无 drift（registry/vector 零变化，matrix 未动）；static_check.py ALL CHECKS PASSED（计数钉扎未触碰）；validate-manifest OK。
- runner：**81/31/0/0/0/50 与 self-check 12/12 维持不变**（本批无向量/schema 计数变化，钉扎零调整；本地复跑确认）。
- codegen regenerate-diff：空（`contracts-codegen` + `cargo fmt --all` 后生成目录干净）。
- golden：canonical 面零变化（无 schema 增删 → live schema-bundle digest 不变），fixture 零改动；双语言 emit 本地 byte-identical。
- 向量：**零改写、零增删**（81 份原样）。

## 4. 未决风险与漂移

- **D-017（deferred-to-v0.2）、D-018（决策落档）已登记**，判定依据全文入台账；无其他新漂移。
- 本机磁盘紧张（D 盘 ~5.4GB 余）：target 全量重建后一次 `cargo test` 曾因内存/临时空间分配失败，`TMP/TEMP` 指到 `D:\tmp` + `-j 2` 后稳定复现绿；接续会话空间不足时只清本 worktree `target`。
- 本机工具链复述（不影响 CI）：rusqlite(bundled) 需 `CC`=llvm-mingw `x86_64-w64-mingw32-gcc.exe` 绝对路径 + `AR`=`llvm-ar.exe`，其 bin 目录不得前置 PATH；dlltool-shim 仍前置。
- 并行通告：Lane-KRN M3 进行中（domain/store/kernel + Cargo.lock）；本批未触碰任何 KRN 属地文件，Cargo.lock 亦零变化（无依赖增删），预期合并零冲突；若 KRN 先进 main，后合并方按 PARALLEL-LANES §2.3（预计仅 PROGRESS/ledger）。

## 5. 下一步入口

- **Lane-KRN**：M3 治理链继续；顺带可做 engine.rs 换生成绑定（§2 替换点）。
- **Lane-CTR 下批**：M3 出口评审后执行 D-018 评估批；F-011 R1 审批合同登记（M5 入口 gate 阻断项）届时一并排程。
- **Lane-CFR**：M3 落地后行为执行扩展批（security-negative/context 向量）。
- 第一个动作（任意接续车道）：`git fetch origin; git merge origin/main`，读 PROGRESS 车道表触碰通告。

## 6. 快照

- PROGRESS 已更新：是（最后更新行、漂移行 D-017/D-018、车道表触碰通告、handoff 列表）。
- 本次提交：`539ec05`（codegen 扩集批）→ 本 handoff 批（ADR-0006 + ledger + PROGRESS + handoff；哈希见 git log）。基线 `2dc3e02`。
