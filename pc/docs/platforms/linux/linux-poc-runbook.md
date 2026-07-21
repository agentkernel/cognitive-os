# Linux Open PoC 执行手册骨架

> 类别：informative runbook ｜ owner：Lane-CON ｜ 状态：全部 `not-run`；evidence `none`
>
> Canonical 定义：[linux-product-design §13](linux-product-design.md#13-open-poc-and-ga-gates)。矩阵固定 Ubuntu 24.04 LTS x86_64 / stock GNOME / Wayland / 原生 `.deb`。共享记录模板：[poc-execution-record](../../../../shared/docs/poc-execution-record.md)。

| ID | 验证目标 | 真实环境要求 | 当前状态 | 证据路径 |
|---|---|---|---|---|
| `LNX-POC-01` | Ubuntu 24.04 x86_64 stock GNOME/Wayland `.deb` 安装/卸载 | 真实 Ubuntu 24.04 桌面 VM/裸机 | not-run | none |
| `LNX-POC-02` | system service、per-user broker、polkit helper 权限与 D-Bus isolation | 真实 systemd/polkit/D-Bus | not-run | none |
| `LNX-POC-03` | unclaimed quarantine、claim、revoke/reclaim、boot enable | 真实 claim 生命周期 | not-run | none |
| `LNX-POC-04` | `.deb` 不改写 slot payload；A/B stage/atomic switch/rollback | 真实双 slot 布局 | not-run | none |
| `LNX-POC-05` | authority drain/no-unknown/maintenance/polkit 组合 gate | 真实 drain + polkit 组合 | not-run | none |
| `LNX-POC-06` | WebKitGTK distro package floor、advisory/backport、kill switch | 真实 distro WebKitGTK 包级 | not-run | none |
| `LNX-POC-07` | Secret Service missing/locked/cancel 无 fallback | 真实 Secret Service fixtures | not-run | none |
| `LNX-POC-08` | lock/logout/switch user 下 broker 与 signed lease eligibility | 真实会话切换 | not-run | none |
| `LNX-POC-09` | portal bundle path/symlink/archive/TOCTOU/budget 负例 | 真实 xdg-desktop-portal | not-run | none |
| `LNX-POC-10` | 无 tray 的 close/exit/reopen/notification 完整旅程 | 无 SNI host 的真实 GNOME | not-run | none |
| `LNX-POC-11` | opaque notification handle、跨 UID/session、action 缺失降级 | 真实多 UID 通知 | not-run | none |
| `LNX-POC-12` | Orca/keyboard/High Contrast/100-200%/reduced-motion 核心旅程 | 真实 Orca 与显示设置 | not-run | none |

**外部阻断**：签名 `.deb` 发布身份、目标 WebKitGTK 安全 floor 元数据载体——未执行。
