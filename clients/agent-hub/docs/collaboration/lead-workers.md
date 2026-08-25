# Agent Hub — 多 Agent 协作（Lead + Workers）

> 类别：informative product design ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 状态：`accepted product direction / implementation not-implemented / evidence none`。

## 1. 协作模式比较

| 模式 | 描述 | v1 取舍 |
|---|---|---|
| 单 Agent | 一个受管 Agent 执行一个 WorkItem | 支持 |
| Lead + Workers（单层） | 一个 Lead 提 proposal，确定性调度器派发多个 Worker | **v1 支持，唯一群组形态** |
| 多层递归编排 | Worker 再生成子 Worker 树 | v1 不支持（复杂度/安全边界未冻结） |
| 跨 Host 群组 | Worker 分布在多台 Host | v1 不支持（单 Host） |
| 对等协商（A2A 自治） | Agent 间自主协商任务 | v1 不支持 |

已冻结（[CONSOLE-AGENTHUB-V1-DEC-017](../decisions/decision-log.md)）：v1 群组仅限**单 Host、一层 Lead+Workers**。

## 2. 职责分离

- **Lead（概率组件）**：只产出 proposal/candidate（任务拆分、分派建议、汇总建议）；不直接改任何 authority 或 Host 控制状态。
- **确定性调度器（Host 内）**：维护 DAG、预算、并发上限、worktree 分配、controller lease、handoff 记录；所有实际派发/停止/合并都经它。
- **Worker（受管 Agent）**：在分配的 worktree/scope/预算内执行；完成需 checks + user acceptance 双轴。

这与仓库架构不变量一致：概率组件只产 candidate/proposal，确定性代码执行调度与提交。

## 3. worktree 与冲突

- coding 类 WorkItem 默认每个 Worker 独立 Git worktree（[CONSOLE-AGENTHUB-V1-DEC-019](../decisions/decision-log.md)），隔离文件与分支。
- 共享同一 workspace 的多 Worker 显示共享文件/Git 状态风险；写冲突进入 `conflict` 状态，需显式合并。
- 每个 Worker 独立 process group/Job/cgroup、环境、credential handle、端口范围。

## 4. handoff 与验收

- 每次 Lead→Worker 或 Worker→Worker handoff 显示：scope、预算、验收标准、允许资源。
- child WorkItem 完成不自动完成 parent；parent 完成需自身 checks + user acceptance。
- 群组完成语言仍分开显示（`agent-reported-done`/`checks`/`user-accepted`），不合并为单一 “done”。

## 5. Open PoC（协作）

全部 `not-run / evidence none`：

- `POC-COLLAB-001` 单层 Lead+Workers DAG 在预算/并发上限下的确定性派发；
- `POC-COLLAB-002` 多 Worker worktree 隔离与冲突检测；
- `POC-COLLAB-003` handoff scope/预算传递无越权。
