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
  - path: apps/kernel-server/src/personal/six_resource_doctor.rs
  - path: apps/admin-cli/src/personal_cli/daemon.rs
  - path: crates/cognitive-store/src/personal_backup.rs
    symbols: ["plan_personal_backup_inventory"]
  - path: crates/cognitive-store/src/personal_db.rs
    symbols: ["prepare_personal_databases"]
tests:
  - apps/kernel-server/tests/p1_t05_personal_readiness.rs
  - crates/cognitive-store/tests/p1_t01_layout_migrations.rs
fingerprint: "sha256:2c75d38146c714c98e1f1d6c9901ad16f604606547d3543d9435f42175451128"
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
  使用同一份已加载配置快照，因此原子替换配置不会混用两个版本的事实。doctor 追加脱敏的六资源、headless vault 与可运维性
  小节（当前为静态 `not_run`/`not_configured` 报告——更多是脱敏校验器而非实时探针）。
- `GET /personal/health`（免认证）仅是存活探测——安装器与服务控制器使用它；不要把
  readiness 读进去。
- 服务日志：`journalctl --user -u cognitiveos-personal.service`。

## 停止、重启、过期状态 —— `implemented`

`cognitive daemon stop` 向记录的 PID 发信号，仅在确认进程消失后移除 `daemon.lock`
与 endpoint 文档；看似存活的锁绝不删除。每次启动 daemon 幂等地重跑迁移、恢复已消费的
worker 交接，并原子地重新发布 endpoint。

## 数据库安全 —— `implemented`

数据库位于 XDG state（`authority.sqlite`、`installation.sqlite`，WAL 模式、0600）。
每次迁移 apply 都先在 `state/backups/` 写带时间戳的备份（不自动清理）。派生数据
（Memory FTS 索引）可从权威行重建；被遗忘的 Memory 绝不会因索引重建而复活。

## 崩溃与未知结果恢复 —— 引擎层 `implemented`

恢复遵循固定八步序（fence 旧写者 → 重放历史 → 用**原**幂等键对账每个在途 Effect →
重授权 → 重建 context → 恢复或隔离）。确定性管理回退（`admin-cli reconcile`）不依赖
任何模型驱动同一序列——未配置执行器时，仍未知的结果会隔离（fail-safe）而非强行了结。
原生 HTTP attempt 在出站前持久化，重启后在终态 receipt 出现前保持 indeterminate。
workspace 变更使用持久原键 receipt；相同文件字节本身不是执行证明，重启会保守清理
orphan staging。

## 备份与恢复 —— 作为用户功能 `unavailable`

盘点/导出/恢复预检的规划代码已存在（secret 路径按设计恒排除），但尚无
`cognitive backup`/`restore` 命令或归档 I/O 接线。今天诚实的替代做法：停止 daemon，
自行复制 XDG state/config 目录，并记住 Provider key **不在**这些文件里——在新机器恢
复后需重跑 `cognitive init` 重新录入 key。
