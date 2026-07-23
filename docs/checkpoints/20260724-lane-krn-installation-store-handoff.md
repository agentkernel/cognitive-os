# 20260724 Lane-KRN durable InstallationStore handoff

## 1. 本次会话完成

- 在 `cognitive-store` 提供 `SqliteInstallationStore` 和
  `InstallationCommit`：SQLite WAL、`synchronous=FULL`、暂存到提交的单事务
  提升、仅已提交记录可读、提交记录 append-only。
- 增加四项先失败后实现的验收/回归测试：第二句柄在提交前不可见且提交后完整
  可见；重开后的受控恢复清理 interrupted staging；普通 reader 不会清理 live
  staging；已提交包不能被后续暂存覆盖。
- 关联：D-020（不新增 installation transition table）、M6 durable-install
  non-claim、Pi integration plan P3。此批没有新增 REQ、错误码、schema、向量或
  Profile。

## 2. 未完成 / 进行中

- Lane-RUN 必须为安装生命周期提供独占恢复调用，并让 management authority 在
  provenance、sandbox、compatibility 和 deterministic authorization 成功后才消费
  committed record。当前内存 `InstallationLedger` 仍不能称 durable authority。
- Pi P2 supply-chain verifier 缺可信 signature/provenance；P4 Linux-native
  sandbox 负例、P5 lifecycle/I/O mediation、P6 governed installation 都尚未完成。

## 3. 测试与证据状态

- 先失败证据：新增测试在实现前因缺少 `InstallationCommit` /
  `SqliteInstallationStore` 编译失败；不可覆盖测试先收到非 `Conflict` 错误，后
  修正为 fail-closed conflict。
- 本地：`cargo test -p cognitive-store` 全绿（全部 crate/unit/integration
  测试）；`cargo clippy -p cognitive-store --all-targets -- -D warnings` 全绿；
  `cargo fmt --check` 全绿。
- 向量：未运行、未改动；最新已合入 pin 仍为 85 total / 60 pass / 25 not-run / 0
  fail，self-check 41/41。此 store 测试不是 conformance/Profile 证据。
- CI：PR #77（Pi candidate boundary）Windows/Ubuntu 绿；`origin/main@50b499e`
  的 merge CI 随后完成成功（run 30033108062）。本 KRN 批仍须经过自己的两 OS CI。

## 4. 未决风险与漂移

- 无新 findings-ledger 漂移：本批为已计划的 adapter/persistence 实现，不触碰
  normative assets。D-020 的“不新增安装迁移表”严格保留。
- `recover_interrupted_staging` 必须由未来有独占 installation-lifecycle lock 的
  manager 调用；普通 reader 的 `open` 不会清理活跃 writer 的 staging。
- SQLite durability 仅作为本地 store 行为证据；未验证跨进程安装 authority、真实
  断电、供应链可信 provenance 或 OS containment。

## 5. 下一步入口

- 建议提示词：`docs/prompts/lane-run.md`（新独立 Lane-RUN 批，先写 runtime
  consumption/recovery/authority 失败测试）。
- 工作分支：`lane/krn-installation-store`。
- 第一个动作：审计 `crates/cognitive-runtime/src/installer.rs` 的
  `InstallationLedger`，设计一个只消费 `SqliteInstallationStore` committed record
  的明确端口；不要在 RUN 批改 KRN store 语义。

## 6. 快照

- PROGRESS 已更新：是。
- 本次提交列表：`fe10751`（KRN durable InstallationStore code + tests）；本 handoff
  与计划同步将随 documentation-close commit 提交。
