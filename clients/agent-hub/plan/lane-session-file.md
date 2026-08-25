# 车道计划 — Session + File（SESS）

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：blocked
>
> 目标：官方 session 采用（条件写）、native 文件只读观察。设计见 [session-and-file-adoption.md](../docs/architecture/session-and-file-adoption.md)。

## 范围与路径

- 允许（激活后）：Session Adopter / File Observer 模块。
- 禁止：他人车道代码；写 provider auth/entitlement/history/内部 DB（L6 阻断）；非 documented 路径读取。
- 依赖：HOST、CTR（接口）。gate：AH-B4（接口）、AH-B2。

## 任务

### AH-SESS-01 官方 session 候选发现与只读 hydrate
- owner/lane：Lane-CON / SESS｜depends_on：AH-CTR-02｜blocked_by：AH-B4
- 交付物：官方 list/import；候选发现与 adopt 分开；只读 hydrate 标 provider-reported/file-observed
- 安全负例：候选发现不等于 adopt
- oracle：无 adopt 时不建立 writer｜evidence：not-run

### AH-SESS-02 条件写采用（single writer 证明）
- owner/lane：Lane-CON / SESS｜depends_on：AH-SESS-01｜blocked_by：AH-B4
- 交付物：写采用仅在旧 writer inactive 或供应商 exclusive lease/fencing；否则退回只读
- 失败测试先行：无法证明单 writer 时写路径不可达
- 安全负例：TM-006 双 writer 拒绝（隔离沙箱验证）
- oracle：POC-SESS-002 pass｜evidence：not-run

### AH-SESS-03 只读文件观察（JSONL）
- owner/lane：Lane-CON / SESS｜depends_on：AH-HOST-04｜blocked_by：AH-B4
- 交付物：opt-in、documented root、只接受完整换行记录、半写 partial、rotation gap、digest、敏感裁剪
- 安全负例：TM-016 半写不产生完成事实
- oracle：POC-FILE-001 pass｜evidence：not-run

### AH-SESS-04 只读文件观察（SQLite）
- owner/lane：Lane-CON / SESS｜depends_on：AH-HOST-04｜blocked_by：AH-B4
- 交付物：SQLite 3.53.3、READONLY|URI|NOFOLLOW、mode=ro&cache=private、短读事务；禁 immutable(live)/checkpoint/主库单文件复制
- 安全负例：写压力下 snapshot 一致
- oracle：POC-FILE-002 pass｜evidence：not-run

### AH-SESS-05 安全路径解析（symlink/reparse/hardlink/TOCTOU）
- owner/lane：Lane-CON / SESS｜depends_on：AH-SESS-03,AH-SESS-04｜blocked_by：—
- 交付物：Linux openat2 RESOLVE_BENEATH/NO_XDEV；Windows reparse 逐层；macOS NOFOLLOW_ANY/RESOLVE_BENEATH/UNIQUE；handle-bound fstat；拒 nlink>1/设备/FIFO/跨用户/cloud placeholder
- 安全负例：TM-007 逃逸样本全拒
- oracle：POC-FILE-003 pass｜evidence：not-run
