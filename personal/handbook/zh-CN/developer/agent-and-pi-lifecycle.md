---
doc_id: dev.agent-pi-lifecycle
locale: zh-CN
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: personal/crates/cognitive-runtime/src/installer.rs
    symbols: ["install_package", "acquire_official_pi_durable"]
  - path: personal/crates/cognitive-runtime/src/agent_registration.rs
    symbols: ["register_official_pi_agent_durable", "activate_official_pi_agent_durable"]
  - path: personal/crates/cognitive-runtime/src/pi_launcher.rs
    symbols: ["admit_pi_launch"]
  - path: personal/packages/pi-cognitiveos/src/pi-route-observation.ts
  - path: personal/packages/pi-cognitiveos/src/extension.ts
  - path: personal/apps/pi-agent-adapter/src/lib.rs
  - path: personal/apps/pi-agent-adapter/src/main.rs
  - path: personal/apps/kernel-server/src/personal/pi_runtime.rs
  - path: personal/crates/cognitive-runtime/src/agent_adapter_manifest.rs
    symbols: ["register_agent_adapter"]
  - path: personal/crates/cognitive-runtime/src/non_pi_agent.rs
  - path: personal/crates/cognitive-runtime/src/dsh_agent.rs
    symbols: ["register_dsh_adapter"]
  - path: core/crates/cognitive-akp/src/deepseek_harness.rs
    symbols: ["DeepSeekHarnessAdapter"]
  - path: core/crates/cognitive-akp/src/bin/dsh-akp-bridge.rs
  - path: personal/packages/dsh-akp-adapter/src/index.ts
  - path: personal/packages/dsh-akp-adapter/src/plugin.ts
    symbols: ["apply", "applyDshAkpCordisPlugin"]
  - path: personal/packages/dsh-akp-adapter/src/index.test.ts
  - path: personal/apps/admin-cli/src/personal_cli/dsh.rs
    symbols: ["configure", "launch", "status"]
  - path: personal/packages/dsh-akp-adapter/scripts/dsh-real-process.mjs
  - path: personal/packages/dsh-akp-adapter/scripts/dsh-web-preflight.mjs
  - path: personal/packages/dsh-akp-adapter/scripts/paired-path.mjs
  - path: personal/docs/product/agent-integration-and-conversations.md
  - path: personal/docs/product/agent-integration-and-conversations.zh-CN.md
  - path: personal/docs/architecture/agent-shell-and-agent-lifecycle.md
  - path: personal/docs/architecture/agent-adapter-contract.md
  - path: personal/docs/architecture/multi-agent-orchestration.md
  - path: docs/adr/0056-personal-2-0-desktop-control-plane.md
tests:
  - personal/crates/cognitive-runtime/tests/p5_t01_pi_acquisition.rs
  - personal/crates/cognitive-runtime/tests/p5_t02_agent_registration.rs
  - personal/crates/cognitive-runtime/tests/p5_t05_identity_recover.rs
  - personal/crates/cognitive-runtime/tests/p5_t05_upgrade_fencing.rs
  - personal/apps/admin-cli/tests/p2_t27_pi_lifecycle.rs
  - personal/packages/pi-cognitiveos/src/pi-route-observation.test.ts
  - personal/apps/pi-agent-adapter/tests/daemon_candidate_protocol.rs
  - personal/apps/kernel-server/tests/p2_t31_live_daemon_scheduler.rs
  - personal/apps/admin-cli/tests/p2_t32_public_daemon_start_scheduler.rs
  - personal/apps/admin-cli/tests/p2_t33_private_candidate_host_path.rs
  - personal/packages/dsh-akp-adapter/src/index.test.ts
fingerprint: "sha256:493cd97bdfc668ddc35c2296d6d091d8b75dcd5a09be040bde2870307e77b3b4"
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
激活这两个名称。每次 agent turn 前，它会在 runtime registry 可用后重复该激活；未知名称
会被忽略，因此任一名称不在 Pi 实际 registry 时 CognitiveOS 会 fail-close。CLI 显式的 `--tools` 列表是完整 Pi registry allowlist，因此不能激活 Pi 原生文件系统、shell 或
mutating tools。

shell 宿主的 Provider 路径有一个显式启用、非权威的 campaign observer。每个并发 Pi
请求用独立不透明 id 与 daemon 测得的两个阶段关联；Node 与 Rust 单调时长始终属于分离
时钟域。成功、取消和失败尝试都有明确终态记录，禁用会话则不产生任何记录。Pi 对话保持
一元（`stream:false`）。公开 management Provider 代理可将 `stream:true` 按 SSE 转发。Provider usage 绝不估算，也不接
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

### Personal 2.0 Agent 对话与监督目标

`Requires-backend`：桌面 Agent Shell 的目标是呈现厂商专用对话适配器，而不是假装
所有 Agent 共用一种通用聊天协议。每个适配器必须保留厂商的 conversation/session
身份、支持的控制、能力缺口与恢复语义，同时只翻译有界 candidate 与 observation。
通用 adapter manifest 或可工作的 dsh bridge 都不构成对话对等或资格化。

Goal 与不可变 Plan revision 也是位于当前 Task 之上的目标权威对象。多 Agent 监督仍由
daemon 拥有：各自独立资格化的 Agent 可以贡献 candidate，但 daemon 分配工作、签发
continuation authority、fence epoch、强制预算、对账 Effect 并取得独立验证。当前没有
Goal/Plan 或多 Agent 监督 API。Pi 仍是唯一已资格化 Agent。

DeepSeek Harness 桥接是仅 candidate 的适配器。Rust 侧钉住精确 dsh git revision 与 AKP
request-envelope schema digest，对进程内 session 做 fencing，强制单调 sequence，并拒绝
authority-shaped 与 secret-shaped payload。`POST /task/akp/dsh` 必须在 daemon 启动后显式
激活；重启会清空会话表并失败闭合。Workspace* candidate 映射到既有 public candidate
admission。WorkspaceRead 无参数对象（digest 仍覆盖 `{"family":"WorkspaceRead"}`）；
WorkspaceSearch 需要 query；WorkspaceWrite/Patch 需要规范 `input_b64` 与 `preimage`。
TypeScript shim 经长驻、长度受限的 snake_case JSONL 或 HTTP transport 发送
事件。它不接收 Provider 凭据、不写权威状态，也不把 dsh 响应当作 Task 完成。linux-002
真机只是 implementation evidence，不构成 Gate、release、Profile、B01 或 Agent-benefit。
timing 字段只是测量入口，不能推出零开销保证。`personal/packages/dsh-akp-adapter/scripts/linux002-e2e.mjs`
在身份确认后的 linux-002 上用 HTTP 驱动 `attachDshCordisPlugin`，并等待 Task
`COMPLETED`。`personal/packages/dsh-akp-adapter/src/plugin.ts` 是 `dsh --patch` 的 Cordis
`apply` 入口；`scripts/dsh-real-process.mjs` 在存在 host `build:lib` 产物时用
编译后的 `apps/cli/lib/bin.js` 启动钉住的 dsh，否则回退
`node --import tsx/esm apps/cli/src/bin.ts`
（不调用 `pnpm dsh`），加载 `plugin.bundle.cjs`（Node 22.23 会拒绝
`require()` ESM `plugin.js`），会先 admit 可丢弃的
WorkspaceRead/Search/Write Task，再由真实 dsh 进程以 plugin `startupEvents`
提交这些 candidate，并把 Flash 经 daemon Provider SSE 代理转发
（`POST /provider/v1/dsh/chat/completions` 且 `stream:true`）。交互式原生面板将
`llm-deepseek.maxTokens` 钉到绑定 LongCat 路由接受的 131,072 token 上限：过小的
256 token 预算可能在推理模型输出 assistant content 前就被消耗，而 dsh 的 256,000
默认值会被上游拒绝。有界 one-shot probe 使用独立的 4,096 token 预算。daemon 仅在流式 OpenAI-compatible `tool_calls` continuation frame
中规范化值为 `null` 的 `id`、`type`、`function.name` 与
`function.arguments`：保留起始 frame 的 identity，避免上游 `null` 覆盖已经累积的
工具名；它不会虚构或授权工具调用。产品安装路径是 `cognitive dsh configure` 然后 `cognitive dsh launch`
（Path B）。`cognitive dsh web` 在钉住 dsh 根执行 `pnpm run build` 产出 `apps/web/dist`
后启动原生面板（`dsh --profile web --no-open`，默认 `http://127.0.0.1:3080`），不是
Personal `/ui/`。Web Path B 会写入 `$DSH_HOME/settings.yaml`，让 `llm-deepseek`
保持 `POST /provider/v1/dsh/chat/completions`，并把官方 Models 目录的密钥引用别名到
daemon management bearer — 不把 SecretStore 材料复制进 dsh，也不写 `.env`。
原生 Models 是当前 dsh 绑定账户目录（不是 Cos/DeepSeek 残留 id）。binding 的
set/remove 与 `op: apply` 会重写该覆盖层；Cos 安装的 web 会重载。
`cognitive dsh status` 读取 `GET /personal/dsh/runtime`。
`POST /personal/dsh/runtime` 的 `op: apply` 把 Cos `agent://personal/dsh` binding
发布为 Path B selected-model，并按该目录重载原生 Models。
`op: clear` 会清掉绑定 pid 与内存中的 session，投影回到 `INACTIVE`。
`op: apply` 只接受已经为 `ACTIVE` 的 runtime，且只用于所支持的 binding/model
overlay 同步。它不是 daemon 重启后的 session 刷新路径：新 daemon 没有先前 runtime
登记，将 dsh 投影为 `INACTIVE`，并拒绝 `apply`。
直接 Flash（`--path a`）只经 `scripts/paired-path.mjs` 做测量。
`dsh.json` 里的 adapter registration digest 不是 SQLite 持久的 daemon adapter 状态。
两者都只是 implementation evidence。

daemon 重启后，当前 dsh Path B web 进程可能继续持有 stale management session，并把
401 显示成 “API key invalid”。必须重启 `cognitive dsh web`，再检查
`cognitive dsh status`；新 daemon 报 `INACTIVE` 时 `apply` 会被拒绝。当前没有 approved
non-logging direct-bearer probe：不得提取该凭据或通过进程 argv 传递。持久化 Provider
账户 `active` 不是实时 SecretStore 解析结果；discovery/proxy 使用时才实时解析，因此
锁定或变化的 store 状态仍是独立可能原因。见
[正式登记缺陷](../../../../docs/bug/dsh-pathb-stale-daemon-bearer-after-daemon-restart.md)。
