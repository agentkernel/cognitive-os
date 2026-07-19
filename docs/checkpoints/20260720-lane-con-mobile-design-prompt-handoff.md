# 20260720 Lane-CON Mobile Design Prompt Handoff

## 1. 本次会话完成

- 新增 `docs/prompts/console-mobile-platform-product-design.md`，作为可直接粘贴到新 Cursor 会话的 iOS/Android 独立产品设计提示词。
- 提示词要求先完成仓库接入、Console/桌面平台基线阅读、官方资料研究和逐轮产品决策，再经计划批准进入纯 informative 文档编辑。
- 研究范围覆盖移动生命周期、APNs/FCM、secure storage、生物识别与 digest-bound R1、App Store/Google Play 政策、动态代码/acquisition、WebView、deep link、后台限制、更新安全下限、无障碍和受限支持矩阵。
- 预定交付为 iOS/Android 独立产品设计、移动 parity matrix 和决策记录；使用 `CONSOLE-IOS-V1-*` / `CONSOLE-AND-V1-*` 产品 ID，并要求每项记录 contract、implementation、evidence、owner、oracle、blocked_by。
- 更新 `docs/README.md` 提示词索引和 `docs/plan/PROGRESS.md`。本次只新增 prompt 类 informative 文档，未修改 normative 机器合同、Console 产品正文或实现代码。

## 2. 未完成 / 进行中

- iOS/Android 产品研究、决策和四份目标平台文档尚未执行；本次交付只是提示词。
- 移动端角色、支持矩阵、商店渠道、后台 supervision、通知 action、R1、生物识别、离线和 acquisition 均须在执行提示词时由用户逐轮确认。
- Console implementation gate 仍未满足，禁止创建移动客户端脚手架、mock、helper 或实现代码。

## 3. 测试与证据状态

- `git diff --check`：通过。
- `pnpm run check:consistency`：通过；输出为 273 requirements、55 error codes、56 schemas、74 vectors，Markdown links 与 traceability 已验证。
- ReadLints：新增提示词和文档索引无诊断。
- 未执行 iOS/Android 模拟器、真机、商店审核、无障碍或安全测试；移动 evidence 为 `none`。
- 74 个既有 conformance vectors 状态未改变，仍为 `not-run`。

## 4. 未决风险与漂移

- 本次未发现或修改规范漂移；没有新增 `REQ-*`、错误码、schema、transition 或 vector。
- 移动商店动态代码政策、后台执行限制、push 不可靠性、设备完整性和 WebView 安全版本必须在提示词执行日重新核实，不能沿用本文档日期作发布证据。
- iOS/Android 证据不得相互外推；模拟器、单一 OEM 或单一 OS build 证据不得扩张为平台支持声明。

## 5. 下一步入口

- 建议提示词：`docs/prompts/console-mobile-platform-product-design.md`
- 工作范围：Lane-CON informative 文档例外；实现仍无可用分支。
- 第一个动作：在新会话粘贴提示词，先完成只读审查和官方资料研究，再提出第一轮 1–2 个关键决策问题。

## 6. 快照

- PROGRESS 已更新：是。
- Console 状态：tracking-only；移动产品设计未执行；implementation 未启动。
- 本次提交列表：本 handoff、提示词、索引和 PROGRESS 批次哈希见提交后的 `git log`。
