---
doc_id: dev.dev-environments
locale: zh-CN
kind: guide
audience: [developer, ai]
status: implemented
generated: false
sources:
  - path: docs/plan/PERSONAL-TEST-ENVIRONMENTS.md
    symbols: ["CI-UBUNTU-01", "DEV-LINUX-NATIVE-01", "RUST-LINK-DEV-WIN-GNU-01"]
  - path: rust-toolchain.toml
  - path: .gitattributes
fingerprint: "sha256:5dcfa03b1941891552feb1c494df0a6bd7008250cd1e23a576da4e5ae4e5e0c9"
non_claims:
  - 环境能力上限由环境注册表拥有；本页只做路由，不扩展任何声明。
---

# 开发环境

环境注册表
（[`PERSONAL-TEST-ENVIRONMENTS.md`](../../../docs/plan/PERSONAL-TEST-ENVIRONMENTS.md)）
拥有每个环境可声明的上限。实用路由：

| 环境 | 用于 | 绝不用于 |
|---|---|---|
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01`（required PR 检查） | 完整 Rust + TS + 符合性 + 漂移门 | Gate/release/Profile 晋升 |
| `DEV-LINUX-NATIVE-01`（native Linux 主机） | exact-revision native 验证、实验性服务/Pi 工作；只消费**已推送 commit** 到可清理 worktree | 未提交代码、生产声明 |
| `CLOUD-AGENT-LINUX-01`（Cursor Cloud Agent pod） | 推送前的完整 bash shell Rust + TS 迭代；由 `.cursor/environment.json` 引导 | native systemd/Secret Service 行为、timing 基线、Gate/release/Profile |
| 本地 Windows GNU 主机 | pnpm 构建/测试、`cargo fmt`、Node 检查器、文档工作 | 任何 workspace `cargo build/test/clippy/run`——已登记 linker exit 121 |
| WSL2 | 历史工程证据 | 产品路径声明（产品目标是 native Linux） |
| `B01-Desktop-Linux-002` | 预注册流程下的专用 Gate campaign guest | 普通开发、部署或测试 |
| `B01-W-DESKTOP-001` | 已注册但未供给的 Windows Gate guest（B01-W） | 按其预注册供给前的一切用途 |

工具链 pin：Rust 1.97.1（`rust-toolchain.toml`）、pnpm 10.33.2 + Node ≥22
（`package.json`）、workspace 级 `unsafe_code = "forbid"` 与 pedantic clippy
（`Cargo.toml`）。文本强制 LF 行尾（`.gitattributes`）——这也是手册指纹跨平台稳定的
原因。

本 Windows 主机的 shell 纪律：PowerShell 5.1——无 `&&`/`||`；用分开的调用或
`if ($LASTEXITCODE -eq 0) { … }` 串接。该规则与 GNU linker 上限都不适用于
`CLOUD-AGENT-LINUX-01`——它是 native GNU/Linux link 主机上的 bash。

Cloud Agent pod 和全新 Linux clone 用 `bash scripts/setup-dev-env.sh` 引导
（依赖、钉住的工具链、docs-sync hooks）。Cloud Agent 以 `cursor[bot]` 身份推送，
其 token 只覆盖该 run 的 environment 中登记的仓库。

命令速查：见 [AI 验证命令](../ai/validation-commands.md)——内容一致，只维护一份。
