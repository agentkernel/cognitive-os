# CognitiveOS 项目身份与工作范围

- Status: active repository governance
- Canonical project id: `cognitiveos-personal`
- Repository workspace: `d:\agent-kernel`
- Effective date: 2026-07-30

## 1. 一句话定义

本仓库现在是 **CognitiveOS 架构参考库 + CognitiveOS Personal 唯一活动实现项目**。
原来的 CognitiveOS 设计、白皮书、规范资产和通用内核不是另一个并行产品项目；它们是
Personal 实现所依赖的架构、合同、研究和验证基础。当前开发目标只有
`cognitiveos-personal`，不得把“CognitiveOS 参考实现”误解为第二个待交付产品。

不可放松的公理（A1–A8）与工程原则（P1–P3）只由
[AXIOMS.md](AXIOMS.md) 拥有；工作流政策由
[DEVELOPMENT-OPERATING-MODEL.md](DEVELOPMENT-OPERATING-MODEL.md) 拥有。

## 2. 两层边界

### 2.1 CognitiveOS Architecture（架构层）

以下内容属于架构/合同/研究层：

- `core/docs/architecture/` 中的 CognitiveOS 白皮书、RFC、评审结论和架构说明；
- `core/specs/` 中的 registry、schema、transition、error 和其他机器合同；
- `core/conformance/`、`core/tests/golden/` 及其用于验证架构合同的夹具；
- 能被 Personal 复用的 Rust/TypeScript 通用层。

架构层的职责是定义和验证边界，不是独立发布产品。架构层的语义变更仍须遵守
规范源优先级和 Lane-CTR；不得为了让某个 Personal 实现通过而放宽负例或篡改规范。
只有确实被 Personal 当前切片需要的架构变化才进入本仓库当前工作面。

### 2.2 CognitiveOS Personal（项目层）

`cognitiveos-personal` 是当前唯一可领取、可实现、可测试和可推进的产品项目。其正式
任务、Gate、证据和发布范围唯一登记在
[PERSONAL-DEVELOPMENT-PLAN.md](../plan/PERSONAL-DEVELOPMENT-PLAN.md)；当前事实唯一
登记在 [PROGRESS.md](../plan/PROGRESS.md)。实现可以使用架构层资产，但任务必须以
`P*-T*` Personal 任务或明确关联的 REQ/F/IMP 为落点。

`docs/plan/archive/DEVELOPMENT-PLAN.md`、M0-M11/M6/v0.1 计划、旧 lane prompts 和历史
milestone 状态只作为架构形成过程、验证资产或复用参考保留。除非 Personal 正式计划
明确引用，它们不能生成当前任务、活动 lease、产品 Gate 或发布要求。

Personal 的稳定产品愿景、用户资源模型和 release-scope 设计位于
[`personal/docs/product/`](../../personal/docs/product/README.md)；Personal 如何组合 CognitiveOS
合同、daemon、Shell 与 Agent runtime 的说明位于
[`personal/docs/architecture/`](../../personal/docs/architecture/README.md)。两者都是 informative
设计源：前者不拥有任务/Gate/current status，后者不创建 public schema/transition/REQ。

## 3. 默认工作范围

没有额外决策时，代理只能：

1. 推进 `PERSONAL-DEVELOPMENT-PLAN.md` 中尚未完成的 Personal 任务；
2. 为当前 Personal 任务修改必要的 `crates/`、`apps/`、`packages/`、`tests/`、`tools/`
   和对应文档；
3. 修复影响 Personal 推进的架构合同漂移，并按 Lane-CTR 完成合同联动；
4. 更新 Personal 当前快照、证据、non-claims 和移交记录。

Console、独立客户端、Memory、Multi-Agent、Web UI、Windows 安装面等仍可在计划中
保持设计或 deferred 状态，但除非 Personal 正式计划、Gate 或新的决策明确激活，
不得作为独立产品车道启动实现，也不得因其未完成阻塞不相关的 Personal 任务。

`personal-blog/` 是独立研究仓库，不属于 `cognitiveos-personal` 的实现范围；它的
内容不能改变本仓库的任务、Gate、规范或发布声明。

## 4. 代理判定优先级

遇到名称、状态或范围冲突时按以下顺序判断：

1. 本文件（项目身份与范围）；
2. [Development Operating Model](DEVELOPMENT-OPERATING-MODEL.md)（通用治理）；
3. [PERSONAL-DEVELOPMENT-PLAN.md](../plan/PERSONAL-DEVELOPMENT-PLAN.md)（正式任务与 Gate）；
4. [PROGRESS.md](../plan/PROGRESS.md) 的 `Current snapshot`（当前事实）；
5. [PARALLEL-LANES.md](../plan/PARALLEL-LANES.md) 的活动 lease（当前可写路径）；
6. Personal 产品/架构文档（稳定设计与组合说明）；
7. handoff（历史操作连续性）；
8. 根目录 `docs/plan/plan.md`（研究和详细任务卡，不是状态源）。

任何历史记录、旧提示词、旧分支名称或聊天上下文都不能覆盖上述来源。

## 5. 整任务交付与反原地打转规则

默认开发出口是一个完整正式 `P*-T*` 任务，不是单个 Slice、checkpoint 或会话。
完整的 branch/PR/lease、持续推进、阻塞恢复、用户确认、性能验证和 deterministic closure
语义只由 [Development Operating Model](DEVELOPMENT-OPERATING-MODEL.md) 定义。本文件仅
固定项目身份和来源优先级，不建立第二套工作流。Personal 正式计划拥有任务验收，Current
snapshot 拥有当前事实，active lease table 拥有可写权限；任何实现都不得放松 daemon-only
authority、SecretStore、Intent/Effect、budget/fencing 或 independent verifier 边界。
