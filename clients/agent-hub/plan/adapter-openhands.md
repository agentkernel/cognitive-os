# Adapter 子车道计划 — OpenHands（AD-OPENHANDS）

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：blocked
>
> dossier：[adapters/tier1/openhands.md](../docs/adapters/tier1/openhands.md)。独立 gate：接口/账号/许可/PoC。

## 范围与路径

- 允许（激活后）：OpenHands Adapter 模块。
- 禁止：他人车道/其他 Adapter；采用 ACP 默认自动批准；复用旧 enterprise 目录（PolyForm 禁分发）。
- 依赖：CTR、HOST/PROC/CRED。gate：AH-B4、AH-B2、AH-B5（MIT/PolyForm 分界）。

## 任务

### AH-AD-OPENHANDS-01 接口一手核验
- owner/lane：Lane-CON / AD-OPENHANDS｜depends_on：AH-CTR-02｜blocked_by：AH-B4
- 交付物：Agent Server HTTP/WS + ACP + pause/resume 官方 URL/version（仓库/release 已核）；conversation list API 完整性；许可分界（Canvas/SDK MIT vs enterprise PolyForm）
- oracle：dossier 无 `待核验` 关键项｜evidence：not-run

### AH-AD-OPENHANDS-02 L1/L2 平台自有 conversation
- owner/lane：Lane-CON / AD-OPENHANDS｜depends_on：AH-AD-OPENHANDS-01,AH-PROC-02｜blocked_by：AH-B2
- 交付物：Host 启动 Agent Server/ACP 子进程；真 pause/resume；恢复平台自有 conversation（非 provider native）
- 安全负例：不把平台 conversation 恢复标为第三方 session takeover
- oracle：pause 行为一致｜evidence：not-run

### AH-AD-OPENHANDS-03 ACP 权限与凭据
- owner/lane：Lane-CON / AD-OPENHANDS｜depends_on：AH-CRED-01｜blocked_by：AH-B4
- 交付物：关闭 ACP 自动批准，改为显式确认（核验可否按部署关闭）；secret registry 注入用 opaque handle；默认启用 Agent Server auth
- 安全负例：默认无 API auth 的本地 Agent Server 不可采用
- oracle：permission 走结构化确认｜evidence：not-run
