# M7 Batch：CFR 行为向量与 self-check

读 `AGENTS.md` → `PROGRESS.md` → `M7-PLAN.md` WP-M7-MEM/DISC + WP-REGRESS → 最近 KRN/RUN handoff。

## 范围（Lane-CFR）

1. 接线并执行 M7 目标向量（MEM-* + DISC-DELTA/READ 等）；更新 pins 仅在实测后改 CI honesty gate。
2. 为每个新行为模式增加 anti-pattern；`--self-check` 必须翻 fail；地板 ≥36 且不得下降。
3. 回归：F-011 三负例；`AGENT-INSTALL-001` / `AGENT-BYPASS-002` / `AGENT-OOB-001`；F-017 claim-freeze digests。
4. 更新 not-run ledger（每条：入 M7 / 残留 / 继续 not-run + 理由）。

## 禁止

改写向量 expected；虚报 pass；扩 F-017；Profile implemented 虚报；混入 CON 实现。

## 出口

报告 + CI pins 实测；下一会话：`m7-exit-review.md`（或先 `m7-perf005-decision.md`）。
