---
doc_id: dev.daemon-http-surface
locale: zh-CN
kind: concept
audience: [developer]
status: implemented
generated: false
sources:
  - path: personal/apps/kernel-server/src/personal/server.rs
    symbols: ["serve_personal_loopback", "PersonalDaemonConfig"]
  - path: personal/apps/kernel-server/src/personal/auth.rs
    symbols: ["LocalSessionAuthority", "ChannelClass"]
  - path: personal/apps/kernel-server/src/personal/bounds.rs
  - path: personal/apps/kernel-server/src/personal/readiness.rs
    symbols: ["evaluate_personal_readiness"]
  - path: personal/apps/kernel-server/src/personal/provider_proxy.rs
  - path: personal/apps/kernel-server/src/personal/route_observation.rs
    symbols: ["observation_response_headers"]
  - path: personal/apps/kernel-server/src/personal/fault_profile.rs
    symbols: ["handle"]
  - path: personal/apps/kernel-server/src/personal/tool_lifecycle.rs
    symbols: ["handle"]
  - path: personal/apps/kernel-server/src/personal/pinned_https.rs
    symbols: ["handle"]
  - path: personal/apps/kernel-server/src/personal/observation.rs
    symbols: ["handle"]
  - path: personal/apps/kernel-server/src/personal/user_backup.rs
    symbols: ["handle"]
  - path: personal/apps/kernel-server/src/personal/resource_manager.rs
    symbols: ["handle", "matches"]
  - path: personal/apps/kernel-server/src/personal/provider_control_plane.rs
    symbols: ["handle", "matches"]
  - path: personal/apps/kernel-server/src/personal/project_aggregate.rs
    symbols: ["handle", "matches"]
  - path: personal/apps/kernel-server/src/personal/windows_host.rs
    symbols: ["handle", "matches"]
  - path: personal/apps/kernel-server/src/personal/x_connector.rs
    symbols: ["handle", "matches"]
  - path: personal/apps/kernel-server/src/personal/hosted_dsh_attempt.rs
    symbols: ["handle", "matches", "HostedAttemptHost"]
  - path: personal/crates/cognitive-store/src/hosted_dsh.rs
    symbols: ["HostedDshPlane", "HostedDshStartSpec", "HOSTED_DSH_ENGINE_ID"]
  - path: personal/crates/cognitive-store/src/hosted_dsh_attempt.rs
    symbols: ["HostedDshAttemptStore", "HostedAttemptIntentSpec", "HostedAttemptTerminalSpec"]
  - path: personal/crates/cognitive-runtime/src/hosted_dsh_broker.rs
    symbols: ["run_hosted_child", "validate_launch_plan", "HostedDshArtifact", "HostedContextPayload"]
  - path: personal/apps/kernel-server/src/personal/task_api.rs
    symbols: ["TaskApi"]
tests:
  - personal/apps/kernel-server/tests/p1_t04_personal_daemon.rs
  - personal/apps/kernel-server/tests/p1_t05_personal_readiness.rs
  - personal/apps/kernel-server/tests/p1_t07_provider_proxy.rs
  - personal/apps/kernel-server/tests/p9_t07_route_observation.rs
  - personal/apps/kernel-server/tests/p2_t24_effect_fault.rs
  - personal/apps/kernel-server/tests/p2_t25_tool_lifecycle.rs
  - personal/apps/kernel-server/tests/p2_t26_observation_plane.rs
  - personal/apps/kernel-server/tests/p2_t27_backup_restore.rs
  - personal/apps/kernel-server/tests/p2_t31_live_daemon_scheduler.rs
  - personal/apps/admin-cli/tests/p2_t32_public_daemon_start_scheduler.rs
  - personal/apps/admin-cli/tests/p2_t33_private_candidate_host_path.rs
  - personal/apps/kernel-server/tests/p8_t12_resource_manager.rs
  - personal/apps/kernel-server/tests/p8_t13_provider_control_plane.rs
  - personal/crates/cognitive-store/tests/p11_t07_hosted_dsh.rs
  - personal/crates/cognitive-store/tests/p13_t02_hosted_dsh_attempt.rs
  - personal/crates/cognitive-runtime/tests/p13_t02_hosted_dsh_broker.rs
  - personal/crates/cognitive-store/tests/p11_t02_windows_host.rs
  - personal/crates/cognitive-store/tests/p11_t14_x_connector.rs
fingerprint: "sha256:7a293332ba147e328d9995caa0d6ca45d859722b76ef419f464a4f02ca77e97d"
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
续 pass 观察。公开 `POST /task/admit` 会为租户 `personal` 持久化 owner-local Context
授权，使后续 pass 能解析 Context 而不是在 Pi 之前跳过。HTTP `TaskApi` 克隆同一
`SqliteAuthorityStore` 句柄（共享连接互斥），而不是每次请求再开一个 writer，因此周期
tick 能看到 admit 刚写入的事实。该 worker 独占调度连接，在非重入门后按固定延迟 250 ms 串行运行；pass
级错误只记录并重试，逐行错误仍在单趟内隔离。顺序退出时会显式取消、唤醒并 join
worker。仍没有 HTTP shutdown 路由（见[执行链状态](execution-chain-status.md)）。
`cognitive daemon start` 把该进程的 stdout/stderr 追加到 `state/cognitiveos/daemon.log`
（权限 `0600`）；systemd `Type=simple` 仍走 journal。

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
连接上限、拒绝 Cookie、Host 校验，以及 ADR-0053 Origin/Referer 允许列表（当请求携带
的 Origin 或 Referer 不是本 daemon 的 loopback HTTP origin 时返回
`LOCAL_ORIGIN_HEADER_REJECTED`；缺少 Origin 仍允许 CLI/curl）——各配稳定错误码。
`GET /ui` 从 `data_dir()/ui` 提供钉住的静态 bundle，并带 CSP `default-src 'self'`；
缺少 bundle 时为 `503` `not_available`（`LOCAL_UI_BUNDLE_UNAVAILABLE`），不构成
readiness 声明。路由是对 `METHOD /path` 字符
串的手写前缀匹配，分布在 `server.rs`、`task_api.rs`、`resource_api.rs`、
`resource_manager.rs`、`project_aggregate.rs`（生成的
[HTTP 参考](../reference/http-api.md)枚举完整表与通道）。已认证的
`POST /task/akp/dsh` 是仅 candidate 的 DeepSeek Harness 前门：会话只存在于进程内，
必须在启动后显式激活；daemon 重启后会话被遗忘并失败闭合。Workspace* candidate 复用
既有 public candidate admission。dsh 响应绝不完成 Task。

management Resource 表面提供只读生命周期前置条件、Memory remember/recall/correct/forget/index.rebuild，
Skill import/inspect/bind/supersede/revoke，以及通用 Resource Manager 信封
（`GET/POST /management/resource/v1/{list,inspect,bind,unbind,enable,disable,revoke}`）。
generic create/install/execute/complete、task 通道 Memory 别名
（`/task/resource/v1/memory/*`）与 task 通道上的相同 Resource Manager 路径失败闭合。watch 仍走
`GET /resource/v1/watch`。公开 remember 接受未封存的 owner
字段，由 daemon 用持久 `GovernanceSeed` 组合封存 header；带封存
source+candidate 的信封仍然有效。未封存路径上调用方不得自行铸造 header。
变更必须持有 management bearer；task bearer 在进入 handler 前失败。创建成功使用
HTTP 状态 `201`，持久行在重启后仍可检查。
task 通道通过 `GET /task/resource/v1/consumption?task_ref=…` 读取 daemon 写入的
最新 Memory/Skill 消费记录：只返回精确钉、session 关联和 `reuse_of`。
`query_text` 与 `skill_binding_id` 视为用户重述并被拒绝。遗忘、撤销或 digest
漂移的钉在响应前失败闭合，且绝不返回 Memory/Skill 正文。session 2 与重启后的
GET 读取同一持久行；带调用方 `query_text` 的 POST 不能替换这些钉。

management Provider Control Plane 路由（`/management/providers/*`、
`/management/agent-bindings`、`/management/usage`、`/management/budgets`、
`/management/alerts`、`/management/audit`）需要 management bearer。task 通道别名
失败闭合（`PROVIDER_CONTROL_CHANNEL_FORBIDDEN`）。命名账户只持久化不透明的
Secret Store `secret_ref`。已绑定 Pi 流量走 `POST /provider/v1/chat/completions`；
已绑定 DeepSeek harness 走 `POST /provider/v1/dsh/chat/completions`。未绑定 agent
仍使用 `provider.json`。private-candidate 补全在存在 Pi binding 时使用同一 binding，
模型不符失败闭合；Pi 从不读取 Secret Store。`POST /management/agent-bindings`
接受可选 `expected_revision`；不匹配时 HTTP 409
`PROVIDER_BINDING_REVISION_STALE`。未带该字段就改账户或模型是 HTTP 409
`PROVIDER_SILENT_REBIND_REJECTED`。`GET /management/usage` 返回带标签费用
（`actual` | `estimated` | `unknown`；unknown 序列化绝不为 `0`）、四层
binding 说明（缺失的 Project/employee/Task 层为 `unbound`），以及省略 secret
的账户与配额对象。localhost Web UI 是同源 daemon 客户端
（`GET /ui/`），不是第二个 writer。

Personal-private Project 聚合路由（`/management/project/v1/{list,detail,axis,roster,employee.catalog,pending-previews,preview-detail,draft.apply,preview.request,preview.reject,preview.narrow,confirm,standing-policies,standing-policy.create,standing-policy.revoke,roster.register,employee.seat.request,employee.seat.confirm,employee.runtime.bind,speech.candidate,conversation.append,conversation.archive,conversation.record,handoff.record,assistant.turn,dsh.hosted.start,dsh.hosted.observe-exit,vault.import,vault.index.rebuild,vault.index,vault.conflicts,vault.apply-authority,routine.revision,routine.trigger,routine.ledger,routine.checkpoint,routine.resume}`）需要 management bearer。它们投影 v26 `p11_*` Project 表、v27 Employee/Blueprint/Assignment/Grant 表、v28 `p11_conversation_archive`（标识 `cognitiveos.personal.conversation-archive/0.1`，不重解释 ADR-0058 `conversation-projection/0.1`）、v29 ApprovalPreview `superseded_by`（HITL reject/narrow）、v30 `grant-expansion` 与 StandingApprovalPolicy 时间盒（`expires_at` 必填、≤7 天；Settings 列表/撤销），v31 隐藏托管 DSH 子进程身份（`p11_hosted_dsh_child`；`dsh.hosted.start` 把 `runtime_binding_ref` 绑到 `hosted-dsh:<digest>:<child_id>`；Windows GNU isolated spawn 失败闭合；Windows OPC E2E 为 `not-run`；不是 Installed Agent chrome；Pi 不是 Member 执行引擎），v32 Markdown Vault（`p11_vault_document` / 可重建 `p11_vault_index_entry` / `p11_vault_conflict`，标识 `cognitiveos.personal.markdown-vault/0.1`；文件不是 Project 权威；Memory FTS 不是 Vault 索引；无冲突记录的 last-write-wins 被拒绝；宿主文件系统 E2E 为 `not-run`），以及 v33 Routine/Trigger（`p11_routine` / `p11_routine_revision` / `p11_routine_occurrence`，标识 `cognitiveos.personal.routine/0.1`；no-overlap-queue-latest；missed/coalesced 可见；复用 `scheduler_entries`；checkpoint 不是完成；无 Temporal；clock/sleep/restart E2E 为 `not-run`），不是 Task 行冒充，也不动 P7-T05 冻结 inventory。空列表无假按钮；未知费用字面量是 `unknown`，序列化里不出现 `0`。空花名册使用 `authority_note: empty-roster`；已就位成员按 `employee_id` 列出。Blueprint 行无 Provider binding。白名单投递发言落档案行；owner `conversation.append` 写入 `note` 等档案 kind；chatter 仅审计。档案索引用 `limit` 1..=32 且只返回引用；`include_bodies` 与缺省 limit 失败闭合。单条正文走 `conversation.record`。档案行只是观察，不是完成。HITL confirm/reject/narrow 与 standing-policy 签发/撤销仅限 management；聊天/task 别名失败闭合（`PROJECT_AGGREGATE_CHANNEL_FORBIDDEN`），不能完成批准。stale 只按机械 `base_state_digest` 不等判定，不是墙钟新鲜度。`preview.request` 返回供画布使用的 `preview_digest`。这不是 Today 页，不是 Inbox 一级，也不是完整 `/ui/` IA。

Windows host 隐藏能力路由（`/management/host/v1/{home.admit,daemon.bind,close.request,offline.record,dsh.bind,recovery.run,recovery.advance,restore-point.record}` 与 `GET /management/host/v1/status`）需要 management bearer。它们持久化 v34 Personal Home `app/`+`data/`、daemon bind、孤儿 DSH 拒绝、close background-or-pause 诚实性、可见 offline/missed 时段、七步有序 wake/restart，以及不是备份的 restore point。task 通道别名失败闭合（`WINDOWS_HOST_CHANNEL_FORBIDDEN`）。托盘只观察与请求，不写权威。原生 tray/ACL/sleep/SecretStore E2E 在 `DEV-WINDOWS-NATIVE-OPC-01` 资格化前为 `not-run`。

X/Twitter connector walking skeleton 路由（`/management/connector/x/v1/{account.bind,preview.request,preview.confirm,publish.dispatch}` 与 `GET /management/connector/x/v1/status`）需要 management bearer。它们持久化 v35 SecretStore-only bind、digest 绑定的原创 preview、HITL confirm、persist-before-dispatch 发布与诚实 `unknown` readback。task 通道别名失败闭合（`X_CONNECTOR_CHANNEL_FORBIDDEN`）。status 不返回 `secret_ref`。不是 P0 hero chrome。不是业务结果。live X API E2E 为 `not-run`。

托管 DSH 真实 Attempt 路由（`POST /management/project/v1/dsh.hosted.attempt.run`、`GET …/dsh.hosted.attempt.list`、`GET …/dsh.hosted.attempt.detail`、`POST …/dsh.hosted.artifact.check`、`GET …/dsh.hosted.artifact.facts`；P13-T02）需要 management bearer，并在 Project 聚合匹配器之前分派。`attempt.run` 先从 `dsh.json` + 钉住文件 + 子脚本 digest 记一条 v36 artifact 事实（非 `pinned` 即 `HOSTED_ARTIFACT_UNHEALTHY` 422，不 spawn），再持久化 Attempt Intent、绑定 v31 子进程身份，然后由 daemon 线程运行 `cognitive-runtime` stdio broker：`env_clear` + 白名单环境、argv 只含路径与 pin、有界 Context（≤64 KiB，secret 形状被拒）作为一条 `request` 帧写入子进程 stdin（带 loopback daemon origin 与 bootstrap 文件*路径*），在墙钟超时与字节/帧上限下读回逐行 JSON 帧，Unix 上独立进程组使超时能连带杀掉 dsh 孙进程。每一帧都是观察；`provider_request`、非 loopback URL、`task_complete` / `effect` / `authority` 帧与无 operation 的 candidate 被拒并记录；自由文本与 `{"status":"success"}` 计为未知行。终态行由 daemon 写入（`exited` / `signaled` / `timed-out` / `spawn-failed`，永无 `success`；`completion_claimed=false`；`verification_status=not-run`）并清除子进程 pid；spawn 前的拒绝落为 durable `spawn-failed` 终态（`HOSTED_ATTEMPT_SPAWN_REFUSED` 422）。启动时把崩溃形状的行 reconcile 为 `unknown-outcome`。task 通道别名失败闭合（`HOSTED_ATTEMPT_CHANNEL_FORBIDDEN`）。仍带 session/bootstrap/`sk-` 形状的响应失败闭合（`HOSTED_ATTEMPT_REDACTION`）。Linux 真实 spawn 只是实现证据；Windows sandbox / ACL / supply-chain E2E 在 P13-T13 前为 `not-run`。

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

management 的 `GET/POST /management/resource/v1/http-origin` 在授权 campaign
（`P2-T25` 或 `PERSONAL-PERF-EVAL-*`）下为一个 `task_ref` 钉住精确 HTTPS origin
（`host` 或 `host:port`）。默认白名单为空，因此生产 HttpFetchReadOnly 在有钉之前
staging 失败闭合。钉只承认 GET/HEAD：无凭据、不跟随重定向、不继承代理、无请求体。
普通 task 调用方被拒绝（`RESOURCE_PINNED_HTTPS_CHANNEL_FORBIDDEN`）。禁用
`native.registered-check.run` 会把它从 Agent 暴露中去掉，且不发明 ProcessRun 族。

task 通道通过 `GET /task/observation?family=o2|o3|o4|o5|o13&task_ref=…`（别名
`GET /task/resource/v1/observation`）读取有界 O2/O3/O4/O5/O13 观测。空 collector 返回带具名
negative control 的 `observed_zero`，而不是沉默的默认 0。prompt、body、receipt 与
capability 查询键失败闭合。O5 复用 `GET /task/effects` 已提供的脱敏 Intent/Effect
历史，仍不暴露原始参数或 receipt。O13 导出持久审计游标、事件 digest 链与有界回放；
过期游标、缺失事件、digest 断裂或序列缺口失败闭合。management 调用方被拒绝
（`RESOURCE_OBSERVATION_CHANNEL_FORBIDDEN`）：这是只读平面，不是第二套 authority
API。样本不含 Context body 或 capability 材料。

management 的 `POST /management/resource/v1/backup` 写入排除 secret 的目录归档；
`POST .../backup/preflight` 校验一个 `archive_id` 且不改写；`POST .../restore`
在快照后覆盖 live 文件，失败则回滚。归档永不复制 authority SQLite、bootstrap
secret、bearer 或 `provider-config.json`。task 通道别名返回 403
`RESOURCE_BACKUP_CHANNEL_FORBIDDEN`。

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
Rustls 传输转发。一元代理成功响应携带 `X-CognitiveOS-Provider-Network-Nanos`。
公开 management `stream:true` 按 HTTP/1.1 SSE 转发，不等待最后一个事件；流式成功
省略该网络耗时头，因为 SSE 响应头会先刷新。仅 dsh 路由会在
OpenAI-compatible `tool_calls` delta 中移除值为 `null` 的 continuation 字段，避免上游
continuation 覆盖起始调用的 id 或 name；所有其他 SSE payload（包括错误和 usage frame）
保持逐字节透传。嵌套 preflight 计时与 correlation 回显默认拒绝，仅当
`COGNITIVEOS_PI_ROUTE_OBSERVATION=enabled` 且请求携带一条形态正确的不透明
correlation id 时才发出；畸形或重复的 id 被忽略，产品 body 不变，观测器不写任何
东西。一次性私有 Unix socket（`POST /chat/completions`）只服务 daemon 启
动的 Pi candidate 进程且禁止 Authorization 头。该私有 candidate 路径在转发前剥离
`tools`/`tool_choice`，接受可含 `role=assistant` 以及 `finish_reason` 等额外
choice 字段的单条文本 choice，并拒绝 `tool_calls` / `function_call`。

## 非 Personal 骨架

`kernel-server --once/--serve` 是 M0 时代的 AKP/shell HTTP 骨架（占位语义、错误也返
回 HTTP 200）。它不是 Personal 表面；将其视为 SDK live 测试使用的历史脚手架。
