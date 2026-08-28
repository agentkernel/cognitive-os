# Personal 助手、项目成员与受治理会话

- 状态：已采纳的 Personal 2.0 产品目标
- 规范语言：[英文原文](agent-integration-and-conversations.md)
- 决策：[ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- 需求基线：
  [Personal 2.0 OPC 需求分析](personal-2.0-opc-requirements-analysis.md)
- 当前交互原型：
  [**personal-20-opc-e2e（旅程减法后）**](../../../clients/docs/design/opc-2.0/personal-20-opc-e2e.canvas.tsx)
- 已归档历史 V2（不是当前 chrome）：
  [pre-subtraction history](../../../clients/docs/design/opc-2.0/history/2026-08-28-pre-subtraction/README.md)
- 原型身份：当前 chrome 是旅程减法后的画布，不是 V2 的 CEO 轨 / X 英雄圈。
- 现有架构输入（待协调）：
  [Agent 生命周期](../architecture/agent-shell-and-agent-lifecycle.md) 与
  [项目、角色与员工](../architecture/project-role-employee.md)

## 1. 三个不可合并的产品身份

| 身份 | 产品职责 | 权威边界 |
|---|---|---|
| **Personal 助手** | 全局解释、导航、研究和提案 | 只产 candidate；只有 daemon 可签发可确认 preview |
| **项目成员 Runtime 定义** | 项目内长期职责、会话、记忆、工作、grant 与历史 | 不等于 Agent 进程；工作权威仍属 daemon |
| **Agent 进程 / Attempt** | 从精确成员版本为单个 Task 启动的可丢弃执行 | 只作有界执行/观察；不拥有项目身份、长期记忆、Secret 或完成判定 |

进程重启不替换成员；会话消息不更新项目；取得 artifact 也不授予执行权限。

## 2. Personal 助手与 Pi

Personal Assistant 是用户看到的系统身份。它拥有最高 UX 权限：可查看当前可用
产品事实并发起所有管理流程，但仍只能通过 daemon 签发的 preview、Owner 确认与
receipt 写入。它可以解释项目、成员、注意事项、来源、不确定性和冲突；导航到
对象；开展项目/角色研究；起草 charter、计划、角色、Model Connection、capability
或恢复 candidate；请求 daemon 的结构化 preview；并解释 receipt。

Pi 可作为精确固定、受管、default-deny 的内部引擎支撑助手，但不进入普通
导航。Pi 不拥有 authority、Provider Secret、Project、Task、
长期 Conversation、episodic archive、semantic Memory 或 completion。Pi 输出始终是
candidate。

解释必须标注来源、scope、freshness、限制与不确定性；不展示模型
chain-of-thought，也不伪造置信度。助手摘要不能直接确认，必须先由 daemon
解析成精确 preview。

## 3. 隐藏的受管 DSH 执行引擎

DeepSeek Harness 是项目成员 Task 进程的隐藏默认执行引擎。它不是 Installed Agent
产品对象、用户可切换的 Harness 或日常目的地。只有故障恢复和高级诊断可显示：

- exact official artifact 的来源、版本、digest、license 与 admission；
- adapter/broker 版本和协议兼容性；
- Windows host/sandbox 资格边界；
- 当前 health 与有界 capability；
- update、兼容变化和 rollback slot；
- 正在使用它的成员与 Task。

DSH 不是 daemon 内的 in-process library，也不是 vendored fork。Personal 以隔离
child process 运行 exact audited artifact，并通过有界 stdio broker 通信。DSH 不得直接
访问 authority database、SecretStore、Provider credential、ambient env secret、原生
MCP/base tools、HMR 或 home patch。Provider traffic 只能经 daemon proxy；可执行动作仍需
Personal admission。

Personal 不嵌入 DSH 原生 UI，也不同步 DSH 原生会话。成员 Conversation、archive、
Memory、Task、Context 与 evidence 归 Personal 所有；DSH 只拿 bounded Context 并返回
candidate/observation。

既有 post-1.0 dsh Path B 事实只在原范围内有效，不资格化 Windows 受管 artifact、
sandbox、supply chain 或 2.0 产品体验。

## 4. 项目群聊与成员工作会话

项目外显示全局 Personal 助手；项目内的主会话是 Owner、管理员和成员群聊。管理员默认
发言；成员只在被 `@`、提交成果、交接、阻塞或需要决策时主动发言。`@manager` 可询问
进度或要求分配；`@member` 可提问或在批准边界内临时调整目标/路径。`@member`
会形成正式 Task revision，而不是影子计划。

Personal 也保留按 scope 隔离的成员工作会话，作为可检查的完整来源记录。成员工作
会话对 Owner、管理员和该成员可见。Conversation
可以包含用户消息、有界检索 Context、engine 输出、Tool/action candidate、receipt 与
来源。完整原始档案在本地保留；每个 Agent 进程只得到相关、有界、脱敏、带 provenance
且标记为 untrusted observation 的片段。

Conversation 不承担 authority。成员/管理员输出先是 candidate；任何改变工作的消息必须
先形成正式 Task 或 revision。涉及 Project、plan、team、Provider/model、Tool/MCP、
permission 或 external rule 的改变，必须请求 daemon preview，再由 Owner 确认、编辑、
收窄或拒绝。应用结果和 receipt 回到会话与对象页。

Agent final text、process exit、Tool result、Provider response、manager agreement 或
engine checkpoint 都不等于 Task completion。

## 5. Composer 与 authority handoff

可见 composer 向当前明确上下文提交：项目外是 Personal 助手，项目内是所选项目群聊。

- 切换项目/助手上下文分别保留未发送草稿；
- 切换不会合并、清空或发送文本；
- `@` 只写入未发送草稿，不能越过项目 scope 或批准 envelope；
- 上下文审批打开中栏 daemon 结构化 preview，不形成第二套 chat authority；聊天里没有
  Approve，也没有 “Don’t ask again”；
- offline/permission 状态保留草稿；
- 普通执行轨迹折叠在 Task/Attempt 后。

## 6. 成员与进程生命周期

`Role Runtime Template -> Project Member Runtime definition -> Task -> Attempt -> Agent process`
必须保持不同身份。daemon 负责 artifact admission、成员版本激活、进程身份、
execution epoch、fencing、health interpretation、update、rollback 与 removal；
process liveness 只是 observation。

停止或丢失进程不删除成员、群聊/工作会话、Memory、work、Attempt 或 evidence。
进程死不等于删成员。
DSH update/rollback 是带 impact preview 的高级 artifact 操作，不能静默删除 Personal
history。

成员 Task 进程可创建有数量、时间、成本与权限限制的内部 subagent。它们是可丢弃
helper，不是项目成员，不保留长期身份或 Memory。

## 7. 替代引擎不在 2.0 范围

Personal 2.0 只资格化 DSH。Hermes、Codex、Cursor 等属于 future adapter candidates；
每项都要独立完成 artifact、license、protocol、capability、secret、sandbox、
lifecycle、platform、negative 和 qualification evidence。DSH/Pi 证据不得转移。

## 8. 状态与 non-claims

Assistant、项目群聊、成员工作会话与高级诊断 surface 覆盖 empty、loading、partial、
stale、permission、error、unknown、offline、long-running、success 与 archived。
未实现 action 显示 `Requires-backend`，不能画成暗示能力已存在的按钮。

本文不实现或资格化 Windows DSH package、sandbox、managed child、Conversation
archive、Personal Assistant、Member Runtime、其他 adapter、support、Gate、
release、Profile 或 multi-Agent benefit。
