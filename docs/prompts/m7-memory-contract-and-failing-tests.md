# M7 Batch-0B：Memory 合同与失败测试先行

读 `AGENTS.md` → `PROGRESS.md` → `M7-PLAN.md` WP-M7-MEM → governed-memory companion → 最近 M7 handoff。

## 范围（Lane-KRN + Lane-CFR；单会话勿跨 RUN/CON）

1. 为下列向量建立/核对真实执行入口（先确认当前 not-run 或 fail）：
   - `MEM-ADMISSION-001`（`memory-admission-denied.json`）
   - `MEM-RYW-001`（`memory-read-your-write.json`）
   - `MEM-PROMOTION-002`（`memory-cross-scope-promotion.json`）
2. 审查 authority 边界：working-set / MemoryCandidate / MemoryAdmissionDecision / MemoryObject。
3. 保留 registered expected 与错误码；测试先行，schema-valid ≠ behavior-pass。

## 禁止

改写负例 expected；新增错误码/对象族/Profile；实现完整 lifecycle（留给 runtime slice）；InstallationStore；Console 实现。

## 出口

失败测试/not-run 分类落档；下一会话：`m7-memory-runtime-slice.md`。若发现合同缺口 → 停并回 WP-SCOPE（勿自行扩面）。
