# PR 说明

## 关联条目（至少一项；确无关联写明原因）

- Personal 任务 / Slice：<!-- 如 P13-T02/D01；owner-directed 文档交付写 DOC-<id> / GOV-<id> -->
- Lease ID：<!-- lease/personal/<task>/<purpose>（PARALLEL-LANES.md 活动表中的行） -->
- REQ / F / IMP / 漂移条目：<!-- 如 REQ-EFF-002 / F-003 / IMP-02 / D-001；无则写"无，原因：…" -->

## 变更分类（docs-sync-contract §1，勾选一项）

- [ ] implementation-only（实现/修正既有合同；声明 `normative surface unchanged`）
- [ ] corrective（typo/断链/漂移/计数；不改语义）
- [ ] product-semantic（产品版本/平台/release scope/任务验收/Gate 阈值；需 owner 决定 + Personal ADR）
- [ ] normative-semantic（public 行为/状态机/错误码/schema/vector/验收口径；走 Lane-CTR）
- [ ] structural（重构/对象族/子系统；需 ADR + 迁移说明）

## 验证与环境路由（PERSONAL-TEST-ENVIRONMENTS.md）

<!-- 逐项列出 pass / fail / not-run 与环境：required CI run 链接（Ubuntu / Windows MSVC）；
     exact-revision DEV-LINUX-NATIVE-01 结果与 revision；本地 Windows 只允许 fmt/TS/静态检查；
     not-run 必须写原因，不得推断为通过。 -->

- Required CI：
- Native Linux（exact revision）：
- 本地静态检查（`check:consistency` / `check:handbook` / `generate-handbook --check` / `check:rules` / `git diff --check`）：
- focused failure-first / negative tests：

## Handbook 联动（docs-sync-contract §2；docs-sync-gate 已在 commit/push 执行）

- [ ] 改动路径命中 `personal/handbook/_meta/source-map.json` 的页面已双语同步，指纹已刷新
- [ ] 生成页只经 `node tools/src/generate-handbook.mjs` 重生成
- [ ] 新增 tracked 文件已在 `source-coverage.json` 归类
- [ ] 确无文档影响：`DOCS_IMPACT_NONE="<具体理由>"`，理由：<!-- 写在这里 -->

## 状态机 / 错误码 / Schema（normative-semantic 与 structural 必填，其余写"无"）

<!-- 触碰的状态域与迁移表；新消费/新触发的 registry 错误码；schema 变更是否已再生成绑定（ADR-0006）、digest 影响、向量联动 -->

## 威胁与安全负例

<!-- 本变更引入/触碰的权限路径；对应负例测试（personal/tests/security/ 或向量）；secret 未进入 argv/配置/SQLite/日志/CI/evidence -->

## 旧文档联动清单（docs-sync-contract §2；无影响项写"无"）

- [ ] registry / schema / vector
- [ ] 白皮书 / companion 对齐或漂移登记
- [ ] `docs/traceability/matrix.yaml`（`gen-matrix`）与 findings-ledger
- [ ] `docs/plan/PROGRESS.md` Current snapshot / 正式计划 / trace
- [ ] 受影响 Personal 产品 / 架构文档

## 影响面扫描结果（docs-sync-contract §3）

<!-- 粘贴 rg 扫描键与结果摘要 -->

## 收口声明（完整任务 PR 转 ready 前）

- [ ] 全部 acceptance 已映射到实现 / 负例 / 证据（Operating Model「Deterministic task closure」）
- [ ] 分支只含本任务 lease 声明的路径；无未知工作树改动混入
- [ ] 合并后将关闭 lease、删除 task branch、本地 `main` fast-forward

## 证据链接

<!-- CI run / artifacts/evidence digest / 本地测试输出摘要；claim ceiling 与 non-claims -->
