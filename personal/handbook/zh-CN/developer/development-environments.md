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
  - path: docs/bug/dsh-pathb-stale-daemon-bearer-after-daemon-restart.md
  - path: rust-toolchain.toml
  - path: .gitattributes
fingerprint: "sha256:75f81edaeae31a1997fba92f475510b22e0bb57209d4a7e3faebfd506fa7f654"
non_claims:
  - 环境能力上限由环境注册表拥有；本页只做路由，不扩展任何声明。
---

# 开发环境

环境注册表
（[`PERSONAL-TEST-ENVIRONMENTS.md`](../../../../docs/plan/PERSONAL-TEST-ENVIRONMENTS.md)）
拥有每个环境可声明的上限。实用路由：

| 环境 | 用于 | 绝不用于 |
|---|---|---|
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01`（required PR 检查） | 完整 Rust + TS + 符合性 + 漂移门 | Gate/release/Profile 晋升 |
| `DEV-LINUX-NATIVE-01`（native Linux 主机） | exact-revision native 验证、实验性服务/Pi 工作；只消费**已推送 commit** 到可清理 worktree | 未提交代码、生产声明 |
| `CLOUD-AGENT-LINUX-01`（Cursor Cloud Agent pod） | 推送前的完整 bash shell Rust + TS 迭代；由 `.cursor/environment.json` 引导 | native systemd/Secret Service 行为、timing 基线、Gate/release/Profile |
| 本地 Windows GNU 主机 | pnpm 构建/测试、`cargo fmt`、Node 检查器、文档工作 | 任何 workspace `cargo build/test/clippy/run`——已登记 linker exit 121 |
| `DEV-WINDOWS-NATIVE-OPC-01` | future qualified native Windows 11 Phase 11 host；当前未 provision | qualification 前的任何 claim |
| WSL2 | 历史工程证据 | Linux 1.0 或 Windows OPC 产品路径声明 |
| `B01-Desktop-Linux-002` | 预注册流程下的专用 Gate campaign guest；自 2026-08-27 起同时为 owner 授权的 Personal 2.0 开发验证主机（仅限 exact-revision 一次性 worktree 与任务声明的可清理目录；B01 campaign 活动期间冻结开发用途） | 在预注册 B01 campaign lease 之外改变 guest 基线、快照或凭据 |
| `B01-W-DESKTOP-001` | 已注册但未供给的 Windows Gate guest（B01-W） | 按其预注册供给前的一切用途 |

Phase 11 Personal 2.0.0 路由：T03/T04 日常权威测试用 `CI-UBUNTU-01` /
`CI-WINDOWS-MSVC-01`（需要 native daemon/store 时加已 push 的 exact-revision
`DEV-LINUX-NATIVE-01`）。T02/T07 原生 host/DSH E2E 仍是
`DEV-WINDOWS-NATIVE-OPC-01` = `Requires-environment` / `not-run`（未资格化）。
T09 是画布 HITL，不是一级 Inbox。`B01-DESKTOP-002` 仅 campaign，不是 2.0 日常
默认机。parked 的 T15 N=15 acceptance 若解冻仍需同一 preregistered qualified
Windows revision。本地 GNU、WSL、Linux、ordinary CI 与 Canvas 明确不能替代
Gate/release；`not-run` 保持 `not-run`。

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

当代理在 `B01-Desktop-Linux-002`（linux-002）上部署 Control Plane 或 dsh 后，
默认让 owner 在 **本机 Windows 浏览器经 SSH 端口转发** 查看，不要只依赖 guest
桌面 Firefox。先在 guest 上用 `cognitive daemon status` 确认 daemon 绑定端口，
再在 `DEV-WIN-GNU-01` 上：

```powershell
ssh -J wuz@192.168.1.2 -L 48681:127.0.0.1:48681 -L 3080:127.0.0.1:3080 hal9001@192.168.123.160
```

保持该会话不退出，然后打开 `http://127.0.0.1:48681/ui/`（Control Plane；粘贴
runtime 的 management bootstrap secret，绝不是 Provider API key）和
`http://127.0.0.1:3080/`（原生 dsh 面板）。guest 上 daemon 重启或替换
kernel-server 之后，须在该 runtime 上重启 `cognitive dsh web`，再期望 dsh
对话可用；新 daemon 将 dsh 报为 `INACTIVE`，所以 `cognitive dsh apply` 不能恢复该
stale session。`apply` 只用于 runtime 已为 `ACTIVE` 时所支持的 binding/model overlay
同步。Vite preview 不是产品源。完整端口表与隔离规则由环境注册表拥有；本页只做路由。

命令速查：见 [AI 验证命令](../ai/validation-commands.md)——内容一致，只维护一份。
