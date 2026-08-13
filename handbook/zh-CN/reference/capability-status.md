---
doc_id: ref.capability-status
locale: zh-CN
kind: reference
audience: [user, developer, ai]
status: implemented
generated: false
sources:
  - path: apps/kernel-server/src/personal/server.rs
  - path: apps/admin-cli/src/personal_cli/mod.rs
  - path: crates/cognitive-store/src/personal_backup.rs
  - path: apps/kernel-server/src/personal/scheduler_authority/dispatch.rs
  - path: crates/cognitive-secret/src/backend_select.rs
  - path: apps/kernel-server/src/personal/tool_executor/mod.rs
  - path: crates/cognitive-management/src/task_application.rs
fingerprint: "sha256:7f0c7ba20963e2864cbcfe4907f518ed4852c56712488accd2ab9df19b32c93c"
non_claims:
  - 状态是记录基线上代码+合同+测试的联合判断，不是 Gate/release/Profile 结论，也不是正式计划的任务状态。
---

# 能力状态矩阵

图例：`implemented`（真实路径 + 测试）、`partial`（可用但有具名缺口）、
`designed`（仅合同/设计）、`unavailable`（无可用路径）。

| 能力 | 状态 | 缺口（如有） |
|---|---|---|
| Linux bundle 安装/升级/回滚/卸载 | implemented | 生产签名/发布待办 |
| systemd 用户服务 + 健康门激活 | implemented | — |
| `cognitive init`（布局、secret、发现、选型） | implemented | — |
| daemon loopback HTTP + 通道认证 + 界限 | implemented | bearer 随机源非密码学 |
| Provider 代理（非流式对话） | implemented | 不支持流式 |
| SecretStore | implemented（Linux Secret Service；Windows Credential Manager） | headless vault 为 designed；macOS 不可用 |
| 经 daemon 的 Pi 对话 | implemented | 单发、仅文本 |
| Pi shell 内工具使用 | unavailable | 策略拒绝全部内置工具 |
| Task record/interpret/preview/admit | implemented | — |
| Task watch | implemented | 进程本地事件源 |
| HTTP 上的 Task control/query | unavailable | 服务方法存在、无路由 |
| 自主调度循环 | partial | 准入原子发布当前 epoch 的 runnable 行、`START` Loop 与硬 Budget；启动修复缺失成员；唯一绑定后非重入周期 worker 可到达 candidate 准入并从生产派发 WorkspaceRead，但其余族与验证仍未接线 |
| 受治理工具执行（全部六个已登记族） | partial | 六族都有已装配 executor，投影因此报告 `execution_ready`；WorkspaceRead 现有周期生产调用者，其余五族仍缺生产请求载体且仅测试调用 |
| workspace search/write/patch 执行器 | partial | Linux/Windows 已测试句柄相对 no-follow 遍历/发布、有界枚举/preimage、逐目标锁 CAS、workspace 外持久原键 receipt 与重启 orphan 恢复；无生产调用者 |
| 独立验证循环 | partial | verifier 接缝仅测试调用 |
| Memory remember/forget/检索/版本 | implemented | 无自动收割 |
| Skill import/bind/revoke/explain | implemented | 脚本绝不执行 |
| Context request/view + 缓存 | implemented | — |
| Artifact CAS | implemented | GC 推迟（仅清理遗弃 staging） |
| 六族资源投影/watch | implemented | 仅 management+task 通道 |
| Agent 生命周期（Pi 获取→sidecar） | implemented | — |
| 非 Pi agent | designed | 仅 Codex fixture 资格化 |
| MCP 工具 | designed | post-1.0 fixture 适配器 |
| 管理回退动词 | implemented | R0/R2/R3 审批流 partial |
| 备份/恢复命令 | unavailable | 仅规划 API |
| Web UI / Console | unavailable | 外部仓库、仅设计 |
| Windows/macOS 产品 | unavailable | 仅 Linux x86_64；Windows 安装模板与凭据后端已成稿并过 CI，但 B01-W 安装战役未执行 |
| 性能 campaign 工具 | implemented | 结果是计划中的 non-claim 记录 |

逐行细节与来源：见 [`_meta/source-map.json`](../../_meta/source-map.json) 所列
user/developer 页面。
