---
doc_id: dev.agent-pi-lifecycle
locale: zh-CN
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: crates/cognitive-runtime/src/installer.rs
    symbols: ["install_package", "acquire_official_pi_durable"]
  - path: crates/cognitive-runtime/src/agent_registration.rs
    symbols: ["register_official_pi_agent_durable", "activate_official_pi_agent_durable"]
  - path: crates/cognitive-runtime/src/pi_launcher.rs
    symbols: ["admit_pi_launch"]
  - path: packages/pi-cognitiveos/src/pi-route-observation.ts
  - path: packages/pi-cognitiveos/src/extension.ts
  - path: apps/pi-agent-adapter/src/lib.rs
  - path: apps/pi-agent-adapter/src/main.rs
  - path: apps/kernel-server/src/personal/pi_runtime.rs
  - path: crates/cognitive-runtime/src/agent_adapter_manifest.rs
    symbols: ["register_agent_adapter"]
  - path: crates/cognitive-runtime/src/non_pi_agent.rs
tests:
  - crates/cognitive-runtime/tests/p5_t01_pi_acquisition.rs
  - crates/cognitive-runtime/tests/p5_t02_agent_registration.rs
  - crates/cognitive-runtime/tests/p5_t05_identity_recover.rs
  - crates/cognitive-runtime/tests/p5_t05_upgrade_fencing.rs
  - apps/admin-cli/tests/p2_t27_pi_lifecycle.rs
  - packages/pi-cognitiveos/src/pi-route-observation.test.ts
  - apps/pi-agent-adapter/tests/daemon_candidate_protocol.rs
  - apps/kernel-server/tests/p2_t31_live_daemon_scheduler.rs
  - apps/admin-cli/tests/p2_t32_public_daemon_start_scheduler.rs
  - apps/admin-cli/tests/p2_t33_private_candidate_host_path.rs
fingerprint: "sha256:12995bbf97d3a3ebeda7e91e859ddf790e70a693f19adb8866de1d868ba00cf6"
non_claims:
  - Pi 的资格化证据不转移给任何其他 agent；Codex 资格化是 fixture 身份矩阵，无网络/二进制声明。B09 类 Gate 记账由正式计划拥有。
---

# Agent 与 Pi 生命周期

三个分离阶段——**install ≠ register ≠ activate**——各自是 daemon 侧持久权威记录，全
部 epoch fencing。

## 获取与安装

`acquire_official_pi_durable` 以精确 `sha512-…` integrity 钉住
`@mariozechner/pi@0.81.1`：对 npm 元数据 URL 做白名单规范化/校验、验证 tarball 哈
希、确定性重打包，并产出失败类别类型化的获取报告。`install_package` 校验 digest +
签名端口并提交**零能力授予**的不可变安装证据。另有 custom-project 校验器路径服务本
地操作者包（路径安全 + digest + 本地策略 id）。

## 注册与 sidecar 会话

`register_official_pi_agent_durable` 要求持久安装证据并绑定精确包 digest；激活在
epoch CAS 下切换单一 active 指针；`SidecarSession` 绑定活进程身份
（`process_bound`），以 fencing 强制 pause/resume/stop/recover 迁移并报告脱敏健
康。升级/卸载 fence 旧 epoch；recover/orphan 负例有测试。`admin-cli`
（`install/register/activate/activate-root/rollback/agent-*`）是确定性调用者。

## 启动准入（shell 宿主角色）

`admit_pi_launch` 在以下条件不满足时一律 fail-close：Linux native（非 WSL2/
Windows）、doctor 全组件 ready、sandbox 适配器存在、`pi.json` 路径绝对且存在、版本
精确 `0.81.1`、模型 egress 绑定注册的 HTTPS 代理端点。它加载已配置 Extension、禁用
Pi 原生工具，并只显式允许 `WorkspaceRead` 与 `WorkspaceSearch`。这些 Extension 工具使用
钉住 Pi runtime 的 TypeBox schema；JSON 形状的替代物会在 live 注册时被拒绝。Pi 绑定
session 后，Extension 重新登记同名 daemon 治理定义以刷新 Pi runtime registry，然后只
激活这两个名称；未知名称会被忽略。该 post-bind 步骤不能激活 Pi 原生文件系统、shell 或
mutating tools。

shell 宿主的 Provider 路径有一个显式启用、非权威的 campaign observer。每个并发 Pi
请求用独立不透明 id 与 daemon 测得的两个阶段关联；Node 与 Rust 单调时长始终属于分离
时钟域。成功、取消和失败尝试都有明确终态记录，禁用会话则不产生任何记录。路径固定非流
式；`stream:true` 仍在解析 secret 前以稳定错误拒绝。Provider usage 绝不估算，也不接
受 runner 自行构造的对象。

## candidate 生产角色

`pi-agent-adapter`（钉住适配器，仅 `daemon-candidate` 能力）运行受限 Pi 子进程：禁
用内置文件系统/shell 工具、skill、会话与扩展发现（`--no-builtin-tools`）、环境白名
单、带字节上限与截止的一次性私有 socketpair、结构化 `AdapterOutcome`（绝非权威状
态）。CognitiveOS 扩展对外广告 daemon 治理的
WorkspaceRead/Search/Write/Patch；其 I/O-free Extension handler 只产出未信任
candidate，适配器只把一次此类工具调用映射到 daemon candidate 路径。WorkspaceRead
只携带 workspace target；其余带参数族仍使用受限参数处理。JSON 回退 candidate 若带
`parameters`，适配器会从参数重算 `parameters_digest`（含省略、空值或非法 digest）；否则 digest 必须是
`sha256:` 加 64 位小写十六进制。daemon 把其输出当作待准入
candidate——仅此而已。测试用 stub 适配器可以在 stdout 发出该未信任 candidate，而不连接
Provider completion socket；daemon 仍校验 descriptor、digest 与授权。completion
socket 绑在 `$XDG_RUNTIME_DIR/cognitiveos/`（其次进程临时目录，再次
`/tmp/cognitiveos`）下，以符合 Linux `UNIX_PATH_MAX`；这与 daemon 布局在
`XDG_RUNTIME_DIR` 缺失时 fail-closed 的行为相互独立。Linux candidate 在
`env_clear()` 之后只转发主机白名单（`HOME`、locale、`XDG_RUNTIME_DIR`、TLS
信任文件），绝不复制 `DBUS_SESSION_BUS_ADDRESS` 或 Provider key。适配器/Pi 的
stderr 经 `sk-` / `api_key=` / `token=` 脱敏后保留尾部真实错误在 `daemon.log`；退出码 2 表示 usage 错误、3 表示运行时失败，令公开 skip 可归因。私有
candidate 的 Provider 代理在转发前剥离 `tools`/`tool_choice`，接受可含
`role=assistant` 的单条文本 choice，并拒绝 `tool_calls`。

## Pi 之外

Universal Agent Adapter Contract（`agent_adapter_manifest`）注册讲 AKP 的适配器
（公网 listener 与权威写者被拒；仅 candidate 能力），生命周期 epoch fencing。首个非
Pi 资格化（OpenAI Codex CLI）是 fixture 范围的身份/生命周期矩阵，证明证据独立于
Pi——明确不是网络或二进制集成。
