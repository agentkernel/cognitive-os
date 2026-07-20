# 20260720 Lane-CON Mobile Product Design Handoff

## 1. 本次会话完成

- 新增 `docs/platforms/ios-product-design.md`：iPhone-only v1 的角色、支持矩阵、IA、信任边界、生命周期、APNs、digest-bound R1、离线/隐私、远端 Agent 生命周期、分发/更新、16 项威胁、15 条旅程、18 个 PoC、38 项 `CONSOLE-IOS-V1-PRD-*` 和 Apple 官方来源 ledger。
- 新增 `docs/platforms/android-product-design.md`：列名 Android phone v1 的 API/OEM/GMS 矩阵、Compose/Activity 生命周期、FCM、Keystore/BiometricPrompt、profile 隔离、Play 分发、22 项威胁、16 条旅程、18 个 PoC、40 项 `CONSOLE-AND-V1-PRD-*` 和 Android/Google 官方来源 ledger。
- 新增 `docs/platforms/mobile-platform-decision-log.md`：唯一 canonical 定义 `CONSOLE-IOS-V1-DEC-001..016` 与 `CONSOLE-AND-V1-DEC-001..016`；平台正文只保留链接摘要。
- 新增 `docs/platforms/mobile-parity-matrix.md`：逐项映射 Windows 17、macOS 11、Linux 11 个既有决策，并补充能力矩阵与桌面本地机制的移动处置。
- 更新平台、Console、路线图、决策与文档入口；仅把桌面 parity 的向量事实计数由 74 修正为当前 76，不修改任何桌面 ID、anchor 或 parity 语义。
- 产品方向已冻结为：US/SG、`zh-CN/en`、phone-only、BYOD+managed、account-first、前台专属 supervision lease、open-only push、严格 device-key R1、最小离线草稿、authority catalog ref-only acquisition、Public/managed 独立 App identity、选择性 fail-closed floor 和完整移动无障碍 gate。
- 本批属于 Lane-CON informative 文档例外；没有修改 registry、error registry、schema、transition、vector、实现代码或 Profile。

## 2. 未完成 / 进行中

- Console 依赖组 1/2/7、M5 出口和目标平台实现 gate 均未满足；iOS/Android implementation 保持 `not-implemented`。
- 移动 account/session/device enrollment、APNs/FCM routing、opaque handle、supervision lease、digest-bound R1 signature/display、support-floor、Public/managed identity 和 revoke/rebind carrier 仍为 `unregistered / planned / blocked`。
- Apple HIG 的 7 个动态页面在查询日只返回 `An unknown error occurred`；已在 iOS ledger 明确标记，发布前须用交互浏览器人工复核。
- App Store/Google Play 审核、Custom/Managed 分发、真机安全、OEM/carrier、辅助技术与恢复 PoC 均未执行。

## 3. 测试与证据状态

- `pnpm run check:consistency`：首次终检进程以 Windows `3221226505` 瞬时退出且无一致性诊断；立即重跑通过，输出为 273 requirements、55 error codes、56 schemas、76 vectors，Markdown links 与 traceability 已验证。
- `git diff --check`：通过；仅出现现有 Windows working-copy 的 CRLF→LF 提示，无 whitespace error。
- ID/结构检查：iOS/Android canonical DEC 各 16；PRD 各 38/40；journey 分别 15/16；PoC 各 18；桌面决策映射 39 行。
- `code-review` skill 的 docs-only 终审：已执行。三组独立只读审查覆盖 Apple/Google 事实、商店政策、安全、可实现性、无障碍、parity 与追踪；高置信问题已修正，包括 canonical display 不等于 compromised-client trusted display、iOS install generation/backup/account deletion、APNs/FCM scope、Android app-build attestation/floor epoch/双 stop 语义、独立 Runtime projection、辅助技术矩阵、stable anchors 与 vector evidence。
- ReadLints：本任务 Markdown 文件无诊断。
- 未执行 iOS simulator、Android emulator、真机、APNs/FCM、商店、MDM/Android Enterprise、无障碍或安全测试；移动 platform evidence 为 `none`。
- 76 个既有 conformance vectors 状态未改变，仍全部为 `not-run`；iOS/Android Console Profile 均未符合。

## 4. 未决风险与漂移

- 未发现需修改 normative 资产的新漂移；产品文档明确区分通用 REQ 的 partial 覆盖与未登记移动 carrier。
- iOS 证据不得外推 Android；Pixel 不得外推 Samsung；单一型号、carrier build、模拟器或商店审核不得外推整个支持矩阵。
- APNs/FCM receipt、通知点击、生物识别成功、App Attest/Play Integrity、远端 completed 和客户端 cache 均不是 authority 授权、任务完成或 Profile 证据。
- 远端 Agent acquisition 的商店可发布性仍须以真实 submission/review notes 验证；手机端 executable bundle 路径已明确排除。
- 工作区存在本任务之外的 `personal-blog/` 并行改动；本批不得暂存或提交这些路径。

## 5. 下一步入口

- 产品入口：`docs/platforms/README.md`
- 实现入口：仍被 `docs/platforms/README.md#console-实现-gate` 阻断，不得创建移动客户端脚手架或 mock。
- 后续合同工作只能经 Lane-CTR/CFR/KRN/RUN 正式流程登记移动 carrier；Lane-CON 继续保持 tracking-only。
- 第一个产品复核动作：按 iOS/Android §18 的 Open PoC 清单规划真实设备、商店账号、Public/managed identity 与证据路径，不得用模拟器替代真机安全结论。

## 6. 快照

- PROGRESS 已更新：是。
- 移动产品决策：已记录；machine contract：partial/unregistered；implementation：not-implemented；platform evidence：none；Profile：not implemented。
- 本次核心文档提交：`0fc3807`（`docs(console): define iOS and Android mobile products`）。
- 本 handoff 的哈希记录补丁见会话最终报告。
