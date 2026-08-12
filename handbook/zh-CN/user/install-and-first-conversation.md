---
doc_id: user.install-first-conversation
locale: zh-CN
kind: guide
audience: [user]
status: partial
generated: false
sources:
  - path: deploy/linux/install.sh
  - path: crates/cognitive-runtime/src/linux_bundle_service.rs
    symbols: ["install_linux_bundle_single_service", "cognitiveos-personal.service"]
  - path: crates/cognitive-runtime/src/bin/linux_bundle_installer.rs
  - path: apps/admin-cli/src/personal_cli/init.rs
    symbols: ["run_init"]
tests:
  - crates/cognitive-runtime/tests/linux_bundle_single_service.rs
  - crates/cognitive-runtime/tests/linux_installer_bootstrap.rs
  - apps/admin-cli/tests/p1_t06_cognitive_cli.rs
fingerprint: "sha256:f543ba606e59b310562b3f241c4b8b140dfb8ff82d9da65d257964ec6da8f5fe"
non_claims:
  - 尚无公开 GitHub Release 或生产签名仪式；迄今可安装产物均为实验签名的 campaign 构建。安装路线正确性证据（B01）由正式计划拥有，此处不复述。
---

# 安装并到达首次对话

`partial`：下述完整路线已实现并在干净 Linux 机器上端到端演练过，但**尚无公开的生产
发布产物**——迄今的 bundle 来自实验 campaign 构建器与非生产签名密钥。平台：Linux
x86_64 + user systemd；桌面需要 Secret Service 密钥环（GNOME Keyring）。

## 1. 运行可检查的引导安装脚本

release 形态的 bundle 附带渲染好的 `install.sh`（模板为可检查的
[`deploy/linux/install.sh`](../../../deploy/linux/install.sh)）。它刻意乏味：
fail-closed 的 shell 设置、HTTPS-only 有界下载、单一钉住的跳转主机、执行前对安装器二
进制做 SHA-256 校验；没有 `curl | sh`、没有 `sudo`、不内嵌密钥。

随后 Rust 安装器验证 Ed25519 签名的 bundle attestation（产品、平台、版本、Pi pin、
安全归档布局），把不可变字节 stage 到 XDG data 目录，安装唯一的用户服务
`cognitiveos-personal.service`（loopback `127.0.0.1:48181`、
`NoNewPrivileges=true`），确认健康与进程身份后才切换原子 `active-version` 指针。任何
失败都会补偿：恢复前一版本、unit 与指针，且不签发成功回执。

## 2. 初始化配置与 secret

```text
cognitive init --provider <id> --base-url <https-url> --api-key-file -
```

`cognitive init` 准备数据库（带迁移前备份），把 Provider key 经 stdin/隐藏输入存入
OS secret store（此处为 Linux Secret Service；在 Windows 主机上同一命令会选择
Credential Manager 后端）——绝不走 argv 或文件落盘——探测 Provider，并持久化两个非
secret 文件：`provider.json`（含不透明 `SecretRef`）与 `selected-model.json`。若无可
用的生产 secret 后端，命令 fail-closed——没有明文回退。

## 3. 启动并检查 daemon

```text
cognitive daemon start          # 默认绑定 127.0.0.1:48181
cognitive status                # 组件投影
cognitive doctor                # 脱敏诊断
```

status 输出中的 `first_conversation_ready` 额外要求 Pi 已配置；整体 readiness 不要求。

## 4. 配置并启动 Pi

```text
cognitive pi configure --executable <pi 绝对路径> --extension-entry <dist/index.js 绝对路径>
cognitive pi launch
```

launch 是 fail-closed 的：要求 doctor 全组件 ready 与精确钉住的 Pi 版本，只传
`--extension`，绝不把 Provider key 交给 Pi。你的第一条消息经 Pi → daemon Provider
代理 → Provider；见 [Pi 对话壳](./pi-shell.md)。

## 值得了解的失败出口

签名/pin 不符 → 什么都不装；健康检查失败 → 恢复前一服务；密钥环锁定或缺失 →
`init` 拒绝；过期 `daemon.lock` → `cognitive daemon stop` 只在证明进程已消失后清理。
