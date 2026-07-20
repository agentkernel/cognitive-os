# ADR-0007: Clients Project Root and Client Documentation Migration

- Status: Accepted（用户 2026-07-20 批准文件级迁移方案）
- Date: 2026-07-20
- Decision owners: Lane-CON（治理联动 Lane-DOC）
- Classification: reference implementation / repository structure decision.
  本 ADR 只约束本仓库的目录与文档组织，不是 CognitiveOS 规范要求，不改变任何
  REQ、错误码、schema、transition、vector 或实现 gate。

## Context

客户端相关 informative 文档此前分散在四处：`docs/clients/`（目录索引）、
`apps/cognitiveos-console/docs/`（Windows 产品设计 + Agent Hub canonical 树）、
`docs/platforms/`（macOS/Linux/iPhone/Android 切片、parity、平台决策与实现 gate）、
`docs/plan/agent-hub*` 与 `docs/prompts/agent-hub/`（开发计划与提示词）。目录索引
维护成本高、canonical 漂移风险大，且 PC/手机/共享/Agent Hub 四域没有统一的项目根。

## Decision

1. 在仓库根建立 `clients/` 作为唯一客户端项目根：`pc/`、`mobile/`、`shared/`、
   `agent-hub/`、`governance/`、`plan/`、`prompts/` 七域，`clients/README.md`
   为唯一 canonical 项目地图（`CLIENTS-DEC-001`，见
   `clients/governance/decision-log.md`）。
2. 客户端 informative 文档按 `clients/governance/canonical-sources.md` 分批
   `git mv` 迁入；old→new 对照、anchor/ID 保全与批次哈希登记在
   `clients/MIGRATION-MAP.md`。
3. 旧路径保留 4 个兼容 stub（deprecated + successor，不复制正文）：
   `docs/clients/README.md`、`apps/cognitiveos-console/README.md`、
   `apps/cognitiveos-console/PRODUCT-DESIGN.md`（23 个既有 anchor 与漂移登记表
   原样保留）、`docs/platforms/README.md`（`implementation-gate` 等 anchor 保留）。
   Console 实现 gate 的 canonical 正文迁入 `clients/governance/readiness-gates.md`。
4. 不移动代码 package：`apps/agent-shell`、`packages/sdk-ts`（Lane-TSC）、
   `packages/contracts-ts`（Lane-CTR）原地不动；`apps/kernel-server`、
   `apps/admin-cli`、`crates/**`、`tools/**` 不属于客户端项目。
5. 手机代码载体从"无已分配路径"变为 `clients/mobile/{ios,android}/app/` 保留
   入口：入口已分配但无任何实现，禁止 manifest/源码/脚手架。
6. consistency checker（`tools/`，Lane-CFR 所有）当前不扫 `clients/`：该自动化
   缺口登记为 Lane-CFR `planned` 任务（纳入扫描根 + 结构校验 + 注入演练），
   不跨车道改 `tools/`。

## Alternatives considered

- **维持四处分散 + 只强化索引**：拒绝——索引每次变更都要跨四棵树同步，
  已发生计数/路径漂移；不能给未来实现提供单一落位。
- **迁入 `apps/` 下（如 `apps/clients/`）**：拒绝——`apps/` 是可构建组合根语义，
  文档项目根放入会诱发"目录存在=实现已提供"误读。
- **同时迁移代码 package**：拒绝——违反车道所有权与接口冻结纪律。

## Consequences

- 全部客户端文档单点导航；旧链接经 stub 与保留 anchor 可达。
- `docs/plan/`、`docs/prompts/`、`docs/platforms/` 变薄；Agent Hub 计划/提示词
  随产品文档同树。
- 在 Lane-CFR 工具落地前，`clients/**` 的链接/anchor/结构检查是**手动 gate**
  （`clients/README.md` §9），每个触碰 PR 必须执行。
- 实现 gate、四类状态、产品 ID 命名空间、向量与 REQ 计数全部不变；本 ADR 不
  构成任何实现授权。

## Compliance checks

`.cursor/rules/16-client-directory-index.mdc`（索引同批义务）与
`.cursor/rules/17-client-project-boundaries.mdc`（边界与 gate 前禁实现）生效；
`pnpm run check:consistency` + `git diff --check` + clients 手动链接检查随批执行。
