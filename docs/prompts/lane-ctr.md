# Lane-CTR 接续提示词：契约与生成（地基车道）

> 用法：将本文件全文粘贴到新 Cursor Agent 会话（工作目录 = 仓库根）。自包含，不依赖历史对话。公共前缀内联自 `docs/prompts/common-prefix.md`（修改需同步）。

---

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

**接入三步**：① 读 `AGENTS.md` → ② 读 `docs/plan/PROGRESS.md` → ③ 读最近 `docs/checkpoints/*-handoff.md`，对照 `docs/plan/PARALLEL-LANES.md` 确认车道边界后再动手。

**硬纪律（全程）**：① 确定性边界：概率组件只产 candidate/proposal，授权/CAS/状态迁移/硬预算/幂等/fencing/最终提交由确定性代码执行；② 规范优先级：机器 schema/registry/transition/vector 与 normative companion > 固定版本 RFC/Core/Profile > 白皮书 > 实现建议，冲突取不扩大权限/范围/风险/预算/完成声明的解释；③ 四类状态用语（规范已登记/实现已提供/测试已执行/Profile 已符合），implemented 仅指全部适用 MUST 有通过证据；④ 测试先行，schema-valid ≠ behavior-pass，完成证明只来自 authority 状态/Effect/Verification/Event；⑤ 规范表面冻结：v0.1 前不新增对象族/Profile/REQ 域，漂移先登记 `docs/traceability/findings-ledger.md` 再最小修正；⑥ P0 门禁：开放 P0 未闭合前对应子系统不得实现；⑦ 每个提交关联 REQ-ID/F/IMP/文档条目；⑧ 红线：禁读 `History/`、禁虚构规范资产、禁改写向量迎合实现。

**会话结束协议**（上下文吃紧时提前执行）：更新 PROGRESS → 按 `docs/checkpoints/TEMPLATE.md` 写 handoff → 分批提交。**DoD**：CI 两 OS 全绿 + 相关向量 pass/not-applicable 有据（未执行保持 not-run）+ 文档联动（docs-sync-contract）+ PROGRESS 更新 + handoff。

---

## 车道范围（Lane-CTR = 所有车道的地基，最先完成）

- **所有权**（PARALLEL-LANES §3）：`crates/cognitive-contracts`、`packages/contracts-ts`、`tests/golden/`、`specs/schemas/`（迁移期）、`specs/registry/`（契约变更）。
- **工作分支**：`lane/ctr`（不存在则从 main 创建；建议 `git worktree add ../agent-kernel-ctr lane/ctr`）。
- M1 任务（按序）：
  1. **F-003 治理对象双轨迁移**（开放 P0，阻断 M1 出口）：把 legacy metadata/strongRef 双轨 schema 单轨化到 `governed-object-header.schema.json`/`object-reference.schema.json` 合同（`docs/standards/governed-object-contract.md` 为准），**负例向量先行**（先写"双轨引用必须被拒"的向量/元校验，再迁移）；产出迁移说明（结构型变更 → 新 ADR + docs-sync-contract §2 全联动）。
  2. **D-001/D-006 `$id` 统一**：56 份 schema 统一 `$id` 策略（相对文件名风格，与 conformance/README"相对 `$ref` 从所在文件解析"一致），`profile-manifest`/`effect` 补 `$id`；同步 `tools/src/check-consistency.mjs` 的注册逻辑（去掉剥离 `$id` 的兼容层）。
  3. **codegen 管线**（ADR-0006）：schema → Rust（`cognitive-contracts`）与 TS（`contracts-ts`）生成器；生成物入库带头部（源 schema 路径 + digest + 生成器版本）；CI "regenerate and diff" 挂钩；**最小核心对象集优先**（IMP-08：common-defs、governed-object-header、object-reference、effect、intent 等 14 对象）。
  4. **注册式 bundle digest**：替换 M0 临时 digest 程序（`docs/standards/conformance-evidence.md` §6 → 按 `canonical-encoding-and-digest.md` §13 落 spec-set/schema-bundle manifest）。
  5. golden fixtures 扩展至标准 §14 全覆盖（digest projection、set manifest、unknown-critical extension 负例）。

## 禁止越界

- 不实现内核行为（KRN）、runner 执行逻辑（CFR）、任何 HTTP/存储；不动 `crates/cognitive-{domain,store,kernel,runtime,management,akp}` 除生成物注入点。
- 不新增 REQ/错误码/对象族（冻结）；schema 变更全部走 `docs-sync-contract` 语义型/结构型流程。
- 不改写向量迎合迁移；向量变更 = 契约变更的一部分，同批提交并说明。

## 相关规范路径

`docs/standards/governed-object-contract.md`、`canonical-encoding-and-digest.md`、`normative-source-and-versioning.md`、`conformance-evidence.md`；`docs/adr/0004、0005、0006`；`specs/schemas/`、`specs/registry/`；`docs/traceability/findings-ledger.md`（F-003、D-001/004/005/006）；`.cursor/rules/12-schemas-protocol.mdc`。

## 入口 gate 与验收

- 入口：M0 出口已通过（PROGRESS 确认）。
- 验收（并入 M1 出口评审）：F-003 关闭（无双轨引用，全 schema 过元校验 + `$ref` 解析）；codegen 再生成 diff 为空；golden 双语言 digest 逐字节一致保持；registry↔schema↔vector 双向无孤儿；findings-ledger 已更新（F-003 → closed、D-001/D-006 → closed）。

## 第一个动作

`git checkout -b lane/ctr`，然后读 `docs/traceability/findings-ledger.md` F-003/D-006 条目与 `docs/standards/governed-object-contract.md` §2-§3，列出双轨 schema 清单（grep `metadata`/`strongRef` vs `governed-object-header`）写入车道工作清单。
