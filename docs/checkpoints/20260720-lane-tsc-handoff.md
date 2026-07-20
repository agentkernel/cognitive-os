# 20260720 Lane-TSC Handoff（客户端骨架批）

## 1. 本次会话完成

按 `docs/prompts/lane-tsc.md` 任务范围交付 M5 集成前的客户端侧全部工作（分支 `lane/tsc`，基线 `b626e88`；提交哈希见 §6）。全部为**实现已提供 + 包内单元测试已执行**；不声称向量已执行、不声称 Profile 已符合（向量执行归 Lane-CFR runner）。

### packages/sdk-ts（8 模块，63 单元测试）

- **`errors.ts`** — 55 个已登记错误码的钉扎表 + 契约驱动重试分类（error-contract §3；REQ-ERR-001/002）：`retryable:false` 永不重试；`EFFECT_OUTCOME_UNKNOWN` → 仅 reconcile（禁盲重发）；`STATE_CONFLICT` → 必须重读 authority 状态；未登记码 = 缺陷，fail closed；wire 与 registry 冲突取不扩大风险侧。**漂移门**：测试逐条对读 `specs/registry/errors.yaml`，registry 变更即红灯。
- **`envelope.ts`** — AKP 请求/结果信封构造与解析（specs/akp §3/§5/§9；REQ-AKP-ENV-001/002、VER-001、CAN-001、IDEM-001、RES-001）。canonical/digest 全走 contracts-ts；接收路径按契约顺序 fail closed：strict parse → shape → 版本 → unknown critical extension → payload digest。effecting 无幂等键拒发；`accepted`/receipt 原样透传，无任何状态改写 helper。
- **`channel.ts`** — 通道品牌化凭据（类型级：`ChannelCredential<"task"|"management">` 跨通道赋值即编译错误）+ 运行时 `ChannelBindingViolation` fail closed + `ProjectionStore`（通道+凭据分区缓存，密钥永不进缓存键，authority 版本单调、只读冻结视图；REQ-SHELL-CHANNEL-001、REQ-SHELL-STATUS-001）。
- **`transport.ts`** — 可注入 `AkpTransport` 接口 + `InMemoryTransport`（可脚本化 fake，记录全部请求/流信封供负例断言）+ `HttpSseTransport` 默认实现（注入 fetch；task/management 端点根分离；路由为 M5 前临时值）。传输状态码永不当结果（REQ-GW-002 类比）。
- **`client.ts`** — `TaskChannelClient` / `ManagementChannelClient`（一实例一通道一凭据一传输一投影缓存）。请求管线：重试仅按 registry 分类驱动；重发复用同一幂等键 + 新 message_id（AKP §6）；传输层失败于 effecting 操作 → 有界重投后抛 `OutcomeUnknownError`（对账信号，不造新键）；task 客户端只发 AKP §13 列名的 12 个任务通道操作，management 客户端拒发之（混用在发出前 fail closed）。
- **`views.ts` + `fixtures.ts`** — shell 族 5 schema 的临时手工绑定（严格对 schema 建模；生成物已有的 Budget/Digest/UriRef/StrongReference/GovernedObjectHeader 一律消费 contracts-ts 生成绑定）+ schema-valid 样例构造器。**漂移门**：ajv 按真实 schema 验证全部样例（含 management 通道条件约束负例）；`SHELL_SCHEMA_DIGESTS`（canonical bytes，domain `schema-bundle/0.1`，与 codegen 头一致配方）测试时从 `specs/schemas/` 重derive。
- **`watch.ts`** — snapshot+cursor watch 消费器（异步迭代器；event-audit-watch §5；REQ-SHELL-WATCH-001、REQ-AKP-SHELL-002、REQ-AKP-STR-001/002）：一次快照 + 有序增量；at-least-once 按 sequence 去重；断流从 last-ack cursor `watch.resume`；`WATCH_CURSOR_STALE` → 丢弃位置、`watch.open` 取新授权快照并向应用 yield 新 snapshot（re-base，绝不静默跳缺口）；缺口触发有界恢复，耗尽 fail closed。负例：乱序/重复/缺口/持续缺口/快照前增量/带登记码的流错误/畸形帧。
- **`index.ts`** — 汇总导出 + 编码 profile 常量。

### apps/agent-shell（会话层，12 单元测试）

- **`session.ts`** — proposal → preview → submit → attach/watch → detach/cancel 交互流骨架（task-loop-verification §6；REQ-SHELL-DETACH-001、ATTACH-001、CONTROL-001、STATUS-001、PREVIEW-001；REQ-AKP-SHELL-001/003）。CLI 展示层后置 M5，命令层与状态机完整：
  - `phase` 为纯客户端 UI 状态；展示任务状态唯一来源 = 投影缓存（watch 注入的 ShellStatusView，按 target 身份 + `target_version` 单调）。
  - **detach ≠ cancel**：detach 停止消费、保留 cursor、零请求发出（测试断言请求日志为空）；reattach 走 `watch.resume` 从保留 cursor 恢复（vector shell-detach-attach-004 的客户端侧行为）。
  - **cancel 经 Effect 闭合**：`shell.control` 单发不重试，`cancel_pending`/`CANCEL_TOO_LATE` 映射为 disposition 原样上抛；展示状态不因 cancel 请求本身变化，只有 authority 投影 `cancelled` 才显示已取消（vector shell-cancel-semantics-005 的客户端侧行为）。
  - **remote completed ≠ 状态**：远端完成报告类 delta 不进投影（`isShellStatusView` 门），展示不变；仅 authority 状态视图推进展示（vector remote-completed-not-acceptance 的客户端侧行为）。`accepted` 提交结果同样不产生任何展示状态。
  - submit 固定 proposal/preview digest + 幂等键（信封与 payload 双处断言）；无 preview 拒发；异 digest preview 拒绑（stale preview binding）。

### 文档联动

- PROGRESS 车道栏 + handoff 列表 + 最后更新行；PARALLEL-LANES 所有权表（lane/tsc 已启动）；本 handoff。

## 2. 未完成 / 进行中

- **M5 集成**（入口 gate：Lane-RUN kernel-server）：真 HTTP+SSE 对接、`HttpSseTransport` 路由/头/SSE 事件格式对齐真实服务、`watch.ack` 服务端语义接入、双 OS 集成测试；M5 出口验收（通道隔离负例、detach/cancel/watch 语义**向量行为侧**执行）由 runner 完成。
- intent 修正流（`intent.supersede` 客户端命令 + 旧 epoch proposal 客户端侧丢弃）未做——依赖 IntentInterpretation/epoch 合同细化，排 M5 前后皆可。
- admin-cli 交互设计评审（Lane-RUN 属地、本车道协作项）未启动。
- CLI 交互展示层（终端 UI）后置 M5。

## 3. 测试与证据状态

- TS：`pnpm -r build; pnpm -r test` 全绿 = **104 测试**（contracts-ts 27 / tools 2 / **sdk-ts 63** / **agent-shell 12**），本地 Windows（Node 24）通过；CI 两 OS 状态见 PR checks。
- `pnpm run check:consistency` OK（273 REQ / 55 码 / 56 schema / 76 向量）。
- Rust 侧零触碰（无需重跑；CI 会照常跑）。
- **向量：76 全 not-run，无变化、无虚报**。本车道单元测试实现了 shell-detach-attach-004 / shell-cancel-semantics-005 / shell-watch-resume-006 / shell-channel-isolation-003 / remote-completed-not-acceptance 的**客户端侧行为语义**，但不构成向量执行（runner 归 Lane-CFR）。
- 新增证据均为包内测试输出；无 artifacts/evidence 产物。

## 4. 未决风险与漂移（契约缺口清单，待 Lane-CTR）

以下为发现的合同缺口，**均未自行修改契约资产**（车道边界）；按契约流程应由 Lane-CTR 处置。皆非漂移（无既有资产互相矛盾），故未登记 findings-ledger 漂移节；列此供 Lane-CTR 排批：

1. **AKP envelope 无机器 schema**：请求/结果信封字段名目前按 specs/akp §3 散文在 `sdk-ts/envelope.ts` 单模块内定名（`in_reply_to`/`result_digest` 等为临时命名）。建议登记 envelope schema + codegen 绑定。
2. **errors.yaml 无生成绑定**：sdk-ts 以钉扎表 + 测试时对读 YAML 兜底；建议 codegen 直接发射 Rust/TS 错误注册表。
3. **shell 族 5 schema 不在 IMP-08 codegen 集**（shell-action-proposal / shell-command-preview / shell-status-view / watch-subscription / user-intent-record）：sdk-ts `views.ts` 手工绑定 + ajv/digest 双漂移门兜底；建议纳入 codegen 后删除手工绑定。
4. **`shell.control` 无 payload schema**（REQ-AKP-SHELL-003 只定了结果区分）：`CancelControl` 形状与 `SHELL_CONTROL_PROVISIONAL_PIN`（模块加载时派生的临时 digest，真实服务端会拒绝——属预期 fail closed）为临时物。
5. **生成模块不导出 schema digest 运行时常量**（现仅在文件头注释）：客户端信封 `schema_digest` 钉扎需要它；现以 `SHELL_SCHEMA_DIGESTS` 钉扎 + 重derive 测试兜底。
6. **watch 流帧（AKP §8 stream fragment）无机器 schema**：`WatchStreamFrame` 形状在 `sdk-ts/watch.ts` 临时定义。
7. **management 通道操作名未登记**：ManagementChannelClient 暂只实施反向门（拒发任务通道操作族）。
- 风险：以上临时命名/临时 pin 在 M5 对接真 kernel-server 时必然暴露为拒绝（fail closed，符合预期）；M5 前 Lane-CTR 落 1/4/6 即可消除。
- 无新增 REQ/错误码/schema/向量；规范表面零触碰。

## 5. 下一步入口

- **Lane-CTR**：按 §4 清单排契约批（优先 1/3/4）；落地后 sdk-ts 删手工绑定换生成物（`views.ts` 头注释已标注替换点）。
- **Lane-CFR**：runner 落地后执行 shell-* / intent-* / remote-completed-not-acceptance 向量组（M5 出口证据）。
- **Lane-TSC（本车道下次会话）**：M5 gate 开后做真集成——建议提示词 `docs/prompts/lane-tsc.md` + `docs/prompts/milestone-m5.md`；工作分支 `lane/tsc`（从合并后 main 续建）。
- 第一个动作：`git fetch origin; git merge origin/main`，读 `apps/kernel-server` 的 AKP HTTP 路由定义，对齐 `HttpSseTransport`。

## 6. 快照

- PROGRESS 已更新：是（车道栏、REQ 覆盖注记、handoff 列表）；PARALLEL-LANES 所有权表已更新。
- 本次提交列表：见 PR（提交 1 = sdk-ts 八模块 + 测试；提交 2 = agent-shell 会话层 + 测试；提交 3 = 文档联动批；哈希以 git log 为准，本 handoff 写于推送前）。
