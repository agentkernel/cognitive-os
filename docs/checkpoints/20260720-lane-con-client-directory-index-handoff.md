# 20260720 Lane-CON 客户端目录索引 Handoff

> 本次为 Lane-CON informative 文档任务，治理由 Lane-DOC 协作；未实现、移动或重命名任何客户端代码。

## 1. 本次会话完成

- 建立唯一 canonical 客户端目录索引 [docs/clients/README.md](../clients/README.md)，覆盖 PC Console、`agent-shell`、`sdk-ts`、`contracts-ts`、跨平台设计、手机 companion 无独立代码载体的事实，以及 Agent Hub 全部有独立职责的子目录；提交 `8919489`。
- 索引逐项记录真实路径、平台、角色、状态、owner、canonical 入口、上游 gate 与 README 状态，并提供按 PC、手机、TS 客户端和 Agent Hub 划分的阅读路线。
- 新增 always-apply 规则 `.cursor/rules/16-client-directory-index.mdc`；编号 13/14/15 已占用，故使用未冲突的 16；规则并入 docs-sync-contract，含触发条件、同批义务、状态口径、所有权与手动 gate；提交 `2feba3f`。
- 最小联动 `docs/README.md`、根 `README.md` 与 `apps/cognitiveos-console/README.md`；保留所有既有产品 ID、Markdown anchor 与 canonical 职责。
- 盘点发现根 `README.md` 仍写 74 份向量，属于工作区已登记 D-012 的同类计数漂移；按 PROGRESS/实测最小修正为 76。D-012 台账 hunk 属会话开始前的并行未提交工作，本会话未覆盖或混入。
- 更新 `docs/plan/PROGRESS.md` 独立“客户端目录治理交付”hunk；本项无新增 REQ-ID、错误码、schema、vector、对象族或 Profile。

## 2. 未完成 / 进行中

- `apps/agent-shell/`、`packages/sdk-ts/`、`packages/contracts-ts/` 缺 README；因分别归 Lane-TSC/Lane-CTR，本任务只在索引登记缺口，未跨车道修改 package。
- 客户端索引专用自动校验保持 `planned`：后续由 Lane-DOC 提出清单格式，Lane-CFR 经所有权确认后扩展 `tools/src/check-consistency.mjs`；交付前执行索引 §9 手动 gate。
- Console、手机 companion 与 Agent Hub implementation 仍为 `not-implemented`；全部实现 gate 未因本次文档交付改变。

## 3. 测试与证据状态

- `pnpm run check:consistency`：通过，`273 requirements, 55 error codes, 56 schemas, 76 vectors, markdown links and traceability verified`。
- `git diff --check`：退出码 0；只有会话开始前文件的 CRLF→LF 提示，无空白错误。
- `ReadLints`：`docs/clients/README.md` 与 `.cursor/rules/16-client-directory-index.mdc` 无诊断。
- 目录盘点：`apps/` 下未发现 Swift/Kotlin/Gradle/Xcode/AndroidManifest/Podfile/Flutter marker；没有独立 iOS/Android 客户端代码目录。
- 链接与 anchor：consistency checker 全量通过；索引 literal path 已按 Glob/ReadFile 盘点。
- 上述均为静态文档/目录验证，不是客户端实现、平台 PoC、向量执行或 Profile 符合证据；76 份向量仍全部 `not-run`。

## 4. 未决风险与漂移

- 后续新增客户端目录但未同步索引的风险，当前由 always-apply 规则和手动 gate 缓解；自动红灯尚未实现。
- Agent Hub canonical 文档、计划与提示词是会话开始前已有并行未提交工作；本任务仅引用其真实路径，未把这些文件混入本任务提交。
- `personal-blog/`、`.cursor/skills/` 与其他既有未提交内容均保持隔离。
- D-012 同类的根 README 74→76 已最小修正；未发现需新增 REQ/错误码/schema/vector 的规范漂移。

## 5. 下一步入口

- 建议提示词：[docs/prompts/console-client-directory-index-and-maintenance.md](../prompts/console-client-directory-index-and-maintenance.md)。
- 工作分支：当前 `main` 工作区；继续操作前先保护既有 staged Agent Hub 文件与其他未提交变更。
- 第一个动作：读 [客户端目录索引](../clients/README.md) §9；若领取自动化任务，先经 Lane-CFR 所有权确认，再修改 `tools/`。

## 6. 快照

- PROGRESS 已更新：是，使用独立 hunk。
- 本次提交列表：
  - `8919489` — canonical 客户端目录索引；
  - `2feba3f` — 持续维护规则；
  - 入口联动、D-012 同类计数修正、PROGRESS 与本 handoff：本交接提交。
