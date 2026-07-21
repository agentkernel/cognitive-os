# iPhone Open PoC 执行手册骨架

> 类别：informative runbook ｜ owner：Lane-CON ｜ 状态：全部 `not-run`；evidence `none`
>
> Canonical 定义：[ios-product-design §18](ios-product-design.md#18-open-poc-与-ga-gates)。共享记录模板：[poc-execution-record](../../../shared/docs/poc-execution-record.md)。

| ID | 验证目标 | 真实环境要求 | 当前状态 | 证据路径 |
|---|---|---|---|---|
| `IOS-POC-01` | iOS 18+ arm64 Public/managed 双 bundle 在最旧 admitted 与当前 build 签名安装升级且数据隔离 | 真实 iPhone；双 channel 签名制品 | not-run | none |
| `IOS-POC-02` | ASWebAuthenticationSession + OIDC/OAuth Code+PKCE 与 callback/state/nonce/cancel 负例 | 真实 IdP + 系统浏览器会话 | not-run | none |
| `IOS-POC-03` | App-container marker、P-256 Keychain 策略、App Attest unsupported/error 不当通过 | 真机 Keychain/Attest fixtures | not-run | none |
| `IOS-POC-04` | CanonicalDisplayEnvelope 严格解码/签名/replay 负例；不宣称 compromised-client trusted display | 真机 R1 显示路径 | not-run | none |
| `IOS-POC-05` | biometric enrollment 改变、lockout、cancel、key missing 无 passcode/file fallback | Touch ID 与 Face ID 真机 | not-run | none |
| `IOS-POC-06` | active/inactive/background/lock/suspend/kill/force-quit 下 lease eligibility | 真机生命周期故障注入 | not-run | none |
| `IOS-POC-07` | APNs sandbox/production、token rotation、delay/drop/duplicate/reorder/410 与 opaque handle | 真实 APNs 环境 | not-run | none |
| `IOS-POC-08` | offline、watch gap、idempotency conflict、response loss、unknown reconcile 无重复 Effect | 真实网络分区 + authority | not-run | none |
| `IOS-POC-09` | marker 排除 backup；restore/reinstall；敏感 draft 零持久化；generation mismatch 删除 | 真实 backup/restore 设备 | not-run | none |
| `IOS-POC-10` | app-switcher privacy cover、recording mask、screenshot-after-event、pasteboard expiry | 真机隐私 UX fixtures | not-run | none |
| `IOS-POC-11` | native Markdown、Universal Link、external browser、file importer malicious corpus；零 native bridge | 真实链接/文件负例 corpus | not-run | none |
| `IOS-POC-12` | Agent lifecycle 只传 refs；iPhone 无 executable/package/archive bytes | 真实网络/存储检查 | not-run | none |
| `IOS-POC-13` | Public App Store / TestFlight / Custom App 分发与账号删除可行性证据 | 真实分发通道 | not-run | none |
| `IOS-POC-14` | signed allowlist、short expiry、anti-rollback、kill switch、update recovery | 真实 signed floor 载体 | not-run | none |
| `IOS-POC-15` | App Attest/jailbreak/system/MDM 仅作风险；unsupported/异常按策略收窄 | 真实 signal fixtures | not-run | none |
| `IOS-POC-16` | VoiceOver/Voice Control/Switch Control/FKA 完成适用 journeys（Touch ID + Face ID） | 真实辅助技术矩阵 | not-run | none |
| `IOS-POC-17` | 屏幕/OS/channel/orientation/Dynamic Type/外接键盘/Reduce Motion GA 等价类 | 多设备等价类真机 | not-run | none |
| `IOS-POC-18` | privacy manifests、required-reason API、Privacy Label、content-free telemetry、diagnostics preview | 真实隐私清单与 egress 检查 | not-run | none |

**外部阻断**：Apple Developer / PLA、真机矩阵、APNs 证书、TestFlight/ABM 租户——未执行。术语：产品设计中 `outcome-unknown` 建议向 PC/共享 `result-unknown` 收敛（见 [design-system README](../../../shared/docs/design-system/README.md)）；执行前不得假装已统一。
