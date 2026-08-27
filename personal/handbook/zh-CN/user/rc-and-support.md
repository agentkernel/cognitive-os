---
doc_id: user.rc-and-support
locale: zh-CN
kind: guide
audience: [user]
status: partial
generated: false
sources:
  - path: tools/src/personal-rc-gate.mjs
    symbols: ["buildPersonalRcDeclarationReport"]
  - path: docs/plan/PERSONAL-SUPPORT-MATRIX.md
  - path: personal/deploy/linux/install.sh
  - path: personal/crates/cognitive-store/src/personal_backup.rs
    symbols: ["plan_personal_lifecycle"]
  - path: personal/apps/admin-cli/src/personal_cli/init.rs
    symbols: ["run_init"]
  - path: personal/apps/admin-cli/src/personal_cli/daemon.rs
tests:
  - tools/test/personal-rc-gate.test.mjs
fingerprint: "sha256:819dbb6082eaba9982c52baf9dbe66d04e1cc5fb2b3bf6e12cc90f0c1b2785a4"
non_claims:
  - 本页是操作地图，不是 Gate 结论、Profile 结果或生产 GitHub Release。
  - 不存在公开的 `cognitive uninstall` 或 `cognitive update` 动词；不要发明它们。
  - Multi-Agent、Web UI、dsh Path B、B10/MCP 与 Windows 安装对等不在 Linux RC 声明内。
---

# Linux RC 操作地图

`partial`：下列每一步都有代码实现，但**尚无公开生产发行物**。目前的 bundle 仍是实验
签名的 campaign 构建。当前 Gate 与任务事实由
[`docs/plan/PROGRESS.md`](../../../../docs/plan/PROGRESS.md) 拥有，本页不复制。
平台与声明政策见
[`PERSONAL-SUPPORT-MATRIX.md`](../../../../docs/plan/PERSONAL-SUPPORT-MATRIX.md)。

Personal Linux RC 声明是对既有证据的 **digest 绑定合成**。它不新跑 clean-VM campaign，
也不改动隔离的 B01 guest。

## 1. 安装

按[安装并到达首次对话](install-and-first-conversation.md) §1：从已签名 bundle 运行
经检查的 `install.sh`。安装器校验 bundle、暂存不可变字节、安装
`cognitiveos-personal.service`（loopback `127.0.0.1:48181`），然后才翻转
`active-version`。失败会补偿：恢复上一版本、unit 与指针。

## 2. 初始化

同一指南 §2 与[快速上手](getting-started.md)：

```text
cognitive init --provider <id> --base-url <https-url> --api-key-file -
```

Provider 密钥只进入批准的 Secret Store。没有明文回退。

## 3. Provider

daemon 运行后，命名账户、密钥、binding 与用量见
[Provider Control Plane](provider-control-plane.md)（CLI 与同源 `GET /ui/`）。
密钥永不进入 SQLite、argv 或浏览器存储。Control Plane Web UI 是非阻塞表面，
**不**属于 Linux RC 产品声明。

## 4. Pi

Linux 1.0 RC 唯一 product-qualify 的 Agent 是钉住的 Pi 及其 per-Agent sidecar。
见[Pi 对话壳](pi-shell.md)。DeepSeek Harness（dsh）Path B 是后续实现，不继承 Pi
证据。

## 5. Task

公开 Task 路径见[Task 与执行](tasks-and-execution.md)。首次对话或 Provider 响应
不是 Task 完成。Task 完成前必须有独立 verification。

## 6. 恢复

[运维与恢复](operations-and-recovery.md) 覆盖 status/doctor、崩溃/未知结果恢复、
备份/恢复与数据库安全。备份永不复制 secret 或 `authority.sqlite`。

## 7. 更新与回滚

没有公开的 `cognitive update`。更新 = 在同一主机上对**更新的** bundle 重新运行经
检查的已签名 `install.sh`。新激活成功前，旧版本留在磁盘上。任何失败都会恢复旧版本、
unit 与 `active-version` 指针，且不发成功收据。见
[安装器与服务](../developer/installer-and-service.md)。

权威路径规划（`plan_personal_lifecycle` 的 Update/Rollback）只记录意图，不替代
安装器补偿。

## 8. 卸载

没有公开的 `cognitive uninstall`。支持的操作序列是：

```text
cognitive daemon stop
systemctl --user disable --now cognitiveos-personal.service
```

这会停止并禁用 user unit。它不删除 Secret Store 条目、`authority.sqlite` 或 XDG
数据。`plan_personal_lifecycle` 的 Uninstall 拒绝 Secret 目标；Data 必须显式确认
删除；提交该 plan 仍不会删除主机文件。managed Pi 卸载仍走
[运维与恢复](operations-and-recovery.md) 中的 `admin-cli` 生命周期。

## 本 RC 不包含什么

- Multi-Agent / B11（对本 RC 记为 disabled-NO-GO；Phase 6 保持默认关闭）
- B10 / MCP / 动态 Tool marketplace
- Linux RC 产品声明中的 Web UI / Control Plane
- Windows 安装器/服务 / B01-W
- CognitiveOS Core Profile `implemented`
- 生产 GitHub Release 或生产签名仪式
