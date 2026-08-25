# Adapter 子车道计划 — Anthropic Claude Agent SDK（AD-CLAUDE）

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：blocked
>
> dossier：[adapters/tier1/claude-agent-sdk.md](../docs/adapters/tier1/claude-agent-sdk.md)。独立 gate：接口/账号/许可/PoC。

## 范围与路径

- 允许（激活后）：Claude Adapter 模块。
- 禁止：他人车道/其他 Adapter；复制 `.credentials.json`/Keychain；写 native JSONL（L6）。
- 依赖：CTR、HOST/PROC/SESS/CRED。gate：AH-B4、AH-B2、AH-B5。

## 任务

### AH-AD-CLAUDE-01 接口一手核验
- owner/lane：Lane-CON / AD-CLAUDE｜depends_on：AH-CTR-02｜blocked_by：AH-B4
- 交付物：Agent SDK / Claude Code `--resume` / JSONL schema 官方 URL/version；凭据位置核验
- 安全负例：不把竞品观察当官方合同
- oracle：dossier 无 `待核验` 关键项｜evidence：not-run

### AH-AD-CLAUDE-02 L1/L2 控制映射
- owner/lane：Lane-CON / AD-CLAUDE｜depends_on：AH-AD-CLAUDE-01,AH-PROC-02｜blocked_by：AH-B2
- 交付物：Agent SDK 主路径；Host-launched；send/interrupt/cancel/stop（Ctrl+C→Ctrl+C→SIGINT 升级）
- 安全负例：普通既有 claude 进程不可 send
- oracle：停止分层一致｜evidence：not-run

### AH-AD-CLAUDE-03 L3/L5 session 与 JSONL
- owner/lane：Lane-CON / AD-CLAUDE｜depends_on：AH-AD-CLAUDE-02,AH-SESS-02,AH-SESS-03｜blocked_by：AH-B4
- 交付物：官方 session resume（新进程）；JSONL 只读 L5；fork/rewind 写 native 归 L6 阻断
- 安全负例：TM-006 外部并行 `claude --resume` 同写同 session 拒绝
- oracle：POC-SESS-002、POC-FILE-001 pass｜evidence：not-run

### AH-AD-CLAUDE-04 账号与凭据
- owner/lane：Lane-CON / AD-CLAUDE｜depends_on：AH-CRED-01｜blocked_by：AH-B5
- 交付物：官方登录连接；opaque handle；多账号 profile
- 安全负例：不复制 keychain/credentials
- oracle：零 secret 落盘｜evidence：not-run
