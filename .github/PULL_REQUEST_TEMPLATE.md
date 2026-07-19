# PR 说明

## 关联条目（至少一项；确无关联写明原因）

- REQ-ID：<!-- 如 REQ-EFF-002；无则写"无，原因：…" -->
- F / IMP / 漂移条目：<!-- 如 F-003 / IMP-02 / D-001 -->
- 计划条目：<!-- DEVELOPMENT-PLAN 里程碑/车道 -->

## 变更分类（docs-sync-contract §1）

- [ ] 修正型（不改语义）
- [ ] 语义型（行为/状态机/错误码/schema/验收口径）
- [ ] 结构型（重构/对象族/子系统；需 ADR + 迁移说明）

## 状态机影响

<!-- 触碰了哪些状态域（agent-execution/task/loop/effect/verification）？迁移表是否变更？无则写"无" -->

## 错误码

<!-- 新消费/新触发的 registry 错误码；确认无未登记码 -->

## Schema 兼容性

<!-- schema 变更？生成绑定已再生成（ADR-0006）？digest 影响？向量联动？ -->

## 威胁与安全负例

<!-- 本变更引入/触碰的权限路径；对应负例测试（tests/security/ 或向量）；缓存键治理维度核对 -->

## 文档联动清单（docs-sync-contract §2；无影响项写"无"）

- [ ] registry / schema / vector
- [ ] 白皮书 / companion 对齐或漂移登记
- [ ] `docs/traceability/matrix.yaml`（`gen-matrix`）与 findings-ledger
- [ ] `docs/plan/PROGRESS.md`
- [ ] 受影响产品文档（Console 漂移标注）

## 影响面扫描结果（docs-sync-contract §3）

<!-- 粘贴 rg 扫描键与结果摘要 -->

## 证据链接

<!-- CI run / artifacts/evidence digest / 本地测试输出摘要 -->
