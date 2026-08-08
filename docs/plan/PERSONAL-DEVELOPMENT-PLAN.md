# CognitiveOS Personal 产品化开发计划与进度表

> **项目身份：** `cognitiveos-personal` 是本仓库当前唯一活动实现项目。原 CognitiveOS
> 设计、规范、符合性资产和通用内核是本项目的架构/合同基础，不是并行产品 backlog。
> 边界与来源优先级见 [PROJECT-IDENTITY.md](../governance/PROJECT-IDENTITY.md)。
> **状态：in-progress（P0-T01..T07、P1-T01..T08、P2-T01..T06、P2-T07、P3-T01 已完成；P1-T09 in-progress；B01 running：固定 N=20 已记账 2 次，1 成功/1 失败；P2/B03/B09/GMVP-LINUX 正式验收尚未完成）**
> **最后更新：2026-08-08**
> **计划追踪 ID：** `P0-T01` 至 `P7-T08` 是本计划的管理 ID，不是 `specs/registry/` 中的 REQ-ID，也不构成实现、测试或 Profile 符合性声明。
> **详细研究与任务卡草案：** 仓库根目录 `plan.md`；本文件是后续开发的**正式任务、typed dependency、验收与 Gate 定义源**。当前 task/Gate/claim 事实只由 [PROGRESS.md](PROGRESS.md) `Current snapshot` 拥有；`plan.md` 只补充经本文件对齐的研究依据、实施细节与验收方法。
> **可机读追踪：** [personal-trace.yaml](personal-trace.yaml) 将 `PERS-PR`、本计划任务与 Gate/benchmark 对齐；它不是 registry matrix，且不构成 REQ、测试执行或 Profile 符合性声明。

> **开发状态解耦（2026-07-30 修订）：** `not-started` 表示尚无任务专属实现或测试
> slice；纯研究、讨论和未提交计划草稿不改变状态，首个真实实现或测试 slice 开始后
> 必须改为 `in-progress`。`done` 才表示完整正式验收已满足。后续 P1/P2/P3 工作可在
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
> 作为 P1/B01、P2 与 P7-T01..T03 的公开 Linux MVP 汇合 Gate。该日将 Context、Memory
> 放在后续能力列车的范围，已由 2026-08-02 ADR-0037/0038 六资源 1.0 重基线取代；
> MCP/dynamic Tool、Windows 安装面、Web UI 和 Multi-Agent 仍为后续独立能力列车；
> Multi-Agent NO-GO 且默认关闭是合法结果。本修订不改变任何既有任务状态、
> 已执行证据、规范机器资产或 Profile 结论。

> **Linux 1.0 与 Agent 管理修订（2026-08-02，ADR-0035/0036）：**
> `GMVP-LINUX` 是 Personal `1.0.0` 的既有发布 Gate，不新增平行 Gate。Pi-hosted
> Agent Shell 与 managed Pi Agent 是独立角色：P2-T02 负责 Shell 到 Task/management
> application service，P5-T01/T02 与 B09 负责官方 npm Pi acquisition、installation、
> registry/instance/supervision/lifecycle。Linux 1.0 只 product-qualify Pi，同时交付可供
> 后续 Agent 独立 qualification 的通用 adapter framework；该日排除 Memory 的 1.0
> scope 已由 ADR-0037 取代。OpenClaw、Hermes、Codex、WorkBuddy、MCP、Multi-Agent、
> Web UI 与 Windows installer 仍不进入 1.0 claim。
> 该 product-semantic 修订不修改 registry/schema/transition/vector，也不产生实现、Gate、
> release 或 Profile 证据。

> **统一认知资源基座与 Agent sidecar 修订（2026-08-02，ADR-0037/0038）：**
> Personal Linux 1.0 重新定位为统一认知资源基座，交付 Memory、Skill、Tool、Context、
> Task、Runtime 六类资源的最小真实 slice；三条 1.0 release track 为 Runtime Spine、
> Resource Value 与 Product Operability。Agent 集成采用 per-Agent sidecar-first 边界，
> Pi 仍是唯一 qualified Agent。复杂 Context 优化和收益 Gate 不阻塞 1.0，但 Context
> correctness 属于 1.0；embedding/vector/graph、MCP/dynamic Tool ecosystem、Multi-Agent、
> Web UI 与 Windows 安装面移入 post-1.0。该 owner-approved `product-semantic + structural`
> 修订不新增、删除或重编号任务，也不改变任何已有 task status、attempt、evidence 或 Gate
> current status：P1-T09 仍为 `in-progress`，B01 仍为 `running`（1/至少 20），P2-T01/
> P2-T03 仍为 `in-progress`，`GMVP-LINUX` 仍为 `not-run`，Profile `implemented: 0`。
> 本批不修改规范或实现，不产生 implementation、Gate、release 或 Profile evidence。

> **计划修订（2026-07-26，生产就绪与低摩擦授权批）：** 依 owner 指令与
> [ADR-0026](../adr/0026-personal-trust-profile-low-friction-authorization.md)
> 落地 Personal 低摩擦授权模型（DEC-P-20）：交互分层 Tier 0/1/2、任务准入预览为
> 唯一默认人工授权点、预算硬轨替代逐动作审批、不建审批链；同时补 P7-T02 面向用户
> 的 backup/restore（排除 secret）。仅修改 not-started 任务的验收摘要
> （P1-T09、P2-T01、P2-T02、P2-T08、P5-T01、P5-T02、P7-T02），治理层
> （Intent/Effect、audit、verifier、capability 类型）全保留。本批不改变任何既有
> 任务状态、Gate、证据或 Profile 结论。

## 1. 使用与更新规则

1. 纯研究、讨论和未提交的计划草稿不改变任务状态。首个任务专属实现或测试 slice
   （包括 failure-first 测试）开始时，在本表把对应任务标为 `in-progress`，并在同一
   atomic delivery 内补齐负责人/分支、开始日期和关联 PR（如有）；若未达到产品 Gate，
   额外填写 `development_track: experimental-local-only`，不得把 acceptance/promotion
   Gate 写成实现阻断。
2. 一个任务只有在其验收条件满足、相关测试真实执行并留有证据后，才可标为 `done`；未执行的测试必须明确标 `not-run`，不得推断为通过。
3. **`TASK-ATOMIC-DELIVERY-01`：** 默认 atomic delivery 是一个完整 `P*-T*` 正式任务。
   一个任务使用一个 task branch、一个持续更新的 Draft PR 和一个 task-scoped lease，并连续
   完成全部必要 Slice、真实集成、负例、supported validation、完整 acceptance assessment
   与文档/分支收口。Slice 完成、checkpoint、push、CI 发起或阶段结果均不是停点或汇报点。
   只有任务 `done`、不可自主消除的外部阻塞、未知并发改动、安全冲突或 owner 明确暂停/改
   范围时才可中断。
4. 发生范围、依赖、验收或安全边界变化时，先将任务标为 `blocked`，记录原因和决策，再更新详细任务卡与依赖图；不得静默改写完成标准。
5. 允许的任务状态为：`not-started`、`in-progress`、`blocked`、`done`、`cancelled`；实现证据为 `none` / `provided` / `tested-local` / `tested-supported-ci`；Gate 为 `not-run` / `running` / `pass` / `fail` / `blocked`。`done` 不等于 Gate pass 或 Profile `implemented`。
6. 如本表与 `plan.md` 的任务卡或依赖图不一致，应先按本表执行并在同一文档修正批中对齐 `plan.md`；不得仅凭详细卡片重新解释或复用既有 `P*-T*` ID。
7. 领取任务前必须一次性核对其完整验收、implementation dependencies、所需路径和验证
   环境，然后连续实现到任务收口。任务、依赖和安全路径已经明确时，不得用继续研究、
   新建平行计划、切换无关任务或阶段性汇报替代实现。本地可修复的代码、测试、格式、CI
   配置或集成问题必须直接修复；只有外部阻塞才能形成带 `blocked_paths` /
   `blocked_task_ids` / `blocked_gate_ids` / owner / next action 的中断记录。
8. 任务内执行步骤登记为稳定 Delivery Slice，ID 为 `<task-id>/DNN`。本文件拥有 slice 的
   目标、依赖、出口和 required validation；当前状态只由 `PROGRESS.md` Current snapshot
   拥有。Slice 是内部检查点，不是独立 branch、PR、lease、handoff 或默认用户汇报单位。
   Slice `done` 后立即进入下一未满足 acceptance 项；最终 Slice 必须汇总完整 acceptance，
   不得另开 `acceptance-assessment` 分支。Slice `done` 不等于 task `done`，task `done` 也不
   等于 Gate pass。
9. 同一正式任务最多一个 `in-progress` slice。一个 foundation/helper slice 后必须优先
   接入真实 caller、durable authority outcome 或端到端负例；不得连续新增 helper-only
   slice 回避集成。实现存在但 required supported validation 未运行时，slice 必须为
   `blocked`，不能凭 fmt、diff 或 consistency 关闭。
10. **MVP-first：** 首个真实路径优先使用 owner-local、single-principal、task-scoped、
    daemon-issued 的最小授权组合，对范围外请求 fail closed。完整 RBAC、审批链、通用
    capability administration、多租户策略语言和未来扩展框架，除非当前任务验收或已登记
    threat boundary 明确要求，否则不得成为 MVP implementation mutex。此规则不放松
    daemon-only authority、SecretStore、Intent/Effect、budget/fencing、audit 或 independent
    verifier 不变量。
11. 完整任务收口是实现的最后一步，不是后续管理任务：逐条映射 acceptance 与 exact
    evidence，完成 supported validation/required CI，同步正式计划、Current snapshot 与
    唯一最终 handoff；确认 task PR 只含已声明路径后 ready/merge；随后关闭 lease、删除安全
    可删的远端 task branch、本地切回并 fast-forward `main`，确认 worktree clean、HEAD/
    upstream 一致且无已完成任务的 active lease。每次正式任务完成后，提交、推送、合并、关
    闭 lease、删除可安全删除的本地与远端分支、切回主分支和本地收尾都应作为同一条收口链
    路一次完成，不得拆成后续“顺便处理”的独立步骤。任一步缺失时任务保持 `in-progress`
    或 `blocked`，不得留下“代码完成但验收、分支或状态待收口”的半完成状态。

### Typed dependency 规则

- `implementation_requires`：开始独立 implementation slice 前必须已有的代码/合同；
- `acceptance_requires`：任务标为 `done` 前必须满足的任务或证据；
- `promotion_requires`：进入产品 Gate、release 或扩大 claim 前必须通过的 Gate。

下方旧表“依赖”列是便于阅读的摘要；遇到顺序歧义，以 release-critical typed dependency
表为准。Acceptance/promotion dependency 不是 implementation mutex。

### 进度汇总

| 阶段 | 任务数 | done | in-progress | blocked | not-started | 阶段 Gate |
|---|---:|---:|---:|---:|---:|---|
| Phase 0 - 基线与决策 | 7 | 7 | 0 | 0 | 0 | G0 |
| Phase 1 - 安装到首次对话 | 9 | 8 | 1 | 0 | 0 | G1 / B01 `running` |
| Phase 2 - 单 Agent 任务闭环 | 8 | 7 | 0 | 0 | 1 | G2 / B02、B04、B05、B12 |
| Phase 3 - Context Resource Value | 6 | 1 | 0 | 0 | 5 | G3 / B03、B06、B07 |
| Phase 4 - Memory 与 Skill | 6 | 0 | 0 | 0 | 6 | G4 / B08 |
| Phase 5 - Agent sidecar 与 Tool 生态 | 5 | 0 | 0 | 0 | 5 | G5 / B09、B10 |
| Phase 6 - post-1.0 Multi-Agent | 4 | 0 | 0 | 0 | 4 | G6 / B11 |
| Phase 7 - 产品化与发布 | 8 | 0 | 0 | 0 | 8 | GMVP-LINUX / G7 / RC |
| **合计** | **53** | **23** | **1** | **0** | **29** | — |

## 2. 产品边界与不变量

- Rust daemon 是唯一 authority writer；Pi、CLI、Web UI 均为客户端，不可直接写 SQLite 或推进 Task、Effect、Verification 状态。
- Provider API Key 只保存在 approved `SecretStore` backend（desktop Secret Service 或
  headless encrypted vault）；不得进入 service unit/credential material、环境、配置、
  SQLite、命令行、日志或证据。唯一例外为 ADR-0018 已登记的 P0-T06 本机 Linux 开发
  路径：显式开关后从 native store 解析、仅传给初始 Pi 子进程、默认拒绝、不得用于
  CI/发布，并在 P2 结束到期。
- Linux-native development smoke 以环境资格清单为准；`wuz@192.168.1.2` 是优先候选而非唯一主机。只有预注册 formal campaign 可推进 B01，任何候选主机名称本身都不表示测试、Gate 或 release 证据。
- 所有外部 mutating operation 均须经 Intent/Effect、持久化后派发、幂等键、fencing 和结果 reconcile；外部工具成功不等于 Task 完成。
- Task 完成由独立 verifier/acceptance authority 推进；Pi Session 不等于 Task，Pi `agent_end` 不等于完成。
- **Pi 双角色（ADR-0035）：** Pi-hosted Agent Shell 是自然语言 UI/client；managed Pi
  是 package/installation/registry/instance/execution 资源。ShellSession、Pi session、
  AgentInstallation、Agent instance、AgentExecution、process 与 Task 身份不得合并；task
  与 management bearer/cache/projection 必须隔离。
- **统一认知资源基座（ADR-0037）：** Linux 1.0 必须提供 Memory、Skill、Tool、Context、
  Task、Runtime 六类资源的最小真实 slice。六类资源经 daemon application services 和
  私有、versioned Personal projection 统一呈现，但不合并各自 authority、生命周期或
  schema，也不新增巨型 `Resource` schema 或 `Process` domain。
- **Sidecar-first（ADR-0038）：** Shell 经 per-Agent sidecar 调用 daemon application
  services；sidecar、Pi Extension、CLI 和其他 client 都不是 authority，不得持有 daemon
  bootstrap/management authority。Linux 1.0 只资格化 pinned Pi + sidecar；其他 Agent 和
  client 必须各自独立 qualification，不能继承 Pi 证据。
- **部署与授权边界（ADR-0038）：** Standard Workspace 保持低摩擦；Extended Home 只扩展
  显式 document/project roots 与可撤销的普通联网，不得触达 Secret/SSH/GPG/browser
  credential、authority/bootstrap、Docker/system socket、system directory 或 privilege
  management。desktop 使用 Secret Service；headless 使用 approved encrypted vault，locked
  start + SSH TTY unlock，optional unattended 仅使用 systemd encrypted credential 的 vault
  unlock material。三种 mode 共用同一 artifact/daemon/ports。
- Personal 计划不改变既有规范优先级，不得用 `PERS-*` ID 冒充 REQ-ID；合同变化必须走 Lane-CTR 流程。
- **低摩擦授权（ADR-0026）：** 治理记录（Intent/Effect、audit、verifier、capability）全保留；人机交互分层——Tier 0（只读与任务范围内可逆本地写）静默自动授权、Tier 1（幂等/可对账外部 mutating）首用一次授予并默认记住为 capability lease、Tier 2（不可逆/毁灭性/超预算）始终显式确认。任务准入预览是唯一默认人工授权点，默认路径人工确认 ≤1/task；预算与边界是硬轨，不建审批链；企业审批留在 Deferred Backlog。
- 产品目标平台仍为 **Linux x86_64 + Windows x86_64**（ADR-0025）；Personal
  `1.0.0` 是 Linux x86_64 single-service、Extended Home、headless/foreground release
  （ADR-0034/0036/0037/0038），由现有 `GMVP-LINUX` Gate 推广。1.0 只正式支持官方 npm
  获取并受管的 exact Pi + per-Agent sidecar；Memory 使用 SQLite FTS5 baseline，Skill
  使用 local package/revision/import/binding，Tool 使用 native catalog，Context/Task/Runtime
  使用真实 daemon ports。复杂 Context 收益优化不阻塞；embedding/vector/graph、MCP、
  dynamic marketplace、Multi-Agent、Web UI 与 Windows 安装面 post-1.0，Windows 专门
  Gate 的唯一任务归宿仍是 P7-T07。

## 3. 阶段路线图

下表的入场条件和“禁止提前作为产品主路径”只约束任务 `done`、产品集成、推广和声明范围；不禁止满足 `implementation_requires` 的隔离实现与 failure-first 测试。具体依赖必须区分 implementation、acceptance 与 promotion，禁止把后两者当作开发互斥锁。

| 能力组 | 目标 | Implementation start | Acceptance exit | 禁止扩大声明 |
|---|---|---|---|---|
| P0 | 平台、架构和安全决策 | 本计划批准 | 工具链、ADR、Secret/Pi PoC、benchmark 规格完成 | 产品功能、Memory、Multi-Agent、UI |
| P1 | 安装到首次对话 | 对应 P0/P1 implementation requirements | 至少 20 次正式 B01 campaign 达标 | 用 dev smoke 或单次 attempt 宣称 B01 |
| P2 | Runtime Spine、统一 projection 与 native Tool | P1 contracts + 对应 P2 implementation requirements；B01 不是实现 mutex | B02/B04/B05/B12 | 未资格化 sidecar/adapter |
| P3 | 真实 Context、Artifact CAS 与 UCR-01 | P2-T01/P2-T02 稳定 application contracts；不等待 P2-T08 acceptance | B03 correctness；采集 B06/B07 | 未执行的 Context 收益 |
| P4 | Memory + Skill 资源价值 | P3-T01/P3-T02 stable ports | B08 correctness/actual consumption | embedding/vector/graph claims |
| P5A | managed Pi 与通用 adapter framework | P0-T06/P1-T08 和所需 P2 supervisor contracts | B09 | 非 Pi Agent support |
| P5B | post-1.0 MCP/dynamic Tool ecosystem | P2 native Tool/Effect 闭环 | B10 | 自动市场发现或未资格化 MCP |
| P6 | post-1.0 Multi-Agent 可选实验 | 单 Agent benchmark 与明确并行假设 | B11 GO 或合法 NO-GO/disabled | 默认开启 Multi-Agent |
| P7 | Linux 1.0 Product Operability 与后续 RC | 对应 P1/P2/P3/P4/P5/P7 implementation requirements | `GMVP-LINUX` 后按声明范围汇合 RC | 用未执行能力扩大 1.0/RC |

### Linux 1.0 active release tracks（不替代现有 Phase/task ID）

| Active track | 1.0 任务范围 | 出口 | 非阻塞项 |
|---|---|---|---|
| Runtime Spine | P1-T08/T09；P2-T01..T08；P5-T01/T02/T05 的 B09；真实 Task/Runtime/native Tool 与 Pi sidecar | B01、B02、B04、B05、B09、B12 | B10、B11、non-Pi adapters |
| Resource Value | P3-T01..T06；P4-T01..T06；真实 Context/Artifact/Memory/Skill 与 UCR-01 | B03、B08；B06/B07 仅采集 | 复杂 Context 收益、embedding/vector/graph |
| Product Operability | P7-T01..T03、P7-T08 汇合 Runtime Spine 与 Resource Value | `GMVP-LINUX` / `1.0.0` | P5-T03/T04、P6、P7-T05、P7-T07 |

P3/P4 是 Linux 1.0 的 active Resource Value track，不是 1.0 之后的 Beta。已完成 P0 与
P1-T01..T07 仍是共同 foundation，但不作为第四条 active release track。

### Post-1.0 capability trains

| Capability train | 任务归宿 | 边界 |
|---|---|---|
| Embedding/semantic retrieval/vector/graph | P4 后续独立决策 | 不替代 SQLite FTS5 authority-first baseline，不阻塞 1.0 |
| MCP 与 dynamic Tool marketplace | P5-T03、P5-T04、B10 | MCP 只做 adapter，dynamic discovery 不自动启用 |
| Multi-Agent | P6-T01..T04、B11 | 默认关闭；NO-GO 是合法结果 |
| Web UI 与 Windows | P7-T05、P7-T07/B01-W | 独立 readiness/qualification，不继承 Linux 证据 |
| non-Pi Agent 与其他 Linux/hardware | 后续 adapter/port qualification | 经既有 ports 演进；不得据此宣称底层 substrate 已支持 |

### Linux 1.0 release-critical typed dependencies

| Task/Gate | implementation_requires | acceptance_requires | promotion_requires |
|---|---|---|---|
| P1-T09 / B01 | P1-T08 与既有 Secret/Provider/daemon/Pi contracts | 至少 20 个 clean-Linux attempts、成功率 ≥90%、关键安全失败 0、完整统计和 independent verifier | B01 只在完整 campaign 后 pass |
| P2-T01 | 既有 authority/store/Intent/TaskContract contracts；P1-T09 route implementation 已可集成，B01 不是 mutex | proposal/clarify/preview/admit/control/query；raw-intent durability、preview-digest binding、epoch/stale-lease fencing focused evidence | G2: B02/B04/B05/B12；task `done` 不要求这些 Gate 已运行 |
| P2-T02 | P2-T01、P1-T07、task/management channel contracts | real authenticated Personal intent record/interpret→server-issued preview→admit Task API/watch; daemon-owned governance-context binding (including the ADR-0022 durable local-root bootstrap rule), Pi Shell and CLI use one application service; channel-isolation negatives | G2: B02/B04/B05/B12 |
| P2-T03 | P2-T01、P1-T01、现有 scheduler/contract slices | durable stop、worker/Effect closure、crash/duplicate/clock/budget evidence | G2: B05/B12 |
| P3-T01 | P2-T01/P2-T02 稳定 application contracts；**不要求 P2-T08 Gate** | real Context workspace/task/evidence source、scope-before-ranking、owner-local management-session MVP authorization | B03 |
| P3-T02 | P3-T01 stable Context source port | minimum Context Builder、required fail-closed、显式 loss 与预算 | B03 |
| P3-T06 | P3-T05 | B03 Context correctness；同 campaign 可采集 B06/B07；UCR-01 固定场景 utility assertions 单独进入 P7-T08 acceptance | GMVP-LINUX Gate composition 只 requires B03；B06/B07 不阻塞 |
| P4-T01 | P3-T01/P3-T02 stable ports | Memory store/admission/policy 与 provenance/scope/freshness | B08 |
| P4-T05 | P4-T01/P4-T02/P4-T03/P4-T04；**不依赖 embedding** | Memory/Skill APIs、统一 projection、actual Context/Task consumption | B08 |
| P4-T06 | P4-T05 | B08 Memory + Skill lifecycle/correctness 与 UCR-01 actual consumption | GMVP-LINUX requires B08 |
| P5-T01 | P0-T04、P0-T06、P1-T08 | official npm exact Pi acquisition；SRI/digest/acquisition-lock；install/upgrade/rollback/uninstall negatives | B09 |
| P5-T02 | P5-T01 与所需 P2-T03/P2-T06 supervision contracts | sidecar contract/registration、instance/process identity、health、epoch fencing、pause/resume/stop、Pi foundation | B09 |
| P5-T05 | P5-T02 | B09 managed Pi + sidecar qualification；任务完成与 B10 解耦 | GMVP-LINUX requires B09 |
| P7-T01 | P0-T03、P1-T08、P2-T08 | production signing、immutable action/tool pins、SBOM、attestation、release manifest 与 acquisition-lock trust | GMVP-LINUX |
| P7-T08 | P1-T09、P2-T08、P3-T06、P4-T06、P5-T01/P5-T02/P5-T05 B09、P7-T01..T03 | Linux 1.0 六类资源 release manifest、native systemd、desktop/headless SecretStore、Pi sidecar、UCR-01 fixed-scenario correctness/utility、lifecycle/backup/doctor evidence | **B01+B02+B03+B04+B05+B08+B09+B12**；B06/B07/B10/B11 不阻塞 |

#### Context MVP authorization scope

The first runnable P3-T01/P2-T04 Context path uses one owner-local management
session as its admission boundary. It preserves daemon-only writes, immutable
source provenance, tenant/scope/conversation filtering, body-after-metadata
ordering, and Pi's candidate-only boundary. It does **not** require complete
multi-principal role evaluation, capability-chain attenuation, dynamic deny
rules, or revocation-policy administration before the MVP path can run. The
existing durable authorization/revocation ledger remains compatible hardening
infrastructure and its regressions remain valid, but advanced policy rollout
is a subsequent optimization rather than an MVP implementation mutex. This
scope change does not relax Intent/Effect, secret handling, scheduler fencing,
independent verification, Task acceptance, formal campaign, release, or
Profile requirements.

### Delivery Slice register（任务内交付出口）

Delivery Slice 是正式 `P*-T*` 任务内的可检查执行单元，不新增任务、Gate、REQ 或
产品声明，也不替代完整任务交付边界。Slice 的定义、依赖和出口由本节拥有；当前 `ready`、`in-progress`、`blocked`、
`done`、`cancelled` 状态及实际 evidence 只写入 [PROGRESS.md](PROGRESS.md) 的
Current snapshot。一个 slice 必须有一个真实 caller、durable authority outcome、可验证
端到端边界或闭合的负例出口；单独的 helper/parser/boundary 不能连续形成交付出口。每个
slice 至少需要 focused failure-first/negative test 和其定义的 supported validation，
除非明确是 non-executable documentation-only slice。实现存在但 required validation 未
执行时只能是 `blocked`，不能标为 `done`。一个任务只使用一个 task branch、Draft PR 和
task lease；一个 slice 完成后在同一工作流中立即继续下一 slice，直到最终 slice 同时完成
formal task acceptance assessment 和收口。

| Slice ID | Formal task | Delivery outcome and exit | Implementation dependency | Required validation / next dependency |
|---|---|---|---|---|
| `P2-T01/D01` | P2-T01 | TaskApplicationService 的 proposal/clarify/preview/admit/control/query 六操作面；raw intent 先持久化、preview digest 绑定、epoch fencing、stale lease 负例全部闭合 | 既有 Intent/TaskContract kernel 与 authority store | Linux focused tests、store regressions、Clippy、fmt、required CI；已闭合后 task 可按正式验收标 `done` |
| `P2-T03/D01` | P2-T03 | scheduler persistence、CAS lease、owner/epoch fencing、next-eligible、cancel 与 monotonic clock eligibility | P1-T01、P2-T01 | scheduler/store focused tests、fmt、Clippy、required CI；完成后进入 authority ceiling slice |
| `P2-T03/D02` | P2-T03 | 从 durable TaskContract/progress/budget authority facts 计算 ceiling，STOP 在 lease acquisition 前注册并保持 fail-closed | `P2-T03/D01` 与既有 TaskContract/transition contracts | exact-Linux authority tests、STOP-ordering regressions、fmt/consistency；完成后进入 Effect closure slice |
| `P2-T03/D03` | P2-T03 | 通过 immutable TaskBinding→Intent reverse lookup 唯一解析 durable Effect，并对缺失、歧义、不一致、未知状态 fail-closed | `P2-T03/D02`、现有 Intent/Effect store ports | exact revision Linux storage/classifier tests；完成后接入 D04，验证阻塞时不得另开同任务 helper slice |
| `P2-T03/D04` | P2-T03 | 真实 leased dispatch 读取 D03 closure disposition，仅 Closed 执行 owner+epoch-fenced scheduler release；Pending/STOP 保留 reconciliation | `P2-T03/D03` 与 daemon worker boundary | exact revision Linux runtime/kernel tests、required CI；完成后才进入 D05 |
| `P2-T03/D05` | P2-T03 | candidate WIA 的 exact lease-bound handoff、Effect recovery/closure 与 private scheduler tick；只有 P2-T07/D01 发行的 continuation authority 才能进入 BoundedHarness | `P2-T03/D04`、`P2-T07/D01` | failure injection + worker/recovery integration tests；完成后为 P2-T03 task acceptance 汇总入口 |
| `P2-T07/D01` | P2-T07 | daemon-private fixed post-state、verification request/report、currentness revalidation、checkpoint 与 append-only continuation authority；只允许 `ACT -> VERIFY -> CONTINUE -> OBSERVE`，绝不触发 Task acceptance/completion | `P2-T03/D05` WIA/recovery boundary与既有 Loop/Effect contracts | durable positive/negative/restart tests、exact Linux validation、required CI；完成后由 D05 消费 continuation authority，P2-T07 的 Artifact、完整 criteria evidence 和 Task completion 仍不因此关闭 |
| `P2-T07/D02` | P2-T07 | daemon-private independent verifier seam that reloads the immutable verification request and fixed post-state, validates currentness and verifier identity, and persists only content-addressed artifact evidence references in an append-only report | `P2-T07/D01` fixed post-state/request/report persistence | verifier identity mismatch, stale post-state, malformed/duplicate artifact reference, fenced writer, and passed-without-evidence negatives; exact Linux validation and required CI; no Task acceptance/completion |

## 6. 收口记录

- `P2-T07` 已完成并在 PR #164 中合并到 `main@7e75e6642d289e1127928c79fed116e00b61c987`。
- `lease/personal/P2-T07/checkpoint-artifact-verifier` 已关闭。
- 后续只从本计划中选择下一个正式任务，不再将该 lease 视为 active。
| `P2-T04/D01` | P2-T04 | private scheduler-to-deterministic-Context-to-pinned-Pi candidate worker composition；Pi output is an opaque candidate only, while scheduler lease, fencing, budget, WIA/continuation, Effect, progress, evidence, and Task state remain daemon-owned | `P2-T02`、`P2-T03/D05`、`P2-T07/D01` | real-store stale lease/fence, required Context failure, duplicate tick, exhausted budget, and self-report rejection coverage; exact Linux validation and required CI; no Tool execution or Task completion |
| `P2-T06/D01` | P2-T06 | validated daemon-private `WorkspaceRead` executor accepts only a descriptor-bound, Intent-keyed staged request; it fences stale writers before I/O, serializes duplicate key dispatch, retains only bounded/redacted output, and answers recovery queries using the original key | P2-T05 static native Tool catalog and validators | failure-first executor negatives plus exact-revision native Linux focused test and required CI; completion enables durable Effect dispatch wiring, without claiming progress, evidence, verification, Task completion, Gate, release, or Profile |
| `P2-T06/D02` | P2-T06 | wire one read-only `WorkspaceRead` through the existing durable Intent/Effect protocol: stage only after the persisted Intent reload, commit `EXECUTING` before filesystem dispatch, record an explicit outcome, and reconcile an unknown outcome by the original idempotency key | `P2-T06/D01` and the existing P2-T03 durable Effect/WIA boundary | real SQLite persist-before-dispatch, unknown-outcome/restart, stale-fence, duplicate-key, bounded-redacted output regressions; exact-revision native Linux and required CI; completion enables the remaining process supervision and mutation family work |
| `P2-T06/D03` | P2-T06 | daemon-private bounded process/check execution with supervisor-owned lifetime, timeout/orphan containment, redacted bounded output, and durable unknown-outcome reconciliation; process execution must consume the same pre-executor descriptor and Effect boundary rather than bypassing it | `P2-T06/D02` and P2-T05 process/check validator | failure-first before/mid/after fault, timeout, orphan, stale fence, output-limit, redaction, idempotency, and restart/reconcile coverage; exact-revision native Linux and required CI; no Task completion or release claim |
| `P2-T06/D04` | P2-T06 | daemon-owned private process supervisor seam with registered attempt identity, PID ownership, fencing/recovery lifecycle, bounded observation source, and fail-closed default; no arbitrary PID attach, public Process resource, or Task-completion implication | `P2-T06/D03` and daemon single-instance/lifecycle boundary | exact Linux supervisor lifecycle tests, supported CI, source-injection boundary review, and explicit evidence that absent production observation wiring fails closed; completion enables final P2-T06 acceptance assessment but does not itself claim product process management |
| `P2-T02/D01` | P2-T02 | real Task API/watch vertical path：server-issued preview→admit、watch cursor resume/dedup，并让 CLI/Shell/sidecar 共享 application service | P2-T01/D01、P1-T07 channel contracts | Rust/TS focused integration、channel isolation negatives、required CI；完成后继续 projection/parity slices |
| `P2-T02/D02` | P2-T02 | private versioned six-family Resource projection/list/watch, family-scoped cursor and task/management channel isolation; unavailable authority sources must be explicit rather than fabricated | P2-T02/D01 and existing daemon authentication boundary | focused daemon process negatives, exact-Linux projection test, fmt, required CI; completion enables deterministic CLI parity |
| `P2-T02/D03` | P2-T02 | deterministic CLI calls the same daemon Task/resource operations with distinct Task/management tokens, caches, cursors, and mutation retry policy | P2-T02/D01/D02 and admin CLI daemon client | daemon-plus-CLI process parity, channel/retry/cursor negatives, exact Linux, required CI; completion enables Shell sidecar parity |
| `P2-T02/D04` | P2-T02 | Pi Shell private sidecar calls the same daemon Task/resource application surfaces and remains a non-authority client | P2-T02/D03 and P1-T07 Pi client boundary | TS parity/isolation negatives, daemon-side integration, exact Linux, required CI; completion is P2-T02 task-closure evidence entry |
| `P3-T01/D01` | P3-T01 | daemon-issued TaskContract v0.4 strong ContextRequest binding plus append-only durable ContextRequest/ContextView persistence and daemon query/reload validation; request perspective must name the Task, views remain request-linked per-resolution artifacts, and no Pi output or Context record becomes Task/progress/evidence/acceptance authority | P2-T01/P2-T02 stable application contracts; existing Context schema and governed-object binding contract | focused schema/store/intent-binding positive and mismatch/replaced-reference/task-perspective negatives; exact-revision supported Linux validation and required CI. Until these run and the formal workspace/task/evidence source, scope-before-ranking, and revocation exits are satisfied, this slice remains in-progress and B03 remains not-run. |

**执行顺序约束：** `P2-T03/D03 -> D04 -> D05` 是同任务闭合顺序；required validation
未满足时不得越过该 slice 新增横向 helper。`P2-T02/D01` 的 implementation dependencies
不包含 B01 或 P2-T03 acceptance，可作为不相关的并行 forward-progress 出口。所有 slice
当前状态只查 `PROGRESS.md`。B01 attempt 统计、clean-reset checkpoint 和 Gate threshold
不由本 register 改写。

### Lane-CTR public-contract prerequisites

后续实际需要 public machine contract 时，Lane-CTR 分别登记并评审
`skill-manifest`、`operation-descriptor`、`agent-adapter-manifest`、TaskContract resource
bindings、server-issued preview/admit 与 Memory codegen。P2-T02/D01 当前仅冻结
`task.preview` / `task.admit` 的窄 request/result schema 与 generated bindings：draft 不可
携带 governed header、epoch、acceptance 或 preview digest；daemon 签发 preview 并在任何
authority mutation 前重算 draft digest、核验显式 acceptance 和 epoch CAS。既有 watch
cursor-resume/dedup contract 不在此批改变。统一 Personal projection 先保持 private +
versioned，出现第二个真实 adapter/client 后再评估最小 public `ResourceSummary`。禁止为此
新增 `Process` domain 或 giant `Resource` schema。

### Informative open-source references

| Reference | 可吸收边界 | 明确拒绝/不得推断 |
|---|---|---|
| Letta Code、Mem0 | Memory/agent-memory UX、provenance 和 lifecycle 研究输入 | 不采纳其存储为 authority，不据宣传推断 B08 |
| LangGraph | checkpoint/thread/store 概念对照 | 不替代 CognitiveOS recovery、fencing、Effect 顺序 |
| OpenHands ACP、Goose | Agent/sidecar protocol、headless lifecycle 研究输入 | 不继承 adapter qualification，不授予 daemon authority |
| ToolHive | MCP/tool packaging、隔离与 operation catalog 研究输入 | MCP/dynamic marketplace 保持 post-1.0，不绕过 native Tool Registry |
| Anthropic Agent Skills | local Skill package、revision、binding 的 informative 输入 | 不允许 skill 自授权、直接执行或成为 public contract 真相源 |
| ElizaOS | 多 Agent/plugin 生态对照 | Multi-Agent 保持 post-1.0 默认关闭 |

这些参考的 star 数、下载量、宣传文案或自报 benchmark 均不是 CognitiveOS
implementation、Gate、release、Profile 或选型证据。

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
> **P1-T09 B01 preregistration (2026-07-31; implementation-only):** campaign
> `B01-clean-linux-first-install-first-conversation-001` is registered as a
> future, separate clean-Linux Gate attempt. Its environment, immutable
> artifact, formal runner, independent verifier, operator-owned SecretStore
> opt-in, reset, attempt accounting, redaction, and cleanup requirements are
> fixed in [the preregistration record](../checkpoints/20260731-personal-p1-t09-b01-preregistration.md), but all execution fields are `not-run`.
> The previously qualified experimental host and campaign `.4` are excluded.
> B01, GMVP-LINUX, release, and Profile remain non-claims. A separately leased
> KVM Ubuntu 24.04/x86_64 guest now exists with a clean pre-install reset
> snapshot, and Operator A / Verifier B are assigned. Its non-secret OS,
> architecture, user-systemd, and clean-state checks passed. `gnome-keyring`
> and `secret-tool` support a non-sensitive transient-session probe, but the
> Product-compatible persistent default/login collection is absent. No B01
> attempt has started. The next action is owner initialization of that
> persistent native Secret Service collection without entering a Provider
> credential; see [the clean-VM handoff](../checkpoints/20260731-personal-p1-t09-b01-clean-vm-handoff.md).

> **P1-T09 product-route bounded blocker (2026-07-30; corrective evidence,
> normative surface unchanged):** The experimental host retains an active
> restored user service and exact Pi `0.81.1`, but has neither a product
> `cognitive` CLI nor a deployed built CognitiveOS Extension entry. The
> non-secret configure, doctor, launch, and direct first-response commands are
> therefore `not-run`; no secret or Provider/internal state was read. This is
> a blocker, not a failed first-response result: `blocked_paths` are the
> product bundle deployment paths; `blocked_task_ids`: `P1-T09`;
> `blocked_gate_ids`: `B01`, `GMVP-LINUX`, Profile; owner: P1-T08
> bundle/release artifact owner; next action: deploy a coherent CLI-plus-
> Extension bundle and rerun the bounded redacted route diagnostic. P1-T09
> remains `in-progress`, and B01 remains `not-run`. Handoff:
> [20260730-personal-p1-t09-product-pi-configuration-timeout-diagnosis-handoff.md](../checkpoints/20260730-personal-p1-t09-product-pi-configuration-timeout-diagnosis-handoff.md).

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

| P1-T09 | 安装到首次对话 route 与 B01 campaign | P1-T08 | route implementation、deterministic fixture、dev smoke、usability 与 formal B01 分阶段；B01 至少 20 次独立 clean Linux VM attempt，全部 attempt 计入，成功率 ≥90%，关键安全失败为 0；除 API Key 与模型选择外无必选交互（ADR-0026/0034） | in-progress | 2026-08-02；`development_track: experimental-local-only`，`implementation_evidence: tested-supported-ci`，`B01 gate_status: running`。Attempt 1 on `B01-Desktop-Linux-002` passed all executed phases with immutable `0.0.0-campaign.20260801.1` from `main@0a5524b`, exact Pi `0.81.1`, response in 6295 ms, `authority_side_effects:false` and post-clear secret deletion. Evidence: [attempt ledger](../checkpoints/20260801-personal-p1-t09-b01-attempt-ledger.md). The campaign still lacks the remaining denominator, aggregate median/p95 and confidence interval, ≥90% calculation, zero-critical-failure closure and final independent verifier disposition. Attempt 1 remains valid evidence and must not be deleted or rerun. |

### Phase 2 - 单 Agent 任务闭环

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P2-T01 | TaskApplicationService | P1-T09 | raw intent、preview digest、epoch fencing；admission preview 为唯一默认人工授权点（ADR-0026） | done | 2026-08-03 closure reconciliation：`P2-T01/D01` 已满足本任务全部验收摘要。`crates/cognitive-management/src/task_application.rs` 提供 proposal/clarify/preview/admit/control/query 六操作面且只组合 kernel intent-chain primitives；raw-intent durable restart、digest mismatch before mutation、epoch supersession/fencing、stale writer lease 均有 focused regression。PR #127 merged as `main@7f763c8`；Linux focused 4/4、management lib 3/3、store 6/6、Clippy/fmt 与 required Ubuntu/Windows CI 通过。P2 Gates B02/B04/B05/B12 仍为 `not-run`，task `done` 不产生 Gate/release/Profile claim。证据：[handoff](../checkpoints/20260801-personal-p2-t01-task-application-service-handoff.md)。 |
| P2-T02 | 真实 Resource + Task API/watch、统一 projection 与 CLI/Shell parity | P2-T01, P1-T07 | 六类资源的 private versioned projection；真实 Task API/watch；deterministic CLI 与 Shell 经 sidecar 调同一 daemon application services；task/management bearer、cache、retry、cursor、projection 隔离（ADR-0026/0035/0037/0038） | done | 2026-08-03 closure assessment: D01-D04 satisfy the unchanged acceptance. D01 supplies generated authenticated intent record/interpret, server-issued preview/admit, daemon-owned governance root, server WriterLease, admission negatives and bounded snapshot-first Task watch (`734cbce`, PR #141). D02 supplies private versioned six-family projection/watch and resource family/cursor/channel negatives (`70f40a5`, PR #142). D03 supplies deterministic CLI parity (`af2f6c9`, PR #143), and D04 supplies Pi sidecar parity with isolated management/Task bearer caches and snapshot-first streams (`ed01c27`, PR #144). Every slice has exact Linux and required Ubuntu/Windows CI evidence. B02/B04/B05/B12, release and Profile remain not-run/incomplete. Evidence: [acceptance handoff](../checkpoints/20260803-personal-p2-t02-acceptance-assessment-handoff.md). |
| P2-T03 | durable scheduler、lease 与 timer | P2-T01, P1-T01 | crash/duplicate lease/clock/budget、durable stop、Effect closure 与 worker fencing 测试 | done | 2026-08-07 closure: D01-D05 satisfy the unchanged acceptance. Scheduler persistence/cancel/monotonic eligibility and owner/epoch CAS fencing establish the durable lease boundary; durable TaskContract/progress/budget ceilings register STOP before lease acquisition; immutable TaskBinding reverse lookup and unknown-state handling fail closed; only a closed durable Effect releases the exact leased owner+epoch while pending/STOP work remains for reconciliation; and the daemon alone persists/adopts a candidate WIA, exact lease-bound one-time handoff, recovery, and independently verified continuation authority. Exact native Linux `08932f7868d46f494aaa76835f4818fd7a1f2962` passed the focused worker/recovery matrix, workspace fmt/build/test/Clippy, and required Ubuntu/Windows CI. B05/B12, release, and Profile remain `not-run`/incomplete. Evidence: [task closure handoff](../checkpoints/20260807-personal-p2-t03-scheduler-runtime-closure.md). |
| P2-T04 | scheduler→Context→Pi sidecar→BoundedHarness worker | P2-T02, P2-T03 | TaskContract/lease 每轮重载；Context 经真实 port；Pi sidecar candidate-only；no-progress/budget/stale-lease fail-closed | done | 2026-08-07 closure: `P2-T04/D01` is complete at `a8ef5c00654e1c05a4c30beb193b9c026654c2f1`. The daemon resolves a durable ContextRequest, revalidates authorization/revocation immediately before every body load, seals a request-bound ContextView before Pi, and admits only a daemon-sealed opaque candidate. Real SQLite negatives cover revocation after metadata discovery, missing required Context, duplicate Pi retry suppression, atomic candidate/WIA handoff, and stale/replaced lease rejection; the Pi protocol rejects authority-shaped output. Exact native Linux and required Ubuntu/Windows CI passed. No Tool execution, Task completion, Gate, release, or Profile claim is created. Evidence: [P2-T04 closure](../checkpoints/20260807-personal-p2-t04-d01-context-view-persistence-checkpoint.md). |
| P2-T05 | Native Tool Registry 与 useful operation family | P2-T04 | workspace read/search/write/patch、bounded process/check、read-only HTTP fetch；descriptor/version/digest/risk 绑定；未注册、drift、disabled 均 dispatch=0 | done | 2026-08-07 closure: the daemon-owned static six-family catalog, immutable descriptor digest binding, pre-admission persisted-descriptor verification, private Tool projection, and bounded workspace/process/HTTP pre-executor validators satisfy the unchanged acceptance. Exact native Linux `72a7e55e5a780827438bfb0fb42172cfd1e5bec1` passed focused Tool registry tests 7/7 and `cargo fmt --all -- --check`; required Ubuntu and Windows CI passed in PR #159. P2-T06 execution/supervision, external I/O, actual workspace mutation, and unknown-outcome reconciliation remain explicitly out of scope. Evidence: [closure handoff](../checkpoints/20260807-personal-p2-t05-native-tool-registry-closure.md). |
| P2-T06 | Tool/process executor、supervisor、cursor 与 reconcile | P2-T05 | persist-before-dispatch；bounded output cursor；before/mid/after fault、orphan、redaction、idempotency、unknown-outcome reconcile | done | 2026-08-08 closure: D01-D04 satisfy the unchanged acceptance at `bfcc684db6685e1077050a4b3c82fcf84c524711`. Exact native Linux passed 26 focused tests, Clippy, and fmt; required Ubuntu/Windows CI passed in Draft PR #162. The daemon-private supervisor uses registered attempt identity, PID ownership, epoch fencing, orphan/recovery/shutdown lifecycle, bounded injected observation, and a fail-closed default source. The Effect path proves persist-before-dispatch and original-key unknown-outcome reconciliation without Task advancement. No public Process resource, arbitrary PID attach, Task completion, release, Gate, or Profile claim is made; platform observation is deferred to managed-Pi/process-supervision work. Evidence: task-scoped Draft PR #162 and `docs/plan/PROGRESS.md` D01-D04 records. |
| P2-T07 | Checkpoint、Artifact、Evidence 与独立 Verifier | P2-T03, P2-T04, P2-T06 | checkpoint/restart、artifact digest、criteria evidence、Effect closure；partial/receipt/exit/`agent_end` 不得 complete | done | 2026-08-08 closure: D01-D02 satisfy the unchanged acceptance. The daemon-private fixed-post-state and verification-request/report boundary stays append-only; verifier identity, currentness, fenced-writer, malformed/duplicate artifact reference, and passed-without-evidence regressions fail closed before report persistence. Exact remote Linux `df7d483282f3ef0a6bbb17bae3d29bb24f13e0f7` passed the focused verifier test module 7/7 and targeted Clippy; local `cargo fmt --all`, `git diff --check`, and lints passed. No Provider/Tool execution, Artifact closure, Task completion, Gate, release, or Profile claim is made. Evidence: [P2-T07 closure](../checkpoints/20260808-personal-p2-t07-d02-verifier-persistence-closure.md). |
| P2-T08 | Runtime Spine E2E Gate | P2-T07 | 真实 projection→scheduler→Context→sidecar→Tool/process→checkpoint/recovery/verifier；B02/B04/B05/B12 与 false-completion negative；ADR-0018 到期核查；Tier-2 负例（ADR-0026） | not-started | — |

### Phase 3 - Context、Token 与 Loop 效率

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P3-T01 | 真实 Context source/retrieval port | P2-T01, P2-T02 | P2 application contracts 稳定即可开始，不等待 P2-T08 acceptance；workspace/task/evidence source；scope-filter 先于 ranking；revocation 测试 | done | 2026-08-07; `P3-T01/D01` closes at `0ad1ddb95f4e347d0c205597e69ad8818819948e`: immutable TaskContract v0.4 ContextRequest binding and workspace Context sources provide task-bound source provenance; source roles preserve working, authoritative-state and evidence inputs without weakening per-source authority. The daemon performs tenant/scope metadata filtering before body access/ranking, reloads durable authorization/revocation facts before every body load, persists the sealed request-bound ContextView before candidate-only Pi transport, and fails closed for revoked or required-missing Context. Exact native Linux passed `cargo test -p kernel-server` and `cargo test -p cognitive-store --test m5_context_store` (9/9); required Ubuntu/Windows CI passed in PR #161. B03 remains `not-run`; task completion creates no Gate, release or Profile claim. |
| P3-T02 | 真实最小 Context Builder 与预算 | P3-T01 | System/Shell/Task/Working/Evidence fragments；required fail-closed；loss 显式；同一 Task trace | not-started | — |
| P3-T03 | 唯一 Artifact CAS | P3-T02 | filesystem CAS + authority metadata；digest/partial-write/GC/access 测试；不得建立第二 artifact store | not-started | — |
| P3-T04 | Context delta、stable prefix、cache 与 telemetry | P3-T02, P3-T03 | delta/stable-prefix 构建、治理绑定 cache key、loss/usage/loop telemetry；stale/revoked cache fail-closed | not-started | — |
| P3-T05 | UCR-01 benefit runner 与稳定基线 | P3-T04 | 跨 Gate workload runner、raw run/CI/non-claim；B06/B07 收益只采集，不作为 1.0 pass 前提 | not-started | — |
| P3-T06 | B03 Context correctness 与收益采集 | P3-T05 | B03 correctness 通过；采集 B06/B07，收益未达或未运行不阻塞 1.0；一次 UCR-01 run 不自动通过多个 Gate | not-started | — |

### Phase 4 - Memory 与 Skill

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P4-T01 | Memory store、admission 与 policy | P3-T01, P3-T02 | 基于 stable Context ports 的 source/version/provenance/scope/freshness/retention；写入只接受 proposal 并由 deterministic policy admission | not-started | — |
| P4-T02 | SQLite FTS5 + metadata filter baseline | P4-T01 | authority metadata pre-filter 后检索；unauthorized/stale/conflict/delete/rebuild 负例 | not-started | — |
| P4-T03 | Memory lifecycle、retention 与 forget | P4-T01 | version/update/conflict/expire/forget/tombstone/audit 可验证；派生索引可完整失效 | not-started | — |
| P4-T04 | Skill package、revision、local import 与 binding | P3-T02 | local package/revision/digest/import；Agent/Task/workspace binding；Skill 不自授权或直接成为 authority | not-started | — |
| P4-T05 | Memory/Skill API 与统一 projection | P4-T01, P4-T02, P4-T03, P4-T04 | CLI/Shell/sidecar 同 application services；Memory/Skill 可解释、可撤销、可实际进入 Context/Task；不依赖 embedding | not-started | — |
| P4-T06 | B08 Memory+Skill correctness 与 UCR-01 consumption | P4-T05 | lifecycle/privacy/forget/binding/actual consumption 证据；UCR-01 同一 Task trace 消费 Memory 与 Skill | not-started | — |

### Phase 5 - Agent sidecar 与 post-1.0 Tool 生态

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P5-T01 | Agent + sidecar package acquisition/install lifecycle | P0-T04, P0-T06, P1-T08 | adapter-neutral Agent/sidecar package pins；fixed official npm exact Pi；identity/version/SRI/digest/dependency/Node checks；signed acquisition lock；stage/commit/upgrade/rollback/uninstall；安装不授权 | not-started | — |
| P5-T02 | Sidecar contract、registration、instance/process identity 与 Pi foundation | P5-T01, P2-T03, P2-T06 | versioned sidecar protocol/adapter/instance pin；AgentInstallation/Instance/Execution、SidecarSession、PiSession、process、Task 分离；health/activate/pause/resume/stop/recover 与 epoch fencing | not-started | — |
| P5-T03 | Post-1.0 MCP Tool adapter qualification | P2-T05, P2-T08 | MCP 不成为 authority；protocol/manifest drift、timeout、direct-bypass 测试；不阻塞 1.0 | not-started | — |
| P5-T04 | Post-1.0 dynamic Tool ecosystem 与 B10 | P5-T03 | dynamic discovery/package/exposure/enable/disable/quarantine/reconcile；B10 独立 campaign；不阻塞 1.0 | not-started | — |
| P5-T05 | B09 managed Pi + sidecar qualification | P5-T02 | 只负责 Pi + sidecar acquisition/install/registration/instance/process/lifecycle/recovery qualification；Pi 证据不资格化其他 adapter；任务完成与 B10 解耦 | not-started | — |

### Phase 6 - Multi-Agent

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P6-T01 | Post-1.0 Multi-Agent policy 与 scheduler 扩展 | P5-T05 | 默认关闭、budget/lease/isolation 正确 | not-started | — |
| P6-T02 | Post-1.0 mailbox 与 append-only findings | P6-T01 | 消息不可作为 authority；replay 可处理；默认关闭 | not-started | — |
| P6-T03 | Post-1.0 Reviewer/Verifier/Integrator 编排 | P6-T02 | child success 不可绕过 verifier；默认关闭 | not-started | — |
| P6-T04 | Post-1.0 B11 收益 Gate | P6-T03 | 仅在质量或速度收益达标时启用；NO-GO 并保持默认关闭是合法结果，不阻塞 GMVP-LINUX/RC | not-started | — |

### Phase 7 - 产品化与发布

| ID | 工作项 | 依赖 | 验收摘要 | 状态 | 证据/备注 |
|---|---|---|---|---|---|
| P7-T01 | Release pipeline、六资源 manifest、SBOM 与 attestation | P0-T03, P1-T08, P2-T08 | release manifest 固定六类 schema/version/digest，以及 sidecar/adapter/skill/tool pins；可验证 Linux artifact；production trust、SBOM/attestation、immutable actions/toolchain/environment | not-started | — |
| P7-T02 | Transactional lifecycle、Memory/Skill backup/restore | P1-T01, P1-T08, P2-T08, P7-T01 | update/rollback/uninstall；`cognitive backup`/`restore` 覆盖 Memory、Skill 与 bindings，排除 secret；migration preflight 与恢复证据 | not-started | — |
| P7-T03 | 六资源 doctor、headless vault、sidecar/process/effect support | P1-T05, P2-T08, P7-T02 | 六类 health；desktop Secret Service 与 headless encrypted-vault locked/TTY/unattended paths；sidecar drift、process/effect/reconcile、migration；仅 redacted facts；stable error code 与可操作恢复路径 | not-started | — |
| P7-T04 | 完整性能 campaign 与回归地板 | P3-T06, P4-T05, P5-T05 | 固定环境、raw evidence、CI 与阈值 | not-started | — |
| P7-T05 | 非阻塞 Web UI | P2-T08, P7-T03 | 通过 clients gate；只读 daemon projection | not-started | — |
| P7-T06 | RC、文档、支持矩阵与声明范围内 B01-B12 | P7-T08, P7-T04, P5-T05 | clean VM suite 与 release claim evidence；P6 可为明确 NO-GO/disabled，不阻塞 RC | not-started | — |
| P7-T07 | Windows 安装面：credential 后端、installer/service 与 B01-W Gate | P1-T02, P7-T01, P7-T02 | Windows credential store 后端（同 fail-closed 边界，无明文 fallback）、可检查 installer/service、专门 B01-W Gate 编写并执行；不阻塞 Linux RC；未执行前不得声称 Windows install parity（ADR-0025） | not-started | — |
| P7-T08 | Public Linux 1.0 Gate（`GMVP-LINUX`） | P1-T09, P2-T08, P3-T06, P4-T06, P5-T01, P5-T02, P5-T05, P7-T01, P7-T02, P7-T03 | 汇合 Runtime Spine、Resource Value、Product Operability；promotion exact benchmarks 为 B01/B02/B03/B04/B05/B08/B09/B12；六类最小真实 slice、UCR-01 fixed-scenario assertions、desktop/headless SecretStore、Pi+sidecar 与 release operability 均有 evidence；B06/B07/B10/B11 不阻塞；不构成 Profile | not-started | — |

## 5. Gate 与证据要求

| Gate | 必需结果 | 最低证据 |
|---|---|---|
| G0 | 基线、决策和 PoC 已完成或明确 NO-GO | ADR、PoC report、支持矩阵、handoff |
| B01 | 安装、初始化、Provider、daemon、Pi 首次响应 | 至少 20 次 clean Linux VM attempts（当前 campaign fixed N=20）；全部 started attempts 入 denominator；成功率 ≥90%；zero critical；aggregate median/p95/CI；independent verifier |
| B02/B04/B05/B12 | 管理、Task/Tool、恢复、unknown outcome 闭环 | 真实六资源 projection、Effect/verification evidence、负例、默认路径人工确认次数记录（ADR-0026） |
| B03 | Context correctness | scope-before-ranking、required fail-closed、explicit loss、revocation/cache/Artifact access negatives；收益不是 pass 前提 |
| B06/B07 | Context/Loop benefit observations | delta/stable-prefix/cache/telemetry raw evidence；采集但不阻塞 Linux 1.0 |
| B08 | Memory + Skill lifecycle 与 actual consumption | provenance/freshness/conflict/forget、package/revision/binding，以及同一 Task trace 的真实 Context/Task consumption |
| B09 | Managed Pi + sidecar lifecycle qualification | exact Agent/sidecar package与 protocol/adapter/instance pins；acquire/install/register/health/activate/pause/resume/upgrade/rollback/stop/uninstall/recover；identity isolation；只支持 Pi |
| B10 | Post-1.0 MCP/dynamic Tool qualification | MCP/dynamic package/manifest、drift/timeout、enable/disable/quarantine/reconcile、sandbox/bypass negatives；不阻塞 Linux 1.0 |
| B11 | Post-1.0 Multi-Agent benefit | 相同模型/预算/任务的 single-Agent baseline；默认关闭且不阻塞 Linux 1.0 |
| GMVP-LINUX | Personal `1.0.0` Linux x86_64 可发布 | benchmark composition **精确为 B01+B02+B03+B04+B05+B08+B09+B12**；P7-T08 acceptance 另汇合 production trust/native systemd/desktop-headless SecretStore/六资源 manifest/UCR-01 fixed-scenario assertions/lifecycle/backup/doctor evidence；不新增第二 release Gate |
| RC | 完整发布声明 | CI、SBOM、attestation、升级/卸载、支持矩阵 |

### UCR-01 cross-Gate preregistered workload（不是新 Gate）

[UCR-01](../evaluation/personal-unified-cognitive-resource-workload.md) 在同一个 Task trace 中
使用 Memory、Skill、Tool、Context、Task、Runtime 六类最小真实资源。它可贡献 B02、
B03、B04、B05、B08、B09、B12，但仅当对应 Gate **分别**绑定自己的 preregistration、
qualified environment、threshold、failure accounting、evidence collector 与 independent
verifier。一次 UCR-01 run 不自动 pass 多个 Gate，某 Gate 的证据也不向其他 Gate 继承；
UCR-01 不创建 B13、第二个 release Gate 或新的 task ID。

其中 required recall、unauthorized/stale exposure、Skill reuse、duplicate Effect、false
completion、stale epoch，以及 stable/changed Context 相对 full replay 的 `>=20%` 重复输入
token 降低且 verified completion 不下降，是该固定 workload 的 P7-T08 acceptance assertions。
这不等于 B06/B07 Gate 阻塞，也不能据此产生跨 W1/W2 的一般 Agent-benefit claim。

### Formal campaign preregistration minimum

B01、B02/B03/B04/B05/B08/B09/B12、GMVP-LINUX 与后续产品 Gate 的 preregistration 必须在
attempt 开始前固定：formal-plan revision/digest、campaign/Gate ID、exact OS image 与
reset、source/artifact/signature/SBOM/attestation、Node/Pi/package/SRI/adapter digest、
SecretStore/operator opt-in、workload/attempt denominator/threshold、所有失败计入规则、
evidence collector/redaction/cleanup、operator 与 independent verifier、允许 claim 和
non-claims。Handoff/attempt ledger 不得覆盖本表 threshold；不一致必须 fail closed 并走
`product-semantic` 修订。环境明细见 [PERSONAL-TEST-ENVIRONMENTS.md](PERSONAL-TEST-ENVIRONMENTS.md)。

B01 的统计解释见
[2026-08-02 addendum](../checkpoints/20260802-personal-p1-t09-b01-statistical-interpretation-addendum.md)：
它不改原 preregistration、attempt 1、fixed minimum `N=20`、成功率 ≥90% 或 zero-critical
threshold，也不把当前 `running` 状态提升为 `pass`。

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

