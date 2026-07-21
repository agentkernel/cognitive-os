# M7 Batch：出口评审

读 `AGENTS.md` → `PROGRESS.md` → `M7-PLAN.md` §A4/§D/§E → 全部 M7 handoff → `20260721-v01-rereview.md`。

## 范围（Lane-DOC；CFR/KRN/RUN 供证）

1. 逐条核对验收矩阵（ENTRY…REVIEW）；安全负例不可豁免。
2. 继承 v0.1 explicit non-claim 列表；确认未静默升级。
3. 写 `docs/checkpoints/YYYYMMDD-m7-milestone-review.md`；结论三选一：
   - **GO**
   - **GO-with-explicit-non-claim**
   - **NO-GO**
4. 更新 PROGRESS、findings-ledger（F-019 等）、matrix（仅有真实 impl/evidence 时）、handoff。
5. CON gate：仅 informative 复评；implementation-ready 仍 blocked。

## 禁止

为凑 GO 改负例/扩声明/虚报 Profile；把残留当已闭合；Console 实现。

## 出口

评审文件 + PROGRESS + handoff 合入；开放 PR 经 CI 两 OS 绿。
