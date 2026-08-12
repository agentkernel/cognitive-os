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
fingerprint: "sha256:8c8e8688bc7913198ecace6ced13319d83d21e1d069945de81493b6b5f45646f"
non_claims:
  - 路由清单在生成的 HTTP 参考中；本页解释组合方式，不承诺完整枚举。
---

# daemon 与 HTTP

## 启动顺序（承重）

`serve_personal_loopback`：词法 loopback 检查 → XDG 布局 → 数据库准备/迁移 →
`daemon.lock` 获取 → 打开一个 `SqliteAuthorityStore`（另有同文件的独立
`SchedulerRepository` 连接）→ 恢复已消费 worker 交接 → **一次**私有调度 tick →
bootstrap secret 加载/创建 → TCP 绑定 → 原子发布 `daemon-endpoint.json` → 每连接一
线程服务。恢复与单次 tick 发生在 endpoint 出现之前；调度错误会直接阻止监听。没有
shutdown 路由，也没有常驻调度线程（见[执行链状态](./execution-chain-status.md)）。

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

## 投影

readiness 从文件系统/配置事实评估六组件（`blocked | degraded | ready` +
`first_conversation_ready`），不做 Provider 实时探测。doctor 追加脱敏的六资源/
vault/可运维小节。Provider 代理校验配置 + selected model、内存中解析 secret、经有界
Rustls 传输转发；一次性私有 Unix socket（`POST /chat/completions`）只服务 daemon 启
动的 Pi candidate 进程且禁止 Authorization 头。

## 非 Personal 骨架

`kernel-server --once/--serve` 是 M0 时代的 AKP/shell HTTP 骨架（占位语义、错误也返
回 HTTP 200）。它不是 Personal 表面；将其视为 SDK live 测试使用的历史脚手架。
