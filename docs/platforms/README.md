<a id="cognitiveos-console-桌面平台产品设计"></a>
# CognitiveOS Console 平台产品设计

> 类别：informative product design
>
> 状态：`planned / implementation not started / test not executed / Profile not implemented`
> 查询基准日：2026-07-20

本目录记录 CognitiveOS Console 的 macOS、受限 Linux、iPhone 与 Android phone 独立产品切片。它不新增或修改任何 CognitiveOS `REQ-*`、错误码、schema、transition table、conformance vector 或实现代码，也不表示 Console 实现已经启动。

## 文档入口

- [macOS v1 产品设计](../../clients/pc/docs/platforms/macos/macos-product-design.md)
- [Linux v1 产品设计](../../clients/pc/docs/platforms/linux/linux-product-design.md)
- [iPhone-only v1 产品设计](../../clients/mobile/ios/docs/ios-product-design.md)
- [Android phone v1 产品设计](../../clients/mobile/android/docs/android-product-design.md)
- [桌面平台 parity matrix](../../clients/pc/docs/platforms/desktop-parity-matrix.md)
- [移动平台 parity matrix](../../clients/mobile/shared/docs/mobile-parity-matrix.md)
- [桌面平台产品决策记录](../../clients/pc/docs/platforms/platform-decision-log.md)
- [移动平台产品决策记录](../../clients/mobile/shared/docs/mobile-platform-decision-log.md)
- [Windows v1 产品设计](../../clients/pc/docs/platforms/windows/windows-v1-scope.md)
- [Agent Hub 平台 parity](../../clients/agent-hub/docs/platforms/agent-hub-platform-parity.md)：Direct Takeover 接管能力的跨平台差异（canonical 设计在 [clients/agent-hub/docs/](../../clients/agent-hub/docs/README.md)）

## 共同不可降级边界

- Agent 操作者优先；核心任务是 Conversation/Task、监督纠偏和 Agent 生命周期。
- Console、per-user broker 和 OS helper 都不是 authority、IdP、CognitiveOS node 或最终安全仲裁器。
- Task、Loop、AgentExecution、Runtime、Effect、Verification 分离呈现与控制。
- `CANDIDATE_COMPLETE` 不等于 `COMPLETED`；`OUTCOME_UNKNOWN` 禁止盲重试或换 key。
- 风险下界由 authority 决定；平台首版只执行 R0/R1，R2/R3 不降级。
- acquisition 始终受治理；平台、host、sandbox、adapter 的证据不得跨平台外推。
- specified、implementation available、test executed、Profile implemented 四类状态不得互相替代。

## 激活前文档例外

Lane-CON 在后端 gate 通过前仍是 `tracking-only`。经 2026-07-20 产品决策批准，激活前允许以下窄幅、可审计的 informative 文档工作：

- 平台研究、产品设计、产品要求与决策记录；
- README、roadmap、index、parity matrix；
- 相关治理说明和已登记漂移的事实修正。

例外不允许 Console 组件、脚手架、mock server、平台 helper、安装器或任何实现代码；不允许修改 normative 机器合同；不允许把 planned 设计写成实现、测试或符合性事实。

<a id="implementation-gate"></a>
## Console 实现 gate

任何 Console 实现里程碑仍须同时满足：

1. `docs/plan/DEVELOPMENT-PLAN.md` Console 依赖组 1、2、7 已交付；
2. M5 出口评审通过；
3. 目标平台文档的 Open PoC 与 GA gates 使用真实 API/真实 OS 行为完成并留下可复现实测证据：
   - [macOS Open PoC 与 GA gates](../../clients/pc/docs/platforms/macos/macos-product-design.md#13-open-poc-and-ga-gates)
   - [Linux Open PoC 与 GA gates](../../clients/pc/docs/platforms/linux/linux-product-design.md#13-open-poc-and-ga-gates)
   - [iPhone Open PoC 与 GA gates](../../clients/mobile/ios/docs/ios-product-design.md#18-open-poc-与-ga-gates)
   - [Android phone Open PoC 与 GA gates](../../clients/mobile/android/docs/android-product-design.md#18-open-poc-与-ga-gates)
4. 技术栈 ADR 已批准；
5. 适用 machine contract、implementation 和 executed evidence 分别达到其声明门槛。

在此之前，所有平台 Profile 均保持 `planned`，所有平台专属测试证据均为 `none`。

## 状态真相

当前计数必须从 [PROGRESS](../plan/PROGRESS.md) 读取。查询基准时：

- registry：273 项要求已登记；
- Console implementation：未提供；
- runner：76 个向量均为 `not-run`；
- Console 平台端到端测试：未执行；
- macOS/Linux/iOS/Android Console Profile：均未符合。
