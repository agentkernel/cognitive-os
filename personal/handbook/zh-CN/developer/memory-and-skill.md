---
doc_id: dev.memory-skill
locale: zh-CN
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: crates/cognitive-store/src/sqlite/memory.rs
  - path: crates/cognitive-store/src/memory_admission.rs
    symbols: ["admit_memory_candidate"]
  - path: crates/cognitive-kernel/src/memory_admission.rs
    symbols: ["decide_memory_admission"]
  - path: crates/cognitive-store/src/sqlite/harness_skill.rs
  - path: apps/kernel-server/src/personal/resource_api.rs
  - path: apps/kernel-server/src/personal/memory_skill_consumer.rs
    symbols: ["load_governed_memory_skill_candidates"]
  - path: crates/cognitive-store/src/memory_skill_consumption.rs
    symbols: ["memory_skill_consumption_migration_entry"]
tests:
  - crates/cognitive-store/tests/p4_t01_memory_store.rs
  - crates/cognitive-store/tests/p4_t02_memory_search.rs
  - crates/cognitive-store/tests/p4_t04_skill_store.rs
  - apps/kernel-server/tests/p4_t05_resource_api.rs
  - apps/kernel-server/tests/p8_t12_resource_manager.rs
fingerprint: "sha256:533041e14638e3cbe085677daa0be1cee801da0d55f923fc0c33390677e1cec3"
non_claims:
  - 生命周期正确性证据是聚焦测试证据；B08 类 Gate 记账由正式计划拥有。
---

# Memory 与 Skill

## Memory：candidate → decision → object

没有任何路径直接写 `MemoryObject`。服务接缝（`admit_memory_candidate`，daemon 唯一
生产调用路径经 `POST /management/resource/v1/memory/remember`）重载当前 Context
source、重推导确定性策略结论（`decide_memory_admission`），并拒绝与之不符的调用方决
定——然后在单事务内持久化 candidate + 带原因码的 decision + object + 版本行 + FTS
行，并复核 source 绑定（过期 source ⇒ 冲突）。

生命周期是追加式事实：forget 与 expiry tombstone（精确截止检查、重复清扫被拒）、
expected-version CAS 下的版本化替换（`UNIQUE(supersedes_memory_id)` 谱系）、FTS 行
的原子迁移。FTS5 索引是可弃的：重建只从权威行填充，被 tombstone 的 Memory 绝不可能
经索引复活。

检索（`search_memory_candidates`）先跑权威过滤 CTE（admit 决定、无 tombstone、精确
scope+purpose、retention 未过期、source 绑定现时），之后才 `MATCH`，按 `bm25` 排序
并稳定破平。

## Skill：不可变包、精确 pin

导入拒绝不安全的本地来源（绝对/UNC/`..` 路径）与 digest/载荷漂移；package 与
revision 原子提交。绑定要求同 workspace scope 下 `compatible` 的 revision；撤销是独
立不可变事实（active = 状态 active 且无撤销行）；同包 supersede 追加谱系、每个
revision 只允许一个后继，既有绑定保持精确 pin——绝不漂移到后继。

## HTTP 可及面

management 通道发布生命周期前置条件，并在不直连 SQLite 的情况下完成 Memory
remember/review/forget 与 Skill import/revision-inspect/bind/supersede/revoke。
通用 Resource Manager（`GET /management/resource/v1/list|inspect`）从同一权威行投影
未墓碑 Memory 对象与 Skill binding；它不是通用 Resource 表，Memory forget 仍是族
`forget` 动词而不是 Manager `revoke`。
公开 remember 接受未封存的 owner 字段（`text`、scope、purpose、retention）；daemon
加载持久 `GovernanceSeed` 并组合封存 header。已持有封存
`WorkspaceContextSource` + `MemoryCandidate` 信封的调用方仍可走原路径。评测或测试
代码不得在公开正例路径上自行铸造封存 header。
`skill/binding/revoke` 必须先于 `skill/bind` 匹配：后者是前者的前缀，否则每次撤销都
会落到 bind handler。所有变更行与 revision 谱系在 daemon 重启后仍可读取。被策略
拒绝的 Memory candidate 可直接重试字段完全相同的封存 source，无需原始清理；相同 id
但字段不同的 source 仍冲突。
task 通道：task 绑定的投影/watch，以及生产受治理消费方。
`resolve_authorized_task_context` 只在元数据资格、精确当前 Task 或 workspace
scope/pin/digest 复核和当前 forget/revoke 重验之后装载 Memory/Skill，并把片段写入
封存 ContextView。
v24 只追加消费记录按 Task、epoch、ContextRequest 与 session 绑定，供跨会话复用；
task 通道用 `GET /task/resource/v1/consumption` 读取该记录，不接受 `query_text`
或调用方提供的 Skill binding，只返回脱敏精确钉、session/`reuse_of` 与
`authorized_exact_pin`；遗忘、撤销或 digest 漂移的钉在返回任何 pin 前失败闭合。
session 2 与重启后的 GET 都只从该持久行恢复（设置 `reuse_of`、精确钉、不重放
chat）；带调用方 `query_text` 的 `POST /task/resource/v1/consumption` 不能替换它。
最近一条是最后追加的行，而不是哈希身份字典序最大的行。
复用必须重读当前权威事实；确定性记录身份绑定 principal/tenant/scope/purpose、
request digest 与精确钉，遗忘、撤销、digest 漂移或竞争持久记录一律失败闭合。
task bearer 在任何管理变更前即被拒绝。

公开备份归档携带 digest 绑定的 Memory/Skill 导出 sidecar，永不复制原始
`authority.sqlite`。
