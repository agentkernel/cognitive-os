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
fingerprint: "sha256:ae40674151ddaa54c7d4e433d65b41928b66bf602ed26477e47bba46cc9295f2"
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
| 本地 Windows 主机 `DEV-WIN-GNU-01`（GNU 默认工具链；已登记目录自 2026-09-03 起带本机 MSVC `rustup override`） | pnpm 构建/测试、`cargo fmt`、Node 检查器、文档工作（任何目录）；workspace `cargo build/test/clippy` **仅在 override 目录内**（`rustc -vV` → `host: x86_64-pc-windows-msvc`）作为开发迭代 | 在 GNU 默认 host 上做 Rust 链接——已登记 linker exit 121；把本机 MSVC 结果当 supported validation、Gate、release、Profile 或 Windows 支持 |
| `DEV-WINDOWS-NATIVE-OPC-01` | D01 已资格化的本机项目运行测试宿主（与 `DEV-WIN-GNU-01` 同一台机器；2026-09-05；OS 版本不是供给门槛）。Unsigned 安装 fail-closed + 现场 daemon admit。Tray/OS-sleep/sandbox/签名安装仍 `not-run` | 把 cargo 当原生 install/tray/sleep E2E；Gate/release/Profile；B01-W |
| WSL2 | 历史工程证据 | Linux 1.0 或 Windows OPC 产品路径声明 |
| `B01-Desktop-Linux-002` | 预注册流程下的专用 Gate campaign guest；自 2026-08-27 起同时为 owner 授权的 Personal 2.0 开发验证主机（仅限 exact-revision 一次性 worktree 与任务声明的可清理目录；B01 campaign 活动期间冻结开发用途） | 在预注册 B01 campaign lease 之外改变 guest 基线、快照或凭据 |
| `B01-W-DESKTOP-001` | 已注册但未供给的 Windows Gate guest（B01-W） | 按其预注册供给前的一切用途 |

Phase 11 Personal 2.0.0 路由：T03/T04 日常权威测试用 `CI-UBUNTU-01` /
`CI-WINDOWS-MSVC-01`（需要 native daemon/store 时加已 push 的 exact-revision
`DEV-LINUX-NATIVE-01`）。T02/T07 原生 host/DSH E2E 仍是
`DEV-WINDOWS-NATIVE-OPC-01` = D01 已资格化（2026-09-05，`P13-T13`）；缺能力的挂单格保持诚实 `not-run`。`P11-T15` 在本资格化宿主上 `in-progress`（N=15 冻结于 `main@4ca9b046`）。
T09 是画布 HITL，不是一级 Inbox。`B01-DESKTOP-002` 仅 campaign，不是 2.0 日常
默认机。`P11-T15` N=15 acceptance 在本 preregistered qualified
Windows revision 上执行，**不是** Phase 12 prototype completeness mutex。Phase 12 Dual Track
UI 用 `DEV-WIN-GNU-01` TS 加 required CI；产品 chrome 原生 UI E2E 仍 `not-run`
（fixture `/ui/` 200 不是该格）。Phase 13 路由
（`PERSONAL-TEST-ENVIRONMENTS.md` §5.2）：P13-T02/T03 真实 child/Pi 路径与其余权威卡用
`CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` + 已 push exact-revision `DEV-LINUX-NATIVE-01`；
`/ui/` 表面用 Dual Track TS；P13-T12/D02 的 rendered / NVDA / 200% / host-theme 复审对
exact-revision guest daemon `/ui/`（SSH 隧道）只是 implementation evidence；
`DOC-LOCAL-RUNTIME-HOST`（2026-09-05）已把本机指定为 `DEV-WINDOWS-NATIVE-OPC-01`
（OS 版本不是供给门槛）。`P13-T13/D01` 已于 2026-09-05 资格化 unsigned 路径；
D02 已逐格记 pass/fail/`not-run`。`P11-T15` 在 T13 收口后 `in-progress`。本机 cargo、WSL、Linux、ordinary CI 与 Canvas 明确不能替代
Gate/release；`not-run` 保持 `not-run`。

工具链 pin：Rust 1.97.1（`rust-toolchain.toml`）、pnpm 10.33.2 + Node ≥22
（`package.json`）、workspace 级 `unsafe_code = "forbid"` 与 pedantic clippy
（`Cargo.toml`）。文本强制 LF 行尾（`.gitattributes`）——这也是手册指纹跨平台稳定的
原因。本机 Windows 主机 `git config core.autocrlf` 为 `true`；该设置被 tracked 的
`.gitattributes` 规则 `* text=auto eol=lf` 覆盖，checkout 与 commit 均保持 LF，
无需改任何本机 Git 配置。

`DEV-WIN-GNU-01` 的本机 MSVC override（P0-T01/D02，owner 2026-09-03 决定，仅限本机）：本机
rustup 默认 host 是 `x86_64-pc-windows-gnu`，因此仅凭 `rust-toolchain.toml` 会解析到链接失败
（exit 121）的 GNU 工具链。修复方式是 rustup **目录 override**——
`rustup override set 1.97.1-x86_64-pc-windows-msvc`——已为 `D:\agent-kernel` 与任务 worktree
登记；它存放在 rustup 自己的 settings 中而不在仓库里，所以 `rust-toolchain.toml`、CI 与其他所有
clone 都不变（本仓 `.cargo/config.toml` 未被 gitignore，故不使用）。已安装的 Visual Studio
Build Tools 17.14.37（`D:\VSBuildTools`）提供 `link.exe` 14.44.35228.0，rustc 会自行找到——
无需改 PATH 或 `vcvars`。运行 `cargo build --workspace --locked`、
`cargo test --workspace --locked -- --test-threads=1`、
`cargo clippy --workspace --all-targets --locked -- -D warnings` 或 `cargo fmt --all -- --check`
之前先确认 `rustc -vV` 报 `host: x86_64-pc-windows-msvc`；新的本地 worktree 需要自己执行
`rustup override set`。在这台磁盘紧张的机器上，workspace 测试构建需要会话环境变量
`CARGO_PROFILE_DEV_DEBUG=0` 才装得下；另外 `kernel-server` `tool_executor` 中四个 fixture 需要
创建 symlink/reparse point 的测试会在 setup 阶段以 OS 错误 1314 失败，因为 shell 未提权且
Developer Mode 关闭（它们在提权的 CI runner 上通过；本机记为 `not-run (host privilege)`，绝不在
代码里跳过）。结果只是开发证据；环境登记 §3 的能力上限不变。本机已装
PowerShell 7.6.5（`pwsh`），但 Cursor Shell 仍是 Windows PowerShell 5.1。

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
