---
doc_id: user.resources-model
locale: zh-CN
kind: concept
audience: [user]
status: partial
generated: false
sources:
  - path: docs/product/personal/cognitive-resource-model.md
  - path: crates/cognitive-store/src/memory_store.rs
  - path: crates/cognitive-store/src/skill_store.rs
  - path: crates/cognitive-kernel/src/tool_registry.rs
    symbols: ["BUILTIN_TOOL_CATALOG"]
  - path: crates/cognitive-store/src/context_store.rs
tests:
  - crates/cognitive-store/tests/p4_t01_memory_store.rs
  - crates/cognitive-store/tests/p4_t04_skill_store.rs
  - crates/cognitive-store/tests/m5_context_store.rs
fingerprint: "sha256:a628e54f3d77af804a2eaeee37a6cf36972210a4a083f3081ff6304fa46f5997"
non_claims:
  - 资源族在权威存储中的存在不等于完整的用户工作流；各族缺口见下文与已知限制页。
---

# 六类资源

Personal 分别治理六个资源族。它们有意**不**共享一张表、一套生命周期或一台状态机。今
天每个族都有真实的权威存储与 daemon 服务；用户可及面参差不齐，诚实标签为 `partial`。

| 族 | 是什么 | 今天的用户可及面 |
|---|---|---|
| **Memory** | 经接纳的持久知识，带 scope、purpose、provenance、版本、过期、遗忘/tombstone | 经 daemon 路由 `remember`/`forget`/explain；全文检索是权威过滤之后的可重建 FTS5 索引；不自动收割对话 |
| **Skill** | 不可变的本地导入包/修订及其绑定 | 经 daemon 路由 import/bind/revoke/explain；脚本绝不自行执行 |
| **Tool** | 六个静态原生操作（workspace 读/搜/写/patch、进程检查、HTTP 抓取） | 目录与校验器已实现；执行需要受治理 Effect 路径（见 [Task 与执行](./tasks-and-execution.md)） |
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

按设计推迟（Linux 1.0 范围）：embedding/向量/图谱 Memory、skill 市场、动态工具生态
（MCP 适配器仅为 post-1.0 fixture 资格化）、多 agent 编排与 Web UI。
