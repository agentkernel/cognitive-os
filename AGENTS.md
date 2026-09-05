# AGENTS.md — CognitiveOS Personal 开发代理入口

本仓库的唯一活动实现项目是 **`cognitiveos-personal`（CognitiveOS Personal）**。
仓库按 [ADR-0054](docs/adr/0054-repository-subproject-structure-and-1.0.0-finalization.md)
组织为 `core/`（合同与权威基底，1.0.0 已定稿）、`personal/`（唯一活动产品，1.0.0
已定稿）、`enterprise/`（设计层，未激活）、`clients/`（并入的客户端子项目）四个
子项目目录加共享 `docs/` 治理层；core 是架构参考与合同基础，不是第二个待交付项目。

本文件只保留代理必须立即知道的操作规则和快速入口。每条规则都有唯一权威来源，本文件
只做摘要与指路，**不得建立第二套更严或不同的语义**；摘要与来源冲突时以来源为准并在
同一交付中修正本文件。

| 事实 | 唯一权威来源 |
|---|---|
| 项目身份与目录边界 | [PROJECT-IDENTITY.md](docs/governance/PROJECT-IDENTITY.md) |
| 公理 A1–A8 / 原则 P1–P3 | [AXIOMS.md](docs/governance/AXIOMS.md) |
| 工作流、证据、Git、阻塞、确认边界、收口语义 | [DEVELOPMENT-OPERATING-MODEL.md](docs/governance/DEVELOPMENT-OPERATING-MODEL.md)（下称 Operating Model） |
| 正式任务、验收、Gate | [PERSONAL-DEVELOPMENT-PLAN.md](docs/plan/PERSONAL-DEVELOPMENT-PLAN.md) |
| 当前事实（task/Gate/claim/campaign） | [PROGRESS.md](docs/plan/PROGRESS.md) 的 `Current snapshot` |
| 当前可写路径（lease） | [PARALLEL-LANES.md](docs/plan/PARALLEL-LANES.md) 活动 lease 表 |
| 环境能力与命令路由 | [PERSONAL-TEST-ENVIRONMENTS.md](docs/plan/PERSONAL-TEST-ENVIRONMENTS.md) |
| 变更分类与文档联动 | [docs-sync-contract.md](docs/standards/docs-sync-contract.md) |
| 验证命令清单 | [handbook ai/validation-commands](personal/handbook/zh-CN/ai/validation-commands.md)（[en](personal/handbook/en/ai/validation-commands.md)） |
| 研究与任务卡细节 | [docs/plan/plan.md](docs/plan/plan.md)（非状态源） |

历史 handoff、旧提示词、分支名和聊天上下文不能覆盖以上来源。禁止读取或引用 `History/`。

## 1. 新会话启动顺序

1. 本文件 → [项目身份](docs/governance/PROJECT-IDENTITY.md) → [公理](docs/governance/AXIOMS.md)
   → [Operating Model](docs/governance/DEVELOPMENT-OPERATING-MODEL.md)；
2. 读 `PROGRESS.md` `Current snapshot`：先看 `Owner-directed campaign` 行（见 §2），再看
   `Active task lease` 行与唯一下一动作；
3. 读 `PARALLEL-LANES.md` 活动 lease 表，确认可写路径；
4. 读所选任务在正式计划中的 acceptance / Delivery Slice 定义，再读最新 matching handoff
   与 `docs/plan/plan.md` 任务卡；
5. 产品/架构任务再读 [Personal 产品设计](personal/docs/product/README.md) 与
   [Personal 架构](personal/docs/architecture/README.md) 的相关章节；
6. 快速恢复：核对 branch、clean/dirty、HEAD/upstream、Draft PR 与 checks、active lease；
   一致则不重复全仓 Git 审计（Operating Model「Fast resume protocol」）。

面向使用者/开发者/AI 工具的派生说明书在 [`personal/handbook/`](personal/handbook/README.md)
（双语；AI 入口 [`en/ai/README.md`](personal/handbook/en/ai/README.md)）。它不拥有任何任务、
Gate、合同或状态事实；改动实现时按 §5 与
[`_meta/sync-policy.md`](personal/handbook/_meta/sync-policy.md) 联动更新。

## 2. 会话路由：评测 campaign 优先于开发续跑

完整语义由 Operating Model §2.5 拥有；Cursor 适配见
`.cursor/rules/15-owner-directed-evaluation-campaign.mdc`。

- `PROGRESS.md` Current snapshot 的 `Owner-directed campaign` 行是评测模式的唯一开关。
  该行登记 **active** campaign 时，新窗口、`继续`、context compression 等 continuation
  event 一律恢复并执行该评测；**不得**领取或继续任何 `P*-T*` 开发任务。
- 评测是 measurement-only：不改产品代码、合同、负例、测试或 handbook 生成源来"补齐"
  能力；缺口如实记 `not-run`/`not_available`。写入前领取 `lease/personal/EVAL-<id>/<purpose>`，
  writable paths 只允许 `docs/evaluation/`、`docs/checkpoints/`、`docs/plan/PROGRESS.md`。
- 该行关闭后评测模式失效；评测结束**不会**自动恢复开发续跑，需 owner 重新给出持续
  交付指令。是否有 active campaign 只看该行，不复制到本文件。

## 3. 持续交付协议（Operating Model 摘要）

以下 ID 由 [Operating Model](docs/governance/DEVELOPMENT-OPERATING-MODEL.md) 定义，本节仅摘要：

- **`TASK-ATOMIC-DELIVERY-01`**（§2.1）：交付单位是一个完整正式任务。一个任务 = 一个
  task branch + 一个 Draft PR + 一个 task lease，连续完成全部 Slice、集成、负例、supported
  validation、正式验收评估与文档收口，直到可诚实标 `done`。Slice（`<task-id>/DNN`）只是
  任务内检查点，不单独开分支/PR/handoff；同一任务最多一个 `in-progress` Slice；有基础
  能力后下一切片必须接真实调用者或 durable authority outcome，不得连续堆叠 helper-only。
- **`CONTINUOUS-AUTONOMOUS-DELIVERY-01`**（§2.1）：owner 要求持续推进时，在当前 lease 内
  连续实施下一个最小垂直切片；checkpoint、commit、push、CI 轮次、阶段总结、上下文压缩都
  **不是**停止条件；除非需要 owner 决定、发现未知外部改动或任务完成，不发临时进度汇报。
- **`CAMPAIGN-BACKLOG-CONTINUATION-01`**（§2.1）：持续交付授权激活时，任务收口后在同一
  会话立即领取下一个 `implementation_requires` 已满足的就绪任务并开始首个垂直切片，直到
  Layer 1 Remaining = 0、owner 暂停/改范围或只剩 owner-only 边界。
- **`RESOLVE-BEFORE-BLOCKED-PROGRESS-01`**（§7）：可自行修复的代码、测试、格式、CI 配置、
  临时环境故障先修再跑，不登记 `blocked`；只有恢复路径耗尽且必须 owner 决策时才
  `blocked`，并带 `blocked_paths` / `blocked_task_ids` / `blocked_gate_ids` / owner / next action。
- **`CHECKPOINT-DELIVERY-01`**（§2.2）：checkpoint 是后台持久化事件，只在远程 CI、
  exact-revision Linux 验证或异常恢复需要 immutable revision 时创建；coherent、secret-free、
  归属清晰且通过本地 eligible checks 的进展可自动 commit/push 并更新同一 Draft PR。Draft
  PR 在完整 acceptance 满足前保持 Draft；失败/待运行检查存在时禁止合并。
- **`TEST-REPORT-INCREMENTAL-01`**（§3.1）：每个测试/验证单元完成后立刻追加到该活动唯一的
  running report，再开始下一个；`fail`/`not-run` 与 pass 同等及时记录。
- **MVP-first**（§2.1.1）：首个可运行路径只实现当前 acceptance 与既有安全不变量要求的最小
  授权面（owner-local、single-principal、task-scoped、daemon-issued，范围外 fail closed），
  不预建 RBAC、审批链、通用 capability 管理；但 daemon-only writer、SecretStore、
  persist-before-dispatch、budget/fencing、独立 verifier 与审计边界不可省略。
- **Handoff / 收口**（§4）：handoff 只在完整任务收口、真正外部阻塞、未知改动或 owner
  暂停时写一次。最终收口一次性完成：acceptance 逐条映射到实现/负例/证据 → supported
  validation + required CI（验证 revision = 待合并 HEAD）→ 同步正式计划、Current snapshot、
  trace、唯一 handoff 与 handbook → Draft 转 ready 并正常合并 → 关闭 lease、删除可删的
  task branch、本地切回并 fast-forward `main`、`git status` clean。任一步未完成则保持
  `in-progress`/`blocked` 并记录唯一 recovery action。
- **上下文压缩与跨窗口**：仓库文档是唯一持久记忆。暂停、换窗口或接近上下文限制前，把
  唯一下一动作、已完成/未完成项、精确 revision、PR 与 lease 写入 `PROGRESS.md` Current
  snapshot；新窗口按"恢复并继续"处理，不得要求用户重述背景，也不得重复已完成工作。

### 3.1 必须停下来问 owner 的情况（Operating Model §2.4）

只有下一动作属于以下之一时才停下并给出最小明确选择（先穷尽可逆的低范围路线）：

1. 暴露 secret 或把它移出 approved Secret Store / 非日志输入路径；
2. 破坏性或不可逆的仓库/数据操作、force push、改共享生产基础设施、在预注册流程外改
   隔离 campaign guest；
3. 未决的产品语义、规范语义、结构、release、support、安全策略、Gate/benchmark 阈值或
   默认 Agent 决定（ADR-0040/0046/0047 类固定分母 MVP Gate 自判定属 §2.3 授权，不在此列）；
4. 扩大文件系统、网络、进程、模型、secret、预算、权限或能力范围超出任务登记边界；
5. 绕过失败测试、required CI、分支保护、签名、review 或其他治理控制。

等待 CI、普通测试失败、本地代码缺陷、可恢复的环境故障都**不是**确认边界。

### 3.2 Standing operator authorization（Operating Model §2.3 摘要）

owner 已授予代理持续自主操作授权：推进已领取任务所需的远程验证、approved Secret Store
使用、最小权限提权、user-service/system 配置修改可自行执行。边界不变：secret 不进 argv、
普通配置、SQLite、日志、CI、测试输出、evidence 或聊天；操作最小范围、可审计、有
cleanup/rollback；远程验证只消费**已推送**的精确 revision。正式 Gate guest、
release/production promotion、force push、破坏性操作和超出任务的系统变更不在授权内。

## 4. 目录与变更边界（ADR-0054）

- `core/`（specs、conformance、crates、contracts-ts、tests/golden、docs）：架构合同与
  符合性资产；不得为实现改写（A6）；语义变更走 Lane-CTR。
- `personal/`（crates、apps、packages、deploy、handbook、tests、docs）：唯一活动实现面；
  `tools/` 为共享检查工具面。SQLite schema 变更沿用编号 migration 机制（见 handbook
  [developer/store-and-migrations](personal/handbook/zh-CN/developer/store-and-migrations.md)）：
  新增编号步骤，不改写已合并步骤；结构型变更需 ADR + 迁移说明。
- Cargo/pnpm 依赖方向固定：`core → personal → clients`；core crate 永不依赖 personal。
- `enterprise/`：设计层；未经 owner 按 [VERSION-1.0.0.md](enterprise/docs/VERSION-1.0.0.md) §4
  激活不得实现。
- `clients/`：自有治理；Web UI 唯一实现路径 `clients/pc/web/`；产品源是 daemon 提供的
  `/ui/`，Vite preview 不是产品源。`clients/legacy/cognitiveos-console/` 只维护台账。
- `docs/governance/`、`docs/plan/`、`docs/checkpoints/`：共享治理、正式计划、快照与移交。
  closure/report 沿用 `docs/checkpoints/<YYYY-MM-DD>-personal-<task-id>-{report,closure}.md`。
- `personal-blog/` 是独立仓库，禁止推入本仓库；`artifacts/` 已 gitignore，其中
  `artifacts/personal-2.0.0-dev-prep/` 等第三方参考仓库副本**只读**，不修改、不暂存、不作为
  后续任务的隐式输入；下载的 package/installer/evidence payload 只放 `artifacts/` 或系统
  临时目录，Git 只记录 digest、attestation reference 与脱敏事实。
- `.cursor/skills/` 下绝大多数是导入的第三方 skill（各目录 `SOURCE.txt` 记录来源）：第三方
  内容只读，除非修正明显的路径引用错误；skill 路由规则见 `.cursor/rules/30-*.mdc`、`40-*.mdc`，
  其引用的真实性由 `pnpm run check:rules` 校验。

变更必须声明 `implementation-only`、`corrective`、`product-semantic`、`normative-semantic`
或 `structural`，并按 docs-sync-contract 完成联动。提交/PR 必须关联 Personal 任务或
REQ/F/IMP；没有关联时说明原因。写入前必须持有精确路径的活动 lease；未知工作树改动不得
覆盖、回退、混入或使用 `git add -A`（A8）。

## 5. 改了什么 → 跑什么 → 联动什么（快速决策表）

命令全文与平台限制以 [validation-commands](personal/handbook/zh-CN/ai/validation-commands.md)
为准；环境路由以 [PERSONAL-TEST-ENVIRONMENTS.md](docs/plan/PERSONAL-TEST-ENVIRONMENTS.md) 为准。

| 改动路径 | 本机（Windows）必跑 | 必须路由到 supported CI / exact-revision Linux | 文档联动 |
|---|---|---|---|
| 任何提交 | `node tools/src/docs-sync-gate.mjs --staged`（hooks 已注册则自动）；`git diff --check` | — | 命中 `source-map.json` 的页面（双语）+ 指纹；否则 `DOCS_IMPACT_NONE="<具体理由>"` 并记入 commit/PR |
| `core/specs/**`、`core/conformance/**` | `pnpm run check:consistency`；`node tools/src/gen-matrix.mjs --check` | codegen 再生成 + `git diff` 生成目录；conformance runner；golden digest | Lane-CTR 联动（registry/schema/bindings/transitions/vectors 一体）；`ref.*` handbook 页 |
| Rust（`core/crates/**`、`personal/crates/**`、`personal/apps/**`） | `cargo fmt --all -- --check`；在 MSVC override 目录（§6）可先本机 `cargo build/test/clippy --workspace --locked` 迭代（开发证据，不替代右列） | `cargo build/test/clippy --workspace --locked`；focused failure-first test | `dev.*` / `ref.*` 映射页；HTTP 路由变化需生成页重生成 |
| TypeScript（`core/packages/**`、`personal/packages/**`、`personal/apps/agent-shell`、`tools/**`） | `pnpm -r build`；`pnpm -r test` | 同左（CI 复跑） | 映射页；`tools/` 变化联动 `meta.sync-policy` / `dev.conformance-testing` |
| `clients/pc/web/**` | `pnpm test`、`pnpm build`（在该目录） | daemon `/ui/` 静态服务测试（Rust） | `clients/docs/**` 自有治理 |
| `personal/handbook/**` | `node tools/src/check-handbook.mjs`；`node tools/src/generate-handbook.mjs --check`；手写页改后 `node tools/src/fill-handbook-fingerprints.mjs` | — | 双语同改；生成页只能重生成 |
| `docs/plan/**`、`docs/governance/**`、`AGENTS.md`、`.cursor/rules/**` | `pnpm run check:consistency`；`pnpm run check:rules` | — | 需 DOC/GOV lease；规则只摘要不定义第二套语义。Phase 14 `/ui/` 闭环另走 `JOURNEY-BROWSER-SYNC-01`（正式计划命名节 + 环境登记 §5.4；产品源 = daemon `/ui/`） |
| `.github/**`、根 manifests、`tools/package.json` | `pnpm run check:rules`；相关 handbook 页 | CI 自身 | `dev.contributing-workflow` / `dev.dev-environments` / `ai.validation-commands` + 指纹 |

Ready/merge 前额外：`node tools/src/check-handbook.mjs`、`node tools/src/generate-handbook.mjs --check`、
`pnpm run check:consistency` 本地绿，required CI 在待合并 HEAD 上绿。

## 6. 本地命令与测试环境预路由

- **`COMMAND-SHELL-PS51`：** Cursor 本地 Shell 按 **Windows PowerShell 5.1** 解析。禁止
  `&&` / `||`；互不依赖的命令用独立并行 Shell 调用，有依赖的命令用
  `if ($LASTEXITCODE -eq 0) { <next-command> }` 或拆成后续调用。解析器拒绝的命令记
  `not-run`，不是测试失败。只有明确进入 bash（CI、`CLOUD-AGENT-LINUX-01`、Linux 主机）才可
  用 bash 连接符。PowerShell 启动慢：能用 Read/Grep/Glob 完成的文件操作不用 Shell。
- **`RUST-LINK-DEV-WIN-GNU-01`（GNU host 历史事实）：** 本机 rustup 默认 host 是
  `x86_64-pc-windows-gnu`；在该 host 下 workspace build/test/Clippy/run/bench 稳定失败于
  linker exit 121（2026-07-25 基线，LLVM-MinGW/shim 重试已穷尽）。**本机 MSVC override（2026-09-03
  `P0-T01/D02`，owner 选定本机 override）：** `D:\agent-kernel` 与登记的任务 worktree 目录级
  `rustup override set 1.97.1-x86_64-pc-windows-msvc`（存于 rustup settings，不在仓库；tracked
  `rust-toolchain.toml` 未改，CI 不受影响；linker 为 `D:\VSBuildTools` 的 `link.exe` 14.44）。
  在 override 目录内、且 `rustc -vV` 报 `host: x86_64-pc-windows-msvc` 时，允许本机运行
  `cargo build/test/clippy/fmt`（结果只是本地开发证据；本机磁盘紧张，测试构建需会话变量
  `CARGO_PROFILE_DEV_DEBUG=0`；未提权 shell 下 4 个 symlink fixture 测试以 OS 1314 失败，记
  `not-run (host privilege)`，不得跳过）。新 worktree 需自行
  `rustup override set`；未 override 的目录仍是 GNU host，禁止在那里重跑 linking 命令，也禁止
  用 PATH、shim、`rust-toolchain.toml` 或源码 workaround 绕过。**能力上限不变**：本机结果不是
  supported CI、Gate、release、Profile 或 Windows 支持证据；Slice 的 supported validation 仍路由
  到 `CI-UBUNTU-01`、`CI-WINDOWS-MSVC-01` 或 exact-revision `DEV-LINUX-NATIVE-01`；环境不可用
  时记 `blocked`/`not-run`。登记细节见环境登记 §3。
- 本机若 `cargo` 不在 PATH：`$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"`；这只恢复
  钉住工具的发现（版本以 `rust-toolchain.toml` / 目录 override 为准）。
- `pnpm run verify:local`（V01 编排器，`scripts/v01-auto-run.*`）自 2026-09-03（`P0-T01/D02`，
  owner 选项 A 重钉）起**只在 MSVC override 目录内**可本机使用：计数已重钉到 `ci.yml` 的
  89/62/27（repo-tools 测试守卫两份数字一致），需 pwsh 7 与会话变量 `CARGO_PROFILE_DEV_DEBUG=0`；
  它跑 `cargo build`、focused `cargo test`、符合性 runner（含 `--self-check`）与 `check:consistency`，
  产出 `artifacts/evidence/v01-auto-run/<run_id>/summary.json`。结果只是本地开发证据，不替代
  required CI，也不升 Gate/release/Profile；非 override 目录仍禁止。日常仍优先用 §5 的单项命令。

| 目的 | Windows PowerShell（本地） | CI / Linux（bash） |
|---|---|---|
| Rust 构建 / 测试 / lint | 仅在 MSVC override 目录（`rustc -vV` host = `x86_64-pc-windows-msvc`）：同右列命令，结果为本地开发证据；GNU host 目录禁止（路由） | `cargo build --workspace --locked`；`cargo test --workspace --locked -- --test-threads=1`；`cargo clippy --workspace --all-targets --locked -- -D warnings` |
| TS 安装 / 构建 / 测试 | `pnpm install --frozen-lockfile` ; `pnpm -r build` ; `pnpm -r test` | 同左 |
| 静态一致性 / handbook / 规则引用 | `pnpm run check:consistency` ; `pnpm run check:handbook` ; `pnpm run check:rules` | 同左 |
| 符合性 runner | 仅在 MSVC override 目录（同上；含 `pnpm run verify:local` 一次性编排器）；GNU host 目录禁止（路由） | `cargo run -p cognitive-conformance --bin conformance-runner` |

## 7. 远程验证主机与 campaign guest 隔离

权威登记与端口/隔离细则见 [PERSONAL-TEST-ENVIRONMENTS.md](docs/plan/PERSONAL-TEST-ENVIRONMENTS.md)
§7 / §10；本节只列操作要点。

- `DEV-LINUX-NATIVE-01`（`wuz@192.168.1.2`，libvirt host `hal9000`）是 Linux daemon、
  Pi/sidecar、installer、user-service、native integration 切片的首选执行环境。先把**已提交
  并推送**的 revision 同步到该主机的可清理 Git worktree，在 exact revision 上构建、测试、
  记录证据；不复制未提交代码，不用无 Git 的 source snapshot。它只支持
  `experimental-local-only` / `tested-local`，不是 B01、release、Profile 或正式 Gate 环境。
  每个 Pi 切片重新确认 Linux/native user-systemd、Rust/Node、exact Pi pin 与可清理目录。
- SSH 仅非交互、无 secret 探针：`ssh -o BatchMode=yes -o ConnectTimeout=10 "wuz@192.168.1.2" "<redacted command>"`。
- `B01-Desktop-Linux-002`（`hal9001@192.168.123.160`，经 ProxyJump `wuz@192.168.1.2`；
  host 上用 `virsh -c qemu:///system`）是唯一活动的专用 B01 campaign guest，同时（环境登记
  §10）登记为 owner 授权的 Personal 2.0 开发验证主机：只用 exact-revision 一次性 worktree 与任务声明
  的可清理目录；不得改 guest 基线、快照或凭据；预注册 B01 campaign 活动期间冻结开发用途。
  `B01-Clean-Linux-001` 已退役，禁止恢复、启动、重置、部署或删除。
- **Owner 本机查看 UI（guest 部署/调试后的默认收尾）**：在 PowerShell 建立转发
  `ssh -J wuz@192.168.1.2 -L 48681:127.0.0.1:48681 -L 3080:127.0.0.1:3080 hal9001@192.168.123.160`
  并保持会话，本机打开 `http://127.0.0.1:48681/ui/`（Control Plane，门禁粘贴 runtime 的
  bootstrap secret，不是 Provider key）与 `http://127.0.0.1:3080/`（dsh 面板）。端口以 guest
  上 `cognitive daemon status` 为准。daemon 重启后须重启 `cognitive dsh web`；`cognitive dsh apply`
  不能恢复 `INACTIVE` 的 stale session（见
  [dsh 恢复记录](docs/bug/dsh-pathb-stale-daemon-bearer-after-daemon-restart.md)）。**每次
  guest 调试/验证回合末尾必须重复给出转发命令与本地 URL。**

## 8. 安全红线（速查）

公理全文只在 [AXIOMS.md](docs/governance/AXIOMS.md)；日常提醒：

1. Rust daemon 是唯一 authority writer（A1）；概率组件与第三方 agent 只产 candidate（A2）。
2. 外部 mutation 必须 persist-before-dispatch Intent/Effect（A3）；独立 verification 才可完成 Task（A4）。
3. Secret 只进 approved Secret Store（A5）；`.cursor/mcp.json`（已 gitignore 的本机文件，
   不得 force-add）、`.cursor/environment.json`、CI 配置、测试 fixture 与 evidence 中不得出现
   token/key 明文，只允许环境变量引用。
4. 合同与负例不得为实现而削弱（A6）；本地/fixture/WSL/ordinary CI 证据不得提升为
   Gate/release/Profile（A7）；未知工作树改动受保护（A8）。
5. Git：禁止 force push、amend 已推送历史、`--no-verify` 跳过 hooks、`git add -A`、直接在
   `main` 累积实现、合并带失败/待运行检查的 PR。
6. 只读区域：`History/`（禁读禁引）、`artifacts/` 第三方副本、`.cursor/skills/` 第三方
   skill、生成目录（`core/crates/cognitive-contracts/src/generated`、
   `core/packages/contracts-ts/src/generated`、handbook `generated: true` 页、
   `docs/traceability/matrix.yaml`、`core/tests/golden/*.json`）只能经生成器再生成。
