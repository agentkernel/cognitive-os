<a id="cognitiveos-client-directory-index"></a>
# CognitiveOS PC + 手机客户端目录索引

> 类别：informative directory index
>
> canonical owner：Lane-CON；治理联动：Lane-DOC
>
> 查询基准日：2026-07-20

本文件是仓库内 **PC 客户端、手机 remote companion、共享 SDK/契约、平台设计与 Agent Hub 客户端文档目录**的唯一 canonical 索引。它只负责“目录在哪里、由谁维护、从哪里开始读、受什么 gate 阻断”，不复制或改写各产品文档、机器合同、计划与证据的正文。

事实边界：

- 工程状态与实测计数以 [PROGRESS](../plan/PROGRESS.md) 为准；
- owner 与车道 gate 以 [PARALLEL-LANES](../plan/PARALLEL-LANES.md) 为准；
- 文档联动遵循 [docs-sync-contract](../standards/docs-sync-contract.md)；
- Console 平台行为与实现 gate 以 [平台设计入口](../platforms/README.md) 为准；
- Agent Hub 行为与保证以 [Agent Hub canonical 根](../../apps/cognitiveos-console/docs/agent-hub/README.md) 为准。

当前基准是 273 项已登记要求、55 个错误码、56 份 schema、5 份迁移表和 76 份向量；76 份向量均为 `not-run`。这些计数不构成客户端实现或平台 PoC 证据。Console 与手机 implementation 均为 `not-implemented`，平台 evidence 为 `none`，相关 Profile 为 `not implemented`。

## 1. 索引口径

- 索引粒度是拥有独立 owner、角色、canonical 入口或 gate 的目录；纯源码子目录继承所属 package，但仍列出关键入口，避免把源码误认成独立产品。
- 表内反引号包围的值都是仓库中的真实路径。尚未分配的手机代码载体明确写作“无独立代码目录”，该文字不是路径。
- `README` 列的“缺（待所属车道补）”是显式缺口登记，不授权 Lane-CON 跨车道修改 Lane-TSC/Lane-CTR package。
- “目录存在”“产品方向 accepted”“包级单元测试通过”均不自动表示 REQ 实现已提供、平台测试已执行或 Profile 已符合。

## 2. PC 客户端

| 路径 | 平台 | 角色 | 当前状态 | owner | canonical 入口 | 上游 gate | README / 目录说明 |
|---|---|---|---|---|---|---|---|
| `apps/cognitiveos-console/` | PC | Console 产品设计占位；不是应用源码 | `planned`；implementation `not-implemented`；evidence `none`；Profile `not implemented` | Lane-CON（治理由 Lane-DOC 协作） | [Console README](../../apps/cognitiveos-console/README.md)；[PRODUCT-DESIGN](../../apps/cognitiveos-console/PRODUCT-DESIGN.md) | [Console 实现 gate](../platforms/README.md#console-实现-gate) | 有 |
| `apps/cognitiveos-console/docs/` | PC | Windows v1 产品设计、信息架构、旅程、设计系统、安全 UX、追踪与路线图 | `planned`；implementation `not-implemented`；platform tests `none` | Lane-CON | [Console README 文档地图](../../apps/cognitiveos-console/README.md)；[Windows v1 范围](../../apps/cognitiveos-console/docs/windows-v1-scope.md) | Console 依赖组 1/2/7、M5、真实平台 PoC、技术栈 ADR | 缺（由父级 README 索引） |
| `apps/agent-shell/` | PC | TypeScript Task Shell 客户端外壳；非 authority | `blocked`；仅有 M0 skeleton，M5 客户端功能 `not-implemented`；conformance `not-run` | Lane-TSC | [package.json](../../apps/agent-shell/package.json)；[源码入口](../../apps/agent-shell/src/index.ts) | [Lane-TSC 当前 gate](../plan/PARALLEL-LANES.md)；M5 集成 | 缺（待 Lane-TSC 补） |
| `apps/agent-shell/src/` | PC | Shell verbs 与 task-channel 绑定的源码/单元测试入口 | 继承 `apps/agent-shell/` 的 `blocked` 状态；不是独立客户端 | Lane-TSC | [index.ts](../../apps/agent-shell/src/index.ts) | 同 `apps/agent-shell/` | 缺（继承 package） |

Console 目录当前只有 Markdown 产品文档，并由 [pnpm workspace](../../pnpm-workspace.yaml) 排除；不得据此推断应用脚手架、运行时或发布包已经存在。`agent-shell` 的 skeleton 只登记客户端边界和计划 verbs，不提供完整 transport、CLI、持久化或 authority 能力。

## 3. 手机 companion

仓库当前**没有独立 iOS 或 Android 客户端代码目录，也没有已分配的未来代码路径**。手机载体状态固定为：

- 产品：`planned`；
- implementation：`not-implemented`；
- platform / PoC evidence：`none`；
- Profile：`not implemented`。

不得创建类似 `apps/ios`、`apps/android` 的虚构索引项。现有且可达的规划入口如下：

| 真实路径 | 平台 | 角色 | 当前状态 | owner | canonical 入口 | 上游 gate | README / 目录说明 |
|---|---|---|---|---|---|---|---|
| `docs/platforms/ios-product-design.md` | 手机（iPhone） | iPhone-only 产品设计 | `planned`；implementation `not-implemented`；evidence `none` | Lane-CON | [iPhone 产品设计](../platforms/ios-product-design.md) | [Console 实现 gate](../platforms/README.md#console-实现-gate) 与 iPhone Open PoC/GA gate | 由 `docs/platforms/README.md` 索引 |
| `docs/platforms/android-product-design.md` | 手机（Android phone） | Android phone 产品设计 | `planned`；implementation `not-implemented`；evidence `none` | Lane-CON | [Android 产品设计](../platforms/android-product-design.md) | [Console 实现 gate](../platforms/README.md#console-实现-gate) 与 Android Open PoC/GA gate | 由 `docs/platforms/README.md` 索引 |
| `apps/cognitiveos-console/docs/agent-hub/platforms/product-scope.md` | 手机 + PC | Agent Hub 平台范围与手机 companion 边界 | `planned`；implementation `not-implemented`；evidence `none` | Lane-CON | [Agent Hub 平台范围](../../apps/cognitiveos-console/docs/agent-hub/platforms/product-scope.md) | [Agent Hub 实现 gate](../../apps/cognitiveos-console/docs/agent-hub/GOVERNANCE.md#7-实现-gate不可跳过) | 缺（由 Agent Hub 根 README 索引） |

手机是 Takeover Host / CognitiveOS 节点的 **remote companion**。手机不承载 Agent runtime、CognitiveOS authority、CognitiveOS node 或完整 Vault；高后果动作的 PC-local 边界与 Relay/配对保证只引用 [平台范围](../../apps/cognitiveos-console/docs/agent-hub/platforms/product-scope.md) 和 [Relay/配对/迁移](../../apps/cognitiveos-console/docs/agent-hub/architecture/relay-pairing-and-migration.md)，本索引不新造保证。

## 4. 共享 SDK 与契约

| 路径 | 平台 | 角色 | 当前状态 | owner | canonical 入口 | 上游 gate | README / 目录说明 |
|---|---|---|---|---|---|---|---|
| `packages/sdk-ts/` | 共享 | PC/未来手机消费的 TypeScript SDK | `blocked`；仅有 M0 skeleton，M5 transport/watch/credential 功能 `not-implemented`；conformance `not-run` | Lane-TSC | [package.json](../../packages/sdk-ts/package.json)；[源码入口](../../packages/sdk-ts/src/index.ts) | [Lane-TSC gate](../plan/PARALLEL-LANES.md)；M5 集成 | 缺（待 Lane-TSC 补） |
| `packages/sdk-ts/src/` | 共享 | task/management 双通道边界与单元测试 | 继承 `packages/sdk-ts/` 的 `blocked` 状态；不是 authority | Lane-TSC | [index.ts](../../packages/sdk-ts/src/index.ts) | 同 `packages/sdk-ts/` | 缺（继承 package） |
| `packages/contracts-ts/` | 共享 | TypeScript 机器契约、canonical 编码、digest、projection 与生成绑定 | 实现已提供（包级合同能力）；不作 REQ 级实现声明；76 向量 `not-run`；Profile `not implemented` | Lane-CTR | [API barrel](../../packages/contracts-ts/src/index.ts)；[代码生成 ADR](../adr/0006-code-generation-policy.md) | Lane-CTR 契约流程；F-003 仍待 Lane-CFR runner 真实执行负例 | 缺（待 Lane-CTR 补） |
| `packages/contracts-ts/src/` | 共享 | 合同编码层、bundle/projection、golden 与 schema 合同测试 | 实现已提供（包级）；测试结果不替代 conformance runner | Lane-CTR | [index.ts](../../packages/contracts-ts/src/index.ts) | 同 `packages/contracts-ts/` | 缺（继承 package） |
| `packages/contracts-ts/src/generated/` | 共享 | 从 schema 生成的 TypeScript bindings；禁止手改 | 实现已提供（生成物）；conformance `not-run` | Lane-CTR | [generated barrel](../../packages/contracts-ts/src/generated/index.ts)；[代码生成 ADR](../adr/0006-code-generation-policy.md) | schema/codegen 同批一致性 gate | 缺（由 package/ADR 说明） |
| `packages/contracts-ts/src/dev/` | 共享 | golden fixture 开发工具；不是客户端 runtime | 实现已提供（开发工具）；不构成客户端功能或 Profile 证据 | Lane-CTR | [fixture generator](../../packages/contracts-ts/src/dev/generate-fixtures.ts)；[golden 说明](../../tests/golden/README.md) | Lane-CTR golden/codegen gate | 缺（由 package/golden 说明） |

依赖方向固定为 `agent-shell → sdk-ts → contracts-ts`。SDK/客户端只能消费机器合同，不能自行重定义 authority 状态、授权、完成或 canonical digest 规则。

## 5. 平台设计

| 路径 | 平台 | 角色 | 当前状态 | owner | canonical 入口 | 上游 gate | README / 目录说明 |
|---|---|---|---|---|---|---|---|
| `docs/platforms/` | PC + 手机 | 跨平台产品设计、parity matrix 与平台决策 | `planned`；implementation `not-implemented`；platform tests `none`；Profile `not implemented` | Lane-CON（治理由 Lane-DOC 协作） | [平台设计 README](../platforms/README.md) | [Console 实现 gate](../platforms/README.md#console-实现-gate) | 有 |

该目录的 canonical 分工：

- [macOS](../platforms/macos-product-design.md)、[Linux](../platforms/linux-product-design.md)、[iPhone](../platforms/ios-product-design.md)、[Android phone](../platforms/android-product-design.md)：各平台详细范围、事实、产品要求与真实 PoC gate；
- [桌面 parity](../platforms/desktop-parity-matrix.md) 与 [移动 parity](../platforms/mobile-parity-matrix.md)：复用、适配、替换、不提供和阻断关系；
- [桌面决策](../platforms/platform-decision-log.md) 与 [移动决策](../platforms/mobile-platform-decision-log.md)：平台产品决策的 canonical 记录；
- [Agent Hub parity](../platforms/agent-hub-platform-parity.md)：Direct Takeover 平台差异的补充矩阵，不替代 Agent Hub canonical 根。

## 6. Agent Hub

下表覆盖 Agent Hub canonical 树及其实现计划/接续提示词。产品/架构文档定义事实；source ledger 记录外部来源；template 只是格式；plan/prompt 只编排受阻断工作，四者不得互相冒充。

| 路径 | 平台 | 角色 | 当前状态 | owner | canonical 入口 | 上游 gate | README / 目录说明 |
|---|---|---|---|---|---|---|---|
| `apps/cognitiveos-console/docs/agent-hub/` | PC + 手机 | Agent Hub canonical 产品/架构/安全/研究根 | `planned`；implementation `not-implemented`；Open PoC `not-run`；Profile `not implemented` | Lane-CON + Lane-DOC（治理） | [Agent Hub README](../../apps/cognitiveos-console/docs/agent-hub/README.md)；[GOVERNANCE](../../apps/cognitiveos-console/docs/agent-hub/GOVERNANCE.md) | [Agent Hub gate](../../apps/cognitiveos-console/docs/agent-hub/GOVERNANCE.md#7-实现-gate不可跳过) | 有 |
| `apps/cognitiveos-console/docs/agent-hub/product/` | PC + 手机 | 两部署模式、产品定位、旅程/页面、状态与无障碍 | `planned`；implementation `not-implemented`；evidence `none` | Lane-CON | [产品设计](../../apps/cognitiveos-console/docs/agent-hub/product/product-design.md)；[部署模式与保证](../../apps/cognitiveos-console/docs/agent-hub/product/deployment-modes-and-guarantees.md) | Agent Hub gate | 缺（由根 README 索引） |
| `apps/cognitiveos-console/docs/agent-hub/architecture/` | PC + 手机 | Takeover、进程/终端、session/file、Relay/配对/迁移 | `planned`；implementation `not-implemented`；evidence `none` | Lane-CON | [Takeover 架构](../../apps/cognitiveos-console/docs/agent-hub/architecture/takeover-architecture.md)；[Relay/配对](../../apps/cognitiveos-console/docs/agent-hub/architecture/relay-pairing-and-migration.md) | Agent Hub gate | 缺（由根 README 索引） |
| `apps/cognitiveos-console/docs/agent-hub/security/` | PC + 手机 | 威胁、凭据、computer control、许可与条款 | `planned`；implementation `not-implemented`；evidence `none` | Lane-CON | [威胁模型](../../apps/cognitiveos-console/docs/agent-hub/security/threat-model.md)；[许可与条款](../../apps/cognitiveos-console/docs/agent-hub/security/licensing-and-terms.md) | Agent Hub gate + Paseo/AGPL 法务 gate | 缺（由根 README 索引） |
| `apps/cognitiveos-console/docs/agent-hub/collaboration/` | PC + 手机 | Lead+Workers 多 Agent 产品约束 | `planned`；implementation `not-implemented` | Lane-CON | [Lead + Workers](../../apps/cognitiveos-console/docs/agent-hub/collaboration/lead-workers.md) | Agent Hub gate | 缺（由根 README 索引） |
| `apps/cognitiveos-console/docs/agent-hub/platforms/` | PC + 手机 | 平台发布顺序与手机 remote-companion 边界 | `planned`；implementation `not-implemented`；evidence `none` | Lane-CON | [平台范围](../../apps/cognitiveos-console/docs/agent-hub/platforms/product-scope.md) | Agent Hub gate + 各平台真实 PoC | 缺（由根 README 索引） |
| `apps/cognitiveos-console/docs/agent-hub/decisions/` | 共享 | Agent Hub 产品决策 canonical 记录 | `planned`；不构成 implementation/evidence | Lane-CON | [decision log](../../apps/cognitiveos-console/docs/agent-hub/decisions/decision-log.md) | 先决策后改专题；仍受 Agent Hub gate | 缺（由根 README 索引） |
| `apps/cognitiveos-console/docs/agent-hub/traceability/` | 共享 | 产品要求三维状态与证据索引 | implementation `not-implemented`；evidence `none/not-run` | Lane-CON | [产品要求](../../apps/cognitiveos-console/docs/agent-hub/traceability/product-requirements.md)；[证据索引](../../apps/cognitiveos-console/docs/agent-hub/traceability/evidence-index.md) | 状态回写全局 PROGRESS；不得冒充 normative registry | 缺（由根 README 索引） |
| `apps/cognitiveos-console/docs/agent-hub/adapters/` | PC + 手机 | Adapter tier、能力矩阵与接口分层 | `planned`；implementation `not-implemented`；evidence `none` | Lane-CON | [Adapter README](../../apps/cognitiveos-console/docs/agent-hub/adapters/README.md) | 一手 provider 接口核验 + Agent Hub gate | 有 |
| `apps/cognitiveos-console/docs/agent-hub/adapters/tier1/` | PC + 手机 | 六个 Tier 1 Agent dossier | `blocked`；implementation `not-implemented`；evidence `none` | Lane-CON | [Adapter README](../../apps/cognitiveos-console/docs/agent-hub/adapters/README.md)；[能力矩阵](../../apps/cognitiveos-console/docs/agent-hub/adapters/capability-matrix.md) | 一手接口/版本/许可核验；Hermes/OpenClaw 仍缺一手接口证据 | 缺（由 adapters README 索引） |
| `apps/cognitiveos-console/docs/agent-hub/sources/` | 共享 | provider、许可、平台安全、Paseo/竞品来源 ledger | `planned`；来源记录不是实现或测试证据 | Lane-CON | [sources README](../../apps/cognitiveos-console/docs/agent-hub/sources/README.md) | 查询日/一手来源/许可复核 | 有 |
| `apps/cognitiveos-console/docs/agent-hub/templates/` | 共享 | dossier/source/threat/PoC/task 模板 | `planned`；模板不是当前事实、计划完成或证据 | Lane-CON | [Agent Hub 根 README](../../apps/cognitiveos-console/docs/agent-hub/README.md) | 使用时回链 canonical owner 与真实状态 | 缺（由根 README 索引） |
| `apps/cognitiveos-console/docs/agent-hub/planning/` | 共享 | Agent Hub 计划/DAG/提示词的薄入口 | `blocked`；implementation `not-implemented` | Lane-CON | [planning README](../../apps/cognitiveos-console/docs/agent-hub/planning/README.md) | Agent Hub gate | 有 |
| `docs/plan/agent-hub/` | 共享 | Master 下的里程碑、DAG、风险、证据与各车道计划 | 全部实现车道 `blocked` | Lane-CON（治理由 Lane-DOC 协作） | [plan README](../plan/agent-hub/README.md)；[Master plan](../plan/agent-hub-development-plan.md) | Agent Hub 六类 gate | 有 |
| `docs/prompts/agent-hub/` | 共享 | 12 个宏车道与 6 个 Adapter 自包含提示词 | `blocked`；不得在 gate 前启动编码或 mock | Lane-CON | [prompt README](../prompts/agent-hub/README.md) | 对应 plan/gate 解阻后方可执行 | 有 |

Agent Hub 只有 **Direct Takeover** 与 **CognitiveOS Governed** 两种部署模式；详细保证和接管 L1–L8 层级只由 [部署模式与保证](../../apps/cognitiveos-console/docs/agent-hub/product/deployment-modes-and-guarantees.md) 定义。本索引不复制能力矩阵。

## 7. 我该从哪里开始读

### 7.1 只想了解当前状态

1. 本索引；
2. [PROGRESS](../plan/PROGRESS.md)；
3. [PARALLEL-LANES](../plan/PARALLEL-LANES.md)。

### 7.2 PC Console 产品与平台

1. [Console README](../../apps/cognitiveos-console/README.md)；
2. [PRODUCT-DESIGN](../../apps/cognitiveos-console/PRODUCT-DESIGN.md)；
3. [Windows v1 范围](../../apps/cognitiveos-console/docs/windows-v1-scope.md)；
4. [平台设计 README](../platforms/README.md) 与目标平台设计/parity。

### 7.3 TypeScript Shell / SDK / 契约

1. [Lane-TSC 提示词](../prompts/lane-tsc.md) 与当前 gate；
2. [sdk-ts 入口](../../packages/sdk-ts/src/index.ts)；
3. [agent-shell 入口](../../apps/agent-shell/src/index.ts)；
4. [contracts-ts 入口](../../packages/contracts-ts/src/index.ts) 与 [代码生成 ADR](../adr/0006-code-generation-policy.md)。

### 7.4 手机 companion

1. [iPhone](../platforms/ios-product-design.md) 或 [Android](../platforms/android-product-design.md) 产品设计；
2. [移动 canonical 决策](../platforms/mobile-platform-decision-log.md)；
3. [移动 parity](../platforms/mobile-parity-matrix.md)；
4. [Agent Hub 平台范围](../../apps/cognitiveos-console/docs/agent-hub/platforms/product-scope.md)；
5. [PC/手机旅程](../../apps/cognitiveos-console/docs/agent-hub/product/journeys-and-screens.md)、[页面状态与无障碍](../../apps/cognitiveos-console/docs/agent-hub/product/states-content-and-accessibility.md)、[Relay/配对](../../apps/cognitiveos-console/docs/agent-hub/architecture/relay-pairing-and-migration.md)。

### 7.5 Agent Hub

按 [Agent Hub README](../../apps/cognitiveos-console/docs/agent-hub/README.md) 的顺序：README → GOVERNANCE → decision log → 两部署模式与保证 → 专题文档 → planning。Adapter dossier、source ledger、template 与 plan 不替代产品行为 canonical。

## 8. 明确不纳入索引主体

- `apps/kernel-server/`：CognitiveOS 服务端组合根，不是客户端；
- `apps/admin-cli/`：确定性管理 CLI，归 Lane-RUN，不是 PC Console、手机 companion 或 TS Task Shell；
- `crates/cognitive-*`：内核、运行时、管理面、AKP、合同或符合性实现，不是客户端目录；
- `tools/`、`tests/`、`conformance/`：工具、测试与向量资产，不是客户端实现。

这些目录可作为客户端上游依赖或证据入口被引用，但不得列成 PC/手机客户端。

## 9. 持续维护与手动 gate

持续维护由 [`.cursor/rules/16-client-directory-index.mdc`](../../.cursor/rules/16-client-directory-index.mdc) 执行，并入 [docs-sync-contract](../standards/docs-sync-contract.md)；任何目录、状态、owner、gate 或 canonical 入口变化必须同一 PR 更新本文件。

当前未修改 Lane-CFR 所有的 `tools/`，因此 `pnpm run check:consistency` 尚未对本索引建立专用结构校验。自动化后续任务登记为 `planned`：由 Lane-DOC 提出索引清单格式，Lane-CFR 经所有权确认后为 consistency checker 增加“真实路径、必填字段、唯一 canonical 与覆盖率”校验。在该工具任务交付前，每个相关 PR 必须手动：

1. 逐项确认表内所有反引号路径存在；“无独立手机代码目录”不得被替换为虚构路径；
2. 盘点 PC、手机、SDK/契约、`docs/platforms/` 与 Agent Hub 的实际目录，与本索引逐项对照；
3. 确认每项 platform、role、status、owner、canonical 入口、gate、README 状态均非空；
4. 确认只有本文件拥有“客户端目录索引”canonical 职责，其他入口只链接；
5. 验证相对链接和既有 anchor 可达，且未改变产品 ID、anchor 或 canonical 含义；
6. 运行 `pnpm run check:consistency` 与 `git diff --check`。

静态检查通过只证明目录/链接/追踪一致，不是客户端实现、平台 PoC、向量执行或 Profile 符合证据。
