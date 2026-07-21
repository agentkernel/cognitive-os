# Windows Open PoC / release-gate 执行手册骨架

> 类别：informative runbook ｜ owner：Lane-CON ｜ 状态：全部 `not-run`；evidence `none`
>
> Canonical 定义：[windows-v1-scope §10](windows-v1-scope.md#10-技术候选与-release-gate)。本表是执行清单骨架，不复制整份 scope。共享记录模板：[poc-execution-record](../../../../shared/docs/poc-execution-record.md)。

Windows 无独立 `WIN-POC-*` 编号表；下列 ID 对应 §10 十条 release-gate 验证项（`WIN-RG-01..10`）。技术栈仍为候选，非 ADR。

| ID | 验证目标 | 真实环境要求 | 当前状态 | 证据路径 |
|---|---|---|---|---|
| `WIN-RG-01` | Windows Service + 原生 Host + renderer 权限隔离 | 真实 Windows 安装；Service/Host/renderer 分进程；禁 mock IPC | not-run | none |
| `WIN-RG-02` | bootstrap bundle / endpoint key / SID、TOFU、账号/session/recovery 威胁模型与合同对照 | 真实本机节点 + 威胁模型演练环境 | not-run | none |
| `WIN-RG-03` | supervision lease：多实例、锁屏、用户切换、session revoke、watch stale、UI hang、崩溃、断网、睡眠、关机 | 真实多会话 / 电源与网络故障注入 | not-run | none |
| `WIN-RG-04` | 任意包 acquisition：SSRF/UNC/ambient credential/path/budget、签名缺失、sandbox bypass、rollback 负例 | 真实文件系统与网络边界；负例 corpus | not-run | none |
| `WIN-RG-05` | WebView2 security floor、IPC fuzz、XSS/prompt injection | 真实 WebView2 运行时 + 恶意 corpus | not-run | none |
| `WIN-RG-06` | Narrator、键盘、高对比、200%/400% 缩放、reduced motion | 真实辅助技术与显示设置 | not-run | none |
| `WIN-RG-07` | 10k 事件、长会话、托盘 24h、内存与电量预算 | 真实长跑主机；资源计量 | not-run | none |
| `WIN-RG-08` | 签名安装、UAC/管理员边界、anti-rollback、Service/客户端更新与失败回滚 | 签名制品 + 真实升级路径 | not-run | none |
| `WIN-RG-09` | 中英文伪本地化、长文本、Unicode/Bidi、时区 | 真实区域与字体矩阵 | not-run | none |
| `WIN-RG-10` | 所有 release-blocking 产品依赖具备 machine contract、实现与已执行证据 | 合同/实现/证据三元闭合审查（非 mock） | not-run | none |

**外部阻断（登记，不假装完成）**：签名账号 / 代码签名证书、可复现的 Service 安装权限环境、WebView2 目标 runtime 矩阵——均未具备执行证据。
