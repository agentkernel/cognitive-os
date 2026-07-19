# M1 接续提示词：合同收敛与符合性 Runner

> 用法：将本文件全文粘贴到新 Cursor Agent 会话（工作目录 = 仓库根）。自包含，不依赖历史对话。M1 = Lane-CTR + Lane-CFR 并行（各自可再用 `docs/prompts/lane-ctr.md`/`lane-cfr.md` 开独立 Multitask 会话；本提示词适合单会话统筹推进）。公共前缀内联自 `docs/prompts/common-prefix.md`。

---

你是 CognitiveOS 参考实现的工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

**接入三步**：① 读 `AGENTS.md` → ② 读 `docs/plan/PROGRESS.md` → ③ 读最近 `docs/checkpoints/*-handoff.md`（M0 起点为 `20260720-m0-milestone-review.md`），对照 `docs/plan/PARALLEL-LANES.md` 确认边界。

**硬纪律（全程）**：① 确定性边界：概率组件只产 candidate/proposal，授权/CAS/状态迁移/硬预算/幂等/fencing/最终提交由确定性代码执行；② 规范优先级：机器 schema/registry/transition/vector 与 normative companion > 固定版本 RFC/Core/Profile > 白皮书 > 实现建议，冲突取不扩大权限/范围/风险/预算/完成声明的解释；③ 四类状态用语，implemented 仅指全部适用 MUST 有通过证据；④ 测试先行，schema-valid ≠ behavior-pass；⑤ 冻结：不新增对象族/Profile/REQ 域，漂移先登记 `docs/traceability/findings-ledger.md` 再最小修正；⑥ P0 门禁；⑦ 提交关联条目；⑧ 禁读 `History/`、禁虚构规范资产、禁改写向量迎合实现。

**会话结束协议**：更新 PROGRESS → 写 handoff（TEMPLATE.md）→ 分批提交。**DoD**：CI 两 OS 全绿 + 向量状态诚实 + 文档联动 + PROGRESS + handoff。

---

## 范围（DEVELOPMENT-PLAN M1 节为准）

1. **F-003 治理对象双轨迁移**（唯一开放 P0）：schema 单轨化到 GovernedObjectHeader/ObjectReference（`docs/standards/governed-object-contract.md`），负例先行，结构型变更全联动（新 ADR + 迁移说明）。
2. **漂移闭合**：D-001/D-006（`$id` 统一 + tools 兼容层移除）、D-004（层 7/8 slug 决策）。
3. **M1 复验项**：F-004/005/006/007/008/010/012/018 的负例向量逐条执行复验，findings-ledger 状态升级 verified-by-vector 或重开。
4. **Runner 执行能力**：分层执行 74 向量、五态输出（`docs/standards/conformance-evidence.md` §2）、机器 JSON + 人读报告、manifest 生成；**故意错误实现自检**（schema-valid 行为错 → 必须 fail）。
5. **codegen 管线**（ADR-0006）：最小核心对象集（IMP-08 的 14 对象）Rust/TS 双端生成物入库 + CI regenerate-diff。
6. **注册式 bundle digest**：替换 M0 临时 digest（`conformance-evidence.md` §6 → `canonical-encoding-and-digest.md` §13）。

## 入口 gate

M0 出口评审通过（PROGRESS 已确认）。无其他前置。

## 验收判据（出口评审逐条对照，未过项 = 阻断 M2）

1. F-003 关闭：全仓无 legacy metadata/strongRef 双轨引用；56+ schema 全过 2020-12 元校验与 `$ref` 解析。
2. runner 五态输出全部向量；**错误实现被判 fail** 的证据在案。
3. 未实现层保持 not-run，无一虚报。
4. M1 复验项负例全部执行并登记台账。
5. codegen 再生成 diff 为空（CI 钉住）。
6. registry↔schema↔vector 双向无孤儿保持绿；`pnpm run check:consistency` 全绿。
7. 出口产出 `docs/checkpoints/YYYYMMDD-m1-milestone-review.md`。

**M1 出口 = tracer bullet 入口 gate 开启（F-002~F-010 类全收敛）——这是独立审查 §1.2 的硬条件，不得绕过。**

## 工作分支

`lane/ctr` 与 `lane/cfr`（统筹会话可在两分支间切换或用两个 worktree；合并顺序 CTR 先）。

## 第一个动作

读 `docs/traceability/findings-ledger.md`（F-003、D-001/004/006、复验项清单）→ grep 双轨字段清点受影响 schema → 先提交"双轨引用必须被拒"的负例（向量/元校验），再开始迁移。
