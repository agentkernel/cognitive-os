# Agent Hub — Relay、配对与模式迁移

> 类别：informative architecture ｜ 日期：2026-07-20 ｜ canonical owner：Lane-CON
>
> 状态：architecture design / implementation not-implemented / evidence none。

## 1. 远程通道形态

已冻结（[CONSOLE-AGENTHUB-V1-DEC-016](../decisions/decision-log.md)）：**Managed E2EE Relay 为主 + LAN/VPN 直连为辅**。

- Relay 只见密文与最小路由 metadata；消息端到端加密，Relay 无法读取或伪造内容。
- LAN/VPN 直连（mDNS/手动地址）作为低延迟备选；两者共享同一 device identity/session 密钥模型。
- 不做：公网明文控制面、Relay 侧解密、把 Relay 当 authority 或 ledger。
- Relay 交付物含 self-host 选项设计（企业未来），但 v1 只设计 managed。

## 2. 配对与设备身份

- 配对：PC 显示 QR + 短码，手机扫码/输码；**两端显示 matching code 比对 + PC-local approve** 完成（防中间人）。
- 每台设备生成硬件绑定密钥对（Secure Enclave / StrongBox / TPM-backed 尽力）；device identity = 公钥指纹。
- 每设备独立 scope（默认手机 = supervise + request；不含 approve-escalation 之外的写）；expiry 与续期显式。
- revoke：PC 端单设备撤销即时生效；key rotation 有交接窗口记录。
- 多 Host：手机可配对多台 PC Host，顶部 Host switcher 显式切换；跨 Host 不共享 lease。

## 3. 会话与消息安全

- 通道协议：双向认证（device key）+ 前向保密会话密钥 + 序号/anti-replay + expiry。
- 命令幂等：request ID 去重；重复/乱序投递不得导致重复信号或重复扩权。
- Push 通知只携 opaque hint（不含 prompt 内容、路径、diff）；通知 action 不直接执行接管/批准/重试/杀进程/写文件，一律回 app 内确认。
- 手机回前台：先 reauth（生物/设备锁按平台策略）→ resnapshot → 才允许发写请求。
- offline 队列：手机端排队的请求带 expiry；过期自动作废并告知，不静默补发。

## 4. 扩权与本机确认边界

手机永远只能**请求**扩权；以下必须 PC-local 确认（沿用 [CONSOLE-AGENTHUB-V1-DEC-014/015](../decisions/decision-log.md)）：

- 第一次附着普通既有进程；扩大文件读取范围；observe→write 升级；
- 发送进程信号（interrupt 及以上）；启用/变更桌面控制；
- 访问新 credential；跨用户或提升权限；新设备配对 approve。

PC-local 批准产生新 ownership generation 与 ledger 记录；批准 UI 显示请求设备、scope、expiry、后果。

## 5. 断连与恢复矩阵

| 场景 | 处置 |
|---|---|
| 手机 offline | 请求入队带 expiry；Host 侧不阻塞本机操作 |
| Relay 不可达 | 自动尝试 LAN/VPN 直连（若已配置）；否则 stale 标注 |
| Host 睡眠/锁屏 | 旧 controller 输入拒绝；恢复后 resnapshot |
| Host 崩溃/重启 | generation 推进；`running` 记录对账（见 [takeover-architecture.md §8](./takeover-architecture.md)） |
| 手机丢失 | PC 端 revoke 设备；所有该设备 pending 请求作废 |
| key rotation | 双密钥交接窗口；旧密钥拒收时间点记录 ledger |
| 重复/乱序消息 | request ID 去重 + 序号窗口；不重放信号 |

## 6. Direct → Governed 迁移（evidence-only）

已冻结（[CONSOLE-AGENTHUB-V1-DEC-018](../decisions/decision-log.md)）：

1. 用户在 Governed 侧发起 import；Governed authority 新建 UserIntentRecord/TaskContract 等对象（全新 ID）。
2. Direct Host ledger、artifact、digest 作为**外部证据**导入：保留原始来源标签（`host-managed`/`file-observed` 等）、时间与 digest。
3. 不追认：历史记录不成为 authority Event，不产生追溯的 Verification/Acceptance/Effect；不重写为 authority audit。
4. 迁移报告列出：导入对象数、证据数、被拒绝项（格式/digest 不符）、语义降级说明。
5. 迁移后 Direct Host 可继续独立运行或退役；退役是显式动作并推进 generation。

## 7. Open PoC（Relay/配对）

全部 `not-run / evidence none`：

- `POC-RELAY-001` 配对 MITM 演练（matching code 绕过尝试）；
- `POC-RELAY-002` 重复/乱序/重放投递下的幂等与 anti-replay；
- `POC-RELAY-003` revoke/rotation 生效延迟实测；
- `POC-RELAY-004` push hint 内容审计（无敏感字段）。
