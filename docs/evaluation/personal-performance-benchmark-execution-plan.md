# CognitiveOS Personal 全面性能 Benchmark 与真实任务测试执行方案

- 文档状态：**execution plan / not-run**
- 编写日期：2026-08-12
- 适用项目：`cognitiveos-personal`
- 设计基线 revision：`9fbd3904a1f8e0893fcb7d8d2b434e636d546e8c`
- 变更分类：`corrective + informative`；不改变产品语义、任务状态、Gate 或 Profile
- 关联实现：`P7-T04`、`P8-T05`、`P9-T01`、`P9-T03`
- 关联合同：`REQ-PERF-001..005`、`cognitiveos.performance-report/0.1`
- 真实 Provider：DeepSeek；专用测试密钥的 owner-local 来源为
  `C:\Users\wuron\Desktop\deepseek.txt`
- 执行状态：**本文未读取该文件、未导入密钥、未构建、未启动服务、未调用 Provider、
  未运行任何测试或 benchmark。本文中的所有结果栏初始值均为 `not-run`。**

## 1. 执行摘要

当前实现已经具备一套可信的性能测量基础，但还不是一个可以直接宣称“端到端性能收益”的
完整 campaign：

1. `p7_t04_module_benchmark` 已覆盖 Context 过滤、Context cache、Artifact CAS、scheduler
   lease CAS、Memory FTS5、Intent/Effect 持久化和 canonical report serialization。
2. `GovernedPathStageCollector` 已能记录 authorization、Context resolution、cache reuse、
   Effect persistence 四阶段原始耗时；`p9_t01_async_decision_gate` 已证明现有 collector 的
   `effect_persistence` 包含 store open/admission/persist/reload，**不能**被解释成
   HTTP/watch/sidecar transport 耗时。
3. `P9-T03` 已把 daemon request path 改为复用长生命周期 `SqliteAuthorityStore`；现有
   `store_access` collector 可做 per-open 与 long-lived 对照，但目前只是 test-level、
   hypothesis-only 观测。
4. 真实 DeepSeek 路径已存在：`cognitive init` 将密钥导入 approved Secret Store，
   daemon 代理 OpenAI-compatible `/chat/completions`，Pi Extension 不接触密钥；已有
   redacted Provider smoke 和 Pi first-response smoke。
5. 当前 Provider 路径是**非流式**请求；因此可测完整响应耗时，但不能伪造 TTFT。
   Pi adapter 当前把 token/cost usage 填成 0，daemon client 也不保留 Provider `usage`，
   因而在增加脱敏 usage collector 前，不得发布 token/cost 节省结论。
6. Task API、scheduler、Context、Pi candidate、Tool/Effect、recovery 和 independent verifier
   的 authority primitives 已实现；但仓库目前没有一个面向性能 campaign 的、可一次提交
   “真实完整 Task”并导出所有阶段时间的公共 runner。该 runner 是正式端到端 campaign
   开始前的必要准备项，不能用 fixture matrix 或单元测试耗时代替。

建议采用六层方案：

| 层 | 目的 | Provider | 结论上限 |
|---|---|---|---|
| L0 | 环境、revision、secret、correctness 资格检查 | 否 | 是否允许开始测量 |
| L1 | deterministic module microbenchmark | 否 | implementation regression floor |
| L2 | daemon governed-path 与 store/transport stage timing | 否 | 瓶颈定位，hypothesis |
| L3 | 真实 DeepSeek + daemon proxy + Pi first response | 是 | route-level tested-local |
| L4 | 真实 governed Task 情景、fault、并发、soak | 是 | scenario-limited performance report |
| L5 | W1/W2 A/B/C/D 对照与统计 | 是 | non-inferiority；满足全部合同后才可能 significant benefit |

推荐先完成 L0-L3 的工程基线，再完成 L4 的真实 Task campaign；只有需要对外声明 Agent
收益时，才执行成本最高的 L5。任何一层失败都保留完整 denominator，并按本方案的退出规则
停止升格，但不删除失败样本。

## 2. 范围、问题与明确非目标

### 2.1 本方案回答的问题

1. 当前实现中，纯本地 deterministic module 的 p50/p95/p99 和抖动是多少？
2. daemon-only 治理路径的耗时由 authorization、Context、cache、SQLite/Effect、HTTP、
   watch、sidecar、Pi 进程、Provider 网络中的哪一段主导？
3. cold start、warm state、稳定 Context、变化 Context、重启恢复时的差异是多少？
4. 在真实 DeepSeek 请求中，CognitiveOS 治理路径相对安全 native baseline 是否非劣？
5. Context compaction、stable prefix/delta、Memory/Skill reuse 是否在不降低 verified
   completion 的前提下降低重复输入或端到端时间？
6. 并发、背压、Provider timeout、sidecar crash、daemon restart、OUTCOME_UNKNOWN 等情况下，
   尾延迟、恢复时间、重复 Effect 和 false completion 是否满足安全边界？
7. 长时间运行时，RSS、CPU、SQLite WAL、Artifact、watch queue 和文件描述符是否稳定？

### 2.2 非目标

- 不把本地结果自动提升为 B01-B12、GMVP-LINUX、RC、release 或 Profile 证据。
- 不把 microbenchmark、fixture matrix、单次 first response 或模型主观评分写成 Agent 收益。
- 不为获得更好数字放松 daemon-only writer、SecretStore、persist-before-dispatch、fencing、
  budget、independent verifier 或负例。
- 不使用 `B01-Desktop-Linux-002` 或退役的 `B01-Clean-Linux-001` 做普通性能开发。
- 不在 Windows GNU host 运行 Rust build/test/bench。
- 不把 Provider 网络波动混入本地治理开销后宣称是 daemon regression。
- 不自动删除 owner 的原始 `deepseek.txt`；只处理 campaign 自己创建的临时输入和 SecretStore
  条目。

## 3. 当前实现能力与测量缺口

| 测量面 | 当前实现/入口 | 当前可测 | 主要缺口 |
|---|---|---|---|
| Module benchmark | `crates/cognitive-runtime/src/bin/p7_t04_module_benchmark.rs` | 7 个 deterministic benchmark 的 raw samples、p50/p95/p99/min/max | fixture 规模偏小；默认 25 samples、3 warmups；尚无多进程重复和 CPU/RSS |
| Governed stages | `crates/cognitive-runtime/src/perf.rs` | authorization、Context、cache、Effect persistence raw nanos | Effect stage 聚合了 open/admit/persist/reload；没有 HTTP/watch/sidecar 分解 |
| Async decision | `p9_t01_async_decision_gate.rs` | cold/warm 每阶段 percentile 与 conservative decision | transport 未单独测量，不能据此决定 async migration |
| Store access | `crates/cognitive-runtime/src/store_access.rs` | per-open 与 long-lived read 总耗时 | 没有独立 release runner、percentile、并发/锁分层 |
| Loopback daemon | `apps/kernel-server/src/personal/server.rs` | 请求总耗时可从外部测；连接/在途上限明确 | 缺少 secret-free server timing headers/trace；thread-per-connection 成本未量化 |
| Provider proxy | `personal/provider_proxy.rs` | non-streaming 完整 completion；60 s upstream timeout | 无 TTFT；未保留 usage；Provider 与本地 stage 未拆分 |
| Pi first response | `tools/personal/p1-t09-product-route-smoke.sh` | redacted complete-response `duration_ms`、marker、exit/timeout | 只测固定 marker；不提供 token、阶段分解、任务 authority closure |
| Pi candidate | `pi_runtime.rs` + `pi-agent-adapter daemon-candidate` | private socket、进程启动、bounded candidate、65/70 s deadlines | 缺少 campaign timing envelope；单次 one-shot 进程成本与模型成本未分离 |
| Task admission | `/task/intent.record`、`interpret`、`preview`、`admit` | 每步可测，Task watch 可恢复 | CLI 只有 watch；性能 runner 不能把 bearer 放入 shell/log |
| Tool/Effect | scheduler authority + tool executor | persist-before-dispatch、original-key reconcile、bounded process | 缺少真实任务级 stage correlation 和统一 report export |
| Verifier | `personal/verification_executor.rs` | append-only report 与 fail-closed negatives | 缺少从 admission 到 independent acceptance 的统一 wall-clock runner |
| UCR-01 | `tools/src/ucr-runner.mjs` | 六资源、stable/changed/full replay、denominator 与 digest | 输入是 raw-run document；它不是自动执行真实资源链的 runner |
| Report policy | schema + `tools/src/performance-policy.mjs` | schema、percentile、CI、comparison、claim-level policy | 不能替代真实数据采集和独立 verifier |

### 3.1 当前 benchmark 的七个固定模块 ID

1. `context-resolution-filter-builder`
2. `context-cache-full-key-hit`
3. `artifact-cas-immutable-publish-readback`
4. `scheduler-eligible-lease-cas`
5. `memory-fts5-metadata-first-retrieval`
6. `intent-effect-durable-persist-before-dispatch`
7. `canonical-performance-report-serialization`

这些 ID 应保持不变以维持历史对照。规模扩展应新增 manifest dimensions，不应复用同一 ID
却静默改变 fixture。

### 3.2 在正式执行前必须补齐的 measurement-only 能力

以下属于 instrumentation，不得改变 authority 语义：

1. **统一 run/correlation ID：** 从 Task admission 到 Context、Pi candidate、Tool/Effect、
   reconciliation、verification 使用同一随机 campaign correlation ID；不得包含 prompt、
   response、secret 或 bearer。
2. **monotonic stage timestamps：** 至少分解为 client→daemon、session mint、Task record、
   interpret、preview、admit、scheduler wait、Context build、cache、Pi process start、
   Provider proxy local preflight、Provider network、candidate parse、Intent persist、dispatch、
   reconcile、verification、acceptance。
3. **Provider usage collector：** 仅在 Provider 响应明确包含有限数值 `prompt_tokens`、
   `completion_tokens`、`total_tokens` 时记录；不记录 raw headers/body。缺失时写
   `not_available`，绝不写 0 代表实测值。
4. **transport collector：** 单独测量 loopback accept/read/auth/route/write、resource/task
   watch、private Unix socket 和 Pi process launch，避免继续把 transport 归入 Effect stage。
5. **OS resource sampler：** 每秒记录 daemon/Pi/adapter 的 CPU、RSS、线程、FD、I/O bytes；
   只记录 process identity 和数值，不抓 argv/environment。
6. **campaign runner：** 直接调用已有 Rust/TS client/service，不在 shell 命令中暴露 bearer；
   输出只含 ID、digest、duration、count、status 和 registered error code。

在这些能力完成前，L1-L3 仍可执行，但 L4/L5 中对应指标必须明确为 `not-run` 或
`not_available`。

## 4. 不可放松的安全、证据和 secret 规则

### 4.1 DeepSeek 密钥处理

1. 本方案只登记源路径，不读取内容，不计算源文件 digest，不复制内容到 Git、聊天、日志、
   SQLite、普通配置、evidence 或 shell history。
2. 正式执行时，推荐在 `DEV-LINUX-NATIVE-01` 的交互 TTY 中运行 `cognitive init`，通过
   CLI 的 hidden-input 路径手工粘贴一次；该路径关闭 terminal echo，并由 daemon/client
   代码把材料写入 production Secret Service。
3. 如果必须使用文件输入，只允许 campaign 临时创建的 `0600` 文件，经
   `--api-key-file <path>` 导入后立即安全删除；不得把 Windows Desktop 原文件复制到
   Git worktree、`artifacts/`、普通远程目录或 VM image。
4. 不使用 `--allow-ephemeral-secret-backend` 保存真实密钥。
5. 不使用已到期的 ADR-0018 direct-Pi secret injection；`pi-agent-adapter run/evaluate` 不能
   作为真实 Provider 路径。
6. DeepSeek 请求必须通过 daemon Provider proxy；Pi、sidecar、benchmark runner 只见
   selected model 和 bounded response，不见 secret。
7. campaign 结束后删除 campaign 专用 SecretStore entry 和临时文件；是否删除 owner 的
   `C:\Users\wuron\Desktop\deepseek.txt` 由 owner 决定，本方案不自动操作。

### 4.2 Evidence 等级

- `DEV-LINUX-NATIVE-01` 的结果最多为 `tested-local` / `experimental-local-only`。
- Ubuntu/Windows ordinary CI 只支持 `tested-supported-ci` implementation evidence。
- 只有另行 preregistered、qualified campaign 才能产生 product Gate/release/Profile claim。
- `not-run`、timeout、refusal、OUTCOME_UNKNOWN、quarantine 和 manual intervention 都保留在
  denominator；不能删除后重算。
- 本文设计本身不改变 `PROGRESS.md` 中已完成任务或 Gate 状态。

## 5. 环境与固定变量

### 5.1 执行环境

| 用途 | 环境 | 要求 |
|---|---|---|
| 文档/静态校验 | `DEV-WIN-GNU-01` | 仅非 linking 工作；不运行 Rust bench |
| 主性能环境 | `DEV-LINUX-NATIVE-01`，`wuz@192.168.1.2` | exact pushed Git revision、disposable worktree、native Linux、user-systemd |
| 支持性回归 | `CI-UBUNTU-01`、`CI-WINDOWS-MSVC-01` | correctness only；floating CI 不作固定硬件性能地板 |
| 正式 release 性能 | 新的 preregistered fixed-native campaign environment | 固定硬件、OS image、governor、background load、reset |

不得将 `B01-Desktop-Linux-002` 用作普通 benchmark guest。

### 5.2 BenchmarkManifest 必须固定的字段

- 完整 Git revision、dirty=false、构建 profile=`release`、Rust `1.97.1`；
- Linux kernel、distribution/image digest、glibc、CPU model/microcode、core/NUMA、RAM、磁盘、
  filesystem、free space；
- CPU governor、turbo 状态、CPU affinity、thermal 状态、background services；
- Node `>=22.19.0`、pnpm `10.33.2`、Pi `0.81.1`、Pi source/SRI；
- daemon artifact、Pi package、Extension、adapter、Skill、Tool descriptor digest；
- DeepSeek provider ID、base URL、selected model ID、discovery snapshot digest、测试时间窗口；
- temperature、top_p、seed 支持情况、max output tokens；
- 并发、request size、Context candidate 数、Memory corpus 规模、cache state；
- timeout/retry policy；Provider completion **不自动 retry**；
- fault profile、risk class、任务集 digest、verifier/grader digest；
- secret backend class，只记 `linux-secret-tool` 等类别，不记 SecretRef 或 key digest；
- raw evidence root、redacted report digest、cleanup result。

### 5.3 执行隔离

- 每个 cold sample 使用新临时 XDG root 或明确重启的 daemon，并记录 cache reset。
- warm sample 在固定 warmup 后测量，warmup 不进入 denominator。
- microbenchmark 固定单 CPU affinity；并发 benchmark 另开 profile，不混合。
- Provider arms 采用 randomized block 顺序，避免把时段、限流或网络拥塞误判为 arm 差异。
- 每轮之间检查温度、CPU throttling、磁盘空间和 Provider rate-limit；违反 preregistration
  时样本仍保留，但标记 `environment_invalid`，不得静默重跑替换。

## 6. 指标体系

### 6.1 正确性和安全指标（硬条件）

| 指标 | 目标 |
|---|---:|
| unauthorized/stale Context exposure | 0 |
| Provider secret exposure | 0 |
| duplicate external Effect | 0 |
| false completion | 0 |
| stale-epoch commit | 0 |
| unreconciled state after bounded recovery window | 0 |
| Task reported complete without independent acceptance | 0 |

任何一项大于 0 都终止 claim 升格；性能改善不能抵消安全失败。

### 6.2 延迟指标

- module operation p50/p95/p99/min/max/MAD；
- daemon startup to health、startup to doctor-ready；
- session mint、Task record/interpret/preview/admit；
- scheduler queue wait 与 lease acquisition；
- Context discovery/authorization/body load/build/cache；
- Pi process spawn、Pi model activation、candidate total；
- Provider proxy local preflight、Provider network complete-response；
- Intent persist、dispatch、receipt/reconcile、verification、acceptance；
- time-to-first-complete-response；
- time-to-verified-completion；
- recovery-to-runnable、recovery-to-reconciled、recovery-to-verified。

**TTFT 当前不可测。** Provider proxy 和 Pi adapter 都是 non-streaming，只有在实现真实
stream timestamp 后才能新增 TTFT 指标。

### 6.3 吞吐和容量

- requests/s、Tasks/hour、verified Tasks/hour；
- concurrent profiles：1、4、8、16；
- overload probes：17 个 in-flight、33 个 connections，验证 bounded 429/fail-closed；
- request body：1 KiB、64 KiB、256 KiB、1 MiB，以及 1 MiB + 1 byte rejection；
- private Pi frame：小、中、接近 256 KiB limit，以及 limit + 1 rejection；
- watch fan-out、resume cursor、128-event replay window 边界；
- scheduler queue depth：1、10、100、1000；
- Memory corpus：1K、10K、100K records，分别测高/低 selectivity 和 miss。

### 6.4 资源与存储

- daemon/Pi/adapter CPU seconds、peak/steady RSS、threads、FD；
- bytes read/write、fsync count、SQLite query count；
- authority DB/WAL、Artifact CAS、cache 增长；
- 每 governed call 额外 writes/bytes；
- 1 h、8 h、24 h soak 的 RSS/WAL/FD 斜率；
- cleanup 后残留进程、socket、temporary file、stale lock 数。

### 6.5 Provider、token 和费用

- Provider request count、success/timeout/rate-limit/upstream-failure denominator；
- Provider complete-response latency；
- prompt/completion/total tokens（仅 Provider 返回真实 usage 时）；
- cache-read/cache-write token（仅 Provider 明确提供时）；
- 每 verified task 的 token 与费用；费用使用 campaign 开始前固定的 pricing snapshot，
  不把硬编码 0 当作真实价格。

### 6.6 统计方法

- 比例：Wilson 95% CI；
- 连续量和 percentile：BCa/bootstrap 95% CI，至少 10,000 resamples；
- A/B/C/D：同任务 paired comparison，报告 absolute delta、relative delta 和配对效应量；
- task order：固定 seed 的 randomized block；
- secondary endpoints 多重比较：Holm correction；
- 不用均值替代 p95/p99；同时报告 median、MAD、min/max 和 outliers；
- pilot 用于方差估计，正式 sample size 由 power >= 0.8 决定；不得 optional stopping。

## 7. Benchmark suite

### 7.1 L1 deterministic module suite

在已有七项基础上使用以下矩阵：

| Family | Dimensions | 重复策略 |
|---|---|---|
| Context resolve | candidates 10/100/1000；authorized 1%/10%/100%；required miss | 每 cell 10 process × 200 measured iterations |
| Context cache | cold miss、full-key hit、stale digest、revoked source | 同上；negative 只看正确拒绝与耗时分布 |
| Artifact CAS | 1 KiB/64 KiB/1 MiB；new/dedup/readback | 每 cell 200 iterations |
| Scheduler CAS | queue 1/10/100/1000；contention 1/4/8 workers | 每 cell 30 independent runs |
| Memory FTS5 | corpus 1K/10K/100K；hit 1/10/miss；scope selectivity | 每 cell 100 queries × 10 runs |
| Intent/Effect | new intent、idempotent replay、reload、unknown reconcile | 每 cell 100 operations × 10 runs |
| Report serialization | metrics 10/100/1000；comparison/no comparison | 每 cell 500 iterations |

当前 binary 的 fixture 只覆盖最小规模。规模矩阵应作为新的 manifest-driven runner 实现，
原有七项先作为 continuity baseline 保留。

**现有命令模板（仅计划，不在本文执行）：**

```bash
export REVISION="<full-immutable-revision>"
export COGNITIVEOS_BENCHMARK_SAMPLES=200
cargo run --release --locked -p cognitive-runtime \
  --bin p7_t04_module_benchmark -- \
  --source-revision "$REVISION" \
  > artifacts/performance/<run-id>/module-observation.json
```

### 7.2 L2 governed-path、store 与 transport suite

1. 运行 cold/warm `GovernedPathStageCollector`，每个 run 100 samples，至少 10 个独立 run。
2. 保留当前四阶段，以便和 P7-T04/P9-T01 历史对照。
3. 新增 transport-only stage 后，再测 HTTP/watch/sidecar；不能回填历史数据。
4. long-lived store 与 per-open 对照使用同一 DB、同一 read set、同一 process affinity；
   先随机化模式顺序，再报告 paired delta。
5. 同时记录 WAL、fsync、RSS 和 CPU，解释 latency 改善是否以资源增长换取。

**现有 P9-T01 命令模板：**

```bash
export REVISION="<full-immutable-revision>"
export P9_T01_RUNS=10
export P9_T01_SAMPLES=100
cargo run --release --locked -p cognitive-runtime \
  --bin p9_t01_async_decision_gate -- \
  --source-revision "$REVISION" \
  > artifacts/performance/<run-id>/governed-stage-observation.json
```

该命令的结果仍是 `hypothesis`；即使 `effect_persistence` 占比超过 50%，也不能推导 stream
async migration。

### 7.3 L3 DeepSeek route suite

| Scenario | 路径 | Primary metric | Samples |
|---|---|---|---:|
| R1 Provider proxy marker | daemon client → proxy → DeepSeek | complete-response latency、success rate | pilot 10；正式 30 |
| R2 Pi first response | Pi → Extension → daemon proxy → DeepSeek | process-to-complete-response latency | pilot 10；正式 30 |
| R3 cold daemon + first response | start → doctor → Pi → Provider | total cold journey | 20 |
| R4 warm repeated conversation | same daemon/session，fixed prompt set | p50/p95/p99、Provider count | 50 |
| R5 selected-model mismatch | wrong model ID | fail-closed latency、dispatch=0 | 20 |
| R6 timeout/cancel/rate-limit | bounded fault/network policy | failure class、bounded time、retry=0 | 每类 10 |

`p1-t09-provider-proxy-smoke.mjs` 和 `p1-t09-product-route-smoke.sh` 可作为 R1/R2 的现有
redacted smoke，但正式 suite 需要 runner 在不输出 response 的前提下保存 correlation、
timing 和 denominator。

### 7.4 L4 真实 governed Task scenarios

#### T1：任务准入与只读项目分析

- 用户目标：分析固定 Git revision 中一个失败测试，给出候选修复计划，不修改文件。
- 路径：intent record → interpret → preview → admit → Context → Pi candidate → read/search Tool
  → Artifact → independent verifier。
- Oracle：候选必须引用固定事实；未执行 mutation；Task 只有 verifier 后才能 complete。
- 指标：admission latency、Context tokens、Provider calls、read calls、verified completion。

#### T2：受控软件修复（W1）

- 输入：固定的小型 Rust 或 TypeScript repository fixture；预先植入一个可由 deterministic
  tests 判定的 bug。
- 操作：read/search → patch/write Effect → bounded check process → Artifact → verifier。
- Oracle：固定测试、build、lint 和 expected diff；模型自述不计完成。
- 负例：越界路径、descriptor drift、stale epoch、budget exhausted、test 不通过。
- 指标：time-to-verified-completion、tool calls、duplicate work、token、cost、Effect count。

#### T3：跨会话 Memory/Skill reuse（W2/UCR-01）

- Session 1 通过 daemon admission 写入一条 required Memory 和一个 immutable SkillRevision。
- Session 2 不重放对话、不让用户重述；Task 必须检索 admitted Memory 并 pin 同一 Skill digest。
- 加入 stale 和 unauthorized distractors。
- Oracle：required recall=100%、user restatement=0、stale/unauthorized exposure=0。
- 指标：retrieval latency、Context size、token、verified completion、Skill reuse digest。

#### T4：stable/changed/full-replay Context

- 对同一 Task、模型、预算和事实运行 full replay、stable prefix、changed delta 三个 strata。
- stable 与 changed 分开统计；Provider cache 设置必须对称。
- 目标观测：重复输入 token 相对 full replay降低 >=20%，且 verified completion 不下降。
- 这是 UCR-01/B06/B07 scenario-limited 观测，不自动成为 generalized benefit。

#### T5：external mutation + OUTCOME_UNKNOWN recovery

- 使用 task-scoped、可查询、幂等的本地 external-state fixture。
- fault point：外部 mutation 完成后、receipt persist 前终止 daemon/sidecar。
- restart 后必须以原 idempotency key query/reconcile，不得 blind redispatch。
- Oracle：外部 mutation 恰好一次；Effect reconciled；independent verifier 后 Task 才 complete。
- 指标：recovery latency、recompute ratio、duplicate Effect、manual intervention。

#### T6：sidecar/Pi lifecycle

- install/register/activate → candidate → pause/resume → stop/recover。
- 注入 sidecar kill、protocol digest drift、epoch replacement 和 orphan process。
- Oracle：旧 epoch 无法提交；Pi/process/AgentExecution/Task identity 不混淆。
- 指标：activation、candidate、recovery latency，orphan count，RSS/FD cleanup。

#### T7：mixed interactive workload

- 同时运行 1 个 Task mutation、4 个 resource watches、4 个 Task watches、8 个 read-only
  projection clients 和 1 个 Provider request。
- 并发从 1/4/8/16 递增，随后执行 17 in-flight 与 33 connection overload probes。
- Oracle：有效请求无 authority corruption；超限请求 bounded 429；watch 无 missing/duplicate。

#### T8：8 h / 24 h soak

- 每分钟执行 status/doctor/resource watch；每 5 分钟一个 bounded read-only Task；每 30 分钟
  一个真实 DeepSeek completion；每小时一次 daemon restart/reconcile。
- 24 h run 只在 8 h 无增长异常后开始。
- Oracle：无 secret leak、stale lock、unbounded WAL、FD/RSS 单调异常增长或 false completion。

## 8. A/B/C/D Agent benefit campaign

### 8.1 Arms

| Arm | 定义 |
|---|---|
| A native baseline | 同一 Pi/DeepSeek/任务/工具/预算，不经过 CognitiveOS；但仍必须通过独立的 approved non-logging secret broker，不能恢复已过期的 direct-secret adapter |
| B governance-only | CognitiveOS 全治理路径开启；语义优化关闭；保留 deterministic Context filtering、Effect、audit、verifier |
| C optimized | B + 被测单项或明确组合：compaction、adaptive budget、stable prefix/delta、Memory/Skill reuse |
| D ablation | 从 C 每次关闭一个机制，形成 `C-minus-x` |

当前仓库没有一个可直接执行 A arm 且同时满足 A5 的 baseline runner。正式 L5 前必须建立
**非产品、隔离、approved secret broker**，或由 owner 明确指定符合 SecretStore 边界的 native
Agent baseline。若做不到，L5 状态必须是 `blocked/not-run`，不能用 daemon proxy 伪装成
“无 CognitiveOS 的 A arm”。

### 8.2 Workload 与样本量

- W1：至少 30 个 paired task-seed，最终 N 取 `max(30, power-analysis result)`；
- W2：至少 30 个 paired task-seed，最终 N 同上；
- Provider 不支持 deterministic seed 时，每个 task-arm 至少重复 3 次，并在 power analysis
  中使用 task block 与 run-level variance；
- pilot 每 family 10 个 task，只用于估计方差和失败模式，不与 confirmatory result 合并；
- arm 顺序按 task block 随机化；所有 arms 使用同一模型 revision、budget、tool、data、
  timeout、grader 和 cache policy。

### 8.3 Primary endpoints

- W1：`verified task completion rate`；非劣时再看每 verified task 的 time/token/cost；
- W2：`verified task completion rate`；secondary 为 repeated-input token 与 cross-session recall；
- Recovery stratum：`time-to-verified-completion` 与 duplicate Effect count。

### 8.4 合同门槛

治理非劣化 B vs A：

- verified completion 绝对下降 <=2 percentage points（按 95% CI）；
- W1 governance latency overhead <=3% p50 / <=8% p99，cost overhead <=2%，cache
  preservation >=0.90；
- W2 governance latency overhead <=10% p50 / <=20% p99，cost overhead <=5%，cache
  preservation >=0.90；
- 安全失败不高于 A。

显著收益必须同时满足：95% CI 支持改善；completion 相对提高 >=10%，或 completion 非劣时
token/cost/time 降低 >=20%；W1/W2 同时成立；ablation 可归因；安全失败不增加；p95/p99、
人工和维护成本不抵消收益。

## 9. 回归地板与判定

### 9.1 工程地板（本方案建议，执行前须 preregister）

以下是建议阈值，不是当前已通过能力，也不是 release Gate：

| 指标 | Alert | Block benchmark promotion |
|---|---:|---:|
| deterministic module p95 vs fixed-native baseline | >5% regression | >10% regression |
| deterministic module p99 | >10% | >15% |
| local daemon throughput | >3% drop | >5% drop |
| steady RSS after warmup | >5% growth | >10% growth or positive leak slope |
| WAL/FD after cleanup | any unexplained residue | unbounded growth / cleanup failure |
| safety counters | — | any non-zero |

阈值只在同一 fixed-native environment、相同 manifest 和足够样本下比较。Floating CI 只执行
correctness 和 hypothesis tracking，不阻断 release performance。

### 9.2 Provider 波动判定

- Provider complete-response latency 不作为本地 module regression gate。
- 同时报告 local pre-provider、Provider network、post-provider 三段；只有 local 段可进入本地
  regression floor。
- rate-limit 和 upstream failures 全部保留，并按时间 block 分层。
- 若 DeepSeek model revision 无法固定，report 必须写 `provider_revision_unpinned`，claim_level
  上限为 `hypothesis`。

## 10. 分阶段执行 runbook

### Phase 0：预注册与 dry preparation

1. 创建新的 campaign ID、manifest、SLO profile、task-set digest 和 cleanup plan。
2. 选定已经 push 的 exact Git revision；远端 disposable worktree 必须 checkout 该 revision。
3. 记录环境、toolchain、Pi/artifact/adapter digest；确认 worktree clean。
4. 完成 measurement-only runner 和 focused negative tests。
5. 用 synthetic Provider fixture 检查报告/redaction；此步骤不产生真实性能结论。
6. 独立 reviewer 检查 manifest、secret path、denominator、fault points 和 claim ceiling。

**Exit：** manifest 冻结、runner tests pass、secret scan pass；否则不进入真实 Provider。

### Phase 1：deterministic baseline

1. release build 一次；保留 build time 但不混入 operation latency。
2. 运行原有七项 continuity benchmark。
3. 运行规模矩阵、governed stages、store access、HTTP/watch/sidecar transport。
4. 每轮前后采集 OS 状态；计算 raw digest 和 summary。
5. 与 P7-T04 fixed-native baseline 做可比项对照；不可比项建立新 baseline version。

**Exit：** correctness=pass、安全计数=0、数据完整；regression breach 进入分析而非重跑删除。

### Phase 2：daemon load、fault 与 soak（无真实 Provider）

1. 使用 Provider fixture 或完全不进入 Provider 的 Task paths。
2. 执行 concurrency/request-size/watch/scheduler/FTS/CAS matrix。
3. 执行 stale epoch、locked SecretStore、disk full、socket timeout、sidecar kill、daemon restart。
4. 先 1 h soak，再 8 h；24 h 只在 8 h 合格后执行。

**Exit：** bounded failure、无重复 Effect/false completion、无资源 leak。

### Phase 3：DeepSeek SecretStore 导入与 route smoke

1. owner 在 native Linux interactive TTY 从 owner-local source 手工粘贴密钥；不在命令、SSH、
   terminal capture 中传递 secret。
2. 运行 `cognitive init --provider deepseek --base-url https://api.deepseek.com/v1`，可按
   preregistration 指定 `--model-id`；不使用 ephemeral backend。
3. 运行 `cognitive daemon start`、`cognitive doctor`，只保留 redacted readiness。
4. 执行 R1/R2 pilot；确认 marker、response、usage collector、redaction 和 Provider budget。
5. pilot 通过后执行正式 route samples；失败和限流都保留。

**命令模板：**

```bash
# Interactive hidden input: do not put the key in argv or environment.
cognitive init \
  --provider deepseek \
  --base-url https://api.deepseek.com/v1 \
  --model-id "<preregistered-selected-model>"

cognitive daemon start --bind 127.0.0.1:48181
cognitive doctor

node tools/personal/p1-t09-provider-proxy-smoke.mjs

bash tools/personal/p1-t09-product-route-smoke.sh \
  --cognitive "<absolute-cognitive-path>" \
  --pi "<absolute-pi-path>" \
  --extension "<absolute-extension-path>" \
  --timeout-seconds 90 \
  --expected-marker cognitiveos-first-response-ok
```

上述现有 smoke 只输出 redacted marker/status。不要把 response body 重定向进 evidence。

### Phase 4：真实 Task scenarios

1. 先执行 T1/T3 只读和跨会话路径。
2. 再执行 T2 的 task-scoped workspace mutation。
3. 执行 T5 fault recovery；每个 fault 使用新 task/idempotency key，不能重用失败 state。
4. 执行 T6 sidecar lifecycle 和 T7 mixed load。
5. 每个 Task 都必须由独立 deterministic oracle 或 out-of-band verifier 结束。

**Exit：** 每个 scenario 的完整 denominator、stage correlation、Effect、Verification、cleanup
齐全；缺失任一 authority stage 时不得写“端到端完成”。

### Phase 5：A/B/C/D confirmatory campaign

1. 使用 pilot 方差做 power analysis，冻结正式 N。
2. 冻结 A/B/C/D manifest delta 和 arm randomization。
3. 执行 W1/W2 paired tasks；禁止看到结果后追加样本至显著。
4. 独立分析程序生成 CI、effect size、tail 和 safety table。
5. `performance-policy.mjs`、JSON schema 和 independent reviewer 三重校验。

**Exit：** 只按实际门槛写 `hypothesis`、`non_inferiority` 或 `significant_benefit`。

### Phase 6：cleanup 与归档

1. 停止 campaign daemon/sidecar/Pi，确认无 orphan/stale lock/socket。
2. 清理 disposable worktree、XDG root、temporary secret file 和 campaign SecretStore entry。
3. 不操作 owner 的 Desktop 原文件。
4. raw payload 保存在 ignored `artifacts/performance/<run-id>/` 或批准的外部 evidence store；
   Git 只保留 redacted report、digest、attestation reference 和 non-claims。
5. 记录所有 `pass/fail/not-run/not_available`；不得把未执行项省略。

## 11. Evidence 目录与报告格式

建议 ignored 目录：

```text
artifacts/performance/<run-id>/
  manifest.json
  environment.json
  task-set.json
  raw/
    module-*.json
    stage-*.json
    task-*.jsonl
    resource-*.jsonl
  redacted/
    safety-summary.json
    statistical-summary.json
    performance-report.json
    performance-report.md
  digests.sha256
  cleanup.json
```

最终 machine report 必须符合
`specs/schemas/performance-report.schema.json`，至少包含：

- `benchmark_manifest`
- `slo_profile`
- `metrics`，每项有 numerator、denominator、window、p50/p95/p99、sample count、95% CI
- `safety_failures`
- `governance_overhead`
- `tail_latency_disclosed: true`
- A/B 或 A/B/C/D 时的 `comparison`

Markdown report 至少包含：

1. revision/environment/model/task-set；
2. claim level 和 non-claims；
3. denominator 与失败分类；
4. module/stage/end-to-end/tail/resource/cost tables；
5. safety and recovery results；
6. A/B/C/D 与 ablation；
7. known measurement limitations；
8. cleanup 与 secret scan；
9. raw/redacted report digest；
10. independent verifier/reviewer disposition。

## 12. 执行状态矩阵模板

| Suite | Status | Revision | Environment | Denominator | Result digest | Claim ceiling |
|---|---|---|---|---:|---|---|
| L1 continuity modules | not-run | — | — | 0 | — | hypothesis |
| L1 scale matrix | not-run | — | — | 0 | — | hypothesis |
| L2 governed stages | not-run | — | — | 0 | — | hypothesis |
| L2 transport/store | not-run | — | — | 0 | — | hypothesis |
| L3 DeepSeek proxy | not-run | — | — | 0 | — | tested-local route |
| L3 Pi first response | not-run | — | — | 0 | — | tested-local route |
| L4 Task scenarios | not-run | — | — | 0 | — | scenario-limited |
| L4 fault/recovery | not-run | — | — | 0 | — | scenario-limited |
| L4 soak | not-run | — | — | 0 | — | implementation evidence |
| L5 W1 A/B/C/D | not-run | — | — | 0 | — | contract-dependent |
| L5 W2 A/B/C/D | not-run | — | — | 0 | — | contract-dependent |

## 13. 风险与控制

| 风险 | 后果 | 控制 |
|---|---|---|
| DeepSeek key 进入日志/argv | 不可逆 secret 泄露 | hidden TTY、SecretStore、redaction、secret scan、禁止 direct Pi injection |
| Provider model 漂移 | arms 不可比 | 同窗口 randomized blocks、snapshot digest、无法 pin 时 hypothesis only |
| 非流式路径伪造 TTFT | 错误结论 | TTFT=`not_available`，只测 complete response |
| token usage 被 0 占位污染 | 虚假成本收益 | 增加 usage collector；缺失写 not_available |
| fixture 时间当真实 Task | 夸大性能 | L1/L2 与 L3/L4 分表，端到端必须有 authority trace + verifier |
| Effect stage 聚合误导 async 决策 | 错误架构迁移 | 新增 transport-only timing；保留 P9-T01 conservative decision |
| floating CI 作为硬件地板 | 不稳定 gate | fixed-native only；CI correctness only |
| warm/cold 不对称 | benchmark gaming | arm 内对称 cache policy、独立 strata |
| timeout/拒绝被删除 | 虚高成功率 | 完整 denominator，失败分类不可删除 |
| benchmark runner 成为第二 writer | 破坏 A1 | runner 只调用 daemon/service；不得直接推进 authority state |
| soak 消耗失控 | Provider 费用/限流 | 每阶段预算、硬调用上限、先 8 h 后 24 h、Provider cadence 限制 |

## 14. 建议实施优先级

### P0：先做，否则不能诚实执行 L4/L5

1. task-scoped performance campaign runner；
2. Provider usage `not_available`/真实数值语义；
3. transport-only stage timing；
4. OS resource sampler；
5. redacted correlation/evidence exporter；
6. A arm approved secret broker 设计。

### P1：建立当前版本工程基线

1. 原有七项 release-mode fixed-native baseline；
2. Context/Memory/scheduler/CAS scale matrix；
3. loopback/watch/sidecar/store stage matrix；
4. DeepSeek proxy 和 Pi first-response 30-sample route baseline；
5. T1、T3、T5 三个优先 vertical scenarios。

### P2：完整产品性能

1. T2 software repair；
2. T4 Context benefit；
3. T6/T7 lifecycle + mixed load；
4. 8 h/24 h soak；
5. W1/W2 A/B/C/D confirmatory campaign。

## 15. 最终结论

当前代码已足以立即设计并执行可信的 deterministic baseline、governed-stage observation、
DeepSeek proxy smoke 和 Pi first-response campaign；但要生成“全面端到端任务性能”和
“Agent 收益”说明，还必须先补齐统一 Task timing runner、Provider usage、transport 分解、
OS resource sampler 和安全 A arm。

因此推荐的第一份实际性能报告应限定为：

> fixed-native implementation performance baseline + real DeepSeek/Pi route observation +
> scenario-limited governed Task evidence；不作 Gate、release、Profile 或 generalized
> Agent-benefit claim。

只有在 W1/W2 四臂、power analysis、complete denominator、independent verifier、tail latency、
cost 和全部 safety counters 均完整后，才评估是否从 `hypothesis` 升格为
`non_inferiority` 或 `significant_benefit`。

## 16. 参考实现与合同

- `crates/cognitive-runtime/src/bin/p7_t04_module_benchmark.rs`
- `crates/cognitive-runtime/src/perf.rs`
- `crates/cognitive-runtime/src/bin/p9_t01_async_decision_gate.rs`
- `crates/cognitive-runtime/src/store_access.rs`
- `apps/kernel-server/src/personal/server.rs`
- `apps/kernel-server/src/personal/provider_proxy.rs`
- `apps/kernel-server/src/personal/task_api.rs`
- `apps/kernel-server/src/personal/pi_runtime.rs`
- `packages/pi-cognitiveos/src/daemon-provider.ts`
- `tools/personal/p1-t09-provider-proxy-smoke.mjs`
- `tools/personal/p1-t09-product-route-smoke.sh`
- `tools/src/ucr-runner.mjs`
- `tools/src/performance-policy.mjs`
- `specs/schemas/performance-report.schema.json`
- `docs/evaluation/agent-benefit-benchmark.md`
- `docs/evaluation/personal-unified-cognitive-resource-workload.md`
- `docs/plan/PERSONAL-TEST-ENVIRONMENTS.md`
- `docs/checkpoints/20260810-personal-p7-t04-performance-governance-closure.md`
- `docs/checkpoints/20260811-personal-p9-t01-async-decision-gate-closure.md`
- `docs/checkpoints/20260811-personal-p9-t03-store-composition-closure.md`
