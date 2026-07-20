# Adapter 子车道计划 — OpenCode（AD-OPENCODE）

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：blocked
>
> dossier：[adapters/tier1/opencode.md](../../../apps/cognitiveos-console/docs/agent-hub/adapters/tier1/opencode.md)。独立 gate：接口/账号/许可/PoC。

## 范围与路径

- 允许（激活后）：OpenCode Adapter 模块。
- 禁止：他人车道/其他 Adapter；写 provider 配置（Vibe 反例）；写 native DB（L6）。
- 依赖：CTR、HOST/PROC/SESS/CRED。gate：AH-B4、AH-B2、AH-B5（许可 SPDX 核验）。

## 任务

### AH-AD-OPENCODE-01 接口一手核验
- owner/lane：Lane-CON / AD-OPENCODE｜depends_on：AH-CTR-02｜blocked_by：AH-B4
- 交付物：server/session API/ACP 官方 URL/version；native 存储格式（疑 SQLite）核验；许可 SPDX
- oracle：dossier 无 `待核验` 关键项｜evidence：not-run

### AH-AD-OPENCODE-02 L1/L2 与连接已运行 server
- owner/lane：Lane-CON / AD-OPENCODE｜depends_on：AH-AD-OPENCODE-01,AH-PROC-02,AH-SESS-01｜blocked_by：AH-B2
- 交付物：Host spawn server（L2）；连接已运行 server 属 L3 条件采用，须单 writer/官方并发语义
- 安全负例：连接已运行 server 无并发保证时不写
- oracle：POC-SESS-002 pass｜evidence：not-run

### AH-AD-OPENCODE-03 L5 SQLite 只读
- owner/lane：Lane-CON / AD-OPENCODE｜depends_on：AH-SESS-04｜blocked_by：AH-B4
- 交付物：只读一致 read transaction；禁 checkpoint/主库单文件复制
- 安全负例：写压力下 snapshot 一致
- oracle：POC-FILE-002 pass｜evidence：not-run

### AH-AD-OPENCODE-04 账号与凭据
- owner/lane：Lane-CON / AD-OPENCODE｜depends_on：AH-CRED-01｜blocked_by：AH-B5
- 交付物：opaque handle；不写 provider 配置
- oracle：零 secret 落盘｜evidence：not-run
