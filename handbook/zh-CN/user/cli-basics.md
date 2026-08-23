---
doc_id: user.cli-basics
locale: zh-CN
kind: guide
audience: [user]
status: implemented
generated: false
sources:
  - path: apps/admin-cli/src/personal_cli/mod.rs
    symbols: ["parse_cognitive_args", "COGNITIVE_USAGE"]
  - path: apps/admin-cli/src/personal_cli/daemon.rs
  - path: apps/admin-cli/src/personal_cli/backup.rs
  - path: apps/admin-cli/src/personal_cli/dsh.rs
    symbols: ["configure", "launch", "status"]
  - path: apps/admin-cli/src/personal_cli/provider.rs
tests:
  - apps/admin-cli/tests/p1_t06_cognitive_cli.rs
  - apps/admin-cli/tests/p2_t27_backup_restore.rs
  - apps/admin-cli/tests/p2_t32_public_daemon_start_scheduler.rs
  - apps/kernel-server/tests/p2_t27_backup_restore.rs
  - apps/admin-cli/src/personal_cli/dsh.rs
fingerprint: "sha256:993aa11e17cb1372bb09c9646f0921ee5ce3daf47bff8527e4dab4754e8c7c59"
non_claims:
  - CLI 是非权威客户端；它打印的任何内容都不意味着 Task 完成或 Gate 结果。
---

# CLI 基础

`cognitive` 二进制是确定性的产品入口。它从不直接写权威状态——只准备配置、启动/停止
daemon 进程、读取已认证投影。退出码：`0` 成功、`1` 运行错误、`2` 用法错误；成功输出
为 JSON。

| 动词 | 实际行为 |
|---|---|
| `cognitive init` | 准备 XDG 布局与数据库（带备份），把 Provider key 存入 Secret Service，或用 `--reuse-existing-secret-binding` 绑定已有条目，探测 Provider，持久化 `provider.json` 与 `selected-model.json` |
| `cognitive status` | 已认证的组件投影（system、database、secret、provider、daemon、pi） |
| `cognitive doctor` | 同一投影外加脱敏诊断小节 |
| `cognitive daemon start` | 启动绑定 `127.0.0.1:48181` 的 `kernel-server --personal`（可用 `--bind`、`--kernel-server` 或 `COGNITIVE_KERNEL_SERVER` 覆盖）；stdout/stderr 追加到 `state/cognitiveos/daemon.log`（权限 `0600`） |
| `cognitive daemon status` | 报告 daemon 锁/endpoint 存活状态 |
| `cognitive daemon stop` | 向记录的 PID 发信号；确认退出后才移除锁与 endpoint |
| `cognitive pi configure` | 写非 secret 的 `pi.json`（可执行文件与扩展入口的绝对路径） |
| `cognitive pi launch [--task-ref <task://URI>] [--append-system-prompt <absolute-path>]` | doctor 全就绪且版本精确匹配后 fail-closed 启动 Pi；任务绑定启动仅暴露 daemon 治理的 WorkspaceRead/Search/Write/Patch，并向 task 通道提交不可信 candidate；`--append-system-prompt` 把已存在的绝对 UTF-8 文件转发给 Pi，不是 Provider 凭据 |
| `cognitive dsh configure --dsh-root <absolute-path> --adapter-root <absolute-path> --revision <git-object>` | 写非 secret 的 `dsh.json`（钉住的 dsh 检出、AKP adapter 根、仅 candidate 的 adapter digest）；revision 必须等于产品 pin |
| `cognitive dsh launch [--print] [--path b] [--task <prompt>]` | daemon-owned ready 后 fail-closed 启动 dsh（Pi 可保持 `not_configured`）；Path B 加载钉住的 AKP 插件，绝不把 dsh 响应当作 Task 完成；`--path a` 在此拒绝，仅能经 `paired-path.mjs` 做测量 |
| `cognitive dsh status` | 认证后观察 dsh 会话/fencing 与可选绑定 pid 存活（`GET /personal/dsh/runtime`）；不是 Task 完成 |
| `cognitive resource get/watch --family <memory\|skill\|tool\|context\|task\|runtime>` | 读取私有六族投影（management 通道） |
| `cognitive resource list/inspect --family <…> [--id <id>]` | 通用 Resource Manager 只读信封（management 通道） |
| `cognitive resource bind\|unbind\|enable\|disable\|revoke --family <…> --id <id> --expected-version <n> --idempotency-key <key>` | 通用 Resource Manager 变异，接到既有 Skill/Tool sinks；不是 generic create/execute/complete |
| `cognitive provider account create\|list\|show\|update\|delete` | management Provider Control Plane 账户；只用 `--api-key-file`；自定义 HTTP/私网端点需要 `--allow-insecure-http` / `--allow-private-network`。细节见 [Provider Control Plane](./provider-control-plane.md) |
| `cognitive provider key set\|rotate\|remove` | 经 daemon 的 Secret Store 密钥操作；永不写 SQLite |
| `cognitive provider models refresh\|list\|add\|set-price` | 前台发现、手动模型、价格 |
| `cognitive agent binding set\|show\|list\|remove` | 固定 pi/dsh 账户+provider+model binding；无回退 |
| `cognitive usage query` / `cognitive budget set\|list\|remove` / `cognitive alerts list\|acknowledge` / `cognitive audit query` | 用量/成本/审计与仅观察预算；本阶段 `usage`/`audit` 无过滤器 |
| `cognitive task watch [--resume-from N]` | 跟随有界 Task watch 流（task 通道） |
| `cognitive task evidence --task-ref <URI>` | 读取由持久 authority 与 Artifact CAS 重建的有界脱敏终态证据（task 通道） |
| `cognitive backup [--output <dir>]` | 写入排除 secret 的 digest 绑定归档（不含 authority SQLite / provider-config / bearer） |
| `cognitive restore --archive <dir> [--preflight]` | 预检后从已验证归档覆盖 live 文件；`--preflight` 不变更 |

两个诚实的怪癖（生成的 [CLI 参考](../reference/cli-cognitive.md)中同样标注）：所有动词
都接受的 `--runtime-root <dir>` 是密封测试逃生口，会整体搬迁布局。

独立的 `admin-cli` 二进制是管理回退入口（inspect / stop / revoke / reconcile 及
agent 生命周期动词），需要特权会话文档；见 [admin-cli 参考](../reference/cli-admin.md)
与[管理面](../developer/management-plane.md)页面。
