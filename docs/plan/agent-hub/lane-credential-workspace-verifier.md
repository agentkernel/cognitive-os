# 车道计划 — Credential + Workspace + Verifier（CRED）

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：blocked
>
> 目标：opaque credential handle、多账号 profile、worktree/workspace、固定 checks verifier。设计见 [security-and-credentials.md](../../../apps/cognitiveos-console/docs/agent-hub/security/security-and-credentials.md)、[collaboration/lead-workers.md](../../../apps/cognitiveos-console/docs/agent-hub/collaboration/lead-workers.md)。

## 范围与路径

- 允许（激活后）：Credential Broker / Workspace Manager / Verifier 模块。
- 禁止：他人车道代码；导出/复制/回写 provider secret；secret 落 ledger/URL/argv/env 明文/云同步。
- 依赖：HOST、CTR。gate：AH-B1、AH-B4。

## 任务

### AH-CRED-01 opaque credential handle broker
- owner/lane：Lane-CON / CRED｜depends_on：AH-HOST-02｜blocked_by：AH-B4
- 交付物：OS secure store/enterprise broker 引用；短期注入；scope/revoke/rotation；audit 只 metadata
- 失败测试先行：ledger/日志/push 零 secret
- 安全负例：TM-009 不抽取 token/cookie/keychain
- oracle：POC-SEC-003 pass｜evidence：not-run

### AH-CRED-02 多账号 profile 与切换
- owner/lane：Lane-CON / CRED｜depends_on：AH-CRED-01｜blocked_by：AH-B4
- 交付物：每 provider 多账号 opaque profile；切换默认只影响新 session；新 session + 显式 handoff 替代热切换
- 安全负例：session 内热切换（无官方支持）被拒（DEC-021）
- oracle：POC-SEC-004 pass｜evidence：not-run

### AH-CRED-03 worktree/workspace 管理
- owner/lane：Lane-CON / CRED｜depends_on：AH-HOST-01｜blocked_by：—
- 交付物：cwd/repo/branch/worktree/文件范围/端口/dev server/artifact/conflict；coding 默认独立 worktree
- 安全负例：不以 cwd 推导 ownership
- oracle：多 Agent worktree 隔离（配合 MULTI）｜evidence：none

### AH-CRED-04 固定 checks verifier
- owner/lane：Lane-CON / CRED｜depends_on：AH-HOST-04｜blocked_by：—
- 交付物：运行固定 build/test/lint/diff/digest/remote query；记录命令/版本/范围/退出
- 安全负例：checks 记录单独不等于 user acceptance；完成双轴
- oracle：完成判定双轴一致（DEC-026）｜evidence：none
