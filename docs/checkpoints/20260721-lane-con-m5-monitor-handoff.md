# 20260721 Lane-CON M5 细监控 Handoff

## 1. 本次会话完成

- **工作树**：`D:\agent-kernel-clients`；协调分支自 `origin/main` 对齐后开 `work/clients-m5-monitor`。
- **合并**：PR #20 → `bd890bbd7cf571890f5f985ab8cb16a1613f3c7d`（Phase 0 status；gate 仍 NO-GO）。CI 双绿后 `gh pr merge 20 --squash`。
- **Gate**：`implementation-ready: no (blocked)`；本回合仅只读上游快照 + handoff/progress 最小同步；**未**改 readiness gate、**未**写 `clients/**` 实现。

### 上游 M5 细快照（只读；`git fetch origin`）

| 引用 | tip | 主题 |
|---|---|---|
| `origin/main` | `bd890bb` | PR #20 合入后；含 KRN M5 + RUN 批 1；**不含** RUN 批 2a |
| `origin/lane/run` | `952bd8c` | `docs(run): M5 batch 2a sync - matrix +AKP/MGMT-APPROVAL/EVT backfill, D-018 partial, handoff`（2026-07-21 08:29 +0800） |
| `origin/lane/krn` | `29d3541` | `docs(krn): PROGRESS post-merge sync…`；相对 `origin/main` **0** 提交领先（已在 main） |

`git log origin/main..origin/lane/run --oneline`（前 7，即全部领先）：

1. `952bd8c` docs(run): M5 batch 2a sync…
2. `d64ecab` Merge origin/main into lane/run before batch 2a docs close
3. `c49d39d` feat(runtime): assemble governed event envelopes
4. `8bdb37c` Merge remote-tracking branch origin/main into lane/run
5. `992c769` feat(server): expose AKP management HTTP and SSE watch
6. `b0b0916` feat(akp): validate envelopes and resume watches
7. `c292241` feat(management): enforce R1 structured approval lifecycle

开放上游 PR：**#21** — `M5 RUN batch 2a: session lifecycle, R1 approval gate, AKP HTTP/SSE, D-018 envelopes`（`lane/run` → main；监控时 windows 仍可能在跑，勿由 CON 代合）。

### crate 抽查（Rust 行数 ≈ `git show` 行计数）

| 路径 | `origin/main` | `origin/lane/run`（未合入） | 判断 |
|---|---|---|---|
| `crates/cognitive-akp` | 2 文件 / `lib.rs` ~22 | +`tests/m5_envelope_watch.rs`；~263 行 | **main 仍骨架**；run 已有 envelope/watch 实现 |
| `crates/cognitive-management` | ~2380 行（批 1 session/plane + fallback 测） | ~2957 行（+`approval.rs` + session 生命周期测） | **main 已非骨架**（批 1）；run 扩 R1 审批 |
| `crates/cognitive-runtime` | 2 文件 / ~18 行；`RUNTIME_ROLE` planned | +`event_envelope.rs`；~119 行；Harness 仍 planned | **main 仍骨架**；run 仅 D-018 组装器，**非**完整 Harness |
| `apps/kernel-server` | 2 文件 / `main.rs` ~34；打印 M0 skeleton、无监听 | ~211 行 + `tests/m5_http_sse.rs`；`--once --bind` TCP | **main 仍骨架**；run 有单次 HTTP/SSE 参考面，默认无 `--once` 仍打印 skeleton 文案 |

### 文档面

- `docs/plan/PROGRESS.md`（**main**）：M5 = **in-progress（RUN 批 1 + KRN kernel 侧批已交付）**；Shell/AKP/Harness 待 RUN 批 2+。
- `docs/plan/PROGRESS.md`（**lane/run tip，未合入**）：M5 = **in-progress（RUN 批 2a + KRN…）**；待批 **2b** Harness Loop + Shell 全动词。
- **无** `docs/checkpoints/*m5*milestone-review*`（main / lane/run 皆无）。
- 存在：`20260720-lane-run-m5-batch1-handoff.md`、`20260720-lane-krn-m5-handoff.md`；lane/run 另有 `20260721-lane-run-m5-batch2a-handoff.md`（**尚未进 main**）。

### 依赖组 1 / 2 / 7（相对 DEVELOPMENT-PLAN Console 节）

台账字面仍为 **未交付**（main 与 lane/run 的 DEVELOPMENT-PLAN 表均未改写）。进展判断：

| # | 组 | 相对进展 | 距「完整交付」仍缺 |
|---|---|---|---|
| 1 | Shell/Management/Watch + AKP | 批 1（main）：确定性 admin fallback。批 2a（**仅 lane/run / PR #21**）：R1 审批门、AKP envelope、HTTP JSON、SSE watch、`--once` server | **Shell** proposal/preview/attach/cancel；**Harness Loop**；长驻服务非 `--once`；批 2a **未合 main**；F-011 向量仍 not-run |
| 2 | AgentExecution/Task/Verification + AcceptanceDecision | M2/M4 内核生命周期 + KRN M5 intent/Loop **端口**在 main | M5 **运行时编排接线**、Acceptance 用户面/Shell 完成证明路径未闭环 |
| 7 | Audit/export、Snapshot/ack、reconciliation/recovery report | M4 recovery/tracer 分量；D-018 组装器在 run 为 **partial** | AuditRecord/export、StateSnapshot/ack、完整对账/恢复报告；D-018 闭合条件（CFR 向量 + 治理对象端口）未满足 |

**结论**：依赖组 1/2/7 **仍未完整交付**；批 2a 若合入也只是组 1（及组 4 R1）部分推进，**不构成** M5 出口或客户端解阻。

## 2. 未完成 / 进行中

- 上游：等 PR #21（批 2a）CI 双绿后由 RUN 车道合入；批 **2b** Harness + Shell；CFR 行为向量；**m5-milestone-review**。
- 客户端：implementation-ready 仍 blocked；Phase 0 本地 informative 已尽；外部 PoC 设备/账号、正式 ADR、POC-LIC 法务仍阻断。

## 3. 测试与证据状态

- 本车道：文档-only；以本 monitor PR CI 为准。
- 上游：PR #20 合入前双绿。PR #21 为 RUN 所有权，CON 只监控不代合。
- 客户端平台 evidence 仍 `none`；未改向量/Profile。

## 4. 未决风险与漂移

- **易误读**：`lane/run` 上 AKP/HTTP/SSE 已现，不等于 main 已解阻，更不等于 M5 出口。监控须同时看 **main vs lane/run** 与 **是否存在 m5-milestone-review**。
- `kernel-server` 默认路径文案仍含 “M0 skeleton”；勿仅凭字符串判断——看是否存在 `--once` HTTP 路径与测试（且仅在 lane/run）。
- 禁止改 gate→GO；禁止 `clients/**` 实现/mock；禁读 History；禁推 personal-blog。

## 5. 下一步入口

- **客户端下一动作**：**等待 M5 出口**（本地 Phase 0 文档已尽，进入等待）。可选 informative：PR #21 合入后再做一轮只读快照；无新文档空档。
- 建议提示词（上游）：`docs/prompts/milestone-m5.md` / `docs/prompts/lane-run.md`（批 2b）。
- 工作分支：`work/clients-m5-monitor`（经 PR 合入）。
- 第一个动作：合并本 PR 后监控 `gh pr view 21`；**仍禁**客户端实现。

## 6. 快照

- PROGRESS 已更新：是（Lane-CON / Console 最小行 + 本 handoff 置顶；**未**把批 2a 计为已合入 main）。
- `clients/plan/progress.md` 已更新：M5 细监控注记。
- 基线 merge：`bd890bb`（PR #20）。
- 关联：CLIENTS-DEC-001 / ADR-0007 / ADR-0008；无 REQ 实现变更；gate 仍 blocked。
