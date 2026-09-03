---
doc_id: ai.code-map
locale: zh-CN
kind: reference
audience: [ai]
status: implemented
generated: false
sources:
  - path: Cargo.toml
  - path: pnpm-workspace.yaml
fingerprint: "sha256:82cdbc9bd0ff0e54eff16793f2ff10706ebc9d1ff878322e6d250ad8dae5d643"
non_claims:
  - 组件存在不构成 Gate、release 或 Profile 声明；接线状态见开发者执行链页面。
---

# 代码地图

十个 Rust crate、三个 Rust app、四个 TypeScript package/app。依赖方向：
`contracts → domain → kernel → store/runtime/management → apps`。确定性内核
（`cognitive-kernel`）按设计不依赖 HTTP、SQLite 或模型 SDK。

| 单元 | 职责 | 承重入口 |
|---|---|---|
| `crates/cognitive-contracts` | canonical JSON/digest、schema codegen、53 个生成 Rust 绑定、55 码错误注册表、golden 奇偶 | `canonical.rs`、`bin/contracts-codegen.rs`、`generated/mod.rs` |
| `crates/cognitive-domain` | ID、capability 算术、内嵌转移表、版本 | `transitions.rs`（`table`、`find_edge`）、`capability.rs`（`intersect_chain`） |
| `crates/cognitive-kernel` | 确定性权威内核：10 步转移门、intent chain、context 管线/缓存、Effect 协议、loop/WIA/continuation、恢复、tool registry、ports | `engine.rs`（`TransitionEngine`）、`intent_chain.rs`、`effects.rs`（`EffectProtocol`）、`harness.rs`（`LoopDriver`）、`ports.rs` |
| `crates/cognitive-store` | SQLite WAL 适配器：迁移 v1–v31（安装库 v1–v4）、调度 lease、Memory/Skill/Context/Artifact 存储、Provider Control Plane（v25 带标签用量读取）、排除 secret 的备份归档、隐藏托管 DSH 子进程（v31） | `sqlite/`、`migration.rs`、`personal_db.rs`（`prepare_personal_databases`）、`scheduler.rs`、`provider_control_plane.rs`、`personal_backup.rs` |
| `crates/cognitive-runtime` | 执行层：Linux bundle 校验/安装/服务、Pi 获取/注册/生命周期、adapter/hook/压缩/学习规划器、性能面 | `installer.rs`、`linux_bundle*.rs`、`agent_registration.rs`、`scheduler_service.rs`、`perf.rs` |
| `crates/cognitive-management` | 确定性管理面（inspect/stop/revoke/reconcile）、特权会话、R1 审批、审计端口、TaskApplicationService | `plane.rs`（`ManagementPlane`）、`session.rs`、`task_application.rs` |
| `crates/cognitive-secret` | SecretStore 后端（Linux Secret Service；Windows Credential Manager；其余 fail-closed）、Provider 配置/发现/传输、端点信任 | `store.rs`（`SecretStore`）、`backend_select.rs`、`provider_service.rs`、`provider_transport.rs`、`endpoint_trust.rs` |
| `crates/cognitive-provider-transport` | 用于确定性测试的 loopback TLS Provider fixture | `bin/p1_t09_provider_fixture.rs` |
| `crates/cognitive-akp` | AKP 0.2 信封解析/digest、内存 watch log | `lib.rs`（`parse_request`、`WatchLog`） |
| `crates/cognitive-conformance` | 符合性 runner：89 向量、五态报告、41 项自检翻转 | `src/main.rs`、`src/exec/` |
| `apps/kernel-server` | Personal daemon（`--personal`）：loopback HTTP、认证通道、readiness/doctor、Provider 代理、调度权威、tool executor、verification executor | `src/personal/server.rs`（`serve_personal_loopback`）、`scheduler_authority/`、`task_api.rs` |
| `apps/admin-cli` | 两个二进制：`cognitive`（产品 CLI）与 `admin-cli`（管理回退） | `src/cognitive_main.rs`、`src/main.rs`、`src/personal_cli/` |
| `apps/pi-agent-adapter` | 钉住版本的 Pi 子进程适配器；仅 `daemon-candidate` 与 `assistant-turn`（P13-T03 隐藏助手）可运行 | `src/main.rs`、`src/lib.rs` |
| `packages/pi-cognitiveos` | Pi 扩展：daemon 发现/客户端、provider 桥、默认拒绝工具 | `src/extension.ts`（`registerCognitiveOsExtension`）、`src/daemon-client.ts` |
| `packages/sdk-ts` | 通道隔离的 AKP 客户端 SDK：信封、传输、watch 消费 | `src/client.ts`、`src/channel.ts`、`src/watch.ts` |
| `packages/contracts-ts` | canonical JSON/digest 孪生 + 55 个生成 TS 模块 + golden 输出器 | `src/canonical.ts`、`src/generated/` |
| `apps/agent-shell` | 可复用 Shell 会话库（preview → submit → attach/cancel）；无 TUI | `src/session.ts`（`ShellSession`） |

关键真实调用链（细节见[开发者指南](../developer/README.md)）：

- CLI 初始化：`cognitive init` → `prepare_personal_databases` → `SecretStore` → `ProviderDiscoveryService` → 快照持久化。
- daemon 启动：`serve_personal_loopback` → 迁移 → 恢复 → 绑定 → endpoint 发布 → 唯一可取消的周期调度 worker。
- Task 准入：`POST /task/*` → `TaskApi` → `KernelTaskApplicationService` → `cognitive_kernel::intent_chain` → SQLite。
- Pi 对话：Pi 扩展 → `POST /provider/v1/chat/completions` → 绑定账户或 `provider.json` + daemon 持有的 SecretStore。
- DeepSeek harness Path B：dsh 插件 → `POST /provider/v1/dsh/chat/completions` → 独立的 `agent://personal/dsh` binding 或 `provider.json`。
- 安装：`personal/deploy/linux/install.sh` → `linux_bundle_installer` → verify → stage → health → activate（单服务事务）。

不得掩盖的执行接线缺口列于[执行链状态](../developer/execution-chain-status.md)。
