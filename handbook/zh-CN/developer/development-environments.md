---
doc_id: dev.dev-environments
locale: zh-CN
kind: guide
audience: [developer, ai]
status: implemented
generated: false
sources:
  - path: docs/plan/PERSONAL-TEST-ENVIRONMENTS.md
    symbols: ["B01-DESKTOP-002", "CI-UBUNTU-01", "DEV-LINUX-NATIVE-01", "RUST-LINK-DEV-WIN-GNU-01"]
  - path: rust-toolchain.toml
  - path: .gitattributes
fingerprint: "sha256:37fcd64ae0f0d44f62e1abd2d7d35ea611921544dc842dcce50cba9d30d71be6"
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
| `linux-002` / `B01-Desktop-Linux-002`（经已登记 ProxyJump 路由访问的 native Linux guest） | Git + Rust 1.97.1 资格完备后，task-scoped、可清理 worktree 中的 designated primary exact-revision native 开发/测试验证 | 未提交代码、生产声明或 B01 campaign 资产 |
| 本地 Windows GNU 主机 | pnpm 构建/测试、`cargo fmt`、Node 检查器、文档工作 | 任何 workspace `cargo build/test/clippy/run`——已登记 linker exit 121 |
| WSL2 | 历史工程证据 | 产品路径声明（产品目标是 native Linux） |
| `B01-Desktop-Linux-002`（B01 用途） | 预注册 B01 campaign | 将普通开发/测试结果混入 campaign denominator，或修改 baseline、snapshot、root、credential 或 evidence |
| `B01-W-DESKTOP-001` | 已注册但未供给的 Windows Gate guest（B01-W） | 按其预注册供给前的一切用途 |

工具链 pin：Rust 1.97.1（`rust-toolchain.toml`）、pnpm 10.33.2 + Node ≥22
（`package.json`）、workspace 级 `unsafe_code = "forbid"` 与 pedantic clippy
（`Cargo.toml`）。文本强制 LF 行尾（`.gitattributes`）——这也是手册指纹跨平台稳定的
原因。

当前 linux-002 资格：路由、user systemd 和 Node v22.23.2 可用；guest PATH 上尚无 Git
和 Rust。待这些前置条件供给前使用已登记 CI 路径；不得把 CI 输出标作 linux-002 native
证据。

本 Windows 主机的 shell 纪律：PowerShell 5.1——无 `&&`/`||`；用分开的调用或
`if ($LASTEXITCODE -eq 0) { … }` 串接。

命令速查：见 [AI 验证命令](../ai/validation-commands.md)——内容一致，只维护一份。
