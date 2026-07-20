# 车道计划 — Relay / Pairing（RELAY）

> 类别：plan（informative）｜ 日期：2026-07-20 ｜ owner：Lane-CON ｜ 状态：blocked
>
> 目标：Managed E2EE Relay + LAN/VPN、配对、设备身份、扩权确认、恢复。设计见 [relay-pairing-and-migration.md](../docs/architecture/relay-pairing-and-migration.md)。

## 范围与路径

- 允许（激活后）：Relay 服务端 + 客户端配对/通道模块。
- 禁止：他人车道代码；公网明文控制面；Relay 侧解密；把 Relay 当 authority/ledger。
- 依赖：HOST、DESK（本机能力稳定后）。gate：AH-B1、AH-B3、AH-B5（若复用 Paseo Relay）。

## 任务

### AH-RELAY-01 E2EE 通道
- owner/lane：Lane-CON / RELAY｜depends_on：AH-HOST-02｜blocked_by：AH-B3,AH-B5
- 交付物：双向认证（device key）+ 前向保密会话密钥 + 序号/anti-replay + expiry；Relay 只见密文+最小路由 metadata
- 失败测试先行：重复/乱序密文不重复交付
- 安全负例：TM-011 replay 被拒
- oracle：POC-RELAY-002 pass｜evidence：not-run

### AH-RELAY-02 配对与设备身份
- owner/lane：Lane-CON / RELAY｜depends_on：AH-RELAY-01｜blocked_by：—
- 交付物：QR+短码+两端 matching code+PC-local approve；硬件绑定密钥；device 公钥指纹；每设备 scope/expiry
- 安全负例：TM-010 MITM（matching code 绕过）被拒
- oracle：POC-RELAY-001 pass｜evidence：not-run

### AH-RELAY-03 扩权与本机确认边界
- owner/lane：Lane-CON / RELAY｜depends_on：AH-RELAY-02｜blocked_by：—
- 交付物：手机只能请求；高后果动作 PC-local approve 生成新 generation；push 仅 opaque hint；通知 action 不直接执行
- 安全负例：TM-013 无 approve 的扩权/信号/桌面控制不可达
- oracle：POC-RELAY-004 pass（push 无敏感字段）｜evidence：not-run

### AH-RELAY-04 撤销、轮换与恢复
- owner/lane：Lane-CON / RELAY｜depends_on：AH-RELAY-02｜blocked_by：—
- 交付物：单设备 revoke 即时；key rotation 交接窗口；offline 队列带 expiry；回前台 reauth+resnapshot
- 安全负例：TM-012 丢失设备 revoke 后全被拒
- oracle：POC-RELAY-003 pass｜evidence：not-run
