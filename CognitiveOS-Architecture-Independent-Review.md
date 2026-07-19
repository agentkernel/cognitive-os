# CognitiveOS 架构独立审查与优化建议

- **审查日期**：2026-07-20
- **审查性质**：独立、反方优先、证据驱动、只读审查
- **审查快照**：Git commit `f2f826adae6d0f61e31cf41d31c2b4d5905d0ca8`
- **主文档摘要**：`CognitiveOS-Architecture.md` SHA-256 `f6853fa6eb55ed873d3f2b020a339f044d2756b94aea5c7d3492c9acb9d3052f`
- **唯一输出资产**：本报告；未修改架构、RFC、spec、registry、schema、transition、vector、计划、标准或 ADR

本报告使用以下结论类型：

- **[confirmed fact]**：由当前仓库内容、可重复静态检查或直接外部原始资料确认；
- **[evidence-backed judgment]**：基于已列证据作出的架构判断，仍可能被实现证据推翻；
- **[hypothesis]**：尚需实现或实验验证；
- **[unresolved question]**：当前资产不能给出唯一答案。

严重度定义：**P0** 阻断安全实现、正确恢复、可信符合性或高风险发布；**P1** 不阻断受限原型，但阻断完整声明、可移植实现或可信性能结论；**P2** 为范围、证据卫生或长期维护问题。

---

## 1. 执行摘要

### 1.1 总体结论

**[evidence-backed judgment] 总判定：需要架构重构。**

这里的“重构”不是否定核心命题，而是要求先收敛机器合同、authority 边界和最小 Core。当前设计中，持久执行、状态 authority、Context 门禁、Intent/Effect、独立验证和恢复顺序构成了一个有价值的 Agent control plane/durable runtime 命题；但现有 normative 资产存在可构造的安全反例和互相冲突的对象体系，不能直接指导一个可声明符合性的实现。

**[evidence-backed judgment] OS 定位仅部分成立。** 架构提出了保护、持久执行、资源治理、设备中介、故障语义和系统调用面等 OS 判据；但仓库尚未证明参考监视器不可绕过、宿主平台隔离、所有提交端 fencing、设备中介或可执行符合性。实现证据出现前，更准确的产品称谓是“Agent control plane + durable governed runtime”；满足宿主级不可绕过性和设备仲裁证据后，OS 称谓才得到工程支撑。

### 1.2 是否可以直接进入实现

- **[evidence-backed judgment] 不可直接进入完整实现。**
- 可以先做一个受限的“合同修复 + tracer bullet”工程尖峰，但不得以当前 schema/transition/vector 直接生成生产代码或宣称 Core 已冻结。
- 单节点 R0/R1 的第一条竖切应在 F-002—F-010 对应 P0 收敛后开始；R2/R3、分布式、多 Agent、具身和持续学习应继续延后。

### 1.3 最大 P0 风险

1. **机器治理对象双轨**：30 个 schema 仍使用不含 tenant/ResourceScope/policy/purpose/retention 的 legacy `common-defs.metadata`，而新治理对象使用 `GovernedObjectHeader`。
2. **Effect 可出现机器合法但语义不可能的状态**：静态反例 `state=COMMITTED + observed_outcome=unknown + verification=pending + decision=pending` 通过当前 `effect.schema.json`。
3. **Context 竖切存在启动循环和授权字段缺口**：`ActivityContext` 必须先引用 `ContextView`，而 ContextView 又应绑定 Activity；现有 `ContextRequest` schema 还缺 TenantContext、ActorChain、Conversation/ResourceScope 和治理版本。
4. **撤销竞态未闭合**：Effect 的 `AUTHORIZED→EXECUTING` 与 `VERIFIED→COMMITTED` transition guard 没有明确重验 capability/revocation。
5. **持久层失效没有 fail-before-effect 规则**：数据库只读、磁盘满、WAL/outbox 无法落盘时是否允许 dispatch 没有唯一规定。

### 1.4 当前证据结论

- **[confirmed fact] 正确运行已经被证明：否。** 仓库无参考实现、无 runner、无测试执行结果。
- **[confirmed fact] 性能不劣化已经被证明：否。** 没有 native 与 governance-only 的实测报告。
- **[confirmed fact] Agent 显著收益已经被证明：否。** 当前只有机制假设、声明式 schema/vector 和外部类比。

### 1.5 Finding 统计

| 严重度 | 数量 | ID |
|---|---:|---|
| P0 | **11** | F-001—F-011 |
| P1 | **15** | F-012—F-026 |
| P2 | **4** | F-027—F-030 |
| 合计 | **30** | F-001—F-030 |

---

## 2. 审查范围与方法

### 2.1 仓库快照与工作树保护

**[confirmed fact]** 审查开始时先执行了 `git status`、`git diff` 和 `git diff --stat`。审查期间仓库 HEAD 被外部更新并最终落到 `f2f826a`；为避免把两个版本混为同一证据，本报告以最终 commit 和上述 SHA-256 为准。最终写报告前，工作树仅有预先存在的未跟踪 `docs/prompts/`。本审查没有回退、格式化或改写这些资产。

**[confirmed fact]** 关键文件快照：

| 资产 | 内容行数 | SHA-256 |
|---|---:|---|
| `CognitiveOS-Architecture.md` | 1915 | `f6853fa6...d3052f` |
| `CognitiveOS-Review-Conclusions.md` | 356 | `474f0013...dc2f3` |
| `RFC-0001-cognitiveos-governance-context-access.md` | 669 | `25bcbd9e...7841` |
| `specs/core/README.md` | 705 | `d478aa40...5f7ad` |
| `specs/registry/requirements.yaml` | 1632 | `69991983...531c` |
| `specs/schemas/performance-report.schema.json` | 365 | `981d9a36...f58da` |

### 2.2 检查资产

**[confirmed fact]** 已全文或结构化检查：

- 主架构全文及当前历史差异；
- `CognitiveOS-Review-Conclusions.md`，仅作为待复核输入；
- RFC-0001、11 份 `specs/**/README.md` companion；
- 3 个 registry、5 个 transition table、56 个 JSON Schema；
- `conformance/README.md` 与 68 个 vector；
- 4 个 `docs/standards/` 标准、2 个 `docs/adr/` ADR；
- 归档开发计划与 History 边界；
- Git 历史、owner path/anchor、REQ/vector/error 双向引用。

### 2.3 只读静态检查

**[confirmed fact]** 执行结果：

- 56/56 schema 通过 Draft 2020-12 meta-schema 结构检查；
- 所有本地相对 `$ref` 文件和 JSON Pointer 可解析；
- 5/5 transition table 通过 table schema；所有状态可达、terminal 无出边；
- 68 个 vector JSON、56 个 schema JSON、5 个 transition JSON 均可解析；
- 270 个 REQ ID、53 个 error code、68 个 vector ID 均唯一；
- registry→vector、vector→REQ、vector→error 未发现路径级孤儿；
- 270 个 REQ 中 175 个只映射到 `SPEC-CONTRACT-COVERAGE-001`；
- 21 个 vector 仅检查 owner/status/evidence 声明，不描述可执行行为；
- 构造了 4 个 schema 反例：不可能 Effect、`untrusted+control` ContextItem、无 acceptance 条件 TaskContract、删除 `governance_overhead` 的性能报告；四者都通过对应 schema。

这些检查只证明资产可解析或暴露合同缺口，**不等于 behavior-pass**。

### 2.4 外部证据

本报告优先使用原始标准、同行评审论文和官方资料：

- `[STD]` NIST SP 800-207 Zero Trust Architecture：https://csrc.nist.gov/pubs/sp/800/207/final
- `[STD]` NIST SP 800-63B-4（2025，取代旧 800-63B）：https://csrc.nist.gov/pubs/sp/800/63/B/4/final
- `[STD]` RFC 9110 HTTP Semantics：https://www.rfc-editor.org/rfc/rfc9110
- `[RFC-INFO]` RFC 8785 JCS（Informational，不是 Standards Track）：https://www.rfc-editor.org/rfc/rfc8785
- `[PEER]` Mohan et al., ARIES, ACM TODS 1992：https://doi.org/10.1145/128765.128770
- `[PEER]` Terry et al., Session Guarantees, PDIS 1994：https://doi.org/10.1109/PDIS.1994.331722
- `[PEER]` Newcombe et al., AWS 使用 TLA+, CACM 2015：https://doi.org/10.1145/2699417
- `[PEER]` Klein et al., seL4, CACM 2010：https://doi.org/10.1145/1743546.1743574
- `[PEER]` Cemri et al., MAST, NeurIPS 2025：https://proceedings.neurips.cc/paper_files/paper/2025/file/b1041e52d3be19f0a9bc491657488e4a-Paper-Datasets_and_Benchmarks_Track.pdf
- `[PEER]` SWE-bench, ICLR 2024：https://openreview.net/forum?id=VTF8yNQM66
- `[PEER]` GAIA, ICLR 2024：https://proceedings.iclr.cc/paper_files/paper/2024/file/25ae35b5b1738d80f1f03a8713e405ec-Paper-Conference.pdf
- `[PEER]` HELM, TMLR 2023：https://openreview.net/forum?id=iO4LZibEqW
- `[PEER, transferable methodology]` CONSORT non-inferiority extension：https://jamanetwork.com/journals/jama/fullarticle/1487502
- `[PRE]` On Randomness in Agentic Evals, 2026：https://arxiv.org/abs/2602.07150
- `[IND, vendor official]` Stripe idempotency：https://docs.stripe.com/api/idempotent_requests
- `[IND, vendor official]` Anthropic prompt caching：https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching
- `[OFFICIAL BENCHMARK RULES]` MLPerf Inference rules：https://github.com/mlcommons/inference_policies/blob/master/inference_rules.adoc
- `[IND, security guidance]` OWASP AI Agent Security：https://cheatsheetseries.owasp.org/cheatsheets/AI_Agent_Security_Cheat_Sheet.html

### 2.5 未检查范围与证据限制

- **[confirmed fact]** 未运行实现测试、fault injection、负载 benchmark 或 sandbox escape 测试：仓库没有实现或 runner。
- **[confirmed fact]** 未执行 TLA+/Alloy：仓库没有形式模型。
- **[confirmed fact]** 未验证真实 MCP/A2A endpoint、模型 provider、机器人、CIM 或多节点部署：没有对应系统。
- **[unresolved question]** 外部网页和预印本未来可能修订；本报告记录审查日可核验状态。
- **[evidence-backed judgment]** 任何外部成功案例只能支持机制选择，不能证明 CognitiveOS 的未来实现正确或更快。

---

## 3. 资产真实性与规范漂移

### 3.1 实际资产数量

| 资产 | 实测数量 | 状态解释 |
|---|---:|---|
| `specs/**/README.md` companion | 11 | 文本 Draft，不是实现 |
| 根目录 normative RFC | 1 | v0.2 Draft |
| JSON Schema | 56 | shape contract；不证明行为 |
| transition table | 5 | 54 个状态、90 条边 |
| requirement | 270 | 全部 `specified`；不等于 implemented |
| error code | 53 | registry 条目 |
| conformance vector | 68 | 声明式输入；无 runner |
| conformance layer | 15 | 分类，不是执行层 |
| standards | 4 | Draft |
| ADR | 2 | ADR-0004、ADR-0005 |
| 当前实现源文件 | 0 | 仅 History 中 2 个旧编辑脚本 |
| 可执行 runner | 0 | `conformance/README.md` 明示不存在 |

**[confirmed fact]** `CognitiveOS-Review-Conclusions.md` §1 的 40 schema/65 vector/244 REQ/49 error 等计数已经过期；归档开发计划中的 65 vector 也已过期。白皮书附录 D 的 270 REQ/68 vector 与当前实测一致。前一评审不能作为当前事实源。

### 3.2 REQ、owner、vector 与错误码闭合

**[confirmed fact]**

- 白皮书中 34 个可识别的精确 REQ ID 均已登记；
- `REQ-CTX-001..012`、`REQ-RUN-001..009`、`REQ-MGMT-SESSION-001..003`、`REQ-KNOW-001..009` 无编号缺口；
- 270 个 `owner_spec` 路径均存在，带 fragment 的锚点均可解析；
- registry 指向的 vector ID 均存在，vector 引用的 REQ/error 均存在；
- 白皮书 Markdown 文件链接未发现断链。

上述是**路径闭合**，不是语义闭合。

### 3.3 三个重点 REQ 核验

| REQ | registry | owner_spec | schema | vector | 独立判定 |
|---|---|---|---|---|---|
| `REQ-PERF-004` | 已登记 | `CognitiveOS-Architecture.md#performance-contract` | `governance_overhead` 存在但非 required | `PERF-REPORT-CONTRACT-001` | **登记真实，但 owner 是 informative 白皮书，且 schema 不强制；不得视为闭合** |
| `REQ-CTX-012` | 已登记 | `specs/core/README.md` §6.6 | 没有 renderer/prefix machine contract | `CTX-RENDER-001` | **登记真实；只验证重复 digest/相对顺序，未保证字节前缀追加稳定** |
| `REQ-MEM-ADMIT-002` | 已登记 | `specs/governed-memory/README.md` §3 | candidate schema 无 writer/Activity/Conversation 私有绑定 | `MEM-RYW-001` | **登记真实；行为文本存在，机器形状不足以执行隔离** |

因此不应简单改回 candidate；应按规范变更流程修复 owner、schema 和 vector 后保留登记。

### 3.4 informative/normative 越界

**[confirmed fact]** 17 个 requirement 的 `owner_spec` 是声明为 informative 的 `CognitiveOS-Architecture.md`，包括：

`REQ-STATE-003/004`、`REQ-CTX-004/008`、`REQ-CAP-002/003`、`REQ-EFF-004/006`、`REQ-RES-001`、`REQ-SEC-002`、5 个 Profile REQ、`REQ-PERF-002/004`。

这与 `docs/standards/normative-source-and-versioning.md` §2—§4 “informative 不定义行为义务”的规则冲突。特别是 crash recovery 和 performance 的 owner 不能同时被白皮书声明为非规范、又在 registry 中承担 normative behavior source。

### 3.5 状态图、schema 与 vector 漂移

**[confirmed fact]**

- 白皮书五状态机文本与 5 个 transition table 的主要边一致；
- transition table 自身结构闭合；
- 但 `effect.schema.json` 没有 `reconciliation_result=executed|not_executed|still_unknown`，无法表达 effect table 的关键 guard；
- `verification-report.schema.json` 没有 `expired`，而 Verification transition 有 `EXPIRED`；
- `TaskContract.allowed_state_domains` 仍是 `world|task|agent|session`，与 registry 的 `agent-execution|task|loop|effect|verification` 及开放 state-domain 原则不一致；
- `EFFECT-STATE-CLOSURE-008` 的 “allowed exits” 容易被解释为 `OUTCOME_UNKNOWN` 可直接进入 compensation/quarantine，而 transition table 要求先 `RECONCILED`；
- 30 个业务 schema 使用 legacy metadata，7 个新 schema使用 GovernedObjectHeader，形成双轨。

### 3.6 Profile 与实现状态

**[confirmed fact]** 仓库没有 profile manifest 实例、实现 commit、执行证据或 performance report 实例。所有 “implemented” 可能性仍为空。schema/vector 已登记、测试已执行、Profile 已符合是四个不同事实；当前只有第一类。

---

## 4. 已确认的合理设计

只保留有直接内部和外部证据支持的设计。

1. **[evidence-backed judgment] 观察、提议、授权、执行、验证、验收分离是必要边界。**  
   内部：架构 §2.3、§4.6、Core §2/§9。外部：NIST SP 800-207 的逐资源认证授权原则；OWASP Agent Security 的工具最小权限。边界：分离本身不证明各 authority 实现正确。

2. **[evidence-backed judgment] `OUTCOME_UNKNOWN`、稳定幂等键和 reconciliation 应保留。**  
   内部：Core §9、Effect transition、`RECOVERY-CRASH-006`。外部：RFC 9110 只允许对已知幂等语义安全自动重试；Stripe 明确同 key 异参报错。边界：不可查询执行器仍只能隔离或人工处置。

3. **[evidence-backed judgment] 事件提交、World authority 与观察证据三分是正确的。**  
   内部：架构 §8。外部：ARIES 支持 WAL/重放，但不会把外部世界自动回滚。边界：需要原子 state+event/outbox 实现证据。

4. **[evidence-backed judgment] Tenant/网络位置不等于授权。**  
   内部：RFC-0001 §3。外部：NIST SP 800-207 明确不因网络位置或资产所有权隐式信任。边界：当前 legacy schema 尚未普遍携带这些治理维度。

5. **[evidence-backed judgment] Context 在 ranker/模型前授权、控制与数据分离应保留。**  
   内部：架构 §9.2—§9.3、RFC-0001 §11。外部：CaMeL（SaTML 2026）与 OWASP 均支持在工具/数据流边界做确定性约束。边界：ContextView schema 当前仍允许 `untrusted+control` 组合。

6. **[evidence-backed judgment] 无模型的 Management API/CLI fallback 是正确的。**  
   内部：Core `REQ-MGMT-FALLBACK-001`、架构 §12.8。边界：当前无实现，审批通道新语义未登记。

7. **[evidence-backed judgment] SMS/CRB、多 Agent 拓扑、检索和 consolidation 留在 Core 外是正确方向。**  
   内部：架构 §7、§10、§15。外部：MAST 显示多 Agent 的系统设计、协调和验证失败并不会由更多 Agent 自动消失。边界：Core 目前仍通过对象和 Profile 表面间接承担了过多可选复杂度。

8. **[evidence-backed judgment] 形式化范围应聚焦 Effect/recovery/fencing。**  
   外部：AWS TLA+ 经验显示模型检查能发现测试/评审遗漏的罕见并发组合；seL4 同时说明形式证明有明确假设边界。形式模型不能替代实现 fault tests。

---

## 5. 架构缺陷与风险

### F-001 — 无实现、runner 或已执行证据

- **严重度**：P0
- **结论类型**：[confirmed fact]
- **证据**：`conformance/README.md` 明示无 runner；仓库没有 Rust/TS/runtime 源码或 profile manifest。
- **失败方式**：声明式 expected 字段可能与实现完全无关，仍被误读为“测试覆盖”。
- **影响**：不能证明正确运行、非劣化或收益；不能标 implemented。
- **建议方向**：先建最小 runner 和故意错误 adapter，再实现 tracer bullet。
- **置信度**：高。

### F-002 — 17 个规范要求由 informative 白皮书拥有

- **严重度**：P0
- **结论类型**：[confirmed fact]
- **证据**：requirements registry 的 17 个 `owner_spec=CognitiveOS-Architecture.md#...`；白皮书首部声明 informative；normative-source 标准 §2—§4。
- **失败方式**：实现者无法判断白皮书修辞、Core 文本或 transition table 哪个定义真实行为。
- **影响**：crash、fencing、profile、performance 等核心语义可发生无版本漂移。
- **建议方向**：将行为正文迁到 Core/Profile/标准，registry owner 改为 normative asset；白皮书只反向引用。
- **置信度**：高。

### F-003 — 两套不兼容的对象头和强引用并存

- **严重度**：P0
- **结论类型**：[confirmed fact]
- **证据**：30 个 schema 引用 legacy `common-defs.metadata`；22 个引用 legacy `strongRef`；新标准明确 legacy ref 不是 `object-reference.strongReference`。
- **失败方式**：Intent、Effect、Event、TaskContract、ContextView 等对象可在 schema-valid 情况下没有 tenant、ResourceScope、policy、purpose、retention 或 compartment。
- **影响**：同 tenant 横向越权、跨 Conversation 污染和审计缺口无法由机器合同阻止。
- **建议方向**：选一个 v0.1 governed header/ref，迁移全部 Core 竖切 schema；禁止隐式桥接。
- **置信度**：高。

### F-004 — Context 创建链存在循环且请求缺治理字段

- **严重度**：P0
- **结论类型**：[confirmed fact]
- **证据**：`activity-context.schema.json` 必需 `context_view_ref`；架构/RFC 又要求 ContextView 绑定 ActivityContext；`context-request.schema.json` 没有 TenantContext、ActorChain、Conversation/ResourceScope、policy/membership/revocation 或 ActivityContext。
- **失败方式**：实现必须发明 draft Activity、placeholder View 或绕过绑定；不同实现会产生不同授权时点。
- **影响**：最小 `Task→Context` 竖切无法由现有 machine contract 唯一构造。
- **建议方向**：引入 `ActivityBinding`/`ContextResolutionContext` 前置对象，或让 Activity 两阶段创建并以 CAS 绑定 View。
- **置信度**：高。

### F-005 — Effect、Verification 和 acceptance 存在重复真相与可构造非法状态

- **严重度**：P0
- **结论类型**：[confirmed fact]
- **证据**：Effect enum 内含 `VERIFIED/VERIFY_FAILED`，另有 Verification machine；Effect 内嵌 verification status，另有 VerificationReport；静态反例 `COMMITTED+unknown+pending` schema-valid。
- **失败方式**：三个位置对同一验证结论发生分歧；replay 或投影器选择不同来源。
- **影响**：`OUTCOME_UNKNOWN` 可能在对象层看似 committed，receipt/verification 可绕过 acceptance。
- **建议方向**：只保留一个 verification authority；Effect 引用 report/decision，不复制状态，或以条件 schema封闭合法组合。
- **置信度**：高。

### F-006 — `ABORTED` 不能说明已执行外部世界是否仍有残留

- **严重度**：P0
- **结论类型**：[evidence-backed judgment]
- **证据**：effect transition 允许 `VERIFY_FAILED→ABORTED`，无需 compensation 或“world restored”证据。
- **失败方式**：外部删除/付款已执行，后置验证失败，Effect 进入 terminal ABORTED；下游把“closed”误解为未发生或已回滚。
- **影响**：取消、Task 失败和恢复可能放过仍存在的真实副作用。
- **建议方向**：分离 execution outcome 与 local disposition；已执行但未接受必须是 `RESIDUAL_EFFECT`、`COMPENSATION_PENDING` 或 `QUARANTINED`。
- **置信度**：高。

### F-007 — Context 后撤销 capability 的竞态未闭合

- **严重度**：P0
- **结论类型**：[confirmed fact]
- **证据**：`AUTHORIZED→EXECUTING` guard 只有 fencing/idempotency；`VERIFIED→COMMITTED` 只有 verification/version/authority；没有 revocation/capability-current guard。
- **失败方式**：授权后、dispatch 前撤销仍可能执行；执行后、commit 前撤销语义也不唯一。
- **影响**：违反用户明确要求的撤销后阻止 Effect。
- **建议方向**：规定 dispatch 和 commit 的重新判定点、revocation snapshot 与“执行已发生但提交被拒”的出口。
- **置信度**：高。

### F-008 — 数据库只读、磁盘满和持久化失败没有唯一安全语义

- **严重度**：P0
- **结论类型**：[confirmed fact]
- **证据**：errors registry、Effect transition、recovery 章节均无 durable-write failure 状态或 dispatch 禁止规则。
- **失败方式**：Intent/dispatch event 写失败后仍调用外部执行器；receipt 又无法落盘。
- **影响**：WAL 不变量失效，恢复无法区分未调用与已调用。
- **建议方向**：定义 `DURABILITY_UNAVAILABLE`、pre-dispatch fsync/transaction barrier、post-dispatch emergency spool 和只读管理模式。
- **置信度**：高。

### F-009 — canonical digest/signature 缺少自引用投影

- **严重度**：P0
- **结论类型**：[confirmed fact]
- **证据**：canonical standard §10 要求显式 `digest_excluded`；仓库搜索无任何 projection/exclusion；大量 schema 必需 `content_digest` 或 signature。
- **失败方式**：对象必须先包含自身 digest 才能计算自身 digest。
- **影响**：强引用、签名、schema bundle、proposal/approval digest 无法按自身标准生成。
- **建议方向**：为每个对象族登记 digest projection schema/domain/excluded paths 和 golden fixture。
- **置信度**：高。

### F-010 — Checkpoint 不能机器表达安全恢复所需稳定事实

- **严重度**：P0
- **结论类型**：[confirmed fact]
- **证据**：`loop-checkpoint.schema.json` 缺 event high-watermark、typed pending Effect、fencing epoch、policy/schema/renderer/model/verifier/environment 版本、reconciliation disposition。
- **失败方式**：旧 checkpoint 在 schema/policy升级后恢复，执行器只能依赖实现私有状态。
- **影响**：无法证明恢复不跳步、不依赖隐藏态。
- **建议方向**：登记通用 Checkpoint envelope 与 compatibility/migration decision。
- **置信度**：高。

### F-011 — 高风险审批安全基线仍是 informative

- **严重度**：P0
- **结论类型**：[confirmed fact]
- **证据**：架构 §12.12 明示 secure channel、anti-fatigue、R3 双人审批的 REQ/vector “另行提交”；当前 approval schema 只能表达单个 decision。
- **失败方式**：假审批界面、审批轰炸或单人批准 R3 在 machine contract 中没有强制拒绝。
- **影响**：R2/R3 管理与具身路径不能发布。
- **建议方向**：登记 approval request/delivery/authenticator/quorum/replay/rate-limit 状态和安全负例。
- **置信度**：高。

### F-012 — TaskContract schema 与开放 state domain、验收语义不一致

- **严重度**：P1
- **结论类型**：[confirmed fact]
- **证据**：`allowed_state_domains` 是旧四值 enum；schema 允许只有 `wait` 条件、没有 acceptance criterion 的 TaskContract，静态反例通过。
- **失败方式**：Task 可被创建却没有可判定完成条件；Effect/verification domain 不能被声明。
- **影响**：bounded Loop 和 acceptance authority 无可移植合同。
- **建议方向**：state domain 使用 registry reference；至少一个 acceptance 条件并固定 verifier/authority。
- **置信度**：高。

### F-013 — 通用幂等冲突语义未登记完整

- **严重度**：P1
- **结论类型**：[confirmed fact]
- **证据**：AKP §6 使用 `IDEMPOTENCY_CONFLICT`，errors registry 无该 code；同 key 异参的行为 vector 只存在于 management 场景。
- **失败方式**：普通 Tool/Effect adapter 对同 key 异参可能返回旧结果、创建新 Effect 或任意错误。
- **影响**：跨实现重试语义不一致。
- **建议方向**：登记通用 key scope、retention、参数摘要冲突错误和 behavior vector。
- **置信度**：高。

### F-014 — fencing 未证明覆盖所有提交端

- **严重度**：P1
- **结论类型**：[evidence-backed judgment]
- **证据**：Effect/Capability schema 的 `fencing_token` 可选；缺少 state store、event store、executor、outbox、memory publish、approval、device 各写端矩阵和行为测试。
- **失败方式**：旧 writer 被状态库拒绝，却仍可写外部 executor 或 outbox。
- **影响**：网络分区后双写。
- **建议方向**：发布 commit-sink inventory，并逐端要求 epoch/fence 或明确 not-applicable。
- **置信度**：高。

### F-015 — vector 数量不能代表行为覆盖

- **严重度**：P1
- **结论类型**：[confirmed fact]
- **证据**：175/270 REQ 只映射 generic trace vector；21 vector 仅 owner/status；`EFF-CRASH-001/002/003` 三个文件本身是相同 traceability 模板，真实三点场景在另一个 aggregate vector 中。
- **失败方式**：runner 可只验证文件存在就“覆盖”大多数 MUST。
- **影响**：符合性声明被路径闭合伪装。
- **建议方向**：每个安全/行为 MUST 至少有 observable positive/negative，generic vector 只算静态 lint。
- **置信度**：高。

### F-016 — Profile manifest 的 implemented 语义可被 degradation 稀释

- **严重度**：P1
- **结论类型**：[confirmed fact]
- **证据**：conformance README 允许 implemented “passing evidence 或 documented degradation”；schema 无 `not-applicable`、逐 REQ result 和适用性依据。
- **失败方式**：关键 MUST 失败仍以 degradation 列出并保留 implemented。
- **影响**：发布声明不满足“全部适用 MUST 有行为证据”。
- **建议方向**：implemented 只允许 pass/not-applicable；degradation 自动缩小 scope 或降为 experimental。
- **置信度**：高。

### F-017 — sandbox 不可绕过证据过于粗粒度

- **严重度**：P1
- **结论类型**：[confirmed fact]
- **证据**：AgentCompatibilityReport 的 features 是任意 map；`AGENT-BYPASS-002` 只有一行复合 scenario；没有 OS/kernel/version、网络、secret、subprocess、IPC、MCP、device 的逐项证据。
- **失败方式**：Linux namespace 通过被错误外推到 Windows，或网络被拦截但 IPC/凭证代理仍可绕过。
- **影响**：C1/C2 不可移植。
- **建议方向**：host-specific interception matrix 和逐通道 escape tests。
- **置信度**：高。

### F-018 — ContextView shape 允许控制/信任矛盾

- **严重度**：P1
- **结论类型**：[confirmed fact]
- **证据**：`trust_level` 与 `role` 无条件关联；`untrusted+control` 反例 schema-valid；loss declaration 缺 sensitivity、适用范围、冲突/未知保留和可回取性。
- **失败方式**：被注入内容以 control role 渲染，或压缩丢失关键冲突而 schema 仍通过。
- **影响**：prompt-injection 与信息损失门禁依赖实现私有规则。
- **建议方向**：条件 schema + behavior guard；补齐 loss machine contract。
- **置信度**：高。

### F-019 — read-your-write 隔离不可由 MemoryCandidate 表达

- **严重度**：P1
- **结论类型**：[confirmed fact]
- **证据**：candidate schema 没有 writer principal、Activity、Conversation、visibility lease/TTL、trust role；vector 仅自然语言 scenario。
- **失败方式**：pending candidate 被错误加入同 tenant 全局索引，或作为 control memory 反馈给写入者。
- **影响**：跨 Conversation 泄露和持久注入。
- **建议方向**：登记 private working-set binding、untrusted role、expiry 和 admission failure disposition。
- **置信度**：高。

### F-020 — performance report 不足以支撑 A/B、消融和非劣效结论

- **严重度**：P1
- **结论类型**：[confirmed fact]
- **证据**：baseline/ablation 只是字符串；无 arm/result linkage、effect size、paired delta、power、randomization、non-inferiority、工具/权限/预算固定、逐失败样本；删除 governance_overhead 后仍 schema-valid。
- **失败方式**：报告可只有一个 context latency metric，却声称 Agent 收益或治理可忽略。
- **影响**：`REQ-PERF-004` 与显著收益门槛不可机器审核。
- **建议方向**：新 major/minor schema，显式 A/B/C/D arms、比较结果和失败样本。
- **置信度**：高。

### F-021 — `REQ-CTX-012` 没有真正保证缓存前缀稳定

- **严重度**：P1
- **结论类型**：[evidence-backed judgment]
- **证据**：REQ 只要求已有条目相对顺序不变；Anthropic 官方缓存按 cache breakpoint 之前**完整累计前缀 hash**匹配。
- **失败方式**：新增对象按排序插入旧对象之前，相对顺序未变但所有后续字节偏移，缓存 miss。
- **影响**：所谓 cache-hit preservation 不能由当前 vector 推出。
- **建议方向**：固定 immutable prefix segments、append-only suffix 或明确 cache breakpoint digest。
- **置信度**：高。

### F-022 — SLO 只有容器，没有阈值和发布判定

- **严重度**：P1
- **结论类型**：[confirmed fact]
- **证据**：`slo_profile` 只要求 id/version/window；无阈值、error budget、risk stratum、release gate。
- **失败方式**：任意高 p99/unknown rate 仍 schema-valid。
- **影响**：不能阻断性能回归。
- **建议方向**：SLO profile 单独 schema，引用 metric、阈值、窗口、分层和 fail action。
- **置信度**：高。

### F-023 — 不可查询/不可幂等执行器缺少准入规则

- **严重度**：P1
- **结论类型**：[unresolved question]
- **证据**：架构 §22 将其列为开放问题；Descriptor 可声明 query/idempotency，但没有按 risk/effect class 的拒绝矩阵。
- **失败方式**：R1/R2 Operation 无法对账却仍被授权，故障后永久 unknown。
- **影响**：人工隔离无限增长，恢复闭环失效。
- **建议方向**：默认拒绝；只有明确 residual-risk authority、低 blast radius 和人工处置预算时例外。
- **置信度**：高。

### F-024 — 四视图映射仍遗漏 owner，Core 边界不一致

- **严重度**：P1
- **结论类型**：[evidence-backed judgment]
- **证据**：新增 Approval Service 未进入 §4.7 责任矩阵；commit/event store、acceptance authority、audit authority 无明确组件 owner；Shell 同时称 Core 参考客户端又可降级。
- **失败方式**：团队把 approval、verification、Context policy 或 event store分别实现在 Harness、Shell 和 kernel，形成多个 authority。
- **影响**：循环依赖和共享状态。
- **建议方向**：建立 view→component→authority→interface 唯一映射，客户端不属于 semantic Core。
- **置信度**：中高。

### F-025 — 当前 OS 身份缺少不可绕过实现证据

- **严重度**：P1
- **结论类型**：[evidence-backed judgment]
- **证据**：严格 OS 判据写在 §3，但无 reference monitor、host adapter、device mediation 或 sandbox evidence。
- **失败方式**：系统最终只包装 workflow/database/IAM/MCP，而无法阻止 legacy Agent 旁路。
- **影响**：OS 品牌超过可证明边界。
- **建议方向**：发布前以 “control plane/runtime” 声明；通过 host-specific mediation 后再声称 OS。
- **置信度**：中高。

### F-026 — Agent 性能收益仍处于 E0 假设

- **严重度**：P1
- **结论类型**：[confirmed fact]
- **证据**：无性能报告实例；前一评审关于“门禁 µs–ms、缓存失效 2–10×”没有仓库测量或固定 workload。
- **失败方式**：局部 recall/cache 指标改善被宣传为端到端任务提升。
- **影响**：路线图和采用决策被不可归因数字驱动。
- **建议方向**：按本报告 §11 的 A/B/C/D 与 kill criteria 评测。
- **置信度**：高。

### F-027 — 规范表面积仍显著大于最小 Core

- **严重度**：P2
- **结论类型**：[evidence-backed judgment]
- **证据**：约 80 个命名对象、270 REQ、15 测试层；当前实现为零。
- **失败方式**：先生成对象/适配器而不是跑通一条 Effect/recovery 竖切。
- **影响**：实现反馈晚、迁移成本上升。
- **建议方向**：缩为 10—14 个对象、3 个 authority 状态机和一个单节点存储事务。
- **置信度**：高。

### F-028 — 外部证据分类和版本有漂移

- **严重度**：P2
- **结论类型**：[confirmed fact]
- **证据**：白皮书仍链接已被 SP 800-63B-4 取代的 800-63B；IETF draft、CISA 指南、MITRE ATT&CK、EBA regulation 被统一标 `[STD]`；CaMeL 截至审查日已有 SaTML 2026 发表信息。
- **失败方式**：读者把草案/指南/法规当成同等级稳定标准。
- **影响**：证据权重失真。
- **建议方向**：拆分 `[STD]/[DRAFT]/[REG]/[GUIDE]/[PEER]/[PRE]/[IND]`。
- **置信度**：高。

### F-029 — 当前开发计划已归档，工程决策链不完整

- **严重度**：P2
- **结论类型**：[confirmed fact]
- **证据**：History 明确开发计划不参与当前规范；仓库只有 4 个标准和 2 个 ADR，而归档计划列出 13 个 ADR/标准族。
- **失败方式**：实现启动时没有当前、受版本控制的里程碑和 ADR 入口。
- **影响**：白皮书路线图无法替代存储、语言、transport、clock、sandbox 决策。
- **建议方向**：P0 收敛后另立当前实施计划；不得恢复旧计数为事实。
- **置信度**：高。

### F-030 — 高级 Profile 应继续延后

- **严重度**：P2
- **结论类型**：[evidence-backed judgment]
- **证据**：distributed/multi-agent/embodied/heterogeneous/learning 均无实现；MAST 显示多 Agent 增益常被协调和验证失败抵消。
- **失败方式**：高级 Profile 反向迫使 Core 接受尚未验证的对象和状态。
- **影响**：Core 被研究假设绑架。
- **建议方向**：保持 planned/experimental；单节点 benchmark 后逐个引入。
- **置信度**：高。

---

## 6. OS 边界与最小 Core 判定

### 6.1 OS 定位

**[evidence-backed judgment] 结论：OS 定位部分成立。**

| 相邻系统 | 应由相邻系统承担 | CognitiveOS 只有在何处增加独特价值 |
|---|---|---|
| 宿主 OS/容器 | 进程、文件、网络、设备、内存隔离 | 把 Agent 身份、capability、Effect 与宿主 enforcement 绑定并证明不可绕过 |
| Workflow engine | durable step、timer、retry、saga | 不确定决策、Context 授权、unknown outcome、acceptance authority 的统一语义 |
| 数据库/日志 | 事务、WAL、索引、复制 | state+event+outbox 的领域不变量；不重造存储 |
| IAM/policy engine | 认证、RBAC/ABAC/ReBAC | ActorChain、Task/Activity purpose、Effect 参数与短期 capability 的交集 |
| MCP/A2A | 工具/Agent 互操作 | 本地重新授权、descriptor≠capability、remote completed≠acceptance |
| Observability | trace/metric/log | 权威提交、安全审计、可丢遥测三分 |
| Agent runtime | 模型、prompt、tool loop | 跨 runtime 的持久身份、受治理 Context、Effect/recovery/acceptance |

必须由 CognitiveOS 统一承担的跨组件不变量只有：authority/acceptance 分离、Intent-before-effect、stable idempotency binding、reconcile-before-success、revocation/fencing、Context-before-disclosure、state+event atomicity、recovery ordering。其余大多可以是 adapter、Profile 或策略。

### 6.2 四视图一致性矩阵

| 视图 | 组件映射 | authority | 接口 | 当前冲突/缺口 |
|---|---|---|---|---|
| 双内核：认知 | Kernel、state/event store、authz、Effect、recovery | state/effect/execution authorities | Core operations | 范围过大；Context policy 与 mechanism 边界需代码级端口 |
| 双内核：实时安全 | safety controller/watchdog | safety authority | bounded setpoint/emergency | 只适用于 R3，不应约束单节点 Core |
| 体验平面 | deterministic CLI、可选 Shell、Task UI | 无；仅调用 intent/acceptance authority | Management/Task API、watch | Shell 被列作 Core，易误持 authority |
| 控制平面 | authority registry、policy、Context gate、scheduler、approval | 各域 authority | CAS/authz/context/effect/approval | Approval Service 未进入唯一责任矩阵 |
| 执行/数据平面 | runtime、executor、object store、outbox、device | executor 只报告；world authority另立 | invoke/query/reconcile | store/outbox owner 不显式 |
| 七层 1—3 | host/resource/operation | host/safety/operation local authority | adapters | sandbox evidence按平台缺失 |
| 七层 4 | microkernel/AKP | protocol and commit authorities | Core syscalls | AKP machine envelope 未登记 |
| 七层 5 | Context/Memory/Knowledge/Catalog | 各对象 authority | discover/read/admit/bind | Core 与 Profile 被合并在一层 |
| 七层 6 | Harness/SMS/CRB/verifier selection | Harness/SMS/CRB 无 authority | candidate/proposal | verifier selection 与 verification authority需区分 |
| 七层 7 | Agent/application | intent authority 仅在明确用户委托时 | task/context candidates | Agent 不能自验、自批 |
| 横切 Context | filter/auth/budget/render/audit | 读取 authority仍在资源域 | resolve/delta | Activity/View 创建循环，schema 双轨 |

### 6.3 建议的最小单节点 R0/R1 Core

**[evidence-backed judgment]** 建议只保留：

1. GovernedHeader/StrongRef 单一合同；
2. Principal + fixed single-tenant GovernanceContext；
3. AgentExecution；
4. Task + TaskContract + AcceptanceDecision；
5. append-only committed Event + StateSnapshot/CAS；
6. ContextResolutionContext + ContextRequest/View 的确定性白名单实现；
7. static OperationDescriptor registry；
8. AuthorizationDecision/Capability；
9. Intent + Effect + idempotency record；
10. VerificationReport；
11. Checkpoint + recovery barrier；
12. security audit reference。

建议把当前五个 authority lifecycle machine 缩为：

- AgentExecution；
- Task；
- Effect（内部再分 execution outcome 与 disposition）；
- Loop 仅作为 Harness checkpoint phase，不是独立 authority domain；
- Verification 使用不可变 report + invalidation/supersession，不再复制到 Effect 状态。

如果保留五状态机，则必须删除 Effect 中重复 verification 状态，并定义跨域原子/因果约束。

### 6.4 可选、下沉、延后或删除

| 能力 | 建议 |
|---|---|
| Context Resolution | 保留确定性 gate；检索/rank/transform 下沉策略 |
| CVM/page/fault 术语 | R0/R1 删除；需要真实 paging benchmark 后再启用 |
| Management API/CLI | 保留为运维最小面；自然语言 Shell 可选客户端 |
| Agent Packaging | 下沉 adapter/Profile；不阻塞首条原生 tracer |
| Governed Memory | 延后到 Effect/recovery 竖切稳定后 |
| Discovery/Catalog | Core 只保留静态 descriptor lookup；两级发现为 Profile |
| SMS/CRB | 延后并保持可关闭 |
| Multi-Agent | 延后到单 Agent A/B 后 |
| Embodied/Heterogeneous/CIM | 独立安全/硬件项目 |
| Controlled Learning | 独立发布系统，不进入首版 Core |

### 6.5 建议 tracer bullet

```text
Create Task/Contract
  -> create ActivityBinding
  -> resolve deterministic ContextView
  -> persist Intent + PROPOSED Effect
  -> authorize against current revocation/epoch
  -> atomically persist DISPATCHED before external call
  -> execute/query with stable (scope,key,params_digest)
  -> persist reconciliation outcome
  -> create VerificationReport against fixed post-state
  -> authority commits state + Event + outbox atomically
  -> acceptance authority completes Task
  -> crash at every boundary and recover from durable facts
```

### 6.6 首个单节点实现复杂度估算

以下均为 **[hypothesis]**，用于控制范围，不是工期承诺：

| 项目 | 最小建议 | 按当前全量规范 |
|---|---:|---:|
| 核心对象族 | 10—14 | 约 80 个命名对象 |
| authority 状态机 | 3 | 5 |
| 状态数 | 25—35 | 54 |
| transition | 40—60 | 90 |
| 持久表/对象族 | 12—18 | 25+ |
| append-only 逻辑日志 | 2（commit、audit）+ outbox | 3+ |
| 每个 governed external Effect 同步事务 | 5—7 | 可能 8—12 |
| Core 接口 | 10—12 | Core 文本 15+、生态扩展更多 |
| 首版 adapter | 3—4 | 标准接口族 6 |
| 首版 conformance 层 | 1—6 的适用子集 | 15 |
| 首版安全负例 | 25—35 | 尚未形成完整清单 |
| fault injection | 15—20 | 本报告 32 场景只是上界起点 |
| 形式模型 | Effect/recovery/fence 1 个模型 | 全系统不可行 |

关键路径是：本地 durable transaction → 外部执行器 → reconcile/query → verifier → state/event commit。主要必要复杂度来自外部结果未知、并发版本和撤销；主要非必要复杂度来自高级 Profile、双对象合同、重复状态和过早协议表面。

---

## 7. 组件完备性矩阵

符号：`✓`=当前职责与行为边界较清楚；`△`=有 prose，但机器合同/恢复/测试不完整；`✗`=当前缺失或冲突。括号引用 Finding。

### 7.1 核心组件合同摘要

| 组件 | 目的/owner | 输入→输出 | 持久对象/状态机 | 关键缺口 |
|---|---|---|---|---|
| Identity/Governance | identity/policy authority | principal/context→decision | Principal、ActorChain、bindings | 双 header/ref（F-003） |
| State/Event/CAS | domain authority | expected version mutation→snapshot/event | transition record、Event | store failure 未定义（F-008） |
| Task/Acceptance | intent/acceptance authority | intent/contract/evidence→terminal decision | Task machine | contract 无 acceptance 强制（F-012） |
| Context | resource authority + deterministic gate | request/bindings→View/error | Request/View | 启动循环、治理字段缺（F-004） |
| Operation/Authz | catalog/resource authority | descriptor+intent→decision | Capability/decision | revocation dispatch race（F-007） |
| Effect | effect/commit authority | Intent→outcome/disposition | Effect machine/idempotency | 非法组合、residual effect（F-005/F-006） |
| Verification | verification authority | criteria+post-state→report | Verification machine/report | 与 Effect 重复（F-005） |
| Harness/Loop | execution authority | evidence→candidate action | checkpoint phase | Checkpoint 不足（F-010） |
| Recovery | recovery/execution authority | checkpoint/log/pending→safe runnable | barrier/epoch | durability与升级语义缺 |
| Management/Approval | management/approval authority | proposal/challenge→decision | session/proposal/approval | 高风险机制未登记（F-011） |
| Packaging/Sandbox | installation authority + host enforcement | package→installation/profile | install lifecycle | 平台证据粗（F-017） |
| Memory/Discovery/Catalog | object authorities | candidate/discover→admit/bind | Profile objects | RYW machine binding缺（F-019） |
| SMS/CRB | 无 authority | candidate→candidate/allocation | 可选记录 | 必须可关闭 |
| Distributed/Multi-Agent | domain/local authorities | delegation/message→evidence | lease/mailbox/conflict | sink fence与协调证据缺 |
| Safety/Heterogeneous | safety/resource authorities | bounded setpoint/placement→effect | Profile states | 无设备证据 |
| Learning | release authority | candidate→release/rollback | release lifecycle | 无实现，必须延后 |

### 7.2 完备性维度

| 组件 | authority | state | concurrency | failure | recovery | security | observability | performance | compatibility | test evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| Identity/Governance | ✓ | △ | △ | △ | △ | ✗ F-003 | △ | ✗ | ✗双合同 | △ |
| State/Event/CAS | ✓ | ✓ | ✓ | △ | △ | △ | ✓ | △ | △ | △无 runner |
| Task/Acceptance | ✓ | ✓ | △ | △ | △ | △ | ✓ | △ | ✗ F-012 | △ |
| Context | ✓ | △ | △ | △ | △ | ✗ F-004/F-018 | ✓ | △ | ✗ | △ |
| Operation/Authz | ✓ | △ | △ | ✗ F-007 | △ | ✗ | △ | △ | △ | △ |
| Effect | ✓ | ✗ F-005/F-006 | △ | ✗ | △ | △ | ✓ | △ | ✗ | △ |
| Verification | ✓ | ✗ F-005 | △ | △ | △ | △ | △ | ✗ | △ | △ |
| Harness/Loop | ✓ | ✓ | △ | △ | ✗ F-010 | △ | ✓ | △ | △ | △ |
| Recovery | ✓ | △ | △ | ✗ F-008 | ✗ | △ | △ | △ | ✗升级 | △ |
| Management/Approval | ✓ | △ | △ | △ | △ | ✗ F-011 | △ | △ | △ | △ |
| Packaging/Sandbox | ✓ | △ | △ | △ | △ | ✗ F-017 | △ | ✗ | ✗平台 | △ |
| Memory/Discovery/Catalog | ✓ | △ | △ | △ | △ | ✗ F-019 | △ | △ | △ | △ |
| SMS/CRB | ✓无 authority | n/a | △ | ✓fallback prose | n/a | △ | △ | ✗ | △ | △ |
| Distributed/Multi-Agent | △ | △ | △ | △ | △ | △ F-014 | △ | ✗ | △ | ✗ |
| Safety/Heterogeneous | ✓ prose | △ | △ | △ | △ | △ | △ | ✗ | ✗硬件 | ✗ |
| Learning | ✓ prose | △ | △ | △ | △ | △ | △ | ✗ | △ | ✗ |

**[evidence-backed judgment]** 只有 State/Event/CAS 的抽象接近可实现合同；其余 Core 组件至少缺一个 P0 维度。性能预算、兼容迁移和行为证据是最普遍的空白。

---

## 8. 故障场景推演

“唯一规定”指当前 normative 资产能否给出唯一安全结果；“安全集合”表示允许多个策略，但都在明确 fail-closed 边界内。

| # | 事件序列 | 预期状态 | authority | 允许/拒绝 | 所需证据 | 恢复出口 | 当前是否唯一 |
|---:|---|---|---|---|---|---|---|
| 1 | Intent durable→调用前崩溃 | Effect 保持 PROPOSED/AUTHORIZED，未执行 | effect/execution | 允许原 key 首次 dispatch | Intent、无 dispatch、key/params | 新 epoch 后 dispatch | 是；`RECOVERY-CRASH-006` |
| 2 | 外部已执行→receipt 前崩溃 | OUTCOME_UNKNOWN | effect/world | 拒绝成功和新 key | dispatch、executor query/observation | reconcile→verify 或 quarantine | 安全集合唯一 |
| 3 | Verification 后→commit 前崩溃 | report durable，Effect 未 committed | commit/acceptance | 只允许重验 current 后 commit | report、post-state、expected version | 不重执行，CAS commit | prose/vector有；Checkpoint不足，P0 F-010 |
| 4 | Effect 重复投递 | 同 key/同参返回同 Effect/等价结果 | executor/effect | 拒绝重复真实效果 | key scope、params digest、first result | dedupe/reconcile | 通用合同不完整，P1 F-013 |
| 5 | Effect 乱序 | 非法 transition 拒绝、状态不变 | effect authority | 拒绝 | table digest、current state/version | 返回安全出口 | 是，需 runner |
| 6 | Effect 超时 | OUTCOME_UNKNOWN | effect authority | 禁止视为未执行/成功 | dispatch、timeout boundary | query、原 key、compensate、quarantine | 是 |
| 7 | 同 key 异参 | 不得复用旧结果 | effect/executor | 拒绝 | key scope、两个 params digest | 新业务意图需新 key | AKP规定但 error 未登记，P1 |
| 8 | 外部结果不可查询 | still_unknown | effect/risk authority | 禁止 commit；例外准入未定义 | Descriptor、查询失败、风险决定 | quarantine/独立补偿/人工 | 安全出口有，准入不唯一，P1 F-023 |
| 9 | 两 writer 并发 CAS | 一成功、一 STATE_CONFLICT | state authority | 只允许一个 | expected/current version、atomic event | loser refresh/retry | 是 |
| 10 | 旧 epoch writer 网络分区后继续 | stale write denied | 每个 commit sink | 拒绝 | epoch/fence at sink | 新 authority reconcile | 原则有，覆盖端不完整，P1 F-014 |
| 11 | Context 后 capability 被撤销→Effect | 不应 dispatch；若已执行则 unknown/residual | auth/effect | 应拒绝新 dispatch/commit | revocation epoch、decision time、dispatch time | reauthorize 或 quarantine | **否，P0 F-007** |
| 12 | 同 tenant 横向读取 | denial，不泄露存在性 | resource authority | 拒绝 | principal/relation/scope/purpose | 显式授权/ShareGrant | 行为唯一；专用 vector 不足 |
| 13 | 跨 Conversation KV/working-set 污染 | binding mismatch、清空/隔离 | conversation/resource | 拒绝复用 | Conversation binding、cache key | 新 View/cache | 是，机器覆盖不足 |
| 14 | 管理员未经授权读正文 | deny；break-glass 独立 | resource/break-glass | 默认拒绝 | body authorization、break-glass decision | 限时最小披露 | 默认唯一；break-glass 协议待决 |
| 15 | ranker 授权前接触敏感正文 | Context auth denied | resource authority | 拒绝 | candidate filter/audit、egress decision | 本地授权后 rank | 是 |
| 16 | renderer/工具描述提示注入 | 保持 untrusted data；无 capability | auth/effect | 拒绝控制变更 | role/provenance、structured proposal | 安全停止/重解析 | prose唯一；schema反例，P1 F-018 |
| 17 | required context 超预算 | CONTEXT_BUDGET_EXCEEDED | Context gate | 拒绝，除非预授权 partial | required set、budget、partial policy | wait/escalate/stop | 是 |
| 18 | required 来源过期/冲突/不可访问 | incomplete/conflict | Context/Task authority | 禁止静默成功 | freshness、conflict、access result | wait/escalate/authorized partial | 安全集合有；具体策略不唯一 |
| 19 | Agent 称 done、后态不满足 | Task 非 COMPLETED | acceptance authority | 拒绝完成 | fixed post-state、failed/inconclusive report | ACTIVE/BLOCKED/FAILED | 是 |
| 20 | remote Task completed、本地验证失败 | local non-complete | local acceptance | 拒绝映射 completed | remote evidence、本地 report | retry/escalate/fail | 是 |
| 21 | Shell 断开 | Task/Effect继续；观察断开 | task/effect authority | 不隐式取消/提交 | stable refs、cursor、auth | reauth attach | 是 |
| 22 | 审批轰炸 | 聚合/限流/告警，不能自动批准 | approval service | 拒绝超限/重放 | source、proposal digest、rate state | cool-down/escalation | **否；仅 informative，P0 F-011** |
| 23 | 旧 proposal 被批准 | digest/version/expiry mismatch | approval authority | 拒绝 | proposal/preview/session digest | 新 preview/challenge | 是 |
| 24 | Agent 伪造审批界面 | 无 authority signature/challenge | approval authority | 拒绝 | trusted surface/authenticator binding | 独立通道重做 | **行为文本有，机器合同无，P0** |
| 25 | legacy Agent 直接改 memory 文件 | digest drift→candidate/conflict | memory authority | 禁止静默采用/覆盖 | old/new digest、lineage | admission/reconcile | 是，`AGENT-OOB-001` |
| 26 | 绕过 tool proxy/network/secret/subprocess/IPC | install/runtime denial 或降级 C profile | host/install authority | 拒绝 | host-specific negative evidence | quarantine/uninstall | 安全方向唯一，平台证据不足 P1 |
| 27 | 数据库只读 | 不应产生新 dispatch | durability/effect authority | 应拒绝新 governed write | failed durable transaction、store health | read-only management/recover | **否，P0 F-008** |
| 28 | 磁盘满 | 同上；不得把 telemetry drop用于 governed log | durability authority | 拒绝 dispatch/commit | fsync/outbox failure、reserved emergency space | free space/spool/reconcile | **否，P0 F-008** |
| 29 | outbox/消费者严重积压 | bounded block/reject/spill；不丢 governed event | event authority | 禁止静默丢失 | lag/high-watermark/ack | drain/resnapshot | 安全集合唯一；SLO缺 |
| 30 | 模型/SMS/ranker/Shell 不可用 | deterministic fallback 或 wait/escalate | respective authority | 不得绕过 gate | service health、fallback decision | static path/Admin CLI | 是，任务可用性依 Profile |
| 31 | 旧 checkpoint 遇 schema/policy升级 | 不得直接 resume | recovery/policy | 拒绝不兼容恢复 | pinned versions、migration digest | migrate/shadow/quarantine | 原则有，checkpoint无法表达，P0 F-010 |
| 32 | 多 Agent 重复/互等/错误共识/委派扩权 | dedupe、bounded wait、冲突保留、attenuation | task/delegation/local authorities | 拒绝扩权/假共识 | parent contract、budget、Task state、evidence | cancel/escalate/single-agent fallback | 权限方向有；死锁/收益不唯一，P2 |

---

## 9. 正确性证据与发布门禁建议

### 9.1 机器合同门禁

发布阻断条件：

1. registry/schema/transition/vector/normative text 双向无孤儿；
2. `owner_spec` 不得指向 informative 或 historical 资产；
3. Core 对象只使用一个 GovernedHeader 和 StrongRef；
4. 所有状态、错误码、authority、digest projection 有唯一机器落点；
5. schema-valid 反例必须被行为层拒绝，schema-valid 绝不直接算 pass；
6. generic traceability vector 不计行为覆盖；
7. specification set、schema bundle、suite digest 可重建。

### 9.2 状态机与 property tests

- transition table 驱动共享 API；
- 每条合法边至少一个正例，每个未登记边自动生成负例；
- 非法迁移返回 current state/version，状态与 Event 均不变；
- terminal 无非法出边；
- 并发 CAS property：恰一成功；
- Effect property：`COMMITTED ⇒ external_outcome=executed ∧ verification=current ∧ commit_authorized`；
- idempotency property：同 scope/key 同参同结果；异参恒拒绝；
- capability property：任何衍生集合不扩大；撤销后新 dispatch=0；
- 每个 sink property：旧 epoch writes=0；
- Context property：`role=control ⇒ trust=control ∧ authority valid`；
- Task property：`COMPLETED ⇒ acceptance criterion非空且当前 report passed`。

### 9.3 形式模型

**[evidence-backed judgment]** 用 TLA+/Alloy 或等价模型只覆盖：

- durable Intent/dispatch/reconcile/commit；
- external outcome `not_called|executed|not_executed|unknown`；
- idempotency map `(scope,key)->params/effect`；
- writer epoch/fencing；
- capability revocation epoch；
- Verification current/expired；
- Task candidate/accepted。

必须验证：

1. stale writer 永不 commit；
2. unknown 永不 success；
3. 同 key 异参永不 execute；
4. receipt/model/remote completed 永不直接 complete Task；
5. revocation 后新 dispatch/commit 永不发生；
6. recovery 不跳过 replay/reconcile/reauthorize；
7. 外部已执行但未接受不会以“无残留 ABORTED”关闭。

AWS TLA+ 经验支持模型检查发现稀有交错；seL4 同时提醒证明只在模型、实现映射和假设内成立。形式模型不能替代故障注入、executor 合约或 host sandbox 测试。

### 9.4 故障注入

最低集合：

- 三个规定 crash point；
- snapshot 损坏、event gap、replay digest 分歧；
- outbox 重复、乱序、积压；
- authority switch、stale fence；
- capability revocation 在 authorize/dispatch/execute/commit 四个切点；
- DB read-only、disk full、fsync error、partial transaction；
- executor query unavailable/lying；
- model/SMS/ranker/Shell unavailable；
- checkpoint schema/policy/renderer/verifier 不兼容；
- audit sink unavailable；
- clock rollback/expiry uncertainty；
- process kill 与 host reboot。

### 9.5 Conformance、readiness 与发布声明

- `implemented`：全部适用 MUST 有可复现 behavior evidence；
- `not-applicable`：有范围和 authority 依据；
- `experimental`：允许运行但不进入符合性；
- `planned`、`unsupported`：不计覆盖；
- 任一 P0、critical safety failure、unreconciled unknown、cross-scope disclosure、duplicate external effect、stale writer commit 均阻断发布；
- readiness case 必须绑定实现 commit、spec set、schema/suite digest、host/kernel、fault profile 和性能报告。

---

## 10. 性能合同审查

### 10.1 `performance-report.schema.json` 表达能力

| 要求 | 当前可表达 | 当前可强制 | 结论 |
|---|---|---|---|
| Memory/Discovery | 是，category | 仅名称/数值形状 | 部分 |
| Catalog/Semantic | 是 | 仅 category | 部分 |
| Governance overhead | 字段存在 | 顶层非 required | 不闭合 |
| Agent benefit | 可用任意 metric name | 无专门语义 | 否 |
| native baseline | 仅字符串 | 无 arm 类型/配置 | 否 |
| governance-only baseline | 同上 | 无 | 否 |
| optimized 版本 | 无结构 | 无 | 否 |
| ablation | 字符串数组 | 无结果 linkage | 否 |
| effect size | 无 | 无 | 否 |
| 95% CI | 每个 metric 有 | 无 arm delta/paired CI | 部分 |
| non-inferiority | 无 | 无 margin/result | 否 |
| 安全失败 | class/count | 无 denominator/severity/evidence | 弱 |
| p50/p95/p99 | 有且 metric 必需 | 有 | 是，但不验证单调/边界 |
| timeout/rejection/unknown/quarantine | 可自定义 metric | 非枚举、非 required | 否 |
| 所有失败样本 | 无 sample record | 无 | 否 |
| 工具/版本/权限/预算 | workload 可任意塞入 | 非 required | 否 |
| dataset split | 无 | 无 | 否 |
| sampling/seed | sampling object | seed 非 required | 部分 |
| warm/cold | 单一 enum | 不能要求双层报告 | 部分 |
| human minutes/maintenance | 任意 metric | 无规范语义 | 否 |
| SLO threshold/release gate | 无 | 无 | 否 |

**[confirmed fact]** 当前 positive vector 只有一个 context latency metric；删除 `governance_overhead` 后仍通过 schema。因此 schema 能承载示例字段，但不能约束白皮书宣称的完整性能合同。

### 10.2 建议 schema 结构

`performance-report/0.2` 至少需要：

- `experiment_manifest`：task IDs/split、model/provider/revision/seed、tools/versions、permissions、hardware、concurrency、budgets、cache regime、faults、grader、intervention policy；
- `arms[]`：`native|governance_only|optimized|ablation`，固定 implementation/config digest；
- `observations[]`：每 task/run/arm 的完成、时间、token、费用、工具调用、人工分钟和所有 failure disposition；
- `comparisons[]`：paired effect、effect size、95% CI、test、multiplicity、non-inferiority margin/result；
- `safety_events[]`：severity、sample、authority/evidence、disposition；
- `tail_metrics[]`：边界、p50/p95/p99、样本量和 censoring；
- `release_decision`：预注册 gate、pass/fail、理由。

### 10.3 SLO 候选

以下是 **[hypothesis]**，必须按 workload 在实验前预注册：

- governance-only 完成率：相对 native 的 95% CI 下界不低于 `-2` 个百分点；
- critical safety failures：观察值为 0，且报告基于样本量的单侧上置信界；
- unknown/quarantine/timeout：全部计入分母，不得 censor；
- cache-hit preservation ratio：95% CI 下界建议不低于 0.95；
- governance p99 latency share：不得超过预注册端到端预算；不要设一个跨模型/工具的通用毫秒数；
- p95/p99、人工分钟和维护成本任一越过预注册上限即阻断“收益”声明。

MLPerf 的尾延迟规则说明 p99 需要与置信度和误差范围匹配的样本量；固定报告 p99 字段而不做 power/sample 设计没有统计意义。

---

## 11. Agent 实际收益审查

### 11.1 机制因果矩阵

分类：A=正确性/治理必需；B=现在固定接口的性能使能合同；C=benchmark 选择的可插拔策略；D=研究假设。

| 机制 | 类 | 中间量→预期收益 | 可能反效果 | baseline/ablation | 指标与最小效应 | kill criteria |
|---|---|---|---|---|---|---|
| 确定性 Context 过滤 | A | 泄露/无关项↓→安全与 precision↑ | required recall↓ | A vs B，不可在真实敏感数据关闭 | disclosure=0、required recall | 任一泄露即阻断 |
| 增量 Context 解析 | B | 重复 token/round latency↓ | 多轮开销/停滞↑ | 单次 resolve | token/time ≥20%↓且完成非劣 | C≤B 或 stagnation↑则关闭 |
| ContextViewDelta | B | 变更传输↓、cache↑ | base 漂移 | full re-resolve | token/latency、wrong-base=0 | 无端到端收益则延后 |
| loss declaration | A | 可见损失↑→错误验收↓ | token/实现成本↑ | 只能在沙箱消融 | loss completeness、verified completion | 安全维度不可删除 |
| 稳定渲染 | B | prefix hit↑→TTFT/cost↓ | stale prefix/排序约束 | unstable renderer | cache ratio≥0.95，成本/TTFT | C≤B 或安全绑定受损则重设 |
| prompt/KV cache 保持 | B | cached tokens↑ | 侧信道/陈旧授权 | cache off/cold | cost/time ≥20%↓ | 任一跨 scope hit 即禁用 |
| 异步 memory admission | B | write latency↓ | backlog/未审候选积压 | synchronous admission | p95 write↓、无泄露 | quarantine/backlog 超 SLO |
| read-your-write | B | 行为一致性↑、重查↓ | 自注入/错误自强化 | RYW off | first-read success、cross-scope=0 | 污染或控制提升即停止 |
| Memory consolidation | C | recall/token precision↑ | 丢失冲突、成本↑ | raw admitted memory | completion/token、recall | 只改善局部指标或 stale↑则删 |
| 两级 Operation Catalog | B | tool tokens↓、top-k↑ | extra lookup latency | full schemas in prompt | token≥20%↓或 top-k↑ | false-no-tool/effect confusion↑则退回 |
| 工具 schema 过滤 | A | effect confusion↓ | 可用工具召回↓ | 仅沙箱消融 | prohibited exposure=0 | 安全失败即阻断 |
| 已登记批量 proxy | B | transport/token overhead↓ | 一批内局部失败复杂 | 单调用 proxy | cost/time↓、每调用 auth=100% | audit/auth 缺失即关闭 |
| bounded Loop | A | runaway/no-progress↓ | 过早停止 | 仅模拟消融 | no-progress/deadline/cost | 完成率显著下降则调策略，不删边界 |
| progress/stagnation detector | C | 重复工作↓ | false stop | fixed max-iteration | repeat/no-progress、completion | C≤B 或 false stop↑则替换 |
| verifier | A | 伪完成↓→verified completion↑ | latency/人工瓶颈 | 不得用模型自述作基线 | verified completion、false accept | false accept/safety failure阻断 |
| checkpoint/recovery | A | 重算/MTTR↓ | 写放大、复杂度↑ | restart-from-scratch | recovery recompute/time | duplicate effect 或跳 gate阻断 |
| event-driven watch | B | polling calls/token↓ | backlog/断流复杂 | fixed polling | calls、latency、gap=0 | gap/权限泄露即回退 |
| 共享权威 Task state | A | 重复劳动/冲突↓ | contention | 文件式 Kanban | duplicate work/conflict | 无收益可简化实现，不删 authority |
| multi-agent delegation | D（治理合同为A） | 并行度↑ | 协调、等待、扩权、错误共识 | 单 Agent/确定 workflow | completion/time/coord cost | 未超过单 Agent或安全失败则禁用 |

Core 不得依赖 C/D 才能正确运行。

### 11.2 A/B 与消融设计

四组：

- **A native**：同一 Agent 原生裸跑；
- **B governance-only**：同一 Agent + 全部正确性/治理路径，关闭 SMS、consolidation、semantic ranker 等 C/D 优化；
- **C optimized**：B + 单个或预注册组合优化；
- **D ablation**：从 C 逐项移除一个优化；安全机制只在隔离模拟环境消融。

工作负载：

1. 长时软件工程 Agent：SWE-bench Verified + 私有无泄漏回归集 + 人工标注长任务；
2. 研究/知识/个人助理：GAIA 类客观可验任务 + 版本固定的检索/文件任务；
3. 若宣称多 Agent：另加天然可并行、具有单 Agent 强基线的协作任务。

固定项：model/provider/revision、sampling/seed、task/split、tools/versions、data、permissions、hardware、concurrency、token/cost/time budget、cache warm/cold、fault profile、grader/verifier、人工介入政策。A 不应获得更少的任务信息；B/C 也不得获得更多权限、token、工具或数据。高风险 Effect 用模拟器/可逆沙箱，公平不等于放宽安全。

随机化：

- 按 task/difficulty/family 分层；
- A/B/C/D 运行顺序随机，抵消 provider 时间漂移；
- 同 task 做 paired comparison，多次独立 run；
- provider seed 不保证确定性，必须实测 run-to-run 方差；
- 用 pilot 方差做 80% 或 90% power analysis，不预设一个通用 n。

统计：

- binary completion 用 paired difference、McNemar/cluster bootstrap；
- time/cost/token 报 paired median/mean effect、95% CI；
- timeout 作为结果，不删除；必要时用 time-to-event/censoring规则；
- p50/p95/p99 分层报告；
- 多机制、多 workload 比较做 multiplicity control；
- 报全部分子/分母、失败样本、人工分钟和维护成本。

### 11.3 显著实际收益门槛

**[evidence-backed judgment]** 用户给出的 10%/20% 规则可作为预注册候选，但应修正：

1. completion 相对提升 ≥10% 时，再要求绝对提升至少 3 个百分点，避免低基数夸大；
2. 或 completion 的 paired 95% CI 下界不低于 -2 个百分点，同时 token/cost/time 至少一项降低 ≥20%；
3. 两类 workload 均成立；
4. C 显著超过 B，且 D 能归因；
5. 无新增越权、泄露、重复 Effect、伪完成、stale writer commit；
6. p95/p99、人工分钟、维护成本不抵消收益；
7. 门槛、主指标和 stopping rule 在看到结果前注册。

若 B≈A 仅可说“治理附加成本可接受”。若 C≤B，对应优化标 experimental、替换、延后或删除。局部 recall/top-k 改善未传导到 verified completion/time/cost，只能声明局部能力改善。

### 11.4 当前证据等级

定义：

- E0：机制论证，无实现数据；
- E1：单组件 microbenchmark；
- E2：固定 workload 的公平 A/B；
- E3：跨 workload/模型复现；
- E4：生产观察且有安全/readiness evidence。

**[confirmed fact] 当前为 E0。** 不能声称性能非劣或显著 Agent 收益。

---

## 12. 修改优化建议

### REC-001 — 冻结完整实现声明，先做合同收敛 tracer

- **优先级**：P0
- **目标文件和章节**：架构 §21、未来当前实施计划
- **当前问题**：270 REQ 与 0 implementation 之间没有可执行最小路径。
- **失败场景/影响**：团队并行实现冲突 schema，后期迁移。
- **内部证据**：F-001、F-027。
- **外部证据**：AWS TLA+、seL4 均从明确小边界建立证据。
- **建议修改**：把首个交付定义为 §6.5 tracer；P0 合同未闭合前不生成广泛 bindings。
- **动作**：删除高级 Profile 对首版的入口依赖；新增 tracer readiness gate。
- **normative semantics**：不改变目标语义，缩小适用范围。
- **同步资产**：Core、profile manifest、conformance index、实施计划。
- **实现成本**：中。
- **兼容/迁移风险**：低；当前无实现。
- **验收标准**：一条 R1 可逆 Effect 在每个 crash boundary 可恢复，所有 evidence 可重放。
- **建议里程碑**：第一批。
- **不修改后果**：实现反馈继续晚于规范扩张。

### REC-002 — 消除 informative owner_spec

- **优先级**：P0
- **目标**：requirements registry、Core/Profile、白皮书 §1/§19
- **问题**：17 个 REQ 由 informative 文件拥有。
- **失败影响**：规范优先级自相矛盾。
- **内部证据**：F-002。
- **外部证据**：项目自己的 normative-source 标准。
- **建议修改**：把每项行为原文迁至 normative behavior asset；registry 指向新锚点；白皮书改为解释性引用。
- **动作**：移动/合并，不新增概念。
- **normative semantics**：是；需版本化。
- **同步资产**：registry、vector input owner、spec set digest。
- **成本**：中。
- **迁移风险**：中；digest/owner 变化。
- **验收**：informative/historical owner 数为 0。
- **里程碑**：第一批。
- **不修改后果**：任何符合性声明均有 authority 歧义。

### REC-003 — 统一 GovernedHeader 与 StrongReference

- **优先级**：P0
- **目标**：`common-defs`、30 个 legacy schema、governed-object standard
- **问题**：双对象合同。
- **失败影响**：tenant/scope/purpose 丢失。
- **内部证据**：F-003。
- **外部证据**：NIST SP 800-207 的逐资源授权原则。
- **建议修改**：选择新 header/ref；旧结构仅 legacy adapter；迁移 Intent/Effect/Event/Task/Context/Checkpoint 首批 schema。
- **动作**：合并、删除重复定义。
- **normative semantics**：是，breaking。
- **同步资产**：全部 schema `$ref`、fixtures、vectors、registry。
- **成本**：高。
- **迁移风险**：当前无实现，数据风险低；schema digest 风险高。
- **验收**：Core 竖切 schema 不再引用 legacy metadata/ref；跨 tenant 缺字段实例 schema-invalid。
- **里程碑**：第一批。
- **不修改后果**：安全治理只能靠实现私有约定。

### REC-004 — 修复 Activity/Context bootstrap

- **优先级**：P0
- **目标**：ContextRequest/View、ActivityContext、RFC §10—§11、Core §6
- **问题**：Activity 必需 View，View 又需 Activity；Request 缺治理字段。
- **失败影响**：授权时点和对象 digest 不可构造。
- **内部证据**：F-004。
- **外部证据**：无新增概念需求；属于内部闭合。
- **建议修改**：先创建 immutable `ActivityBinding`（无 View），Resolve 引用它；成功后用 CAS 创建/更新 ActivityContext 指向 View。View 强引用 binding。
- **动作**：合并 Execution/Activity 的重复字段，新增最小 binding 或两阶段状态。
- **normative semantics**：是。
- **同步资产**：RFC、schema、REQ-CTX、vectors、checkpoint。
- **成本**：中高。
- **迁移风险**：中。
- **验收**：从 TaskContract 到 ContextView 有无 placeholder 的唯一构造序列；撤销竞态测试通过。
- **里程碑**：第一批。
- **不修改后果**：实现必然发明非标准 bootstrap。

### REC-005 — 重构 Effect/Verification/Acceptance 单一真相

- **优先级**：P0
- **目标**：Effect/Verification schema/transition、Task transition、Core §9—§10
- **问题**：重复状态和 schema-valid 非法组合。
- **失败影响**：unknown/pending 被 committed。
- **内部证据**：F-005。
- **外部证据**：ARIES 支持日志恢复但要求明确事务状态；不能靠重复字段猜真相。
- **建议修改**：Effect 只保留 execution outcome、reconciliation result、verification_ref、disposition；VerificationReport 为唯一判定；AcceptanceDecision 推进 Task。
- **伪 diff**：删除 Effect 内 `verification.status`；禁止 `COMMITTED` 除非 report current + outcome executed。
- **动作**：删除/合并。
- **normative semantics**：是，breaking。
- **同步资产**：2 个 schema、2 个 transition、REQ/vector/error。
- **成本**：高。
- **迁移风险**：高，但当前无实现。
- **验收**：不可能组合 schema/behavior 均拒绝；跨域 property tests 通过。
- **里程碑**：第一批。
- **不修改后果**：恢复与验收无法证明。

### REC-006 — 表达 residual external effect

- **优先级**：P0
- **目标**：Effect transition/Core §9.4
- **问题**：executed 后可直接 ABORTED。
- **失败影响**：真实世界残留被“关闭”隐藏。
- **内部证据**：F-006。
- **外部证据**：Saga/ARIES 类恢复不保证外部世界自动回滚。
- **建议修改**：增加 disposition `REJECTED_WITH_RESIDUAL_EFFECT|COMPENSATION_PENDING|QUARANTINED`；只有证明未执行或世界已恢复才无残留关闭。
- **动作**：替换含混 ABORTED 语义。
- **normative semantics**：是。
- **同步资产**：transition、Effect schema、Task cancel guards、ShellStatus。
- **成本**：中。
- **迁移风险**：中。
- **验收**：已执行+verify fail 不能进入无残留 terminal。
- **里程碑**：第一批。
- **不修改后果**：取消/失败报告可欺骗操作者。

### REC-007 — 统一撤销、fencing 和幂等提交门禁

- **优先级**：P0
- **目标**：Capability/Effect/AKP/Distributed、errors registry
- **问题**：dispatch/commit 重验缺失、sink 覆盖不全、通用冲突 error 缺。
- **失败影响**：撤销后执行、旧 epoch 双写、异参复用。
- **内部证据**：F-007、F-013、F-014。
- **外部证据**：Stripe 同 key 异参拒绝；Gray/Cheriton lease 与 fencing 工程经验。
- **建议修改**：每个 dispatch/commit 固定 current revocation+epoch；发布 sink inventory；登记 `IDEMPOTENCY_CONFLICT`/fencing/revoked codes。
- **动作**：合并三个门禁为共享 commit guard。
- **normative semantics**：是。
- **同步资产**：Core、AKP、Distributed、schemas/transitions/vectors/errors。
- **成本**：高。
- **迁移风险**：中。
- **验收**：四切点 revocation、全部 sink stale-writer、同 key异参 property 全通过。
- **里程碑**：第一批。
- **不修改后果**：安全撤销和 split-brain 声明不成立。

### REC-008 — 定义 durability unavailable 模式

- **优先级**：P0
- **目标**：Core Effect/Event/Recovery、error contract、部署章节
- **问题**：磁盘满/DB只读无安全语义。
- **失败影响**：外部动作无 durable Intent/dispatch。
- **内部证据**：F-008。
- **外部证据**：ARIES 的 WAL 先行原则。
- **建议修改**：dispatch 前事务 barrier；写失败进入 `DURABILITY_UNAVAILABLE`；保留只读管理与 emergency spool；spool 也须有完整性/容量边界。
- **动作**：新增错误和 readiness state，不新增业务对象。
- **normative semantics**：是。
- **同步资产**：errors、Effect/Event、fault vectors、SLO。
- **成本**：中高。
- **迁移风险**：低。
- **验收**：fsync/commit/outbox 任一点失败时 external calls=0；调用后 receipt 写失败可恢复。
- **里程碑**：第一批。
- **不修改后果**：最基本 WAL 承诺不可实现。

### REC-009 — 登记 digest projection 与 golden fixtures

- **优先级**：P0
- **目标**：canonical standard、全部 digest/signature schema
- **问题**：自引用 digest 无定义。
- **失败影响**：强引用和签名无法互操作。
- **内部证据**：F-009。
- **外部证据**：RFC 8785 只给 canonical bytes，不替项目定义签名投影。
- **建议修改**：每个合同给 domain、projection schema、excluded JSON Pointers、signature preimage；生成语言无关 positive/negative fixtures。
- **动作**：新增 machine fixture，补充而非猜测。
- **normative semantics**：是。
- **同步资产**：ADR-0004、schemas、spec set。
- **成本**：中。
- **迁移风险**：高 digest 变化。
- **验收**：Rust/TS/参考脚本字节与 digest 一致；无 self-reference failure。
- **里程碑**：第一批。
- **不修改后果**：所有 digest-pinned 声明都是不可执行文本。

### REC-010 — 建立通用 Checkpoint/Resume 合同

- **优先级**：P0
- **目标**：Checkpoint、LoopCheckpoint、AKP continuation、Recovery
- **问题**：稳定事实不足。
- **失败影响**：升级后跳过 reconcile 或依赖隐藏态。
- **内部证据**：F-010。
- **外部证据**：AWS formal methods 支持对恢复顺序建模。
- **建议修改**：Checkpoint envelope 必需 high-watermark、pending effects、epochs、spec/schema/policy/renderer/verifier/environment digests、remaining budget、compatibility decision。
- **动作**：新增一个通用 envelope；Loop payload 下沉。
- **normative semantics**：是。
- **同步资产**：schema、Core/AKP、recovery vectors。
- **成本**：高。
- **迁移风险**：中。
- **验收**：旧版本恢复只有 migrate/reject/quarantine 三个显式结果。
- **里程碑**：第一批。
- **不修改后果**：C2 recovery 声明不可移植。

### REC-011 — 规范化审批、break-glass 与可信确认面

- **优先级**：P0
- **目标**：RFC management、approval schemas、§12.12
- **问题**：关键安全设计仍 informative，R3 单 decision。
- **失败影响**：假 UI、轰炸、旧 challenge、单人 R3。
- **内部证据**：F-011。
- **外部证据**：NIST SP 800-63B-4 phishing resistance；MITRE T1621 仅作威胁资料。
- **建议修改**：审批请求/投递/确认分离；authenticator/session/proposal binding；m-of-n quorum；rate state；break-glass 独立 authority/TTL/notification。
- **动作**：新增最小 approval protocol，删除“聊天文本可证明批准”的空间。
- **normative semantics**：是。
- **同步资产**：RFC、AKP、schema、errors、vectors、readiness。
- **成本**：高。
- **迁移风险**：中。
- **验收**：伪卡片、重放、轰炸、旧 proposal、R3 单人全部拒绝。
- **里程碑**：第一批；只阻断 R2/R3。
- **不修改后果**：高风险管理面不得发布。

### REC-012 — 修正 TaskContract 与 Profile 声明

- **优先级**：P1
- **目标**：TaskContract、profile manifest、conformance README
- **问题**：旧 state enum、无必需 acceptance、implemented 可带降级。
- **失败影响**：不可判定 Task 和虚假实现声明。
- **内部证据**：F-012、F-016。
- **外部证据**：无外部依赖。
- **建议修改**：domain registry ref；至少一个 acceptance criterion；逐 REQ pass/fail/NA/not-run；implemented 禁止 fail/degradation。
- **动作**：合并状态声明语义。
- **normative semantics**：是。
- **同步资产**：schemas、REQ、manifest vectors。
- **成本**：中。
- **迁移风险**：中。
- **验收**：无 acceptance 合同 schema-invalid；implemented manifest 每个适用 REQ 可追踪。
- **里程碑**：第一/第二批。
- **不修改后果**：release gate 可被绕过。

### REC-013 — 把 conformance 从路径检查变成行为 runner

- **优先级**：P0
- **目标**：conformance 资产与未来 runner
- **问题**：175 个 REQ 只有 generic trace mapping。
- **失败影响**：schema-valid 被误当 behavior-pass。
- **内部证据**：F-001、F-015。
- **外部证据**：HELM/MLPerf 的可重复条件与原始结果原则。
- **建议修改**：静态 lint、schema、transition、implementation adapter、fault tests 分层；故意错误实现必须 fail。
- **动作**：新增 runner；generic vector 降级为 lint。
- **normative semantics**：否，证据机制变化。
- **同步资产**：conformance README、suite manifest、profile evidence。
- **成本**：高。
- **迁移风险**：低。
- **验收**：无实现=not-run；错误实现=fail；每个 P0 行为有 observable negative。
- **里程碑**：第一批末/第二批初。
- **不修改后果**：所有 Profile 状态不可置信。

### REC-014 — 建立宿主平台 sandbox profile

- **优先级**：P1
- **目标**：Agent Compatibility、package/report schema、vectors
- **问题**：不可绕过证据未按 host 分层。
- **失败影响**：IPC/secret/network 旁路。
- **内部证据**：F-017。
- **外部证据**：OWASP Agent Security 仅是指导，不是平台证明。
- **建议修改**：固定 OS/kernel/container/runtime、通道矩阵、负例和已知缺口；每个平台独立 max C/R。
- **动作**：下沉 host adapter，不扩大 Core。
- **normative semantics**：是，Profile 语义。
- **同步资产**：schema/vector/readiness。
- **成本**：高。
- **迁移风险**：低。
- **验收**：network/filesystem/secret/subprocess/IPC/MCP/device 每项有 pass/fail。
- **里程碑**：第二批。
- **不修改后果**：OS 不可绕过声明不成立。

### REC-015 — 发布 performance-report 0.2 与 SLO profile

- **优先级**：P1
- **目标**：performance schema、REQ-PERF、vector
- **问题**：A/B、effect size、non-inferiority 和 failure samples 缺失。
- **失败影响**：伪性能收益。
- **内部证据**：F-020、F-022。
- **外部证据**：MLPerf tail methodology、CONSORT CI/margin 方法（仅方法可迁移）。
- **建议修改**：采用 §10.2 arms/observations/comparisons/safety/release 结构。
- **动作**：新增 0.2，保留 0.1 只作 legacy。
- **normative semantics**：是，性能声明。
- **同步资产**：profile manifest、REQ-PERF、vectors。
- **成本**：中高。
- **迁移风险**：中。
- **验收**：缺 A/B arm、effect size、CI、failure sample 或 safety gate 的收益报告 schema-invalid。
- **里程碑**：第三批。
- **不修改后果**：不能声明非劣或显著收益。

### REC-016 — 预注册 A/B/C/D 与 kill criteria

- **优先级**：P1
- **目标**：§19/路线图/未来 benchmark plan
- **问题**：当前只有“性能对齐报告”一句。
- **失败影响**：更多资源、挑任务、改主指标造成虚假收益。
- **内部证据**：F-026。
- **外部证据**：SWE-bench、GAIA、HELM、agent-eval randomness 预印本。
- **建议修改**：采用 §11.2—§11.3，固定两类 workload、paired randomized runs、power 和门槛。
- **动作**：新增 benchmark protocol；优化策略保持可拔。
- **normative semantics**：否，除性能声明门禁。
- **同步资产**：performance schema、release checklist。
- **成本**：高运行成本。
- **迁移风险**：低。
- **验收**：B vs A、C vs B、D 归因均可重放；所有失败公开。
- **里程碑**：第三批。
- **不修改后果**：性能叙事仍不可证伪。

### REC-017 — 把 prefix stability 定义为真实字节前缀

- **优先级**：P1
- **目标**：Core §6.6、REQ-CTX-012、CTX-RENDER-001
- **问题**：相对顺序不等于累计前缀不变。
- **失败影响**：缓存 miss、TTFT/cost 回归。
- **内部证据**：F-021。
- **外部证据**：Anthropic 官方按 prefix hash/cache breakpoint 匹配。
- **建议修改**：定义 segment IDs、immutable prefix digest、append-only suffix 和 breakpoint；安全绑定变化必须主动 bust cache。
- **动作**：澄清，不引入 CVM 新对象。
- **normative semantics**：是。
- **同步资产**：renderer contract/vector/perf metrics。
- **成本**：中。
- **迁移风险**：低。
- **验收**：新增 suffix 后旧 breakpoint 前字节完全一致；撤销后缓存必失效。
- **里程碑**：第二/第三批。
- **不修改后果**：REQ 名称承诺超过其测试。

### REC-018 — 重画唯一组件/authority 地图并瘦 Core

- **优先级**：P1
- **目标**：架构 §4.7、§7、附录 A/B
- **问题**：Approval/store/acceptance owner 缺，Shell/Memory/Profile 混入 Core。
- **失败影响**：多个 authority 和循环依赖。
- **内部证据**：F-024、F-027。
- **外部证据**：L4/seL4 只支持最小性原则，不替代本项目划分。
- **建议修改**：采用 §6.2/§6.3；Shell 客户端、Harness policy、Memory/SMS 均在 kernel 外。
- **动作**：删除、合并、下沉。
- **normative semantics**：部分；状态机缩减需 major 版本。
- **同步资产**：Core、state registry、profiles、roadmap。
- **成本**：高设计成本，低实现迁移成本。
- **迁移风险**：中高。
- **验收**：每个组件一个 owner、一个 authority 接口、无反向依赖；tracer 不加载高级 Profile。
- **里程碑**：第二批。
- **不修改后果**：代码归属继续模糊。

### REC-019 — 为不可对账 Operation 建立准入矩阵

- **优先级**：P1
- **目标**：OperationDescriptor、Core Effect、R0—R3
- **问题**：不可 query/idempotent/compensate 的操作只有开放问题。
- **失败影响**：永久 unknown。
- **内部证据**：F-023。
- **外部证据**：RFC 9110 只允许已知幂等语义安全自动重试。
- **建议修改**：按 effect/risk 定义 required executor properties；R1+ 默认拒绝不可对账写，例外需 residual-risk approval。
- **动作**：新增矩阵，删除隐式允许。
- **normative semantics**：是。
- **同步资产**：Descriptor schema、authz、vectors、readiness。
- **成本**：中。
- **迁移风险**：未来 adapter 兼容风险中等。
- **验收**：每种 Operation 可机器判定 allow/restrict/deny。
- **里程碑**：第二批。
- **不修改后果**：recovery 只能无限 quarantine。

### REC-020 — 更新证据分类并继续冻结高级 Profile

- **优先级**：P2
- **目标**：附录 C、§15、§21、当前实施计划
- **问题**：证据等级混类、版本过期、高级 Profile 无实现。
- **失败影响**：把指南/草案当标准，把研究潜力当能力。
- **内部证据**：F-028—F-030。
- **外部证据**：NIST SP 800-63B-4、MAST NeurIPS 2025。
- **建议修改**：更新证据 taxonomy；distributed/multi-agent/embodied/hetero/learning 保持 planned/experimental，需单节点 benchmark 才解冻。
- **动作**：延后、更新引用。
- **normative semantics**：否。
- **同步资产**：白皮书引用、roadmap、profile manifest。
- **成本**：低。
- **迁移风险**：低。
- **验收**：每个来源类型准确；高级 Profile 无实现时不进入 Core release。
- **里程碑**：第四批。
- **不修改后果**：证据可信度继续下降。

---

## 13. 建议实施顺序

### 第一批：P0 正确性和漂移修复

REC-002 → REC-003 → REC-004 → REC-005 → REC-006 → REC-007 → REC-008 → REC-009 → REC-010 → REC-011 → REC-012。  
完成条件：规范 owner 唯一、对象合同单一、Context 可构造、Effect 不可伪成功、durability/revocation/checkpoint/approval闭合。

### 第二批：Core 简化和职责闭合

REC-001 → REC-013 → REC-014 → REC-017 → REC-018 → REC-019。  
完成条件：单节点 tracer 运行、错误实现可被 runner 拒绝、宿主边界按平台证明、Core 不依赖高级 Profile。

### 第三批：性能合同和 benchmark

REC-015 → REC-016。  
完成条件：native/governance-only/optimized/ablation 可机器比较，非劣效和显著收益门槛预注册。

### 第四批：实现反馈驱动的后续优化

REC-020，以及只有 C 相对 B 达到门槛的 Memory consolidation、SMS/CRB、多 Agent、CIM、learning 扩展。未达门槛的 C/D 机制关闭、替换、延后或删除。

---

## 14. 最终 Go/No-Go 判定

### 14.1 唯一总判定

**需要架构重构。**

核心命题“用 authority、Context gate、Intent/Effect、Verification 和 recovery 治理持久 Agent”没有被否定；但当前 machine contract 的双轨、循环和非法状态意味着仅“修几处文字”不足，需要结构化收敛。

### 14.2 分层判断

| 范围 | 判定 | 进入条件 |
|---|---|---|
| 单节点 R0/R1 | **条件性 Go，仅限合同修复后的 tracer；当前生产 No-Go** | F-002—F-010 闭合，runner 与 crash tests 可执行 |
| 分布式 | **No-Go** | 单节点正确性 + 全 sink fencing + partition model |
| 多 Agent | **No-Go/研究性** | 单 Agent 强基线，C>B，MAST 类负例通过 |
| 具身 | **No-Go** | 独立 safety case、WCET、设备/host 证据 |
| 持续学习 | **No-Go/experimental** | 候选/发布/回滚机器合同和独立 verifier |

### 14.3 性能声明

- 正确运行已证明：**否**；
- 性能不劣化已证明：**否**；
- Agent 显著实际收益已证明：**否**；
- 可合理声称的仅是：**架构提出了一组有待实现和 A/B 验证的治理与性能使能假设。**

---

## 15. 待决问题与证据缺口

### 15.1 unresolved questions

1. **[unresolved question]** Effect commit 表示“本地记录接受”还是“外部世界已达到目标”？两者需拆分。
2. **[unresolved question]** capability 在 dispatch 后撤销时，允许执行完成、禁止 commit、还是强制补偿？需按 effect class 明示。
3. **[unresolved question]** 不可查询执行器在哪些 R 等级可被例外准入？
4. **[unresolved question]** Activity/Context 两阶段对象的最终 digest 和 authority 是什么？
5. **[unresolved question]** Verification passed 后世界变化，历史 Task COMPLETED 是历史接受事实还是必须重新打开？当前 Task terminal 语义需说明。
6. **[unresolved question]** break-glass 正文访问的 quorum、最小披露、通知和撤销如何机器化？
7. **[unresolved question]** 单节点真实 fsync 边界、额外写次数与 p99 预算是多少？
8. **[unresolved question]** 哪些 workload 上 Context/Catalog/Memory 的局部改善能传导到 verified completion？
9. **[unresolved question]** 多 Agent 何种任务分解在协调成本后仍优于单 Agent？
10. **[unresolved question]** Windows/Linux/macOS 的 sandbox 可证明边界分别是什么？

### 15.2 hypotheses

- **[hypothesis]** 确定性 Context 过滤可提高安全性而不显著降低 completion；
- **[hypothesis]** 真正的 append-only prefix 渲染可保持 ≥0.95 cache-hit ratio；
- **[hypothesis]** async admission + scoped RYW 可降低写延迟且不产生持久注入；
- **[hypothesis]** 两级 Catalog 可减少 token 并保持/提高正确工具选择；
- **[hypothesis]** checkpoint/recovery 的额外写成本会被故障后的少重算抵消；
- **[hypothesis]** 多 Agent 只在天然并行且状态 authority 清晰的任务中产生净收益。

### 15.3 未执行检查

- 未运行任何实现或 conformance suite：仓库没有实现/runner；
- 未做性能 A/B：没有可执行系统和 baseline；
- 未做 fault injection、sandbox escape、分布式 partition、具身硬件测试：没有目标部署；
- 未做形式验证：没有模型；
- 未核验每篇附录参考文献的全部实验复现：本审查只核验与核心结论直接相关的标准、论文和官方资料；
- 未修改任何既有资产：受本任务写入边界限制。

### 15.4 完成检查

- [x] 除本独立审查报告外未修改现有文件
- [x] 保护并记录用户/外部未提交改动
- [x] 盘点真实规范与测试资产
- [x] 核验白皮书 REQ/schema/vector 引用
- [x] 判定 OS 定位和最小 Core
- [x] 完成视图与组件完备性矩阵
- [x] 推演 32 个故障场景
- [x] 给出正确性证据与发布门禁
- [x] 审查性能 schema 和 SLO
- [x] 建立 native/governance-only/optimized/ablation 方案
- [x] 区分非劣化与显著收益
- [x] 给出统计、实际效应和 kill criteria
- [x] 生成稳定 F/REC ID
- [x] 未应用任何建议到原始资产
