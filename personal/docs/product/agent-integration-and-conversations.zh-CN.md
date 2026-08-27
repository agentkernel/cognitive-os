# Personal 助手、已安装 Agent 与数字员工会话

- 状态：已采纳的 Personal 2.0 产品目标
- 规范语言：[英文原文](agent-integration-and-conversations.md)
- 决策：[ADR-0059](../../../docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md)
- 架构：
  [Agent 生命周期](../architecture/agent-shell-and-agent-lifecycle.md) 与
  [项目、角色与员工](../architecture/project-role-employee.md)

## 1. 三个不可合并的产品身份

| 身份 | 产品职责 | 权威边界 |
|---|---|---|
| **Personal 助手** | 全局解释、导航、研究和提案 | 只产 candidate；只有 daemon 可签发可确认 preview |
| **数字员工** | 项目内长期成员，拥有职责、会话、记忆、工作和历史 | 不等于 Agent 进程；工作权威仍属 daemon |
| **已安装 Agent / Runtime** | 员工使用的已资格化执行集成 | 只作有界执行/观察；不拥有项目、记忆、Secret 或完成判定 |

runtime 重启不替换员工；会话消息不更新项目；安装包也不授予执行权限。

## 2. Personal 助手与 Pi

Personal 助手是用户看到的系统身份。它可解释项目、员工、待办、来源、
不确定性和冲突；导航到对象；开展项目/角色研究；起草 charter、计划、角色、
绑定、预算或恢复 candidate；请求 daemon 的结构化 preview；并解释 receipt。

Pi 可作为精确固定、受管、default-deny 的内部引擎支撑助手，但不进入普通
“已安装 Agent”列表。Pi 不拥有 authority、Provider Secret、Project、Task、
长期 Conversation、episodic archive、semantic Memory 或 completion。Pi 输出始终是
candidate。

解释必须标注来源、scope、freshness、限制与不确定性；不展示模型
chain-of-thought，也不伪造置信度。助手摘要不能直接确认，必须先由 daemon
解析成精确 preview。

## 3. 随产品提供的受管 DSH Agent

DeepSeek Harness 是 Personal 2.0 的 **preinstalled managed Installed Agent**，
也是项目数字员工的默认 runtime。Settings > Installed Agents 显示：

- exact official artifact 的来源、版本、digest、license 与 admission；
- adapter/broker 版本和协议兼容性；
- Windows host/sandbox 资格边界；
- 当前 health 与有界 capability；
- update、兼容变化和 rollback slot；
- 正在使用它的员工与 Task。

DSH 不是 daemon 内的 in-process library，也不是 vendored fork。Personal 以隔离
child process 运行 exact audited artifact，并通过有界 stdio broker 通信。DSH 不得直接
访问 authority database、SecretStore、Provider credential、ambient env secret、原生
MCP/base tools、HMR 或 home patch。Provider traffic 只能经 daemon proxy；可执行动作仍需
Personal admission。

Personal 不嵌入 DSH 原生 UI，也不同步 DSH 原生会话。员工 Conversation、archive、
Memory、Task、Context 与 evidence 归 Personal 所有；DSH 只拿 bounded Context 并返回
candidate/observation。

既有 post-1.0 dsh Path B 事实只在原范围内有效，不资格化 Windows 受管 artifact、
sandbox、supply chain 或 2.0 产品体验。

## 4. 数字员工会话

每个 Personal-owned Conversation 绑定 Owner、Project 和 employee。它可以包含用户消息、
有界检索 Context、engine 输出、Tool/action candidate、receipt 与来源。全部会话在本地
归档并索引，但每次只检索相关、有界、脱敏、带 provenance 且标记为 untrusted
observation 的片段。

Conversation 不承担 authority。员工/管理员输出先是 candidate；涉及 Project、plan、
team、budget、Provider、Tool、permission 或 external rule 的改变，必须请求 daemon
preview，再由 Owner 确认、编辑、收窄或拒绝。应用结果和 receipt 回到会话与对象页。

Agent final text、process exit、Tool result、Provider response、manager agreement 或
engine checkpoint 都不等于 Task completion。

## 5. 单一活动 composer

右侧 rail 可与 Personal 助手、项目管理员或员工对话，但同一时刻只允许一个 composer
提交：

- composer 与 submit 明确写出 recipient；
- 切换 recipient 会保留双方草稿；
- 切换不会合并、清空或发送草稿；
- 只有一个 keyboard focus owner；
- Inbox approval 打开结构化 preview，不创建第二个 chat composer；
- offline/permission 状态保留草稿。

## 6. Runtime 生命周期

`Artifact -> Installation -> Agent definition -> Runtime instance -> Task execution -> OS process -> Conversation`
必须保持不同身份。daemon 负责 artifact admission、installation activation、
employee/runtime binding、execution epoch、budget、fencing、health interpretation、
update、rollback 与 removal；process liveness 只是 observation。

断开 runtime 不删除 employee、Conversation、Memory、work 或 evidence。卸载 DSH
必须经过 impact preview，不能静默删除 Personal history。

## 7. Future adapters

Personal 2.0 只资格化 DSH。Hermes、Codex、Cursor 等属于 future adapter candidates；
每项都要独立完成 artifact、license、protocol、capability、secret、sandbox、
lifecycle、platform、negative 和 qualification evidence。DSH/Pi 证据不得转移。

## 8. 状态与 non-claims

Installed Agent 与 Conversation surface 覆盖 empty、loading、partial、stale、
permission、error、unknown、offline、long-running、success 与 archived。
未实现 action 显示 `Requires-backend`，不能画成暗示能力已存在的按钮。

本文不实现或资格化 Windows DSH package、sandbox、managed child、Conversation
archive、Personal Assistant、employee runtime、其他 adapter、support、Gate、
release、Profile 或 multi-Agent benefit。
