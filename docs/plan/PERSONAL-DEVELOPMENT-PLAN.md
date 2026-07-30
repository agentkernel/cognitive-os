# CognitiveOS Personal 产品化开发计划与进度表

> **状态：in-progress（P0-T01..T07、P1-T01..T08 已完成；P1-T09 implementation in-progress，B01 not-run；P2 及以后正式验收尚未开始）**
> **最后更新：2026-07-30**
> **计划追踪 ID：** `P0-T01` 至 `P7-T08` 是本计划的管理 ID，不是 `specs/registry/` 中的 REQ-ID，也不构成实现、测试或 Profile 符合性声明。
> **详细研究与任务卡草案：** 仓库根目录 `plan.md`；本文件是后续开发的**正式入口和唯一进度台账**。任务 ID 的名称、范围、依赖和阶段 Gate 以本文件为准；`plan.md` 只补充经本文件对齐的研究依据、实施细节与验收方法。
> **可机读追踪：** [personal-trace.yaml](personal-trace.yaml) 将 `PERS-PR`、本计划任务与 Gate/benchmark 对齐；它不是 registry matrix，且不构成 REQ、测试执行或 Profile 符合性声明。

> **开发状态解耦（2026-07-30 修订）：** `not-started` 表示尚无任务专属设计、实现或测试批；首个真实任务批开始后必须改为 `in-progress`。`done` 才表示完整正式验收已满足。后续 P1/P2/P3 工作可在
> `experimental-local-only` 开发轨道推进，并将真实执行记录为 `tested-local`；这两个
> 是开发/证据标签，不是 Gate、Profile 或 release 结论。任务状态、实现证据、Gate 状态与声明范围按 [Development Operating Model](../governance/DEVELOPMENT-OPERATING-MODEL.md) 分列；真实 Provider/Pi 测试仍须使用当前 SecretStore/daemon 边界并脱敏记录。

> **计划修订（2026-07-26，一致性评审批）：** (a) 新增 P7-T07，为 ADR-0025 已决定但此前
> 无任务归宿的 Windows x86_64 安装面（credential 后端、installer/service、专门
> B01-W Gate）建立唯一落点；未执行前不得声称 Windows install parity，且不阻塞
> Linux RC。(b) P2-T08 验收增补 ADR-0018 本机开发例外的到期移除核查，使"P2 结束到期"
> 有明确验收归属。(c) `plan.md` 与本台账对齐：修正 P1-T04 过期状态行、任务卡状态行改为
> 指向本台账、ADR 候选表改用 DEC-P-* 编号以消除与 `docs/adr/0017-0025` 的编号冲突、
> 依赖图补 P7-T07 并修正 critical path。本批不改变任何既有任务状态、Gate、证据或
> Profile 结论。

> **MVP-first 路线修订（2026-07-29，ADR-0034）：** 首个生产安装路径收敛为
> 一个 canonical user service（`cognitiveos-personal.service`）和一个固定
> loopback 端口（48181），接受 Alpha 显式升级的有界停机。ADR-0032/0033
> 的 candidate unit、48182 和双服务 promotion 保留为后续数据触发的升级优化，
> 不再阻塞 P1-T08/P1-T09。既有任务 ID 不重编号；新增 `P7-T08 / GMVP-LINUX`
> 作为 P1/B01、P2 与 P7-T01..T03 的公开 Linux MVP 汇合 Gate。Context、Memory、
> Agent/Tool、Windows 安装面、Web UI 和 Multi-Agent 改为后续独立能力列车；
> Multi-Agent NO-GO 且默认关闭是合法结果。本修订不改变任何既有任务状态、
> 已执行证据、规范机器资产或 Profile 结论。

> **计划修订（2026-07-26，生产就绪与低摩擦授权批）：** 依 owner 指令与
> [ADR-0026](../adr/0026-personal-trust-profile-low-friction-authorization.md)
> 落地 Personal 低摩擦授权模型（DEC-P-20）：交互分层 Tier 0/1/2、任务准入预览为
> 唯一默认人工授权点、预算硬轨替代逐动作审批、不建审批链；同时补 P7-T02 面向用户
> 的 backup/restore（排除 secret）。仅修改 not-started 任务的验收摘要
> （P1-T09、P2-T01、P2-T02、P2-T08、P5-T01、P5-T02、P7-T02），治理层
> （Intent/Effect、audit、verifier、capability 类型）全保留。本批不改变任何既有
> 任务状态、Gate、证据或 Profile 结论。

## 1. 使用与更新规则

1. 首个任务专属设计、实现或测试批开始时，在本表把对应任务标为 `in-progress`，填入负责人/分支、开始日期和关联 PR（如有）；若未达到产品 Gate，额外填写 `development_track: experimental-local-only`，不得把 acceptance/promotion Gate 写成实现阻断。
2. 一个任务只有在其验收条件满足、相关测试真实执行并留有证据后，才可标为 `done`；未执行的测试必须明确标 `not-run`，不得推断为通过。
3. 每个 atomic delivery/PR 必须更新该任务的状态、证据链接或命令结果及阻塞项；实现 commit 后可跟一个 closure docs commit 记录其 immutable hash。PROGRESS 与 handoff 必须在 merge 或会话移交前完成，不要求与实现位于同一 commit。
4. 发生范围、依赖、验收或安全边界变化时，先将任务标为 `blocked`，记录原因和决策，再更新详细任务卡与依赖图；不得静默改写完成标准。
5. 允许的任务状态为：`not-started`、`in-progress`、`blocked`、`done`、`cancelled`；实现证据为 `none` / `provided` / `tested-local` / `tested-supported-ci`；Gate 为 `not-run` / `running` / `pass` / `fail` / `blocked`。`done` 不等于 Gate pass 或 Profile `implemented`。
6. 如本表与 `plan.md` 的任务卡或依赖图不一致，应先按本表执行并在同一文档修正批中对齐 `plan.md`；不得仅凭详细卡片重新解释或复用既有 `P*-T*` ID。

### 进度汇总

| 阶段 | 任务数 | done | in-progress | blocked | not-started | 阶段 Gate |
|---|---:|---:|---:|---:|---:|---|
| Phase 0 - 基线与决策 | 7 | 7 | 0 | 0 | 0 | G0 |
| Phase 1 - 安装到首次对话 | 9 | 8 | 1 | 0 | 0 | G1 / B01 `not-run` |
| Phase 2 - 单 Agent 任务闭环 | 8 | 0 | 0 | 0 | 8 | G2 / B02、B04、B05、B12 |
| Phase 3 - Context 与效率 | 6 | 0 | 0 | 0 | 6 | G3 / B03、B06、B07 |
| Phase 4 - Memory | 6 | 0 | 0 | 0 | 6 | G4 / B08 |
| Phase 5 - Agent 与 Tool 生态 | 5 | 0 | 0 | 0 | 5 | G5 / B09、B10 |
| Phase 6 - Multi-Agent | 4 | 0 | 0 | 0 | 4 | G6 / B11 |
| Phase 7 - 产品化与发布 | 8 | 0 | 0 | 0 | 8 | GMVP-LINUX / G7 / RC |
| **合计** | **53** | **15** | **1** | **0** | **37** | — |

## 2. 产品边界与不变量

- Rust daemon 是唯一 authority writer；Pi、CLI、Web UI 均为客户端，不可直接写 SQLite 或推进 Task、Effect、Verification 状态。
- Provider API Key 只保存在原生 Secret Store；不得进入配置、SQLite、命令行、日志或证据。唯一例外为 ADR-0018 已登记的 P0-T06 本机 Linux 开发路径：显式开关后从 native store 解析、仅传给初始 Pi 子进程、默认拒绝、不得用于 CI/发布，并在 P2 结束到期。
- Linux-native development smoke 以环境资格清单为准；`wuz@192.168.1.2` 是优先候选而非唯一主机。只有预注册 formal campaign 可推进 B01，任何候选主机名称本身都不表示测试、Gate 或 release 证据。
- 所有外部 mutating operation 均须经 Intent/Effect、持久化后派发、幂等键、fencing 和结果 reconcile；外部工具成功不等于 Task 完成。
- Task 完成由独立 verifier/acceptance authority 推进；Pi Session 不等于 Task，Pi `agent_end` 不等于完成。
- Personal 计划不改变既有规范优先级，不得用 `PERS-*` ID 冒充 REQ-ID；合同变化必须走 Lane-CTR 流程。
- **低摩擦授权（ADR-0026）：** 治理记录（Intent/Effect、audit、verifier、capability）全保留；人机交互分层——Tier 0（只读与任务范围内可逆本地写）静默自动授权、Tier 1（幂等/可对账外部 mutating）首用一次授予并默认记住为 capability lease、Tier 2（不可逆/毁灭性/超预算）始终显式确认。任务准入预览是唯一默认人工授权点，默认路径人工确认 ≤1/task；预算与边界是硬轨，不建审批链；企业审批留在 Deferred Backlog。
- 产品目标平台仍为 **Linux x86_64 + Windows x86_64**（ADR-0025）；首个公开 MVP 是 Linux x86_64 single-service bundle（ADR-0034）。Windows 的 credential 后端、安装面与专门 Gate 的唯一任务归宿是 P7-T07；其证据齐备前，任何 install/B01 声明仅覆盖 Linux。Memory、MCP、Multi-Agent、Web UI 与 Windows 安装面均不阻塞 `GMVP-LINUX`。

## 3. 阶段路线图

下表的入场条件和“禁止提前作为产品主路径”只约束任务 `done`、产品集成、推广和声明范围；不禁止满足 `implementation_requires` 的隔离实现与 failure-first 测试。具体依赖必须区分 implementation、acceptance 与 promotion，禁止把后两者当作开发互斥锁。

| 阶段 | 目标 | 入场条件 | 出场条件 | 禁止提前作为产品主路径 |
|---|---|---|---|---|
| P0 | 冻结平台、架构和安全决策 | 本计划批准 | 工具链、ADR、Secret/Pi PoC、benchmark 规格完成 | 产品功能、Memory、Multi-Agent、UI |
| P1 | 从安装到受治理的首次对话 | G0 | 干净 Linux VM 的 B01 通过 | Task autonomy、Memory、MCP、多 Agent |
| P2 | 单 Agent 可恢复任务闭环 | B01 | B02/B04/B05/B12 通过 | Memory、embedding、多 Agent |
| P3 | Context、Token 与 Loop 效率 | P2 稳定 | B03/B06/B07 通过，指标可采集 | Memory consolidation、多 Agent |
| P4 | 有 provenance 的 durable Memory | P3 基线冻结 | B08 通过，Embedding 有明确 go/no-go | 自动跨工作区记忆 |
| P5 | 可审核的 Agent/Tool 生态 | P4 稳定 | B09/B10 通过 | 自动市场发现 |
| P6 | 有收益证据的 Multi-Agent 可选实验 | 单 Agent benchmark 稳定且存在可并行收益假设 | B11 产生 GO 或保持默认关闭的 NO-GO | 未达收益 Gate 即默认启用多 Agent |
| P7 | Linux MVP、能力列车与完整 RC | P2 稳定后可启动 Linux MVP 发布可运维链 | `GMVP-LINUX` 后按声明范围汇合完整 RC | 用未执行能力扩大 MVP/RC 声明 |

### MVP-first release train（不替代现有任务 ID）

| Release train | 任务范围 | 出口 | 不阻塞该出口的能力 |
|---|---|---|---|
| RP1 Foundation | 已完成的 P0 与 P1-T01..T07 | 当前实现基线 | 后续产品能力 |
| RP2 Install-to-Conversation Alpha | P1-T08、P1-T09 | B01 clean Linux VM | Task、Memory、MCP、Multi-Agent、UI、Windows installer |
| RP3 Governed Single-Agent MVP | P2-T01..T08 | B02/B04/B05/B12 | Context 优化、Memory、生态、多 Agent |
| RP4 Public Linux MVP | P7-T01..T03、P7-T08 | `GMVP-LINUX` | P3..P6、P7-T05、P7-T07 |
| RP5 Context Efficiency Beta | P3-T01..T06 | B03/B06/B07 | Memory 与生态 |
| RP6 Durable Memory Beta | P4-T01..T06 | B08；embedding GO/NO-GO | embedding GO、生态、多 Agent |
| RP7 Optional Capability Trains | P5、P6、P7-T05、P7-T07 | 各自独立 Gate | 任何未声明能力 |
| RP8 Full-Scope RC | P7-T04、P7-T06 与已选择能力 | 声明范围内 B01-B12 / RC | 明确 DEFER/NO-GO 的能力 |

### 本机实验轨道（不改变产品 Gate）

`experimental-local-only` 允许在正式阶段 Gate 尚未完成时并行实现和测试
P1/P2/P3 及后续 Personal 功能。它只表示代码和测试被限制在本机、临时目录或
明确隔离的实验环境；`tested-local` 只表示该次命令真实执行过。两者都不能填写
为 `implemented`，不能生成 Profile/release 证据，也不能把 sample、fixture、
smoke 或局部测试升级为产品验收。

允许并行推进的工作包包括：

- Pi proxy、Task runtime、scheduler、Effect、recovery、Context、Memory；
- 真实 Pi Extension/RPC load（只保存脱敏的事件、计时和状态结果）；
- `kernel/store/runtime` benchmark harness；
- Personal 端到端性能 runner，并明确拆分 CognitiveOS deterministic overhead、
  Pi/Node process overhead、Provider/network/model latency、filesystem/SQLite overhead。

实验轨道的硬约束仍与正式路径相同：secret 不得进入日志、argv、普通配置、
SQLite、证据或 CI；Pi、CLI、SDK、UI 不得成为 authority；状态迁移、授权、CAS、
预算、幂等、fencing、Effect 提交和完成验收只能由确定性服务端代码执行；规范向量
只能被真实规范修正流程修改，不能为实现迎合。

后续 Linux-native development smoke 使用环境资格清单，而不是绑定唯一主机：Linux x86_64、non-WSL、native user-systemd、支持的 Secret Service、可清理/重置、exact artifact/Pi pins 和脱敏 evidence collector。`wuz@192.168.1.2` 是优先候选而非唯一允许环境。只有预注册的 formal campaign 环境能推进 B01；其他合格主机仍按 `experimental-local-only` / `tested-local` 记账，并与 CI Ubuntu、Windows/MSVC 证据分开陈述。

正式平台 campaign 与 A/B/C/D agent-benefit 测评后置到固定平台、固定拓扑、
预注册 workload 和独立 verifier 准备完成之后；在此之前所有 Personal 性能结果
均属于 local experimental evidence，收益字段必须保持 non-claim。

## 4. 任务进度表

> **填写约定：** “证据/备注”须写测试命令、evidence digest、PR、handoff 或阻塞原因。首次领取时只改当前任务，避免并行会话争用本文件。

### Phase 0 - 基线与决策

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P0-T01 | 固定可复现基线与支持工具链 | — | Linux runner 与支持的 Windows 工具链结论可复现 | done | 2026-07-25；Lane-DOC `lane/personal-p0-t01-baseline-2`。基线 `01ceb93`：CI run [30140381194](https://github.com/agentkernel/cognitive-os/actions/runs/30140381194) 的 Ubuntu/Windows jobs 均成功；本机 `pnpm install --frozen-lockfile`、3 次 `pnpm -r build` + `pnpm -r test`（p50 29.669s）和 `cargo fmt --all -- --check` 通过。本机 Windows GNU `cargo build --workspace --locked` 在 linker exit 121 下失败，LLVM-MinGW/shim 重试同样失败；因此支持组合明确为 CI Linux 与 Windows/MSVC，GNU host 非支持基线。详见 [tests/baseline/README.md](../../tests/baseline/README.md)。 |
| P0-T02 | 冻结 Personal 需求、追踪与架构边界 | P0-T01 | PERS-PR、任务、benchmark 无孤儿映射 | done | 2026-07-25；Lane-DOC `lane/personal-p0-t02-trace`。新增 [personal-trace.yaml](personal-trace.yaml)：20 个 PERS-PR、51 个正式任务和 21 个 Gate/benchmark 均可交叉核对；仅引用真实登记 REQ，product-only 项明确为空映射，所有证据状态为 `not-run`。验证：专用 Node 映射核对、`pnpm run check:consistency`、`git diff --check` 均通过。 |
| P0-T03 | License、首发平台与分发决策 | P0-T02 | owner GO/NO-GO 与 notices 完整 | done | 2026-07-26；PR [#99](https://github.com/agentkernel/cognitive-os/pull/99) 合入 `main@fd6ff6b`。Owner GO：Apache-2.0；首发产品平台 Linux x86_64 + Windows x86_64；GitHub Release 可检查 bundle；**不** vendor Pi/Node；crates.io/npm 仍不发布。交付：根 `LICENSE`/`NOTICE`、[ADR-0025](../adr/0025-personal-license-platform-distribution.md)、[THIRD-PARTY-NOTICES](../legal/THIRD-PARTY-NOTICES.md)、[PERSONAL-SUPPORT-MATRIX](PERSONAL-SUPPORT-MATRIX.md)；workspace `license=Apache-2.0` 且 `publish=false`/`private=true`。验证：`pnpm run check:consistency`、`git diff --check`；CI [30180002937](https://github.com/agentkernel/cognitive-os/actions/runs/30180002937) / [30179991223](https://github.com/agentkernel/cognitive-os/actions/runs/30179991223) Ubuntu/Windows-MSVC SUCCESS。非 G0/B01-B12/Profile；SBOM/attestation 归 P7-T01。handoff：[20260726-personal-p0-t03-license-platform-distribution-handoff.md](../checkpoints/20260726-personal-p0-t03-license-platform-distribution-handoff.md)。 |
| P0-T04 | 数据布局、迁移、备份与回滚设计验证 | P0-T02 | migration dry-run、重放与失败恢复评审 | done | 2026-07-25；Lane-KRN `lane/krn-personal-p0-t04-migrations`，PR #89。ADR-0017 + adapter-local `schema_migrations` dry-run/apply/replay/digest-drift/failure-recovery tests；CI run [30150183941](https://github.com/agentkernel/cognitive-os/actions/runs/30150183941) 在 Ubuntu 与 Windows/MSVC 均通过 `cargo test --workspace --locked`。本机 Windows GNU linker exit 121 是已知非支持环境，不再阻断。未修改 registry/schema/vector 或 authority transition 语义。 |
| P0-T05 | Linux Secret Service PoC | P0-T01 | set/get/rotate/delete 与泄漏负例通过 | done | 2026-07-25；分支 `lane/personal-p0-t05-secret-store-api`，PR [#90](https://github.com/agentkernel/cognitive-os/pull/90)。隔离 crate `cognitive-secret` + ADR-0018：冻结 `SecretStore::{probe,put,get,delete}` 与 opaque `SecretRef`；模拟 put/get/rotate/delete、absent/locked/prompt fail-closed、Debug/Display/env 泄漏负例；Linux native probe-only（mutating D-Bus 归 P1-T02）；无明文 fallback。CI run [30153311857](https://github.com/agentkernel/cognitive-os/actions/runs/30153311857) Ubuntu/Windows-MSVC `cargo test --workspace --locked` 通过（含 `p0_t05_secret_store`）。本机 Windows GNU linker exit 121 为非支持基线。非 G0/B01-B12/Profile 声明。handoff：[20260725-personal-p0-t05-secret-store-api-handoff.md](../checkpoints/20260725-personal-p0-t05-secret-store-api-handoff.md)。 |
| P0-T06 | Pi 版本、Extension 与 RPC 兼容性 PoC | P0-T03 | 固定版本、integrity、Extension/RPC fixture 通过 | done | 2026-07-27；Lane-RUN `lane/run-personal-p0-t06-extension-poc`。第一个原子部分已固定 `@earendil-works/pi-coding-agent@0.81.1`、npm SRI、source commit、repository path 与 Node engine；candidate adapter 在读取 scoped Provider key 前以 `pi --version` fail-closed 拒绝版本漂移；strict-LF JSONL parser fixtures覆盖 CRLF normalization、U+2028 preservation、malformed/non-object negative。第二个独立原子部分新增 pinned-API Extension fixture：`project_trust` 固定拒绝、`write`/`edit`/`bash` 在 `tool_call` 前阻断、`session_start` 仅展示 session-local status，且 Rust safety guard 断言无 provider credential/durable-state access。Owner 于 2026-07-26 批准默认关闭的本机 Linux 开发例外：只有显式 `--allow-local-native-provider-secret-development` 与独立 `--provider-config-dir` 同时提供时，adapter 才从 `ProviderKeyService`/native Secret Service 解析已配置的 DeepSeek key，并只注入初始 Pi 子进程；不读取父进程 Key 环境变量，Windows/CI/unavailable backend fail-closed，例外在 P2 结束到期。后续 Linux-native 本地执行优先使用相和歌设备 `wuz@192.168.1.2`，并继续与 WSL/CI 证据分列。WSL test：`CARGO_TARGET_DIR=/tmp/cognitiveos-p0-t06-exception-target-two cargo test -p pi-agent-adapter --offline`（16 passed）。无 Provider key 的 RPC clean-run 无法自然退出，未将 CLI version check 表述为 Extension runtime load evidence。第三个原子部分新增 `extension-load` 证据动词：只接受 pinned fixture 路径与已注册的 `/cognitiveos-p0-t06-status` 命令，以 `--mode rpc` 驱动一次真实 Pi 子进程会话，读取 `get_commands`/`get_state`/`prompt` 三条 RPC 响应，30s 超时后强杀，并只输出**脱敏且不含原始输出**的证据记录（固定 `authority_committed=false`/`effects_created=false`/`task_transitions=0`/`capabilities_granted=0`/`classification=uncontained_candidate_only`）；同时把 host 分类从 `cfg!(target_os)` 升级为读取 `/proc/version`、`/proc/sys/kernel/osrelease` 与 WSL 环境变量，使 WSL2 guest 在解析任何 credential **之前**即被拒绝。2026-07-27 在 `wuz@192.168.1.2` 上实际执行 `extension-load` probe，证据记录已脱敏并核对：`extension_command_registered=true`、`session_start_hook_observed=true`、`status_command_observed=true`、`status=executed`、`raw_output_included=false`、`output_redacted=true`、`authority_committed=false`、`effects_created=false`、`task_transitions=0`、`capabilities_granted=0`；这仍是 PoC / non-claim evidence，不构成 containment、Profile 或 release claim。2026-07-26 本机 Linux 工具链恢复后已在 WSL2 guest 真实执行受支持的测试面：`cargo test --workspace --locked` **358 passed / 0 failed（67 suites）**、clippy `-D warnings` 通过、`cargo fmt --all -- --check` 通过、`pnpm -r build`/`pnpm -r test`/`check:consistency` 通过（`tested-local`，平台标签 `windows_wsl2_linux_guest`）。archive integrity/source provenance verifier 仍归 Pi P2。非 G0/B01-B12/C0/C1/Profile 声明。 |
| P0-T07 | daemon transport、认证和威胁模型 | P0-T02 | transport 限制与威胁模型评审完成 | done | 2026-07-25；分支 lane/personal-p0-t07-daemon-transport-threat-model，PR #91 合入 main@ff341ef；CI run 30154100260。ADR-0019 冻结：默认 HTTP/1.1 over UDS（XDG runtime）、可选 loopback TCP、disabled-by-default listener、channel-scoped bearer bootstrap、请求/并发/会话上限；threat model 覆盖 CSRF、DNS rebinding、token theft、channel confusion、replay。不实现业务路由；非 G0/B01-B12/Profile 声明。handoff：[20260725-personal-p0-t07-daemon-transport-threat-model-handoff.md](../checkpoints/20260725-personal-p0-t07-daemon-transport-threat-model-handoff.md)。 |

### Phase 1 - 安装到首次对话

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P1-T01 | 版本化数据库迁移与 XDG 布局 | P0-T04 | upgrade/reapply/corruption/disk-failure 测试 | done | 2026-07-25；Lane-KRN `lane/krn-personal-p1-t01-xdg-migrations`，PR [#92](https://github.com/agentkernel/cognitive-os/pull/92)。`PersonalDataLayout` + `prepare_personal_databases`：XDG 五根、Unix 0700/0600、`migration.lock`、双库 v1 plan（与 open 共享 schema 常量）、state/backups 非覆盖备份。CI run [30155053950](https://github.com/agentkernel/cognitive-os/actions/runs/30155053950) Ubuntu/Windows-MSVC 全绿（含 `p1_t01_layout_migrations` 7 pass、clippy、fmt）。本机 Windows GNU linker exit 121 为非支持基线。未改 registry/schema/vector/transition；非 G0/B01-B12/Profile。handoff：[20260725-lane-krn-personal-p1-t01-handoff.md](../checkpoints/20260725-lane-krn-personal-p1-t01-handoff.md)。 |
| P1-T02 | SecretStore 正式后端与 Provider 配置 | P0-T05, P1-T01 | rotation/restart/redaction negatives 通过 | done | 2026-07-25；分支 `lane/personal-p1-t02-secret-provider-config`，PR [#93](https://github.com/agentkernel/cognitive-os/pull/93)。`cognitive-secret`：ProviderConfig/ProviderKeyService、LinuxSecretToolStore、production selection、hidden-input helper；ADR-0020。CI run [30156079691](https://github.com/agentkernel/cognitive-os/actions/runs/30156079691) Ubuntu/Windows-MSVC 全绿（含 `p1_t02_provider_secret`）。本机 Windows GNU linker exit 121 为非支持基线。非 G0/B01-B12/Profile。handoff：[20260725-personal-p1-t02-secret-provider-config-handoff.md](../checkpoints/20260725-personal-p1-t02-secret-provider-config-handoff.md)。 |
| P1-T03 | Provider、模型发现与能力快照 | P1-T02 | probe 正负例与 model snapshot 通过 | done | 2026-07-25；分支 `lane/personal-p1-t03-provider-discovery-probe`，PR [#94](https://github.com/agentkernel/cognitive-os/pull/94) 合入 `main@118d20a`。`cognitive-secret`：`ProviderTransport`、`ProviderDiscoveryService`、capability snapshot + `fnv1a64` identity digest、`persist_selected_snapshot_digest`；ADR-0021。CI runs [30157577277](https://github.com/agentkernel/cognitive-os/actions/runs/30157577277) / [30157576277](https://github.com/agentkernel/cognitive-os/actions/runs/30157576277) Ubuntu/Windows-MSVC `cargo test --workspace --locked` SUCCESS（含 `p1_t03_provider_discovery`）。本机 Windows GNU linker exit 121 为非支持基线。非 G0/B01-B12/Profile；无真实 Provider Key；无 registry/schema/vector 变更。handoff：[20260725-personal-p1-t03-provider-discovery-handoff.md](../checkpoints/20260725-personal-p1-t03-provider-discovery-handoff.md)。 |
| P1-T04 | 有界 Personal daemon 与本地认证 | P0-T07, P1-T01 | auth/size/timeout/concurrency/restart 测试 | done | 2026-07-25；PR #95（auth/size/host/cookie/restart）+ PR [#96](https://github.com/agentkernel/cognitive-os/pull/96)（timeout/concurrency）。`kernel-server --personal`：loopback、daemon lock、bootstrap secret、channel bearer、header/body 读超时 408、connection/in-flight 429、Host/Cookie fail-closed；ADR-0022。CI runs [30162481713](https://github.com/agentkernel/cognitive-os/actions/runs/30162481713) / [30162477963](https://github.com/agentkernel/cognitive-os/actions/runs/30162477963) Ubuntu/Windows-MSVC SUCCESS（含 timeout/concurrency 单元测试与既有 `p1_t04_personal_daemon`）。本机 Windows GNU linker exit 121 为非支持基线。非 G0/B01-B12/Profile；无 Task/Memory/MCP；无 registry/schema/vector 变更。handoff：[20260725-personal-p1-t04-timeout-concurrency-handoff.md](../checkpoints/20260725-personal-p1-t04-timeout-concurrency-handoff.md)。 |
| P1-T05 | Readiness、status 与 doctor 应用服务 | P1-T03, P1-T04 | blocked/degraded/ready 事实区分 | done | 2026-07-25；分支 `lane/personal-p1-t05-readiness-doctor`，PR [#97](https://github.com/agentkernel/cognitive-os/pull/97)。`kernel-server` Personal composition root：`evaluate_personal_readiness` + management-auth `GET /personal/status|readiness|doctor`；ADR-0023；blocked/degraded/ready 分离；`static_check_is_not_runtime_ready`；secret_ref/bootstrap 不入投影。CI runs [30164114878](https://github.com/agentkernel/cognitive-os/actions/runs/30164114878) / [30164113787](https://github.com/agentkernel/cognitive-os/actions/runs/30164113787) Ubuntu/Windows-MSVC SUCCESS。本机 Windows GNU linker exit 121 为非支持基线。非 G0/B01-B12/Profile。handoff：[20260725-personal-p1-t05-readiness-doctor-handoff.md](../checkpoints/20260725-personal-p1-t05-readiness-doctor-handoff.md)。 |
| P1-T06 | `cognitive init/doctor/status/daemon` | P1-T02, P1-T05 | 重复 init、hidden input、可操作错误 | done | 2026-07-25；分支 `lane/personal-p1-t06-cognitive-cli`，PR [#98](https://github.com/agentkernel/cognitive-os/pull/98) 合入 `main@adbb0e5`。`cognitive` bin + `personal_cli`（init/status/doctor/daemon）、ADR-0024、`tests/p1_t06_cognitive_cli.rs`（live daemon 路径以 Ubuntu 为权威；Windows 跑 init/usage）。CI run [30167503487](https://github.com/agentkernel/cognitive-os/actions/runs/30167503487) Ubuntu/Windows-MSVC SUCCESS。本机 Windows GNU linker exit 121 为非支持基线。非 G0/B01-B12/Profile。handoff：[20260725-personal-p1-t06-cognitive-cli-handoff.md](../checkpoints/20260725-personal-p1-t06-cognitive-cli-handoff.md)。 |
| P1-T07 | CognitiveOS Pi Package/Extension 与 proxy | P0-T06, P1-T03, P1-T04, P1-T05 | 禁用直接 mutating tool；无 key 泄漏 | done | 2026-07-27；PR [#105](https://github.com/agentkernel/cognitive-os/pull/105) merged as `main@9d4c3d9` after the Ubuntu and Windows/MSVC CI checks succeeded. `packages/pi-cognitiveos/` defaults to denying `project_trust` and every Pi tool, displays only daemon facts, and source-scan tests forbid Provider key/config, `SecretRef`, SQLite, subprocess, and filesystem-write access. The daemon exposes a management-authenticated, non-secret selected-model projection and a bounded Pi complete-provider bridge: exactly one daemon-projected model forwards a one-shot `stream:false` completion through the authenticated daemon proxy. Provider material remains daemon-only; the proxy creates no Intent/Effect, capability, or state transition. `RustlsProviderTransport` remains HTTPS-only, redirect-free, time- and 1 MiB-response-bounded, and rejects URL user-info/header injection; `stream:true` remains fail-closed. Local WSL tests and the supported CI matrix passed. This is implementation and test evidence only, not a G0/B01-B12, Profile, containment, or release claim. Handoff: [20260727-personal-p1-t07-closeout-handoff.md](../checkpoints/20260727-personal-p1-t07-closeout-handoff.md). |
| P1-T08 | 可检查 Linux bundle installer 与 user service | P0-T03, P1-T01, P1-T04, P1-T06, P1-T07 | verifier、interruption、rollback 测试 | done | 2026-07-29；`lane/personal-p1-t08-mvp-single-service`。已交付固定单服务安装事务、release-shaped campaign builder 与 Linux-native user-systemd 验证：clean install `.3`、healthy upgrade `.4`、pre-pointer `.5` failure 与 post-pointer `.6` failure；两种 failure 后均恢复 canonical unit/service、48181 liveness 及 non-secret `active-version=.4`，并保留 immutable campaign versions。聚焦 WSL tests：campaign builder、service lifecycle、single-service 及 adapter **20/20 passed**；strict runtime Clippy、formatting、consistency 和 whitespace 均通过。该结论仅完成 P1-T08 installer 验收；campaign 仍为 `experimental-local-only` / `tested-local` evidence，不构成 production release/signing、B01、Gate、Profile、containment、uninstall 或 first-conversation claim。 |
| P1-T09 | 首次安装到首次对话 route 与 B01 campaign | P1-T08 及现有 Secret/Provider/daemon/Pi contracts | deterministic binary Provider fixture、真实 pinned Pi Extension load、真实首个响应、native Secret Service smoke 与预注册 B01 runner；B01 仍由独立 formal campaign 决定 | in-progress | `experimental-local-only`；deterministic binary Provider fixture 已在 supported Ubuntu/Windows CI **3/3** 通过，故该 slice 的 `implementation_evidence: tested-supported-ci`。`personal-linux-native-01`（`wuz@192.168.1.2`）已 SSH-qualified 用于 disposable debugging，但 exact Pi `0.81.1` availability 仍 `not-run`：`pi` 不在 PATH，uncredentialed exact-package probe 两分钟无 version output 后被停止。下一步先版本验证远端 Pi artifact/binary，再进行真实 `--extension <absolute-path>` load。不得将 fixture、SSH host、WSL 或普通 CI 提升为 B01、GMVP-LINUX、release 或 Profile；无 normative surface 变更。下一 handoff：[20260730-personal-p1-t09-provider-fixture-handoff.md](../checkpoints/20260730-personal-p1-t09-provider-fixture-handoff.md)。 |

#### Historical P1-T08 slice journal

以下 blockquote 保留各原子批当时的状态与 non-claim，属于历史记录；不得覆盖上表当前状态。

> **2026-07-28 P1-T08 bootstrap/download slice:** `lane/personal-p1-t08-linux-bootstrap` adds an inspectable POSIX `deploy/linux/install.sh` template and a narrow `linux-bundle-verifier` adapter. The unrendered source template rejects before network access; release rendering must bind the version, HTTPS object directory, allowed redirect host, bootstrap verifier digest, public keyring, and Pi pin. Downloads use a private temporary directory, partial paths, bounded HTTPS curl calls, and invocation-owned cleanup. The adapter verifies its script-bound SHA-256 before execution and delegates only to the existing offline `verify_linux_bundle`; it performs no activation because bounded service health is not yet defined. Focused WSL behavior tests are local `windows_wsl2_linux_guest` evidence only. P1-T08 remains `in-progress` with `development_track: experimental-local-only`; P1-T09 remains `not-started`. No production key/release, service/health/rollback, uninstall, Linux-native campaign, B01, Gate, Profile, containment, or release claim is added. |

> **2026-07-28 P1-T08 service-health slice:** `lane/personal-p1-t08-service-health` adds a separate lease-held service-lifecycle transaction, a fail-closed source user-unit template, a fixed-argument `systemctl --user` controller boundary, strict bounded loopback `/personal/health` parsing, and deterministic compensation tests. The existing offline installer callback remains unchanged. The checked-in unit and controller reject unresolved templates and the still-absent safe extracted `bin/kernel-server` layout before any systemd action; no runnable archive or real systemd service is claimed. Focused service tests passed **6/6** in `windows_wsl2_linux_guest`; this is local fixture evidence only. P1-T08 remains `in-progress` with `development_track: experimental-local-only`; P1-T09 remains `not-started`. |

> **2026-07-28 P1-T08 rendered-user-service foundation:** ADR-0032 fixes
> product-owned candidate and canonical unit identities, disjoint loopback
> ports, staged-versus-active executable paths, and candidate-stop before
> pointer activation/canonical restart. The runtime renders fixed unit content
> only from a constrained version and product deployment root, and fixture unit
> publication uses a private temporary file followed by atomic rename. Focused
> service lifecycle tests **9/9 passed** in `windows_wsl2_linux_guest`.
> PR [#114](https://github.com/agentkernel/cognitive-os/pull/114) merged as
> `main@b151b54`; after a Windows path-separator correction (`0a90033`), both
> supported Ubuntu/Windows-MSVC push and pull-request matrices passed in runs
> [30379506413](https://github.com/agentkernel/cognitive-os/actions/runs/30379506413)
> and
> [30379508772](https://github.com/agentkernel/cognitive-os/actions/runs/30379508772).
> This remains implementation-fixture and supported-matrix evidence only; a
> production user-systemd installation path, daemon-reload fixture,
> Linux-native campaign, B01, Gate, Profile, containment, and release evidence
> remain absent. P1-T08 remains `in-progress`; P1-T09 remains `not-started`. |

> **2026-07-28 P1-T08 fake-systemctl controller fixture:** ADR-0033 fixes the
> private/injected unit-root controller boundary and the fixed daemon-reload,
> candidate start/stop, and canonical active restart actions. The focused Unix
> fake harness verifies candidate unit publication then daemon-reload then
> fixed candidate start, with candidate work never publishing the canonical
> active unit. Focused lifecycle tests **10/10 passed** in
> `windows_wsl2_linux_guest`. PR
> [#115](https://github.com/agentkernel/cognitive-os/pull/115) merged as
> `main@aa09f6c` after supported Ubuntu/Windows-MSVC push and pull-request CI
> passed in runs
> [30382894322](https://github.com/agentkernel/cognitive-os/actions/runs/30382894322)
> and
> [30382932475](https://github.com/agentkernel/cognitive-os/actions/runs/30382932475).
> This is implementation-fixture evidence only;
> pointer/unit/service compensation fault injection, Linux-native systemd,
> B01, Gate, Profile, containment, and release evidence remain absent.

> **2026-07-28 P1-T08 safe-extraction slice:** ADR-0031 specifies the
> implementation-local `tar.gz` extraction policy: a verified artifact is
> re-hashed and extracted only in the lease-held private staging area; the
> direct fixed layout is `bin/kernel-server`; traversal, links, special files,
> unsafe modes, duplicate/conflicting paths, unsupported layouts, and bounded
> resource violations are fail-closed. `cognitive-runtime` now provides the
> bounded in-process extraction and only publishes `staged/<version>` after the
> fixed layout validates; an extraction failure leaves the active pointer and
> canonical active service untouched and returns no receipt. Successful fixture
> candidates satisfy the controller's static layout preflight, but the
> unresolved user-unit still prevents every systemd action. Focused installation
> tests executed **12/12**, lifecycle tests **12/12** plus one ignored child
> entrypoint, and service-lifecycle tests **6/6** in
> `windows_wsl2_linux_guest`; strict feature Clippy, formatting, and consistency
> checks also executed successfully. This is implementation and local-test
> evidence only. P1-T08 remains `in-progress` with
> `development_track: experimental-local-only`; it is not Linux-native systemd,
> B01, Gate, Profile, containment, or release evidence. |

> **2026-07-28 P1-T08 verifier merge update:** PR
> [#108](https://github.com/agentkernel/cognitive-os/pull/108) merged as
> `main@afa1d5d`; both push and pull-request Ubuntu/Windows-MSVC CI matrices
> passed. This closes only the offline verifier slice. P1-T08 remains
> `in-progress`, and every installer, service, B01, Gate, Profile, containment,
> Linux-native, and release non-claim above remains unchanged.

> **2026-07-27 verification correction for P1-T07:** Cargo is available in the
> WSL guest at `/root/.cargo/bin/cargo`. The focused provider-proxy process test
> and `kernel-server` strict Clippy check passed there. This is focused local
> Linux evidence only. The supported CI matrix subsequently passed for PR #105,
> including Windows/MSVC. Windows GNU linker exit 121 remains a non-supported
> local limitation.

> **2026-07-29 P1-T08 production-path amendment:** ADR-0034 supersedes the
> dual-service controller only as the first production path. P1-T08 now closes
> against one canonical `cognitiveos-personal.service`, port 48181, a shared
> verify/lease/stage transaction, deterministic single-service rollback and a
> Linux-native user-systemd campaign. Existing candidate/active fixture results
> remain valid implementation-fixture evidence, but candidate port 48182 and
> zero-downtime promotion no longer block P1-T08 or B01. P1-T08 remains
> `in-progress`; no implementation or Gate status changes in this planning batch.

> **2026-07-29 P1-T08 single-service implementation slice:** the shell-to-Rust
> handoff now invokes a digest-bound `linux-bundle-installer`; the runtime
> shares the existing offline verification, OS lifecycle lease and private
> staging prefix before separately publishing immutable version bytes and the
> active pointer. A narrow canonical-service controller owns only
> `cognitiveos-personal.service`, fixed 48181 health, fixed user-systemd
> actions and Rust-rendered unit content. Upgrade and first-install failures
> compensate pointer/unit/service state deterministically and issue no receipt;
> incomplete compensation returns `RollbackIncomplete`. Focused WSL2 Linux
> guest tests passed **46/46** with one ignored child-process entrypoint;
> runtime strict Clippy, formatting, consistency and diff checks passed. This
> is implementation/fixture evidence only. P1-T08 remains `in-progress`;
> Linux-native user-systemd, production release/signing, uninstall, B01,
> product Gate, Profile and first-conversation evidence remain outstanding.

| P1-T09 | 安装到首次对话 route 与 B01 campaign | P1-T08 | route implementation、deterministic fixture、dev smoke、usability 与 formal B01 分阶段；B01 预注册至少 20 次独立 clean Linux VM attempt，全部 attempt 计入，成功率 ≥90%，关键安全失败为 0；除 API Key 与模型选择外无必选交互（ADR-0026/0034） | in-progress | 2026-07-30；`development_track: experimental-local-only`，`implementation_evidence: tested-supported-ci`，`B01 gate_status: not-run`。已提供 shared Rustls Provider discovery、digest-matched selected model readiness、真实 XDG/daemon endpoint、非秘密 `cognitive pi configure` 及 fail-closed `cognitive pi launch`：launch 只接受 daemon-owned numeric loopback endpoint、authenticated ready doctor projection、exact Pi `0.81.1` 与 `--extension <absolute-path>`，子进程环境清空后只恢复 OS allowlist，绝不传递 Provider/secret material。deterministic binary Provider fixture 的真实 Rustls discovery focused suite 在 PR #117 的 Ubuntu 与 Windows supported CI workspace job 均 **3/3 passed**（Ubuntu run 30513254161；Windows run 30513254161）；它保持 loopback-only HTTPS、secret redaction 和无 Task/Effect/Verification/capability/authority side effect。WSL focused admin Personal unit 15/15、Pi/readiness 1/1、Personal readiness 1/1、Provider-proxy 2/2 及 cognitive CLI 5/5 已执行。尚缺当前 route 的真实 Pi Extension load、真实首次对话、native Secret Service smoke、usability 和 formal B01；因此任务如实为 `in-progress`，B01 仍 `not-run`，无 Gate/release/Profile 声明。 |

### Phase 2 - 单 Agent 任务闭环

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P2-T01 | TaskApplicationService | P1-T09 | raw intent、preview digest、epoch fencing；admission preview 为唯一默认人工授权点（ADR-0026） | not-started | — |
| P2-T02 | 真实 Task API、watch 与自然语言管理映射 | P2-T01, P1-T07 | detach/watch/cancel 的 authority 语义正确；trust profile Tier 0/1/2 在 daemon/CLI/Pi 一致应用（ADR-0026） | not-started | — |
| P2-T03 | durable scheduler、lease 与 timer | P2-T01, P1-T01 | crash/duplicate lease/clock/budget 测试 | not-started | — |
| P2-T04 | 单 Agent worker 与 BoundedHarness 接入 | P2-T02, P2-T03 | no-progress/budget/stale-lease 测试 | not-started | — |
| P2-T05 | Tool Registry 与第一个安全 operation | P2-T04 | 未注册、drift、disabled 均 dispatch=0 | not-started | — |
| P2-T06 | Process supervisor 与首个 executor | P2-T05 | dispatch 故障、orphan、redaction、idempotency | not-started | — |
| P2-T07 | Checkpoint、Evidence 与独立 Verifier | P2-T03, P2-T04, P2-T06 | criteria evidence；partial 不得 complete | not-started | — |
| P2-T08 | Phase 2 E2E Gate | P2-T07 | B02/B04/B05/B12 与 false-completion negative；核查 ADR-0018 本机开发例外已到期移除（或已替换为 daemon proxy 并重新批准）；B04 记录默认路径人工确认次数 ≤1/task 并含 Tier-2 负例（ADR-0026） | not-started | — |

### Phase 3 - Context、Token 与 Loop 效率

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P3-T01 | Context source/retrieval port | P2-T08 | scope-filter 先于 ranking；revocation 测试 | not-started | — |
| P3-T02 | 最小充分 Context Builder 与预算 | P3-T01 | required fail-closed；loss 显式 | not-started | — |
| P3-T03 | Artifact Store 与 Context externalization | P3-T02 | CAS/partial-write/GC/access 测试 | not-started | — |
| P3-T04 | Loop telemetry、progress 与 strategy controls | P3-T02, P3-T03 | no-progress/repeat/strategy 控制可观测 | not-started | — |
| P3-T05 | Benchmark harness 与性能基线 | P3-T04 | raw run、CI、non-claim 与稳定基线 | not-started | — |
| P3-T06 | Phase 3 E2E Gate | P3-T05 | B03/B06/B07 通过 | not-started | — |

### Phase 4 - Memory

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P4-T01 | Memory schema、repository 与 policy | P3-T06 | scope/provenance/freshness/forgetting 语义 | not-started | — |
| P4-T02 | FTS retrieval 与 authority-first ranking | P4-T01 | unauthorized/stale/conflict 负例通过 | not-started | — |
| P4-T03 | Memory lifecycle、retention 与 privacy | P4-T01 | update/expire/forget/audit 可验证 | not-started | — |
| P4-T04 | Embedding 实验 Gate | P4-T02, P4-T03 | 与 FTS baseline 对比后 go/no-go | not-started | — |
| P4-T05 | Memory API、CLI/Pi projection 与 B08 | P4-T04 | B08、隐私和删除语义通过 | not-started | — |
| P4-T06 | Phase 4 E2E Gate | P4-T05 | memory 回归与 benchmark 证据完整 | not-started | — |

### Phase 5 - Agent 与 Tool 生态

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P5-T01 | Agent package manifest 与安装生命周期 | P4-T05 | digest/health/activate/stop/uninstall；首用一键授予并记住为 capability lease（ADR-0026） | not-started | — |
| P5-T02 | Agent registry 与 instance lifecycle | P5-T01 | capability 默认拒绝、实例隔离；install ≠ permission 保留（ADR-0026） | not-started | — |
| P5-T03 | MCP Tool adapter qualification | P2-T05, P4-T05 | MCP 不成为 authority；drift/timeout 测试 | not-started | — |
| P5-T04 | Tool lifecycle、sandbox 与审计 | P5-T03 | enable/disable/quarantine/reconcile | not-started | — |
| P5-T05 | B09/B10 生态 Gate | P5-T02, P5-T04 | agent/tool 安装与负例 E2E 通过 | not-started | — |

### Phase 6 - Multi-Agent

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P6-T01 | Multi-Agent policy 与 scheduler 扩展 | P5-T05 | 默认关闭、budget/lease/isolation 正确 | not-started | — |
| P6-T02 | Mailbox 与 append-only findings | P6-T01 | 消息不可作为 authority；replay 可处理 | not-started | — |
| P6-T03 | Reviewer/Verifier/Integrator 编排 | P6-T02 | child success 不可绕过 verifier | not-started | — |
| P6-T04 | B11 收益 Gate | P6-T03 | 仅在质量或速度收益达标时启用；NO-GO 并保持默认关闭是合法结果，不阻塞 GMVP-LINUX/RC | not-started | — |

### Phase 7 - 产品化与发布

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P7-T01 | Release pipeline、SBOM 与 attestation | P0-T03, P1-T08, P2-T08 | 可验证 Linux artifact；无 secret/dev path；先交付 Linux MVP slice，后续能力补充 inventory | not-started | — |
| P7-T02 | Transactional update、rollback 与 uninstall | P1-T01, P1-T08, P2-T08, P7-T01 | stage/health/rollback/uninstall 语义通过；面向用户的 `cognitive backup`/`restore` 命令（排除 secret，ADR-0026） | not-started | — |
| P7-T03 | Doctor、support bundle 与故障排查 | P1-T05, P2-T08, P7-T02 | 仅 redacted facts；stable error code；GMVP-LINUX 前可操作恢复路径 | not-started | — |
| P7-T04 | 完整性能 campaign 与回归地板 | P3-T06, P4-T05, P5-T05 | 固定环境、raw evidence、CI 与阈值 | not-started | — |
| P7-T05 | 非阻塞 Web UI | P2-T08, P7-T03 | 通过 clients gate；只读 daemon projection | not-started | — |
| P7-T06 | RC、文档、支持矩阵与声明范围内 B01-B12 | P7-T08, P7-T04, P5-T05 | clean VM suite 与 release claim evidence；P6 可为明确 NO-GO/disabled，不阻塞 RC | not-started | — |
| P7-T07 | Windows 安装面：credential 后端、installer/service 与 B01-W Gate | P1-T02, P7-T01, P7-T02 | Windows credential store 后端（同 fail-closed 边界，无明文 fallback）、可检查 installer/service、专门 B01-W Gate 编写并执行；不阻塞 Linux RC；未执行前不得声称 Windows install parity（ADR-0025） | not-started | — |
| P7-T08 | Public Linux MVP Gate（`GMVP-LINUX`） | P1-T09, P2-T08, P7-T01, P7-T02, P7-T03 | production trust/signing、native user-systemd、B01、受治理单 Agent、update/rollback/uninstall、doctor/support 证据汇合；功能声明明确排除未执行能力；不构成 Profile | not-started | — |

## 5. Gate 与证据要求

| Gate | 必需结果 | 最低证据 |
|---|---|---|
| G0 | 基线、决策和 PoC 已完成或明确 NO-GO | ADR、PoC report、支持矩阵、handoff |
| B01 | 安装、初始化、Provider、daemon、Pi 首次响应 | 至少 20 次 clean Linux VM redacted run |
| B02/B04/B05/B12 | 管理、任务、恢复、unknown outcome 闭环 | E2E logs、effect/verification evidence、负例、默认路径人工确认次数记录（ADR-0026） |
| B03/B06/B07 | Context 正确性与效率控制 | benchmark raw run、预算/loss/telemetry evidence |
| B08 | Memory 生命周期与隐私 | provenance/freshness/conflict/forget 测试 |
| B09/B10 | Agent/Tool/MCP 资格与隔离 | manifest、health、disable/uninstall、negative tests |
| B11 | Multi-Agent 相对单 Agent 有可复现收益 | 相同模型/预算/任务的 baseline 对比 |
| GMVP-LINUX | scoped public Linux MVP 可发布 | B01 + P2 Gate + production trust + native systemd + update/rollback/uninstall + doctor/support；明确 non-Profile 与能力排除项 |
| RC | 完整发布声明 | CI、SBOM、attestation、升级/卸载、支持矩阵 |

Windows install parity 声明需要 P7-T07 的专门 B01-W Gate 与已执行证据；在此之前，
RC 与支持矩阵中的安装声明仅覆盖 Linux bundle（Windows 仅为 daemon/CLI 产品路径，
按 ADR-0025 表述）。

## 6. 实施时的文档联动清单

- [ ] 更新本文件的任务状态、日期和证据/阻塞说明。
- [ ] 更新 `docs/plan/PROGRESS.md` 的全局状态（不将 planning 任务误报为已实现）。
- [ ] 如有合同、schema、vector 或 generated binding 改动，遵循 Lane-CTR 与 docs-sync 流程。
- [ ] 如有开放风险或漂移，更新 `docs/traceability/findings-ledger.md`。
- [ ] 写入 `docs/checkpoints/` handoff，记录完成项、未完成项、测试、证据、风险与下一步。
- [ ] 在 PR/提交中关联 `PERS-*` 计划 ID，并在适用时关联真实 REQ-ID。
- [ ] 如发生计划结构变化（新增/取消任务、验收或依赖变化），在同一修订批中同步 `plan.md` 任务卡与依赖图、`personal-trace.yaml` 映射，保持无孤儿任务。
