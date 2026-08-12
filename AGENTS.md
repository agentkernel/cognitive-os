# AGENTS.md — CognitiveOS Personal 开发代理入口

本仓库的唯一活动实现项目是 **`cognitiveos-personal`（CognitiveOS Personal）**。
原 CognitiveOS 设计、白皮书、规范和通用内核是架构参考与合同基础，不是第二个待交付
项目。完整边界见 [PROJECT-IDENTITY.md](docs/governance/PROJECT-IDENTITY.md)；本文件只
保留代理必须立即知道的操作规则，通用治理正文见
[DEVELOPMENT-OPERATING-MODEL.md](docs/governance/DEVELOPMENT-OPERATING-MODEL.md)。

## 新会话启动顺序

1. 阅读本文件和 [项目身份](docs/governance/PROJECT-IDENTITY.md)；
2. 阅读 [公理体系](docs/governance/AXIOMS.md) 与
   [Development Operating Model](docs/governance/DEVELOPMENT-OPERATING-MODEL.md)；
3. 阅读 Personal 正式计划 [PERSONAL-DEVELOPMENT-PLAN.md](docs/plan/PERSONAL-DEVELOPMENT-PLAN.md)；
4. 只读 `PROGRESS.md` 的 `Current snapshot`；
5. 只读 [PARALLEL-LANES.md](docs/plan/PARALLEL-LANES.md) 的活动 lease；
6. 产品/架构任务再读 [Personal 产品设计](docs/product/personal/README.md) 与
   [Personal 架构](docs/architecture/personal/README.md) 的相关章节；
7. 再阅读所选任务对应的最新 matching handoff 和根 `plan.md` 任务卡。

正式计划决定任务和 Gate，`PROGRESS.md` 决定当前事实，Parallel Lanes 决定当前可写路径，
handoff 只提供操作连续性，根 `plan.md` 只提供研究和细节。历史 handoff、旧提示词和
聊天上下文不能覆盖正式来源。禁止读取或引用 `History/`。

面向使用者/开发者/AI 工具的派生说明书在 [`handbook/`](handbook/README.md)（双语；AI
入口 `handbook/en/ai/README.md`）。它不拥有任何任务、Gate、合同或状态事实；改动实现
时按 `.cursor/rules/20-cognitiveos-personal-handbook-sync.mdc` 与
`handbook/_meta/sync-policy.md` 联动更新。

### 上下文压缩与跨窗口连续推进

- 仓库文档是唯一持久记忆；新窗口、上下文压缩或聊天历史缺失都按“恢复并继续”处理，
  不得要求用户重新提供任务背景，也不得因为当前窗口看不到旧对话而重复已完成工作。
- 每次恢复先核对当前 task/slice、精确可写路径、acceptance 完成项、验证
  `pass/not-run`、阻塞原因、完整 HEAD、upstream、PR 和 lease；这些事实必须来自规范来源，
  不能来自模型记忆、分支名或旧聊天。
- 工作中的 `PROGRESS.md` Current snapshot 或匹配 handoff 必须保留唯一下一动作、已完成项、
  未完成项和恢复所需的精确 revision。上下文接近限制、换窗口或暂停前，先写入恢复记录，
  不得把下一动作只留在聊天消息中。
- 只要没有真实外部阻塞、未知并发改动、必须由 owner 决定的事项、安全确认边界或完整任务
  收口，就继续在同一 task branch、Draft PR 和 lease 内推进；checkpoint、commit、push、CI
  轮次、阶段总结和上下文压缩都不是停止条件。

### Owner-directed 评测 campaign 路由（优先于开发续跑）

完整语义由 Operating Model §2.5 拥有；本节只是入口摘要。

- `PROGRESS.md` Current snapshot 的 `Owner-directed campaign` 行是评测模式的唯一激活
  开关。该行登记 active 评测 campaign（如 `PERSONAL-PERF-EVAL-002`）时，新窗口、
  `继续`、context compression 等 continuation event 一律恢复并执行该评测，**不得**自动
  领取或继续任何 `P*-T*` 开发任务，也不得进入 `CAMPAIGN-BACKLOG-CONTINUATION-01`
  backlog 循环。
- 评测恢复顺序：读该行 → 读其 execution plan（`docs/evaluation/`）→ 读最新评测
  checkpoint/preregistration → 读 active lease 表 → 核对 branch/HEAD。写入前领取
  `lease/personal/EVAL-<id>/<purpose>` 评测 lease，writable paths 只允许
  `docs/evaluation/`、`docs/checkpoints/` 与 `docs/plan/PROGRESS.md`。
- 评测是 measurement-only：不修改产品代码、合同、负例、测试或 handbook 生成源来"补齐"
  能力；能力/runner/凭据缺口如实记 `not-run`/`not_available`。公理、secret、campaign
  guest 隔离、exact-revision 与证据/claim 边界全部不变。
- 该行由 owner 关闭或 campaign 收口后失效；评测结束**不会**自动恢复开发续跑，backlog
  循环只在 owner 重新给出持续交付指令后恢复。

## 整任务连续交付协议（Operating Model 摘要）

本节仅是代理入口摘要。完整定义、确认边界、性能验证和冲突解释只由
[Development Operating Model](docs/governance/DEVELOPMENT-OPERATING-MODEL.md) 拥有；
本文件不得建立更严格或不同的第二套工作流。

- 默认只领取 `cognitiveos-personal` 的 `P*-T*` 任务；架构层改动必须服务于当前 Personal
  切片，规范合同改动必须走 Lane-CTR。
- Operating Model **`TASK-ATOMIC-DELIVERY-01`**：默认交付单位是一个完整正式任务，而不是单个 Delivery
  Slice。领取 `P2-T04` 等任务后，代理必须在同一个 task branch、Draft PR 和 task lease
  中连续完成其全部必要 Slice、集成、负例、supported validation、正式验收评估和文档收口，
  直到任务可诚实标为 `done`。只有不可自主消除的外部阻塞、未知并发改动、安全边界冲突或
  用户明确暂停/改范围时才能中断。不得因一个 Slice 完成、一次 commit、push、CI 发起、
  checkpoint 或阶段性结果而停止。
- 纯阅读、研究、计划草稿不会改变任务状态。第一个任务专属实现或测试 slice（包括
  failure-first 测试）开始时，将任务设为 `in-progress`。
- 领取前一次性核对完整任务 acceptance、implementation dependencies、所需可写路径和
  validation route，并创建一个 task-scoped branch、Draft PR 和 lease。Delivery Slice 使用
  `<task-id>/DNN` ID，但只是任务内的执行检查点，不是独立领取、独立分支、独立 PR 或默认
  汇报单位。不要把 acceptance 或 promotion Gate 当成 implementation mutex。
- 同一正式任务最多一个 `in-progress` Delivery Slice。已有足够基础能力时，下一切片
  必须优先接入真实调用者或 durable authority outcome；不得连续堆叠 helper-only
  切片来回避集成或验证阻塞。
- 任务内一个 Slice 达到出口后必须立即进入下一个未满足 acceptance 项；不得另开
  acceptance-assessment 分支或把“稍后做正式验收评估”作为悬空动作。最终 Slice 必须同时
  汇总完整任务 acceptance 并完成 task status 收口。
- 每个整任务工作流必须最终产出完整实现，或带
  `blocked_paths` / `blocked_task_ids` / `blocked_gate_ids` / owner / next action 的阻塞记录。
  已确认依赖后不得继续无出口审计；本地可修复的代码、测试、格式、CI 配置或集成问题不算
  中断理由，代理应直接修复并重跑。
- Operating Model **`CONTINUOUS-AUTONOMOUS-DELIVERY-01`**：用户要求持续推进时，代理必须在当前
  已领取任务的可写 task lease 内连续选择并实施下一个最小垂直交付切片，直到该任务完成、
  出现不可自主消除的明确阻塞，或用户明确要求暂停/切换。不得把 checkpoint、CI 发起、
  阶段性总结、单个验证结束或可恢复的临时环境故障当作停止工作的理由；这些只是在继续
  下一个实现、修复或预注册验证动作时记录的中间事实。除非需要用户决定、发现未知外部
  改动或完成用户请求，否则保持工具驱动的开发节奏，不发送临时进度汇报。
- Operating Model **`CAMPAIGN-BACKLOG-CONTINUATION-01`**：持续交付授权激活时，单个任务
  收口不是战役结束。同一会话内完成 ready/merge/lease/branch/main 后，立即领取下一个
  `implementation_requires` 已满足的就绪 Personal 任务，并开始其首个垂直实现切片；不得
  以 claim-only / docs-only / “下一动作已记录”作为回合结束。战役循环直到
  `PROGRESS.md` Layer 1 Remaining = 0、用户暂停/改范围，或只剩 owner 确认边界 / 真正
  外部阻塞。Gate disposition 若属 Operating Model §2.3 ADR-0040 类可自行判定则直接
  记账收口；仅真 owner-only 边界才正式 `blocked` 记账后改领不重叠的
  就绪任务继续追进度，不得空等聊天。
- Operating Model **`RESOLVE-BEFORE-BLOCKED-PROGRESS-01`**：对可由当前任务自行恢复的临时依赖、
  artifact 可用性、CI 或环境故障，先在当前 lease 内修复或走已登记的恢复路径；不得在
  `PROGRESS.md` 把它登记为 task `blocked`。B01 等 Gate 的不可逆 attempt ledger 仍须
  立即、如实记录每个已开始 attempt。只有恢复路径已耗尽且必须由 owner 作出不可替代的
  决策或执行操作时，才可将任务标为 `blocked` 并请求交互协助。
- 不为每个 Slice、commit、CI 轮次或普通可恢复故障创建 handoff/checkpoint 报告。Handoff
  只在完整任务收口、真正外部阻塞、未知改动或用户明确暂停时创建一次。
- Delivery Slice 只有在 focused failure-first/negative test 和其定义的 supported
  validation 实际通过后才能关闭。实现已存在但验证环境不可用时，记录为 `blocked`
  并转移到预先声明的 Linux/CI 验证路径；不得把格式或 consistency 通过写成切片完成。

### MVP-first 授权与实现深度

- 首个可运行 MVP 只实现当前任务验收和既有安全不变量要求的最小授权面。默认使用
  owner-local、single-principal、task-scoped、daemon-issued 的短路径，并对范围外请求
  fail closed；不得在真实垂直路径跑通前先建设完整 RBAC、审批链、通用 capability 管理、
  多租户策略语言或未来扩展框架。
- “最小权限”表示调用者只得到完成当前操作所需的最窄 authority，不表示省略 daemon-only
  writer、SecretStore、Intent/Effect persist-before-dispatch、budget/fencing、独立 verifier
  或审计边界。若一个授权机制不属于当前 task acceptance、已登记 threat boundary 或真实
  caller 的必要条件，则记录为后续 hardening，而不是当前实现 mutex。
- 优先复用已有 daemon authentication、management session 和 task bearer；只有现有机制
  无法安全表达当前 MVP 路径时才新增授权抽象。新增 public contract 或通用策略子系统前
  必须证明最小私有组合无法满足任务验收。

### Git checkpoint 与交付协议

- Operating Model **`CHECKPOINT-DELIVERY-01`**：Git 持久化与任务完成是独立维度。一个正式任务使用一个
  task branch、一个 Draft PR 和一个 task lease。实现不得直接在 `main` 累积，也不得把同一
  任务拆成多个 Slice branch/PR。checkpoint 只在远程 CI、exact-revision Linux 验证或异常
  恢复确实需要 immutable revision 时创建；它是后台持久化事件，不是会话停点、用户汇报点、
  Slice 关闭或 merge 理由。
- coherent、secret-free、归属清晰且通过本地 eligible checks 的任务进展可以自动 commit、
  push 并更新同一个 Draft PR，无需逐次等待授权。不得为每个 checkpoint 单独创建 handoff、
  closure docs 或新 PR；验证结果累计到任务最终收口记录。Draft PR 在完整 task acceptance
  未满足前必须保持 Draft，禁止 merge。
- 每次 commit/push 之前必须满足 docs-sync 义务（docs-sync-contract §2/§5）：改动路径
  命中 `handbook/_meta/source-map.json` 时，同一变更集内同步受影响 handbook 页面
  （双语）、重生成生成页并刷新指纹；确无文档影响时以
  `DOCS_IMPACT_NONE="<具体理由>"` 过门并把理由记入 commit/PR。本地门为
  `node tools/src/docs-sync-gate.mjs --staged|--push`（`pnpm run hooks:install`
  一次注册 `.githooks` 后自动执行）；merge 前由 CI handbook 步骤无条件红灯。
- 下载的 campaign package、installer、runtime archive 和执行 evidence payload 必须放在
  已忽略的 `/artifacts/` 或系统临时目录；Git 只记录可复验的 digest、attestation reference、
  脱敏事实和 checkpoint。不得暂存这些 payload，也不得以 untracked artifact 作为后续任务的
  隐式输入；后续任务必须按已记录的来源和 digest 重新取得所需 artifact。
- 只有完整任务 acceptance、全部必要 Slice、focused negatives、supported validation、
  required CI、最终 acceptance assessment、文档/证据同步和 review 要求全部满足后，代理
  才可将该 task PR 转为 ready 并合并。失败/待运行检查或产品/规范语义待决存在时禁止合并；
  force push 永不包含在持续授权中。
- 新窗口优先做快速恢复：确认 branch、clean/dirty、HEAD 与 upstream、Draft PR/checks、最新
  handoff 和 active lease；若它们一致，不重复全仓 Git 审计。Linux/native validation 只消费
  已 push 的 immutable checkpoint revision。
- 正常会话禁止留下未提交的 coherent 任务改动。`dirty handoff` 只允许用于 non-coherent
  中间态、未知改动、ownership/safety 冲突或用户明确暂停交付；必须列出 affected paths、
  原因、已执行检查、owner 和单一 recovery action，而不是无说明退出。
- 每个正式任务完成后，必须把收口做完整：提交并推送最终改动、将 task PR 合并、关闭 task
  lease、删除可安全删除的本地与远端 task branch、本地切回并 fast-forward 到 `main`，最后
  确认 `git status` clean 且 HEAD/upstream 一致。未完成这些动作前，任务不得视为真正收口。
- Handoff 至少记录：Slice/status、branch、完整 HEAD、upstream、PR URL/状态、worktree 状态、
  implemented、remaining、validation 的 pass/fail/not-run、non-claims 和 next action。

### 完整任务收口协议

完整任务的最后一个连续步骤必须一次性完成以下检查，不得在代码完成后另留一个
“acceptance assessment”任务：

1. 逐条将正式 task acceptance 映射到实现、focused negative 和已执行 evidence；任何缺项
   立即继续实现或验证，不能先标 `done`。
2. 运行该任务要求的 supported validation 和 required CI，并确认验证 revision 等于待合并
   HEAD；普通 product Gate 仍按独立 campaign 记账，不与 task completion 混淆。
3. 同步正式计划、`PROGRESS.md` Current snapshot、必要 trace 与唯一最终 handoff；同时完成
   handbook 联动（source-map 路由的页面、生成页与指纹，`check:handbook` 与生成器
   `--check` 绿）；未执行项写 `not-run`，不得推断为通过。
4. 确认 task branch 只包含该任务允许的改动，worktree 无未归属改动，PR 从 Draft 转 ready
   后正常合并，禁止 force push 或 amend 已推送历史。
5. 合并后关闭 task lease，确认远端 PR 为 merged、远端 task branch 可删除；本地安全切回
   `main` 并 fast-forward 到合并结果，确认 `git status` clean、`HEAD` 与目标远端一致、活动
   lease 不再引用已完成任务。无法完成任一步时保持任务 `in-progress`/`blocked` 并记录唯一
   recovery action，不得留下“代码完成但分支/状态未收口”的半完成状态。

## 不可放松的不变量

完整公理体系（A1–A8）与工程原则层（P1–P3）只由
[docs/governance/AXIOMS.md](docs/governance/AXIOMS.md) 拥有；本入口不得维护第二套
编号清单。日常提醒（以 AXIOMS 为准）：

1. Rust daemon 是唯一 authority writer（A1）；概率组件与第三方 agent 只产 candidate（A2）。
2. 外部 mutation 必须 persist-before-dispatch Intent/Effect（A3）；独立 verification 才可完成 Task（A4）。
3. Secret 只进批准的 Secret Store（A5）；合同与负例不得为实现而削弱（A6）。
4. 本地/fixture/WSL/ordinary CI 证据不得提升为 Gate/release/Profile（A7）；未知工作树改动受保护（A8）。

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

仓库 owner 已授予代理持续的自主操作授权。对于推进已领取 Personal task 所必需的
远程验证、approved Secret Store 使用、最小权限提权，以及 user-service/system
configuration 修改，代理可以自行决定和执行，无需逐次询问。此授权不改变安全或证据
边界：secret 不得进入 argv、普通配置、SQLite、日志、CI、测试输出或 evidence；操作必须
最小范围、可审计并在可行时提供 cleanup/rollback；远程验证仍只消费已推送的精确 Git
revision。正式 Gate guest、release/production promotion、force push、破坏性不可逆操作和
任何超出已领取 task 的系统变更仍须遵守其各自的预注册/隔离规则，不得借此授权虚假扩大
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
