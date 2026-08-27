---
doc_id: ref.capability-status
locale: zh-CN
kind: reference
audience: [user, developer, ai]
status: implemented
generated: false
sources:
  - path: personal/apps/kernel-server/src/personal/server.rs
  - path: personal/apps/admin-cli/src/personal_cli/mod.rs
  - path: personal/crates/cognitive-store/src/personal_backup.rs
  - path: personal/apps/kernel-server/src/personal/scheduler_authority/dispatch.rs
  - path: personal/crates/cognitive-secret/src/backend_select.rs
  - path: personal/apps/kernel-server/src/personal/tool_executor/mod.rs
  - path: personal/crates/cognitive-management/src/task_application.rs
  - path: personal/apps/kernel-server/src/personal/capability_truth.rs
    symbols: ["FROZEN_UJ_CAPABILITY_TRUTH", "validate_capability_truth_matrix"]
  - path: personal/docs/product/personal-2.0-scope.md
  - path: personal/docs/product/account-hub.md
  - path: personal/docs/product/account-hub.zh-CN.md
  - path: personal/docs/product/agent-integration-and-conversations.md
  - path: personal/docs/product/agent-integration-and-conversations.zh-CN.md
  - path: personal/docs/product/mcp-resource-family.md
  - path: personal/docs/product/mcp-resource-family.zh-CN.md
  - path: docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md
  - path: docs/adr/0056-personal-2-0-desktop-control-plane.md
  - path: docs/adr/0057-personal-2-0-mcp-resource-family.md
fingerprint: "sha256:b6528d3add7eb3a54cdcbb14b2567282e3b0c81584d1048aaf78511ee17a37d6"
non_claims:
  - 状态是记录基线上代码+合同+测试的联合判断，不是 Gate/release/Profile 结论，也不是正式计划的任务状态。
---

# 能力状态矩阵

图例：`implemented`（真实路径 + 测试）、`partial`（可用但有具名缺口）、
`designed`（仅合同/设计）、`unavailable`（无可用路径）、`Requires-backend`
（已采纳目标但缺所需 daemon/API 实现）与 `Requires-core`（还需要批准的 core
合同/权威工作）。

未带 Personal 2.0 限定的行描述当前 Linux/当前 API 基线：六个资源族，Pi 是唯一已
资格化 Agent。同源 `/ui/` 已存在于 `clients/pc/web/`；已采纳的桌面优先重设计是另一
个尚未实现的目标。

| 能力 | 状态 | 缺口（如有） |
|---|---|---|
| Linux bundle 安装/升级/回滚/卸载 | implemented | 生产签名/发布待办 |
| systemd 用户服务 + 健康门激活 | implemented | — |
| `cognitive init`（布局、secret、发现、选型） | implemented | — |
| daemon loopback HTTP + 通道认证 + 界限 | implemented | token 使用 OS CSPRNG；session 为进程内状态，daemon 重启会失效，且无 logout/introspection 路由 |
| Provider 代理（一元对话 + 公开 SSE） | implemented | Pi/private-candidate 保持一元；无 disconnect-to-cancel |
| Provider Control Plane（命名账户、binding、用量） | partial | daemon API + `cognitive` CLI + 当前同源 `/ui/`；usage/audit 查询无过滤器；目标 Account Hub 重设计未实现 |
| SecretStore | implemented（Linux Secret Service；Windows Credential Manager） | headless vault 为 designed；macOS 不可用 |
| Account Hub 用户定向凭据导入 | Requires-backend | ADR-0055 定义精确来源同意、daemon-only 读写、默认保留与显式删除；具体浏览器/Agent/订阅/OAuth 导入机制不存在 |
| dsh runtime inspect | implemented | `/proc` 存活仅 Linux；Windows 报告 unknown 而非 CRASHED |
| 经 daemon 的 Pi 对话 | implemented | 单发、仅文本 |
| Pi shell 内工具使用 | unavailable | 策略拒绝全部内置工具 |
| Task record/interpret/preview/admit | implemented | — |
| Task watch | implemented | 进程本地事件源 |
| HTTP 上的 Task control/query | unavailable | 服务方法存在、无路由 |
| 自主调度循环 | partial | 公开 admit 在发布 runnable 行、`START` Loop 与硬 Budget 的同时持久化 owner-local Context 授权事实与租户 `personal` 撤销 epoch；首个调度 tick 用封存 ContextView 把 Loop 从 `START` 走到 `DECIDE` 再准入一条私有 Pi candidate；后续 tick 获取 lease 并激活 Task；启动修复缺失成员；唯一绑定后非重入周期 worker 可到达 candidate 准入并从生产派发 WorkspaceRead、WorkspaceSearch、WorkspaceWrite/Patch、ProcessCheck、HttpFetchReadOnly 与仅含 `check_id` 的 RegisteredCheckRun；RegisteredCheck 收口 Task 上的中间 mutation Effect 闭合后 Loop 回到 `DECIDE`，以便后续 tick 准入 RegisteredCheckRun |
| 受治理工具执行（当前全部七个原生 Tool 操作族） | partial | 七个 Tool 操作族都有生产请求载体；WorkspaceRead、WorkspaceSearch 与 WorkspaceWrite/Patch 经周期调用者派发；ProcessCheck 在受监督进程 registry 接线前经 fail-closed 载体 staging；HttpFetchReadOnly 经评测授权的钉住 HTTPS 登记表 staging（默认为空）；RegisteredCheckRun 经不可变目录仅凭 `check_id` 派发，禁用后从 Agent 暴露中去掉 |
| workspace write/patch 执行器 | implemented，生产调用 | Linux/Windows 已测试句柄相对 no-follow 遍历/发布、有界 preimage、逐目标锁 CAS、workspace 外持久原键 receipt 与重启 orphan 恢复；payload + 期望 preimage 由持久 Intent 携带；`digest:sha256:<文件原始 SHA-256>` 与带域前缀的 workspace-image digest 等价为 CAS 令牌；Effect 仍在对账中时不请求独立 verification |
| 独立验证与 Task 验收 | implemented；公共 C1 native-proven | 生产 WorkspaceRead 与 RegisteredCheckRun 可到达登记的独立 verifier；RegisteredCheck 只有在 CAS Evidence、精确 descriptor/file digest 与全部安全观察通过后才产生 passed report、checkpoint、一次性 continuation authority 与 Loop `OBSERVE`；WorkspaceRead 再经独立 daemon acceptance authority 完成 evidence-bound `COMPLETED` |
| Memory remember/forget/检索/版本 | implemented | 无自动收割 |
| Skill import/bind/revoke/explain | implemented | 脚本绝不执行 |
| 受治理 Memory/Skill Context 消费 | implemented | 精确 scope/pin/digest 装载、v24 持久记录、第二会话复用，以及 forget/revoke 失败闭合；公开 HTTP 生命周期循环仍单独交付 |
| Context request/view + 缓存 | implemented | O2/O3/O4/O5/O13 有界观测平面为 task 通道只读；空 collector 返回具名 negative control 而不是沉默的 0；O13 审计回放在过期游标或 digest 断裂时失败闭合 |
| Artifact CAS | implemented | GC 推迟（仅清理遗弃 staging） |
| 当前六族资源投影/watch | implemented | 仅 management+task 通道；不是已采纳的 MCP 第七族 |
| Agent 生命周期（Pi 获取→sidecar） | implemented | — |
| 非 Pi agent | designed | 仅 Codex fixture 资格化 |
| 厂商专用 Agent 对话适配器 | Requires-backend | 当前 Pi/dsh 路径不提供通用桌面对话适配层；只有 Pi 已资格化 |
| 既有 MCP Tool transport + 有界 dynamic-Tool MVP | 在其已接受 P5-T03/P5-T04 范围内 implemented | interop 产出 Tool candidate；没有 Personal 2.0 server/package/connection/binding/health/quarantine 资源族生命周期 |
| Personal 2.0 MCP 第七族 + 联邦资源 | Requires-backend / Requires-core | ADR-0057 已采纳；当前没有资源族 API、持久化、trust policy、联邦投影或目标 UI |
| Goal + 不可变 Plan revision + 多 Agent 监督 | Requires-backend / Requires-core | 仅已采纳目标；当前持久工作对象仍是 Task，且没有 Goal/Plan/supervisor API |
| 管理回退动词 | implemented | R0/R2/R3 审批流 partial |
| 备份/恢复命令 | partial | 排除 secret/bearer/provider-config/authority SQLite；Memory/Skill 为 digest 绑定 sidecar；公开 `admin-cli` 覆盖 Pi install→recover |
| 当前 Web UI / Console | partial | daemon 同源提供的 `/ui/` 已存在于 `clients/pc/web/`；HTTP Task/Agent 控制仍缺失，Personal 2.0 桌面优先重设计未实现 |
| Windows/macOS 产品 | unavailable | 仅 Linux x86_64；Windows 安装模板与凭据后端已成稿并过 CI，但 B01-W 安装战役未执行 |
| 性能 campaign 工具 | implemented | 结果是计划中的 non-claim 记录 |
| UJ1–UJ6 capability-truth 登记 | implemented | 冻结 public-caller/oracle/cleanup/evidence 行；Web UI/Multi-Agent 为 scope-excluded，不得阻塞 required arm；linux-002 命名 oracle 是产品证据，不是 EVAL/Gate |

逐行细节与来源：见 [`_meta/source-map.json`](../../_meta/source-map.json) 所列
user/developer 页面。
