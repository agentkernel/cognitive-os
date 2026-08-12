---
doc_id: user.provider-and-secrets
locale: zh-CN
kind: guide
audience: [user]
status: implemented
generated: false
sources:
  - path: crates/cognitive-secret/src/store.rs
    symbols: ["SecretStore", "SecretRef"]
  - path: crates/cognitive-secret/src/backend_select.rs
  - path: crates/cognitive-secret/src/provider_service.rs
    symbols: ["ProviderKeyService"]
  - path: crates/cognitive-secret/src/provider_transport.rs
    symbols: ["ProviderHttpRequest"]
  - path: apps/kernel-server/src/personal/provider_proxy.rs
tests:
  - crates/cognitive-secret/tests/p1_t02_provider_secret.rs
  - crates/cognitive-secret/tests/p1_t03_provider_discovery.rs
  - apps/kernel-server/tests/p1_t07_provider_proxy.rs
fingerprint: "sha256:444eac6bfc507c208cf35852ad059a2a1290b9cca6f4c112c420ac0c0b1dc7c9"
non_claims:
  - 尽力而为的内存清零不构成侧信道或 mlock 保证。headless 加密 vault 运行仍是设计目标。Windows 后端不意味着受支持的 Windows 安装路线（B01-W 尚未执行）。
---

# Provider 与 secret

## key 存在哪里——以及绝不会出现在哪里

Provider API key 在 `cognitive init` 期间经隐藏输入或 stdin 进入，**只**存储在批准
的 OS secret store：Linux Secret Service（经 `secret-tool`、会话 D-Bus），或在
Windows 主机上的 Windows Credential Manager（经固定且经审计的 PowerShell helper、
从绝对系统路径调用；secret 材料只经 helper 的 stdin/stdout 传递、持久化仅限本机、
blob 上限 2560 字节）。配置只保留不透明引用（`SecretRef`），绝无材料本体。强制禁
区——进程参数、普通配置、SQLite、日志、CI/测试输出、证据、Pi 进程环境——均有聚焦测试
与源码扫描覆盖。

后端选择基于探测且 fail-closed：在其他平台（今天的 macOS），或密钥环/凭据库锁定、不
可用时，一切 secret 操作拒绝执行；有意不提供明文回退。轮换：
`cognitive init --rotate-key`。

## Provider 流量如何流动

客户端从不直连 Provider。egress 由 daemon 独占：

1. `POST /provider/v1/chat/completions`（management 通道）按 `provider.json` 与
   `selected-model.json` 校验请求——拒绝流式与模型不符。
2. daemon 在内存中解析 `SecretRef` 并附加 bearer 头。
3. `RustlsProviderTransport` 强制 HTTPS-only、禁跳转、禁 URL user-info、拒绝头部
   CR/LF、1 MiB 响应上限与调用方超时。
4. 成功的代理响应携带 `X-CognitiveOS-Provider-Network-Nanos` 头（仅 daemon 测得的
   Provider 网络耗时）。客户端可为自身测量记录发送不透明的
   `x-cognitiveos-correlation-id` 请求头；daemon 忽略且绝不持久化它。

发现流程（`cognitive init`）探测 `GET /models` 及 chat/stream/tool/cancel 战役，持久
化带身份 digest 的非 secret 能力快照；selected model 必须匹配该快照。

## 诚实的限制

- readiness 投影检查的是配置/后端存在性，而非实时 Provider 往返——`ready` 不证明你
  的 key 当前有效。
- `secret-tool` 探测无法区分集合是否解锁；锁定的密钥环会在首次真实使用时表现为不可用。
- 轮换先清除旧条目再存新条目；两步之间崩溃需要重新录入 key。
