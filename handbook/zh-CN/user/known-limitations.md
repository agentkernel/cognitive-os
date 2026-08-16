---
doc_id: user.limitations
locale: zh-CN
kind: reference
audience: [user]
status: implemented
generated: false
sources:
  - path: apps/kernel-server/src/personal/server.rs
  - path: apps/kernel-server/src/personal/auth.rs
  - path: crates/cognitive-store/src/personal_backup.rs
  - path: apps/admin-cli/src/personal_cli/mod.rs
tests:
  - apps/kernel-server/tests/p2_t18_local_token_csprng.rs
fingerprint: "sha256:3b02e623743069f429947bf52b1f787f18f8ad3745cb1a50be64e2629b48a711"
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
- **无备份/恢复命令**；仅有规划 API（secret 恒排除）。
- **无 Web UI、无 Windows/macOS 安装、无多 agent 编排**；Pi shell 尚无资源/任务浏览
  UX。
- Pi 对话按次单发（无流式、仅文本、客户端固定 8192/1024 窗口常量）。
- `TaskApplicationService` 已实现 `control`/`query_intent`，但尚无 HTTP 路由暴露。

## 运维怪癖

- 未知 `/task/*` 路径返回 HTTP 200 加注记而非 404。
- `cognitive` usage 文本漏列已实现的 `resource`/`task` 动词；
  `admin-cli install --mode official` 的 usage 漏写必需的 `--package-id`。
- 单独运行 `kernel-server --personal` 默认临时端口（`127.0.0.1:0`）；canonical 的
  `48181` 来自 `cognitive daemon start`。
- Provider key 过期时 readiness 仍可能显示 `ready`（无实时探测）。
- `state/backups/` 下的迁移备份只增不清。
- 迁移中崩溃可能留下过期 `migration.lock`，需人工移除。
- `pnpm run verify:local` 钉住过期符合性计数（过期的开发者入口）。

## 平台

- 产品平台：Linux x86_64 + user systemd；桌面需要 Secret Service 密钥环。WSL2 是工
  程环境，不是产品目标。
- headless 加密 vault 运行已设计但今天不可选。
- Windows：daemon/CLI 在 CI 可编译，Credential Manager 后端与安装器/scheduled-task
  模板已存在，但 B01-W 安装战役未执行——没有可安装的 Windows 产品，本地文件也无
  ACL 加固。本地 bootstrap/session token 已使用 OS CSPRNG；该修正不增强 Windows 文件
  ACL。
- 升级后若 runtime 仍留有 CSPRNG 修正前的 bootstrap 形状，daemon 会有意拒绝启动。
  停止 daemon，并只删除其私有 runtime 目录中的 `local-bootstrap.secret`，下次启动即可
  签发替代凭据。
