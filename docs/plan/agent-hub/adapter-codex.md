# Adapter 子车道计划 — OpenAI Codex（AD-CODEX）

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：blocked
>
> dossier：[adapters/tier1/codex.md](../../../apps/cognitiveos-console/docs/agent-hub/adapters/tier1/codex.md)。独立 gate：接口/账号/许可/PoC。

## 范围与路径

- 允许（激活后）：Codex Adapter 模块。
- 禁止：他人车道/其他 Adapter 模块；复制/刷新/回写 Codex 凭据；写 native 文件（L6）。
- 依赖：CTR（接口核验）、HOST/PROC/SESS/CRED。gate：AH-B4（Codex 接口）、AH-B2、AH-B5（条款）。

## 任务

### AH-AD-CODEX-01 接口一手核验
- owner/lane：Lane-CON / AD-CODEX｜depends_on：AH-CTR-02｜blocked_by：AH-B4
- 交付物：App Server/CLI/thread resume-fork 官方 URL/version 补齐 provider-interfaces-ledger；native 文件格式/位置/凭据位置核验
- 安全负例：不把竞品观察当官方合同
- oracle：dossier 无 `待核验` 关键项｜evidence：not-run

### AH-AD-CODEX-02 L1/L2 控制映射
- owner/lane：Lane-CON / AD-CODEX｜depends_on：AH-AD-CODEX-01,AH-PROC-02｜blocked_by：AH-B2
- 交付物：App Server 驱动；Host-launched 监管；send/interrupt/cancel/stop 映射
- 安全负例：普通既有 Codex 进程不可 send（归 L7）
- oracle：停止分层状态机一致｜evidence：not-run

### AH-AD-CODEX-03 L3 thread resume/fork 采用
- owner/lane：Lane-CON / AD-CODEX｜depends_on：AH-AD-CODEX-02,AH-SESS-02｜blocked_by：AH-B4
- 交付物：thread resume；fork 显示新旧 ID 映射；写采用需单 writer 证明
- 安全负例：双 writer 拒绝；fork 不显示为“继续原 session”
- oracle：POC-SESS-001/002 pass（Codex）｜evidence：not-run

### AH-AD-CODEX-04 账号与凭据
- owner/lane：Lane-CON / AD-CODEX｜depends_on：AH-CRED-01｜blocked_by：AH-B5
- 交付物：官方登录连接；opaque handle；多账号 profile
- 安全负例：不复制/刷新/回写 auth.json（Paseo 反例）
- oracle：零 secret 落盘｜evidence：not-run
