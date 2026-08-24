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
  - path: apps/kernel-server/src/personal/pinned_https.rs
    symbols: ["handle"]
  - path: apps/kernel-server/src/personal/observation.rs
    symbols: ["handle"]
  - path: apps/kernel-server/src/personal/six_resource_doctor.rs
  - path: apps/admin-cli/src/personal_cli/daemon.rs
  - path: apps/kernel-server/src/personal/user_backup.rs
    symbols: ["handle"]
  - path: apps/admin-cli/src/personal_cli/backup.rs
  - path: apps/admin-cli/src/personal_cli/dsh.rs
    symbols: ["launch"]
  - path: apps/admin-cli/src/personal_cli/provider.rs
  - path: crates/cognitive-store/src/personal_backup.rs
    symbols: ["write_personal_backup_archive", "restore_personal_backup_archive"]
  - path: crates/cognitive-store/src/personal_db.rs
    symbols: ["prepare_personal_databases"]
  - path: crates/cognitive-store/src/sqlite/intent_chain.rs
    symbols: ["insert_task_contract_with_execution_bootstrap"]
tests:
  - apps/kernel-server/tests/p1_t05_personal_readiness.rs
  - apps/kernel-server/tests/p2_t27_backup_restore.rs
  - apps/admin-cli/tests/p2_t27_backup_restore.rs
  - apps/admin-cli/tests/p2_t32_public_daemon_start_scheduler.rs
  - crates/cognitive-store/tests/p1_t01_layout_migrations.rs
fingerprint: "sha256:ff6528b108538e8df4c0615d53ad1e8c2dbbd49cb1ab32a5e58c69815e85b9f9"
non_claims:
  - "`ready` 是配置/存活投影，不是实时 Provider 或端到端保证。备份/恢复排除 secret，且不复制 authority SQLite。"
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
- `GET /ui`（免认证）从 `data_dir()/ui` 提供同源静态 Web UI，并带 CSP
  `default-src 'self'`。缺少 bundle 时为 `503` `not_available`
  （`LOCAL_UI_BUNDLE_UNAVAILABLE`），不构成 readiness。若请求携带 `Origin` 或
  `Referer`，必须是本 daemon 的 loopback HTTP origin。
- 服务日志：`journalctl --user -u cognitiveos-personal.service`。CLI
  `cognitive daemon start` 还会把 kernel-server 的 stdout/stderr 追加到
  `state/cognitiveos/daemon.log`（权限 `0600`）；调度 skip 行不是公开 HTTP 事实。
  私有 candidate 适配器拒绝时，脱敏诊断（去掉 `sk-` / `api_key=` / `token=` 片段）
  会保留在该日志中。
- `cognitive pi launch --print` 是有界的非交互 Pi 路径：Pi 从 stdin 读取 prompt 时，
  public CLI 会保持连接，直到 Pi 退出。它仍要求 daemon 的完整 ready 投影，不向 Pi
  传递 Provider 凭据，禁用 Pi 原生工具，也不得以直接 daemon/private-candidate 调用替代。
  `--append-system-prompt <绝对路径>` 把已存在且非空的 UTF-8 文件转发给 Pi；它不是
  Provider 凭据，文件字节不会被打印。
- `cognitive dsh launch --print` 是有界的非交互 dsh Path B：要求 daemon-owned
  的 system/database/secret/daemon 就绪（Pi 与 Pi `provider.json` 可保持
  blocked），加载钉住的 AKP 插件，绝不把 dsh 响应当作 Task 完成。直接 Flash
  （`--path a`）被拒绝；同机 Path A/B 测量只用
  `packages/dsh-akp-adapter/scripts/paired-path.mjs`。
- `cognitive dsh web` 启动原生 dsh 控制面板（`dsh --profile web --no-open`），默认
  `http://127.0.0.1:3080`。这不是 Personal `/ui/`。只绑定 loopback（拒绝
  `--host 0.0.0.0`）。钉住的 dsh 根必须有 `apps/web/dist`（`pnpm run build`）。
  Path B 仍走 daemon Provider 代理与 SecretStore；Models 页不应再索要第二把
  DeepSeek 密钥。不要把 SecretStore 材料写入 dsh `.env`。
  面板会话绝不是 Task 完成。SSH guest 上保持 `--no-open`（产品默认）。
- `cognitive dsh apply` 把 Cos dsh Agent binding 发布为 Path B selected-model
  （`POST /personal/dsh/runtime` `op=apply`），并按该绑定账户目录写入原生 Models
  覆盖层。Cos 安装的 web 会重载，使对话与 Models 与控制面一致；解绑 dsh 会去掉
  那些模型（包括 grok）。聊天走绑定账户（Cos 指定 grok 时绝不会发到 DeepSeek）。
  web 为 INACTIVE 或模型不在该账户目录时失败闭合。遗留的 grok-on-DeepSeek binding
  会以 `PERSONAL_PROVIDER_BINDING_MISMATCH` 失败闭合 Path B，而不是向 DeepSeek
  发请求。
- `cognitive dsh status` 读取 `GET /personal/dsh/runtime`：由进程内会话与可选绑定
  pid 得到 INACTIVE / ACTIVE / CRASHED。Linux 存活只看 `/proc/{pid}` 是否存在
  （永不打开 cmdline/environ）。它不是 authority writer。UI 起来也不是 Task 完成。

## 停止、重启、过期状态 —— `implemented`

`cognitive daemon stop` 向记录的 PID 发信号，仅在确认进程消失后移除 `daemon.lock`
与 endpoint 文档；看似存活的锁绝不删除。每次启动 daemon 幂等地重跑迁移、恢复已消费的
worker 交接，仅修复当前已准入合同所缺 Loop/Budget/调度前置而不重置既有行，并原子地
重新发布 endpoint。随后才启动唯一周期调度 worker；顺序退出会在释放 daemon 状态前取
消、唤醒并 join 它。

Tool overlay 与钉住 HTTPS origin 文件位于 Personal data 目录
（`personal-tool-lifecycle.json`、`personal-pinned-https.json`）。重启会重新加载；
它们不是 Artifact CAS 对象。生产 HttpFetchReadOnly 在 management 以授权 campaign
钉住精确 HTTPS origin 之前保持失败闭合。有界 O2/O3/O4/O5/O13 观测样本位于
`personal-observation-plane.json`（O2–O4）与权威事件日志（O5/O13）并在重启后保留；
空窗口返回带具名 negative control 的 `observed_zero`。

## 数据库安全 —— `implemented`

数据库位于 XDG state（`authority.sqlite`、`installation.sqlite`，WAL 模式、0600）。
每次迁移 apply 都先在 `state/backups/` 写带时间戳的备份（不自动清理）。派生数据
（Memory FTS 索引）可从权威行重建；被遗忘的 Memory 绝不会因索引重建而复活。权威库
迁移现含 v24 只追加 Memory/Skill 消费记录；后续会话可复用精确钉，但遗忘、撤销或
digest 漂移或竞争记录会失败闭合，而不是让已遗忘事实复活。management Memory/Skill
生命周期行与 Skill revision 谱系在 daemon 重启后仍可检查，包括通过
`cognitive resource list|inspect`。公开 Memory remember 可发送
未封存的 owner 字段；daemon 用持久治理根组合封存 header。

## 崩溃与未知结果恢复 —— 引擎层 `implemented`

恢复遵循固定八步序（fence 旧写者 → 重放历史 → 用**原**幂等键对账每个在途 Effect →
重授权 → 重建 context → 恢复或隔离）。确定性管理回退（`admin-cli reconcile`）不依赖
任何模型驱动同一序列——未配置执行器时，仍未知的结果会隔离（fail-safe）而非强行了结。
原生 HTTP attempt 在出站前持久化，重启后在终态 receipt 出现前保持 indeterminate。
workspace 变更使用持久原键 receipt；相同文件字节本身不是执行证明，重启会保守清理
orphan staging。生产 HTTP fetch staging 还会咨询评测授权的钉住 origin 登记表；没有
钉时白名单保持为空，请求失败闭合。

成功的 Task 准入在权威库内同样具备崩溃原子性：合同、`DRAFT` governed Task、
`START` Loop、硬 Budget 与 runnable 调度行一起出现。daemon 自有的 Context 授权事实与
租户 `personal` 撤销 epoch 在该 CAS 之前作为幂等 owner-local 策略持久化，不是调用方
能力通道。提交前失败不会留下这些准入成员；
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

## Provider Control Plane —— `partial`

`cognitive provider …`、`cognitive agent binding …`、`cognitive usage query`、
`cognitive budget …`、`cognitive alerts …` 与 `cognitive audit query` 调用
daemon management 表面。localhost Web UI 作为 daemon 客户端使用同一套路由
（`GET /ui/`）；密钥只经 management key POST 或 `--api-key-file` 进入
SecretStore，永不进入 SQLite、argv 或浏览器存储。没有桌面控制面板。预算告警只
观察/查询，不阻断也不改路。自定义 HTTP 或私网端点需要持久的
`--allow-insecure-http` / `--allow-private-network` 授权。binding 更新可发送可选
`expected_revision`；不匹配时 HTTP 409 `PROVIDER_BINDING_REVISION_STALE`。操作
步骤、可执行命令与常见失败见
[Provider Control Plane](./provider-control-plane.md)。

## 备份与恢复 —— `partial`

`cognitive backup [--output <dir>]` 写入 digest 绑定的目录归档（config/data/state/
artifacts 与 Memory/Skill 导出 sidecar）。永不复制 `authority.sqlite`、
`provider-config.json`、bootstrap secret 或 bearer。`cognitive restore --archive
<dir>`（或 `--archive-id`）先做 schema/digest 预检，再从 staging 覆盖 live 文件；
失败则回滚快照。`--preflight` 只校验不变更。management 通道提供相同操作：
`POST /management/resource/v1/backup` 与 `.../restore`。恢复后若 Secret Store 中没有
Provider key，需用 `cognitive init` 重新录入。公开 `admin-cli` 调用方覆盖 managed
Pi install→activate-root→register→activate→pause/resume→upgrade/rollback→stop→
recover→uninstall。聚焦测试把恢复后字节相等和有限墙钟记为 hypothesis-only
事实；本页不声明 RTO/RPO 或 Gate 结果。
