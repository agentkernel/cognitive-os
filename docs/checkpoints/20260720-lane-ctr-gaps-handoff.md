# 20260720 Lane-CTR Handoff（契约缺口批）

## 1. 本次会话完成

处置 Lane-TSC 登记的 7 项契约缺口（权威清单：`20260720-lane-tsc-handoff.md` §4；分支 `lane/ctr`，基线 `31f9f4f` = PR #3 merge）。逐缺口终态：

| 缺口 | 终态 | 交付/理由 |
|---|---|---|
| ① AKP envelope 无机器 schema | **closed（D-013）** | `akp-request-envelope.schema.json` + `akp-result-envelope.schema.json`：companion §3 成员全量 + §10.1 管理信封可选成员（management_session_ref 出现 ⇒ 必带 actor_chain_digest+activity_context_ref）+ 条件（status=error ⇒ error；status=partial ⇒ continuation；payload/payload_ref 恰一）；原请求关联定名 `in_reply_to`，审计引用定名 `audit_ref`（与 sdk-ts 临时命名一致，改动面最小）；负例向量 ×3（schema-gate 已执行 pass）；codegen 双语言绑定 + 运行时 digest 常量 |
| ② errors.yaml 无 codegen 绑定 | **closed** | contracts-codegen 0.2.0 新输入类：`generated/error_registry.rs` / `generated/error-registry.ts`——55 码穷尽 enum（Rust）/字面量 union（TS）+ code/category/retryable/description 表（category 消费生成的 common-defs `ErrorCategory`，未知类别生成期 fail）+ fail-closed `parse`/`parseErrorCode` + `REGISTRY_DIGEST` 常量（spec-set/0.1 canonical JSON projection，与 spec-set manifest 每资产配方一致）；Rust 侧逐条对读 YAML 平价测试 + TS 侧表不变量测试 + regenerate-diff 门 |
| ③ shell 族 5 schema 不在 codegen 集 | **closed** | shell-action-proposal / shell-command-preview / shell-status-view / watch-subscription / user-intent-record 入 CORE_SET（"remaining families follow their consuming milestones"条款兑现，消费方 = sdk-ts）；生成 28 schema 模块 × 双语言；根类型名剥 `CognitiveOS ` 标题前缀（现有 19 模块无前缀根标题，零改名） |
| ④ shell.control 无 payload schema | **closed（D-014）** | `shell-control-request.schema.json`（schema_version/control=cancel/target_ref/reason；deadline+principal 由请求信封承载，§7 四要素闭合）；负例向量 ×1；结果侧十态区分不另造 schema——收口在结果信封 status + 已登记 lifecycle 码 + ShellStatusView（SHELL-CANCEL-SEMANTICS-005 机器期望即此口径），行为验收归 M5 |
| ⑤ 生成模块无 digest 运行时常量 | **closed** | 每 schema 模块导出 `SCHEMA_ID` + `SCHEMA_DIGEST`；`generated::SCHEMA_DIGESTS`（mod.rs）/ `SCHEMA_DIGESTS` as const（index.ts）聚合表 = 信封 `schema_digest` 钉扎表；无再生成平价测试双侧钉扎（对实况 schema 重derive 相等 + 计数 28 钉扎） |
| ⑥ AKP §8 流帧无 schema | **closed（D-015）** | `akp-stream-frame.schema.json`（stream_id/sequence/kind/payload_digest/final/cost；kind 固定 watch profile：snapshot ⇒ snapshot_version+payload、delta ⇒ payload、error ⇒ 已登记机器错误）；负例向量 ×1；event-audit-watch §5 登记指针。**注意**：流错误码从 sdk-ts 临时 `payload.code` 收口到 `error.code`（common-defs error 形状），watch.ts 换绑定时需同步 |
| ⑦ management 操作名未登记 | **deferred-to-v0.2（D-016）** | companion §10.1 只描述 session 生命周期消息、未命名 wire 操作族，§12 生态操作未绑定通道——登记操作名 = 新增未描述规范面，冻结纪律（IMP-01）下不做；且 management 面（Lane-RUN，M5）无实现，命名属推测性设计。反向门（ManagementChannelClient 拒发任务通道操作族）维持 fail closed；M5 以实现反馈驱动修正型登记 |

交付批次：

- **提交 1（`707b7da`）**：①④⑥ 四份 wire schema + 5 份 schema-gate 负例向量（AKP-ENVELOPE-NO-SCHEMA-PIN-001 / AKP-ENVELOPE-AMBIGUOUS-PAYLOAD-002 / AKP-RESULT-ERROR-WITHOUT-MACHINE-CODE-003 / AKP-STREAM-FRAME-UNSEQUENCED-004 / SHELL-CONTROL-UNREASONED-CANCEL-001）+ registry tests 映射（REQ-AKP-ENV-001/002、REQ-ERR-001、REQ-AKP-STR-001、REQ-AKP-CAN-001、REQ-AKP-SHELL-003、REQ-SHELL-CONTROL-001）+ matrix 再生成 + companion/标准机器资产指针 + 双语言合同层正/负例复验测试 + 钉扎计数同批红→绿（static_check.py 60/81、ci.yml 81/30/0/0/0/51 与 self-check ≥11、runner_execution.rs 81/30/51 与 must_flip 11、rules/12 计数）。
- **提交 2（`ef95993`）**：②③⑤ contracts-codegen 0.2.0 + 28 schema 模块 ×2 语言 + error registry ×2 + 平价测试（Rust `error_registry_matches_errors_yaml` / `schema_digest_constants_match_live_schemas`，TS twins）+ `final` 保留字转义 + serde_yaml 入 contracts（只读，与 conformance 同保留理由）+ ADR-0006 Delivery record（0.2.0 节）。
- **提交 3（本 handoff 批）**：findings-ledger（D-013~D-016）、PROGRESS、AGENTS 计数、Console PRODUCT-DESIGN 漂移标注、本 handoff。

## 2. 未完成 / 进行中

- **sdk-ts 换绑定（归 Lane-TSC，本车道不碰）**——就绪状态：
  - `views.ts`：5 个手工接口 + `SHELL_SCHEMA_DIGESTS` + `CancelControl` + `SHELL_CONTROL_PROVISIONAL_PIN` 可全量替换为 `@cognitiveos/contracts-ts` 生成绑定（`shellActionProposal`/`shellCommandPreview`/`shellStatusView`/`watchSubscription`/`userIntentRecord`/`shellControlRequest` 命名空间 + `SCHEMA_DIGESTS` 聚合或各模块 `SCHEMA_DIGEST`）。
  - `envelope.ts`：`RequestEnvelope`/`ResultEnvelope` 手工接口可替换为 `akpRequestEnvelope`/`akpResultEnvelope` 生成类型；注意生成结构体成员按字母序、`extensions` 条目类型名为 `Extension`。
  - `errors.ts`：手写 55 码钉扎表 + 对读 YAML 漂移门可替换为 `errorRegistry`（`ERROR_REGISTRY`/`REGISTERED_ERRORS`/`parseErrorCode`/`REGISTRY_DIGEST`）。
  - `watch.ts`：`WatchStreamFrame` 可替换为 `akpStreamFrame` 生成类型；**行为适配点**：流错误码读取从 `payload.code` 改为 `error.code`（D-015 收口）；schema 允许 error 帧不带 payload。
- ⑦ 的 v0.2 登记（M5 Lane-RUN management API 落地时，见 D-016）。
- matrix 的 AKP 域 impl/impl_tests 字段未回填（等各 REQ 证据口径，同 M1 批遗留口径）。

## 3. 测试与证据状态

- Rust：workspace 全绿（contracts 15 单元 + 9 schema-contract + **6 generated-types**（+digest 常量平价、error registry 平价）+ 6 projection + 2 golden + bundle/canonical 内嵌单元；conformance 3 单元 + **5 集成**含新钉扎 81/30/51 与 must_flip 11）；clippy -D warnings 绿；fmt --check 绿。
- TS：`pnpm -r build; pnpm -r test` 全绿 = **110 测试**（contracts-ts **33**（+SCHEMA_DIGESTS 平价 + error registry 表不变量）/ tools 2 / sdk-ts 63 / agent-shell 12），sdk-ts/agent-shell 零触碰、其对读 YAML/重derive digest 漂移门在新 schema 集下自然保持绿。
- 静态门：check-consistency OK（273 REQ / 55 码 / **60 schema / 81 向量**）；gen-matrix --check 无 drift；static_check.py ALL CHECKS PASSED（计数钉扎 60/81 同批红→绿）；validate-manifest OK。
- runner 实测：**81 枚举 / 30 pass / 0 fail / 51 not-run**（5 份新负例全部 schema-gate 真实执行，grounding+evidence 逐条；报告 sha256:ce71e2b25c9ef6446819d63cb119af151002396643f913374dd3a1201dd6e8c7）；self-check **11/11 corrupted 翻 fail**（报告 sha256:d0dd7bc60001b0b6a5f51c5beb15405e1621ce81400e6f7d626f062e86547fe0）；artifacts/evidence gitignore，digest 由 runner 打印。
- codegen regenerate-diff：本地空 diff（`cargo run -p cognitive-contracts --bin contracts-codegen; cargo fmt --all` 后 `git diff --exit-code` 生成目录干净）。
- golden：双语言 emit（含 live schema-bundle manifest digest，60 schema 面）本地 byte-identical；fixture 文件零改动（canonical 编码面未动）。
- 向量状态口径：新增 5 份为**静态合同层 schema-gate pass**，不构成行为覆盖与 Profile 声明；51 份行为向量维持 not-run 逐条理由。

## 4. 未决风险与漂移

- **D-013/D-014/D-015 已登记并闭合、D-016 deferred-to-v0.2**（findings-ledger 漂移节；处置详情见 §1 表）。
- 命名裁量点（接续者复核面）：结果信封成员 `in_reply_to`/`audit_ref`、管理信封成员沿用 management-action-proposal 的 `policy_version`/`revocation_epoch`/`expected_versions` 命名先例、流帧 digest 成员定名 `payload_digest`——均为 companion 散文未定名处的 schema 定名，已在 companion §3/§7/§8 注明映射。
- 结果信封 status 枚举固定 8 值（§5 基集"至少为"+ §10.1 verified/committed）：这是参考实现 profile 钉扎；若 v0.2 扩集属修正型 schema 变更。
- 钉扎计数纪律不变（IMP-17）：下批向量/schema 增补须同批调整 ci.yml、runner_execution.rs、static_check.py（本批已演示红→绿路径）。
- 本机 Windows 工具链坑复述（不影响 CI）：dlltool shim 前置 PATH；禁 `>` 重定向写 JSON/golden（本批 golden 比对仅在 TEMP 做对称比较，未写任何 fixture）。
- serde_yaml（archived）面扩大一处（contracts codegen bin + 平价测试，只读）：保留决策与 conformance 相同，YAML 面再扩大时统一换库。

## 5. 下一步入口

- **Lane-TSC**：按 §2 就绪状态换生成绑定（建议提示词 `docs/prompts/lane-tsc.md`；替换点标注在 sdk-ts 各模块头注释）；换绑后删除手工形状与临时 pin。
- **Lane-KRN**：M2 并行中；本批触碰 `Cargo.lock`（serde_yaml 边）与 PROGRESS/ledger，后合并方按 PARALLEL-LANES §2.3 处理冲突（预计仅这三处）。
- **Lane-CFR**：无 runner 逻辑改动（仅测试钉扎数）；下批行为执行能力扩展时注意 wire-schema 层现有 9 pass 口径。
- 第一个动作（任意接续车道）：`git fetch origin; git merge origin/main`，读 PROGRESS 车道表触碰通告。

## 6. 快照

- PROGRESS 已更新：是（计数 60/81、五态 30/51、漂移行 D-013~D-016、车道表触碰通告、handoff 列表）。
- 本次提交：`707b7da`（schema+向量+钉扎批）→ `ef95993`（codegen 0.2.0 批）→ 本 handoff 批（ledger + PROGRESS + AGENTS + PRODUCT-DESIGN 标注 + handoff；哈希见 git log）。基线 `31f9f4f`。
