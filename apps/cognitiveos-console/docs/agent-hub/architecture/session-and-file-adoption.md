# Agent Hub — Session 采用与文件观察

> 类别：informative architecture ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> Provider 接口事实与查询日见 [../sources/provider-interfaces-ledger.md](../sources/provider-interfaces-ledger.md)。全部能力 `not-implemented / evidence none`。

## 1. 官方 session 采用（L3）

### 1.1 流程

1. **候选发现**：官方 list（如 `codex resume` 列表、OpenCode session list API、Claude SDK resume handle）或 opt-in 文件枚举（只作候选，不作 adopt 依据）。
2. **验证**：session 归属账号/workspace、schema/版本兼容、旧 runtime 是否 inactive。
3. **采用决策**（用户确认 + 预览）：
   - 旧 writer inactive（进程已退出且无其他 controller）→ 可写 adopt；
   - 供应商提供 exclusive lease/fencing → 按供应商机制取得后可写 adopt；
   - 两者皆无 → 只读 hydrate（`read-only-observed`），不写。
4. **登记**：provider-native session handle + ownership generation 入 Host ledger。
5. **release/handoff**：显式动作，推进 generation，记录去向。

### 1.2 规则

- 禁止双 writer：无法证明单 writer 即不得写。
- resume/fork 语义按 provider 区分：fork 产生新 session ID 时必须显示新旧 ID 映射；不得把 fork 显示为“继续原 session”。
- history hydration 是读取官方 session 数据构建视图；hydrated 历史仍是 `provider-reported`/`file-observed`，不是 Host 见证过的事实。
- adopt 不改变账号边界：session 属谁的 provider 账号就用谁的凭据语义，不做跨账号移植。
- provider 端 completed/archived session：只读打开，不提供写动作。

## 2. native 文件只读发现（L5）

### 2.1 范围与 opt-in

- 默认关闭；用户逐 provider opt-in，并显示 documented root（如 `~/.codex/sessions`、`~/.claude/`、OpenCode storage 目录）、读取内容类别与隐私影响。
- 只读 metadata/摘要优先；敏感字段（token、cookie、API key、email、组织 ID）在 parser 层裁剪，不落 Host ledger。
- 不读取：credential/auth 文件、entitlement、provider 内部数据库可写副本、非 documented 路径、其他 OS 用户数据。

### 2.2 打开前检查

real path 解析（拒绝 symlink/reparse point 跳出 documented root）、owner、mode/ACL、hardlink 计数异常、大小上限、文件类型（拒绝设备文件/FIFO）、跨用户边界、路径穿越。Windows 额外处理 reparse point 类型（symlink/junction/OneDrive 等 cloud placeholder）；cloud placeholder 不触发 hydration 的读法（`FILE_FLAG_OPEN_REPARSE_POINT` 元数据模式）或明示会触发下载。

### 2.3 一致性 snapshot

- **JSONL**（Codex/Claude Code session 常见格式）：只接受完整换行结束的记录；尾部半行按 `partial` 挂起，等待补全；rotation/gap 记录序号断点。
- **SQLite**（OpenCode 等）：只读连接 + WAL 模式下开启 read transaction 取一致 snapshot；`-wal`/`-shm` 不同步时标 `version-unknown`；不对 provider DB 执行任何写（包括不无意触发 WAL checkpoint 的写模式打开）。
- 每个 snapshot 记录：digest、文件 identity（dev+ino / file ID）、mtime、大小、parser 版本、schema 版本判定。
- parser 版本化：schema 未知/字段未知 → `session-schema-unknown`，只显示已识别子集并标注；不猜测语义。

### 2.4 watch 与 freshness

- 文件 watch（ReadDirectoryChangesW / FSEvents / inotify）只驱动 resnapshot 调度；UI freshness 显示 last-good snapshot 时间。
- watch 丢事件（队列溢出）→ 全量 rescan；不假设无变化。

## 3. L6（documented file write）为什么 v1 阻断

已冻结决策（[CONSOLE-AGENTHUB-V1-DEC-011](../decisions/decision-log.md)）：当前六个 Tier 1 provider 均未公开提供带版本/并发控制/CAS/rollback/migration 承诺的外部 session 写协议；在缺乏供应商承诺时写 provider 文件等于伪造 provider 状态，破坏其内部一致性并可能违反条款。触发条件（全部满足才可解锁为未来能力）：

1. 供应商公开文档承诺外部写协议（格式、版本、并发、迁移）；
2. 提供并发控制原语（lock/lease/CAS）；
3. 提供损坏恢复/rollback 路径；
4. 条款允许第三方写；
5. 本仓库完成独立威胁建模 + Open PoC 实证。

## 4. Open PoC（session/file)

全部 `not-run / evidence none`：

- `POC-SESS-001` 各 Tier 1 官方 resume/fork/list 在两个连续版本间的行为与 schema 漂移记录；
- `POC-SESS-002` 旧 writer 存活时写 adopt 的实际冲突后果（在隔离沙箱、非用户数据上）；
- `POC-FILE-001` JSONL 半写/rotation/truncate 注入样本的 parser 处置；
- `POC-FILE-002` SQLite WAL 只读 snapshot 在写压力下的一致性；
- `POC-FILE-003` symlink/reparse/junction/cloud placeholder 逃逸样本被拒率。
