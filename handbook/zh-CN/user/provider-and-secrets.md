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
  - path: crates/cognitive-secret/src/endpoint_trust.rs
    symbols: ["TrustedEndpoint", "ProviderKind"]
  - path: apps/kernel-server/src/personal/provider_control_plane.rs
tests:
  - crates/cognitive-secret/tests/p1_t02_provider_secret.rs
  - crates/cognitive-secret/tests/p1_t03_provider_discovery.rs
  - apps/kernel-server/tests/p1_t07_provider_proxy.rs
  - apps/kernel-server/tests/p9_t07_route_observation.rs
  - apps/kernel-server/tests/p8_t13_provider_control_plane.rs
  - crates/cognitive-secret/tests/p8_t13_endpoint_trust.rs
fingerprint: "sha256:61f5d8866329a357a2bc38ca61ba8a4ddc0acc15c776866e0a3d6842b30b5161"
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
`cognitive init --rotate-key`。命名控制面账户改用
`cognitive provider key set|rotate|remove --api-key-file`，不要把 key 放进 argv。完整
操作步骤（账户、信任标志、binding、用量、仅观察预算）见
[Provider Control Plane](./provider-control-plane.md)。localhost Web UI 是同一套
management 路由的 daemon 客户端；没有桌面面板。

## Provider 流量如何流动

客户端从不直连 Provider。egress 由 daemon 独占：

1. `POST /provider/v1/chat/completions`（management 通道，Pi 路径）校验请求。若
   `agent://personal/pi` 已有控制面 binding，请求 model 必须匹配且无回退。未绑定
   agent    仍使用 `provider.json` 与 `selected-model.json`。公开 `stream:true` 按 SSE
   转发；Pi 对话与 private-candidate 保持一元，且在存在 Pi binding 时使用该绑定账户
   而不是 `provider.json`。模型不符仍失败闭合。DeepSeek
   harness 走独立的 `POST /provider/v1/dsh/chat/completions`（`agent://personal/dsh`）。
2. daemon 在内存中解析 `SecretRef` 并附加 bearer 头。
3. `RustlsProviderTransport` 强制 HTTPS-only、禁跳转、禁 URL user-info、拒绝头部
   CR/LF、1 MiB 响应上限与调用方超时。公开 `stream:true` 直接读取 HTTP/1.1 TLS
   记录，因此首个 SSE 事件不会等到最后一个事件才下发。该传输实例上的 hermetic
   additional root 替换（而非叠加）平台 CA；生产路径的平台根只加载一次。
4. 一元代理成功响应携带 `X-CognitiveOS-Provider-Network-Nanos` 头（仅 daemon 测得的
   Provider 网络耗时）。流式成功省略该头，因为刷新 SSE 响应头时总时长未知；仍报告
   `X-CognitiveOS-Daemon-Preflight-Nanos`。客户端可发送一条不透明的 `campaign-<32 位小写 hex>`
   `x-cognitiveos-correlation-id` 请求头；daemon 绝不持久化它。当
   `COGNITIVEOS_PI_ROUTE_OBSERVATION=enabled` 且该头形态正确时，成功响应还会回显
   该 id 并报告 `X-CognitiveOS-Daemon-Preflight-Nanos`（配置/selected-model/
   SecretStore，与网络交换互不重叠）。畸形或重复的 correlation 头被忽略，产品 body
   不变。
5. 私有 Pi candidate completion 走同一 daemon 代理：转发前剥离 `tools`/`tool_choice`，
   接受可含 `role=assistant` 的单条文本 choice，并拒绝 `tool_calls`。适配器 stderr
   出现在 `daemon.log` 前会脱敏（`sk-` / `api_key=` / `token=`）。

发现流程（`cognitive init`）探测 `GET /models` 及 chat/stream/tool/cancel 战役，持久
化带身份 digest 的非 secret 能力快照；selected model 必须匹配该快照。

## 诚实的限制

- readiness 投影检查的是配置/后端存在性，而非实时 Provider 往返——`ready` 不证明你
  的 key 当前有效。
- `secret-tool` 探测无法区分集合是否解锁；锁定的密钥环会在首次真实使用时表现为不可用。
- 轮换先清除旧条目再存新条目；两步之间崩溃需要重新录入 key。
- 控制面命名账户只持久化 OpenAI 兼容 API 根，而不是 chat RPC 路径。粘贴
  `…/v1/chat/completions` 会收成 `…/v1`；其它路径以
  `PROVIDER_ENDPOINT_PATH_FORBIDDEN` 失败闭合。
