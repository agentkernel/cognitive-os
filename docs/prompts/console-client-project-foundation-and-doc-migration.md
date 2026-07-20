# CognitiveOS 客户端项目根、文档迁移与开发就绪基线提示词

> 用法：将下方提示词全文粘贴到新的 Cursor Agent 窗口，工作目录设为仓库根 `agent-kernel`。
>
> 目标：在仓库根建立统一的 `clients/` 客户端项目空间，覆盖 PC、iPhone、Android、共享客户端能力与 Agent Hub；把现有分散的客户端产品文档按 canonical 职责迁移进来；补齐进入实质开发前必须的治理、架构、安全、测试、发布、计划、追踪与规则体系。
>
> 重要：本提示词不能用“目录/文档已创建”替代后端、平台 PoC、技术栈 ADR、接口合同或法务 gate。最终必须分别给出 `structure-ready` 与 `implementation-ready` 结论；真实 gate 未满足时只能得到 `structure-ready: yes / implementation-ready: no (blocked)`。

---

你是 CognitiveOS 参考实现的客户端平台主管、跨平台架构师、文档迁移负责人和工程治理代理，工作目录为仓库根 `agent-kernel`。

## 一、总目标

建立一个单一、可导航、可持续维护的客户端项目根：

```text
clients/
├─ README.md
├─ GOVERNANCE.md
├─ READINESS.md
├─ MIGRATION-MAP.md
├─ pc/
├─ mobile/
├─ shared/
├─ agent-hub/
├─ governance/
├─ plan/
└─ prompts/
```

该项目根必须：

1. 清楚区分 PC 客户端、手机 remote companion、共享 SDK/契约消费层和 Agent Hub；
2. 迁移现有相关产品文档，不复制出第二份 canonical；
3. 为尚未授权实现的代码载体只建立带 README 的保留入口，不创建 package manifest、源码、组件、mock、假 API 或构建脚手架；
4. 补齐进入开发前必须的文档系统、owner、gate、测试策略、证据口径和维护规则；
5. 最后执行正式 readiness review，只有全部实现 gate 有真实证据时才允许写 `implementation-ready: yes`。

## 二、开工前硬检查

### 2.1 保护工作区

第一条命令必须是：

```powershell
git status --short --branch
```

然后运行：

```powershell
git worktree list --porcelain
```

规则：

- 不覆盖、不回退、不 stash、不提交、不移动任何会话开始前已有改动；
- 不自动创建新的 `D:\agent-kernel-*` sibling worktree；
- 暂存只能逐路径执行，禁止 `git add -A`；
- 如果当前根目录有与本任务重叠的未提交文件，或其他 worktree 正在修改客户端文档、PROGRESS、PARALLEL-LANES、规则或 Agent Hub 文档，立即停止变更并向用户报告冲突路径；
- 不得为了“完成迁移”强制删除、reset、checkout 或覆盖其他代理工作。

### 2.2 按顺序阅读

1. `AGENTS.md`
2. `docs/plan/PROGRESS.md`
3. 最近一份 `docs/checkpoints/*-handoff.md`
4. `docs/plan/PARALLEL-LANES.md`
5. `docs/plan/DEVELOPMENT-PLAN.md`
6. `docs/standards/docs-sync-contract.md`
7. `docs/clients/README.md`
8. `.cursor/rules/11-typescript-clients.mdc`
9. `.cursor/rules/16-client-directory-index.mdc`

再盘点：

- `apps/cognitiveos-console/**`
- `apps/agent-shell/**`
- `packages/sdk-ts/**`
- `packages/contracts-ts/**`
- `docs/platforms/**`
- `apps/cognitiveos-console/docs/agent-hub/**`
- `docs/plan/agent-hub*`
- `docs/prompts/agent-hub/**`

禁止读取、引用、搜索或迁移 `History/`。

### 2.3 先核验真实工程状态

从 `PROGRESS.md` 与实际检查读取：

- 当前 M0–M6 里程碑状态；
- REQ、错误码、schema、transition、vector 实测计数；
- Lane-CTR、CFR、KRN、RUN、TSC、CON 的当前 gate；
- Console 依赖组 1/2/7；
- M5 出口评审；
- 各平台 Open PoC / GA evidence；
- 技术栈 ADR；
- Agent Hub provider 接口与 Paseo/AGPL 法务 gate。

不得沿用本提示词中的任何旧计数。

## 三、不可突破的边界

1. 概率组件只能产出 candidate/proposal；客户端不是 authority。
2. 手机是 remote companion，不承载 Agent runtime、CognitiveOS authority、CognitiveOS node 或完整 Vault。
3. 目录存在不等于实现已提供；README 存在不等于测试已执行；PoC 计划不等于 PoC pass。
4. 四类状态必须分开：
   - 规范已登记；
   - 实现已提供；
   - 测试已执行；
   - Profile 已符合。
5. 未启动使用 `planned`、`blocked`、`not-implemented`；未执行使用 `none`、`not-run`。
6. v0.1 前不得新增对象族、Profile 或 REQ 域。
7. 不虚构 REQ-ID、错误码、schema、transition 或 vector。
8. 不改写向量、删除负例或放宽 expected 迎合未来客户端。
9. `apps/agent-shell`、`packages/sdk-ts` 属 Lane-TSC；`packages/contracts-ts` 属 Lane-CTR。未经所属车道批准，不移动、不重命名、不改写这些 package。
10. `apps/kernel-server`、`apps/admin-cli`、`crates/**`、`tools/**` 不是客户端项目目录，不迁入 `clients/`。
11. 本任务默认只授权目录、文档、索引、规则和计划。实现 gate 未通过时禁止：
    - `package.json`、Cargo/Gradle/Xcode project、Podfile；
    - React/Tauri/Swift/Kotlin 源码；
    - UI 组件、路由、状态管理；
    - mock server、假 transport、假 authority；
    - CI 中伪造的平台测试或 Profile 通过状态。

## 四、工作方式

### 4.1 第一阶段只读审查

并行启动 4 个只读子代理：

1. **目录与 owner 盘点**：实测全部客户端相关目录、README、package、owner 和 gate；
2. **canonical/link/anchor 盘点**：生成旧路径、canonical 职责、显式 anchor、仓库内 inbound link 清单；
3. **开发前文档缺口**：按 PC、iOS、Android、shared、Agent Hub 识别产品/架构/安全/测试/发布缺口；
4. **readiness 与规则审计**：核对 docs-sync、Cursor rules、PROGRESS、handoff、平台 PoC 和实现 gate。

子代理不得编辑文件。主代理必须自行复核关键路径，不得把子代理多数意见当事实。

### 4.2 先提交迁移方案，再执行

只读阶段完成后，先向用户提交：

- 目标树；
- old → new 迁移表；
- 将保留的兼容入口；
- anchor/ID 保留方案；
- 预计修改文件；
- 与其他未提交工作的冲突；
- 当前 `structure-ready` / `implementation-ready` 预判。

这是结构型文档迁移。未经用户批准文件级迁移方案，不得执行 `git mv`。

## 五、目标目录结构

使用以下默认结构。若真实仓库已有批准 ADR 与此冲突，停止并向用户提出互斥选项，不得自行创造第二套结构。

```text
clients/
├─ README.md                         # 唯一客户端项目地图
├─ GOVERNANCE.md                     # canonical、状态、owner、同步、弃用
├─ READINESS.md                      # structure/implementation GO-NO-GO
├─ MIGRATION-MAP.md                  # old→new、anchor、兼容入口、提交
│
├─ pc/
│  ├─ README.md                      # Windows 首发，macOS/Linux parity
│  ├─ app/
│  │  └─ README.md                   # 保留的未来实现根；无 gate 不建脚手架
│  ├─ docs/
│  │  ├─ product/
│  │  ├─ architecture/
│  │  ├─ platforms/
│  │  │  ├─ windows/
│  │  │  ├─ macos/
│  │  │  └─ linux/
│  │  ├─ ux/
│  │  ├─ security/
│  │  ├─ accessibility/
│  │  ├─ quality/
│  │  └─ release/
│  └─ plan/
│
├─ mobile/
│  ├─ README.md                      # remote companion 共同边界
│  ├─ shared/
│  │  ├─ README.md
│  │  └─ docs/
│  ├─ ios/
│  │  ├─ README.md
│  │  ├─ app/
│  │  │  └─ README.md
│  │  ├─ docs/
│  │  └─ plan/
│  └─ android/
│     ├─ README.md
│     ├─ app/
│     │  └─ README.md
│     ├─ docs/
│     └─ plan/
│
├─ shared/
│  ├─ README.md                      # SDK/契约消费关系，不复制机器合同
│  ├─ docs/
│  │  ├─ contracts-sdk/
│  │  ├─ identity-session/
│  │  ├─ relay-pairing/
│  │  ├─ design-system/
│  │  ├─ security-privacy/
│  │  └─ telemetry-evidence/
│  └─ plan/
│
├─ agent-hub/
│  ├─ README.md
│  ├─ docs/
│  ├─ plan/
│  └─ prompts/
│
├─ governance/
│  ├─ README.md
│  ├─ ownership.md
│  ├─ canonical-sources.md
│  ├─ readiness-gates.md
│  ├─ decision-log.md
│  ├─ traceability.md
│  └─ evidence-index.md
│
├─ plan/
│  ├─ README.md
│  ├─ milestones.md
│  ├─ dependency-dag.md
│  ├─ risk-register.md
│  └─ progress.md
│
└─ prompts/
   └─ README.md
```

约束：

- Git 不跟踪空目录；每个保留目录必须有薄 README，说明用途、边界、owner、状态、canonical 入口和 gate；
- 不使用 `.gitkeep` 冒充项目已建立；
- `app/README.md` 只能声明未来实现位置和 NO-GO 条件，不得含技术栈已批准或实现已启动的暗示；
- 不创建与现有 `packages/`、`apps/agent-shell/` 平行的 SDK/契约代码副本。

## 六、文档迁移原则与默认映射

### 6.1 单一 canonical

新结构完成后：

- `clients/README.md` 成为客户端项目和目录的唯一 canonical 地图；
- `docs/clients/README.md` 降为兼容入口，只链接 `clients/README.md`，不保留并行正文；
- 具体产品、平台、Agent Hub 事实仍各有且只有一个 canonical 文件；
- 机器合同仍只在 `specs/**`、`conformance/**` 与 normative companion 中；
- `PROGRESS.md` 仍是全局工程状态真相，`clients/plan/progress.md` 只能记录客户端局部准备状态。

### 6.2 默认迁移范围

在 migration map 中逐文件确认后，默认采用：

1. `apps/cognitiveos-console/docs/*` → `clients/pc/docs/**`；
2. `docs/platforms/macos-product-design.md`、`linux-product-design.md`、桌面 parity/decision → `clients/pc/docs/platforms/**`；
3. `docs/platforms/ios-product-design.md`、`android-product-design.md`、移动 parity/decision → `clients/mobile/**/docs` 与 `clients/mobile/shared/docs`；
4. `apps/cognitiveos-console/docs/agent-hub/**` → `clients/agent-hub/docs/**`；
5. `docs/plan/agent-hub*` → `clients/agent-hub/plan/**`；
6. `docs/prompts/agent-hub/**` → `clients/agent-hub/prompts/**`；
7. 共享 SDK、双通道、Relay、身份和设计系统的说明 → `clients/shared/docs/**`，但只移动 informative 文档，不移动机器合同或 package；
8. `apps/cognitiveos-console/README.md`、`PRODUCT-DESIGN.md`、`docs/platforms/README.md` 保留为薄兼容入口；既有依赖的 §17、§20.3 和显式 anchor 必须可达。

不得机械移动。若一个文件同时拥有 PC、手机、shared 或 Agent Hub canonical 职责，先在 `canonical-sources.md` 决定唯一 owner，再移动或拆成“canonical 正文 + 薄引用”，不得复制正文。

### 6.3 ID、anchor 与链接

迁移前生成：

- 产品 ID 清单；
- 显式 `<a id>` 清单；
- 仓库内被引用的 Markdown heading anchor；
- 全仓 inbound link 清单；
- old path → new path → compatibility stub 清单。

迁移时：

- 优先使用 `git mv` 保留历史；
- 产品 ID 不重编号、不重用；
- 旧路径保留薄兼容文件时，必须写 deprecated/successor，不复制正文；
- 对仓库内使用的旧 anchor，在兼容入口保留 anchor alias 或全部更新到新位置；
- 更新所有相对链接；
- 禁止产生循环入口或两个都自称 canonical 的 README。

## 七、必须补齐的开发前文档系统

先复用、迁移和瘦化已有文档；只有确有缺口才新建。每份文档必须声明类别、owner、状态、事实来源和不代表什么。

### 7.1 全局治理

必须具备：

- 客户端项目地图；
- canonical source/owner 矩阵；
- old→new migration map；
- 状态用语与证据口径；
- decision log 与 deprecated/superseded 规则；
- docs-sync 和同 PR 更新义务；
- 跨车道变更流程；
- readiness gate 与 GO/NO-GO 模板；
- 本地进度与全局 PROGRESS 的职责边界。

### 7.2 每个平台项目

PC、iOS、Android 至少分别具备：

- product brief、persona/JTBD、范围与非目标；
- 支持 OS/设备/架构/分发矩阵；
- 信息架构、关键旅程、页面/状态模型；
- 客户端非 authority 与 trust boundary；
- 网络、身份、session、credential、storage/cache 边界；
- 生命周期、离线、断线、恢复、升级与 rollback；
- security/privacy threat model；
- accessibility 验收矩阵；
- test strategy、真实平台 PoC 和 evidence 入口；
- release/GA gate、支持窗口与 incident/kill-switch 策略；
- 技术栈 ADR 状态；
- upstream API/contract dependency ledger；
- milestone、risk register 和开发任务入口。

### 7.3 共享客户端能力

必须具备：

- `agent-shell → sdk-ts → contracts-ts` 依赖说明；
- task/management 通道隔离；
- snapshot/watch/cursor/reconnect 语义；
- canonical encoding/digest 只消费 `contracts-ts`；
- identity/session/device/credential 边界；
- Relay/E2EE/pairing/revoke 边界；
- shared design tokens 与平台原生差异；
- telemetry、redaction、retention 与 evidence 分类；
- shared test pyramid 与跨平台 contract tests；
- 不移动 Lane-TSC/Lane-CTR package 的明确说明。

### 7.4 Agent Hub

迁移并保持：

- Direct Takeover / CognitiveOS Governed 两模式；
- L1–L8 接管层级；
- Host/Control/Ledger；
- Process/Terminal；
- Session/File；
- Credential/Workspace/Verifier；
- Adapter 能力矩阵与一手接口 ledger；
- Relay/Pairing；
- PC/手机旅程、状态与无障碍；
- threat/licensing/Paseo/AGPL gate；
- AH-M0–M6 dependency DAG 与 readiness。

## 八、规则系统

先检查 `.cursor/rules/` 编号与职责，不得重复已有规则。

### 8.1 更新既有规则

更新 `.cursor/rules/16-client-directory-index.mdc`：

- canonical 索引路径改为 `clients/README.md`；
- `docs/clients/README.md` 明确为兼容入口；
- 触发条件覆盖 `clients/**`；
- 新增/更名/删除、状态、owner、gate、canonical 变化必须同 PR 更新；
- 路径、链接、anchor、README 和无虚构手机代码路径检查继续保留。

### 8.2 新增跨平台客户端边界规则

若编号未冲突，新建 `.cursor/rules/17-client-project-boundaries.mdc`，控制在 50 行左右，至少包含：

- globs 覆盖 `clients/**` 及仍在旧位置的 `apps/agent-shell/**`、`packages/sdk-ts/**`、`packages/contracts-ts/**`；
- PC/手机/SDK 都不是 authority；
- 手机 remote companion 边界；
- 四类状态与 gate；
- 未过 gate 禁脚手架/mock/假证据；
- 接口/代码目录移动须经所属车道；
- UI 实现时必须按需加载 frontend、responsive、visual、WCAG、testing skills；
- 完成前运行对应 build/test、consistency、link/anchor 和平台 gate。

不要复制 `.cursor/rules/11-typescript-clients.mdc`；17 只补跨语言、跨平台和项目根边界。

### 8.3 治理入口联动

最小更新：

- 根 `README.md`
- `AGENTS.md` 目录地图
- `docs/README.md`
- `docs/plan/PARALLEL-LANES.md` 所有权表
- `docs/plan/DEVELOPMENT-PLAN.md` Console 依赖入口
- `docs/plan/PROGRESS.md`
- `clients/GOVERNANCE.md`

不得整体重写这些文件。

## 九、开发就绪判定

在 `clients/READINESS.md` 分开记录：

### 9.1 Structure readiness

只有全部满足才为 `yes`：

- 目标目录和 README 完整；
- old→new migration map 完整；
- 无重复 canonical；
- 产品 ID、anchor、相对链接可达；
- owner 与 gate 非空；
- 必要文档系统齐全；
- rules 已生效；
- docs-sync/PROGRESS/handoff 已联动；
- consistency 与 whitespace 检查通过。

### 9.2 Implementation readiness

只有真实证据同时满足才为 `yes`：

1. Console 后端依赖组 1、2、7 已交付；
2. M5 出口评审通过；
3. 目标平台真实 Open PoC / GA gate 通过并留证；
4. PC/iOS/Android 技术栈 ADR 已批准；
5. 适用 machine contract 已登记且接口冻结；
6. 必需实现能力与 executed evidence 达到声明门槛；
7. Agent Hub 还须 provider 一手接口核验与 Paseo/AGPL 法务 gate；
8. 无开放的适用 P0 blocker；
9. owner、分支、CI、测试矩阵、发布与 rollback 责任已确认。

任何一项不满足：

```text
structure-ready: yes|no
implementation-ready: no
status: blocked
blocked-by:
  - ...
next-unblock:
  - ...
```

不得为了满足任务目标把 `blocked` 改写成 GO。

## 十、实施批次

批准迁移方案后，按以下批次执行；每批独立审计和提交：

1. `clients/` 根、GOVERNANCE、MIGRATION-MAP、READINESS 与薄 README；
2. PC 产品/平台文档迁移；
3. mobile shared/iOS/Android 文档迁移；
4. shared 客户端说明迁移；
5. Agent Hub docs/plan/prompts 迁移；
6. 旧路径兼容入口、全仓链接/anchor 修复；
7. Cursor rules 与治理入口联动；
8. readiness review、PROGRESS 与 handoff。

每批：

- 使用 `git mv` 或显式 `git add <path>`；
- 审计 `git diff --cached`；
- 禁止 `git add -A`；
- 提交信息关联文档条目、产品 ID、F/IMP 或“无 REQ 影响”的明确原因；
- 不混入其他 worktree、`personal-blog/`、skills 或未提交工作。

## 十一、验证

至少执行：

```powershell
pnpm run check:consistency
git diff --check
```

并完成：

- 新文件 ReadLints；
- `clients/**` literal path 全量存在性检查；
- old→new migration map 覆盖率检查；
- 所有 Markdown 相对链接与仓库内引用 anchor 检查；
- 产品 ID 重复/丢失检查；
- canonical 自称重复检查；
- 旧路径兼容入口检查；
- README、owner、status、gate 必填检查；
- 手机代码目录不存在时无虚构实现；
- `apps/agent-shell`、`packages/sdk-ts`、`packages/contracts-ts` 未被越权移动；
- `History/` 无引用；
- `git status` 与 `git diff --cached --name-status` 审计。

如果现有 consistency checker 尚不能验证 `clients/**` 结构：

- 不跨 Lane-CFR 直接修改 `tools/`；
- 在 `clients/READINESS.md` 登记自动化缺口、owner、状态与手动 gate；
- 经 Lane-CFR 领取后再单独实现自动红灯。

静态检查只能证明目录/链接/追踪一致，不能写成客户端实现、平台 PoC、向量执行或 Profile 证据。

## 十二、交付报告

最终报告必须包含：

1. 新目录树；
2. 已迁移文件清单；
3. 保留兼容入口；
4. canonical/owner 变化；
5. 未移动的代码/package 及原因；
6. 新增/更新规则；
7. 产品 ID、anchor 与链接验证结果；
8. `structure-ready` 结论；
9. `implementation-ready` 结论与逐项 blocker；
10. 检查命令和结果；
11. 提交哈希；
12. 未决风险与下一步提示词。

## 十三、会话结束协议

更新 `docs/plan/PROGRESS.md`，按 `docs/checkpoints/TEMPLATE.md` 新建 handoff，记录：

- 已完成/未完成；
- 迁移批次与提交哈希；
- 状态与证据；
- 兼容入口；
- 未决漂移；
- readiness 结论；
- 下一步入口与本提示词路径。

最后逐路径暂存并提交。交接文档是跨会话唯一记忆载体，不得依赖聊天历史。

## 十四、完成定义

只有以下全部满足，本次“客户端项目基础设施与文档迁移”任务才完成：

1. `clients/` 已成为唯一客户端项目根；
2. PC、mobile、shared、Agent Hub 目录职责清晰；
3. 相关 informative 文档已迁移或有据保留；
4. 旧路径只剩薄兼容入口，不存在第二份 canonical 正文；
5. 产品 ID、anchor、链接全部可达；
6. 开发前必需文档系统完整；
7. rules、owner、docs-sync、PROGRESS、handoff 已联动；
8. 未移动所属车道未批准的代码/package；
9. 未创建实现脚手架、mock 或假证据；
10. consistency、whitespace、lint、路径和 canonical 检查通过；
11. `structure-ready` 有逐项证据；
12. `implementation-ready` 按真实 gate 判定，不能预设为 yes。
