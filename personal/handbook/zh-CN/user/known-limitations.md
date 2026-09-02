---
doc_id: user.limitations
locale: zh-CN
kind: reference
audience: [user]
status: implemented
generated: false
sources:
  - path: personal/apps/kernel-server/src/personal/server.rs
  - path: personal/apps/kernel-server/src/personal/auth.rs
  - path: personal/crates/cognitive-store/src/personal_backup.rs
  - path: personal/apps/admin-cli/src/personal_cli/mod.rs
  - path: docs/adr/0053-personal-web-ui-stack.md
  - path: docs/adr/0054-repository-subproject-structure-and-1.0.0-finalization.md
  - path: docs/adr/0055-personal-credential-import-boundary-and-a5-revision.md
  - path: docs/adr/0056-personal-2-0-desktop-control-plane.md
  - path: docs/adr/0057-personal-2-0-mcp-resource-family.md
  - path: docs/adr/0059-personal-2-0-opc-project-runtime-and-memory-boundary.md
tests:
  - personal/apps/kernel-server/tests/p2_t18_local_token_csprng.rs
  - personal/apps/admin-cli/tests/p2_t32_public_daemon_start_scheduler.rs
fingerprint: "sha256:cbd883404855f4b3aef7ac9edbbe8a5391ff23aa44f07000b25e517897d48dc8"
non_claims:
  - 本清单对应记录的阅读基线；后续合并可能增减真实限制——指纹检查会标记过期。
---

# 已知限制

一份诚实且经核验的清单。此处的 "implemented" 指该限制本身是代码的当前事实。

## 功能

- **自主执行未端到端接线**：Task 准入会入列完整调度引导，绑定后的周期 worker 可到
  candidate 准入。无参数 WorkspaceRead 现有持久生产 Effect 调用者、independent
  verifier 与 evidence-bound Task acceptance caller。exact native `22c3f502`
  到达公共 C1 `COMPLETED`。open Effect、被取代 report 与缺失 CAS 负例已写入；
  stale fixed post-state 仍开放。其余 Tool 请求载体仍未接线。
- **备份/恢复排除 secret 与 authority SQLite**；`cognitive backup` / `restore` 与
  management HTTP 路由写入 digest 绑定归档，并在预检后覆盖 live 文件。Provider
  key 留在 Secret Store，换机后需重新录入。managed Pi recover 尚未接在这条路径上。
- **Control Plane Web UI 不在 Linux RC 声明内**：
  [ADR-0053](../../../../docs/adr/0053-personal-web-ui-stack.md) 已接受 React +
  TypeScript + Vite 与 daemon 同源 `GET /ui` 静态服务。
  [ADR-0054](../../../../docs/adr/0054-repository-subproject-structure-and-1.0.0-finalization.md)
  之后 SPA 位于本仓库 `clients/pc/web/`，产品路径是复制到 `data_dir()/ui`。daemon
  执行 loopback Origin/Referer 允许列表；当 `data_dir()/ui/index.html` 不存在时返回
  `503` `not_available`。HTTP cancel 与 class-C Agent 生命周期仍为 `not-run`。无
  Windows/macOS 安装产品，也无多 agent 编排。Pi shell 尚无资源/任务浏览 UX。已采纳
  的 Personal 2.0 桌面优先重设计尚未应用到这个当前 SPA。操作步骤见
  [Provider Control Plane](provider-control-plane.md)。Linux RC 声明集见
  [Linux RC 操作地图](rc-and-support.md)。
- **Personal 2.0 OPC 能力不是 current**：Windows host/tray/background、
  Project/Role/Employee/Routine/Attempt authority、Personal Conversation
  archive/Vault/retrieval、Pi-backed Assistant、managed DSH artifact/child/sandbox、
  上下文关注（Inbox 不是一级导航）、binding/budget enforcement、OPC UI 与 X connector 都是
  `Requires-backend`/`Requires-environment`。
- **Installed Agent target 很窄**：DSH 是唯一 2.0 runtime qualification target。
  Existing dsh Path B 不证明 Windows managed artifact；Pi 是 hidden Assistant target；
  Hermes/Codex/Cursor 是 future candidates。产品不做 native DSH UI/conversation sync。
- **固定验收不是用户或 release evidence**：unparked 的 Phase 11 T15 使用 N=15 Windows OPC
  scenarios，**不是** prototype completeness mutex，目前一个也未执行。Canvas/ordinary CI 不证明 human desirability、
  usability、adoption、WTP、support、release/Gate readiness 或 Agent benefit。
- **冻结 prototype `/ui/` 完备是单独计划阶段**：daemon `/ui/` 上默认可走场景是 Phase 12 卡。
  不是 canvas 像素复制，不是 2.1，不是 T15。Dual Track：无权威则 empty / Requires-backend；
  0 假 Create/Activate/Approve。
- **Phase 12 收口不等于成员真的会干活**：截至 2026-09-02，托管 DSH 只有 start 骨架、
  隐藏 Pi 助手不调用 Pi、`runs`/`outputs` 只显示流程轴、Settings 连接仍指路旧
  `/providers`、Memory 纠正/遗忘无 OPC 表面、没有视觉规格、Windows 原生环境未
  provisioned。这些缺口由正式计划 **Phase 13**（`P13-T01`–`T13`）逐卡承接；Phase 13
  done 也不是 release / signing / B01-W。
- 预算告警只观察/查询，不阻断也不改路 Provider 调用。
- 自定义端点只允许 OpenAI 兼容；第三方 Anthropic 兼容 URL 被拒绝。`cognitive usage
  query` 与 `cognitive audit query` 无过滤器；用量 JSON 含带标签事件（`cost` /
  `cost_label` actual|estimated|unknown，绝不为 `0`）、`binding_explanation` 层，
  以及分开的 `account` 与 `quota` 对象。
- Pi 对话按次单发（无流式、仅文本、客户端固定 8192/1024 窗口常量）。
- `TaskApplicationService` 已实现 `control`/`query_intent`，但尚无 HTTP 路由暴露。

## 运维怪癖

- 未知 `/task/*` 路径返回 HTTP 200 加注记而非 404。
- `cognitive` usage 文本漏列已实现的 `resource`/`task` 动词；
  `admin-cli install --mode official` 的 usage 漏写必需的 `--package-id`。
- 单独运行 `kernel-server --personal` 默认临时端口（`127.0.0.1:0`）；canonical 的
  `48181` 来自 `cognitive daemon start`。
- `cognitive doctor` 的 `first_conversation_ready` 是对话壳就绪，不是 C1/C2 Task
  生命周期；已准入 Task 在调度器拿到 lease 之前可以停在 `DRAFT`。CLI
  `cognitive daemon start` 把 kernel-server stdio 留在 `state/cognitiveos/daemon.log`，
  不再丢到 `/dev/null`。
- Provider key 过期时 readiness 仍可能显示 `ready`（无实时探测）。
- 重启/替换 daemon 会使 dsh Path B 的进程内 management session 失效。新 daemon
  将 dsh 投影为 `INACTIVE`，所以 `cognitive dsh apply` 会被拒绝，不能恢复 stale
  bearer。不要提取 bearer 做直接探测；必须重启 `cognitive dsh web`，再检查
  `cognitive dsh status`。`apply` 只用于 daemon 未重启且 runtime 已为 `ACTIVE` 时所
  支持的 binding/model overlay 同步。持久化账户 `active` 不是实时 SecretStore 解析
  结果；discovery/proxy 使用时才实时解析，锁定或变化的 store 仍是独立可能原因。这是
  带运维恢复、尚无产品代码修复的
  [open 正式登记缺陷](../../../../docs/bug/dsh-pathb-stale-daemon-bearer-after-daemon-restart.md)。
- `state/backups/` 下的迁移备份只增不清。
- 迁移中崩溃可能留下过期 `migration.lock`，需人工移除。
- `pnpm run verify:local` 钉住过期符合性计数（过期的开发者入口）。

## 平台

- 产品平台：Linux x86_64 + user systemd；桌面需要 Secret Service 密钥环。WSL2 是工
  程环境，不是产品目标。
- Personal 2.0 是 Windows-first，但 qualified native Windows dev environment 与
  B01-W 都不存在。Linux、WSL、ordinary CI、Canvas 与 Windows GNU evidence 不转移。
- native mobile/device pairing/E2E relay remote deferred 到 Personal 2.1，且仅
  host-online；不下发 Secret 原文。
- headless 加密 vault 运行已设计但今天不可选。
- Windows：daemon/CLI 在 CI 可编译，Credential Manager 后端与安装器/scheduled-task
  模板已存在，但 B01-W 安装战役未执行——没有可安装的 Windows 产品，本地文件也无
  ACL 加固。本地 bootstrap/session token 已使用 OS CSPRNG；该修正不增强 Windows 文件
  ACL。
- 升级后若 runtime 仍留有 CSPRNG 修正前的 bootstrap 形状，daemon 会有意拒绝启动。
  停止 daemon，并只删除其私有 runtime 目录中的 `local-bootstrap.secret`，下次启动即可
  签发替代凭据。
