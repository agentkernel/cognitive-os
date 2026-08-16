---
doc_id: dev.daemon-http-surface
locale: zh-CN
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: apps/kernel-server/src/personal/server.rs
    symbols: ["serve_personal_loopback", "PersonalDaemonConfig"]
  - path: apps/kernel-server/src/personal/auth.rs
    symbols: ["LocalSessionAuthority", "ChannelClass"]
  - path: apps/kernel-server/src/personal/bounds.rs
  - path: apps/kernel-server/src/personal/readiness.rs
    symbols: ["evaluate_personal_readiness"]
  - path: apps/kernel-server/src/personal/provider_proxy.rs
  - path: apps/kernel-server/src/personal/route_observation.rs
    symbols: ["observation_response_headers"]
  - path: apps/kernel-server/src/personal/fault_profile.rs
    symbols: ["handle"]
  - path: apps/kernel-server/src/personal/tool_lifecycle.rs
    symbols: ["handle"]
  - path: apps/kernel-server/src/personal/task_api.rs
    symbols: ["TaskApi"]
tests:
  - apps/kernel-server/tests/p1_t04_personal_daemon.rs
  - apps/kernel-server/tests/p1_t05_personal_readiness.rs
  - apps/kernel-server/tests/p1_t07_provider_proxy.rs
  - apps/kernel-server/tests/p9_t07_route_observation.rs
  - apps/kernel-server/tests/p2_t24_effect_fault.rs
  - apps/kernel-server/tests/p2_t25_tool_lifecycle.rs
fingerprint: "sha256:1ffc627dfa7517a333219c321f1d22e75e5eb5fdb0a93de1f396b7f698a11a52"
non_claims:
  - 路由清单在生成的 HTTP 参考中；本页解释组合方式，不承诺完整枚举。
---

# daemon 与 HTTP

## 启动顺序（承重）

`serve_personal_loopback`：词法 loopback 检查 → XDG 布局 → 数据库准备/迁移 →
`daemon.lock` 获取 → 打开一个 `SqliteAuthorityStore`（另有同文件的独立
`SchedulerRepository` 连接）→ 恢复已消费 worker 交接 → 在
`data_dir()/artifacts` 打开唯一有界 ArtifactStore → 组合共享该 CAS 的 native Tool
descriptor/router → bootstrap
secret 加载/创建 → TCP 绑定 → 原子发布 `daemon-endpoint.json` → 启动唯一周期调度
worker → 每连接一线程服务。监听器与 endpoint 出现前不执行调度 pass，因此本进程随后接纳的 Task 可被后
续 pass 观察。该 worker 独占调度连接，在非重入门后按固定延迟 250 ms 串行运行；pass
级错误只记录并重试，逐行错误仍在单趟内隔离。顺序退出时会显式取消、唤醒并 join
worker。仍没有 HTTP shutdown 路由（见[执行链状态](./execution-chain-status.md)）。

## 认证

两个刻意无关的凭据平面：

- **本地通道 bearer**（本表面）：`POST /local/session` 用每次启动的 bootstrap
  secret 换取 `management` 或 `task` 令牌；每个认证路由先检查通道绑定。进程本地、
  12 小时/30 分钟过期、无逐操作 scope。bootstrap 与 session token 各自使用 OS CSPRNG
  的 256 bit；熵失败或无效/重复探针会在创建文件/session 前 fail closed，绝无
  PID/时间/hash fallback。bootstrap 重载只接受当前 lowercase
  `boot-32hex-32hex` 形状；旧版可预测或畸形非空凭据会阻止启动，不会被兼容接受。
- **特权管理会话**（`admin-cli`）：由 `cognitive-management` 校验的 JSON 文档——独立
  平面，与本地 bearer 不可互换。

## 请求卫生

路由前的固定界限：1 MiB 请求体（硬读 8 MiB）、16 KiB/64 头、10 s/30 s 超时、32/16
连接上限、拒绝 Cookie、可选 Host 校验——各配注册错误码。路由是对 `METHOD /path` 字符
串的手写前缀匹配，分布在 `server.rs`、`task_api.rs`、`resource_api.rs`（生成的
[HTTP 参考](../reference/http-api.md)枚举完整表与通道）。

management Resource 表面提供只读生命周期前置条件、封存 Context source 准入、
Memory remember/review/forget，以及 Skill import/inspect/bind/supersede/revoke。
变更必须持有 management bearer；task bearer 在进入 handler 前失败。创建成功使用
HTTP 状态 `201`，持久行在重启后仍可检查。
task 通道通过 `GET /task/resource/v1/consumption?task_ref=…` 读取 daemon 写入的
最新 Memory/Skill 消费记录：只返回精确钉、session 关联和 `reuse_of`。
`query_text` 与 `skill_binding_id` 视为用户重述并被拒绝。遗忘、撤销或 digest
漂移的钉在响应前失败闭合，且绝不返回 Memory/Skill 正文。session 2 与重启后的
GET 读取同一持久行；带调用方 `query_text` 的 POST 不能替换这些钉。

management 的 `POST/GET /management/resource/v1/fault-profile` 为一个
`task_ref` 持久化默认关闭、评测授权的固定 fault profile。普通 task 调用方被拒绝
（`RESOURCE_FAULT_PROFILE_CHANNEL_FORBIDDEN`）。task 通道通过
`GET /task/effects?task_ref=…` 读取有界 Effect 历史：不透明 original-key digest、
stage、outcome/reconcile class、mutation count 仅 0/1 或在不确定时缺省，以及
report refs。receipt、原始参数和额外查询字段失败闭合。

management 的 `GET/POST /management/resource/v1/tool*` 投影已登记原生 Tool 的
overlay lifecycle（`enabled` / `disabled` / `quarantined` / `revoked`）、
`execution_readiness` 与 `agent_exposed`。overlay 状态永不进入不可变 descriptor
digest。task 通道不能变更 lifecycle。`GET /task/resource/v1/tool/exposure`
返回最窄 Agent 暴露集合与 digest；`POST /task/resource/v1/tool/selection` 仅在
`candidate_set_digest` 匹配该 digest 且所选 operation 已被暴露时记录收据。
prompt/body/receipt 重述失败闭合。

## 投影

readiness 从文件系统/配置事实评估六组件（`blocked | degraded | ready` +
`first_conversation_ready`），从不发出 Provider 请求；但它会把配置里的
`secret_ref` 拿到 SecretStore 真实解析一次——后端可达并不代表该引用仍指向已存条目：
悬空引用报 `secret_ref_resolves: false` 并以 `provider_secret_unresolvable` 阻塞，
后端无法作答则以 `provider_secret_store_unavailable` 阻塞。解析出的材料立即丢弃，
绝不进入任何 fact。解析只使用已加载的 Provider 配置快照；绝不重载 `provider.json`
后把较新的 secret 引用与较旧的 provider/model/digest 事实混合。一次 status/doctor
评估只绑定一次 SecretStore：secret 探针与 provider 的 `secret_ref` 解析共用该后端，
探针已证明后端无法作答时不再发起 `get`，材料立即丢弃，也不跨请求缓存就绪结果
（没有 stale-ready TTL）。doctor 追加脱敏的六资源/vault/可运维小节。Provider 代理校验配置 + selected model、内存中解析 secret、经有界
Rustls 传输转发。成功的代理响应始终携带 `X-CognitiveOS-Provider-Network-Nanos`。
嵌套 preflight 计时与 correlation 回显默认拒绝，仅当
`COGNITIVEOS_PI_ROUTE_OBSERVATION=enabled` 且请求携带一条形态正确的不透明
correlation id 时才发出；畸形或重复的 id 被忽略，产品 body 不变，观测器不写任何
东西。一次性私有 Unix socket（`POST /chat/completions`）只服务 daemon 启
动的 Pi candidate 进程且禁止 Authorization 头。

## 非 Personal 骨架

`kernel-server --once/--serve` 是 M0 时代的 AKP/shell HTTP 骨架（占位语义、错误也返
回 HTTP 200）。它不是 Personal 表面；将其视为 SDK live 测试使用的历史脚手架。
