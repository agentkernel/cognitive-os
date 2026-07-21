# macOS Open PoC 执行手册骨架

> 类别：informative runbook ｜ owner：Lane-CON ｜ 状态：全部 `not-run`；evidence `none`
>
> Canonical 定义：[macos-product-design §13](macos-product-design.md#13-open-poc-and-ga-gates)。共享记录模板：[poc-execution-record](../../../../shared/docs/poc-execution-record.md)。

| ID | 验证目标 | 真实环境要求 | 当前状态 | 证据路径 |
|---|---|---|---|---|
| `MAC-POC-01` | Universal 2 Console/daemon/broker/helper 在候选 OS/CPU 安装、签名、公证和启动 | 真实 Mac + 签名/公证流水线；候选 macOS/CPU | not-run | none |
| `MAC-POC-02` | GUI App Sandbox + Hardened Runtime + 最小 entitlements 可通过 XPC 完成必要 IPC | 真实 Sandbox/HR 构建；XPC 旅程 | not-run | none |
| `MAC-POC-03` | renderer compromise 不能访问 daemon/helper/Keychain/R1 capability | 真实分权进程；妥协演练（非 mock） | not-run | none |
| `MAC-POC-04` | node key、claim、revoke/reclaim 和机器转让故障模型 | 真实 Keychain/claim 状态机 | not-run | none |
| `MAC-POC-05` | lock/switch user/sleep/crash 下 broker session 与 signed lease eligibility | 真实多用户锁屏/睡眠/崩溃 | not-run | none |
| `MAC-POC-06` | Keychain locked/cancelled/missing 无 fallback 且正确收窄能力 | 真实 Keychain fixtures | not-run | none |
| `MAC-POC-07` | Touch ID device-key signature、native display、nonce/session/expiry replay negative | 真实生物识别设备 + 负例 | not-run | none |
| `MAC-POC-08` | PKG/MDM、threshold metadata、anti-rollback、drain/repair 失败恢复 | 真实 PKG/MDM 或等价通道 | not-run | none |
| `MAC-POC-09` | local bundle symlink/archive/TOCTOU/budget/signature 负例 | 真实文件系统负例 corpus | not-run | none |
| `MAC-POC-10` | WKWebView floor/kill switch 和纯文本/Markdown 降级 | 真实 WKWebView + floor 元数据 | not-run | none |
| `MAC-POC-11` | 多用户 broker 通知路由、opaque handle 重放和锁屏隐私 | 真实多用户通知路径 | not-run | none |
| `MAC-POC-12` | VoiceOver/FKA/Contrast/Reduce Motion 核心旅程 | 真实辅助技术矩阵 | not-run | none |

**外部阻断**：Apple Developer / 公证账号、真机矩阵、MDM 试验租户——未执行。
