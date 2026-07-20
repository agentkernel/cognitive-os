# 20260720 Lane-KRN Handoff（M2 对象/状态/事件内核批）

## 1. 本次会话完成

按 `docs/prompts/milestone-m2.md` / `docs/prompts/lane-krn.md` 交付 M2 内核批（分支 `lane/krn`，基线 `31f9f4f` = M1 CFR runner 批 merge）。三 crate 全部从空骨架落到实现 + 行为测试；提交哈希见 §6。

- **`cognitive-domain`（纯域层，无 I/O）**：
  - `transitions.rs`：五份 `specs/transitions/*.transitions.json` 以 `include_str!` 编译期嵌入（单一来源，零手抄常量），进程内一次解析 + 结构校验（状态唯一/初始态与终态闭合/终态无出边/`(from,to,reason)` 选择器无歧义），每表钉扎 canonical digest（`spec-set/0.1` 域，与 conformance runner 的 spec-set 资产 digest 公式逐字节一致）；`LifecycleDomain` 枚举 + 表行查找（`find_edge`/`legal_exits`/`is_terminal`）。附「嵌入副本 == 磁盘注册资产」digest 钉扎测试。
  - `ids.rs`：ObjectId/EventId/RecordId/BudgetId（小写 canonical UUIDv7 校验，REQ-GOBJ-ID-001）、UriRef、StateName/ReasonCode（表文法）、WallTimestamp（canonical RFC 3339 UTC 形式，复用 contracts projection 校验）。`version.rs`：logical_version newtype（1..=2^53-1，`next()` 恰 +1，溢出拒绝；ADR-0005 时钟域分型）。
- **`cognitive-kernel`（确定性核心）**：
  - `ports.rs`：`AuthorityStore`（原子提交单元 = 对象 CAS + 事件 append + 迁移记录 + 预算扣减 + outbox）、`Clock`、`IdGenerator` 端口 trait 及纯 DTO——无 SQLite/HTTP/模型类型。
  - `engine.rs`：集中 transition 入口（唯一 authority 写路径）：表 digest 钉扎校验 → 权威状态/版本 CAS 前置比对 → 表行 + reason 匹配 → guard 全数确定性 attests（缺失/false 即 fail-closed）→ required_evidence 在场校验 → 硬预算确定性准入 → 组装 `state-transition-record.schema.json` 形状的 canonical 记录与事件值 → 单事务提交。拒绝携带权威 current state/version + 安全出口（标准 §3）。`admit_object`（初始态 version 1 + 准入事件同事务）、`create_budget`。
  - `error.rs`：**注册错误码单点映射**（详见 §4 映射口径）：版本失配/非法迁移/guard/evidence 缺失 → `STATE_CONFLICT`；effect 处于 `OUTCOME_UNKNOWN` 的非法出口 → `EFFECT_OUTCOME_UNKNOWN`（对齐向量 `effect-state-closure-008.json` 期望）；表钉扎失配 → `DIGEST_MISMATCH`；超预算 → `RESOURCE_BUDGET_EXHAUSTED`；提交路径不可持久化 → `STATE_STORE_UNAVAILABLE`。常量三元组（code/category/retryable）有测试逐条钉扎 `specs/registry/errors.yaml`。
  - `budget.rs`：AttentionBudget 九维硬预算纯整数准入（fail-closed、无部分扣减、维度白名单 = `common-defs.schema.json#/$defs/budget`）。
  - `replay.rs`：committed 事件历史确定性重放 → 状态投影 canonical bytes + digest（REQ-STATE-002）；未知事件类型/版本缺口/before_state 失配/列与事件值分歧 = replay barrier（标准 §6），绝不猜。
- **`cognitive-store`（SQLite WAL 适配器，ADR-0002 五条绑定规则全实现）**：
  - `sqlite.rs`：`SqliteAuthorityStore`（单写者连接 + Mutex；WAL+`synchronous=FULL` 开库断言；STRICT 表）；**append-only 在存储层强制**：`events`/`transition_records` 上 BEFORE UPDATE/DELETE 触发器 RAISE(ABORT)，对任意连接生效（REQ-EVT-004）；CAS = `UPDATE ... WHERE version=?expected`（零行 → Conflict 无副作用）；对象 CAS+预算扣减+事件+记录+outbox 单事务；一切提交失败 → `Unavailable`（内核映射 `STATE_STORE_UNAVAILABLE`）fail-closed，无内存缓冲（REQ-REC-003）；`open_read_only`（读者快照 / 降级卷模型）。
  - `clock.rs`：SystemClock → canonical UTC（civil-from-days 手写换算，无新依赖；毫秒去尾零）。`ids.rs`：`uuid` crate v7 生成器（RFC 9562）。
- **工作区**：根 `Cargo.toml` workspace 依赖新增 `rusqlite 0.40.1（bundled）`/`uuid 1.24（v7）`/`tempfile 3`（仅 store 消费）；`Cargo.lock` 更新。`KERNEL_PORTS` 占位常量改为真实端口能力面（authority-store/event-log/outbox/clock/id-generator——event-log 与 outbox 是 AuthorityStore 的同事务能力，不是独立提交存储），保持 runtime/management/kernel-server 占位断言不跨车道改动。
- **文档联动**：matrix 回填 5 个 REQ 的 impl/impl_tests/evidence/notes（REQ-STATE-002/003、REQ-EVT-004、REQ-REC-003、REQ-GOBJ-ID-001；再生成 + `--check` 绿）；PROGRESS（M2 行、车道表、计数、handoff 列表）；本 handoff。

## 2. 未完成 / 进行中

- **M2 出口评审未做（有意）**：里程碑评审等 Lane-CFR 行为向量执行批（runner 对 `state-conflict`、`effect-state-closure-008`、`state-store-degradation` 行为侧等真实执行）之后由协调者安排；本批四类状态用语 =「实现已提供 + Rust 行为测试已执行」，**不声称任何向量 pass**（向量计数保持 25/51 不变，runner 未动）。
- 事件值目前是实现事件（携带迁移事实 + 表钉扎 + causation），非 `event.schema.json` 完整 envelope（需 GovernedObjectHeader 全治理字段）——等 M3 治理链 + M5 运行时上下文再升格（§4 缺口 3）。
- 请求幂等（同 request_id 重放）未做——属 M4 Intent/Effect 幂等域，勿在 M2 补。
- 深层 ADR-0005 合规测试（时钟回拨、same-tick 单调、租约不可延展）归 M4 恢复批。

## 3. 测试与证据状态

- **Rust（本地 + 待 CI 复核）**：`cargo build/test/clippy -D warnings/fmt --check` 全绿；工作区 106 测试通过，其中本批新增 51：domain 17（含表加载/终态/歧义选择器/腐蚀资产负例）+ kernel 单元 9（预算 5/错误映射 2/钉扎 2）+ kernel 行为 10（`tests/engine_gate.rs`，fake 端口）+ store 单元 7 + **M2 验收套件 8（`crates/cognitive-store/tests/m2_acceptance.rs`，真 SQLite）**。
- **验收判据 ↔ 测试对照**（每条含负例，判据编号按任务书）：
  1. 并发 CAS 单胜者 → `criterion_1_concurrent_cas_exactly_one_winner_others_state_conflict`（8 线程 barrier 真并发，1 胜 7 败全 `STATE_CONFLICT`，事件/记录恰 +1）；
  2. 非法迁移全拒 → `criterion_2_every_unregistered_pair_rejected_with_registry_codes_and_state_unchanged`（五表全体非法有序 `(from,to)` 对穷举 >400 例，状态/版本不变、零追加，错误码与 registry 一致）+ kernel 层同型穷举 + 全体合法边提交正例（`every_registered_edge_commits_through_the_gate`，91 边全过）；
  3. 投影重放 digest 稳定 → `criterion_3_replaying_committed_history_yields_byte_identical_projection_digests`（同库重放两次 byte-identical；重开句柄同 digest；固定 clock/ids 下两独立库收敛同 digest）；
  4. 事件不可原地改 → `criterion_4_committed_events_and_records_reject_update_and_delete`（裸连接 UPDATE/DELETE/时间戳改写 6+1 式全拒，历史 digest 不变）；
  5. 预算 fail-closed + 同事务扣减 → `criterion_5_over_budget_rejected_fail_closed_and_debit_commits_atomically`（超额拒绝无任何写、扣减与状态同 commit、耗尽后再拒）+ kernel 层 `hard_budget_admission_is_deterministic_and_rides_the_commit`；
  6. crash 一致性初步 → `criterion_6_mid_transaction_failure_leaves_no_partial_commit`（对象 CAS 与预算扣减已在事务内执行后注入事件 append 冲突 → 整体回滚，四表零残留，`STATE_STORE_UNAVAILABLE`；故障移除后同命令可提交）+ `criterion_6_read_only_store_fails_closed_and_keeps_reads_available`（只读降级卷：写全拒 `STATE_STORE_UNAVAILABLE`、读路径存活、无缓冲、历史 digest 不丢、恢复后同写可提交——`state-store-degradation` 行为侧的 M2 子集）。
- **向量**：0 变化（pass 25 / not-run 51 保持；conformance runner 与 CI 钉扎计数未触碰，本地复跑报告 sha256:5eb3150b...abf0 与 M1 一致）。
- TS/工具：`pnpm -r build/test` 绿（75 客户端测试 + tools）；`check:consistency` OK（273/55/56/76）；`gen-matrix --check` 无 drift。
- CI：push + PR 后 Windows+Linux 矩阵全绿方可合并（合并事实见 PR）。

## 4. 未决风险与漂移（含交 Lane-CTR 契约缺口清单）

无规范资产间漂移登记（findings-ledger 未动）。以下为**契约缺口**，登记待 Lane-CTR 处置，本车道未越界自改：

1. **state-transition-request/record 无生成绑定**：codegen（ADR-0006 IMP-08 集）未覆盖这两份 schema；kernel 以手写 serde 组装 schema 形状 canonical JSON（`engine.rs` 记录/事件组装处）。建议 CTR 评估纳入 codegen 或登记豁免理由。
2. **状态投影无注册合同/digest 域**：`event-audit-watch.md` §3 要求重放 digest 稳定，但未注册投影形状与 digest 域。kernel 暂用实现域 `cognitiveos.impl.execution-status-projection/0.1`（projection_version 与 digest 域同串，`replay.rs` 有注释声明）。字节稳定性不受影响；若 CTR 注册正式投影合同，本车道跟进换域（修正型）。
3. **事件 envelope 升格路径**：`event.schema.json` 的 Event 要求完整 GovernedObjectHeader（owner/authority/scope/retention 等治理引用），M2 无治理链可填。当前事件日志行是实现事件值；升格计划：M3 治理链落地后由 CTR 决定是否登记「内核内部事件」形状或直接在 M5 组装完整 envelope。
4. **错误码映射口径**（非缺口，落档防漂移）：guard 不成立 / evidence 缺失同样收 `STATE_CONFLICT`（state 域仅此拒绝码；`10-rust-kernel.mdc` 点名 STATE_CONFLICT 为非法迁移码）；如需更细粒度码属注册面冻结外的 v0.1 后议题。`effect@OUTCOME_UNKNOWN` 非法出口特判 `EFFECT_OUTCOME_UNKNOWN` 已被 `effect-state-closure-008.json` 期望钉扎。
5. **本机 Windows 工具链坑**（不影响 CI）：gnu 宿主编 `libsqlite3-sys(bundled)` 需 C 编译器——`winget install MartinStorsjo.LLVM-MinGW.UCRT`（便携免提权），构建时 `CC` 指到其 `x86_64-w64-mingw32-gcc.exe` **绝对路径**且勿把其 bin 目录前置 PATH（其内的 gcc 会顶掉 rust 链接期望的 mingw gcc 触发 `-lgcc_eh` 失败）；dlltool-shim 仍前置（getrandom raw-dylib）。LLVM.LLVM 系统安装器需提权，已失败一次，勿再试。

## 5. 下一步入口

- **CFR 行为执行批**（协调者安排）：runner 增加行为执行模式对接 `cognitive-kernel`/`cognitive-store` 公开 API；候选脱 not-run 向量：`state-conflict`（STATE-CAS-002）、`effect-state-closure-008` 行为侧、`state-store-degradation`（M2 子集：只读降级；disk-full 归 M4 故障注入框架）、`remote-completed-not-acceptance`（task 表 ACTIVE 强推 COMPLETED 拒绝 = 本批穷举已覆盖的行为）。M2 出口评审随其后。
- **Lane-KRN M3**：`docs/prompts/lane-krn.md` M3 节（治理链/capability/九阶段 Context Resolution）；入口 gate = M2 出口 + F-007 行为侧测试计划评审。
- 工作分支：`lane/krn`（本批合并后可从新 main 重建 worktree）。
- 第一个动作（M3 会话）：读 `docs/standards/authn-authz-capability.md` + `context-resolution-and-cache.md`，先写「capability 交集只缩不扩」与「撤销后缓存复用被拒」失败测试。

## 6. 快照

- PROGRESS 已更新：是（M2 行、计数、车道表、handoff 列表）。
- 本次提交（哈希以 git log 为准，按序）：domain 表驱动批 → kernel 端口/引擎/预算/重放批 → store SQLite 适配器 + 验收套件批 → docs 联动批（matrix/PROGRESS/handoff）。
