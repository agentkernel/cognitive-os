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
  - path: docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md
  - path: personal/docs/architecture/windows-host-background.md
  - path: personal/crates/cognitive-store/src/windows_host.rs
    symbols: ["WINDOWS_HOST_SCHEMA_V34", "WindowsHostStore", "WAKE_RECOVERY_STEPS"]
  - path: personal/docs/architecture/x-twitter-connector.md
  - path: personal/crates/cognitive-store/src/x_connector.rs
    symbols: ["X_CONNECTOR_SCHEMA_V35", "XConnectorStore"]
fingerprint: "sha256:3f2efaf4a89b32f77db5e17a825d6d80ffdb0426a658ea890256952296f21683"
non_claims:
  - 状态是记录基线上代码+合同+测试的联合判断，不是 Gate/release/Profile 结论，也不是正式计划的任务状态。
---

# 能力状态矩阵

图例：`implemented`（真实路径 + 测试）、`partial`（可用但有具名缺口）、
`designed`（仅合同/设计）、`unavailable`（无可用路径）、`Requires-backend`
（已采纳目标但缺所需 daemon/API 实现）、`Requires-environment`（缺 qualified
native/campaign environment）与 `Requires-core`（还需要批准的 core 合同/权威工作）。

未带 Personal 2.0 限定的行描述当前 Linux/当前 API 基线：六个资源族，Pi 是唯一已
资格化 Agent。同源 `/ui/` 已存在于 `clients/pc/web/`；已采纳的桌面优先重设计是另一
个尚未实现的目标。Personal 2.0 是完整版本承诺，但每个缺失行仍为 `Requires-backend`，
且每个平台与 Agent 都需独立资格化。

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
| Personal 2.0 Windows OPC 产品 | Requires-backend + Requires-environment | Today/Projects/Knowledge（Team 与 Inbox 不是一级导航）；Dual Track L1 在 `P12-T01`–`T09` 后是 daemon `/ui/` 上的 **Now / hypothesis chrome**（merged PR [#302](https://github.com/agentkernel/cognitive-os/pull/302)）；canvas v9 是冻结设计原型，不是产品；NVDA/200%/host-theme `not-run`；**Phase 13**（`P13-T01`–`T13`，2026-09-02 登记）承接 walking skeleton → 原型程度 + 设计目标；**Phase 14**（`P14-T01`–`T08`）承接 EVAL-016 之后的 live `/ui/` 残差（`JOURNEY-BROWSER-SYNC-01`）；`P11-T15` 为 Phase 13 验收出口；单模块目录 [`00-maintenance-index.md`](../../../../clients/docs/design/opc-2.0/00-maintenance-index.md)；Linux/WSL/CI/Canvas evidence 不转移 |
| Project/Charter/Goal/Plan/Routine/Task/Attempt | Requires-backend | current Task authority 可复用，但 Project activation、manager envelope、Routine/missed ledger 与完整 hierarchy 不存在 |
| Role Blueprint/Assignment/Digital Employee | Requires-backend | 没有完整 authority/projection；employee identity 必须与 runtime/process 分离 |
| Pi-backed Personal Assistant | Requires-backend | Pi 是 hidden、candidate-only target engine；current Pi Shell/Linux qualification 不构成 OPC Assistant |
| 隐藏托管 DSH 真实 Attempt 循环 | partial（实现已存在）+ Requires-environment | v36 + management `dsh.hosted.attempt.run` / `attempt.list` / `attempt.detail` / `artifact.check` / `artifact.facts` 已证明 persist-before-dispatch Intent、经 daemon stdio broker 真实 spawn exact-artifact 子进程（stdin 递交有界 Context、白名单 env、仅 Path B）、candidates/observations 只追加台账、daemon 写入且永不为 `success` 的终态、`completion_claimed=false`、崩溃 → `unknown-outcome`，以及 artifact health/update/rollback 事实。不是 Installed Agent chrome，不是原生 DSH UI，不是 Pi 当 Member 引擎。Linux 真实 spawn 只是实现证据；Windows sandbox / ACL / supply-chain E2E 在 `P13-T13` 前为 `not-run`。所产文本的独立验证属于 `P13-T04`。 |
| Attempt 产物 → CAS → 独立 verifier → 末环验收 → 发布预览 | partial（实现已存在）+ Requires-environment | v37 + management `outputs` / `outputs.detail` / `outputs.open` / `outputs.export` / `attempt.artifact.verify` / `attempt.artifact.stage-test` / `run.acceptance.request` / `run.acceptance` / `publication.packet` / `publication.external-send.request` / `publication.sends`（P13-T04）。终态托管 Attempt 的 `DeliverableDraft` 候选带 digest / 格式 / 来源帧 / 新鲜度进入唯一的 P3-T03 CAS；独立 verifier `verifier://personal/attempt-artifact` 重读 CAS 字节并追加 evidence（报告放在同一 CAS）；StageTestPassed 由该 evidence + 真实就位 + CAS 重读推导（无调用方 `passed`）；run 验收是 `run-acceptance` ApprovalPreview，不在末环即拒绝；发布包是 `planned: true` / `published: false` 的 AUTONOMY 发布包；external send 是 `external-send` ApprovalPreview，确认只记 `planned` Intent——`published` 不可表示。模型文本、`response done`、exit 0、HTTP 回执与文件永远不是完成。宿主打开文件 E2E 在 `P13-T13` 前为 `not-run`。 |
| Preinstalled managed DSH Installed Agent | Requires-backend + Requires-environment | existing dsh Path B 不是 exact Windows artifact/isolated child/sandbox/update/rollback qualification；没有 native DSH UI/conversation target |
| Personal Conversation archive/index/retrieval | Requires-backend | Personal-owned scoped archive 与 single composer 不存在；不得重解释 ADR-0058 `conversation-projection/0.1` |
| Knowledge/Markdown Vault/episodic retrieval | Requires-backend | 没有 OPC Personal Home/import/OCR/index/Vault/conflict/Obsidian companion 产品路径 |
| Semantic Memory privacy/correct/forget integration | Requires-backend | current Memory admission/forget 已有，但 Conversation/Vault extraction/retrieval 与 privacy matrix 不存在 |
| Routine/Trigger/Inbox/offline-missed recovery | partial（实现已存在）+ Requires-environment | v33 + management `routine.*` 已证明 no-overlap/queue-latest 与可见 missed ledger；v38 + `routine.arm` / `routine.instruction` / `routine.runs` / `today.overview`（P13-T05）在 G2 后武装 Routine，并让 daemon scheduler tick——`task://personal/routine/*` 行的唯一派发者——触发 schedule、租约每个 active occurrence、驱动一个托管 Attempt（P13-T02 路径）并把观察到的终态写回为 occurrence 结果（`attempted`，永无 `success`；`completion_claimed=false`）；P11-T02 宿主 paused / offline 时 schedule 触发落为可见 `missed` 行；新指令在安全点应用（`continue` / `pause` / `restart`）且不触碰运行中的 Attempt。Dual Track L1 chrome 已在 `main`（`P11-T13`）；Inbox 一级仍缺（HITL 是 T09 画布）；clock/sleep/restart 宿主 E2E 在 `P13-T13` 前为 `not-run`；Attempt 产出的独立验证归 `P13-T04`。 |
| Windows host/tray/background（隐藏） | partial（walking skeleton）+ Requires-environment | v34 + management `host.*` 已证明 Personal Home `app/`/`data/`、close 诚实性、missed 时段与七步有序恢复。不是 chrome。不是第二套凭据平面。原生 install/tray/ACL/sleep/SecretStore E2E 在 `DEV-WINDOWS-NATIVE-OPC-01` 资格化前为 `not-run`。 |
| Provider global→Project→employee→Task binding 与 hard budget | Requires-backend | current fixed Agent binding 与 advisory budget 保持 partial；DSH/Pi 必须经 no-raw-secret daemon proxy |
| X/Twitter connector（隐藏） | partial（walking skeleton）+ Requires-environment | v35 + management `connector/x.*` 已证明 SecretStore-only bind、原创 digest 绑定 preview、HITL confirm、persist-before-dispatch 与诚实 unknown readback。不是 P0 hero。不是业务结果。禁止 evasion。live X API / CAPTCHA / platform qualification 为 `not-run`。 |
| 既有 MCP Tool transport + 有界 dynamic-Tool MVP | 在其已接受 P5-T03/P5-T04 范围内 implemented | interop 产出 Tool candidate；没有 Personal 2.0 server/package/connection/binding/health/quarantine 资源族生命周期 |
| Personal MCP 第七族 | deferred / Requires-backend | ADR-0057/0058 retained advanced private target；不是 OPC P0，无 current family API，DSH native MCP/base tools 继续禁用 |
| Windows OPC fixed-denominator acceptance | Requires-environment / not-run | unparked N=15、同一 qualified Windows revision；15 个场景草案已由 `P13-T01` 写在 plan.md T15 卡（领取时冻结）；验收前置 = `P13-T02`–`T13` done + `P13-T13` 资格化 `DEV-WINDOWS-NATIVE-OPC-01`；不是 Phase 12 prototype completeness mutex；required CI/Canvas 不执行；signing/B01-W/release 分离 |
| 管理回退动词 | implemented | R0/R2/R3 审批流 partial |
| 备份/恢复命令 | partial | 排除 secret/bearer/provider-config/authority SQLite；Memory/Skill 为 digest 绑定 sidecar；公开 `admin-cli` 覆盖 Pi install→recover |
| 当前 Web UI / Console | partial | daemon 同源 `/ui/` Dual Track L1（Today/Projects/Knowledge + Settings + 右栏）位于 `clients/pc/web/`；空 Home 只创建（`#/projects/new` 五段向导）已在 `main`（`P12-T02`）；Project 四子菜单（`#/projects/:id` 及 members/runs/outputs）已在 `main`（`P12-T03`）；先选后配 + 加成员已在 `main`（`P12-T04`）；Today 决策包（未验收只继续创建 / 已上线才 pending-previews）已在 `main`（`P12-T05`）；HITL 画布 Confirm（`P12-T06`）已在 `main`；Knowledge ingest（`P12-T07`）已在 `main`；Settings 连接表（`P12-T08`）已在 `main`；右栏写画布（`P12-T09`）已在 `main`（merged PR [#302](https://github.com/agentkernel/cognitive-os/pull/302)）；Dual Track **Now / hypothesis chrome**；Linux 1.0 六族仍在高级/二级；NVDA/200%/host-theme `not-run`；不是 Windows OPC |
| 当前 Windows 产品 | unavailable | installer/credential fragment 与 ordinary CI 不是 Windows OPC host/DSH/UI support；qualified native environment 与 B01-W 均不存在 |
| Personal 2.1 native mobile/E2E relay remote | deferred | 仅 host-online；device-bound key/revocation/short session/preview/audit/no secret downlink 为 future controls |
| 性能 campaign 工具 | implemented | 结果是计划中的 non-claim 记录 |
| UJ1–UJ6 capability-truth 登记 | implemented | 冻结 public-caller/oracle/cleanup/evidence 行；Web UI/Multi-Agent 为 scope-excluded，不得阻塞 required arm；linux-002 命名 oracle 是产品证据，不是 EVAL/Gate |

逐行细节与来源：见 [`_meta/source-map.json`](../../_meta/source-map.json) 所列
user/developer 页面。
