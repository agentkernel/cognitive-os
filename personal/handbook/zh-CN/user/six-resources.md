---
doc_id: user.resources-model
locale: zh-CN
kind: concept
audience: [user]
status: partial
generated: false
sources:
  - path: personal/docs/product/cognitive-resource-model.md
  - path: personal/docs/product/mcp-resource-family.md
  - path: personal/docs/product/mcp-resource-family.zh-CN.md
  - path: docs/adr/0057-personal-2-0-mcp-resource-family.md
  - path: personal/crates/cognitive-store/src/memory_store.rs
  - path: personal/crates/cognitive-store/src/skill_store.rs
  - path: core/crates/cognitive-kernel/src/tool_registry.rs
    symbols: ["BUILTIN_TOOL_CATALOG"]
  - path: personal/crates/cognitive-store/src/context_store.rs
tests:
  - personal/crates/cognitive-store/tests/p4_t01_memory_store.rs
  - personal/crates/cognitive-store/tests/p4_t04_skill_store.rs
  - personal/crates/cognitive-store/tests/m5_context_store.rs
fingerprint: "sha256:3d105a259aa8c0437805c6a0b44008b912510b5ac9b69acaab196041f2aaaa5f"
non_claims:
  - 资源族在权威存储中的存在不等于完整的用户工作流；各族缺口见下文与已知限制页。
---

# 当前六类资源

Linux 1.0 与当前 API 分别治理六个资源族。它们有意**不**共享一张表、一套生命周期或
一台状态机。今天每个族都有真实的权威存储与 daemon 服务；用户可及面参差不齐，诚实
标签为 `partial`。

| 族 | 是什么 | 今天的用户可及面 |
|---|---|---|
| **Memory** | 经接纳的持久知识，带 scope、purpose、provenance、版本、过期、遗忘/tombstone | 经 daemon 路由 `remember`/`forget`/explain；全文检索是权威过滤之后的可重建 FTS5 索引；不自动收割对话 |
| **Skill** | 不可变的本地导入包/修订及其绑定 | 经 daemon 路由 import/bind/revoke/explain；脚本绝不自行执行 |
| **Tool** | 七个静态原生操作（workspace 读/搜/写/patch、进程检查、HTTP 抓取、登记检查） | 目录、overlay lifecycle 与校验器已实现；投影将注册、overlay 状态与执行就绪分开报告（已装配族在启用时为 `execution_ready`）；Agent 暴露跟随 overlay 与就绪；HTTP 抓取在 campaign 钉住 HTTPS origin 之前保持失败闭合；执行需要受治理 Effect 路径（见 [Task 与执行](tasks-and-execution.md)） |
| **Context** | 每 Task 的授权输入请求 + 带显式损失的解析视图 | 全部在 daemon 侧：元数据先行过滤、逐 body 重授权、封存视图、digest 绑定缓存 |
| **Task** | 原始意图 → 解释 → 预览 → 已接纳合同 | 四个准入操作可经 HTTP 使用；watch 有界且快照先行 |
| **Runtime/Process** | agent 包、安装、注册、实例、sidecar 会话、进程 attempt | 经 `admin-cli` 的完整 Pi 生命周期；身份绝不合并 |

横切对象（预算、权限、Model、Artifact、Intent/Effect、Evidence、Event）出现在各族内
部，而非新增族。

两条规则解释你将看到的大多数行为：

1. **内容不隐含权限。** 导入的 Skill、安装的 agent、可发现的 Tool、已接纳的 Memory
   本身都不授予运行时能力。
2. **先过滤后排序。** Memory 与 Context 候选先经授权、scope、tombstone、新鲜度过滤，
   任何排序才能看到它们；被拒内容连排序都无法影响。

从 Linux 1.0 推迟：embedding/向量/图谱 Memory、skill 市场、已采纳的 MCP 资源族
（已交付的 post-1.0 MCP Tool transport/dynamic-Tool MVP 不等于该资源族）、多 Agent
编排与桌面优先 UI 重设计。

## Personal 2.0 第七族（`Requires-backend`）

Personal 2.0 已采纳 **MCP** 为第七个用户可见资源族。这不改变当前六族 Resource
Manager API，也不会把 MCP 内容自动变成原生 Tool。

目标资源族拥有彼此分离的 server、package、connection、advertised capability、
binding、health 与 quarantine 身份。**联邦资源**投影保留来源身份、provenance、
revision/freshness、trust、availability 与允许动作，而不把外部权威复制进 Personal。

MCP 广告对象仍经各自准入路径进入既有资源族：tool 是带版本绑定的 Tool candidate，
protocol resource 是 Context candidate，prompt/可复用指令是 Skill candidate。仅发现不
授予读取或派发权限。daemon policy 仍须授权每次使用；变更仍须遵守
persist-before-dispatch Intent/Effect、fencing、budget 与独立验证。

目标视图严格区分：

- `Native`：Personal 拥有的本地能力/资源；
- `Observed`：只读发现的事实；
- `Governed`：经 daemon 授权、界定且可审计的使用；
- `Verified`：经独立验证的结果或当前事实。

它们不是自动成熟度阶梯，`Verified` 也不是 release 或资格化声明。MCP 资源族、联邦
API、持久化、trust policy 与 UI 均尚未实现。
