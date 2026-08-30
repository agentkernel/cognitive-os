---
doc_id: user.provider-control-plane
locale: zh-CN
kind: guide
audience: [user]
status: partial
generated: false
sources:
  - path: personal/apps/admin-cli/src/personal_cli/provider.rs
    symbols: ["parse_provider_args", "parse_agent_args", "CONTROL_PLANE_FLAGS"]
  - path: personal/apps/admin-cli/src/personal_cli/mod.rs
    symbols: ["COGNITIVE_USAGE"]
  - path: personal/apps/admin-cli/src/personal_cli/secret_input.rs
    symbols: ["read_api_key_material"]
  - path: personal/apps/kernel-server/src/personal/provider_control_plane.rs
    symbols: ["PI_AGENT", "DSH_AGENT", "set_binding", "query_usage"]
  - path: personal/apps/kernel-server/src/personal/provider_proxy.rs
    symbols: ["BindingMismatch"]
  - path: personal/crates/cognitive-secret/src/endpoint_trust.rs
    symbols: ["TrustedEndpoint", "ProviderKind"]
  - path: personal/crates/cognitive-store/src/provider_control_plane.rs
    symbols: ["USAGE_EVENT_RETENTION_MS", "USAGE_AGGREGATE_RETENTION_MS", "honest_usage_read_model", "labelled_cost_source"]
  - path: personal/docs/product/provider-control-plane.md
  - path: personal/docs/product/account-hub.md
  - path: personal/docs/product/account-hub.zh-CN.md
  - path: docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md
  - path: docs/adr/0056-personal-2-0-desktop-control-plane.md
tests:
  - personal/apps/kernel-server/tests/p8_t13_provider_control_plane.rs
  - personal/crates/cognitive-secret/tests/p8_t13_endpoint_trust.rs
  - personal/crates/cognitive-store/tests/p8_t13_provider_store.rs
  - personal/crates/cognitive-store/tests/p11_t12_honest_usage.rs
  - personal/apps/admin-cli/src/personal_cli/mod.rs
fingerprint: "sha256:d87eae91c8bf8e2e5291fa03ac783a7f18e33a84eba0e95abf99bee7455902b4"
non_claims:
  - 本页记录已交付的 daemon API、cognitive CLI 与当前 localhost Web UI 路径。不声称 live Secret Store 证明、live Provider/Pi/dsh 资格化、Gate、release、Profile、B01、Personal 2.0 桌面重设计/Account Hub 导入或 Agent-benefit。
---

# Provider Control Plane

`partial`：下面的 daemon management API 与 `cognitive` CLI 调用方已经实现，并有聚焦
测试覆盖。localhost-only Web UI 位于本仓库 `clients/pc/web/`，由 `GET /ui/` 同源
提供，是同一套 management 路由的 daemon 客户端：命名账户、SecretStore 密钥交接、
有界探测以及固定 Agent binding。已采纳的桌面优先 Personal 2.0 重设计**尚未实现**。
原生 dsh 控制面板
（`cognitive dsh web`，默认 `http://127.0.0.1:3080`）是独立的 dsh 自带 UI，
不是本 Provider Control Plane 面，也不是 Personal `/ui/`。当 store 或上游不可用时，
live Secret Store 轮换/删除与 live Provider/Pi/dsh 资格化仍然失败闭合；它们不是
Gate 证明。

精确动词文本也出现在生成的 [CLI 参考](../reference/cli-cognitive.md)。`cognitive
init` 已经使用的 Secret Store 机制见
[Provider 与 secret](provider-and-secrets.md)。

## 它是什么

Provider Control Plane 是 owner-local 的管理方式：管理**命名 LLM 账户**、把 API
key 只存进批准的 OS Secret Store、维护模型目录、把 Pi agent 与 DeepSeek harness
（`dsh`）各自绑定到一组固定的 account+provider+model，并查询用量、仅观察的预算、
告警与脱敏审计日志。

daemon 是唯一 writer。CLI 与 localhost Web UI 都是非权威客户端：它们从不打开
SQLite 或 Secret Store。它们向 daemon 发送 management 通道 HTTP。浏览器不得在
提交后把 API key 留在 DOM、URL 或 Web storage；SecretRef 只显示 present/absent。
这些路由的 task 通道副本返回 HTTP 403 `PROVIDER_CONTROL_CHANNEL_FORBIDDEN`。

该平面**不**取代 `cognitive init`。首次对话设置仍写入 `provider.json` /
`selected-model.json`。未绑定的 agent 仍使用那一对文件。一旦你为 `pi` 或 `dsh`
设置了 control-plane binding，该 binding 就是该 agent 唯一允许的 account+model——
`provider.json` 不是回退。

当前 API 未交付：OAuth、浏览器/Agent 凭据导入、自动路由或负载均衡、硬预算阻断、
第三方 Anthropic 兼容端点、后台模型刷新，以及已采纳的桌面优先 Account Hub 重设计。
前两项与重设计是标记为 `Requires-backend` 的 Personal 2.0 目标，不是当前功能。
Web UI 不发明 Task cancel 或 Agent pause/resume/stop/restart/quarantine HTTP。

## Personal 2.0 Account Hub 目标（`Requires-backend`）

目标 Account Hub 把当前命名 API-key 账户扩展为一个桌面入口，用于 Provider 账户、
订阅、模型访问与已安装 Agent binding。厂商专用对话适配器将连接每个 Agent 真正支持的
对话行为；当前 API 仍只绑定已交付的 `pi` 与 `dsh` 身份，且只有 Pi 已资格化。

凭据导入严格遵守 ADR-0055：用户发起并在读取前逐个同意精确来源；只有 daemon 读取该
来源并写入 approved SecretStore；默认保留来源，安全删除是每次导入的显式选择。原始
材料绝不返回 UI 或 Agent，也绝不进入 argv、环境变量、CognitiveOS 普通配置、SQLite、
日志、证据、支持输出或聊天。浏览器 profile、Agent 凭据文件、订阅与 OAuth 的导入机制
当前均不存在。见 [Provider 与 secret](provider-and-secrets.md)。

## 前置条件

1. 正在运行的 Personal daemon（`cognitive daemon start`，默认回环
   `127.0.0.1:48181`）。
2. 可用的 management 会话。CLI 从本机 bootstrap secret 取得会话；不要在命令行粘贴
   bearer。
3. 批准的 Secret Store：Linux Secret Service（`secret-tool`、会话 D-Bus），或在
   Windows 主机上的 Windows Credential Manager。没有明文回退。macOS 以及锁定或缺失
   的密钥环会失败闭合。
4. 永远不要把 Provider key 放进进程参数、普通配置、SQLite、环境变量、服务单元、
   日志、证据或聊天。

下列每个动词还接受 `--runtime-root <dir>`（hermetic 测试布局逃生口）和
`--endpoint <host:port>`（daemon 地址，**不是** Provider URL）。Provider URL 标志是
`--endpoint-url`。

退出码：`0` 成功、`1` 运行错误、`2` 用法错误。成功输出为 JSON。CLI 在打印前脱敏
`sk-`、`Bearer `、`bearer ` 与 `x-api-key` 片段。

## 账户

`--name` 是**显示名**（ASCII 字母、数字、连字符、下划线；最长 64；唯一）。持久
账户 id 生成为 `acct-<uuid>`，后续命令的 `--id` 使用它。

Provider 种类：

| `--provider-kind` | 端点 | 线上认证 |
|---|---|---|
| `openai_official` | 始终 `https://api.openai.com/v1` | `Authorization: Bearer` |
| `anthropic_official` | 始终 `https://api.anthropic.com` | `x-api-key` 加 `anthropic-version` |
| `openai_compatible` | 必须传 `--endpoint-url` | 仅 `Authorization: Bearer` |

官方端点不可变。对官方种类传入不同的 `--endpoint-url` 会以
`PROVIDER_ENDPOINT_OFFICIAL_IMMUTABLE` 失败。主机为 `api.anthropic.com` 的自定义端点
被拒绝（`PROVIDER_ENDPOINT_ANTHROPIC_COMPATIBLE_FORBIDDEN`）。调用方不能注入
`headers` 或 `authorization` 字段。

不带 key 创建会使账户处于 `revoked`（不可调用）。带 `--api-key-file` 创建会把 key
存入 Secret Store，然后做一次前台模型发现。发现失败会使账户变为 `degraded`，保留
既有目录与 binding，并记录 `last_discovery_error`。

```text
cognitive provider account create --name openai-work --provider-kind openai_official --api-key-file ./provider.key

cognitive provider account create --name lan-proxy --provider-kind openai_compatible --endpoint-url https://llm.internal.example/v1 --allow-private-network --api-key-file ./provider.key

cognitive provider account create --name xai-grok --provider-kind openai_compatible --endpoint-url https://api.x.ai/v1 --api-key-file ./provider.key

cognitive provider account list
cognitive provider account show --id acct-YOUR-ID
cognitive provider account update --id acct-YOUR-ID --endpoint-url http://127.0.0.1:8080/v1 --allow-insecure-http --reconfirm
cognitive provider account delete --id acct-YOUR-ID
```

`--api-key-file -` 从 stdin 读取 key（不回显）。在 Unix 上省略该标志会使用隐藏 TTY
输入。无法关闭回显的主机上，CLI 失败闭合并要求你传 `--api-key-file`。不接受
`--api-key`。

当仍有**活动** agent binding 指向该账户时，删除会以 `PROVIDER_CONTROL_CONFLICT`
失败。先移除 binding。

list/show 投影包含 `id`、`display_name`、`provider_kind`、`endpoint`、不透明
`secret_ref`、`status`（`active` / `degraded` / `revoked`）、`catalog_revision`、
`last_discovery_error`、信任标志与 `network_scope`。它们从不包含 API key。

## 密钥

```text
cognitive provider key set --id acct-YOUR-ID --api-key-file ./provider.key
cognitive provider key rotate --id acct-YOUR-ID --api-key-file ./provider.key
cognitive provider key remove --id acct-YOUR-ID
```

set 与 rotate 经回环 HTTP 发送一次 key。daemon 把它放入 Secret Store，只持久化不透明
`secret_ref`。rotate 在新 put 之后删除先前的 Secret Store 条目。remove 尽力删除
store 条目并把账户标为 `revoked`。

缺失或不可用的 Secret Store 返回 `PROVIDER_SECRET_STORE_UNAVAILABLE`（HTTP 503）。
remove 之后，发现与绑定调用会失败，直到你再次 set key。

## 端点信任

自定义 OpenAI 兼容 URL 在公网 HTTPS 上不需要额外标志。更窄或更明文的目标必须带持久
的账户级授权：

- `--allow-private-network` —— 回环、LAN 或其他私网段（以及解析进这些范围的 DNS
  结果）。
- `--allow-insecure-http` —— 使用 `http://` 而非 HTTPS。

daemon 拒绝嵌入的 userinfo、fragment、query、重定向、调用方提供的 header 模板，以及
隐式 URL 改写。请求时会再次检查 DNS（若名称现在指向比授权更私密的地址，则为
`PROVIDER_ENDPOINT_DNS_REBINDING`）。

自定义端点只持久化 OpenAI 兼容的 **API 根**：空/`/`、`/v1`、`/api/v1`、`/openai/v1`
或 `/compatible-mode/v1`。控制面板若粘贴 chat 或 models 的 RPC URL（例如
`https://api.x.ai/v1/chat/completions`），会存成根路径（`https://api.x.ai/v1`）。
其它路径——包括本机 daemon 代理 `/provider/v1/...`——返回 HTTP 400
`PROVIDER_ENDPOINT_PATH_FORBIDDEN`。

当 authority、DNS/网络范围或 HTTPS→HTTP 将要变化时，`account update` 必须带
`--reconfirm`。否则 daemon 返回 HTTP 409 `PROVIDER_ENDPOINT_RECONFIRM_REQUIRED`。

没有 `--allow-insecure-http` 的 HTTP 是
`PROVIDER_ENDPOINT_HTTP_REQUIRES_GRANT`。没有 `--allow-private-network` 的私网/回环
是 `PROVIDER_ENDPOINT_PRIVATE_REQUIRES_GRANT`。

## 模型

带 key 的账户创建以及 `key set` / `key rotate` 都会运行**一次**前台发现。没有后台
刷新。

- 官方 OpenAI 与 OpenAI 兼容：`GET` `{endpoint}/models`。
- 官方 Anthropic：`GET` `{endpoint}/v1/models`。

```text
cognitive provider models refresh --account-id acct-YOUR-ID
cognitive provider models list --account-id acct-YOUR-ID
cognitive provider models add --account-id acct-YOUR-ID --model-id my-local-model --price-input-per-million 1.00 --price-output-per-million 2.00
cognitive provider models set-price --account-id acct-YOUR-ID --model-id my-local-model --pricing-version manual --price-input-per-million 1.00 --price-output-per-million 2.00
```

`refresh` 与 `list` 使用 `--account-id`。失败的 refresh 会记入审计，返回
`PROVIDER_DISCOVERY_FAILED` 或 `PROVIDER_DISCOVERY_MALFORMED`（HTTP 502），把状态设为
`degraded`，并**保留**上次目录与任何 binding。

目录 `source` 为 `provider_discovered` 或 `manually_configured`。`add` 插入手动模型
（在绑定提供方未列出的模型之前必须这样做）。`set-price` 更新价格；若省略
`--pricing-version`，CLI 发送 `manual`。价格标志是每百万 token 的十进制美元：
`--price-input-per-million`、`--price-output-per-million`、
`--price-cache-read-per-million`、`--price-cache-write-per-million`。

少数官方模型 id 带有内置版本化价格表。自定义与手动模型在你定价之前没有价格。缺价
是 `cost_unavailable`——不是零成本。

## Agent binding

两个已交付 agent 各自最多一个活动 binding：固定的 account + provider + model。请求
不能另选模型。没有回退，也不跨 agent 共享 Pi 证据。

CLI `--agent` 取值：`pi` 或 `dsh`（daemon 存储 `agent://personal/pi` 与
`agent://personal/dsh`）。

```text
cognitive agent binding set --agent pi --account-id acct-YOUR-ID --model-id gpt-4o
cognitive agent binding set --agent dsh --account-id acct-YOUR-ID --model-id deepseek-chat
cognitive agent binding list
cognitive agent binding show --agent pi
cognitive agent binding remove --agent pi
```

除非该模型已在账户目录中（发现它或 `models add`），`set` 会以
`PROVIDER_MODEL_NOT_FOUND` 失败。仅在目录中还不够：DeepSeek 主机只服务
`deepseek-*`，`grok-*` 只能绑到非 DeepSeek 的 `openai_compatible` 账户（例如 xAI）。
否则 `models add` 与 `binding set` 以 `PROVIDER_MODEL_ENDPOINT_MISMATCH` 失败闭合。
HTTP `POST /management/agent-bindings` 接受可选
整数 `expected_revision`（当前 binding revision，未绑定时为 `0`）。不匹配时 HTTP
409 `PROVIDER_BINDING_REVISION_STALE`。未带 `expected_revision` 就改账户或模型是
HTTP 409 `PROVIDER_SILENT_REBIND_REJECTED`——先 `remove` 再 `set`，或提交匹配的
`expected_revision`。CLI `binding set` 不发送该字段，因此切换账户/模型走
`remove` 再 `set`。同一账户+模型刷新仍成功。`show` 在解析时
要求 `--agent`，但当前调用与 `list` 相同的列表端点（不过滤）。用 `list` 查看两个
binding。

Pi 流量使用 `POST /provider/v1/chat/completions`。DeepSeek harness 流量使用独立的
`POST /provider/v1/dsh/chat/completions` 路由。已绑定的 Pi 私有 candidate 调用也使用
binding 而不是 `provider.json`。若 **Pi** 请求的 `model` 与 Pi binding 不符，代理以 HTTP 400
`PERSONAL_PROVIDER_BINDING_MISMATCH` 失败闭合。**dsh** Path B 代理会把请求模型改写为
Cos `agent://personal/dsh` binding，因此原生目录 id 仍能用 Cos 指定的模型、走
**绑定账户** 对话。若该账户不能服务绑定模型（grok 绑在 `api.deepseek.com`），
Path B 以 HTTP 400 `PERSONAL_PROVIDER_BINDING_MISMATCH` 失败闭合，不会向 DeepSeek
发请求。设置、移除或改 dsh 绑定账户目录会按**当前 dsh 绑定账户**写入原生 Models
覆盖层。Personal `/ui/` Bindings 的 **Apply to running dsh**（`POST
/personal/dsh/runtime` `op=apply`）会重发 selected-model 并重载 Cos 安装的 web，
使该列表与控制面一致；解绑 dsh 会从原生 Models 去掉 grok（以及该账户的其他 id）。
已吊销账户或缺失 key 是 HTTP 409
`PERSONAL_PROVIDER_ACCOUNT_UNAVAILABLE`。官方 Anthropic binding 不支持公共 SSE
（`stream:true`）。

Pi 与 `dsh` binding 相互隔离：设置一个绝不会复制另一个。

## 用量、成本与审计

绑定代理调用会持久化一条用量事件（不含 prompt、completion、key 或可逆载荷）。账本
上的 token 类别是 input、output、cache-read 与 cache-write。缺失字段保持**未知**；
绝不会存成 `0`。账本上的 `metering_source` 在 input 与 output token 计数都存在时为
`provider_reported`，否则为 `unavailable`。

只有当每个已出现的 token 类别都有价格时，成本才是 `priced`；否则 `cost_status` 为
`cost_unavailable`，且省略 `cost_micros`——那不是零账单。仅当提供方分母语义已知时才
派生缓存命中率；否则账本保留原始缓存计数与未知命中率。

```text
cognitive usage query
cognitive audit query
```

这两个 CLI 动词在本阶段**没有过滤器**（设计文本里提到的时间范围/账户过滤不是已交付
标志）。`usage query` 转储 `GET /management/usage`：每条事件含 `event_id`、
`account_id`、`cost`（`unknown` 或非零微美元数）、`cost_label`
（`actual` | `estimated` | `unknown`）、`cost_micros`、`cost_status` 与
`metering_source`。未知费用绝不为 JSON `0` 或 `"0"`。只有确实记录了
`locally_estimated` 时才标为 `estimated`。同一响应含
`binding_explanation.layers`，顺序为 global → Project → employee → Task
（缺失层为 `unbound`，不编造零），以及 `accounts[]` 中分开的 `account` 与
`quota` 对象（在真实配额源出现前 `quota.status` 为 `unknown`）。不返回
secret。`audit query` 返回 `audit_id`、`action`、`outcome` 与脱敏
`detail`。每次调用的用量事件保留 **30 天**；聚合保留 **90 天**。查询用量会运行该
清理。
清理。

## 预算与告警

预算按日历月、仅观察。它们从不阻断、限流或改路 Provider 调用。范围是 `account`
（账户 id）或 `agent`（使用存储的 agent id，例如 `agent://personal/pi`，以便与用量
行匹配）。

```text
cognitive budget set --scope-kind account --scope-id acct-YOUR-ID --token-limit 2000000 --amount-micros-limit 10000000
cognitive budget list
cognitive budget remove --budget-id bud-YOUR-ID
cognitive alerts list
cognitive alerts acknowledge --alert-id YOUR-ALERT-ID
```

`--amount-micros-limit` 是整数 **micro-USD**（1 USD = 1_000_000）。set 上的
`--budget-id` 可选（省略时 daemon 生成 `bud-<timestamp>`）。remove 使用
`--budget-id`，不是 `--id`。

每个周期在 token 或金额限额的 80% 最多发出一条 `warning_80`，在 100% 最多发出一条
`exceeded_100`，按预算去重。不可用成本不按零支出处理。`alerts list` 在读取时可能
签发刚越过的阈值。确认使用 `--alert-id`。

## 安全

- 信任标志与 DNS 钉扎用于约束 SSRF。除非你确实打算让该账户到达那个网络与协议，否则
  不要授予 `--allow-private-network` 或 `--allow-insecure-http`。
- prompt、completion、key 与请求头不会保留在用量、审计或 CLI 输出中。
- management 变更需要 management 通道。未认证调用会失败（daemon 前门通常为 HTTP
  401）。
- 保留：已交付的 30 天事件 / 90 天聚合。

## 常见失败

| 你看到的 | 该做什么 |
|---|---|
| CLI `--api-key is not accepted` | 使用 `--api-key-file` 或 `--api-key-file -`。 |
| `cognitive provider` 动词上的 HTTP 401 | daemon 未运行、没有 management 会话，或 bootstrap secret 缺失。启动 daemon；不要把 key 放进 argv。 |
| HTTP 403 `PROVIDER_CONTROL_CHANNEL_FORBIDDEN` | 这些操作仅限 management。使用 `cognitive` 产品 CLI，不要用 task bearer。 |
| `PROVIDER_SECRET_STORE_UNAVAILABLE` | 解锁或安装 OS Secret Store。没有文件/环境回退。 |
| `PROVIDER_DISCOVERY_FAILED` / 详情 `upstream 401` 或 `upstream 403` | key 或账户权益不对。轮换 key；不要把它粘贴进日志。先前目录仍保留。 |
| `PROVIDER_DISCOVERY_FAILED`（传输）或 `PROVIDER_DISCOVERY_MALFORMED` | 网络、TLS 或意外的 `/models` JSON。账户保持 `degraded`；binding 保留。 |
| `PROVIDER_KEY_MISSING` | 在 refresh 或绑定调用前 set key。 |
| `PROVIDER_MODEL_NOT_FOUND` | 在 `agent binding set` 之前 `models refresh` 或 `models add`。 |
| `PROVIDER_MODEL_ENDPOINT_MISMATCH` | 只把 grok 绑到能服务 grok 的非 DeepSeek `openai_compatible` 账户。不要把 grok 加进 DeepSeek 目录。 |
| `PERSONAL_PROVIDER_BINDING_MISMATCH` | 请求模型不是绑定模型。更改 binding 或发送已绑定 id。没有回退。 |
| HTTP 409 `PROVIDER_BINDING_REVISION_STALE` | 重新读取 binding revision，再用确认过的 `expected_revision` 重试。 |
| 删除时的 `PROVIDER_CONTROL_CONFLICT` | 先 `agent binding remove`。 |
| `PROVIDER_ENDPOINT_RECONFIRM_REQUIRED` | 若你确实要新的主机、协议或范围，带 `--reconfirm` 重跑 `account update`。 |
| `PROVIDER_ENDPOINT_HTTP_REQUIRES_GRANT` / `PROVIDER_ENDPOINT_PRIVATE_REQUIRES_GRANT` | 在 create 或 update 时传入匹配的 `--allow-*` 标志（需要时带 `--reconfirm`）。 |
| `cost_status: cost_unavailable` | 为自定义/手动模型定价。不要把缺失成本当成 `$0`。 |
| 官方 Anthropic + `stream:true` | 绑定路径不支持。Pi 无论是否绑定都保持 unary。 |
| daemon 重启后 dsh 面板报 “API key invalid”，但账户/binding 显示持久化 `active` 状态 | 新 daemon 把 dsh 报为 `INACTIVE`，因此 `cognitive dsh apply` 会被拒绝，不能恢复 stale session。不要提取或直接探测 bearer。必须重启 `cognitive dsh web`，再检查 `cognitive dsh status`。`apply` 只用于 daemon 未重启且 runtime 已为 `ACTIVE` 时所支持的 binding/model overlay 同步。持久化账户 `active` 不证明当前 SecretStore 可解析；实时解析发生在 discovery/proxy 使用期间，锁定或变化的 store 仍是独立可能原因。见[正式登记缺陷](../../../../docs/bug/dsh-pathb-stale-daemon-bearer-after-daemon-restart.md)。 |

## 操作序列（官方 OpenAI，然后绑定 Pi）

```text
cognitive daemon start
cognitive provider account create --name openai-work --provider-kind openai_official --api-key-file ./provider.key
cognitive provider account list
cognitive provider models list --account-id acct-YOUR-ID
cognitive agent binding set --agent pi --account-id acct-YOUR-ID --model-id gpt-4o
cognitive agent binding list
cognitive usage query
cognitive budget set --scope-kind account --scope-id acct-YOUR-ID --token-limit 2000000
cognitive alerts list
cognitive audit query
```

用 create/list 返回的 `id` 替换 `acct-YOUR-ID`。保持 `./provider.key` 权限为 `0600`
且不进 Git。该序列在发现或真实调用成功之前，不证明上游 key 有效，也不完成 Task。
