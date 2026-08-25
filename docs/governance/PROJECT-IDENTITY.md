# CognitiveOS 项目身份与工作范围

- Status: active repository governance
- Canonical project id: `cognitiveos-personal`
- Repository workspace: `d:\agent-kernel`
- Effective date: 2026-08-25（ADR-0054 子项目化修订；上一版 2026-07-30）

## 1. 一句话定义

本仓库按 [ADR-0054](../adr/0054-repository-subproject-structure-and-1.0.0-finalization.md)
组织为 **core / personal / enterprise / clients 四个子项目目录加共享治理层**，
但仍然只有一个活动实现项目：**`cognitiveos-personal` 是唯一活动实现项目**。
`core/` 是 Personal 所依赖的架构、合同、研究和验证基础，不是第二个并行产品；
`enterprise/` 只是设计层；`clients/` 是并入的客户端子项目。不得把任何其他目录
误解为可独立领取的产品 backlog。

不可放松的公理（A1–A8）与工程原则（P1–P3）只由
[AXIOMS.md](./AXIOMS.md) 拥有；工作流政策由
[DEVELOPMENT-OPERATING-MODEL.md](./DEVELOPMENT-OPERATING-MODEL.md) 拥有。

## 2. 四个子项目边界

### 2.1 core/ — cognitiveos-core（架构与合同层，1.0.0 已定稿）

- `core/specs/`：registry、schema、transition、error 等机器合同；
- `core/crates/`：`cognitive-contracts`、`cognitive-domain`、`cognitive-kernel`、
  `cognitive-akp`（产品中立权威原语与协议）；
- `core/packages/contracts-ts/`、`core/tests/golden/`：TS 合同绑定与跨语言金样；
- `core/conformance/`：符合性向量；
- `core/docs/`：CognitiveOS 白皮书、RFC、评审结论与
  [1.0.0 边界定稿](../../core/docs/VERSION-1.0.0.md)。

core 的职责是定义和验证边界，不是独立发布产品。core 合同的语义变更仍须遵守
规范源优先级和 Lane-CTR；不得为了让某个 Personal 实现通过而放宽负例或篡改规范。
只有确实被 Personal 当前切片需要的合同变化才进入当前工作面。

### 2.2 personal/ — cognitiveos-personal（唯一活动实现项目，1.0.0 已定稿）

`cognitiveos-personal` 是当前唯一可领取、可实现、可测试和可推进的产品项目。其正式
任务、Gate、证据和发布范围唯一登记在
[PERSONAL-DEVELOPMENT-PLAN.md](../plan/PERSONAL-DEVELOPMENT-PLAN.md)；当前事实唯一
登记在 [PROGRESS.md](../plan/PROGRESS.md)。实现位于 `personal/crates/`、
`personal/apps/`、`personal/packages/`、`personal/deploy/`；产品手册在
`personal/handbook/`；稳定产品/架构设计在
[`personal/docs/product/`](../../personal/docs/product/README.md) 与
[`personal/docs/architecture/`](../../personal/docs/architecture/README.md)（均为
informative 设计源，不拥有任务/Gate/current status）。

Personal 1.0.0 的边界与验收见
[personal/docs/VERSION-1.0.0.md](../../personal/docs/VERSION-1.0.0.md)：由已通过的
GMVP-LINUX 与 B01/B02/B03/B04/B05/B08/B09/B12 Gate 定稿，逐字保留 MVP 声明上限。
已知混合现实：`cognitive-store`/`cognitive-runtime`/`cognitive-management` 同时含
可复用 adapter 代码与 Personal 产品代码，整体归属 `personal/`；内部拆分是 ADR-0054
登记的后续重构，不是当前边界的一部分。

### 2.3 enterprise/ — cognitiveos-enterprise（设计层，未激活）

`enterprise/docs/` 只承载候选设计与
[1.0.0 边界/激活门槛定义](../../enterprise/docs/VERSION-1.0.0.md)。在 owner 按该
文件 §4 正式激活前，禁止启动 enterprise 实现、登记实现任务或建立第二产品身份。

### 2.4 clients/ — 客户端子项目（并入）

原独立仓库 `cognitiveos-clients` 于 2026-08-25 以 subtree 并回 `clients/`（历史
保留）。它拥有自己的治理/计划/评审文档树；Web UI 唯一实现路径是
`clients/pc/web/`（ADR-0053，位置条款由 ADR-0054 更新为本仓路径）。客户端永远是
非权威消费者：不得直接写 SQLite 或推进 Task/Effect/Verification 状态。客户端实现
工作默认不在 Personal `P*-T*` 工作面内，除非正式计划明确引用（如 P7-T05）。

## 3. 默认工作范围

没有额外决策时，代理只能：

1. 推进 `PERSONAL-DEVELOPMENT-PLAN.md` 中尚未完成的 Personal 任务；
2. 为当前 Personal 任务修改必要的 `personal/`（及被引用时的 `clients/pc/web/`）、
   `tools/`、共享 `docs/` 和对应文档；
3. 修复影响 Personal 推进的 core 合同漂移，并按 Lane-CTR 完成合同联动；
4. 更新 Personal 当前快照、证据、non-claims 和移交记录。

Console、Memory 深化、Multi-Agent、Windows 安装面等仍可在计划中保持设计或
deferred 状态，但除非 Personal 正式计划、Gate 或新的决策明确激活，不得作为独立
产品车道启动实现，也不得因其未完成阻塞不相关的 Personal 任务。

`personal-blog/` 是独立研究仓库，不属于本仓库任何子项目；它的内容不能改变本仓库
的任务、Gate、规范或发布声明。`History/` 为冻结归档，禁止读取或引用。

## 4. 代理判定优先级

遇到名称、状态或范围冲突时按以下顺序判断：

1. 本文件（项目身份与范围）；
2. [Development Operating Model](./DEVELOPMENT-OPERATING-MODEL.md)（通用治理）；
3. [PERSONAL-DEVELOPMENT-PLAN.md](../plan/PERSONAL-DEVELOPMENT-PLAN.md)（正式任务与 Gate）；
4. [PROGRESS.md](../plan/PROGRESS.md) 的 `Current snapshot`（当前事实）；
5. [PARALLEL-LANES.md](../plan/PARALLEL-LANES.md) 的活动 lease（当前可写路径）；
6. 子项目版本边界与产品/架构文档（[core](../../core/docs/VERSION-1.0.0.md) /
   [personal](../../personal/docs/VERSION-1.0.0.md) /
   [enterprise](../../enterprise/docs/VERSION-1.0.0.md)、Personal 产品/架构设计）；
7. handoff（历史操作连续性）；
8. 根目录 `docs/plan/plan.md`（研究和详细任务卡，不是状态源）。

任何历史记录、旧提示词、旧分支名称或聊天上下文都不能覆盖上述来源。

## 5. 整任务交付与反原地打转规则

默认开发出口是一个完整正式 `P*-T*` 任务，不是单个 Slice、checkpoint 或会话。
完整的 branch/PR/lease、持续推进、阻塞恢复、用户确认、性能验证和 deterministic closure
语义只由 [Development Operating Model](./DEVELOPMENT-OPERATING-MODEL.md) 定义。本文件仅
固定项目身份和来源优先级，不建立第二套工作流。Personal 正式计划拥有任务验收，Current
snapshot 拥有当前事实，active lease table 拥有可写权限；任何实现都不得放松 daemon-only
authority、SecretStore、Intent/Effect、budget/fencing 或 independent verifier 边界。
