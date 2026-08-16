# CognitiveOS Personal performance benchmark readiness closure plan

- Status: proposed execution plan
- Owner direction: close every required product and environment prerequisite
  before executing the complete performance benchmark
- Parent contract:
  [personal-performance-benchmark-execution-plan.md](personal-performance-benchmark-execution-plan.md)
- C1/C2 contract:
  [personal-c1-c2-benchmark-execution-plan.md](personal-c1-c2-benchmark-execution-plan.md)
- Current full-OS disposition:
  [personal-performance-benchmark-full-os-only-addendum.md](personal-performance-benchmark-full-os-only-addendum.md)
- Target execution environment: `B01-DESKTOP-002` / `B01-Desktop-Linux-002`
  (简称 `linux-002`)
- Host route: `wuz@192.168.1.2` (`hal9000`, system libvirt) -> ProxyJump ->
  `hal9001@192.168.123.160`
- Local Windows testing: prohibited for this plan
- Claim ceiling: implementation evidence while closing product gaps;
  `hypothesis` / non-claim during the benchmark until independent review
- Change class of this document: informative execution planning; it does not
  create task status, a lease, a Gate result, or benchmark evidence

## 1. Outcome

本计划的完成条件不是“把当前不可执行项写成 `not-run`”，而是同时做到：

1. 补全 parent benchmark 及 full-OS addendum 所需的真实产品能力；
2. 每条能力都有 public/authenticated product caller、daemon authority path、独立
   verifier/acceptance 和 bounded redacted observation；
3. 所有产品切片均在已推送的精确 revision 上，于 `linux-002` 执行完整正例、负例、
   restart/reconcile、资源与 cleanup 验证；
4. 为 pure Pi 与 governed OS arm 建立公平、固定、可复验的 campaign assets；
5. 重新冻结 `PERSONAL-PERF-EVAL-004`，使 B0 可以诚实判定为 pass，并允许进入 B1；
6. 按 parent plan 完整执行 B1、B2、B3、B4、B5、cleanup、独立分析和最终报告。

`B6` 仍是优化后的 replay，不是首次完整 benchmark 的前置条件。24 h soak 仍遵守 parent
plan 的 conditional 规则：只有 8 h 留下未决 slope 且 owner budget 允许时执行。

## 2. 不可改变的边界

- Rust daemon 仍是唯一 authority writer；Pi、runner、CLI、fixture 和 sidecar 只产生
  candidate、调用 public surface 或读取 redacted projection。
- Search/Write/Patch 参数必须由 daemon canonicalize、重算 digest 并绑定 governed
  Intent；runner 不得写 SQLite、调用 test-only seam 或注入私有 transport。
- 所有 external mutation 必须 persist-before-dispatch，使用原 idempotency key
  reconcile，并在独立 verification 和 acceptance 前保持 Task 未完成。
- Provider secret 只进入 guest approved SecretStore/non-logging input；不得进入 argv、
  environment、普通配置、SQLite、日志、测试输出、evidence 或聊天。
- `linux-002` 是隔离 campaign guest。产品能力验证与正式 performance measurement 必须
  使用不同 lease、不同根目录、不同端口和不同 evidence denominator。
- 产品验证不能写成 EVAL 结果；EVAL campaign 不能为了让 cell 可运行而修改产品代码。
- Windows 本机、Windows GNU、Windows MSVC CI、WSL 和普通 Windows VM 均不属于本计划
  的测试路径。若现有 formal acceptance 或 branch protection 强制 Windows job，必须先
  用 owner-approved product-semantic/CI-policy amendment 将本计划任务的 required
  validation 改为 `linux-002`；不得跳过门禁，也不得把 Windows `not-run` 写成 pass。
- `B01-Clean-Linux-001` 始终禁止接触、恢复、启动、重置、部署或删除。

## 3. 当前基线与缺口总表

当前 `PERSONAL-PERF-EVAL-004` 在 B0 后暂停，尚无 B1/B2/B3/B4 sample。目标 guest、
isolated root、reserved port、Pi 0.81.1、Extension、daemon/CLI 和 SecretStore readiness
已部分资格化；既有 EVAL-004 asset 和 SecretStore entry 不得自动继承到新 revision。

| 范围 | 当前事实 | 必须闭合的产品或环境缺口 |
|---|---|---|
| C0 / P-O route | EVAL-002 已证明 route 可测 | 为新 revision 重新冻结 broker、runner、corpus、oracle、redactor |
| C1 Read/Search | Read 有真实受治理路径；Search 参数 carrier 已开始修复 | 完成 P2-T21 D02；真实 admitted Task 必须到达 Search sink 并完成验证/验收 |
| C2a Write/Patch/check | router/sink/参数 carrier 基础存在 | 完成 P2-T21；补产品级 write/patch/check journey、hidden test、diff Artifact 和 terminal evidence |
| C2b Memory/Skill | authority、生命周期和 daemon-private consumer 存在 | 增加 public task-bound consumption journey、session-2 resume 和 durable redacted use evidence |
| C2c unknown outcome | durable original-key fixture/实现基础存在 | 增加 public/default-off fault campaign caller、Effect/reconcile history 和 restart query surface |
| C2d completion | verifier 和 acceptance authority 已接线 | 完成 P2-T21 D03 durable terminal evidence；公开证明 current report、CAS 和 acceptance |
| Tool T1/T2/T3 | catalog/dynamic lifecycle/selection 基础存在 | 将 lifecycle、selection 和 exposure 连接到真实 Agent/task caller与 observation |
| Tool T4/T5 | sink 与 carrier 基础存在 | 完成真实 Search/Write/Patch production-chain positives 和 negatives |
| Tool T6 | ProcessCheck 目前正向 production observation 不完整 | 建立 daemon-owned immutable check registry 的成功路径；禁止通用 ProcessRun |
| Tool T7 | HTTP executor 有空 allowlist | 建立 campaign-scoped pinned HTTPS origin registration 和只读成功路径 |
| Tool T8/T9 | denial/reconcile 基础存在 | 公开 descriptor-drift denial 与 original-key reconcile 的 redacted evidence |
| O2 | authorization/reauthorization 内部存在 | public redacted decision receipt，不暴露 Context body/capability secret |
| O3 | cache/compaction 内部存在 | public bounded cache/compaction observation 与主动 negative control |
| O4 | scheduler/budget/fencing production path 存在 | queue/fairness/starvation/fence telemetry 与多 runnable Task workload |
| O5 | Intent/Effect production path 存在 | authenticated task-bound Effect history、reconcile class 和 stage timing |
| O6 | verifier/acceptance production path 存在 | durable terminal evidence 和 mechanical completion collector |
| O7/O8 | Memory API/authority 存在 | task consumption evidence、forget non-resurrection 和 session-2 journey |
| O9 | dynamic Tool implementation 存在 | public lifecycle→projection→Agent exposure propagation journey |
| O10 | managed Pi lifecycle 存在 | 当前 revision 的独立 lifecycle procedure 与 redacted outcome collection |
| O11/O12 | projection/SecretStore public surface 存在 | 新 revision B0 重跑和 active negative controls |
| O13 | bounded replay 存在、full audit 内部化 | task-bound durable audit export/replay consistency surface |
| O14 | backup internals/计划存在，用户路径真值有漂移 | 核对并补齐 `cognitive backup/restore` 的真实 archive、preflight、restore、RTO/RPO 和 cleanup |
| UJ1-UJ6 | 多数只有历史 evidence 或 register | 在新 revision 逐项执行；UJ5 归 B3，UJ6 不得漏项 |
| Capacity/soak | 历史 EVAL-002 可参考但不可继承 | 新 revision 的 resource sampler、B4 profiles、B5 1 h/8 h 与条件式 24 h |

## 4. 治理与任务注册

产品能力不能在一个超大 branch 中堆叠。实施前必须把下列 work package 注册为正式
Personal task（或经正式计划确认合并进已有 task），每个 task 使用一个 branch、一个 Draft
PR 和一个 task lease。这里的 `BR-*` 是本计划的 work-package ID，不是新的正式 `P*-T*`
状态源。

注册批必须同步：

- `PERSONAL-DEVELOPMENT-PLAN.md`：正式 task、typed dependencies、acceptance、Delivery
  Slices 和 Linux-only validation route；
- `PROGRESS.md` Current snapshot：当前 task/slice 和唯一下一动作；
- `PARALLEL-LANES.md`：一个 active task lease；
- `personal-trace.yaml`：PERS-PR/benchmark mapping；
- matching product/architecture/handbook pages；
- CI policy：本计划不启动 Windows test job，required product evidence 绑定 exact
  `linux-002` revision。

Owner 2026-08-15 已批准并登记以下正式 crosswalk；`BR-*` 继续只作为本文件中的
informative work-package ID：

| Work package | Formal task | Initial status |
|---|---|---|
| BR-01 | P2-T21 | done |
| BR-02 | P2-T22 | done |
| BR-03 | P2-T23 | done |
| BR-04 | P2-T24 | done (merged PR #223 at `main@2b803e0f`) |
| BR-05 | P2-T25 | done (merged PR #224 at `main@4b10db9a`) |
| BR-06 | P2-T26 | done (merged PR #225 at `main@9e1404a1`) |
| BR-07 | P2-T27 | done (merged PR [#226](https://github.com/agentkernel/cognitive-os/pull/226) at `main@70980131`) |
| BR-08 | P2-T28 | in-progress (D01 UJ1..UJ6 capability-truth freeze) |

CI route resolution is fail closed by task branch: P2-T21..P2-T28 run Ubuntu
supporting CI and require an append-only report from the exact pushed revision
on `linux-002`; Windows is recorded `not-run by owner-directed Linux-only
route`. Other tasks retain the default dual-platform matrix. The stable
`required-ci` aggregate may only pass when every job selected by the resolved
route passes; it does not substitute for `linux-002` evidence.

任何 task 都不能用“benchmark 需要”为理由放宽 schema、negative、authority、secret、
Effect 或 verifier 语义。

## 5. 产品能力闭合工作包

### BR-01 — 完成 P2-T21：参数、真实 caller 与终态证据

**依赖：** 已完成的 P2-T20 与当前 P2-T21/D01。

**实施：**

1. D02 贯通 public Task admission → private candidate → daemon canonicalization → governed
   Intent → scheduler → production router → WorkspaceSearch/Write/Patch sink。
2. 保留 containment/no-follow、preimage CAS、atomic publish、persist-before-dispatch、
   original-key reconcile、idempotency、budget 和 fencing。
3. D03 增加 authenticated task-channel read-only terminal evidence endpoint 与 CLI caller；
   从 authority/CAS rehydrate，不依赖进程内 watch log。
4. 输出仅含 bounded lifecycle、opaque refs、reconcile class、verification/acceptance
   refs/digests/currentness、terminal transition 和 cursor。

**Linux-002 exit：** Search/Write/Patch 各至少一个真实正例；missing/malformed/oversize/
digest mismatch/family mismatch/stale epoch/preimage drift/cross-task/wrong-channel/restart/
missing CAS 全部 fail closed；Task 仅在 current verification + acceptance 后完成。

### BR-02 — C2a 完整软件修复 journey

**Status (2026-08-16):** D01–D03 complete at `4a803070` / Draft PR #221.
Ubuntu required CI run `31919267639` green; exact-revision `DEV-LINUX-NATIVE-01`
workspace tests 0-failed, Clippy `-D warnings`, fmt. Ready/merge remains.
No EVAL/Gate/release/Profile claim.

**实施：**

1. 建立固定、无 secret、可 reset 的小型 repo corpus。
2. governed Agent 可执行 read/search → write/patch → `RegisteredCheckRun` → diff Artifact →
   independent verifier → acceptance。
3. daemon-owned check registry 固定 executable identity、argv template、cwd、最小环境、
   timeout、write roots、network=deny 和 descriptor digest。
4. public terminal evidence 绑定 mutation set、hidden-test result digest、diff Artifact、
   Effect closure 和 acceptance；不返回 source/body/raw output。

**Linux-002 exit：** 至少覆盖 TypeScript 与 Rust fixture family；hidden tests、越界写、
test weakening、descriptor drift、timeout、orphan、oversize、exit-0-without-verification、
restart unknown outcome 全部有 focused evidence。

### BR-03 — C2b Memory/Skill 真实消费与 session-2 恢复

**Status (2026-08-16):** D01–D03 complete at `79764387` / Draft PR #222.
Ubuntu supporting CI run `31922660543` green; exact-revision `DEV-LINUX-NATIVE-01`
kernel-server bin 299/299, p4_t05 5/5, workspace tests 0-failed, Clippy
`-D warnings`, fmt. Ready/merge remains. No EVAL/Gate/release/Profile claim.

**实施：**

1. 提供 task-bound public journey：remember/review/forget；import/inspect/bind/supersede/
   revoke。
2. Agent 的 Context 构建必须在 rank/body load 前完成 scope、purpose、epoch、lifecycle、
   exact digest/pin 授权。
3. 增加 durable redacted consumption record：task/request/context refs、selected revision
   digest、decision class、session/resume linkage；不返回 Memory/Skill body。
4. session 2 通过 durable state 恢复，不重放 chat、不要求用户重述 procedure。

**Linux-002 exit：** required recall=100%、user restatement=0、cross-scope/stale/revoked/
forgotten exposure=0；revoked reuse、digest drift、manual prompt forgery、partial residue 和
restart recovery negatives 全通过。

### BR-04 — C2c Effect/reconcile 与 fault observation

**Status (2026-08-16):** D01–D03 complete at `8b7dea7a` / Draft PR #223.
Ubuntu supporting CI run `31925698730` green; exact-revision `DEV-LINUX-NATIVE-01`
kernel-server bin 309/309, `p2_t24_d02` 3/3, `fault_profile` 5/5, P2-T17 15/15,
workspace tests 0-failed, Clippy `-D warnings`, fmt. Ready/merge remains. No
EVAL/Gate/release/Profile claim.

**实施：**

1. 产品提供 default-off、task/campaign-authorized 的 fault profile；普通用户不能启用。
2. 固定 fault points：dispatch-before、mutation-after/receipt-before、receipt-after/
   Effect-close-before、verification-before。
3. task-channel 提供 bounded Effect history：opaque original-key ref/digest、stage、outcome
   class、reconcile class/timing、mutation count、fixed-post-state/report refs。
4. restart 后只能按 original key query/reconcile；禁止 replacement key 和 blind retry。

**Linux-002 exit：** 每个 fault point 都证明 mutation count 恰好一次或明确未发生；
ambiguous outcome 保持 Indeterminate；open/unknown Effect 永不完成 Task；cleanup residue=0。

### BR-05 — Tool 全生命周期与实际调用

**实施：**

1. T1/T2/O9：公开 registered/enabled/execution-ready/quarantined/revoked 投影与 lifecycle
   mutation，Agent exposure 随 lifecycle 原子更新。
2. T3：记录 bounded Tool selection decision（candidate set digest、selected descriptor、
   denial/selection class），不得记录 prompt/body。
3. T6：只增加 immutable `RegisteredCheckRun` 成功 registry，不增加通用 `ProcessRun`、
   任意 argv/env/cwd/shell 或 credential capability。
4. T7：增加 task/campaign-scoped、pinned HTTPS origin registry；仅 GET/HEAD、Rustls、
   no redirect、no credentials、no inherited proxy、bounded response。
5. T8/T9：公开 descriptor drift denial 和 original-key reconcile 的 redacted evidence。

**Linux-002 exit：** enable→call→disable/quarantine/revoke propagation、最窄 Tool selection、
registered check success/timeout、pinned HTTPS success/redirect/oversize/credential URL denial、
duplicate dispatch 和 restart reconcile 全部通过。

### BR-06 — O2/O3/O4/O5/O13 OS observation plane

这是只读、bounded、authenticated 的观测面，不是第二 authority API。

**实施：**

- O2：authorization decision receipt，包含 scope/purpose/epoch/input digest、decision class
  和 redacted reason code；不包含 Context body 或 capability material。
- O3：cache hit/miss/revalidated/evicted、compaction input/output token/byte counts、loss
  manifest digest、stable-prefix facts；提供 active negative control，禁止默认 0。
- O4：queue wait、lease acquisition、runnable count、budget stop、stale-fence denial、
  starvation/fairness counters；绑定 task/epoch 和采样窗口。
- O5：复用 BR-04 的 Intent/Effect history，不暴露 raw parameter/receipt。
- O13：durable audit export cursor、event digest chain、bounded replay result 和 gap
  detection；跨 restart 保持一致。

**Linux-002 exit：** 每个 `observed_zero` 都有 collector、适用 denominator 和 negative
control；跨 scope/channel、stale cursor、oversize、missing event、digest break、restart
replay、concurrency 负例全通过。

### BR-07 — O10/O14 用户生命周期与 backup/restore

**实施：**

1. 对 P7-T02 的正式 acceptance 与当前 public surface 做真值核对；若 `cognitive backup`
   / `restore` 不可调用，补齐真实 CLI/API，而不是修改评测计划迁就实现。
2. backup 包含 Memory、Skill、bindings、必要 metadata/digests/migration version；永远排除
   secret、bearer、raw Provider/Pi 内容和普通 authority SQLite copy。
3. restore 先做 schema/version/digest/preflight，再以事务方式恢复；失败不留 partial state。
4. 执行 managed Pi install/register/activate/pause/resume/upgrade/rollback/stop/uninstall/
   recover 的独立 current-revision procedure，并记录 redacted identity/outcome。

**Linux-002 exit：** fresh root backup→destroy test state→restore→independent equality、
cross-version migration、tamper、missing part、secret exclusion、rollback、RTO/RPO、cleanup；
Pi lifecycle 每一状态转移和 recover/orphan negative 均通过。

### BR-08 — 全链用户旅程与 capability truth 收口

**实施：**

1. UJ1：install/init/first response。
2. UJ2：cold/warm conversation，使用完整 nested timing。
3. UJ3：status/doctor/resource/Task operations 和 restart 后 bounded replay。
4. UJ4：Task admission、真实执行、durable terminal query 分开计时。
5. UJ5：并入 B3，覆盖 daemon/Pi kill、deadline、restart、cleanup。
6. UJ6：Memory、Skill、read/search、write/patch/check、Pi lifecycle、backup/restore、
   verified completion 全部必须有 executable disposition。

Web UI 和 Multi-Agent 不进入本 benchmark 的 required arm：parent plan 明确将它们列为
deferred capability truth。最终 UJ6 仍保留这些行并标明 product scope，但不允许它们阻塞
本计划的 B0。若 owner 要求把它们也纳入 benchmark，则必须先扩大正式 scope，完成
P7-T05、P6-T01..T04/B11，并重新设计 arm/corpus/denominator；不得在本计划中静默加入。

**Linux-002 exit：** 除明确 scope-excluded 的 Web UI/Multi-Agent 外，UJ6 每行均有真实
product caller、mechanical oracle、cleanup 和 evidence schema。

## 6. Linux-002 产品验证环境

### 6.1 隔离与 lease

产品能力验证使用单独的 preregistered procedure，例如：

- environment: `B01-DESKTOP-002` / guest `B01-Desktop-Linux-002`；
- purpose: exact-revision product capability validation，非 B01、非 EVAL sample；
- root: `/home/hal9001/cos-product-validation/<task-id>/<full-revision>`，mode `0700`；
- endpoint: 为每个 task 预留独立 loopback port，不使用 `48181`、`48282` 或 `48284`；
- allowed operations: user-local toolchain、build/test/runtime、fixture reset、cleanup；
- prohibited: snapshot revert/delete、system package/global install、P9-T04/EVAL roots、
  retired guest、owner secret file、系统级共享配置。

每个 task lease 必须在 procedure 中登记 guest contact、root、port、revision、expected
artifacts、cleanup 和 evidence ceiling。不得在 EVAL lease 下测试未合并产品代码。

### 6.2 User-local toolchain

`linux-002` 当前没有 Git/Rust/Cargo/pnpm。完整测试要求在隔离根内固定并验证：

- clean source archive produced from already pushed full revision；
- Rust `1.97.1` + rustfmt + Clippy；
- Node `>=22.19.0`；
- pnpm `10.33.2`；
- Pi `0.81.1` tarball/SRI/source commit；
- Extension、sidecar、test fixture、oracle 和 scanner digest；
- guest kernel/glibc/CPU/RAM/governor/background load。

优先使用可复验的 user-local archives。每个 archive 在 host 与 guest 双端核对 SHA-256；
不修改系统 PATH，执行时使用隔离根绝对路径。若构建必须在 `hal9000` 完成，测试 binary、
source archive、build manifest 和 dependency digest 必须一起传入；最终行为、集成、fault、
restart、resource 和 cleanup 测试仍必须在 `linux-002` 执行。

### 6.3 每个产品 task 的 required test set

1. exact revision/dirty=false/archive digest；
2. focused failure-first and negative suite；
3. affected package/workspace Rust tests；
4. Clippy、fmt、contract/codegen/consistency/handbook drift；
5. real public HTTP/CLI/task-channel integration；
6. restart、duplicate、stale epoch、unknown outcome、redaction、cross-channel；
7. resource bounds、process/socket/temp/artifact residue；
8. secret scan；
9. cleanup and cold restart；
10. one append-only task validation report, updated after every completed unit。

Windows validation 一律记为 `not-run by owner-directed Linux-only route`，不得计入 acceptance
evidence。正式计划和 branch protection 必须在首个 BR task 开始前与此路由一致。

## 7. EVAL-004 重新冻结与 B0 completion

所有 BR task 合并、lease 关闭、main clean 后：

1. 关闭产品验证 roots/processes/ports，验证 `linux-002` 无 residue。
2. 新建 owner-directed EVAL-004 evaluation lease；measurement-only writable paths 仍仅限
   `docs/evaluation/`、`docs/checkpoints/`、`docs/plan/PROGRESS.md`。
3. 从最新 clean `origin/main` 生成 source archive；记录 full revision、artifact/SBOM、
   daemon/CLI/Extension/sidecar digests。
4. 新建全新 campaign root 和全新 loopback port；不得复用旧 EVAL-004 runtime state、
   SecretStore entry、broker、runner、corpus 或 evidence denominator。
5. 由 owner-approved hidden input/non-logging path 创建 campaign SecretStore entry；公开
   doctor 只记录 redacted readiness。
6. 冻结 pure-Pi broker：loopback-only、single-user、memory-only key handling、无 header/
   body log、固定 non-secret local token、bounded response、cleanup。
7. 冻结 equivalent fixture adapters：P/O 的 workspace bytes、Tool schema、network、budget、
   model parameters、timeout、retry=0 和 oracle 必须相同。
8. 冻结 corpus、hidden oracle、seed、arm order、runner、analysis、redactor、resource sampler
   和 cleanup digests。
9. independent reviewer 在 B0 前检查 fairness、secret、denominator、hard counters、
   measurement-only boundary 和 negative controls。
10. 执行 B0：每 arm 三次 non-counted warmup，每 family 一次 qualification sample；所有
    started outcomes 保留；secret/redaction/reset/cleanup 全通过。

### B0 Go 条件

只有以下项目全部满足才允许 B1：

- C0、C1、C2a、C2b、C2c、C2d 的 required arms 均可从 public product surface 启动；
- P/O equivalent tools、input bytes、workspace、model、budget、timeout、oracle 完全一致；
- governed arm 的 Effect、verification、acceptance 与 terminal evidence 可复验；
- O2-O14 required observation 具备 collector 与 negative control；
- broker/runner/corpus/oracle/redactor/analysis/cleanup 全部 digest-frozen；
- Provider/Pi ready，secret scan=0，scenario boundary violation=0；
- reset 后 workspace/authority/campaign root digest 返回预期；
- independent reviewer disposition=`approved-for-B1`。

任一项失败，立即将 B0 写入运行报告并停在 B0；不得边跑 B1 边补产品能力或修改 oracle。

## 8. 完整 benchmark 执行

### Phase E1 — B1 pilot

- 每个通过 B0 的 family 5 seeds；P/O 各 2 次；pilot seeds 不进入 confirmatory。
- 验证 instrumentation、timeout、failure taxonomy、Provider window 和 cleanup。
- 只允许调整 preregistered timeout/sample-size 参数；不得按效果方向调 corpus/oracle。
- 输出 power analysis 和冻结的 B2 N/randomization。

### Phase E2 — B2 confirmatory

- 使用 held-out seeds；每 capability class 至少 30 paired task-seeds；无 deterministic
  Provider seed 时每 arm 三次。
- 每 10 blocks 检查 model snapshot、thermal/load、rate-limit 和 environment drift。
- headline 先报告 complete denominator 与 completion，再报告 completed-pair efficiency。
- C1/C2d 的 OS Task completion 只认 daemon acceptance authority；pure Pi 使用外部 mechanical
  oracle，不把两者混成 authority 等价声明。

### Phase E3 — B3 faults/restart/safety

- 每个 applicable fault × arm 固定 10 seeds。
- 覆盖 deadline、daemon/broker unavailable、model mismatch、Provider timeout、Pi kill、
  response cap、stale epoch、descriptor drift、preimage mismatch、duplicate dispatch、
  OUTCOME_UNKNOWN 和 verification-before fault。
- 10 次 daemon stop/start；每次检查 orphan/socket/lock/FD/RSS/temp/artifact residue。

### Phase E4 — B4 concurrency

- concurrency 1/8/16；overload 17 in-flight / 33 connections。
- 每 profile 100 local observations；Provider Agent tasks 仅按冻结 budget 运行。
- mix 包含 Pi task、status/doctor、six-resource get/watch、Task watch/terminal query、
  Memory/Skill、Tool/Effect observation。
- 报告 throughput、queue/fairness、p50/p95/p99（样本量允许时）、CPU/RSS/FD/thread、
  cursor completeness 和 recovery time。

### Phase E5 — B5 soak

- 1 h 必跑；通过且无 leak/safety anomaly 后运行 8 h。
- 24 h 仅在 8 h 留下 unresolved slope 且 owner budget 允许时运行。
- 每分钟 local read/watch/observation；每小时 cold restart；paired block 频率遵守 parent
  plan；记录 RSS/FD/WAL/I/O slope、Provider denominator、orphan、socket、lock 和 secret scan。

### Phase E6 — cleanup、analysis、report

1. 停止 daemon/Pi/broker/sampler/runner；清除 campaign-created SecretStore entry。
2. 校验 process/socket/lock/temp/workspace/authority residue 和 scenario boundary。
3. 对 raw evidence 建 digest、retention locator/deadline；Git 只保存 redacted aggregate、
   digest、attestation、non-claims 和 final report。
4. independent analysis 使用 parent plan 的 paired statistics、cluster bootstrap、McNemar、
   Holm correction 和 non-inferiority rules。
5. 最终报告分开 route、general Agent、software/operations、Skill、Tool、OS-unique、
   authority、reliability、capacity、soak、safety、capability truth 和 non-claims。

## 9. 完成判定

本计划只有在以下全部成立时完成：

- BR-01..BR-08 所需正式 Personal tasks 均为 `done`，task PR/lease/branch/main 全收口；
- 所有 required product capability tests 均在 exact-revision `linux-002` 实际通过；
- Windows tests 全部未运行且正式 validation policy 已诚实对齐；
- EVAL-004 fresh B0 pass，B1/B2/B3/B4/B5 按 preregistration 执行并逐单元记录；
- 所有 started samples retained，所有 fail/partial/not-run/instrument_error 有明确原因；
- hard safety counters 全部满足 target，secret scan 和 cleanup pass；
- final report、raw evidence locator/digests、independent reviewer disposition 和 capability
  matrix 完整；
- 未产生 Gate、release、Profile、B01 或 general Agent benefit 的越界声明。

若某一产品能力经 owner 明确从 benchmark scope 排除，必须先同步 parent plan、full-OS
addendum、corpus/denominator 和本计划，并重新 preregister；不能在执行中把失败项临时改成
“非适用”。
