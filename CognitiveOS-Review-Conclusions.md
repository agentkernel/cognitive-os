# CognitiveOS 架构审查报告（证据化）

- **版本**：2.0
- **日期**：2026-07-20
- **文档性质**：Informative 审查报告。本文不构成规范义务；本轮登记的 REQ/错误码/向量均已按规范变更流程写入 registry/schema/vector 并在白皮书附录 D 1.0.1 留有迁移记录。
- **评审对象**：[CognitiveOS-Architecture.md](./CognitiveOS-Architecture.md) v1.0.0（未提交工作区状态）及全部仓库机器资产。
- **评审方法**：反方优先。先实测资产（不信历史计数），再逐项核验白皮书声明与 registry/schema/transition/vector 的双向一致性，推演 26 类故障场景，核验外部证据，最后在证据支持处直接修复并复验。评审期间仓库出现另一份独立只读审查报告（[CognitiveOS-Architecture-Independent-Review.md](./CognitiveOS-Architecture-Independent-Review.md)，快照 f2f826a，F-001–F-030）；本报告对其每条 finding 独立复核后逐条处置（§6.4），凡采纳修复均以静态反例复验。
- **变更记录**：v2.0 全文重写为本轮证据化审查报告，替代 v1.2（v1.2 的评审基线与改进项追踪已完成历史使命，其结论逐条处置见 §6）。v1.2 及更早内容可从 git 历史获取。

---

## 0. 总判定

**修复 P0 后可实现——规范侧 P0 已在本轮修复完毕，仓库现在可以直接启动单节点 R0/R1 参考实现（M0）；"发布"仍被 §7.5 所列实现侧 P0 门禁阻断。**

判定依据（每条指向可核验证据）：

1. **机器合同闭合**：273 条 REQ、55 个错误码、56 份 schema、5 张迁移表（54 状态/90 迁移）、74 个向量，REQ↔向量双向无孤儿、owner_spec 全部可解析、向量引用的错误码全部已登记（§8 验证记录）；非法状态组合经条件 schema 封闭并以静态反例复验（§7.1）。
2. **状态机一致**：白皮书 §6.3 五状态机文本图与 [specs/transitions/](./specs/transitions/) 逐边一致（含 Effect `reconciliation_result` 三分、`Verification PASSED→EXPIRED`、Task 验收 guard `acceptance_authority_matches` + `verification_passed_and_current` + `fixed_post_state_unchanged`）；终态无出边、无未声明状态（脚本核验）。
3. **收益声明有合同**：本轮新增 [Agent 收益评测合同](./docs/evaluation/agent-benefit-benchmark.md)（REQ-PERF-005），四臂设计 + 预注册门槛 + kill criteria + gaming 防护；performance-report schema 可表达 arms/CI/effect size/ablation/claim_level。修复前该能力不存在（F1）。
4. **核心命题成立但未被实现证明**：仓库实现代码为 0 行，一切"正确运行"均处于"规范已登记"状态；本报告把正确性转化为可执行门禁（§7），不宣称任何能力已实现。

---

## 1. 资产实测（2026-07-20，本轮修复后）

| 资产 | 修复前（本轮起点） | 修复后 | v1.2 报告值（已过时） |
|---|---|---|---|
| 白皮书 | 1916 行 / v1.0.0 | 1937 行 / v1.0.1 | 1776 行 / v0.8 |
| RFC-0001 | 669 行 v0.2 Draft | 不变 | 507 行 |
| companion 规范 | 12 份 | 不变 | 12 份 |
| JSON Schema | 56 份 | 56 份（7 份修订：effect、task-contract、context-view、memory-candidate、loop-checkpoint、activity-context、performance-report） | 40 份 |
| 测试向量 | 68 个（21 个 stub） | **74 个（18 个 stub，4 个升级 v0.2）** | 65 个 |
| REQ 条目 | 270 条 | **273 条** | 244 条 |
| 错误码 | 53 个 | **55 个** | 49 个 |
| 迁移表 | 5 张 / 90 迁移 | effect 表升 v0.2（guard 加固，迁移数不变） | 未计 |
| 实现代码 | 0 行 | 0 行 | 0 行 |

v1.2 的计数在其测量时点（v0.8 评审期）为真，但已不能描述当前仓库；引用资产计数必须重新实测（v1.2 IMP-17 教训重演，本表即修正）。

**外部证据抽查（本轮新核验）**：

- CaMeL（Debenedetti et al., arXiv:2503.18813，Google DeepMind）真实存在；其"从可信查询提取控制/数据流 + capability 标签 + 自定义解释器"设计与 §9.2/§17.3 语义隔离不变量同构；AgentDojo 上 77% 任务可证明安全 vs 无防御 84%——同时说明治理有实际效用代价，佐证本报告"治理开销必须计量、收益必须实测"的立场。
- 供应商 prompt cache 计价核验（Anthropic 官方文档）：cache read 0.1× 基础输入价、5 分钟 TTL 写 1.25×、1 小时写 2×；长 prompt TTFT 改善公开口径最高约 85%（≈2–7× 加速，依长度而变）。IMP-02"缓存失效造成 2–10 倍成本/TTFT 退化"的量级声明**在成本维度成立**（0.1×→1× 为 10 倍）、在延迟维度**上界偏乐观**（公开数据支持 2–7×）；白皮书 §9.4 使用"数倍"表述，无需修正；REQ-CTX-012 的动机成立。

---

## 2. 最关键发现（F1–F10）

严重度定义：P0 = 阻断实现或使收益/安全声明不可信；P1 = 造成规范权威混乱或留下唯一安全行为未定义；P2 = 一致性清理。

### F1（P0，已修复）：Agent 收益声明无机器合同，schema 无法表达对照实验

- **证据**：修复前 [performance-report.schema.json](./specs/schemas/performance-report.schema.json) 的 `baselines`/`ablations` 仅为字符串名数组；无 effect size、无逐臂结果、无 claim level；`category` 枚举无收益类；仓库全文搜索无任何 A/B/ablation 实验设计文档。
- **失败方式**：任何人都可以发布"装上 CognitiveOS 后 Agent 更快更准"的报告而无须提供对照臂、置信区间或预注册门槛；REQ-PERF-001–004 只约束报告格式与治理开销，不约束收益声明的因果证据。这直接违背最终目标 4（无收益只能声明治理/非劣化）。
- **影响范围**：§19.4、§21 Phase 1、全部性能叙事的可信度。
- **修复**：新增[评测合同](./docs/evaluation/agent-benefit-benchmark.md)（四臂、W1/W2/W3、六条门槛、非劣化边界、kill criteria、gaming 防护 10 条）；登记 REQ-PERF-005；schema 加 `agent_benefit` 类别与 `comparison` 块；PERF-REPORT-CONTRACT-001 增 comparison 正例 + "无对照收益声明→PERFORMANCE_REPORT_INCOMPLETE"负例；§19.4/§21 回写。

### F2（P0，已修复主体，残余见 F8/F9）：关键安全负例与崩溃路径向量是空壳

- **证据**：修复前 21/68 向量的 expected 仅为 `{requirement_semantics_enforced: true, ...}` 通用三键（脚本判定），其中包括全部三个崩溃点向量 EFF-CRASH-001/002/003；§4.5 要求的确定性拒绝证据中，rank-before-auth、撤销后缓存复用、同租户横向读取、远端 completed≠本地验收四类**没有任何专属向量**（对应 REQ 仅挂 SPEC-CONTRACT-COVERAGE-001 兜底追溯向量）；22 条 REQ-GOBJ 全部无行为向量。
- **失败方式**：实现者可以让 runner 对崩溃恢复"通过"而实际未测任何状态断言；四类最常见的越权/伪完成路径在符合性层面不可见。
- **修复**：EFF-CRASH-001/002/003 升级为逐崩溃点行为断言（前态/崩溃点/恢复动作/原键重放/无重复 Effect/审计闭合）；新增 CTX-RANK-AUTH-001、CTX-REVOKE-CACHE-001、GOBJ-TENANT-LATERAL-001、GW-REMOTE-COMPLETE-001 四个安全负例与 EFF-IDEM-CONFLICT-001、STATE-STORE-DEGRADE-001 两个契约负例。不弱化、不删除任何既有负例。

### F3（P1，已修复）：18 条 REQ 的规范权威落点指向 informative 文档

- **证据**：修复前 17 条 REQ 的 `owner_spec` 指向 `CognitiveOS-Architecture.md#...` 锚点（REQ-STATE-003/004、CTX-004/008、CAP-002/003、EFF-004/006、RES-001、SEC-002、五条 PROFILE-*、PERF-002/004），另有 REQ-PROFILE-HET-001 指向 schema 文件（行为文本落在形状资产）。这与仓库自身规范优先级（digest 固定机器资产 > companion > 白皮书）及 [normative-source-and-versioning](./docs/standards/normative-source-and-versioning.md) §2/§3 的资产分类与形状/行为分离直接冲突。
- **失败方式**：修改白皮书叙述即事实上修改规范义务，绕开 companion 的版本纪律；实现者无法确定哪个文本是权威条文。
- **修复**：18 条全部 repoint 至已存在对应条文的 companion（REQ-PROFILE-CVM-001 条文本轮补入 Core §15.3；REQ-PERF-002/004 条文本轮补入符合性索引）；白皮书 §1.2 对齐说明同步更新，锚点降级为检索入口。

### F4（P1，已修复）：提交路径存储失败无定义语义（审查场景 21）

- **证据**：修复前全仓（白皮书、Core、errors.yaml）无"数据库只读/磁盘满/日志不可写"时受治理写入的行为定义；§16.1 故障表无此行。
- **失败方式**：实现者面临未定义行为的三岔口——内存缓冲后补（违反因果不变量）、静默丢弃（违反审计闭合）、或全面崩溃（丧失确定性停止路径）；任何选择都无规范依据。
- **修复**：登记 REQ-REC-003（fail closed；禁止内存缓冲冒充提交；禁止 Intent 未持久化时分派；保持只读检查/停止/撤销/审计导出可用）、错误码 STATE_STORE_UNAVAILABLE、向量 STATE-STORE-DEGRADE-001；白皮书 §16.1 故障表增行、§16.5 增语义段。

### F5（P1，已修复）：同键异参拒绝只在管理域登记

- **证据**：修复前"同 key 异参拒绝"仅存在于 REQ-MGMT-IDEM-001（管理动作域）；通用 Effect 域只有 effect.transitions 的隐式 guard `idempotency_binding_valid`；AKP README §6 使用未登记错误名 `IDEMPOTENCY_CONFLICT`。而形式化义务集（§7.5）第 7 条恰要求"同 key 异参必须拒绝"为全域不变量。
- **修复**：REQ-EFF-002 条文扩充拒绝语义；登记 EFFECT_IDEMPOTENCY_CONFLICT；向量 EFF-IDEM-CONFLICT-001；§12.5 回写；AKP 错误名对齐到已登记码。

### F6（P2，已修复）：companion 错误名边界与链接缺失

- **证据**：heterogeneous §9 列 8 个未登记错误名、learning §8 列 7 个，均无 Core §13 式"待登记类别"免责声明；semantic-mediation 是五个生态 companion 中唯一无"机器 schema"链接段的（其两个 schema 实际存在且被 registry 引用）；白皮书 §9.10"本文不据此虚构向量"表述在负例向量已登记后过时。
- **修复**：三处 companion 补免责/链接；§9.10 改为引用已登记向量清单。

### F7（P0 语境，非缺陷）：全部正确性声明仍处于"规范已登记"状态

- **证据**：实现代码 0 行；无 conformance runner；74 个向量是声明式输入/期望，从未被执行；形式化模型是 §21 Phase 4 承诺，未交付。
- **处置**：不是本轮可修复的缺陷，而是判定框架——本报告所有"已闭合"均指规范/向量层闭合；行为证据从 M1（runner）起才存在。§7.5 把它转化为发布阻断门禁。四类状态语言（规范已登记/实现已提供/测试已执行/Profile 已符合）在全文严格区分。

### F8（P2，残余）：18 个 stub 向量仍存在

- **证据**：修复后仍有 18 个向量的 expected 为通用三键（CAP-BIND-001、CAP-LEASE-003、CTX-SCHEMA-001、CTX-VIEW-001、CVM-STALE-001、EFF-FLOW-001、EVT-SCHEMA-001、HARNESS-*×2、LOOP-*×3、MANIFEST-*×2、PROFILE-CORE/DIST-001、SCHEMA-*×2）。
- **风险评估**：这些多为 wire/schema/manifest/profile-声明类（runner 可用 schema 校验兜底），且 PROFILE-DIST-001（旧 epoch 拒绝）与 LOOP 三向量属行为类，风险高于其余。**处置**：列入 M1 runner 交付时的向量行为化清单（§7.5），本轮不批量伪造行为断言——没有实现对照的行为断言批量生产会制造虚假置信。

### F9（P2，残余）：REQ-GOBJ 族 21/22 条仍无行为向量

- **证据**：GOBJ-TENANT-LATERAL-001 是首个 GOBJ 行为向量（覆盖 REQ-GOBJ-HEADER-001）；其余 21 条（ID/DOMAIN/TENANT/REF/BIND/AUTHDEL/MIG/VALID）仍仅挂 SPEC-CONTRACT-COVERAGE-001。
- **处置**：绑定一致性五条（BIND-001..005）与 tenant 不可变（TENANT-001）应在 M3 补行为向量；schema 形状类（VALID/REF）由 M1 静态校验兜底。列入 §7.5。

### F10（P2，残余）：conformance README 15 层与向量 layer 标记 14 值无正式映射

- **证据**：README 列 15 个累积测试层；74 个向量实际使用 14 个 `layer` token（`contract-traceability` 等不在 15 层命名中）。
- **处置**：M1 runner 建立 layer token↔15 层的机器映射表；本轮不改动（避免无实现对照的批量重标）。

---

## 3. OS 边界与最小 Core 判定

### 3.1 OS 判据

§3.1 的 10 条判据（虚拟化/保护/并发/持久性/资源治理/设备中介/故障语义/系统调用面/可观察一致性/参考监视器）在**架构层**各有明确机制落点（§2.4 表、§7.1 最小可信计算基），且这些落点均有已登记对象/REQ/错误码支撑——这区别于"品牌性命名"。但"是 OS"的最终证据是判据 9（可观察一致性）与判据 10（不可绕过监视器）的**行为测试**，当前不存在。判定：**OS 命题在规范层成立、待实现证明；白皮书自我定位（informative、定稿≠实现）与此一致，无虚假声明。**

CognitiveOS 与宿主 OS/Runtime/工作流/DB/消息系统/MCP/A2A 的边界（§3.2–§3.5）经逐条核对无职责复制：它规定这些设施共同承载的对象语义与治理边界，不重新实现事务/传输/编排。双内核、三平面、七层、横切 Context 四视图无循环依赖（Context Engineering 显式不构成第四 authority 平面，决策十二）。

### 3.2 最小 Core 规模（首个单节点实现的实测估算）

| 维度 | 数量 | 来源 |
|---|---|---|
| 最小核心对象 | 14 | 附录 A.1 |
| 执行生命周期状态机 | 5（54 状态、90 迁移、13 终态） | specs/transitions/ 实测 |
| 最小传输无关操作 | 16（Core §12）+ 管理门禁 5 条 | Core §12/§12.1 |
| 生态扩展操作（M6+ 才需要） | 13 | Core §17 |
| 错误码 | 55 | errors.yaml |
| 每个 governed_external Effect 的同步持久化写 | 5–7 次（Intent、authorization decision、dispatch event、receipt/reconcile、verification、commit event；含 outbox 合并可至 4–5） | Core §9 + effect.transitions 证据字段 |
| 持久存储 | 单节点 8–10 张表/日志（对象、事件、Effect、checkpoint、capability、审计、快照、幂等索引） | M2 交付推导 |
| 适用 R0/R1 向量子集 | ≈55/74（排除 dist/emb/het/learning 专属） | 向量 profiles 字段 |

**R0/R1 单节点竖切可行**：§20.5 降级映射（事件日志=追加文件、Context 门禁=确定性选择、TaskContract=静态模板、验收=用户本人）+ §20.5 不可降级边界（tenant/scope 过滤、注入不提升、缓存绑定、无外部写）给出了合法最小形态；tracer bullet（§7.5）可在 M4 末端到端走通。**规范表面积（273 REQ）大于首版可验证面**是已识别风险，缓解是 §21 冻结条款 + profiles 声明边界（R0/R1 只需 core_digital 适用子集）。

### 3.3 非 Core 子系统处置结论

| 子系统 | 结论 | 依据 |
|---|---|---|
| SMS/CRB、语义检索排序 | **保留为可选 Profile，延后到 M8**；收益按评测合同 C 类验收 | 决策二十一；Core 正确性不依赖 |
| 多 Agent/分布式 | **延后 M9**；lease/epoch/fencing 语义已登记，不阻塞单节点 | REQ-DIST-*；V10 |
| 具身/CIM | **延后 M10**；双内核边界保留在白皮书，实现细节已下沉 companion | IMP-13 已应用 |
| 受控学习/知识编译 | **延后 M11**；知识对象仍为伪 schema（已如实标注） | RFC-0001 §21 |
| 审批分发子系统（§12.12） | **保留设计，REQ/向量登记为发布前 P1 义务**（见 §7.5） | 1.0.0 递延声明 |
| "认知经济学"统一市场 | **维持降格**（可观测核算，非机制） | V16 |

---

## 4. 组件完备性矩阵

14 要素：目的 / owner-authority / 输入输出 / 持久对象 / 状态机 / 不变量 / 并发 / 失败语义 / 恢复 / 安全边界 / 可观测 / 性能预算 / 版本兼容 / 测试证据。状态：●=已登记且有专属真实向量；◐=语义已登记、向量为 stub 或共享兜底；○=仅设计文本。

| 组件 | 状态 | 主要缺口（→处置） |
|---|---|---|
| Effect/Intent | ● | 形式模型未交付（→Phase 4/M4，§7.5） |
| Task/验收 | ● | 验收容量（人审带宽）是部署声明，无法预先测试 |
| AgentExecution/Checkpoint | ◐ | 恢复向量真实（AGENT-RECOVERY-003、RECOVERY-CRASH-006），但升级重放隔离（§18.6）无专属向量 |
| Loop | ◐ | LOOP-CONTRACT/GATE/VERIFY 三向量为 stub（→M1 行为化） |
| Verification | ● | PASSED→EXPIRED 有迁移表与 guard；independent-verifier 独立性度量是开放问题（§22.5） |
| Context Resolution | ● | 九阶段/required/注入/渲染稳定/rank-before-auth/撤销缓存均有真实向量 |
| Capability/授权 | ◐ | 衰减真实（CAP-ATTEN-004）；lease/bind 为 stub（→M3） |
| 事件/State/CAS | ● | CAS、状态闭包、存储降级真实；背压（REQ-EVT-005）无专属向量（→M2） |
| 治理对象族（GOBJ） | ◐ | F9：21/22 无行为向量（→M3） |
| Governed Memory | ● | 准入/晋升/失效/RYW 四向量真实 |
| Operation Catalog | ● | 效果混淆/陈旧绑定真实 |
| Management Shell | ● | 12 个 management 向量真实；审批分发 REQ 递延（→发布前登记） |
| Agent 安装/适配 | ● | 绕过/带外/恢复/安装验证真实 |
| 性能/收益合同 | ● | 本轮闭合（F1）；数值门槛是候选预算（hypothesis，→Phase 1 校准） |

## 5. 26 类故障场景推演

每行给出：预期终点状态（唯一安全行为）/ 机器落点 / 证据状态。标记：**[有]**=REQ+真实向量；**[补]**=本轮补齐；**[registered]**=语义已登记但向量为 stub 或缺专属向量；**[设计]**=仅白皮书文本。

| # | 场景 | 唯一安全行为与恢复出口 | 落点 |
|---|---|---|---|
| 1 | Intent 持久化后、调用前崩溃 | 恢复对账确认未分派→用原幂等键分派一次；无重复 Effect | [补] EFF-CRASH-001、REQ-EFF-006 |
| 2 | 外部已执行、receipt 前崩溃 | OUTCOME_UNKNOWN→查询对账 executed→RECONCILED→验证；禁止盲重试/换键 | [补] EFF-CRASH-002 |
| 3 | 验证后、commit 前崩溃 | 重放证据后直接 commit，不重执行；后态已变则 Verification EXPIRED、阻断 commit | [补] EFF-CRASH-003 |
| 4 | Effect 重复/乱序/超时/同键异参/不可查询 | 重复→幂等去重；同键异参→EFFECT_IDEMPOTENCY_CONFLICT 拒绝；不可查询 still_unknown→补偿（独立授权）或 QUARANTINED | [有]+[补] EFF-UNK-003、EFF-IDEM-CONFLICT-001、effect-state-closure-008 |
| 5 | 两 writer 并发 CAS | 恰一成功；另一 STATE_CONFLICT，状态不变 | [有] STATE-CAS-002 |
| 6 | 旧 epoch writer 分区后继续 | 资源端按 fencing token 拒绝旧 epoch 提交 | [registered] REQ-PROFILE-DIST-001 + transitions guard`fencing_epoch_current`；PROFILE-DIST-001 为 stub（→M9 前行为化） |
| 7 | Context 解析后、执行前撤销 capability | dispatch/commit 门禁重验 `capability_and_revocation_current`→拒绝；缓存视图不得复用 | [补] CTX-REVOKE-CACHE-001 + effect 表 v0.2 guard（dispatch 与 commit 两边） |
| 8 | 同 tenant 横向读取 | CONTEXT_AUTH_DENIED；membership 不构成读取权 | [补] GOBJ-TENANT-LATERAL-001 |
| 9 | 跨 Conversation KV/working-set 污染 | 未准入候选跨 Conversation 读取被拒；复用执行必须六步隔离 | [有] MEM-RYW-001、SHELL-CHANNEL-ISOLATION-003 |
| 10 | 管理员未经正文授权读取数据 | 拒绝；管理权≠正文读取权，break-glass 独立限时授权 | [registered] REQ-GOBJ-HEADER-001/RFC-0001 §3；管理员变体专属向量缺（→M3） |
| 11 | ranker/renderer/工具描述/远端内容注入 | 注入内容保持数据语义；控制不可变更；越权 Effect 被确定性门禁拒绝 | [有]+[补] CTX-TRUST-004、CTX-RANK-AUTH-001、CAT-EFFECT-CONFUSION-002 |
| 12 | required 超预算/过期/冲突/不可访问 | fail closed（CONTEXT_BUDGET_EXCEEDED/INCOMPLETE）；停滞出口 WAIT/ESCALATE/授权 partial/STOP | [有] CTX-REQ-007、DISC-STAGNATION-004 |
| 13 | Agent 声称 done 但后态不满足 | 只到 CANDIDATE_COMPLETE；验收 authority 依据 PASSED+当前后态才推进 COMPLETED | [有] INTENT-ACCEPTANCE-007 + task.transitions guards |
| 14 | 远端 Task completed 但本地验证不通过 | 远端 completed 仅为 observation evidence；本地 Task 不迁移 | [补] GW-REMOTE-COMPLETE-001 |
| 15 | 工具 HTTP 200 但外部效果未发生 | receipt 仅执行证据；reconcile 三分/verifier 后态判定；VERIFY_FAILED→ABORTED/补偿 | [registered] REQ-EFF-003 + transitions；专属向量缺（→M4 故障注入覆盖） |
| 16 | Shell 断开 | 执行继续；重新认证 attach 恢复观察；取消≠远端已停止 | [有] SHELL-DETACH-ATTACH-004、SHELL-CANCEL-SEMANTICS-005、MGMT-RECOVERY-007 |
| 17 | 审批重复/轰炸/旧 proposal 被确认 | 频控+聚合+连续拒绝告警；旧 proposal→SHELL_PREVIEW_STALE/INTENT_VERSION_SUPERSEDED；R2/R3 独立批准 | [registered] §12.12 设计 + MGMT-APPROVAL-005；轰炸/限流向量为候选（→发布前登记，§7.5 P1） |
| 18 | Agent 伪造审批卡片 | 确认面渲染在被审对象不可写表面或用不可预知一次性挑战；假卡片无有效短码不生效 | [设计] §12.12/§17.2；向量候选（→发布前登记） |
| 19 | legacy Agent 直接改 memory 文件 | digest 漂移检测→ingest 为候选→准入或标冲突；双向不静默覆盖 | [有] AGENT-OOB-001 |
| 20 | legacy Agent 绕过 proxy/网络/secret/子进程/IPC | AGENT_ADAPTER_BYPASS_DETECTED；Profile 降级 | [有] AGENT-BYPASS-002 |
| 21 | 数据库只读/磁盘满 | STATE_STORE_UNAVAILABLE fail closed；禁内存缓冲冒充提交；保只读/停止/撤销/审计导出 | [补] STATE-STORE-DEGRADE-001、REQ-REC-003 |
| 22 | 事件消费者严重落后 | 有界阻塞/拒绝/溢写/标记；governed event 不无标记丢弃；watch cursor 过期→新快照 | [registered] REQ-EVT-005 + SHELL-WATCH-RESUME-006（cursor 侧）；背压专属向量缺（→M2） |
| 23 | 遥测丢失 | 不影响权威状态与 Effect 正确性；三类记录分离 | [补]（部分）STATE-STORE-DEGRADE-001 断言 + §19.1 |
| 24 | 模型/SMS/ranker 不可用 | SEMANTIC_SERVICE_UNAVAILABLE≠无结果；降级不扩权；确定性 inspect/stop/revoke/reconcile 保持可用 | [有] SEM-FALLBACK-001、MGMT-FALLBACK-008 |
| 25 | schema/策略/模型/renderer/verifier 升级后恢复旧 checkpoint | RECOVERING→RUNNABLE guard `continuation_versions_current`；版本不符→重验/迁移，不静默采用新逻辑重放 | [registered] transitions + AGENT-RECOVERY-003 + REQ-GOBJ-MIG；升级重放隔离专属向量缺（→M4） |
| 26 | 多 Agent 重复劳动/互等/错误共识/委派扩权 | 委派单调衰减（CAP-ATTEN-004 拒绝扩权）；多副本一致≠事实；重复劳动由 §19.4 协作指标计量而非机制保证 | [有]（扩权）+[设计]（效率类→M9 前按评测合同 W3 登记） |

**结论**：26 场景全部有唯一安全行为定义；其中 19 个有真实向量（含本轮 8 个）、5 个语义已登记但专属向量列入里程碑清单、2 个（审批伪造/轰炸）为白皮书设计+候选向量，构成发布前 P1 义务。无 P0 设计缺口残留。

## 6. 既有结论逐条处置

### 6.1 v1.2 §2 设计基线 V1–V17

| ID | 处置 | 说明 |
|---|---|---|
| V1–V13、V15–V17 | **confirmed** | 本轮资产核验与故障推演未发现反例；V4/V5（Effect/恢复）由升级后的 EFF-CRASH 向量进一步机器化；V7 由 CaMeL 原文复核加强 |
| V14（引用真实性） | **confirmed（抽样扩展）** | 本轮再核验 CaMeL 与供应商缓存计价；全部真实。注意：IMP-02 的"2–10 倍 TTFT"上界在公开延迟数据中偏乐观（成本维度成立），白皮书"数倍"表述无需修正 |

### 6.2 v1.2 §3 改进项 IMP-01–18

| ID | 处置 | 说明 |
|---|---|---|
| IMP-01/02/03/04/06/08/09/10/11/12/13/14/15 | **confirmed** | 1.0.0 已应用且本轮实测其机器资产真实存在（REQ-CTX-012/MEM-ADMIT-002/AGENT-OOB-001/PERF-004 三角闭合） |
| IMP-05（审批子系统） | **confirmed + 残余标注** | 文本已应用；REQ/向量递延是 1.0.0 显式决定，本报告将其转为发布前 P1 义务（§7.5），防止递延变遗忘 |
| IMP-07（形式化） | **confirmed + 义务化** | 路线图承诺真实；本报告 §7.5 固化形式模型义务集（7 条性质）与验收判据 |
| IMP-16（瘦身） | **confirmed（持续）** | 本轮净增约 19 行（1916→1935），全部为规范边界/版本记录，符合"净增有理由" |
| IMP-17（资产漂移） | **revised** | v1.2 判其"已过时"正确；但 v1.2 自身 §1 的计数（40 schema/65 向量/244 REQ）同样过时——资产计数必须随每轮实测（见 §1 表） |
| IMP-18（性能路径） | **revised（扩展）** | "不新建性能基建层、优化后置、性能前提现在定型"三判断 confirmed；但其遗漏了收益**声明合同**本身也是"现在不定、之后就是虚假宣传通道"的契约形状——本轮以 F1 修复（评测合同+REQ-PERF-005）。IMP-18 的 M6 A/B 对齐报告已升级为绑定四臂合同的预注册对照（§21 Phase 1） |

### 6.3 v1.2 其他内容

- §1 评审方法与外部证据附录：**confirmed**（抽查引用真实）。
- §5 开发计划衔接表：**unverified→归档**（开发计划已归档 History/，衔接点由白皮书 §21 与本报告 §7.5 承接）。
- §6 实施顺序：**已完成历史使命**（三批次均已执行）。

### 6.4 独立审查报告（F-001–F-030）逐条处置

对 [CognitiveOS-Architecture-Independent-Review.md](./CognitiveOS-Architecture-Independent-Review.md) 的 30 条 finding，本报告逐条独立复核（不预设其正确）后处置如下。凡"已修复"均经静态反例复验（修复前反例 schema-valid / 修复后被拒）：

| Finding | 复核结论 | 处置 |
|---|---|---|
| F-001（零实现） | confirmed | 与本报告 F7 一致；判定框架而非可修缺陷 |
| F-002（17 条 REQ 白皮书拥有） | confirmed | 已修复（=本报告 F3，18 条 repoint） |
| F-003（legacy metadata/strongRef 双头并存：30 份 schema 用 legacy metadata、22 份用 legacy strongRef） | **confirmed（复测证实）** | **残余 P0→M0 迁移义务**：governed-object-contract §3 已登记差异与迁移要求；一次性迁移 30 份 schema 超出本轮最小闭合边界，列为 M0 合同车道首项（§7.5），迁移前 legacy 形状按已登记差异解释 |
| F-004（ActivityContext↔ContextView 循环） | confirmed（复测：`context_view_ref` 必填 + View 绑定 Activity） | 已修复：两阶段创建 + CAS 恰一绑定（schema 必填集缩小 + REQ-GOBJ-BIND-004 条文） |
| F-005（Effect 冗余真相/非法组合 schema-valid） | confirmed（复测：`COMMITTED+unknown+pending` 修复前通过） | 已修复：effect schema 条件闭包；反例复验被拒 |
| F-006（ABORTED 掩盖外部残留） | confirmed | 已修复：`VERIFIED/VERIFY_FAILED→ABORTED` 增 `residual_effect_disposition` guard+证据 |
| F-007（Context 后撤销竞态） | confirmed（复测：dispatch/commit 无 revocation guard） | 已修复：`capability_and_revocation_current` guard 加入 dispatch 与 commit 边 |
| F-008（存储失败无语义） | confirmed | 已修复（=本报告 F4，双方独立收敛于同一缺口） |
| F-009（digest 自引用无投影） | confirmed（全仓无 digest_excluded 声明） | 已修复：REQ-GOBJ-REF-004 登记默认投影（排除自身 content_digest/signature 路径） |
| F-010（Checkpoint 缺恢复稳定事实） | confirmed | 已修复：LoopCheckpoint 增 event_high_watermark/fencing_epoch/pending_effects/pinned_environment_versions（可选加法） |
| F-011（审批基线 informative） | confirmed | 残余 P1（=本报告场景 17/18 处置；发布前登记义务） |
| F-012（TaskContract 闭合枚举/无验收条件） | confirmed（复测：wait-only contract 修复前通过） | 已修复：开放域名 + acceptance contains 约束；反例复验被拒 |
| F-013（幂等冲突未登记） | confirmed | 已修复（=本报告 F5，独立收敛） |
| F-014（fencing 提交端覆盖未证明） | evidence-backed judgment，成立 | 列入 §7.5：M4 交付 commit-sink inventory（state store/event store/executor/outbox/memory publish/approval/device 逐端 fence 或 not-applicable） |
| F-015（向量数≠行为覆盖） | confirmed | 部分修复（EFF-CRASH 行为化 + 6 新负例）；剩余以 §7.5 M1"generic vector 只算静态 lint"规则承接 |
| F-016（implemented 可被 degradation 稀释） | confirmed | 已修复：conformance README 收紧（degradation 必须缩 scope 或降 experimental；安全负例不可降级豁免） |
| F-017（sandbox 证据粗粒度） | confirmed | 残余 P2→M6：host-specific interception matrix 与逐通道 escape tests（§7.5） |
| F-018（untrusted+control schema-valid） | confirmed（复测通过） | 已修复：trust/role 条件约束；反例复验被拒 |
| F-019（RYW 不可机器表达） | confirmed | 已修复：MemoryCandidate `pending_visibility`（writer_private/expiry/trust_role） |
| F-020（性能 schema 不支撑 A/B） | confirmed | 已修复（=本报告 F1，独立收敛；comparison 块+REQ-PERF-005+负例断言） |
| F-021（相对顺序≠前缀稳定） | confirmed（供应商按累计前缀字节匹配） | 已修复：REQ-CTX-012 补前缀段字节稳定 + immutable prefix/append-only suffix/cache breakpoint 语义；CTX-RENDER-001 v0.2 增 `existing_prefix_segment_bytes_unchanged` 断言 |
| F-022（SLO 无阈值落点） | confirmed | 已修复：slo_profile.thresholds（metric/comparator/percentile/risk_class/workload/release gate/on_breach） |
| F-023（不可查询执行器准入） | 开放问题，成立 | 列入 §7.5 M4：默认拒绝 + residual-risk authority 例外的候选 REQ 设计 |
| F-024（审批服务缺 owner） | confirmed | 已修复：§4.7 责任矩阵补审批服务行（明确不拥有 capability 签发与确认决定权） |
| F-025（OS 身份缺实现证据） | confirmed | =本报告 §3.1 判定 |
| F-026（收益处于 E0 假设） | confirmed | =本报告 F1 修复后的声明边界：当前只能声明 hypothesis |
| F-027（规范面＞最小 Core） | confirmed | =本报告 §3.2 风险与冻结缓解 |
| F-028（外部证据分级漂移） | 复核：属实但影响小 | 残余 P2：附录 C 个别条目发表状态随时间漂移，M0 文档车道校准 |
| F-029（开发计划归档断链） | confirmed | §7.5 已承接 M0–M6 门禁；实现启动时恢复计划文档 |
| F-030（高级 Profile 延后） | confirmed | =本报告 §3.3 |

## 7. 正确性与收益证据闭环

### 7.1 已机器化（规范层闭合）

- 273 REQ ↔ 74 向量双向无孤儿；owner_spec 全部指向 normative 资产（F3 修复后）；
- 5 状态机迁移表（guard/reason/evidence 齐全）与白皮书文本一致；effect 表 v0.2 增撤销/持久化/残留处置 guard；
- 55 错误码全部被引用处存在；两个新码有 REQ 与向量绑定；
- 收益声明合同：REQ-PERF-005 + comparison schema + 正负例向量；SLO 阈值有机器落点（slo_profile.thresholds）；
- 治理开销合同：REQ-PERF-004 + governance_overhead schema + 向量断言；
- 非法组合封闭（静态反例复验）：`Effect COMMITTED+unknown+pending`、无验收条件的 TaskContract、`untrusted+control` ContextView item 修复后均不再 schema-valid；
- 恢复稳定事实与 RYW 隔离可机器表达（LoopCheckpoint/MemoryCandidate 扩展）；digest 自引用有默认投影（REQ-GOBJ-REF-004）。

### 7.2 已有测试 / 已执行测试

**无**（实现 0 行、runner 不存在）。74 个向量是声明式合同；本轮"验证"限于静态一致性检查（§8）。这是当前最大的诚实边界。

### 7.3 形式化证据

**无已交付**。义务集（Phase 4/M4 交付，作用域 = effect.transitions × 恢复顺序 × fencing epoch，TLA+/Alloy 有界模型）：

1. 不重复提交外部效果（任意 crash/重放调度下 `duplicate external effect = 0`）；
2. 旧 epoch writer 不能提交（fencing 安全性）；
3. `OUTCOME_UNKNOWN`/`still_unknown` 不可达 `COMMITTED`（无路径伪装成功）；
4. receipt 或模型自述不能使 Task 到 `COMPLETED`（验收路径唯一性）；
5. 恢复门禁顺序不可跳过（barrier→epoch→fence→replay→reconcile→reauthorize→resolve→resume 的顺序违反不可达 RUNNABLE）；
6. capability 撤销后不能继续提交（撤销版本单调性）；
7. 同键异参必须拒绝（幂等绑定唯一性）。

验收判据：模型 checker 在声明的有界状态空间内零反例；每条性质映射回 REQ 与向量；形式模型不替代 M4 故障注入测试。

### 7.4 仍只是设计（无机器资产）

审批轰炸/伪造向量（候选）；多 Agent 协作效率指标族（M9 前候选）；知识对象机器 schema（伪 schema，M11）；workload-specific 开销预算数值（候选预算，Phase 1 校准）。

### 7.5 实现就绪清单（发布门禁）

**Tracer bullet（M4 末验收）**：单节点、SQLite、硬编码 Operation，走通 `Task → Context resolve → Intent → Effect(Authorize/Execute/Reconcile) → Verify → Commit → Event append → kill -9 → 恢复顺序八步 → 重放 digest 一致`。oracle：EFF-CRASH-001/002/003、RECOVERY-CRASH-006、STATE-STORE-DEGRADE-001 行为通过；恢复后无重复 Effect、无丢失提交。

**不变量→测试映射（节选，全量在 registry tests 字段）**：

| 不变量 | REQ | 向量 | 测试层 |
|---|---|---|---|
| 权威（CAS/fencing） | STATE-003、PROFILE-DIST-001 | STATE-CAS-002、PROFILE-DIST-001(待行为化) | 2/8 |
| 证据（验收唯一路径） | INTENT-ACCEPT-001、GW-002 | INTENT-ACCEPTANCE-007、GW-REMOTE-COMPLETE-001 | 6/15 |
| 恢复（三崩溃点+存储降级） | EFF-006、REC-001/002/003 | EFF-CRASH-001/002/003、STATE-STORE-DEGRADE-001 | 3 |
| 语义隔离（注入/rank/缓存/横向） | CTX-002/008、SEC-002、PROFILE-CVM-001、GOBJ-HEADER-001 | CTX-TRUST-004、CTX-RANK-AUTH-001、CTX-REVOKE-CACHE-001、GOBJ-TENANT-LATERAL-001 | 4/5 |
| 幂等（同键异参/unknown） | EFF-002/004、MGMT-IDEM-001 | EFF-IDEM-CONFLICT-001、EFF-UNK-003、MGMT-IDEM-006 | 3 |

**里程碑正确性门禁（M0–M6，承接归档开发计划并补充本轮项）**：

- M0：本报告 §8 静态检查进 CI；Rust/TS golden digest 一致；全 profile 标 planned。
- M1：runner 交付；**18 个 stub 向量行为化或显式降层为 schema 检查**（F8）；layer 映射表（F10）；schema-valid≠pass 的 mutation 测试。
- M2：CAS 并发/事件不可变/投影重放 digest 稳定/**背压专属向量**（场景 22）。
- M3：授权交集/撤销传播/**GOBJ 绑定五条行为向量 + 管理员正文拒绝向量**（F9、场景 10）/rank-before-auth 与撤销缓存行为通过。
- M4：三崩溃点故障注入 + tracer bullet + **形式模型七性质**（§7.3）+ 升级重放隔离向量（场景 25）。
- M5：管理 fallback（无模型 inspect/stop/revoke/reconcile）行为通过；**审批分发 REQ/向量登记**（场景 17/18，含轰炸限流与假卡片负例）。
- M6：C1 不可绕过负例全通过；**Phase 1 A/B 非劣化报告**（评测合同 §8 条款 5：A/B 两臂、非劣化判定、开销预算校准）。

**性能 Profile 收益门禁（M7/M8）**：每个 C 类优化交付时按评测合同四臂+预注册门槛验收；C≤B 即触发 kill criteria（experimental/延后/删除）；收益声明必须 `claim_level=significant_benefit` 且六条门槛全过。

**候选预算（hypothesis，Phase 1 实测校准）**：W1 治理延迟占比 ≤3%(p50)/≤8%(p99)、费用 ≤2%、cache 保持 ≥0.90；W2 ≤10%/≤20%、≤5%、≥0.90；W3 ≤15%、≤8%、≥0.85。校准前禁止引用为已达成。

**未决研究问题**：白皮书 §22 的 18 项维持开放；本轮新增：预算候选值的实证校准方法、stub 向量行为化的 oracle 设计。

**明确非目标**：白皮书 §1.4 不变；本轮补充——不在无实现对照时批量生产行为断言；不为静态检查通过而弱化向量。

**发布阻断 P0（任一未满足则不得声明 `implemented`）**：

1. runner 存在且适用向量按行为判定（非 schema-parse）；
2. 三崩溃点 + 存储降级 + fencing 行为测试通过；
3. 安全负例十类（跨 tenant/scope/Conversation、撤销缓存、rank-before-auth、注入越权、审批伪造、sandbox 绕过、通道混用、remote completed）全部确定性拒绝，安全失败计数不可被综合分抵消；
4. 无模型/无 Shell 时确定性 inspect/stop/revoke/reconcile/quarantine/审计导出可用；
5. Phase 1 A/B 非劣化报告发布且达标（未达标只能 experimental）；
6. 审批分发 REQ/向量完成登记（管理 Shell profile 声明的前置）；
7. 形式模型七性质零反例（R2 及以上声明的前置）。

## 8. 验证结果（本轮实际运行）

| 检查 | 结果 |
|---|---|
| 全仓 JSON/YAML 解析（56 schema、74 向量、3 registry、5 迁移表） | 通过，零解析错误 |
| REQ↔向量双向孤儿 | 零孤儿（273 REQ tests 字段 ↔ 74 向量 id 双射） |
| 向量引用 REQ 存在性 | 全部存在 |
| 向量引用错误码存在性 | 全部已登记（含两个新码） |
| owner_spec 文件存在性 | 273 条全部可解析到存在的文件 |
| 迁移表状态封闭性（无未声明状态、终态无出边） | 通过（5 域、90 迁移） |
| performance-report 向量实例 vs 扩展后 schema（jsonschema draft 2020-12） | 通过 |
| 白皮书↔registry REQ 引用一致性 | 白皮书引用的全部 REQ-ID 均已登记 |
| 编辑文件 linter | 无错误 |
| **未运行**：conformance runner（不存在）、行为测试（无实现）、形式模型 checker（未交付）、Rust/TS golden digest（无实现）、外部链接全量可达性（仅抽查 CaMeL/缓存计价/MITRE 等关键项） | 如实标注 |

## 9. 残余风险与下一步（按优先级）

1. **[P0→M0] legacy metadata/strongRef 双头迁移**（§6.4 F-003）：30 份 schema 用 legacy metadata、22 份用 legacy strongRef，与 governed-object 合同并存；M0 合同车道一次性迁移到 GovernedObjectHeader/object-reference 形状（合同已登记映射规则），迁移前不得混合解释。
2. **[P1] 审批子系统 REQ/向量登记**（场景 17/18，F-011）：M5 前完成，否则 intelligent_management_shell profile 不得声明。
3. **[P1] 18 个 stub 向量行为化或显式降层**（F8/F-015）：M1 runner 交付时逐个处置，PROFILE-DIST-001 与 LOOP 三向量优先；generic traceability 向量只计静态 lint，不计行为覆盖。
4. **[P1] commit-sink fencing 清单**（F-014）：M4 交付 state store/event store/executor/outbox/memory publish/approval/device 逐端 fence 矩阵或 not-applicable 判定。
5. **[P2] GOBJ 绑定行为向量**（F9）与管理员正文拒绝向量（场景 10）：M3。
6. **[P2] 背压（场景 22）、HTTP-200-无效果（场景 15）、升级重放隔离（场景 25）专属向量**：M2/M4；不可查询执行器默认拒绝候选 REQ（F-023）：M4。
7. **[P2] layer 映射表**（F10）、sandbox 逐通道 escape 矩阵（F-017，M6）、附录 C 发表状态校准（F-028，M0）。
8. **[校准] workload 开销预算与 cache 门槛**：Phase 1 A/B 报告实测后以新版本固化。
9. **启动 M0**：仓库骨架、CI 静态检查（§8 清单固化为脚本）、双语言 digest 工具——无阻塞项。
