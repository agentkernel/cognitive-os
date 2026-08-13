# P2-T19 Memory/Skill Agent consumer — running validation report

- Task: `P2-T19`
- Branch: `personal/P2-T19-memory-skill-consumer`
- Base: P2-T13 head `b514d278ef4a3daafe9cceeb62ced2dc649d186b`; PR #210 was
  unmerged when the sibling worktree was created and merged afterward
- Lease: `lease/personal/P2-T19/memory-skill-agent-consumer`
- Change class: `implementation-only` unless an explicit public contract change
  is later required
- Claim ceiling: implementation evidence only; no Gate, release, Profile, B08,
  EVAL or Agent-benefit claim

本文件是本任务唯一的增量验证报告。每个已完成单元在下一个单元开始前追加记录；已发布
结果只通过追加的 superseding entry 更正。

## 预登记验证路由

- 本地 `DEV-WIN-GNU-01`：只运行 `cargo fmt`、静态一致性、Node、handbook、
  docs-sync 与 diff 检查；不运行 Rust build/test/Clippy。
- Rust 主验证：已推送精确 revision 的 GitHub Ubuntu/Windows required CI。
- `DEV-LINUX-NATIVE-01`：仅在 CI 不能覆盖 native/restart 语义时，从已推送精确
  revision 建立本任务自己的可清理 clone；不触碰 B01 guest/campaign roots。

## 增量结果

### D01-TEST-01 — failure-first governed Memory/Skill consumption

- Instrument:
  `personal::scheduler_authority::tests::admitted_task_context_consumes_authorized_memory_and_exact_skill_pin`
- Fixture: 一个已持久化当前 epoch TaskContract/ContextRequest、一个同 workspace/purpose
  的 admitted Memory、一个绑定到该 Task 的 exact Skill revision。
- Oracle: 生产 `resolve_authorized_task_context` 必须在候选型 Pi 之前返回同时绑定
  Memory identity 与 Skill binding identity 的受治理片段。
- Initial status: `not-run`；本地 Windows GNU 不支持 Rust linking，预期失败将由首个已推送
  精确 revision 的 required CI 观察并追加。该状态不是 pass，也不关闭 D01。

### D01-FMT-01 — local Rust formatting

- Instrument: `cargo fmt --all`
- Environment: `DEV-WIN-GNU-01`
- Revision: uncommitted D01 worktree based on `b514d278ef4a3daafe9cceeb62ced2dc649d186b`
- Outcome: `pass`；命令退出码 0。
- Disposition: 仅为格式证据，不是 Rust build/test、Slice 完成或产品声明。

### D01-DIFF-01 — patch whitespace validation

- Instrument: `git diff --check`
- Environment: `DEV-WIN-GNU-01`
- Revision: uncommitted D01 worktree based on `b514d278ef4a3daafe9cceeb62ced2dc649d186b`
- Outcome: `pass`；无输出，退出码 0。
- Disposition: 仅证明当前补丁没有 Git 空白错误。

### D01-CONSISTENCY-01 — repository consistency

- Instrument: `pnpm run check:consistency`
- Environment: `DEV-WIN-GNU-01`, fresh sibling worktree
- Revision: uncommitted D01 worktree based on `b514d278ef4a3daafe9cceeb62ced2dc649d186b`
- Started/retained: 1/1
- Outcome: `not_available`；检查器启动前因该新 worktree 尚无 `node_modules`，Node 报
  `ERR_MODULE_NOT_FOUND: ajv`。没有一致性断言被执行。
- Disposition: 这不是产品或一致性失败；依赖安装后必须重跑，当前结果不得记为 pass。

### D01-CONSISTENCY-02 — repository consistency after dependency install

- Instrument: `pnpm run check:consistency`
- Environment: `DEV-WIN-GNU-01`
- Revision: uncommitted D01 worktree based on `b514d278ef4a3daafe9cceeb62ced2dc649d186b`
- Started/retained: 1/1
- Outcome: `fail`；检查器执行并报告 10 项任务登记问题：当时的 P2-T16 全局汇总未更新、
  `PROGRESS.md` 未登记 D01–D05 状态、lease 日期格式不合约定，并因该格式误判 lease
  非 active/current。
- Disposition: 均为本任务可修复的登记缺口；在下一验证单元前修复并重跑，不弱化检查器。

### D01-CONSISTENCY-03 — repository consistency after registration repair

- Instrument: `pnpm run check:consistency`
- Environment: `DEV-WIN-GNU-01`
- Revision: uncommitted D01 worktree based on `b514d278ef4a3daafe9cceeb62ced2dc649d186b`
- Started/retained: 1/1
- Outcome: `fail`；前 9 项已消除，仅剩 active-table 行被解析为 non-active lease。
- Disposition: 定位该行格式与检查器约定的差异后原地修复；不跳过门禁。

### D01-CONSISTENCY-04 — repository consistency after final lease repair

- Instrument: `pnpm run check:consistency`
- Environment: `DEV-WIN-GNU-01`
- Revision: uncommitted D01 worktree based on `b514d278ef4a3daafe9cceeb62ced2dc649d186b`
- Started/retained: 1/1
- Outcome: `pass`；275 requirements、55 errors、74 schemas、89 vectors，以及
  Personal plan/Gates、trace、delivery slices 与 leases 全部通过。
- Disposition: D01 登记一致性已闭合；Rust failure-first 行为仍等待 exact pushed CI。

### D01-DIFF-02 — pre-commit patch whitespace validation

- Instrument: `git diff --check`
- Environment: `DEV-WIN-GNU-01`
- Revision: complete uncommitted D01 candidate based on
  `b514d278ef4a3daafe9cceeb62ced2dc649d186b`
- Outcome: `pass`；无输出，退出码 0。
- Disposition: 首个 checkpoint 的完整补丁没有 Git 空白错误。

### D01-DOCSYNC-01 — staged documentation synchronization gate

- Instrument: `node tools/src/docs-sync-gate.mjs --staged`
- Environment: `DEV-WIN-GNU-01`
- Revision: staged D01 candidate based on
  `b514d278ef4a3daafe9cceeb62ced2dc649d186b`
- Started/retained: 1/1
- Outcome: `pass`；scheduler test 映射到三组 handbook 文档；以
  `DOCS_IMPACT_NONE="Failure-first task registration and expected-failing test add no shipped behavior or public documentation"`
  诚实声明本 checkpoint 尚无已交付行为。双语 handbook 54×2 检查与 18 个生成页
  byte gate 同时通过。
- Disposition: 该理由必须进入 commit/PR；实现行为落地时必须真正同步双语页面与指纹。

### D01-ID-01 — concurrent task-id collision supersession

- Observation revision: pushed checkpoint
  `9ead35a30f6fa491d1f164250f601e85b7b11b59`
- Outcome: `partial`；该 checkpoint 诚实记录了当时无冲突的 P2-T16 allocation，但在
  Draft PR 创建前发现并行 sibling branches 已登记 P2-T16、P2-T17 与 P2-T18。
- Superseding disposition: 不改写已推送历史；当前任务、lease、branch、slice 与本报告
  统一改为下一非冲突 ID `P2-T19`。旧 P2-T16 branch 不用于 PR 或后续实现。

### D01-CONSISTENCY-05 — consistency after P2-T19 supersession

- Instrument: `pnpm run check:consistency`
- Environment: `DEV-WIN-GNU-01`
- Revision: uncommitted P2-T19 supersession over
  `9ead35a30f6fa491d1f164250f601e85b7b11b59`
- Started/retained: 1/1
- Outcome: `pass`；275 requirements、55 errors、74 schemas、89 vectors，以及
  task/slice/lease/current-snapshot 关系全部通过。
- Disposition: P2-T19 是当前唯一操作 ID；继续使用同一 failure-first 行为测试。

### D01-FMT-02 — formatting after P2-T19 supersession

- Instrument: `cargo fmt --all -- --check`
- Environment: `DEV-WIN-GNU-01`
- Revision: uncommitted P2-T19 supersession over
  `9ead35a30f6fa491d1f164250f601e85b7b11b59`
- Outcome: `pass`；无输出，退出码 0。
- Disposition: 格式通过，不替代 Rust build/test。

### D01-DIFF-03 — P2-T19 supersession patch whitespace

- Instrument: `git diff --check`
- Environment: `DEV-WIN-GNU-01`
- Revision: complete uncommitted P2-T19 supersession over
  `9ead35a30f6fa491d1f164250f601e85b7b11b59`
- Outcome: `pass`；无输出，退出码 0。
- Disposition: 重编号补丁没有 Git 空白错误。

### D01-DOCSYNC-02 — P2-T19 supersession staged gate

- Instrument: `node tools/src/docs-sync-gate.mjs --staged`
- Environment: `DEV-WIN-GNU-01`
- Revision: staged P2-T19 supersession over
  `9ead35a30f6fa491d1f164250f601e85b7b11b59`
- Outcome: `pass`；双语 handbook 54×2 与 18 个生成页检查通过；以
  `DOCS_IMPACT_NONE="Concurrent task-ID supersession changes identifiers only; shipped behavior and public documentation remain unchanged"`
  记录纯协调更正。
- Disposition: 实现行为提交仍必须同步实际受影响的双语 handbook 与指纹。

### D01-CI-UBUNTU-01 — failure-first Ubuntu observation

- Instrument: GitHub CI `verify (ubuntu-latest)`, run `31718358206`, job
  `94508750827`
- Revision: `19767e363e0eda5c1bd7d851d1340151a5180a68`
- Started/retained: 1/1
- Outcome: `fail`；job 在 1m55s 后完成失败，满足 failure-first 必须先观察红灯的边界。
  workflow 尚在等待 Windows job，GitHub 暂不提供失败日志，因此具体断言将在 run 完成后
  以追加条目分类。
- Disposition: 不将红灯写成实现回归或 pass；等待同一 run 完成并确认失败来自新增
  governed-consumption oracle，再开始 D02。

### D01-CI-UBUNTU-02 — failure-first observation retained

- Instrument: GitHub CI run `31718358206` at `19767e36`
- Outcome: `fail` retained as the D01 oracle. D01 is closed as the observed
  missing-consumer baseline; implementation continues in D02.
- Disposition: 该红灯不得改写为 pass。

### D02-FMT-01 — local Rust formatting

- Instrument: `cargo fmt --all -- --check`
- Environment: `DEV-WIN-GNU-01`
- Outcome: `pass`；无输出，退出码 0。
- Disposition: 仅为格式证据，不是 Rust build/test。

### D02-DIFF-01 — patch whitespace validation

- Instrument: `git diff --check`
- Environment: `DEV-WIN-GNU-01`
- Outcome: `pass`；无输出，退出码 0。
- Disposition: 当前 D02 补丁没有 Git 空白错误。

### D02-HANDBOOK-01 — bilingual handbook and fingerprints

- Instrument: `node tools/src/fill-handbook-fingerprints.mjs` then
  `node tools/src/check-handbook.mjs` and `node tools/src/generate-handbook.mjs --check`
- Environment: `DEV-WIN-GNU-01`
- Outcome: `pass`；54×2 handbook 检查通过，18 个生成页 byte-identical。
- Disposition: 已同步受治理消费行为与 v24 迁移说明；不得使用 `DOCS_IMPACT_NONE`。

### D02-CONSISTENCY-01 — repository consistency

- Instrument: `pnpm run check:consistency`
- Environment: `DEV-WIN-GNU-01`
- Outcome: `pass`；275 requirements、55 errors、74 schemas、89 vectors，以及
  task/slice/lease/current-snapshot 关系全部通过。
- Disposition: 登记一致性已闭合；Rust 行为仍等待 exact-revision Linux/CI。

### D02-RUST-01 — focused governed-consumer tests

- Instrument:
  `admitted_task_context_consumes_authorized_memory_and_exact_skill_pin`,
  `forgotten_memory_and_revoked_skill_fail_closed_on_reuse`,
  `revoked_skill_and_digest_mismatch_fail_closed`,
  `digest_mismatch_on_durable_consumption_record_fails_closed`,
  `session_two_reuses_durable_pins_without_restating_memory_or_skill`
- Environment: `DEV-WIN-GNU-01`
- Outcome: `not-run`；`RUST-LINK-DEV-WIN-GNU-01` 禁止本机 GNU 链接。
- Disposition: 推送后在 exact-revision `DEV-LINUX-NATIVE-01` 与 required CI 观察。

### D02-RUST-02 — native compile observation at `0c3520cc`

- Instrument: `cargo test -p kernel-server --locked` focused filter on
  `DEV-LINUX-NATIVE-01` disposable worktree `~/p2-t19-msconsumer`
- Revision: `0c3520cce78975e9d6744dd75e30a8ef64836e3f`
- Started/retained: 1/1
- Outcome: `fail`；三个负例 `matches!` 模式把 `detail` 移出后再在断言消息里借用
  `forgotten`/`revoked`/`mismatched`，触发 `E0382`。不是产品语义失败。
- Disposition: 已改为 `ref detail`；该红灯不得写成消费行为回归。

### D02-RUST-03 — native focused scheduler_authority tests at `627885cd`

- Instrument: `cargo test -p kernel-server --locked scheduler_authority::tests`
- Environment: `DEV-LINUX-NATIVE-01` disposable worktree `~/p2-t19-msconsumer`
- Revision: `627885cd428bce62770c4683a6653f3b4ae49ec3`
- Started/retained: 1/1
- Outcome: `fail`；52 passed, 3 failed. Skill revoke and forged-digest negatives
  passed. D01/session-2/forget first-resolve failed because Memory shared the
  workspace source digest+body and was omitted as `DUPLICATE_CONTENT_DIGEST`.
- Disposition: 受治理钉必须替换相同正文的普通源并进入 required 集；不是削弱
  去重不变量。

### D02-RUST-04 — native retest after identity preservation at `cebf64fe`

- Instrument: `cargo test -p kernel-server --locked scheduler_authority::tests`
- Environment: `DEV-LINUX-NATIVE-01` `~/p2-t19-msconsumer`
- Revision: `cebf64fe9250dacd482e3bb61385161b565aca96`
- Started/retained: 1/1
- Outcome: `fail`；D01、forget/revoke 与 digest-mismatch 已绿。仅
  `session_two_reuses_durable_pins_without_restating_memory_or_skill` 失败：
  第二会话 `CONTEXT_INCOMPLETE`，因为 Memory 候选仍绑定第一会话
  `conversation_ref`，预过滤后 required 集无法闭合。
- Disposition: 复用路径按当前会话重写 Memory governance conversation，不改钉或
  digest。
