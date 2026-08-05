# AGENTS.md — CognitiveOS Personal 开发代理入口

本仓库的唯一活动实现项目是 **`cognitiveos-personal`（CognitiveOS Personal）**。
原 CognitiveOS 设计、白皮书、规范和通用内核是架构参考与合同基础，不是第二个待交付
项目。完整边界见 [PROJECT-IDENTITY.md](docs/governance/PROJECT-IDENTITY.md)；本文件只
保留代理必须立即知道的操作规则，通用治理正文见
[DEVELOPMENT-OPERATING-MODEL.md](docs/governance/DEVELOPMENT-OPERATING-MODEL.md)。

## 新会话启动顺序

1. 阅读本文件和 [项目身份](docs/governance/PROJECT-IDENTITY.md)；
2. 阅读 [Development Operating Model](docs/governance/DEVELOPMENT-OPERATING-MODEL.md)；
3. 阅读 Personal 正式计划 [PERSONAL-DEVELOPMENT-PLAN.md](docs/plan/PERSONAL-DEVELOPMENT-PLAN.md)；
4. 只读 `PROGRESS.md` 的 `Current snapshot`；
5. 只读 [PARALLEL-LANES.md](docs/plan/PARALLEL-LANES.md) 的活动 lease；
6. 产品/架构任务再读 [Personal 产品设计](docs/product/personal/README.md) 与
   [Personal 架构](docs/architecture/personal/README.md) 的相关章节；
7. 再阅读所选任务对应的最新 matching handoff 和根 `plan.md` 任务卡。

正式计划决定任务和 Gate，`PROGRESS.md` 决定当前事实，Parallel Lanes 决定当前可写路径，
handoff 只提供操作连续性，根 `plan.md` 只提供研究和细节。历史 handoff、旧提示词和
聊天上下文不能覆盖正式来源。禁止读取或引用 `History/`。

## 任务领取与反循环协议

- 默认只领取 `cognitiveos-personal` 的 `P*-T*` 任务；架构层改动必须服务于当前 Personal
  切片，规范合同改动必须走 Lane-CTR。
- 纯阅读、研究、计划草稿不会改变任务状态。第一个任务专属实现或测试 slice（包括
  failure-first 测试）开始时，将任务设为 `in-progress`。
- 领取前确认没有路径重叠的活动 lease；领取后立即实施正式计划登记的最小 Delivery
  Slice。Delivery Slice 使用 `<task-id>/DNN` ID；正式计划拥有出口定义，`PROGRESS.md`
  Current snapshot 拥有当前状态。不要把 acceptance 或 promotion Gate 当成
  implementation mutex。
- 同一正式任务最多一个 `in-progress` Delivery Slice。已有足够基础能力时，下一切片
  必须优先接入真实调用者或 durable authority outcome；不得连续堆叠 helper-only
  切片来回避集成或验证阻塞。
- 每次会话必须产出实现、失败优先测试、可验证文档修正，或带
  `blocked_paths` / `blocked_task_ids` / `blocked_gate_ids` / owner / next action 的阻塞记录。
  已确认依赖后不得继续无出口审计。
- **`CONTINUOUS-AUTONOMOUS-DELIVERY-01`：** 用户要求持续推进时，代理必须在当前
  已领取任务的可写 lease 内连续选择并实施下一个最小垂直交付切片，直到该任务完成、
  出现不可自主消除的明确阻塞，或用户明确要求暂停/切换。不得把 checkpoint、CI 发起、
  阶段性总结、单个验证结束或可恢复的临时环境故障当作停止工作的理由；这些只是在继续
  下一个实现、修复或预注册验证动作时记录的中间事实。除非需要用户决定、发现未知外部
  改动或完成用户请求，否则保持工具驱动的开发节奏，不发送临时进度汇报。
- 结束会话前同步正式计划、`PROGRESS.md` current snapshot 和 handoff；未执行检查写
  `not-run`，不可推断为通过。
- Delivery Slice 只有在 focused failure-first/negative test 和其定义的 supported
  validation 实际通过后才能关闭。实现已存在但验证环境不可用时，记录为 `blocked`
  并转移到预先声明的 Linux/CI 验证路径；不得把格式或 consistency 通过写成切片完成。

### Git checkpoint 与交付协议

- **`CHECKPOINT-DELIVERY-01`：** Git 持久化与 Slice 完成是独立维度。一个 coherent、
  secret-free、归属清晰且通过全部本地 eligible checks 的 `in-progress` / `blocked` Slice
  通常应形成 checkpoint commit；required remote CI 是 ready/merge 和 Slice `done` 的
  条件，不是创建 checkpoint commit 的前置条件。
- 实现任务不得直接在 `main` 累积。仓库 owner 已授予持续 Git 交付权限：使用 Slice 专属
  branch，coherent 改动通过本地 eligible checks 后，代理必须自动 commit、push，并创建或
  更新 **Draft PR**，不在每个新窗口重复等待授权。Draft PR 用于 CI、exact-revision Linux
  验证和跨窗口恢复，必须保持 Draft，禁止 merge，也不得因为 commit、push、PR 或绿色的
  非完整检查而把 Slice 标为 `done`；用户明确要求暂停交付时除外。
- 同一 PR 可随实现推进自动追加 checkpoint；只有正式 Slice 出口、focused negatives、
  supported validation、required CI、文档/证据同步和 review 要求全部满足后，代理才默认
  自动转为 ready 并合并。未完成 checkpoint、失败/待运行检查或产品/规范语义待决存在时
  禁止合并；force push 永不包含在持续授权中。远程 secret/基础设施操作的自主授权边界见
  “Standing operator authorization”。
- 新窗口优先做快速恢复：确认 branch、clean/dirty、HEAD 与 upstream、Draft PR/checks、最新
  handoff 和 active lease；若它们一致，不重复全仓 Git 审计。Linux/native validation 只消费
  已 push 的 immutable checkpoint revision。
- 正常会话禁止留下未提交的 coherent 任务改动。`dirty handoff` 只允许用于 non-coherent
  中间态、未知改动、ownership/safety 冲突或用户明确暂停交付；必须列出 affected paths、
  原因、已执行检查、owner 和单一 recovery action，而不是无说明退出。
- Handoff 至少记录：Slice/status、branch、完整 HEAD、upstream、PR URL/状态、worktree 状态、
  implemented、remaining、validation 的 pass/fail/not-run、non-claims 和 next action。

## 不可放松的不变量

1. Rust daemon 是唯一 authority writer；Pi、CLI、SDK、UI 和 fixture 都是客户端。
2. 概率组件只能产 candidate/proposal；授权、CAS、状态迁移、预算、幂等、fencing、
   Effect 提交、reconcile 和最终验收由确定性服务端执行。
3. 外部 mutating operation 必须使用 persist-before-dispatch 的 Intent/Effect；外部成功、
   Provider response 或 Pi `agent_end` 都不等于 Task 完成。
4. Provider/user secret 只能进入批准的 Secret Store，绝不进入 argv、普通配置、SQLite、
   日志、CI、测试输出或 evidence。
5. 负例、规范向量和合同不能为了迎合实现而删除、放宽或改写。
6. local、WSL、fixture 和 ordinary CI 只有在 Gate 预注册 campaign 明确纳入时才能贡献
   Gate 证据；否则只能是 non-claim implementation evidence。

## 命令速查

### 本地命令与测试环境预路由

- **`COMMAND-SHELL-PS51`：** Cursor 本地 Shell 当前按 **Windows PowerShell 5.1**
  解析。禁止使用 `&&` 或 `||` 连接命令；解析器在命令启动前拒绝它们，该结果只能记为
  `not-run`，不能记为测试失败。互不依赖的命令使用独立并行 Shell 调用；有依赖的命令
  使用 `if ($LASTEXITCODE -eq 0) { <next-command> }`，或拆成后续调用。只有明确进入 bash
  环境后才能使用 bash 连接符。
- **`RUST-LINK-DEV-WIN-GNU-01`：** 当前本机是已登记且不受支持的
  `x86_64-pc-windows-gnu` Rust link host。其 workspace build/test/Clippy/run/bench 已稳定
  在 linker exit 121 失败；除显式领取的 P0-T01 工具链修复 Slice 外，禁止重复运行这些
  linking/compiling 命令，也禁止再次尝试 LLVM-MinGW、shim、PATH、toolchain pin 或源码
  workaround。
- 本机 GNU allowlist 仅包括不触发 Rust 编译/链接的工作：`cargo fmt`、文档/静态
  consistency、Node/TypeScript 检查和 diff 检查。需要 Rust build/test/Clippy 的 Slice
  必须在开始前路由到 `CI-UBUNTU-01`、`CI-WINDOWS-MSVC-01`，或按 exact-revision 规则
  使用 `DEV-LINUX-NATIVE-01`；环境不可用时按 Slice 规则记 `blocked`/`not-run`，不得先在
  本机 GNU 重现已知 linker failure。
- 权威环境能力和命令路由见
  [PERSONAL-TEST-ENVIRONMENTS.md](docs/plan/PERSONAL-TEST-ENVIRONMENTS.md)。每个 Slice 在
  写 failure-first test 前先选择其 required validation environment。

| 目的 | Windows PowerShell（本地） | CI（bash） |
|---|---|---|
| Rust 构建 | `DEV-WIN-GNU-01` 禁止；路由到 supported CI/Linux | `cargo build --workspace` |
| Rust 测试 | `DEV-WIN-GNU-01` 禁止；路由到 supported CI/Linux | `cargo test --workspace` |
| Rust lint | `DEV-WIN-GNU-01` 禁止；路由到 supported CI/Linux | `cargo clippy --workspace --all-targets` |
| TS 安装 | `pnpm install` | `pnpm install --frozen-lockfile` |
| TS 构建/测试 | `pnpm -r build ; pnpm -r test` | `pnpm -r build && pnpm -r test` |
| 静态一致性检查 | `pnpm run check:consistency` | 同左 |
| 符合性 runner | `DEV-WIN-GNU-01` 禁止；路由到 supported CI/Linux | `cargo run -p cognitive-conformance --bin conformance-runner` |

本机若 `cargo` 不在 PATH：`$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"`。该命令
只恢复已钉住工具的发现，不授权在本机 GNU 重跑 Rust linking 命令。工具链钉在
`rust-toolchain.toml`（1.97.1）。

## Personal 实验环境边界

`personal-linux-native-01`（`wuz@192.168.1.2`）只是已资格化的实验性开发主机，优先
用于明确授权的 `experimental-local-only` / `tested-local` 验证，不是 B01、release、
Profile、containment 或正式产品 Gate 环境。每个 Pi slice 都必须重新确认 Linux/native
user-systemd、Rust/Node、exact Pi `0.81.1` 和可清理目录；`pi` 不在 PATH 不是 pin 已
满足的证据。真实 load 只允许 `--extension <absolute-path>` 与脱敏 observation。

SSH 仅使用非交互、无 secret 探针，例如：
`ssh -o BatchMode=yes -o ConnectTimeout=10 "wuz@192.168.1.2" "<redacted command>"`。

### Standing operator authorization

仓库 owner 已授予代理持续的自主操作授权。对于推进已领取 Personal slice 所必需的
远程验证、approved Secret Store 使用、最小权限提权，以及 user-service/system
configuration 修改，代理可以自行决定和执行，无需逐次询问。此授权不改变安全或证据
边界：secret 不得进入 argv、普通配置、SQLite、日志、CI、测试输出或 evidence；操作必须
最小范围、可审计并在可行时提供 cleanup/rollback；远程验证仍只消费已推送的精确 Git
revision。正式 Gate guest、release/production promotion、force push、破坏性不可逆操作和
任何超出已领取 slice 的系统变更仍须遵守其各自的预注册/隔离规则，不得借此授权虚假扩大
声明。

Linux daemon、Pi/sidecar、installer、user-service、native integration 和实验部署切片必须
优先在 `personal-linux-native-01`（`wuz@192.168.1.2`）上执行。先将**已提交并可追溯的**
revision 同步到该主机的可清理 Git worktree，再在该 exact revision 上构建、测试和记录证据；
不得复制未提交本地代码，也不得用旧的无 Git source snapshot 替代当前 revision。Windows
只可运行格式、静态、文档或不依赖 Linux runtime 的检查，不能替代 native Linux 结果。

`B01-Desktop-Linux-002` 是唯一活动的专用 KVM B01 campaign guest，不能作为普通开发或
部署目标。只有预注册 B01 lease 与 campaign procedure 可改变其状态、快照、产品安装或
凭据。`B01-Clean-Linux-001` 已因不满足 headless Secret Service 前置条件而退役：仅保留
历史资格失败记录，禁止把它恢复为 B01 或常规测试候选，也禁止普通开发恢复、启动、重置、
部署或删除该 VM；其隔离状态不受 standing authorization 影响。其他实验部署只可使用 SSH
宿主上任务声明的可清理目录；user-service 修改、最小权限提权、approved Secret Store 使用
或外部 Provider 操作按 Standing operator authorization 自主执行。

## 目录和变更边界

- `specs/`、`conformance/`：架构合同和符合性资产；不得为实现改写。
- `crates/`、`apps/`、`packages/`、`tests/`、`tools/`：Personal 实现及其验证工作面。
- `docs/governance/`、`docs/plan/`、`docs/checkpoints/`：治理、正式计划、快照和移交。
- `apps/cognitiveos-console/`、独立客户端和其他 deferred 能力：默认只维护设计/台账，
  不启动独立实现。
- `personal-blog/` 是独立仓库，禁止推入本仓库；禁止在其他路径建立平行副本。

变更必须声明 `implementation-only`、`corrective`、`product-semantic`、
`normative-semantic` 或 `structural`，
并按 [docs-sync-contract](docs/standards/docs-sync-contract.md) 完成联动。提交/PR 必须
关联 Personal 任务或 REQ/F/IMP；没有关联时说明原因。未知工作树改动不得覆盖、回退、
混入或使用 `git add -A`。
