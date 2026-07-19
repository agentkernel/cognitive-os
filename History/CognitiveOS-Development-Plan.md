# CognitiveOS 开发计划、里程碑与 Cursor AI 协作指南

- 文档版本：1.0 Draft
- 生成日期：2026-07-19
- 参考实现：Rust 核心与服务 + TypeScript 客户端与 Shell
- 首个正式交付：单节点 R0/R1、Core 最小闭环、C1/C2 适配
- 文档性质：当前仓库原地转型为实现仓库的开发规划；不表示任何能力已实现或符合规范

## 1. 结论与实施边界

- 当前仓库的现状是规范和声明式测试资产包：已有 56 份 JSON Schema、65 个 conformance vector，但 [conformance/README.md](conformance/README.md) 明确说明尚无 runner；GovernanceDomainContext、Conversation、Execution/ActivityContext 等治理对象族已由 [docs/standards/governed-object-contract.md](docs/standards/governed-object-contract.md) 登记 v0.1 machine schema，[RFC-0001](RFC-0001-cognitiveos-governance-context-access.md) 的 Knowledge 对象与 AKP governance envelope 仍是伪 schema。接下来的目标是在当前仓库内原地加入参考实现，使其成为同时承载规范、实现、测试与符合性证据的实现单仓库，而不是另建独立实现仓库。
- 仓库转型从 M0 开始：保留现有 `CognitiveOS-Architecture.md`、`RFC-0001`、`specs/`、`conformance/` 和 `History/`，在仓库根目录新增 Rust workspace、TypeScript workspace、应用、测试、标准、ADR 和发布证据；规范资产与实现资产必须保持清晰边界。
- 首个正式版本定义为 `v0.1 Single-node R0/R1`：单节点 Core 闭环、确定性 Management API/CLI、任务 Shell、Context Resolution、Intent/Effect、watch、崩溃恢复、C1/C2 adapter；不把 distributed、R2/R3、SMS/CRB、具身、CIM、在线学习伪装为已实现。
- 参考实现默认技术基线：Rust stable + Tokio；SQLite WAL 作为首个事务型对象/事件/Effect 存储；Rust 内部使用 trait 端口隔离存储和执行器；JSON Schema Draft 2020-12 作为跨语言机器形状合同；TypeScript + pnpm 构建 SDK、确定性 CLI 和可选 Shell UI；单节点外部管理/任务 API 使用 HTTP JSON，watch 使用 SSE。以上均须由 ADR 固定，不是 CognitiveOS 规范要求；行为语义仍以固定版本的 normative requirements 为准。
- 工期基准按 4–6 名具备 Rust、TypeScript、系统与安全测试经验的工程师估算：首个 R0/R1 正式版约 34–47 周；完整路线约 72–103 周。M10 的硬件、安全分析和行业认证是独立关键路径；全部工期须在 M0 根据团队与部署目标重估。

```mermaid
flowchart LR
  contracts[协议与机器契约] --> runner[符合性Runner]
  runner --> kernel[单节点微内核]
  kernel --> context[Context与治理]
  context --> effects[Effect与恢复]
  effects --> shell[Shell与Harness]
  shell --> adapters[C1/C2安装适配]
  adapters --> release[R0/R1正式版]
  release --> discovery[Memory与Discovery]
  discovery --> semantic[SMS与CRB]
  semantic --> distributed[分布式与多Agent]
  distributed --> embodied[具身与异构]
  embodied --> learning[持续学习]
```

## 2. 开工前必须统一的协议与标准

这些资产必须先生成并评审；否则 Rust、TypeScript、存储、Shell 和测试会各自解释草案。新增机器合同必须先明确是 registered normative schema 还是 implementation-private extension，不能把 RFC 中的伪 schema 直接冒充已登记规范资产。

JSON Schema 只定义机器形状，不能单独定义 authority、状态迁移、恢复和验收等行为语义。

1. `docs/standards/normative-source-and-versioning.md`
   - 固定优先级：digest-pinned schema/requirements > RFC/Core/Profile > 白皮书；定义 Draft 变更、SemVer、schema bundle digest、兼容窗口和 legacy `agentos.*` 拒绝/迁移规则。
2. `docs/standards/canonical-encoding-and-digest.md`
   - 采用 UTF-8、RFC 3339 UTC、UUIDv7/ULID 取舍 ADR、RFC 8785 JCS 或等价规范 JSON；明确数字、Unicode、缺省值、未知 critical 字段和 digest/signature 输入。
3. `docs/standards/governed-object-contract.md` + 对应 schemas
   - 机器化 `GovernedObjectHeader`、GovernanceDomainContext、Principal/Membership/ActorChain/Delegation、ResourceScope、Conversation/Binding、ExecutionContext/ActivityContext、strong/weak ref；每个对象显式 `scope_domain`，tenant 生命周期不可变。
4. `docs/standards/state-and-transition-contract.md`
   - 五个执行生命周期状态机、开放 authority state domain、CAS、expected version、reason/causation、非法迁移、terminal semantics；生成可执行 transition tables，禁止在业务代码散落状态判断。
5. `docs/standards/error-contract.md`
   - 统一 `code/stage/retryable/safe_reason/details_ref/observed_versions/correlation_id/retry_after`；对齐 [specs/registry/errors.yaml](specs/registry/errors.yaml)，消除 Core 文本名和 registry 机器码不一致；规定安全拒绝不泄露存在性。
6. `docs/standards/authn-authz-capability.md`
   - Human/Workload/Device 身份、ActorChain、RBAC+ABAC+ReBAC、default deny、deny precedence、capability 单调衰减、lease/revocation/fencing、逐 hop 本地重授权和 break-glass 边界。
7. `docs/standards/context-resolution-and-cache.md`
   - 九阶段解析、required/forbidden、partial、loss declaration、逐对象读取授权、cache key 安全维度、失效与 reason code；禁止 ranker 在授权前接触正文。
8. `docs/standards/intent-effect-idempotency.md`
   - `Intent→Authorize→Execute→Reconcile→Verify→Commit/Abort`、Effect transition table、idempotency scope、同 key 异参、三个 crash point、unknown outcome、补偿与 quarantine。
9. `docs/standards/event-audit-watch.md`
   - authoritative event、security audit、telemetry 分离；outbox/ack/high-watermark、snapshot-plus-delta、dedupe、连续性、背压、SSE 映射和审计闭合。
10. `docs/standards/task-loop-verification.md`
   - UserIntentRecord→Interpretation→TaskContract→proposal→Effect→Verification→Acceptance；定义 progress、stagnation、checkpoint、candidate complete 和 acceptance authority。
11. `docs/standards/akp-envelope-and-http-profile.md`
   - 将 [specs/akp/README.md](specs/akp/README.md) 的传输无关语义机器化；定义 negotiation epoch、request/result、cancel、partial、continuation、HTTP/SSE 映射和 `PROTOCOL_MAPPING_INCOMPLETE`。
12. `docs/standards/conformance-evidence.md`
   - requirement↔schema↔vector↔test↔evidence 可追踪矩阵；定义 pass/fail/not-applicable/degradation、profile manifest、suite digest、性能报告和 readiness case。
13. ADR 集：`docs/adr/0001-rust-typescript.md`、`0002-sqlite-wal.md`、`0003-json-http-sse.md`、`0004-canonical-json.md`、`0005-id-and-clock.md`、`0006-code-generation-policy.md`。

协议冻结验收：所有首版跨边界对象均有已分类的机器合同；Rust/TS 对同一 golden fixture 得到同一 digest；registry 不再引用不存在的 schema/vector；错误码、状态迁移和操作集合无文本/机器命名漂移；Core v0.3 candidate extension 不自动进入 v0.2 Core 声明；breaking change 有迁移与协商策略。

## 3. 当前仓库转型原则

- 采用原地演进，不创建与规范仓库分离的第二个实现仓库。
- `specs/` 继续承载规范文本、schema 与 registry；实现代码不得反向把私有类型声明成规范事实。
- `conformance/` 继续承载声明式向量；可执行 runner 放入 `crates/cognitive-conformance/`，运行证据放入独立构建产物或 `artifacts/evidence/`，不得改写测试向量来迎合实现。
- `History/` 保持只读历史用途，不参与当前构建、schema bundle 或符合性声明。
- 实现代码、生成代码、测试和证据均在同一版本控制边界内，以便固定 specification digest、implementation commit 和 suite digest。
- 仓库根目录的 README、构建入口和 CI 必须明确区分“规范已登记”“实现已提供”“测试已执行”“Profile 已符合”四类状态。
- 转型过程不重写现有规范历史；需要修改 normative asset 时使用独立、可评审的规范变更，并同步 registry、schema、vector 和 migration 说明。

转型完成的仓库角色是：CognitiveOS 规范源、Rust/TypeScript 参考实现、符合性 runner、参考应用、测试资产和发布证据的统一单仓库。

## 4. 建议代码结构
- `Cargo.toml`：Rust workspace。
- `crates/cognitive-contracts`：schema 绑定、canonical encoding、ID/ref、错误与 golden fixtures。
- `crates/cognitive-domain`：状态机与纯领域不变量，不依赖网络/数据库/模型。
- `crates/cognitive-store`：SQLite repositories、append log、outbox、snapshot、WAL recovery。
- `crates/cognitive-kernel`：authority、CAS、capability、budget、Context gate、Effect、checkpoint/recovery。
- `crates/cognitive-runtime`：Operation executor、sandbox/adapter ports、Harness loop。
- `crates/cognitive-management`：Management API、session/proposal/approval 和 deterministic fallback。
- `crates/cognitive-akp`：AKP envelope、HTTP/SSE transport profile。
- `crates/cognitive-conformance`：schema/registry/vector runner 与 evidence bundle。
- `apps/kernel-server`：单节点组合根和 readiness 状态。
- `packages/contracts-ts`、`packages/sdk-ts`、`apps/admin-cli`、`apps/agent-shell`：TS 合同、SDK、确定性 CLI 与 Shell。
- `tests/e2e`、`tests/faults`、`tests/security`、`tests/golden`：跨语言、故障注入、负例和 digest fixtures。

依赖方向固定为 contracts → domain → ports → adapters/apps；domain/kernel 不依赖 TypeScript、HTTP、SQLite 具体类型或 LLM SDK。

## 5. 里程碑、交付物与验收

### M0 规范闭合与工程基线（3–5 周）

交付：完成当前仓库向实现单仓库的目录与构建入口转型；保留并隔离现有规范资产；生成上述标准/ADR、首版伪 schema 分类与必要机器合同登记提案、schema bundle/requirement/suite digest 工具、Rust workspace、pnpm workspace、CI、格式化/lint、供应链和提交规范。

验收：仓库根目录可统一发现规范、Rust/TypeScript workspace、应用、测试与证据入口；现有规范文件和历史资产未被实现初始化破坏；全部 JSON/YAML 可解析；所有 `$ref` 可解析；schema、REQ-ID、error、vector 双向无孤儿；Rust/TS golden digest 一致；CI 在 Windows/Linux 运行；首版 profile 全部标 `planned`，不得误报实现。

### M1 可执行符合性 Runner（3–4 周）

交付：建立静态资产验证、implementation adapter、vector selection/execution、observable expected 比较，以及 evidence bundle、JUnit/JSON 报告和 profile manifest；runner 按当前已有实现分层启用 vector，不要求 M1 当期让全部 65 个场景通过。

验收：无实现时只报告 not-run；故意错误实现必须 fail；schema-valid 不等于 behavior-pass；`spec-contract-coverage` 不能替代行为测试；结果可追溯到 requirement、schema/vector digest、实现 commit；runner 自身 mutation/negative tests 通过。

### M2 单节点对象、状态与事件内核（5–6 周）

交付：GovernedObject、strong ref、authority registry、五状态机、CAS、snapshot/projection、append-only event/outbox、budget、统一错误；SQLite 原子提交状态+Event。

验收：并发 CAS 只有一个成功；非法迁移全拒绝且状态不变；projection 重放 digest 稳定；event 不可原地修改；tenant/platform scope fail closed；状态/事件/CAS vectors 与新增 property tests 通过。

### M3 身份、Conversation、Capability 与 Context（5–6 周）

交付：TenantContext、Principal/Membership/ActorChain、Conversation/ResourceScope、Execution/ActivityContext；授权交集；capability 衰减/撤销；确定性九阶段 Context Resolution、loss、cache binding。

验收：同 tenant 横向读取、管理员正文读取、跨 Conversation cache、旧 revocation、URI bearer、rank-before-auth 全被拒绝；required 超预算 fail closed；partial 只按 TaskContract；每个 loaded item 可追溯版本/digest；Context vectors 和安全负例通过。

### M4 Intent/Effect、Verification 与恢复（5–7 周）

交付：Intent 持久化、OperationDescriptor/binding、Effect 状态机、幂等记录、executor/reconciler/verifier ports、checkpoint、recovery barrier/fencing/replay/quarantine、故障注入框架。

验收：覆盖 Intent 后/调用前、执行后/receipt 前、验证后/commit 前三个 crash point；unknown outcome 不成功、不换 key；同 key 异参拒绝；receipt/remote completed 不完成 Task；恢复严格按 barrier→identity/epoch→fence→replay→reconcile→reauthorize→resolve→resume；Effect/recovery vectors 全通过。

### M5 Task/Harness、确定性 Management 与普通任务 Shell（6–8 周）

交付：UserIntentRecord、IntentInterpretation admission、TaskContract、bounded Loop、progress/stagnation、watch；Management API + 不依赖模型的 admin CLI；普通任务 Shell proposal/preview/attach/cancel/status；可插拔 parser 的 LLM 仅生成 candidate。自然语言 Intelligent Management Shell 保持禁用或 experimental，不能成为首版确定性管理依赖。

验收：物质歧义必须澄清；preview 漂移重预览；用户修正推进 epoch 并 fence 旧 dispatch；Shell 退出不取消；watch 可 snapshot+cursor 恢复并发现缺口；session 关闭不丢 pending Effect；无模型仍能 inspect/stop/revoke/reconcile；intent/shell/management vectors 通过。

### M6 Agent 安装、C1/C2 适配与 R0/R1 发布（5–7 周）

交付：package/install/compatibility、sandbox interception、Identity/Memory/Tool/Completion/Checkpoint adapters、静态 Catalog、一个 legacy reference agent、installation rollback、readiness/profile/evidence 包。Linux 作为首个可声明 sandbox interception 的参考平台；Windows 在完成等价负例前仅用于开发与合同测试。

验收：未声明 network/secret/filesystem/subprocess/IPC/MCP/tool proxy 绕过被测试拒绝；Agent `done` 仅 candidate complete；C2 恢复先对账；C/R 两轴独立报告；R0 无外部写，R1 仅可查询、可撤销或可人工确认 Effect；完整 `implemented` 声明要求全部适用 MUST 通过，未关闭降级应缩小范围或标 experimental；发布 `MANAGEMENT_READY`、`USER_READY`、`OPERATIONAL` 三种状态及安装/恢复演练。

### M7 Governed Memory 与 Cognitive Discovery（5–7 周）

交付：Memory candidate/admission/promotion/invalidation/delete；ResourceManifest、InformationGap、ContextViewDelta、累计预算、停滞检测、authority refresh。

验收：跨 scope 不原地改标签；删除沿 embedding/index/cache/view 闭合；discover/read/action 分离；delta 不扩大父 scope/budget；服务不可用不等于 no-result；memory/discovery vectors 通过。

### M8 SMS、CRB 与 Operation Catalog（4–6 周）

交付：两级 Catalog、match/bind/dry-run；SMS provider plugin；CRB reservation；static/BM25/local/model fallback；按 risk 分层 eval。

验收：语义输出永远是 candidate；被确定性过滤的 Operation 不可被模型恢复；effect class 歧义不 top-1 执行；fallback 不扩权/出域/预算；模型不可用时核心管理和停止路径不受影响。

### M9 Distributed 与 Multi-Agent（8–12 周）

交付：AKP 远程 transport、node identity、lease/epoch/fencing、mailbox/outbox、delegation、partition policy、ConflictSet、ShareGrant 本地重授权。

验收：split brain 只有 authority 侧可提交；旧 leader/lease 写被 executor 拒绝；重复/乱序消息不重复 Effect；子预算/capability 不放大；跨 tenant capability 不直接复用；分区和灾恢复演练达到声明 RPO/RTO。

### M10 Embodied 与 Heterogeneous（12–18 周，独立安全项目）

交付：安全内核边界、三回路、SkillContract、watchdog/safe state、ResourceGraph/handles/placement、校准/漂移/误差/fallback；行业 hazard analysis 与认证接口。

验收：断网/认知服务失效仍可 safe state；过期、错 frame/unit、旧 epoch、超 envelope 命令拒绝；动态 Context/SMS 不进入硬实时周期；CIM 超误差不用于 auth/audit/safety；WCET/jitter/deadline 由目标设备证据证明。未完成独立安全 case 前只标 experimental。

### M11 Controlled Learning 与受治理知识编译（8–12 周）

交付：Evidence/Claim/Knowledge machine contracts、依赖图和有界 recompile；Policy/Model/Verifier candidates；offline→shadow→canary→release→rollback。

验收：Episode 不直改生产；self-corroboration、跨租户训练、verifier 自我弱化被拒绝；source 删除传播；shadow 无写权；canary 有硬停止；rollback 区分代码/状态/世界效果；知识与学习 vectors、性能报告和 readiness case 通过。

## 6. 横向质量门禁

- 每个 PR 必须列出关联 REQ-ID、状态机影响、错误码、schema 兼容性、威胁与测试证据；无关联要求时说明原因。
- 领域不变量优先 property/state-machine tests；持久层做 crash/fault tests；跨语言做 golden contract tests；安全路径必须有负例。
- 禁止使用 mock receipt 证明 Task 完成；测试必须观察 authority state、Effect、Verification 和 Event。
- 每个里程碑生成与风险相称的 SBOM/依赖审计、profile manifest、degradation 和性能基线；涉及持久状态、Effect 或发布的里程碑必须生成恢复演练记录。
- 任何完整 `implemented` 状态必须满足全部适用 MUST；未关闭降级只能缩小声明范围、列为不适用并给出依据，或标记 experimental。演示、schema-valid 或 vector 存在均不算通过。

## 7. Cursor 规则文档撰写提示词

将下列提示词投喂给 Cursor，要求其先阅读主架构、Core、RFC-0001、目标 Profile、requirements/errors registry，再创建 `.cursor/rules/*.mdc`。规则应一关注点一文件、通常少于 50 行，使用 `alwaysApply` 或明确 globs。

```text
你正在为 CognitiveOS Rust + TypeScript 单仓库创建 Cursor Project Rules。先只读检查 CognitiveOS-Architecture.md、specs/core/README.md、RFC-0001-cognitiveos-governance-context-access.md、specs/akp/README.md、specs/registry/{requirements,errors}.yaml 与 conformance/README.md。不要把白皮书说明性文字、伪 schema、planned vector 当作已实现机器合同。

请在 .cursor/rules/ 设计并撰写以下精简 .mdc：
1. architecture-invariants.mdc（alwaysApply）：authority/observation/proposal/execution/verification 分离；五状态机分离；ContextView 非 authority；模型仅产 candidate；跨治理边界必须中介。
2. normative-traceability.mdc（alwaysApply）：每次实现先定位 REQ-ID、固定 schema/vector digest，冲突采用不扩权解释，禁止虚构 requirement/error/schema。
3. rust-kernel.mdc（globs: **/*.rs）：依赖方向、newtype ID、无 panic、显式 Result、状态迁移集中化、确定性 gate 不调用模型、数据库事务边界。
4. typescript-clients.mdc（globs: **/*.{ts,tsx}）：TS 只消费生成合同，Shell 是客户端非 authority，普通/管理通道隔离，状态来自 authority projection。
5. schemas-protocol.mdc（globs: specs/schemas/**/*.json）：Draft 2020-12、canonical encoding、critical extension、兼容性、strong ref、scope_domain 与安全错误。
6. effect-recovery.mdc（globs: crates/{cognitive-kernel,cognitive-store,cognitive-runtime}/**/*）：稳定幂等键、OUTCOME_UNKNOWN、补偿独立授权、恢复顺序和 crash-point 测试。
7. security-testing.mdc（globs: {crates,packages,apps,tests}/**/*）：default deny、对象级重验、缓存绑定、无存在性泄露、每个安全功能的负例。
8. conformance.mdc（globs: {conformance,crates/cognitive-conformance,tests}/**/*）：schema-valid 不等于 pass、pass/fail/not-applicable/degradation/not-run、evidence 与 manifest 规则。

每份规则写清“必须做/禁止做/完成前检查”，引用仓库真实路径和 REQ-ID；不要塞入大段架构复述，不要规定尚未由 ADR 决定的库。完成后列出每个 rule 的触发范围、理由和潜在冲突，并运行静态检查确认 frontmatter 正确。
```

## 8. 各阶段投喂 Cursor 的提示词

所有阶段先附加这一公共前缀：

```text
工作模式：先检查当前分支和未提交改动，不覆盖用户已有修改。只实现本阶段明确范围。以 digest-pinned schema/REQ-ID 为准；遇到白皮书、companion、registry、schema/vector 漂移时先报告并用最小 ADR/规范修复闭合，不自行猜测安全语义。概率组件只能产 candidate；authorization、CAS、hard budget、state transition、idempotency 和 commit 必须确定性执行。先写/更新失败测试，再实现，再运行 format/lint/unit/integration/conformance。最终报告改动文件、REQ-ID、测试证据、degradation、未决风险；不得把 planned/experimental 报为 implemented。
```

- M0 提示词：`完成 M0。将当前仓库原地转型为同时承载规范与实现的单仓库，保留并隔离现有 architecture/RFC/specs/conformance/History；建立 Rust/pnpm workspace、应用/测试/证据入口、canonical digest golden fixtures、registry-schema-vector 完整性检查和 CI。分类首版 Governance/Conversation/Execution/Activity/AKP envelope 伪 schema，只有经规范变更登记后才作为 normative machine contract。只做仓库转型、合同与工程基线，不实现业务。`
- M1 提示词：`完成 M1。实现分层启用 declarative vectors 的 conformance foundation 和 implementation adapter，输出 pass/fail/not-applicable/degradation/not-run 及证据 digest。加入故意错误 adapter，证明仅 JSON/schema-valid 不能通过；未实现层必须保持 not-run。`
- M2 提示词：`完成 M2。实现纯 Rust GovernedObject、authority、状态 transition tables、CAS、snapshot/projection、event/outbox、budget 和 SQLite 原子性。用并发、property、replay 和非法迁移测试覆盖 Core state/event requirements。`
- M3 提示词：`完成 M3。实现治理上下文、ActorChain、Conversation/ResourceScope、capability 与确定性 Context Resolution。重点验证同租户横向越权、管理员正文拒绝、检索前过滤、撤销后缓存失效、跨 Conversation 污染和 required 超预算。`
- M4 提示词：`完成 M4。实现 Operation binding、Intent/Effect/Verification、幂等与 recovery barrier。注入规范要求的三个 crash point，证明 unknown outcome 只能对账、原 key 重试、独立补偿或隔离，Task 只由 acceptance authority 完成。`
- M5 提示词：`完成 M5。实现意图记录/解释准入、TaskContract、bounded Harness Loop、Management API/admin CLI、普通任务 Shell 和 snapshot-plus-delta watch。LLM parser 仅作为可替换 candidate provider；Intelligent Management Shell 保持 disabled/experimental；无模型时管理、停止和对账必须可用。`
- M6 提示词：`完成 M6。实现 package/install/compatibility、sandbox 和 C1/C2 adapters，提供 legacy reference agent。执行绕过、Conversation 隔离、completion、checkpoint/recovery 负例，生成 R0/R1 readiness case 和 profile manifest。`
- M7 提示词：`完成 M7。实现 Governed Memory 与 Cognitive Discovery 的 candidate/admission/promotion/invalidation、manifest/gap/delta/stagnation；跨 scope 创建新对象，删除闭合到全部派生缓存和索引。`
- M8 提示词：`完成 M8。实现 versioned Operation Catalog、match/bind/dry-run、SMS plugin 和 CRB。所有语义结果保持 candidate；测试模型不可用、错误 effect top-1、catalog rug pull、fallback 扩权和预算逃逸。`
- M9 提示词：`完成 M9。实现 AKP 远程映射、lease/epoch/fencing、mailbox、delegation、partition/ConflictSet 和 ShareGrant 本地重授权。用故障注入证明 split-brain、乱序重复、孤儿 Effect 和恢复竞态正确收敛。`
- M10 提示词：`完成 M10 的实验性具身/异构 Profile。先固定目标设备 hazard、deadline/WCET、frame/unit 和安全 authority；实现三回路、安全仲裁、ResourceGraph/placement/calibration/fallback。没有独立安全证据时保持 experimental，禁止宣称 R3 implemented。`
- M11 提示词：`完成 M11。实现受治理知识依赖图和有界维护，以及四类学习 candidate 的 offline/shadow/canary/release/rollback。测试投毒、自证、跨租户数据、verifier 篡改、删除传播、canary kill 和 rollback failure。`

每次只把公共前缀、当前阶段提示词、该阶段相关规范路径和当前失败报告投喂给 Cursor；不要一次把未来阶段全部要求放进实现窗口。阶段结束开启新对话，以 checkpoint 文档承接：已完成 REQ-ID、manifest/evidence digest、架构决策、未决 Effect/风险、下一阶段入口条件。

## 9. 发布判定

`v0.1 Single-node R0/R1` 只有在 M0–M6 全部完成后发布。硬门槛为：Core 适用 MUST 闭合；符合性 runner 可重复执行；安全负例通过；崩溃恢复演练通过；确定性管理 fallback 可用；C1/C2 声明有不可绕过证据；profile manifest 固定所有 digest；所有未实现 Profile 明示 planned/experimental/unsupported。M7–M11 分别独立发布实验 Profile，不阻塞首版，但不得倒灌改变已冻结 Core 语义而无版本迁移。
## 10. 复核结论与关键修正

本计划以当前仓库原地转型为实现单仓库为前提，并与架构的最小闭环、R0/R1 风险基线以及“先安装与不可绕过，再发现，后语义优化”的路线一致。正式采用时必须保留下列修正：

1. Rust、TypeScript、SQLite、HTTP 和 SSE 是参考实现 ADR，不是 CognitiveOS 规范要求。
2. JSON Schema 是机器形状合同，不替代 normative behavior；authority、状态机、Effect 与恢复仍以固定 requirement 为准。
3. Conformance runner 从 M1 建立基础能力，此后按实现分层启用；未实现 vector 必须是 not-run，不能虚报 pass。
4. `specs/core/README.md` 的 v0.3 candidate extension 不能自动纳入 v0.2 Core 符合性声明。
5. 确定性 Management API/Admin CLI 属于首版基线；自然语言 Intelligent Management Shell 是可选 experimental profile。
6. C1 sandbox 的不可绕过性按宿主平台分别证明；Linux 与 Windows 不共享未经验证的声明。
7. 完整 `implemented` 采用全部适用 MUST 通过的严格口径；降级不应被用来包装完整符合。
8. 首版 34–47 周是资源假设下的范围估算，不是发布日期承诺。

## 11. AI 对话交接要求

每个阶段使用独立 Cursor 对话。阶段结束生成 `docs/checkpoints/Mx-handoff.md`，记录固定规范/digest、已完成和未完成 REQ-ID、manifest/evidence、ADR、已知 degradation、未决 Effect/风险及下一阶段入口条件。新对话只投喂公共前缀、当前阶段提示词、相关规范、上一阶段 handoff 和当前失败报告，避免一次加载未来全部阶段。
