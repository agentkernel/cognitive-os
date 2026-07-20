<a id="cognitiveos-client-directory-index"></a>
# CognitiveOS 客户端项目根与目录索引

> 类别：informative directory index / project map
>
> canonical owner：Lane-CON；治理联动：Lane-DOC
>
> 查询基准日：2026-07-20

本文件是仓库内 **PC 客户端、手机 remote companion、共享 SDK/契约消费层与 Agent Hub 客户端项目**的唯一 canonical 项目地图与目录索引（迁移决策 `CLIENTS-DEC-001`，见 [governance/decision-log.md](governance/decision-log.md)）。旧入口 `docs/clients/README.md` 已降为兼容 stub，只链接本文件。本文件只负责"目录在哪里、由谁维护、从哪里开始读、受什么 gate 阻断"，不复制或改写各产品文档、机器合同、计划与证据的正文。

事实边界：

- 工程状态与实测计数以 [PROGRESS](../docs/plan/PROGRESS.md) 为准；本地 [plan/progress.md](plan/progress.md) 只记录客户端文档/结构的局部准备状态，不承载全局工程真相；
- owner 与车道 gate 以 [PARALLEL-LANES](../docs/plan/PARALLEL-LANES.md) 为准；
- 文档联动遵循 [docs-sync-contract](../docs/standards/docs-sync-contract.md) 与 [GOVERNANCE.md](GOVERNANCE.md)；
- Console 实现 gate 的 canonical 定义点是 [governance/readiness-gates.md](governance/readiness-gates.md#console-实现-gate)（B6 已迁入；旧 `docs/platforms/README.md` 为兼容 stub）；
- Agent Hub 行为与保证以 [Agent Hub canonical 根](agent-hub/docs/README.md) 为准；
- old→new 迁移对照与兼容 stub 清单以 [MIGRATION-MAP.md](MIGRATION-MAP.md) 为准；readiness 双结论以 [READINESS.md](READINESS.md) 为准。

当前基准是 273 项已登记要求、55 个错误码、61 份 schema、5 份迁移表和 84 份向量；向量分布为 46 `pass` / 38 `not-run`（以全局 PROGRESS 实测为准）。这些结果不构成客户端实现或平台 PoC 证据。Console 与手机 implementation 均为 `not-implemented`，平台 evidence 为 `none`，相关 Profile 为 `not implemented`。

## 1. 索引口径

- 索引粒度是拥有独立 owner、角色、canonical 入口或 gate 的目录；纯源码子目录继承所属 package，但仍列出关键入口，避免把源码误认成独立产品。
- 表内反引号包围的值都是仓库中的真实路径。手机代码载体已分配 `clients/mobile/{ios,android}/app/` 保留入口，但**无任何实现**，不得出现 manifest、源码或构建脚手架。
- `README` 列的"缺（待所属车道补）"是显式缺口登记，不授权 Lane-CON 跨车道修改 Lane-TSC/Lane-CTR package。
- 四类状态用语严格区分：规范已登记 / 实现已提供 / 测试已执行 / Profile 已符合。"目录存在""产品方向 accepted""包级单元测试通过"均不自动表示 REQ 实现已提供、平台测试已执行或 Profile 已符合。
- 文档迁移已完成；canonical 入口必须始终指向当前真实文件，旧路径只允许保留 MIGRATION-MAP 登记的兼容 stub。

## 2. PC 客户端（`clients/pc/`）

| 路径 | 平台 | 角色 | 当前状态 | owner | canonical 入口 | 上游 gate | README / 目录说明 |
|---|---|---|---|---|---|---|---|
| `clients/pc/` | PC | PC 客户端项目根：Windows 首发，macOS/Linux parity 文档域 | `planned`；implementation `not-implemented`；evidence `none`；Profile `not implemented` | Lane-CON | [pc/README.md](pc/README.md) | [Console 实现 gate](governance/readiness-gates.md#console-实现-gate) | 有 |
| `clients/pc/app/` | PC | 保留的未来实现根；**无任何实现** | `blocked`；NO-GO 条件见 README | Lane-CON | [pc/app/README.md](pc/app/README.md) | 依赖组 1/2/7 + M5 + Windows 真实 PoC + 技术栈 ADR | 有 |
| `clients/pc/docs/` | PC | Windows v1 产品文档与 macOS/Linux 平台切片的 canonical 文档域 | `planned`；implementation `not-implemented`；platform tests `none` | Lane-CON | [PC README](pc/README.md)；[Windows v1 范围](pc/docs/platforms/windows/windows-v1-scope.md) | Console 依赖组 1/2/7、M5 出口、真实平台 PoC、技术栈 ADR | 各专题由 pc/README 索引 |
| `clients/pc/plan/` | PC | PC 里程碑与 roadmap 落位（B2 批迁入 roadmap） | `planned` | Lane-CON | [pc/plan/README.md](pc/plan/README.md) | Console 实现 gate | 有 |
| `apps/agent-shell/` | PC | TypeScript Task Shell 客户端外壳；非 authority；**不迁移**（Lane-TSC 所有） | M5 前骨架与包内测试已提供；真实 server transport 集成仍 `blocked` | Lane-TSC | [package.json](../apps/agent-shell/package.json)；[源码入口](../apps/agent-shell/src/index.ts) | [Lane-TSC 当前 gate](../docs/plan/PARALLEL-LANES.md)；M5 RUN 集成 | 缺（待 Lane-TSC 补） |

`apps/cognitiveos-console/` 迁移后只保留 README 与 PRODUCT-DESIGN 兼容 stub（既有 §17/§20.3 与旧章节 anchor 全部保留）。Console 目录当前只有 Markdown 兼容入口，并由 [pnpm workspace](../pnpm-workspace.yaml) 排除；不得据此推断应用脚手架、运行时或发布包已经存在。`agent-shell` 已有 M5 前客户端能力，但仍不提供真实 server transport 或 authority 能力。

## 3. 手机 companion（`clients/mobile/`）

手机是 Takeover Host / CognitiveOS 节点的 **remote companion**。手机不承载 Agent runtime、CognitiveOS authority、CognitiveOS node 或完整 Vault；高后果动作的 PC-local 边界与 Relay/配对保证只引用 [Agent Hub 平台范围](agent-hub/docs/platforms/product-scope.md) 和 [Relay/配对/迁移](agent-hub/docs/architecture/relay-pairing-and-migration.md)，本索引不新造保证。

手机载体状态固定为：产品 `planned`；implementation `not-implemented`；platform / PoC evidence `none`；Profile `not implemented`。`clients/mobile/{ios,android}/app/` 保留入口已分配但**无任何实现**，不得出现 manifest、源码或对"实现已启动"的暗示。

| 路径 | 平台 | 角色 | 当前状态 | owner | canonical 入口 | 上游 gate | README / 目录说明 |
|---|---|---|---|---|---|---|---|
| `clients/mobile/` | 手机 | remote companion 文档域根与共同边界 | `planned`；implementation `not-implemented`；evidence `none` | Lane-CON | [mobile/README.md](mobile/README.md) | [Console 实现 gate](governance/readiness-gates.md#console-实现-gate) + 各平台 Open PoC/GA gate | 有 |
| `clients/mobile/shared/` | 手机 | iOS+Android 共享决策与 parity（mobile-parity-matrix、mobile-platform-decision-log 已迁入） | `planned` | Lane-CON | [mobile/shared/README.md](mobile/shared/README.md)；[移动决策](mobile/shared/docs/mobile-platform-decision-log.md) | 同上 | 有 |
| `clients/mobile/ios/` | 手机（iPhone） | iPhone-only 产品文档/计划/保留实现入口（ios-product-design 已迁入） | `planned`；implementation `not-implemented`；evidence `none` | Lane-CON | [iPhone 产品设计](mobile/ios/docs/ios-product-design.md) | Console 实现 gate + iPhone Open PoC/GA gate | 有 |
| `clients/mobile/ios/app/` | 手机（iPhone） | 保留入口；**无任何实现** | `blocked` | Lane-CON | [ios/app/README.md](mobile/ios/app/README.md) | iPhone 真实 PoC + 技术栈 ADR + Console 实现 gate | 有 |
| `clients/mobile/android/` | 手机（Android phone） | Android phone 产品文档/计划/保留实现入口（android-product-design 已迁入） | `planned`；implementation `not-implemented`；evidence `none` | Lane-CON | [Android 产品设计](mobile/android/docs/android-product-design.md) | Console 实现 gate + Android Open PoC/GA gate | 有 |
| `clients/mobile/android/app/` | 手机（Android phone） | 保留入口；**无任何实现** | `blocked` | Lane-CON | [android/app/README.md](mobile/android/app/README.md) | Android 真实 PoC + 技术栈 ADR + Console 实现 gate | 有 |

## 4. 共享 SDK 与契约（`clients/shared/` + 不迁移的 package）

| 路径 | 平台 | 角色 | 当前状态 | owner | canonical 入口 | 上游 gate | README / 目录说明 |
|---|---|---|---|---|---|---|---|
| `clients/shared/` | 共享 | SDK/契约消费关系、身份/session、Relay、设计系统、安全隐私、遥测证据的共享说明域；不复制机器合同 | `planned`；文档域 | Lane-CON | [shared/README.md](shared/README.md) | 各消费方 gate | 有 |
| `packages/sdk-ts/` | 共享 | PC/未来手机消费的 TypeScript SDK；**不迁移**（Lane-TSC 所有） | 双通道、envelope、watch 等 M5 前能力与包内测试已提供；真实 HTTP/SSE 集成仍 `blocked` | Lane-TSC | [package.json](../packages/sdk-ts/package.json)；[源码入口](../packages/sdk-ts/src/index.ts) | [Lane-TSC gate](../docs/plan/PARALLEL-LANES.md)；M5 RUN 集成 | 缺（待 Lane-TSC 补） |
| `packages/contracts-ts/` | 共享 | TypeScript 机器契约、canonical 编码、digest、projection 与生成绑定；**不迁移**（Lane-CTR 所有） | 实现已提供（包级合同能力）；REQ 级状态以 PROGRESS/matrix 为准；Profile `not implemented` | Lane-CTR | [API barrel](../packages/contracts-ts/src/index.ts)；[代码生成 ADR](../docs/adr/0006-code-generation-policy.md) | Lane-CTR 契约流程；M5 消费已登记 F-011/AKP 合同 | 缺（待 Lane-CTR 补） |
| `packages/contracts-ts/src/generated/` | 共享 | 从 schema 生成的 TypeScript bindings；禁止手改 | 实现已提供（生成物）；conformance `not-run` | Lane-CTR | [generated barrel](../packages/contracts-ts/src/generated/index.ts)；[代码生成 ADR](../docs/adr/0006-code-generation-policy.md) | schema/codegen 同批一致性 gate | 缺（由 package/ADR 说明） |
| `packages/contracts-ts/src/dev/` | 共享 | golden fixture 开发工具；不是客户端 runtime | 实现已提供（开发工具）；不构成客户端功能或 Profile 证据 | Lane-CTR | [fixture generator](../packages/contracts-ts/src/dev/generate-fixtures.ts)；[golden 说明](../tests/golden/README.md) | Lane-CTR golden/codegen gate | 缺（由 package/golden 说明） |

依赖方向固定为 `agent-shell → sdk-ts → contracts-ts`。SDK/客户端只能消费机器合同，不能自行重定义 authority 状态、授权、完成或 canonical digest 规则。canonical encoding/digest 只在 `specs/**` 与 `packages/contracts-ts`，`clients/shared/docs/` 只放消费侧说明。

## 5. 治理、计划与提示词（`clients/governance|plan|prompts`）

| 路径 | 平台 | 角色 | 当前状态 | owner | canonical 入口 | 上游 gate | README / 目录说明 |
|---|---|---|---|---|---|---|---|
| `clients/GOVERNANCE.md` | 共享 | 客户端文档体系治理规则（canonical 唯一性、状态口径、deprecated 规则、ID 命名空间） | 生效 | Lane-CON + Lane-DOC | [GOVERNANCE.md](GOVERNANCE.md) | docs-sync 联动义务 | 单文件（自述） |
| `clients/READINESS.md` | 共享 | structure/implementation 双 readiness 判定 | structure-ready `yes`；implementation `blocked` | Lane-CON | [READINESS.md](READINESS.md) | 真实 gate 证据 | 单文件（自述） |
| `clients/MIGRATION-MAP.md` | 共享 | old→new 逐文件迁移对照、anchor/ID 保全、stub 与批次哈希 | 随批次维护 | Lane-CON | [MIGRATION-MAP.md](MIGRATION-MAP.md) | 每批验证绿灯后回填哈希 | 单文件（自述） |
| `clients/governance/` | 共享 | ownership/canonical/readiness-gates/decision-log/traceability/evidence 六件 | 生效（informative） | Lane-CON | [governance/README.md](governance/README.md) | docs-sync 联动义务 | 有 |
| `clients/plan/` | 共享 | 客户端全里程碑、依赖 DAG、风险与局部进度 | 全部里程碑 `blocked` | Lane-CON | [plan/README.md](plan/README.md) | 各实现 gate | 有 |
| `clients/prompts/` | 共享 | 客户端提示词索引（Agent Hub 提示词 B5 批迁入 `clients/agent-hub/prompts/`） | 索引 | Lane-CON | [prompts/README.md](prompts/README.md) | 对应 plan gate | 有 |

本地进度与全局 PROGRESS 的职责边界：`clients/plan/progress.md` 只记录客户端文档/结构状态，全局工程状态、里程碑与计数唯一真相是 [docs/plan/PROGRESS.md](../docs/plan/PROGRESS.md)。

## 6. Agent Hub（`clients/agent-hub/`）

Agent Hub canonical 树已迁入本项目根（docs/plan/prompts 三子树）。产品/架构文档定义事实；source ledger 记录外部来源；template 只是格式；plan/prompt 只编排受阻断工作，四者不得互相冒充。

| 路径 | 平台 | 角色 | 当前状态 | owner | canonical 入口 | 上游 gate | README / 目录说明 |
|---|---|---|---|---|---|---|---|
| `clients/agent-hub/` | PC + 手机 | Agent Hub 项目入口（docs/plan/prompts 三子树） | `planned`；implementation `not-implemented`；Open PoC `not-run`；Profile `not implemented` | Lane-CON + Lane-DOC（治理） | [agent-hub/README.md](agent-hub/README.md) | [Agent Hub gate](agent-hub/docs/GOVERNANCE.md#7-实现-gate不可跳过) | 有 |
| `clients/agent-hub/docs/` | PC + 手机 | Agent Hub canonical 产品/架构/安全/研究根 | `planned`；implementation `not-implemented`；Open PoC `not-run`；Profile `not implemented` | Lane-CON + Lane-DOC（治理） | [Agent Hub README](agent-hub/docs/README.md)；[GOVERNANCE](agent-hub/docs/GOVERNANCE.md) | Agent Hub gate + Paseo/AGPL 法务 gate + 一手 provider 接口核验 | 有 |
| `clients/agent-hub/plan/` 含 [Master plan](agent-hub/plan/agent-hub-development-plan.md) | 共享 | 里程碑、DAG、风险、证据与各车道计划 | 全部实现车道 `blocked` | Lane-CON | [plan README](agent-hub/plan/README.md) | Agent Hub 六类 gate | 有 |
| `clients/agent-hub/prompts/` | 共享 | 12 个宏车道与 6 个 Adapter 自包含提示词 | `blocked`；不得在 gate 前启动编码或 mock | Lane-CON | [prompt README](agent-hub/prompts/README.md) | 对应 plan/gate 解阻后方可执行 | 有 |

Agent Hub 只有 **Direct Takeover** 与 **CognitiveOS Governed** 两种部署模式；详细保证和接管 L1–L8 层级只由 [部署模式与保证](agent-hub/docs/product/deployment-modes-and-guarantees.md) 定义。本索引不复制能力矩阵。

## 7. 我该从哪里开始读

### 7.1 只想了解当前状态

1. 本索引与 [READINESS.md](READINESS.md)；
2. [PROGRESS](../docs/plan/PROGRESS.md)；
3. [PARALLEL-LANES](../docs/plan/PARALLEL-LANES.md)。

### 7.2 PC Console 产品与平台

1. [pc/README.md](pc/README.md)；
2. [Console README](../apps/cognitiveos-console/README.md) 与 [PRODUCT-DESIGN](../apps/cognitiveos-console/PRODUCT-DESIGN.md)；
3. [Windows v1 范围](pc/docs/platforms/windows/windows-v1-scope.md)；
4. [客户端实现 gate](governance/readiness-gates.md#console-实现-gate) 与 [pc/docs/platforms 平台切片](pc/README.md)。

### 7.3 TypeScript Shell / SDK / 契约

1. [shared/README.md](shared/README.md) 与 [Lane-TSC 提示词](../docs/prompts/lane-tsc.md)；
2. [sdk-ts 入口](../packages/sdk-ts/src/index.ts)；
3. [agent-shell 入口](../apps/agent-shell/src/index.ts)；
4. [contracts-ts 入口](../packages/contracts-ts/src/index.ts) 与 [代码生成 ADR](../docs/adr/0006-code-generation-policy.md)。

### 7.4 手机 companion

1. [mobile/README.md](mobile/README.md)；
2. [iPhone](mobile/ios/docs/ios-product-design.md) 或 [Android](mobile/android/docs/android-product-design.md) 产品设计；
3. [移动 canonical 决策](mobile/shared/docs/mobile-platform-decision-log.md) 与 [移动 parity](mobile/shared/docs/mobile-parity-matrix.md)；
4. [Agent Hub 平台范围](agent-hub/docs/platforms/product-scope.md) 与 [PC/手机旅程](agent-hub/docs/product/journeys-and-screens.md)。

### 7.5 Agent Hub

按 [Agent Hub README](agent-hub/docs/README.md) 的顺序：README → GOVERNANCE → decision log → 两部署模式与保证 → 专题文档 → planning。Adapter dossier、source ledger、template 与 plan 不替代产品行为 canonical。

## 8. 明确不纳入

- `apps/kernel-server/`：CognitiveOS 服务端组合根，不是客户端；
- `apps/admin-cli/`：确定性管理 CLI，归 Lane-RUN，不是 PC Console、手机 companion 或 TS Task Shell；
- `crates/cognitive-*`：内核、运行时、管理面、AKP、合同或符合性实现，不是客户端目录；
- `tools/`、`tests/`、`conformance/`：工具、测试与向量资产，不是客户端实现。

这些目录可作为客户端上游依赖或证据入口被引用，但不得列成 PC/手机客户端，也不迁入 `clients/`。

## 9. 持续维护与手动 gate

持续维护由 [`.cursor/rules/16-client-directory-index.mdc`](../.cursor/rules/16-client-directory-index.mdc) 执行，并入 [docs-sync-contract](../docs/standards/docs-sync-contract.md)；任何目录、状态、owner、gate 或 canonical 入口变化必须同一 PR 更新本文件。

自动化缺口登记：`tools/src/lib.mjs` 的 `SCAN_ROOTS` 与 `check-consistency.mjs` 的 `LIVING_SCOPES` 均不含 `clients/`，因此 `pnpm run check:consistency` **不扫描本目录**的链接、REQ 引用与结构。自动化后续任务登记为 `planned`：owner Lane-CFR（经所有权确认后修改 `tools/`），义务包含把 `clients/` 纳入扫描根、增加"真实路径、必填字段、唯一 canonical 与覆盖率"校验，并按 docs-sync-contract §5 完成一次注入演练。在该工具任务交付前，每个相关 PR 必须手动执行：

1. 逐项确认表内所有反引号路径存在；`app/` 保留入口不得被替换为虚构实现路径；
2. 盘点 `clients/` 各子树、`apps/agent-shell/`、`packages/`、`docs/platforms/`（stub 化前）与 Agent Hub 的实际目录，与本索引逐项对照；
3. 确认每项 platform、role、status、owner、canonical 入口、gate、README 状态均非空；
4. 确认只有本文件拥有"客户端项目地图/目录索引"canonical 职责，其他入口（含 `docs/clients/README.md` 兼容 stub）只链接；
5. 验证 `clients/**` 相对链接和既有 anchor 可达（checker 不扫本目录，需临时脚本或逐链核对），且未改变产品 ID、anchor 或 canonical 含义；
6. 运行 `pnpm run check:consistency` 与 `git diff --check`。

静态检查通过只证明目录/链接/追踪一致，不是客户端实现、平台 PoC、向量执行或 Profile 符合证据。
