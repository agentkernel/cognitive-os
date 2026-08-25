# Android Open PoC 执行手册骨架

> 类别：informative runbook ｜ owner：Lane-CON ｜ 状态：全部 `not-run`；evidence `none`
>
> Canonical 定义：[android-product-design §18](android-product-design.md#18-open-poc-与-ga-gates)。ID 使用完整 `CONSOLE-AND-V1-POC-*`。共享记录模板：[poc-execution-record](../../../shared/docs/poc-execution-record.md)。

| ID | 验证目标 | 真实环境要求 | 当前状态 | 证据路径 |
|---|---|---|---|---|
| `CONSOLE-AND-V1-POC-001` | API 33–36、arm64、GMS/Play Protect support detection 与 fail-closed | 真实 Pixel/Samsung + GMS 矩阵 | not-run | none |
| `CONSOLE-AND-V1-POC-002` | personal/work profile 独立 UID/install/storage/FCM/key/binding | 真实 work profile 设备 | not-run | none |
| `CONSOLE-AND-V1-POC-003` | OIDC/OAuth 2.1 Code+PKCE Custom Tab 与 verified App Link | 真实 IdP + App Links | not-run | none |
| `CONSOLE-AND-V1-POC-004` | Keystore hardware-backed key、attestationChallenge、root/CRL、repack/wrong-cert 负例 | 真实 Class 3 / attestation 设备 | not-run | none |
| `CONSOLE-AND-V1-POC-005` | CanonicalDisplayEnvelope 单源显示/完整 digest；Class 3 auth-per-use；不宣称抵御 compromised client | 真机 R1 路径 | not-run | none |
| `CONSOLE-AND-V1-POC-006` | foreground-only lease across background/lock/death/crash/rotation/UI hang | 真机生命周期故障注入 | not-run | none |
| `CONSOLE-AND-V1-POC-007` | FCM delay/drop/duplicate/token rotation/onDeletedMessages/permission/channel | 真实 FCM 环境 | not-run | none |
| `CONSOLE-AND-V1-POC-008` | Settings Force stop / hibernation 下 job/alarm/push 受阻且零续租 | 真实系统 Settings 路径 | not-run | none |
| `CONSOLE-AND-V1-POC-009` | Task Manager Stop、Recents kill、WorkManager/FGS/Doze/OEM 只 resync 不续租 | 真实 OEM 设备子集 | not-run | none |
| `CONSOLE-AND-V1-POC-010` | backup/D2D/restore/reinstall/clear-data 不恢复 key/token/binding | 真实 backup/D2D | not-run | none |
| `CONSOLE-AND-V1-POC-011` | App Links/Intent/PendingIntent/notification handle 跨 app/profile/replay 负例 | 真实跨 profile Intents | not-run | none |
| `CONSOLE-AND-V1-POC-012` | allowlist Markdown、WebView no-use gate、browser/Custom Tabs provider、SAF upload | 真实浏览器 provider 矩阵 | not-run | none |
| `CONSOLE-AND-V1-POC-013` | FLAG_SECURE/HIDE_OVERLAY_WINDOWS/occlusion/Recents/clipboard 降级与文案 | 真机隐私/安全 UX | not-run | none |
| `CONSOLE-AND-V1-POC-014` | 远端 Agent install/upgrade/rollback/uninstall；手机无 executable bytes | 真实网络/存储检查 | not-run | none |
| `CONSOLE-AND-V1-POC-015` | Public/Managed 独立 identity；fresh challenge enrollment 拒绝 stale/repack | 双 channel 真机构建 | not-run | none |
| `CONSOLE-AND-V1-POC-016` | target API 36、Play App Signing、update 后 re-attest/rebind、floor/kill switch | 真实 Play 内测轨道 | not-run | none |
| `CONSOLE-AND-V1-POC-017` | TalkBack/Switch/Voice Access/keyboard、200% font、Display size、contrast、rotation | 真实辅助技术矩阵 | not-run | none |
| `CONSOLE-AND-V1-POC-018` | Pixel/Samsung 每型号、carrier build、patch、GMS、browser provider 月度矩阵 | 多型号真机矩阵 | not-run | none |

**外部阻断**：Play Console / App Signing、work profile 试验设备、FCM 项目、多 OEM 真机——未执行。
