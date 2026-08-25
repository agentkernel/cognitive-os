# CognitiveOS Personal 全面性能与真实 Agent 任务对照测试方案

- 文档状态：**execution plan / not-run**
- 方案版本：v1.1（2026-08-12 owner 指示的去重与预算收敛修订；v1.0 为初版全量设计）
- 更新日期：2026-08-12
- 适用项目：`cognitiveos-personal`
- 实现阅读基线：`d514e8ac6aa539864a0a889b9f0a58be009521ef`
- 目标环境 ID：`B01-DESKTOP-002`
- 目标 guest/domain：`B01-Desktop-Linux-002`（下文简称 `linux-002`）
- 对照主体：同版本 Pi、同一 DeepSeek 模型、同一任务、同一预算下的
  **纯 Pi** 与 **Pi + CognitiveOS Personal**
- 变更分类：`corrective + informative`
- 本次范围：只完善方案；**未执行 benchmark，不改变任务、Gate、release 或 Profile**

## 0. 执行摘要

本方案明确设计了多批次、同任务、成对的真实 Pi Agent 对照，但必须先说明当前实现的
能力边界：

1. **当前唯一具备产品路径的成对候选：** 对 prompt-contained、无工具的 Agent 任务，
   可以设计为使用同一个 Pi `0.81.1` 进程分别运行：
   - `P`：纯 Pi，不经过 CognitiveOS；
   - `O`：Pi + CognitiveOS Extension + daemon Provider proxy。
   但仓库当前只有固定 marker 的单臂 runner，没有安全 pure-Pi broker 和 RAT-01 paired
   runner；因此正式 `P/O` campaign 初始仍是 `not-run`，不能把“设计可行”写成“已可执行”。
2. **当前不可诚实执行的成对对照：** 需要 workspace read/search/write/patch、process
   check、Memory/Skill、Effect/reconcile 或 independent verifier 的完整 Agent 任务。
   原因不是缺少测试描述，而是当前产品尚未把 admitted Task 接入持续 scheduler，
   production Tool executor 与 verifier 也没有真实 caller。
3. 因此本方案把任务分成三个 capability class：
   - `C0`：prompt-contained Agent task，当前两条产品路线原则上可达，但 paired runner/
     broker 尚未资格化；
   - `C1`：read-only workspace Agent task，当前纯 Pi 可执行，但 OS arm 不可达；
   - `C2`：mutation/recovery/cross-session governed Task，当前 OS arm 不可达。
4. `C1/C2` 的单臂纯 Pi 结果可以说明“纯 Pi 能做什么”，但不能冒充 `P/O` 性能对比。
   只有两个 arm 使用相同任务、工具和 oracle 并完整保留 denominator，才计算 OS 开销或
   收益。

多批次结构为：

- `B0` 环境、secret broker、能力和 oracle 资格；
- `B1` 小样本 pilot；
- `B2` 独立 held-out confirmatory paired campaign；
- `B3` 故障、恢复和边界任务；
- `B4` 并发与 mixed workload；
- `B5` 1 h → 8 h → 24 h soak；
- `B6` 优化后的原样 replay。

这使最终报告同时回答：

- OS 上的 Pi 相对纯 Pi 增加了多少延迟、CPU、RSS、token 和失败；
- CognitiveOS 是否提高任务正确性、约束遵守、恢复能力或跨会话复用；
- 哪些任务当前根本无法由 OS 上的 Pi 完成；
- 后续优化应优先解决 Pi 启动、Provider、daemon transport，还是产品执行链接线。

### 0.1 v1.1 修订原则（执行前收敛，非结果修改）

owner 指示在任何 batch 执行之前收敛预算与重复度。修订只做四类事：

1. 能力维度重叠的 task family 合并为一个代表 family，被吸收形态成为其冻结 seeds 中
   的 named variant（`G` 12→6；`A` confirmatory 4→3，`A2` 并入 `A1`、`A8` 并入
   `G3`）；
2. 与既有 Gate/CI 回归重复的逐项 authority 矩阵收敛为单一 lifecycle smoke
   （`MS-AUTH`、`T-GOV`），逐项深度 negative 仍由已合并的测试套件拥有；
3. 完全同形的 cell 去重（`UJ5` 并入 `B3`；`O1`–`O14` 中可由现有 cell 承载的行不再
   单设 cell）；
4. 纯诊断类样本量对齐 §7.1 的统计使用（不做无 claim 的超额采样），24 h soak 改为
   条件触发。

不变项：paired fairness contract、pure-Pi secret boundary、runner 资格门、safety hard
conditions、claim 上限、`not-run`/`not_available` 诚实台账和统计规则。该修订发生在
全部 cell 均为 `not-run` 时，不改变任何已执行 denominator。

## 1. 当前实现真值与历史基线

### 1.1 当前真实用户路径

| 能力 | 当前状态 | 对 benchmark 的含义 |
|---|---|---|
| Linux install、SecretStore、daemon | implemented | 可做真实启动、日常操作与 Provider route |
| Pi + CognitiveOS Extension 对话 | implemented / non-streaming | 可做 `C0` 真实 Agent 任务 |
| Pi built-in tools | OS 路径全部拒绝 | `C1/C2` 不能从 Pi shell 偷跑 |
| Task record→interpret→preview→admit→watch | implemented | 可测真实准入与持久化 |
| admitted Task → scheduler bootstrap | 未接线 | 准入后不会自主进入完整执行 |
| WorkspaceRead/ProcessCheck executor | implemented，仅测试 caller | 不可冒充真实用户任务 |
| WorkspaceSearch/Write/Patch/HTTP executor | registered，但未 execution-ready | 相关任务 OS arm=`not-run` |
| independent verifier/acceptance | implemented，仅测试 caller | Provider/Pi 成功不等于 Task complete |
| Memory/Skill authority | implemented，用户面 partial | 跨会话 Agent 对照尚无完整产品路径 |

稳定事实来源：

- [Task 与执行](../../personal/handbook/zh-CN/user/tasks-and-execution.md)
- [Pi 对话壳](../../personal/handbook/zh-CN/user/pi-shell.md)
- [已知限制](../../personal/handbook/zh-CN/user/known-limitations.md)
- [执行链状态](../../personal/handbook/zh-CN/developer/execution-chain-status.md)
- [性能面](../../personal/handbook/zh-CN/developer/performance-surfaces.md)

### 1.2 历史真机结果

以下只作比较基线，不自动成为新 campaign 结果：

| 历史 cell | Result | Claim boundary |
|---|---|---|
| B01 successor `002` | clean Linux N=6，5 success / 1 failure | 已有 B01 结论，不重判 |
| P9 L1 | 七模块各 200 samples | hypothesis |
| P9 L2 | governed path 52/52 | hypothesis |
| P9 L3 | DeepSeek 六 cell 共 160/160 retained | hypothesis |
| P9 L4 T1 | 10 retained，8 admitted，0 verified completion | partial |
| P9 L5 | owner closed as `not-run` | 无 Agent-benefit 结论 |

关键历史观测：

- daemon route 的 `local_non_provider_residual`：126.5–128.5 ms p50；
- DeepSeek network：898.9–1016.1 ms p50；
- Pi first response：4625 ms p50 / 5004 ms p95；
- cold daemon startup-to-ready：182.6 ms p50；
- Task admission：68.17 ms p50 / 140.77 ms p95；
- Pi route 相对 direct daemon route 的未归因增量约 3.5 s；历史 cell 使用不同 revision，
  不能把该差值直接归因于 spawn 或初始化；
- 八个 hard safety counters 均为 0。

权威报告：
[P9-T04 closure](../checkpoints/20260812-personal-p9-t04-performance-campaign-closure.md)。

## 2. 对照实验模型

### 2.1 Arms

| Arm | 定义 | 作用 |
|---|---|---|
| `D` daemon diagnostic | client → CognitiveOS daemon → DeepSeek，不启动 Pi | 分解 Provider 与 daemon local cost；不是 Agent baseline |
| `P` pure Pi | official Pi `0.81.1` → approved baseline credential broker → DeepSeek；无 CognitiveOS Extension/daemon/Task/Context/Memory | 用户要求的纯 Pi baseline |
| `O` OS Pi | official Pi `0.81.1` → CognitiveOS Extension → daemon proxy → DeepSeek | 当前可执行的 OS Agent arm |
| `G` governed Task | Task admission → Context → Pi candidate → Tool/Effect → verifier → acceptance | 完整 OS 目标 arm；当前实现 `not-run` |

Primary comparison 是 `O vs P`。`D` 只用于诊断 `O` 中 daemon/loopback 与 Pi launch
的贡献；`G` 只有在 production call chain 真正可达时才启用。

### 2.2 Pure Pi secret boundary

纯 Pi 不能通过环境变量、argv、普通配置、日志或 evidence 接收 Provider key，也不能恢复
已到期的 direct-Pi secret injection。

`P` arm 的 start gate 必须选择并独立评审以下一种路径：

1. Pi 已支持的 approved OS SecretStore + non-logging input；或
2. campaign-only loopback credential broker：
   - 从 Linux Secret Service 读取 key；
   - 只在内存中给上游请求注入认证；
   - loopback-only、single-user、无 request/response/header 日志；
   - Pi 只看到固定非敏感本地 endpoint/token；
   - 每请求有计数、时长、response-byte bound；
   - cleanup 删除 broker socket/process，不删除 owner key。

broker 只解决 secret transport，不能做 Context、Tool、Memory、Task、retry、cache 或
verification；否则 `P` 已不是 pure Pi。broker 自身的 local latency 必须独立记录。若这条
路径未通过评审，`P` arm 为 `blocked/not-run`，不能用 daemon proxy 伪装纯 Pi。

### 2.3 Paired fairness contract

每个 task-seed 的 `P/O` 必须固定：

- 同一 Pi package/version/SRI 与 Node 版本；
- 同一 DeepSeek provider、base URL、selected model snapshot；
- 相同 system/task prompt 字节与 task input digest；
- 相同 temperature、top_p、seed 支持、max output tokens；
- 相同 task timeout、retry=`0`、最大 Agent turn；
- 相同可见工具集合、工具 schema、workspace snapshot 和网络策略；
- 相同 CPU affinity、memory limit、cwd、filesystem state；
- 相同 oracle/verifier version；
- 相同 warm/cold stratum 与 Provider 时间 block。

唯一预期差异：

- `P` 不经过 CognitiveOS；
- `O` 使用 Extension、daemon proxy 和当前可达的 OS read/governance surface。

如果工具集合不相同，该 task 不进入 `O vs P` 性能比较，只进入 capability-gap 报告。

### 2.4 当前可比性矩阵

| Capability class | Example | `P` | `O` | 当前是否可成对 |
|---|---|---:|---:|---|
| `C0` prompt-contained reasoning | 代码片段诊断、事件抽取、约束计划 | route yes，broker pending | route yes，paired runner pending | **conditional / initial not-run** |
| `C1` workspace read/search | 在固定 repo 找失败原因 | yes | no product path | **no** |
| `C2a` workspace mutation/test | patch bug + run deterministic test | yes | no product path | **no** |
| `C2b` Memory/Skill reuse | session 2 无重述继续任务 | native Pi baseline可定义 | OS user path partial | **no** |
| `C2c` Effect recovery | mutation 后 receipt 前 crash | baseline fixture可定义 | no product caller | **no** |
| `C2d` verified completion | independent oracle closes Task | external oracle可定义 | verifier未接线 | **no** |

当前 campaign 若补齐并资格化 broker/runner，正式 paired conclusion 最多覆盖 `C0`。
`C1/C2` 是预注册好的后续 acceptance matrix，并在本次报告中诚实显示 `not-run` 及缺失
call chain。

### 2.5 Runner 可执行性门

现有 runner 不能直接执行本方案的多任务 paired campaign：

| Existing runner | 实际能力 | 不得扩大的结论 |
|---|---|---|
| `p9-t04-l3-provider-route-runner.mjs` | 单一 hard-coded marker 的 daemon route | 不能执行 RAT-01 或 pure Pi |
| `p1-t09-product-route-smoke.sh` | 单一 marker 的 OS Pi route | 不能执行 A1–A8 或输出完整 token/task metrics |
| `p9-t04-l4-t1-scenario-runner.mjs` | Task admission，固定 identity/deadline | 不执行 Context/Tool/verifier |
| `p9_t04_l0_l1_campaign_runner` | offline L0–L2 | 不能标记为 B01/P/O Agent evidence |
| `resource_sampler` | library/test helper | 没有 transient Pi/daemon campaign driver |

执行 B1/B2 前必须 preregister 一个 campaign-only paired runner，冻结：

- P/O 进程命令、输入字节、task-seed、arm order；
- pure Pi broker 与 OS Extension 的 digest；
- response 在内存/受控 raw store 中的 oracle 判定与立即 redaction；
- output schema、denominator、timeout、retry、process cleanup；
- runner 与 analysis code digest。

runner 只负责测量，不能成为第二 authority writer。未满足时，B1/B2 保持 `not-run`；不得
临时拼接 shell 命令后称为正式 paired campaign。

## 3. 多批次执行设计

### 3.1 `B0` — qualification 与 dry run

目的：证明环境和对照公平，而不是产生性能结论。

固定：

- exact pushed Git revision、Pi SRI、Extension digest；
- OS/kernel/glibc/CPU/RAM/disk/governor/background services；
- Provider/model snapshot、sampling window 和调用预算；
- pure Pi secret broker digest、threat review 和 cleanup；
- task corpus version、task-seed list、oracle digest；
- arm delta manifest；
- raw/redacted evidence policy。

执行：

- 每个 arm 3 个 non-counted warmup；
- 每个 task family 1 个 qualification sample；
- secret scan、response redaction、timeout、retry=0；
- 确认 `P/O` 工具集合和输入字节相同。

任何不一致都阻止进入 paired campaign。

### 3.2 `B1` — pilot batch

- 独立 pilot seeds，不进入正式样本；
- 每个通过 B0 的 C0 task family：5 seeds；
- 每个 seed：`P/O` 各 2 次；
- task block 内 arm 顺序随机；
- 估计 completion variance、latency variance、Provider failure 和 task timeout；
- 只调整未来 manifest 中预先允许调整的 timeout/sample size；
- 不基于“接近显著”追加样本。

输出：power analysis、正式 N、异常分类、不可执行 capability。

### 3.3 `B2` — confirmatory paired batch

- 使用未在 B1 出现的 held-out seeds；
- 每个 capability class 至少 30 个 paired task-seeds，最终
  `N=max(30, power-analysis result)`；
- Provider 不支持 deterministic seed 时，每个 task-arm 重复 3 次；
- 每个 seed 的 `P/O` 组成一个 paired block；
- block 顺序使用冻结 seed 随机化；
- 每 10 个 blocks 检查温度、throttle、rate-limit 和 model snapshot；
- started outcome 全部 retained。

Primary endpoints：

1. deterministic-oracle completion rate；
2. completion 非劣时的 time-to-oracle-completion；
3. total/provider/local/Pi-launch latency decomposition；
4. token/cost（仅真实 usage + 固定 pricing snapshot）；
5. human intervention 和 retry count。

### 3.4 `B3` — fault、recovery、restart 与 safety batch

每个可执行 fault × arm 固定 10 个 task-seeds：

- client deadline；
- daemon/broker unavailable；
- selected-model mismatch；
- Provider upstream timeout；
- Pi process kill；
- response-size bound；
- stale task/epoch（仅 OS）；
- OUTCOME_UNKNOWN（只有真实可达 mutation path 才执行）。

原 `UJ5` 与本批完全同形，已并入（§5.5）：restart/cleanup 作为独立 cell 保留——10 个
daemon stop/start 周期，每周期后检查 orphan/socket/lock/FD/RSS 残留。selected-model
mismatch 与 bounded deadline 已有 P9-T04 单臂历史基线（R5/R6，不可迁移），在当前
revision 按标准 N=10 重跑，不再按 20 份超采。B5 每小时 cold restart 与 UJ2 cold
stratum 提供额外重启样本。

所有 timeout、refusal、unknown、manual intervention 都保留。第三方 429 不通过主动
hammering 制造；没有受控 Provider fixture 时 rate-limit cell=`not-run`。真实 external
mutation fault 只在 production `G` arm 可达后执行。

### 3.5 `B4` — concurrency 与 mixed workload

Profiles：

- concurrency `1/8/16`（`4` 的行为可由 `1↔8` 插值，不再单列）；
- `17` in-flight 和 `33` connections 的 bounded overload；
- 每 profile 100 local reads；
- Agent tasks 在 `1/4` 并发下按 Provider budget 执行；
- mix：Pi task、health/status/doctor、六资源 get/watch、Task watch。

比较：

- `P/O` Agent throughput；
- p50/p95/p99；
- Provider dispatch count；
- CPU/RSS/FD/thread；
- watch cursor completeness；
- overload 后恢复到 warm baseline 的时间。

### 3.6 `B5` — soak

逐级晋升：

1. 1 h：必须先通过；paired C0 task block 每 5 分钟一个；
2. 8 h：1 h 无 leak/safety anomaly 才运行；paired block 放宽到每 10 分钟，降低
   Provider 消耗而不影响 slope 观测；
3. 24 h：默认 deferred；仅当 8 h 出现需要更长窗口确认的未决 slope 且 owner budget
   允许才运行。

每分钟执行 local read/watch；每小时 cold restart。记录 RSS/FD/WAL/I/O slope、Provider
denominator、orphan、socket、lock、secret scan。

### 3.7 `B6` — optimization replay

后续每项优化必须：

- 使用同一环境 class、任务集版本和 arm contract；
- 保留原始 B2 held-out set，不把它变成调参集；
- 另加一份新的 generalization set；
- 一次只改变一个 dominant-stage hypothesis；
- 同时报 before/after 和 `P/O` delta；
- 任何 safety regression 直接否决优化。

## 4. 真实 Agent 任务与场景

### 4.1 Route smoke 不算真实 Agent task

固定 marker、简单算术和单字段抽取只用于：

- Provider route qualification；
- redaction 和 oracle plumbing；
- cold/warm transport decomposition。

它们不得进入“Agent 完成真实任务”的 headline 指标。

### 4.2 `RAT-DEV-01` software、运维与 governed-task corpus

每个任务来自固定、无 secret、可 reset 的小型 Git repository 或输入包。每个 seed 包含：

- immutable input/workspace digest；
- user goal 和明确 scope；
- allowed tools/capabilities；
- budget、deadline、max turns；
- hidden deterministic oracle；
- expected mutation set 或 `mutation_budget=0`；
- cleanup/reset；
- difficulty tags。

#### `A1` — 失败测试根因与跨文件影响分析（`C0` 与 `C1`）

root-cause 形态：给出 failing test、相关代码和约束，要求定位根因并给出修复计划，
不修改文件。impact 形态（吸收原 `A2`）：一个 config field 将重命名，列出需要修改的
代码、测试和文档，并指出兼容风险。两种形态共用同一 family 预算，份额由冻结 seeds
决定。

- `C0` variant：将最小代码、test output 和目录摘要或 dependency slice 放入 prompt，
  当前 `P/O` 可成对；
- `C1` variant：Agent 必须真实 read/search 固定 workspace，当前 OS arm=`not-run`。

Oracle：

- root-cause 形态：正确文件、symbol、错误机制；引用至少两个固定事实；无 mutation；
  不把“测试失败”误报为完成；
- impact 形态：对照预先生成的 impact manifest，漏报、越界和不存在路径分别计数。

#### `A3` — 受控软件修复（`C2a`，registered `not-run`）

在 TypeScript/Rust 小仓中修复一个真实 bug：read/search → patch/write → bounded
test/lint → diff artifact → independent oracle。Oracle 要点：hidden tests 全过、diff
只覆盖 allowlist、无 dependency/source tampering 或 test weakening、process/output/turn
budget 未超、OS arm 必须有 closed Effect + independent acceptance 才算完成。当前
pure Pi 可做单臂 baseline；OS arm 无 production write/test/verifier chain，paired
result=`not-run`；oracle 细节在 corpus version 冻结时固化。

#### `A4` — 运维事件诊断与恢复计划（`C0`）

输入一组脱敏 journal、status、doctor、resource facts，要求输出：

1. root-cause class；
2. 三步恢复计划；
3. 明确哪些动作尚未执行；
4. 验证与 rollback。

这是当前最接近真实用户问题、且 `P/O` 可公平执行的任务。

Oracle：原因码、顺序、禁止 blind retry、禁止 false completion、包含 independent check。

#### `A5` — 不完整需求澄清与 Task preview（`C0` + OS admission）

用户提出有实质歧义的目标。Agent 必须先提出 clarification，不能直接声称完成或扩大 scope。

两层结果：

- `P/O` 对话质量：是否识别全部 material ambiguities；
- OS-only authority：intent record/interpret/preview/admit 是否在未澄清时 fail closed。

authority 结果不与 pure Pi 混成一个分数。

#### `A6` — 跨会话 Memory/Skill reuse（`C2b`，registered `not-run`）

Session 1 形成一条可复验事实和一个固定 procedure；Session 2 在不重放对话、不让用户
重述的情况下继续任务，并加入 stale/unauthorized distractors。Oracle 要点：required
recall=100%、user restatement=0、stale/unauthorized exposure=0、Skill digest 正确、
verified completion 不下降。当前 OS 用户路径不足，整组 paired=`not-run`；不以手工拼
prompt 伪装 Memory。

#### `A7` — external mutation + unknown outcome（`C2c`，registered `not-run`）

使用 task-scoped、可查询、幂等的本地 external-state fixture，fault point 位于 mutation
成功后、receipt persist 前。Oracle 要点：mutation 恰好一次、使用原 idempotency key
query/reconcile、无 blind redispatch、Effect closed/reconciled、independent
verification 后才 complete。当前 production Tool caller 不可达，OS arm=`not-run`。

#### `A8` — 长程约束任务：并入 `G3`

原 A8（8–12 条事实、3 个约束、2 个显式禁止项、一个中途追加事实，要求分阶段计划、
修订、保留未完成项并给出验证）与 `G3` 的 interleaved/adversarial 难度完全同形，不再
单列 confirmatory family；该任务形态成为 `G3` 的 long-horizon variant（§4.3）。
workspace/mutation 版本仍等待完整 governed Task path。

### 4.3 `RAT-GEN-01` 常见复杂通用 Agent 任务

软件开发只占总 corpus 的一部分。通用任务优先使用固定离线 document/data catalog，保证
纯 Pi 与 OS Pi 看到相同事实，并允许 deterministic 或 rubric-backed independent oracle。

v1.1 起原 12 个 family 按能力维度合并为 6 个：每个维度只保留一个代表 family，被吸收
family 的任务形态成为该 family 的 named variant，由冻结 seeds 生成，不再各占一份
30-pair 预算。

| ID | 真实用户任务（含吸收 variants） | 输入与过程 | Primary oracle |
|---|---|---|---|
| `G1` 多文档研究、知识整理与文档工作流（吸收原 `G5`/`G12`） | 8–20 份相互重叠且部分冲突的报告或混合笔记；带证据综合，或提取→去重→比较→生成决策 memo/可检索摘要 | claim→source 映射、遗漏/矛盾识别、unsupported claim=0、重复合并/provenance、schema/格式/受众约束 |
| `G2` 表格数据分析与核对（吸收原 `G7`） | CSV/JSON 销售、预算、发票或实验数据；计算、异常、重复项、违规、建议 | 数值 tolerance、公式正确、缺失值处理、金额/税/重复与违规检测、结论与证据链可追溯 |
| `G3` 约束规划、排程与重规划（吸收原 `G8`/`G10`/`A8`） | 人员/时间/依赖/预算/优先级；固定时刻表/价格 snapshot；10–30 步长程任务、途中新增约束、暂停/恢复 | hard constraints 全满足、objective score、时间/预算可行、冲突=0、变更后重规划成本、milestone/未完成项诚实 |
| `G4` 采购/方案比较 | 固定产品 catalog、需求和风险偏好 | hard filter、Pareto/权衡、不得虚构规格 |
| `G6` 政策约束的多轮沟通处置（吸收原 `G11`） | 多轮用户描述、policy、历史记录；会议记录、邮件线程、角色和截止时间 | policy 合规、问题分类、action owner/date recall、冲突识别、下一动作、不得越权承诺 |
| `G9` 安全/隐私审查 | 配置、data-flow、policy 摘要 | findings recall/precision、严重度、secret/PII 处理 |

每个 family 至少含三种 difficulty：

- `basic`：单一目标、无冲突；
- `interleaved`：多来源、多约束、一个中途变更；
- `adversarial`：冲突来源、无关信息、诱导越权或虚假完成。

每个被吸收 variant 至少出现在一个 difficulty 层的冻结 seeds 中，保证任务形态覆盖不因
合并丢失。

#### 通用任务的真实执行形态

1. `C0-contained`：所有输入以 bounded Context 分轮提供；当前可设计 P/O paired；
2. `C1-files`：Agent 必须真实浏览固定 document/data workspace；当前 OS arm `not-run`；
3. `C2-actions`：任务要求写 artifact、调用 Tool、持久状态或外部 action；当前 OS arm
   依真实 production Tool/Effect path 决定。

对 `G1/G4` 及 `G3` 的行程/价格类 variant 如需在线实时信息，必须在 campaign 前冻结
网页/API snapshot 与采集时间；
否则 Provider/网络/外部内容变化会破坏 paired comparison。默认 confirmatory 使用离线
snapshot，live-web 仅为单独 observation stratum。

### 4.4 `RAT-SKL-01` Skill 安装、管理与实际复用任务

Skill 不只是 prompt 片段。评测必须覆盖 package、immutable revision、provenance、binding、
scope、revoke/supersede 和 Agent 实际消费。

| ID | Skill 任务 | 成功条件 | 当前产品状态 |
|---|---|---|---|
| `S1` 发现与检查 | 从候选 package 中识别兼容 Skill；核对 manifest/digest/provenance | 正确选择；不读取未授权 body；无隐式 capability | authority/API 可测，Agent 消费 partial |
| `S2` 安全安装/import | 导入本地 Skill package/revision | digest/SRI/来源/大小/兼容性通过；不可变 revision | daemon management path 已有 |
| `S3` scope/task bind | 绑定到精确 workspace/Task/目的 | cross-scope bind=0；expected version/fence 正确 | authority path 已有 |
| `S4` Agent 实际调用 | Agent 在目标任务中选择并遵循 Skill | 正确调用、步骤 adherence、结果提升、无 capability expansion | paired Agent path 当前 `not-run` |
| `S5` update/supersede | 导入新 revision，保留旧 pin，可回滚 | lineage 完整、旧 exact-pin 可解释、current 选择正确 | store path 已有 |
| `S6` revoke/forget | revoke binding 后立即不可再用 | revoked reuse=0；audit/explain 保留 | authority path 已有 |
| `S7` 冲突与恶意 Skill | capability-grant、越界路径、prompt injection、digest drift | 全部 fail closed；未产生 partial binding | negative path 可设计 |
| `S8` 跨任务复用 | 新 session 不重述 procedure，按允许 scope 复用 | reuse success、token/time delta、stale distractor exposure=0 | 用户执行闭环不足 |

Skill paired comparison 分三层：

- `P-no-skill`：纯 Pi 只有 task input；
- `P-static-procedure`：纯 Pi 得到相同 procedure bytes，但无 OS Skill lifecycle；
- `O-governed-skill`：Skill 经 OS import/bind/select/revoke 后消费。

只有三者的 procedure bytes、任务、预算和模型一致，才能区分“procedure 本身收益”和
“OS Skill 管理收益”。当前 `O-governed-skill` 缺少完整 Agent consumer，故 S4/S8 初始
`not-run`；不得手工把 Skill 文本拼入 prompt 后声称 OS Skill 已被消费。

#### 执行合并：`MS-AUTH` Memory/Skill authority lifecycle smoke

`S1`–`S3`/`S5`–`S7` 与 Memory remember/review/forget（§4.6 `O7`/`O8`）在本 campaign
合并为一个 `MS-AUTH` cell：在当前 revision 的公开 authority/API 面上按固定脚本执行
10 轮 Skill lifecycle（import→inspect→bind→supersede→revoke→revoked-reuse 拒绝）、
10 轮 Memory remember→review→forget→forget non-resurrection，外加 `S7` 每个 negative
类各 1 次。记录 authority outcome、公开 API 延迟与 §6.3 指标。逐项深度 negative 矩阵
由 B08 Gate 与已合并 CI 回归拥有，不在 campaign 内重复展开；某操作在当前公开面不可
达时如实记 `not_available`，不得为测试补实现。`S4`/`S8` 维持 `not-run` register。

### 4.5 `RAT-TOL-01` Tool 管理、选择与调用任务

Tool 评测分开衡量 catalog 管理与真实 invocation：

| ID | Tool 任务 | Oracle / hard conditions | 当前产品状态 |
|---|---|---|---|
| `T1` catalog/projection | 列出 registered、enabled、execution-ready、risk/digest | projection 与 assembled executor 一致 | 可读；仅 Read/ProcessCheck ready |
| `T2` enable/disable/quarantine | lifecycle 切换后 Agent exposure 更新 | disabled/quarantined invocation=0 | dynamic path 仅 post-1.0 fixture evidence |
| `T3` Tool 选择 | 多个相似 Tool 中选择最窄能力 | selection precision、unnecessary call=0 | C0 可做选择题；真实调用未接线 |
| `T4` Workspace read/search | 定位 document/code/data fact | path/scope 正确、bounded output、fact recall | Read test-only；Search registered-only |
| `T5` write/patch | 产生 allowlisted artifact/diff | preimage、atomic publish、diff oracle、越界写=0 | registered-only / no caller |
| `T6` process/check | 运行 bounded deterministic verifier | command allowlist、timeout/output、exit semantics | executor test-only，production observation fail closed |
| `T7` HTTP read-only | 获取 pinned HTTPS snapshot | redirect/credential URL/size/timeout policy | registered-only |
| `T8` descriptor/version drift | invocation 时 descriptor 被替换 | deny before dispatch、dispatch=0 | negative 可设计 |
| `T9` unknown outcome/reconcile | dispatch 后 receipt 前 crash | original key、duplicate Effect=0、bounded recovery | no production caller |
| `T10` MCP/dynamic Tool | discover→enable→use→quarantine→reconcile | package/manifest pin、no bypass、cleanup | fixed fixture qualification；非 live ecosystem |

Agent-level Tool metrics：

- Tool selection precision/recall；
- unnecessary Tool calls / completed task；
- call success、denial、timeout、unknown denominator；
- arguments schema validity；
- first-use latency、steady-state latency；
- output bytes/tokens、redaction loss；
- task improvement versus no-Tool arm；
- duplicate dispatch、side effect count、reconcile time；
- capability/scope/risk violations；
- disable/quarantine propagation latency。

若 `P` 可以调用某 Tool 而 `O` 不能，记录 capability gap，不计算 latency non-inferiority。
若比较 Tool 收益，P/O 必须看到语义等价 Tool schema、同一 workspace snapshot、同一
side-effect budget 和同一 oracle。

#### 执行合并：`T-GOV` governance smoke 与 optional `T3`

`T1`/`T2`/`T10` 合并为一个 `T-GOV` cell：一次 projection dump 对照 assembled executor
集合（`T1`，含 §4.6 `O9` 的 descriptor integrity 面），加一轮 fixture-only dynamic
lifecycle（discover→enable→quarantine→disable，`T2`/`T10` 的当前公开面），记录
propagation 延迟与 projection 诚实性；lifecycle 不可由公开面驱动时如实记
`not_available`，并保持无 live ecosystem claim。`T3` Tool 选择题降级为 optional
pilot-only observation：不进入 confirmatory composition，只在 B1 有剩余预算时执行。
`T4`–`T9` 真实调用 cells 维持 `not-run` register 不变。

### 4.6 `RAT-OS-01` CognitiveOS 独有任务

这些任务没有“纯 Pi 完全等价”的 authority 语义。评价方式是 OS-only correctness/SLO，
或将纯 Pi 作为无治理参考，不计算传统功能 parity。

| ID | OS 独有任务 | 关键评价指标 | 本 campaign 执行处置 |
|---|---|---|---|
| `O1` intent→clarify→preview→admit | intent fidelity、material ambiguity recall、preview digest、wrong-digest/stale-epoch deny | 由 `UJ4` 承载，不另设 cell |
| `O2` Context authorization | scope-before-rank、required fail-closed、revoked/stale exposure、explicit loss | 内部路径无公开 observation surface：`not_available`；B03 历史固定证据不迁移 |
| `O3` Context cache/compaction | reauthorization=100%、cache correctness、token reduction、quality non-degradation | 同 `O2`（P8-T05 历史固定证据不迁移） |
| `O4` budget/fencing/scheduler | pre-dispatch stop、budget overshoot=0、stale writer=0、queue/fairness/starvation | scheduler 未接线：`not-run` register |
| `O5` Intent/Effect mutation | persist-before-dispatch、idempotency、exactly-once outcome、unknown reconcile | 无 production caller：`not-run` register |
| `O6` independent verification | self-report acceptance=0、evidence completeness、freshness、false completion=0 | verifier 无真实 caller：`not-run` register |
| `O7` Memory lifecycle | provenance/freshness、admission precision、forget non-resurrection、cross-scope leak=0 | 并入 `MS-AUTH`（§4.4） |
| `O8` Skill lifecycle | import/bind/pin/supersede/revoke、revoked reuse=0、explain completeness | 并入 `MS-AUTH`（§4.4） |
| `O9` Tool governance | descriptor integrity、least authority、quarantine、dispatch/reconcile | 并入 `T-GOV`（§4.5） |
| `O10` Agent/sidecar lifecycle | install≠permission、register≠activate、epoch fencing、pause/resume/recover/orphan | B09 固定矩阵历史证据；live 重跑仅在单独预注册 lifecycle procedure 时执行（§5.6） |
| `O11` six-resource projection | projection completeness/freshness、unavailable honesty、channel isolation | 并入 `UJ3` |
| `O12` secret isolation | secret exposure=0、backend fail-closed、no plaintext fallback | 由 `B0` 资格 + §6.8 全程 hard counters 承载，不另设 cell |
| `O13` audit/evidence/replay | event completeness、digest chain、deterministic replay、evidence retrievability | 公开 bounded replay 并入 `UJ3`；内部 digest chain `not_available` |
| `O14` lifecycle/backup/restore | transactional rollback、secret exclusion、restore integrity、RTO/RPO | 无用户 CLI/archive wiring：`not_available` |

当前不可执行项仍必须留在最终 capability register：

- scheduler fairness/global picker：未实现；
- production Tool/Effect/verifier full path：未接线；
- Skill/Memory Agent 消费：partial；
- backup/restore user command：unavailable；
- live MCP ecosystem、多 Agent：deferred/not-run。

OS-only 指标不得与 P/O conversation quality 简单加权成一个总分。报告至少分成：

1. general Agent capability；
2. managed Skill/Tool capability；
3. OS authority/correctness；
4. performance/resource；
5. safety/recovery。

### 4.7 Seeds 与防泄漏

- B1 pilot 与 B2 confirmatory 的 seed 不重叠；
- hidden oracle 不进入 prompt、Context 或 Agent 可写目录；
- task family 模板公开，具体值/bug/location 由冻结 seed 生成；
- report 只记录 task ID、seed digest、结果和指标，不提交 raw model response；
- oracle 变更必须产生新 corpus version，不能回改已执行 denominator。

### 4.8 当前建议的 confirmatory composition

在不推进产品实现的前提下，只有在 §2.2/§2.5 的 broker 与 paired runner 资格化后，以下
C0 composition 才可运行：

- `G1/G2/G3/G4/G6/G9-C0`：每 family 30 paired seeds，共 180；
- `A1-C0`（含 impact variant）：30 paired seeds；
- `A4-C0`：30 paired seeds；
- `A5-C0`：30 paired seeds；
- 每 arm/seed 3 replicas（Provider 无确定性 seed 时）。

核心总量为 270 paired task-seeds（v1.0 为 480；合并重叠 family 后能力维度覆盖不变）。
3 replicas 时 `P/O` 最多 1620 Agent runs，统计独立单位仍是 270 个 task clusters。B1
pilot 不计入。正式 N 可由 power analysis 上调，但不得低于每 family 30 pairs。该 N 支持
per-family completion/median 的 paired analysis，不支持 per-family 稳健 p95/p99 tail
inference。调用预算不足时，应在执行前按完整 stratum（例如 live-web 或某个 family）
删减并重新 preregister，不能执行一半后挑结果。

Provider 调用预算上限（非承诺，B0 冻结时定稿）：B1 ≈180 runs、B2 ≤1620 runs、B3
≤160（多数 fault cell 在 dispatch 前 deny，不消耗 Provider）、B4 Agent cells 按预算
封顶、B5 soak paired blocks（1 h 每 5 min + 8 h 每 10 min）≈120，合计约 2100–2300，
约为 v1.0 组成的 55–60%（且不含 v1.0 默认包含的 24 h soak）。

`G*-C1/C2`、`A1-C1`（含 impact variant）、`A3/A6/A7` 与 `G3` long-horizon 的 `C2`
版本、`S4/S8` 和真实调用型 `T4–T9` 全部保留在状态矩阵中为 `not-run`，并明确列出缺失
production call chain。

## 5. 真机用户旅程与系统场景

### 5.1 `UJ1` install → init → first response

复用已完成 B01 clean-install 统计作为历史证据；新 campaign 只在独立 preregistration
需要当前 revision 安装回归时重跑，不能污染既有 B01 ledger。

指标：安装成功率、startup-to-ready、doctor-ready、Pi first response、cleanup。

### 5.2 `UJ2` cold/warm conversation

- cold：daemon/Pi 都未启动；
- daemon-warm/Pi-cold；
- daemon/Pi 都 warm（仅在产品真实支持 process reuse 时）；
- 每 stratum 使用相同 paired tasks。

分解：

- daemon startup；
- Pi route total；
- 有真实 nested timestamp 时才单列 process spawn 与 Extension load；
- `local_non_provider_residual`；
- Provider network；
- oracle。

若没有同一次 paired run 的嵌套计时，`Pi total - daemon total` 只能报告为未归因
incremental delta，不能命名为 spawn、Extension 或 governance cost。

### 5.3 `UJ3` daily operations

| Operation | Samples | 报告层级（§7.1） |
|---|---:|---|
| health | 200 | p50/p95 |
| `cognitive status` | 100 | p50/p95 |
| `cognitive doctor` | 50 | median/MAD |
| daemon status | 50 | median/MAD |
| 六资源 get | 每 family 50 | median/MAD |
| 六资源 bounded snapshot/replay | 每 family 10 | outcome + median |
| Task same-process bounded snapshot/replay | 20 | outcome + median |

检查 JSON、snapshot-first、cursor 单调/去重、channel isolation、Tool
`registered/enabled/execution_readiness` 诚实性和敏感字段缺失。

样本量对齐 §7.1 tail 规则（p95 需 N>=100），本 cell 不做 p99 claim；`O11`/`O13` 的
公开投影与 bounded replay 诚实性由本 cell 承载，B4 在并发下复测同类操作。

当前 resource/task “watch” 都是 bounded response，不是长期订阅或真实 fan-out。不得据此
报告持续 watch throughput、跨进程 cursor durability 或断线后长期恢复。

### 5.4 `UJ4` Task admission truth

创建 30 个唯一 read-only Task，测 session mint、intent record、interpret、preview、admit
和同一 daemon process 内的 bounded watch delivery。

当前 public surface 的 watch log 在进程内存中，snapshot 固定不枚举 durable Tasks；重启后
不能通过 public API 证明 Task 仍可见。因此分开记账：

1. admission HTTP outcome 和同进程 watch event；
2. restart 后 watch 返回的实际 bounded snapshot；
3. durable Task post-restart query：`not_available`；
4. scheduler/Context/candidate/Effect/Verification/acceptance 内部状态：`not_available`。

不得读取 raw SQLite 或把 60 秒沉默当作“内部路径未运行”的证据。基于 source-backed
capability truth，当前 verified completion 是不可达能力；runtime campaign 只能断言 public
surface 上没有 observed false-completion event，不能把未观测内部事实默认计为 0。

### 5.5 `UJ5` restart、fault 与 cleanup — 并入 `B3`

原 UJ5 的 daemon stop/start、Pi kill、selected-model mismatch、bounded deadline 与
cleanup/orphan 检查和 `B3` fault 类完全同形，为消除重复计数并入 §3.4：每类固定 10 个
task-seeds，restart/cleanup 作为独立 cell 保留。same-process bounded watch cursor
negatives 由 `UJ3` 的 cursor 单调/去重检查承载。

### 5.6 `UJ6` canonical journey coverage register

全面评估不能因当前不可执行而省略用户旅程：

| Journey | Current disposition |
|---|---|
| Memory remember/review/forget | authority/API partial；Agent consumption paired path `not-run` |
| Skill import/pin/revoke | authority/API partial；Agent execution paired path `not-run` |
| Workspace read/search | read executor test-only、search registered-only；real OS Agent `not-run` |
| Workspace write/patch/check | production call chain absent；`not-run` |
| Pi install/register/activate/pause/resume/stop/recover | fixed-matrix/history evidence；live current campaign需独立 lifecycle procedure |
| Pi upgrade/rollback/uninstall | implementation/history evidence；current live run `not-run` by default |
| backup/restore | no user CLI/archive wiring；`not_available` |
| Web UI / Multi-Agent | unavailable/deferred |
| full Task independent acceptance | production verifier caller absent；`not_available` |

最终报告必须保留这些行，不能只汇报成功运行的 C0 route。

## 6. 指标体系

### 6.1 Primary task metrics

- deterministic-oracle completion rate；
- paired completion delta `O-P`；
- time-to-oracle-completion；
- 对 `G` arm：time-to-independent-verified-completion；
- user intervention count/time；
- turns、Provider calls、Tool calls；
- duplicate work/retry；
- prompt/completion/total tokens；
- 每 completed task 的费用（只有固定 pricing snapshot 才报告）。

不要把 C0 oracle pass 叫作 CognitiveOS Task completion。

### 6.2 通用复杂任务质量指标

不同 family 使用各自 oracle，同时保留统一维度：

| Dimension | Metric |
|---|---|
| Correctness | exact/partial score、事实错误、数值误差、constraint violations |
| Grounding | supported claim precision/recall、citation correctness、source coverage |
| Completeness | required item recall、遗漏 critical fact、unfinished item honesty |
| Planning | hard-constraint satisfaction、dependency order、replan quality、schedule score |
| Robustness | 输入顺序扰动、无关信息、冲突来源、中途变化后的 score delta |
| Efficiency | turns、tokens、time、Tool calls、重复步骤、每成功任务费用 |
| Autonomy | human intervention count/time、clarification quality、escalation correctness |
| Honesty | unsupported completion、fabricated source/path/action、unknown 保留 |
| Artifact quality | schema/format、diff、可执行性、可复验性、受众/长度约束 |
| Continuity | pause/resume state fidelity、user restatement、duplicate work |

开放式输出不能只用 model-as-judge。优先级：

1. deterministic parser/test/calculation；
2. hidden fact/constraint manifest；
3. independent rule-based rubric；
4. 只有机械 oracle 不可能时才用 blind out-of-band model judge，并报告 judge agreement、
   position bias 和人工抽检。

### 6.3 Skill 安装、管理与复用指标

- package/revision provenance completeness；
- digest/integrity/compatibility validation pass rate；
- import/bind/revoke/supersede latency；
- immutable revision 和 exact-pin correctness；
- scope/task binding violation count；
- revoked/stale Skill reuse count；
- Agent Skill selection precision/recall；
- Skill procedure adherence；
- skill-assisted task completion/time/token delta；
- wrong-Skill/unused-Skill/unnecessary-load rate；
- context cost 与 graded-load 命中；
- rollback/revoke propagation latency；
- explain/audit completeness；
- malicious Skill rejection 与 partial-state residue。

必须把“Skill 安装成功”“Skill 被选中”“Skill 改善任务”分成三个 denominator。

### 6.4 Tool 管理与真实调用指标

- catalog completeness 与 projection truth；
- enable/disable/quarantine/revoke propagation；
- registered→execution-ready 差异；
- Tool selection precision/recall；
- argument schema validity；
- pre-dispatch denial correctness；
- invocation success/timeout/unknown/denied denominator；
- first-use/warm latency、output bytes/tokens；
- unnecessary call、duplicate dispatch、repeated read；
- side-effect count 与 mutation budget；
- descriptor/version/policy drift detection；
- original-key reconciliation rate/time；
- sandbox/path/network/credential boundary violations；
- Tool-assisted task completion/time/token delta；
- cleanup 后 process/socket/temp/artifact residue。

Tool exit 0 不是任务完成；任务收益必须由任务 oracle/independent verifier判定。

### 6.5 OS 独有指标

| OS property | Metrics |
|---|---|
| Authority separation | non-daemon write attempts accepted=0、authority escape=0 |
| Task contract | intent fidelity、clarification recall、preview/admit digest mismatch denial |
| Context governance | unauthorized/stale exposure=0、scope-before-rank、explicit-loss completeness |
| Budget/fencing | budget overshoot=0、dispatch-after-stop=0、stale-epoch commit=0 |
| Scheduling | admission→lease、queue wait、throughput、fairness、starvation；未实现则 `not_available` |
| Intent/Effect | persist-before-dispatch=100%、duplicate mutation=0、unknown reconcile RTO |
| Verification | self-report acceptance=0、evidence completeness、false completion=0 |
| Memory | admission precision、recall、forget non-resurrection、cross-scope leak=0 |
| Skill | provenance/binding/revoke/explain correctness |
| Tool | descriptor integrity、least authority、quarantine、reconcile |
| Runtime/sidecar | install/register/activate identity、pause/resume/recover、orphan=0 |
| Secret | exposure=0、plaintext fallback=0、locked backend fail-closed |
| Evidence/audit | event coverage、digest/replay consistency、raw evidence retrievability |
| Lifecycle | startup/restart、rollback、backup/restore RTO/RPO、cleanup residue |

不生成跨维度“OS 总分”。Safety/correctness 是 hard condition；capability、quality、latency、
resource 分别报告，避免高速度掩盖 authority failure。

### 6.6 Latency decomposition

- process start → Pi ready；
- Extension/baseline adapter load；
- client → local endpoint；
- daemon/broker local preflight；
- Provider network complete-response；
- candidate parse/render；
- oracle；
- 对 G：admission、scheduler、Context、Tool/Effect、reconcile、verification、acceptance。

当前 non-streaming，TTFT=`not_available`。

### 6.7 Resource and capacity

- CPU seconds、peak/steady RSS、threads、FD、I/O bytes；
- daemon、broker、Pi 分进程；
- authority DB/WAL/cache/artifact growth；
- tasks/hour、completed tasks/hour；
- concurrency tail；
- 1 h/8 h/24 h slope；
- cleanup residue。

采样器禁止读取 argv、environment、descriptor target、prompt 或 response。

### 6.8 Safety hard conditions

| Counter | Target |
|---|---:|
| Provider secret exposure | 0 |
| unauthorized/stale Context exposure | 0 |
| duplicate external Effect | 0 |
| false completion | 0 |
| stale-epoch commit | 0 |
| unreconciled Effect after bounded window | 0 |
| completion without independent acceptance | 0 |
| scenario boundary violation | 0 |

任一非零：停止 claim promotion；保留完整 denominator；latency/quality 改善不能抵消。

每项还必须记录 evidence disposition：

- `observed_zero`：存在主动 collector、适用 denominator 和 negative control；
- `not_applicable`：该 cell 没有进入对应 mutation/authority path；
- `not_available`：路径适用但当前没有授权 observation surface；
- `observed_nonzero`：保留计数并触发 fail-closed。

禁止用结构体默认值或 runner hard-coded `0` 产生 `observed_zero`。在当前 C0 P/O 对照中，
duplicate Effect、stale epoch、reconciliation 和 independent acceptance 通常是
`not_applicable`；在 Task truth cell 中，内部 scheduler/Effect/verifier counters 通常是
`not_available`。只有 Provider secret/redaction、response oracle 和公开边界事件能够按实际
collector 判定。

## 7. 统计与结论规则

### 7.1 Paired analysis

- binary completion：paired difference + 95% CI，补充 McNemar exact test；
- latency/token/cost：仅在双方完成的 matched pairs 上报告 paired absolute/relative delta，
  同时报告所有失败 denominator；
- percentile：始终报告 median/MAD/min/max；至少有 5 个期望 tail observations 时才做
  tail inference，因此 p95 约需 N>=100、p99 约需 N>=500；
- CI：以 task-seed + Provider time block 为 cluster 做 block bootstrap（至少 10,000
  resamples），不能把同一 task 的 3 次 replica 当成 3 个独立任务；
- secondary endpoints：Holm correction；
- Provider 时间窗口作为 block；
- 不删除 outlier，不 optional stopping。

只分析双方完成会引入 survivorship bias，因此 headline 必须先给 completion，再给
completed-pair efficiency。

### 7.2 Non-inferiority calibration

以下仅在执行前 owner 批准并 preregister 后才是 blocking threshold；否则 record-only：

- `O vs P` completion 绝对下降不超过 2 percentage points；
- completion 非劣后：
  - C0 total latency overhead：p50 <=10%，p95 <=20%；
  - token/cost overhead <=5%；
  - `local_non_provider_residual` 单独报告，不把 Provider 波动归给 OS，也不把 residual
    直接命名为 governance；
- safety failure 不高于 P，且 OS hard counters 必须为 0。

### 7.3 Benefit claim

只有同时满足才允许 scenario-limited benefit：

- 95% CI 支持 completion 提升或 completion 非劣；
- completion 相对提高 >=10%，或 completion 非劣时 time/token/cost 降低 >=20%；
- 至少两个真实 task families；
- held-out confirmatory set；
- 完整 denominator；
- 可归因 ablation；
- safety 不下降；
- human/maintenance cost 未抵消。

当前只有 C0 可比，因此即使结果很好，也不能推广到 autonomous workspace Agent。

## 8. 环境、secret 与 evidence

### 8.1 Fixed manifest

必须固定：

- full Git revision、dirty=false、release build；
- environment ID=`B01-DESKTOP-002` 与 guest/domain=`B01-Desktop-Linux-002` 分字段；
- linux-002 image/kernel/glibc/hardware/governor/background load；
- Pi 0.81.1 source/SRI、Node、Extension digest；
- pure Pi broker digest 与 policy；
- DeepSeek model snapshot、parameters、timeout、retry=0；
- task corpus/oracle/seed digests；
- arm delta、tool set、budget；
- randomization seed、sample policy；
- raw evidence root、redaction、cleanup。

### 8.2 Secret

- key 只在 approved SecretStore；
- P arm 走 §2.2 approved path；
- O arm 只由 daemon proxy 解析；
- 不进入 argv/env/config/SQLite/log/test/evidence/chat；
- raw response 不进 Git；
- source key file不读取摘要、不删除；
- campaign-created SecretStore entry 在 cleanup 清除。

### 8.3 Evidence layout

```text
artifacts/performance/<campaign-id>/
  manifest.json
  environment.json
  corpus.json
  randomization.json
  arms/
    D/
    P/
    O/
    G/
  raw/
    tasks-*.jsonl
    stages-*.jsonl
    resources-*.jsonl
    faults-*.jsonl
    soak-*.jsonl
  redacted/
    denominator.json
    paired-summary.json
    capability-gaps.json
    safety-summary.json
    cleanup.json
  digests.sha256
```

Git 只保存 redacted aggregate、digest、attestation、non-claims 与最终报告。
Raw payload 必须在访问受控的 evidence store 中至少保留到 independent review 完成，并记录
可恢复 locator、digest、retention deadline 和 reviewer disposition；只有 digest 而无可取回
payload 不足以支持后续复核。

## 9. 计划执行顺序

1. 冻结 source/environment/corpus/arms/oracles/secret broker；
2. independent reviewer 检查公平性、secret、denominator；
3. `B0` qualification；
4. `B1` pilot + power analysis；
5. 冻结 B2 N 和 randomization；
6. `B2` held-out paired campaign；
7. `B3` faults；
8. `B4` concurrency；
9. `B5` soak；
10. cleanup + secret scan；
11. independent analysis；
12. 生成 current capability matrix 和优化优先级；
13. 后续优化只用 `B6` replay 验证。

任何阶段失败均保留结果并停止向更高 claim 升格，不删除失败样本重跑美化。

## 10. 执行状态矩阵模板

| Batch / class | Status initial | Required result |
|---|---|---|
| B0 qualification | not-run | arm fairness + secret/redaction pass |
| B1 pilot C0 | not-run | variance/power/failure map |
| B2 C0 general-task P/O paired (`G1/G2/G3/G4/G6/G9`) | not-run | >=30 pairs/family + complete denominator |
| B2 C0 technical/operations paired (`A1/A4/A5`) | not-run | >=30 pairs/family + complete denominator |
| B2 C1 read-only workspace | expected not-run on current OS | missing product Tool caller recorded |
| B2 C2 mutation | expected not-run on current OS | missing write/test/verifier path recorded |
| B2 C2 Memory/Skill | expected not-run on current OS | missing user execution path recorded |
| `MS-AUTH` Memory/Skill authority smoke（原 S1–S3/S5–S7 + O7/O8） | not-run | lifecycle/negative outcomes + §6.3 延迟 |
| Skill S4/S8 actual Agent consumption | expected not-run | missing governed consumer recorded |
| `T-GOV` Tool projection + fixture lifecycle（原 T1/T2/T10 + O9） | not-run | projection truth + propagation；no live ecosystem claim |
| Tool T3 selection | optional（pilot-only） | 不进入 confirmatory |
| Tool T4–T9 actual governed calls | expected not-run | missing production caller recorded |
| OS O1 Task admission（=UJ4） | not-run | public-surface outcomes only |
| OS O2/O3 Context correctness/cache | not_available（无公开内部 observation 面） | no evidence transfer |
| OS O4–O6 scheduler/Effect/verifier | expected not-run/partial | unavailable production path explicit |
| OS O10–O14 lifecycle/projection/secret/audit/backup | covered by UJ3/B0/hard counters or not_available | no omitted capability |
| B3 faults + restart/cleanup（含原 UJ5） | not-run | bounded outcomes, retry=0 |
| B4 concurrency | not-run | tail/backpressure/resource |
| B5 1 h | not-run | leak/safety/cleanup |
| B5 8 h | not-run | only after 1 h exit |
| B5 24 h | conditional（default deferred） | only after 8 h unresolved slope + owner budget |
| B6 replay | not-run | exact before/after comparability |

## 11. 最终报告必须分开的结论

1. **Route performance：** daemon、pure-Pi broker、OS local、Provider、Pi launch；
2. **General Agent result：** `G1/G2/G3/G4/G6/G9`（含吸收 variants）同任务 `O vs P`
   correctness/grounding/planning/robustness/time/token/cost；
3. **Software/operations result：** `A1/A4/A5` 与 C2 注册项（`A3/A6/A7`），不能代表
   全部 Agent 能力；
4. **Skill result：** `MS-AUTH` 安装/绑定/版本/撤销与实际 Agent 消费（`not-run`）
   分开；
5. **Tool result：** catalog/lifecycle/selection/invocation/reconcile 分开；
6. **OS-unique result：** O1–O14 authority、Context、budget、Effect、verification、
   resource lifecycle；
7. **Capability truth：** C1/C2 哪些 OS arm 不可达；
8. **Authority truth：** admission 与 verified completion 分开；
9. **Reliability：** fault/restart/timeout/cleanup；
10. **Capacity：** concurrency、throughput、tail、resource；
11. **Long-run：** soak slope；
12. **Safety：** hard counters；
13. **Optimization priority：** 证据排序，不按架构直觉；
14. **Non-claims：** 不向 Gate/release/Profile/general Agent benefit 扩大。

## 12. 数据驱动的优化决策

| 观测 | 优先动作 |
|---|---|
| matched nested timing 证明 `O-P` 主要差在 Pi/Extension 启动 | persistent/reusable Pi process，先做 bounded lifecycle 设计 |
| O 的 `local_non_provider_residual` p95 主导 | 先补 nested timing，再 profile loopback/auth/route/store，不先迁移 async |
| Provider 主导且 O/P 相同 | 不优化 daemon；考虑模型/网络/streaming 产品决策 |
| C0 quality O<P | 检查 prompt/context alteration、output bounds、model parameters |
| C0 quality O>P | 做 ablation，确认不是时间窗口或 Provider 随机性 |
| 某个 G family 明显退化 | 先按 grounding/planning/constraint/tool-use 子指标定位，不以总平均掩盖 |
| Skill 安装正确但任务无收益 | 检查 selection/load/context cost；不扩 Skill framework |
| revoked Skill 仍被使用 | priority 0 authority bug；停止收益 claim |
| Tool selection 差 | 优化 descriptor/exposure/context，不先增加更多 Tool |
| Tool 调用主导 tail | 分离 first-use、dispatch、external latency、reconcile 后再优化 |
| registered 与 execution-ready 漂移 | 优先修 projection 诚实性，禁止 UI/Agent 暴露不可调用能力 |
| C1/C2 OS not-run | 优先闭合 scheduler→Tool→verifier production call chain |
| restart/soak 异常 | recovery/resource leak 优先于功能扩张 |
| safety counter 非零 | priority 0；任何性能收益无效 |

当前最重要的判断很可能不是“OS 慢了多少”，而是：

> CognitiveOS 上的 Pi 目前只可与纯 Pi 公平比较 prompt-contained Agent 任务；真正需要
> workspace Tool、mutation、Memory/Skill 和 independent completion 的任务，OS arm 仍是
> 不可达能力，而不是一个可以测量的慢路径。

## 13. 参考

- `tools/personal/p1-t09-product-route-smoke.sh`
- `tools/personal/p9-t04-l3-provider-route-runner.mjs`
- `tools/personal/p9-t04-l3-cold-journey-runner.sh`
- `tools/personal/p9-t04-l4-t1-scenario-runner.mjs`
- `crates/cognitive-runtime/src/performance_campaign.rs`
- `crates/cognitive-runtime/src/campaign_report.rs`
- `crates/cognitive-runtime/src/task_scenario_harness.rs`
- `crates/cognitive-runtime/src/resource_sampler.rs`
- `apps/kernel-server/src/personal/task_api.rs`
- `apps/kernel-server/src/personal/pi_runtime.rs`
- `packages/pi-cognitiveos/src/daemon-provider.ts`
- [Agent benefit benchmark](agent-benefit-benchmark.md)
- [UCR-01](personal-unified-cognitive-resource-workload.md)
- [Test environments](../plan/PERSONAL-TEST-ENVIRONMENTS.md)
