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
tests:
  - apps/kernel-server/tests/p1_t04_personal_daemon.rs
  - apps/kernel-server/tests/p1_t05_personal_readiness.rs
  - apps/kernel-server/tests/p1_t07_provider_proxy.rs
fingerprint: "sha256:ff5eba81b9e30287dd02a194f895540c4221fa746d1152f85dcea2d9dc0abbc3"
non_claims:
  - 路由清单在生成的 HTTP 参考中；本页解释组合方式，不承诺完整枚举。
---

# daemon 与 HTTP

## 启动顺序（承重）

`serve_personal_loopback`：词法 loopback 检查 → XDG 布局 → 数据库准备/迁移 →
`daemon.lock` 获取 → 打开一个 `SqliteAuthorityStore`（另有同文件的独立
`SchedulerRepository` 连接）→ 恢复已消费 worker 交接 → 组合 native Tool
descriptor/router → 在 `data_dir()/artifacts` 打开唯一有界 ArtifactStore → bootstrap
secret 加载/创建 → TCP 绑定 → 原子发布 `daemon-endpoint.json` → 启动唯一周期调度
worker → 每连接一线程服务。监听器与 endpoint 出现前不执行调度 pass，因此本进程随后接纳的 Task 可被后
续 pass 观察。该 worker 独占调度连接，在非重入门后按固定延迟 250 ms 串行运行；pass
级错误只记录并重试，逐行错误仍在单趟内隔离。顺序退出时会显式取消、唤醒并 join
worker。仍没有 HTTP shutdown 路由（见[执行链状态](./execution-chain-status.md)）。

## 认证

两个刻意无关的凭据平面：

- **本地通道 bearer**（本表面）：`POST /local/session` 用每次启动的 bootstrap
  secret 换取 `management` 或 `task` 令牌；每个认证路由先检查通道绑定。进程本地、
  12 小时/30 分钟过期、无逐操作 scope。
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

## 投影

readiness 从文件系统/配置事实评估六组件（`blocked | degraded | ready` +
`first_conversation_ready`），从不发出 Provider 请求；但它会把配置里的
`secret_ref` 拿到 SecretStore 真实解析一次——后端可达并不代表该引用仍指向已存条目：
悬空引用报 `secret_ref_resolves: false` 并以 `provider_secret_unresolvable` 阻塞，
后端无法作答则以 `provider_secret_store_unavailable` 阻塞。解析出的材料立即丢弃，
绝不进入任何 fact。解析只使用已加载的 Provider 配置快照；绝不重载 `provider.json`
后把较新的 secret 引用与较旧的 provider/model/digest 事实混合。doctor 追加脱敏的六资源/vault/可运维小节。Provider 代理校验配置 + selected model、内存中解析 secret、经有界
Rustls 传输转发；一次性私有 Unix socket（`POST /chat/completions`）只服务 daemon 启
动的 Pi candidate 进程且禁止 Authorization 头。

## 非 Personal 骨架

`kernel-server --once/--serve` 是 M0 时代的 AKP/shell HTTP 骨架（占位语义、错误也返
回 HTTP 200）。它不是 Personal 表面；将其视为 SDK live 测试使用的历史脚手架。
