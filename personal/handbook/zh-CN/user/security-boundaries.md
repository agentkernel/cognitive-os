---
doc_id: user.security-boundaries
locale: zh-CN
kind: concept
audience: [user]
status: implemented
generated: false
sources:
  - path: apps/kernel-server/src/personal/auth.rs
    symbols: ["LocalSessionAuthority"]
  - path: apps/kernel-server/src/personal/bounds.rs
  - path: packages/pi-cognitiveos/src/tool-policy.ts
  - path: crates/cognitive-runtime/src/pi_launcher.rs
    symbols: ["admit_pi_launch"]
  - path: docs/governance/AXIOMS.md
tests:
  - apps/kernel-server/tests/p1_t04_personal_daemon.rs
  - apps/kernel-server/tests/p2_t18_local_token_csprng.rs
  - packages/pi-cognitiveos/src/safety.test.ts
  - crates/cognitive-runtime/tests/pi_linux_launcher.rs
fingerprint: "sha256:4434049915e38bf4384f04b027af5b52b9f08d158a0f14755a05a6ed4eb0ba42"
non_claims:
  - Windows 本地 runtime 文件仍缺少显式 ACL 加固——OS CSPRNG 令牌生成不构成 ACL 声明。
---

# 安全边界

## 网络

daemon 只绑定 loopback，并在监听前就以词法拒绝非 loopback 绑定地址。本地监听器无
TLS（有意仅限 localhost）、无 cookie（任何 `Cookie` 头被拒）、`Host` 校验可选。
Provider egress 仅 HTTPS 且禁跳转。

## 身份与通道

`POST /local/session` 用每次启动生成的 `local-bootstrap.secret`（XDG runtime 目录、
0600）换取**通道绑定**的 bearer：management 令牌永远不能调用 task 路由，反之亦然。
会话会过期（绝对 12 小时 / 空闲 30 分钟），并随 daemon 进程消亡。

bootstrap 与 session opaque token 各自携带 OS CSPRNG 生成的 256 bit。OS 熵不可用、
输出过短、零 block 或独立探针 block 重复时，初始化/会话签发会在创建文件、session 或
token 前 fail closed，绝无 PID/时间/hash fallback。持久文件若仍是旧版可预测形状或任意
畸形非空形状，不会被兼容接受：启动 fail closed。daemon 停止时只删除该 runtime 凭据，
下次启动即可重新签发 CSPRNG bootstrap。能读 bootstrap 文件者仍可自命任意 principal；
按 OS 用户的隔离依赖文件权限（无 Windows ACL 加固）。

## Agent 遏制

- Pi shell 扩展拒绝 `project_trust` 与全部 Pi 内置工具；源码扫描断言扩展自身无文件系
  统/子进程/SQLite/key 访问。
- daemon 启动的 Pi **candidate** 进程在禁用工具、skill、会话与扩展发现、清空环境白名
  单、帧字节上限与硬截止下运行；其唯一网络路径是回连 daemon 的一次性私有 socket。
- `admit_pi_launch` 对 Windows 原生/WSL2 主机、缺失 sandbox 适配器、digest 不符及注
  册 HTTPS 代理端点之外的任何模型 egress 一律 fail-close。

## 请求界限（DoS 卫生）

1 MiB 请求体、16 KiB 头部块、64 个头、10 s/30 s 读超时、32 连接（16 在途）——全部
fail-closed 并返回注册错误码。

## 静态数据靠什么保护

权威数据库是 daemon 拥有的 0600 WAL SQLite 文件；secret 只存在于 Secret Service
（见 [Provider 与 secret](./provider-and-secrets.md)）。命名 Provider Control Plane
账户在 SQLite 中只持久化不透明 `secret_ref`；API key 永不出现在权威行、CLI 输出、
审计载荷或 agent 可读文件中。CLI 操作（含 `--allow-private-network` /
`--allow-insecure-http` 与 `--reconfirm`）见
[Provider Control Plane](./provider-control-plane.md)。备份按构造排除 secret 材料。
追加式审计/事件历史无法经任何 daemon 面被改写——数据库触发器直接拒绝 update/delete。
