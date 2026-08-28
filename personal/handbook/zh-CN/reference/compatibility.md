---
doc_id: ref.compatibility
locale: zh-CN
kind: reference
audience: [user, developer]
status: implemented
generated: false
sources:
  - path: rust-toolchain.toml
  - path: package.json
  - path: personal/apps/admin-cli/src/personal_cli/pi.rs
    symbols: ["PINNED_PI_VERSION"]
  - path: personal/docs/product/linux-1.0-scope.md
  - path: docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md
  - path: docs/plan/PERSONAL-TEST-ENVIRONMENTS.md
fingerprint: "sha256:4dba5f897e903a91dcaced1e1f8572176e6e07a4c4edc560fddda344dd598495"
non_claims:
  - 在某平台可编译不等于产品支持；只有所列产品目标带安装与服务路径。
---

# 兼容性

## 当前产品目标

Linux x86_64 + 用户 systemd（开机启动需 lingering）。桌面会话需要 Secret Service
密钥环（GNOME Keyring）。headless 运行已设计（加密 vault）但尚不可选。WSL2 与
Windows 原生主机被 Pi 启动准入路径显式拒绝。

Personal 2.0 是 Windows-first，仍为 `Requires-backend` +
`Requires-environment`。qualified native Windows OPC dev environment 与 B01-W
均未 provision。目标 runtime 是 preinstalled managed DSH；Pi 是 hidden Personal
Assistant engine。Existing Linux Pi/dsh、WSL、ordinary CI、Canvas、CLI、Provider、
model、account、adapter 或 Windows GNU evidence 都不转移 Windows support/DSH
qualification。Hermes、Codex、Cursor 等是 future candidates。native mobile/E2E
relay remote 属于 Personal 2.1。

## 钉住版本

| 组件 | Pin | 位置 |
|---|---|---|
| Rust 工具链 | 1.97.1 | `rust-toolchain.toml` |
| pnpm | 10.33.2 | 根 `package.json` `packageManager` |
| Node | ≥ 22 | 根 `package.json` engines；CI 用 Node 22 |
| Pi agent | 精确 `0.81.1`（`@mariozechner/pi`，钉住 sha512 integrity） | 获取 + 启动准入 |
| SQLite 模式 | WAL、`synchronous=FULL`、外键开启 | store 打开断言 |
| HTTP 面 | 仅本地 loopback，约定端口 48181 | daemon 配置 |

## 可编译 vs 受支持

CI 在 Ubuntu 与 Windows MSVC 上构建并测试 workspace；那是工程证据，不是 Windows 产
品支持。仓库中现已存在 Windows 安装**表面**——Windows Credential Manager 生产
secret 后端，以及可检查的引导安装器与按用户 scheduled-task 模板（ADR-0052）——但端到
端 Windows 安装战役（B01-W）尚未执行，因此不声明安装对等，本地文件也仍无 ACL 加固。
已登记的本地 Windows GNU 主机完全无法链接 Rust。macOS 无 CI 泳道也无后端。

## 客户端兼容性

Pi 扩展与 TypeScript SDK 以 AKP 0.2 信封语义和生成合同类型对接 daemon 的本地 HTTP
面；消费者必须把未知错误码与未知响应字段当作协议失败处理（fail closed），与 Rust 侧
一致。
