# CognitiveOS Personal 下一批次开发分析报告

- Status: informative analysis report（非规范源、非计划源、非任务台账）
- Date: 2026-08-12
- Scope: 结合未来 Agent OS 形态、用户交互、真实使用场景和当前实现事实，分析下一批次最值得补充的能力
- Current fact owner: `docs/plan/PROGRESS.md` `Current snapshot`
- Task/Gate owner: `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md`
- Safety owner: `docs/governance/AXIOMS.md`

> 本文只提供分析和建议，不创建任务、REQ、Gate、lease、release 或 Profile 声明。
> 正式采纳时必须在 Personal 正式计划中登记，并按任务 lease、branch、Draft PR 和
> supported validation 流程交付。

---

## 0. 执行摘要

CognitiveOS Personal 下一批最应该建设的，不是 Web UI、Multi-Agent、更多协议或更多
治理抽象，而是打通一条用户真正可以使用的完整路径：

```text
用户目标
  → daemon 持久化原始意图
  → Agent 产生解释候选
  → daemon 签发精确 preview
  → 用户一次授权
  → 持续、多步、受预算约束的真实 Tool 执行
  → 持久 Activity 与全局 Overview
  → Artifact / Evidence
  → 独立 verifier
  → Task 完成、继续、阻塞或恢复
```

当前系统已经拥有大量正确的 OS 原语：daemon-only authority、Context、预算、scheduler
lease、Intent/Effect、fencing、Artifact CAS、Memory/Skill、独立 verifier 和恢复规则。
但最新实现评估明确记录：

- admitted Task 尚未自动进入持续 scheduler；
- production Tool dispatch 尚未接入真实用户路径；
- production independent verification/acceptance 尚未接线；
- 已执行真实 Task admission 场景的 verified completion 为 0；
- 当前首要问题是**能力不可达**，而不是局部性能不够快。

因此，本报告建议下一批次定义为：

> **Useful Task Loop & Control Surface：实用任务闭环与控制面。**

优先级为：

1. 真实任务执行闭环；
2. Tool 与受控检查能力；
3. 全局 Overview 和 Task Activity；
4. Agent Shell / CLI 的真实用户入口；
5. exact-revision 场景验证；
6. 之后才进入跨 Agent、Web UI 和 Multi-Agent。

---

## 1. 分析依据与当前事实

### 1.1 当前已经成立的能力

根据 `docs/plan/PROGRESS.md` Current snapshot，当前已经具备：

- Task raw intent、interpretation candidate、preview、admission 和 watch 原语；
- durable scheduler row、lease、epoch、CAS fencing、budget ceiling 和 STOP-before-lease；
- ContextRequest/ContextView、authorization-before-ranking、stable prefix/delta 和缓存；
- WorkspaceRead、ProcessCheck 的 executor 原语；
- Intent/Effect persist-before-dispatch、original-key idempotency 和
  `OUTCOME_UNKNOWN` reconcile；
- Artifact CAS；
- append-only verification request/report 和独立 verifier seam；
- Memory admission、FTS5、版本、冲突、过期、forget/tombstone；
- Skill package/revision/import/binding/revoke；
- managed Pi package、installation、registration、sidecar 和 lifecycle；
- status、doctor、resource get/watch、Task watch 等局部客户端入口；
- Linux `/proc` 性能采样器，可采集声明进程的 CPU、RSS、线程、FD 和 I/O。

这些能力说明 CognitiveOS Personal 不是只有文档的概念项目，也不是单纯的 Pi wrapper。

### 1.2 当前最关键的断层

`docs/evaluation/personal-performance-benchmark-execution-plan.md` 的当前实现评估指出：

- Task admission 可以持久化，但不会形成完整的持续执行；
- scheduler、Tool executor、Artifact、verifier 之间没有 production composition；
- current Task watch 不能代表完整 scheduler/Effect/verification 时间线；
- 当前真实 Task 场景只能证明 admission 与边界诚实，不能证明 autonomous verified
  completion；
- P9-T04 的 Agent-benefit A/B/C/D 没有执行，不能声称比直接使用 Agent 更快、更省或
  成功率更高。

代码结构也反映了这一点：

- `apps/kernel-server/src/personal/task_api.rs` 的 watch 仍以进程生命周期内事件为主；
- `apps/kernel-server/src/personal/resource_api.rs` 的公共投影还没有完整返回六族真实对象；
- `apps/kernel-server/src/personal/tool_executor/` 中只有部分 operation family 有 executor；
- `apps/kernel-server/src/personal/verification_executor.rs` 仍缺生产调用链；
- `crates/cognitive-runtime/src/resource_sampler.rs` 目前主要服务性能 campaign，而不是
  产品 Task Manager。

---

## 2. 从第一性原理推导下一批重点

### 2.1 用户需要的不是组件，而是可信结果

用户不会因为系统拥有很多 schema、状态机和 Gate 就获得价值。用户真正需要的是：

1. 描述一个目标；
2. 知道系统将访问什么、修改什么、花费多少；
3. 授权一次明确边界；
4. 系统能够实际执行；
5. 中途可以观察、暂停、恢复和纠错；
6. 最终结果由可验证事实而不是 Agent 自述确认；
7. 有价值的经验可以在授权后复用。

如果 Task 只能 admission，不能执行、验证和完成，那么 Memory、Context、Event、UI 和
Multi-Agent 都缺少真实消费路径。

### 2.2 Agent OS 的最小闭环

一个实用 Agent OS 至少需要同时拥有：

- **持久认知进程：** Task 不依赖终端连接或一次模型会话；
- **受保护执行：** Agent 只提出 candidate，daemon 才能授权和提交；
- **真实系统调用：** Tool 必须有可执行 caller，而不只是 descriptor；
- **副作用恢复：** crash 后按原 idempotency key reconcile，不盲目重试；
- **可观察状态：** 用户能看到当前 Task、Agent、Process、Effect、budget 和 blocker；
- **独立完成判定：** Tool、Provider、进程或 Agent 的成功都不能直接完成 Task；
- **可替换 Agent：** 长期应让不同 Agent 共用同一 Task、Context、Tool 和 Evidence
  语义。

当前 CognitiveOS Personal 的保护、持久性和恢复原语较强，最薄弱的是系统调用的真实
组合、用户可观察性和用户入口。

### 2.3 用户交互将从“聊天”演进为“对话 + 后台任务”

未来 Agent 产品不会只依赖一个持续打开的聊天窗口。更合理的交互结构是：

1. 用户用自然语言表达目标；
2. Shell 展示 daemon 签发的 canonical preview；
3. 用户批准固定 digest；
4. Task 在后台持续运行；
5. Shell 可以 detach，Task 不取消；
6. 全局 Overview 显示所有受管工作；
7. Task Activity 提供单任务时间线；
8. 需要补充信息或高风险授权时产生 interrupt；
9. 用户回复后 resume；
10. verifier 决定完成，系统再通知用户。

MCP 的 Tool/Resource、A2A 的持久 Task/订阅、AG-UI 的 snapshot/delta 与
interrupt/resume 都体现了这一方向。但这些协议只能作为交互和适配参考，不能替代
CognitiveOS daemon authority、Intent/Effect 或 verifier。

---

## 3. 建议的下一批次能力

## 3.1 持续、多步的真实 Task 执行入口

目标是让 admitted Task 真正进入 durable execution，而不是停在 authority record。

需要补充：

1. Task admission 后原子创建或唤醒 scheduler work；
2. daemon 在服务运行期间持续、受预算约束地驱动 tick；
3. 区分 pre-candidate scheduling 与 post-Intent dispatch；
4. 支持一个 Task 内多个 iteration/action/Intent；
5. 每一步绑定 Task、Loop、iteration、epoch、budget 和 idempotency key；
6. restart 后按顺序 reload、fence、reconcile、reauthorize、rebuild Context，再继续或
   quarantine；
7. no-progress、budget exhaustion 和 unavailable dependency 必须进入明确 STOP、WAIT、
   BLOCK 或 ESCALATE，而不是无限循环。

主要实现面：

- `apps/kernel-server/src/personal/task_api.rs`
- `apps/kernel-server/src/personal/scheduler_authority/`
- `crates/cognitive-store/src/scheduler.rs`
- `crates/cognitive-store/src/sqlite/`

### 为什么优先

没有持续执行入口，新增 Tool、Overview 或 Shell 都只能观察和操作测试夹具，无法构成
真实用户价值。

---

## 3.2 完成真实 Tool execution parity

下一批应首先补齐已经登记但没有真实 executor 的 operation family：

1. `WorkspaceSearch`
2. `WorkspaceWrite`
3. `WorkspacePatch`
4. `HttpFetchReadOnly`

它们不能只新增独立 executor 类型，必须接入：

- production scheduler caller；
- immutable descriptor identity/digest；
- daemon-issued authorization snapshot；
- durable parameter material或 Artifact reference；
- Intent/Effect；
- budget；
- idempotency；
- fencing；
- bounded output；
- original-key reconcile；
- Artifact / Context result publication。

### WorkspaceSearch

必须覆盖：

- approved root；
- canonical path；
- traversal、absolute path、symlink/hardlink escape；
- query length、file count、match count、line length、output bytes 和 cursor 上限；
- binary/secret-shaped output 的有界处理；
- stale fence 和 duplicate key；
- read-only，mutation count 必须为 0。

### WorkspaceWrite/Patch

必须覆盖：

- expected preimage digest；
- same-directory temporary file；
- atomic publish；
- before/after Artifact；
- recovery journal；
- partial-write cleanup；
- symlink/path race；
- duplicate dispatch；
- crash after I/O before receipt；
- `OUTCOME_UNKNOWN` 从 durable journal 按原 key 对账；
- 失败时保留原始 bytes。

### HttpFetchReadOnly

必须覆盖：

- HTTPS only；
- origin 和 URL policy；
- user-info/credential-in-URL 拒绝；
- redirect revalidation；
- DNS/private/loopback/link-local target policy；
- timeout；
- header/body/解压后大小上限；
- 无 ambient cookie、Provider credential 或 inherited proxy secret；
- response provenance、digest 和 Artifact/Context 绑定。

主要实现面：

- `apps/kernel-server/src/personal/tool_executor/`
- `crates/cognitive-kernel/src/tool_registry.rs`
- `apps/kernel-server/src/personal/resource_api.rs`
- `crates/cognitive-store/src/artifact_store.rs`

---

## 3.3 增加窄幅 RegisteredCheckRun

软件修复场景需要运行测试，但不应直接增加任意 shell 或通用 `ProcessRun`。

建议引入一个产品语义明确、静态登记的 `RegisteredCheckRun`：

- 用户和 Agent 只能提供 `check_id`；
- executable/toolchain identity 由 daemon 固定；
- argv 使用固定模板和有限参数槽；
- cwd 必须位于 Task workspace；
- environment 使用最小 allowlist；
- 禁止 shell 字符串、管道、重定向和调用方自由 argv；
- timeout、stdout/stderr、子进程树和写入范围有硬上限；
- check descriptor、input、workspace state 和 expected output 绑定 digest；
- 运行前仍需 Intent/Effect、budget、fencing 和 idempotency；
- result 写入有界 Artifact/Evidence；
- exit 0 只能成为证据，不能直接完成 Task；
- unknown outcome 通过原 dispatch identity 查询或保守 quarantine。

### 为什么不是通用 ProcessRun

通用 ProcessRun 会立即引入：

- 任意 executable；
- 环境注入；
- shell escaping；
- 子进程逃逸；
- package manager；
- credential helper；
- Git hook；
- 网络访问；
- privilege escalation；
- 难以查询和 reconcile 的副作用。

RegisteredCheckRun 足以闭合首个软件修复用户场景，同时保持最小权限。

---

## 3.4 接通 Artifact、独立 verifier 与 Task acceptance

完整路径必须成为：

```text
Context
  → Agent candidate
  → daemon admission
  → Tool Intent/Effect
  → dispatch/reconcile
  → Artifact
  → RegisteredCheckRun
  → Evidence
  → independent verifier
  → CONTINUE / STOP / ACCEPT
```

需要补充：

- Tool 输出发布到 Artifact CAS；
- fixed post-state 与 verification request 在 production path 创建；
- verifier 读取当前 epoch、Effect closure、Artifact digest 和 acceptance criteria；
- wrong verifier/version、stale post-state、missing/tampered Artifact、
  passed-without-evidence 全部 fail closed；
- failed check 可以产生下一轮受预算限制的修复 candidate；
- open 或 outcome-unknown Effect 阻止 acceptance；
- 最终 Task transition 只能由 daemon 在当前 version/epoch 下提交。

主要实现面：

- `apps/kernel-server/src/personal/verification_executor.rs`
- `crates/cognitive-store/src/artifact_store.rs`
- `crates/cognitive-store/src/sqlite/store.rs`
- `apps/kernel-server/src/personal/scheduler_authority/`

---

## 3.5 建立全局 Overview 与 Task Activity

本批次应同时补充一个类似任务管理器的只读控制面，但第一版只提供 daemon API、CLI 和
Agent Shell 文本视图，不先做 Web UI。

### 全局 Overview

Overview 应显示所有 **CognitiveOS 管理的**：

- Task；
- AgentInstance；
- SidecarSession；
- AgentExecution；
- ProcessAttempt；
- scheduler state/lease/epoch；
- budget 上限、消耗和剩余；
- open、unknown、reconciling Effect；
- Artifact/Evidence/Verification 状态；
- blocker 和 next action；
- Tool execution readiness；
- 已绑定进程的 CPU、RSS、thread、FD、I/O。

它不得枚举所有宿主 Linux/Windows 进程。只有 daemon 已登记、且 PID 与
process-start identity 绑定的进程才能进入投影。

### Task Activity

每个 Task 可下钻到有序时间线：

```text
intent
→ preview
→ admit
→ schedule
→ Context
→ Agent candidate
→ Tool
→ Effect
→ Process/check
→ Artifact/Evidence
→ verification
→ completion/block/recovery
```

### Event 实现原则

应复用现有 SQLite append-only `events.sequence`：

- snapshot 固定 high-watermark；
- delta 使用 durable sequence；
- reconnect 从 cursor 恢复；
- stale cursor 强制重新 snapshot；
- client 去重但不推断状态；
- 重启后不依赖进程内 `VecDeque`；
- 不新增 Kafka、D-Bus、第二 authority database 或通用 Event Bus；
- 暂时无法覆盖的事件类别必须在 projection 中显式列为 `not_available`。

资源采样可复用：

- `crates/cognitive-runtime/src/resource_sampler.rs`

但产品化前必须增加：

- daemon-owned PID；
- process start-time identity；
- PID reuse 防护；
- Task/AgentExecution 归属；
- sampling interval 和 retention 上限；
- 禁止读取 argv、environment 和 FD target。

### 建议文本入口

```text
cognitive overview
cognitive overview --watch
cognitive activity <task-ref>
cognitive activity <task-ref> --watch
```

Agent Shell 对应自然语言：

- “现在有什么任务正在运行？”
- “哪些任务被阻塞，为什么？”
- “这个 Task 最近执行了什么？”
- “哪个 Effect 结果未知？”
- “Pi 和 daemon 当前占用多少资源？”

主要实现面：

- `crates/cognitive-store/src/sqlite/store.rs`
- `apps/kernel-server/src/personal/task_api.rs`
- `apps/kernel-server/src/personal/resource_api.rs`
- `apps/admin-cli/src/personal_cli/`
- `packages/pi-cognitiveos/`

---

## 3.6 将 Agent Shell 变成真正的 Task Shell

Agent Shell 当前更接近 Pi 对话/status 前端。下一批应让 Pi、CLI 和 SDK 汇入同一个真实
Task application service。

需要补充：

1. 自然语言只生成 interpretation candidate；
2. daemon 解析精确资源、Tool、预算、外部副作用和 acceptance；
3. Shell 展示 canonical preview；
4. 用户批准 exact preview digest；
5. Task 转入后台 durable execution；
6. Shell 支持 attach、detach、interrupt、resume 和 cancel request；
7. Shell 断开不取消 Task；
8. 高风险操作产生结构化 challenge，而不是把聊天文本当批准；
9. CLI 与 Shell 消费同一 Overview/Activity projection；
10. Pi 不得拥有 ambient bootstrap 或 management authority；
11. SDK route、AKP envelope 和 Personal endpoint 语义必须统一。

主要实现面：

- `packages/pi-cognitiveos/`
- `packages/sdk-ts/src/transport.ts`
- `packages/sdk-ts/src/watch.ts`
- `apps/admin-cli/src/personal_cli/`
- `apps/kernel-server/src/personal/task_api.rs`

---

## 4. 首批真实用户场景

## 4.1 只读代码诊断

用户目标：

> 分析固定 Git revision 中一个失败测试，给出候选修复方案，不修改文件。

路径：

```text
intent
→ preview/admit
→ Context
→ WorkspaceSearch/Read
→ analysis Artifact
→ deterministic oracle/verifier
```

验收重点：

- 引用固定事实；
- mutation count = 0；
- unauthorized source exposure = 0；
- Task 只在 verifier 后完成。

## 4.2 受控软件修复

用户目标：

> 修复隔离 workspace 中的确定性缺陷，并运行仓库登记的检查。

路径：

```text
Search/Read
→ Patch Effect
→ RegisteredCheckRun
→ diff/test Artifact
→ verifier
→ completion 或 bounded next iteration
```

验收重点：

- patch 只作用于 approved workspace；
- check 不是任意 shell；
- check fail 不得被包装成成功；
- open/unknown Effect 阻止 completion；
- 重启不重复 patch 或 check dispatch。

## 4.3 受控资料研究

用户目标：

> 从允许的 HTTPS 来源读取资料，生成带来源的分析结论。

路径：

```text
HttpFetchReadOnly
→ provenance Artifact
→ Context
→ result Artifact
→ verifier
```

验收重点：

- URL/redirect/target policy；
- 无 credential leakage；
- source digest 可追踪；
- Provider/HTTP 成功不直接完成 Task。

## 4.4 后台任务与恢复

场景：

- 用户关闭 Shell；
- daemon 在 Tool I/O 后、receipt 前崩溃；
- 用户重新进入 Shell；
- Overview 找到原 Task；
- Activity 从 durable cursor 恢复；
- Effect 使用原 key reconcile；
- Task resume、block 或 quarantine；
- 不产生重复副作用。

---

## 5. 验收与评价维度

不建议在开发后临时发明漂亮门槛。应复用正式场景和预注册 evidence policy。

### 5.1 功能可达性

- CLI 和 Agent Shell 都能从用户目标进入真实 Task；
- 所有步骤经过 production daemon caller；
- 不使用 fixture-only helper 冒充产品路径；
- 三类首批场景均能进入 verifier；
- 失败时产生可解释 blocker 和唯一 next action。

### 5.2 Authority 与安全

以下计数必须为 0：

- unauthorized Context exposure；
- secret exposure；
- duplicate Effect；
- blind retry；
- stale-epoch commit；
- unmanaged PID attach；
- out-of-scope filesystem mutation；
- completion without independent acceptance；
- Agent/Tool/process self-reported completion。

### 5.3 恢复

- restart 后 Task、Effect、Artifact 和 Activity 不丢失；
- unknown outcome 按原 key reconcile；
- cursor 无静默缺口；
- PID reuse 不归属到旧 AgentExecution；
- 无法证明安全恢复时进入 block/quarantine。

### 5.4 可观察性

- Overview 中的所有对象来自 daemon authority；
- Task、Agent、Sidecar、Execution、Process、Effect 关系不混淆；
- CLI 与 Agent Shell projection 语义一致；
- missing/unavailable/not-run 显式显示；
- stdout/stderr、prompt、response、secret、argv 和 environment 不进入普通 Activity。

### 5.5 性能

记录但不预设未经批准的 blocking threshold：

- task admission；
- scheduler wait；
- Context build；
- Tool dispatch/reconcile；
- RegisteredCheckRun；
- verifier；
- time-to-verified-completion；
- CPU、RSS、thread、FD、I/O、DB/WAL；
- Provider token/cost availability；
- Pi launch/first response。

性能结论继续受
`docs/evaluation/personal-performance-benchmark-execution-plan.md` 约束，不能从单次
本地结果升级为 Agent-benefit、Gate、release 或 Profile 声明。

---

## 6. 为什么不是其他路线

### 6.1 不应 Web UI first

当前部分 projection 仍为空或 process-local。先做 UI 会：

- 美化空状态；
- 复制客户端状态；
- 形成第二事实源；
- 延迟真实 caller 和 durable projection。

正确顺序是 daemon read model → CLI/Shell → 稳定后 Web UI。

### 6.2 不应 Multi-Agent first

单 Agent 尚不能完成真实 verified Task。此时增加多个 Agent 会扩大：

- 重复工作；
- budget 消耗；
- candidate 冲突；
- attribution 难度；
- Tool/Effect 竞争；
- false progress。

只有在单 Agent benchmark 证明存在可归因并行瓶颈后，才进入 Multi-Agent。

### 6.3 不应 adapter/platform first

第二活体 Agent 和完整 AKP 是后续重要方向，但如果真实 Task ABI、Tool 参数、Activity 和
completion 尚不稳定，新 adapter 会被迫加入 Agent-specific 特例。

先让 Pi dogfood 完整闭环，再用第二 Agent 验证抽象是否通用。

### 6.4 不应性能重写 first

当前历史数据表明：

- 本地 governance + loopback 约 126.5–128.5 ms p50；
- Provider 网络约 898.9–1016.1 ms p50；
- Pi first response 约 4.625 s p50；
- Pi spawn/init 是明显用户等待来源；
- 但 verified completion 路径尚未接通。

在闭环不可达时迁移 async、重写 store 或做 streaming，无法证明提升了用户任务价值。

---

## 7. 明确后置

下一批不建议实现：

1. Web UI；
2. Multi-Agent；
3. 第二活体 Agent；
4. 公网 A2A listener；
5. 通用 Event Bus/Kafka；
6. 任意 `ProcessRun`；
7. root shell 或 session-global trust toggle；
8. Git 专用 Tool、自动 commit/push/PR；
9. 通用 privilege broker；
10. 动态 check marketplace；
11. embedding/vector/graph Memory；
12. 企业 RBAC、审批链、多租户、HA、cloud sync；
13. async runtime 全面迁移；
14. Provider streaming 重写；
15. 未完成 prerequisites 的 Windows B01-W 扩张；
16. generalized Agent-benefit、release 或 Profile 声明。

---

## 8. 后续批次顺序

完成 Useful Task Loop & Control Surface 后，建议后续顺序为：

1. **真实 AKP product transport：** 让 Pi 先走统一 envelope；
2. **宿主隔离：** 将真实 PID、sidecar 和 AgentExecution 绑定到可验证 sandbox；
3. **第二活体 Agent：** 验证同一 Task/Tool/Activity/Verifier 语义可移植；
4. **MCP 真实执行：** 作为 Tool adapter，不绕过 CognitiveOS authority；
5. **SDK 与 conformance kit：** 让外部开发者无需改 daemon 即可接入；
6. **Web UI：** 消费已经稳定的 Overview/Activity read model；
7. **Multi-Agent/B11：** 仅在等预算收益成立时启用；
8. **更多平台：** Windows、Linux aarch64、macOS 分别独立资格化。

---

## 9. 风险与停止条件

### 9.1 Tool mutation 风险

如果 WorkspaceWrite/Patch 无法可靠 query/reconcile：

- 不启用 mutation；
- 保持只读产品定位；
- 不通过弱化 Intent/Effect 或 verifier 解决。

### 9.2 RegisteredCheckRun 膨胀风险

如果 check 需要调用方自由 executable、argv、environment 或 privilege：

- 停止扩张；
- 拆成新的 product-semantic 决策；
- 不把它伪装成已登记 check。

### 9.3 Overview 成为第二 authority

如果 client 开始根据日志、PID 或 Agent 文本推断状态：

- 停止该投影；
- 回到 daemon authority read model；
- 不允许 optimistic completion 或 client-local lifecycle。

### 9.4 交互复杂度

如果用户仍需每一步批准：

- 优先调整 task preview、scope 和 capability lease；
- 不删除审计、Effect、budget 或验证；
- 默认目标仍是每个 Task 一次主要授权，高风险操作除外。

---

## 10. 最终建议

CognitiveOS Personal 下一批最重要的产品转变是：

> 从“拥有大量正确 Agent OS 原语”转变为“用户可以完成、观察、恢复和验证真实任务的
> Agent OS”。

建议优先级：

```text
真实执行闭环
  > Tool + RegisteredCheckRun
  > Artifact / verifier / acceptance
  > 全局 Overview + Task Activity
  > Agent Shell / CLI
  > 第二 Agent / MCP / Web UI
  > Multi-Agent
```

这一批完成后，CognitiveOS Personal 才真正拥有：

- 可执行的认知进程；
- 可观察的 OS 状态；
- 可恢复的副作用；
- 可验证的完成；
- 用户可理解的控制面；
- 为后续跨 Agent 生态提供稳定基础的真实系统调用路径。
