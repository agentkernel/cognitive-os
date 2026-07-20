# CognitiveOS 客户端目录索引、说明与持续维护提示词

> 用法：将下方提示词全文粘贴到新的 Cursor Agent 会话，工作目录设为仓库根 `agent-kernel`。
>
> 目标：为 **PC 客户端 + 手机 companion** 的相关目录建立一份 canonical 索引与说明，提升可读性/可导航性，并新增一条持续维护规则，确保后续开发中索引与真实目录不漂移。
>
> 类别：prompt（informative + 规则）。本任务只产出**索引、说明与维护规则**，不实现客户端代码，也**不物理移动/重命名**任何 crate/package/代码目录（那属于所属车道的契约流程，不在本任务范围）。

---

你是 CognitiveOS 参考实现的文档与工程代理，工作目录为仓库根 `agent-kernel`。开工前先 `git status`：保护一切已有未提交改动——不覆盖、不回退、不混入你的提交；暂存一律逐路径 `git add`，禁止 `git add -A`。

## 接入三步（动手前必做）

1. 读 `AGENTS.md`（命令速查、目录地图、DoD、红线）。
2. 读 `docs/plan/PROGRESS.md`（当前里程碑/车道状态与开放 P0）。
3. 读最近一份 `docs/checkpoints/*-handoff.md`，并对照 `docs/plan/PARALLEL-LANES.md` 确认本任务归属 Lane-CON（文档由 Lane-DOC 协作）与所有权边界后再动手。

## 硬纪律（全程有效）

1. **确定性边界**：本任务不触碰授权/状态迁移/提交等确定性路径，只产出文档与规则。
2. **规范优先级**：机器 schema/registry/transition/vector 与 normative companion > 固定版本 RFC/Core/Profile > 白皮书 > 实现建议；冲突取不扩大权限/范围/风险/预算/完成声明的解释。
3. **四类状态用语**：规范已登记 / 实现已提供 / 测试已执行 / Profile 已符合，严格区分；`implemented` 仅指全部适用 MUST 有通过证据。索引中每个客户端目录的状态一律用四态之一或 `planned/blocked/not-implemented`，禁止把“目录存在”写成“实现已提供”。
4. **规范表面冻结**：v0.1 前不新增对象族/Profile/REQ 域；发现漂移先登记 `docs/traceability/findings-ledger.md` 再最小修正。
5. **Lane-CON 文档例外边界**：后端 gate 通过前只允许 informative 文档、索引、说明与相关治理规则；**不启动客户端实现、不搭脚手架、不写 mock、不移动或重命名代码目录**。
6. **可追溯提交**：每个提交/PR 关联文档条目或产品 ID；确无关联时写明原因。
7. **红线**：禁止读取/引用 `History/`；禁止虚构 REQ-ID/错误码/schema/向量；禁止改写向量；禁止把别人未提交的工作（如 `personal-blog/`、`.cursor/skills/`）混入本次提交。

## 目标产物

1. **一份 canonical 客户端目录索引**：覆盖 PC 客户端、手机 companion 与共享/SDK 层的全部相关目录，按平台与角色分组，逐条给出：路径、平台（PC / 手机 / 共享）、角色（客户端外壳 / 产品设计文档 / SDK / 契约 / 平台设计 / Agent Hub 等）、状态（四态或 planned/blocked/not-implemented）、owner 车道、canonical 入口文档、上游 gate。
2. **说明文档**：解释每个目录的用途、边界、事实来源与阅读顺序；给出“我该从哪里开始读”的导航。
3. **一条持续维护规则**：确保后续任何客户端目录的新增/更名/删除/状态变化都会同批更新索引，使索引长期不漂移。
4. **最小联动**：把新索引挂到既有文档地图（`docs/README.md`）与相关入口，保留既有 ID、anchor 与 canonical 含义，不整体重构。

## 任务步骤

### 第 1 步：盘点客户端相关目录（PC + 手机 + 共享）

用 `Glob`/`Grep`/`Read` 实测盘点，不臆造。至少覆盖并逐一核实以下候选（存在与否、真实状态都以仓库当前内容为准）：

- PC 客户端与产品设计：`apps/cognitiveos-console/`（`README.md`、`PRODUCT-DESIGN.md`、`docs/**`，含 `docs/agent-hub/**`）。
- 客户端外壳（TS）：`apps/agent-shell/`。
- 共享/SDK/契约层（客户端消费）：`packages/sdk-ts/`、`packages/contracts-ts/`。
- 跨平台产品设计：`docs/platforms/**`（macOS/Linux/iPhone/Android 产品设计、parity matrix、决策）。
- Agent Hub 客户端设计：`apps/cognitiveos-console/docs/agent-hub/**`（两部署模式、平台 parity、旅程与页面、无障碍）。
- 手机 companion：如仓库尚无独立移动客户端目录，索引中以 `planned` 记录其规划位置与入口（`docs/platforms/ios-product-design.md`、`android-product-design.md`、`docs/agent-hub/platforms/product-scope.md`），不虚构不存在的代码目录。
- 其他：`kernel-server`、`admin-cli` 等**非客户端**目录明确标注为“非客户端，不纳入索引主体”，避免误收。

对每个目录记录：路径、平台、角色、状态（四态/planned/blocked）、owner 车道（对照 `PARALLEL-LANES.md` 所有权表）、canonical 入口、上游 gate、是否已有自己的 README/说明。

### 第 2 步：生成 canonical 索引

- 选定唯一 canonical 位置（推荐新建 `docs/clients/README.md` 作为“PC + 手机客户端目录索引”单一入口；若你判断扩展某既有入口更合适，需说明理由并保证不产生第二个 canonical）。
- 索引主体是一张可导航的表，按 **PC 客户端 / 手机 companion / 共享·SDK·契约 / 平台设计 / Agent Hub** 分组，字段见“目标产物”第 1 条。
- 明确“单一事实来源”：索引只**引用**各目录既有 canonical 文档，不复制其正文；对已有的 `docs/README.md`、`docs/platforms/README.md`、`apps/cognitiveos-console/docs/agent-hub/README.md` 只做交叉引用，不重定义。
- 固定状态真相：客户端 implementation 未启动处一律 `not-implemented`；测试/Profile 未达处标 `none`/`not implemented`；计数从 `docs/plan/PROGRESS.md` 实测读取，禁止沿用旧数。

### 第 3 步：生成说明与阅读导航

- 在索引同文档或相邻说明文档中，给出每组目录的用途、边界与阅读顺序（“先读什么，再读什么”）。
- 对缺少自身 README 的客户端目录，补一份薄 README/说明（仅入口与作用域，不复制 canonical 正文），或在索引中标注“缺 README（待补）”。
- 明确 PC 与手机的关系：手机是远程 companion，不承载 runtime/authority；引用 Agent Hub 相关 canonical，不新造保证。

### 第 4 步：新增持续维护规则

- 新建一条 Cursor 规则（推荐 `.cursor/rules/` 下取一个未占用的编号，如 `13-client-directory-index.mdc`；先 `ls` 确认编号未冲突），或在 `.cursor/rules/02-workflow-docs-sync.mdc` 增补一节，二选一并说明理由。
- 规则至少规定：
  - **触发条件**：任何客户端相关目录（PC/手机/共享 SDK/契约/平台/Agent Hub）新增、更名、删除、状态变化，或其 canonical 入口文档变化时；
  - **同批义务**：必须在同一 PR 更新 canonical 客户端索引与相关说明，保持四态状态用语、保留既有 ID/anchor、维持“单一 canonical、其余引用”的结构；
  - **与既有契约的关系**：并入 `docs/standards/docs-sync-contract.md` 的联动义务，不与之冲突；
  - **完成前检查**：给出可执行校验——索引中每条路径真实存在、每个客户端目录都在索引中有条目（缺失即红灯）；能并入 `pnpm run check:consistency` 则并入，否则写明手动 gate 步骤；
  - **所有权**：Lane-CON（文档由 Lane-DOC 协作）；不跨车道修改他人 crate/package；接口/代码目录变更仍走所属车道契约流程。

### 第 5 步：最小联动与验证

- 在 `docs/README.md` 文档地图新增一行指向 canonical 客户端索引；如新增规则，同步在规则清单/相关入口登记。
- 运行并留证：`pnpm run check:consistency`（应通过：REQ/错误码/schema/向量计数 + markdown 链接 + 追溯）、`git diff --check`、以及对新文件的 `ReadLints`。
- 全仓校对新索引/说明中的相对链接与 anchor 是否可达；修正断链。
- 不把这些静态检查写成客户端“实现/测试证据”。

### 第 6 步：提交与交接

- 逐路径 `git add`，审计 `git diff --cached`，分批提交：①客户端索引与说明；②维护规则；③`docs/README.md` 等索引联动与 `PROGRESS` 更新。禁止 `git add -A`，禁止混入他人未提交改动。
- 更新 `docs/plan/PROGRESS.md`（只加本任务独立 hunk）。
- 会话结束按 `docs/checkpoints/TEMPLATE.md` 写 handoff：已完成/未完成、提交哈希、检查输出摘要、未决漂移、下一步入口。

## 约束与红线（务必遵守）

- 只产出索引、说明与维护规则；**不实现客户端、不搭脚手架、不写 mock、不物理移动/重命名代码目录**。
- 单一 canonical：不制造第二个客户端索引；既有 `docs/README.md`、`docs/platforms/README.md`、`apps/cognitiveos-console/docs/agent-hub/README.md` 的 canonical 含义与 anchor 保持不变。
- 四态状态用语严格区分；目录存在 ≠ 实现已提供；文档存在 ≠ 测试已执行。
- 保护他人未提交工作（`personal-blog/`、`.cursor/skills/` 等），逐路径暂存。
- 禁 `History/`；禁虚构规范资产；计数一律实测。

## 完成定义（DoD）

1. canonical 客户端索引存在，覆盖全部实测到的 PC/手机/共享客户端目录，字段完整、状态用语正确、单一事实来源。
2. 说明与阅读导航可用；缺 README 的目录已补薄入口或明确标注待补。
3. 维护规则已落地，含触发条件、同批义务、docs-sync 关联、完成前检查与所有权；`.cursor/rules/` 编号无冲突。
4. `docs/README.md` 等已最小联动；既有 ID/anchor 未被破坏。
5. `pnpm run check:consistency` 通过；`git diff --check` 无空白错误；新文件无 linter 错误；链接/anchor 全可达。
6. `PROGRESS` 已更新；handoff 已写入 `docs/checkpoints/`；分批提交完成且未混入他人改动。

## 会话结束协议（上下文接近极限时提前执行）

更新 `docs/plan/PROGRESS.md` → 按 `docs/checkpoints/TEMPLATE.md` 写 handoff（已完成/未完成、提交哈希、检查证据、未决漂移、下一步入口与本提示词路径）→ 逐路径分批提交。交接文档是跨会话唯一记忆载体，禁止依赖对话历史承载工程状态。
