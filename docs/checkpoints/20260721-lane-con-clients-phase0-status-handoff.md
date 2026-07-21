# 20260721 Lane-CON clients Phase 0 status Handoff

## 1. 本次会话完成

- **工作树**：`D:\agent-kernel-clients`；协调分支对齐 `origin/main` @ `44e44c8`（PR #19 squash 合入）。
- **合并**：PR #19 → `44e44c862dac66919574bd01c2887e9b824ceaa7`（PoC runbooks / tech-stack 草案 / design-system 缺口；gate 仍 NO-GO）。
- **Gate**：`implementation-ready: no (blocked)`；禁止 `clients/**` 实现代码/manifest/mock；本回合仅 docs/handoff/progress。
- **上游只读监控**（`git fetch origin --prune`；基线 `5d4a892..origin/main`）：
  - `origin/main` 相对 PR #18 tip 仅多 PR #19（`44e44c8`）。
  - **`origin/lane/run` 已存在**（tip 含 M5 批 1 management fallback + docs sync）。
  - `docs/plan/PROGRESS.md`：**M5 = in-progress**（RUN 批 1 + KRN kernel 侧批已交付）；**无** `*-m5-milestone-review*`。
  - 骨架抽查（`origin/main`）：`cognitive-akp` 2 文件 / `lib.rs` ~22 行；`cognitive-runtime` 2 文件 / ~18 行；`apps/kernel-server` 2 文件 / `main.rs` ~34 行——仍为骨架。`cognitive-management` 已非骨架（session/plane + `m5_fallback_verbs` 测试；RUN 批 1）。
- **结论**：M5 **已启动但未出口**（Shell/AKP/Harness/HTTP/`kernel-server` 运行时面仍待 RUN 批 2+）；客户端 **仍 blocked**（implementation-ready NO-GO）。
- **Phase 0 收口**：本地允许的文档准备已尽；未发现需另开的断链/漏指针小修。关联：CLIENTS-DEC-001 / ADR-0007 / ADR-0008；无 REQ 实现变更。

### Phase 0 允许工作清单

| 项 | 状态 | 备注 |
|---|---|---|
| AH-CTR-02 文档级接口核验 | **done** | PR #18 |
| POC-LIC 材料整理 | **partial** | 材料 PR #18；法务评估仍 not-run |
| 威胁 oracle + planned PoC | **done** | PR #18；oracle/evidence 全 not-run |
| 五平台 PoC runbook | **done** | PR #19；零执行 |
| 技术栈比较草案 | **done** | PR #19；非正式 ADR |
| 真实平台 PoC **执行** | **blocked** | 外部：Mac / 签名账号 / 真机 / APNs·FCM·Play 等；勿假跑 |
| 正式技术栈 ADR | **blocked** | 被 PoC 执行阻断 |
| M5 / 依赖组 1·2·7 | **blocked**（上游） | M5 in-progress 但出口未到；AKP/runtime/kernel-server 仍骨架；clients 实现仍禁 |

## 2. 未完成 / 进行中

- POC-LIC-001..003 法务评估执行与留证。
- 五平台 / Agent Hub Open PoC 真实执行（手册 ≠ pass）。
- M5 出口 + 依赖组 1/2/7 完整交付（RUN 批 2+：Management HTTP、AKP envelope、Harness Loop、Operation 执行器、`kernel-server`）。
- 正式技术栈 ADR（须 PoC 留证后）。

## 3. 测试与证据状态

- CI：以本 status PR checks 为准；PR #19 合入前双绿（ubuntu + windows）。
- 向量 / Profile：无本车道变更；客户端平台 evidence 仍 `none`。
- 未执行任何真实平台 PoC（无设备/账号则登记阻断，不假跑）。

## 4. 未决风险与漂移

- 上游 M5 已 in-progress，易被误读为「客户端可开工」——**否**：implementation-ready 仍 blocked，直至出口 + PoC + ADR + 法务等 gate。
- `origin/lane/run` 存在且领先于「纯骨架」叙事，但 AKP / runtime / kernel-server 仍骨架；监控时勿只看 management。
- 禁止触碰本地 `D:\agent-kernel` 未跟踪文件与 personal-blog；禁读 History。

## 5. 下一步入口

- **唯一聚焦问题（需所有者确认）**：是否已有另一窗口在跑 Lane-RUN M5 批 2；以及外部资源能否提供 **Apple 开发者账号 / Mac / Android 真机**（缺任一则对应平台 PoC 继续外部阻断）。
- 建议提示词：`docs/prompts/milestone-m5.md` / `docs/prompts/lane-run.md`（上游）；客户端侧待外部资源后按某一平台 `*-poc-runbook.md` 执行首个真实 PoC。
- 工作分支：`work/clients-phase0-status`（经 PR 合入）。
- 第一个动作：合并本 PR 后 **等待** M5 出口或所有者提供设备/账号——**仍禁** `clients/**` 实现代码。

## 6. 快照

- PROGRESS 已更新：是（Lane-CON / Console 最小行 + 本 handoff 置顶）。
- `clients/plan/progress.md` 已更新：Phase 0 文档准备收口注记。
- 基线 merge：`44e44c8`（PR #19）。
