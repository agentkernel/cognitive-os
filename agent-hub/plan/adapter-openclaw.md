# Adapter 子车道计划 — OpenClaw（AD-OPENCLAW）

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：blocked（接口未核验）
>
> dossier：[adapters/tier1/openclaw.md](../docs/adapters/tier1/openclaw.md)。**本轮无官方接口一手事实**；接口核验前整车道 `blocked`，不得以占位内容启动实现。

## 范围与路径

- 允许（激活后）：OpenClaw Adapter 模块。
- 禁止：他人车道/其他 Adapter；臆造接口/session 格式/条款；任意 PID 注入。
- 依赖：CTR。gate：AH-B4（接口，硬前置）、AH-B2、AH-B5。

## 任务

### AH-AD-OPENCLAW-01 接口存在性与一手核验（硬前置）
- owner/lane：Lane-CON / AD-OPENCLAW｜depends_on：AH-CTR-02｜blocked_by：AH-B4
- 交付物：官方仓库/产品页/接口/session 格式/条款/许可（是否 source-available）用 URL/version/commit + 查询日 补齐
- 安全负例：无一手核验前不得声明任何 `目标` 之外能力
- oracle：dossier 全部 `待核验` 清空｜evidence：not-run
- 说明：本任务未完成前，AH-AD-OPENCLAW-02+ 全部 `blocked`。

### AH-AD-OPENCLAW-02 分级决策
- owner/lane：Lane-CON / AD-OPENCLAW｜depends_on：AH-AD-OPENCLAW-01｜blocked_by：AH-B4
- 交付物：依据核验结果定级；无稳定接口则 launch-only/observe-only
- oracle：分级与接口事实一致｜evidence：not-run

### AH-AD-OPENCLAW-03 控制/账号映射（条件解锁）
- owner/lane：Lane-CON / AD-OPENCLAW｜depends_on：AH-AD-OPENCLAW-02,AH-PROC-02,AH-CRED-01｜blocked_by：AH-B4,AH-B2,AH-B5
- 交付物：按核验接口映射层级；opaque handle
- 安全负例：普通既有进程不可 send
- oracle：与 dossier 一致｜evidence：not-run
