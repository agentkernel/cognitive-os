---
doc_id: user.operations-recovery
locale: zh-CN
kind: guide
audience: [user]
status: partial
generated: false
sources:
  - path: apps/kernel-server/src/personal/readiness.rs
    symbols: ["evaluate_personal_readiness"]
  - path: apps/kernel-server/src/personal/tool_lifecycle.rs
    symbols: ["handle"]
  - path: apps/kernel-server/src/personal/six_resource_doctor.rs
  - path: apps/admin-cli/src/personal_cli/daemon.rs
  - path: crates/cognitive-store/src/personal_backup.rs
    symbols: ["plan_personal_backup_inventory"]
  - path: crates/cognitive-store/src/personal_db.rs
    symbols: ["prepare_personal_databases"]
  - path: crates/cognitive-store/src/sqlite/intent_chain.rs
    symbols: ["insert_task_contract_with_execution_bootstrap"]
tests:
  - apps/kernel-server/tests/p1_t05_personal_readiness.rs
  - crates/cognitive-store/tests/p1_t01_layout_migrations.rs
fingerprint: "sha256:aec1a48ae6b28c57fde2d521fcb3ffdafb2258e01802f9e70579877f7745497b"
non_claims:
  - "`ready` 是配置/存活投影，不是实时 Provider 或端到端保证。备份/恢复今天没有可运行的命令。"
---

# 运维与恢复

## 日常检查 —— `implemented`

- `cognitive status` / `cognitive doctor`：六组件（system、database、secret、
  provider、daemon、pi），级别为 `blocked | degraded | ready`，另有
  `first_conversation_ready`。`provider` 组件会真实解析配置中的 `secret_ref`：若已存
  的密钥被移除，它报 `provider_secret_unresolvable` 并阻塞，而不会谎称 ready——重新执行
  `cognitive init` 即可再次存入密钥。一次评估对 provider、model/digest 与 secret 解析
  使用同一份已加载配置快照，因此原子替换配置不会混用两个版本的事实。一次
  status/doctor 评估还只绑定一次 SecretStore 做 secret 探针与 provider 解析——
  不保留 secret 材料，后续请求重新评估而不是靠 stale-ready TTL。doctor 追加脱敏的六资源、headless vault 与可运维性
  小节（当前为静态 `not_run`/`not_configured` 报告——更多是脱敏校验器而非实时探针）。
- `GET /personal/health`（免认证）仅是存活探测——安装器与服务控制器使用它；不要把
  readiness 读进去。
- 服务日志：`journalctl --user -u cognitiveos-personal.service`。

## 停止、重启、过期状态 —— `implemented`

`cognitive daemon stop` 向记录的 PID 发信号，仅在确认进程消失后移除 `daemon.lock`
与 endpoint 文档；看似存活的锁绝不删除。每次启动 daemon 幂等地重跑迁移、恢复已消费的
worker 交接，仅修复当前已准入合同所缺 Loop/Budget/调度前置而不重置既有行，并原子地
重新发布 endpoint。随后才启动唯一周期调度 worker；顺序退出会在释放 daemon 状态前取
消、唤醒并 join 它。

## 数据库安全 —— `implemented`

数据库位于 XDG state（`authority.sqlite`、`installation.sqlite`，WAL 模式、0600）。
每次迁移 apply 都先在 `state/backups/` 写带时间戳的备份（不自动清理）。派生数据
（Memory FTS 索引）可从权威行重建；被遗忘的 Memory 绝不会因索引重建而复活。权威库
迁移现含 v24 只追加 Memory/Skill 消费记录；后续会话可复用精确钉，但遗忘、撤销或
digest 漂移或竞争记录会失败闭合，而不是让已遗忘事实复活。management Memory/Skill
生命周期行与 Skill revision 谱系在 daemon 重启后仍可检查。

## 崩溃与未知结果恢复 —— 引擎层 `implemented`

恢复遵循固定八步序（fence 旧写者 → 重放历史 → 用**原**幂等键对账每个在途 Effect →
重授权 → 重建 context → 恢复或隔离）。确定性管理回退（`admin-cli reconcile`）不依赖
任何模型驱动同一序列——未配置执行器时，仍未知的结果会隔离（fail-safe）而非强行了结。
原生 HTTP attempt 在出站前持久化，重启后在终态 receipt 出现前保持 indeterminate。
workspace 变更使用持久原键 receipt；相同文件字节本身不是执行证明，重启会保守清理
orphan staging。

成功的 Task 准入在权威库内同样具备崩溃原子性：合同、`DRAFT` governed Task、
`START` Loop、硬 Budget 与 runnable 调度行一起出现。提交前失败不会留下这些准入成员；
成功响应后崩溃重开会看到完整发布。启动可修复缺失的旧 Task 投影，且不会重置任何既有
生命周期状态。

验证启动时，闭合 Effect pin、verification request 与 Loop `ACT -> VERIFY` 发布属于同
一崩溃原子权威事务；过期 writer 或 Loop 不会留下其中任何新成员。

Task 完成是更靠后的权威边界。daemon 先重查最新 independent passed report 及每个所引
Artifact CAS；随后 SQLite 在 candidate/acceptance 各自事务内重查 fixed Effect 版本、
完整闭合 Effect 集合、当前 epoch 与 Task CAS。两条 transition 之间崩溃会保留
`CANDIDATE_COMPLETE`，供同一 evidence-bound acceptance 重试；重复 acceptance 不能写
第二次完成。

已登记原生 Tool 默认启用。management 会话可按 `operation_id` disable、quarantine
或 revoke；Agent 暴露立即跟随该 overlay，且永不改写不可变 descriptor。普通 Task
调用方可读取当前最窄暴露集合并记录选择收据，但不能 enable、disable、quarantine
或 revoke Tool。

## 备份与恢复 —— 作为用户功能 `unavailable`

盘点/导出/恢复预检的规划代码已存在（secret 路径按设计恒排除），但尚无
`cognitive backup`/`restore` 命令或归档 I/O 接线。今天诚实的替代做法：停止 daemon，
自行复制 XDG state/config 目录，并记住 Provider key **不在**这些文件里——在新机器恢
复后需重跑 `cognitive init` 重新录入 key。
