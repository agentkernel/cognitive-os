---
doc_id: user.getting-started
locale: zh-CN
kind: guide
audience: [user]
status: partial
generated: false
sources:
  - path: personal/deploy/linux/install.sh
  - path: personal/crates/cognitive-runtime/src/linux_bundle_service.rs
    symbols: ["cognitiveos-personal.service"]
  - path: personal/apps/admin-cli/src/personal_cli/init.rs
    symbols: ["run_init"]
  - path: personal/apps/admin-cli/src/personal_cli/mod.rs
    symbols: ["COGNITIVE_USAGE"]
  - path: personal/apps/admin-cli/src/personal_cli/daemon.rs
  - path: personal/apps/admin-cli/src/personal_cli/pi.rs
tests:
  - personal/crates/cognitive-runtime/tests/linux_bundle_single_service.rs
  - personal/crates/cognitive-runtime/tests/linux_installer_bootstrap.rs
  - personal/apps/admin-cli/tests/p1_t06_cognitive_cli.rs
  - personal/apps/admin-cli/tests/p2_t27_backup_restore.rs
fingerprint: "sha256:ace7c10d3efbc0516ab7a519ec384fd251e8e95b467cfc77d83ebcdcc42481f6"
non_claims:
  - 尚无公开生产发布产物；可安装 bundle 来自实验 campaign 构建。
  - 本页不构成 Gate、Profile、生产就绪或 Windows 安装声明。
  - 第一条对话只证明链路可用，不证明自主 Task 完成或 agent 质量。
---

# 快速上手

这是 Linux x86_64 + user-systemd 主机上的最短支持路径。前提是可用的操作系统
Secret Service（例如 GNOME Keyring）、Provider HTTPS 地址和精确钉住版本的 Pi 包。
系统没有明文 secret 回退。

## 1. 安装 bundle

使用 bundle 内的 `install.sh`。引导过程只走 HTTPS，在激活前校验安装器和签名 bundle，
安装唯一的用户服务 `cognitiveos-personal.service`，daemon 默认绑定 `127.0.0.1:48181`。
健康检查或进程身份检查失败时会恢复上一版本，而不是返回成功。

## 2. 初始化 Provider

通过 stdin 或受保护的文件描述符传入 key，绝不要把 key 放在参数中：

```text
cognitive init --provider <id> --base-url <https-url> --api-key-file -
```

命令会探测 Provider，把 key 存入批准的 secret store，并只把不透明的 `SecretRef` 与
selected-model 元数据写入普通文件。临时 runtime 如需复用已有绑定，使用
`--reuse-existing-secret-binding`、Provider 和 base URL，不要再次采集 key。轮换使用
`cognitive init --rotate-key`。

## 3. 启动并检查 daemon

```text
cognitive daemon start
cognitive status
cognitive doctor
```

`status` 是组件投影，`doctor` 追加脱敏诊断。关注 `first_conversation_ready`；它还要求
Pi 已配置。`ready` 只表示本地配置和进程检查通过，不保证每次 Provider 请求都成功。

## 4. 配置并启动 Pi

```text
cognitive pi configure \
  --executable <Pi 绝对路径> \
  --extension-entry <dist/index.js 绝对路径>
cognitive pi launch
```

启动前会检查完整 doctor 投影和钉住的 Pi 版本。Extension 通过 daemon Provider 代理工作，
Pi 永远拿不到 Provider key。Pi 原生 shell 与文件工具保持禁用。要执行一次有界的非交互
提示，可使用：

```text
printf '%s\n' '请总结当前工作区。' | cognitive pi launch --print
```

可选 `--append-system-prompt <绝对路径>` 转发一个已存在且非空的 UTF-8 文件；相对、缺失
或空文件都会 fail closed。

## 5. 观察持久事实

使用 CLI 投影，而不是把聊天输出当作 authority：

```text
cognitive resource get --family memory
cognitive resource get --family task
cognitive task watch
cognitive task evidence --task-ref task://<id>
```

认证投影会返回精确 Task URI 和可用资源族。Task 在调度器拿到 lease 前保持 `DRAFT` 是
正常现象，不等于对话失败。

## 6. 备份与恢复

```text
cognitive backup --output <目录>
cognitive restore --archive <目录> --preflight
cognitive restore --archive <目录>
```

备份是 digest 绑定的归档，不包含 Provider key、bearer 或 authority SQLite。恢复前先做
preflight。恢复后重新运行 `cognitive status` 与 `cognitive doctor`；如果 secret store
中没有 key，再运行 `cognitive init`。

排障请看[运维与恢复](operations-and-recovery.md)、[Provider 与 secret](provider-and-secrets.md)、
[已知限制](known-limitations.md) 和 [Linux RC 操作地图](rc-and-support.md)。
